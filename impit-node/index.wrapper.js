const native = require('./index.js');

class ResponsePatches {
    static async text() {
        const buffer = await this.bytes();
        return this.decodeBuffer(buffer);
    }
}

class Impit extends native.Impit {
    async fetch(url, options) {
        if (options.body) {
            if (typeof options.body === 'string') {
                options.body = new TextEncoder().encode(options.body);
            } else if (Buffer.isBuffer(options.body)) {
                options.body = Uint8Array.from(options.body);
            } else if (options.body instanceof Blob) {
                options.body = new Uint8Array(await options.body.arrayBuffer());
            } else if (options.body instanceof DataView) {
                options.body = new Uint8Array(options.body.buffer);
            } else if (options.body instanceof FormData) {
                const encoder = new TextEncoder();
                const formDataString = [...options.body.entries()]
                    .map(([key, value]) => `${encodeURIComponent(key)}=${encodeURIComponent(value)}`)
                    .join('&');
                options.body = encoder.encode(formDataString);
                options.headers = options.headers || [];
                options.headers.push(['Content-Type', 'application/x-www-form-urlencoded']);
            } else if (options.body instanceof URLSearchParams) {
                const encoder = new TextEncoder();
                options.body = encoder.encode(options.body.toString());
                options.headers = options.headers || [];
                options.headers.push(['Content-Type', 'application/x-www-form-urlencoded']);
            }
            options.body = Uint8Array.from(options.body);
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

