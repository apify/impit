/**
 * A wrapper around @apify-projects/node-curl-impersonate that provides
 * a simple sendRequest interface for use in the BasicCrawler request handler.
 *
 * Supported native presets: chrome-110, chrome-116, firefox-109, firefox-117.
 * Higher Chrome/Firefox versions are mapped to the closest supported preset
 * but use the custom headers from our browser-headers generator.
 *
 * Cookie persistence is supported via -b/-c flags pointing to a cookie file.
 */

import { CurlImpersonate } from '@apify-projects/node-curl-impersonate';
import { log } from 'apify';

import type { CurlPreset } from './browser-headers.js';

export interface CurlClientOptions {
    impersonate: CurlPreset;
    proxyUrl?: string;
    /** Path to a cookie file for persistence (used with -b and -c flags). */
    cookieFile?: string;
}

export interface CurlSendRequestOptions {
    url: string;
    method?: string;
    headers?: Record<string, string>;
}

export interface CurlHttpResponse {
    statusCode: number;
    body: string;
    headers: Record<string, string>;
}

export class CurlImpersonateHttpClient {
    private readonly impersonate: CurlPreset;
    private readonly proxyUrl?: string;
    private readonly cookieFile?: string;

    constructor(options: CurlClientOptions) {
        this.impersonate = options.impersonate;
        this.proxyUrl = options.proxyUrl;
        this.cookieFile = options.cookieFile;
    }

    async sendRequest(options: CurlSendRequestOptions): Promise<CurlHttpResponse> {
        const { url, method = 'GET', headers = {} } = options;

        const flags: string[] = [];
        if (this.proxyUrl) {
            flags.push('-x', this.proxyUrl);
        }

        // Cookie persistence: read from and write to the same file
        if (this.cookieFile) {
            flags.push('-b', this.cookieFile);  // read cookies
            flags.push('-c', this.cookieFile);  // write cookies
        }

        const curl = new CurlImpersonate(url, {
            method,
            headers,
            impersonate: this.impersonate,
            flags,
            followRedirects: true,
            timeout: 30_000,
            verbose: false,
            debugLogger: (msg: string) => log.debug('curl-impersonate', { msg }),
        });

        const result = await curl.makeRequest();

        return {
            statusCode: result.statusCode ?? 0,
            body: result.response,
            headers: (result.responseHeaders ?? {}) as Record<string, string>,
        };
    }
}
