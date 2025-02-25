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

native.ImpitResponse = Response;
native.Impit = Impit;

module.exports = native;

