mod ffdhe;
mod statics;

use std::sync::Arc;

use crate::emulation::Browser;
use crate::fingerprint::{self, TlsFingerprint};
use reqwest::Version;
use rustls::client::danger::NoVerifier;
use rustls::client::{BrowserEmulator as RusTLSBrowser, BrowserType, EchGreaseConfig};
use rustls::crypto::aws_lc_rs::kx_group::{SECP256R1, SECP384R1, X25519};
use rustls::crypto::CryptoProvider;
use rustls_platform_verifier::Verifier;

pub struct TlsConfig {}

impl TlsConfig {
    pub fn builder() -> TlsConfigBuilder {
        TlsConfigBuilder::default()
    }
}

#[derive(Debug, Clone)]
pub struct TlsConfigBuilder {
    browser: Option<Browser>,
    tls_fingerprint: Option<TlsFingerprint>,
    max_http_version: Version,
    ignore_tls_errors: bool,
}

impl Default for TlsConfigBuilder {
    fn default() -> Self {
        TlsConfigBuilder {
            browser: None,
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

    pub fn with_browser(&mut self, browser: Option<Browser>) -> &mut Self {
        self.browser = browser;
        self
    }

    /// Sets the TLS fingerprint directly.
    /// This takes precedence over the browser parameter.
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
        // Save fields before consuming self
        let ignore_tls_errors = self.ignore_tls_errors;
        let max_http_version = self.max_http_version;
        let browser = self.browser;

        // Determine which fingerprint to use
        let fingerprint = if let Some(fp) = self.tls_fingerprint {
            // Use provided fingerprint directly
            Some(fp)
        } else if let Some(browser) = browser {
            // Fall back to looking up fingerprint from browser enum
            Some(fingerprint::database::get_fingerprint(browser).tls().clone())
        } else {
            None
        };

        let mut config = if let Some(_fp) = fingerprint {
            // TODO: Use fingerprint data to configure rustls
            // For now, fall back to the old browser-based approach
            // This requires updates to the patched rustls library to expose configuration APIs

            // Temporary: Use old approach if browser is set
            if let Some(browser_val) = browser {
                let rustls_browser = match browser_val {
                    Browser::Chrome => RusTLSBrowser {
                        browser_type: BrowserType::Chrome,
                        version: 125,
                    },
                    Browser::Firefox => RusTLSBrowser {
                        browser_type: BrowserType::Firefox,
                        version: 125,
                    },
                };

                let mut crypto_provider = CryptoProvider::builder()
                    .with_browser_emulator(&rustls_browser)
                    .build();

                if browser_val == Browser::Firefox {
                    crypto_provider.kx_groups = vec![
                        X25519,
                        SECP256R1,
                        SECP384R1,
                        // TODO : add SECPR521R1
                        &ffdhe::FFDHE2048_KX_GROUP,
                        &ffdhe::FFDHE3072_KX_GROUP,
                    ];
                }

                let crypto_provider_arc: Arc<CryptoProvider> = crypto_provider.into();

                // Create verifier with embedded Mozilla CAs as fallback for minimal containers
                let verifier = Verifier::new_with_extra_roots(
                    webpki_root_certs::TLS_SERVER_ROOT_CERTS.iter().cloned(),
                    crypto_provider_arc.clone(),
                )
                .expect("Failed to create certificate verifier with embedded CA roots");

                let mut config: rustls::ClientConfig =
                    rustls::ClientConfig::builder_with_provider(crypto_provider_arc)
                        // TODO - use the ECH extension consistently
                        .with_ech(get_ech_mode())
                        .unwrap()
                        .dangerous()
                        .with_custom_certificate_verifier(Arc::new(verifier))
                        .with_browser_emulator(&rustls_browser)
                        .with_no_client_auth();

                if ignore_tls_errors {
                    config
                        .dangerous()
                        .set_certificate_verifier(Arc::new(NoVerifier::new(Some(rustls_browser))));
                }

                config
            } else {
                // No browser set, use default config
                let crypto_provider: Arc<CryptoProvider> = CryptoProvider::builder().build().into();

                let verifier = Verifier::new_with_extra_roots(
                    webpki_root_certs::TLS_SERVER_ROOT_CERTS.iter().cloned(),
                    crypto_provider.clone(),
                )
                .expect("Failed to create certificate verifier with embedded CA roots");

                let mut config: rustls::ClientConfig =
                    rustls::ClientConfig::builder_with_provider(crypto_provider)
                        .with_ech(get_ech_mode())
                        .unwrap()
                        .dangerous()
                        .with_custom_certificate_verifier(Arc::new(verifier))
                        .with_no_client_auth();

                if ignore_tls_errors {
                    config
                        .dangerous()
                        .set_certificate_verifier(Arc::new(NoVerifier::new(None)));
                }

                config
            }
        } else {
            // No fingerprint or browser set, use vanilla config
            let crypto_provider: Arc<CryptoProvider> = CryptoProvider::builder().build().into();

            let verifier = Verifier::new_with_extra_roots(
                webpki_root_certs::TLS_SERVER_ROOT_CERTS.iter().cloned(),
                crypto_provider.clone(),
            )
            .expect("Failed to create certificate verifier with embedded CA roots");

            let mut config: rustls::ClientConfig =
                rustls::ClientConfig::builder_with_provider(crypto_provider)
                    .with_ech(get_ech_mode())
                    .unwrap()
                    .dangerous()
                    .with_custom_certificate_verifier(Arc::new(verifier))
                    .with_no_client_auth();

            if ignore_tls_errors {
                config
                    .dangerous()
                    .set_certificate_verifier(Arc::new(NoVerifier::new(None)));
            }

            config
        };

        if max_http_version == Version::HTTP_3 {
            config.alpn_protocols = vec![b"h3".to_vec()];
        };

        config
    }
}
