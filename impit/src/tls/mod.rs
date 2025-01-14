mod ffdhe;
mod statics;

use std::sync::Arc;

use crate::emulation::Browser;
use reqwest::Version;
use rustls::client::danger::NoVerifier;
use rustls::client::{BrowserEmulator as RusTLSBrowser, BrowserType, EchGreaseConfig};
use rustls::crypto::aws_lc_rs::kx_group::{SECP256R1, SECP384R1, X25519};
use rustls::crypto::CryptoProvider;
use rustls::RootCertStore;

pub struct TlsConfig {}

impl TlsConfig {
    pub fn builder() -> TlsConfigBuilder {
        TlsConfigBuilder::default()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TlsConfigBuilder {
    browser: Option<Browser>,
    max_http_version: Version,
    ignore_tls_errors: bool,
}

impl Default for TlsConfigBuilder {
    fn default() -> Self {
        TlsConfigBuilder {
            browser: None,
            max_http_version: Version::HTTP_2,
            ignore_tls_errors: false,
        }
    }
}

impl TlsConfigBuilder {
    fn get_ech_mode(self) -> rustls::client::EchMode {
        let (public_key, _) = statics::GREASE_HPKE_SUITE.generate_key_pair().unwrap();

        EchGreaseConfig::new(statics::GREASE_HPKE_SUITE, public_key).into()
    }

    pub fn with_browser(&mut self, browser: Option<Browser>) -> &mut Self {
        self.browser = browser;
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
        let mut root_store = RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        let mut config = match self.browser {
            Some(browser) => {
                let rustls_browser = match browser {
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

                if browser == Browser::Firefox {
                    crypto_provider.kx_groups = vec![
                        X25519,
                        SECP256R1,
                        SECP384R1,
                        // TODO : add SECPR521R1
                        &ffdhe::FFDHE2048_KX_GROUP,
                        &ffdhe::FFDHE3072_KX_GROUP,
                    ];
                }

                let mut config: rustls::ClientConfig =
                    rustls::ClientConfig::builder_with_provider(crypto_provider.into())
                        // TODO - use the ECH extension consistently
                        .with_ech(self.get_ech_mode())
                        .unwrap()
                        .with_root_certificates(root_store)
                        .with_browser_emulator(&rustls_browser)
                        .with_no_client_auth();

                if self.ignore_tls_errors {
                    config
                        .dangerous()
                        .set_certificate_verifier(Arc::new(NoVerifier::new(Some(rustls_browser))));
                }

                config
            }
            None => {
                let crypto_provider = CryptoProvider::builder().build();

                let mut config: rustls::ClientConfig =
                    rustls::ClientConfig::builder_with_provider(crypto_provider.into())
                        // TODO - use the ECH extension consistently
                        .with_ech(self.get_ech_mode())
                        .unwrap()
                        .with_root_certificates(root_store)
                        .with_no_client_auth();

                if self.ignore_tls_errors {
                    config
                        .dangerous()
                        .set_certificate_verifier(Arc::new(NoVerifier::new(None)));
                }

                config
            }
        };

        if self.max_http_version == Version::HTTP_3 {
            config.alpn_protocols = vec![b"h3".to_vec()];
        };

        config
    }
}
