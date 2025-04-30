const { castToTypedArray } = require('./request.js');
const native = require('./index.js');

class ResponsePatches {
    static async text() {
        const buffer = await this.bytes();
        return this.decodeBuffer(buffer);
    }
}

class Impit extends native.Impit {
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

