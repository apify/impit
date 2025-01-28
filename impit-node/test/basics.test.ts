import { test, describe, expect } from 'vitest';
import { Buffer } from 'node:buffer';

import { HttpMethod, Impit, Browser } from '../index.js';

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
            'https://httpbin.org/headers',
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
            'https://httpbin.org/post',
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
                'https://httpbin.org/post',
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
        const response = await impit.fetch('https://httpbin.org/html');
        const text = await response.text();

        t.expect(text).toContain('Herman Melville');
        });

        test('.json() method works', async (t) => {
        const response = await impit.fetch('https://httpbin.org/json');
        const json = await response.json();

        t.expect(json?.slideshow?.author).toBe('Yours Truly');
        });

        test('.bytes() method works', async (t) => {
        const response = await impit.fetch('https://httpbin.org/xml');
        const bytes = await response.bytes();

        // test that first 5 bytes of the response are the `<?xml` XML declaration
        t.expect(bytes.slice(0, 5)).toEqual(Buffer.from([0x3c, 0x3f, 0x78, 0x6d, 0x6c]));
        });
    });

    describe('Redirects', () => {
        test('redirects work by default', async (t) => {
            const response = await impit.fetch(
                'https://httpbin.org/absolute-redirect/1',
            );

            t.expect(response.status).toBe(200);
        });

        test('disabling redirects', async (t) => {
            const impit = new Impit({
                followRedirects: false
            });

            const response = await impit.fetch(
                'https://httpbin.org/absolute-redirect/1',
            );

            t.expect(response.status).toBe(302);
            t.expect(response.headers['location']).toBe('http://httpbin.org/get');
        });

        test('limiting redirects', async (t) => {
            const impit = new Impit({
                followRedirects: true,
                maxRedirects: 1
            });

            const response = impit.fetch(
                'https://httpbin.org/absolute-redirect/2',
            );

            await t.expect(response).rejects.toThrowError('TooManyRedirects');
        });
    })
});
