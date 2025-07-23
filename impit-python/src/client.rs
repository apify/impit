use std::{collections::HashMap, time::Duration};

use impit::{
    emulation::Browser,
    errors::ImpitError,
    impit::{Impit, ImpitBuilder},
    request::RequestOptions,
};
use pyo3::{ffi::c_str, prelude::*};

use crate::{
    cookies::PythonCookieJar,
    errors::ImpitPyError,
    request::{form_to_bytes, RequestBody},
    response::{self, ImpitPyResponse},
};

#[pyclass]
pub(crate) struct Client {
    impit: Impit<PythonCookieJar>,
    default_encoding: Option<String>,
}

#[pymethods]
impl Client {
    pub fn __enter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    pub fn __exit__(
        &self,
        _exc_type: &crate::Bound<'_, crate::PyAny>,
        _exc_value: &crate::Bound<'_, crate::PyAny>,
        _traceback: &crate::Bound<'_, crate::PyAny>,
    ) {
    }

    #[new]
    #[pyo3(signature = (browser=None, http3=None, proxy=None, timeout=None, verify=None, default_encoding=None, follow_redirects=None, max_redirects=Some(20), cookie_jar=None, cookies=None, headers=None))]
    pub fn new(
        py: Python<'_>,
        browser: Option<String>,
        http3: Option<bool>,
        proxy: Option<String>,
        timeout: Option<f64>,
        verify: Option<bool>,
        default_encoding: Option<String>,
        follow_redirects: Option<bool>,
        max_redirects: Option<u16>,
        cookie_jar: Option<crate::Bound<'_, crate::PyAny>>,
        cookies: Option<crate::Bound<'_, crate::PyAny>>,
        headers: Option<HashMap<String, String>>,
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

        let builder = match headers {
            Some(headers) => builder.with_headers(headers.into_iter().collect::<Vec<_>>()),
            None => builder,
        };

        let builder = match (cookie_jar, cookies) {
            (Some(_), Some(_)) => {
                panic!("Both cookie_jar and cookies cannot be provided at the same time")
            }
            (Some(cookie_jar), None) => {
                builder.with_cookie_store(PythonCookieJar::new(py, cookie_jar.into()))
            }
            (None, Some(cookies)) => {
                builder.with_cookie_store(PythonCookieJar::from_httpx_cookies(py, cookies.into()))
            }
            (None, None) => builder,
        };
        pyo3_async_runtimes::tokio::get_runtime().block_on(async {
            Self {
                impit: builder.build(),
                default_encoding,
            }
        })
    }

    #[pyo3(signature = (url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
    pub fn get(
        &self,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<RequestBody>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
    ) -> Result<response::ImpitPyResponse, ImpitPyError> {
        self.request(
            "get",
            url,
            content,
            data,
            headers,
            timeout,
            force_http3,
            Some(false),
        )
    }

    #[pyo3(signature = (url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
    pub fn head(
        &self,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<RequestBody>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
    ) -> Result<response::ImpitPyResponse, ImpitPyError> {
        self.request(
            "head",
            url,
            content,
            data,
            headers,
            timeout,
            force_http3,
            Some(false),
        )
    }

    #[pyo3(signature = (url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
    pub fn post(
        &self,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<RequestBody>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
    ) -> Result<response::ImpitPyResponse, ImpitPyError> {
        self.request(
            "post",
            url,
            content,
            data,
            headers,
            timeout,
            force_http3,
            Some(false),
        )
    }

    #[pyo3(signature = (url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
    pub fn patch(
        &self,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<RequestBody>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
    ) -> Result<response::ImpitPyResponse, ImpitPyError> {
        self.request(
            "patch",
            url,
            content,
            data,
            headers,
            timeout,
            force_http3,
            Some(false),
        )
    }

    #[pyo3(signature = (url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
    pub fn put(
        &self,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<RequestBody>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
    ) -> Result<response::ImpitPyResponse, ImpitPyError> {
        self.request(
            "put",
            url,
            content,
            data,
            headers,
            timeout,
            force_http3,
            Some(false),
        )
    }

    #[pyo3(signature = (url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
    pub fn delete(
        &self,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<RequestBody>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
    ) -> Result<response::ImpitPyResponse, ImpitPyError> {
        self.request(
            "delete",
            url,
            content,
            data,
            headers,
            timeout,
            force_http3,
            Some(false),
        )
    }

    #[pyo3(signature = (url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
    pub fn options(
        &self,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<RequestBody>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
    ) -> Result<response::ImpitPyResponse, ImpitPyError> {
        self.request(
            "options",
            url,
            content,
            data,
            headers,
            timeout,
            force_http3,
            Some(false),
        )
    }

    #[pyo3(signature = (url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
    pub fn trace(
        &self,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<RequestBody>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
    ) -> Result<response::ImpitPyResponse, ImpitPyError> {
        self.request(
            "trace",
            url,
            content,
            data,
            headers,
            timeout,
            force_http3,
            Some(false),
        )
    }

    #[pyo3(signature = (method, url, content=None, data=None, headers=None, timeout=None, force_http3=false))]
    pub fn stream<'python>(
        &self,
        py: Python<'python>,
        method: &str,
        url: String,
        content: Option<Vec<u8>>,
        data: Option<RequestBody>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
    ) -> Result<Bound<'python, PyAny>, PyErr> {
        let response = self.request(
            method,
            url,
            content,
            data,
            headers,
            timeout,
            force_http3,
            Some(true),
        )?;

        let fun: Py<PyAny> = PyModule::from_code(
            py,
            c_str!(
                "def wrap_with_context_manager(response):
    class SyncContextManager:
        def __enter__(self):
            self.response = response
            return self.response
        def __exit__(self, exc_type, exc_value, traceback):
            self.response.close()
    return SyncContextManager()"
            ),
            c_str!(""),
            c_str!(""),
        )?
        .getattr("wrap_with_context_manager")?
        .into();

        let wrapped_response = fun.call1(py, (response,))?;
        Ok(wrapped_response.into_bound(py))
    }

    #[pyo3(signature = (method, url, content=None, data=None, headers=None, timeout=None, force_http3=false, stream=false))]
    pub fn request(
        &self,
        method: &str,
        url: String,
        content: Option<Vec<u8>>,
        mut data: Option<RequestBody>,
        headers: Option<HashMap<String, String>>,
        timeout: Option<f64>,
        force_http3: Option<bool>,
        stream: Option<bool>,
    ) -> Result<ImpitPyResponse, ImpitPyError> {
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
                RequestBody::CatchAll(e) => Err(ImpitPyError(ImpitError::BindingPassthroughError(
                    format!("Unsupported data type: {e:?}").to_string(),
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

        pyo3_async_runtimes::tokio::get_runtime().block_on(async {
            let response = match method.to_lowercase().as_str() {
                "get" => self.impit.get(url, Some(options)).await,
                "post" => self.impit.post(url, Some(body), Some(options)).await,
                "patch" => self.impit.patch(url, Some(body), Some(options)).await,
                "put" => self.impit.put(url, Some(body), Some(options)).await,
                "options" => self.impit.options(url, Some(options)).await,
                "trace" => self.impit.trace(url, Some(options)).await,
                "head" => self.impit.head(url, Some(options)).await,
                "delete" => self.impit.delete(url, Some(options)).await,
                _ => Err(ImpitError::InvalidMethod(method.to_string())),
            };

            match response {
                Ok(response) => {
                    let py_response = ImpitPyResponse::from_async(
                        response,
                        self.default_encoding.clone(),
                        stream.unwrap_or(false),
                    )
                    .await;
                    Ok(py_response)
                }
                Err(err) => Err(ImpitPyError(err)),
            }
        })
    }
}
