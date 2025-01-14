use std::{collections::HashMap, str::FromStr};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use crate::emulation::Browser;

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

impl Into<HeaderMap> for HttpHeaders {
    fn into(self) -> HeaderMap {
        let mut headers = HeaderMap::new();

        let header_values = match self.context.browser {
            Some(Browser::Chrome) => statics::CHROME_HEADERS,
            Some(Browser::Firefox) => statics::FIREFOX_HEADERS,
            None => &[]
        };

        let pseudo_headers_order: &[&str] = match self.context.browser {
            Some(Browser::Chrome) => statics::CHROME_PSEUDOHEADERS_ORDER.as_ref(),
            Some(Browser::Firefox) => statics::FIREFOX_PSEUDOHEADERS_ORDER.as_ref(),
            None => &[]
        };

        if pseudo_headers_order.len() != 0 {
            std::env::set_var("RETCH_H2_PSEUDOHEADERS_ORDER", pseudo_headers_order.join(","));
        }

        let mut used_custom_headers: Vec<String> = vec![];

        // TODO: don't use HTTP2 headers for HTTP1.1
        for (name, impersonated_value) in header_values {
            let value: &str = match self.context.custom_headers.get(*name)  { 
                Some(custom_value) => {
                    used_custom_headers.push(name.to_string());
                    custom_value.as_str()
                },
                None => impersonated_value,
            };

            headers.append(
                HeaderName::from_str(name).unwrap(), 
                HeaderValue::from_str(value).unwrap()
            );
        }

        self.context.custom_headers.iter().for_each(|(name, value)| {
            if !used_custom_headers.contains(name) {
                headers.append(
                    HeaderName::from_str(name).unwrap(), 
                    HeaderValue::from_str(value).unwrap()
                );
            }
        });

        headers
    }
}

#[derive(Default, Clone)]
pub struct HttpHeadersBuilder {
    host: String,
    browser: Option<Browser>,
    https: bool,
    custom_headers: HashMap<String, String>,
}

impl HttpHeadersBuilder {
    // TODO: Enforce `with_host` to be called before `build`
    pub fn with_host (&mut self, host: &String) -> &mut Self {
        self.host = host.to_owned();
        self
    }

    pub fn with_browser (&mut self, browser: &Option<Browser>) -> &mut Self {
        self.browser = browser.to_owned();
        self
    }

    pub fn with_https (&mut self, https: bool) -> &mut Self {
        self.https = https;
        self
    }

    pub fn with_custom_headers (&mut self, custom_headers: &HashMap<String, String>) -> &mut Self {
        self.custom_headers = custom_headers.to_owned();
        self
    }

    pub fn build(&self) -> HttpHeaders {
        HttpHeaders::new(self)
    }
}


