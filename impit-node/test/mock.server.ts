import express from 'express';
import { Server } from 'http';
import { Server as ProxyServer } from 'proxy-chain';

// "Příliš žluťoučký kůň úpěl ďábelské ódy" encoded in Windows-1250
const WIN1250_BODY = Buffer.from([0x50, 0xf8, 0xed, 0x6c, 0x69, 0x9a, 0x20, 0x9e, 0x6c, 0x75, 0x9d, 0x6f, 0x75, 0xe8, 0x6b, 0xfd, 0x20, 0x6b, 0xf9, 0xf2, 0x20, 0xfa, 0x70, 0xec, 0x6c, 0x20, 0xef, 0xe1, 0x62, 0x65, 0x6c, 0x73, 0x6b, 0xe9, 0x20, 0xf3, 0x64, 0x79]);
const WIN1250_STRING = 'Příliš žluťoučký kůň úpěl ďábelské ódy';

export const routes = {
    charset: {
        path: '/charset',
        bodyBuffer: WIN1250_BODY,
        bodyString: WIN1250_STRING,
    },
    charsetMetaCharset: {
        path: '/charset/meta-charset',
        bodyString: WIN1250_STRING,
    },
    charsetMetaHttpEquiv: {
        path: '/charset/meta-http-equiv',
        bodyString: WIN1250_STRING,
    },
    nonAsciiHeader: {
        path: '/non-ascii-header',
        headerValue: 'Dienstag, 31. März 2026',
    },
}

export async function runServer(port: number): Promise<Server> {
    const app = express();

    app.get(routes.charset.path, (req, res) => {
        res.set('Content-Type', 'text/plain; charset=windows-1250');
        res.send(routes.charset.bodyBuffer);
    });

    app.get(routes.charsetMetaCharset.path, (req, res) => {
        const html = Buffer.concat([
            Buffer.from('<html><head><meta charset="windows-1250"></head><body>'),
            WIN1250_BODY,
            Buffer.from('</body></html>'),
        ]);
        res.writeHead(200, { 'Content-Type': 'text/html' });
        res.end(html);
    });

    app.get(routes.charsetMetaHttpEquiv.path, (req, res) => {
        const html = Buffer.concat([
            Buffer.from('<html><head><meta http-equiv="content-type" content="text/html; charset=windows-1250"></head><body>'),
            WIN1250_BODY,
            Buffer.from('</body></html>'),
        ]);
        res.writeHead(200, { 'Content-Type': 'text/html' });
        res.end(html);
    });

    app.get(routes.nonAsciiHeader.path, (req, res) => {
        const socket = res.socket!;
        socket.write('HTTP/1.1 200 OK\r\n');
        socket.write('Content-Type: text/plain\r\n');
        socket.write(Buffer.concat([
            Buffer.from('X-Non-Ascii: Dienstag, 31. M'),
            Buffer.from([0xE4]), // ä in ISO-8859-1
            Buffer.from('rz 2026\r\n'),
        ]));
        socket.write('Content-Length: 2\r\n');
        socket.write('\r\n');
        socket.write('ok');
        socket.end();
    });

    app.get('/socket', (req, res) => {
        const socket = req.socket;
        const clientAddress = socket.remoteAddress;
        const clientPort = socket.remotePort;

        res.json({ ip: clientAddress, port: clientPort });
    });

    app.get('/delay/:ms', (req, res) => {
        const delay = parseInt(req.params.ms, 10);

        res.setHeader('Content-Type', 'text/plain');
        res.write('Headers sent. Preparing body...\n');

        setTimeout(() => {
            res.end('Body sent after delay.\n');
        }, delay);
    });

    app.get('/cookies', (req, res) => {
        const cookies: Record<string, string> = {};
        const cookieHeader = req.headers.cookie;
        if (cookieHeader) {
            cookieHeader.split(';').forEach(cookie => {
                const [name, ...rest] = cookie.trim().split('=');
                cookies[name] = rest.join('=');
            });
        }
        res.json({ cookies });
    });

    app.get('/cookies/set', (req, res) => {
        for (const [name, value] of Object.entries(req.query)) {
            res.cookie(name, value as string, { path: '/' });
        }
        res.redirect(302, '/cookies');
    });

    app.get('/cookies/set/:status', (req, res) => {
        const status = parseInt(req.params.status, 10);
        for (const [name, value] of Object.entries(req.query)) {
            res.cookie(name, value as string, { path: '/' });
        }
        res.redirect(status, '/cookies');
    });

    app.get('/cookies/set-no-redirect', (req, res) => {
        for (const [name, value] of Object.entries(req.query)) {
            res.cookie(name, value as string, { path: '/' });
        }
        res.json({ status: 'cookies set' });
    });

    app.get('/cookies/chain/:hops', (req, res) => {
        const hops = parseInt(req.params.hops, 10);
        const hop = parseInt(req.query.hop as string || '1', 10);
        res.cookie(`hop${hop}`, `value${hop}`, { path: '/' });
        if (hop < hops) {
            res.redirect(302, `/cookies/chain/${hops}?hop=${hop + 1}`);
        } else {
            res.redirect(302, '/cookies');
        }
    });

    app.get('/redirect/:n', (req, res) => {
        const n = parseInt(req.params.n, 10);
        if (n > 1) {
            res.redirect(302, `/redirect/${n - 1}`);
        } else {
            res.redirect(302, '/get');
        }
    });

    app.get('/redirect-to', (req, res) => {
        const url = req.query.url as string;
        const statusCode = parseInt(req.query.status_code as string || '302', 10);
        res.redirect(statusCode, url);
    });

    app.get('/get', (req, res) => {
        res.json({ url: `http://localhost:${port}/get` });
    });

    app.get('/cookies/delete', (req, res) => {
        for (const name of Object.keys(req.query)) {
            res.clearCookie(name);
        }
        res.redirect(302, '/cookies');
    });

    return new Promise((res, rej) => {
        const server = app.listen(port, (err) => {
            if (err) {
                rej(err);
            } else {
                res(server);
            }
        });
    });
}

export async function runProxyServer(port: number): Promise<ProxyServer> {
    const server = new ProxyServer({ port });
    return new Promise((res, rej) => {
        server.listen(() => {
            res(server);
        }).catch((err) => {
            rej(err);
        });
    });
}
