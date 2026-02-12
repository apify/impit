/**
 * Progressive HTTP client switching manager.
 *
 * Tracks success/block rates for each client type and switches to the next
 * when the block rate exceeds a threshold. Supports three clients:
 *
 * curl-impersonate (BoringSSL) -> impit (rustls) -> got-scraping (OpenSSL)
 *
 * Each uses a fundamentally different TLS stack, producing different JA3/JA4
 * fingerprint families that Cloudflare tracks separately.
 */

import { log } from 'apify';

export type ClientType = 'curl-impersonate' | 'impit' | 'got-scraping';

/** The default rotation order for progressive switching. */
const ALL_CLIENTS: readonly ClientType[] = ['curl-impersonate', 'impit', 'got-scraping'];

interface ClientStats {
    success: number;
    blocked: number;
}

export interface ClientStatsReport {
    success: number;
    blocked: number;
    blockRate: string;
}

export class HttpClientManager {
    private activeClient: ClientType;
    private stats: Record<ClientType, ClientStats>;
    private readonly blockRateThreshold: number;
    private readonly windowSize: number;
    /** Only these clients will be used for rotation. */
    private readonly enabledClients: readonly ClientType[];

    constructor(options?: {
        blockRateThreshold?: number;
        windowSize?: number;
        startWith?: ClientType;
        /** Subset of clients to use. Defaults to all clients. */
        enabledClients?: ClientType[];
    }) {
        this.enabledClients = options?.enabledClients ?? ALL_CLIENTS;
        if (this.enabledClients.length === 0) {
            throw new Error('At least one client must be enabled');
        }
        this.activeClient = options?.startWith ?? this.enabledClients[0]!;
        if (!this.enabledClients.includes(this.activeClient)) {
            throw new Error(`startWith client "${this.activeClient}" is not in the enabled clients list`);
        }
        this.blockRateThreshold = options?.blockRateThreshold ?? 0.5;
        this.windowSize = options?.windowSize ?? 20;
        this.stats = {
            'curl-impersonate': { success: 0, blocked: 0 },
            'impit': { success: 0, blocked: 0 },
            'got-scraping': { success: 0, blocked: 0 },
        };
    }

    /** Returns which client type should be used for the next request. */
    getActiveClient(): ClientType {
        return this.activeClient;
    }

    /** Report a successful request for the given client type. */
    reportSuccess(client: ClientType): void {
        this.stats[client].success++;
        this.maybeSwitch(client);
    }

    /** Report a blocked request for the given client type. */
    reportBlock(client: ClientType): void {
        this.stats[client].blocked++;
        this.maybeSwitch(client);
    }

    /** Get current stats for all clients, including computed block rates. */
    getStats(): Record<ClientType, ClientStatsReport> {
        const result = {} as Record<ClientType, ClientStatsReport>;
        for (const clientType of ALL_CLIENTS) {
            const s = this.stats[clientType];
            const total = s.success + s.blocked;
            result[clientType] = {
                success: s.success,
                blocked: s.blocked,
                blockRate: total > 0 ? `${((s.blocked / total) * 100).toFixed(1)}%` : '0.0%',
            };
        }
        return result;
    }

    /**
     * After each windowSize requests on the active client, check the block rate.
     * If it exceeds the threshold, switch to the next client in round-robin order
     * and reset its window counters.
     */
    private maybeSwitch(client: ClientType): void {
        // Only evaluate for the currently active client
        if (client !== this.activeClient) return;

        const s = this.stats[client];
        const totalInWindow = s.success + s.blocked;

        // Only evaluate after windowSize requests
        if (totalInWindow < this.windowSize) return;

        // Check if total requests is a multiple of windowSize (evaluate periodically)
        if (totalInWindow % this.windowSize !== 0) return;

        const blockRate = s.blocked / totalInWindow;

        if (blockRate > this.blockRateThreshold) {
            const currentIndex = this.enabledClients.indexOf(client);
            const nextIndex = (currentIndex + 1) % this.enabledClients.length;
            const nextClient = this.enabledClients[nextIndex]!;

            log.warning('Switching HTTP client due to high block rate', {
                from: client,
                to: nextClient,
                blockRate: `${(blockRate * 100).toFixed(1)}%`,
                threshold: `${(this.blockRateThreshold * 100).toFixed(1)}%`,
                stats: this.getStats(),
            });

            // Reset the stats for the next client so it gets a fresh evaluation window
            this.stats[nextClient] = { success: 0, blocked: 0 };
            this.activeClient = nextClient;
        }
    }
}
