use std::sync::Arc;

use crate::fingerprint::TlsFingerprint;
use reqwest::Version;
use rustls::client::danger::NoVerifier;
use rustls::client::EchGreaseConfig;
use rustls::crypto::CryptoProvider;
use rustls::RootCertStore;
use rustls::crypto::{aws_lc_rs, hpke::Hpke};

pub static GREASE_HPKE_SUITE: &dyn Hpke = aws_lc_rs::hpke::DH_KEM_X25519_HKDF_SHA256_AES_128;

pub struct TlsConfig {}

impl TlsConfig {
    pub fn builder() -> TlsConfigBuilder {
        TlsConfigBuilder::default()
    }
}

#[derive(Debug, Clone, Copy)]
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

impl TlsConfigBuilder {
    fn get_ech_mode(self) -> rustls::client::EchMode {
        let (public_key, _) = GREASE_HPKE_SUITE.generate_key_pair().unwrap();

        EchGreaseConfig::new(GREASE_HPKE_SUITE, public_key).into()
    }

    pub fn with_fingerprint(&mut self, fingerprint: Option<TlsFingerprint>) -> &mut Self {
        self.tls_fingerprint = fingerprint;
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

        let mut config = match self.tls_fingerprint {
            Some(fingerprint) => {
                let mut crypto_provider = CryptoProvider::builder()
                    .with_tls_fingerprint(&fingerprint)
                    .build();

                let mut config: rustls::ClientConfig =
                    rustls::ClientConfig::builder_with_provider(crypto_provider.into())
                        // TODO - use the ECH extension consistently
                        .with_ech(self.get_ech_mode())
                        .unwrap()
                        .with_root_certificates(root_store)
                        .with_tls_fingerprint(&fingerprint)
                        .with_no_client_auth();

                if self.ignore_tls_errors {
                    config
                        .dangerous()
                        .set_certificate_verifier(Arc::new(NoVerifier::new(Some(fingerprint))));
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
