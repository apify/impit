mod cert_cache;
mod statics;

#[cfg(test)]
mod handshake_tests;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use crate::fingerprint::TlsFingerprint;
use reqwest::Version;
use rustls::client::danger::{NoVerifier, ServerCertVerifier};
use rustls::client::{EchGreaseConfig, Resumption, Tls12Resumption};
use rustls::crypto::CryptoProvider;
use rustls_platform_verifier::Verifier;

pub use cert_cache::CachingServerCertVerifier;

static VANILLA_CRYPTO_PROVIDER: OnceLock<Arc<CryptoProvider>> = OnceLock::new();
static VANILLA_VERIFIER: OnceLock<Verifiers> = OnceLock::new();

type BrowserCacheValue = (Arc<CryptoProvider>, Verifiers);
static BROWSER_CACHE: OnceLock<Mutex<HashMap<TlsFingerprint, BrowserCacheValue>>> = OnceLock::new();

/// The platform verifier, alone and wrapped in the chain verification cache. They live as long
/// as the process (see the statics above), so all clients share the cache.
#[derive(Clone)]
struct Verifiers {
    plain: Arc<Verifier>,
    cached: Arc<CachingServerCertVerifier>,
}

impl Verifiers {
    fn new(provider: Arc<CryptoProvider>) -> Self {
        let plain = Arc::new(
            Verifier::new_with_extra_roots(
                webpki_root_certs::TLS_SERVER_ROOT_CERTS.iter().cloned(),
                provider,
            )
            .expect("Failed to create certificate verifier with embedded CA roots"),
        );
        let cached = Arc::new(CachingServerCertVerifier::new(plain.clone()));
        Verifiers { plain, cached }
    }

    fn get(&self, cert_verification_cache: bool) -> Arc<dyn ServerCertVerifier> {
        if cert_verification_cache {
            self.cached.clone()
        } else {
            self.plain.clone()
        }
    }
}

fn get_browser_cache() -> &'static Mutex<HashMap<TlsFingerprint, BrowserCacheValue>> {
    BROWSER_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn get_vanilla_provider() -> Arc<CryptoProvider> {
    VANILLA_CRYPTO_PROVIDER
        .get_or_init(|| CryptoProvider::builder().build().into())
        .clone()
}

fn get_vanilla_verifier() -> Verifiers {
    let provider = get_vanilla_provider();
    VANILLA_VERIFIER
        .get_or_init(|| Verifiers::new(provider))
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

    let verifier = Verifiers::new(provider.clone());

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
    cert_verification_cache: bool,
}

impl Default for TlsConfigBuilder {
    fn default() -> Self {
        TlsConfigBuilder {
            tls_fingerprint: None,
            max_http_version: Version::HTTP_2,
            ignore_tls_errors: false,
            cert_verification_cache: true,
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

    /// Whether to reuse successful certificate-chain verifications, see
    /// [`CachingServerCertVerifier`]. On by default.
    pub fn with_cert_verification_cache(&mut self, cert_verification_cache: bool) -> &mut Self {
        self.cert_verification_cache = cert_verification_cache;
        self
    }

    pub fn build(self) -> rustls::ClientConfig {
        let ignore_tls_errors = self.ignore_tls_errors;
        let cert_verification_cache = self.cert_verification_cache;
        let max_http_version = self.max_http_version;

        let (fingerprint, cache_browser) = if let Some(fp) = self.tls_fingerprint {
            (Some(fp.clone()), Some(fp))
        } else {
            (None, None)
        };

        let mut config = if let Some(fp) = fingerprint {
            let rustls_fingerprint = fp.to_rustls_fingerprint();

            let alpn_protocols = fp.alpn_protocols.to_vec();
            let enable_session_ticket = fp.extensions.session_ticket;
            let ech_enabled = fp.ech_config.is_some();

            let (crypto_provider_arc, verifier) = if let Some(b) = cache_browser {
                get_or_create_browser_provider_and_verifier(b)
            } else {
                let provider: Arc<CryptoProvider> = CryptoProvider::builder()
                    .with_tls_fingerprint(rustls_fingerprint.clone())
                    .build()
                    .into();

                let verifier = Verifiers::new(provider.clone());

                (provider, verifier)
            };
            let verifier = verifier.get(cert_verification_cache);

            // Select protocol versions based on fingerprint
            let protocol_versions: &[&rustls::SupportedProtocolVersion] =
                if fp.extensions.supported_versions {
                    rustls::DEFAULT_VERSIONS
                } else {
                    &[&rustls::version::TLS12]
                };

            let mut config: rustls::ClientConfig = if ech_enabled {
                // ECH requires a different builder chain (no explicit protocol versions)
                rustls::ClientConfig::builder_with_provider(crypto_provider_arc)
                    .with_ech(get_ech_mode())
                    .unwrap()
                    .dangerous()
                    .with_custom_certificate_verifier(verifier)
                    .with_tls_fingerprint(rustls_fingerprint)
                    .with_no_client_auth()
            } else {
                rustls::ClientConfig::builder_with_provider(crypto_provider_arc)
                    .with_protocol_versions(protocol_versions)
                    .unwrap()
                    .dangerous()
                    .with_custom_certificate_verifier(verifier)
                    .with_tls_fingerprint(rustls_fingerprint)
                    .with_no_client_auth()
            };

            config.alpn_protocols = alpn_protocols;

            // Configure session resumption based on fingerprint
            if !enable_session_ticket {
                // Disable session tickets but keep session ID resumption
                config.resumption = Resumption::in_memory_sessions(256)
                    .tls12_resumption(Tls12Resumption::SessionIdOnly);
            }

            if ignore_tls_errors {
                config
                    .dangerous()
                    .set_certificate_verifier(Arc::new(NoVerifier::with_default_schemes()));
            }

            config
        } else {
            let crypto_provider = get_vanilla_provider();
            let verifier = get_vanilla_verifier().get(cert_verification_cache);

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
