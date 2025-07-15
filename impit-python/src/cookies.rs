use std::time::{SystemTime, UNIX_EPOCH};

use cookie::Cookie;
use pyo3::{
    prelude::*,
    types::{IntoPyDict, PyDict, PyIterator, PyTuple},
};
use reqwest::{cookie::CookieStore, Url};

pub struct PythonCookieJar {
    cookie_jar: Py<PyAny>,
    cookie_constructor: Py<PyAny>,
}

impl CookieStore for PythonCookieJar {
    fn set_cookies(
        &self,
        cookie_headers: &mut dyn Iterator<Item = &reqwest::header::HeaderValue>,
        url: &Url,
    ) {
        Python::with_gil(|py| {
            for header_value in cookie_headers {
                let cookie = std::str::from_utf8(header_value.as_bytes())
                    .map_err(cookie::ParseError::from)
                    .and_then(Cookie::parse)
                    .unwrap();

                let kwargs = PyDict::new(py);

                kwargs.set_item("name", cookie.name()).unwrap();
                kwargs.set_item("value", cookie.value()).unwrap();
                kwargs
                    .set_item("path", cookie.path().unwrap_or(""))
                    .unwrap();
                kwargs
                    .set_item("secure", cookie.secure().unwrap_or(false))
                    .unwrap();
                kwargs
                    .set_item(
                        "domain",
                        cookie
                            .domain()
                            .unwrap_or_else(|| url.host_str().unwrap_or_default()),
                    )
                    .unwrap();
                kwargs.set_item("comment", None::<&str>).unwrap();
                kwargs.set_item("comment_url", None::<&str>).unwrap();
                kwargs.set_item("port", None::<&str>).unwrap();
                kwargs.set_item("port_specified", false).unwrap();
                kwargs
                    .set_item("path_specified", cookie.path().is_some())
                    .unwrap();
                kwargs
                    .set_item(
                        "discard",
                        cookie.max_age().is_none() && cookie.expires().is_none(),
                    )
                    .unwrap();
                kwargs
                    .set_item("domain_specified", cookie.domain().is_some())
                    .unwrap();
                kwargs
                    .set_item(
                        "domain_initial_dot",
                        cookie.domain().map(|d| d.starts_with('.')),
                    )
                    .unwrap();
                kwargs
                    .set_item(
                        "expires",
                        cookie.expires_datetime().map(|f| f.unix_timestamp()),
                    )
                    .unwrap();
                kwargs.set_item("version", 0).unwrap();

                let rest = PyDict::new(py);
                if let Some(http_only) = cookie.http_only() {
                    rest.set_item("HttpOnly", http_only).unwrap();
                }

                if let Some(same_site) = cookie.same_site() {
                    let same_site_str = match same_site {
                        cookie::SameSite::Strict => "Strict",
                        cookie::SameSite::Lax => "Lax",
                        cookie::SameSite::None => "None",
                    };
                    rest.set_item("SameSite", same_site_str).unwrap();
                }

                kwargs.set_item("rest", rest).unwrap();

                let py_cookie = self.cookie_constructor.call(py, (), Some(&kwargs)).unwrap();

                let args = PyTuple::new(py, vec![py_cookie]).unwrap();

                self.cookie_jar
                    .call_method1(py, "set_cookie", args)
                    .unwrap();
            }
        });
    }

    fn cookies(&self, url: &Url) -> Option<reqwest::header::HeaderValue> {
        Python::with_gil(|py| {
            let cookie_list = PyIterator::from_object(&self.cookie_jar.bind_borrowed(py)).unwrap();

            cookie_list
                .filter_map(|py_cookie| {
                    let py_cookie = py_cookie.unwrap();

                    let domain = py_cookie
                        .getattr("domain")
                        .unwrap()
                        .extract::<String>()
                        .unwrap();
                    let path = py_cookie
                        .getattr("path")
                        .unwrap()
                        .extract::<String>()
                        .unwrap();
                    let secure = py_cookie
                        .getattr("secure")
                        .unwrap()
                        .extract::<bool>()
                        .unwrap_or_default();

                    if !domain.is_empty() && !url.host_str().unwrap_or_default().contains(&domain) {
                        return None;
                    }
                    if !url.path().starts_with(&path) {
                        return None;
                    }
                    if secure && !url.scheme().eq("https") {
                        return None;
                    }
                    let is_expired = py_cookie
                        .getattr("is_expired")
                        .unwrap()
                        .call(
                            (),
                            [(
                                "now",
                                SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .ok()
                                    .map(|now| now.as_secs()),
                            )]
                            .into_py_dict(py)
                            .ok()
                            .as_ref(),
                        )
                        .unwrap();

                    if is_expired.is_truthy().unwrap() {
                        None
                    } else {
                        let name = py_cookie
                            .getattr("name")
                            .unwrap()
                            .extract::<String>()
                            .unwrap();
                        let value = py_cookie
                            .getattr("value")
                            .unwrap()
                            .extract::<String>()
                            .unwrap();

                        Some(format!("{name}={value}"))
                    }
                })
                .collect::<Vec<String>>()
                .join("; ")
                .parse::<reqwest::header::HeaderValue>()
                .ok()
        })
    }
}

impl PythonCookieJar {
    pub fn new(py: Python<'_>, cookie_jar: Py<PyAny>) -> Self {
        let httpmodule = PyModule::import(py, "http.cookiejar").unwrap();
        let cookie_constructor = httpmodule.getattr("Cookie").unwrap().into();

        PythonCookieJar {
            cookie_jar,
            cookie_constructor,
        }
    }

    pub fn from_httpx_cookies(py: Python<'_>, cookies: Py<PyAny>) -> Self {
        PythonCookieJar::new(py, cookies.getattr(py, "jar").unwrap())
    }
}
