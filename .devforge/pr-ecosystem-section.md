## Consistency with ecosystem

impit's two bindings each emulate a reference client, so header decoding is deliberately
**asymmetric** — and each side matches its reference exactly. Both bindings additionally expose
the raw header bytes, following the byte-access pattern each ecosystem already relies on.

### Python — matches `httpx` (which impit-python implements)

impit-python advertises the httpx interface ("drop-in replacement for `httpx.AsyncClient`"), and
httpx decodes header values **UTF-8-first with an ISO-8859-1 fallback** — exactly what this PR
does via the shared `decode_header_value` helper:

- httpx `Headers.encoding` tries `ascii`, then `utf-8`, then falls back to `iso-8859-1`:
  [`httpx/_models.py` @ v0.28.1, `encoding` property](https://github.com/encode/httpx/blob/0.28.1/httpx/_models.py#L125-L145)
  — *"Header encoding is mandated as ascii, but we allow fallbacks to utf-8 or iso-8859-1."*
- httpx exposes raw bytes via `Headers.raw: list[tuple[bytes, bytes]]`:
  [`httpx/_models.py` @ v0.28.1, `raw` property](https://github.com/encode/httpx/blob/0.28.1/httpx/_models.py#L152-L156).
  Our new `Response.raw_headers` returns the same `list[tuple[bytes, bytes]]` shape.

So Python callers get the same decoded strings *and* a raw-bytes escape hatch like httpx's.

> **Caveat vs. httpx `.raw`:** impit is built on `reqwest`, whose `HeaderMap` normalizes header
> names to lowercase and does not retain the original cross-header wire order — that information
> is gone before impit ever sees the response. So `raw_headers` (and JS `rawHeaders`) is
> httpx-`.raw`-*like* but not byte-identical: header **names** are lowercased and cross-header
> order is not guaranteed. Header **values** — the bytes that matter for signature/HMAC
> verification — are exact, and duplicate values for a name are preserved.

### JavaScript — matches the Fetch API / undici (which impit-node implements)

impit-node is "API-compatible with the Fetch API `Response`". In Fetch, header values are a
**byte sequence** exposed to JS as a `ByteString`, i.e. via **isomorphic decode** — each byte
`0x00–0xFF` maps to the code point of equal value (ISO-8859-1). This PR keeps impit-node on that
exact behavior (`b as char`):

- Fetch Standard: a header value is a [byte sequence](https://fetch.spec.whatwg.org/#concept-header-value),
  and the `Headers` interface types names/values as
  [`ByteString`](https://fetch.spec.whatwg.org/#headers-class) (`ByteString get(ByteString name)`).
- [WebIDL `ByteString`](https://webidl.spec.whatwg.org/#idl-ByteString) is the isomorphic
  (byte ↔ code-point) mapping — i.e. ISO-8859-1.
- undici (Node's `fetch`) implements exactly this: [nodejs/undici#1560 "ByteString checks &
  conversion in Headers"](https://github.com/nodejs/undici/pull/1560) and
  [#1317](https://github.com/nodejs/undici/issues/1317) confirm header values are handled as
  Latin-1 `ByteString`s.
- Node's core `http` parser likewise decodes header values as `latin1`/`binary`
  ([nodejs/node#17390](https://github.com/nodejs/node/issues/17390),
  [#58240](https://github.com/nodejs/node/issues/58240)); **axios** inherits this because its
  Node adapter reads `http.IncomingMessage` headers and its browser adapter reads
  `XMLHttpRequest`/Fetch headers.

Because ISO-8859-1 is isomorphic, the JS string stays **byte-recoverable** — the standard Fetch
workaround `Buffer.from(value, 'latin1')` (or `Uint8Array.from(value, c => c.charCodeAt(0))` in
the browser) reproduces the exact wire bytes, so a UTF-8 header can be recovered with
`Buffer.from(value, 'latin1').toString('utf8')`.

The Fetch `Headers` interface has **no** raw-byte accessor, so `response.rawHeaders`
(`Array<[string, Uint8Array]>`) is an explicit impit extension. It's justified because
signature/HMAC callers need the exact bytes without the manual round-trip, and it mirrors the
byte-pair access httpx already offers on the Python side.

### Net effect on #479

- **Python**: fully fixed — UTF-8 header values decode correctly (httpx behavior).
- **JavaScript**: string values remain ISO-8859-1 **by design** (Fetch parity, byte-recoverable);
  callers needing the decoded UTF-8 value read `response.rawHeaders` and decode with `TextDecoder`.
