use std::{collections::HashMap, time::Duration};

use impit::{
    emulation::Browser,
    impit::{Impit, ImpitBuilder},
    request::RequestOptions,
};
use pyo3::prelude::*;

use crate::response;

#[pyclass]
pub(crate) struct Client {
    impit: Impit,
}

#[pymethods]
impl Client {
    #[new]
    pub fn new(
        browser: Option<String>,
        http3: Option<bool>,
        proxy: Option<String>,
        timeout: Option<f64>,
        verify: Option<bool>,
    ) -> Self {
        let builder = ImpitBuilder::default();

        let builder = match browser {
            Some(browser) => match browser.to_lowercase().as_str() {
                "chrome" => builder.with_browser(Browser::Chrome),
                "firefox" => builder.with_browser(Browser::Firefox),
                _ => panic!("Unsupported browser"),
            },
            None => builder,
        };

        let builder = match http3 {
            Some(true) => builder.with_http3(),
            _ => builder,
        };

        let builder = match proxy {
            Some(proxy) => builder.with_proxy(proxy),
            None => builder,
        };

        let builder = match timeout {
            Some(secs) => builder.with_default_timeout(Duration::from_secs_f64(secs)),
            None => builder,
        };

        let builder = match verify {
            Some(false) => builder.with_ignore_tls_errors(true),
            _ => builder,
        };

        Self {
            impit: builder.build(),
        }
    }

    pub fn get(
        &mut self,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<HashMap<String, String>>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
    ) -> response::ImpitPyResponse {
        self.request("get", url, content, data, headers, timeout)
    }
    pub fn head(
        &mut self,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<HashMap<String, String>>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
    ) -> response::ImpitPyResponse {
        self.request("head", url, content, data, headers, timeout)
    }

    pub fn post(
        &mut self,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<HashMap<String, String>>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
    ) -> response::ImpitPyResponse {
        self.request("post", url, content, data, headers, timeout)
    }

    pub fn patch(
        &mut self,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<HashMap<String, String>>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
    ) -> response::ImpitPyResponse {
        self.request("patch", url, content, data, headers, timeout)
    }

    pub fn put(
        &mut self,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<HashMap<String, String>>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
    ) -> response::ImpitPyResponse {
        self.request("put", url, content, data, headers, timeout)
    }

    pub fn delete(
        &mut self,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<HashMap<String, String>>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
    ) -> response::ImpitPyResponse {
        self.request("delete", url, content, data, headers, timeout)
    }

    pub fn options(
        &mut self,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<HashMap<String, String>>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
    ) -> response::ImpitPyResponse {
        self.request("options", url, content, data, headers, timeout)
    }

    pub fn trace(
        &mut self,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<HashMap<String, String>>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
    ) -> response::ImpitPyResponse {
        self.request("trace", url, content, data, headers, timeout)
    }

    pub fn request(
        &mut self,
        method: &str,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<HashMap<String, String>>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
    ) -> response::ImpitPyResponse {
        let mut headers = headers.clone();

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
                    headers.get_or_insert_with(HashMap::new).insert(
                        "Content-Type".to_string(),
                        "application/x-www-form-urlencoded".to_string(),
                    );

                    body
                }
                None => Vec::new(),
            },
        };

        let options = RequestOptions {
            headers: headers.unwrap_or_default(),
            timeout: timeout.map(Duration::from_secs_f64),
            ..Default::default()
        };

        let response = pyo3_asyncio::tokio::get_runtime()
            .block_on(async {
                match method.to_lowercase().as_str() {
                    "get" => self.impit.get(url, Some(options)).await,
                    "post" => self.impit.post(url, Some(body), Some(options)).await,
                    "patch" => self.impit.patch(url, Some(body), Some(options)).await,
                    "put" => self.impit.put(url, Some(body), Some(options)).await,
                    "options" => self.impit.options(url, Some(options)).await,
                    "trace" => self.impit.trace(url, Some(options)).await,
                    "head" => self.impit.head(url, Some(options)).await,
                    "delete" => self.impit.delete(url, Some(options)).await,
                    _ => panic!("Unsupported method"),
                }
            })
            .unwrap();

        response.into()
    }
}
