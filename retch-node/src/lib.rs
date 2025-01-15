use std::time::Duration;

use retch::{retcher::{ErrorType, Retcher, RetcherBuilder}, RequestOptions};
use napi_derive::napi;

mod response;
mod request;
mod retcher_builder;

use request::{serialize_body, HttpMethod, RequestInit};
use retcher_builder::RetcherOptions;
use self::response::RetchResponse;


#[napi(js_name="Retcher")]
pub struct RetcherWrapper {
  inner: Retcher,
}

#[napi]
impl RetcherWrapper {
    #[napi(constructor)]
    pub fn new(options: Option<RetcherOptions>) -> Self {
      let config: RetcherBuilder = options.unwrap_or_default().into();

      Self {
        inner: config.build(),
      }
    }

    #[napi]
    pub async unsafe fn fetch(&mut self, url: String, request_init: Option<RequestInit>) -> Result<RetchResponse, napi::Error> {
      let request_options = Some(RequestOptions {
        headers: request_init.as_ref().and_then(|init| init.headers.as_ref()).cloned().unwrap_or_default(),
        timeout: if let Some(timeout) = request_init.as_ref().and_then(|init| init.timeout) { Some(Duration::from_millis(timeout.into())) } else { None },
        http3_prior_knowledge: request_init.as_ref().and_then(|init| init.force_http3).unwrap_or_default(),
      });

      let body = request_init.as_ref().and_then(|init| init.body.as_ref()).cloned();

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
        Ok(response) => Ok(RetchResponse::from(response).await),
        Err(err) => {
          let status = match err {
            ErrorType::UrlMissingHostnameError => napi::Status::InvalidArg,
            ErrorType::UrlProtocolError => napi::Status::InvalidArg,
            ErrorType::UrlParsingError => napi::Status::InvalidArg,
            ErrorType::ImpersonationError => napi::Status::GenericFailure,
            ErrorType::RequestError(_) => napi::Status::GenericFailure,
            ErrorType::ResponseError => napi::Status::GenericFailure,
          };
          let reason = format!("{:#?}", err);
          Err(napi::Error::new(status, reason))
        }
      }
    }
}

