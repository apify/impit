import statistics
import sys
import time
import warnings

warnings.filterwarnings('ignore')

sys.argv = [sys.argv[0], 'impit', '--smoke']
_ns = {}
exec(compile(open('bench/bench_py.py').read().split('name = sys.argv[1]')[0], 'bench_py.py', 'exec'), _ns)

URL = _ns['URL']
N = 1000
ROUNDS = 15

clients = {}
for name, factory in _ns['FACTORIES'].items():
    fetch = factory()
    assert len(fetch()) > 900, name
    clients[name] = fetch

for fetch in clients.values():
    for _ in range(200):
        fetch()

samples = {name: [] for name in clients}
for r in range(ROUNDS):
    for name, fetch in clients.items():
        start = time.perf_counter()
        for _ in range(N):
            fetch()
        samples[name].append(N / (time.perf_counter() - start))
    print(f'round {r + 1}/{ROUNDS}', file=sys.stderr)

print('library\tbest\tmedian\tmean\tstdev')
for name, rs in sorted(samples.items(), key=lambda kv: -statistics.median(kv[1])):
    print(
        f'{name}\t{max(rs):.0f}\t{statistics.median(rs):.0f}\t{statistics.mean(rs):.0f}\t{statistics.stdev(rs):.0f}'
    )
