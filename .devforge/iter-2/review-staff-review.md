VERDICT: PASS

## Summary

Reviewed the iter-2 diff fresh (both prior blockers claimed fixed by implementer) against
`_verified_task.md` and `2-design.md`, with independent local verification (rustc/rustfmt/rustdoc),
not by trusting the oracle output or implementer claims alone.

## Verification performed

1. **Algorithm correctness (`impit/src/response_parsing/mod.rs:159-162`)**
   `decode_header_value` = `String::from_utf8(bytes.to_vec()).unwrap_or_else(|e| e.into_bytes().iter().map(|&b| b as char).collect())`.
   - Confirmed `FromUtf8Error::into_bytes()` returns the **complete original buffer**, not a
     truncated one, via standalone test (`[b'a', b'b', 0xE4, b'c', b'd']` round-trips fully through
     the fallback) — this is the crux of correctness for #434/#430: any single invalid byte
     anywhere in the header falls back to whole-buffer latin-1, not partial UTF-8 + partial mangle.
   - Verified all four use cases directly by compiling and running equivalent Rust:
     - UTF-8 (`naïve.pdf`, `héllo 世界 🎉`) decodes as UTF-8 (#479 fixed).
     - Invalid-UTF-8 latin-1 (lone `0xE4`) falls back to `ä` byte-for-byte (#434 preserved).
     - ASCII and empty-string pass through unchanged.
     - No input (tried lone `0xE4`, `0xFF 0xFE 0x41`, truncated multi-byte `0xC3` at end of buffer)
       ever produces `U+FFFD` (#430 non-crash/non-empty guarantee preserved).
   - This is strictly better than `from_utf8_lossy` as the design claims — confirmed lossy would
     introduce `U+FFFD` for the lone-`0xE4` case; the chosen implementation does not.

2. **Doctest / byte-literal blocker (prior iteration's blocker #1)**
   Extracted the exact doctest from `impit/src/response_parsing/mod.rs:150-155` into a standalone
   crate and ran real `rustdoc --test` against it (compiled a `.rlib` and linked it properly, not
   just `rustc` on a `fn main`). Result: **passes** (`test ... - response_parsing::decode_header_value
   (line 10) ... ok`). The old broken single out-of-range byte literal is gone; `0xC3, 0xAF` are now
   two separate valid `u8` array elements. This matches the oracle's `test-results.txt` doctest
   result and is independently confirmed as a genuine fix, not just an oracle artifact.

3. **rustfmt cleanliness (prior iteration's blocker #2)**
   Ran `rustfmt --check` (not the oracle's cached result) on all four touched files, respecting the
   project's actual `impit-node/rustfmt.toml` (`tab_spaces = 2`) by running from within
   `impit-node/`:
   - `impit/src/response_parsing/mod.rs` — clean
   - `impit/src/lib.rs` — clean
   - `impit-node/src/response.rs` — clean (2-space indent matches diff)
   - `impit-python/src/response.rs` — clean
   All exit 0. Matches oracle's "FMT CLEAN".

4. **Binding call sites**
   - Node (`impit-node/src/response.rs:3,94`): import adds `decode_header_value` alongside existing
     `decode, ContentType`; call site `decode_header_value(v.as_bytes())` where `v: &HeaderValue`
     (`.as_bytes()` returns `&[u8]`) — matches `fn decode_header_value(bytes: &[u8]) -> String`;
     assigned into `Vec<(String, String)>` element — types match exactly what was there before.
   - Python (`impit-python/src/response.rs:8-11,542-546`): import restructured into a multi-item
     `use impit::{errors::ImpitError, utils::{decode_header_value, ContentType}};` — syntactically
     valid Rust (verified structurally); call site inside `HashMap::from_iter(...)` closure,
     `decode_header_value(v.as_bytes())` returns `String`, matching the `HashMap<String, String>`
     target type exactly as before.
   - Confirmed no naming collision with the pre-existing unrelated `impit::utils::decode` (body
     decoder) — `impit-python/src/response.rs:458` still calls fully-qualified `impit::utils::decode`
     for body content, untouched.
   - Confirmed no leftover duplicate inline `b as char` logic anywhere outside the shared helper
     (`grep` for `as_bytes().iter()` in both bindings' `src/` returns nothing).
   - Re-export chain verified: `impit/src/lib.rs:81` adds `pub use crate::response_parsing::decode_header_value;`
     inside the existing `pub mod utils { ... }` block, consistent with how `decode`,
     `determine_encoding`, `ContentType` are already re-exported.

5. **Test coverage**
   - Core unit tests (`impit/src/response_parsing/mod.rs:169-212`, oracle: 5/5 pass) cover ASCII,
     empty, UTF-8, invalid-UTF-8-latin-1, and an explicit round-trip/no-replacement-char assertion.
     Bytes are genuinely exercised as raw `&[u8]` / byte arrays, not derived from a `String` that
     would mask the code path (e.g. `utf8_is_decoded_as_utf8` uses `"...".as_bytes()`, but since the
     literal is valid UTF-8 source text this correctly represents the UTF-8-bytes-on-the-wire case).
   - Node regression test (`impit-node/test/basics.test.ts:574-577` + `mock.server.ts:27-30,124-138`)
     mirrors the existing latin-1 guard exactly: writes the raw HTTP response via a raw socket with
     literal UTF-8 bytes for the header value (`Buffer.from(routes.utf8Header.headerValue, 'utf-8')`),
     bypassing any client-side normalization, so it genuinely exercises the wire-bytes decode path
     inside the Rust binding rather than a value Node's HTTP stack might re-encode. This is the
     correct test shape (same pattern as the pre-existing `nonAsciiHeader` test) and satisfies
     acceptance criterion #5 (UTF-8 regression test present in Node).
   - Design explicitly defers a Python-side test as a nice-to-have (build unavailable locally); this
     is an accepted, documented risk in `2-design.md`, not a gap introduced by this diff.

6. **Scope check**
   - No out-of-scope raw-header-bytes API was added (design explicitly deferred this).
   - No stray edits outside the five touched files (`impit-node/src/response.rs`,
     `impit-node/test/basics.test.ts`, `impit-node/test/mock.server.ts`,
     `impit-python/src/response.rs`, `impit/src/lib.rs`, `impit/src/response_parsing/mod.rs`).

No blockers, majors, minors, or nits found. Both prior-iteration blockers are genuinely fixed
(independently reproduced, not just re-reading the oracle log), the algorithm satisfies all four
acceptance criteria under adversarial edge-case testing, both binding call sites compile-shape
correctly, and test coverage is adequate and genuinely exercises the byte-level decode path.
