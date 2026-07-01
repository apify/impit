VERDICT: FAIL

## MINOR — needless allocation on the common path + inaccurate doc claim in `decode_header_value`

**File:** `impit/src/response_parsing/mod.rs:159-162`

```rust
pub fn decode_header_value(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec())
        .unwrap_or_else(|e| e.into_bytes().iter().map(|&b| b as char).collect())
}
```

`bytes.to_vec()` unconditionally copies the entire input buffer into a freshly allocated
`Vec<u8>` *before* UTF-8 validity is even checked, on every call, for every header, on every
response. The design doc (`.devforge/2-design.md:22-25`) and the function's own rustdoc-adjacent
commentary claim this is "a single move with no per-byte work" on the common path — that's not
accurate for `to_vec()`, which is a byte-for-byte `memcpy`, not a move. `Vec::from(bytes)`/`to_vec`
never reuses the caller's buffer since the caller only hands over a borrowed `&[u8]`
(`HeaderValue::as_bytes()`), so there's no way to "move" here regardless.

The direct, idiomatic, and cheaper version is the standard borrow-first pattern:

```rust
pub fn decode_header_value(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => bytes.iter().map(|&b| b as char).collect(),
    }
}
```

This validates against the borrowed slice with zero allocation, and only allocates once (via
`to_string()`) on the success path — i.e. it does strictly less work than the current
`to_vec()` (which allocates+copies unconditionally) followed by `String::from_utf8` (which just
re-wraps that buffer). It's also easier to read: no `Result`-unwrapping through
`.into_bytes()` in the error arm, no implicit conversion of "the same owned buffer" that isn't
actually being reused (the current code's error path calls `.into_bytes()` on the `FromUtf8Error`
only to immediately throw the `Vec<u8>` away byte-by-byte in the `.iter().map(...)` — so the
"reuses the same owned buffer" claim in the design doc is also not realized: the fallback path
re-walks the vec one byte at a time and builds a brand new `String`, it does not reuse the
allocation).

Concrete scenario this matters for: this helper runs on *every response header, for every
request*, in both bindings (Node hot path: `impit-node/src/response.rs:94`; Python hot path:
`impit-python/src/response.rs:545`). It's exactly the kind of small, shared, called-everywhere
core-crate helper where an avoidable per-call heap copy is worth eliminating now rather than
carrying it forward as "how the shared helper has always worked" — and the mis-description in
the design doc/rustdoc-adjacent rationale ("single move," "reuses the same owned buffer") makes
the code read as more optimized than it actually is, which will mislead the next person who
touches this function into thinking the allocation profile is already minimal.

This is a one-line fix, low risk, behavior-preserving (verified via a standalone rustc oracle:
both the current implementation and the `std::str::from_utf8` version produce identical output
for the UTF-8, invalid-UTF-8/latin-1-fallback, and empty-input cases). Given the ENGINE.md
standard of "code-judo simplification" and "no needless allocation," this should be fixed before
merge rather than accepted as-is.

Everything else in this diff is sound: the helper lives in the correct canonical location
(`impit/src/response_parsing/mod.rs`, re-exported via `impit::utils`), both bindings now call the
one shared implementation with no leftover duplicate `b as char` logic anywhere in the tree, no
file approaches the 1k-line ceiling (largest touched file is `impit-python/src/response.rs` at
604 lines), there is no new branching/spaghetti introduced into `response.rs` in either binding
(the call sites are direct one-line substitutions), and the unit tests plus the new Node UTF-8
regression test are well-targeted and correctly assert the documented invariants (UTF-8 first,
latin-1 fallback, no `U+FFFD`, byte-reversibility). The only issue is the allocation/doc-accuracy
point above.
