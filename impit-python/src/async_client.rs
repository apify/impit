use std::{collections::HashMap, time::Duration};

use impit::{emulation::Browser, errors::ImpitError, impit::ImpitBuilder, request::RequestOptions};
use pyo3::{exceptions::PyTypeError, prelude::*};
use tokio::sync::oneshot;

use crate::{errors::ImpitPyError, request::form_to_bytes, response::ImpitPyResponse, RequestBody};

#[pyclass]
pub(crate) struct AsyncClient {
    impit_config: ImpitBuilder,
    default_encoding: Option<String>,
}

#[pymethods]
impl AsyncClient {
    #[new]
    #[pyo3(signature = (browser=None, http3=None, proxy=None, timeout=None, verify=None, default_encoding=None, follow_redirects=None, max_redirects=Some(20)))]
    pub fn new(
        browser: Option<String>,
        http3: Option<bool>,
        proxy: Option<String>,
        timeout: Option<f64>,
        verify: Option<bool>,
        default_encoding: Option<String>,
        follow_redirects: Option<bool>,
        max_redirects: Option<u16>,
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

        let builder = match follow_redirects {
            Some(true) => builder.with_redirect(impit::impit::RedirectBehavior::FollowRedirect(
                max_redirects.unwrap_or(20).into(),
            )),
            _ => builder.with_redirect(impit::impit::RedirectBehavior::ManualRedirect),
        };

        Self {
            impit_config: builder,
            default_encoding,
        }
    }

    #[pyo3(signature = (url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
    pub fn get<'python>(
        &self,
        py: Python<'python>,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<RequestBody>,
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
        data: Option<RequestBody>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
    ) -> Result<pyo3::Bound<'python, PyAny>, PyErr> {
        self.request(
            py,
            "head",
            url,
            content,
            data,
            headers,
            timeout,
            force_http3,
        )
    }

    #[pyo3(signature = (url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
    pub fn post<'python>(
        &self,
        py: Python<'python>,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<RequestBody>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
    ) -> Result<pyo3::Bound<'python, PyAny>, PyErr> {
        self.request(
            py,
            "post",
            url,
            content,
            data,
            headers,
            timeout,
            force_http3,
        )
    }

    #[pyo3(signature = (url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
    pub fn patch<'python>(
        &self,
        py: Python<'python>,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<RequestBody>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
    ) -> Result<pyo3::Bound<'python, PyAny>, PyErr> {
        self.request(
            py,
            "patch",
            url,
            content,
            data,
            headers,
            timeout,
            force_http3,
        )
    }

    #[pyo3(signature = (url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
    pub fn put<'python>(
        &self,
        py: Python<'python>,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<RequestBody>,
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
        data: Option<RequestBody>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
    ) -> Result<pyo3::Bound<'python, PyAny>, PyErr> {
        self.request(
            py,
            "delete",
            url,
            content,
            data,
            headers,
            timeout,
            force_http3,
        )
    }

    #[pyo3(signature = (url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
    pub fn options<'python>(
        &self,
        py: Python<'python>,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<RequestBody>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
    ) -> Result<pyo3::Bound<'python, PyAny>, PyErr> {
        self.request(
            py,
            "options",
            url,
            content,
            data,
            headers,
            timeout,
            force_http3,
        )
    }

    #[pyo3(signature = (url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
    pub fn trace<'python>(
        &self,
        py: Python<'python>,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<RequestBody>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
    ) -> Result<pyo3::Bound<'python, PyAny>, PyErr> {
        self.request(
            py,
            "trace",
            url,
            content,
            data,
            headers,
            timeout,
            force_http3,
        )
    }

    #[pyo3(signature = (method, url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
    pub fn request<'python>(
        &self,
        py: Python<'python>,
        method: &str,
        url: String,
        content: Option<Vec<u8>>,
        mut data: Option<RequestBody>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
    ) -> Result<pyo3::Bound<'python, PyAny>, PyErr> {
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
            headers: headers
                .unwrap_or_default()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            timeout: timeout.map(Duration::from_secs_f64),
            http3_prior_knowledge: force_http3.unwrap_or(false),
        };

        let (tx, rx) = oneshot::channel();

        let impit_config = self.impit_config.clone();
        let method = method.to_string();

        pyo3_async_runtimes::tokio::get_runtime().spawn(async move {
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
                _ => Err(ImpitError::InvalidMethod(method.to_string())),
            };

            tx.send(response).unwrap();
        });

        let default_encoding = self.default_encoding.clone();

        pyo3_async_runtimes::async_std::future_into_py::<_, ImpitPyResponse>(py, async {
            let response = rx.await.unwrap();

            response
                .map(|response| ImpitPyResponse::from(response, default_encoding))
                .map_err(|err| ImpitPyError(err).into())
        })
    }
}
