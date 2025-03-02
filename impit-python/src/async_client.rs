use std::{collections::HashMap, time::Duration};

use impit::{
    emulation::Browser,
    impit::ImpitBuilder,
    request::RequestOptions,
};
use pyo3::prelude::*;
use tokio::sync::oneshot;

use crate::response::ImpitPyResponse;

#[pyclass]
pub(crate) struct AsyncClient {
    impit_config: ImpitBuilder,
}

#[pymethods]
impl AsyncClient {
    #[new]
    #[pyo3(signature = (browser=None, http3=None, proxy=None, timeout=None, verify=None))]
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
            impit_config: builder,
        }
    }

    #[pyo3(signature = (url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
    pub fn get<'python>(
        &self,
        py: Python<'python>,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<HashMap<String, String>>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
    ) -> Result<pyo3::Bound<'python, PyAny>, PyErr> {
        self.request(py, "get", url, content, data, headers, timeout, force_http3)
    }

    #[pyo3(signature = (url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
    pub fn head<'python>(
        &self,
        py: Python<'python>,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<HashMap<String, String>>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
    ) -> Result<pyo3::Bound<'python, PyAny>, PyErr> {
        self.request(py, "head", url, content, data, headers, timeout, force_http3)
    }

    #[pyo3(signature = (url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
    pub fn post<'python>(
        &self,
        py: Python<'python>,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<HashMap<String, String>>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
    ) -> Result<pyo3::Bound<'python, PyAny>, PyErr> {
        self.request(py, "post", url, content, data, headers, timeout, force_http3)
    }

    #[pyo3(signature = (url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
    pub fn patch<'python>(
        &self,
        py: Python<'python>,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<HashMap<String, String>>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
    ) -> Result<pyo3::Bound<'python, PyAny>, PyErr> {
        self.request(py, "patch", url, content, data, headers, timeout, force_http3)
    }

    #[pyo3(signature = (url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
    pub fn put<'python>(
        &self,
        py: Python<'python>,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<HashMap<String, String>>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
    ) -> Result<pyo3::Bound<'python, PyAny>, PyErr> {
        self.request(py, "put", url, content, data, headers, timeout, force_http3)
    }

    #[pyo3(signature = (url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
    pub fn delete<'python>(
        &self,
        py: Python<'python>,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<HashMap<String, String>>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
    ) -> Result<pyo3::Bound<'python, PyAny>, PyErr> {
        self.request(py, "delete", url, content, data, headers, timeout, force_http3)
    }

    #[pyo3(signature = (url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
    pub fn options<'python>(
        &self,
        py: Python<'python>,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<HashMap<String, String>>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
    ) -> Result<pyo3::Bound<'python, PyAny>, PyErr> {
        self.request(py, "options", url, content, data, headers, timeout, force_http3)
    }

    #[pyo3(signature = (url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
    pub fn trace<'python>(
        &self,
        py: Python<'python>,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<HashMap<String, String>>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
    ) -> Result<pyo3::Bound<'python, PyAny>, PyErr> {
        self.request(py, "trace", url, content, data, headers, timeout, force_http3)
    }

    #[pyo3(signature = (method, url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
    pub fn request<'python>(
        &self,
        py: Python<'python>,
        method: &str,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<HashMap<String, String>>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
    ) -> Result<pyo3::Bound<'python, PyAny>, PyErr> {
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
            http3_prior_knowledge: force_http3.unwrap_or(false),
        };

        let (tx, rx) = oneshot::channel();

        let impit_config = self.impit_config.clone();
        let method = method.to_string();

        pyo3_async_runtimes::tokio::get_runtime()
            .spawn(async move {
                let mut impit = impit_config.build();

                let response = match method.to_lowercase().as_str() {
                    "get" => impit.get(url, Some(options)).await,
                    "post" => impit.post(url, Some(body), Some(options)).await,
                    "patch" => impit.patch(url, Some(body), Some(options)).await,
                    "put" => impit.put(url, Some(body), Some(options)).await,
                    "options" => impit.options(url, Some(options)).await,
                    "trace" => impit.trace(url, Some(options)).await,
                    "head" => impit.head(url, Some(options)).await,
                    "delete" => impit.delete(url, Some(options)).await,
                    _ => panic!("Unsupported method"),
                };

                tx.send(response).unwrap();
            });

        pyo3_async_runtimes::async_std::future_into_py::<_, ImpitPyResponse>(
            py,
            async {
                let response = rx.await.unwrap();

                match response {
                    Ok(response) => Ok(response.into()),
                    Err(err) => Err(PyErr::new::<pyo3::exceptions::PyException, _>(format!("{:#?}", err))),
                }
            }
        )


    }
}
