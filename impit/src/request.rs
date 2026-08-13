use std::time::Duration;

use bytes::Bytes;
use futures_core::TryStream;
use url::Url;

/// A struct that holds the request options.
///
/// Unlike the [`ImpitBuilder`](crate::impit::ImpitBuilder) struct, these options are specific to a single request.
///
/// Used by the [`Impit`](crate::impit::Impit) struct's methods.
#[derive(Debug, Clone, Default)]
pub struct RequestOptions {
    /// A `Vec` of string pairs that represent custom HTTP request headers. These take precedence over the headers set in [`ImpitBuilder`](crate::impit::ImpitBuilder)
    /// (both from the `with_headers` and the `with_browser` methods).
    pub headers: Vec<(String, String)>,
    /// The per-request timeout, with three possible states:
    ///
    /// - `None` — inherit the client-level default timeout set via [`ImpitBuilder::with_default_timeout`](crate::impit::ImpitBuilder::with_default_timeout).
    /// - `Some(None)` — disable the timeout entirely for this request (wait indefinitely).
    /// - `Some(Some(d))` — use the given duration, overriding the client-level default.
    pub timeout: Option<Option<Duration>>,
    /// Enforce the use of HTTP/3 for this request. This will cause broken responses from servers that don't support HTTP/3.
    ///
    /// If [`ImpitBuilder::with_http3`](crate::impit::ImpitBuilder::with_http3) wasn't called, this option will cause [`ErrorType::Http3Disabled`](crate::impit::ErrorType::Http3Disabled) errors.
    pub http3_prior_knowledge: bool,
}

/// The body of a request.
#[derive(Default)]
pub enum ImpitBody {
    /// No request body.
    #[default]
    Empty,
    /// A body that is fully buffered in memory before the request is sent.
    Bytes(Vec<u8>),
    /// A body that is streamed into the request as its chunks are produced.
    ///
    /// Note that streamed bodies can only be sent once, so requests using them are never retried.
    Stream(reqwest::Body),
    /// A streamed body that has already been sent and cannot be replayed.
    Consumed,
}

impl ImpitBody {
    /// Creates a streaming body from a stream of byte chunks.
    ///
    /// Unlike [`ImpitBody::Bytes`], the chunks are sent as they are produced, so the whole body
    /// never has to be held in memory. The request uses `Transfer-Encoding: chunked` unless a
    /// `Content-Length` header is set explicitly.
    pub fn from_stream<S>(stream: S) -> Self
    where
        S: TryStream + Send + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
        Bytes: From<S::Ok>,
    {
        Self::Stream(reqwest::Body::wrap_stream(stream))
    }

    pub(crate) fn take(&mut self) -> Option<reqwest::Body> {
        match std::mem::replace(self, Self::Consumed) {
            Self::Bytes(bytes) => {
                *self = Self::Bytes(bytes.clone());
                Some(bytes.into())
            }
            Self::Stream(body) => Some(body),
            body => {
                *self = body;
                None
            }
        }
    }

    pub(crate) fn is_replayable(&self) -> bool {
        !matches!(self, Self::Consumed)
    }
}

impl From<Vec<u8>> for ImpitBody {
    fn from(bytes: Vec<u8>) -> Self {
        Self::Bytes(bytes)
    }
}

pub struct ImpitRequest {
    pub url: Url,
    pub body: ImpitBody,
    pub headers: Vec<(String, String)>,
    pub method: String,
}
