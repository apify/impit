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

class Impit extends native.Impit {
    constructor(options) {
        const jsCookieJar = options?.cookieJar;
        super({
            ...options,
            cookieJar: jsCookieJar ? {
                setCookie: async (args) => jsCookieJar.setCookie?.bind?.(jsCookieJar)(...args),
                getCookieString: async (args) => jsCookieJar.getCookieString?.bind?.(jsCookieJar)(args),
            } : undefined,
        });
    }

    async fetch(url, options) {
        if (options?.headers) {
            if (options.headers instanceof Headers) {
                options.headers = [...options.headers.entries()];
            } else if (!Array.isArray(options.headers)) {
                options.headers = Object.entries(options.headers || {});
            }
        }

        if (options?.body) {
            const { body, type } = await castToTypedArray(options.body);
            options.body = body;
            if (type) {
                options.headers = options.headers || [];
                options.headers.push(['Content-Type', type]);
            }
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

