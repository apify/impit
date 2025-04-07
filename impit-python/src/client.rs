use std::{collections::HashMap, time::Duration};

use impit::{
    emulation::Browser,
    impit::{ErrorType, Impit, ImpitBuilder},
    request::RequestOptions,
};
use pyo3::{
    exceptions::{PyRuntimeError, PyTypeError, PyValueError},
    prelude::*,
};

use crate::{
    request::{form_to_bytes, RequestBody},
    response::{self, ImpitPyResponse},
};

#[pyclass]
pub(crate) struct Client {
    impit: Impit,
}

#[pymethods]
impl Client {
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

        pyo3_async_runtimes::tokio::get_runtime().block_on(async {
            Self {
                impit: builder.build(),
            }
        })
    }

    #[pyo3(signature = (url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
    pub fn get(
        &mut self,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<RequestBody>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
    ) -> Result<response::ImpitPyResponse, PyErr> {
        self.request("get", url, content, data, headers, timeout, force_http3)
    }

    #[pyo3(signature = (url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
    pub fn head(
        &mut self,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<RequestBody>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
    ) -> Result<response::ImpitPyResponse, PyErr> {
        self.request("head", url, content, data, headers, timeout, force_http3)
    }

    #[pyo3(signature = (url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
    pub fn post(
        &mut self,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<RequestBody>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
    ) -> Result<response::ImpitPyResponse, PyErr> {
        self.request("post", url, content, data, headers, timeout, force_http3)
    }

    #[pyo3(signature = (url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
    pub fn patch(
        &mut self,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<RequestBody>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
    ) -> Result<response::ImpitPyResponse, PyErr> {
        self.request("patch", url, content, data, headers, timeout, force_http3)
    }

    #[pyo3(signature = (url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
    pub fn put(
        &mut self,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<RequestBody>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
    ) -> Result<response::ImpitPyResponse, PyErr> {
        self.request("put", url, content, data, headers, timeout, force_http3)
    }

    #[pyo3(signature = (url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
    pub fn delete(
        &mut self,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<RequestBody>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
    ) -> Result<response::ImpitPyResponse, PyErr> {
        self.request("delete", url, content, data, headers, timeout, force_http3)
    }

    #[pyo3(signature = (url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
    pub fn options(
        &mut self,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<RequestBody>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
    ) -> Result<response::ImpitPyResponse, PyErr> {
        self.request("options", url, content, data, headers, timeout, force_http3)
    }

    #[pyo3(signature = (url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
    pub fn trace(
        &mut self,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<RequestBody>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
    ) -> Result<response::ImpitPyResponse, PyErr> {
        self.request("trace", url, content, data, headers, timeout, force_http3)
    }

    #[pyo3(signature = (method, url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
    pub fn request(
        &mut self,
        method: &str,
        url: String,
        content: Option<Vec<u8>>,
        mut data: Option<RequestBody>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
    ) -> Result<ImpitPyResponse, PyErr> {
        let mut headers = headers.clone();

        if let Some(content) = content {
            data = Some(RequestBody::Bytes(content));
        }

        let body: Vec<u8> = match data {
            Some(data) => match data {
                RequestBody::Bytes(bytes) => Ok(bytes),
                RequestBody::Form(form) => {
                    headers.get_or_insert_with(HashMap::new).insert(
                        "Content-Type".to_string(),
                        "application/x-www-form-urlencoded".to_string(),
                    );
                    Ok(form_to_bytes(form))
                }
                RequestBody::CatchAll(e) => Err(PyErr::new::<PyTypeError, _>(format!(
                    "Unsupported data type in request body: {:#?}",
                    e
                ))),
            },
            None => Ok(Vec::new()),
        }?;

        let options = RequestOptions {
            headers: headers.unwrap_or_default(),
            timeout: timeout.map(Duration::from_secs_f64),
            http3_prior_knowledge: force_http3.unwrap_or(false),
        };

        pyo3_async_runtimes::tokio::get_runtime()
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
                    _ => Err(ErrorType::InvalidMethod(method.to_string())),
                }
            })
            .map(|response| response.into())
            .map_err(|err| match err {
                ErrorType::RequestError(r) => PyErr::new::<PyRuntimeError, _>(format!("{:#?}", r)),
                e => PyErr::new::<PyValueError, _>(e.to_string()),
            })
    }
}
