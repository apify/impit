//! # impit | browser impersonation made simple
//!
//! impit is a `rust` library that allows you to impersonate a browser and make requests to websites. It is built on top of `reqwest`, `rustls` and `tokio` and supports HTTP/1.1, HTTP/2, and HTTP/3.
//!
//! The library provides a simple API for making requests to websites, and it also allows you to customize the request headers, use proxies, custom timeouts and more.
//!
//! ```rust
//! use impit::impit::Impit;
//! use impit::emulation::Browser;
//!  
//! #[tokio::main]
//! async fn main() {
//!    let mut impit = Impit::builder()
//!        .with_browser(Browser::Firefox)
//!        .with_http3()
//!        .build();
//!
//!    let response = impit.get(String::from("https://example.com"), None).await;
//!
//!    match response {
//!        Ok(response) => {
//!            println!("{}", response.text().await.unwrap());
//!        }
//!        Err(e) => {
//!            println!("{:#?}", e);
//!        }
//!    }
//! }
//! ```
//!
//! ### Other projects
//!
//! If you are looking for a command-line tool that allows you to make requests to websites, check out the [`impit-cli`](https://github.com/apify/impit/tree/master/impit-cli) project.
//!
//! If you'd prefer to use `impit` from a Node.js application, check out the [`impit-node`](https://github.com/apify/impit/tree/master/impit-node) folder, or download the package from npm:
//! ```bash
//! npm install impit
//! ```
//!
//! ### Usage from Rust
//!
//! Technically speaking, the `impit` project is a somewhat thin wrapper around `reqwest` that provides a more ergonomic API for making requests to websites.
//! The real strength of `impit` is that it uses patched versions of `rustls` and other libraries that allow it to make browser-like requests.
//!
//! Note that if you want to use this library in your rust project, you have to add the following dependencies to your `Cargo.toml` file:
//! ```toml
//! [dependencies]
//! impit = { git="https://github.com/apify/impit.git", branch="master" }
//!
//! [patch.crates-io]
//! rustls = { git="https://github.com/apify/rustls.git" }
//! h2 = { git="https://github.com/apify/h2.git" }
//! ```
//!
//! Without the patched dependencies, the project won't build.
//!
//! Note that you also have to build your project with `rustflags = "--cfg reqwest_unstable"`, otherwise, the build will also fail.
//! This is because `impit` uses unstable features of `reqwest` (namely `http3` support), which are not available in the stable version of the library.

#![deny(unused_crate_dependencies)]
mod http_headers;
mod response_parsing;
mod tls;

pub(crate) mod http3;

/// Main module that contains the `Impit` struct and its methods.
pub mod impit;

/// Customizing request options.
pub mod request;

/// Errors and error handling.
pub mod errors;

/// Contains browser emulation-related types and functions.
pub mod emulation {

    /// The `Browser` enum is used to specify the browser that should be impersonated.
    ///
    /// It can be passed as a parameter to [`ImpitBuilder::with_browser`](crate::impit::ImpitBuilder::with_browser)
    /// to use the browser emulation with the built [`Impit`](crate::impit::Impit) instance.
    #[derive(PartialEq, Debug, Clone, Copy, Default)]
    pub enum Browser {
        #[default]
        Chrome,
        Firefox,
    }
}

/// Various utility functions and types.
pub mod utils {
    pub use crate::response_parsing::decode;
    pub use crate::response_parsing::determine_encoding;
    pub use crate::response_parsing::ContentType;
    pub use encoding::all as encodings;
}
