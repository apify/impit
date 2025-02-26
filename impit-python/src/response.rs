use std::collections::HashMap;

use pyo3::prelude::*;
use reqwest::{Response, Version};

#[pyclass]
#[derive(Debug, Clone)]
pub(crate) struct ImpitPyResponse {
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

impl Into<ImpitPyResponse> for Response {
    fn into(self) -> ImpitPyResponse {
        ImpitPyResponse {
            status_code: self.status().as_u16(),
            reason_phrase: self.status().canonical_reason().unwrap().to_string(),
            http_version: match self.version() {
                Version::HTTP_09 => "HTTP/0.9".to_string(),
                Version::HTTP_10 => "HTTP/1.0".to_string(),
                Version::HTTP_11 => "HTTP/1.1".to_string(),
                Version::HTTP_2 => "HTTP/2".to_string(),
                Version::HTTP_3 => "HTTP/3".to_string(),
                _ => "Unknown".to_string(),
            },
            is_redirect: self.status().is_redirection(),
            headers: HashMap::from_iter(
                self.headers()
                    .iter()
                    .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap().to_string())),
            ),
            encoding: self
                .headers()
                .get("content-type")
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
            text: pyo3_asyncio::tokio::get_runtime().block_on(async { self.text().await.unwrap() }),
        }
    }
}
