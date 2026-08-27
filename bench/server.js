import http2 from 'node:http2';
import fs from 'node:fs';

const body = JSON.stringify({ data: 'x'.repeat(1000) });

const server = http2.createSecureServer({
    key: fs.readFileSync(new URL('./key.pem', import.meta.url)),
    cert: fs.readFileSync(new URL('./cert.pem', import.meta.url)),
    allowHTTP1: true,
    settings: { maxConcurrentStreams: 1000 },
    maxSessionMemory: 512,
});

server.on('request', (req, res) => {
    res.writeHead(200, { 'content-type': 'application/json' });
    res.end(body);
});

server.on('session', (s) => {
    s.setTimeout(0);
    s.on('error', (e) => console.log('session error', e.code, e.message));
    s.on('localSettings', () => {});
});

server.on('sessionError', (e) => console.log('sessionError', e.code, e.message));
server.on('clientError', (e) => console.log('clientError', e.code, e.message));

server.listen(8443, '127.0.0.1', () => console.log('listening'));
