# Triage — issue #479 (header decoding)

## Problem
Response header values are decoded byte-by-byte with `b as char` (ISO-8859-1) in
`impit-node/src/response.rs:96` and `impit-python/src/response.rs:542`. This was introduced
intentionally by PR #434 to fix #430 (non-ASCII header bytes crashed Node / returned empty in
Python). But ISO-8859-1 decoding garbles the common case of UTF-8 header values (e.g.
`Content-Disposition: filename="naïve.pdf"`) into mojibake (`ï` → `Ã¯`).

The two positions genuinely conflict: a header byte sequence can't be decoded as both latin-1
and UTF-8. #434 wants byte-preservation (RFC 9110 obs-text is latin-1); #479 wants correct
UTF-8. The maintainer explicitly left it open: "We might reinvestigate the best way forward."

## Decision: PROCEED
Both code claims VALID (code present at both sites; Python line is now 542, not 544 — minor
staleness only). Real, unresolved defect the maintainer wants revisited.

## Complexity: medium
Small code change (~2 call sites + tests in 2 bindings) but it alters the public response-header
contract across both language bindings → blast-radius override lifts it to at least medium.

## Review-only? no — there is a fix to build.

## Approach sketch (high level)
Decode as UTF-8 when the bytes are valid UTF-8, otherwise fall back to the existing
byte-preserving latin-1 decode. This fixes #479's UTF-8 case while keeping #434's test (which
sends bare `0xE4`, invalid UTF-8 → latin-1 `ä`) green. Never emits replacement chars, so #430's
crash/empty regression stays fixed. Apply symmetrically in Node and Python; add a UTF-8 test.

## Open questions
- Should a shared helper live in the core `impit` crate vs. duplicated per binding?
- Do we want to also expose raw header bytes for signature/HMAC callers (issue mentions this)?
  Likely out of scope for the core fix; note as follow-up.
