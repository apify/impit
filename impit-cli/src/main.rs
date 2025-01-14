use std::ffi::OsString;

use clap::{Parser, ValueEnum};
use impit::{
    emulation::Browser as ImpitBrowser,
    impit::{Impit, RedirectBehavior},
    request::RequestOptions,
};

mod headers;

#[derive(Parser, Debug, Clone, Copy, ValueEnum)]
enum Browser {
    Chrome,
    Firefox,
    Impit,
}

#[derive(Parser, Debug, Clone, Copy, ValueEnum)]
enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
    TRACE,
}

/// CLI interface for the impit library.
/// Something like CURL for libcurl, for making impersonated HTTP(2) requests.
#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct CliArgs {
    /// Method to use for the request.
    #[arg(short = 'X', long, default_value = "get")]
    method: Method,

    /// HTTP headers to add to the request.
    #[arg(short = 'H', long)]
    headers: Vec<String>,

    /// What browser to use for the request.
    #[arg(short = 'A', long, default_value = "impit")]
    impersonate: Browser,

    /// If set, impit will ignore TLS errors.
    #[arg(short = 'k', long, action)]
    ignore_tls_errors: bool,

    /// If set, impit will fallback to vanilla HTTP if the impersonated browser fails.
    #[arg(short = 'f', long, action)]
    fallback: bool,

    /// Proxy to use for the request.
    #[arg(short = 'x', long = "proxy")]
    proxy: Option<String>,

    /// Maximum time in seconds to wait for the request to complete.
    #[arg(short = 'm', long = "max-time")]
    max_time: Option<u64>,

    /// Data to send with the request.
    #[arg(short, long)]
    data: Option<OsString>,

    /// Enforce the use of HTTP/3 for the request. Note that if the server does not support HTTP/3, the request will fail.
    #[arg(long = "http3-only", action)]
    http3_prior_knowledge: bool,

    /// Enable the use of HTTP/3. This will attempt to use HTTP/3, but fall back to earlier versions of HTTP if the server does not support it.
    #[arg(long = "http3", action)]
    enable_http3: bool,

    /// Follow redirects
    #[arg(short = 'L', long = "location", action)]
    follow_redirects: bool,

    /// Follow redirects
    #[arg(long = "max-redirs", default_value = "50")]
    maximum_redirects: usize,

    /// URL of the request to make
    url: String,
}

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();

    let mut client = Impit::builder()
        .with_ignore_tls_errors(args.ignore_tls_errors)
        .with_fallback_to_vanilla(args.fallback);

    client = match args.impersonate {
        Browser::Chrome => client.with_browser(ImpitBrowser::Chrome),
        Browser::Firefox => client.with_browser(ImpitBrowser::Firefox),
        Browser::Impit => client,
    };

    if args.proxy.is_some() {
        client = client.with_proxy(args.proxy.unwrap())
    }

    if args.enable_http3 || args.http3_prior_knowledge {
        client = client.with_http3()
    }

    if args.follow_redirects {
        client = client.with_redirect(RedirectBehavior::FollowRedirect(args.maximum_redirects));
    } else {
        client = client.with_redirect(RedirectBehavior::ManualRedirect);
    }

    let body: Option<Vec<u8>> = match args.data {
        Some(data) => Some(data.into_string().unwrap().into_bytes()),
        None => None,
    };

    let mut client = client.build();

    let timeout = match args.max_time {
        Some(time) => Some(std::time::Duration::from_secs(time)),
        None => None,
    };

    let options = RequestOptions {
        headers: headers::process_headers(args.headers),
        http3_prior_knowledge: args.http3_prior_knowledge,
        timeout,
    };

    let response = match args.method {
        Method::GET => client.get(args.url, Some(options)).await.unwrap(),
        Method::POST => client.post(args.url, body, Some(options)).await.unwrap(),
        Method::PUT => client.put(args.url, body, Some(options)).await.unwrap(),
        Method::DELETE => client.delete(args.url, Some(options)).await.unwrap(),
        Method::PATCH => client.patch(args.url, body, Some(options)).await.unwrap(),
        Method::HEAD => client.head(args.url, Some(options)).await.unwrap(),
        Method::OPTIONS => client.options(args.url, Some(options)).await.unwrap(),
        Method::TRACE => client.trace(args.url, Some(options)).await.unwrap(),
    };

    print!("{}", response.text().await.unwrap());
}
