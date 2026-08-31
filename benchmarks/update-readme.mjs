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
  return report;
}

const [python, node] = await Promise.all([load(options.python), load(options.node)]);

// The caption speaks for both tables at once, so it may only be written when
// the two reports really are comparable.
if (JSON.stringify(python.options) !== JSON.stringify(node.options)) {
  throw new Error('the two reports were taken with different parameters; rerun both');
}

/** Footnote markers are assigned in the order the tables reference them. */
const footnotes = [];
function footnote(text) {
  const existing = footnotes.indexOf(text);
  return `[^${(existing === -1 ? footnotes.push(text) : existing + 1)}]`;
}

const dominantAlpn = [...python.results, ...node.results]
  .map((result) => result.alpn)
  .reduce((agreed, alpn) => (agreed === alpn ? agreed : null));

/** Above this best-to-worst ratio a client's throughput is too unsteady to quote as one number. */
const UNSTABLE_RATIO = 1.5;

/** Notes about how the throughput figure was reached, rendered next to it. */
function throughputNotes(result, report) {
  const notes = [];
  if (result.alpn !== dominantAlpn) {
    notes.push(footnote(`\`${result.label}\` negotiated ${result.alpn} rather than ${dominantAlpn}.`));
  }
  if (result.rps > result.rpsWorst * UNSTABLE_RATIO) {
    notes.push(footnote(`\`${result.label}\` was erratic across runs — ${result.rpsWorst.toFixed(0)} to `
      + `${result.rps.toFixed(0)} req/s — so its median says less than the others'.`));
  }
  const total = report.options.runs * report.options.requests;
  if (result.connections >= total / 2) {
    notes.push(footnote(`\`${result.label}\` opens a new connection for every request, so its figure `
      + 'includes a TLS handshake each time instead of reusing a warm one.'));
  } else if (result.connections > report.options.runs) {
    notes.push(footnote(`\`${result.label}\` reconnected ${result.connections} times mid-run.`));
  }
  return notes.join('');
}

function table(report, sizeHeading) {
  const ordered = [...report.results]
    .sort((a, b) => (a.baseline - b.baseline) || (b.rpsMedian - a.rpsMedian));
  const rows = ordered.map((result) => {
    const name = result.repo ? `[\`${result.label}\`](${result.repo})` : `\`${result.label}\``;
    return [
      result.baseline ? `${name} (no impersonation)` : (result.repo ? name : `**${name}**`),
      `${result.rpsMedian.toFixed(0)}${throughputNotes(result, report)}`,
      formatMB(result.sizeBytes),
      `${result.profiles ?? '—'}${result.note ? footnote(result.note) : ''}`,
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
const caption = [
  `Sequential requests from a single client against the local HTTP/2 origin in [\`benchmarks/\`](benchmarks),`,
  `${bodyBytes / 1024} KiB JSON response, median of ${runs} runs of ${requests} requests.`,
  dominantAlpn ? `Every client negotiated ${dominantAlpn}.` : '',
  'Each one keeps a single connection warm for the whole run unless a footnote says otherwise.',
  '`Profiles` counts the distinct impersonation targets each public API accepts, ignoring aliases that',
  'resolve to another target. Python sizes are the platform wheel; Node.js sizes are what',
  '`npm install <package>` leaves on disk, transitive dependencies included.',
].filter(Boolean).join(' ');

const platforms = [...new Set([python.platform, node.platform])].join(' / ');
const provenance = `Measured on ${platforms} with ${python.runtime} and ${node.runtime}`
  + ` on ${python.measuredAt.slice(0, 10)}. Hardware moves these numbers around, so rerun`
  + ' `benchmarks/` yourself before drawing conclusions.';

const body = [
  '### Comparison',
  '',
  caption,
  '',
  '**Python**',
  '',
  table(python, 'Wheel'),
  '',
  '**Node.js**',
  '',
  table(node, 'Install'),
  '',
  provenance,
  ...(footnotes.length > 0
    ? ['', ...footnotes.map((text, index) => `[^${index + 1}]: ${text}`)]
    : []),
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
