VERDICT: PASS

## Verification of prior findings

### Finding 1 — JS `rawHeaders` dropped after `clone()` — RESOLVED
`impit-node/index.wrapper.js` (`clone()`, ~line 483-487) now does:
```js
Object.defineProperty(clone, 'rawHeaders', {
    value: this.rawHeaders,
    enumerable: true,
});
```
`this` here is the native `ImpitResponse` (`originalResponse`), so `this.rawHeaders` invokes the
napi getter (`impit-node/src/response.rs:141-152`) and copies the resulting `Array<[string,
Uint8Array]>` onto the clone as a static value — correct, since `rawHeaders` is plain data, not a
stream that can be double-consumed. Covered by a new test in
`impit-node/test/basics.test.ts:574-595` (`'raw header bytes preserve the exact wire value while
the string stays ISO-8859-1 (Fetch-style)'`), which explicitly clones the response and asserts
`cloned.rawHeaders` still contains the exact bytes. Confirmed rustfmt-clean on
`impit-node/src/response.rs`.

### Finding 2 — `HeaderMap::iter()` order claim — RESOLVED
All public-facing wording now correctly states the wire order is NOT preserved (removed the prior
overclaim of "in wire order"):
- `impit-node/index.d.ts:153-155` (`rawHeaders` getter doc)
- `impit-node/src/response.rs:138-140` (napi getter `///` doc)
- `impit-python/src/response.rs:456-459` (`raw_headers` `///` doc, "note two differences... the
  original wire order is not preserved")
- `.devforge/2-design.md:30-34`, `.devforge/_verified_task.md:20-24`

Verified against the actual `http` crate (v1.4.2, vendored at
`/root/.cargo/registry/src/.../http-1.4.2/src/header/map.rs:943`): `HeaderMap::iter()` docs state
"The iterator order is arbitrary, but consistent across platforms for the same crate version" —
i.e., not wire order and not even a stable/documented order guarantee beyond same-key insertion
order for duplicates. The corrected docs ("wire order not preserved") are accurate (if anything,
conservative, since the map's overall order isn't even loosely tied to wire order).

### Finding 3 — Python `raw_headers` lowercases names vs. httpx `.raw` — RESOLVED
`impit-python/src/response.rs:456-459` no longer claims unqualified "httpx `Response.headers.raw`
equivalent." It now reads: "Similar to httpx's `Response.headers.raw`, but note two differences
imposed by the underlying HTTP client: header names are normalized to lowercase and the original
wire order is not preserved... Header *values* are the exact bytes received." This is accurate:
`k.as_str()` on an `http::HeaderName` always returns the lowercased form (confirmed prior round),
and the new doc no longer implies name-case parity. `.devforge/pr-ecosystem-section.md:22-27` and
`2-design.md` state the same caveat consistently. New test coverage:
`impit-python/test/async_client_test.py` (`test_header_value_decoding_and_raw_bytes`) asserts
`raw[b'x-utf8']` / `raw[b'x-latin1']` using lowercase keys, consistent with actual behavior; ruff
check passes clean on the touched test files. The unrelated constructor path
(`ImpitPyResponse::new`, used by `impit-python/test/response_test.py::test_response_raw_headers`)
builds `raw_headers` directly from the caller-supplied Python dict (no `HeaderMap` involved), so
it correctly preserves `Content-Type` casing there — consistent with the getter doc, which only
promises byte-exact values and flags the lowercasing caveat as a limitation "imposed by the
underlying HTTP client" (i.e., only applies to responses that actually went through reqwest).

## Additional checks performed
- `rustfmt --check` on all touched Rust files (`impit-node/src/response.rs`,
  `impit-python/src/response.rs`, `impit/src/response_parsing/mod.rs`, `impit/src/lib.rs`): clean.
- Standalone `rustc` compile of the `decode_header_value` logic: UTF-8 input decodes as UTF-8
  (`naïve.pdf`), invalid-UTF-8 single byte (`0xE4`) falls back to ISO-8859-1 (`März`) — matches
  docstring and unit tests in `impit/src/response_parsing/mod.rs:454-497`.
- `ruff check` on `impit-python/test/async_client_test.py` and `impit-python/test/response_test.py`:
  all checks passed.
- Working tree matches `diff.patch` exactly (`git status` clean); no drift to account for.
- No other file references stale "wire order preserved" / unqualified "httpx equivalent" wording
  in any public doc, design doc, or verified-task doc.

## Notes (not reported as findings, confidence < 80 / non-doc)
- `impit-node/src/response.rs:65` and `impit-python/src/response.rs:230` retain stale plain `//`
  (non-doc) comments above the private `raw_header_pairs`/`raw_headers` struct fields ("in wire
  order (duplicates preserved)", "httpx `Headers.raw` equivalent"). These are internal
  implementation comments, not rendered rustdoc/public API documentation, and the actual `///`
  getter docs immediately below are correct and consistent with the design. Cosmetic only; no
  functional or user-facing documentation impact.

No correctness or compatibility regressions found. Value bytes remain exact end-to-end in both
bindings.
