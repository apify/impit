use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex as AsyncMutex;

use bytes::Bytes;
use encoding::label::encoding_from_whatwg_label;
use futures::{Stream, StreamExt};
use impit::utils::ContentType;
use pyo3::prelude::*;
use reqwest::{Response, Version};
use std::pin::Pin;

use crate::errors::ImpitPyError;

type SharedStream =
    Arc<AsyncMutex<Option<Pin<Box<dyn Stream<Item = reqwest::Result<Bytes>> + Send + 'static>>>>>;

#[pyclass]
pub struct PyResponseBytesIterator {
    ready_content: Option<Vec<u8>>,
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

    fn __next__(mut slf: PyRefMut<'_, Self>) -> PyResult<Option<Vec<u8>>> {
        if let Some(content) = slf.ready_content.take() {
            slf.content_returned = true;
            return Ok(Some(content));
        }

        if slf.content_returned {
            return Ok(None);
        }

        if let Some(parent) = &slf.parent_response {
            let is_parent_closed = Python::with_gil(|py| parent.borrow(py).is_closed);

            if is_parent_closed && !slf.content_returned {
                slf.content_returned = true;
                return Ok(None);
            }
        }

        let runtime = slf.runtime.clone();

        if let Some(stream) = &mut slf.stream {
            match runtime.block_on(stream.next()) {
                Some(Ok(chunk)) => Ok(Some(chunk.to_vec())),
                Some(Err(e)) => {
                    slf.content_returned = true;
                    if let Some(parent) = &slf.parent_response {
                        Python::with_gil(|py| {
                            if let Ok(mut parent_ref) = parent.try_borrow_mut(py) {
                                parent_ref.inner_state = InnerResponseState::StreamingClosed;
                                parent_ref.is_closed = true;
                            }
                        });
                    }
                    Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                        "Stream error: {e}"
                    )))
                }
                None => {
                    slf.content_returned = true;
                    if let Some(parent) = &slf.parent_response {
                        Python::with_gil(|py| {
                            if let Ok(mut parent_ref) = parent.try_borrow_mut(py) {
                                parent_ref.inner_state = InnerResponseState::StreamingClosed;
                                parent_ref.is_stream_consumed = true;
                                parent_ref.is_closed = true;
                            }
                        });
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
    ready_content: Option<Vec<u8>>,
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
            let future =
                pyo3_async_runtimes::tokio::future_into_py::<_, Vec<u8>>(py, async move {
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

        let future = pyo3_async_runtimes::tokio::future_into_py::<_, Vec<u8>>(py, async move {
            let chunk_result = {
                let mut stream_guard = stream_arc.lock().await;
                if let Some(stream) = stream_guard.as_mut() {
                    stream.next().await
                } else {
                    None
                }
            };
            match chunk_result {
                Some(Ok(chunk)) => Ok(chunk.to_vec()),
                Some(Err(e)) => {
                    if let Some(parent) = parent_response {
                        Python::with_gil(|py| {
                            if let Ok(mut parent_ref) = parent.try_borrow_mut(py) {
                                parent_ref.inner_state = InnerResponseState::StreamingClosed;
                                parent_ref.is_closed = true;
                            }
                        });
                    }
                    Err(pyo3::exceptions::PyStopAsyncIteration::new_err(format!(
                        "Stream error: {e}"
                    )))
                }
                None => {
                    if let Some(parent) = parent_response {
                        Python::with_gil(|py| {
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

#[pyclass(name = "Response")]
#[derive(Debug)]
pub struct ImpitPyResponse {
    #[pyo3(get)]
    status_code: u16,
    #[pyo3(get)]
    reason_phrase: String,
    #[pyo3(get)]
    http_version: String,
    #[pyo3(get)]
    headers: HashMap<String, String>,
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
    content: Option<Vec<u8>>,
    inner: Option<Response>,
    inner_state: InnerResponseState,
}

#[pymethods]
impl ImpitPyResponse {
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
                Python::with_gil(|py| {
                    let mut slf_ref = slf.borrow_mut(py);
                    (
                        slf_ref.inner_state,
                        slf_ref.inner.take(),
                        slf_ref.content.clone(),
                        slf_ref.is_stream_consumed,
                    )
                });

            match state {
                InnerResponseState::Read => Ok(content_option.unwrap_or_default()),
                InnerResponseState::Unread => {
                    if let Some(response) = response_option {
                        let content =
                            response.bytes().await.map(|b| b.to_vec()).map_err(|_| {
                                ImpitPyError(impit::errors::ImpitError::NetworkError)
                            })?;

                        Python::with_gil(|py| {
                            let mut slf_ref = slf.borrow_mut(py);
                            slf_ref.content = Some(content.clone());
                            slf_ref.inner_state = InnerResponseState::Read;
                            slf_ref.is_stream_consumed = true;
                            slf_ref.is_closed = true;
                        });

                        Ok(content)
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

    #[getter]
    fn content(&mut self) -> PyResult<Vec<u8>> {
        self.read()
    }

    #[getter]
    fn text(&mut self) -> PyResult<String> {
        if let Some(cached_text) = &self.text {
            return Ok(cached_text.clone());
        }

        let content_bytes = self.read()?;
        let decoder = encoding_from_whatwg_label(&self.encoding);
        let decoded_text = impit::utils::decode(&content_bytes, decoder);

        self.text = Some(decoded_text.clone());
        Ok(decoded_text)
    }

    fn read(&mut self) -> PyResult<Vec<u8>> {
        match self.inner_state {
            InnerResponseState::Read => self
                .content
                .as_ref()
                .cloned()
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

                let content = pyo3_async_runtimes::tokio::get_runtime().block_on(async {
                    response
                        .bytes()
                        .await
                        .map(|b| b.to_vec())
                        .map_err(|_| ImpitPyError(impit::errors::ImpitError::NetworkError))
                })?;

                self.content = Some(content.clone());
                self.inner_state = InnerResponseState::Read;
                self.is_stream_consumed = true;
                self.is_closed = true;

                Ok(content)
            }
        }
    }
}

impl ImpitPyResponse {
    fn close_async_impl(slf: Py<Self>) -> PyResult<()> {
        Python::with_gil(|py| {
            let mut slf_ref = slf.borrow_mut(py);
            slf_ref.close()
        })
    }

    pub async fn from_async(
        val: Response,
        preferred_encoding: Option<String>,
        stream: bool,
    ) -> Self {
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
        let headers = HashMap::from_iter(val.headers().iter().map(|(k, v)| {
            (
                k.as_str().to_string(),
                v.to_str().unwrap_or_default().to_string(),
            )
        }));

        let content_type_charset = headers
            .get("content-type")
            .and_then(|ct| ContentType::from(ct).ok())
            .and_then(|ct| ct.into());

        let (content, inner_state, encoding, inner, is_closed, is_stream_consumed) = if !stream {
            let content = val.bytes().await.map(|b| b.to_vec()).unwrap_or_default();
            let encoding = preferred_encoding
                .and_then(|e| encoding_from_whatwg_label(&e))
                .or(content_type_charset)
                .or(impit::utils::determine_encoding(content.as_slice()))
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

        ImpitPyResponse {
            status_code,
            url,
            reason_phrase,
            http_version,
            is_redirect,
            headers,
            encoding: encoding.name().to_string(),
            text: None,
            content,
            is_closed,
            is_stream_consumed,
            inner_state,
            inner,
        }
    }
}
