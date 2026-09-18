"""Throughput of the last N published PyPI releases of impit, against each other.

Unlike bench.py, which compares impit to other clients, this pip installs several
versions of impit itself into isolated dirs and benchmarks them in turn. Both the
sync `Client` and the async `AsyncClient` are measured, so the version chart can
show the cost of bridging each call through an asyncio event loop - the same kind
of bridge the Node.js binding's Promise-returning `fetch()` pays on every call. See
../README.md for how the numbers are taken.
"""

from __future__ import annotations

import argparse
import asyncio
import json
import platform
import re
import subprocess
import sys
import tempfile
import time
import urllib.request
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from bench import ARCH_ALIASES, measure, start_server

HERE = Path(__file__).resolve().parent


def recent_versions(pkg: str, count: int) -> list[dict[str, str]]:
    with urllib.request.urlopen(f'https://pypi.org/pypi/{pkg}/json') as response:  # noqa: S310
        manifest = json.load(response)

    entries = []
    for version, files in manifest['releases'].items():
        # Stable releases only, no `0.9.0rc1`-style prereleases.
        if not re.fullmatch(r'\d+\.\d+\.\d+', version):
            continue
        upload_times = [f['upload_time_iso_8601'] for f in files if not f.get('yanked')]
        if upload_times:
            entries.append((version, min(upload_times)))

    entries.sort(key=lambda entry: entry[1])
    return [{'version': version, 'publishedAt': published} for version, published in entries[-count:]]


def _purge_impit_modules() -> None:
    for name in [name for name in sys.modules if name == 'impit' or name.startswith('impit.')]:
        del sys.modules[name]


async def measure_async(request, *, requests: int, runs: int, warmup: int) -> dict[str, float]:
    """Async twin of bench.measure: same batches-of-sequential-awaits algorithm."""
    for _ in range(warmup):
        await request()

    rates = []
    for _ in range(runs):
        started = time.perf_counter()
        for _ in range(requests):
            await request()
        rates.append(requests / (time.perf_counter() - started))
    rates.sort()
    return {'rps': rates[-1], 'rpsMedian': rates[len(rates) // 2], 'rpsWorst': rates[0]}


def benchmark_version(entry: dict[str, str], url: str, *, requests: int, runs: int, warmup: int,
                       body_bytes: int) -> dict[str, dict[str, Any]]:
    """Measures both impit.Client (sync) and impit.AsyncClient for one installed version."""
    version = entry['version']
    with tempfile.TemporaryDirectory() as target:
        subprocess.run(
            [sys.executable, '-m', 'pip', 'install', '--quiet', '--disable-pip-version-check',
             '--target', target, f'impit=={version}'],
            check=True,
        )

        sys.path.insert(0, target)
        try:
            _purge_impit_modules()
            import impit  # noqa: PLC0415

            client = impit.Client(browser='chrome', verify=False)

            def request() -> tuple[bytes, str | None]:
                response = client.get(url)
                return response.content, response.headers.get('x-alpn')

            body, alpn = request()
            if len(body) != body_bytes:
                raise RuntimeError(f'expected a {body_bytes} byte body, got {len(body)}')

            sync_timings = measure(request, requests=requests, runs=runs, warmup=warmup)

            async def run_async_client() -> dict[str, Any]:
                async_client = impit.AsyncClient(browser='chrome', verify=False)

                async def async_request() -> tuple[bytes, str | None]:
                    response = await async_client.get(url)
                    return response.content, response.headers.get('x-alpn')

                async_body, async_alpn = await async_request()
                if len(async_body) != body_bytes:
                    raise RuntimeError(f'expected a {body_bytes} byte body, got {len(async_body)}')

                async_timings = await measure_async(async_request, requests=requests, runs=runs, warmup=warmup)
                return {**entry, 'alpn': async_alpn, **async_timings}

            return {
                'sync': {**entry, 'alpn': alpn, **sync_timings},
                'async': asyncio.run(run_async_client()),
            }
        finally:
            _purge_impit_modules()
            sys.path.remove(target)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--versions', type=int, default=5)
    parser.add_argument('--requests', type=int, default=2000)
    parser.add_argument('--runs', type=int, default=11)
    parser.add_argument('--warmup', type=int, default=200)
    parser.add_argument('--body-bytes', type=int, default=1024)
    parser.add_argument('--out', type=Path, default=HERE.parent / 'results-python-versions.json')
    parser.add_argument('--out-async', type=Path,
                         default=HERE.parent / 'results-python-async-versions.json')
    args = parser.parse_args()

    versions = recent_versions('impit', args.versions)
    process, url = start_server(args.body_bytes)

    sync_results: list[dict[str, Any]] = []
    async_results: list[dict[str, Any]] = []
    failures: list[str] = []
    try:
        for entry in versions:
            print(f'impit=={entry["version"]}: ', end='', flush=True, file=sys.stderr)
            try:
                timings = benchmark_version(
                    entry, url, requests=args.requests, runs=args.runs, warmup=args.warmup,
                    body_bytes=args.body_bytes,
                )
                sync_results.append(timings['sync'])
                async_results.append(timings['async'])
                print(f'{timings["sync"]["rpsMedian"]:.0f} req/s sync, '
                      f'{timings["async"]["rpsMedian"]:.0f} req/s async', file=sys.stderr)
            except Exception as exc:  # noqa: BLE001
                failures.append(f'{entry["version"]}: {exc}')
                print(f'FAILED ({exc})', file=sys.stderr)
    finally:
        process.kill()

    def write_report(out: Path, variant: str, results: list[dict[str, Any]]) -> None:
        out.write_text(json.dumps({
            'ecosystem': 'python',
            'variant': variant,
            'package': 'impit',
            'runtime': f'CPython {platform.python_version()}',
            'platform': f'{sys.platform}-{ARCH_ALIASES.get(platform.machine(), platform.machine())}',
            'measuredAt': datetime.now(timezone.utc).isoformat(),
            'options': {
                'requests': args.requests,
                'runs': args.runs,
                'warmup': args.warmup,
                'bodyBytes': args.body_bytes,
            },
            'results': results,
        }, indent=2) + '\n')
        print(f'wrote {out}', file=sys.stderr)

    write_report(args.out, 'sync', sync_results)
    write_report(args.out_async, 'async', async_results)

    if failures:
        print(f'{len(failures)} version(s) failed:', *failures, sep='\n', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
