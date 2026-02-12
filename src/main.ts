import { setTimeout as setTimeoutPromise } from 'node:timers/promises';

import { BasicCrawler } from '@crawlee/basic';
import { ImpitHttpClient } from '@crawlee/impit-client';
import { Actor, log } from 'apify';
import * as cheerio from 'cheerio';

import { URLS } from './urls.js';

Actor.on('aborting', async () => {
    await setTimeoutPromise(1000);
    await Actor.exit();
});

type Input = {
    maxProducts: number;
    maxConcurrency: number;
    maxRequestRetries: number;
    proxyConfiguration?: {
        useApifyProxy?: boolean;
        apifyProxyGroups?: string[];
        apifyProxyCountry?: string;
        proxyUrls?: string[];
    };
};

await Actor.init();

const input = await Actor.getInputOrThrow<Input>();

// Named dataset to track which impit settings actually work
const workingHeadersDataset = await Actor.openDataset('WORKING-ARROW-HEADERS');

// ---------------------------------------------------------------------------
// Apify Residential Proxy — sticky session IPs via session ID
// ---------------------------------------------------------------------------
// When we pass a sessionId to proxyConfiguration.newUrl(sessionId), the Apify
// proxy assigns a consistent residential IP for that session.  As long as the
// session is alive and we keep making requests, the same IP is reused.
// Combined with our Impit client cache (connection reuse), this means:
//   session = same IP + same TLS fingerprint + same cookies + h2 conn reuse
const proxyConfiguration = await Actor.createProxyConfiguration({
    groups: input.proxyConfiguration?.apifyProxyGroups ?? ['RESIDENTIAL'],
    countryCode: input.proxyConfiguration?.apifyProxyCountry ?? 'US',
    ...input.proxyConfiguration,
});

if (!proxyConfiguration) {
    throw new Error('Proxy configuration is required — enable Apify residential proxy in input.');
}

// ---------------------------------------------------------------------------
// Direct Rust fingerprint profile selection (no Safari — can't emulate on Linux)
// ---------------------------------------------------------------------------
// Each profile maps to a unique TLS + HTTP/2 + header fingerprint in Rust.
// The seed from the session ID is passed to impit's `randomize()` which toggles
// optional cipher suites, extensions, and signature algorithms to produce
// ~196,000 unique JA4 TLS hashes across all profiles.
const IMPIT_PROFILES: { browser: string; os: string }[] = [
    { browser: 'chrome142', os: 'windows' },
    { browser: 'chrome142', os: 'linux' },
    { browser: 'chrome136', os: 'windows' },
    { browser: 'chrome136', os: 'linux' },
    { browser: 'chrome133', os: 'windows' },
    { browser: 'chrome133', os: 'linux' },
    { browser: 'chrome131', os: 'windows' },
    { browser: 'firefox144', os: 'windows' },
    { browser: 'firefox144', os: 'linux' },
    { browser: 'firefox135', os: 'windows' },
    { browser: 'firefox135', os: 'linux' },
    { browser: 'firefox133', os: 'windows' },
    { browser: 'edge136', os: 'windows' },
    { browser: 'edge136', os: 'linux' },
    { browser: 'edge131', os: 'windows' },
];

/** Deterministic hash of a string to a positive 32-bit integer. */
function hashString(str: string, init = 5381): number {
    let hash = init;
    for (let i = 0; i < str.length; i++) {
        hash = ((hash << 5) + hash + str.charCodeAt(i)) | 0; // djb2
    }
    return Math.abs(hash);
}

/** Pick a profile deterministically from the session ID. */
function getProfileForSession(sessionId: string): { browser: string; os: string } {
    const idx = hashString(sessionId) % IMPIT_PROFILES.length;
    return IMPIT_PROFILES[idx]!;
}

/** Convert session ID to a numeric seed for TLS fingerprint randomization. */
function sessionSeed(sessionId: string): number {
    return hashString(sessionId, 7919);
}

/** Sanitize session ID for Apify proxy (max 50 chars, alphanumeric + ._~). */
function sanitizeSessionId(sessionId: string): string {
    return sessionId.replace(/[^a-zA-Z0-9._~]/g, '_').slice(0, 50);
}

// ---------------------------------------------------------------------------
// Cookie persistence (per-session in-memory cookie jars)
// ---------------------------------------------------------------------------
const impitCookieJars = new Map<string, { setCookie: (cookie: string, url: string) => void; getCookieString: (url: string) => string }>();

function getImpitCookieJar(sessionId: string) {
    let jar = impitCookieJars.get(sessionId);
    if (!jar) {
        const store = new Map<string, Map<string, string>>();
        jar = {
            setCookie(cookie: string, url: string) {
                try {
                    const { hostname } = new URL(url);
                    if (!store.has(hostname)) store.set(hostname, new Map());
                    const eqIdx = cookie.indexOf('=');
                    if (eqIdx > 0) {
                        const name = cookie.substring(0, eqIdx).trim();
                        const value = cookie.substring(eqIdx + 1).split(';')[0]!.trim();
                        store.get(hostname)!.set(name, `${name}=${value}`);
                    }
                } catch { /* ignore malformed cookies */ }
            },
            getCookieString(url: string): string {
                try {
                    const { hostname } = new URL(url);
                    const cookies = store.get(hostname);
                    if (!cookies || cookies.size === 0) return '';
                    return Array.from(cookies.values()).join('; ');
                } catch { return ''; }
            },
        };
        impitCookieJars.set(sessionId, jar);
    }
    return jar;
}

// ---------------------------------------------------------------------------
// Session-level Impit client cache
// ---------------------------------------------------------------------------
// Reuse the same ImpitHttpClient (and underlying reqwest::Client / h2
// connection) for all requests within a session.
//
// Each session gets:
//   - One consistent residential IP (Apify proxy session stickiness)
//   - One unique JA4 TLS hash (from seed-based randomization)
//   - Persistent cookies across requests
//   - Connection reuse (HTTP/2 multiplexing, no repeated TLS handshakes)
//
// The proxy URL is baked into the Impit client at creation time — every
// request on the same client goes through the same proxy session → same IP.
const impitClients = new Map<string, { client: ImpitHttpClient; proxyUrl: string }>();

async function getOrCreateImpitClient(sessionId: string): Promise<{ client: ImpitHttpClient; proxyUrl: string }> {
    const existing = impitClients.get(sessionId);
    if (existing) return existing;

    const profile = getProfileForSession(sessionId);
    const seed = sessionSeed(sessionId);
    const cookieJar = getImpitCookieJar(sessionId);

    // Get a sticky proxy URL for this session — same sessionId = same IP
    const proxyUrl = await proxyConfiguration!.newUrl(sanitizeSessionId(sessionId)) ?? '';
    if (!proxyUrl) {
        throw new Error('Failed to get proxy URL from Apify proxy configuration');
    }

    const client = new ImpitHttpClient({
        browser: profile.browser as any,
        os: profile.os as any,
        cookieJar,
        fingerprintSeed: seed,
    } as any);

    const entry = { client, proxyUrl };
    impitClients.set(sessionId, entry);

    log.debug('Created new Impit client for session', {
        sessionId,
        browser: profile.browser,
        os: profile.os,
        seed,
    });

    return entry;
}

/** Clean up all session state when a session is retired. */
function retireSession(sessionId: string) {
    impitClients.delete(sessionId);
    impitCookieJars.delete(sessionId);
    warmedUpSessions.delete(sessionId);
}

// ---------------------------------------------------------------------------
// Homepage warm-up tracker
// ---------------------------------------------------------------------------
// First request per session visits the homepage to acquire cookies and create
// a natural navigation pattern (homepage → product page).
const warmedUpSessions = new Set<string>();

async function warmUpSession(sessionId: string, httpClient: ImpitHttpClient, proxyUrl: string) {
    if (warmedUpSessions.has(sessionId)) return;
    warmedUpSessions.add(sessionId);

    try {
        log.debug('Warming up session with homepage visit', { sessionId });
        await httpClient.sendRequest({
            url: 'https://www.arrow.com/',
            method: 'GET',
            headers: {},
            proxyUrl,
            responseType: 'text',
        });
        // Small delay between warm-up and actual request (2-4s)
        await setTimeoutPromise(humanDelay(2000, 4000));
    } catch (err) {
        log.debug('Homepage warm-up failed (non-fatal)', {
            sessionId,
            error: err instanceof Error ? err.message : String(err),
        });
    }
}

// ---------------------------------------------------------------------------
// Referer URLs — simulate navigation from the site itself
// ---------------------------------------------------------------------------
const REFERER_URLS = [
    'https://www.arrow.com/',
    'https://www.arrow.com/en/products',
    'https://www.arrow.com/en/products/search',
    'https://www.arrow.com/en/categories',
    'https://www.google.com/',
    'https://www.google.com/search?q=arrow+electronics+components',
];

function getRefererForSession(sessionId: string): string {
    const h = hashString(sessionId, 7919 * 31);
    return REFERER_URLS[h % REFERER_URLS.length]!;
}

// ---------------------------------------------------------------------------
// Timing — more human-like with normal distribution
// ---------------------------------------------------------------------------
function humanDelay(minMs: number, maxMs: number): number {
    const u1 = Math.random();
    const u2 = Math.random();
    const normal = Math.sqrt(-2 * Math.log(u1 || 0.001)) * Math.cos(2 * Math.PI * u2);
    const mean = (minMs + maxMs) / 2;
    const stddev = (maxMs - minMs) / 4;
    return Math.max(minMs, Math.min(maxMs, mean + normal * stddev));
}

// ---------------------------------------------------------------------------
// Crawler
// ---------------------------------------------------------------------------
let requestCount = 0;
let successfulRequestCount = 0;
let blockedRequestCount = 0;

const MAX_CONCURRENCY = Math.min(input.maxConcurrency ?? 20, 20);

const crawler = new BasicCrawler({
    maxConcurrency: MAX_CONCURRENCY,
    maxRequestRetries: 10,

    useSessionPool: true,
    sessionPoolOptions: {
        // Small pool → sessions are reused heavily → residential proxy keeps
        // the same IP sticky → connection/cookie state accumulates naturally.
        // With concurrency 20 and pool 40, each session is likely picked
        // multiple times before it reaches maxUsageCount.
        maxPoolSize: Math.max(MAX_CONCURRENCY * 2, 40),
        sessionOptions: {
            // 15 requests per session: homepage warm-up + 14 product pages,
            // all on the same sticky residential IP with connection reuse.
            maxUsageCount: 15,
            maxErrorScore: 1,
        },
    },

    requestHandler: async ({ request, session }) => {
        try {
            requestCount++;

            // Human-like delay before request (1.5-6s, normally distributed)
            const delayMs = humanDelay(1500, 6000);
            await setTimeoutPromise(delayMs);

            const sessionId = session?.id ?? request.uniqueKey;

            // Get or create session-level Impit client with sticky proxy URL
            const { client: httpClient, proxyUrl } = await getOrCreateImpitClient(sessionId);

            // Warm up new sessions with a homepage visit
            await warmUpSession(sessionId, httpClient, proxyUrl);

            // Only pass Referer as a custom header.
            // ALL other headers (User-Agent, Accept, Accept-Encoding, Accept-Language,
            // Sec-Fetch-*, Sec-Ch-Ua-*, Priority) come from the Rust fingerprint
            // in the correct browser-specific order matching the TLS fingerprint.
            const referer = getRefererForSession(sessionId);
            const profile = getProfileForSession(sessionId);

            log.debug('Making request', {
                url: request.url,
                browser: profile.browser,
                os: profile.os,
                sessionId,
                referer,
            });

            const response = await httpClient.sendRequest({
                url: request.url,
                method: 'GET',
                headers: { 'Referer': referer },
                proxyUrl,
                responseType: 'text',
            });

            const statusCode = response.statusCode;
            const rawBody = response.body;
            let html: string;
            if (typeof rawBody === 'string') {
                html = rawBody;
            } else if (Buffer.isBuffer(rawBody)) {
                html = (rawBody as Buffer).toString('utf-8');
            } else {
                html = String(rawBody);
            }

            // Treat 403/429 as blocks
            if (statusCode === 403 || statusCode === 429) {
                blockedRequestCount++;
                session?.retire();
                retireSession(sessionId);
                throw new Error(`Blocked with status ${statusCode}`);
            }

            log.info('Response status', { statusCode });

            const $ = cheerio.load(html);
            const title = $('title').text();

            if (
                title.toLowerCase().includes('access denied') ||
                title.toLowerCase().includes('cloudflare')
            ) {
                blockedRequestCount++;
                session?.retire();
                retireSession(sessionId);
                throw new Error('Blocked by Cloudflare');
            }

            const errorTitleCount = $('.Errors-title').length;
            const exists = errorTitleCount === 0;

            const name = $('.product-summary-name--Original').text().trim() || '';
            const breadcrumbs = $('.Breadcrumb .Content li')
                .map((_, el) => $(el).text().trim())
                .get();

            const specifications: Record<string, string> = {};
            $('.PartSpecifications-list tr.row').each((_, row) => {
                const key = $(row).find('td').eq(0).text().trim();
                const value = $(row).find('td').eq(1).find('strong').text().trim();
                if (key) {
                    specifications[key] = value || '';
                }
            });

            await Actor.pushData({
                url: request.url,
                exists,
                name,
                breadcrumbs,
                specifications,
            });

            successfulRequestCount++;
            session?.markGood();

            // Track the successful settings for analysis
            await workingHeadersDataset.pushData({
                url: request.url,
                browser: profile.browser,
                os: profile.os,
                sessionId,
                referer,
                statusCode,
                timestamp: new Date().toISOString(),
            });

            log.info(`Success: ${successfulRequestCount}/${input.maxProducts}: ${request.url}`);

            // Human-like post-delay (0.5-2s, normally distributed)
            const postDelayMs = humanDelay(500, 2000);
            await setTimeoutPromise(postDelayMs);
        } catch (error) {
            const sessionId = session?.id ?? request.uniqueKey;
            const errorMessage = error instanceof Error ? error.message : 'Unknown error';
            log.error(`Failed ${request.url}: ${errorMessage}`);
            session?.markBad();
            session?.retire();
            retireSession(sessionId);
            throw error;
        }
    },

    failedRequestHandler: async ({ request }) => {
        log.error(`Request failed: ${request.url}`);
        blockedRequestCount++;
    },
});

await crawler.run(URLS.map((url) => ({ url })).slice(0, input.maxProducts));

log.info('Scraping finished', {
    totalRequests: requestCount,
    successfulRequests: successfulRequestCount,
    blockedOrFailedRequests: blockedRequestCount,
    successRate: requestCount > 0 ? `${((successfulRequestCount / requestCount) * 100).toFixed(2)}%` : '0%',
    blockRate: requestCount > 0 ? `${((blockedRequestCount / requestCount) * 100).toFixed(2)}%` : '0%',
    activeSessions: impitClients.size,
    uniqueProfiles: IMPIT_PROFILES.length,
});

await Actor.exit();
