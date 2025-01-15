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
  pub fn new(options: Option<ImpitOptions>) -> Self {
    let config: ImpitBuilder = options.unwrap_or_default().into();

    Self {
      inner: config.build(),
    }
  }

  #[napi]
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
      timeout: if let Some(timeout) = request_init.as_ref().and_then(|init| init.timeout) {
        Some(Duration::from_millis(timeout.into()))
      } else {
        None
      },
      http3_prior_knowledge: request_init
        .as_ref()
        .and_then(|init| init.force_http3)
        .unwrap_or_default(),
    });

    let body = request_init
      .as_ref()
      .and_then(|init| init.body.as_ref())
      .cloned();

    let body: Option<Vec<u8>> = match body {
      Some(body) => Some(serialize_body(body)),
      None => None,
    };

    let response = match request_init.unwrap_or_default().method.unwrap_or_default() {
      HttpMethod::GET => self.inner.get(url, request_options).await,
      HttpMethod::POST => self.inner.post(url, body, request_options).await,
      HttpMethod::PUT => self.inner.put(url, body, request_options).await,
      HttpMethod::DELETE => self.inner.delete(url, request_options).await,
      HttpMethod::PATCH => self.inner.patch(url, body, request_options).await,
      HttpMethod::HEAD => self.inner.head(url, request_options).await,
      HttpMethod::OPTIONS => self.inner.options(url, request_options).await,
    };

    match response {
      Ok(response) => Ok(ImpitResponse::from(response).await),
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
