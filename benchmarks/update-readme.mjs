import { readFile, writeFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { formatMB, parseArgs } from './harness.mjs';

const here = dirname(fileURLToPath(import.meta.url));

const START = '<!-- comparison:start -->';
const END = '<!-- comparison:end -->';

const options = parseArgs(process.argv.slice(2), {
  readme: join(here, '..', 'README.md'),
  node: join(here, 'results-node.json'),
  python: join(here, 'results-python.json'),
});

async function load(path) {
  const report = JSON.parse(await readFile(path, 'utf8'));
  if (report.results.length === 0) throw new Error(`${path} contains no results`);
  // The caption speaks for every row, so a client that fell back to HTTP/1.1 has
  // to stop the run rather than end up mislabelled.
  const odd = report.results.find((result) => result.alpn !== 'h2');
  if (odd) throw new Error(`${odd.key} negotiated ${odd.alpn}, not h2`);
  return report;
}

const [python, node] = await Promise.all([load(options.python), load(options.node)]);

if (JSON.stringify(python.options) !== JSON.stringify(node.options)) {
  throw new Error('the two reports were taken with different parameters; rerun both');
}

function table(report, sizeHeading) {
  const ordered = [...report.results]
    .sort((a, b) => (a.baseline - b.baseline) || (b.rpsMedian - a.rpsMedian));
  const rows = ordered.map((result) => {
    const name = result.repo ? `[\`${result.label}\`](${result.repo})` : `\`${result.label}\``;
    return [
      result.baseline ? `${name} (no impersonation)` : (result.repo ? name : `**${name}**`),
      result.rpsMedian.toFixed(0),
      formatMB(result.sizeBytes),
      result.profiles ?? result.profilesLabel ?? '—',
      result.backend,
    ];
  });
  return [
    `| Package | req/s | ${sizeHeading} | Profiles | Backend |`,
    '| --- | --- | --- | --- | --- |',
    ...rows.map((cells) => `| ${cells.join(' | ')} |`),
  ].join('\n');
}

const { requests, runs, bodyBytes } = python.options;

const body = [
  '### Comparison',
  '',
  `Median of ${runs} runs of ${requests} sequential requests to a local HTTP/2 server, `
    + `${bodyBytes / 1024} KiB JSON responses over one warm connection. \`Profiles\` counts the `
    + 'impersonation targets each API exposes.',
  '',
  '**Python**',
  '',
  table(python, 'Wheel'),
  '',
  '**Node.js**',
  '',
  table(node, 'Install'),
  '',
  `Measured by [\`benchmarks/\`](benchmarks) on ${node.platform}, ${python.measuredAt.slice(0, 10)}.`
    + ' Rerun it on your own hardware.',
].join('\n');

const readme = await readFile(options.readme, 'utf8');
const start = readme.indexOf(START);
const end = readme.indexOf(END);
if (start === -1 || end === -1) {
  throw new Error(`${options.readme} is missing the ${START} / ${END} markers`);
}

const updated = `${readme.slice(0, start + START.length)}\n${body}\n${readme.slice(end)}`;
if (updated === readme) {
  process.stderr.write('README is already up to date\n');
} else {
  await writeFile(options.readme, updated);
  process.stderr.write(`updated ${options.readme}\n`);
}
