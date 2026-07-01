VERDICT: PASS

## Method
Independent fresh review per code-review.md's approach (read-diff, shallow-bug-scan, comment/doc-guidance-compliance, historical-context reasoning), applied to `impit/src/response_parsing/mod.rs`, `impit/src/lib.rs`, `impit-node/src/response.rs`, `impit-python/src/response.rs`, and the Node test additions. Verified the pure helper with a standalone rustc oracle (rustc 1.x local toolchain) since the full workspace cannot build here.

## Verification performed
- Extracted `decode_header_value` into a standalone snippet and ran the 4 tests from the diff plus 4 adversarial tests I added (mixed valid-UTF-8-prefix-then-invalid-byte, truncated multibyte lead byte, whole-buffer-fallback-not-partial-decode check, char-count-equals-byte-count check). All 8 passed.
- Confirmed `String::from_utf8` failure causes the *entire* original byte buffer (via `FromUtf8Error::into_bytes()`, which returns the original vec unmodified) to fall back to the byte-for-byte latin-1 map — no partial UTF-8 decoding/mixing occurs, so PR #434's guarantee (byte-exact latin-1 on invalid UTF-8) holds even for buffers with a valid UTF-8 prefix followed by a bad byte.
- Confirmed the fallback path never invokes `DecoderTrap::Replace` or any lossy path, so no `U+FFFD` can appear (issue #430 guarantee) — this is structural, not incidental (there is no lossy call in either branch).
- Confirmed the new doctest on `decode_header_value` compiles and passes under `rustdoc --test` once given a matching `--edition 2021` (my harness's first failure was a self-inflicted edition mismatch in the oracle harness, not a defect in the source).
- Confirmed rustfmt reports no formatting diff on the new function.
- Confirmed both binding call sites (`impit-node/src/response.rs:94`, `impit-python/src/response.rs:545`) import and call `decode_header_value(v.as_bytes())` with matching signature `&[u8] -> String`, directly replacing the old `v.as_bytes().iter().map(|&b| b as char).collect()` inline closures 1:1 — no behavior divergence between bindings.
- Confirmed `impit/src/lib.rs` re-exports `decode_header_value` through `pub mod utils`, alongside the existing `decode`/`determine_encoding`/`ContentType` exports, so both bindings' `use impit::utils::{..., decode_header_value, ...}` imports resolve.
- Confirmed the Node regression test (`impit-node/test/basics.test.ts:574-577`) and its mock route (`impit-node/test/mock.server.ts:124-138`) mirror the pre-existing, already-proven `nonAsciiHeader` raw-socket-header-injection pattern; verified with a quick Node snippet that `Buffer.from(headerValue, 'utf-8')` produces the expected UTF-8 bytes (0xC3 0xAF for `ï`), which `decode_header_value` will correctly round-trip back to the original string.

## Acceptance criteria (from `_verified_task.md`)
1. UTF-8 header decodes as UTF-8 — verified (oracle test + doctest).
2. Invalid-UTF-8 latin-1 bytes still decode byte-for-byte as latin-1 — verified, including the harder case of a valid-UTF-8-looking prefix followed by an invalid byte (whole buffer still falls back atomically).
3. No `U+FFFD` ever introduced — verified structurally (no lossy decode call exists in either code path).
4. Applied symmetrically in Node and Python — verified, identical call pattern in both `response.rs` files.
5. Regression test present for UTF-8 case in Node — verified, added and consistent with existing test infra.

## Findings with confidence >= 80
None.

Minor items noted but explicitly out of scope per the design doc's own risk section (ambiguous-bytes tradeoff, no Python test due to unavailable maturin build here, un-runnable full workspace build due to blocked `h2` git dependency) — these are called out and accepted as intentional/environment-limited in `.devforge/2-design.md`, not defects introduced by this change, and do not meet the >=80 confidence bar as unaddressed regressions.
