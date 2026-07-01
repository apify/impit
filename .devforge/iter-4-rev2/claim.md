# Iteration 4 (rev 2, final-reopen round 2) — implementer claim

## Addressed the remaining thermonuclear finding
- **Internal field comments still said "in wire order"** (`impit-node/src/response.rs:65`,
  `impit-python/src/response.rs:230`), contradicting the corrected getter docstrings — FIXED.
  Both field comments now state values are exact, names lowercased, order not the original wire
  order, and point to the getter docs. Also dropped the stale "httpx `Headers.raw` equivalent"
  phrasing from the Python field comment.
- Verified no remaining overstated "wire order" claim: the only surviving mentions are accurate
  negations ("… not preserved").

## code-review: PASS last round (no further changes needed).

## Oracle — green
Rust: `rustfmt --check` CLEAN; `rustc --test` 5/5; `rustdoc --test` 1/1.
Python: `ruff` + `ruff format --check` clean; `py_compile` OK.
(Comment-only change since iter-3-rev2; binding compile + JS/Py tests remain CI-gated.)
