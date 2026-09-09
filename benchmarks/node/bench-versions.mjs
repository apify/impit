/**
 * Throughput of the last N published npm releases of impit, against each other.
 *
 * Unlike bench.mjs, which compares impit to other clients, this installs several
 * versions of impit itself into isolated temp dirs and benchmarks them in turn.
 */
import { createRequire } from 'node:module';
import { rm, writeFile } from 'node:fs/promises';
import { arch, platform } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { installPackage, measure, parseArgs, spawnOrigin } from '../harness.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);

async function recentVersions(pkg, count) {
  const response = await fetch(`https://registry.npmjs.org/${pkg}`);
  if (!response.ok) throw new Error(`npm registry returned ${response.status} for ${pkg}`);
  const manifest = await response.json();
  return Object.keys(manifest.versions)
    // Stable releases only, no `-beta.1`-style prereleases.
    .filter((version) => /^\d+\.\d+\.\d+$/.test(version))
    .sort((a, b) => new Date(manifest.time[a]) - new Date(manifest.time[b]))
    .slice(-count)
    .map((version) => ({ version, publishedAt: manifest.time[version] }));
}

async function benchmarkVersion({ version, publishedAt }, url, options) {
  const dir = await installPackage('impit', version);
  try {
    const { Impit } = require(join(dir, 'node_modules', 'impit'));
    const client = new Impit({ browser: 'chrome', ignoreTlsErrors: true });
    const request = async () => {
      const response = await client.fetch(url);
      return { body: await response.text(), alpn: response.headers.get('x-alpn') };
    };

    const probe = await request();
    if (probe.body.length !== options.bodyBytes) {
      throw new Error(`expected a ${options.bodyBytes} byte body, got ${probe.body.length}`);
    }

    const timings = await measure(request, options);
    return { version, publishedAt, alpn: probe.alpn, ...timings };
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
}

const options = parseArgs(process.argv.slice(2), {
  versions: 5,
  requests: 2000,
  runs: 11,
  warmup: 200,
  bodyBytes: 1024,
  out: join(here, '..', 'results-node-versions.json'),
});

const versions = await recentVersions('impit', options.versions);
const { child, url } = await spawnOrigin(options.bodyBytes);

const results = [];
const failures = [];
try {
  for (const entry of versions) {
    process.stderr.write(`impit@${entry.version}: `);
    try {
      results.push(await benchmarkVersion(entry, url, options));
      const { rpsMedian, alpn } = results.at(-1);
      process.stderr.write(`${rpsMedian.toFixed(0)} req/s over ${alpn}\n`);
    } catch (error) {
      failures.push(`${entry.version}: ${error.message}`);
      process.stderr.write(`FAILED (${error.message})\n`);
    }
  }
} finally {
  child.kill();
}

await writeFile(options.out, `${JSON.stringify({
  ecosystem: 'node',
  package: 'impit',
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
  process.stderr.write(`${failures.length} version(s) failed:\n${failures.join('\n')}\n`);
  process.exitCode = 1;
}
