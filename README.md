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

### Comparison

Sequential requests from a single client against a local Node.js HTTP/2 server (1 KiB JSON body, keep-alive), best of 11 runs of 2000 requests, pinned to one core. Profile counts are the distinct versioned impersonation targets exposed by the public API. Numbers are indicative — rerun them on your own hardware before drawing conclusions.

**Python**

| Package | Version | req/s | Wheel | Profiles | Backend |
| --- | --- | --- | --- | --- | --- |
| [`rnet`](https://github.com/0x676e67/rnet) | 2.4.2 | 3808 | 3.7 MB | 75 | Rust |
| [`primp`](https://github.com/deedy5/primp) | 1.3.1 | 3547 | 5.3 MB | n/a[^1] | Rust |
| **`impit`** | 0.14.0 | 2885 | 4.3 MB | 20 | Rust |
| [`tls-client`](https://github.com/FlorianREGAZ/Python-Tls-Client) | 1.0.1 | 1831 | 41.3 MB | 51 | Go |
| [`curl_cffi`](https://github.com/lexiforest/curl_cffi) | 0.16.2 | 1548 | 13.5 MB | 38 | C (libcurl) |
| `httpx` (no impersonation) | 0.28.1 | 759 | 0.1 MB | — | Python |

**Node.js**

| Package | Version | req/s | Install | Profiles | Backend |
| --- | --- | --- | --- | --- | --- |
| **`impit`** | 0.14.4 | 1353 | 8.7 MB | 20 | Rust |
| [`got-scraping`](https://github.com/apify/got-scraping) | 4.2.1 | 1149[^2] | 5.2 MB | 3[^3] | Node.js TLS |
| [`node-tls-client`](https://github.com/Sahil1337/node-tls-client) | 2.1.0 | 901 | 31.1 MB | 63 | Go |
| [`cycletls`](https://github.com/Danny-Dasilva/CycleTLS) | 2.0.5 | 287 | 133.3 MB | raw JA3 | Go subprocess |
| `undici` (no impersonation) | 8.10.0 | 2030 | 2.0 MB | — | Node.js |

[^1]: `primp` accepts arbitrary version strings and snaps to the nearest shipped profile, so the set is not enumerable through the public API.
[^2]: Over HTTP/1.1 — with HTTP/2 enabled the server closes the session with `GOAWAY` after roughly a thousand requests.
[^3]: Cipher suite and signature algorithm order only; no control over extension order, GREASE, or HTTP/2 `SETTINGS`.

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
