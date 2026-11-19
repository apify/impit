#[cfg(feature = "http3")]
use tokio::sync::RwLock;

use log::{debug, warn};
use reqwest::{
    cookie::CookieStore,
    header::{
        HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_TYPE,
        COOKIE, LOCATION, PROXY_AUTHORIZATION, REFERER, TRANSFER_ENCODING, WWW_AUTHENTICATE,
    },
    Method, Response, StatusCode, Version,
};
use std::{
    fmt::Debug,
    net::IpAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use url::Url;

#[cfg(feature = "http3")]
use crate::http3::H3Engine;
use crate::{
    errors::{ErrorContext, ImpitError},
    fingerprint::BrowserFingerprint,
    http_headers::HttpHeadersBuilder,
    request::{ImpitBody, ImpitRequest, RequestOptions},
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
    pub(self) vanilla_client: Option<reqwest::Client>,
    #[cfg(feature = "http3")]
    h3_engine: Arc<RwLock<Option<H3Engine>>>,
    config: ImpitBuilder<CookieStoreImpl>,
}

struct PreparedRequest {
    method: Method,
    url: Url,
    headers: HeaderMap,
    body: ImpitBody,
}

impl<CookieStoreImpl: CookieStore + 'static> Default for Impit<CookieStoreImpl> {
    fn default() -> Self {
        ImpitBuilder::<CookieStoreImpl>::default().build().unwrap()
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
/// ```rust,no_run
/// use impit::{impit::Impit, fingerprint::database as fingerprints};
/// use reqwest::cookie::Jar;
/// use std::time::Duration;
///
/// # #[tokio::main]
/// # async fn main() {
/// let impit = Impit::<Jar>::builder()
///   .with_fingerprint(fingerprints::firefox_144::fingerprint())
///   .with_ignore_tls_errors(true)
///   .with_proxy("http://localhost:8080".to_string())
///   .with_default_timeout(Duration::from_secs(10))
///   .build()
///   .unwrap();
///
/// let response = impit.get("https://example.com".to_string(), None, None).await;
/// # }
/// ```
#[derive(Debug)]
pub struct ImpitBuilder<CookieStoreImpl: CookieStore + 'static> {
    fingerprint: Option<BrowserFingerprint>,
    ignore_tls_errors: bool,
    vanilla_fallback: bool,
    proxy_url: String,
    request_timeout: Duration,
    max_http_version: Version,
    redirect: RedirectBehavior,
    cookie_store: Option<Arc<CookieStoreImpl>>,
    headers: Option<Vec<(String, String)>>,
    local_address: Option<IpAddr>,
}

impl<CookieStoreImpl: CookieStore + 'static> Clone for ImpitBuilder<CookieStoreImpl> {
    fn clone(&self) -> Self {
        ImpitBuilder {
            fingerprint: self.fingerprint.clone(),
            ignore_tls_errors: self.ignore_tls_errors,
            vanilla_fallback: self.vanilla_fallback,
            proxy_url: self.proxy_url.clone(),
            request_timeout: self.request_timeout,
            max_http_version: self.max_http_version,
            redirect: self.redirect.clone(),
            cookie_store: self.cookie_store.clone(),
            headers: self.headers.clone(),
            local_address: self.local_address,
        }
    }
}

impl<CookieStoreImpl: CookieStore + 'static> Default for ImpitBuilder<CookieStoreImpl> {
    fn default() -> Self {
        ImpitBuilder {
            fingerprint: None,
            ignore_tls_errors: false,
            vanilla_fallback: false,
            proxy_url: String::new(),
            request_timeout: Duration::from_secs(30),
            max_http_version: Version::HTTP_2,
            redirect: RedirectBehavior::FollowRedirect(10),
            cookie_store: None,
            headers: None,
            local_address: None,
        }
    }
}

impl<CookieStoreImpl: CookieStore + 'static> ImpitBuilder<CookieStoreImpl> {
    /// Sets a complete browser fingerprint.
    ///
    /// This method allows you to provide a complete fingerprint that includes TLS, HTTP/2, and HTTP header configurations.
    /// When set, this takes precedence over the `with_browser` method.
    ///
    /// You can use pre-defined fingerprints from [`crate::fingerprint::database`] or create custom fingerprints.
    pub fn with_fingerprint(mut self, fingerprint: BrowserFingerprint) -> Self {
        self.fingerprint = Some(fingerprint);
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
    /// Requires the `http3` feature of this crate (and the `--cfg reqwest_unstable` rustflag).
    ///
    /// HTTP/3 is never used with a proxy, set either with [`with_proxy`](Self::with_proxy) or in the
    /// `HTTPS_PROXY` or `ALL_PROXY` environment variables: proxies only carry TCP, so QUIC requests would
    /// skip the proxy and go out from this machine's IP address. With a proxy, the client uses HTTP/2.
    ///
    /// Note that this feature is experimental and may not work as expected with all servers.
    #[cfg(feature = "http3")]
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

    /// Sets the local address to bind the client to.
    ///
    /// This is useful for testing purposes or when you want to bind the client to a specific network interface.
    /// Note that this address should be a valid IP address in the format "xxx.xxx.xxx.xxx" (for IPv4) or
    /// "ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff" (for IPv6).
    pub fn with_local_address(mut self, local_address: String) -> Result<Self, ImpitError> {
        let ip_addr = local_address.parse::<IpAddr>().map_err(|_| {
            ImpitError::ReqwestError(format!("Invalid local address: {local_address}"))
        })?;

        self.local_address = Some(ip_addr);
        Ok(self)
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

    /// Whether requests go through a proxy: the one set with [`with_proxy`](Self::with_proxy), or one
    /// that `reqwest` reads from the environment for HTTPS URLs.
    fn uses_proxy(&self) -> bool {
        !self.proxy_url.is_empty()
            || ["HTTPS_PROXY", "https_proxy", "ALL_PROXY", "all_proxy"]
                .iter()
                .any(|name| std::env::var_os(name).is_some_and(|value| !value.is_empty()))
    }

    /// Builds the [`Impit`] instance.
    pub fn build(self) -> Result<Impit<CookieStoreImpl>, ImpitError> {
        Impit::new(self)
    }
}

impl<CookieStoreImpl: CookieStore + 'static> Impit<CookieStoreImpl> {
    pub fn builder() -> ImpitBuilder<CookieStoreImpl> {
        ImpitBuilder::default()
    }

    fn new_reqwest_client(
        config: &ImpitBuilder<CookieStoreImpl>,
    ) -> Result<reqwest::Client, ImpitError> {
        let mut client = reqwest::Client::builder();
        let mut tls_config_builder = tls::TlsConfig::builder();

        // Use fingerprint if provided, otherwise fall back to browser enum
        if let Some(ref fingerprint) = config.fingerprint {
            tls_config_builder.with_tls_fingerprint(fingerprint.tls.clone());

            if let Some(window_size) = fingerprint.http2.initial_stream_window_size {
                client = client.http2_initial_stream_window_size(window_size);
            }

            if let Some(window_size) = fingerprint.http2.initial_connection_window_size {
                client = client.http2_initial_connection_window_size(window_size);
            }

            if let Some(max_size) = fingerprint.http2.max_header_list_size {
                client = client.http2_max_header_list_size(max_size);
            }

            if let Some(size) = fingerprint.http2.header_table_size {
                client = client.http2_header_table_size(size);
            }

            if let Some(max) = fingerprint.http2.max_concurrent_streams {
                client = client.http2_max_concurrent_streams(max);
            }

            if let Some(size) = fingerprint.http2.max_frame_size {
                client = client.http2_max_frame_size(size);
            }
        }

        if config.max_http_version == Version::HTTP_3 {
            tls_config_builder.with_http3();
        }

        tls_config_builder.with_ignore_tls_errors(config.ignore_tls_errors);

        let tls_config = tls_config_builder.build();

        client = client
            .danger_accept_invalid_certs(config.ignore_tls_errors)
            .danger_accept_invalid_hostnames(config.ignore_tls_errors)
            .use_preconfigured_tls(tls_config)
            .timeout(config.request_timeout);

        if let Some(cookie_provider) = &config.cookie_store {
            client = client.cookie_provider(cookie_provider.clone());
        }

        #[cfg(feature = "http3")]
        if config.max_http_version == Version::HTTP_3 {
            client = client.http3_prior_knowledge();
        }

        if !config.proxy_url.is_empty() {
            client = client.proxy(
                reqwest::Proxy::all(&config.proxy_url)
                    .map_err(|_| ImpitError::ProxyError(config.proxy_url.clone()))?,
            );
        }

        if let Some(ip_addr) = config.local_address {
            client = client.local_address(ip_addr);
        }

        // Redirects are followed by `Impit::send`, so they can be configured per request.
        client = client.redirect(reqwest::redirect::Policy::none());

        client
            .build()
            .map_err(|e| ImpitError::ReqwestError(format!("{e:#?}")))
    }

    /// Creates a new [`Impit`] instance based on the options stored in the [`ImpitBuilder`] instance.
    fn new(mut config: ImpitBuilder<CookieStoreImpl>) -> Result<Self, ImpitError> {
        if config.max_http_version == Version::HTTP_3 && config.uses_proxy() {
            warn!("HTTP/3 is disabled because a proxy is set: QUIC requests would skip the proxy");
            config.max_http_version = Version::HTTP_2;
        }

        let mut h3_client: Option<reqwest::Client> = None;
        let mut base_client = Self::new_reqwest_client(&config)?;

        if config.max_http_version == Version::HTTP_3 {
            h3_client = Some(base_client);
            base_client = Self::new_reqwest_client(&ImpitBuilder::<CookieStoreImpl> {
                max_http_version: Version::HTTP_2,
                ..config.clone()
            })?;
        }

        let vanilla_client = if config.vanilla_fallback && config.fingerprint.is_some() {
            Some(Self::new_reqwest_client(
                &ImpitBuilder::<CookieStoreImpl> {
                    fingerprint: None,
                    max_http_version: Version::HTTP_2,
                    ..config.clone()
                },
            )?)
        } else {
            None
        };

        // Set pseudo-header order from fingerprint or fall back to browser enum
        let pseudo_headers_order: Vec<String> = if let Some(ref fingerprint) = config.fingerprint {
            fingerprint.http2.pseudo_header_order.to_vec()
        } else {
            vec![]
        };

        if !pseudo_headers_order.is_empty() {
            std::env::set_var(
                "IMPIT_H2_PSEUDOHEADERS_ORDER",
                pseudo_headers_order.join(","),
            );
        }

        // Always set, even to "", so a profile doesn't inherit the previous one's values.
        if let Some(ref fingerprint) = config.fingerprint {
            for (name, value) in fingerprint.http2.h2_settings_env() {
                std::env::set_var(name, value);
            }
        }

        Ok(Impit {
            base_client,
            h3_client,
            vanilla_client,
            config,
            #[cfg(feature = "http3")]
            h3_engine: Arc::new(RwLock::new(None)),
        })
    }

    fn parse_url(&self, url: String) -> Result<Url, ImpitError> {
        let url = Url::parse(&url).map_err(|_| ImpitError::UrlParsingError(url.clone()))?;

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

    #[cfg(not(feature = "http3"))]
    async fn should_use_h3(&self, _host: &str) -> bool {
        false
    }

    #[cfg(feature = "http3")]
    async fn should_use_h3(&self, host: &str) -> bool {
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

            match engine_guard.as_ref() {
                None => false,
                Some(engine) => engine.host_supports_h3(host).await,
            }
        }
    }

    fn build_request(
        &self,
        method: Method,
        url: Url,
        body: Option<ImpitBody>,
        headers: &[(String, String)],
    ) -> Result<ImpitRequest, ImpitError> {
        let headers = HttpHeadersBuilder::default()
            .with_fingerprint(self.config.fingerprint.as_ref())
            .with_custom_headers(self.config.headers.as_deref().unwrap_or_default())
            .with_custom_headers(headers)
            .build()?;

        Ok(ImpitRequest {
            url,
            body: body.unwrap_or_default(),
            headers,
            method,
        })
    }

    async fn execute_request(
        &self,
        client: &reqwest::Client,
        prepared: &mut PreparedRequest,
        timeout: Option<Duration>,
        h3: bool,
    ) -> Result<Response, reqwest::Error> {
        let mut req = client
            .request(prepared.method.clone(), prepared.url.clone())
            .headers(prepared.headers.clone());

        if h3 {
            req = req.version(Version::HTTP_3);
        }

        if let Some(t) = timeout {
            req = req.timeout(t);
        }

        if let Some(body) = prepared.body.take() {
            req = req.body(body);
        }

        req.send().await
    }

    /// Sends a single request of the redirect chain. `remaining` is the time left from `timeout`.
    async fn send_once(
        &self,
        prepared: &mut PreparedRequest,
        timeout: Duration,
        remaining: Duration,
        http3_prior_knowledge: bool,
    ) -> Result<Response, ImpitError> {
        if http3_prior_knowledge && self.config.max_http_version < Version::HTTP_3 {
            return Err(ImpitError::Http3Disabled);
        }

        let h3 = http3_prior_knowledge
            || self
                .should_use_h3(prepared.url.host_str().unwrap_or_default())
                .await;
        let client = if h3 {
            debug!("Using QUIC for request to {}", prepared.url);
            self.h3_client.as_ref().unwrap_or(&self.base_client)
        } else {
            debug!("{} doesn't seem to have HTTP3 support", prepared.url);
            &self.base_client
        };

        let primary_result = self
            .execute_request(client, prepared, Some(remaining), h3)
            .await;

        let response = match primary_result {
            Ok(resp) => resp,
            Err(err) => {
                let primary_error = ImpitError::from(
                    err,
                    Some(ErrorContext {
                        timeout: Some(timeout),
                        method: Some(prepared.method.to_string()),
                        protocol: Some(prepared.url.scheme().to_string()),
                        url: Some(prepared.url.to_string()),
                    }),
                );

                let fallback_client = self
                    .vanilla_client
                    .as_ref()
                    .filter(|_| primary_error.is_connect_error() && prepared.body.is_sendable());
                let Some(vanilla_client) = fallback_client else {
                    return Err(primary_error);
                };

                debug!(
                    "Primary request to {} failed with {primary_error}, retrying with vanilla client",
                    prepared.url
                );
                match self
                    .execute_request(vanilla_client, prepared, Some(remaining), false)
                    .await
                {
                    Ok(resp) => resp,
                    Err(_) => return Err(primary_error),
                }
            }
        };

        #[cfg(feature = "http3")]
        if !h3 {
            let engine_guard = self.h3_engine.read().await;
            if let Some(h3_engine) = engine_guard.as_ref() {
                let host = prepared.url.host_str().unwrap_or_default();
                h3_engine.set_h3_support(host, false).await;

                if let Some(alt_svc) = response.headers().get("Alt-Svc") {
                    if let Ok(alt_svc_str) = alt_svc.to_str() {
                        if alt_svc_str.contains("h3") {
                            debug!(
                                "{host} supports HTTP/3 (alt-svc header), adding to Alt-Svc cache"
                            );
                            h3_engine.set_h3_support(host, true).await;
                        }
                    }
                }
            }
        }

        Ok(response)
    }

    async fn send(
        &self,
        request: ImpitRequest,
        timeout: Option<Duration>,
        http3_prior_knowledge: bool,
        redirect: &RedirectBehavior,
    ) -> Result<Response, ImpitError> {
        let timeout = timeout.unwrap_or(self.config.request_timeout);
        let start = Instant::now();
        let mut redirects = 0;

        let mut prepared = PreparedRequest {
            method: request.method,
            url: request.url,
            headers: request.headers,
            body: request.body,
        };

        loop {
            // The timeout bounds the whole redirect chain, not each request.
            let remaining = timeout.saturating_sub(start.elapsed());
            let response = self
                .send_once(&mut prepared, timeout, remaining, http3_prior_knowledge)
                .await?;

            let RedirectBehavior::FollowRedirect(max_redirects) = *redirect else {
                return Ok(response);
            };
            if !follow_redirect(&mut prepared, &response)? {
                return Ok(response);
            }
            if redirects == max_redirects {
                return Err(ImpitError::TooManyRedirects(Some(max_redirects)));
            }
            redirects += 1;
        }
    }

    async fn make_request(
        &self,
        method: Method,
        url: String,
        body: Option<ImpitBody>,
        options: Option<RequestOptions>,
    ) -> Result<Response, ImpitError> {
        let url = self.parse_url(url)?;
        let request_options = options.unwrap_or_default();

        let request = self.build_request(method, url, body, &request_options.headers)?;

        let timeout = match request_options.timeout {
            None => None,
            // reqwest has no per-request "no timeout" API; overriding with Duration::MAX is the
            // conventional way to disable a timeout without rebuilding the client.
            Some(None) => Some(Duration::MAX),
            Some(Some(d)) => Some(d),
        };
        self.send(
            request,
            timeout,
            request_options.http3_prior_knowledge,
            request_options
                .redirect
                .as_ref()
                .unwrap_or(&self.config.redirect),
        )
        .await
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
        body: Option<ImpitBody>,
        options: Option<RequestOptions>,
    ) -> Result<Response, ImpitError> {
        self.make_request(Method::GET, url, body, options).await
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
        body: Option<ImpitBody>,
        options: Option<RequestOptions>,
    ) -> Result<Response, ImpitError> {
        self.make_request(Method::HEAD, url, body, options).await
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
        body: Option<ImpitBody>,
        options: Option<RequestOptions>,
    ) -> Result<Response, ImpitError> {
        self.make_request(Method::OPTIONS, url, body, options).await
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
        body: Option<ImpitBody>,
        options: Option<RequestOptions>,
    ) -> Result<Response, ImpitError> {
        self.make_request(Method::TRACE, url, body, options).await
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
        body: Option<ImpitBody>,
        options: Option<RequestOptions>,
    ) -> Result<Response, ImpitError> {
        self.make_request(Method::DELETE, url, body, options).await
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
        body: Option<ImpitBody>,
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
        body: Option<ImpitBody>,
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
        body: Option<ImpitBody>,
        options: Option<RequestOptions>,
    ) -> Result<Response, ImpitError> {
        self.make_request(Method::PATCH, url, body, options).await
    }

    pub fn generate_multipart_boundary(&self) -> String {
        match &self.config.fingerprint {
            Some(fp) => fp.generate_multipart_boundary(),
            None => crate::fingerprint::default_multipart_boundary(),
        }
    }
}

/// Turns `request` into the request following the redirect in `response`, the same way `reqwest` does.
///
/// Returns `false` and leaves `request` as is if `response` isn't a redirect that can be followed.
fn follow_redirect(request: &mut PreparedRequest, response: &Response) -> Result<bool, ImpitError> {
    let drop_body = match response.status() {
        StatusCode::MOVED_PERMANENTLY | StatusCode::FOUND => request.method == Method::POST,
        StatusCode::SEE_OTHER => true,
        StatusCode::TEMPORARY_REDIRECT | StatusCode::PERMANENT_REDIRECT => false,
        _ => return Ok(false),
    };

    if !drop_body && !request.body.is_sendable() {
        return Ok(false);
    }

    let Some(url) = response
        .headers()
        .get(LOCATION)
        .and_then(|location| std::str::from_utf8(location.as_bytes()).ok())
        .and_then(|location| request.url.join(location).ok())
    else {
        return Ok(false);
    };

    if !matches!(url.scheme(), "http" | "https") {
        return Err(ImpitError::UrlProtocolError(url.scheme().to_string()));
    }

    if drop_body {
        if request.method != Method::HEAD {
            request.method = Method::GET;
        }
        request.body = ImpitBody::Empty;
        for header in [
            CONTENT_TYPE,
            CONTENT_LENGTH,
            CONTENT_ENCODING,
            TRANSFER_ENCODING,
        ] {
            request.headers.remove(header);
        }
    }

    let previous = &request.url;
    if url.host_str() != previous.host_str()
        || url.port_or_known_default() != previous.port_or_known_default()
        || url.scheme() != previous.scheme()
    {
        for header in [AUTHORIZATION, COOKIE, PROXY_AUTHORIZATION, WWW_AUTHENTICATE] {
            request.headers.remove(header);
        }
        request.headers.remove("cookie2");
    }

    if !(url.scheme() == "http" && previous.scheme() == "https") {
        let mut referer = previous.clone();
        let _ = referer.set_username("");
        let _ = referer.set_password(None);
        referer.set_fragment(None);
        if let Ok(referer) = HeaderValue::from_str(referer.as_str()) {
            request.headers.insert(REFERER, referer);
        }
    }

    request.url = url;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use reqwest::{cookie::Jar, header::HeaderMap, Method, Response};

    use super::{follow_redirect, Impit, PreparedRequest};
    use crate::{
        errors::ImpitError,
        request::{ImpitBody, RequestOptions},
    };

    fn request(method: Method, url: &str, headers: &[(&str, &str)]) -> PreparedRequest {
        let headers: HeaderMap = headers
            .iter()
            .map(|(name, value)| (name.parse().unwrap(), value.parse().unwrap()))
            .collect();
        PreparedRequest {
            url: url.parse().unwrap(),
            body: ImpitBody::from("body"),
            headers,
            method,
        }
    }

    fn redirect(status: u16, location: &str) -> Response {
        hyper::Response::builder()
            .status(status)
            .header("location", location)
            .body("")
            .unwrap()
            .into()
    }

    #[test]
    fn found_redirects_post_as_get() {
        let mut request = request(
            Method::POST,
            "https://example.com/form",
            &[("content-type", "text/plain")],
        );

        assert!(follow_redirect(&mut request, &redirect(302, "/done")).unwrap());
        assert_eq!(request.method, Method::GET);
        assert_eq!(request.url.as_str(), "https://example.com/done");
        assert!(request.body.take().is_none());
        assert!(request.headers.get("content-type").is_none());
        assert_eq!(request.headers["referer"], "https://example.com/form");
    }

    #[test]
    fn cross_origin_redirects_drop_credentials() {
        let mut request = request(
            Method::PUT,
            "https://example.com/",
            &[
                ("authorization", "secret"),
                ("cookie", "a=b"),
                ("x-custom", "1"),
            ],
        );

        assert!(follow_redirect(&mut request, &redirect(307, "https://example.org/")).unwrap());
        assert_eq!(request.method, Method::PUT);
        assert!(request.body.take().is_some());
        assert!(request.headers.get("authorization").is_none());
        assert!(request.headers.get("cookie").is_none());
        assert_eq!(request.headers["x-custom"], "1");
    }

    #[test]
    fn sent_streamed_bodies_are_not_replayed() {
        let mut request = request(Method::POST, "https://example.com/", &[]);
        request.body = ImpitBody::Consumed;

        assert!(!follow_redirect(&mut request, &redirect(307, "/next")).unwrap());
        assert_eq!(request.url.as_str(), "https://example.com/");
        assert!(follow_redirect(&mut request, &redirect(303, "/next")).unwrap());
    }

    async fn get_with_http3_prior_knowledge(impit: &Impit<Jar>) -> Result<(), ImpitError> {
        let options = RequestOptions {
            http3_prior_knowledge: true,
            ..Default::default()
        };
        impit
            .get("https://example.com".to_string(), None, Some(options))
            .await
            .map(|_| ())
    }

    #[tokio::test]
    async fn http3_is_off_by_default() {
        let impit = Impit::<Jar>::builder().build().unwrap();
        assert!(impit.h3_client.is_none());
        assert!(matches!(
            get_with_http3_prior_knowledge(&impit).await,
            Err(ImpitError::Http3Disabled)
        ));
    }

    #[cfg(feature = "http3")]
    #[tokio::test]
    async fn http3_is_disabled_with_a_proxy() {
        let impit = Impit::<Jar>::builder()
            .with_http3()
            .with_proxy("http://127.0.0.1:9".to_string())
            .build()
            .unwrap();
        assert!(impit.h3_client.is_none());
        assert!(matches!(
            get_with_http3_prior_knowledge(&impit).await,
            Err(ImpitError::Http3Disabled)
        ));
    }
}
