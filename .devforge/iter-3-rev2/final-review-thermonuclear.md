VERDICT: FAIL

## Findings

### Major — stale "in wire order" comments contradict the corrected public docstrings (docs accuracy regression not fully fixed)

The rev-2 fix correctly rewrote the *public* docstrings for both `rawHeaders` (Node) and `raw_headers`
(Python) to state that header names are lowercased and the original wire order is **not** preserved
(only duplicate values for a given name are preserved). However, the private field comments sitting
right next to the struct definitions in the same files were left unchanged and still assert the
opposite:

- `impit-node/src/response.rs:65`
  ```rust
  // Raw, undecoded header name/value byte pairs, in wire order (duplicates preserved).
  ```
  This directly contradicts the getter's own doc comment 70 lines below it
  (`impit-node/src/response.rs:138`): *"the original wire order is not preserved."*

- `impit-python/src/response.rs:230`
  ```rust
  // Raw, undecoded header name/value byte pairs, in wire order (duplicates preserved).
  ```
  Same contradiction against `impit-python/src/response.rs:458`: *"the original wire order is not
  preserved."*

Concrete scenario: a maintainer skimming the struct definition (the first thing you read when
opening the file) sees "in wire order" and walks away with the exact overstated/incorrect claim the
prior review round flagged and that the design doc explicitly calls out as a reqwest `HeaderMap`
caveat. The public-facing docs were fixed, but the adjacent internal comments were not brought into
sync, leaving self-contradictory documentation in both `impit-node/src/response.rs` and
`impit-python/src/response.rs`. This is exactly the kind of leftover overstated claim the re-review
was asked to confirm is gone — it is not gone, it just moved one comment upward.

Fix: reword both field comments to match the getter docs, e.g. "Raw, undecoded header name/value
byte pairs (names lowercased, cross-header wire order not preserved by the underlying `HeaderMap`;
duplicate values for the same name are preserved in insertion order)."

## Verified as correctly fixed (no findings)

- `impit-node/index.d.ts:157` now declares `get rawHeaders(): Array<[string, Uint8Array]>` on
  `ImpitResponse`, in the same getter style as `get body()`. The main test's direct access
  (`impit-node/test/basics.test.ts:582`, `response.rawHeaders.find(...)`) no longer casts.
- `impit-node/index.wrapper.js:483-487` propagates `rawHeaders` onto the `clone()`-created plain
  `Response` object via `Object.defineProperty(clone, 'rawHeaders', { value: this.rawHeaders, ... })`,
  reading from the original native `ImpitResponse` (`this` inside the `clone` function is
  `originalResponse`, the patched native instance) — correct source of truth. The test
  (`impit-node/test/basics.test.ts:591-594`) asserts the raw bytes survive `clone()` and match the
  pre-clone bytes.
  - Note: `clone()` is declared as `clone(): Response` (`impit-node/index.d.ts:250`), the global DOM
    `Response` type, which has no `rawHeaders` member — so the test's
    `response.clone() as unknown as { rawHeaders: ... }` cast at
    `impit-node/test/basics.test.ts:591` is still present. This is a *different*, pre-existing typing
    gap (the clone's declared return type was always the plain DOM `Response`, unrelated to this
    fix) rather than a recurrence of the originally flagged issue, which was about accessing
    `rawHeaders` directly on `ImpitResponse` before the getter existed. Not treated as a blocker since
    the acceptance criteria only requires runtime survival across `clone()`, not a fully-typed clone
    contract — but flagged here for visibility in case the intent was to eliminate all casts.
- Docstrings in `impit-node/index.d.ts:147-156`, `impit-node/src/response.rs:125-140`, and
  `impit-python/src/response.rs:456-462` now accurately state: names lowercased, wire order not
  preserved (reqwest `HeaderMap` caveat), duplicate values preserved, value bytes exact. Matches
  `2-design.md`'s caveat section and `_verified_task.md` acceptance item 3.
- Header-pair construction (`impit-node/src/response.rs:99-106`, `impit-python/src/response.rs:574-579`)
  correctly derives both the decoded-string headers and the raw byte pairs from the same
  `response.headers().iter()` pass, so the "duplicates preserved, values exact" guarantee actually
  holds against `HeaderMap::iter()` semantics — not just claimed.
- No new maintainability/structural issue introduced: the raw-header plumbing is a small, additive
  field + getter in each binding, reusing the existing `decode_header_value` core helper where
  applicable; no file crossed a size threshold, no new branching was bolted onto unrelated code
  paths, and no bespoke duplicate-of-an-existing-helper was added.
- No concrete compile hazard spotted in the new Rust code (napi `Uint8Array` import, `getter`/
  `js_name`/`ts_return_type` attributes, and pyo3 `PyBytes` construction all follow existing patterns
  already used elsewhere in the same files).
