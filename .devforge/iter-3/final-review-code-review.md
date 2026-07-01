VERDICT: PASS

## Scope of this round
Reviewed the single targeted change: `decode_header_value` in `impit/src/response_parsing/mod.rs:159-164` was rewritten from
`String::from_utf8(bytes.to_vec()).unwrap_or_else(|_| bytes.iter().map(|&b| b as char).collect())`
to
`match std::str::from_utf8(bytes) { Ok(valid) => valid.to_owned(), Err(_) => bytes.iter().map(|&b| b as char).collect() }`.
Call sites (`impit-node/src/response.rs:94`, `impit-python/src/response.rs:545`), the re-export (`impit/src/lib.rs:81`), the public signature, and tests are confirmed unchanged from the prior iteration.

## Verification performed
- Extracted both the old and new implementations into a standalone Rust program and ran an exhaustive equivalence check with `rustc -O`: all 256 single-byte inputs, all 65,536 two-byte inputs, a curated set of UTF-8 edge cases (valid multi-byte sequences, truncated 2/3/4-byte sequences, overlong encodings, encoded surrogate halves, empty input), and 2,000 deterministic pseudo-random byte sequences (lengths 0-11). Result: 67,807/67,807 identical outputs between old and new — zero mismatches. This is expected since both paths use the same UTF-8 validator (`std::str::from_utf8` internally backs `String::from_utf8`); the change only avoids validating/copying via an intermediate `Vec<u8>`.
- Compiled and ran the two doc-comment examples (naïve UTF-8 case, März latin-1 fallback case) standalone — both pass.
- Compiled and ran the four `#[cfg(test)]` unit tests (`ascii_is_unchanged`, `empty_is_empty`, `utf8_is_decoded_as_utf8`, `invalid_utf8_falls_back_to_iso_8859_1`, `iso_8859_1_fallback_never_produces_replacement_char`) via standalone `rustc --test` — all 5 pass.
- Ran `cargo clippy --all-targets -- -D warnings` on the extracted function in an isolated crate — zero warnings (no `needless_collect`/allocation lints triggered).

## Acceptance criteria re-confirmed
1. Valid-UTF-8 header bytes decode as UTF-8 (fixes #479 mojibake) — confirmed via unit test and doctest.
2. Lone `0xE4` still decodes byte-for-byte as latin-1 → `ä` (keeps #434) — confirmed via unit test and equivalence check.
3. No `U+FFFD` ever introduced; function never panics or returns empty for non-empty invalid-UTF-8 input (keeps #430) — confirmed via `str::from_utf8` never causing a panic path here (only produces `Result`, no `.unwrap()`), and the fallback branch is a total function over `u8` (`b as char` is defined for all `0..=255`).

## Findings with confidence >= 80
None. No correctness, regression, or compilation-risk findings meet the threshold.
