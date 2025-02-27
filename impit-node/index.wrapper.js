const native = require('./index.js');

class ResponsePatches {
    static async text() {
        const buffer = await this.bytes();
        return this.decodeBuffer(buffer);
    }
}

class Impit extends native.Impit {
    async fetch(url, options) {
        const originalResponse = await super.fetch(url, options);

        Object.defineProperty(originalResponse, 'text', {
            value: ResponsePatches.text.bind(originalResponse)
        });

        return originalResponse;
    }
}

module.exports.Impit = Impit
module.exports.ImpitWrapper = native.ImpitWrapper
module.exports.ImpitResponse = native.ImpitResponse
module.exports.Browser = native.Browser
module.exports.HttpMethod = native.HttpMethod

