const errors = require('./errors.js');
const { rethrowNativeError } = errors;
const {
    VanillaFallbackController,
    cloneHeadersWithSetCookie,
} = require('./vanilla-fallback.js');
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
    #vanillaFallbackEnabled;
    #vanillaClient = null;
    #vanillaClientOptions;
    #vanillaFallbackController;

    constructor(options) {
        const normalizedOptions = {
            ...options,
            headers: canonicalizeHeaders(options?.headers),
        };

        // Pass options to native. When cookieJar is provided, pass a truthy value
        // to signal that JS handles cookies (actual cookie ops happen in JS).
        // Redirects are always handled in JS layer.
        super(normalizedOptions);

        this.#cookieJar = options?.cookieJar;
        this.#followRedirects = options?.followRedirects ?? true;
        this.#maxRedirects = options?.maxRedirects ?? 20;
        this.#vanillaFallbackEnabled = Boolean(options?.browser && options?.vanillaFallback);
        this.#vanillaFallbackController = this.#vanillaFallbackEnabled
            ? new VanillaFallbackController(options)
            : null;
        this.#vanillaClientOptions = this.#vanillaFallbackEnabled
            ? {
                ...normalizedOptions,
                browser: undefined,
                vanillaFallback: false,
            }
            : null;
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

    // Taken from https://github.com/nodejs/undici/blob/14e62db0d0cff4bea27357aa5bd14881459b27c7/lib/web/fetch/body.js#L120
    async #generateMultipartFormData(formData) {
        const boundary = super.getMultipartBoundary();
        const prefix = `--${boundary}\r\nContent-Disposition: form-data`;

        /*! formdata-polyfill. MIT License. Jimmy Wärting <https://jimmy.warting.se/opensource> */
        const escape = (str) => str.replace(/\n/g, '%0A').replace(/\r/g, '%0D').replace(/"/g, '%22');
        const normalizeLinefeeds = (value) => value.replace(/\r?\n|\r/g, '\r\n');

        const blobParts = [];
        const rn = new Uint8Array([13, 10]);
        const textEncoder = new TextEncoder();

        for (const [name, value] of formData) {
            if (typeof value === 'string') {
                const chunk = textEncoder.encode(prefix +
                    `; name="${escape(normalizeLinefeeds(name))}"` +
                    `\r\n\r\n${normalizeLinefeeds(value)}\r\n`);
                blobParts.push(chunk);
            } else {
                const chunk = textEncoder.encode(`${prefix}; name="${escape(normalizeLinefeeds(name))}"` +
                    (value.name ? `; filename="${escape(value.name)}"` : '') + '\r\n' +
                    `Content-Type: ${value.type || 'application/octet-stream'}\r\n\r\n`);
                blobParts.push(chunk, value, rn);
            }
        }

        blobParts.push(textEncoder.encode(`--${boundary}--\r\n`));

        const action = async function* () {
            for (const part of blobParts) {
                if (part.stream) {
                    yield* part.stream();
                } else {
                    yield part;
                }
            }
        };

        const parts = [];
        for await (const part of action()) {
            if (part instanceof Uint8Array) {
                parts.push(part);
            } else if (part instanceof Blob) {
                parts.push(new Uint8Array(await part.arrayBuffer()));
            } else {
                throw new TypeError('Unsupported part type');
            }
        }
        const body = new Uint8Array(parts.reduce((acc, part) => acc + part.length, 0));
        let offset = 0;
        for (const part of parts) {
            body.set(part, offset);
            offset += part.length;
        }

        return { body, type: `multipart/form-data; boundary=${boundary}` };
    }

    // Based on https://github.com/nodejs/undici/blob/14e62db0d0cff4bea27357aa5bd14881459b27c7/lib/web/fetch/body.js#L90
    async #serializeBody(body) {
        if (typeof body === 'string') {
            return { body: new TextEncoder().encode(body), type: 'text/plain;charset=UTF-8' };
        } else if (body instanceof URLSearchParams) {
            return { body: new TextEncoder().encode(body.toString()), type: 'application/x-www-form-urlencoded;charset=UTF-8' };
        } else if (body instanceof ArrayBuffer) {
            return { body: new Uint8Array(body.slice()), type: '' };
        } else if (ArrayBuffer.isView(body)) {
            return { body: new Uint8Array(body.buffer.slice(body.byteOffset, body.byteOffset + body.byteLength)), type: '' };
        } else if (body instanceof Blob) {
            return { body: new Uint8Array(await body.arrayBuffer()), type: body.type };
        } else if (body instanceof FormData) {
            return await this.#generateMultipartFormData(body);
        } else if (body instanceof ReadableStream) {
            const reader = body.getReader();
            const chunks = [];
            let done = false;
            while (!done) {
                const { done: streamDone, value } = await reader.read();
                done = streamDone;
                if (value) chunks.push(value);
            }
            const totalLength = chunks.reduce((acc, chunk) => acc + chunk.length, 0);
            const typedArray = new Uint8Array(totalLength);
            let offset = 0;
            for (const chunk of chunks) {
                typedArray.set(chunk, offset);
                offset += chunk.length;
            }
            return { body: typedArray, type: '' };
        }
        return { body, type: '' };
    }

    async #parseFetchOptions(resource, init) {
        let url;
        let options = { ...init };

        if (resource instanceof Request) {
            url = resource.url;
            options = {
                method: resource.method,
                headers: resource.headers,
                body: resource.body,
                ...init,
            };
            if (!('redirect' in options) && resource.redirect !== 'follow') {
                options.redirect = resource.redirect;
            }
        } else if (resource.toString) {
            url = resource.toString();
        } else {
            url = resource;
        }

        options.headers = canonicalizeHeaders(options?.headers);

        if (options?.body) {
            const { body: requestBody, type } = await this.#serializeBody(options.body);
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
            redirect: options.redirect,
        };
    }

    async #fetchWithCompatibilityTransport(rawUrl, options) {
        return this.#vanillaFallbackController.fetchWithCompatibilityTransport(rawUrl, options);
    }

    #getVanillaClient() {
        if (!this.#vanillaFallbackEnabled) {
            return null;
        }

        if (!this.#vanillaClient) {
            this.#vanillaClient = new native.Impit(this.#vanillaClientOptions);
        }

        return this.#vanillaClient;
    }

    async #fetchWithVanillaFallback(url, options) {
        try {
            return await super.fetch(url, options);
        } catch (error) {
            if (!this.#vanillaFallbackEnabled || !this.#vanillaFallbackController?.shouldRetry(error)) {
                throw error;
            }

            const parsedUrl = new URL(url);
            if (this.#vanillaFallbackController.canUseCompatibilityTransport(parsedUrl)) {
                return this.#fetchWithCompatibilityTransport(url, options);
            }

            const vanillaClient = this.#getVanillaClient();
            if (!vanillaClient) {
                throw error;
            }

            return vanillaClient.fetch(url, options);
        }
    }

    async fetch(resource, init) {
        const { url: initialUrl, signal, redirect, ...options } = await this.#parseFetchOptions(resource, init);

        // Check immediately if already aborted (before creating any promises)
        signal?.throwIfAborted();

        let abortHandler;
        const waitForAbort = new Promise((_, reject) => {
            abortHandler = () => reject(signal.reason);
            signal?.addEventListener?.("abort", abortHandler, { once: true });
        });

        try {
            return await this.#fetchWithRedirectHandling(initialUrl, options, signal, waitForAbort, redirect);
        } catch (err) {
            rethrowNativeError(err);
        } finally {
            signal?.removeEventListener?.("abort", abortHandler);
        }
    }

    /**
     * Fetch with manual redirect handling
     * @param {string} initialUrl
     * @param {object} options
     * @param {AbortSignal} signal
     * @param {Promise} waitForAbort
     * @param {'follow' | 'manual' | 'error'} [redirect] Per-request redirect mode override
     */
    async #fetchWithRedirectHandling(initialUrl, options, signal, waitForAbort, redirect) {
        let url = initialUrl;
        let method = options.method || 'GET';
        let redirectCount = 0;
        const maxRedirects = this.#maxRedirects;
        const followRedirects = redirect
            ? redirect === 'follow'
            : this.#followRedirects;
        const errorOnRedirect = redirect === 'error';

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

            const response = this.#fetchWithVanillaFallback(url, {
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

            if (isRedirectStatus(originalResponse.status)) {
                if (errorOnRedirect) {
                    throw new TypeError(`URI requested responds with a redirect, redirect mode is set to 'error': ${url}`);
                }

                if (followRedirects) {
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

        let abortHandler;
        const cleanup = () => {
            signal?.removeEventListener?.("abort", abortHandler);
        };

        if (signal) {
            abortHandler = () => originalResponse.abort();
            signal.addEventListener("abort", abortHandler);
        }

        Object.defineProperty(originalResponse, 'text', {
            value: ResponsePatches.text.bind(originalResponse),
            configurable: true,
        });

        let bodyConsumed = false;

        const nativeBytes = originalResponse.bytes.bind(originalResponse);
        Object.defineProperty(originalResponse, 'bytes', {
            value: async function() {
                bodyConsumed = true;
                try { return await nativeBytes(); } finally { cleanup(); }
            },
            configurable: true,
        });

        const nativeArrayBuffer = originalResponse.arrayBuffer.bind(originalResponse);
        Object.defineProperty(originalResponse, 'arrayBuffer', {
            value: async function() {
                bodyConsumed = true;
                try { return await nativeArrayBuffer(); } finally { cleanup(); }
            },
            configurable: true,
        });

        const nativeJson = originalResponse.json.bind(originalResponse);
        Object.defineProperty(originalResponse, 'json', {
            value: async function() {
                bodyConsumed = true;
                try { return await nativeJson(); } finally { cleanup(); }
            },
            configurable: true,
        });

        Object.defineProperty(originalResponse, 'headers', {
            value: cloneHeadersWithSetCookie(originalResponse.headers)
        });

        Object.defineProperty(originalResponse, 'clone', {
            value: function () {
                if (bodyConsumed) {
                    throw new TypeError('Response body has already been consumed');
                }

                const [stream1, stream2] = this.body.tee();

                // Create a delegate Response from stream1 for the original's body methods
                const delegate = new Response(stream1, {
                    status: this.status,
                    statusText: this.statusText,
                    headers: this.headers,
                });

                // Re-patch original's body getter to return the delegate's stream
                // (the original stream is now locked after tee)
                Object.defineProperty(this, 'body', {
                    get: () => delegate.body,
                    configurable: true,
                });

                // Re-patch original's body methods to read from the delegate
                const decodeBuffer = this.decodeBuffer.bind(this);
                Object.defineProperty(this, 'arrayBuffer', {
                    value: async function () {
                        bodyConsumed = true;
                        try { return await delegate.arrayBuffer(); } finally { cleanup(); }
                    },
                    configurable: true,
                });
                Object.defineProperty(this, 'bytes', {
                    value: async function () {
                        bodyConsumed = true;
                        try { return await delegate.bytes(); } finally { cleanup(); }
                    },
                    configurable: true,
                });
                Object.defineProperty(this, 'json', {
                    value: async function () {
                        bodyConsumed = true;
                        try { return await delegate.json(); } finally { cleanup(); }
                    },
                    configurable: true,
                });
                Object.defineProperty(this, 'text', {
                    value: async function () {
                        bodyConsumed = true;
                        try {
                            const buffer = await delegate.arrayBuffer();
                            return decodeBuffer(Buffer.from(buffer));
                        } finally { cleanup(); }
                    },
                    configurable: true,
                });

                // Create the clone from stream2
                const clone = new Response(stream2, {
                    status: this.status,
                    statusText: this.statusText,
                    headers: this.headers,
                });
                Object.defineProperty(clone, 'url', {
                    value: this.url,
                    enumerable: true,
                });
                Object.defineProperty(clone, 'text', {
                    value: async function () {
                        const buffer = await clone.arrayBuffer();
                        return decodeBuffer(Buffer.from(buffer));
                    },
                });

                return clone;
            },
        });

        return originalResponse;
    }
}

module.exports.Impit = Impit
module.exports.ImpitWrapper = native.ImpitWrapper
module.exports.ImpitResponse = native.ImpitResponse
module.exports.Browser = native.Browser
module.exports.HttpMethod = native.HttpMethod
Object.assign(module.exports, errors)
delete module.exports.rethrowNativeError
