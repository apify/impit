use crate::{emulation::Browser, errors::ImpitError};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::{collections::HashSet, str::FromStr};

mod statics;

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

impl From<HttpHeaders> for Result<HeaderMap, ImpitError> {
    fn from(val: HttpHeaders) -> Self {
        let impersonated_headers = match val.context.browser {
            Some(Browser::Chrome) => statics::CHROME_HEADERS,
            Some(Browser::Firefox) => statics::FIREFOX_HEADERS,
            None => &[],
        }
        .to_owned();

        let custom_headers = val
            .context
            .custom_headers
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()));

        let pseudo_headers_order: &[&str] = match val.context.browser {
            Some(Browser::Chrome) => statics::CHROME_PSEUDOHEADERS_ORDER.as_ref(),
            Some(Browser::Firefox) => statics::FIREFOX_PSEUDOHEADERS_ORDER.as_ref(),
            None => &[],
        };

        if !pseudo_headers_order.is_empty() {
            std::env::set_var(
                "IMPIT_H2_PSEUDOHEADERS_ORDER",
                pseudo_headers_order.join(","),
            );
        }

        let mut headers = HeaderMap::new();

        let mut used_header_names: HashSet<String> = HashSet::new();

        for (name, value) in custom_headers.chain(impersonated_headers) {
            if used_header_names.contains(&name.to_lowercase()) {
                continue;
            }

            let header_name = HeaderName::from_str(name);
            let header_value = HeaderValue::from_str(value);

            match (header_name, header_value) {
                (Err(_), _) => {
                    return Err(ImpitError::InvalidHeaderName(name.to_string()));
                }
                (_, Err(_)) => {
                    return Err(ImpitError::InvalidHeaderValue(value.to_string()));
                }
                (Ok(header_name), Ok(header_value)) => {
                    headers.append(header_name, header_value);
                }
            }

            used_header_names.insert(name.to_lowercase());
        }
        Ok(headers)
    }
}

#[derive(Default, Clone)]
pub struct HttpHeadersBuilder {
    host: String,
    browser: Option<Browser>,
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

    pub fn with_https(&mut self, https: bool) -> &mut Self {
        self.https = https;
        self
    }

    pub fn with_custom_headers(&mut self, custom_headers: &Vec<(String, String)>) -> &mut Self {
        self.custom_headers = custom_headers.to_owned();
        self
    }

    pub fn build(&self) -> HttpHeaders {
        HttpHeaders::new(self)
    }
}
