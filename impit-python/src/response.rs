use std::collections::HashMap;

use bytes::Bytes;
use encoding::label::encoding_from_whatwg_label;
use futures::{Stream, StreamExt};
use impit::utils::ContentType;
use pyo3::prelude::*;
use reqwest::{Response, Version};
use std::pin::Pin;

use crate::errors::ImpitPyError;

#[pyclass]
struct PyResponseBytesIterator {
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

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<Vec<u8>> {
        if let Some(content) = slf.ready_content.take() {
            slf.content_returned = true;
            return Some(content);
        }

        if slf.content_returned {
            return None;
        }

        if let Some(parent) = &slf.parent_response {
            let is_parent_closed = Python::with_gil(|py| parent.borrow(py).is_closed);

            if is_parent_closed && !slf.content_returned {
                slf.content_returned = true;
                return None;
            }
        }

        let runtime = slf.runtime.clone();

        if let Some(stream) = &mut slf.stream {
            match runtime.block_on(stream.next()) {
                Some(Ok(chunk)) => Some(chunk.to_vec()),
                Some(Err(_)) => {
                    slf.content_returned = true;
                    None
                }
                None => {
                    slf.content_returned = true;
                    if let Some(parent) = &slf.parent_response {
                        Python::with_gil(|py| {
                            let mut parent_ref = parent.borrow_mut(py);
                            parent_ref.inner_state = InnerResponseState::StreamingClosed;
                            parent_ref.is_stream_consumed = true;
                            parent_ref.is_closed = true;
                        });
                    }
                    None
                }
            }
        } else {
            slf.content_returned = true;
            None // Up error (incorrect usage)
        }
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

    pub fn __enter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    pub fn __exit__(
        &mut self,
        _exc_type: &Bound<'_, PyAny>,
        _exc_value: &Bound<'_, PyAny>,
        _traceback: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        self.close()
    }

    fn raise_for_status(&self) -> PyResult<()> {
        if self.status_code >= 400 {
            return Err(
                ImpitPyError(impit::errors::ImpitError::HTTPStatusError(self.status_code)).into(),
            );
        }
        Ok(())
    }

    fn read(&mut self) -> PyResult<Vec<u8>> {
        match self.inner_state {
            InnerResponseState::Read => self
                .content
                .as_ref()
                .cloned()
                .ok_or_else(|| ImpitPyError(impit::errors::ImpitError::UrlParsingError).into()),
            InnerResponseState::Streaming | InnerResponseState::StreamingClosed => {
                Err(ImpitPyError(impit::errors::ImpitError::UrlParsingError).into())
            }
            InnerResponseState::Unread => {
                let response = self
                    .inner
                    .take()
                    .ok_or_else(|| ImpitPyError(impit::errors::ImpitError::StreamClosed))?;

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

    fn iter_bytes(slf: Py<Self>, py: Python) -> PyResult<PyResponseBytesIterator> {
        let runtime = pyo3_async_runtimes::tokio::get_runtime().handle().clone();

        let (current_state, content, response) = {
            let mut slf_ref = slf.borrow_mut(py);
            let state = slf_ref.inner_state;
            match state {
                InnerResponseState::Read => (state, slf_ref.content.clone(), None),
                InnerResponseState::Unread => {
                    slf_ref.inner_state = InnerResponseState::Streaming;
                    let response = slf_ref.inner.take();
                    (InnerResponseState::Streaming, None, response)
                }
                _ => (state, None, None),
            }
        };

        match current_state {
            InnerResponseState::Read => Ok(PyResponseBytesIterator {
                ready_content: content,
                stream: None,
                runtime,
                content_returned: false,
                parent_response: None,
            }),
            InnerResponseState::Streaming => {
                let response = response
                    .ok_or_else(|| ImpitPyError(impit::errors::ImpitError::StreamClosed))?;

                Ok(PyResponseBytesIterator {
                    ready_content: None,
                    stream: Some(Box::pin(response.bytes_stream())),
                    runtime,
                    content_returned: false,
                    parent_response: Some(slf), // Теперь можно безопасно передать
                })
            }
            _ => Err(ImpitPyError(impit::errors::ImpitError::UrlParsingError).into()),
        }
    }

    fn close(&mut self) -> PyResult<()> {
        if self.is_closed {
            return Ok(());
        }

        self.inner = None;
        self.inner_state = InnerResponseState::StreamingClosed;
        self.is_closed = true;
        self.is_stream_consumed = true;

        Ok(())
    }

    #[getter]
    fn content(&mut self) -> PyResult<Vec<u8>> {
        return self.read();
    }

    #[getter]
    fn text(&mut self) -> PyResult<String> {
        if let Some(cached_text) = &self.text {
            return Ok(cached_text.clone());
        }
        let decoder = encoding_from_whatwg_label(&self.encoding);

        let content_bytes = self.read()?;
        let decoded_text = impit::utils::decode(&content_bytes, decoder);

        self.text = Some(decoded_text.clone());

        Ok(decoded_text)
    }
}

impl ImpitPyResponse {
    pub fn from(val: Response, preferred_encoding: Option<String>, stream: bool) -> Self {
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
            let content = pyo3_async_runtimes::tokio::get_runtime()
                .block_on(async { val.bytes().await.map(|b| b.to_vec()).unwrap_or_default() });
            let encoding = preferred_encoding
                .and_then(|e| encoding::label::encoding_from_whatwg_label(&e))
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
                .and_then(|e| encoding::label::encoding_from_whatwg_label(&e))
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
            content: content,
            is_closed,
            is_stream_consumed,
            inner_state,
            inner,
        }
    }
}
