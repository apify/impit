use std::time::Duration;

/// A struct that holds the request options.
///
/// Unlike the [`ImpitBuilder`](crate::impit::ImpitBuilder) struct, these options are specific to a single request.
///
/// Used by the [`Impit`](crate::impit::Impit) struct's methods.
#[derive(Debug, Clone, Default)]
pub struct RequestOptions {
    /// A `Vec` of string pairs that represent custom HTTP request headers. These are added to the default headers.
    pub headers: Vec<(String, String)>,
    /// The timeout for the request. This option overrides the global [`Impit`] timeout.
    pub timeout: Option<Duration>,
    /// Enforce the use of HTTP/3 for this request. This will cause broken responses from servers that don't support HTTP/3.
    ///
    /// If [`ImpitBuilder::with_http3`](crate::impit::ImpitBuilder::with_http3) wasn't called, this option will cause [`ErrorType::Http3Disabled`](crate::impit::ErrorType::Http3Disabled) errors.
    pub http3_prior_knowledge: bool,
}
