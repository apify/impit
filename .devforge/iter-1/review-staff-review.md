VERDICT: FAIL

## blocker

1. **Broken doctest — will fail `cargo test -p impit` in CI.**
   `impit/src/response_parsing/mod.rs:154`:
   ```
   /// assert_eq!(decode_header_value(&[b'n', b'a', 0xC3, 0xAF, b've']), "naïve");
   ```
   `b've'` is not valid Rust syntax: `b'...'` is a *byte literal* and must contain exactly one
   ASCII byte (e.g. `b'v'`), not the two-character sequence `'ve'`. This is a compile error, not
   a runtime failure — rustdoc rejects it outright.

   Verified independently: extracted the exact doctest body (using only `String::from_utf8` /
   `into_bytes` — no dependency on the blocked `apify/h2` git patch) into a standalone file and
   ran `rustdoc --test`:
   ```
   error: if you meant to write a byte string literal, use double quotes
     6 - assert_eq!(decode_header_value(&[b'n', b'a', 0xC3, 0xAF, b've']), "naïve");
     6 + assert_eq!(decode_header_value(&[b'n', b'a', 0xC3, 0xAF, b"ve"]), "naïve");
   test result: FAILED. 0 passed; 1 failed
   ```
   The doctest block is plain ` ```rust ` (not `no_run`/`ignore`/`compile_fail`), so it is
   collected and compiled by `cargo test --doc` (part of the plain `cargo test -p impit` the
   `test` job in `.github/workflows/format.yaml` runs). This is a real CI-breaking bug, distinct
   from the disclosed "workspace won't build here" limitation — the syntax error is detectable
   with a bare `rustdoc --test` on the snippet alone and has nothing to do with the blocked `h2`
   dependency. The task's oracle (`iter-1/test-results.txt`) only ran the `#[cfg(test)] mod tests`
   unit tests via a standalone `rustc --test`, which does not execute doctests, so this bug slipped
   through undetected.

   Fix: change `b've'` to `b"ve"` and adjust the closing type (e.g. build the array with a byte
   string / `..*b"ve"` or just spell out `b'v', b'e'` as separate elements) so the example
   actually compiles.

2. **rustfmt violation — will fail the `fmt` job in `.github/workflows/format.yaml`.**
   `impit/src/response_parsing/mod.rs:184` (test `invalid_utf8_falls_back_to_iso_8859_1`):
   ```
   let bytes = [b'D', b'i', b'e', b'n', b's', b't', b'a', b'g', b',', b' ', b'3', b'1', b'.', b' ', b'M', 0xE4, b'r', b'z', b' ', b'2', b'0', b'2', b'6'];
   ```
   This line exceeds rustfmt's line-width limit and is not wrapped. Verified by running
   `rustfmt --check` (installed locally, rustfmt 1.8.0) against the file as shipped in the diff:
   it reports a diff at exactly this line, reformatting the array onto multiple lines. The repo's
   `format.yaml` workflow runs `actions-rust-lang/rustfmt@v1` on every PR — this diff has not been
   run through `cargo fmt` and will fail that check as-is.

   Fix: run `cargo fmt` (or manually wrap the array literal) before landing.

## Notes (not separate findings, context for the two blockers above)

- The core algorithm in `decode_header_value` (`impit/src/response_parsing/mod.rs:159-162`) is
  correct and matches the design exactly: `String::from_utf8` succeeds and returns UTF-8-decoded
  text whenever the whole byte slice is valid UTF-8 (fixes #479), and on failure
  `e.into_bytes()` yields the **entire original** byte vector (verified experimentally — not
  just the invalid tail), which is then mapped 1:1 byte→codepoint, reproducing the exact `#434`
  latin-1 fallback (e.g. lone `0xE4` → `ä`) with no `U+FFFD` ever introduced (#430). Confirmed
  with an independent standalone `rustc` build exercising mixed valid/invalid byte sequences,
  multi-byte lead-without-continuation sequences, and empty input — all behave as designed.
- Both binding call sites are correctly updated: `impit-node/src/response.rs:96` and
  `impit-python/src/response.rs:545` both now call `decode_header_value(v.as_bytes())`, and the
  imports (`impit-node/src/response.rs:3`, `impit-python/src/response.rs:8-11`) correctly pull
  the newly re-exported `impit::utils::decode_header_value` (re-export added at
  `impit/src/lib.rs:81`, mirroring the existing `decode`/`determine_encoding`/`ContentType`
  re-export pattern from the private `response_parsing` module — consistent and correct). No
  other `b as char` header-decoding call sites were missed (grepped the whole tree; the only
  other `as char` usage is an unrelated random-string generator in `fingerprint/mod.rs:47`).
- The Node test addition (`impit-node/test/basics.test.ts:574-577`,
  `impit-node/test/mock.server.ts:27-30,124-138`) correctly exercises the intended bytes: it
  writes the UTF-8 bytes for `naïve.pdf` (`0xC3 0xAF` for `ï`) directly onto the raw socket,
  mirroring the existing raw-socket pattern used by the `#434` regression test for `0xE4`, so it
  actually validates the wire-level decode path rather than something already normalized by a
  higher-level HTTP client on the server side. Good test; adequate to cover acceptance criterion
  1 and 5 from `_verified_task.md`.
- The two blockers above are cheap, mechanical fixes (change one byte literal; run `cargo fmt`)
  and do not implicate the core design or algorithm, but per review discipline any verified
  finding — including things this trivial — means the diff cannot pass as-is.
