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

## Acceptance
1. A header whose bytes are valid UTF-8 decodes as UTF-8 (fixes #479 mojibake).
2. A header with invalid-UTF-8 latin-1 bytes still decodes byte-for-byte as latin-1 (keeps #434).
3. No `U+FFFD` replacement chars introduced (keeps #430 non-crash / non-empty).
4. Applied symmetrically in Node and Python bindings.
5. Regression test present for the UTF-8 case (at minimum in Node, which has the existing suite).

## Explicitly out of scope (note as follow-up)
Exposing raw header bytes for HMAC/signature callers — larger API addition, separate issue.
