use tokio::sync::RwLock;

use log::debug;
use reqwest::{cookie::CookieStore, header::HeaderMap, Method, Response, Version};
use std::{fmt::Debug, str::FromStr, sync::Arc, time::Duration};
use url::Url;

use crate::{
    emulation::Browser,
    errors::{ErrorContext, ImpitError},
    http3::H3Engine,
    http_headers::HttpHeaders,
    request::RequestOptions,
    tls,
};

/// Impit is the main struct used to make (impersonated) requests.
///
/// It uses `reqwest::Client` to make requests and holds info about the impersonated browser.
///
/// To create a new [`Impit`] instance, use the [`Impit::builder()`](ImpitBuilder) method.
pub struct Impit<CookieStoreImpl: CookieStore + 'static> {
    pub(self) base_client: reqwest::Client,
    pub(self) h3_client: Option<reqwest::Client>,
    h3_engine: Arc<RwLock<Option<H3Engine>>>,
    config: ImpitBuilder<CookieStoreImpl>,
}

impl<CookieStoreImpl: CookieStore + 'static> Default for Impit<CookieStoreImpl> {
    fn default() -> Self {
        ImpitBuilder::<CookieStoreImpl>::default().build()
    }
}

/// Customizes the behavior of the [`Impit`] struct when following redirects.
///
/// The `RedirectBehavior` enum is used to specify how the client should handle redirects.
#[derive(Debug, Clone)]
pub enum RedirectBehavior {
    /// Follow up to `usize` redirects.
    ///
    /// If the number of redirects is exceeded, the client will return an error.
    FollowRedirect(usize),
    /// Don't follow any redirects.
    ///
    /// The client will return the response for the first request, even with the `3xx` status code.
    ManualRedirect,
}

/// A builder struct used to create a new [`Impit`] instance.
///
/// The builder allows setting the browser to impersonate, ignoring TLS errors, setting a proxy, and other options.
///
/// ### Example
/// ```rust
/// let mut impit = Impit::builder()
///   .with_browser(Browser::Firefox)
///   .with_ignore_tls_errors(true)
///   .with_proxy("http://localhost:8080".to_string())
///   .with_default_timeout(Duration::from_secs(10))
///   .with_http3()
///   .build();
///
/// let response = impit.get("https://example.com".to_string(), None).await;
/// ```
#[derive(Debug)]
pub struct ImpitBuilder<CookieStoreImpl: CookieStore + 'static> {
    browser: Option<Browser>,
    ignore_tls_errors: bool,
    vanilla_fallback: bool,
    proxy_url: String,
    request_timeout: Duration,
    max_http_version: Version,
    redirect: RedirectBehavior,
    cookie_store: Option<Arc<CookieStoreImpl>>,
    headers: Option<Vec<(String, String)>>,
}

impl<CookieStoreImpl: CookieStore + 'static> Clone for ImpitBuilder<CookieStoreImpl> {
    fn clone(&self) -> Self {
        ImpitBuilder {
            browser: self.browser,
            ignore_tls_errors: self.ignore_tls_errors,
            vanilla_fallback: self.vanilla_fallback,
            proxy_url: self.proxy_url.clone(),
            request_timeout: self.request_timeout,
            max_http_version: self.max_http_version,
            redirect: self.redirect.clone(),
            cookie_store: self.cookie_store.clone(),
            headers: self.headers.clone(),
        }
    }
}

impl<CookieStoreImpl: CookieStore + 'static> Default for ImpitBuilder<CookieStoreImpl> {
    fn default() -> Self {
        ImpitBuilder {
            browser: None,
            ignore_tls_errors: false,
            vanilla_fallback: true,
            proxy_url: String::from_str("").unwrap(),
            request_timeout: Duration::from_secs(30),
            max_http_version: Version::HTTP_2,
            redirect: RedirectBehavior::FollowRedirect(10),
            cookie_store: None,
            headers: None,
        }
    }
}

impl<CookieStoreImpl: CookieStore + 'static> ImpitBuilder<CookieStoreImpl> {
    /// Sets the browser to impersonate.
    ///
    /// The [`Browser`] enum is used to set the HTTP headers, TLS behaviour and other markers to impersonate a specific browser.
    ///
    /// If not used, the client will use the default `reqwest` fingerprints.
    pub fn with_browser(mut self, browser: Browser) -> Self {
        self.browser = Some(browser);
        self
    }

    /// If set to true, the client will ignore TLS-related errors.
    pub fn with_ignore_tls_errors(mut self, ignore_tls_errors: bool) -> Self {
        self.ignore_tls_errors = ignore_tls_errors;
        self
    }

    /// If set to `true`, the client will retry the request without impersonation
    /// if the impersonated browser encounters an error.
    pub fn with_fallback_to_vanilla(mut self, vanilla_fallback: bool) -> Self {
        self.vanilla_fallback = vanilla_fallback;
        self
    }

    /// Sets the proxy URL to use for requests.
    ///
    /// Note that this proxy will be used for all the requests
    /// made by the built [`Impit`] instance.
    pub fn with_proxy(mut self, proxy_url: String) -> Self {
        self.proxy_url = proxy_url;
        self
    }

    /// Sets the default timeout for requests.
    ///
    /// This setting can be overridden when making the request by using the `RequestOptions` struct.
    pub fn with_default_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// Enables HTTP/3 usage for requests.
    ///
    /// `impit` currently supports HTTP/3 negotiation via the HTTPS DNS record and the `Alt-Svc` header.
    /// To enforce HTTP/3 usage, use the `http3_prior_knowledge` option in the `RequestOptions` struct when
    /// making the request.
    ///
    /// Note that this feature is experimental and may not work as expected with all servers.
    pub fn with_http3(mut self) -> Self {
        self.max_http_version = Version::HTTP_3;
        self
    }

    /// Sets the desired redirect behavior.
    ///
    /// By default, the client will follow up to 10 redirects.
    /// By passing the `RedirectBehavior::ManualRedirect` option, the client will not follow any redirects
    /// (i.e. it will return the response for the first request, with the 3xx status code).
    pub fn with_redirect(mut self, behavior: RedirectBehavior) -> Self {
        self.redirect = behavior;
        self
    }

    /// Sets whether to store cookies in the internal Client cookie store.
    ///
    /// If set to `true`, the client will store cookies in the internal cookie store.
    /// If set to `false`, the client will not store cookies. Response headers will contain the
    /// `Set-Cookie` header.
    pub fn with_cookie_store(mut self, cookie_store: CookieStoreImpl) -> Self {
        self.cookie_store = Some(Arc::new(cookie_store));
        self
    }

    /// Sets additional headers to include in every request made by the built [`Impit`] instance.
    ///
    /// This can be used to add e.g. custom user-agent or authorization headers that should be included in every request.
    /// These headers override the "impersonation" headers set by the `with_browser` method.
    ///
    /// If you want to add custom headers to a specific request, use the `RequestOptions` struct instead.
    pub fn with_headers(mut self, headers: Vec<(String, String)>) -> Self {
        self.headers = Some(headers);
        self
    }

    /// Builds the [`Impit`] instance.
    pub fn build(self) -> Impit<CookieStoreImpl> {
        Impit::new(self)
    }
}

impl<CookieStoreImpl: CookieStore + 'static> Impit<CookieStoreImpl> {
    pub fn builder() -> ImpitBuilder<CookieStoreImpl> {
        ImpitBuilder::default()
    }

    fn new_reqwest_client(
        config: &ImpitBuilder<CookieStoreImpl>,
    ) -> Result<reqwest::Client, reqwest::Error> {
        let mut client = reqwest::Client::builder();
        let mut tls_config_builder = tls::TlsConfig::builder();
        let mut tls_config_builder = tls_config_builder.with_browser(config.browser);

        if config.max_http_version == Version::HTTP_3 {
            tls_config_builder = tls_config_builder.with_http3();
        }

        tls_config_builder = tls_config_builder.with_ignore_tls_errors(config.ignore_tls_errors);

        let tls_config = tls_config_builder.build();

        client = client
            .danger_accept_invalid_certs(config.ignore_tls_errors)
            .danger_accept_invalid_hostnames(config.ignore_tls_errors)
            .use_preconfigured_tls(tls_config)
            .timeout(config.request_timeout);

        if let Some(cookie_provider) = &config.cookie_store {
            client = client.cookie_provider(cookie_provider.clone());
        }

        if config.max_http_version == Version::HTTP_3 {
            client = client.http3_prior_knowledge();
        }

        if !config.proxy_url.is_empty() {
            client = client.proxy(reqwest::Proxy::all(&config.proxy_url)?);
        }

        match config.redirect {
            RedirectBehavior::FollowRedirect(max) => {
                client = client.redirect(reqwest::redirect::Policy::limited(max));
            }
            RedirectBehavior::ManualRedirect => {
                client = client.redirect(reqwest::redirect::Policy::none());
            }
        }

        client.build()
    }

    /// Creates a new [`Impit`] instance based on the options stored in the [`ImpitBuilder`] instance.
    fn new(config: ImpitBuilder<CookieStoreImpl>) -> Self {
        let mut h3_client: Option<reqwest::Client> = None;
        let mut base_client = Self::new_reqwest_client(&config).unwrap();

        if config.max_http_version == Version::HTTP_3 {
            h3_client = Some(base_client);
            base_client = Self::new_reqwest_client(&ImpitBuilder::<CookieStoreImpl> {
                max_http_version: Version::HTTP_2,
                ..config.clone()
            })
            .unwrap();
        }

        Impit {
            base_client,
            h3_client,
            config,
            h3_engine: Arc::new(RwLock::new(None)),
        }
    }

    fn parse_url(&self, url: String) -> Result<Url, ImpitError> {
        let url = Url::parse(&url).map_err(|_| ImpitError::UrlParsingError)?;

        if url.host_str().is_none() {
            return Err(ImpitError::UrlMissingHostnameError(url.to_string()));
        }

        let protocol = url.scheme();

        match protocol {
            "http" => Ok(url),
            "https" => Ok(url),
            _ => Err(ImpitError::UrlProtocolError(protocol.to_string())),
        }
    }

    async fn should_use_h3(&self, host: &String) -> bool {
        if self.config.max_http_version < Version::HTTP_3 {
            debug!("HTTP/3 is disabled, falling back to TCP-based requests.");
            return false;
        }

        {
            let engine_guard = self.h3_engine.read().await;
            if let Some(engine) = engine_guard.as_ref() {
                return engine.host_supports_h3(host).await;
            }
        }

        {
            let mut engine_guard = self.h3_engine.write().await;
            if engine_guard.is_none() {
                *engine_guard = Some(H3Engine::init().await);
            }
            engine_guard.as_ref().unwrap().host_supports_h3(host).await
        }
    }

    async fn make_request(
        &self,
        method: Method,
        url: String,
        body: Option<Vec<u8>>,
        options: Option<RequestOptions>,
    ) -> Result<Response, ImpitError> {
        let options = options.unwrap_or_default();

        if options.http3_prior_knowledge && self.config.max_http_version < Version::HTTP_3 {
            return Err(ImpitError::Http3Disabled);
        }

        let parsed_url = self.parse_url(url.clone())?;
        let host = parsed_url.host_str().unwrap().to_string();

        let h3 = options.http3_prior_knowledge || self.should_use_h3(&host).await;

        let headers = HttpHeaders::get_builder()
            .with_browser(&self.config.browser)
            .with_host(&host)
            .with_https(parsed_url.scheme() == "https")
            .with_custom_headers(self.config.headers.to_owned())
            .with_custom_headers(Some(options.headers))
            .build();

        let client = if h3 {
            debug!("Using QUIC for request to {url}");
            self.h3_client.as_ref().unwrap()
        } else {
            debug!("{url} doesn't seem to have HTTP3 support");
            &self.base_client
        };

        let header_map: Result<HeaderMap, ImpitError> = headers.into();

        let mut request = client
            .request(method.clone(), parsed_url.clone())
            .headers(header_map?);

        if h3 {
            request = request.version(Version::HTTP_3);
        }

        if let Some(timeout) = options.timeout {
            request = request.timeout(timeout);
        }

        request = match body {
            Some(body) => request.body(body),
            None => request,
        };

        let response = request.send().await;

        if response.is_err() {
            let max_redirects = match self.config.redirect {
                RedirectBehavior::FollowRedirect(max) => max,
                RedirectBehavior::ManualRedirect => 0,
            };

            return Err(ImpitError::from(
                response.err().unwrap(),
                ErrorContext {
                    timeout: options.timeout.unwrap_or(self.config.request_timeout),
                    max_redirects,
                    method: method.to_string(),
                    protocol: parsed_url.scheme().to_string(),
                    url: url.clone(),
                },
            ));
        }

        let response = response.unwrap();

        if !h3 {
            let engine_guard = self.h3_engine.read().await;
            if let Some(h3_engine) = engine_guard.as_ref() {
                h3_engine.set_h3_support(&host, false).await;

                if let Some(alt_svc) = response.headers().get("Alt-Svc") {
                    let alt_svc = alt_svc.to_str().unwrap();
                    if alt_svc.contains("h3") {
                        debug!("{host} supports HTTP/3 (alt-svc header), adding to Alt-Svc cache");
                        h3_engine.set_h3_support(&host, true).await;
                    }
                }
            }
        }

        Ok(response)
    }

    /// Makes a `GET` request to the specified URL.
    ///
    /// The `url` parameter should be a valid URL.
    /// Additional options like `headers`, `timeout` or HTTP/3 usage can be passed via the `RequestOptions` struct.
    ///
    /// If the request is successful, the `reqwest::Response` struct is returned.
    pub async fn get(
        &self,
        url: String,
        options: Option<RequestOptions>,
    ) -> Result<Response, ImpitError> {
        self.make_request(Method::GET, url, None, options).await
    }

    /// Makes a `HEAD` request to the specified URL.
    ///
    /// The `url` parameter should be a valid URL.
    /// Additional options like `headers`, `timeout` or HTTP/3 usage can be passed via the `RequestOptions` struct.
    ///
    /// If the request is successful, the `reqwest::Response` struct is returned.
    pub async fn head(
        &self,
        url: String,
        options: Option<RequestOptions>,
    ) -> Result<Response, ImpitError> {
        self.make_request(Method::HEAD, url, None, options).await
    }

    /// Makes an OPTIONS request to the specified URL.
    ///
    /// The `url` parameter should be a valid URL.
    /// Additional options like `headers`, `timeout` or HTTP/3 usage can be passed via the `RequestOptions` struct.
    ///
    /// If the request is successful, the `reqwest::Response` struct is returned.
    pub async fn options(
        &self,
        url: String,
        options: Option<RequestOptions>,
    ) -> Result<Response, ImpitError> {
        self.make_request(Method::OPTIONS, url, None, options).await
    }

    /// Makes a `TRACE` request to the specified URL.
    ///
    /// The `url` parameter should be a valid URL.
    /// Additional options like `headers`, `timeout` or HTTP/3 usage can be passed via the `RequestOptions` struct.
    ///
    /// If the request is successful, the `reqwest::Response` struct is returned.
    pub async fn trace(
        &self,
        url: String,
        options: Option<RequestOptions>,
    ) -> Result<Response, ImpitError> {
        self.make_request(Method::TRACE, url, None, options).await
    }

    /// Makes a `DELETE` request to the specified URL.
    ///
    /// The `url` parameter should be a valid URL.
    /// Additional options like `headers`, `timeout` or HTTP/3 usage can be passed via the `RequestOptions` struct.
    ///
    /// If the request is successful, the `reqwest::Response` struct is returned.
    pub async fn delete(
        &self,
        url: String,
        options: Option<RequestOptions>,
    ) -> Result<Response, ImpitError> {
        self.make_request(Method::DELETE, url, None, options).await
    }

    /// Makes a `POST` request to the specified URL.
    ///
    /// The `url` parameter should be a valid URL.
    /// Additional options like `headers`, `timeout` or HTTP/3 usage can be passed via the `RequestOptions` struct.
    ///
    /// If the request is successful, the `reqwest::Response` struct is returned.
    pub async fn post(
        &self,
        url: String,
        body: Option<Vec<u8>>,
        options: Option<RequestOptions>,
    ) -> Result<Response, ImpitError> {
        self.make_request(Method::POST, url, body, options).await
    }

    /// Makes a `PUT` request to the specified URL.
    ///
    /// The `url` parameter should be a valid URL.
    /// Additional options like `headers`, `timeout` or HTTP/3 usage can be passed via the `RequestOptions` struct.
    ///
    /// If the request is successful, the `reqwest::Response` struct is returned.
    pub async fn put(
        &self,
        url: String,
        body: Option<Vec<u8>>,
        options: Option<RequestOptions>,
    ) -> Result<Response, ImpitError> {
        self.make_request(Method::PUT, url, body, options).await
    }

    /// Makes a `PATCH` request to the specified URL.
    ///
    /// The `url` parameter should be a valid URL.
    /// Additional options like `headers`, `timeout` or HTTP/3 usage can be passed via the `RequestOptions` struct.
    ///
    /// If the request is successful, the `reqwest::Response` struct is returned.
    pub async fn patch(
        &self,
        url: String,
        body: Option<Vec<u8>>,
        options: Option<RequestOptions>,
    ) -> Result<Response, ImpitError> {
        self.make_request(Method::PATCH, url, body, options).await
    }
}
