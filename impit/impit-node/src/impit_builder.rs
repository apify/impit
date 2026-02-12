use std::time::Duration;

use impit::{
  fingerprint::BrowserFingerprint,
  impit::{ImpitBuilder, RedirectBehavior},
};

use napi::bindgen_prelude::Object;
use napi_derive::napi;

use crate::cookies::NodeCookieJar;

/// Operating system for TLS fingerprint matching.
///
/// When set alongside {@link ImpitOptions.browser}, the TLS fingerprint and HTTP headers
/// will match the specified OS. For example, setting `browser: 'firefox144'` with `os: 'windows'`
/// will produce a TLS handshake and User-Agent that looks like Firefox 144 running on Windows,
/// even if the actual machine runs Linux.
///
/// See {@link ImpitOptions.os} for usage.
#[napi(string_enum = "lowercase")]
pub enum OperatingSystem {
  Windows,
  Macos,
  Linux,
}

/// Supported browsers for emulation.
///
/// See {@link ImpitOptions.browser} for more details and usage.
#[napi(string_enum = "lowercase")]
pub enum Browser {
  Chrome,
  Chrome100,
  Chrome101,
  Chrome104,
  Chrome107,
  Chrome110,
  Chrome116,
  Chrome124,
  Chrome125,
  Chrome131,
  Chrome133,
  Chrome136,
  Chrome142,
  Firefox,
  Firefox128,
  Firefox133,
  Firefox135,
  Firefox144,
  Safari,
  Safari17,
  Safari18,
  Safari184,
  Safari172Ios,
  Edge,
  Edge131,
  Edge136,
}

/// Options for configuring an {@link Impit} instance.
///
/// These options allow you to customize the behavior of the Impit instance, including browser emulation, TLS settings, proxy configuration, timeouts, and more.
///
/// If no options are provided, default settings will be used.
///
/// See {@link Impit} for usage.
#[derive(Default)]
#[napi(object)]
pub struct ImpitOptions<'a> {
  /// What browser to emulate.
  ///
  /// @default `undefined` (no browser emulation)
  pub browser: Option<Browser>,
  /// Operating system to emulate for TLS fingerprint matching.
  ///
  /// When set, the TLS fingerprint and HTTP headers will match the specified OS,
  /// allowing cross-OS impersonation (e.g., Windows Chrome TLS on a Linux machine).
  ///
  /// @default `undefined` (uses the browser's default OS headers)
  pub os: Option<OperatingSystem>,
  /// Ignore TLS errors such as invalid certificates.
  ///
  /// @default `false`
  pub ignore_tls_errors: Option<bool>,
  /// Whether to fallback to a vanilla user-agent if the emulated browser
  /// is not supported by the target website.
  ///
  /// @default `false`
  pub vanilla_fallback: Option<bool>,
  /// Proxy URL to use for this Impit instance.
  ///
  /// Supports HTTP, HTTPS, SOCKS4 and SOCKS5 proxies.
  ///
  /// **Warning:** Not supported when HTTP/3 is enabled.
  ///
  /// @default `undefined` (no proxy)
  pub proxy_url: Option<String>,
  /// Default timeout for this Impit instance in milliseconds.
  pub timeout: Option<u32>,
  /// Enable HTTP/3 support.
  ///
  /// **Warning:** Proxies are not supported when HTTP/3 is enabled.
  ///
  /// @default `false`
  pub http3: Option<bool>,
  /// Whether to follow redirects or not.
  ///
  /// @default `true`
  pub follow_redirects: Option<bool>,
  /// Maximum number of redirects to follow.
  ///
  /// If this number is exceeded, the request will be rejected with an error.
  ///
  /// @default `10`
  pub max_redirects: Option<u32>,
  /// Pass a {@link https://github.com/salesforce/tough-cookie | `ToughCookie`} instance to Impit.
  ///
  /// This `impit` instance will use the provided cookie jar for both storing and retrieving cookies.
  ///
  /// @default `undefined` (no cookie jar, i.e., cookies are not stored or sent across requests)
  #[napi(
    ts_type = "{ setCookie: (cookie: string, url: string, cb?: any) => Promise<void> | void, getCookieString: (url: string) => Promise<string> | string }"
  )]
  pub cookie_jar: Option<Object<'a>>,
  /// Additional headers to include in every request made by this Impit instance.
  ///
  /// Can be an object, a Map, or an array of tuples or an instance of the {@link https://developer.mozilla.org/en-US/docs/Web/API/Headers | Headers} class.
  ///
  /// These headers override any browser impersonation headers (set via the {@link ImpitOptions.browser} option)
  /// and are in turn overridden by request-specific headers (set via {@link RequestInit.headers}).
  /// Header matching is **case-insensitive** — for example, setting `user-agent` here will override
  /// the impersonation `User-Agent` header.
  ///
  /// To remove an impersonated header, pass an empty string as the value.
  ///
  /// @default `undefined` (no additional headers)
  #[napi(ts_type = "Headers | Record<string, string> | [string, string][]")]
  pub headers: Option<Vec<(String, String)>>,
  /// Local address to bind the client to. Useful for testing purposes or when you want to bind the client to a specific network interface.
  ///
  /// Can be an IP address in the format `xxx.xxx.xxx.xxx` (for IPv4) or `ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff` (for IPv6).
  ///
  /// @default `undefined` (the OS will choose the local address)
  pub local_address: Option<String>,
  /// Seed for TLS fingerprint randomization.
  ///
  /// When set, the base browser TLS fingerprint is deterministically mutated
  /// (toggling optional cipher suites, extensions, and signature algorithms)
  /// so that each unique seed produces a unique JA4 TLS hash.
  ///
  /// This is critical for large-scale scraping: instead of ~10 static JA4
  /// hashes shared across millions of requests, each session can have its
  /// own unique TLS fingerprint.
  ///
  /// The seed should be derived from a session identifier (e.g., a hash of
  /// the session ID) so that the same session always gets the same fingerprint.
  ///
  /// @default `undefined` (no randomization, base browser fingerprint is used as-is)
  pub fingerprint_seed: Option<i64>,
}

impl From<Browser> for BrowserFingerprint {
  fn from(val: Browser) -> Self {
    resolve_fingerprint(val, None)
  }
}

/// Resolves a browser fingerprint, optionally with OS-specific headers and TLS matching.
/// When `os` is provided, uses `fingerprint_with_os()` to produce OS-matched headers.
fn resolve_fingerprint(browser: Browser, os: Option<&OperatingSystem>) -> BrowserFingerprint {
  let os_str = os.map(|o| match o {
    OperatingSystem::Windows => "windows",
    OperatingSystem::Macos => "macos",
    OperatingSystem::Linux => "linux",
  });

  match (browser, os_str) {
    // Chrome with OS
    (Browser::Chrome | Browser::Chrome142, Some(os)) => impit::fingerprint::database::chrome_142::fingerprint_with_os(os),
    (Browser::Chrome136, Some(os)) => impit::fingerprint::database::chrome_136::fingerprint_with_os(os),
    (Browser::Chrome133, Some(os)) => impit::fingerprint::database::chrome_133::fingerprint_with_os(os),
    (Browser::Chrome131, Some(os)) => impit::fingerprint::database::chrome_131::fingerprint_with_os(os),
    // Older Chrome versions without OS-specific support — fall through to default
    (Browser::Chrome100, _) => impit::fingerprint::database::chrome_100::fingerprint(),
    (Browser::Chrome101, _) => impit::fingerprint::database::chrome_101::fingerprint(),
    (Browser::Chrome104, _) => impit::fingerprint::database::chrome_104::fingerprint(),
    (Browser::Chrome107, _) => impit::fingerprint::database::chrome_107::fingerprint(),
    (Browser::Chrome110, _) => impit::fingerprint::database::chrome_110::fingerprint(),
    (Browser::Chrome116, _) => impit::fingerprint::database::chrome_116::fingerprint(),
    (Browser::Chrome124, _) => impit::fingerprint::database::chrome_124::fingerprint(),
    (Browser::Chrome125, _) => impit::fingerprint::database::chrome_125::fingerprint(),
    // Chrome without OS
    (Browser::Chrome | Browser::Chrome142, None) => impit::fingerprint::database::chrome_142::fingerprint(),
    (Browser::Chrome136, None) => impit::fingerprint::database::chrome_136::fingerprint(),
    (Browser::Chrome133, None) => impit::fingerprint::database::chrome_133::fingerprint(),
    (Browser::Chrome131, None) => impit::fingerprint::database::chrome_131::fingerprint(),

    // Firefox with OS
    (Browser::Firefox | Browser::Firefox144, Some(os)) => impit::fingerprint::database::firefox_144::fingerprint_with_os(os),
    (Browser::Firefox128, Some(os)) => impit::fingerprint::database::firefox_128::fingerprint_with_os(os),
    (Browser::Firefox133, Some(os)) => impit::fingerprint::database::firefox_133::fingerprint_with_os(os),
    (Browser::Firefox135, Some(os)) => impit::fingerprint::database::firefox_135::fingerprint_with_os(os),
    // Firefox without OS
    (Browser::Firefox | Browser::Firefox144, None) => impit::fingerprint::database::firefox_144::fingerprint(),
    (Browser::Firefox128, None) => impit::fingerprint::database::firefox_128::fingerprint(),
    (Browser::Firefox133, None) => impit::fingerprint::database::firefox_133::fingerprint(),
    (Browser::Firefox135, None) => impit::fingerprint::database::firefox_135::fingerprint(),

    // Safari with OS (macOS only — pass None for macos_version to use default)
    (Browser::Safari | Browser::Safari18, Some(_os)) => impit::fingerprint::database::safari_18_0::fingerprint_with_os("macos", None),
    (Browser::Safari17, Some(_os)) => impit::fingerprint::database::safari_17_0::fingerprint_with_os("macos", None),
    (Browser::Safari184, Some(_os)) => impit::fingerprint::database::safari_18_4::fingerprint_with_os("macos", None),
    (Browser::Safari172Ios, Some(_os)) => impit::fingerprint::database::safari_17_2_ios::fingerprint_with_os("macos", None),
    // Safari without OS
    (Browser::Safari | Browser::Safari18, None) => impit::fingerprint::database::safari_18_0::fingerprint(),
    (Browser::Safari17, None) => impit::fingerprint::database::safari_17_0::fingerprint(),
    (Browser::Safari184, None) => impit::fingerprint::database::safari_18_4::fingerprint(),
    (Browser::Safari172Ios, None) => impit::fingerprint::database::safari_17_2_ios::fingerprint(),

    // Edge with OS
    (Browser::Edge | Browser::Edge136, Some(os)) => impit::fingerprint::database::edge_136::fingerprint_with_os(os),
    (Browser::Edge131, Some(os)) => impit::fingerprint::database::edge_131::fingerprint_with_os(os),
    // Edge without OS
    (Browser::Edge | Browser::Edge136, None) => impit::fingerprint::database::edge_136::fingerprint(),
    (Browser::Edge131, None) => impit::fingerprint::database::edge_131::fingerprint(),
  }
}

impl ImpitOptions<'_> {
  pub fn into_builder(self) -> Result<ImpitBuilder<NodeCookieJar>, napi::Error> {
    let mut config: ImpitBuilder<NodeCookieJar> = ImpitBuilder::default();

    if let Some(browser) = self.browser {
      let mut fingerprint = resolve_fingerprint(browser, self.os.as_ref());
      if let Some(seed) = self.fingerprint_seed {
        fingerprint = fingerprint.randomize(seed as u64);
      }
      config = config.with_fingerprint(fingerprint);
    }
    if let Some(ignore_tls_errors) = self.ignore_tls_errors {
      config = config.with_ignore_tls_errors(ignore_tls_errors);
    }
    if let Some(vanilla_fallback) = self.vanilla_fallback {
      config = config.with_fallback_to_vanilla(vanilla_fallback);
    }
    if let Some(proxy_url) = self.proxy_url {
      config = config.with_proxy(proxy_url);
    }
    if let Some(timeout) = self.timeout {
      config = config.with_default_timeout(Duration::from_millis(timeout.into()));
    }
    if let Some(http3) = self.http3 {
      if http3 {
        config = config.with_http3();
      }
    }
    if let Some(headers) = self.headers {
      config = config.with_headers(headers);
    }

    // Always use ManualRedirect - redirects are handled in the JS layer
    config = config.with_redirect(RedirectBehavior::ManualRedirect);

    if let Some(local_address) = self.local_address {
      config = config
        .with_local_address(local_address)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    }

    Ok(config)
  }
}
