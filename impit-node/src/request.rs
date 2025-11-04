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

/// Options for configuring an individual HTTP request.
///
/// These options allow you to customize the behavior of a specific request, including the HTTP method, headers, body, timeout, and whether to force HTTP/3.
///
/// If no options are provided, default settings will be used.
///
/// See {@link Impit.fetch} for usage.
#[derive(Default)]
#[napi(object)]
pub struct RequestInit {
  /// HTTP method to use for the request. Default is `GET`.
  ///
  /// Can be one of: `GET`, `POST`, `PUT`, `DELETE`, `PATCH`, `HEAD`, `OPTIONS`.
  pub method: Option<HttpMethod>,
  /// Additional headers to include in the request.
  ///
  /// Can be an object, a Map, or an array of tuples or an instance of the {@link https://developer.mozilla.org/en-US/docs/Web/API/Headers | Headers} class.
  ///
  /// Note that headers set here will override any default headers set in {@link ImpitOptions.headers}.
  #[napi(ts_type = "Headers | Record<string, string> | [string, string][]")]
  pub headers: Option<Vec<(String, String)>>,
  #[napi(
    ts_type = "string | ArrayBuffer | Uint8Array | DataView | Blob | File | URLSearchParams | FormData | ReadableStream"
  )]
  /// Request body. Can be a string, Buffer, ArrayBuffer, TypedArray, DataView, Blob, File, URLSearchParams, FormData or ReadableStream.
  pub body: Option<Uint8Array>,
  /// Request timeout in milliseconds. Overrides the Impit-wide timeout option from {@link ImpitOptions.timeout}.
  pub timeout: Option<u32>,
  /// Force the request to use HTTP/3. If the server doesn't expect HTTP/3 or the Impit instance doesn't have HTTP/3 enabled (via the {@link ImpitOptions.http3} option), the request will fail.
  pub force_http3: Option<bool>,
}
