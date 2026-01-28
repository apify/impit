//! Browser fingerprint data structures
//!
//! This module contains all the types needed to define a complete browser fingerprint,
//! including TLS, HTTP/2, and HTTP header configurations.

pub mod database;
mod types;

pub use types::*;

/// A complete browser fingerprint containing TLS, HTTP/2, and HTTP header configurations.
#[derive(Clone, Debug)]
pub struct BrowserFingerprint {
    pub name: String,
    pub version: String,
    pub tls: TlsFingerprint,
    pub http2: Http2Fingerprint,
    pub headers: Vec<(String, String)>,
}

impl BrowserFingerprint {
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        tls: TlsFingerprint,
        http2: Http2Fingerprint,
        headers: Vec<(String, String)>,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            tls,
            http2,
            headers,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TlsFingerprint {
    pub cipher_suites: Vec<CipherSuite>,
    pub key_exchange_groups: Vec<KeyExchangeGroup>,
    pub signature_algorithms: Vec<SignatureAlgorithm>,
    pub extensions: TlsExtensions,
    pub ech_config: Option<EchConfig>,
    pub alpn_protocols: Vec<Vec<u8>>,
}

impl TlsFingerprint {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cipher_suites: Vec<CipherSuite>,
        key_exchange_groups: Vec<KeyExchangeGroup>,
        signature_algorithms: Vec<SignatureAlgorithm>,
        extensions: TlsExtensions,
        ech_config: Option<EchConfig>,
        alpn_protocols: Vec<Vec<u8>>,
    ) -> Self {
        Self {
            cipher_suites,
            key_exchange_groups,
            signature_algorithms,
            extensions,
            ech_config,
            alpn_protocols,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Http2Fingerprint {
    pub pseudo_header_order: Vec<String>,
}

/// TLS extensions configuration.
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct TlsExtensions {
    pub server_name: bool,
    pub status_request: bool,
    pub supported_groups: bool,
    pub signature_algorithms: bool,
    pub application_layer_protocol_negotiation: bool,
    pub signed_certificate_timestamp: bool,
    pub key_share: bool,
    pub psk_key_exchange_modes: bool,
    pub supported_versions: bool,
    pub compress_certificate: Option<Vec<CertificateCompressionAlgorithm>>,
    pub application_settings: bool,
    /// Use new ALPS codepoint (17613) instead of old (17513). Chrome 136+ uses new codepoint.
    pub use_new_alps_codepoint: bool,
    pub delegated_credentials: bool,
    pub record_size_limit: Option<u16>,
    pub extension_order: Vec<ExtensionType>,
    /// Whether to enable session tickets (TLS 1.2). Defaults to true.
    /// Set to false for browsers like Safari 18.0 that don't send session_ticket extension.
    pub session_ticket: bool,
    /// Whether to send padding extension (RFC7685).
    pub padding: bool,
}

impl TlsExtensions {
    /// Creates a new TLS extensions configuration.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        server_name: bool,
        status_request: bool,
        supported_groups: bool,
        signature_algorithms: bool,
        application_layer_protocol_negotiation: bool,
        signed_certificate_timestamp: bool,
        key_share: bool,
        psk_key_exchange_modes: bool,
        supported_versions: bool,
        compress_certificate: Option<Vec<CertificateCompressionAlgorithm>>,
        application_settings: bool,
        delegated_credentials: bool,
        record_size_limit: Option<u16>,
        extension_order: Vec<ExtensionType>,
    ) -> Self {
        Self {
            server_name,
            status_request,
            supported_groups,
            signature_algorithms,
            application_layer_protocol_negotiation,
            signed_certificate_timestamp,
            key_share,
            psk_key_exchange_modes,
            supported_versions,
            compress_certificate,
            application_settings,
            use_new_alps_codepoint: false,
            delegated_credentials,
            record_size_limit,
            extension_order,
            session_ticket: true,
            padding: false,
        }
    }

    pub fn with_session_ticket(mut self, enabled: bool) -> Self {
        self.session_ticket = enabled;
        self
    }

    pub fn with_new_alps_codepoint(mut self, use_new: bool) -> Self {
        self.use_new_alps_codepoint = use_new;
        self
    }

    pub fn with_padding(mut self, enabled: bool) -> Self {
        self.padding = enabled;
        self
    }
}

/// ECH (Encrypted Client Hello) configuration.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EchConfig {
    mode: EchMode,
    config_list: Option<Vec<u8>>,
}

impl EchConfig {
    /// Creates a new ECH configuration.
    pub fn new(mode: EchMode, config_list: Option<Vec<u8>>) -> Self {
        Self { mode, config_list }
    }

    /// Returns the ECH mode.
    pub fn mode(&self) -> &EchMode {
        &self.mode
    }

    /// Returns the ECH configuration list.
    pub fn config_list(&self) -> Option<&[u8]> {
        self.config_list.as_deref()
    }
}

/// ECH mode configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EchMode {
    /// ECH is disabled
    Disabled,
    /// ECH GREASE mode with specified HPKE suite
    Grease { hpke_suite: HpkeKemId },
    /// Real ECH with actual configuration
    Real,
}

impl TlsFingerprint {
    /// Converts this fingerprint to a rustls TlsFingerprint.
    pub fn to_rustls_fingerprint(&self) -> rustls::client::TlsFingerprint {
        use rustls::client::{
            FingerprintCertCompressionAlgorithm, FingerprintCipherSuite,
            FingerprintKeyExchangeGroup, FingerprintSignatureAlgorithm, TlsExtensionsConfig,
        };

        let cipher_suites: Vec<FingerprintCipherSuite> = self
            .cipher_suites
            .iter()
            .map(|cs| match cs {
                CipherSuite::TLS13_AES_128_GCM_SHA256 => {
                    FingerprintCipherSuite::TLS13_AES_128_GCM_SHA256
                }
                CipherSuite::TLS13_AES_256_GCM_SHA384 => {
                    FingerprintCipherSuite::TLS13_AES_256_GCM_SHA384
                }
                CipherSuite::TLS13_CHACHA20_POLY1305_SHA256 => {
                    FingerprintCipherSuite::TLS13_CHACHA20_POLY1305_SHA256
                }
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256 => {
                    FingerprintCipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256
                }
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256 => {
                    FingerprintCipherSuite::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256
                }
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384 => {
                    FingerprintCipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384
                }
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384 => {
                    FingerprintCipherSuite::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384
                }
                CipherSuite::TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256 => {
                    FingerprintCipherSuite::TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256
                }
                CipherSuite::TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256 => {
                    FingerprintCipherSuite::TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256
                }
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA => {
                    FingerprintCipherSuite::TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA
                }
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA => {
                    FingerprintCipherSuite::TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA
                }
                CipherSuite::TLS_RSA_WITH_AES_128_GCM_SHA256 => {
                    FingerprintCipherSuite::TLS_RSA_WITH_AES_128_GCM_SHA256
                }
                CipherSuite::TLS_RSA_WITH_AES_256_GCM_SHA384 => {
                    FingerprintCipherSuite::TLS_RSA_WITH_AES_256_GCM_SHA384
                }
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_CBC_SHA => {
                    FingerprintCipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_CBC_SHA
                }
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_CBC_SHA => {
                    FingerprintCipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_CBC_SHA
                }
                CipherSuite::TLS_RSA_WITH_AES_128_CBC_SHA => {
                    FingerprintCipherSuite::TLS_RSA_WITH_AES_128_CBC_SHA
                }
                CipherSuite::TLS_RSA_WITH_AES_256_CBC_SHA => {
                    FingerprintCipherSuite::TLS_RSA_WITH_AES_256_CBC_SHA
                }
                CipherSuite::TLS_ECDHE_ECDSA_WITH_3DES_EDE_CBC_SHA => {
                    FingerprintCipherSuite::TLS_ECDHE_ECDSA_WITH_3DES_EDE_CBC_SHA
                }
                CipherSuite::TLS_ECDHE_RSA_WITH_3DES_EDE_CBC_SHA => {
                    FingerprintCipherSuite::TLS_ECDHE_RSA_WITH_3DES_EDE_CBC_SHA
                }
                CipherSuite::TLS_RSA_WITH_3DES_EDE_CBC_SHA => {
                    FingerprintCipherSuite::TLS_RSA_WITH_3DES_EDE_CBC_SHA
                }
                CipherSuite::Grease => FingerprintCipherSuite::Grease,
            })
            .collect();

        let key_exchange_groups: Vec<FingerprintKeyExchangeGroup> = self
            .key_exchange_groups
            .iter()
            .map(|kg| match kg {
                KeyExchangeGroup::X25519 => FingerprintKeyExchangeGroup::X25519,
                KeyExchangeGroup::X25519MLKEM768 => FingerprintKeyExchangeGroup::X25519MLKEM768,
                KeyExchangeGroup::Secp256r1 => FingerprintKeyExchangeGroup::Secp256r1,
                KeyExchangeGroup::Secp384r1 => FingerprintKeyExchangeGroup::Secp384r1,
                KeyExchangeGroup::Secp521r1 => FingerprintKeyExchangeGroup::Secp521r1,
                KeyExchangeGroup::Ffdhe2048 => FingerprintKeyExchangeGroup::Ffdhe2048,
                KeyExchangeGroup::Ffdhe3072 => FingerprintKeyExchangeGroup::Ffdhe3072,
                KeyExchangeGroup::Ffdhe4096 => FingerprintKeyExchangeGroup::Ffdhe4096,
                KeyExchangeGroup::Ffdhe6144 => FingerprintKeyExchangeGroup::Ffdhe6144,
                KeyExchangeGroup::Ffdhe8192 => FingerprintKeyExchangeGroup::Ffdhe8192,
                KeyExchangeGroup::Grease => FingerprintKeyExchangeGroup::Grease,
            })
            .collect();

        let signature_algorithms: Vec<FingerprintSignatureAlgorithm> = self
            .signature_algorithms
            .iter()
            .map(|sa| match sa {
                SignatureAlgorithm::EcdsaSecp256r1Sha256 => {
                    FingerprintSignatureAlgorithm::EcdsaSecp256r1Sha256
                }
                SignatureAlgorithm::EcdsaSecp384r1Sha384 => {
                    FingerprintSignatureAlgorithm::EcdsaSecp384r1Sha384
                }
                SignatureAlgorithm::EcdsaSecp521r1Sha512 => {
                    FingerprintSignatureAlgorithm::EcdsaSecp521r1Sha512
                }
                SignatureAlgorithm::RsaPssRsaSha256 => {
                    FingerprintSignatureAlgorithm::RsaPssRsaSha256
                }
                SignatureAlgorithm::RsaPssRsaSha384 => {
                    FingerprintSignatureAlgorithm::RsaPssRsaSha384
                }
                SignatureAlgorithm::RsaPssRsaSha512 => {
                    FingerprintSignatureAlgorithm::RsaPssRsaSha512
                }
                SignatureAlgorithm::RsaPkcs1Sha256 => FingerprintSignatureAlgorithm::RsaPkcs1Sha256,
                SignatureAlgorithm::RsaPkcs1Sha384 => FingerprintSignatureAlgorithm::RsaPkcs1Sha384,
                SignatureAlgorithm::RsaPkcs1Sha512 => FingerprintSignatureAlgorithm::RsaPkcs1Sha512,
                SignatureAlgorithm::RsaPkcs1Sha1 => FingerprintSignatureAlgorithm::RsaPkcs1Sha1,
                SignatureAlgorithm::Ed25519 => FingerprintSignatureAlgorithm::Ed25519,
                SignatureAlgorithm::Ed448 => FingerprintSignatureAlgorithm::Ed448,
                SignatureAlgorithm::EcdsaSha1Legacy => {
                    FingerprintSignatureAlgorithm::EcdsaSha1Legacy
                }
            })
            .collect();

        // Check if GREASE is needed based on extension order
        let has_grease = self
            .extensions
            .extension_order
            .iter()
            .any(|e| matches!(e, ExtensionType::Grease));

        let extensions_config = TlsExtensionsConfig {
            grease: has_grease,
            signed_certificate_timestamp: self.extensions.signed_certificate_timestamp,
            application_settings: self.extensions.application_settings,
            use_new_alps_codepoint: self.extensions.use_new_alps_codepoint,
            delegated_credentials: self.extensions.delegated_credentials,
            record_size_limit: self.extensions.record_size_limit,
            renegotiation_info: true, // Common for both browsers
            padding: self.extensions.padding,
        };

        let cert_compression = self.extensions.compress_certificate.clone().map(|algos| {
            algos
                .iter()
                .map(|alg| match alg {
                    CertificateCompressionAlgorithm::Zlib => {
                        FingerprintCertCompressionAlgorithm::Zlib
                    }
                    CertificateCompressionAlgorithm::Brotli => {
                        FingerprintCertCompressionAlgorithm::Brotli
                    }
                    CertificateCompressionAlgorithm::Zstd => {
                        FingerprintCertCompressionAlgorithm::Zstd
                    }
                })
                .collect()
        });

        rustls::client::TlsFingerprint::new(
            cipher_suites,
            key_exchange_groups,
            signature_algorithms,
            extensions_config,
            self.alpn_protocols.clone(),
            cert_compression,
        )
    }
}
