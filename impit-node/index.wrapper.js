const { castToTypedArray } = require('./request.js');
let native = null;
try {
    native = require('./index.js');
} catch (e) {
    throw new Error(`
impit couldn't load native bindings.

This can have several reasons:
- The native bindings are not compiled for your platform (${process.platform}-${process.arch}).
- You skipped installation of optional dependencies (using e.g. \`npm i --omit=optional\`).
        While the main package (impit) still installs, your package manager will skip installing the prebuilt native bindings for your platform.
        If you still want to skip installing other optional dependencies, please install the native bindings for your platform as a direct dependency of your project.
- You are using a non-standard Node.js runtime (e.g. Deno, Bun, Cloudflare workers etc.) that might not support native modules.
${process.platform === 'win32' ? `- On Windows, sometimes the Visual C++ Redistributable for Visual Studio is missing.
        Please install it from https://learn.microsoft.com/en-US/cpp/windows/latest-supported-vc-redist?view=msvc-170` : ''
        }

Run your script with IMPIT_VERBOSE=1 environment variable to get more information about the error.
`, process.env['IMPIT_VERBOSE'] === '1' ? { cause: e } : undefined);
}

class ResponsePatches {
    static async text() {
        const buffer = await this.bytes();
        return this.decodeBuffer(buffer);
    }
}

function canonicalizeHeaders(headers) {
    if (headers instanceof Headers) {
        return [...headers.entries()];
    } else if (Array.isArray(headers)) {
        return headers;
    } else if (typeof headers === 'object') {
        return Object.entries(headers || {});
    }
    return [];
}

async function parseFetchOptions(resource, init) {
    let url;
    let options = { ...init };

    // Handle Request instance
    if (resource instanceof Request) {
        url = resource.url;
        options = {
            method: resource.method,
            headers: resource.headers,
            body: resource.body,
            ...init, // init overrides Request fields
        };
    } else if (resource.toString) {
        url = resource.toString();
    } else {
        url = resource;
    }

    options.headers = canonicalizeHeaders(options?.headers);

    if (options?.body) {
        const { body: requestBody, type } = await castToTypedArray(options.body);
        options.body = requestBody;
        if (type && !options.headers.some(([key]) => key.toLowerCase() === 'content-type')) {
            options.headers.push(['Content-Type', type]);
        }
    } else {
        delete options.body;
    }

    return {
        url: url,
        method: options.method,
        headers: options.headers,
        body: options.body,
        timeout: options.timeout,
        forceHttp3: options.forceHttp3,
        signal: options.signal,
    };
}

function isRedirectStatus(status) {
    return [301, 302, 303, 307, 308].includes(status);
}

function shouldRewriteRedirectToGet(httpStatus, method) {
    // See https://github.com/mozilla-firefox/firefox/blob/911b3eec6c5e58a9a49e23aa105e49aa76e00f9c/netwerk/protocol/http/HttpBaseChannel.cpp#L4801
    if ([301, 302].includes(httpStatus)) {
        return method === 'POST';
    }

    if (httpStatus === 303) return method !== 'HEAD';

    return false;
}

class Impit extends native.Impit {
    #cookieJar;
    #followRedirects;
    #maxRedirects;

    constructor(options) {
        // Pass options to native. When cookieJar is provided, pass a truthy value
        // to signal that JS handles cookies (actual cookie ops happen in JS).
        // Redirects are always handled in JS layer.
        super({
            ...options,
            cookieJar: options?.cookieJar ? {} : undefined,
            headers: canonicalizeHeaders(options?.headers),
        });

        this.#cookieJar = options?.cookieJar;
        this.#followRedirects = options?.followRedirects ?? true;
        this.#maxRedirects = options?.maxRedirects ?? 20;
    }

    /**
     * Get cookies from the cookie jar for a URL
     * @param {string} url
     * @returns {Promise<string>}
     */
    async #getCookies(url) {
        if (!this.#cookieJar) return '';

        try {
            const cookies = this.#cookieJar.getCookieString?.(url);
            // Handle both sync and async getCookieString
            return cookies instanceof Promise ? await cookies : (cookies || '');
        } catch {
            return '';
        }
    }

    /**
     * Set cookies from Set-Cookie headers
     * @param {Headers} headers
     * @param {string} url
     */
    async #setCookies(headers, url) {
        if (!this.#cookieJar) return;

        const setCookieHeaders = headers.getSetCookie?.() || [];

        for (const cookie of setCookieHeaders) {
            try {
                await this.#cookieJar.setCookie?.(cookie, url);
            } catch {
                // Ignore cookie parsing errors
            }
        }
    }

    async fetch(resource, init) {
        const { url: initialUrl, signal, ...options } = await parseFetchOptions(resource, init);

        const waitForAbort = new Promise((_, reject) => {
            signal?.throwIfAborted();
            signal?.addEventListener?.(
                "abort",
                () => {
                    reject(signal.reason);
                },
                { once: true },
            );
        });

        // Redirects are always handled in the JS layer
        if (this.#followRedirects) {
            return this.#fetchWithRedirectHandling(initialUrl, options, signal, waitForAbort);
        }

        // No redirects - make a single request
        return this.#fetchSingle(initialUrl, options, signal, waitForAbort);
    }

    /**
     * Make a single request without following redirects
     * @param {string} url
     * @param {object} options
     * @param {AbortSignal} signal
     * @param {Promise} waitForAbort
     */
    async #fetchSingle(url, options, signal, waitForAbort) {
        // Build headers, checking for user-provided Cookie header
        const headers = [...(options.headers || [])];
        const hasUserCookie = headers.some(([k]) => k.toLowerCase() === 'cookie');

        // Only add cookies from jar if user didn't provide their own Cookie header
        if (this.#cookieJar && !hasUserCookie) {
            const cookieHeader = await this.#getCookies(url);
            if (cookieHeader) {
                headers.push(['Cookie', cookieHeader]);
            }
        }

        const response = super.fetch(url, {
            ...options,
            headers,
        });

        const originalResponse = await Promise.race([
            response,
            waitForAbort
        ]);

        // Store cookies from response if we have a cookie jar
        if (this.#cookieJar) {
            const responseHeaders = new Headers(originalResponse.headers);
            await this.#setCookies(responseHeaders, url);
        }

        return this.#wrapResponse(originalResponse, signal);
    }

    /**
     * Fetch with manual redirect handling
     * @param {string} initialUrl
     * @param {object} options
     * @param {AbortSignal} signal
     * @param {Promise} waitForAbort
     */
    async #fetchWithRedirectHandling(initialUrl, options, signal, waitForAbort) {
        let url = initialUrl;
        let method = options.method || 'GET';
        let redirectCount = 0;
        const maxRedirects = this.#maxRedirects;

        while (true) {
            signal?.throwIfAborted();

            // Build headers, checking for user-provided Cookie header
            const headers = [...(options.headers || [])];
            const hasUserCookie = headers.some(([k]) => k.toLowerCase() === 'cookie');

            // Only add cookies from jar if user didn't provide their own Cookie header
            if (this.#cookieJar && !hasUserCookie) {
                const cookieHeader = await this.#getCookies(url);
                if (cookieHeader) {
                    headers.push(['Cookie', cookieHeader]);
                }
            }

            // Make single-hop request (redirects are always disabled in Rust)
            const response = super.fetch(url, {
                ...options,
                method,
                headers,
                // Don't send body on redirect (for GET conversions)
                body: method === 'GET' ? undefined : options.body,
            });

            const originalResponse = await Promise.race([
                response,
                waitForAbort
            ]);

            // Wrap headers for easier access
            const responseHeaders = new Headers(originalResponse.headers);

            // Store cookies from response if we have a cookie jar
            if (this.#cookieJar) {
                await this.#setCookies(responseHeaders, url);
            }

            // Check for redirect
            if (isRedirectStatus(originalResponse.status)) {
                const location = responseHeaders.get('location');

                if (!location) {
                    // No location header - return the response as-is
                    return this.#wrapResponse(originalResponse, signal);
                }

                redirectCount++;
                if (redirectCount > maxRedirects) {
                    throw new Error(`Maximum redirect limit (${maxRedirects}) exceeded`);
                }

                // Resolve redirect URL and update method
                url = new URL(location, url).toString();
                method = shouldRewriteRedirectToGet(originalResponse.status, method) ? 'GET' : method;

                // Continue to next iteration (follow redirect)
                continue;
            }

            // Not a redirect - return final response
            return this.#wrapResponse(originalResponse, signal);
        }
    }

    /**
     * Wrap a native response with JS enhancements
     * @param {object} originalResponse
     * @param {AbortSignal} signal
     * @returns {object}
     */
    #wrapResponse(originalResponse, signal) {
        signal?.throwIfAborted();
        signal?.addEventListener?.(
            "abort",
            () => {
                originalResponse.abort();
            },
        );

        Object.defineProperty(originalResponse, 'text', {
            value: ResponsePatches.text.bind(originalResponse)
        });

        Object.defineProperty(originalResponse, 'headers', {
            value: new Headers(originalResponse.headers)
        });

        return originalResponse;
    }
}

module.exports.Impit = Impit
module.exports.ImpitWrapper = native.ImpitWrapper
module.exports.ImpitResponse = native.ImpitResponse
module.exports.Browser = native.Browser
module.exports.HttpMethod = native.HttpMethod
