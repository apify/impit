use reqwest::Url;

use reqwest::cookie::CookieStore;
use reqwest::header::HeaderValue;

/// A no-op cookie store implementation.
///
/// This struct is used when the user provides a JavaScript cookie jar.
/// All cookie operations are handled in JavaScript, so this Rust implementation
/// does nothing. Cookies are passed as headers from JavaScript, and redirect
/// handling with cookie interop happens in the JS wrapper.
pub struct NoCookieStore;

impl CookieStore for NoCookieStore {
  fn set_cookies(
    &self,
    _cookie_headers: &mut dyn Iterator<Item = &reqwest::header::HeaderValue>,
    _url: &Url,
  ) {
    // No-op: cookies are handled in JavaScript
  }

  fn cookies(&self, _url: &Url) -> Option<HeaderValue> {
    // No-op: cookies are handled in JavaScript
    None
  }
}
