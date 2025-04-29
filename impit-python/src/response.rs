use std::collections::HashMap;

use impit::utils::ContentType;
use pyo3::prelude::*;
use reqwest::{Response, Version};

use crate::errors::ImpitPyError;

#[pyclass(name = "Response")]
#[derive(Debug, Clone)]
pub struct ImpitPyResponse {
    #[pyo3(get)]
    status_code: u16,
    #[pyo3(get)]
    reason_phrase: String,
    #[pyo3(get)]
    http_version: String,
    #[pyo3(get)]
    headers: HashMap<String, String>,
    #[pyo3(get)]
    text: String,
    #[pyo3(get)]
    encoding: String,
    #[pyo3(get)]
    is_redirect: bool,
    #[pyo3(get)]
    url: String,
    #[pyo3(get)]
    content: Vec<u8>,
    // #[pyo3(get)]
    // request: Request,
    // #[pyo3(get)]
    // next_request: Option<Request>,
    // #[pyo3(get)]
    // cookies: Cookies,
    // #[pyo3(get)]
    // history: Vec<Response>,
    // #[pyo3(get)]
    // elapsed: Duration,
}

#[pymethods]
impl ImpitPyResponse {
    fn __repr__(&self) -> String {
        format!("<Response [{} {}]>", self.status_code, self.reason_phrase)
    }

    fn raise_for_status(&self) -> PyResult<()> {
        if self.status_code >= 400 {
            return Err(
                ImpitPyError(impit::errors::ImpitError::HTTPStatusError(self.status_code)).into(),
            );
        }
        Ok(())
    }
}

impl ImpitPyResponse {
    pub fn from(val: Response, preferred_encoding: Option<String>) -> Self {
        let status_code = val.status().as_u16();
        let url = val.url().to_string();
        let reason_phrase = val
            .status()
            .canonical_reason()
            .unwrap_or_default()
            .to_string();
        let http_version = match val.version() {
            Version::HTTP_09 => "HTTP/0.9".to_string(),
            Version::HTTP_10 => "HTTP/1.0".to_string(),
            Version::HTTP_11 => "HTTP/1.1".to_string(),
            Version::HTTP_2 => "HTTP/2".to_string(),
            Version::HTTP_3 => "HTTP/3".to_string(),
            _ => "Unknown".to_string(),
        };
        let is_redirect = val.status().is_redirection();
        let headers = HashMap::from_iter(val.headers().iter().map(|(k, v)| {
            (
                k.as_str().to_string(),
                v.to_str().unwrap_or_default().to_string(),
            )
        }));

        let content = pyo3_async_runtimes::tokio::get_runtime().block_on(async {
            match val.bytes().await {
                Ok(bytes) => bytes.to_vec(),
                Err(_) => Vec::new(),
            }
        });

        let content_type_charset = headers
            .get("content-type")
            .and_then(|ct| ContentType::from(ct).ok())
            .and_then(|ct| ct.into());

        let encoding = preferred_encoding
            .and_then(|e| encoding::label::encoding_from_whatwg_label(&e))
            .or(content_type_charset)
            .or(impit::utils::determine_encoding(content.as_slice()))
            .unwrap_or(impit::utils::encodings::UTF_8);

        let text = impit::utils::decode(&content, Some(encoding));

        ImpitPyResponse {
            status_code,
            url,
            reason_phrase,
            http_version,
            is_redirect,
            headers,
            encoding: encoding.name().to_string(),
            text,
            content,
        }
    }
}
