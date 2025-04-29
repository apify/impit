use napi_derive::napi;
use std::collections::HashMap;

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
  pub headers: Option<HashMap<String, String>>,
  #[napi(ts_type = "string | ArrayBuffer | TypedArray | DataView | Blob | File | URLSearchParams | FormData | ReadableStream")]
  pub body: Option<Vec<u8>>,
  /// Request timeout in milliseconds. Overrides the Impit-wide timeout option.
  pub timeout: Option<u32>,
  /// Force the request to use HTTP/3. If the server doesn't expect HTTP/3, the request will fail.
  pub force_http3: Option<bool>,
}
