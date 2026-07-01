# Iteration 1 (rev 2) — implementer claim

## Done — asymmetric decode
- **Python (httpx):** keeps `decode_header_value` (UTF-8-first, latin-1 fallback) for the string
  `headers` dict. Unchanged from before.
- **JS (Fetch):** reverted the string decode to strict ISO-8859-1 (`b as char`) in
  `try_from_response`, and removed `decode_header_value` from the Node import. String header
  values now match Fetch and stay byte-recoverable via `Buffer.from(v, 'latin1')`.

## Done — raw-bytes accessor (new public API)
- **Python** `Response.raw_headers` → `list[tuple[bytes, bytes]]` (httpx `Headers.raw` parity),
  wire order + duplicates preserved. New private field `raw_headers: Vec<(Vec<u8>, Vec<u8>)>`
  captured in `from_async` (exact wire bytes) and in `new` (UTF-8 bytes of the string headers);
  getter builds `PyBytes`.
- **JS** `response.rawHeaders` → `Array<[string, Uint8Array]>` (impit extension). New private
  field `raw_header_pairs: Vec<(String, Vec<u8>)>` captured in `try_from_response`; napi getter
  maps to `Uint8Array`. The JS wrapper (`index.wrapper.js`) returns the patched native object and
  does not overwrite `rawHeaders`, so the native getter surfaces to users without extra plumbing
  (verified by reading `#wrapResponse`, which returns `originalResponse`).

## Tests
- **Core:** existing 5 `decode_header_value` unit tests unchanged (Python path).
- **JS** (`basics.test.ts` + `mock.server.ts`): existing latin-1 test (#434) kept; my previous
  UTF-8-string test replaced with a `rawHeaders` test — asserts the string form is latin-1
  (mojibake), `rawHeaders` yields the exact UTF-8 bytes, and `Buffer.from(latin1,'latin1')`
  round-trips to those bytes.
- **Python** (`response_test.py`): new `test_response_raw_headers` asserting `(bytes, bytes)`
  shape and exact UTF-8 bytes for a non-ASCII value.

## Oracle — green (what it can cover)
- `rustfmt --check` on all four touched Rust files: CLEAN.
- `rustc --test` core unit tests: 5/5. `rustdoc --test`: 1/1.

## NOT verifiable in this environment — must be confirmed by CI (disclosed at the design gate)
- **Binding compilation.** napi (`Uint8Array::from(Vec<u8>)`, `Vec<(String, Uint8Array)>` getter
  return) and pyo3 (`PyBytes::new`, `Vec<(Bound<PyBytes>, Bound<PyBytes>)>` getter) glue cannot
  be compiled here — the `github.com/apify/h2` git dep is egress-blocked (403). These follow the
  existing patterns in each crate but are UNVERIFIED against the compiler.
- **napi `index.d.ts` regeneration** for the new `rawHeaders` getter happens at `napi build` in
  CI; the committed `.d.ts` is intentionally not hand-edited.
- **JS/Python test execution** needs the built native module (napi/maturin) — CI only.
- Highest-risk specifics to watch in CI: the exact `Uint8Array` constructor, tuple→array
  ToNapiValue, and the pyo3 `Bound<PyBytes>` tuple return.
