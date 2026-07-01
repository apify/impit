# Iteration 2 (rev 2) — implementer claim

## Addressed the iter-1-rev2 review finding (major)
- **Python `raw_headers`/decode only tested via the manual `new` constructor, not the real
  `from_async` fetch path** — FIXED. Added a wire-level integration test
  (`test_header_value_decoding_and_raw_bytes` in `async_client_test.py`) using a new raw-socket
  `header_encoding_server`, mirroring the JS mock-server approach. It sends a UTF-8 header value
  (`X-Utf8`) and a lone `0xE4` latin-1 byte (`X-Latin1`), then asserts:
  - `response.headers['x-utf8']` decodes correctly as UTF-8 (httpx path, exercises
    `decode_header_value` on a real response),
  - `response.headers['x-latin1'] == 'März'` (latin-1 fallback),
  - `response.raw_headers` yields the exact wire bytes for both.

## Oracle — green (extended this iteration to cover Python)
- Rust: `rustfmt --check` CLEAN; `rustc --test` 5/5; `rustdoc --test` 1/1.
- Python (new): `ruff check` clean; `ruff format --check` clean; `py_compile` OK. (ruff caught a
  real `UP012` `.encode('utf-8')` lint on the first pass — fixed.)

## Still CI-gated (unchanged, disclosed at design gate)
- Binding compilation (napi/pyo3) and execution of the JS/Python tests need the built native
  module; the `github.com/apify/h2` git dep is egress-blocked here. The new Python test's
  behavior is verified by CI, not locally.
