#!/bin/bash
cd "$(dirname "$0")/.."
for n in impit rnet primp tls-client curl_cffi httpx; do
    taskset -c 2 impit-python/.venv/bin/python bench/bench_py.py "$n" "$@" 2>&1 | tail -2
done
