use std::time::Duration;

use bytes::Bytes;
use futures_core::TryStream;
use reqwest::header::HeaderMap;
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
    Bytes(Bytes),
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
    /// never has to be held in memory. On HTTP/1.1, the request uses `Transfer-Encoding: chunked`
    /// unless a `Content-Length` header is set explicitly.
    ///
    /// As streamed bodies can't be replayed, `307` and `308` redirects aren't followed for such
    /// requests - the redirect response is returned to the caller instead.
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

    /// Whether a body is still available to send - `false` only for a streamed body that has
    /// already been consumed by a previous attempt.
    pub(crate) fn is_sendable(&self) -> bool {
        !matches!(self, Self::Consumed)
    }
}

impl<T: Into<Bytes>> From<T> for ImpitBody {
    fn from(bytes: T) -> Self {
        Self::Bytes(bytes.into())
    }
}

pub struct ImpitRequest {
    pub url: Url,
    pub body: ImpitBody,
    pub headers: HeaderMap,
    pub method: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffered_bodies_can_be_sent_repeatedly() {
        let mut body = ImpitBody::from("hello");

        for _ in 0..2 {
            assert!(body.is_sendable());
            assert!(body.take().is_some());
        }
    }

    #[test]
    fn streamed_bodies_can_only_be_sent_once() {
        let mut body =
            ImpitBody::from_stream(futures_util::stream::empty::<Result<Bytes, std::io::Error>>());

        assert!(body.take().is_some());
        assert!(!body.is_sendable());
        assert!(body.take().is_none());
    }

    #[test]
    fn empty_bodies_yield_no_body() {
        let mut body = ImpitBody::Empty;

        assert!(body.take().is_none());
        assert!(body.is_sendable());
    }
}
