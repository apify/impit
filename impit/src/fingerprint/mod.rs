//! Browser fingerprint data structures
//!
//! This module contains all the types needed to define a complete browser fingerprint,
//! including TLS, HTTP/2, and HTTP header configurations.

pub mod database;
mod types;

use rand::Rng;

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

    pub fn generate_multipart_boundary(&self) -> String {
        match self.name.as_str() {
            "Chrome" => {
                const CHARS: &[u8] =
                    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
                let mut rng = rand::rng();
                let suffix: String = (0..16)
                    .map(|_| CHARS[rng.random_range(0..CHARS.len())] as char)
                    .collect();
                format!("----WebKitFormBoundary{suffix}")
            }
            "Firefox" => {
                let mut rng = rand::rng();
                let a: u64 = rng.random();
                let b: u64 = rng.random();
                format!("----geckoformboundary{a:x}{b:x}")
            }
            "OkHttp" => uuid::Uuid::new_v4().to_string(),
            _ => default_multipart_boundary(),
        }
    }
}

pub fn default_multipart_boundary() -> String {
    let mut rng = rand::rng();
    let suffix: String = (0..16)
        .map(|_| rng.random_range(0u8..10).to_string())
        .collect();
    format!("----formdata-impit-{suffix}")
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

/// HTTP/2 configuration of a browser fingerprint, mostly the client's first SETTINGS frame.
///
/// `Some` values are passed to hyper's HTTP/2 builder. For `max_frame_size` (SETTINGS id 5) and
/// `max_header_list_size` (id 6), `None` means the setting is not sent; for the other `Option`
/// fields, `None` keeps hyper's default. `send_enable_push: true` sends ENABLE_PUSH (id 2) as `0`
/// (hyper always disables push); `false` leaves it out, and the server may then push, but every
/// pushed stream is refused. `no_rfc7540_priorities: Some(b)` adds SETTINGS_NO_RFC7540_PRIORITIES
/// (id 9, RFC 9218) with the value `b as u32`.
///
/// Leaving settings out and adding setting 9 need impit's h2 (apify/h2) with support for the
/// `IMPIT_H2_OMIT_SETTINGS` and `IMPIT_H2_NO_RFC7540_PRIORITIES` environment variables. impit sets
/// both whenever it builds a client with a fingerprint, and h2 reads them when an HTTP/2
/// connection starts. Like `pseudo_header_order` (`IMPIT_H2_PSEUDOHEADERS_ORDER`), they are
/// process-wide: the last client built with a fingerprint decides them for every new HTTP/2
/// connection in the process. So clients with different profiles in one process share the most
/// recently built one's left-out settings and setting 9. (Clients without a fingerprint don't
/// offer `h2` in ALPN and never use HTTP/2.) Only the SETTINGS frame changes; the limits h2
/// enforces stay as configured.
///
/// For custom fingerprints: a struct literal must set the newer fields `max_frame_size`,
/// `send_enable_push` and `no_rfc7540_priorities`, and `max_header_list_size: None` now leaves
/// setting 6 out instead of sending hyper's 16 KiB default. To send what hyper sends by default,
/// use `max_frame_size: Some(16_384)`, `max_header_list_size: Some(16_384)`,
/// `send_enable_push: true` and `no_rfc7540_priorities: None`.
#[derive(Clone, Debug)]
pub struct Http2Fingerprint {
    pub pseudo_header_order: Vec<String>,
    pub initial_stream_window_size: Option<u32>,
    pub initial_connection_window_size: Option<u32>,
    /// SETTINGS_MAX_HEADER_LIST_SIZE (id 6); `None`: not sent.
    pub max_header_list_size: Option<u32>,
    pub header_table_size: Option<u32>,
    pub max_concurrent_streams: Option<u32>,
    /// SETTINGS_MAX_FRAME_SIZE (id 5); `None`: not sent. `Some(v)` must be in
    /// `16_384..=16_777_215` (RFC 9113 §6.5.2): h2 panics on other values when a connection
    /// starts, not when the client is built.
    pub max_frame_size: Option<u32>,
    /// Whether SETTINGS_ENABLE_PUSH (id 2) is sent, always as `0`.
    pub send_enable_push: bool,
    /// SETTINGS_NO_RFC7540_PRIORITIES (id 9); `None`: not sent.
    pub no_rfc7540_priorities: Option<bool>,
}

impl Http2Fingerprint {
    /// The values of the env vars that make apify/h2 send these SETTINGS: the ids to leave out
    /// (ascending, comma-separated) and setting 9's value. Empty strings change nothing.
    pub(crate) fn h2_settings_env(&self) -> [(&'static str, String); 2] {
        let omit = [
            (!self.send_enable_push, "2"),
            (self.max_frame_size.is_none(), "5"),
            (self.max_header_list_size.is_none(), "6"),
        ]
        .into_iter()
        .filter(|&(left_out, _)| left_out)
        .map(|(_, id)| id)
        .collect::<Vec<_>>()
        .join(",");
        let no_rfc7540_priorities = match self.no_rfc7540_priorities {
            Some(true) => "1",
            Some(false) => "0",
            None => "",
        };
        [
            ("IMPIT_H2_OMIT_SETTINGS", omit),
            (
                "IMPIT_H2_NO_RFC7540_PRIORITIES",
                no_rfc7540_priorities.to_string(),
            ),
        ]
    }
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
    /// Trust Anchor IDs to advertise (draft-ietf-tls-trust-anchor-ids, Chrome's codepoint `0xca34`), one
    /// byte string of 1-32 bytes per ID. `Some(vec![])` sends an empty list; `None` doesn't send the
    /// extension. List only anchors impit's verifier trusts: a server may leave a listed intermediate
    /// out of its chain, and impit then can't verify it.
    pub trust_anchors: Option<Vec<Vec<u8>>>,
    /// Send Chrome's two GREASE extensions: an empty one first and a 1-byte one last (just before padding
    /// and `pre_shared_key`). Padding moves to right after the last one and a listed `Grease` becomes the
    /// first one; the rest of the order is unchanged.
    pub grease_first_and_last: bool,
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
            trust_anchors: None,
            grease_first_and_last: false,
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

    pub fn with_trust_anchors(mut self, ids: Vec<Vec<u8>>) -> Self {
        self.trust_anchors = Some(ids);
        self
    }

    pub fn with_grease_first_and_last(mut self, enabled: bool) -> Self {
        self.grease_first_and_last = enabled;
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
                CipherSuite::TLS_RSA_WITH_3DES_EDE_CBC_SHA => {
                    FingerprintCipherSuite::TLS_RSA_WITH_3DES_EDE_CBC_SHA
                }
                CipherSuite::TLS_ECDHE_ECDSA_WITH_3DES_EDE_CBC_SHA => {
                    FingerprintCipherSuite::TLS_ECDHE_ECDSA_WITH_3DES_EDE_CBC_SHA
                }
                CipherSuite::TLS_ECDHE_RSA_WITH_3DES_EDE_CBC_SHA => {
                    FingerprintCipherSuite::TLS_ECDHE_RSA_WITH_3DES_EDE_CBC_SHA
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
                SignatureAlgorithm::MlDsa44 => FingerprintSignatureAlgorithm::MlDsa44,
                SignatureAlgorithm::MlDsa65 => FingerprintSignatureAlgorithm::MlDsa65,
                SignatureAlgorithm::MlDsa87 => FingerprintSignatureAlgorithm::MlDsa87,
                SignatureAlgorithm::EcdsaSha1Legacy => {
                    FingerprintSignatureAlgorithm::EcdsaSha1Legacy
                }
                SignatureAlgorithm::Grease => FingerprintSignatureAlgorithm::Grease,
            })
            .collect();

        // Check if GREASE is needed based on extension order
        let has_grease = self
            .extensions
            .extension_order
            .iter()
            .any(|e| matches!(e, ExtensionType::Grease));

        use rustls::internal::msgs::enums::ExtensionType as RustlsExtType;

        let extension_order: Vec<RustlsExtType> = self
            .extensions
            .extension_order
            .iter()
            .filter_map(|ext| {
                match ext {
                    ExtensionType::ServerName => Some(RustlsExtType::ServerName),
                    ExtensionType::StatusRequest => Some(RustlsExtType::StatusRequest),
                    ExtensionType::SupportedGroups => Some(RustlsExtType::EllipticCurves),
                    ExtensionType::EcPointFormats => Some(RustlsExtType::ECPointFormats),
                    ExtensionType::SignatureAlgorithms => Some(RustlsExtType::SignatureAlgorithms),
                    ExtensionType::ApplicationLayerProtocolNegotiation => {
                        Some(RustlsExtType::ALProtocolNegotiation)
                    }
                    ExtensionType::SignedCertificateTimestamp => Some(RustlsExtType::SCT),
                    ExtensionType::KeyShare => Some(RustlsExtType::KeyShare),
                    ExtensionType::PskKeyExchangeModes => Some(RustlsExtType::PSKKeyExchangeModes),
                    ExtensionType::SupportedVersions => Some(RustlsExtType::SupportedVersions),
                    ExtensionType::CompressCertificate => Some(RustlsExtType::CompressCertificate),
                    ExtensionType::DelegatedCredentials => {
                        Some(RustlsExtType::DelegatedCredentials)
                    }
                    ExtensionType::RecordSizeLimit => Some(RustlsExtType::RecordSizeLimit),
                    ExtensionType::ApplicationSettings => Some(RustlsExtType::ApplicationSettings),
                    ExtensionType::ExtendedMasterSecret => {
                        Some(RustlsExtType::ExtendedMasterSecret)
                    }
                    ExtensionType::SessionTicket => Some(RustlsExtType::SessionTicket),
                    ExtensionType::RenegotiationInfo => Some(RustlsExtType::RenegotiationInfo),
                    ExtensionType::Padding => Some(RustlsExtType::Padding),
                    ExtensionType::Grease => Some(RustlsExtType::ReservedGrease),
                    ExtensionType::EarlyData => Some(RustlsExtType::EarlyData),
                    ExtensionType::PostHandshakeAuth => Some(RustlsExtType::PostHandshakeAuth),
                    ExtensionType::SignatureAlgorithmsCert => {
                        Some(RustlsExtType::SignatureAlgorithmsCert)
                    }
                    ExtensionType::TrustAnchors => Some(RustlsExtType::TrustAnchors),
                    // PreSharedKey and ECH are handled separately by rustls (always last)
                    ExtensionType::PreSharedKey => None,
                    // Skip types that don't map cleanly
                    _ => None,
                }
            })
            .collect();

        let extensions_config = TlsExtensionsConfig {
            grease: has_grease,
            grease_first_and_last: self.extensions.grease_first_and_last,
            signed_certificate_timestamp: self.extensions.signed_certificate_timestamp,
            application_settings: self.extensions.application_settings,
            use_new_alps_codepoint: self.extensions.use_new_alps_codepoint,
            delegated_credentials: self.extensions.delegated_credentials,
            record_size_limit: self.extensions.record_size_limit,
            trust_anchors: self.extensions.trust_anchors.clone(),
            renegotiation_info: true, // Common for both browsers
            padding: self.extensions.padding,
            supported_versions: self.extensions.supported_versions,
            extension_order,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn http2(
        max_frame_size: Option<u32>,
        max_header_list_size: Option<u32>,
        send_enable_push: bool,
        no_rfc7540_priorities: Option<bool>,
    ) -> Http2Fingerprint {
        Http2Fingerprint {
            pseudo_header_order: vec![],
            initial_stream_window_size: None,
            initial_connection_window_size: None,
            max_header_list_size,
            header_table_size: None,
            max_concurrent_streams: None,
            max_frame_size,
            send_enable_push,
            no_rfc7540_priorities,
        }
    }

    #[test]
    fn h2_settings_env_lists_left_out_ids_and_setting_9() {
        let sent = Some(16_384);
        let cases = [
            (http2(sent, sent, true, None), "", ""),
            (http2(None, sent, true, None), "5", ""),
            (http2(sent, None, true, None), "6", ""),
            (http2(sent, sent, false, None), "2", ""),
            (http2(None, None, false, None), "2,5,6", ""),
            (http2(sent, None, false, Some(true)), "2,6", "1"),
            (http2(sent, sent, true, Some(false)), "", "0"),
        ];
        for (fingerprint, omit, no_rfc7540) in cases {
            assert_eq!(
                fingerprint.h2_settings_env(),
                [
                    ("IMPIT_H2_OMIT_SETTINGS", omit.to_string()),
                    ("IMPIT_H2_NO_RFC7540_PRIORITIES", no_rfc7540.to_string()),
                ],
                "{fingerprint:?}"
            );
        }
    }
}
