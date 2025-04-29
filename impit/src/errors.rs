use std::time::Duration;

use thiserror::Error;

pub struct ErrorContext {
    pub timeout: Duration,
    pub max_redirects: usize,
    pub method: String,
    pub protocol: String,
    pub url: String,
}

/// Error types that can be returned by the [`Impit`] struct.
///
/// The `ImpitError` enum is used to represent the different types of errors that can occur when making requests.
/// The `RequestError` variant is used to wrap the `reqwest::Error` type.
#[derive(Error, Debug)]
pub enum ImpitError {
    #[error("HTTP error occurred.")]
    HTTPError,
    #[error("Request error occurred.")]
    RequestError,
    #[error("Transport error occurred.")]
    TransportError,
    #[error("Request timeout ({0}) ms exceeded.")]
    TimeoutException(u128),
    #[error("Connection timed out.")]
    ConnectTimeout,
    #[error("Read operation timed out.")]
    ReadTimeout,
    #[error("Write operation timed out.")]
    WriteTimeout,
    #[error("Connection pool timed out.")]
    PoolTimeout,
    #[error("Network error occurred.")]
    NetworkError,
    #[error("Failed to connect to the server.")]
    ConnectError,
    #[error("Failed to read data from the server.")]
    ReadError,
    #[error("Failed to write data to the server.")]
    WriteError,
    #[error("Failed to close the connection.")]
    CloseError,
    #[error("Protocol error occurred.")]
    ProtocolError,
    #[error("Local protocol error occurred.")]
    LocalProtocolError,
    #[error("Remote protocol error occurred.")]
    RemoteProtocolError,
    #[error("The proxy URL `{0}` is invalid or unreachable.")]
    ProxyError(String),
    #[error("The protocol is unsupported.")]
    UnsupportedProtocol,
    #[error("The response body couldn't be decoded.")]
    DecodingError,
    #[error("Too many redirects occurred. Maximum allowed: {0}")]
    TooManyRedirects(usize),
    #[error("HTTP status error occurred with status code {0}.")]
    HTTPStatusError(u16),
    #[error("The URL is invalid.")]
    InvalidURL,
    #[error("A cookie conflict occurred.")]
    CookieConflict,
    #[error("A stream error occurred.")]
    StreamError,
    #[error("The stream has already been consumed.")]
    StreamConsumed,
    #[error("The response has not been read.")]
    ResponseNotRead,
    #[error("The request has not been read.")]
    RequestNotRead,
    #[error("The stream has been closed.")]
    StreamClosed,
    #[error("The URL couldn't be parsed.")]
    UrlParsingError,
    #[error("The URL ({0}) is missing the hostname.")]
    UrlMissingHostnameError(String),
    #[error("The URL uses an unsupported protocol (`{0}`). Currently, only HTTP and HTTPS are supported.")]
    UrlProtocolError(String),
    #[error("The request was made with http3_prior_knowledge, but HTTP/3 usage wasn't enabled.")]
    Http3Disabled,
    #[error("The request method `{0}` is invalid. Only GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS and TRACE are supported.")]
    InvalidMethod(String),
    #[error("{0}")]
    BindingPassthroughError(String),
    #[error("The header name `{0}` is invalid. Header names must be ASCII and cannot contain control characters or whitespace.")]
    InvalidHeaderName(String),
    #[error("The header value `{0}` is invalid.")]
    InvalidHeaderValue(String),
    #[error("{0:#?}")]
    ReqwestError(reqwest::Error),
}

impl ImpitError {
    pub fn from(error: reqwest::Error, context: ErrorContext) -> Self {
        if error.is_timeout() {
            return ImpitError::TimeoutException(context.timeout.as_millis());
        }

        if error.is_redirect() {
            return ImpitError::TooManyRedirects(context.max_redirects);
        }

        ImpitError::RequestError
    }
}
