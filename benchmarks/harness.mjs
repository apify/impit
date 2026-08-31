import { spawn } from 'node:child_process';
import { mkdtemp, readdir, rm, stat } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

export function parseArgs(argv, defaults) {
  const out = { ...defaults };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (!arg.startsWith('--')) throw new Error(`unexpected argument ${arg}`);
    const key = arg.slice(2).replace(/-([a-z])/g, (_, c) => c.toUpperCase());
    if (!(key in out)) throw new Error(`unknown option ${arg}`);
    const value = argv[i + 1];
    if (value === undefined || value.startsWith('--')) throw new Error(`${arg} needs a value`);
    out[key] = typeof out[key] === 'number' ? Number(value) : value;
    i += 1;
  }
  return out;
}

/**
 * Runs `requests` sequential requests `runs` times and keeps the best run.
 * Sequential traffic over one warm connection isolates per-request client
 * overhead, which is what the comparison is about; best-of-N absorbs scheduler
 * noise without hiding a client that is genuinely slow.
 */
export async function measure(request, { requests, runs, warmup }) {
  for (let i = 0; i < warmup; i += 1) await request();

  const rates = [];
  for (let run = 0; run < runs; run += 1) {
    const start = process.hrtime.bigint();
    for (let i = 0; i < requests; i += 1) await request();
    const elapsedNs = Number(process.hrtime.bigint() - start);
    rates.push((requests * 1e9) / elapsedNs);
  }
  rates.sort((a, b) => a - b);
  return {
    rps: rates.at(-1),
    rpsMedian: rates[Math.floor(rates.length / 2)],
    rpsWorst: rates[0],
  };
}

async function treeSize(dir) {
  let total = 0;
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) total += await treeSize(path);
    else if (entry.isFile()) total += (await stat(path)).size;
  }
  return total;
}

function run(command, args, options = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, { stdio: ['ignore', 'ignore', 'pipe'], ...options });
    let stderr = '';
    child.stderr.on('data', (chunk) => { stderr += chunk; });
    child.on('error', reject);
    child.on('close', (code) => {
      if (code === 0) resolve();
      else reject(new Error(`${command} exited with ${code}: ${stderr.trim().slice(0, 500)}`));
    });
  });
}

/** Bytes a fresh `npm install <pkg>` drops on disk, transitive dependencies included. */
export async function installSize(pkg, version) {
  const dir = await mkdtemp(join(tmpdir(), 'impit-bench-size-'));
  try {
    await run('npm', [
      'install', `${pkg}@${version}`,
      '--prefix', dir,
      '--no-save', '--no-audit', '--no-fund', '--loglevel', 'error',
    ]);
    return await treeSize(join(dir, 'node_modules'));
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
}

export function formatMB(bytes) {
  return `${(bytes / 1e6).toFixed(1)} MB`;
}
