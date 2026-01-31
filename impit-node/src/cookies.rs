use reqwest::Url;

use reqwest::cookie::CookieStore;
use reqwest::header::HeaderValue;

/// A no-op cookie store implementation.
///
/// This struct is only used to silence compiler errors when building `impit` Node bindings,
/// as cookies are handled in JavaScript when using `impit` from Node.js.
pub struct NodeCookieJar;

impl CookieStore for NodeCookieJar {
  fn set_cookies(
    &self,
    _cookie_headers: &mut dyn Iterator<Item = &reqwest::header::HeaderValue>,
    _url: &Url,
  ) {
  }

  fn cookies(&self, _url: &Url) -> Option<HeaderValue> {
    None
  }
}
