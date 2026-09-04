# Comparison benchmark

Generates the comparison tables in the root [`README.md`](../README.md). Nothing is pinned and no
lockfile is committed, so every run measures the latest published release of each client, `impit`
included.

## Running it

```bash
npm install --prefix node --no-audit --no-fund
uv venv --seed --python 3.12 python/.venv
uv pip install --python python/.venv/bin/python -r python/requirements.txt

node node/bench.mjs                       # writes results-node.json
python/.venv/bin/python python/bench.py   # writes results-python.json
node update-readme.mjs                    # rewrites the table in ../README.md
```

`--requests`, `--runs` and `--warmup` shrink a run while iterating, `--body-bytes` changes the
response size, and `--only impit,undici` limits it to a few clients. Pass the same values to both
scripts — `update-readme.mjs` rejects reports taken with different parameters. `uv venv --seed`
matters, because `bench.py` shells out to `pip download` to size each wheel.

Run the two scripts one after the other, never in parallel: they compete for the same cores and both
sets of numbers come out low.

## What is measured

[`server.mjs`](server.mjs) is the origin — a Node.js `http2` server on a self-signed certificate
serving a 1 KiB JSON body. Each client then issues sequential requests over one connection, N
times, and the median run is reported. Sequential single-client traffic isolates per-request client
overhead, which is what differs between these libraries.

The result files also carry the best and worst run, the negotiated protocol, and how many
connections the client opened while being measured. That last one is worth checking when a number
looks off: `cycletls` opens a fresh connection per request, so its figure includes a handshake every
time.

`Profiles` counts the impersonation targets each public API accepts, minus aliases that resolve to
another target. There is no uniform way to ask for that, so each client has its own accessor in the
`CLIENTS` table.

Sizes are what each ecosystem distributes: the platform wheel for Python, and for Node.js whatever a
fresh `npm install` leaves on disk with its transitive dependencies.

## Notes on the origin

Node's HTTP/2 Rapid-Reset mitigation is off in `server.mjs`. Clients that `RST_STREAM` each response
once they have read it — got's `http2-wrapper` does — otherwise burn the default budget of 1000
resets and take a `GOAWAY` mid-run. Right for a public origin, wrong for a benchmark.

## Adding a client

Add an entry to `CLIENTS`: how to build it, how to issue one request, and how to count its profiles.
`update-readme.mjs` takes care of ordering and the caption.
