import { test, describe, expect, beforeAll, afterAll } from 'vitest';
import type { Server } from 'net';
import { runServer } from './mock.server.js';
import {
    Impit,
    ImpitError,
    RequestError,
    TransportError,
    TimeoutError,
    NetworkError,
    ConnectError,
    ProxyError,
    InvalidURL,
} from '../index.wrapper.js';

let localServer: Server | null = null;

beforeAll(async () => {
    localServer = await runServer(3003);
});

afterAll(async () => {
    await new Promise<void>((res) => {
        localServer?.close(() => res());
    });
});

describe('Integration: errors from fetch', () => {
    const impit = new Impit();

    test('invalid URL throws InvalidURL', async () => {
        await expect(impit.fetch('not-a-valid-url')).rejects.toThrow(InvalidURL);
    });

    test('unsupported protocol throws InvalidURL', async () => {
        await expect(impit.fetch('ftp://example.com')).rejects.toThrow(InvalidURL);
    });

    test('connection refused throws ConnectError', async () => {
        await expect(impit.fetch('http://localhost:1')).rejects.toThrow(ConnectError);
    });

    test('ConnectError is instanceof NetworkError and TransportError', async () => {
        try {
            await impit.fetch('http://localhost:1');
            expect.unreachable('should have thrown');
        } catch (e) {
            expect(e).toBeInstanceOf(ConnectError);
            expect(e).toBeInstanceOf(NetworkError);
            expect(e).toBeInstanceOf(TransportError);
            expect(e).toBeInstanceOf(RequestError);
            expect(e).toBeInstanceOf(ImpitError);
        }
    });

    test('timeout throws TimeoutError', async () => {
        const impit = new Impit({ timeout: 1 });

        try {
            await impit.fetch('http://127.0.0.1:3003/delay/3000');
            expect.unreachable('should have thrown');
        } catch (e) {
            expect(e).toBeInstanceOf(TimeoutError);
            expect(e).toBeInstanceOf(TransportError);
            expect(e).toBeInstanceOf(ImpitError);
        }
    });

    test('request-level timeout throws TimeoutError', async () => {
        await expect(
            impit.fetch('http://127.0.0.1:3003/delay/3000', { timeout: 1 })
        ).rejects.toThrow(TimeoutError);
    });

    test('invalid proxy URL throws ProxyError or ConnectError', async () => {
        const impit = new Impit({ proxyUrl: 'http://localhost:1' });

        try {
            await impit.fetch('https://example.com');
            expect.unreachable('should have thrown');
        } catch (e) {
            // Proxy connection failures can surface as either ProxyError or ConnectError
            expect(e).toBeInstanceOf(ImpitError);
            expect(
                e instanceof ProxyError || e instanceof ConnectError
            ).toBe(true);
        }
    });

    test('errors have correct name property', async () => {
        try {
            await impit.fetch('not-a-valid-url');
            expect.unreachable('should have thrown');
        } catch (e) {
            expect((e as Error).name).toBe('InvalidURL');
        }
    });

    test('errors have a message', async () => {
        try {
            await impit.fetch('not-a-valid-url');
            expect.unreachable('should have thrown');
        } catch (e) {
            expect((e as Error).message).toBeTruthy();
            expect((e as Error).message.length).toBeGreaterThan(0);
        }
    });
});
