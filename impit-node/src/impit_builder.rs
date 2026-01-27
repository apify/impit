use std::time::Duration;

use impit::{
  fingerprint::BrowserFingerprint,
  impit::{ImpitBuilder, RedirectBehavior},
};
use napi::{bindgen_prelude::Object, Env};
use napi_derive::napi;

use crate::cookies::NodeCookieJar;

/// Supported browsers for emulation.
///
/// See {@link ImpitOptions.browser} for more details and usage.
#[napi(string_enum = "lowercase")]
pub enum Browser {
  Chrome,
  Firefox,
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
  /// @default `undefined` (no proxy)
  pub proxy_url: Option<String>,
  /// Default timeout for this Impit instance in milliseconds.
  pub timeout: Option<u32>,
  /// Enable HTTP/3 support.
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
      Browser::Chrome => impit::fingerprint::database::chrome_125::fingerprint(),
      Browser::Firefox => impit::fingerprint::database::firefox_128::fingerprint(),
    }
  }
}

impl ImpitOptions<'_> {
  pub fn into_builder(self, env: &Env) -> Result<ImpitBuilder<NodeCookieJar>, napi::Error> {
    let mut config = ImpitBuilder::default();
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
    if let Some(cookie_jar) = self.cookie_jar {
      match NodeCookieJar::new(env, cookie_jar) {
        Ok(cookie_jar) => {
          config = config.with_cookie_store(cookie_jar);
        }
        Err(e) => return Err(e),
      }
    }
    if let Some(headers) = self.headers {
      config = config.with_headers(headers);
    }

    let follow_redirects: bool = self.follow_redirects.unwrap_or(true);
    let max_redirects: usize = self.max_redirects.unwrap_or(10).try_into().unwrap();

    if !follow_redirects {
      config = config.with_redirect(RedirectBehavior::ManualRedirect);
    } else {
      config = config.with_redirect(RedirectBehavior::FollowRedirect(max_redirects));
    }

    if let Some(local_address) = self.local_address {
      config = config
        .with_local_address(local_address)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    }

    Ok(config)
  }
}
