import { createServer } from 'node:http';
import type { AddressInfo, Socket } from 'node:net';
import { Impit } from '..';
import { CookieJar } from 'tough-cookie';
import {
    afterAll,
    beforeAll,
    beforeEach,
    describe,
    expect,
    test,
} from 'vitest';

// Distinct loopback cookie hosts, with no external endpoints or real credentials.
describe('redirect credential boundaries', () => {
    const seen: {
        path: string;
        authorization: string | undefined;
        cookie: string | undefined;
    }[] = [];
    let origin = '';
    let destination = '';
    const server = createServer((req, res) => {
        seen.push({
            path: req.url ?? '',
            authorization: req.headers.authorization,
            cookie: req.headers.cookie,
        });
        if (req.url === '/start') {
            res.writeHead(302, { location: '/same' });
        } else if (req.url === '/same') {
            res.writeHead(302, { location: `${destination}/renew` });
        } else if (req.url === '/renew') {
            res.writeHead(302, {
                location: `${origin}/return`,
                'set-cookie': ['b_seed=renewed; Path=/', 'b_added=new; Path=/'],
            });
        } else if (req.url === '/empty-start') {
            res.writeHead(302, {
                location: `${destination}/empty`,
                'set-cookie': 'a_jar=origin; Path=/',
            });
        } else if (req.url === '/empty') {
            res.writeHead(302, { location: `${origin}/return` });
        }
        res.end('ok');
    });

    const sockets = new Set<Socket>();
    server.on('connection', (socket) => {
        sockets.add(socket);
        socket.once('close', () => sockets.delete(socket));
    });

    beforeAll(async () => {
        await new Promise<void>((resolve, reject) => {
            server.once('error', reject);
            server.listen(0, '127.0.0.1', () => {
                server.removeListener('error', reject);
                resolve();
            });
        });
        const { port } = server.address() as AddressInfo;
        origin = `http://127.0.0.1:${port}`;
        destination = `http://localhost:${port}`;
    }, 5000);

    beforeEach(() => {
        seen.length = 0;
    });

    afterAll(async () => {
        if (server.listening) {
            const closed = new Promise<void>((resolve, reject) => {
                server.close((error) => {
                    error ? reject(error) : resolve();
                });
            });
            const socketsClosed = [...sockets].map(
                (socket) =>
                    new Promise<void>((resolve) => socket.once('close', resolve))
            );
            server.closeAllConnections();
            await Promise.all([closed, ...socketsClosed]);
        }
        expect(server.listening).toBe(false);
        expect(sockets.size).toBe(0);
    }, 5000);

    test('native request overrides stop at the first cross-origin hop and use each destination jar', async () => {
        const jar = new CookieJar();
        await jar.setCookie('a_jar=origin; Path=/', origin);
        await jar.setCookie('b_seed=old; Path=/', destination);
        const defaults: [string, string][] = [
            ['AUTHORIZATION', 'Bearer dummy-default'],
            ['COOKIE', 'default=dummy'],
        ];
        const headers: [string, string][] = [
            ['aUtHoRiZaTiOn', 'Bearer dummy-request'],
            ['cOoKiE', 'request=dummy'],
        ];
        const before = structuredClone({ defaults, headers });
        const client = new Impit({
            headers: defaults,
            cookieJar: jar,
            timeout: 2000,
        });
        const response = await client.fetch(`${origin}/start`, {
            headers,
            signal: AbortSignal.timeout(2000),
        });
        await expect(response.text()).resolves.toBe('ok');
        expect({ defaults, headers }).toEqual(before);
        expect(seen).toEqual([
            {
                path: '/start',
                authorization: 'Bearer dummy-request',
                cookie: 'request=dummy',
            },
            {
                path: '/same',
                authorization: 'Bearer dummy-request',
                cookie: 'request=dummy',
            },
            { path: '/renew', authorization: undefined, cookie: 'b_seed=old' },
            { path: '/return', authorization: undefined, cookie: 'a_jar=origin' },
        ]);
        const probe = await new Impit({ cookieJar: jar, timeout: 2000 }).fetch(
            `${destination}/inspect`,
            { signal: AbortSignal.timeout(2000) }
        );
        await expect(probe.text()).resolves.toBe('ok');
        expect(seen.at(-1)?.cookie?.split('; ').sort()).toEqual([
            'b_added=new',
            'b_seed=renewed',
        ]);
    });

    test.each([false, true])(
        'native constructor defaults are suppressed for an empty destination (jar=%s), not future fetches',
        async (useJar) => {
            const defaults: [string, string][] = [
                ['AUTHORIZATION', 'Bearer dummy-default'],
                ['COOKIE', 'default=dummy'],
            ];
            const before = structuredClone(defaults);
            const client = new Impit({
                headers: defaults,
                ...(useJar ? { cookieJar: new CookieJar() } : {}),
                timeout: 2000,
            });
            const response = await client.fetch(`${origin}/empty-start`, {
                signal: AbortSignal.timeout(2000),
            });
            await expect(response.text()).resolves.toBe('ok');
            expect(defaults).toEqual(before);
            expect(seen).toEqual([
                {
                    path: '/empty-start',
                    authorization: 'Bearer dummy-default',
                    cookie: 'default=dummy',
                },
                { path: '/empty', authorization: undefined, cookie: undefined },
                {
                    path: '/return',
                    authorization: undefined,
                    cookie: useJar ? 'a_jar=origin' : undefined,
                },
            ]);
            const independent = await client.fetch(`${origin}/inspect`, {
                signal: AbortSignal.timeout(2000),
            });
            await expect(independent.text()).resolves.toBe('ok');
            expect(seen.at(-1)).toEqual({
                path: '/inspect',
                authorization: 'Bearer dummy-default',
                cookie: useJar ? 'a_jar=origin' : 'default=dummy',
            });
        }
    );
});
