mod statics;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use crate::fingerprint::TlsFingerprint;
use reqwest::Version;
use rustls::client::danger::NoVerifier;
use rustls::client::EchGreaseConfig;
use rustls::crypto::CryptoProvider;
use rustls_platform_verifier::Verifier;

static VANILLA_CRYPTO_PROVIDER: OnceLock<Arc<CryptoProvider>> = OnceLock::new();
static VANILLA_VERIFIER: OnceLock<Arc<Verifier>> = OnceLock::new();

type BrowserCacheValue = (Arc<CryptoProvider>, Arc<Verifier>);
static BROWSER_CACHE: OnceLock<Mutex<HashMap<TlsFingerprint, BrowserCacheValue>>> = OnceLock::new();

fn get_browser_cache() -> &'static Mutex<HashMap<TlsFingerprint, BrowserCacheValue>> {
    BROWSER_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn get_vanilla_provider() -> Arc<CryptoProvider> {
    VANILLA_CRYPTO_PROVIDER
        .get_or_init(|| CryptoProvider::builder().build().into())
        .clone()
}

fn get_vanilla_verifier() -> Arc<Verifier> {
    let provider = get_vanilla_provider();
    VANILLA_VERIFIER
        .get_or_init(|| {
            Arc::new(
                Verifier::new_with_extra_roots(
                    webpki_root_certs::TLS_SERVER_ROOT_CERTS.iter().cloned(),
                    provider,
                )
                .expect("Failed to create certificate verifier with embedded CA roots"),
            )
        })
        .clone()
}

fn get_or_create_browser_provider_and_verifier(
    tls_fingerprint: TlsFingerprint,
) -> BrowserCacheValue {
    {
        let cache = get_browser_cache().lock().unwrap();
        if let Some(cached) = cache.get(&tls_fingerprint) {
            return cached.clone();
        }
    }

    let rustls_fp = tls_fingerprint.to_rustls_fingerprint();

    let provider: Arc<CryptoProvider> = CryptoProvider::builder()
        .with_tls_fingerprint(rustls_fp)
        .build()
        .into();

    let verifier = Arc::new(
        Verifier::new_with_extra_roots(
            webpki_root_certs::TLS_SERVER_ROOT_CERTS.iter().cloned(),
            provider.clone(),
        )
        .expect("Failed to create certificate verifier with embedded CA roots"),
    );

    {
        let mut cache = get_browser_cache().lock().unwrap();
        cache.insert(tls_fingerprint, (provider.clone(), verifier.clone()));
    }

    (provider, verifier)
}

pub struct TlsConfig {}

impl TlsConfig {
    pub fn builder() -> TlsConfigBuilder {
        TlsConfigBuilder::default()
    }
}

#[derive(Debug, Clone)]
pub struct TlsConfigBuilder {
    tls_fingerprint: Option<TlsFingerprint>,
    max_http_version: Version,
    ignore_tls_errors: bool,
}

impl Default for TlsConfigBuilder {
    fn default() -> Self {
        TlsConfigBuilder {
            tls_fingerprint: None,
            max_http_version: Version::HTTP_2,
            ignore_tls_errors: false,
        }
    }
}

fn get_ech_mode() -> rustls::client::EchMode {
    let (public_key, _) = statics::GREASE_HPKE_SUITE.generate_key_pair().unwrap();
    EchGreaseConfig::new(statics::GREASE_HPKE_SUITE, public_key).into()
}

impl TlsConfigBuilder {
    pub fn with_tls_fingerprint(&mut self, fingerprint: TlsFingerprint) -> &mut Self {
        self.tls_fingerprint = Some(fingerprint);
        self
    }

    pub fn with_http3(&mut self) -> &mut Self {
        self.max_http_version = Version::HTTP_3;
        self
    }

    pub fn with_ignore_tls_errors(&mut self, ignore_tls_errors: bool) -> &mut Self {
        self.ignore_tls_errors = ignore_tls_errors;
        self
    }

    pub fn build(self) -> rustls::ClientConfig {
        let ignore_tls_errors = self.ignore_tls_errors;
        let max_http_version = self.max_http_version;

        let (fingerprint, cache_browser) = if let Some(fp) = self.tls_fingerprint {
            (Some(fp.clone()), Some(fp))
        } else {
            (None, None)
        };

        let mut config = if let Some(fp) = fingerprint {
            let rustls_fingerprint = fp.to_rustls_fingerprint();

            let alpn_protocols = fp.alpn_protocols.to_vec();

            let (crypto_provider_arc, verifier) = if let Some(b) = cache_browser {
                get_or_create_browser_provider_and_verifier(b)
            } else {
                let provider: Arc<CryptoProvider> = CryptoProvider::builder()
                    .with_tls_fingerprint(rustls_fingerprint.clone())
                    .build()
                    .into();

                let verifier = Arc::new(
                    Verifier::new_with_extra_roots(
                        webpki_root_certs::TLS_SERVER_ROOT_CERTS.iter().cloned(),
                        provider.clone(),
                    )
                    .expect("Failed to create certificate verifier with embedded CA roots"),
                );

                (provider, verifier)
            };

            let mut config: rustls::ClientConfig =
                rustls::ClientConfig::builder_with_provider(crypto_provider_arc)
                    .with_ech(get_ech_mode())
                    .unwrap()
                    .dangerous()
                    .with_custom_certificate_verifier(verifier)
                    .with_tls_fingerprint(rustls_fingerprint)
                    .with_no_client_auth();

            config.alpn_protocols = alpn_protocols;

            if ignore_tls_errors {
                config
                    .dangerous()
                    .set_certificate_verifier(Arc::new(NoVerifier::with_default_schemes()));
            }

            config
        } else {
            let crypto_provider = get_vanilla_provider();
            let verifier = get_vanilla_verifier();

            let mut config: rustls::ClientConfig =
                rustls::ClientConfig::builder_with_provider(crypto_provider)
                    .with_ech(get_ech_mode())
                    .unwrap()
                    .dangerous()
                    .with_custom_certificate_verifier(verifier)
                    .with_no_client_auth();

            if ignore_tls_errors {
                config
                    .dangerous()
                    .set_certificate_verifier(Arc::new(NoVerifier::with_default_schemes()));
            }

            config
        };

        if max_http_version == Version::HTTP_3 {
            config.alpn_protocols = vec![b"h3".to_vec()];
        };

        config
    }
}
