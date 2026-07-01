# Iteration 2 — implementer claim

## Addressed both iter-1 review findings (blockers)
1. **Broken doctest (`mod.rs` byte literal `b've'`)** — FIXED. Changed to separate elements
   `b'v', b'e'`. Verified: the doctest now compiles and passes under `rustdoc --test` (see
   `iter-2/test-results.txt`).
2. **rustfmt violation (long array literal)** — FIXED. Ran `rustfmt` on all touched files.

## Additional fix surfaced by the strengthened oracle
- Collapsing the header-map closures to call `decode_header_value` changed line lengths, so
  `rustfmt` rewrapped the Node push (now one line) and the Python `HashMap::from_iter` map.
  Applied. `rustfmt --check` is now clean across all four touched Rust files.

## Oracle (strengthened this iteration to close the gap that let #1/#2 through)
Three commands, all green:
- `rustfmt --check` on core + both binding files → CLEAN.
- `rustc --test` unit tests → 5/5 pass.
- `rustdoc --test` doctest → 1/1 pass (this is what would have caught the byte-literal typo;
  `rustc --test` alone does not run doctests).

## Unchanged from iter-1 (still true)
- Algorithm: UTF-8-first, byte-preserving ISO-8859-1 fallback; both binding call sites use the
  shared helper; Node UTF-8 regression test added.
- Full `cargo`/napi/maturin build still cannot run here (github.com/apify/h2 egress 403);
  binding compile + JS/Py suites must run in CI. No code reason they would fail.
