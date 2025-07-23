import { test, describe, expect, beforeAll, afterAll } from 'vitest';

import { HttpMethod, Impit, Browser } from '../index.wrapper.js';
import { Server } from 'http';
import { routes, runServer } from './mock.server.js';

import { CookieJar } from 'tough-cookie';
import socks from 'socksv5';

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

async function startSocksServer() {
    return new Promise<Server>((resolve, reject) => {
        const srv = (socks as any).createServer((info, accept, deny) => accept());
        srv.listen(7625, 'localhost', () => {
            resolve(srv);
        });
        srv.on('error', (err: Error) => {
            reject(err);
        });
        srv.useAuth(socks.auth.None());
    });
}

let socksServer: Server | null = null;
beforeAll(async () => {
    // Warms up the httpbin instance, so that the first tests don't timeout.
    // Has a longer timeout itself (5s vs 30s) to avoid flakiness.
    await fetch(getHttpBinUrl('/get'));
    // Start the local server
    await getServer();

    socksServer = await startSocksServer();
}, 30e3);

afterAll(async () => {
    const server = await getServer();
    await new Promise<void>(res => {
        server?.close(() => res())
    });
    await new Promise<void>(res => {
        socksServer?.close(() => res())
    });
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

        test('supports socks proxy', async (t) => {
            const impit = new Impit({
                browser,
                proxyUrl: 'socks5://localhost:7625',
            });

            const response = await impit.fetch(
                getHttpBinUrl('/get'),
            );

            t.expect(response.status).toBe(200);
            const json = await response.json();
            expect(json).toHaveProperty('url');
            expect(json).toHaveProperty('headers');
            expect(json).toHaveProperty('origin');
        });

        test('impit accepts custom cookie jars', async (t) => {
            const cookieJar = new CookieJar();
            cookieJar.setCookieSync('preset-cookie=123; Path=/', getHttpBinUrl('/cookies/'));

            const impit = new Impit({
                cookieJar,
                browser,
            })

            const response1 = await impit.fetch(
                getHttpBinUrl('/cookies/'),
            ).then(x => x.json());

            t.expect(response1.cookies).toEqual({
                'preset-cookie': '123'
            });

            await impit.fetch(
                getHttpBinUrl('/cookies/set?set-by-server=321'),
            );

            const response2 = await impit.fetch(
                getHttpBinUrl('/cookies/'),
            ).then(x => x.json());

            t.expect(response2.cookies).toEqual({
                'preset-cookie': '123',
                'set-by-server': '321'
            });

            t.expect(cookieJar.serializeSync()?.cookies).toHaveLength(2);
        })

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
            const response = impit.fetch(getHttpBinUrl('/anything'), {
                method
            });
            await expect(response).resolves.toBeTruthy();
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

        test('.text() method works with decoding', async (t) => {
            const response = await impit.fetch(new URL(routes.charset.path, "http://127.0.0.1:3001").href);
            const text: string = await response.text();

            t.expect(text).toContain(routes.charset.bodyString);
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
            t.expect(response.headers.get('location')).toBe(getHttpBinUrl('/get', false));
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

            await t.expect(response).rejects.toThrowError('Too many redirects occurred. Maximum allowed');
        });
    })
});
