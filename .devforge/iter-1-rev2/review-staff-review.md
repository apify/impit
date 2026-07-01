VERDICT: FAIL

## Findings

### major: Python has no test exercising the real-fetch path (`from_async`) for either UTF-8 header decoding or `raw_headers`

- File: `impit-python/test/response_test.py:43-53` (the only new Python test, `test_response_raw_headers`)
- File: `impit-python/src/response.rs:545-635` (`from_async`, the constructor actually used for real HTTP responses; wires `decode_header_value` at line 570 and populates `raw_headers` at lines 573-577)

`test_response_raw_headers` builds its `Response` via the `#[new]` constructor (`impit-python/src/response.rs:237-285`), which never calls `decode_header_value` and derives `raw_headers` by simply re-encoding the Python string headers as UTF-8 (`impit-python/src/response.rs:249-252`). That path can't fail: any Python `str` header value round-trips through `.encode('utf-8')` trivially, so the test cannot detect a bug in `decode_header_value`'s UTF-8-first/ISO-8859-1-fallback logic, nor in `from_async`'s wiring of `headers`/`raw_headers` from real `reqwest::HeaderMap` bytes, nor an order/duplicate mismatch between the two collections built by two separate `val.headers().iter()` passes (lines 567-571 and 573-577).

This matters because issue #479 and the design are specifically about *real HTTP responses* carrying UTF-8 header bytes — the `new()` constructor path is not where the bug lived. Concretely, if a regression were introduced in `from_async` (e.g. a typo mapping `raw_headers` from `k.as_str()` on the wrong header, or `decode_header_value` never actually being called on the live path), the added test suite would not catch it; only the pre-existing `test_response_headers_encoding`/ASCII-header tests exercise `from_async`, and none of them use non-ASCII or UTF-8 header bytes.

The JS side, by contrast, does exercise the equivalent real-fetch path end-to-end: `impit-node/test/basics.test.ts:574-590` fetches through `impit.fetch(...)` (going through `try_from_response`, the wrapper, and the `rawHeaders` getter) against a raw-socket mock server route (`impit-node/test/mock.server.ts:124-138`) that writes literal UTF-8 bytes on the wire, then asserts both the latin-1 string mojibake and the exact `rawHeaders` bytes.

Python test infrastructure for this already exists and is used elsewhere in the same manner needed here: `impit-python/test/async_client_test.py:16-46` defines raw-socket servers (`thread_server`, `truncating_server`) that hand-craft an HTTP response header block and are exercised via `AsyncClient`/`Client` — i.e., through `from_async`. The acceptance doc (`_verified_task.md` item 4: "Python UTF-8 decode + raw bytes exact") and the design (`2-design.md`: "Python — UTF-8 decodes correctly + `raw_headers` returns exact bytes") both call for this; the change as submitted only satisfies it for the constructor path, not the fetch path, leaving the actually-fixed behavior (issue #479) unverified by any Python test.

**Fix scope**: add one raw-socket-based Python test (mirroring `thread_server`/`truncating_server`) that sends a header value with UTF-8 bytes (e.g. `naïve.pdf`) over `AsyncClient`/`Client`, and asserts (a) `response.headers[...]` decodes to the correct UTF-8 string and (b) `response.raw_headers` contains the exact wire bytes for that header, in order, matching what `headers` decoded.
