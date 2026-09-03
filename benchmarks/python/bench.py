"""Throughput comparison of Python browser-impersonation HTTP clients.

Runs every client against the shared local HTTP/2 origin in ../server.mjs and
writes ../results-python.json. See ../README.md for how the numbers are taken.
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import re
import shutil
import subprocess
import sys
import tempfile
import time
import typing
from collections.abc import Callable, Iterable
from dataclasses import dataclass
from datetime import datetime, timezone
from importlib.metadata import version as installed_version
from pathlib import Path
from typing import Any

HERE = Path(__file__).resolve().parent
SERVER = HERE.parent / 'server.mjs'

# Node's `os.arch()` vocabulary, so both reports name the same machine the same way.
ARCH_ALIASES = {'x86_64': 'x64', 'AMD64': 'x64', 'aarch64': 'arm64'}

CHROME_PROFILES = {
    'impit': 'chrome',
    'primp': 'chrome_146',
    'tls_client': 'chrome_131',
    'curl_cffi': 'chrome',
}


@dataclass
class Client:
    key: str
    """Distribution name on PyPI."""
    label: str
    backend: str
    setup: Callable[[str], tuple[Callable[[], tuple[bytes, str | None]], Callable[[], None] | None]]
    profiles: Callable[[], int | None]
    repo: str | None = None
    baseline: bool = False
    profiles_label: str | None = None
    """Shown in the Profiles cell when there is no set to count."""


def _versioned(names: Iterable[str]) -> list[str]:
    """Drop unversioned aliases such as impit's `chrome`, which means "newest Chrome"."""
    return [name for name in names if re.search(r'\d', name)]


def impit_profiles() -> int:
    import impit

    return len(_versioned(typing.get_args(impit.Browser)))


def rnet_profiles() -> int:
    import rnet

    return len([name for name in dir(rnet.Impersonate) if not name.startswith('_')])


def tls_client_profiles() -> int:
    from tls_client.settings import ClientIdentifiers

    # Every identifier is a distinct target; the unversioned ones are app
    # fingerprints (nike, zalando, ...), not aliases.
    return len(typing.get_args(ClientIdentifiers))


def curl_cffi_profiles() -> int:
    from curl_cffi.requests import impersonate as ci

    names = set(typing.get_args(ci.BrowserTypeLiteral))
    # REAL_TARGET_MAP holds the "newest of this family" aliases.
    names -= set(ci.REAL_TARGET_MAP)
    # curl_cffi also kept the pre-rename spellings ("safari18_0") alongside the
    # current ones ("safari180"); un-underscoring the version reaches the target.
    deduped = {re.sub(r'(\d)_(\d)', r'\1\2', name) for name in names}
    return len(names & deduped)


def setup_impit(url: str):
    import impit

    client = impit.Client(browser=CHROME_PROFILES['impit'], verify=False)

    def request():
        response = client.get(url)
        return response.content, response.headers.get('x-alpn')

    return request, None


def setup_rnet(url: str):
    import rnet

    # rnet has no unversioned alias, so pick the newest Chrome it ships.
    newest_chrome = max(
        (name for name in dir(rnet.Impersonate) if name.startswith('Chrome')),
        key=lambda name: int(name.removeprefix('Chrome')),
    )
    client = rnet.BlockingClient(impersonate=getattr(rnet.Impersonate, newest_chrome), verify=False)

    def request():
        response = client.get(url)
        alpn = response.headers.get('x-alpn')
        return response.bytes(), alpn.decode() if isinstance(alpn, bytes) else alpn

    return request, None


def setup_primp(url: str):
    import primp

    client = primp.Client(impersonate=CHROME_PROFILES['primp'], verify=False)
    if client.impersonate != CHROME_PROFILES['primp']:
        raise RuntimeError(f'primp fell back to {client.impersonate!r}; the profile name needs updating')

    def request():
        response = client.get(url)
        return response.content, response.headers.get('x-alpn')

    return request, None


def setup_tls_client(url: str):
    import tls_client

    session = tls_client.Session(client_identifier=CHROME_PROFILES['tls_client'])

    def request():
        response = session.get(url, insecure_skip_verify=True)
        return response.content, response.headers.get('X-Alpn')

    return request, session.close


def setup_curl_cffi(url: str):
    from curl_cffi import requests

    session = requests.Session(impersonate=CHROME_PROFILES['curl_cffi'], verify=False)

    def request():
        response = session.get(url)
        return response.content, response.headers.get('x-alpn')

    return request, session.close


def setup_httpx(url: str):
    import httpx

    client = httpx.Client(verify=False, http2=True)

    def request():
        response = client.get(url)
        return response.content, response.headers.get('x-alpn')

    return request, client.close


CLIENTS = [
    Client(
        key='rnet',
        label='rnet',
        repo='https://github.com/0x676e67/rnet',
        backend='Rust',
        setup=setup_rnet,
        profiles=rnet_profiles,
    ),
    Client(
        key='primp',
        label='primp',
        repo='https://github.com/deedy5/primp',
        backend='Rust',
        setup=setup_primp,
        # primp does not expose its profile list, and an unknown name falls back to
        # a random profile rather than erroring, so there is nothing to count.
        profiles=lambda: None,
        profiles_label='n/a',
    ),
    Client(
        key='impit',
        label='impit',
        backend='Rust',
        setup=setup_impit,
        profiles=impit_profiles,
    ),
    Client(
        key='tls-client',
        label='tls-client',
        repo='https://github.com/FlorianREGAZ/Python-Tls-Client',
        backend='Go',
        setup=setup_tls_client,
        profiles=tls_client_profiles,
    ),
    Client(
        key='curl_cffi',
        label='curl_cffi',
        repo='https://github.com/lexiforest/curl_cffi',
        backend='C (libcurl)',
        setup=setup_curl_cffi,
        profiles=curl_cffi_profiles,
    ),
    Client(
        key='httpx',
        label='httpx',
        backend='Python',
        baseline=True,
        setup=setup_httpx,
        profiles=lambda: None,
    ),
]


def start_server(body_bytes: int) -> tuple[subprocess.Popen, str]:
    node = shutil.which('node')
    if node is None:
        raise RuntimeError('node is needed to run the benchmark origin server')
    process = subprocess.Popen(
        [node, str(SERVER)],
        env={**os.environ, 'PORT': '0', 'BODY_BYTES': str(body_bytes)},
        stdout=subprocess.PIPE,
        text=True,
    )
    url = process.stdout.readline().strip()
    if not url:
        process.kill()
        raise RuntimeError('the origin server exited before it printed its URL')
    return process, url


def wheel_size(pkg: str, version: str) -> int:
    """Size of the wheel pip installs for this interpreter and platform."""
    with tempfile.TemporaryDirectory() as target:
        subprocess.run(
            [sys.executable, '-m', 'pip', 'download', '--no-deps', '--only-binary', ':all:',
             '--quiet', '--dest', target, f'{pkg}=={version}'],
            check=True,
            capture_output=True,
        )
        wheels = list(Path(target).glob('*.whl'))
        if len(wheels) != 1:
            raise RuntimeError(f'expected one wheel for {pkg}, got {[w.name for w in wheels]}')
        return wheels[0].stat().st_size


def measure(request, *, requests: int, runs: int, warmup: int) -> dict[str, float]:
    """Best of `runs` batches of `requests` sequential calls; see ../harness.mjs for the rationale."""
    for _ in range(warmup):
        request()

    rates = []
    for _ in range(runs):
        started = time.perf_counter()
        for _ in range(requests):
            request()
        rates.append(requests / (time.perf_counter() - started))
    rates.sort()
    return {'rps': rates[-1], 'rpsMedian': rates[len(rates) // 2], 'rpsWorst': rates[0]}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--requests', type=int, default=2000)
    parser.add_argument('--runs', type=int, default=11)
    parser.add_argument('--warmup', type=int, default=200)
    parser.add_argument('--body-bytes', type=int, default=1024)
    parser.add_argument('--out', type=Path, default=HERE.parent / 'results-python.json')
    parser.add_argument('--only', default='')
    args = parser.parse_args()

    selected = [c for c in CLIENTS if not args.only or c.key in args.only.split(',')]
    if not selected:
        raise SystemExit(f'--only matched no client: {args.only}')

    import httpx

    process, url = start_server(args.body_bytes)
    stats_client = httpx.Client(verify=False)
    read_stats = lambda: stats_client.get(f'{url}__stats').json()  # noqa: E731
    read_stats()

    results: list[dict[str, Any]] = []
    failures: list[str] = []
    try:
        for client in selected:
            print(f'{client.key}: ', end='', flush=True, file=sys.stderr)
            teardown = None
            try:
                request, teardown = client.setup(url)
                body, alpn = request()
                if len(body) != args.body_bytes:
                    raise RuntimeError(f'expected a {args.body_bytes} byte body, got {len(body)}')

                before = read_stats()
                timings = measure(request, requests=args.requests, runs=args.runs, warmup=args.warmup)
                after = read_stats()

                version = installed_version(client.key)
                results.append({
                    'key': client.key,
                    'label': client.label,
                    'repo': client.repo,
                    'backend': client.backend,
                    'baseline': client.baseline,
                    'version': version,
                    'alpn': alpn,
                    'profiles': client.profiles(),
                    'profilesLabel': client.profiles_label,
                    'sizeBytes': wheel_size(client.key, version),
                    'connections': after['connections'] - before['connections'],
                    **timings,
                })
                print(
                    f'{results[-1]["rps"]:.0f} req/s over {alpn}, '
                    f'{results[-1]["connections"]} connection(s)',
                    file=sys.stderr,
                )
            except Exception as exc:  # noqa: BLE001
                failures.append(f'{client.key}: {exc}')
                print(f'FAILED ({exc})', file=sys.stderr)
            finally:
                if teardown is not None:
                    teardown()
    finally:
        stats_client.close()
        process.kill()

    args.out.write_text(json.dumps({
        'ecosystem': 'python',
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

    print(f'wrote {args.out}', file=sys.stderr)
    if failures:
        print(f'{len(failures)} client(s) failed:', *failures, sep='\n', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
