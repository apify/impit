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

Median of 11 runs of 2000 sequential requests to a local HTTP/2 server, 1 KiB JSON responses over one warm connection. `Profiles` counts the impersonation targets each API exposes.

**Python**

| Package | req/s | Wheel | Profiles | Backend |
| --- | --- | --- | --- | --- |
| [`primp`](https://github.com/deedy5/primp) | 6312 | 5.9 MB | n/a | Rust |
| [`rnet`](https://github.com/0x676e67/rnet) | 5093 | 3.7 MB | 75 | Rust |
| **`impit`** | 3808 | 4.2 MB | 20 | Rust |
| [`curl_cffi`](https://github.com/lexiforest/curl_cffi) | 3428 | 13.5 MB | 38 | C (libcurl) |
| [`tls-client`](https://github.com/FlorianREGAZ/Python-Tls-Client) | 3223 | 41.3 MB | 51 | Go |
| `httpx` (no impersonation) | 2311 | 0.1 MB | — | Python |

**Node.js**

| Package | req/s | Install | Profiles | Backend |
| --- | --- | --- | --- | --- |
| [`got-scraping`](https://github.com/apify/got-scraping) | 2332 | 4.7 MB | 3 | Node.js TLS |
| **`impit`** | 2260 | 8.7 MB | 20 | Rust |
| [`cycletls`](https://github.com/Danny-Dasilva/CycleTLS) | 610 | 133.0 MB | raw JA3 | Go subprocess |
| `undici` (no impersonation) | 4661 | 1.9 MB | — | Node.js |

Measured by [`benchmarks/`](benchmarks) on linux-x64, 2026-09-03. Rerun it on your own hardware.
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
