# Changelog

All notable changes to this project will be documented in this file.


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

