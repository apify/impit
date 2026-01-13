#![allow(clippy::await_holding_refcell_ref, deprecated)]
use crate::abortable_stream::AbortableStream;
use impit::utils::{decode, ContentType};
use napi::bindgen_prelude::JsObjectValue;
use napi::{
  bindgen_prelude::{
    Buffer, FromNapiValue, Function, Object, ReadableStream, Result, This, ToNapiValue,
  },
  sys, Env, JsValue, Unknown,
};
use napi_derive::napi;
use reqwest::Response;
use std::cell::RefCell;
use std::sync::Arc;
use tokio_stream::StreamExt;

const INNER_RESPONSE_PROPERTY_NAME: &str = "__js_response";

pub struct Headers(Vec<(String, String)>);

impl Headers {
  fn get(&self, key: &str) -> Option<&str> {
    self
      .0
      .iter()
      .find(|(k, _)| k.eq_ignore_ascii_case(key))
      .map(|(_, v)| v.as_str())
  }
}

/// Represents an HTTP response.
///
/// The `ImpitResponse` class provides access to the response status, headers, and body.
/// It also includes methods to read the response body in various formats such as text, JSON,
/// ArrayBuffer, and as a stream.
///
/// This class is designed to be API-compatible with the {@link https://developer.mozilla.org/en-US/docs/Web/API/Response | Fetch API Response} class.
///
/// @hideconstructor
#[napi]
pub struct ImpitResponse {
  inner: RefCell<Option<Response>>,
  /// HTTP status code of the response.
  ///
  /// Example: `200` for a successful response.
  pub status: u16,
  /// Status text of the response.
  ///
  /// A short description of the status code.
  ///
  /// Example: "OK" for status code 200.
  pub status_text: String,
  /// HTTP headers of the response.
  ///
  /// An instance of the {@link https://developer.mozilla.org/en-US/docs/Web/API/Headers | Headers} class.
  #[napi(ts_type = "Headers")]
  pub headers: Headers,
  /// `true` if the response status code is in the range 200-299.
  pub ok: bool,
  /// URL of the response.
  ///
  /// In case of redirects, this will be the final URL after all redirects have been followed.
  pub url: String,
  // Shared sender used to immediately signal abort to the JS ReadableStream without polling.
  abort_receiver: Arc<tokio::sync::Mutex<Option<tokio::sync::mpsc::Receiver<()>>>>,
  abort_sender: Arc<tokio::sync::Mutex<Option<tokio::sync::mpsc::Sender<()>>>>,
}

impl ToNapiValue for &mut Headers {
  unsafe fn to_napi_value(raw_env: sys::napi_env, val: Self) -> Result<sys::napi_value> {
    let headers = val.0.clone();

    Vec::to_napi_value(raw_env, headers)
  }
}

impl FromNapiValue for Headers {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> Result<Self> {
    Vec::from_napi_value(env, napi_val).map(Headers)
  }
}

#[napi]
impl<'env> ImpitResponse {
  pub(crate) fn try_from_response(response: Response) -> Result<Self> {
    let status = response.status().as_u16();
    let status_text = response
      .status()
      .canonical_reason()
      .unwrap_or("")
      .to_string();
    let mut headers_vec: Vec<(String, String)> = Vec::new();
    for (k, v) in response.headers().iter() {
      match v.to_str() {
        Ok(val) => headers_vec.push((k.as_str().to_string(), val.to_string())),
        Err(e) => {
          return Err(napi::Error::new(
            napi::Status::GenericFailure,
            format!("Failed to parse header value for '{}': {:?}", k.as_str(), e),
          ));
        }
      }
    }
    let headers = Headers(headers_vec);
    let ok = response.status().is_success();
    let url = response.url().to_string();

    Ok(Self {
      inner: RefCell::new(Some(response)),
      status,
      status_text,
      headers,
      ok,
      url,
      abort_receiver: Arc::new(tokio::sync::Mutex::new(None)),
      abort_sender: Arc::new(tokio::sync::Mutex::new(None)),
    })
  }

  fn get_inner_response(&self, env: &Env, mut this: This<Object>) -> Result<Object<'_>> {
    let cached_response = this.get::<Object>(INNER_RESPONSE_PROPERTY_NAME)?;

    if cached_response.is_none() {
      let mut response = self.inner.borrow_mut();
      let response = response.take();

      let reqwest_stream = match response {
        Some(inner_response) => inner_response.bytes_stream(),
        None => panic!("fatal: Response already consumed, but stream was not cached?"),
      };

      let napi_stream = reqwest_stream.filter_map(|chunk| match chunk {
        Ok(bytes) => {
          if bytes.is_empty() {
            return None;
          }

          Some(Ok(bytes.to_vec()))
        }
        Err(e) => Some(Err(napi::Error::new(
          napi::Status::Unknown,
          format!("Error reading response stream: {e:?}"),
        ))),
      });

      let (tx_abort, rx_abort) = tokio::sync::mpsc::channel::<()>(1);
      if let Ok(mut guard) = self.abort_sender.try_lock() {
        *guard = Some(tx_abort);
      }
      if let Ok(mut guard) = self.abort_receiver.try_lock() {
        *guard = Some(rx_abort);
      }

      let abortable_stream = AbortableStream::new(napi_stream, Arc::clone(&self.abort_receiver));
      let js_stream = ReadableStream::create_with_stream_bytes(env, abortable_stream)?;

      let response_constructor = env
        .get_global()
        .and_then(|global| global.get_named_property::<Function>("Response"))
        .expect("fatal: Couldn't get Response constructor");

      this.set(
        INNER_RESPONSE_PROPERTY_NAME,
        response_constructor.new_instance(js_stream.to_unknown())?,
      )?;
    }

    this
      .get(INNER_RESPONSE_PROPERTY_NAME)
      .transpose()
      .ok_or_else(|| {
        napi::Error::new(
          napi::Status::GenericFailure,
          "fatal: Couldn't get cached response stream".to_string(),
        )
      })?
  }

  /// @ignore
  #[napi(ts_return_type = "string")]
  pub fn decode_buffer(&self, buffer: Buffer) -> Result<String> {
    let encoding = self
      .headers
      .get("content-type")
      .and_then(|content_type| ContentType::from(content_type).ok());

    let string = decode(
      buffer.to_vec().as_slice(),
      match encoding {
        Some(encoding) => encoding.into(),
        None => None,
      },
    );
    Ok(string)
  }

  /// Returns the response body as an `ArrayBuffer`.
  ///
  /// This method is asynchronous and returns a promise that resolves to an `ArrayBuffer` containing the response body data.
  ///
  /// @example
  /// ```ts
  /// const response = await impit.fetch('https://example.com');
  /// const arrayBuffer = await response.arrayBuffer();
  ///
  /// console.log(arrayBuffer); // ArrayBuffer([ 0x3c, 0x68, 0x74, 0x6d, 0x6c, ... ])
  /// ```
  ///
  /// Note that you cannot call this method multiple times on the same response instance,
  /// as the response body can only be consumed once. Subsequent calls will result in an error.
  #[napi(ts_return_type = "Promise<ArrayBuffer>")]
  pub fn array_buffer(&'env self, env: &'env Env, this: This<'env>) -> Result<Object<'env>> {
    let response = self.get_inner_response(env, this)?;

    response
      .get_named_property::<Function<'_, (), Object>>("arrayBuffer")?
      .apply(Some(&response), ())?
      .coerce_to_object()
  }

  /// Returns the response body as a `Uint8Array`.
  ///
  /// This method is asynchronous and returns a promise that resolves to a `Uint8Array` containing the response body data.
  ///
  /// @example
  /// ```ts
  /// const response = await impit.fetch('https://example.com');
  /// const uint8Array = await response.bytes();
  ///
  /// console.log(uint8Array); // Uint8Array([ 0x3c, 0x68, 0x74, 0x6d, 0x6c, ... ])
  /// ```
  ///
  /// Note that you cannot call this method multiple times on the same response instance,
  /// as the response body can only be consumed once. Subsequent calls will result in an error.
  #[napi(ts_return_type = "Promise<Uint8Array>")]
  pub fn bytes(&'env self, env: &'env Env, this: This<'env>) -> Result<Object<'env>> {
    let array_buffer_promise = self.array_buffer(env, this)?;
    let then: Function<'_, Function<Object, Unknown>, Object> =
      array_buffer_promise.get_named_property("then")?;

    let cb = env
      .get_global()?
      .get_named_property::<Function<'_, String, Function<Object, Unknown>>>("eval")?
      .call("(buf) => new Uint8Array(buf)".to_string())?;

    then.apply(Some(&array_buffer_promise), cb)
  }

  /// Returns the response body as a string.
  ///
  /// This method is asynchronous and returns a promise that resolves to a string containing the response body data.
  ///
  /// @example
  /// ```ts
  /// const response = await impit.fetch('https://example.com');
  /// const text = await response.text();
  ///
  /// console.log(text); // "<!doctype html><html>...</html>"
  /// ```
  #[napi(ts_return_type = "Promise<string>")]
  pub fn text(&'env self, env: &'env Env, this: This<'env>) -> Result<Unknown<'env>> {
    let response = self.get_inner_response(env, this)?;

    response
      .get_named_property::<Function<'_, (), Unknown>>("text")?
      .apply(response, ())
  }

  /// Parses the response body as JSON.
  ///
  /// This method is asynchronous and returns a promise that resolves to the parsed JSON object.
  ///
  /// @example
  /// ```ts
  /// const response = await impit.fetch('https://api.example.com/data');
  /// const data = await response.json();
  ///
  /// console.log(data); // Parsed JSON object
  /// ```
  #[napi(ts_return_type = "Promise<any>")]
  pub fn json(&'env self, env: &'env Env, this: This<'env>) -> Result<Unknown<'env>> {
    let response = self.get_inner_response(env, this)?;

    response
      .get_named_property::<Function<'_, (), Unknown>>("json")?
      .apply(response, ())
  }

  /// Returns the response body as a `ReadableStream`.
  ///
  /// This property provides access to the response body as a stream of data, allowing you to read it in chunks.
  ///
  /// @example
  /// ```ts
  /// const response = await impit.fetch('https://example.com');
  /// const reader = response.body.getReader();
  ///
  /// let result;
  /// while (!(result = await reader.read()).done) {
  ///    console.log(result.value); // Uint8Array chunk
  /// }
  /// ```
  #[napi(
    getter,
    js_name = "body",
    ts_return_type = "ReadableStream<Uint8Array>"
  )]
  pub fn body(&'env self, env: &'env Env, this: This<'env>) -> Result<Object<'env>> {
    let response = self.get_inner_response(env, this)?;

    response.get_named_property("body")
  }

  /// Aborts the response.
  ///
  /// This API is called internally and can change without notice.
  ///
  /// Use `fetch(url, { signal: AbortSignal })` to abort a request instead.
  #[napi]
  pub fn abort(&mut self) {
    if let Ok(mut guard) = self.abort_sender.try_lock() {
      if let Some(ref mut tx) = *guard {
        let _ = tx.try_send(());
      }
    }
  }
}
