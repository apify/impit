use std::collections::HashMap;

use pyo3::prelude::*;
use reqwest::{header::HeaderValue, Response, Version};

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
}

impl From<Response> for ImpitPyResponse {
    fn from(val: Response) -> Self {
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
        let encoding = val
            .headers()
            .get("content-type")
            .unwrap_or(&HeaderValue::from_static("text/plain"))
            .to_str()
            .unwrap_or_default()
            .to_string();

        let content = pyo3_async_runtimes::tokio::get_runtime().block_on(async {
            match val.bytes().await {
                Ok(bytes) => bytes.to_vec(),
                Err(_) => Vec::new(),
            }
        });

        let text = String::from_utf8_lossy(&content).to_string();

        ImpitPyResponse {
            status_code,
            url,
            reason_phrase,
            http_version,
            is_redirect,
            headers,
            encoding,
            text,
            content,
        }
    }
}
