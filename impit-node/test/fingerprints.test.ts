import { test, describe, expect } from 'vitest';

import { Impit, Browser } from '../index.wrapper.js';

describe.each([
    [Browser.Chrome, "t13d1516h2_8daaf6152771_02713d6af862"],
    [Browser.Chrome125, "t13d1516h2_8daaf6152771_02713d6af862"],
    [Browser.Firefox, "t13d1717h2_5b57614c22b0_3cbfd9057e0d"],
    [Browser.Firefox128, "t13d1715h2_5b57614c22b0_5c2c66f702b0"],
    [Browser.Safari, "t13d2014h2_d5497b48ffb0_7baf387fc6ff"],
])(`Browser emulation [%s]`, (browser, ja4) => {

    test(`[${browser}] emulates JA4 fingerprint`, async () => {
        const impit = new Impit({ browser });
        const response = await impit.fetch("https://headers.superuser.one/");
        const text = await response.text();

        const ja4Line = text.split('\n').find(line => line.startsWith('cf-ja4 => '));
        expect(ja4Line).toBeDefined();
        if (ja4Line) {
            expect(ja4Line.split('=> ')[1]).toBe(ja4);
        }
    });

    test(`[${browser}] without TLS verifier emulates JA4 fingerprint`, async () => {
        const impit = new Impit({ browser, ignoreTlsErrors: false });
        const response = await impit.fetch("https://headers.superuser.one/");
        const text = await response.text();

        const ja4Line = text.split('\n').find(line => line.startsWith('cf-ja4 => '));
        expect(ja4Line).toBeDefined();
        if (ja4Line) {
            expect(ja4Line.split('=> ')[1]).toBe(ja4);
        }
    });
});
