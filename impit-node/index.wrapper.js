const native = require('./index.js');

class ResponsePatches {
    static async text() {
        const buffer = await this.bytes();
        return this.decodeBuffer(buffer);
    }
}

class Impit extends native.Impit {
    async fetch(url, options) {
        if (options.headers instanceof Headers) {
            options.headers = options.headers.entries();
        } else if (!Array.isArray(options.headers)) {
            options.headers = Object.entries(options.headers || {});
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

