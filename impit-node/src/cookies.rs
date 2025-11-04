use napi::bindgen_prelude::{Function, JsObjectValue, Object};

use napi::Env;

use crate::utils::await_promise;

use reqwest::Url;

use reqwest::cookie::CookieStore;
use reqwest::header::HeaderValue;

use napi::Status;

use napi::bindgen_prelude::Promise;

use napi::threadsafe_function::ThreadsafeFunction;

pub struct NodeCookieJar {
  pub(crate) set_cookie_tsfn:
    ThreadsafeFunction<(String, String), Promise<()>, (String, String), Status, false>,
  pub(crate) get_cookies_tsfn: ThreadsafeFunction<String, Promise<String>, String, Status, false>,
}

impl CookieStore for NodeCookieJar {
  fn set_cookies(
    &self,
    cookie_headers: &mut dyn Iterator<Item = &reqwest::header::HeaderValue>,
    url: &Url,
  ) {
    for header in cookie_headers {
      let header = header.to_str().unwrap_or_default().to_string();
      let url = url.as_str().to_string();

      let _ = await_promise(&self.set_cookie_tsfn, (header.clone(), url.clone()));
    }
  }

  fn cookies(&self, url: &Url) -> Option<reqwest::header::HeaderValue> {
    let url = url.as_str().to_string();

    await_promise(&self.get_cookies_tsfn, url.clone())
      .ok()
      .and_then(|header| {
        if header.is_empty() {
          return None;
        }

        HeaderValue::from_str(&header).ok()
      })
  }
}

impl NodeCookieJar {
  pub fn new(env: &Env, tough_cookie: Object) -> Result<Self, napi::Error> {
    let set_cookie_js_method = match tough_cookie
      .get_named_property::<Function<'_, (String, String), Promise<()>>>("setCookie")
    {
      Ok(method) => method,
      Err(e) => {
        return Err(napi::Error::new(
          napi::Status::GenericFailure,
          format!("[impit] Couldn't find `setCookie` method on the external cookie store: {e}"),
        ));
      }
    };

    let get_cookie_js_method = match tough_cookie
      .get_named_property::<Function<'_, String, Promise<String>>>("getCookieString")
    {
      Ok(method) => method,
      Err(e) => {
        return Err(napi::Error::new(
          napi::Status::GenericFailure,
          format!(
            "[impit] Couldn't find `getCookieString` method on the external cookie store: {e}"
          ),
        ));
      }
    };

    let mut set_cookie = set_cookie_js_method
      .build_threadsafe_function::<(std::string::String, std::string::String)>()
      .build_callback(|ctx| Ok(ctx.value))?;

    let mut get_cookies = get_cookie_js_method
      .build_threadsafe_function::<std::string::String>()
      .build_callback(|ctx| Ok(ctx.value))?;

    // Unless the `ThreadsafeFunction` is unreferenced, the Node.JS application will hang on exit
    // https://nodejs.github.io/node-addon-examples/special-topics/thread-safe-functions/#q-my-application-isnt-exiting-correctly-it-just-hangs
    #[allow(deprecated)]
    let _ = set_cookie.unref(env);
    #[allow(deprecated)]
    let _ = get_cookies.unref(env);

    Ok(Self {
      set_cookie_tsfn: set_cookie,
      get_cookies_tsfn: get_cookies,
    })
  }
}
