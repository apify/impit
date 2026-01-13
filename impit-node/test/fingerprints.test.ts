import { test, describe, expect } from 'vitest';

import { Impit, Browser } from '../index.wrapper.js';

describe.each([
    [Browser.Chrome, "t13d1516h2_8daaf6152771_02713d6af862"],
    [Browser.Firefox, "t13d1715h2_5b57614c22b0_5c2c66f702b0"],
])(`Browser emulation [%s]`, (browser, ja4) => {
    const impit = new Impit({ browser });

    test('emulates JA4 fingerprint', async () => {
        const response = await impit.fetch("https://headers.superuser.one/");
        const text = await response.text();

        text.split('\n').forEach(line => {
            if (line.startsWith('cf-ja4 => ')) {
                expect(line.split('=> ')[1]).toBe(ja4);
            }
        });
    });
});
