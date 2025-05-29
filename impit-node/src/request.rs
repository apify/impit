use std::{
  thread::{self, sleep},
  time::Duration,
};

use napi::{
  bindgen_prelude::{FromNapiValue, Function, JsValuesTupleIntoVec, Promise, Uint8Array},
  threadsafe_function::ThreadsafeFunction,
  Env,
};
use napi_derive::napi;
use reqwest::{cookie::CookieStore, header::HeaderValue, Url};
use tokio::sync::oneshot;

#[derive(Default, Clone)]
#[napi(string_enum = "UPPERCASE")]
pub enum HttpMethod {
  #[default]
  Get,
  Post,
  Put,
  Delete,
  Patch,
  Head,
  Options,
}

#[derive(Default)]
#[napi(object)]
pub struct RequestInit {
  pub method: Option<HttpMethod>,
  #[napi(ts_type = "Headers | Record<string, string> | [string, string][]")]
  pub headers: Option<Vec<(String, String)>>,
  #[napi(
    ts_type = "string | ArrayBuffer | Uint8Array | DataView | Blob | File | URLSearchParams | FormData | ReadableStream"
  )]
  pub body: Option<Uint8Array>,
  /// Request timeout in milliseconds. Overrides the Impit-wide timeout option.
  pub timeout: Option<u32>,
  /// Force the request to use HTTP/3. If the server doesn't expect HTTP/3, the request will fail.
  pub force_http3: Option<bool>,
}

fn await_promise<
  T: Send,
  CallbackArgs: JsValuesTupleIntoVec,
  RustReturn: FromNapiValue + std::fmt::Debug + Sync + Send,
>(
  tsfn: &ThreadsafeFunction<T, Promise<RustReturn>, CallbackArgs, false>,
  args: T,
) -> Result<RustReturn, napi::Error> {
  thread::scope(|scope| {
    let (tx, mut rx) = oneshot::channel();

    scope.spawn(move || match tokio::runtime::Runtime::new() {
      Ok(runtime) => {
        runtime.block_on(async {
          let result = tsfn.call_async(args).await.unwrap().await;

          let _ = tx.send(result);
        });
      }
      Err(e) => {
        let _ = tx.send(Err(napi::Error::new(
          napi::Status::GenericFailure,
          format!(
            "[impit] failed to retrieve cookies from the external cookie store: {}",
            e
          ),
        )));
      }
    });

    let mut result = rx.try_recv();

    let max_retries = 5;
    let mut retries = 0;

    while result.is_err() && retries < max_retries {
      sleep(Duration::from_millis(5));
      result = rx.try_recv();
      retries += 1;
    }

    match result {
      Ok(Ok(result)) => Ok(result),
      Ok(Err(e)) => Err(e),
      Err(_) => Err(napi::Error::new(
        napi::Status::GenericFailure,
        "[impit] failed to retrieve cookies from the external cookie store".to_string(),
      )),
    }
  })
}

pub struct NodeCookieJar {
  set_cookie_tsfn: ThreadsafeFunction<(String, String), Promise<()>, (String, String), false>,
  get_cookies_tsfn: ThreadsafeFunction<String, Promise<String>, String, false>,
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
  pub fn new(env: &Env, tough_cookie: napi::JsObject) -> Result<Self, napi::Error> {
    let set_cookie_js_method = match tough_cookie
      .get_named_property::<Function<'_, (String, String), Promise<()>>>("setCookie")
    {
      Ok(method) => method,
      Err(e) => {
        return Err(napi::Error::new(
          napi::Status::GenericFailure,
          format!(
            "[impit] Couldn't find `setCookie` method on the external cookie store: {}",
            e
          ),
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
            "[impit] Couldn't find `getCookieString` method on the external cookie store: {}",
            e
          ),
        ));
      }
    };

    let mut set_cookie = set_cookie_js_method
      .build_threadsafe_function::<(std::string::String, std::string::String)>()
      .build_callback(|ctx| Ok(ctx.value))
      .unwrap();

    let mut get_cookies = get_cookie_js_method
      .build_threadsafe_function::<std::string::String>()
      .build_callback(|ctx| Ok(ctx.value))
      .unwrap();

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
