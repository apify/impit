import { test, describe, expect, beforeAll } from 'vitest';

import { HttpMethod, Impit, Browser } from '../index.js';

function getHttpBinUrl(path: string, https?: boolean): string {
    https ??= true;

    let url: URL;
    if (process.env.APIFY_HTTPBIN_TOKEN) {
        url = new URL(path, 'https://httpbin.apify.actor');
        url.searchParams.set('token', process.env.APIFY_HTTPBIN_TOKEN);
    } else {
        url = new URL(path, 'https://httpbin.org');
    }

    url.protocol = https ? 'https:' : 'http:';
    return url.href;
}

beforeAll(async () => {
    // Warms up the httpbin instance, so that the first tests don't timeout.
    // Has a longer timeout itself (5s vs 30s) to avoid flakiness.
    await fetch(getHttpBinUrl('/get'));
}, 30e3);

describe.each([
    Browser.Chrome,
    Browser.Firefox,
    undefined,
])(`Browser emulation [%s]`, (browser) => {
    const impit = new Impit({ browser });

    describe('Basic requests', () => {
        test.each([
            'http://',
            'https://',
        ])('to an %s domain', async (protocol) => {
            const response = impit.fetch(`${protocol}example.org`);
            await expect(response).resolves.toBeTruthy();
        });

        test('headers work', async (t) => {
            const response = await impit.fetch(
            getHttpBinUrl('/headers'),
            {
                headers: {
                'Impit-Test': 'foo',
                'Cookie': 'test=123; test2=456'
                }
            }
            );
            const json = await response.json();

            t.expect(json.headers?.['Impit-Test']).toBe('foo');
        })
    });

    describe('HTTP methods', () => {
        test.each([
            'GET',
            'POST',
            'PUT',
            'DELETE',
            'PATCH',
            'HEAD',
            'OPTIONS'
        ] as HttpMethod[])('%s', async (method) => {
            const response = impit.fetch(`https://example.org`, {
                method
            });
            await expect(response).resolves.toBeTruthy();
        });
    });

    describe('Request body', () => {
        test('passing string body', async (t) => {
            const response = await impit.fetch(
            getHttpBinUrl('/post'),
            {
                    method: HttpMethod.Post,
                    body: '{"Impit-Test":"foořžš"}',
                    headers: { 'Content-Type': 'application/json' }
            }
            );
            const json = await response.json();

            t.expect(json.data).toEqual('{"Impit-Test":"foořžš"}');
        });

        test('passing binary body', async (t) => {
            const response = await impit.fetch(
                getHttpBinUrl('/post'),
                {
                    method: HttpMethod.Post,
                    body: Uint8Array.from([0x49, 0x6d, 0x70, 0x69, 0x74, 0x2d, 0x54, 0x65, 0x73, 0x74, 0x3a, 0x66, 0x6f, 0x6f, 0xc5, 0x99, 0xc5, 0xbe, 0xc5, 0xa1]),
                    headers: { 'Content-Type': 'application/json' }
                }
            );
            const json = await response.json();

            t.expect(json.data).toEqual('Impit-Test:foořžš');
        });

        test.each(['post', 'put', 'patch'])('using %s method', async (method) => {
            const response = impit.fetch('https://example.org', {
                method: method.toUpperCase() as HttpMethod,
                body: 'foo'
            });
            await expect(response).resolves.toBeTruthy();
        });
    });

    describe('Response parsing', () => {
        test('.text() method works', async (t) => {
        const response = await impit.fetch(getHttpBinUrl('/html'));
        const text = await response.text();

        t.expect(text).toContain('Herman Melville');
        });

        test('.json() method works', async (t) => {
        const response = await impit.fetch(getHttpBinUrl('/json'));
        const json = await response.json();

        t.expect(json?.slideshow?.author).toBe('Yours Truly');
        });

        test('.bytes() method works', async (t) => {
        const response = await impit.fetch(getHttpBinUrl('/xml'));
        const bytes = await response.bytes();

        // test that first 5 bytes of the response are the `<?xml` XML declaration
        t.expect(bytes.slice(0, 5)).toEqual(Uint8Array.from([0x3c, 0x3f, 0x78, 0x6d, 0x6c]));
        });

        test('streaming response body works', async (t) => {
        const response = await impit.fetch(
            'https://apify.github.io/impit/impit/index.html',
        );

        let found = false;

        for await (const chunk of response.body) {
            const text = new TextDecoder('utf-8', { fatal: false }).decode(chunk);

            if (text.includes('impersonation')) {
                found = true;
                break;
            }
        }

        t.expect(found).toBe(true);
        });
    });

    describe('Redirects', () => {
        test('redirects work by default', async (t) => {
            const response = await impit.fetch(
                getHttpBinUrl('/absolute-redirect/1'),
            );

            t.expect(response.status).toBe(200);
            t.expect(response.url).toBe(getHttpBinUrl('/get', true));
        });

        test('disabling redirects', async (t) => {
            const impit = new Impit({
                followRedirects: false
            });

            const response = await impit.fetch(
                getHttpBinUrl('/absolute-redirect/1'),
            );

            t.expect(response.status).toBe(302);
            t.expect(response.headers['location']).toBe(getHttpBinUrl('/get', false));
            t.expect(response.url).toBe(getHttpBinUrl('/absolute-redirect/1', true));
        });

        test('limiting redirects', async (t) => {
            const impit = new Impit({
                followRedirects: true,
                maxRedirects: 1
            });

            const response = impit.fetch(
                getHttpBinUrl('/absolute-redirect/2'),
            );

            await t.expect(response).rejects.toThrowError('TooManyRedirects');
        });
    })
});
