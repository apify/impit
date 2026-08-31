import { spawn } from 'node:child_process';
import { readFile, writeFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import { arch, platform } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { installSize, measure, parseArgs } from '../harness.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);

const CHROME_JA3 = '771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,'
  + '0-23-65281-10-11-35-16-5-13-18-51-45-43-27-17513,29-23-24,0';
const CHROME_UA = 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) '
  + 'Chrome/131.0.0.0 Safari/537.36';

// Read off disk rather than through `require`: some of these packages have an
// `exports` map that hides ./package.json.
async function packageVersion(pkg) {
  const manifest = await readFile(join(here, 'node_modules', pkg, 'package.json'), 'utf8');
  return JSON.parse(manifest).version;
}

/**
 * Number of distinct impersonation targets the public API accepts, with aliases
 * that merely resolve to another target left out. `null` means there is no set to
 * count, and the client supplies a `profilesLabel` for the cell instead.
 */
const profileCounts = {
  // The browser list is a TS union, so the shipped declaration file is the only
  // machine-readable form of it. `chrome`/`firefox`/`okhttp` are aliases for the
  // newest version of their family.
  async impit() {
    const dts = await readFile(join(here, 'node_modules/impit/index.d.ts'), 'utf8');
    const union = /export type Browser =([^;]+);/.exec(dts);
    if (!union) throw new Error('could not find the Browser union in impit/index.d.ts');
    const names = [...union[1].matchAll(/'([^']+)'/g)].map((m) => m[1]);
    if (names.length === 0) throw new Error('impit Browser union parsed as empty');
    return names.filter((name) => /\d/.test(name)).length;
  },
  nodeTlsClient() {
    return Object.keys(require('node-tls-client').ClientIdentifier).length;
  },
};

const CLIENTS = [
  {
    key: 'impit',
    label: 'impit',
    repo: null,
    backend: 'Rust',
    profiles: profileCounts.impit,
    async setup(url) {
      const { Impit } = await import('impit');
      const client = new Impit({ browser: 'chrome', ignoreTlsErrors: true });
      return {
        request: async () => {
          const response = await client.fetch(url);
          return { body: await response.text(), alpn: response.headers.get('x-alpn') };
        },
      };
    },
  },
  {
    key: 'got-scraping',
    label: 'got-scraping',
    repo: 'https://github.com/apify/got-scraping',
    backend: 'Node.js TLS',
    // `knownCiphers` in got-scraping's bundle is module-private: chrome, firefox
    // and safari. Nothing exposes it at runtime, so it cannot be derived. They
    // cover cipher and signature algorithm order only, not extension order,
    // GREASE or HTTP/2 SETTINGS.
    profiles: () => 3,
    async setup(url) {
      const { gotScraping } = await import('got-scraping');
      const client = gotScraping.extend({
        https: { rejectUnauthorized: false },
        retry: { limit: 0 },
        headerGeneratorOptions: { browsers: [{ name: 'chrome' }] },
      });
      return {
        request: async () => {
          const response = await client(url);
          return { body: response.body, alpn: response.headers['x-alpn'] };
        },
      };
    },
  },
  {
    key: 'node-tls-client',
    label: 'node-tls-client',
    repo: 'https://github.com/Sahil1337/node-tls-client',
    backend: 'Go',
    profiles: profileCounts.nodeTlsClient,
    async setup(url) {
      const { ClientIdentifier, Session, destroyTLS, initTLS } = await import('node-tls-client');
      await initTLS();
      const session = new Session({
        clientIdentifier: ClientIdentifier.chrome_131,
        insecureSkipVerify: true,
      });
      return {
        request: async () => {
          const response = await session.get(url);
          return { body: await response.text(), alpn: response.headers['X-Alpn']?.[0] };
        },
        teardown: async () => {
          await session.close();
          await destroyTLS();
        },
      };
    },
  },
  {
    key: 'cycletls',
    label: 'cycletls',
    repo: 'https://github.com/Danny-Dasilva/CycleTLS',
    backend: 'Go subprocess',
    // Configured with a raw JA3 string, so there is no fixed set to count.
    profiles: () => null,
    profilesLabel: 'raw JA3',
    async setup(url) {
      const initCycleTLS = (await import('cycletls')).default;
      const client = await initCycleTLS();
      return {
        request: async () => {
          const response = await client.get(url, {
            ja3: CHROME_JA3,
            userAgent: CHROME_UA,
            insecureSkipVerify: true,
          });
          return { body: await response.text(), alpn: response.headers['X-Alpn']?.[0] };
        },
        teardown: () => client.exit(),
      };
    },
  },
  {
    key: 'undici',
    label: 'undici',
    repo: null,
    backend: 'Node.js',
    baseline: true,
    profiles: () => null,
    async setup(url) {
      const { Agent, request } = await import('undici');
      const dispatcher = new Agent({ connect: { rejectUnauthorized: false }, allowH2: true });
      return {
        request: async () => {
          const response = await request(url, { dispatcher });
          return { body: await response.body.text(), alpn: response.headers['x-alpn'] };
        },
        teardown: () => dispatcher.close(),
      };
    },
  },
];

function startServer() {
  const child = spawn(process.execPath, [join(here, '..', 'server.mjs')], {
    env: { ...process.env, PORT: '0' },
    stdio: ['ignore', 'pipe', 'inherit'],
  });
  return new Promise((resolve, reject) => {
    let buffered = '';
    child.stdout.on('data', (chunk) => {
      buffered += chunk;
      const newline = buffered.indexOf('\n');
      if (newline !== -1) resolve({ child, url: buffered.slice(0, newline) });
    });
    child.on('exit', (code) => reject(new Error(`server exited with ${code} before listening`)));
  });
}

const options = parseArgs(process.argv.slice(2), {
  requests: 2000,
  runs: 11,
  warmup: 200,
  bodyBytes: 1024,
  out: join(here, '..', 'results-node.json'),
  only: '',
});

const selected = options.only
  ? CLIENTS.filter((client) => options.only.split(',').includes(client.key))
  : CLIENTS;
if (selected.length === 0) throw new Error(`--only matched no client: ${options.only}`);

const { child, url } = await startServer();
const results = [];
const failures = [];

// One long-lived dispatcher, so the stats connection is opened once and does not
// show up in any client's connection count.
const { Agent, request } = await import('undici');
const statsDispatcher = new Agent({ connect: { rejectUnauthorized: false } });
const readStats = async () => {
  const response = await request(new URL('/__stats', url), { dispatcher: statsDispatcher });
  return response.body.json();
};
await readStats();

try {
  for (const client of selected) {
    process.stderr.write(`${client.key}: `);
    let teardown;
    try {
      const setup = await client.setup(url);
      teardown = setup.teardown;

      const probe = await setup.request();
      if (probe.body.length !== options.bodyBytes) {
        throw new Error(`expected a ${options.bodyBytes} byte body, got ${probe.body.length}`);
      }

      const before = await readStats();
      const timings = await measure(setup.request, options);
      const after = await readStats();

      const version = await packageVersion(client.key);
      results.push({
        key: client.key,
        label: client.label,
        repo: client.repo,
        backend: client.backend,
        baseline: client.baseline ?? false,
        version,
        alpn: probe.alpn ?? null,
        profiles: await client.profiles(),
        profilesLabel: client.profilesLabel ?? null,
        sizeBytes: await installSize(client.key, version),
        connections: after.connections - before.connections,
        ...timings,
      });
      const { rps, connections } = results.at(-1);
      process.stderr.write(`${rps.toFixed(0)} req/s over ${probe.alpn}, ${connections} connection(s)\n`);
    } catch (error) {
      failures.push(`${client.key}: ${error.message}`);
      process.stderr.write(`FAILED (${error.message})\n`);
    } finally {
      await teardown?.();
    }
  }
} finally {
  await statsDispatcher.close();
  child.kill();
}

await writeFile(options.out, `${JSON.stringify({
  ecosystem: 'node',
  runtime: `Node.js ${process.version}`,
  platform: `${platform()}-${arch()}`,
  measuredAt: new Date().toISOString(),
  options: {
    requests: options.requests,
    runs: options.runs,
    warmup: options.warmup,
    bodyBytes: options.bodyBytes,
  },
  results,
}, null, 2)}\n`);

process.stderr.write(`wrote ${options.out}\n`);
if (failures.length > 0) {
  process.stderr.write(`${failures.length} client(s) failed:\n${failures.join('\n')}\n`);
  process.exitCode = 1;
}
