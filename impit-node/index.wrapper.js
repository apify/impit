const exports = require('./index.js');

class ResponsePatches {
    static async text() {
        const buffer = await this.bytes();
        return this.decodeBuffer(buffer);
    }
}

class Impit extends exports.Impit {
    async fetch(url, options) {
        const originalResponse = await super.fetch(url, options);

        Object.defineProperty(originalResponse, 'text', {
            value: ResponsePatches.text.bind(originalResponse)
        });

        return originalResponse;
    }
}

exports.ImpitResponse = Response;
exports.Impit = Impit;

module.exports = exports;

