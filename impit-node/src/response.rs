use std::{cell::RefCell, collections::HashMap, ops::Deref};

use impit::utils::{decode, ContentType};
use napi::{
  bindgen_prelude::{Buffer, BufferSlice, ReadableStream, Result},
  Env, JsFunction, JsObject, JsString, JsUnknown,
};
use napi_derive::napi;
use reqwest::Response;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

#[napi]
pub struct ImpitResponse {
  bytes: RefCell<Option<Vec<u8>>>,
  inner: RefCell<Option<Response>>,
  pub status: u16,
  pub status_text: String,
  pub headers: HashMap<String, String>,
  pub ok: bool,
}

/// Ensures that the response has been read and the bytes are available.
/// This is not part of the impl ImpitResponse block because it is an async function,
/// which causes issues with the napi_derive macro.
///
/// Note that calling this method will consume the response.
async fn read_response_bytes(impit: &ImpitResponse) {
  let mut response = impit.inner.borrow_mut();
  if let Some(inner_response) = response.take() {
    let bytes = inner_response.bytes().await.unwrap().to_vec();
    impit.bytes.replace_with(|_| Some(bytes));
  } else {
    panic!("fatal: Response already consumed");
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
    let headers = response
      .headers()
      .iter()
      .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap().to_string()))
      .collect();
    let ok = response.status().is_success();
    Self {
      bytes: RefCell::new(None),
      inner: RefCell::new(Some(response)),
      status,
      status_text,
      headers,
      ok,
    }
  }

  fn get_bytes(&self) -> Vec<u8> {
    if self.bytes.borrow().is_none() {
      tokio::runtime::Runtime::new().unwrap().block_on(async {
        read_response_bytes(self).await;
      });
    }

    return self.bytes.borrow().deref().clone().unwrap();
  }

  #[napi]
  pub fn bytes(&self) -> Buffer {
    let bytes = self.get_bytes();
    bytes.into()
  }

  #[napi]
  pub fn text(&self) -> String {
    let bytes = self.get_bytes();
    let content_type_header = self.headers.get("content-type");

    decode(
      &bytes,
      content_type_header.and_then(|ct| {
        let parsed = ContentType::from(ct);

        match parsed {
          Ok(ct) => ct.into(),
          Err(_) => None,
        }
      }),
    )
  }

  #[napi(getter, js_name = "body")]
  pub fn stream_body(&self, env: &Env) -> Result<ReadableStream<BufferSlice>> {
    let mut response = self.inner.borrow_mut();
    let response = response.take();

    let reqwest_stream = match response {
      Some(inner_response) => {
        let stream = inner_response.bytes_stream();
        stream
      }
      None => {
        return Err(napi::Error::new(
          napi::Status::Unknown,
          "This response has been already consumed.",
        ));
      }
    };

    let napi_stream = reqwest_stream.map(|chunk| match chunk {
      Ok(bytes) => Ok(bytes.to_vec()),
      Err(e) => Err(napi::Error::new(
        napi::Status::Unknown,
        format!("Error reading response stream: {:?}", e),
      )),
    });

    ReadableStream::create_with_stream_bytes(env, napi_stream)
  }

  #[napi(ts_return_type = "any")]
  pub fn json(&self, env: Env) -> JsUnknown {
    let text = self.text();

    env
      .get_global()
      .and_then(|global| global.get_named_property::<JsObject>("JSON"))
      .and_then(|json| json.get_named_property::<JsFunction>("parse"))
      .expect("fatal: Couldn't get JSON.parse")
      .call::<JsString>(
        None,
        &[env
          .create_string_from_std(text)
          .expect("Couldn't create JS string from the response text")],
      )
      .expect("fatal: Couldn't parse response JSON")
  }
}
