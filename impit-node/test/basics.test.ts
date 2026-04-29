import http from 'http';
import https from 'https';
import { test, describe, expect, beforeAll, afterAll } from 'vitest';

import { HttpMethod, Impit, Browser } from '../index.wrapper.js';
import type { AddressInfo, Server } from 'net';
import { routes, runProxyServer, runServer } from './mock.server.js';
import {
    createTransportFailure,
    createWrappedResponse,
    LOCALHOST_TLS_CERT,
    LOCALHOST_TLS_KEY,
} from './vanilla-fallback.helpers.js';

import { CookieJar } from 'tough-cookie';
import { runSocksServer } from 'socks-server-lib';
import { Server as ProxyServer } from 'proxy-chain';

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

let localServer: Server | null = null;
async function getServer() {
    localServer ??= await runServer(3001);
    return localServer;
}

let proxyServer: ProxyServer | null = null;
async function getProxyServer() {
    proxyServer ??= await runProxyServer(3002);
    return proxyServer;
}

let socksServer: Server | null = null;
let socksConnectionCount = 0;
beforeAll(async () => {
    // Warms up the httpbin instance, so that the first tests don't timeout.
    // Has a longer timeout itself (5s vs 30s) to avoid flakiness.
    await fetch(getHttpBinUrl('/get'));
    // Start the local server
    await getServer();
    // Start the proxy server
    await getProxyServer()

    socksServer = await runSocksServer({ host: 'localhost', port: 7625, onData: () => { socksConnectionCount++; }});
}, 30e3);

afterAll(async () => {
    await Promise.all([
        new Promise<void>(async (res) => {
            const server = await getServer();
            server?.close(() => res())
        }),
        new Promise<void>(async (res) => {
            const server = await getProxyServer();
            server?.close(true, () => res())
        }),
        Promise.race([
            new Promise<void>(res => {
                socksServer?.on('close', () => res());
                socksServer?.close();
            }),
            new Promise<void>(res => {
                setTimeout(() => {
                    res();
                }, 5000);
            })
        ]),
    ]);

    expect(socksConnectionCount).toBe(6);
});

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
            const response = impit.fetch(`${protocol}apify.com`);
            await expect(response).resolves.toBeTruthy();
        });

        test('to a BoringSSL-based server', async () => {
            const response = impit.fetch('https://www.google.com');
            await expect(response).resolves.toBeTruthy();
        });

        test.each(
            [
            ['object', {
                'Impit-Test': 'foo',
                'Cookie': 'test=123; test2=456'
            }],
            ['array', [
                ['Impit-Test', 'foo'],
                ['Cookie', 'test=123; test2=456']
            ]],
            ['Headers', new Headers([
                ['Impit-Test', 'foo'],
                ['Cookie', 'test=123; test2=456']
            ])],
            ]
        )('headers (%s) work', async (_, value) => {
            const response = await impit.fetch(
            getHttpBinUrl('/headers'),
            {
                headers: value
            }
            );
            const json = await response.json();
            const headers = response.headers;

            // request headers
            expect(json.headers?.['Impit-Test']).toBe('foo');

            // response headers
            expect(headers.get('content-type')).toEqual('application/json');
        })

        test('multiple same-named response headers work', async (t) => {
            const impit = new Impit({ browser, followRedirects: false })

            const { headers } = await impit.fetch(
                getHttpBinUrl('/cookies/set?a=1&b=2&c=3'),
            );

            t.expect(headers.getSetCookie())
                .toEqual([
                    'a=1; Path=/',
                    'b=2; Path=/',
                    'c=3; Path=/'
                ]);
        });

        test.each([
            { scheme: 'socks4', url: 'socks4://localhost:7625' },
            { scheme: 'socks5', url: 'socks5://localhost:7625' },
            { scheme: 'http', url: 'http://localhost:3002' },
        ])('supports %s proxy', async ({ scheme, url }) => {
            const impit = new Impit({
                browser,
                proxyUrl: url,
            });

            const response = await impit.fetch(
                getHttpBinUrl('/get'),
            );

            expect(response.status).toBe(200);
            const json = await response.json();
            expect(json).toHaveProperty('url');
            expect(json).toHaveProperty('headers');
            expect(json).toHaveProperty('origin');
        });

        test('receives content-length header', async () => {
            const impit = new Impit({
                browser,
            });

            const response = await impit.fetch(
                'https://crawlee.dev'
            );

            expect(response.status).toBe(200);
            expect(response.headers.get('content-length')).toBeTruthy();
            expect(response.headers.get('content-encoding')).toBeTruthy();
        });

        test('proxy with incomplete authentication works', async () => {
            let proxyHit = false;
            const proxy = http.createServer((req, res) => {
                proxyHit = true;
                res.writeHead(200);
                res.end('OK');
            });
            await new Promise((r) => proxy.listen(0, '127.0.0.1', r));
            const proxyPort = (proxy.address() as AddressInfo)?.port;

            const impit = new Impit({
                browser,
                proxyUrl: `http://user:@127.0.0.1:${proxyPort}`,
            });

            const resp = await impit.fetch('http://example.com/'); // the URL doesn't matter, request should go through the proxy
            expect(await resp.text()).toBe('OK');
            expect(proxyHit).toBe(true);

            proxy.close();
        });

        test('impit accepts custom cookie jars', async (t) => {
            const cookieJar = new CookieJar();
            cookieJar.setCookieSync('preset-cookie=123; Path=/', getHttpBinUrl('/cookies'));

            const impit = new Impit({
                cookieJar,
                browser,
            })

            const response1 = await impit.fetch(
                getHttpBinUrl('/cookies'),
            ).then(x => x.json());

            t.expect(response1.cookies).toEqual({
                'preset-cookie': '123'
            });

            await impit.fetch(
                getHttpBinUrl('/cookies/set?set-by-server=321'),
            );

            const response2 = await impit.fetch(
                getHttpBinUrl('/cookies'),
            ).then(x => x.json());

            t.expect(response2.cookies).toEqual({
                'preset-cookie': '123',
                'set-by-server': '321'
            });

            t.expect(cookieJar.serializeSync()?.cookies).toHaveLength(2);
        })

        test('client-scoped headers override impersonation headers (case-insensitive)', async (t) => {
            if (!browser) return t.skip();

            const impit = new Impit({
                browser,
                headers: { 'user-agent': 'custom-client-ua' },
            });

            const response = await impit.fetch(getHttpBinUrl('/headers'));
            const json = await response.json();

            t.expect(json.headers?.['User-Agent']).toBe('custom-client-ua');
        });

        test('request headers override client and impersonation headers (case-insensitive)', async (t) => {
            if (!browser) return t.skip();

            const impit = new Impit({
                browser,
                headers: { 'User-Agent': 'client-level-ua' },
            });

            const response = await impit.fetch(getHttpBinUrl('/headers'), {
                headers: { 'user-agent': 'request-level-ua' },
            });
            const json = await response.json();

            t.expect(json.headers?.['User-Agent']).toBe('request-level-ua');
        });

        test('impersonation headers are included when no overrides', async (t) => {
            if (!browser) return t.skip();

            const impit = new Impit({ browser });

            const response = await impit.fetch(getHttpBinUrl('/headers'));
            const json = await response.json();

            if (browser === Browser.Chrome) {
                t.expect(json.headers?.['User-Agent']).toContain('Chrome');
            } else if (browser === Browser.Firefox) {
                t.expect(json.headers?.['User-Agent']).toContain('Firefox');
            }
        });

        test('overwriting impersonated headers works', async (t) => {
            const response = await impit.fetch(
            getHttpBinUrl('/headers'),
            {
                headers: {
                    'User-Agent': 'this is impit!',
                }
            }
            );
            const json = await response.json();

            t.expect(json.headers?.['User-Agent']).toBe('this is impit!');
        })

        test('removing impersonated headers with empty string works', async (t) => {
            const response = await impit.fetch(
                getHttpBinUrl('/headers'),
                {
                    headers: {
                        'Sec-Fetch-User': '',
                    }
                }
            );
            const json = await response.json();

            t.expect(json.headers?.['Sec-Fetch-User']).toBeUndefined();
        });

        test('client-scoped headers work', async (t) => {
            const headers = new Headers();
            headers.set('User-Agent', 'client-scoped user agent');

            const impit = new Impit({
                browser,
                headers
            });

            const response = await impit.fetch(getHttpBinUrl('/headers'));
            const json = await response.json();

            t.expect(json.headers?.['User-Agent']).toBe('client-scoped user agent');

            const response2 = await impit.fetch(getHttpBinUrl('/headers'), { headers: { 'User-Agent': 'overwritten user agent' } });
            const json2 = await response2.json();

            t.expect(json2.headers?.['User-Agent']).toBe('overwritten user agent');
        })

        test('http3 works', async (t) => {
            const impit = new Impit({
                http3: true,
                browser,
            })

            const response = await impit.fetch(
                'https://curl.se',
                {
                    forceHttp3: true,
                }
            );

            const text = await response.text();

            t.expect(text).toContain('curl');
        });

        test.each([
            ['client'],
            ['request'],
        ])('%s timeout works', async (mode) => {
            const impit = new Impit({
                browser,
                timeout: mode === 'client' ? 1 : undefined,
            })

            const responsePromise = impit.fetch(
                getHttpBinUrl('/delay/3'),
                {
                    timeout: mode === 'request' ? 1 : undefined,
                }
            );

            await expect(responsePromise).rejects.toThrow(/timeout/);
        });
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
            const response = impit.fetch(getHttpBinUrl('/anything'), {
                method
            });
            await expect(response).resolves.toBeTruthy();
        });
    });

    describe('Advanced options', () => {
        test.each([
            ['127.0.0.1', '::ffff:127.0.0.1'],
            ['::1', '::1']
        ])('localAddress switches %s / %s', async (localAddress, remoteAddress) => {
            const impit = new Impit({
                browser,
                localAddress
            });

            const response = await impit.fetch(new URL('/socket', "http://localhost:3001").href);
            const json = await response.json();

            expect(json.ip).toBe(remoteAddress);
        });
    });

    describe('Parameter types', () => {
        test.each([
            ['string', getHttpBinUrl('/get')],
            ['URL', new URL('/get', getHttpBinUrl('/', false))],
            ['Request', new Request(getHttpBinUrl('/get'))],
        ])('passing %s as input', async (type, resource) => {
            const response = impit.fetch(resource as any);
            await expect(response).resolves.toBeTruthy();
        });

        test.each([
            ['string', getHttpBinUrl('/get')],
            ['URL', new URL(getHttpBinUrl('/get'))],
            ['Request', new Request(getHttpBinUrl('/get'))],
        ])('passing %s as input with init', async (type, resource) => {
            const response = impit.fetch(resource as any, { headers: { 'Impit-Test': 'foo' } });

            const res = await response;
            const json = await res.json();

            expect(json.headers?.['Impit-Test']).toBe('foo');
        });

        test('passing Request with body and init overrides body', async () => {
            const request = new Request(getHttpBinUrl('/post'), {
                method: 'post',
                body: 'this body will be overridden',
            });

            const response = await impit.fetch(request, {
                body: 'this is the real body',
            });
            const json = await response.json();

            expect(json.data).toBe('this is the real body');
        });
    });

    describe('Request body', () => {
        const STRING_PAYLOAD = '{"Impit-Test":"foořžš"}';
        test.each([
            ['string', STRING_PAYLOAD],
            ['ArrayBuffer', new TextEncoder().encode(STRING_PAYLOAD).buffer],
            ['TypedArray', new TextEncoder().encode(STRING_PAYLOAD)],
            ['DataView', new DataView(new TextEncoder().encode(STRING_PAYLOAD).buffer)],
            ['Blob', new Blob([STRING_PAYLOAD], { type: 'application/json' })],
            ['File', new File([STRING_PAYLOAD], 'test.txt', { type: 'application/json' })],
            ['URLSearchParams', new URLSearchParams(JSON.parse(STRING_PAYLOAD))],
            ['FormData', (() => { const form = new FormData(); form.append('Impit-Test', 'foořžš'); return form; })()],
            ['ReadableStream', new ReadableStream({ start(controller) { controller.enqueue(new TextEncoder().encode(STRING_PAYLOAD)); controller.close(); } })],
            ['undefined', undefined],
            ['null', null],
        ])('passing %s body', async (type, body) => {
            const response = await impit.fetch(getHttpBinUrl('/post'), { method: HttpMethod.Post, body });
            const json = await response.json();

            if (type === 'URLSearchParams' || type === 'FormData') {
                expect(json.form).toEqual(JSON.parse(STRING_PAYLOAD));
            } else if (type === 'undefined' || type === 'null') {
                expect(json.data).toEqual('');
            } else {
                expect(json.data).toEqual(STRING_PAYLOAD);
            }
        });

        test.each(['post', 'put', 'patch'])('using %s method', async (method) => {
            const response = impit.fetch(getHttpBinUrl('/anything'), {
                method: method.toUpperCase() as HttpMethod,
                body: 'foo'
            });
            await expect(response).resolves.toBeTruthy();
        });
    });

    describe('Response parsing', () => {
        test('.text() method works', async (t) => {
            const response = await impit.fetch(getHttpBinUrl('/html'));
            const text: string = await response.text();

            t.expect(text).toContain('Herman Melville');
        });

        test('.text() decodes using Content-Type header charset param', async (t) => {
            const response = await impit.fetch(new URL(routes.charset.path, "http://127.0.0.1:3001").href);
            const text: string = await response.text();

            t.expect(text).toContain(routes.charset.bodyString);
        });

        test('.text() decodes using <meta charset> prescan', async (t) => {
            const response = await impit.fetch(new URL(routes.charsetMetaCharset.path, "http://127.0.0.1:3001").href);
            const text: string = await response.text();

            t.expect(text).toContain(routes.charsetMetaCharset.bodyString);
        });

        test('.text() decodes using <meta http-equiv> prescan', async (t) => {
            const response = await impit.fetch(new URL(routes.charsetMetaHttpEquiv.path, "http://127.0.0.1:3001").href);
            const text: string = await response.text();

            t.expect(text).toContain(routes.charsetMetaHttpEquiv.bodyString);
        });

        test('non-ASCII header values are decoded as ISO-8859-1', async (t) => {
            const response = await impit.fetch(new URL(routes.nonAsciiHeader.path, "http://127.0.0.1:3001").href);
            t.expect(response.headers.get('x-non-ascii')).toBe(routes.nonAsciiHeader.headerValue);
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

        test('.arrayBuffer() method works', async (t) => {
            const response = await impit.fetch(getHttpBinUrl('/xml'));
            const bytes = await response.arrayBuffer();

            // test that first 5 bytes of the response are the `<?xml` XML declaration
            t.expect(new Uint8Array(bytes.slice(0, 5))).toEqual(Uint8Array.from([0x3c, 0x3f, 0x78, 0x6d, 0x6c]));
        });

        test('streaming response body works', async (t) => {
        const response = await impit.fetch(
            'https://apify.github.io/impit/js',
        );

        let found = false;

        for await (const chunk of response.body) {
            const text = new TextDecoder('utf-8', { fatal: false }).decode(chunk);

            if (text.includes('impit')) {
                found = true;
                break;
            }
        }

        t.expect(found).toBe(true);
        });
    });

    describe('AbortSignal', () => {
        test('aborts immediately if signal is already aborted', async () => {
            const controller = new AbortController();
            controller.abort();

            const start = Date.now();
            const responsePromise = impit.fetch(getHttpBinUrl('/delay/3'), { signal: controller.signal });

            await expect(responsePromise).rejects.toThrow(/abort|aborted|canceled/i);
            expect(Date.now() - start).toBeLessThan(200);
        });

        test('aborts an in-flight request', async () => {
            const server = await getServer();
            const tAbort = Date.now();
            const result = impit.fetch(
                `http://127.0.0.1:${(server?.address() as AddressInfo).port}/delay/3000`,
                { signal: AbortSignal.timeout(500) }
            ).then(x => x.text())

            await expect(result).rejects.toThrow(/abort|aborted|canceled/i);
            const tDone = Date.now();
            const abortEffectDelay = tDone - tAbort;
            expect(abortEffectDelay).toBeGreaterThanOrEqual(500);
            expect(abortEffectDelay).toBeLessThan(750);
        });

        test('finishes before the abort signal fires', async () => {
            const server = await getServer();
            const tAbort = Date.now();
            const result = impit.fetch(
                `http://127.0.0.1:${(server?.address() as AddressInfo).port}/delay/10`,
                { signal: AbortSignal.timeout(500) }
            ).then(x => x.text())

            await expect(result).resolves.toBeDefined();
            const tDone = Date.now();
            const abortEffectDelay = tDone - tAbort;
            expect(abortEffectDelay).toBeGreaterThanOrEqual(10);
            expect(abortEffectDelay).toBeLessThan(400);
        });

        test('does not accumulate listeners on a reused AbortSignal', async () => {
            const server = await getServer();
            const controller = new AbortController();
            const signal = controller.signal;

            for (let i = 0; i < 10; i++) {
                const response = await impit.fetch(
                    `http://127.0.0.1:${(server?.address() as AddressInfo).port}/delay/10`,
                    { signal }
                );
                await response.text();
            }

            // After all requests complete, the signal should have no lingering abort listeners.
            const { getEventListeners } = await import('node:events');
            expect(getEventListeners(signal, 'abort').length).toBe(0);
        });

        test('cleans up listeners when body read throws', async () => {
            const server = await getServer();
            const controller = new AbortController();
            const signal = controller.signal;
            const { getEventListeners } = await import('node:events');

            const response = await impit.fetch(
                `http://127.0.0.1:${(server?.address() as AddressInfo).port}/delay/3000`,
                { signal }
            );
            // Access body to initialize the abort channel, then abort before reading.
            void response.body;
            controller.abort();
            await expect(response.text()).rejects.toThrow(/abort|aborted|canceled/i);

            expect(getEventListeners(signal, 'abort').length).toBe(0);
        });
    });

    describe('Response.clone()', () => {
        test('clone returns a standard Response', async () => {
            const response = await impit.fetch(getHttpBinUrl('/get'));
            const clone = response.clone();

            expect(clone).toBeInstanceOf(Response);
            expect(clone.status).toBe(200);
            expect(clone.ok).toBe(true);
        });

        test('clone preserves url', async () => {
            const response = await impit.fetch(getHttpBinUrl('/get'));
            const clone = response.clone();

            expect(clone.url).toBe(response.url);
        });

        test('clone preserves headers', async () => {
            const response = await impit.fetch(getHttpBinUrl('/get'));
            const clone = response.clone();

            expect(clone.headers.get('content-type')).toBe(
                response.headers.get('content-type'),
            );
        });

        test('both original and clone bodies are independently readable', async () => {
            const response = await impit.fetch(getHttpBinUrl('/get'));
            const clone = response.clone();

            const cloneData = await clone.json();
            const originalData = await response.json();

            expect(cloneData).toEqual(originalData);
        });

        test('text() works on both original and clone', async () => {
            const response = await impit.fetch(getHttpBinUrl('/get'));
            const clone = response.clone();

            const cloneText = await clone.text();
            const originalText = await response.text();

            expect(cloneText.length).toBeGreaterThan(0);
            expect(cloneText).toBe(originalText);
        });

        test('multiple clones produce independent readable bodies', async () => {
            const response = await impit.fetch(getHttpBinUrl('/get'));
            const clone1 = response.clone();
            const clone2 = response.clone();

            const [original, first, second] = await Promise.all([
                response.json(),
                clone1.json(),
                clone2.json(),
            ]);

            expect(original).toEqual(first);
            expect(original).toEqual(second);
        });

        test('clone() after body consumed throws TypeError', async () => {
            const response = await impit.fetch(getHttpBinUrl('/get'));
            await response.text();

            expect(() => response.clone()).toThrow(TypeError);
            expect(() => response.clone()).toThrow(/body has already been consumed/);
        });

        test('response.body is streamable after clone', async () => {
            const response = await impit.fetch(getHttpBinUrl('/get'));
            response.clone();

            const reader = response.body.getReader();
            const chunks: Uint8Array[] = [];
            while (true) {
                const { done, value } = await reader.read();
                if (done) break;
                chunks.push(value);
            }

            expect(chunks.length).toBeGreaterThan(0);
        });

        test('arrayBuffer() works on both original and clone', async () => {
            const response = await impit.fetch(getHttpBinUrl('/get'));
            const clone = response.clone();

            const cloneBuf = await clone.arrayBuffer();
            const originalBuf = await response.arrayBuffer();

            expect(cloneBuf.byteLength).toBeGreaterThan(0);
            expect(cloneBuf.byteLength).toBe(originalBuf.byteLength);
        });

        test('reading original first, then clone', async () => {
            const response = await impit.fetch(getHttpBinUrl('/get'));
            const clone = response.clone();

            const originalData = await response.json();
            const cloneData = await clone.json();

            expect(originalData).toEqual(cloneData);
        });

        test('clone preserves non-200 status', async () => {
            const response = await impit.fetch(getHttpBinUrl('/status/404'));
            const clone = response.clone();

            expect(clone.status).toBe(404);
            expect(clone.ok).toBe(false);
        });
    });

    describe('Redirects', () => {
        test('follows redirects by default', async () => {
            const response = await impit.fetch('http://localhost:3001/redirect/1');

            expect(response.status).toBe(200);
            expect(response.url).toBe('http://localhost:3001/get');
        });

        test('instance-level followRedirects: false disables redirects', async () => {
            const noRedirect = new Impit({ browser, followRedirects: false });
            const response = await noRedirect.fetch('http://localhost:3001/redirect/1');

            expect(response.status).toBe(302);
            expect(response.headers.get('location')).toBe('/get');
        });

        test('instance-level maxRedirects limits redirect chain', async () => {
            const limited = new Impit({ browser, maxRedirects: 1 });

            await expect(
                limited.fetch('http://localhost:3001/redirect/2'),
            ).rejects.toThrow('Maximum redirect limit (1) exceeded');
        });

        test('per-request redirect: "manual" returns 3xx response', async () => {
            const response = await impit.fetch('http://localhost:3001/redirect/1', {
                redirect: 'manual',
            });

            expect(response.status).toBe(302);
            expect(response.headers.get('location')).toBe('/get');
        });

        test('per-request redirect: "error" throws TypeError on redirect', async () => {
            await expect(
                impit.fetch('http://localhost:3001/redirect/1', { redirect: 'error' }),
            ).rejects.toThrow(TypeError);
        });

        test('per-request redirect: "follow" follows redirects', async () => {
            const noRedirect = new Impit({ browser, followRedirects: false });
            const response = await noRedirect.fetch('http://localhost:3001/redirect/1', {
                redirect: 'follow',
            });

            expect(response.status).toBe(200);
        });

        test('per-request redirect: "manual" overrides instance followRedirects: true', async () => {
            const response = await impit.fetch('http://localhost:3001/redirect/1', {
                redirect: 'manual',
            });

            expect(response.status).toBe(302);
        });

        test('redirect via Request object', async () => {
            const request = new Request('http://localhost:3001/redirect/1', {
                redirect: 'manual',
            });
            const response = await impit.fetch(request);

            expect(response.status).toBe(302);
        });

        test('bare Request does not override instance followRedirects: false', async () => {
            const noRedirect = new Impit({ browser, followRedirects: false });
            const request = new Request('http://localhost:3001/redirect/1');
            const response = await noRedirect.fetch(request);

            expect(response.status).toBe(302);
        });

        test('init overrides Request.redirect', async () => {
            const request = new Request('http://localhost:3001/redirect/1', {
                redirect: 'manual',
            });
            const response = await impit.fetch(request, { redirect: 'follow' });

            expect(response.status).toBe(200);
        });

        test('redirect: "error" does not throw on non-redirect response', async () => {
            const response = await impit.fetch('http://localhost:3001/get', {
                redirect: 'error',
            });

            expect(response.status).toBe(200);
        });

        test('redirect: "follow" still respects instance maxRedirects', async () => {
            const limited = new Impit({ browser, followRedirects: false, maxRedirects: 1 });

            await expect(
                limited.fetch('http://localhost:3001/redirect/2', { redirect: 'follow' }),
            ).rejects.toThrow('Maximum redirect limit (1) exceeded');
        });

        test('redirect: "manual" with 301 status code', async () => {
            const response = await impit.fetch(
                'http://localhost:3001/redirect-to?url=/get&status_code=301',
                { redirect: 'manual' },
            );

            expect(response.status).toBe(301);
            expect(response.headers.get('location')).toBe('/get');
        });

        test('redirect: "manual" with 307 status code', async () => {
            const response = await impit.fetch(
                'http://localhost:3001/redirect-to?url=/get&status_code=307',
                { redirect: 'manual' },
            );

            expect(response.status).toBe(307);
            expect(response.headers.get('location')).toBe('/get');
        });
    })
});

describe('vanillaFallback', () => {
    test('retries with the compatibility transport and preserves per-request options', async () => {
        const nativePrototype = Object.getPrototypeOf(Impit.prototype);
        const originalFetch = nativePrototype.fetch;
        const cookieJar = {
            async getCookieString() {
                return 'session=abc';
            },
            async setCookie() {},
        };
        const callers: Array<{
            isWrapper: boolean;
            headers: [string, string][];
            timeout: number | undefined;
        }> = [];

        nativePrototype.fetch = async function (url: string, init?: any) {
            callers.push({
                isWrapper: this instanceof Impit,
                headers: [...(init?.headers ?? [])],
                timeout: init?.timeout,
            });

            if (this instanceof Impit) {
                throw createTransportFailure();
            }

            return originalFetch.call(this, url, init);
        };

        try {
            const impit = new Impit({
                browser: Browser.Chrome,
                vanillaFallback: true,
                cookieJar,
            });
            const response = await impit.fetch(getHttpBinUrl('/headers'), {
                headers: { 'Impit-Test': 'foo' },
                timeout: 1234,
            });
            const json = await response.json();

            expect(response.status).toBe(200);
            expect(callers).toHaveLength(1);
            expect(callers[0]?.isWrapper).toBe(true);
            expect(callers[0]?.headers).toContainEqual(['Impit-Test', 'foo']);
            expect(callers[0]?.headers).toContainEqual(['Cookie', 'session=abc']);
            expect(callers[0]?.timeout).toBe(1234);
            expect(json.headers?.['Impit-Test']).toBe('foo');
            expect(json.headers?.Cookie).toBe('session=abc');
            expect(json.headers?.['Accept-Encoding']).toBe('gzip, deflate, br');
        } finally {
            nativePrototype.fetch = originalFetch;
        }
    });

    test('uses the compatibility transport through an HTTP proxy', async () => {
        const nativePrototype = Object.getPrototypeOf(Impit.prototype);
        const originalFetch = nativePrototype.fetch;
        let wrapperCalls = 0;

        nativePrototype.fetch = async function () {
            wrapperCalls += 1;
            throw createTransportFailure();
        };

        try {
            const impit = new Impit({
                browser: Browser.Chrome,
                vanillaFallback: true,
                proxyUrl: 'http://localhost:3002',
            });
            const response = await impit.fetch(getHttpBinUrl('/get'));
            const json = await response.json();

            expect(response.status).toBe(200);
            expect(wrapperCalls).toBe(1);
            expect(json).toHaveProperty('url');
            expect(json).toHaveProperty('headers');
        } finally {
            nativePrototype.fetch = originalFetch;
        }
    });

    test('aborts compatibility fallback requests and closes the socket', async () => {
        const nativePrototype = Object.getPrototypeOf(Impit.prototype);
        const originalFetch = nativePrototype.fetch;
        let clientSocketClosed = false;
        let resolveConnectionSeen!: () => void;
        let resolveSocketClosed!: () => void;
        const connectionSeen = new Promise<void>((resolve) => {
            resolveConnectionSeen = resolve;
        });
        const socketClosed = new Promise<void>((resolve) => {
            resolveSocketClosed = resolve;
        });
        const hangingServer = https.createServer({
            key: LOCALHOST_TLS_KEY,
            cert: LOCALHOST_TLS_CERT,
        }, (_req, res) => {
            setTimeout(() => {
                res.end('late');
            }, 1000);
        });
        hangingServer.on('secureConnection', (socket) => {
            resolveConnectionSeen();
            socket.on('close', () => {
                clientSocketClosed = true;
                resolveSocketClosed();
            });
        });

        await new Promise<void>((resolve) => hangingServer.listen(0, '127.0.0.1', () => resolve()));
        const port = (hangingServer.address() as AddressInfo).port;

        nativePrototype.fetch = async function () {
            throw createTransportFailure();
        };

        try {
            const impit = new Impit({
                browser: Browser.Chrome,
                vanillaFallback: true,
                ignoreTlsErrors: true,
            });

            await expect(
                impit.fetch(`https://127.0.0.1:${port}/`, {
                    signal: AbortSignal.timeout(200),
                }),
            ).rejects.toBeTruthy();

            await Promise.race([
                connectionSeen,
                new Promise((_, reject) => setTimeout(() => reject(new Error('socket was never opened')), 1000)),
            ]);
            await Promise.race([
                socketClosed,
                new Promise((_, reject) => setTimeout(() => reject(new Error('socket was not closed after abort')), 1000)),
            ]);
            expect(clientSocketClosed).toBe(true);
        } finally {
            nativePrototype.fetch = originalFetch;
            hangingServer.closeAllConnections?.();
            await new Promise<void>((resolve, reject) =>
                hangingServer.close((error) => (error ? reject(error) : resolve())),
            );
        }
    }, 10_000);

    test('delegates to the native vanilla client when compatibility transport cannot preserve proxy semantics', async () => {
        const nativePrototype = Object.getPrototypeOf(Impit.prototype);
        const originalFetch = nativePrototype.fetch;
        const callers: boolean[] = [];

        nativePrototype.fetch = async function (url: string) {
            callers.push(this instanceof Impit);

            if (this instanceof Impit) {
                throw createTransportFailure();
            }

            return createWrappedResponse('vanilla ok', url);
        };

        try {
            const impit = new Impit({
                browser: Browser.Chrome,
                vanillaFallback: true,
                proxyUrl: 'socks5://localhost:7625',
            });

            const response = await impit.fetch('https://example.com/');

            expect(await response.text()).toBe('vanilla ok');
            expect(callers).toEqual([true, false]);
        } finally {
            nativePrototype.fetch = originalFetch;
        }
    });

    test('does not retry post-send transport failures', async () => {
        const nativePrototype = Object.getPrototypeOf(Impit.prototype);
        const originalFetch = nativePrototype.fetch;
        const originalHttpRequest = http.request;
        const originalHttpsRequest = https.request;
        let compatibilityTransportCalls = 0;

        nativePrototype.fetch = async function () {
            throw new Error('ReadError: Failed to read data from the server.');
        };
        http.request = ((...args: Parameters<typeof http.request>) => {
            compatibilityTransportCalls += 1;
            return originalHttpRequest(...args);
        }) as typeof http.request;
        https.request = ((...args: Parameters<typeof https.request>) => {
            compatibilityTransportCalls += 1;
            return originalHttpsRequest(...args);
        }) as typeof https.request;

        try {
            const impit = new Impit({
                browser: Browser.Chrome,
                vanillaFallback: true,
            });

            await expect(impit.fetch('https://example.com/')).rejects.toThrow(
                /Failed to read data from the server/i,
            );
            expect(compatibilityTransportCalls).toBe(0);
        } finally {
            nativePrototype.fetch = originalFetch;
            http.request = originalHttpRequest;
            https.request = originalHttpsRequest;
        }
    });

    test('does not retry when vanillaFallback is disabled', async () => {
        const nativePrototype = Object.getPrototypeOf(Impit.prototype);
        const originalFetch = nativePrototype.fetch;
        let callCount = 0;

        nativePrototype.fetch = async function () {
            callCount += 1;
            throw createTransportFailure();
        };

        try {
            const impit = new Impit({
                browser: Browser.Chrome,
                vanillaFallback: false,
            });

            await expect(impit.fetch('http://localhost:3001/get')).rejects.toThrow(
                /internal HTTP library/i,
            );
            expect(callCount).toBe(1);
        } finally {
            nativePrototype.fetch = originalFetch;
        }
    });
});
