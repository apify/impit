//! Browser fingerprint data structures
//!
//! This module contains all the types needed to define a complete browser fingerprint,
//! including TLS, HTTP/2, and HTTP header configurations.

pub mod database;
mod types;

pub use types::*;

/// A complete browser fingerprint containing TLS, HTTP/2, and HTTP header configurations.
///
/// This struct is immutable after creation to ensure consistency and prevent
/// accidental modifications that could break fingerprint accuracy.
#[derive(Clone, Debug)]
pub struct BrowserFingerprint {
    name: String,
    version: String,
    tls: TlsFingerprint,
    http2: Http2Fingerprint,
    headers: Vec<(String, String)>,
}

impl BrowserFingerprint {
    /// Creates a new browser fingerprint.
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

    /// Returns the browser name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the browser version.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Returns the TLS fingerprint.
    pub fn tls(&self) -> &TlsFingerprint {
        &self.tls
    }

    /// Returns the HTTP/2 fingerprint.
    pub fn http2(&self) -> &Http2Fingerprint {
        &self.http2
    }

    /// Returns the HTTP headers.
    pub fn headers(&self) -> &[(String, String)] {
        &self.headers
    }
}

/// TLS fingerprint configuration.
#[derive(Clone, Debug)]
pub struct TlsFingerprint {
    /// Cipher suites in preference order
    cipher_suites: Vec<CipherSuite>,
    /// Supported key exchange groups in preference order
    key_exchange_groups: Vec<KeyExchangeGroup>,
    /// Signature algorithms in preference order
    signature_algorithms: Vec<SignatureAlgorithm>,
    /// TLS extensions configuration
    extensions: TlsExtensions,
    /// ECH (Encrypted Client Hello) configuration
    ech_config: Option<EchConfig>,
    /// ALPN protocols in preference order
    alpn_protocols: Vec<Vec<u8>>,
}

impl TlsFingerprint {
    /// Creates a new TLS fingerprint.
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

    /// Returns the cipher suites.
    pub fn cipher_suites(&self) -> &[CipherSuite] {
        &self.cipher_suites
    }

    /// Returns the key exchange groups.
    pub fn key_exchange_groups(&self) -> &[KeyExchangeGroup] {
        &self.key_exchange_groups
    }

    /// Returns the signature algorithms.
    pub fn signature_algorithms(&self) -> &[SignatureAlgorithm] {
        &self.signature_algorithms
    }

    /// Returns the TLS extensions configuration.
    pub fn extensions(&self) -> &TlsExtensions {
        &self.extensions
    }

    /// Returns the ECH configuration.
    pub fn ech_config(&self) -> Option<&EchConfig> {
        self.ech_config.as_ref()
    }

    /// Returns the ALPN protocols.
    pub fn alpn_protocols(&self) -> &[Vec<u8>] {
        &self.alpn_protocols
    }
}

/// HTTP/2 fingerprint configuration.
#[derive(Clone, Debug)]
pub struct Http2Fingerprint {
    /// Pseudo-header ordering
    pseudo_header_order: Vec<String>,
    /// SETTINGS frame values
    settings: Http2Settings,
    /// Initial window sizes
    window_size: Http2WindowSize,
    /// Priority configuration
    priority: Option<Http2Priority>,
}

impl Http2Fingerprint {
    /// Creates a new HTTP/2 fingerprint.
    pub fn new(
        pseudo_header_order: Vec<String>,
        settings: Http2Settings,
        window_size: Http2WindowSize,
        priority: Option<Http2Priority>,
    ) -> Self {
        Self {
            pseudo_header_order,
            settings,
            window_size,
            priority,
        }
    }

    /// Returns the pseudo-header order.
    pub fn pseudo_header_order(&self) -> &[String] {
        &self.pseudo_header_order
    }

    /// Returns the SETTINGS frame values.
    pub fn settings(&self) -> &Http2Settings {
        &self.settings
    }

    /// Returns the window sizes.
    pub fn window_size(&self) -> &Http2WindowSize {
        &self.window_size
    }

    /// Returns the priority configuration.
    pub fn priority(&self) -> Option<&Http2Priority> {
        self.priority.as_ref()
    }
}

/// HTTP/2 SETTINGS frame configuration.
#[derive(Clone, Debug)]
pub struct Http2Settings {
    header_table_size: Option<u32>,
    enable_push: Option<bool>,
    max_concurrent_streams: Option<u32>,
    initial_window_size: Option<u32>,
    max_frame_size: Option<u32>,
    max_header_list_size: Option<u32>,
    /// Custom settings (non-standard settings IDs)
    custom: Vec<(u16, u32)>,
}

impl Http2Settings {
    /// Creates a new HTTP/2 settings configuration.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        header_table_size: Option<u32>,
        enable_push: Option<bool>,
        max_concurrent_streams: Option<u32>,
        initial_window_size: Option<u32>,
        max_frame_size: Option<u32>,
        max_header_list_size: Option<u32>,
        custom: Vec<(u16, u32)>,
    ) -> Self {
        Self {
            header_table_size,
            enable_push,
            max_concurrent_streams,
            initial_window_size,
            max_frame_size,
            max_header_list_size,
            custom,
        }
    }

    /// Returns the header table size.
    pub fn header_table_size(&self) -> Option<u32> {
        self.header_table_size
    }

    /// Returns whether push is enabled.
    pub fn enable_push(&self) -> Option<bool> {
        self.enable_push
    }

    /// Returns the maximum concurrent streams.
    pub fn max_concurrent_streams(&self) -> Option<u32> {
        self.max_concurrent_streams
    }

    /// Returns the initial window size.
    pub fn initial_window_size(&self) -> Option<u32> {
        self.initial_window_size
    }

    /// Returns the maximum frame size.
    pub fn max_frame_size(&self) -> Option<u32> {
        self.max_frame_size
    }

    /// Returns the maximum header list size.
    pub fn max_header_list_size(&self) -> Option<u32> {
        self.max_header_list_size
    }

    /// Returns the custom settings.
    pub fn custom(&self) -> &[(u16, u32)] {
        &self.custom
    }
}

/// HTTP/2 window size configuration.
#[derive(Clone, Copy, Debug)]
pub struct Http2WindowSize {
    connection_window_size: u32,
    stream_window_size: u32,
}

impl Http2WindowSize {
    /// Creates a new window size configuration.
    pub fn new(connection_window_size: u32, stream_window_size: u32) -> Self {
        Self {
            connection_window_size,
            stream_window_size,
        }
    }

    /// Returns the connection window size.
    pub fn connection_window_size(&self) -> u32 {
        self.connection_window_size
    }

    /// Returns the stream window size.
    pub fn stream_window_size(&self) -> u32 {
        self.stream_window_size
    }
}

/// HTTP/2 priority configuration.
#[derive(Clone, Copy, Debug)]
pub struct Http2Priority {
    weight: u8,
    depends_on: u32,
    exclusive: bool,
}

impl Http2Priority {
    /// Creates a new priority configuration.
    pub fn new(weight: u8, depends_on: u32, exclusive: bool) -> Self {
        Self {
            weight,
            depends_on,
            exclusive,
        }
    }

    /// Returns the priority weight.
    pub fn weight(&self) -> u8 {
        self.weight
    }

    /// Returns the stream this depends on.
    pub fn depends_on(&self) -> u32 {
        self.depends_on
    }

    /// Returns whether this dependency is exclusive.
    pub fn exclusive(&self) -> bool {
        self.exclusive
    }
}

/// TLS extensions configuration.
#[derive(Clone, Debug)]
pub struct TlsExtensions {
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
    /// Delegated credentials extension (Firefox-specific)
    delegated_credentials: bool,
    /// Record size limit extension (Firefox-specific)
    record_size_limit: Option<u16>,
    /// Extension order matters for fingerprinting
    extension_order: Vec<ExtensionType>,
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
            delegated_credentials,
            record_size_limit,
            extension_order,
        }
    }

    /// Returns whether server_name extension is enabled.
    pub fn server_name(&self) -> bool {
        self.server_name
    }

    /// Returns whether status_request extension is enabled.
    pub fn status_request(&self) -> bool {
        self.status_request
    }

    /// Returns whether supported_groups extension is enabled.
    pub fn supported_groups(&self) -> bool {
        self.supported_groups
    }

    /// Returns whether signature_algorithms extension is enabled.
    pub fn signature_algorithms(&self) -> bool {
        self.signature_algorithms
    }

    /// Returns whether ALPN extension is enabled.
    pub fn application_layer_protocol_negotiation(&self) -> bool {
        self.application_layer_protocol_negotiation
    }

    /// Returns whether signed_certificate_timestamp extension is enabled.
    pub fn signed_certificate_timestamp(&self) -> bool {
        self.signed_certificate_timestamp
    }

    /// Returns whether key_share extension is enabled.
    pub fn key_share(&self) -> bool {
        self.key_share
    }

    /// Returns whether psk_key_exchange_modes extension is enabled.
    pub fn psk_key_exchange_modes(&self) -> bool {
        self.psk_key_exchange_modes
    }

    /// Returns whether supported_versions extension is enabled.
    pub fn supported_versions(&self) -> bool {
        self.supported_versions
    }

    /// Returns the certificate compression algorithms.
    pub fn compress_certificate(&self) -> Option<&[CertificateCompressionAlgorithm]> {
        self.compress_certificate.as_deref()
    }

    /// Returns whether application_settings extension is enabled.
    pub fn application_settings(&self) -> bool {
        self.application_settings
    }

    /// Returns whether delegated_credentials extension is enabled.
    pub fn delegated_credentials(&self) -> bool {
        self.delegated_credentials
    }

    /// Returns the record size limit if set.
    pub fn record_size_limit(&self) -> Option<u16> {
        self.record_size_limit
    }

    /// Returns the extension order.
    pub fn extension_order(&self) -> &[ExtensionType] {
        &self.extension_order
    }
}

/// ECH (Encrypted Client Hello) configuration.
#[derive(Clone, Debug)]
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
#[derive(Clone, Copy, Debug)]
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
                CipherSuite::Grease => FingerprintCipherSuite::Grease,
            })
            .collect();

        let key_exchange_groups: Vec<FingerprintKeyExchangeGroup> = self
            .key_exchange_groups
            .iter()
            .map(|kg| match kg {
                KeyExchangeGroup::X25519 => FingerprintKeyExchangeGroup::X25519,
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
            .extension_order()
            .iter()
            .any(|e| matches!(e, ExtensionType::Grease));

        let extensions_config = TlsExtensionsConfig {
            grease: has_grease,
            signed_certificate_timestamp: self.extensions.signed_certificate_timestamp(),
            application_settings: self.extensions.application_settings(),
            delegated_credentials: self.extensions.delegated_credentials(),
            record_size_limit: self.extensions.record_size_limit(),
            renegotiation_info: true, // Common for both browsers
        };

        let cert_compression = self.extensions.compress_certificate().map(|algos| {
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
