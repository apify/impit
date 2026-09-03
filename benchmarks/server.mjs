import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync } from 'node:fs';
import { createSecureServer } from 'node:http2';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const certDir = join(here, '.cert');
const keyPath = join(certDir, 'key.pem');
const certPath = join(certDir, 'cert.pem');

function ensureCert() {
  if (existsSync(keyPath) && existsSync(certPath)) return;
  mkdirSync(certDir, { recursive: true });
  execFileSync('openssl', [
    'req', '-x509', '-newkey', 'rsa:2048', '-nodes', '-sha256',
    '-days', '365',
    '-subj', '/CN=localhost',
    '-addext', 'subjectAltName=DNS:localhost,IP:127.0.0.1',
    '-keyout', keyPath,
    '-out', certPath,
  ], { stdio: 'ignore' });
}

// Precomputed so the server does no per-request work beyond the write.
function jsonBody(bytes) {
  const envelope = Buffer.byteLength('{"filler":""}');
  return Buffer.from(`{"filler":"${'x'.repeat(bytes - envelope)}"}`);
}

export const BODY_BYTES = 1024;

export function startServer({ port = 0, bodyBytes = BODY_BYTES } = {}) {
  ensureCert();
  const body = jsonBody(bodyBytes);
  const server = createSecureServer({
    key: readFileSync(keyPath),
    cert: readFileSync(certPath),
    allowHTTP1: true,
    ALPNProtocols: ['h2', 'http/1.1'],
    // Clients that RST_STREAM every response (got's http2-wrapper does) burn
    // Node's Rapid-Reset budget after ~1000 requests and take a GOAWAY mid-run.
    streamResetBurst: Number.MAX_SAFE_INTEGER,
    streamResetRate: Number.MAX_SAFE_INTEGER,
  });

  // Only connections that carried benchmark traffic count, so the benchmark's own
  // polling of /__stats stays out of a client's total.
  const stats = { connections: 0, requests: 0 };
  const counted = new WeakSet();

  server.on('request', (req, res) => {
    if (req.url === '/__stats') {
      const json = Buffer.from(JSON.stringify(stats));
      res.writeHead(200, { 'content-type': 'application/json', 'content-length': json.length });
      res.end(json);
      return;
    }

    const connection = req.stream?.session ?? req.socket;
    if (!counted.has(connection)) {
      counted.add(connection);
      stats.connections += 1;
    }

    stats.requests += 1;
    res.writeHead(200, {
      'content-type': 'application/json',
      'content-length': body.length,
      'x-alpn': req.httpVersion === '2.0' ? 'h2' : `http/${req.httpVersion}`,
    });
    res.end(body);
  });

  // Clients drop their connection at the end of a run; that must not kill the server.
  server.on('session', (session) => session.on('error', () => {}));
  server.on('clientError', () => {});
  server.on('sessionError', () => {});

  return new Promise((resolve) => {
    server.listen(port, '127.0.0.1', () => {
      resolve({ server, url: `https://localhost:${server.address().port}/`, certPath });
    });
  });
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const { url } = await startServer({
    port: Number(process.env.PORT ?? 8443),
    bodyBytes: Number(process.env.BODY_BYTES ?? BODY_BYTES),
  });
  process.stdout.write(`${url}\n`);
}
