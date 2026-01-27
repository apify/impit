use crate::{emulation::Browser, errors::ImpitError, fingerprint::BrowserFingerprint};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::{collections::HashSet, str::FromStr};

pub mod statics;

pub struct HttpHeaders {
    context: HttpHeadersBuilder,
}

impl HttpHeaders {
    pub fn new(options: &HttpHeadersBuilder) -> HttpHeaders {
        HttpHeaders {
            context: options.clone(),
        }
    }

    pub fn get_builder() -> HttpHeadersBuilder {
        HttpHeadersBuilder::default()
    }
}

impl HttpHeaders {
    pub fn iter(&self) -> impl Iterator<Item = (String, String)> + '_ {
        // Use fingerprint headers if available, otherwise fall back to browser enum
        let impersonated_headers: Vec<(String, String)> =
            if let Some(ref fp) = self.context.fingerprint {
                fp.headers.to_vec()
            } else {
                match self.context.browser {
                    Some(Browser::Chrome) => statics::CHROME_HEADERS
                        .iter()
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                        .collect(),
                    Some(Browser::Firefox) => statics::FIREFOX_HEADERS
                        .iter()
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                        .collect(),
                    None => vec![],
                }
            };

        let custom_headers = self
            .context
            .custom_headers
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()));

        let mut used_header_names: HashSet<String> = HashSet::new();

        custom_headers
            .chain(impersonated_headers)
            .filter_map(move |(name, value)| {
                if used_header_names.contains(&name.to_lowercase()) {
                    None
                } else {
                    used_header_names.insert(name.to_lowercase());
                    Some((name, value))
                }
            })
    }
}

impl From<Vec<(String, String)>> for HttpHeaders {
    fn from(val: Vec<(String, String)>) -> Self {
        let mut builder = HttpHeaders::get_builder();
        builder.with_custom_headers(Some(val));
        builder.build()
    }
}

impl From<HttpHeaders> for Result<HeaderMap, ImpitError> {
    fn from(val: HttpHeaders) -> Self {
        let mut headers = HeaderMap::new();

        for (name, value) in val.iter() {
            let header_name = HeaderName::from_str(&name);
            let header_value = HeaderValue::from_str(&value);

            match (header_name, header_value) {
                (Err(_), _) => {
                    return Err(ImpitError::InvalidHeaderName(name));
                }
                (_, Err(_)) => {
                    return Err(ImpitError::InvalidHeaderValue(value));
                }
                (Ok(header_name), Ok(header_value)) => {
                    headers.append(header_name, header_value);
                }
            }
        }
        Ok(headers)
    }
}

#[derive(Default, Clone)]
pub struct HttpHeadersBuilder {
    host: String,
    browser: Option<Browser>,
    fingerprint: Option<BrowserFingerprint>,
    https: bool,
    custom_headers: Vec<(String, String)>,
}

impl HttpHeadersBuilder {
    // TODO: Enforce `with_host` to be called before `build`
    pub fn with_host(&mut self, host: &String) -> &mut Self {
        self.host = host.to_owned();
        self
    }

    pub fn with_browser(&mut self, browser: &Option<Browser>) -> &mut Self {
        self.browser = browser.to_owned();
        self
    }

    pub fn with_fingerprint(&mut self, fingerprint: &Option<BrowserFingerprint>) -> &mut Self {
        self.fingerprint = fingerprint.clone();
        self
    }

    pub fn with_https(&mut self, https: bool) -> &mut Self {
        self.https = https;
        self
    }

    pub fn with_custom_headers(
        &mut self,
        custom_headers: Option<Vec<(String, String)>>,
    ) -> &mut Self {
        match custom_headers {
            Some(headers) => {
                // Later call to with_custom_headers should override existing headers.
                // We need to prepend the new headers (higher prio) to the existing ones (lower prio).
                self.custom_headers = headers
                    .iter()
                    .chain(self.custom_headers.iter())
                    .map(|(k, v)| (k.to_owned(), v.to_owned()))
                    .collect();
                self
            }
            None => self,
        }
    }

    pub fn build(&self) -> HttpHeaders {
        HttpHeaders::new(self)
    }
}
