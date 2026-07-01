VERDICT: PASS

## Scope of this round's diff (verified independently)

Diffed `iter-2-rev2/diff.patch` against the correct prior-round baseline
(`iter-1-rev2/diff.patch` — the design-rev-2 implementation that received the major finding;
`iter-2`/`iter-3` belong to the earlier, abandoned design-rev-1 track and are not the right
baseline). Result:

- `impit-node/src/response.rs`, `impit-node/test/basics.test.ts`, `impit-node/test/mock.server.ts`,
  `impit-python/src/response.rs`, `impit/src/lib.rs`, `impit/src/response_parsing/mod.rs` — **byte-
  identical** to the prior round. No regression risk introduced.
- `impit-python/test/response_test.py` — one line changed: `'naïve'.encode('utf-8')` →
  `'naïve'.encode()`. Behaviorally identical (`str.encode()` defaults to UTF-8); cosmetic only.
- `impit-python/test/async_client_test.py` — new `header_encoding_server` helper and new test
  `test_header_value_decoding_and_raw_bytes`, purely additive (file didn't exist in the prior
  round's diff in this form; no existing test modified).

This matches the expected shape: the fix is scoped exactly to adding the missing Python
integration test, with no incidental changes to the asymmetric-decode/raw-accessor logic itself.

## Verification of the new test

- **Real wire path, not the `#[new]` shortcut**: `test_header_value_decoding_and_raw_bytes` uses
  `AsyncClient(browser=browser).get(...)` against a real `socket`-based server
  (`header_encoding_server`), landing in the `From<Response>` conversion in
  `impit-python/src/response.rs:566-577` (the code path with `decode_header_value` and the
  `raw_headers` Vec built from `val.headers()`), not the manually-constructed `Response(...)`
  path. This is the exact gap the prior review flagged, and it's now closed — it mirrors the
  existing JS wire-level pattern (`utf8Header` route in `mock.server.ts` + `basics.test.ts`).
- **Hand-built HTTP/1.1 response is well-formed**: reconstructed the exact byte sequence locally
  and confirmed `Content-Length: 2` matches the 2-byte body `ok`, headers are correctly
  `\r\n`-terminated, and the header/body boundary (`\r\n\r\n`) is correct. Verified via a live
  IPv4 socket round-trip that a client reading this stream would parse it exactly as intended:
  `X-Utf8: attachment; filename="naïve.pdf"` (0xC3 0xAF for `ï`) and `X-Latin1: Märtz`-style value
  carrying a lone `0xE4` byte, matching PR #434's existing regression scenario.
- **Header-name case handled correctly**: the `From<Response>` conversion builds both the string
  `headers` dict and `raw_headers` from `k.as_str()` on `reqwest`'s `HeaderName`, which the `http`
  crate always normalizes to lowercase. The test asserts against lowercase keys
  (`response.headers['x-utf8']`, `raw[b'x-utf8']`) even though the server sends `X-Utf8`/
  `X-Latin1` — this is correct given `as_str()` semantics, and is consistent with how the
  pre-existing, unchanged constructor-path test (`test_response_constructor_with_headers`, uses
  `'Content-Type'` verbatim) differs because that path does *not* lowercase (no `HeaderName`
  involved) — no contradiction, just two different, correctly-modeled code paths.
- **Assertions are correct**: UTF-8 value decodes as the original Python `str` (httpx path via
  `decode_header_value`'s UTF-8-first branch); the lone `0xE4` byte falls back to ISO-8859-1
  producing `'März'`; `raw_headers` (as a `dict`) returns the exact wire bytes for both headers,
  matching the manually reconstructed byte sequences.
- **IPv6 dual-stack binding pattern**: `header_encoding_server` binds `('::', 0)` with
  `IPV6_V6ONLY=0` and the test connects via `127.0.0.1:{port}` — this exactly follows the
  pre-existing, proven pattern already used by `truncating_server`/`test_truncated_response` in
  the same file (not a new/untested pattern).
- **Test isolation**: uses its own dedicated `header_encoding_server` (own port via `port_holder`,
  own thread), doesn't interfere with or reuse state from other tests.

## Style / lint

- `ruff check` (`select = ["ALL"]`) and `ruff format --check` both pass clean on
  `async_client_test.py` and `response_test.py` (verified locally, matches `test-results.txt`).
- `rustfmt --check --edition 2021` passes clean on all three touched Rust files.
- `py_compile` succeeds on both test files.
- New test function/class placement is consistent with siblings (correct indentation inside
  `TestBasicRequests`, correct blank-line spacing, no orphaned/duplicate definitions).

## Conclusion

The prior major finding — Python's `raw_headers`/decode behavior being tested only via the
`#[new]` manual-construction path and not the real `from_async` fetch path — is genuinely
resolved. The new test exercises the identical wire-level scenario the JS suite already covered,
the hand-built response is protocol-correct, and the assertions correctly reflect httpx-style
decode semantics and exact-byte `raw_headers` parity. No new issues found; the diff since the
last review is exactly the two Python test files (plus one no-op cosmetic edit), with zero
changes to the reviewed decode/raw-accessor implementation itself.
