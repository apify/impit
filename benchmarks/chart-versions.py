"""Renders the version-history throughput chart from bench-versions.mjs / bench_versions.py.

Reads results-node-versions.json, results-python-versions.json (the sync `Client`) and
results-python-async-versions.json (the async `AsyncClient`), and plots median req/s
against release date, one line per client. Node's `fetch()` and Python's `AsyncClient`
both bridge each call through an event loop; the sync `Client` doesn't - splitting
Python's two clients out shows how much of the npm/PyPI gap that bridge accounts for.
The PNG is a CI artifact, not a committed file - see ../.github/workflows/version-benchmark.yaml.
"""

from __future__ import annotations

import argparse
import json
from datetime import datetime
from pathlib import Path

import matplotlib

matplotlib.use('Agg')

import matplotlib.dates as mdates  # noqa: E402
import matplotlib.pyplot as plt  # noqa: E402
import matplotlib.ticker as mticker  # noqa: E402

HERE = Path(__file__).resolve().parent

INK = '#0b0b0b'
SECONDARY_INK = '#52514e'
MUTED = '#898781'
GRIDLINE = '#e1e0d9'
SURFACE = '#fcfcfb'
SERIES = {
    ('node', None): {'label': 'npm (Node.js)', 'color': '#2a78d6'},
    ('python', 'sync'): {'label': 'PyPI (Python, sync)', 'color': '#eb6834'},
    ('python', 'async'): {'label': 'PyPI (Python, async)', 'color': '#1baf7a'},
}


def load(path: Path) -> dict | None:
    if not path.exists():
        return None
    report = json.loads(path.read_text())
    if not report['results']:
        return None
    return report


def plot(reports: list[dict], out: Path) -> None:
    fig, ax = plt.subplots(figsize=(8, 4.5), dpi=200, facecolor=SURFACE)
    ax.set_facecolor(SURFACE)

    max_rate = max(point['rpsMedian'] for report in reports for point in report['results'])

    for report in reports:
        series = SERIES[(report['ecosystem'], report.get('variant'))]
        points = sorted(report['results'], key=lambda r: r['publishedAt'])
        dates = [datetime.fromisoformat(p['publishedAt'].replace('Z', '+00:00')) for p in points]
        rates = [p['rpsMedian'] for p in points]

        ax.plot(dates, rates, color=series['color'], linewidth=2, solid_capstyle='round',
                marker='o', markersize=8, markerfacecolor=series['color'],
                markeredgecolor=SURFACE, markeredgewidth=2, label=series['label'])

        for point, date, rate in zip(points, dates, rates):
            ax.annotate(point['version'], (date, rate), textcoords='offset points',
                        xytext=(0, 10), ha='center', fontsize=8, color=MUTED)

        last_date, last_rate = dates[-1], rates[-1]
        ax.annotate(f'{last_rate:,.0f} req/s', (last_date, last_rate), textcoords='offset points',
                    xytext=(10, -4), ha='left', fontsize=9, color=SECONDARY_INK, fontweight='bold')

    ax.set_title('impit throughput by release', fontsize=13, color=INK, loc='left', pad=14)
    ax.set_ylabel('req/s (median)', fontsize=10, color=SECONDARY_INK)
    ax.yaxis.set_major_formatter(mticker.FuncFormatter(lambda value, _: f'{value:,.0f}'))
    # Headroom above the highest point so its label never collides with the legend.
    ax.set_ylim(0, max_rate * 1.3)

    ax.xaxis.set_major_locator(mdates.AutoDateLocator(minticks=3, maxticks=6))
    ax.xaxis.set_major_formatter(mdates.ConciseDateFormatter(ax.xaxis.get_major_locator()))
    fig.autofmt_xdate(rotation=0, ha='center')

    ax.grid(axis='y', color=GRIDLINE, linewidth=1)
    ax.set_axisbelow(True)
    for spine in ('top', 'right', 'left'):
        ax.spines[spine].set_visible(False)
    ax.spines['bottom'].set_color('#c3c2b7')
    ax.tick_params(axis='both', colors=MUTED, labelsize=9, length=0)

    legend = ax.legend(loc='upper left', frameon=False, fontsize=9, labelcolor=SECONDARY_INK)
    legend.set_zorder(10)

    fig.tight_layout()
    fig.savefig(out, facecolor=SURFACE)
    plt.close(fig)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--node', type=Path, default=HERE / 'results-node-versions.json')
    parser.add_argument('--python', type=Path, default=HERE / 'results-python-versions.json')
    parser.add_argument('--python-async', type=Path,
                         default=HERE / 'results-python-async-versions.json')
    parser.add_argument('--out', type=Path, default=HERE / 'version-chart.png')
    args = parser.parse_args()

    reports = [report for report in (load(args.node), load(args.python), load(args.python_async))
               if report is not None]
    if not reports:
        raise SystemExit('neither results file has any results; nothing to chart')

    plot(reports, args.out)
    print(f'wrote {args.out}')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
