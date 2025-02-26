use std::{collections::HashMap, future::Future, option};

use impit::{impit::Impit, request::RequestOptions};
use pyo3::prelude::*;

use crate::response;

#[pyclass]
pub(crate) struct Client {
    impit: Impit,
}

#[pymethods]
impl Client {
    #[new]
    pub fn new() -> Self {
        Self {
            impit: Impit::default(),
        }
    }

    pub fn get(&mut self, url: String) -> response::ImpitPyResponse {
        self.request("get", url, None, None, None, None, None, None)
    }

    pub fn post(&mut self, url: String, body: Vec<u8>) -> response::ImpitPyResponse {
        self.request("post", url, Some(body), None, None, None, None, None)
    }

    pub fn request(
        &mut self,
        method: &str,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<HashMap<String, String>>,
        // files: Option<String>,
        json: Option<String>,
        // params: Option<String>,
        headers: Option<HashMap<String, String>>,
        // cookies: Option<String>,
        // auth: Option<String>,
        follow_redirects: Option<bool>,
        timeout: Option<u64>,
    ) -> response::ImpitPyResponse {
        let body: Vec<u8> = match content {
            Some(content) => content,
            None => match data {
                Some(data) => {
                    let mut body = Vec::new();
                    for (key, value) in data {
                        body.extend_from_slice(key.as_bytes());
                        body.extend_from_slice(b"=");
                        body.extend_from_slice(value.as_bytes());
                        body.extend_from_slice(b"&");
                    }
                    body
                }
                None => Vec::new(),
            }
        };

        let options = RequestOptions {
            headers: headers.unwrap_or_default(),
            ..Default::default()
        };

        let response = pyo3_asyncio::tokio::get_runtime().block_on(async {
            match method {
                method if method.to_lowercase() == "get" => self.impit.get(url, Some(options)).await,
                method if method.to_lowercase() == "post" => self.impit.post(url, Some(body), Some(options)).await,
                method if method.to_lowercase() == "put" => self.impit.put(url, Some(body), Some(options)).await,
                method if method.to_lowercase() == "delete" => self.impit.delete(url, Some(options)).await,
                _ => panic!("Unsupported method"),
            }
        }).unwrap();

        response.into()
    }
}
