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
        let secure_origin = is_secure_origin(url);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |now| now.as_secs() as i64);

        Python::attach(|py| {
            for header_value in cookie_headers {
                let Ok(cookie) = std::str::from_utf8(header_value.as_bytes())
                    .map_err(cookie::ParseError::from)
                    .and_then(Cookie::parse)
                else {
                    continue;
                };

                let stored_domain = cookie
                    .domain()
                    .unwrap_or_else(|| url.host_str().unwrap_or_default());
                let stored_path = cookie.path().unwrap_or("");

                // RFC 6265bis §5.7: only a secure origin may set a `Secure` cookie or replace one.
                if !secure_origin
                    && (cookie.secure().unwrap_or(false)
                        || self.has_secure_cookie(py, cookie.name(), stored_domain, stored_path))
                {
                    continue;
                }
                if !prefix_rules_hold(&cookie) {
                    continue;
                }

                // RFC 6265 §5.3: `Max-Age` takes precedence over `Expires`.
                let expires = cookie
                    .max_age()
                    .map(|age| now.saturating_add(age.whole_seconds()))
                    .or_else(|| cookie.expires_datetime().map(|f| f.unix_timestamp()));
                // An already expired cookie removes the stored one, as `CookieJar.extract_cookies` does.
                if expires.is_some_and(|expires| expires <= now) {
                    let key = (stored_domain, stored_path, cookie.name());
                    let _ = self.cookie_jar.call_method1(py, "clear", key);
                    continue;
                }

                let kwargs = PyDict::new(py);

                kwargs.set_item("name", cookie.name()).unwrap_or_default();
                kwargs.set_item("value", cookie.value()).unwrap_or_default();
                kwargs
                    .set_item("path", cookie.path().unwrap_or(""))
                    .unwrap_or_default();
                kwargs
                    .set_item("secure", cookie.secure().unwrap_or(false))
                    .unwrap_or_default();
                kwargs
                    .set_item(
                        "domain",
                        cookie
                            .domain()
                            .unwrap_or_else(|| url.host_str().unwrap_or_default()),
                    )
                    .unwrap_or_default();
                kwargs.set_item("comment", None::<&str>).unwrap_or_default();
                kwargs
                    .set_item("comment_url", None::<&str>)
                    .unwrap_or_default();
                kwargs.set_item("port", None::<&str>).unwrap_or_default();
                kwargs.set_item("port_specified", false).unwrap_or_default();
                kwargs
                    .set_item("path_specified", cookie.path().is_some())
                    .unwrap_or_default();
                kwargs
                    .set_item(
                        "discard",
                        cookie.max_age().is_none() && cookie.expires().is_none(),
                    )
                    .unwrap_or_default();
                kwargs
                    .set_item("domain_specified", cookie.domain().is_some())
                    .unwrap_or_default();
                kwargs
                    .set_item(
                        "domain_initial_dot",
                        cookie.domain().map(|d| d.starts_with('.')),
                    )
                    .unwrap_or_default();
                kwargs.set_item("expires", expires).unwrap_or_default();
                kwargs.set_item("version", 0).unwrap_or_default();

                let rest = PyDict::new(py);
                if let Some(http_only) = cookie.http_only() {
                    rest.set_item("HttpOnly", http_only).unwrap_or_default();
                }

                if let Some(same_site) = cookie.same_site() {
                    let same_site_str = match same_site {
                        cookie::SameSite::Strict => "Strict",
                        cookie::SameSite::Lax => "Lax",
                        cookie::SameSite::None => "None",
                    };
                    rest.set_item("SameSite", same_site_str).unwrap_or_default();
                }

                kwargs.set_item("rest", rest).unwrap_or_default();

                // Malformed cookies and cookie-jar insertion errors are ignored silently.
                let py_cookie = match self.cookie_constructor.call(py, (), Some(&kwargs)) {
                    Ok(py_cookie) => py_cookie,
                    Err(_) => continue,
                };

                let args = match PyTuple::new(py, vec![py_cookie]) {
                    Ok(args) => args,
                    Err(_) => continue,
                };

                let _ = self.cookie_jar.call_method1(py, "set_cookie", args);
            }
        });
    }

    fn cookies(&self, url: &Url) -> Option<reqwest::header::HeaderValue> {
        Python::attach(|py| {
            let cookie_list = PyIterator::from_object(&self.cookie_jar.bind_borrowed(py)).unwrap();

            cookie_list
                .filter_map(|py_cookie| {
                    let py_cookie = py_cookie.unwrap();

                    let domain = py_cookie
                        .getattr("domain")
                        .and_then(|attr| attr.extract::<String>())
                        .unwrap_or_default();
                    let path = py_cookie
                        .getattr("path")
                        .and_then(|attr| attr.extract::<String>())
                        .unwrap_or_default();
                    let secure = py_cookie
                        .getattr("secure")
                        .and_then(|attr| attr.extract::<bool>())
                        .unwrap_or_default();

                    if !domain_matches(url.host_str().unwrap_or_default(), &domain) {
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

/// Checks whether a request `host` may receive a cookie scoped to `cookie_domain`,
/// following the domain matching rules of
/// [RFC 6265, §5.1.3](https://www.rfc-editor.org/rfc/rfc6265#section-5.1.3).
///
/// Leading dot on the cookie domain (e.g. `.example.com`) is ignored, as
/// permitted by [RFC 6265, §4.1.2.3](https://www.rfc-editor.org/rfc/rfc6265#section-4.1.2.3).
///
/// An empty cookie domain imposes no host restriction and therefore matches any host.
fn domain_matches(host: &str, cookie_domain: &str) -> bool {
    let cookie_domain = cookie_domain.strip_prefix('.').unwrap_or(cookie_domain);

    if cookie_domain.is_empty() {
        return true;
    }

    // Host names are case-insensitive; normalise both sides instead of trusting the
    // caller to pass a lowercased host.
    let host = host.to_ascii_lowercase();
    let cookie_domain = cookie_domain.to_ascii_lowercase();

    // RFC 6265 §5.1.3: an IP-address host only matches an identical cookie domain.
    if host.parse::<std::net::IpAddr>().is_ok() {
        return host == cookie_domain;
    }

    // Exact match, or subdomain match where the cookie domain is a suffix of the host
    // on a `.` label boundary (e.g. host `www.example.com`, cookie domain `example.com`).
    host == cookie_domain
        || host
            .strip_suffix(cookie_domain.as_str())
            .is_some_and(|prefix| prefix.ends_with('.'))
}

/// Returns `true` if `url` may set or replace `Secure` cookies: HTTPS, or a loopback host as in browsers.
fn is_secure_origin(url: &Url) -> bool {
    let host = url.host_str().unwrap_or_default();
    let host = host.trim_start_matches('[').trim_end_matches(']');

    url.scheme() == "https"
        || host == "localhost"
        || host.ends_with(".localhost")
        || host
            .parse::<std::net::IpAddr>()
            .is_ok_and(|ip| ip.is_loopback())
}

/// Checks the `__Secure-` and `__Host-` cookie name prefix rules of
/// [RFC 6265bis, §4.1.3](https://www.ietf.org/archive/id/draft-ietf-httpbis-rfc6265bis-22.html#name-cookie-name-prefixes).
fn prefix_rules_hold(cookie: &Cookie) -> bool {
    let name = cookie.name().to_ascii_lowercase();
    let secure = cookie.secure().unwrap_or(false);

    if name.starts_with("__secure-") {
        return secure;
    }
    if name.starts_with("__host-") {
        return secure && cookie.domain().is_none() && cookie.path() == Some("/");
    }
    true
}

impl PythonCookieJar {
    /// Returns `true` if the jar holds a `Secure` cookie with the given name, domain and path.
    fn has_secure_cookie(&self, py: Python<'_>, name: &str, domain: &str, path: &str) -> bool {
        let Ok(cookies) = PyIterator::from_object(&self.cookie_jar.bind_borrowed(py)) else {
            return false;
        };

        cookies.flatten().any(|cookie| {
            let attr = |key: &str| {
                cookie
                    .getattr(key)
                    .and_then(|value| value.extract::<String>())
                    .unwrap_or_default()
            };

            cookie
                .getattr("secure")
                .and_then(|value| value.extract::<bool>())
                .unwrap_or(false)
                && attr("name") == name
                && attr("domain").eq_ignore_ascii_case(domain)
                && attr("path") == path
        })
    }

    pub fn new(py: Python<'_>, cookie_jar: Py<PyAny>) -> Self {
        let httpmodule = PyModule::import(py, "http.cookiejar").unwrap();
        let cookie_constructor = httpmodule.getattr("Cookie").unwrap().into();

        PythonCookieJar {
            cookie_jar,
            cookie_constructor,
        }
    }

    pub fn from_httpx_cookies(py: Python<'_>, cookies: Py<PyAny>) -> PyResult<Self> {
        cookies
            .getattr(py, "jar")
            .map(|jar| PythonCookieJar::new(py, jar))
    }
}
