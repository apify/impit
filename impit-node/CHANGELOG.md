# Changelog

All notable changes to this project will be documented in this file.


## js-0.7.6 - 2026-01-05

#### Bug Fixes

- Do not panic on missing attributes for encoding-related `meta` elements (#346)
  - Ignores encoding-related `meta` elements with missing `content` or `charset` attributes.  Related to #344


- Throw `Error` on invalid header value (do not panic) (#347)
  - Unparseable response header values now only return `Error` in the Node bindings instead of panicking and killing the process.  Related to #344



## js-0.7.5 - 2025-12-17

#### Bug Fixes

- Do not drop request-scoped options (`timeout` and `forceHttp3`) (#340)


## js-0.7.4 - 2025-12-09

#### Bug Fixes

- Authenticate with HTTPS proxy and HTTP target (#333)
  - Propagates upstream fixes from `reqwest`.



## js-0.7.3 - 2025-12-03

#### Features

- Enable `TRACE` method in the bindings (#328)
  - Unifies all clients by enabling the `trace` method in all of them. Required for type parity (`HttpMethod`) in downstream repositories - Crawlee et al.



## js-0.7.2 - 2025-12-02

#### Bug Fixes

- Raise Python exception on response body read error (#313)
  - Originally, Python Impit bindings would return a response with an empty body on a body read error. This didn't make much sense and caused issues in the downstream dependencies. Now we rethrow the error so it can be properly handled.  Closes https://github.com/apify/apify-sdk-python/issues/672


- Treat unexpected EOF error as `RemoteProtocolError` (#314)
  - Related to https://github.com/apify/apify-sdk-python/issues/672


- Proxy authenticates with empty password (#327)


## js-0.7.1 - 2025-11-11

#### Bug Fixes

- Align anonymous client API with httpx (#310)


## js-0.7.0 - 2025-11-07

#### Features

- Align `Impit.fetch` with `fetch` interface (#309)
  - Enables passing `Request` and `URL` instances to `Impit.fetch`.   Related to #227


#### Refactor

- Introduce `ImpitRequest` struct for storing all request-related data (#307)
  - Refactors the `impit.make_request` method by splitting it into `build_request` and `send`.  Prerequisite for the solution to #227 proposed in https://github.com/apify/impit/issues/227#issuecomment-3184109259



## js-0.6.1 - 2025-10-22

#### Bug Fixes

- Downgrade `napi-rs` tooling to fix random Windows hang ups (#296)


## js-0.6.0 - 2025-10-16

#### Bug Fixes

- Fallback to HTTP/2 on HTTP3 DNS error (#255)
  - Makes DNS client in HTTP/3 record resolution optional. If the initial connection fails with `Error`, impit will return `false` for every call to `host_supports_h3` (unless, e.g. `alt-svc` header has been registered for this domain).


- Do not panic on constructor param errors (#285)
  - Introduces better error handling for constructor parameter errors.


#### Features

- Improve error typing for certain HTTP errors (#250)
  - Improves error typing (mostly for Python version) on HTTP (network / server) errors and aligns the behaviour with HTTPX.


- Add `local_address` option to `Impit` constructor (#225)
  - Adds a `local_address` option to the Impit HTTP client constructor across all language bindings (Rust, Python, and Node.js), allowing users to bind the client to a specific network interface. This feature is useful for testing purposes or when working with multiple network interfaces.


- Include error message in `ConnectError` (#258)
  - Injects `cause` to the `ConnectError` display string. This allows for better error introspection in dependent packages.  Unblocks https://github.com/apify/crawlee-python/pull/1389/



## js-0.5.4 - 2025-08-13

#### Bug Fixes

- Allow passing request body in all HTTP methods except `TRACE` (#238)


## js-0.5.3 - 2025-07-24

#### Bug Fixes

- Log correct timeout duration on `TimeoutException` (#222)
  - Logs the default `Impit`-instance-wide timeout if the request-specific timeout is missing.


#### Refactor

- Improve thread safety, make `Impit` `Sync` (#212)


## js-0.5.2 - 2025-06-25

#### Features

- Client-scoped `headers` option (#200)
  - Adds `headers` setting to `Impit` constructor to set headers to be included in every request made by the built [`Impit`] instance.  This can be used to add e.g. custom user-agent or authorization headers that should be included in every request. These headers override the "impersonation" headers set by the `with_browser` method. In turn, these are overridden by request-specific `headers` setting.



## js-0.5.1 - 2025-06-11

#### Bug Fixes

- Solve memory leak on response read (#191)
  - Memory leak in `napi-rs`'s implementation of `ReadableStream` was causing `impit` to leak small amounts of memory on response read (`.text()`, `.json()`, `.bytes()` etc.).


#### Features

- Support `socks` proxy (#197)
  - Enables support for `socks` proxies to `impit-node`. This theoretically enables `socks` proxies for CLI and the Python binding as well, but this behaviour is untested due to a lack of working socks proxy server implementations in Python.



## js-0.5.0 - 2025-05-29

#### Bug Fixes

- Support `null` request payload, don't modify options (#190)
  - Removes parameter reassignment code smell. Fixes errors on `null` (or other nullish, but not expected) body.


#### Features

- Support for custom cookie stores for Node.JS (#181)
  - Adds `cookieJar` constructor parameter for `Impit` class, accepting `tough-cookie`'s `CookieJar` (or a custom implementation thereof, implementing at least `setCookie(cookie: string, url: string)` and `getCookieString(url: string)`).  `impit` will write to and read from this custom cookie store.  Related to #123



## js-0.4.7 - 2025-05-20

#### Features

- Add `resp.arrayBuffer`, improve Node <22 compatibility (#188)
  - Adds new `response.arrayBuffer` method ([MDN](https://developer.mozilla.org/en-US/docs/Web/API/Response/arrayBuffer)) and implements `response.bytes` using `arrayBuffer()` to improve compatibility with Node < 22 (`.bytes()` support was rather experimental until this version).



## js-0.4.6 - 2025-05-16

#### Features

- Add support for custom cookie store implementations (#179)
  - Allows to pass custom cookie store implementations to the `ImpitBuilder` struct (using the new `with_cookie_store` builder method). Without passing the store implementation, `impit` client in both bindings is by default stateless (doesn't store cookies).  Enables implementing custom support for language-specific cookie stores (in JS and Python).


- Show the underlying `reqwest` error on unrecognized error type (#183)
  - Improve error logs in bindings by tunneling the lower-level `reqwest` errors through to binding users.



## js-0.4.5 - 2025-05-07

#### Bug Fixes

- Render helpful error message on import errors (#171)
  - Wraps the original `napi-rs` `ENOENT` error with a more descriptive error message. Understands `IMPIT_VERBOSE` envvar for printing the original error message.



## js-0.4.2 - 2025-04-30

#### Features

- Better errors (#150)
  - Improves the error handling in `impit` and both the language bindings. Improves error messages.  For Python bindings, this PR adds the same exception types as in `httpx`.


- Switch to `Vec<(String, String)>` for request headers (#156)
  - Allows sending multiple request headers of the same name across all bindings / tools.  Broadens the `RequestInit.headers` type in the `impit-node` bindings. Closes #151


- Accept more `request.body` types (match `fetch` API) (#157)


## js-0.4.0 - 2025-04-23

#### Bug Fixes

- Set `response.text()` return type to `string` (#136)
  - The type definitions for `response.text()` return type used `String` (see capital S) instead of `string` (like `fetch` API does). Note that this is a strictly type-level issue and simple cast (or type change, in this case) solves this.


- Do not publish ambient `const enum`s (#144)
  - Switches from exporting `const enum`s from the `.d.ts` file ([which is problematic](https://www.typescriptlang.org/docs/handbook/enums.html#const-enum-pitfalls)) to exporting union types.  Makes the `browser` option accept lowercase browser names.


#### Features

- `response.headers` is a `Headers` object (#137)
  - Turns the `response.headers` from a `Record<string, string>` into a `Headers` object, matching the original `fetch` API.



## js-0.3.2 - 2025-04-15

#### Bug Fixes

- `response.encoding` contains the actual encoding (#119)
  - Parses the `content-type` header for the `charset` parameter.


- Case-insensitive request header deduplication (#127)
  - Refactors and simplifies the custom header logic. Custom headers now override "impersonated" browser headers regardless on the (upper|lower)case.


#### Features

- Add `url` and `content` property for `Response`, smart `.text` decoding, options for redirects  (#122)
  - Adds `url`, `encoding` and `content` properties for `Response`. Decodes the response using the automatic encoding-determining algorithm. Adds redirect-related options.  ---------



## js-0.3.1 - 2025-03-28

#### Bug Fixes

- Bundle native binary in `linux-arm64` packages (#100)
  - Followup to #94 , the `impit-linux-arm64-(gnu|musl)` published packages didn't have the native binary included due to naming mismatch.



## js-0.3.0 - 2025-03-28

#### Features

- Use `thiserror` for better error handling DX (#90)
  - Adds `std::error::Error` implementation with `thiserror`. This should improve the developer experience and error messages across the monorepo.



## js-0.2.5 - 2025-02-25

#### Bug Fixes

- Enable ESM named imports from `impit` packages (#78)
  - Lists the "named" exports from the `impit` package verbatim so they can be imported with  ```typescript import { Impit } from 'impit';  ```  from ESM packages without causing the following error:  ``` import { Impit } from "impit";          ^^^^^ SyntaxError: Named export 'Impit' not found. The requested module 'impit' is a CommonJS module, which may not support all module.exports as named exports. CommonJS modules can always be imported via the default export, for example using:  import pkg from 'impit'; const { Impit } = pkg; ```



## js-0.2.4 - 2025-02-25

#### Bug Fixes

- Allow `impit` usage from ESM (#74)
  - Renames native code import to mitigate naming collision with global `exports`.



## js-0.2.3 - 2025-02-21

#### Bug Fixes

- Reenable HTTP/3 features in JS bindings (#57)
  - Recent package updates might have broken the `http3` feature in Node.JS bindings. This PR solves the underlying problems by building the reqwest's `Client` from within the `napi-rs`-managed `tokio` runtime.  Adds tests for `http3` usage from Node bindings.  Removes problematic Firefox header (`Connection` is not allowed in HTTP2 and HTTP3 requests or responses and together with `forceHttp3` was causing panics inside the Rust code).


- Decode response charset with headers and BOM sniffing (#68)
  - Calling  ```typescript const res = await impit.fetch('https://resource-with-non-utf8-response'); const text = await res.text(); ```  results in mangling the response text (as Impit expected only `utf-8`).  This PR fixes this by patching the `Impit.fetch()` return value. While it would be possible to implement this fully in Rust, this implementation is much simpler and opens the door for other patches like this.


#### Security

- Regenerate Node.JS lockfile with latest dependencies (#53)
  - Closes Dependabot security warnings.



## js-0.2.1 - 2025-02-11

#### Features

- Add `.url` field to `impit-node` Response (#43)
  - Adds [Response.url](https://developer.mozilla.org/en-US/docs/Web/API/Response/url) property to the `ImpitResponse` struct / object.



## js-0.2.0 - 2025-02-07

#### Features

- Add `ReadableStream` in `Response.body` (#28)
  - Accessing `Response.body` now returns an instance of JS `ReadableStream`. This API design matches the (browser) Fetch API spec. In order to correctly manage the `Response` consumption, the current codebase has been slightly refactored.  Note that the implementation relies on a prerelease version of the `napi-rs` tooling.


#### Security

- Bump `vitest` version (#31)
  - Solves Dependabot security warnings.



## 0.1.5 - 2025-01-23

#### Bug Fixes

- Use replacement character on invalid decode (#25)
  - When decoding incorrectly encoded content, the binding now panics and takes down the entire JS script. This change replaces the incorrect sequence with the U+FFFD replacement character (�) in the content. This is most often the desired behaviour.



## js-0.1.1 - 2025-01-15


<!-- generated by git-cliff -->

