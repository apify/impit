use std::time::Duration;

use impit::{
  impit::{ErrorType, Impit, ImpitBuilder},
  request::RequestOptions,
};
use napi_derive::napi;

mod impit_builder;
mod request;
mod response;

use self::response::ImpitResponse;
use impit_builder::ImpitOptions;
use request::{serialize_body, HttpMethod, RequestInit};

#[napi(js_name = "Impit")]
pub struct ImpitWrapper {
  inner: Impit,
}

#[napi]
impl ImpitWrapper {
  #[napi(constructor)]
  pub fn new(options: Option<ImpitOptions>) -> Result<Self, napi::Error> {
    let config: ImpitBuilder = options.unwrap_or_default().into();

    // `quinn` for h3 requires existing async runtime.
    // This runs the `config.build` function in the napi-managed tokio runtime which remains available
    // throughout the lifetime of the `ImpitWrapper` instance.
    napi::bindgen_prelude::block_on(async {
      Ok(Self {
        inner: config.build(),
      })
    })
  }

  #[allow(clippy::missing_safety_doc)] // This method is `unsafe`, but is only ever used from the Node.JS bindings.
  #[napi]
  /// Fetch a URL with the given options.
  pub async unsafe fn fetch(
    &mut self,
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

    let body = request_init
      .as_ref()
      .and_then(|init| init.body.as_ref())
      .cloned();

    let body: Option<Vec<u8>> = body.map(serialize_body);

    let response = match request_init.unwrap_or_default().method.unwrap_or_default() {
      HttpMethod::Get => self.inner.get(url, request_options).await,
      HttpMethod::Post => self.inner.post(url, body, request_options).await,
      HttpMethod::Put => self.inner.put(url, body, request_options).await,
      HttpMethod::Delete => self.inner.delete(url, request_options).await,
      HttpMethod::Patch => self.inner.patch(url, body, request_options).await,
      HttpMethod::Head => self.inner.head(url, request_options).await,
      HttpMethod::Options => self.inner.options(url, request_options).await,
    };

    match response {
      Ok(response) => Ok(ImpitResponse::from(response)),
      Err(err) => {
        let status = match err {
          ErrorType::UrlMissingHostnameError => napi::Status::InvalidArg,
          ErrorType::UrlProtocolError => napi::Status::InvalidArg,
          ErrorType::UrlParsingError => napi::Status::InvalidArg,
          ErrorType::Http3Disabled => napi::Status::GenericFailure,
          ErrorType::RequestError(_) => napi::Status::GenericFailure,
        };
        let reason = format!("{:#?}", err);
        Err(napi::Error::new(status, reason))
      }
    }
  }
}
