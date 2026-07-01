# Design (rev 2) — fix #479: per-ecosystem header decoding + raw-bytes accessor

## What we're solving
Header values are decoded as ISO-8859-1 (`b as char`), garbling UTF-8 headers (#479). Rather
than force one behavior on both bindings, we make each binding faithful to the reference client
it already claims to implement, and give callers who need exact bytes (HMAC/signature checks) a
raw accessor — mirroring what httpx already offers.

Decision (confirmed with maintainer): **behave like httpx in Python, like Fetch in JS**, and
**add a raw-header-bytes accessor to both**.

## How
**Decoding (the split):**
- **Python = httpx semantics.** Decode UTF-8-first with an ISO-8859-1 fallback (httpx tries
  ascii→utf-8→iso-8859-1). This is the shared `decode_header_value` helper we already built.
- **JS = Fetch semantics.** Keep strict ISO-8859-1 isomorphic decode (`b as char`, i.e. PR
  #434's behavior). This means impit-node's string headers stay byte-recoverable via the standard
  `Buffer.from(v, 'binary')` idiom, matching `fetch()`/undici/axios. Revert the JS call site to
  `b as char` and drop the JS "UTF-8 header" string test.

**Raw-bytes accessor (new public API, both bindings):**
- **Python** — mirror httpx's `Response.headers.raw`: expose `raw_headers` on the response as
  `list[tuple[bytes, bytes]]` (name, value), preserving order and duplicates (reqwest yields
  repeated headers separately). This closes a real httpx-compat gap.
- **JS** — no Fetch precedent, so this is an explicit impit extension: expose `rawHeaders` on the
  response as `Array<[string, Uint8Array]>` (name as string per Fetch conventions, value as raw
  bytes), same order/duplicate semantics. Justified because HMAC callers need exact bytes and
  latin-1 strings, while recoverable, are error-prone to reverse by hand.

Both accessors return the untouched wire bytes, so a signature/HMAC caller never depends on any
string decoding.

## Alternatives + the call
- **Symmetric UTF-8-first in both (previous rev):** rejected per maintainer — deviates from
  strict Fetch on JS and breaks the `Buffer.from(v,'binary')` recovery idiom.
- **Latin-1 everywhere + raw bytes only:** rejected — leaves Python worse than httpx.
- **Skip the raw accessor:** rejected — HMAC/signature callers have no correct alternative once
  decoding is lossy (distinct byte sequences can decode to the same string).
- **Chosen:** per-ecosystem decode + raw accessor in both.

## Major changes (key areas)
- Core crate: keep `decode_header_value` (UTF-8-first); Python consumes it, JS does not.
- `impit-python/src/response.rs`: keep helper for the string dict; add `raw_headers` getter
  returning byte-pair tuples.
- `impit-node/src/response.rs`: revert string decode to `b as char`; add `rawHeaders` accessor
  returning name/`Uint8Array` pairs; update the `.d.ts`/napi surface.
- Tests: Python — UTF-8 decodes correctly + `raw_headers` returns exact bytes. JS — existing
  latin-1 test stays; add a `rawHeaders` test asserting exact wire bytes (and that string decode
  remains latin-1). Core — existing `decode_header_value` unit tests unchanged.
- Docs: note the intentional Python/JS decoding difference; note JS `rawHeaders` is an impit
  extension beyond Fetch.

## Risks / open questions
- **#479 becomes a partial fix by design:** JS UTF-8 headers stay latin-1 (mojibake) on the
  string API; the fix for JS callers is `rawHeaders` + their own decode, matching Fetch. The
  issue/PR must state this explicitly so it isn't read as "not fixed."
- **New public API surface in both bindings** — naming (`raw_headers` / `rawHeaders`), return
  shapes, and multi-value/duplicate semantics are the things to lock at this gate.
- **Build/oracle limit unchanged:** full workspace + napi/maturin can't build here
  (github.com/apify/h2 egress 403). Core helper is oracle-tested via standalone rustc; the new
  accessors' binding compilation + JS/Py tests must be verified in CI. The `rawHeaders` napi/pyo3
  wiring in particular is only compile-checkable in CI.
