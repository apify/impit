use std::{collections::HashMap, io, time::Duration};

use bytes::Bytes;
use either::{Either, Left, Right};
use futures::{stream, Stream};
use impit::request::ImpitBody;
use pyo3::{
    exceptions::{PyStopAsyncIteration, PyStopIteration, PyTypeError},
    types::{PyAnyMethods, PyMapping},
    Borrowed, Bound, Py, PyAny, PyErr, PyResult, PyTypeInfo, Python,
};
use pyo3_async_runtimes::TaskLocals;

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

use pyo3::FromPyObject;

#[derive(FromPyObject)]
pub(crate) enum RequestBody {
    #[pyo3(transparent, annotation = "bytes")]
    Bytes(Vec<u8>),
    #[pyo3(transparent, annotation = "dict[str, str]")]
    Form(HashMap<String, String>),
    #[pyo3(transparent, annotation = "Iterable[bytes] | AsyncIterable[bytes]")]
    Iterator(PyIterator),
    #[pyo3(transparent)]
    CatchAll(Py<PyAny>), // This extraction never fails
}

/// A Python iterator over the chunks of a request body.
pub(crate) enum PyIterator {
    Sync(Py<PyAny>),
    Async(Py<PyAny>, TaskLocals),
}

impl<'py> FromPyObject<'_, 'py> for PyIterator {
    type Error = PyErr;

    fn extract(object: Borrowed<'_, 'py, PyAny>) -> PyResult<Self> {
        if object.is_instance_of::<PyMapping>() {
            return Err(PyTypeError::new_err("not an iterator over byte chunks"));
        }

        match object.call_method0("__aiter__") {
            Ok(iterator) => Ok(Self::Async(
                iterator.unbind(),
                TaskLocals::with_running_loop(object.py())?.copy_context(object.py())?,
            )),
            Err(_) => Ok(Self::Sync(object.try_iter()?.into_any().unbind())),
        }
    }
}

fn to_chunk<Stop: PyTypeInfo>(next: PyResult<Py<PyAny>>) -> Option<io::Result<Bytes>> {
    Python::attach(|py| match next {
        Ok(chunk) => Some(
            chunk
                .extract::<Vec<u8>>(py)
                .map(Bytes::from)
                .map_err(io::Error::other),
        ),
        Err(err) if err.is_instance_of::<Stop>(py) => None,
        Err(err) => Some(Err(io::Error::other(err))),
    })
}

fn to_stream(iterator: PyIterator) -> impl Stream<Item = io::Result<Bytes>> {
    stream::unfold(iterator, |iterator| async move {
        let chunk = match &iterator {
            PyIterator::Sync(iterator) => {
                let iterator = Python::attach(|py| iterator.clone_ref(py));
                tokio::task::spawn_blocking(move || {
                    to_chunk::<PyStopIteration>(Python::attach(|py| {
                        iterator
                            .bind(py)
                            .call_method0("__next__")
                            .map(Bound::unbind)
                    }))
                })
                .await
                .unwrap_or_else(|err| Some(Err(io::Error::other(err))))
            }
            PyIterator::Async(iterator, locals) => {
                let next = Python::attach(|py| {
                    pyo3_async_runtimes::into_future_with_locals(
                        locals,
                        iterator.bind(py).call_method0("__anext__")?,
                    )
                });
                to_chunk::<PyStopAsyncIteration>(match next {
                    Ok(next) => next.await,
                    Err(err) => Err(err),
                })
            }
        };

        Some((chunk?, iterator))
    })
}

/// Converts the Python request body into an [`ImpitBody`], streaming it if it is an iterator.
pub(crate) fn to_body(
    data: Option<RequestBody>,
    headers: &mut Option<HashMap<String, String>>,
) -> PyResult<ImpitBody> {
    Ok(match data {
        None => ImpitBody::Empty,
        Some(RequestBody::Bytes(bytes)) => bytes.into(),
        Some(RequestBody::Form(form)) => {
            headers.get_or_insert_default().insert(
                "Content-Type".to_string(),
                "application/x-www-form-urlencoded".to_string(),
            );
            form_to_bytes(form).into()
        }
        Some(RequestBody::Iterator(iterator)) => ImpitBody::from_stream(to_stream(iterator)),
        Some(RequestBody::CatchAll(object)) => {
            return Err(Python::attach(|py| {
                PyErr::new::<PyTypeError, _>(format!(
                    "Unsupported data type in request body: {}",
                    object.bind(py).get_type()
                ))
            }))
        }
    })
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
