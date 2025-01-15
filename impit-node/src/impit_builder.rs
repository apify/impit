use std::time::Duration;

use impit::{
  emulation::Browser as ImpitBrowser,
  impit::{ImpitBuilder, RedirectBehavior},
};
use napi_derive::napi;

#[napi(string_enum)]
pub enum Browser {
  Chrome,
  Firefox,
}

impl From<Browser> for ImpitBrowser {
  fn from(val: Browser) -> Self {
    match val {
      Browser::Chrome => ImpitBrowser::Chrome,
      Browser::Firefox => ImpitBrowser::Firefox,
    }
  }
}

#[derive(Default)]
#[napi(object)]
pub struct ImpitOptions {
  pub browser: Option<Browser>,
  pub ignore_tls_errors: Option<bool>,
  pub vanilla_fallback: Option<bool>,
  pub proxy_url: Option<String>,
  /// Default timeout for this Impit instance in milliseconds.
  pub timeout: Option<u32>,
  /// Enable HTTP/3 support.
  pub http3: Option<bool>,
  /// Follow redirects.
  pub follow_redirects: Option<bool>,
  /// Maximum number of redirects to follow. Default is `10`.
  ///
  /// If this number is exceeded, the request will be rejected with an error.
  pub max_redirects: Option<u32>,
}

impl From<ImpitOptions> for ImpitBuilder {
  fn from(val: ImpitOptions) -> Self {
    let mut config = ImpitBuilder::default();
    if let Some(browser) = val.browser {
      config = config.with_browser(browser.into());
    }
    if let Some(ignore_tls_errors) = val.ignore_tls_errors {
      config = config.with_ignore_tls_errors(ignore_tls_errors);
    }
    if let Some(vanilla_fallback) = val.vanilla_fallback {
      config = config.with_fallback_to_vanilla(vanilla_fallback);
    }
    if let Some(proxy_url) = val.proxy_url {
      config = config.with_proxy(proxy_url);
    }
    if let Some(timeout) = val.timeout {
      config = config.with_default_timeout(Duration::from_millis(timeout.into()));
    }
    if let Some(http3) = val.http3 {
      if http3 {
        config = config.with_http3();
      }
    }

    let follow_redirects: bool = val.follow_redirects.unwrap_or(true);
    let max_redirects: usize = val.max_redirects.unwrap_or(10).try_into().unwrap();

    if !follow_redirects {
      config = config.with_redirect(RedirectBehavior::ManualRedirect);
    } else {
      config = config.with_redirect(RedirectBehavior::FollowRedirect(max_redirects));
    }

    config
  }
}
