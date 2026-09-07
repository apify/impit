use crate::{errors::ImpitError, fingerprint::BrowserFingerprint};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::str::FromStr;

/// Merges the impersonated fingerprint headers with the caller-supplied ones into a [`HeaderMap`].
///
/// Header names are matched case-insensitively and the first source that provides a name wins.
/// Custom header sources are consulted in reverse registration order (the last
/// [`with_custom_headers`](HttpHeadersBuilder::with_custom_headers) call has the highest priority),
/// followed by the fingerprint headers. An empty value drops the header instead of sending it.
#[derive(Default)]
pub struct HttpHeadersBuilder<'a> {
    fingerprint: Option<&'a BrowserFingerprint>,
    custom_headers: Vec<&'a [(String, String)]>,
}

impl<'a> HttpHeadersBuilder<'a> {
    pub fn with_fingerprint(mut self, fingerprint: Option<&'a BrowserFingerprint>) -> Self {
        self.fingerprint = fingerprint;
        self
    }

    pub fn with_custom_headers(mut self, custom_headers: &'a [(String, String)]) -> Self {
        self.custom_headers.push(custom_headers);
        self
    }

    pub fn build(self) -> Result<HeaderMap, ImpitError> {
        let fingerprint_headers = self
            .fingerprint
            .map(|fingerprint| fingerprint.headers.as_slice())
            .unwrap_or_default();

        let mut headers = HeaderMap::new();
        let mut dropped: Vec<HeaderName> = Vec::new();

        for (name, value) in self
            .custom_headers
            .iter()
            .rev()
            .copied()
            .chain(std::iter::once(fingerprint_headers))
            .flatten()
        {
            if value.is_empty() {
                // An empty value removes the header, but still shadows the lower-priority sources.
                if let Ok(name) = HeaderName::from_str(name) {
                    dropped.push(name);
                }
                continue;
            }

            let name = HeaderName::from_str(name)
                .map_err(|_| ImpitError::InvalidHeaderName(name.clone()))?;

            if headers.contains_key(&name) || dropped.contains(&name) {
                continue;
            }

            let value = HeaderValue::from_str(value)
                .map_err(|_| ImpitError::InvalidHeaderValue(value.clone()))?;

            headers.insert(name, value);
        }

        Ok(headers)
    }
}
