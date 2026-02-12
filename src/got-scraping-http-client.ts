/**
 * A wrapper around got-scraping that provides custom TLS cipher configurations
 * for additional JA3/JA4 fingerprint diversity.
 *
 * got-scraping uses Node.js's native OpenSSL TLS stack, which produces
 * fundamentally different fingerprints from curl-impersonate (BoringSSL)
 * and impit (rustls). This adds a third "family" of TLS fingerprints.
 *
 * got-scraping's tlsHook auto-selects ciphers based on User-Agent unless
 * custom https.ciphers are provided. We define multiple TLS profiles that
 * each produce a different JA3/JA4 hash.
 */

import { log } from 'apify';
import { gotScraping } from 'got-scraping';
import type { CookieJar } from 'tough-cookie';

// ---------------------------------------------------------------------------
// TLS cipher profiles — each produces a different JA3/JA4 fingerprint
// ---------------------------------------------------------------------------
export type GotTlsProfile = 'chrome-auto' | 'firefox-auto' | 'safari-auto' | 'chrome-shuffled' | 'edge-auto';

export interface GotTlsConfig {
    ciphers?: string;
    signatureAlgorithms?: string;
    ecdhCurve?: string;
    minVersion?: 'TLSv1' | 'TLSv1.1' | 'TLSv1.2' | 'TLSv1.3';
    maxVersion?: 'TLSv1.2' | 'TLSv1.3';
}

const CHROME_SIGALGS = [
    'ecdsa_secp256r1_sha256',
    'rsa_pss_rsae_sha256',
    'rsa_pkcs1_sha256',
    'ecdsa_secp384r1_sha384',
    'rsa_pss_rsae_sha384',
    'rsa_pkcs1_sha384',
    'rsa_pss_rsae_sha512',
    'rsa_pkcs1_sha512',
].join(':');

// Chrome "shuffled" — same ciphers but TLS 1.3 suites in different order = different JA3
const CHROME_SHUFFLED_CIPHERS = [
    'TLS_CHACHA20_POLY1305_SHA256',
    'TLS_AES_256_GCM_SHA384',
    'TLS_AES_128_GCM_SHA256',
    'ECDHE-ECDSA-CHACHA20-POLY1305',
    'ECDHE-RSA-CHACHA20-POLY1305',
    'ECDHE-ECDSA-AES256-GCM-SHA384',
    'ECDHE-RSA-AES256-GCM-SHA384',
    'ECDHE-ECDSA-AES128-GCM-SHA256',
    'ECDHE-RSA-AES128-GCM-SHA256',
    'ECDHE-RSA-AES256-SHA',
    'ECDHE-RSA-AES128-SHA',
    'AES256-GCM-SHA384',
    'AES128-GCM-SHA256',
    'AES256-SHA',
    'AES128-SHA',
].join(':');

// Safari v14+ cipher suite
const SAFARI_CIPHERS = [
    'TLS_AES_128_GCM_SHA256',
    'TLS_AES_256_GCM_SHA384',
    'TLS_CHACHA20_POLY1305_SHA256',
    'ECDHE-ECDSA-AES256-GCM-SHA384',
    'ECDHE-ECDSA-AES128-GCM-SHA256',
    'ECDHE-ECDSA-CHACHA20-POLY1305',
    'ECDHE-RSA-AES256-GCM-SHA384',
    'ECDHE-RSA-AES128-GCM-SHA256',
    'ECDHE-RSA-CHACHA20-POLY1305',
    'ECDHE-ECDSA-AES256-SHA384',
    'ECDHE-ECDSA-AES128-SHA256',
    'ECDHE-ECDSA-AES256-SHA',
    'ECDHE-ECDSA-AES128-SHA',
    'ECDHE-RSA-AES256-SHA384',
    'ECDHE-RSA-AES128-SHA256',
    'ECDHE-RSA-AES256-SHA',
    'ECDHE-RSA-AES128-SHA',
    'AES256-GCM-SHA384',
    'AES128-GCM-SHA256',
    'AES256-SHA256',
    'AES128-SHA256',
    'AES256-SHA',
    'AES128-SHA',
    'ECDHE-ECDSA-DES-CBC3-SHA',
    'ECDHE-RSA-DES-CBC3-SHA',
    'DES-CBC3-SHA',
].join(':');

const SAFARI_SIGALGS = [
    'ecdsa_secp256r1_sha256',
    'rsa_pss_rsae_sha256',
    'rsa_pkcs1_sha256',
    'ecdsa_secp384r1_sha384',
    'ecdsa_secp521r1_sha512',
    'rsa_pss_rsae_sha384',
    'rsa_pss_rsae_sha512',
    'rsa_pkcs1_sha384',
    'rsa_pkcs1_sha512',
].join(':');

// Edge has the same TLS stack as Chrome but with slightly different ALPN/headers
// We use a slightly modified cipher order to produce a different fingerprint
const EDGE_CIPHERS = [
    'TLS_AES_256_GCM_SHA384',
    'TLS_AES_128_GCM_SHA256',
    'TLS_CHACHA20_POLY1305_SHA256',
    'ECDHE-ECDSA-AES128-GCM-SHA256',
    'ECDHE-RSA-AES128-GCM-SHA256',
    'ECDHE-ECDSA-AES256-GCM-SHA384',
    'ECDHE-RSA-AES256-GCM-SHA384',
    'ECDHE-ECDSA-CHACHA20-POLY1305',
    'ECDHE-RSA-CHACHA20-POLY1305',
    'ECDHE-RSA-AES128-SHA',
    'ECDHE-RSA-AES256-SHA',
    'AES128-GCM-SHA256',
    'AES256-GCM-SHA384',
    'AES128-SHA',
    'AES256-SHA',
].join(':');

/**
 * Returns TLS config for a given profile. Profiles that return undefined
 * let got-scraping's tlsHook auto-configure based on User-Agent.
 */
function getTlsConfig(profile: GotTlsProfile): GotTlsConfig | undefined {
    switch (profile) {
        case 'chrome-auto':
            // Let got-scraping auto-detect from UA → Chrome ciphers
            return undefined;
        case 'firefox-auto':
            // Let got-scraping auto-detect from UA → Firefox ciphers
            return undefined;
        case 'safari-auto':
            return {
                ciphers: SAFARI_CIPHERS,
                signatureAlgorithms: SAFARI_SIGALGS,
                ecdhCurve: 'X25519:prime256v1:secp384r1:secp521r1',
                minVersion: 'TLSv1.2',
                maxVersion: 'TLSv1.3',
            };
        case 'chrome-shuffled':
            return {
                ciphers: CHROME_SHUFFLED_CIPHERS,
                signatureAlgorithms: CHROME_SIGALGS,
                ecdhCurve: 'X25519:prime256v1:secp384r1',
                minVersion: 'TLSv1',
                maxVersion: 'TLSv1.3',
            };
        case 'edge-auto':
            return {
                ciphers: EDGE_CIPHERS,
                signatureAlgorithms: CHROME_SIGALGS,
                ecdhCurve: 'X25519:prime256v1:secp384r1',
                minVersion: 'TLSv1',
                maxVersion: 'TLSv1.3',
            };
        default:
            return undefined;
    }
}

// ---------------------------------------------------------------------------
// Client interface
// ---------------------------------------------------------------------------
export interface GotScrapingClientOptions {
    tlsProfile: GotTlsProfile;
    proxyUrl?: string;
    cookieJar?: CookieJar;
}

export interface GotScrapingSendRequestOptions {
    url: string;
    method?: string;
    headers?: Record<string, string>;
}

export interface GotScrapingHttpResponse {
    statusCode: number;
    body: string;
    headers: Record<string, string>;
}

export class GotScrapingHttpClient {
    private readonly tlsProfile: GotTlsProfile;
    private readonly proxyUrl?: string;
    private readonly cookieJar?: CookieJar;

    constructor(options: GotScrapingClientOptions) {
        this.tlsProfile = options.tlsProfile;
        this.proxyUrl = options.proxyUrl;
        this.cookieJar = options.cookieJar;
    }

    async sendRequest(options: GotScrapingSendRequestOptions): Promise<GotScrapingHttpResponse> {
        const { url, method = 'GET', headers = {} } = options;

        const tlsConfig = getTlsConfig(this.tlsProfile);

        try {
            const response = await gotScraping({
                url,
                method: method as 'GET',
                headers,
                responseType: 'text',
                followRedirect: true,
                timeout: { request: 30_000 },
                // got-scraping context options
                proxyUrl: this.proxyUrl,
                useHeaderGenerator: false,  // We provide our own headers
                // TLS configuration via got's https option
                ...(tlsConfig ? {
                    https: {
                        ciphers: tlsConfig.ciphers,
                        signatureAlgorithms: tlsConfig.signatureAlgorithms,
                        ecdhCurve: tlsConfig.ecdhCurve,
                        minVersion: tlsConfig.minVersion,
                        maxVersion: tlsConfig.maxVersion,
                    },
                } : {}),
                // Cookie jar for session persistence
                ...(this.cookieJar ? { cookieJar: this.cookieJar } : {}),
            });

            return {
                statusCode: response.statusCode,
                body: typeof response.body === 'string' ? response.body : String(response.body),
                headers: response.headers as Record<string, string>,
            };
        } catch (error) {
            // got throws on non-2xx by default; extract status if available
            const gotError = error as { response?: { statusCode: number; body: string; headers: Record<string, string> } };
            if (gotError.response) {
                return {
                    statusCode: gotError.response.statusCode,
                    body: typeof gotError.response.body === 'string' ? gotError.response.body : String(gotError.response.body ?? ''),
                    headers: (gotError.response.headers ?? {}) as Record<string, string>,
                };
            }
            log.debug('got-scraping request failed', {
                url,
                error: error instanceof Error ? error.message : String(error),
            });
            throw error;
        }
    }
}
