use std::time::Duration;

use impit::{
  emulation::Browser as ImpitBrowser,
  impit::{ImpitBuilder, RedirectBehavior},
};
use napi::{bindgen_prelude::Object, Env};
use napi_derive::napi;

use crate::request::NodeCookieJar;

#[napi(string_enum = "lowercase")]
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
pub struct ImpitOptions<'a> {
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
  /// Pass a ToughCookie instance to Impit.
  #[napi(
    ts_type = "{ setCookie: (cookie: string, url: string, cb?: any) => Promise<void> | void, getCookieString: (url: string) => Promise<string> | string }"
  )]
  pub cookie_jar: Option<Object<'a>>,
}

impl ImpitOptions<'_> {
  pub fn into_builder(self, env: &Env) -> Result<ImpitBuilder<NodeCookieJar>, napi::Error> {
    let mut config = ImpitBuilder::default();
    if let Some(browser) = self.browser {
      config = config.with_browser(browser.into());
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

    let follow_redirects: bool = self.follow_redirects.unwrap_or(true);
    let max_redirects: usize = self.max_redirects.unwrap_or(10).try_into().unwrap();

    if !follow_redirects {
      config = config.with_redirect(RedirectBehavior::ManualRedirect);
    } else {
      config = config.with_redirect(RedirectBehavior::FollowRedirect(max_redirects));
    }

    Ok(config)
  }
}
