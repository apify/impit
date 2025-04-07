use std::collections::HashMap;

use impit::utils::ContentType;
use pyo3::prelude::*;
use reqwest::{Response, Version};

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
}

impl From<Response> for ImpitPyResponse {
    fn from(val: Response) -> Self {
        ImpitPyResponse {
            status_code: val.status().as_u16(),
            reason_phrase: val.status().canonical_reason().unwrap().to_string(),
            http_version: match val.version() {
                Version::HTTP_09 => "HTTP/0.9".to_string(),
                Version::HTTP_10 => "HTTP/1.0".to_string(),
                Version::HTTP_11 => "HTTP/1.1".to_string(),
                Version::HTTP_2 => "HTTP/2".to_string(),
                Version::HTTP_3 => "HTTP/3".to_string(),
                _ => "Unknown".to_string(),
            },
            is_redirect: val.status().is_redirection(),
            headers: HashMap::from_iter(
                val.headers()
                    .iter()
                    .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap().to_string())),
            ),
            encoding: val
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .and_then(|ct| ContentType::from(ct).ok().map(|f| f.charset))
                .unwrap_or_default(),
            text: pyo3_async_runtimes::tokio::get_runtime()
                .block_on(async { val.text().await.unwrap_or_default() }),
        }
    }
}
