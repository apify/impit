# Request fact-check — claim ledger (verified against HEAD 9d2204f)

| # | Claim | Verdict | Evidence |
|---|-------|---------|----------|
| 1 | Node decodes headers with `b as char` at `impit-node/src/response.rs:96` | VALID | Line 96: `v.as_bytes().iter().map(|&b| b as char).collect(),` |
| 2 | Python decodes headers with `b as char` at `impit-python/src/response.rs:544` | STALE(→ `impit-python/src/response.rs:542`) | Same code, line shifted to 542: `...collect::<String>()` |
| 3 | This interprets each byte as Latin-1 / ISO-8859-1 | VALID | `b as char` on a `u8` maps 0x00–0xFF → U+0000–U+00FF (Latin-1 code points) |
| 4 | UTF-8 header values are garbled into mojibake | VALID | For `ï` (UTF-8 `0xC3 0xAF`), latin-1 decode yields two chars `Ã¯`; re-encoding yields different bytes |
| 5 | Behavior was intentional (PR #434, fixes #430) | VALID | PR #434 merged 2026-04-13; maintainer comment on #479: "This was intentional... We might reinvestigate the best way forward." |
| 6 | Suggested `from_utf8_lossy` is the right fix | LIKELY-FIXED-BUT-INCOMPLETE | `from_utf8_lossy` fixes UTF-8 but REGRESSES #434's latin-1 case: bare `0xE4` (invalid UTF-8) → `U+FFFD` replacement char, not `ä`, and is lossy/irreversible. A try-UTF-8-then-latin-1 fallback is strictly better. |

## Existing locked-in behavior (regression guard)
- `impit-node/test/basics.test.ts:569` sends header byte `0xE4` (mock.server.ts:111, invalid
  UTF-8) and asserts it decodes to `ä`. Any fix MUST keep this green.
- No equivalent Python test exists yet.

## Verdict
PROCEED. Core defect is real and open. The issue's own suggested fix (`from_utf8_lossy`) is
incomplete — it would reintroduce corruption for the exact case #434 fixed. Correct resolution
is a UTF-8-first decode with latin-1 fallback, applied to both bindings.
