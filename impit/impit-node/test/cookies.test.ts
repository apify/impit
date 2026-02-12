import { test, describe, expect, beforeAll, afterAll } from 'vitest';
import { Impit, Browser } from '../index.wrapper.js';
import type { Server } from 'net';
import { runServer } from './mock.server.js';
import { CookieJar } from 'tough-cookie';

let localServer: Server | null = null;
let baseUrl: string;

beforeAll(async () => {
    localServer = await runServer(3003);
    baseUrl = 'http://127.0.0.1:3003';
});

afterAll(async () => {
    await new Promise<void>((res) => {
        localServer?.close(() => res());
    });
});

describe('Cookie handling', () => {
    describe('with cookie jar', () => {
        test('preset cookies are sent with requests', async () => {
            const cookieJar = new CookieJar();
            cookieJar.setCookieSync('test=value123; Path=/', baseUrl);

            const impit = new Impit({ cookieJar });

            const response = await impit.fetch(`${baseUrl}/cookies`);
            const json = await response.json();

            expect(json.cookies).toEqual({ test: 'value123' });
        });

        test('multiple preset cookies are sent', async () => {
            const cookieJar = new CookieJar();
            cookieJar.setCookieSync('cookie1=value1; Path=/', baseUrl);
            cookieJar.setCookieSync('cookie2=value2; Path=/', baseUrl);
            cookieJar.setCookieSync('cookie3=value3; Path=/', baseUrl);

            const impit = new Impit({ cookieJar });

            const response = await impit.fetch(`${baseUrl}/cookies`);
            const json = await response.json();

            expect(json.cookies).toEqual({
                cookie1: 'value1',
                cookie2: 'value2',
                cookie3: 'value3',
            });
        });

        test('cookies set by server (no redirect) are stored', async () => {
            const cookieJar = new CookieJar();
            const impit = new Impit({ cookieJar });

            // Set a cookie without redirect
            await impit.fetch(`${baseUrl}/cookies/set-no-redirect?server-cookie=abc`);

            // Verify cookie is stored in jar
            const cookies = cookieJar.getCookieStringSync(baseUrl);
            expect(cookies).toContain('server-cookie=abc');

            // Verify cookie is sent in subsequent request
            const response = await impit.fetch(`${baseUrl}/cookies`);
            const json = await response.json();
            expect(json.cookies['server-cookie']).toBe('abc');
        });

        test('cookies set during redirect (302) are stored', async () => {
            const cookieJar = new CookieJar();
            const impit = new Impit({ cookieJar });

            // This endpoint sets a cookie and redirects to /cookies
            const response = await impit.fetch(`${baseUrl}/cookies/set?redirect-cookie=xyz`);
            const json = await response.json();

            // Should see the cookie in the response (from the redirect target)
            expect(json.cookies['redirect-cookie']).toBe('xyz');

            // Cookie should be in the jar
            expect(cookieJar.getCookieStringSync(baseUrl)).toContain('redirect-cookie=xyz');
        });

        test.each([301, 302, 303, 307, 308])('cookies set during %i redirect are stored', async (status) => {
            const cookieJar = new CookieJar();
            const impit = new Impit({ cookieJar });

            const response = await impit.fetch(`${baseUrl}/cookies/set/${status}?status${status}=value`);
            const json = await response.json();

            expect(json.cookies[`status${status}`]).toBe('value');
            expect(cookieJar.getCookieStringSync(baseUrl)).toContain(`status${status}=value`);
        });

        test('cookies are collected across multiple redirect hops', async () => {
            const cookieJar = new CookieJar();
            const impit = new Impit({ cookieJar });

            // This endpoint sets a cookie at each hop and redirects 3 times
            const response = await impit.fetch(`${baseUrl}/cookies/chain/3`);
            const json = await response.json();

            // Should have cookies from all 3 hops
            expect(json.cookies).toEqual({
                hop1: 'value1',
                hop2: 'value2',
                hop3: 'value3',
            });

            // All cookies should be in the jar
            const cookieString = cookieJar.getCookieStringSync(baseUrl);
            expect(cookieString).toContain('hop1=value1');
            expect(cookieString).toContain('hop2=value2');
            expect(cookieString).toContain('hop3=value3');
        });

        test('preset and server-set cookies coexist', async () => {
            const cookieJar = new CookieJar();
            cookieJar.setCookieSync('preset=existing; Path=/', baseUrl);

            const impit = new Impit({ cookieJar });

            // Set additional cookie via server
            await impit.fetch(`${baseUrl}/cookies/set-no-redirect?from-server=new`);

            const response = await impit.fetch(`${baseUrl}/cookies`);
            const json = await response.json();

            expect(json.cookies).toEqual({
                preset: 'existing',
                'from-server': 'new',
            });
        });

        test('cookie jar is shared across multiple Impit instances', async () => {
            const cookieJar = new CookieJar();

            const impit1 = new Impit({ cookieJar });
            const impit2 = new Impit({ cookieJar });

            // Set cookie via first instance
            await impit1.fetch(`${baseUrl}/cookies/set-no-redirect?shared=123`);

            // Should be visible via second instance
            const response = await impit2.fetch(`${baseUrl}/cookies`);
            const json = await response.json();

            expect(json.cookies.shared).toBe('123');
        });

        test('request headers can override cookie jar cookies', async () => {
            const cookieJar = new CookieJar();
            cookieJar.setCookieSync('override=original; Path=/', baseUrl);

            const impit = new Impit({ cookieJar });

            const response = await impit.fetch(`${baseUrl}/cookies`, {
                headers: {
                    'Cookie': 'override=replaced'
                }
            });
            const json = await response.json();

            // The request header should take precedence
            expect(json.cookies.override).toBe('replaced');
        });
    });

    describe('without cookie jar', () => {
        test('cookies in request headers are sent', async () => {
            const impit = new Impit();

            const response = await impit.fetch(`${baseUrl}/cookies`, {
                headers: {
                    'Cookie': 'manual=cookie123'
                }
            });
            const json = await response.json();

            expect(json.cookies.manual).toBe('cookie123');
        });

        test('cookies are not persisted without cookie jar', async () => {
            const impit = new Impit();

            // Server sets a cookie
            await impit.fetch(`${baseUrl}/cookies/set-no-redirect?transient=value`);

            // Cookie should NOT be sent in subsequent request (no jar to store it)
            const response = await impit.fetch(`${baseUrl}/cookies`);
            const json = await response.json();

            expect(json.cookies.transient).toBeUndefined();
        });

        test('Set-Cookie headers are still returned in response', async () => {
            const impit = new Impit({ followRedirects: false });

            const response = await impit.fetch(`${baseUrl}/cookies/set?visible=true`);

            const setCookies = response.headers.getSetCookie();
            expect(setCookies.some(c => c.includes('visible=true'))).toBe(true);
        });
    });

    describe('with followRedirects disabled', () => {
        test('redirect response with Set-Cookie is returned', async () => {
            const cookieJar = new CookieJar();
            const impit = new Impit({ cookieJar, followRedirects: false });

            const response = await impit.fetch(`${baseUrl}/cookies/set?no-follow=test`);

            expect(response.status).toBe(302);
            expect(response.headers.get('location')).toBe('/cookies');

            // Cookie should still be stored even though redirect wasn't followed
            const setCookies = response.headers.getSetCookie();
            expect(setCookies.some(c => c.includes('no-follow=test'))).toBe(true);
        });
    });

    describe('with browser emulation', () => {
        test.each([Browser.Chrome, Browser.Firefox])('%s: cookies work with browser emulation', async (browser) => {
            const cookieJar = new CookieJar();
            cookieJar.setCookieSync('browser-test=emulated; Path=/', baseUrl);

            const impit = new Impit({ cookieJar, browser });

            const response = await impit.fetch(`${baseUrl}/cookies`);
            const json = await response.json();

            expect(json.cookies['browser-test']).toBe('emulated');
        });

        test.each([Browser.Chrome, Browser.Firefox])('%s: cookies persist through redirects', async (browser) => {
            const cookieJar = new CookieJar();
            const impit = new Impit({ cookieJar, browser });

            const response = await impit.fetch(`${baseUrl}/cookies/set?browser-redirect=works`);
            const json = await response.json();

            expect(json.cookies['browser-redirect']).toBe('works');
        });
    });
});
