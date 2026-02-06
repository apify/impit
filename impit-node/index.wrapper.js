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

/**
 * ImpitRequest is a Request-like class that allows inspecting the final headers
 * that will be sent by Impit, including browser fingerprint headers.
 *
 * This class implements the fetch Request interface.
 */
class ImpitRequest {
    #impit;
    #url;
    #init;
    #headers;
    #body;

    /**
     * Creates a new ImpitRequest instance.
     *
     * @param {Impit} impit - The Impit instance to use for header generation
     * @param {string | URL | Request | ImpitRequest} input - The URL or Request to use
     * @param {RequestInit} [init] - Optional request options
     */
    constructor(impit, input, init) {
        if (!(impit instanceof Impit)) {
            throw new TypeError('First argument must be an Impit instance');
        }

        this.#impit = impit;

        // Handle ImpitRequest input (clone)
        if (input instanceof ImpitRequest) {
            this.#url = input.url;
            this.#init = {
                method: input.method,
                headers: [...input.#init?.headers || []],
                body: input.#body,
                timeout: input.#init?.timeout,
                forceHttp3: input.#init?.forceHttp3,
                signal: input.#init?.signal,
                ...init, // init overrides cloned fields
            };
            this.#body = init?.body !== undefined ? init.body : input.#body;
        }
        // Handle native Request input
        else if (input instanceof Request) {
            this.#url = input.url;
            this.#init = {
                method: input.method,
                headers: canonicalizeHeaders(input.headers),
                body: input.body,
                ...init, // init overrides Request fields
            };
            this.#body = init?.body !== undefined ? init.body : input.body;
        }
        // Handle URL or string input
        else {
            this.#url = input?.toString ? input.toString() : String(input);
            this.#init = init || {};
            this.#body = init?.body;
        }

        // Ensure headers are canonicalized
        if (this.#init.headers) {
            this.#init.headers = canonicalizeHeaders(this.#init.headers);
        }

        // Lazily compute headers (will be computed on first access)
        this.#headers = null;
    }

    /**
     * The URL of the request.
     * @returns {string}
     */
    get url() {
        return this.#url;
    }

    /**
     * The HTTP method of the request.
     * @returns {string}
     */
    get method() {
        return this.#init?.method || 'GET';
    }

    /**
     * The request body.
     * @returns {ReadableStream | null}
     */
    get body() {
        return this.#body ?? null;
    }

    /**
     * The final merged headers that will be sent with this request.
     * This includes browser fingerprint headers, instance headers, and request-specific headers.
     *
     * @returns {Headers}
     */
    get headers() {
        if (this.#headers === null) {
            this.#computeHeaders();
        }
        return this.#headers;
    }

    /**
     * The abort signal for the request.
     * @returns {AbortSignal | null}
     */
    get signal() {
        return this.#init?.signal ?? null;
    }

    /**
     * Computes and caches the final merged headers.
     * @private
     */
    #computeHeaders() {
        const rawHeaders = this.#impit._getRequestHeaders(this.#url, {
            headers: this.#init?.headers || [],
        });
        this.#headers = new Headers(rawHeaders);
    }

    /**
     * Returns the internal init object for use by Impit.fetch().
     * @returns {object}
     * @internal
     */
    _getInit() {
        return this.#init;
    }

    /**
     * Returns the body.
     * @returns {any}
     * @internal
     */
    _getBody() {
        return this.#body;
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

    // Handle ImpitRequest instance
    if (resource instanceof ImpitRequest) {
        url = resource.url;
        const impitInit = resource._getInit();
        const body = resource._getBody();
        options = {
            method: resource.method,
            headers: impitInit?.headers || [],
            body: body,
            timeout: impitInit?.timeout,
            forceHttp3: impitInit?.forceHttp3,
            signal: impitInit?.signal,
            ...init, // init overrides ImpitRequest fields
        };
    }
    // Handle native Request instance
    else if (resource instanceof Request) {
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

    /**
     * A Request constructor bound to this Impit instance.
     *
     * Use this to create Request objects that can be inspected for their final headers
     * (including browser fingerprint headers) before sending.
     *
     * @example
     * ```js
     * const impit = new Impit({ browser: 'chrome' });
     *
     * // Create a request and inspect headers
     * const request = new impit.Request('https://example.com', {
     *   headers: { 'X-Custom': 'value' }
     * });
     *
     * // Inspect the final headers (includes fingerprint headers)
     * console.log([...request.headers.entries()]);
     *
     * // Optionally modify and send
     * const response = await impit.fetch(request);
     * ```
     *
     * @type {typeof ImpitRequest}
     */
    Request;

    constructor(options) {
        // Pass options to native. When cookieJar is provided, pass a truthy value
        // to signal that JS handles cookies (actual cookie ops happen in JS).
        // Redirects are always handled in JS layer.
        super({
            ...options,
            headers: canonicalizeHeaders(options?.headers),
        });

        this.#cookieJar = options?.cookieJar;
        this.#followRedirects = options?.followRedirects ?? true;
        this.#maxRedirects = options?.maxRedirects ?? 20;

        // Bind Request constructor to this Impit instance
        const impitInstance = this;
        this.Request = class extends ImpitRequest {
            constructor(input, init) {
                super(impitInstance, input, init);
            }
        };
    }

    /**
     * Returns the final merged headers that would be sent for a request to the specified URL.
     * Internal method used by ImpitRequest.
     *
     * @param {string} url - The URL to compute headers for
     * @param {RequestInit} [init] - Optional request options
     * @returns {Array<[string, string]>} The final merged headers as an array of tuples
     * @internal
     */
    _getRequestHeaders(url, init) {
        return super.getRequestHeaders(url, {
            ...init,
            headers: canonicalizeHeaders(init?.headers),
        });
    }

    /**
     * Get cookies from the cookie jar for a URL
     * @param {string} url
     * @returns {Promise<string>}
     */
    async #getCookies(url) {
        try {
            return (await this.#cookieJar?.getCookieString?.(url)) ?? '';
        } catch {
            return '';
        }
    }

    /**
     * Given response headers, set cookies in the cookie jar
     * @param {Headers} headers
     * @param {string} url
     */
    async #setCookies(headers, url) {
        if (!this.#cookieJar) return;

        for (const cookie of (headers.getSetCookie?.() ?? [])) {
            try {
                await this.#cookieJar.setCookie?.(cookie, url);
            } catch {
                // Ignore cookie parsing errors
            }
        }
    }

    async fetch(resource, init) {
        const { url: initialUrl, signal, ...options } = await parseFetchOptions(resource, init);

        // Check immediately if already aborted (before creating any promises)
        signal?.throwIfAborted();

        const waitForAbort = new Promise((_, reject) => {
            signal?.addEventListener?.(
                "abort",
                () => {
                    reject(signal.reason);
                },
                { once: true },
            );
        });

        return this.#fetchWithRedirectHandling(initialUrl, options, signal, waitForAbort);
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

            const headers = [...(options.headers || [])];
            const hasUserCookie = headers.some(([k]) => k.toLowerCase() === 'cookie');

            if (this.#cookieJar && !hasUserCookie) {
                const cookieHeader = await this.#getCookies(url);
                if (cookieHeader) {
                    headers.push(['Cookie', cookieHeader]);
                }
            }

            const response = super.fetch(url, {
                ...options,
                method,
                headers,
                body: method === 'GET' ? undefined : options.body,
            });

            const originalResponse = await Promise.race([
                response,
                waitForAbort
            ]);

            const responseHeaders = new Headers(originalResponse.headers);

            if (this.#cookieJar) {
                await this.#setCookies(responseHeaders, url);
            }

            if (this.#followRedirects && isRedirectStatus(originalResponse.status)) {
                const location = responseHeaders.get('location');

                if (!location) {
                    return this.#wrapResponse(originalResponse, signal);
                }

                redirectCount++;
                if (redirectCount > maxRedirects) {
                    throw new Error(`Maximum redirect limit (${maxRedirects}) exceeded`);
                }

                url = new URL(location, url).toString();
                method = shouldRewriteRedirectToGet(originalResponse.status, method) ? 'GET' : method;

                continue;
            }

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
module.exports.ImpitRequest = ImpitRequest
module.exports.ImpitWrapper = native.ImpitWrapper
module.exports.ImpitResponse = native.ImpitResponse
module.exports.Browser = native.Browser
module.exports.HttpMethod = native.HttpMethod
