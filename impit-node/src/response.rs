#![allow(clippy::await_holding_refcell_ref, deprecated)]
use impit::utils::{decode, ContentType};
use napi::{
  bindgen_prelude::{Buffer, ReadableStream, Result, This},
  Env, JsFunction, JsObject, JsUnknown,
};
use napi_derive::napi;
use reqwest::Response;
use std::{cell::RefCell, collections::HashMap};
use tokio_stream::StreamExt;

const INNER_RESPONSE_PROPERTY_NAME: &str = "__js_response";
#[napi]
pub struct ImpitResponse {
  inner: RefCell<Option<Response>>,
  pub status: u16,
  pub status_text: String,
  pub headers: HashMap<String, String>,
  pub ok: bool,
  pub url: String,
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
    let headers = response
      .headers()
      .iter()
      .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap().to_string()))
      .collect();
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

  fn get_inner_response(&self, env: &Env, mut this: This<JsObject>) -> Result<napi::JsObject> {
    let cached_response = this.get::<JsObject>(INNER_RESPONSE_PROPERTY_NAME)?;

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
        .and_then(|global| global.get_named_property::<JsFunction>("Response"))
        .expect("fatal: Couldn't get Response constructor");

      this.set(
        INNER_RESPONSE_PROPERTY_NAME,
        response_constructor.new_instance(&[js_stream])?,
      )?;
    }

    Ok(this.get(INNER_RESPONSE_PROPERTY_NAME)?.unwrap())
  }

  #[napi(ts_return_type = "String")]
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

  #[napi(ts_return_type = "Promise<Uint8Array>")]
  pub fn bytes(&self, env: &Env, this: This<JsObject>) -> Result<JsUnknown> {
    let response = self.get_inner_response(env, this)?;

    response
      .get_named_property::<JsFunction>("bytes")?
      .call_without_args(Some(&response))
  }

  #[napi(ts_return_type = "Promise<String>")]
  pub fn text(&self, env: &Env, this: This<JsObject>) -> Result<JsUnknown> {
    let response = self.get_inner_response(env, this)?;

    response
      .get_named_property::<JsFunction>("text")?
      .call_without_args(Some(&response))
  }

  #[napi(ts_return_type = "Promise<any>")]
  pub fn json(&self, env: &Env, this: This<JsObject>) -> Result<JsUnknown> {
    let response = self.get_inner_response(env, this)?;

    response
      .get_named_property::<JsFunction>("json")?
      .call_without_args(Some(&response))
  }

  #[napi(
    getter,
    js_name = "body",
    ts_return_type = "ReadableStream<Uint8Array>"
  )]
  pub fn body(&self, env: &Env, this: This<JsObject>) -> Result<napi::JsObject> {
    let response = self.get_inner_response(env, this)?;

    response.get_named_property("body")
  }
}
