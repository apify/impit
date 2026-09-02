use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex as AsyncMutex;

use bytes::Bytes;
use encoding::label::encoding_from_whatwg_label;
use futures::{Stream, StreamExt};
use impit::{errors::ImpitError, utils::ContentType};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyString};
use reqwest::{Response, StatusCode, Version};
use std::pin::Pin;

use crate::errors::ImpitPyError;

type SharedStream =
    Arc<AsyncMutex<Option<Pin<Box<dyn Stream<Item = reqwest::Result<Bytes>> + Send + 'static>>>>>;

#[pyclass]
pub struct PyResponseBytesIterator {
    ready_content: Option<Bytes>,
    stream: Option<Pin<Box<dyn Stream<Item = reqwest::Result<Bytes>> + Send + Sync>>>,
    runtime: tokio::runtime::Handle,
    content_returned: bool,
    parent_response: Option<Py<ImpitPyResponse>>,
}

#[pymethods]
impl PyResponseBytesIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> PyResult<Option<Py<PyBytes>>> {
        if let Some(content) = slf.ready_content.take() {
            slf.content_returned = true;
            let py = slf.py();
            return Ok(Some(PyBytes::new(py, &content).unbind()));
        }

        if slf.content_returned {
            return Ok(None);
        }

        if let Some(parent) = &slf.parent_response {
            let py = slf.py();
            let is_parent_closed = parent.borrow(py).is_closed;

            if is_parent_closed && !slf.content_returned {
                slf.content_returned = true;
                return Ok(None);
            }
        }

        let runtime = slf.runtime.clone();
        let py = slf.py();

        if let Some(stream) = &mut slf.stream {
            let result = py.detach(|| runtime.block_on(stream.next()));

            match result {
                Some(Ok(chunk)) => Ok(Some(PyBytes::new(py, &chunk).unbind())),
                Some(Err(e)) => {
                    slf.content_returned = true;
                    if let Some(parent) = &slf.parent_response {
                        if let Ok(mut parent_ref) = parent.try_borrow_mut(py) {
                            parent_ref.inner_state = InnerResponseState::StreamingClosed;
                            parent_ref.is_stream_consumed = true;
                            parent_ref.is_closed = true;
                        }
                    }
                    Err(ImpitPyError(ImpitError::from(e, None)).into())
                }
                None => {
                    slf.content_returned = true;
                    if let Some(parent) = &slf.parent_response {
                        if let Ok(mut parent_ref) = parent.try_borrow_mut(py) {
                            parent_ref.inner_state = InnerResponseState::StreamingClosed;
                            parent_ref.is_stream_consumed = true;
                            parent_ref.is_closed = true;
                        }
                    }
                    Ok(None)
                }
            }
        } else {
            slf.content_returned = true;
            Ok(None)
        }
    }
}

#[pyclass]
pub struct PyResponseAsyncBytesIterator {
    ready_content: Option<Bytes>,
    stream: Option<SharedStream>,
    content_returned: bool,
    parent_response: Option<Py<ImpitPyResponse>>,
}

#[pymethods]
impl PyResponseAsyncBytesIterator {
    fn __aiter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __anext__<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        if let Some(content) = self.ready_content.take() {
            self.content_returned = true;
            let content = PyBytes::new(py, &content).unbind();
            let future =
                pyo3_async_runtimes::tokio::future_into_py::<_, Py<PyBytes>>(py, async move {
                    Ok(content)
                })?;
            return Ok(future);
        }

        if self.content_returned {
            return Err(pyo3::exceptions::PyStopAsyncIteration::new_err(""));
        }

        if let Some(parent) = &self.parent_response {
            let is_closed = parent.borrow(py).is_closed;
            if is_closed {
                self.content_returned = true;
                return Err(pyo3::exceptions::PyStopAsyncIteration::new_err(
                    "Response is closed",
                ));
            }
        }

        let stream_arc = match &self.stream {
            Some(arc) => arc.clone(),
            None => {
                self.content_returned = true;
                return Err(pyo3::exceptions::PyStopAsyncIteration::new_err(
                    "No stream available",
                ));
            }
        };

        let parent_response = self.parent_response.as_ref().map(|p| p.clone_ref(py));

        let future =
            pyo3_async_runtimes::tokio::future_into_py::<_, Py<PyBytes>>(py, async move {
                let chunk_result = {
                    let mut stream_guard = stream_arc.lock().await;
                    if let Some(stream) = stream_guard.as_mut() {
                        stream.next().await
                    } else {
                        None
                    }
                };
                match chunk_result {
                    Some(Ok(chunk)) => Ok(Python::attach(|py| PyBytes::new(py, &chunk).unbind())),
                    Some(Err(e)) => {
                        if let Some(parent) = parent_response {
                            Python::attach(|py| {
                                if let Ok(mut parent_ref) = parent.try_borrow_mut(py) {
                                    parent_ref.inner_state = InnerResponseState::StreamingClosed;
                                    parent_ref.is_stream_consumed = true;
                                    parent_ref.is_closed = true;
                                }
                            });
                        }
                        Err(ImpitPyError(ImpitError::from(e, None)).into())
                    }
                    None => {
                        if let Some(parent) = parent_response {
                            Python::attach(|py| {
                                if let Ok(mut parent_ref) = parent.try_borrow_mut(py) {
                                    parent_ref.inner_state = InnerResponseState::StreamingClosed;
                                    parent_ref.is_stream_consumed = true;
                                    parent_ref.is_closed = true;
                                }
                            });
                        }
                        Err(pyo3::exceptions::PyStopAsyncIteration::new_err(""))
                    }
                }
            })?;

        Ok(future)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum InnerResponseState {
    Unread,
    Read,
    Streaming,
    StreamingClosed,
}

#[pyclass(name = "Response", dict, weakref)]
#[derive(Debug)]
pub struct ImpitPyResponse {
    #[pyo3(get)]
    status_code: u16,
    #[pyo3(get)]
    reason_phrase: String,
    #[pyo3(get)]
    http_version: String,
    #[pyo3(get)]
    encoding: String,
    #[pyo3(get)]
    is_redirect: bool,
    #[pyo3(get)]
    url: String,
    #[pyo3(get)]
    is_closed: bool,
    #[pyo3(get)]
    is_stream_consumed: bool,
    // #[pyo3(get)]
    // request: Request,
    // #[pyo3(get)]
    // next_request: Option<Request>,
    // #[pyo3(get)]
    // cookies: Cookies,
    // #[pyo3(get)]
    // history: Vec<Response>,
    // #[pyo3(get)]
    // elapsed: Duration,
    text: Option<String>,
    content: Option<Bytes>,
    inner: Option<Response>,
    inner_state: InnerResponseState,
    // Raw, undecoded header name/value byte pairs. The `headers` getter builds the Python-side
    // `Headers` object (httpx-style: str access + `.raw`) from these exact wire bytes.
    raw_headers: Vec<(Vec<u8>, Vec<u8>)>,
}

#[pymethods]
impl ImpitPyResponse {
    #[new]
    #[pyo3(signature = (status_code, content=None, headers=None, url=None, default_encoding="utf-8"))]
    fn new(
        status_code: u16,
        content: Option<Vec<u8>>,
        headers: Option<HashMap<String, String>>,
        url: Option<String>,
        default_encoding: Option<&str>,
    ) -> Self {
        let headers = headers.unwrap_or_default();

        // No wire bytes for a manually constructed response; use the UTF-8 bytes of the strings.
        let raw_headers: Vec<(Vec<u8>, Vec<u8>)> = headers
            .iter()
            .map(|(k, v)| (k.clone().into_bytes(), v.clone().into_bytes()))
            .collect();

        let encoding = match headers
            .iter()
            .find(|(k, _)| k.to_lowercase() == "content-type")
        {
            Some((_, ct)) => ContentType::from(ct)
                .ok()
                .map(|ct| ct.charset)
                .unwrap_or_else(|| default_encoding.unwrap_or("utf-8").to_string()),
            None => default_encoding.unwrap_or("utf-8").to_string(),
        };

        let reason_phrase = StatusCode::from_u16(status_code)
            .map(|s| s.canonical_reason().unwrap_or("Unknown").to_string())
            .unwrap_or_else(|_| "Unknown".to_string());

        Self {
            status_code,
            reason_phrase,
            http_version: "HTTP/1.1".to_string(),
            encoding,
            is_redirect: false,
            url: url.unwrap_or_default(),
            is_closed: true,
            is_stream_consumed: true,
            text: None,
            content: Some(content.map(Bytes::from).unwrap_or_default()),
            inner: None,
            inner_state: InnerResponseState::Read,
            raw_headers,
        }
    }

    fn __repr__(&self) -> String {
        format!("<Response [{} {}]>", self.status_code, self.reason_phrase)
    }

    fn raise_for_status(&self) -> PyResult<()> {
        if self.status_code >= 400 {
            return Err(
                ImpitPyError(impit::errors::ImpitError::HTTPStatusError(self.status_code)).into(),
            );
        }
        Ok(())
    }

    fn aclose(slf: Py<Self>, py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            Self::close_async_impl(slf)?;
            Ok(())
        })
    }

    fn aiter_bytes(slf: Py<Self>, py: Python) -> PyResult<PyResponseAsyncBytesIterator> {
        let mut slf_ref = slf.borrow_mut(py);

        match slf_ref.inner_state {
            InnerResponseState::Read => {
                let content = slf_ref.content.clone();
                drop(slf_ref);

                Ok(PyResponseAsyncBytesIterator {
                    ready_content: content,
                    stream: None,
                    content_returned: false,
                    parent_response: Some(slf),
                })
            }
            InnerResponseState::Unread => {
                slf_ref.inner_state = InnerResponseState::Streaming;

                let response = slf_ref
                    .inner
                    .take()
                    .ok_or(ImpitPyError(impit::errors::ImpitError::StreamClosed))?;

                drop(slf_ref);

                let stream: Pin<Box<dyn Stream<Item = reqwest::Result<Bytes>> + Send>> =
                    Box::pin(response.bytes_stream());
                let shared_stream = Arc::new(AsyncMutex::new(Some(stream)));

                Ok(PyResponseAsyncBytesIterator {
                    ready_content: None,
                    stream: Some(shared_stream),
                    content_returned: false,
                    parent_response: Some(slf),
                })
            }
            InnerResponseState::Streaming | InnerResponseState::StreamingClosed => {
                drop(slf_ref);
                Err(ImpitPyError(impit::errors::ImpitError::StreamConsumed).into())
            }
        }
    }

    fn iter_bytes(slf: Py<Self>, py: Python) -> PyResult<PyResponseBytesIterator> {
        let runtime = pyo3_async_runtimes::tokio::get_runtime().handle().clone();
        let mut slf_ref = slf.borrow_mut(py);

        match slf_ref.inner_state {
            InnerResponseState::Read => {
                let content = slf_ref.content.clone();
                drop(slf_ref);

                Ok(PyResponseBytesIterator {
                    ready_content: content,
                    stream: None,
                    runtime,
                    content_returned: false,
                    parent_response: Some(slf),
                })
            }
            InnerResponseState::Unread => {
                slf_ref.inner_state = InnerResponseState::Streaming;

                let response = slf_ref
                    .inner
                    .take()
                    .ok_or(ImpitPyError(impit::errors::ImpitError::StreamClosed))?;

                drop(slf_ref);

                let stream: Pin<Box<dyn Stream<Item = reqwest::Result<Bytes>> + Send + Sync>> =
                    Box::pin(response.bytes_stream());

                Ok(PyResponseBytesIterator {
                    ready_content: None,
                    stream: Some(stream),
                    runtime,
                    content_returned: false,
                    parent_response: Some(slf),
                })
            }
            InnerResponseState::Streaming | InnerResponseState::StreamingClosed => {
                drop(slf_ref);
                Err(ImpitPyError(impit::errors::ImpitError::StreamConsumed).into())
            }
        }
    }

    fn aread(slf: Py<Self>, py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let (state, response_option, content_option, is_stream_consumed) =
                Python::attach(|py| {
                    let mut slf_ref = slf.borrow_mut(py);
                    (
                        slf_ref.inner_state,
                        slf_ref.inner.take(),
                        slf_ref.content.clone(),
                        slf_ref.is_stream_consumed,
                    )
                });

            match state {
                InnerResponseState::Read => Ok(Python::attach(|py| {
                    PyBytes::new(py, &content_option.unwrap_or_default()).unbind()
                })),
                InnerResponseState::Unread => {
                    if let Some(response) = response_option {
                        let content = response
                            .bytes()
                            .await
                            .map_err(|_| ImpitPyError(impit::errors::ImpitError::NetworkError))?;

                        Ok(Python::attach(|py| {
                            let mut slf_ref = slf.borrow_mut(py);
                            slf_ref.content = Some(content.clone());
                            slf_ref.inner_state = InnerResponseState::Read;
                            slf_ref.is_stream_consumed = true;
                            slf_ref.is_closed = true;
                            drop(slf_ref);

                            PyBytes::new(py, &content).unbind()
                        }))
                    } else {
                        Err(ImpitPyError(impit::errors::ImpitError::StreamClosed).into())
                    }
                }
                InnerResponseState::Streaming | InnerResponseState::StreamingClosed => {
                    if is_stream_consumed {
                        Err(ImpitPyError(impit::errors::ImpitError::StreamConsumed).into())
                    } else {
                        Err(ImpitPyError(impit::errors::ImpitError::StreamClosed).into())
                    }
                }
            }
        })
    }

    pub fn close(&mut self) -> PyResult<()> {
        if self.is_closed {
            return Ok(());
        }

        self.inner = None;
        self.inner_state = match self.inner_state {
            InnerResponseState::Streaming => InnerResponseState::StreamingClosed,
            _ => self.inner_state,
        };
        self.is_closed = true;

        Ok(())
    }

    /// Response headers as an httpx-style [`Headers`] object: case-insensitive `str` access plus a
    /// `.raw` property exposing the exact header value bytes received on the wire (useful for UTF-8
    /// values or header signature/HMAC verification). Built from the raw wire bytes; `Headers`
    /// itself picks the decoding (ascii, then utf-8, then iso-8859-1), matching httpx.
    ///
    /// Read view: each access rebuilds the object, so in-place mutations do not persist.
    #[getter]
    fn headers<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let raw: Vec<(Bound<'py, PyBytes>, Bound<'py, PyBytes>)> = self
            .raw_headers
            .iter()
            .map(|(name, value)| (PyBytes::new(py, name), PyBytes::new(py, value)))
            .collect();
        py.import("impit.headers")?
            .getattr("Headers")?
            .call1((raw,))
    }

    #[getter]
    fn content<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        self.read(py)
    }

    #[getter]
    fn text<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyString>> {
        if self.text.is_none() {
            let decoder = encoding_from_whatwg_label(&self.encoding);
            let decoded_text = impit::utils::decode(self.read_bytes(py)?, decoder);

            self.text = Some(decoded_text);
        }

        Ok(PyString::new(py, self.text.as_deref().unwrap_or_default()))
    }

    fn json(&mut self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let text = self.text(py)?;
        let json_module = py.import("json")?;
        let parsed = json_module.call_method1("loads", (text,))?;
        Ok(parsed.into())
    }

    fn read<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        Ok(PyBytes::new(py, self.read_bytes(py)?))
    }
}

impl ImpitPyResponse {
    /// Buffers the response body if it hasn't been read yet and borrows it, so that the callers
    /// that only need to look at the bytes don't have to copy the whole body first.
    fn read_bytes(&mut self, py: Python<'_>) -> PyResult<&[u8]> {
        match self.inner_state {
            InnerResponseState::Read => self
                .content
                .as_deref()
                .ok_or_else(|| ImpitPyError(impit::errors::ImpitError::ResponseNotRead).into()),
            InnerResponseState::Streaming | InnerResponseState::StreamingClosed => {
                if self.is_stream_consumed {
                    Err(ImpitPyError(impit::errors::ImpitError::StreamConsumed).into())
                } else {
                    Err(ImpitPyError(impit::errors::ImpitError::StreamClosed).into())
                }
            }
            InnerResponseState::Unread => {
                let response = self
                    .inner
                    .take()
                    .ok_or(ImpitPyError(impit::errors::ImpitError::StreamClosed))?;

                let runtime = pyo3_async_runtimes::tokio::get_runtime();
                let content = py.detach(|| {
                    runtime.block_on(async {
                        response
                            .bytes()
                            .await
                            .map_err(|_| ImpitPyError(impit::errors::ImpitError::NetworkError))
                    })
                })?;

                self.inner_state = InnerResponseState::Read;
                self.is_stream_consumed = true;
                self.is_closed = true;

                Ok(self.content.insert(content))
            }
        }
    }

    fn close_async_impl(slf: Py<Self>) -> PyResult<()> {
        Python::attach(|py| {
            let mut slf_ref = slf.borrow_mut(py);
            slf_ref.close()
        })
    }

    pub async fn from_async(
        val: Response,
        preferred_encoding: Option<String>,
        stream: bool,
    ) -> Result<Self, ImpitError> {
        let status_code = val.status().as_u16();
        let url = val.url().to_string();
        let reason_phrase = val
            .status()
            .canonical_reason()
            .unwrap_or_default()
            .to_string();
        let http_version = match val.version() {
            Version::HTTP_09 => "HTTP/0.9".to_string(),
            Version::HTTP_10 => "HTTP/1.0".to_string(),
            Version::HTTP_11 => "HTTP/1.1".to_string(),
            Version::HTTP_2 => "HTTP/2".to_string(),
            Version::HTTP_3 => "HTTP/3".to_string(),
            _ => "Unknown".to_string(),
        };
        let is_redirect = val.status().is_redirection();
        // Exact wire header bytes; the Python `Headers` object (str access + `.raw`) is built from
        // these, and it — not Rust — chooses the decoding (ascii/utf-8/iso-8859-1), matching httpx.
        let raw_headers: Vec<(Vec<u8>, Vec<u8>)> = val
            .headers()
            .iter()
            .map(|(k, v)| (k.as_str().as_bytes().to_vec(), v.as_bytes().to_vec()))
            .collect();

        // Charset detection only needs the content-type value, which is ASCII per RFC 9110
        // (media type + charset are tokens), so a lossy UTF-8 decode is sufficient.
        let content_type_charset = val
            .headers()
            .get("content-type")
            .map(|v| String::from_utf8_lossy(v.as_bytes()).into_owned())
            .and_then(|ct| ContentType::from(&ct).ok())
            .and_then(|ct| ct.into());

        let (content, inner_state, encoding, inner, is_closed, is_stream_consumed) = if !stream {
            let content = val.bytes().await.map_err(|e| ImpitError::from(e, None))?;
            let encoding = preferred_encoding
                .and_then(|e| encoding_from_whatwg_label(&e))
                .or(content_type_charset)
                .or(impit::utils::determine_encoding(&content))
                .unwrap_or(impit::utils::encodings::UTF_8);

            (
                Some(content),
                InnerResponseState::Read,
                encoding,
                None,
                true,
                true,
            )
        } else {
            let encoding = preferred_encoding
                .and_then(|e| encoding_from_whatwg_label(&e))
                .or(content_type_charset)
                .unwrap_or(impit::utils::encodings::UTF_8);
            (
                None,
                InnerResponseState::Unread,
                encoding,
                Some(val),
                false,
                false,
            )
        };

        Ok(ImpitPyResponse {
            status_code,
            url,
            reason_phrase,
            http_version,
            is_redirect,
            encoding: encoding.name().to_string(),
            text: None,
            content,
            is_closed,
            is_stream_consumed,
            inner_state,
            inner,
            raw_headers,
        })
    }
}
