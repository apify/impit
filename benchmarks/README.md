# Comparison benchmark

Produces the comparison tables in the root [`README.md`](../README.md). Everything here measures the
**latest published** release of each client, `impit` included — nothing is pinned and no lockfile is
committed, so a run always reflects what the ecosystems ship today.

## Running it

```bash
npm install --prefix node --no-audit --no-fund
uv venv --seed --python 3.12 python/.venv
uv pip install --python python/.venv/bin/python -r python/requirements.txt

node node/bench.mjs                       # writes results-node.json
python/.venv/bin/python python/bench.py   # writes results-python.json
node update-readme.mjs                    # rewrites the table in ../README.md
```

`--requests`, `--runs` and `--warmup` shrink a run while iterating, and `--only impit,undici` limits
it to a few clients. `update-readme.mjs` refuses to run when the two reports disagree on the
parameters, so pass the same values to both. `uv venv --seed` matters: `bench.py` shells out to `pip
download` to size each wheel.

## What is measured

[`server.mjs`](server.mjs) is the origin: a Node.js `http2` server on a self-signed certificate,
serving a fixed 1 KiB JSON body over `h2` or `http/1.1`, whichever the client negotiates. Each
client then issues sequential requests over one connection; the best of N runs is reported, which
absorbs scheduler noise without flattering a client that is genuinely slow. Sequential single-client
traffic is deliberate — it isolates per-request client overhead, which is what differs between these
libraries, rather than measuring how well each one saturates a socket.

The server also reports its connection count at `/__stats`, and both scripts record how many
connections a client opened while being measured. That is what surfaces `cycletls` handshaking on
every request rather than reusing a warm connection; without it the number would look like plain
per-request overhead.

The `Profiles` column counts the distinct impersonation targets each public API accepts, minus
aliases that merely resolve to another target. There is no uniform way to ask for that, so every
client has its own small accessor in the two `bench.py`/`bench.mjs` client tables; where a library
does not expose the set at all, the count is dropped and a footnote says why.

Sizes are the artifact each ecosystem actually distributes: for Python the platform wheel that `pip
download` picks, for Node.js what a fresh `npm install <package>` leaves on disk with its transitive
dependencies.

## Notes on the origin

Node's HTTP/2 Rapid-Reset mitigation is switched off in `server.mjs`. Clients that `RST_STREAM` each
response once they have read it — got's `http2-wrapper` does — otherwise exhaust the default budget
of 1000 resets and are hit with a `GOAWAY` a thousand requests into a run. The mitigation is correct
for a public origin and wrong here, where the server must never be the thing that rate-limits the
client.

## Adding a client

Add an entry to `CLIENTS` in the relevant script: how to build it, how to issue one request, how to
count its profiles, and a `note` if either the profile count or the throughput figure needs a caveat.
`update-readme.mjs` picks up the rest — ordering, footnote numbering and the caption — from the two
result files.
