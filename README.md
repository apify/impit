# impit | browser impersonation made simple

impit is a `rust` library that allows you to impersonate a browser and make requests to websites. It is built on top of `reqwest`, `rustls` and `tokio` and supports HTTP/1.1, HTTP/2, and HTTP/3.

The library provides a simple API for making requests to websites, and it also allows you to customize the request headers, use proxies, custom timeouts and more.

```rust
use impit::cookie::Jar;
use impit::{impit::Impit, fingerprint::database as fingerprints};

#[tokio::main]
async fn main() {
    let impit = Impit::<Jar>::builder()
        .with_fingerprint(fingerprints::firefox_144::fingerprint())
        .with_http3()
        .build()
        .unwrap();

    let response = impit
        .get(String::from("https://example.com"), None, None)
        .await;

    match response {
        Ok(response) => {
            println!("{}", response.text().await.unwrap());
        }
        Err(e) => {
            println!("{:#?}", e);
        }
    }
}
```

<!-- comparison:start -->
### Comparison

Sequential requests from a single client against the local HTTP/2 origin in [`benchmarks/`](benchmarks), 1 KiB JSON response, median of 11 runs of 2000 requests. Every client negotiated h2. Each one keeps a single connection warm for the whole run unless a footnote says otherwise. `Profiles` counts the distinct impersonation targets each public API accepts, ignoring aliases that resolve to another target. Python sizes are the platform wheel; Node.js sizes are what `npm install <package>` leaves on disk, transitive dependencies included.

**Python**

| Package | req/s | Wheel | Profiles | Backend |
| --- | --- | --- | --- | --- |
| [`primp`](https://github.com/deedy5/primp) | 6214 | 5.9 MB | —[^1] | Rust |
| [`rnet`](https://github.com/0x676e67/rnet) | 5104 | 3.7 MB | 75 | Rust |
| **`impit`** | 3780 | 4.2 MB | 20 | Rust |
| [`curl_cffi`](https://github.com/lexiforest/curl_cffi) | 3420 | 13.5 MB | 38 | C (libcurl) |
| [`tls-client`](https://github.com/FlorianREGAZ/Python-Tls-Client) | 3211 | 41.3 MB | 51 | Go |
| `httpx` (no impersonation) | 2323 | 0.1 MB | — | Python |

**Node.js**

| Package | req/s | Install | Profiles | Backend |
| --- | --- | --- | --- | --- |
| **`impit`** | 2289 | 8.7 MB | 20 | Rust |
| [`got-scraping`](https://github.com/apify/got-scraping) | 2234 | 4.7 MB | 3[^2] | Node.js TLS |
| [`node-tls-client`](https://github.com/Sahil1337/node-tls-client) | 1659[^3] | 30.7 MB | 63 | Go |
| [`cycletls`](https://github.com/Danny-Dasilva/CycleTLS) | 626[^4][^5] | 133.0 MB | —[^6] | Go subprocess |
| `undici` (no impersonation) | 5064 | 1.9 MB | — | Node.js |

Measured on linux-x64 with CPython 3.12.3 and Node.js v24.19.0 on 2026-08-31. Hardware moves these numbers around, so rerun `benchmarks/` yourself before drawing conclusions.

[^1]: `primp` does not expose its profile list, and an unknown name silently falls back to a random profile rather than erroring, so the set cannot be counted.
[^2]: `got-scraping` matches cipher suite and signature algorithm order only; it has no control over extension order, GREASE, or HTTP/2 `SETTINGS`. Its three profiles are not enumerable through the public API, so this count is hard-coded from its bundled cipher table.
[^3]: `node-tls-client` was erratic across runs — 900 to 3095 req/s — so its median says less than the others'.
[^4]: `cycletls` was erratic across runs — 409 to 640 req/s — so its median says less than the others'.
[^5]: `cycletls` opens a new connection for every request, so its figure includes a TLS handshake each time instead of reusing a warm one.
[^6]: `cycletls` is configured with a raw JA3 string instead of named profiles, so it has no fixed set to count.
<!-- comparison:end -->

### Other projects

If you'd prefer to use `impit` from a Node.js application, check out the [`impit-node`](https://github.com/apify/impit/tree/master/impit-node) folder, or download the package from npm:

```bash
npm install impit
```

The interface is the same as the native [`fetch`](https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API/Using_Fetch).

```typescript
import { Impit } from 'impit';

// Set up the Impit instance
const impit = new Impit({
    browser: "chrome", // or "firefox"
    proxyUrl: "http://localhost:8080",
    ignoreTlsErrors: true,
});

// Use the `fetch` method as you would with the built-in `fetch` function
const response = await impit.fetch("https://example.com");

console.log(response.status);
console.log(response.headers);
console.log(await response.text());
// console.log(await response.json());
// ...
```

### Usage from Rust

Technically speaking, the `impit` project is a somewhat thin wrapper around `reqwest` that provides a more ergonomic API for making requests to websites.
The real strength of `impit` is that it uses patched versions of `rustls` and other libraries that allow it to make browser-like requests.

Note that if you want to use this library in your rust project, you have to add the following dependencies to your `Cargo.toml` file:
```toml
[dependencies]
impit = { git="https://github.com/apify/impit.git", branch="master" }

[patch.crates-io]
rustls = { git="https://github.com/apify/rustls.git" }
h2 = { git="https://github.com/apify/h2.git" }
```

Without the patched dependencies, the project won't build.

Note that you also have to build your project with `rustflags = "--cfg reqwest_unstable"`, otherwise, the build will also fail.
This is because `impit` uses unstable features of `reqwest` (namely `http3` support), which are not available in the stable version of the library.
