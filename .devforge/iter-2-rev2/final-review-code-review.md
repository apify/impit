VERDICT: FAIL

## Findings (confidence >= 80)

### 1. [High severity] JS `rawHeaders` does not survive `Response.clone()` — silently `undefined`

**File:** `impit-node/index.wrapper.js:417-492` (the `clone` override inside `#wrapResponse`), specifically lines 474-478.

The task's own verification step asks whether `#wrapResponse` clobbers `rawHeaders`. It does not clobber it on the *primary* response object — `#wrapResponse` only calls `Object.defineProperty` on `text`, `bytes`, `arrayBuffer`, `json`, `headers`, and `clone` (`index.wrapper.js:379,387,396,405,413,417`), so `rawHeaders`, being a napi prototype getter on the returned `ImpitResponse` instance (`impit-node/src/response.rs:140-151`), is reachable on `originalResponse` as-is.

However, `clone()` (redefined at `index.wrapper.js:417-492`) constructs its return value via `new Response(stream2, { status, statusText, headers })` at line 474 — the **standard built-in Web `Response`**, not an `ImpitResponse`. That class has no `rawHeaders` getter at all (own or inherited). Only `url` and `text` are manually stapled onto the clone (lines 479-488).

**Concrete failing scenario:**
```js
const response = await impit.fetch(url); // has rawHeaders, works
const clone = response.clone();
clone.rawHeaders; // undefined — silently absent, not an error
```
Any caller who clones a response (a standard, encouraged Fetch pattern, e.g. to read headers in one branch and body in another) loses access to the new raw-bytes accessor with no error or warning. Since `rawHeaders` is the first `ImpitResponse`-only extension with no standard-`Response` equivalent (unlike `body`/`text`/`json`/`arrayBuffer`, which the clone already re-implements), this is a real, newly-introduced gap in the "surfaces to users through `index.wrapper.js`" requirement, not a pre-existing limitation that this diff merely inherits.

### 2. [High severity] `raw_headers`/`rawHeaders` do not actually preserve wire order when duplicate header names are interleaved with other headers

**Files:**
- `impit-node/src/response.rs:65-67` (doc comment) and `:101-107` (construction via `response.headers().iter()`)
- `impit-python/src/response.rs:230-231` (doc comment) and `:566-577` (construction via `val.headers().iter()`)

Both doc comments explicitly claim: *"Raw, undecoded header name/value byte pairs, in wire order (duplicates preserved)."* Both are built by a single pass over `reqwest::Response::headers().iter()`, which delegates to `http::HeaderMap::iter()`.

`http::HeaderMap`'s own documentation states iteration order is "arbitrary, but consistent... Each key will be yielded once per associated value" — i.e., it does NOT guarantee wire order. Structurally, `HeaderMap` stores the first value per distinct name in a primary table and all subsequent values for a repeated name in a separate `extra_values` side-list; `Iter::next()` drains all of a repeated name's extra values as soon as it reaches that name's slot, before moving to the next distinct name — regardless of what other headers arrived on the wire in between.

**Concrete failing scenario:** A server sends, in this exact wire order:
```
Content-Type: ...
X-Trace: span-1
X-Multi: first
X-Trace: span-2
X-Multi: second
```
`raw_headers`/`rawHeaders` (and the underlying `HeaderMap::iter()`) will yield `X-Trace: span-1`, `X-Trace: span-2`, `X-Multi: first`, `X-Multi: second` — the second `X-Trace` value is reordered ahead of `X-Multi: first`, even though `X-Multi: first` arrived earlier on the wire. This was verified empirically by compiling and running a reproduction against the exact `http` crate version pinned in this repo's `Cargo.lock`.

This directly undermines the acceptance criterion "both raw accessors return EXACT wire bytes with order + duplicates preserved" for any response with interleaved duplicate header names — precisely the scenario where the stated HMAC/signature use case (order-sensitive by nature) would silently get wrong data with no indication of failure. Existing tests only exercise duplicates that are sent back-to-back (e.g. consecutive `Set-Cookie` headers), which happens to preserve apparent order and does not catch this.

### 3. [Medium-high severity] Python `raw_headers` header **names** are lowercased, not the exact wire bytes, contradicting the docstring's httpx-parity claim

**File:** `impit-python/src/response.rs:456-461` (docstring) and `:573-577` (construction via `val.headers().iter()`, using `k.as_str().as_bytes()`).

The docstring states this getter is the "httpx `Response.headers.raw` equivalent" and returns "exact wire bytes." `k` here is an `http::HeaderName`, whose `as_str()` always returns the lowercased ASCII form — `HeaderName` normalizes and stores names case-insensitively at construction and retains no original casing. Verified against real httpx 0.28.1 (`httpx.Headers.raw` returns `raw_key`, the original casing exactly as received, e.g. `[(b'X-Utf8', b'val')]` stays `X-Utf8`, not lowercased).

**Concrete failing scenario:** A server sends `X-Signature: abc123`. `response.raw_headers` yields `(b'x-signature', b'abc123')` — the name is lowercased, unlike httpx's `.raw`, which would preserve `b'X-Signature'`. A caller building an HMAC over the literal header line (name included) using impit's `raw_headers` to match behavior documented as httpx-equivalent gets a different byte sequence than httpx would produce for the same wire response. Existing tests (`impit-python/test/async_client_test.py:481-482`) don't catch this because they only assert against already-lowercase expected keys (`raw[b'x-utf8']`), and the JS test defensively lowercases before comparing (`impit-node/test/basics.test.ts:583`, `k.toLowerCase() === 'x-utf8'`), so neither test suite exercises or would catch a case-sensitivity mismatch.

Note: this is not a finding for the JS side — the design doc explicitly scopes JS's `rawHeaders` name as "name as string per Fetch conventions" (not claiming byte-exactness for the name), so JS's behavior matches its own documented contract. Only Python's docstring makes the stronger "exact wire bytes" / "httpx equivalent" claim that this contradicts.
