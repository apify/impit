# Iteration 3 (rev 2, final-reopen round 1) — implementer claim

Addressed all findings from both final reviewers (thermonuclear: 2; code-review: 3; one overlaps).

## Fixed
1. **`rawHeaders` dropped after `clone()`** (both reviewers) — the clone is a plain Fetch
   `Response`. `index.wrapper.js` now copies `rawHeaders` onto the clone via `Object.defineProperty`.
   Added a clone-preservation assertion to the JS test.
2. **`rawHeaders` missing from public TS surface / test needed `as unknown as`** (thermonuclear) —
   added `get rawHeaders(): Array<[string, Uint8Array]>` to the `ImpitResponse` class in
   `index.d.ts` (fetch returns `ImpitResponse`). The main test now accesses `response.rawHeaders`
   without a cast (only `clone()`, typed `Response`, still casts — honest, since clone returns a
   Fetch Response augmented at runtime).
3. **Overstated "wire order + original casing" claims** (code-review findings 2 & 3) — corrected.
   Root cause: reqwest's `HeaderMap` normalizes header names to lowercase and discards original
   cross-header wire order before impit sees the response, so true httpx-`.raw` parity is
   impossible. Softened the docstrings (Node + Python), `2-design.md`, `_verified_task.md`, and
   the PR ecosystem section to state: header **values** are exact bytes (the part that matters for
   HMAC), duplicate values are preserved, but **names are lowercased and cross-header order is not
   guaranteed**. No code change needed for values — they were already exact.

## No compile-breaking issues
Both final reviewers verified (against published crate source) that the napi tuple return and
pyo3 per-method lifetime signatures compile; no code change there.

## Oracle — green
Rust: `rustfmt --check` CLEAN; `rustc --test` 5/5; `rustdoc --test` 1/1.
Python: `ruff check` + `ruff format --check` clean; `py_compile` OK.
(Binding compilation + JS/Py test execution remain CI-gated — h2 egress block.)
