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

class Impit extends native.Impit {
    constructor(options) {
        const jsCookieJar = options?.cookieJar;
        super({
            ...options,
            cookieJar: jsCookieJar ? {
                setCookie: async (args) => jsCookieJar.setCookie?.bind?.(jsCookieJar)(...args),
                getCookieString: async (args) => jsCookieJar.getCookieString?.bind?.(jsCookieJar)(args),
            } : undefined,
            headers: canonicalizeHeaders(options?.headers),
        });
    }

    async fetch(url, opts) {
        let options = { ...opts };

        options.headers = canonicalizeHeaders(options?.headers);

        if (options?.body) {
            const { body: requestBody, type } = await castToTypedArray(options.body);
            options.body = requestBody;
            if (type) {
                options.headers.push(['Content-Type', type]);
            }
        } else {
            delete options.body;
        }

        const originalResponse = await super.fetch(url, options);

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

