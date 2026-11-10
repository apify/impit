use std::{collections::HashMap, sync::Arc};

use futures::future::BoxFuture;
use impit::{
    errors::ImpitError,
    request::{ImpitRequest, RequestHooks},
};
use pyo3::{prelude::*, types::PyBytes};
use pyo3_async_runtimes::TaskLocals;
use reqwest::{
    header::{HeaderName, HeaderValue},
    Response,
};

use crate::response::ImpitPyResponse;

/// The request passed to the `request` event hooks. Changes to its `headers` apply to the sent request.
#[pyclass(frozen)]
pub(crate) struct Request {
    #[pyo3(get)]
    method: String,
    #[pyo3(get)]
    url: String,
    #[pyo3(get)]
    headers: Py<PyAny>,
}

/// The `event_hooks` of a client, called for every request and response of a redirect chain.
#[derive(Debug)]
pub(crate) struct EventHooks {
    request: Vec<Py<PyAny>>,
    response: Vec<Py<PyAny>>,
    default_encoding: Option<String>,
    /// Set for `AsyncClient`, whose hooks are awaited.
    locals: Option<TaskLocals>,
}

impl EventHooks {
    pub(crate) fn new(
        hooks: Option<HashMap<String, Vec<Py<PyAny>>>>,
        default_encoding: Option<String>,
    ) -> Self {
        let mut hooks = hooks.unwrap_or_default();
        Self {
            request: hooks.remove("request").unwrap_or_default(),
            response: hooks.remove("response").unwrap_or_default(),
            default_encoding,
            locals: None,
        }
    }

    /// Returns the hooks for a single request, or `None` if there are no hooks.
    pub(crate) fn for_request(
        &self,
        py: Python<'_>,
        is_async: bool,
    ) -> PyResult<Option<Arc<Self>>> {
        if self.request.is_empty() && self.response.is_empty() {
            return Ok(None);
        }

        Ok(Some(Arc::new(Self {
            request: self.request.iter().map(|hook| hook.clone_ref(py)).collect(),
            response: self
                .response
                .iter()
                .map(|hook| hook.clone_ref(py))
                .collect(),
            default_encoding: self.default_encoding.clone(),
            locals: match is_async {
                true => Some(TaskLocals::with_running_loop(py)?.copy_context(py)?),
                false => None,
            },
        })))
    }

    async fn call(&self, hooks: &[Py<PyAny>], argument: &Py<PyAny>) -> PyResult<()> {
        for hook in hooks {
            let result = Python::attach(|py| hook.call1(py, (argument,)))?;
            if let Some(locals) = &self.locals {
                Python::attach(|py| {
                    pyo3_async_runtimes::into_future_with_locals(locals, result.into_bound(py))
                })?
                .await?;
            }
        }
        Ok(())
    }

    /// Runs the `response` hooks of `Client` with the final response.
    pub(crate) fn on_response_sync(
        &self,
        py: Python<'_>,
        response: &Py<ImpitPyResponse>,
    ) -> PyResult<()> {
        for hook in &self.response {
            hook.call1(py, (response,))?;
        }
        Ok(())
    }

    /// Runs the `response` hooks of `AsyncClient` with the final response.
    pub(crate) async fn on_response(
        &self,
        response: ImpitPyResponse,
    ) -> PyResult<Py<ImpitPyResponse>> {
        let response = Python::attach(|py| Py::new(py, response))?;
        self.call(&self.response, response.as_any()).await?;
        Ok(response)
    }
}

fn hook_error(err: PyErr) -> ImpitError {
    ImpitError::HookError(Box::new(err))
}

impl RequestHooks for EventHooks {
    fn on_request<'a>(
        &'a self,
        request: &'a mut ImpitRequest,
    ) -> BoxFuture<'a, Result<(), ImpitError>> {
        Box::pin(async move {
            if self.request.is_empty() {
                return Ok(());
            }

            let py_request = Python::attach(|py| {
                let headers: Vec<_> = request
                    .headers
                    .iter()
                    .map(|(name, value)| {
                        (
                            PyBytes::new(py, name.as_ref()),
                            PyBytes::new(py, value.as_bytes()),
                        )
                    })
                    .collect();
                Py::new(
                    py,
                    Request {
                        method: request.method.to_string(),
                        url: request.url.to_string(),
                        headers: py
                            .import("impit.headers")?
                            .getattr("Headers")?
                            .call1((headers,))?
                            .unbind(),
                    },
                )
            })
            .map_err(hook_error)?;

            self.call(&self.request, py_request.as_any())
                .await
                .map_err(hook_error)?;

            let headers: Vec<(Vec<u8>, Vec<u8>)> =
                Python::attach(|py| py_request.get().headers.bind(py).getattr("raw")?.extract())
                    .map_err(hook_error)?;
            request.headers = headers
                .into_iter()
                .map(|(name, value)| {
                    let name = HeaderName::from_bytes(&name).map_err(|_| {
                        ImpitError::InvalidHeaderName(String::from_utf8_lossy(&name).into_owned())
                    })?;
                    let value = HeaderValue::from_bytes(&value).map_err(|_| {
                        ImpitError::InvalidHeaderValue(String::from_utf8_lossy(&value).into_owned())
                    })?;
                    Ok((name, value))
                })
                .collect::<Result<_, ImpitError>>()?;

            Ok(())
        })
    }

    fn on_redirect(&self, response: Response) -> BoxFuture<'_, Result<(), ImpitError>> {
        Box::pin(async move {
            if self.response.is_empty() {
                return Ok(());
            }

            let response =
                ImpitPyResponse::from_async(response, self.default_encoding.clone(), false).await?;
            self.on_response(response).await.map_err(hook_error)?;
            Ok(())
        })
    }
}
