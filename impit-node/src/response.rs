#![allow(clippy::await_holding_refcell_ref, deprecated)]
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

#[napi]
pub struct ImpitResponse {
  inner: RefCell<Option<Response>>,
  pub status: u16,
  pub status_text: String,
  #[napi(ts_type = "Headers")]
  pub headers: Headers,
  pub ok: bool,
  pub url: String,
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
impl ImpitResponse {
  pub(crate) fn from(response: Response) -> Self {
    let status = response.status().as_u16();
    let status_text = response
      .status()
      .canonical_reason()
      .unwrap_or("")
      .to_string();
    let headers = Headers(
      response
        .headers()
        .iter()
        .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap().to_string()))
        .collect(),
    );
    let ok = response.status().is_success();
    let url = response.url().to_string();

    Self {
      inner: RefCell::new(Some(response)),
      status,
      status_text,
      headers,
      ok,
      url,
    }
  }

  fn get_inner_response(&self, env: &Env, mut this: This<Object>) -> Result<Object> {
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
          format!("Error reading response stream: {:?}", e),
        ))),
      });

      let js_stream = ReadableStream::create_with_stream_bytes(env, napi_stream)?;

      let response_constructor = env
        .get_global()
        .and_then(|global| global.get_named_property::<Function>("Response"))
        .expect("fatal: Couldn't get Response constructor");

      this.set(
        INNER_RESPONSE_PROPERTY_NAME,
        response_constructor.new_instance(js_stream.to_unknown())?,
      )?;
    }

    Ok(this.get(INNER_RESPONSE_PROPERTY_NAME)?.unwrap())
  }

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

  #[napi(ts_return_type = "Promise<ArrayBuffer>")]
  pub fn array_buffer(&self, env: &Env, this: This<Object>) -> Result<Object> {
    let response = self.get_inner_response(env, this)?;

    response
      .get_named_property::<Function<'_, (), Object>>("arrayBuffer")?
      .apply(Some(&response), ())?
      .coerce_to_object()
  }

  #[napi(ts_return_type = "Promise<Uint8Array>")]
  pub fn bytes(&self, env: &Env, this: This<Object>) -> Result<Object> {
    let array_buffer_promise = self.array_buffer(env, this)?;
    let then: Function<'_, Function<Object, Unknown>, Object> =
      array_buffer_promise.get_named_property("then")?;

    let cb = env
      .get_global()?
      .get_named_property::<Function<'_, String, Function<Object, Unknown>>>("eval")?
      .call("(buf) => new Uint8Array(buf)".to_string())?;

    then.apply(Some(&array_buffer_promise), cb)
  }

  #[napi(ts_return_type = "Promise<string>")]
  pub fn text(&self, env: &Env, this: This<Object>) -> Result<Unknown> {
    let response = self.get_inner_response(env, this)?;

    response
      .get_named_property::<Function<'_, (), Unknown>>("text")?
      .apply(response, ())
  }

  #[napi(ts_return_type = "Promise<any>")]
  pub fn json(&self, env: &Env, this: This<Object>) -> Result<Unknown> {
    let response = self.get_inner_response(env, this)?;

    response
      .get_named_property::<Function<'_, (), Unknown>>("json")?
      .apply(response, ())
  }

  #[napi(
    getter,
    js_name = "body",
    ts_return_type = "ReadableStream<Uint8Array>"
  )]
  pub fn body(&self, env: &Env, this: This<Object>) -> Result<Object> {
    let response = self.get_inner_response(env, this)?;

    response.get_named_property("body")
  }
}
