# Iteration 3 (final-reopen round 1) — implementer claim

## Addressed the thermonuclear final-review finding
- **Needless allocation on the decode path** — FIXED. Rewrote `decode_header_value` to validate
  against the borrow with `std::str::from_utf8(bytes)`:
  ```rust
  match std::str::from_utf8(bytes) {
      Ok(valid) => valid.to_owned(),
      Err(_) => bytes.iter().map(|&b| b as char).collect(),
  }
  ```
  This removes the unconditional `bytes.to_vec()` copy. The common UTF-8 path now allocates
  exactly once (`to_owned`); the latin-1 fallback allocates exactly once (`collect`) instead of
  twice. Byte semantics are identical (whole-buffer UTF-8 check, else full byte→codepoint map).
- **Inaccurate design-doc wording** ("single move / reuses the same buffer") — FIXED in
  `2-design.md` to describe the borrow-validate approach accurately.

## code-review final reviewer: PASS (no changes needed).

## Oracle — green
- `rustfmt --check` (core + both bindings): CLEAN.
- `rustc --test` unit tests: 5/5 pass (unchanged behavior confirmed after rewrite).
- `rustdoc --test` doctest: 1/1 pass.

## Scope
- Change is confined to the helper body + doc wording; no call-site, test, or public-signature
  change. Behavior is byte-for-byte identical, so this is a final-reopen (final reviewers only).
- Full workspace build / JS-Py suites still deferred to CI (github.com/apify/h2 egress 403).
