use std::{collections::HashMap, sync::Arc, time::Duration};

use impit::{
    emulation::Browser,
    errors::ImpitError,
    impit::{Impit, ImpitBuilder},
    request::RequestOptions,
};
use pyo3::{exceptions::PyTypeError, ffi::c_str, prelude::*};

use crate::{
    cookies::PythonCookieJar, errors::ImpitPyError, request::form_to_bytes,
    response::ImpitPyResponse, RequestBody,
};

#[pyclass]
pub(crate) struct AsyncClient {
    impit: Arc<Impit<PythonCookieJar>>,
    default_encoding: Option<String>,
}

#[pymethods]
impl AsyncClient {
    pub fn __aenter__(
        slf: Py<Self>,
        py: Python<'_>,
    ) -> Result<pyo3::Bound<'_, pyo3::PyAny>, pyo3::PyErr> {
        pyo3_async_runtimes::tokio::future_into_py::<_, Py<AsyncClient>>(py, async { Ok(slf) })
    }

    pub fn __aexit__<'python>(
        &self,
        _exc_type: &crate::Bound<'_, crate::PyAny>,
        _exc_value: &crate::Bound<'_, crate::PyAny>,
        _traceback: &crate::Bound<'_, crate::PyAny>,
        py: Python<'python>,
    ) -> Result<pyo3::Bound<'python, pyo3::PyAny>, pyo3::PyErr> {
        pyo3_async_runtimes::tokio::future_into_py::<_, ()>(py, async { Ok(()) })
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
    ) -> PyResult<Self> {
        let builder = ImpitBuilder::default();

        let builder = match browser {
            Some(browser) => match browser.to_lowercase().as_str() {
                "chrome" => builder.with_browser(Browser::Chrome),
                "firefox" => builder.with_browser(Browser::Firefox),
                _ => {
                    return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        "Unsupported browser",
                    ))
                }
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

        let builder = match (cookie_jar, cookies) {
            (Some(_), Some(_)) => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Both cookie_jar and cookies cannot be provided at the same time",
                ));
            }
            (Some(cookie_jar), None) => {
                builder.with_cookie_store(PythonCookieJar::new(py, cookie_jar.into()))
            }
            (None, Some(cookies)) => {
                builder.with_cookie_store(PythonCookieJar::from_httpx_cookies(py, cookies.into()))
            }
            (None, None) => builder,
        };

        let builder = match headers {
            Some(headers) => builder.with_headers(headers.into_iter().collect::<Vec<_>>()),
            None => builder,
        };

        let impit =
            pyo3_async_runtimes::tokio::get_runtime().block_on(async { Arc::new(builder.build()) });

        Ok(Self {
            impit,
            default_encoding,
        })
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
        self.request(
            py,
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
            Some(false),
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
            Some(false),
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
            Some(false),
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
        self.request(
            py,
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
            Some(false),
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
            Some(false),
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
    ) -> Result<pyo3::Bound<'python, PyAny>, PyErr> {
        let response = self.request(
            py,
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
            class AsyncContextManager:
                async def __aenter__(self):
                    self.response = await response
                    return self.response
                async def __aexit__(self, exc_type, exc_value, traceback):
                    await self.response.aclose()
            return AsyncContextManager()"
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
        stream: Option<bool>,
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
                    "Unsupported data type in request body: {e:#?}"
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

        let method_str = method.to_string();
        let default_encoding = self.default_encoding.clone();
        let stream_value = stream.unwrap_or(false);
        let impit = Arc::clone(&self.impit);

        pyo3_async_runtimes::tokio::future_into_py::<_, ImpitPyResponse>(py, async move {
            let response = match method_str.to_lowercase().as_str() {
                "get" => impit.get(url, Some(options)).await,
                "post" => impit.post(url, Some(body), Some(options)).await,
                "patch" => impit.patch(url, Some(body), Some(options)).await,
                "put" => impit.put(url, Some(body), Some(options)).await,
                "options" => impit.options(url, Some(options)).await,
                "trace" => impit.trace(url, Some(options)).await,
                "head" => impit.head(url, Some(options)).await,
                "delete" => impit.delete(url, Some(options)).await,
                _ => Err(ImpitError::InvalidMethod(method_str.to_string())),
            };

            match response {
                Ok(response) => {
                    let py_response =
                        ImpitPyResponse::from_async(response, default_encoding, stream_value).await;
                    Ok(py_response)
                }
                Err(err) => Err(ImpitPyError(err).into()),
            }
        })
    }
}
