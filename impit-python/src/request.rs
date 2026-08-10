use std::{collections::HashMap, time::Duration};

use either::{Either, Left, Right};
use pyo3::{Bound, PyAny};

/// The sentinel string used as the Python default value for per-request `timeout` parameters.
///
/// When a user does not explicitly supply a `timeout`, this string is received by the Rust
/// method and treated as "inherit the client-level default".  It is exposed to Python as
/// the `USE_CLIENT_DEFAULT` module constant so that callers can also pass it explicitly.
pub(crate) const USE_CLIENT_DEFAULT_SENTINEL: &str = "USE_CLIENT_DEFAULT";
/// Parse a Python `timeout` argument into `Option<Option<Duration>>`:
///
/// - `USE_CLIENT_DEFAULT_SENTINEL` string (not provided / explicit sentinel) → `None`
///   (inherit client default)
/// - Python `None` → `Some(None)` (disable timeout)
/// - Python `float` → `Some(Some(Duration))` (specific timeout)
pub(crate) fn parse_timeout(
    timeout: Option<Either<f64, &str>>,
) -> pyo3::PyResult<Option<Option<Duration>>> {
    match timeout {
        None => Ok(Some(None)),
        Some(Left(secs)) => Ok(Some(Some(Duration::from_secs_f64(secs)))),
        Some(Right(s)) => {
            if s != USE_CLIENT_DEFAULT_SENTINEL {
                Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Invalid timeout value: {s:?}"
                )))
            } else {
                Ok(None)
            }
        }
    }
}

use pyo3::exceptions::{PyStopAsyncIteration, PyTypeError};
use pyo3::types::{PyAnyMethods, PyIterator};
use pyo3::{Borrowed, BoundObject, FromPyObject, Py, PyErr, PyResult, Python};

/// Wraps a Python object that implements `__anext__`. Extraction only checks for the attribute
/// (no side effects), unlike `Vec<u8>`/`PyIterator` extraction which can consume items.
pub(crate) struct PyAsyncIterator(pub Py<PyAny>);

impl<'a, 'py> FromPyObject<'a, 'py> for PyAsyncIterator {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, PyErr> {
        if obj.hasattr("__anext__")? {
            Ok(PyAsyncIterator(BoundObject::unbind(obj)))
        } else {
            Err(PyTypeError::new_err("object is not an async iterator"))
        }
    }
}

#[derive(FromPyObject)]
pub(crate) enum RequestBody<'py> {
    #[pyo3(transparent, annotation = "AsyncIterator[bytes]")]
    AsyncIterator(PyAsyncIterator),
    // Placed before `Bytes` because `Vec<u8>` extraction would otherwise partially consume the
    // iterator while trying (and failing) to coerce its items into bytes.
    #[pyo3(transparent, annotation = "Iterator[bytes]")]
    Iterator(Bound<'py, PyIterator>),
    #[pyo3(transparent, annotation = "bytes")]
    Bytes(Vec<u8>),
    #[pyo3(transparent, annotation = "dict[str, str]")]
    Form(HashMap<String, String>),
    #[pyo3(transparent)]
    CatchAll(Bound<'py, PyAny>), // This extraction never fails
}

pub fn iterator_to_bytes(iter: Bound<'_, PyIterator>) -> PyResult<Vec<u8>> {
    let mut body = Vec::new();
    for chunk in iter {
        body.extend(chunk?.extract::<Vec<u8>>()?);
    }
    Ok(body)
}

/// Drains a Python async iterator by repeatedly awaiting `__anext__`, until it raises
/// `StopAsyncIteration`.
pub async fn async_iterator_to_bytes(iter: Py<PyAny>) -> PyResult<Vec<u8>> {
    let mut body = Vec::new();
    loop {
        let next = Python::attach(|py| {
            pyo3_async_runtimes::tokio::into_future(
                iter.call_method0(py, "__anext__")?.into_bound(py),
            )
        })?;

        match next.await {
            Ok(chunk) => {
                body.extend(Python::attach(|py| chunk.extract::<Vec<u8>>(py))?);
            }
            Err(err) if Python::attach(|py| err.is_instance_of::<PyStopAsyncIteration>(py)) => {
                return Ok(body);
            }
            Err(err) => return Err(err),
        }
    }
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
