use napi::bindgen_prelude::Uint8Array;
use napi_derive::napi;

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
