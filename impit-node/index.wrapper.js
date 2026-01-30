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

class Impit extends native.Impit {
    // prevent GC of the cookie jar wrapper - prevents use-after-free in native code
    #cookieJarWrapper;

    constructor(options) {
        const jsCookieJar = options?.cookieJar;
        const cookieJarWrapper = jsCookieJar ? {
            setCookie: async (args) => jsCookieJar.setCookie?.bind?.(jsCookieJar)(...args),
            getCookieString: async (args) => jsCookieJar.getCookieString?.bind?.(jsCookieJar)(args),
        } : undefined;

        super({
            ...options,
            cookieJar: cookieJarWrapper,
            headers: canonicalizeHeaders(options?.headers),
        });

        this.#cookieJarWrapper = cookieJarWrapper;
    }

    async fetch(resource, init) {
        const { url, signal, ...options } = await parseFetchOptions(resource, init);

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

        const response = super.fetch(url, options);

        const originalResponse = await Promise.race([
            response,
            waitForAbort
        ]);

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

