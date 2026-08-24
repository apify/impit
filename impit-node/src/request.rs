use bytes::Bytes;
use futures_util::{Stream, TryStreamExt};
use napi::bindgen_prelude::{sys, FromNapiValue, ReadableStream, Reader, Uint8Array};

use napi_derive::napi;

/// A JS `ReadableStream` of byte chunks, read chunk by chunk as the request body is sent.
pub struct BodyStream(Reader<Uint8Array>);

impl FromNapiValue for BodyStream {
  unsafe fn from_napi_value(env: sys::napi_env, value: sys::napi_value) -> napi::Result<Self> {
    let stream = unsafe { ReadableStream::<Uint8Array>::from_napi_value(env, value)? };
    Ok(Self(stream.read()?))
  }
}

impl BodyStream {
  pub fn into_byte_stream(self) -> impl Stream<Item = napi::Result<Bytes>> {
    self.0.map_ok(|chunk| Bytes::copy_from_slice(&chunk))
  }
}

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
  Trace,
}

/// Options for configuring an individual HTTP request.
///
/// These options allow you to customize the behavior of a specific request, including the HTTP method, headers, body, timeout, and whether to force HTTP/3.
///
/// If no options are provided, default settings will be used.
///
/// See {@link Impit.fetch} for usage.
#[derive(Default)]
#[napi(object, object_to_js = false)]
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
  ///
  /// Header matching is **case-insensitive** — for example, setting `user-agent` here will override
  /// the impersonation `User-Agent` header.
  ///
  /// To remove an impersonated header, pass an empty string as the value.
  #[napi(ts_type = "Headers | Record<string, string> | [string, string][]")]
  pub headers: Option<Vec<(String, String)>>,
  #[napi(
    ts_type = "string | ArrayBuffer | Uint8Array | DataView | Blob | File | URLSearchParams | FormData | ReadableStream"
  )]
  /// Request body. Can be a string, Buffer, ArrayBuffer, TypedArray, DataView, Blob, File, URLSearchParams, FormData or ReadableStream.
  pub body: Option<Uint8Array>,
  /// Set by the JS wrapper instead of `body` when the body is a stream. Takes precedence over `body`.
  #[napi(skip_typescript)]
  pub body_stream: Option<BodyStream>,
  /// Request timeout in milliseconds. Overrides the Impit-wide timeout option from {@link ImpitOptions.timeout}.
  pub timeout: Option<u32>,
  /// Force the request to use HTTP/3. If the server doesn't expect HTTP/3 or the Impit instance doesn't have HTTP/3 enabled (via the {@link ImpitOptions.http3} option), the request will fail.
  pub force_http3: Option<bool>,
  /// Abort signal to cancel the request.
  #[napi(ts_type = "AbortSignal")]
  pub signal: Option<()>, // This value is consumed in the JS wrapper and is not passed through to the Rust layer.
  /// The redirect mode to use for this request.
  ///
  /// - `'follow'` (default): Follow redirects automatically.
  /// - `'manual'`: Do not follow redirects; return the 3xx response as-is.
  /// - `'error'`: Throw a `TypeError` if the response is a redirect.
  ///
  /// When set, this overrides the instance-level {@link ImpitOptions.followRedirects} option for this request.
  ///
  /// @see {@link https://developer.mozilla.org/en-US/docs/Web/API/RequestInit#redirect | Fetch API `redirect` option}
  #[napi(ts_type = "'follow' | 'manual' | 'error'")]
  pub redirect: Option<()>, // This value is consumed in the JS wrapper and is not passed through to the Rust layer.
}
