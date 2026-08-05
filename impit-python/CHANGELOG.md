# Changelog

All notable changes to this project will be documented in this file.


## py-0.13.2 - 2026-08-05

#### Bug Fixes

- Fix panic on a malformed server cookie (#491)
  - Malformed cookies are silently ignored like in JS version.


- Decode header values per-ecosystem (Python/JS), expose httpx-style Response.headers (#492)

- Add `raise_for_status` to type stub (#501)
  - The method is implemented and works at runtime, but was missing from the type stub, so type checkers were complaining.


- Align `Browser` literal with the fingerprints actually supported at runtime (#509)


## py-0.13.1 - 2026-06-25

#### Bug Fixes

- Cookie domain matching (#488)
  - Replace substring match by matching according to [RFC 6265, §5.1.3](https://www.rfc-editor.org/rfc/rfc6265#section-5.1.3). Add test.  # Issues: Closes: #473  ---------



## py-0.13.0 - 2026-06-19

#### Bug Fixes

- Decode non-ASCII response header values as ISO-8859-1 (#434)

- Use browser-matching multipart boundary format (#435)
  - Moves multipart boundary generation from JS to Rust where the browser fingerprint is available. Each browser profile now produces boundaries matching the real browser format:  - Chrome: `----WebKitFormBoundary` + 16 alphanumeric chars - Firefox: `----geckoformboundary` + two random uint64 hex values - OkHttp: UUID v4 (`xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx`) - No fingerprint: `----formdata-impit-*` (default, unchanged)  The boundary is generated lazily — the NAPI call only happens when the body is actually a `FormData` instance. The method is not exposed in public types.


- Migrate `http3` DNS lookup from `hickory-client` to `hickory-resolver` (#454)
  - `impit/src/http3.rs` only used hickory to fire a single HTTPS-record DNS query against a hard-coded `8.8.8.8:53` for h3 discovery. Migrated that to `hickory-resolver 0.26.1`, which: - pulls in the patched `hickory-proto 0.26.1`, - uses the system DNS config instead of hard-coding Google's resolver, - drops the manual background-task plumbing and `Drop` impl since the resolver manages its own connections.  API shifts handled along the way: `Record::data()` → `Record.data` (now a public field), `SVCB::svc_params()` → `SVCB.svc_params` (public field).


- Share browser-string resolution across sync and async clients (#481)
  - The sync and async Python clients each hand-maintained their own `browser` string → fingerprint match, and the two had drifted. The async client mapped `chrome124` to the `chrome_125` fingerprint (mislabeled, even though a real `chrome_124` fingerprint exists in the core database), while the sync client didn't recognize `chrome124` at all and fell back to `panic!("Unsupported browser")`, aborting across the FFI boundary instead of raising a catchable Python exception.  The bare `chrome` alias is intentionally left at `chrome_125` in both clients to preserve current behavior and the pinned JA4 test.


- Raise on mid-stream body errors instead of silent EOF (#482)
  - Mid-stream body errors (connection reset, truncated chunked transfer) were mapped to `StopIteration`/`StopAsyncIteration`, which signal normal end-of-iteration — so `for`/`async for` ended silently and callers processed partial bodies as complete (a silent data-integrity bug, [#475](https://github.com/apify/impit/issues/475)). Both sync and async iterators now propagate the classified `ImpitError` as a real exception and set the consumed/closed flags consistently with the clean-EOF branch.  Streamed truncation surfaces in reqwest as a `Decode`-kinded error rather than `Body`, so the existing unexpected-EOF classification missed it and fell through to the catch-all `HTTPError`. The guard is widened to cover `is_decode()`, so truncated streams now raise `RemoteProtocolError`, matching httpx.  Verified against a vanilla server that truncates the body mid-response; added sync + async regression tests.


#### Features

- Better errors for Node.JS bindings (#406)
  - Closes https://github.com/apify/impit/issues/397.


- Add new OkHTTP fingerprints  (#416)
  - Adds profiles for emulating the fingerprints of the OkHTTP library (JVM / Android HTTP client).


- Return the `vanillaFallback` option as an alternative for failing requests (#441)
  - The `vanillaFallback` option has been noop in a few of the latest versions of `impit`. The changes from this PR return this feature to support, e.g., servers with old TLS stacks that uncover some of the emulation discrepancies and cause the requests to fail.


- Add iOS 18 system TLS fingerprint (#465)
  - Adds an iOS 18 system TLS fingerprint as `Browser::Ios18` (string: `"ios18"`).



## py-0.12.0 - 2026-03-06

#### Features

- Add HTTP/2 SETTINGS fingerprinting (#386)
  - Adds custom HTTP2 profiles to the emulated browser fingerprints.  ---------


- Support `timeout=None` to disable timeout (#402)
  - Updates the timeout handling in Python. The default behaviour stays the same, but passing timeout=None now disables the timeout (either client-wide or for the current request). This aligns impit with how httpx handles timeouts.  ---------


#### Refactor

- Replace scraper with lol_html for HTML charset prescanning (#398)
  - Replaces `scraper` dependency with a more lightweight HTML parser from `lol_html`. Adds regression tests to ensure the behaviour stays the same.  ---------



## py-0.10.0 - 2026-02-10

#### Bug Fixes

- Proxy authenticates with empty password (#327)

- Authenticate with HTTPS proxy and HTTP target (#333)
  - Propagates upstream fixes from `reqwest`.


- Do not panic on missing attributes for encoding-related `meta` elements (#346)
  - Ignores encoding-related `meta` elements with missing `content` or `charset` attributes.  Related to #344


- Use the `rustls` `Verifier` / `CryptoProvider` cache with custom fingerprints (#371)
  - Speeds up repeated client instantiation and lowers the memory footprint if the custom fingerprints are used.  Related to #370


- Allow removing impersonated headers by passing empty string (#382)
  - Users can now remove impersonated headers (like `Sec-Fetch-User`) from requests by passing an empty string as the header value. When an empty string is provided, the header is filtered out before the request is sent.  This enables users, e.g., to manually control which `Sec-Fetch-*` headers should be included in their requests, addressing use cases where the default impersonated headers don't match the actual request context.


#### Features

- Enable `TRACE` method in the bindings (#328)
  - Unifies all clients by enabling the `trace` method in all of them. Required for type parity (`HttpMethod`) in downstream repositories - Crawlee et al.


- Use rustls-platform-verifier for system CA support (#357)
  - Replaces the static `webpki-roots` dependency with `rustls-platform-verifier` to enable `impit` to rely on the operating system's trust store.  ---------


- Custom fingerprint support (#366)
  - Extracts all fingerprinting logic (from e.g. the `rustls` patch) to `impit`. Prepares the codebase for new, non-hardcoded browser fingerprints.  Related to #99


- Add more Chrome and Firefox fingerprints (#367)
  - Adds more browser fingerprints and passes these to the Node.JS and Python bindings.



## py-0.9.2 - 2025-11-13

#### Bug Fixes

- Treat unexpected EOF error as `RemoteProtocolError` (#314)
  - Related to https://github.com/apify/apify-sdk-python/issues/672



## py-0.9.1 - 2025-11-13

#### Bug Fixes

- Raise Python exception on response body read error (#313)
  - Originally, Python Impit bindings would return a response with an empty body on a body read error. This didn't make much sense and caused issues in the downstream dependencies. Now we rethrow the error so it can be properly handled.  Closes https://github.com/apify/apify-sdk-python/issues/672



## py-0.9.0 - 2025-11-11

#### Bug Fixes

- Align anonymous client API with httpx (#310)

#### Refactor

- Introduce `ImpitRequest` struct for storing all request-related data (#307)
  - Refactors the `impit.make_request` method by splitting it into `build_request` and `send`.  Prerequisite for the solution to #227 proposed in https://github.com/apify/impit/issues/227#issuecomment-3184109259



## py-0.8.0 - 2025-10-22

#### Features

- Add support for Python 3.14, drop support for Python 3.9 and PyPy (#289)


## py-0.7.3 - 2025-10-17

#### Bug Fixes

- Do not panic on constructor param errors (#285)
  - Introduces better error handling for constructor parameter errors.


#### Features

- Add `json()` method for `Response` object (#277)
  - Adds a `Response.json()` method that parses `Response.text` using the native `json` module.



## py-0.6.1 - 2025-09-03

#### Features

- Include error message in `ConnectError` (#258)
  - Injects `cause` to the `ConnectError` display string. This allows for better error introspection in dependent packages.  Unblocks https://github.com/apify/crawlee-python/pull/1389/



## py-0.6.0 - 2025-09-02

#### Bug Fixes

- Fallback to HTTP/2 on HTTP3 DNS error (#255)
  - Makes DNS client in HTTP/3 record resolution optional. If the initial connection fails with `Error`, impit will return `false` for every call to `host_supports_h3` (unless, e.g. `alt-svc` header has been registered for this domain).


#### Features

- Add `local_address` option to `Impit` constructor (#225)
  - Adds a `local_address` option to the Impit HTTP client constructor across all language bindings (Rust, Python, and Node.js), allowing users to bind the client to a specific network interface. This feature is useful for testing purposes or when working with multiple network interfaces.



## py-0.5.4 - 2025-08-26

#### Features

- Improve error typing for certain HTTP errors (#250)
  - Improves error typing (mostly for Python version) on HTTP (network / server) errors and aligns the behaviour with HTTPX.



## py-0.5.3 - 2025-08-13

#### Bug Fixes

- Allow passing request body in all HTTP methods except `TRACE` (#238)


## py-0.5.2 - 2025-08-11

#### Bug Fixes

- Resolve blocking behavior in synchronous `Client` while reading response (#234)
  - - Resolve blocking behavior for read stream `Response` for `impit.Client`


#### Features

- Add constructor for `Response` (#233)
  - - Add constructor for `Response`. This can be useful when creating tests and mocks. - Allow to set custom attributes in `Response`  ---------



## py-0.5.1 - 2025-08-05

#### Bug Fixes

- Resolve blocking behavior in `impit.Client` during multithreaded operations (#230)


## py-0.5.0 - 2025-07-30

#### Bug Fixes

- Log correct timeout duration on `TimeoutException` (#222)
  - Logs the default `Impit`-instance-wide timeout if the request-specific timeout is missing.



## py-0.4.1 - 2025-07-22

#### Bug Fixes

- Exception class types are inheriting from `Exception` (#207)

- Fix cookies with attributes 'SameSite' and `domain` (#213)

#### Refactor

- Improve thread safety, make `Impit` `Sync` (#212)


## py-0.4.0 - 2025-07-07

#### Bug Fixes

- Fix the data types for `rest` and `version` with which the `Cookie` object is created. (#202)
  - Fix the data types with which a `Cookie` object is created, such as `rest` and `version`.


#### Features

- Add `(Async)Client.stream()` method for Python (#201)
  - Adds `stream` support for `Client` and `AsyncClient`. Implements for `Response` support for the `read` and `iter_bytes` methods, and their counterparts `aread` and `aiter_bytes` according to the `httpx` API  closes https://github.com/apify/impit/issues/142



## py-0.3.0 - 2025-06-25

#### Features

- Add support for custom cookie store implementations (#179)
  - Allows to pass custom cookie store implementations to the `ImpitBuilder` struct (using the new `with_cookie_store` builder method). Without passing the store implementation, `impit` client in both bindings is by default stateless (doesn't store cookies).  Enables implementing custom support for language-specific cookie stores (in JS and Python).


- Show the underlying `reqwest` error on unrecognized error type (#183)
  - Improve error logs in bindings by tunneling the lower-level `reqwest` errors through to binding users.


- Support `socks` proxy (#197)
  - Enables support for `socks` proxies to `impit-node`. This theoretically enables `socks` proxies for CLI and the Python binding as well, but this behaviour is untested due to a lack of working socks proxy server implementations in Python.


- Support for custom cookie stores for Python (#182)
  - Adds `cookie_jar` constructor parameter for `Client` and `AsyncClient` classes, accepting `http.cookiejar`'s `CookieJar` (or a custom implementation thereof, implementing at least `setCookie(cookie: http.cookiejar.Cookie)` and `iter(): Cookie[]`.  impit will write to and read from this custom cookie store.  Related to #123


- Client-scoped `headers` option (#200)
  - Adds `headers` setting to `Impit` constructor to set headers to be included in every request made by the built [`Impit`] instance.  This can be used to add e.g. custom user-agent or authorization headers that should be included in every request. These headers override the "impersonation" headers set by the `with_browser` method. In turn, these are overridden by request-specific `headers` setting.



## py-0.2.3 - 2025-05-09

#### Features

- Add context manager interface for `Client` and `AsyncClient` (#176)
  - Enables using `AsyncClient` and `Client` constructors as context managers inside the `with statements`. Closes #174



## py-0.2.2 - 2025-05-07

#### Bug Fixes

- Reenable HTTP/3 features in JS bindings (#57)
  - Recent package updates might have broken the `http3` feature in Node.JS bindings. This PR solves the underlying problems by building the reqwest's `Client` from within the `napi-rs`-managed `tokio` runtime.  Adds tests for `http3` usage from Node bindings.  Removes problematic Firefox header (`Connection` is not allowed in HTTP2 and HTTP3 requests or responses and together with `forceHttp3` was causing panics inside the Rust code).


- `response.encoding` contains the actual encoding (#119)
  - Parses the `content-type` header for the `charset` parameter.


- Case-insensitive request header deduplication (#127)
  - Refactors and simplifies the custom header logic. Custom headers now override "impersonated" browser headers regardless on the (upper|lower)case.


#### Features

- Add `ReadableStream` in `Response.body` (#28)
  - Accessing `Response.body` now returns an instance of JS `ReadableStream`. This API design matches the (browser) Fetch API spec. In order to correctly manage the `Response` consumption, the current codebase has been slightly refactored.  Note that the implementation relies on a prerelease version of the `napi-rs` tooling.


- Add Python bindings for `impit` (#49)
  - Adds Python bindings for `impit`. The interface is inspired by the `httpx` library (in the same way the JS bindings' interface is inspired by `fetch`, i.e. the end goal is to provide an almost drop-in replacement with extra features).


- Use `thiserror` for better error handling DX (#90)
  - Adds `std::error::Error` implementation with `thiserror`. This should improve the developer experience and error messages across the monorepo.


- Allow passing binary body to the `data` parameter (#103)
  - Following the discussion under #97 , this PR widens the type accepted by the `data` parameter.  `data` now accepts both a `dict[str,str]` and a binary representation of the request body. This PR intentionally doesn't update the recently added typings in the `impit.pyi` file, as this behaviour (accepting the binary body) is considered deprecated in the `httpx` library we use as the design master.


- Make `ImpitPyResponse` public for Python with the name `Response` (#110)
  - - Make `ImpitPyResponse` public for Python with the name `Response`. This will improve type handling and code navigation in the IDE  ---------


- Add `url` and `content` property for `Response`, smart `.text` decoding, options for redirects  (#122)
  - Adds `url`, `encoding` and `content` properties for `Response`. Decodes the response using the automatic encoding-determining algorithm. Adds redirect-related options.  ---------


- `response.headers` is a `Headers` object (#137)
  - Turns the `response.headers` from a `Record<string, string>` into a `Headers` object, matching the original `fetch` API.


- Better errors (#150)
  - Improves the error handling in `impit` and both the language bindings. Improves error messages.  For Python bindings, this PR adds the same exception types as in `httpx`.


- Switch to `Vec<(String, String)>` for request headers (#156)
  - Allows sending multiple request headers of the same name across all bindings / tools.  Broadens the `RequestInit.headers` type in the `impit-node` bindings. Closes #151



## 0.1.5 - 2025-01-23

#### Bug Fixes

- Use replacement character on invalid decode (#25)
  - When decoding incorrectly encoded content, the binding now panics and takes down the entire JS script. This change replaces the incorrect sequence with the U+FFFD replacement character (�) in the content. This is most often the desired behaviour.



<!-- generated by git-cliff -->

