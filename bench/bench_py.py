import sys
import time
import warnings

warnings.filterwarnings('ignore')

URL = 'https://127.0.0.1:8443/'
N = 2000
RUNS = 11


def make_impit():
    from impit import Client

    c = Client(browser='chrome', verify=False)
    return lambda: c.get(URL).content


def make_rnet():
    import rnet

    c = rnet.BlockingClient(emulation=rnet.Impersonate.Chrome136, verify=False)
    return lambda: c.get(URL).bytes()


def make_primp():
    import primp

    c = primp.Client(impersonate='chrome_146', verify=False)
    return lambda: c.get(URL).content


def make_tls_client():
    import tls_client

    c = tls_client.Session(client_identifier='chrome_120', random_tls_extension_order=True)
    return lambda: c.get(URL, insecure_skip_verify=True).content


def make_curl_cffi():
    from curl_cffi import requests

    c = requests.Session(impersonate='chrome', verify=False)
    return lambda: c.get(URL).content


def make_httpx():
    import httpx

    c = httpx.Client(verify=False, http2=True)
    return lambda: c.get(URL).content


FACTORIES = {
    'impit': make_impit,
    'rnet': make_rnet,
    'primp': make_primp,
    'tls-client': make_tls_client,
    'curl_cffi': make_curl_cffi,
    'httpx': make_httpx,
}

name = sys.argv[1]
fetch = FACTORIES[name]()

body = fetch()
assert len(body) > 900, f'{name}: unexpected body {body[:200]!r}'

if '--smoke' in sys.argv:
    print(f'{name} ok {len(body)}')
    raise SystemExit(0)


def run():
    start = time.perf_counter()
    for _ in range(N):
        fetch()
    return N / (time.perf_counter() - start)


run()
results = [run() for _ in range(RUNS)]
print(' '.join(f'{r:.0f}' for r in results), file=sys.stderr)
print(f'{name}\t{max(results):.0f}\t{sorted(results)[RUNS // 2]:.0f}')
