use std::{collections::HashMap, time::Duration};

use pyo3::{prelude::*, types::PyAnyMethods, Bound, PyAny};

/// The sentinel string used as the Python default value for per-request `timeout` parameters.
///
/// When a user does not explicitly supply a `timeout`, this string is received by the Rust
/// method and treated as "inherit the client-level default".  It is exposed to Python as
/// the `USE_CLIENT_DEFAULT` module constant so that callers can also pass it explicitly.
pub(crate) const USE_CLIENT_DEFAULT_SENTINEL: &str = "__impit_use_client_default__";

/// Returns the sentinel value used as the Python default for per-request `timeout`.
///
/// Calling convention: `#[pyo3(signature = (timeout=crate::request::default_timeout()))]`.
pub(crate) fn default_timeout() -> Py<PyAny> {
    Python::attach(|py| pyo3::intern!(py, USE_CLIENT_DEFAULT_SENTINEL).clone().into())
}

/// Parse a Python `timeout` argument into `Option<Option<Duration>>`:
///
/// - `USE_CLIENT_DEFAULT_SENTINEL` string (not provided / explicit sentinel) → `None`
///   (inherit client default)
/// - Python `None` → `Some(None)` (disable timeout)
/// - Python `float` → `Some(Some(Duration))` (specific timeout)
pub(crate) fn timeout_from_pyobj(
    timeout: &Bound<'_, PyAny>,
) -> pyo3::PyResult<Option<Option<Duration>>> {
    if timeout.is_none() {
        Ok(Some(None))
    } else if let Ok(s) = timeout.extract::<&str>() {
        if s == USE_CLIENT_DEFAULT_SENTINEL {
            Ok(None)
        } else {
            Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Invalid timeout value: {s:?}"
            )))
        }
    } else {
        let secs = timeout.extract::<f64>()?;
        Ok(Some(Some(Duration::from_secs_f64(secs))))
    }
}

use pyo3::FromPyObject;

#[derive(FromPyObject)]
pub(crate) enum RequestBody<'py> {
    #[pyo3(transparent, annotation = "bytes")]
    Bytes(Vec<u8>),
    #[pyo3(transparent, annotation = "dict[str, str]")]
    Form(HashMap<String, String>),
    #[pyo3(transparent)]
    CatchAll(Bound<'py, PyAny>), // This extraction never fails
}

pub fn form_to_bytes(data: HashMap<String, String>) -> Vec<u8> {
    let mut body = Vec::new();
    for (key, value) in data {
        body.extend_from_slice(urlencoding::encode(key.as_str()).as_bytes());
        body.extend_from_slice("=".as_bytes());
        body.extend_from_slice(urlencoding::encode(value.as_str()).as_bytes());
        body.extend_from_slice("&".as_bytes());
    }
    body.pop(); // Remove the last "&"
    body
}
