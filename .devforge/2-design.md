# Design — fix #479 header decoding without regressing #434

## What we're solving
Response header values are decoded as ISO-8859-1 (`b as char`) in both bindings. That was a
deliberate choice in PR #434 to stop non-ASCII header bytes from crashing Node / emptying
Python (issue #430). But ISO-8859-1 mangles the far more common case — headers whose bytes are
UTF-8 (e.g. `Content-Disposition: filename="naïve.pdf"`) — into mojibake (`ï` → `Ã¯`), breaking
filename extraction and any byte-exact re-encoding. We need both cases correct at once.

## How
Decode with **UTF-8 first, ISO-8859-1 fallback**:

- If the header bytes are valid UTF-8 → decode as UTF-8 (fixes #479's mojibake).
- Otherwise → fall back to the existing byte-preserving `b as char` latin-1 decode (keeps #434;
  e.g. bare `0xE4` → `ä`).

This never emits `U+FFFD` replacement characters, so #430's non-crash / non-empty guarantee is
preserved and the latin-1 fallback stays byte-reversible. It is strictly better than the
issue's own suggestion (`from_utf8_lossy`), which would turn #434's bare `0xE4` into `U+FFFD`
and reintroduce corruption for exactly the case #434 fixed.

Rust expresses this cleanly and allocation-efficiently:
`String::from_utf8(bytes.to_vec()).unwrap_or_else(|e| e.into_bytes().iter().map(|&b| b as char).collect())`
— the common UTF-8 path is a single move with no per-byte work; the fallback reuses the same
owned buffer.

## Alternatives + the call
- **`from_utf8_lossy` (issue's suggestion):** rejected — lossy and regresses #434 (replacement
  chars, irreversible) as shown above.
- **Latin-1 always (status quo):** rejected — this is the bug.
- **UTF-8 always / error on invalid:** rejected — re-breaks #430 (invalid-UTF-8 latin-1 headers).
- **Expose raw header bytes API for HMAC/signature callers:** deferred — useful but a separate,
  larger public-API addition; note as follow-up, out of scope here.
- **Chosen: UTF-8-first with latin-1 fallback**, placed in one shared helper.

## Major changes (key areas, not exhaustive)
- Add a shared `decode_header_value(&[u8]) -> String` helper in the core crate's
  `response_parsing` module, re-exported through `impit::utils`, so both bindings share one
  tested implementation instead of duplicating the closure. Cover it with core unit tests
  (ASCII, UTF-8, invalid-UTF-8 latin-1, empty).
- Node (`impit-node/src/response.rs`): replace the inline `b as char` map with a call to the
  shared helper.
- Python (`impit-python/src/response.rs`): same replacement.
- Tests: add a UTF-8 header regression test to the Node suite (mirrors the existing latin-1
  test in `basics.test.ts` / `mock.server.ts`); the existing latin-1 test is the guard that the
  fallback still works.

## Risks / open questions
- **Ambiguous bytes:** a byte sequence that is *coincidentally* valid UTF-8 but was meant as
  latin-1 will now decode as UTF-8. This is unavoidable without out-of-band charset info and
  UTF-8 is the correct modern default; the tradeoff is intended.
- **Environment/oracle limitation:** the full Rust workspace cannot compile here — the pinned
  git dep `github.com/apify/h2` is blocked (403) by org egress and its cache is empty. The
  devforge oracle therefore runs a standalone `rustc --test` copy of the helper to prove the
  algorithm; full binding compilation/integration must be verified in CI. Reviewers should treat
  binding-compile as unverified-locally.
- **Python test gap:** Python has no existing header-decode test; adding one requires the
  maturin build (also unavailable here). Node coverage + shared-helper unit tests carry the
  correctness signal; a Python test is a nice-to-have follow-up if the build is available in CI.
