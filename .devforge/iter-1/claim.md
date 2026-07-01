# Iteration 1 — implementer claim

## Done
- Added `decode_header_value(&[u8]) -> String` to `impit/src/response_parsing/mod.rs`:
  UTF-8-first (`String::from_utf8`) with a byte-preserving ISO-8859-1 fallback
  (`b as char`) on invalid UTF-8. Documented with rustdoc + doctest.
- Re-exported it via `impit::utils` in `impit/src/lib.rs`.
- Replaced the inline `b as char` header map in both bindings with a call to the shared helper:
  - `impit-node/src/response.rs` (import + call site).
  - `impit-python/src/response.rs` (import + call site).
- Added core unit tests (ASCII, empty, UTF-8/#479, invalid-UTF-8 latin-1/#434, and a
  no-replacement-char + byte-roundtrip guard for #430).
- Added a Node UTF-8 regression test: new `/utf8-header` route in `mock.server.ts` sending real
  UTF-8 bytes (`ï` = 0xC3 0xAF) and an assertion in `basics.test.ts` that it decodes to
  `attachment; filename="naïve.pdf"`. The existing latin-1 test is the fallback guard.

## Oracle
- Green. `rustc --test` on a standalone exact copy of the helper: 5/5 tests pass
  (see `iter-1/test-results.txt`). This proves the algorithm for all four use cases.

## Skipped / not done — with reason
- **Full `cargo build`/`cargo test` and the Node/Python test suites: NOT run.** The workspace
  pins git dep `github.com/apify/h2`, which returns 403 through the org egress proxy (cache
  empty). This is an environment/policy limit, not a code issue. Binding compilation and the JS
  test I added must be verified in CI where github egress is allowed. The core-crate helper is
  simple, self-contained, and validated by the standalone oracle.
- **Raw-header-bytes API (for HMAC/signature callers):** intentionally out of scope per
  `2-design.md`; noted as a follow-up.
- Did not touch `impit/src/fingerprint/mod.rs:47` `... as char` — unrelated random-string
  generation, not header decoding.
