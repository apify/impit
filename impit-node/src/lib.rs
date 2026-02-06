use std::time::Duration;

use impit::{errors::ImpitError, impit::Impit, request::RequestOptions};
use napi::{bindgen_prelude::ObjectFinalize, Env};
use napi_derive::napi;

mod abortable_stream;
mod cookies;
mod impit_builder;
mod request;
mod response;

use self::response::ImpitResponse;
use cookies::NodeCookieJar;
use impit_builder::ImpitOptions;
use request::{HttpMethod, RequestInit};

/// The main class of the `impit` package
///
/// This class is the primary interface for making HTTP requests.
/// It provides methods to configure the Impit instance and to perform requests.
///
/// @example
/// ```ts
/// import { Impit } from 'impit';
///
/// const impit = new Impit();
/// const response = await impit.fetch('https://example.com');
/// console.log(await response.text());
/// ```
///
/// One `Impit` instance represents a single (possibly impersonated) user agent.
///
/// Note that all the requests made by this instance will share the same configuration,
/// resources (e.g. cookie jar and connection pool), and other settings.
#[napi(js_name = "Impit", custom_finalize)]
pub struct ImpitWrapper {
  inner: Impit<NodeCookieJar>,
}

impl ObjectFinalize for ImpitWrapper {
  fn finalize(self, env: Env) -> napi::Result<()> {
    env.adjust_external_memory(-500 * 1024)?;
    Ok(())
  }
}

#[napi]
impl ImpitWrapper {
  /// Creates a new `Impit` instance with the given options.
  ///
  /// The `options` parameter allows you to customize the behavior of the Impit instance.
  /// If no options are provided, default settings will be used.
  ///
  /// @example
  /// ```ts
  /// import { Impit } from 'impit';
  ///
  /// const impit = new Impit({
  ///    timeout: 5e3, // Set a default timeout of 5000
  ///    headers: {
  ///       'Authorization: 'Bearer <token>',
  ///    },
  ///    browser: 'chrome',
  /// });
  /// ```
  #[napi(constructor)]
  pub fn new(env: &Env, options: Option<ImpitOptions>) -> Result<Self, napi::Error> {
    let config = options.unwrap_or_default().into_builder()?;

    let _ = env.adjust_external_memory(500 * 1024);

    // `quinn` for h3 requires existing async runtime.
    // This runs the `config.build` function in the napi-managed tokio runtime which remains available
    // throughout the lifetime of the `ImpitWrapper` instance.
    napi::bindgen_prelude::block_on(async {
      Ok(Self {
        inner: config.build().map_err(|e| {
          napi::Error::new(
            napi::Status::GenericFailure,
            format!("Failed to build Impit instance: {e}"),
          )
        })?,
      })
    })
  }

  #[napi(ts_args_type = "resource: string | URL | Request, init?: RequestInit")]
  /// Fetch a URL with the given options.
  ///
  /// This method performs an HTTP request to the specified URL using the provided options.
  /// It returns a promise that resolves to an {@link ImpitResponse} object containing the response data.
  ///
  /// This method is designed to be API-compatible with the {@link https://developer.mozilla.org/en-US/docs/Web/API/fetch | Fetch API `fetch`} global method.
  ///
  /// @example
  /// ```ts
  /// import { Impit } from 'impit';
  ///
  /// const impit = new Impit();
  /// const response = await impit.fetch('https://example.com', {
  ///     method: 'GET',
  ///     headers: {
  ///         'Accept': 'application/json'
  ///     },
  ///     timeout: 5e3,
  /// });
  /// ```
  pub async fn fetch(
    &self,
    url: String,
    request_init: Option<RequestInit>,
  ) -> Result<ImpitResponse, napi::Error> {
    let request_options = Some(RequestOptions {
      headers: request_init
        .as_ref()
        .and_then(|init| init.headers.as_ref())
        .cloned()
        .unwrap_or_default(),
      timeout: request_init
        .as_ref()
        .and_then(|init| init.timeout)
        .map(|timeout| Duration::from_millis(timeout.into())),
      http3_prior_knowledge: request_init
        .as_ref()
        .and_then(|init| init.force_http3)
        .unwrap_or_default(),
    });

    let method = request_init
      .as_ref()
      .and_then(|init| init.method.to_owned())
      .unwrap_or_default();
    let body = request_init
      .and_then(|init| init.body)
      .map(|array| array.to_vec());

    let response = if matches!(method, HttpMethod::Get | HttpMethod::Head) && body.is_some() {
      Err(ImpitError::BindingPassthroughError(
        "GET/HEAD methods don't support passing a request body".to_string(),
      ))
    } else {
      // Match the HTTP method and execute the corresponding request
      match method {
        HttpMethod::Get => self.inner.get(url, body, request_options).await,
        HttpMethod::Head => self.inner.head(url, body, request_options).await,
        HttpMethod::Post => self.inner.post(url, body, request_options).await,
        HttpMethod::Put => self.inner.put(url, body, request_options).await,
        HttpMethod::Delete => self.inner.delete(url, body, request_options).await,
        HttpMethod::Patch => self.inner.patch(url, body, request_options).await,
        HttpMethod::Options => self.inner.options(url, body, request_options).await,
        HttpMethod::Trace => self.inner.trace(url, body, request_options).await,
      }
    };

    match response {
      Ok(response) => ImpitResponse::try_from_response(response),
      Err(err) => {
        let status = match err {
          ImpitError::UrlMissingHostnameError(_) => napi::Status::InvalidArg,
          ImpitError::UrlProtocolError(_) => napi::Status::InvalidArg,
          ImpitError::UrlParsingError(_) => napi::Status::InvalidArg,
          ImpitError::InvalidMethod(_) => napi::Status::InvalidArg,
          ImpitError::Http3Disabled => napi::Status::GenericFailure,
          _ => napi::Status::GenericFailure,
        };

        let reason = format!("impit error: {err}");

        Err(napi::Error::new(status, reason))
      }
    }
  }

  /// Returns the final merged headers that would be sent for a request to the specified URL.
  ///
  /// This method computes the headers by merging:
  /// 1. Browser fingerprint headers (if browser emulation is enabled)
  /// 2. Instance-level headers (from `ImpitOptions.headers`)
  /// 3. Request-specific headers (from `options.headers`)
  ///
  /// This is useful for:
  /// - Debugging and verification of the final headers
  /// - Request signing (e.g., AWS S3, custom APIs that require signed headers)
  /// - Advanced logging and auditing
  /// - Dynamic/conditional logic based on generated headers
  ///
  /// @example
  /// ```ts
  /// import { Impit } from 'impit';
  ///
  /// const impit = new Impit({ browser: 'chrome' });
  /// const headers = impit.getRequestHeaders('https://example.com', {
  ///   headers: { 'X-Custom': 'value' }
  /// });
  ///
  /// // Inspect headers, e.g., for signing
  /// const signature = createSignature(headers);
  /// ```
  #[napi]
  pub fn get_request_headers(
    &self,
    url: String,
    request_init: Option<RequestInit>,
  ) -> Result<Vec<(String, String)>, napi::Error> {
    let request_options = Some(RequestOptions {
      headers: request_init
        .as_ref()
        .and_then(|init| init.headers.as_ref())
        .cloned()
        .unwrap_or_default(),
      timeout: None,
      http3_prior_knowledge: false,
    });

    match self.inner.get_request_headers(url, request_options) {
      Ok(headers) => Ok(headers),
      Err(err) => {
        let status = match err {
          ImpitError::UrlMissingHostnameError(_) => napi::Status::InvalidArg,
          ImpitError::UrlProtocolError(_) => napi::Status::InvalidArg,
          ImpitError::UrlParsingError(_) => napi::Status::InvalidArg,
          _ => napi::Status::GenericFailure,
        };

        Err(napi::Error::new(status, format!("impit error: {err}")))
      }
    }
  }
}
