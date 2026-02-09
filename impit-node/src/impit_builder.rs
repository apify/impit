use std::time::Duration;

use impit::{
  fingerprint::BrowserFingerprint,
  impit::{ImpitBuilder, RedirectBehavior},
};

use napi::bindgen_prelude::Object;
use napi_derive::napi;

use crate::cookies::NodeCookieJar;

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
  Chrome136,
  Chrome142,
  Firefox,
  Firefox128,
  Firefox133,
  Firefox135,
  Firefox144,
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
}

impl From<Browser> for BrowserFingerprint {
  fn from(val: Browser) -> Self {
    match val {
      Browser::Chrome | Browser::Chrome124 => {
        impit::fingerprint::database::chrome_124::fingerprint()
      }
      Browser::Chrome100 => impit::fingerprint::database::chrome_100::fingerprint(),
      Browser::Chrome101 => impit::fingerprint::database::chrome_101::fingerprint(),
      Browser::Chrome104 => impit::fingerprint::database::chrome_104::fingerprint(),
      Browser::Chrome107 => impit::fingerprint::database::chrome_107::fingerprint(),
      Browser::Chrome110 => impit::fingerprint::database::chrome_110::fingerprint(),
      Browser::Chrome116 => impit::fingerprint::database::chrome_116::fingerprint(),
      Browser::Chrome125 => impit::fingerprint::database::chrome_125::fingerprint(),
      Browser::Chrome131 => impit::fingerprint::database::chrome_131::fingerprint(),
      Browser::Chrome136 => impit::fingerprint::database::chrome_136::fingerprint(),
      Browser::Chrome142 => impit::fingerprint::database::chrome_142::fingerprint(),
      Browser::Firefox | Browser::Firefox128 => {
        impit::fingerprint::database::firefox_128::fingerprint()
      }
      Browser::Firefox133 => impit::fingerprint::database::firefox_133::fingerprint(),
      Browser::Firefox135 => impit::fingerprint::database::firefox_135::fingerprint(),
      Browser::Firefox144 => impit::fingerprint::database::firefox_144::fingerprint(),
    }
  }
}

impl ImpitOptions<'_> {
  pub fn into_builder(self) -> Result<ImpitBuilder<NodeCookieJar>, napi::Error> {
    let mut config: ImpitBuilder<NodeCookieJar> = ImpitBuilder::default();

    if let Some(browser) = self.browser {
      config = config.with_fingerprint(browser.into());
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
