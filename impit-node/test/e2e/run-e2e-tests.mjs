import { readdir } from 'node:fs/promises';
import { spawn } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const exts = new Set(['.mts', '.mjs', '.js', '.cts', '.cjs', '.ts']);

const selfPath = fileURLToPath(import.meta.url);
const selfDir = path.dirname(selfPath);
const selfBase = path.basename(selfPath);

const toAbs = (p) => (path.isAbsolute(p) ? p : path.resolve(process.cwd(), p));

function buildSpawnArgsFor(filePath) {
    const nodeCmd = process.argv[0]; // path to node
    const resolvedArgv = process.argv.map(toAbs);
    const selfIdx = resolvedArgv.findIndex((a) => a === selfPath);

    if (selfIdx > -1) {
        // Replicate the original invocation "prefix" (e.g. tsx/ts-node loaders etc.)
        const prefix = process.argv.slice(1, selfIdx); // keep as-is (can include non-absolute args)
        return { cmd: nodeCmd, args: [...prefix, filePath] };
    }

    // Fallback: use execArgv (e.g. --loader ts-node/esm, --experimental-loader, etc.)
    return { cmd: nodeCmd, args: [...process.execArgv, filePath] };
}

async function runOne(filePath) {
    const { cmd, args } = buildSpawnArgsFor(filePath);
    await new Promise((resolve, reject) => {
        const cp = spawn(cmd, args, { stdio: 'inherit', cwd: process.cwd(), env: process.env });

        let settled = false;
        const timeoutMs = 30_000;

        const done = (err) => {
            if (settled) return;
            settled = true;
            clearTimeout(timer);
            if (err) {
                reject(err);
            } else {
                resolve();
            }
        };

        const timer = setTimeout(() => {
            try {
                cp.kill('SIGKILL');
            } catch {
                // ignore
            }
            const err = new Error(`Subprocess timed out after ${timeoutMs} ms`);
            err.code = 'ETIMEDOUT';
            done(err);
        }, timeoutMs);

        cp.on('error', (err) => done(err));
        cp.on('exit', (code, signal) => {
            if (signal) {
                done(new Error(`Subprocess terminated by signal ${signal}`));
            } else if (code && code !== 0) {
                const err = new Error(`Subprocess exited with code ${code}`);
                // @ts-expect-error attach code for upper-level handling
                err.code = code;
                done(err);
            } else {
                done();
            }
        });
    });
}

const entries = await readdir(selfDir, { withFileTypes: true });

const files = entries
    .filter((ent) => ent.isFile())
    .map((ent) => ent.name)
    .filter((name) => name !== selfBase)
    .filter((name) => exts.has(path.extname(name)))
    .sort((a, b) => a.localeCompare(b));

for (let i = 0; i < files.length; i++) {
    const file = files[i];
    const abs = path.join(selfDir, file);
    try {
        await runOne(abs);
        console.log(`[e2e] (${i + 1}/${files.length}) Passed ${file}`);
    } catch (err) {
        console.error(`[e2e] Failed: ${file}`);
        if (err && typeof err.code === 'number') {
            process.exit(err.code);
        } else {
            process.exit(1);
        }
    }
}

console.log('[e2e] All done.');
