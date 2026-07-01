# Verified task — issue #479

## What must be true
Response header values must decode correctly for the common modern case (UTF-8, e.g.
`Content-Disposition: filename="naïve.pdf"`) WITHOUT regressing the case PR #434 fixed
(non-ASCII latin-1 bytes such as `0xE4` = `ä`, which previously crashed Node / emptied Python).

## Corrected references (verified at HEAD 9d2204f)
- Node: `impit-node/src/response.rs:96`
- Python: `impit-python/src/response.rs:542` (issue said 544 — stale)
- Existing regression guard: `impit-node/test/basics.test.ts:569` + `impit-node/test/mock.server.ts:105-118`
- Shared helper candidate home: `impit/src/response_parsing/mod.rs`, re-exported via `impit::utils`

## Acceptance (rev 2 — per-ecosystem, confirmed with maintainer)
1. Python decodes headers httpx-style: UTF-8-first with ISO-8859-1 fallback, never crash/empty/
   `U+FFFD` (fixes #479 for Python; keeps #430/#434 guarantees).
2. JS decodes headers Fetch-style: strict ISO-8859-1 isomorphic decode (`b as char`), so string
   values stay byte-recoverable via `Buffer.from(v,'binary')`. JS UTF-8 mojibake is intentional.
3. Both bindings expose a raw-bytes accessor returning the exact header VALUE bytes (duplicate
   values preserved): Python `raw_headers: list[tuple[bytes,bytes]]`; JS
   `rawHeaders: Array<[string, Uint8Array]>` (impit extension). Caveat imposed by reqwest's
   `HeaderMap`: header names are normalized to lowercase and original cross-header wire order is
   NOT preserved — so this is httpx-`.raw`-*like*, not byte-identical. JS `rawHeaders` survives
   `clone()`.
4. Tests: Python UTF-8 decode + raw bytes exact; JS latin-1 decode retained + raw bytes exact.
5. #479 resolution documented as intentionally split (JS = Fetch parity + rawHeaders escape hatch).

## Now in scope (was previously deferred)
Raw header bytes accessor for HMAC/signature callers — included per maintainer decision.
