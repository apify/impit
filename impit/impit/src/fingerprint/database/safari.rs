//! Safari browser fingerprints
//!
//! Safari uses Apple's Network.framework TLS stack which produces a distinctly
//! different JA3/JA4 fingerprint from Chrome (BoringSSL) and Firefox (NSS).
//! Key differences:
//! - No GREASE in cipher suites (only in extensions via ECH GREASE)
//! - No session_ticket extension
//! - No compress_certificate
//! - No application_settings (ALPS)
//! - No signed_certificate_timestamp
//! - No post-quantum key exchange groups
//! - Uses ec_point_formats extension
//! - Different cipher suite ordering (ECDSA prioritized differently)

use crate::fingerprint::*;

/// Helper to create macOS-version-specific Safari headers.
/// Safari only runs on macOS/iOS, so the OS parameter here is a macOS version variant.
/// `os` is "macos" (the only valid OS for Safari — always macOS).
/// `macos_version` selects the macOS version string: "10_15_7", "13_6_9", "14_7_1", or "15_2".
pub fn safari_headers_for_os(version: &str, macos_version: Option<&str>) -> Vec<(String, String)> {
    let mac_ver = macos_version.unwrap_or("10_15_7");
    let ua = format!(
        "Mozilla/5.0 (Macintosh; Intel Mac OS X {mac_ver}) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/{version} Safari/605.1.15"
    );

    vec![
        ("User-Agent".to_string(), ua),
        ("Accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".to_string()),
        ("Accept-Language".to_string(), "en-US,en;q=0.9".to_string()),
        ("Accept-Encoding".to_string(), "gzip, deflate, br".to_string()),
        ("sec-fetch-dest".to_string(), "document".to_string()),
        ("sec-fetch-mode".to_string(), "navigate".to_string()),
        ("sec-fetch-site".to_string(), "none".to_string()),
    ]
}

/// Safari 18.0 fingerprint module (macOS Sequoia 15.x)
/// Based on Safari 18.0 / WebKit on macOS Sequoia with Apple TLS stack
pub mod safari_18_0 {
    use super::*;

    /// Returns the complete Safari 18.0 fingerprint
    pub fn fingerprint() -> BrowserFingerprint {
        BrowserFingerprint::new(
            "Safari",
            "18.0",
            tls_fingerprint(),
            http2_fingerprint(),
            headers(),
        )
    }

    /// Returns Safari 18.0 fingerprint with macOS-version-specific headers.
    /// `os` should always be "macos" (Safari doesn't run on other OSes).
    /// `macos_version` can be "10_15_7", "13_6_9", "14_7_1", or "15_2".
    pub fn fingerprint_with_os(_os: &str, macos_version: Option<&str>) -> BrowserFingerprint {
        BrowserFingerprint::new(
            "Safari",
            "18.0",
            tls_fingerprint(),
            http2_fingerprint(),
            super::safari_headers_for_os("18.0", macos_version),
        )
    }

    /// Safari 18.0 TLS fingerprint
    pub(crate) fn tls_fingerprint() -> TlsFingerprint {
        TlsFingerprint::new(
            // Cipher suites in Safari 18.0 preference order
            // Safari does NOT use GREASE in cipher suites (key differentiator from Chrome)
            // TLS 1.3 suites first, then ECDHE-ECDSA, then ECDHE-RSA, then RSA
            vec![
                CipherSuite::TLS13_AES_128_GCM_SHA256,
                CipherSuite::TLS13_AES_256_GCM_SHA384,
                CipherSuite::TLS13_CHACHA20_POLY1305_SHA256,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
                CipherSuite::TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_CBC_SHA,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_CBC_SHA,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA,
                CipherSuite::TLS_RSA_WITH_AES_256_GCM_SHA384,
                CipherSuite::TLS_RSA_WITH_AES_128_GCM_SHA256,
                CipherSuite::TLS_RSA_WITH_AES_256_CBC_SHA,
                CipherSuite::TLS_RSA_WITH_AES_128_CBC_SHA,
            ],
            // Key exchange groups - Safari does NOT include post-quantum (no X25519MLKEM768)
            // No GREASE in supported_groups either
            vec![
                KeyExchangeGroup::X25519,
                KeyExchangeGroup::Secp256r1,
                KeyExchangeGroup::Secp384r1,
                KeyExchangeGroup::Secp521r1,
            ],
            // Signature algorithms - Safari includes Ed25519 and legacy SHA1
            vec![
                SignatureAlgorithm::EcdsaSecp256r1Sha256,
                SignatureAlgorithm::EcdsaSecp384r1Sha384,
                SignatureAlgorithm::EcdsaSecp521r1Sha512,
                SignatureAlgorithm::RsaPssRsaSha256,
                SignatureAlgorithm::RsaPssRsaSha384,
                SignatureAlgorithm::RsaPssRsaSha512,
                SignatureAlgorithm::RsaPkcs1Sha256,
                SignatureAlgorithm::RsaPkcs1Sha384,
                SignatureAlgorithm::RsaPkcs1Sha512,
                SignatureAlgorithm::EcdsaSha1Legacy,
                SignatureAlgorithm::RsaPkcs1Sha1,
            ],
            // TLS extensions configuration
            // Safari has very different extension set from Chrome:
            // - No session_ticket
            // - No compress_certificate
            // - No application_settings (ALPS)
            // - No signed_certificate_timestamp
            // - No delegated_credentials
            // - Uses ec_point_formats
            TlsExtensions::new(
                true,  // server_name
                true,  // status_request (OCSP stapling)
                true,  // supported_groups
                true,  // signature_algorithms
                true,  // application_layer_protocol_negotiation
                false, // signed_certificate_timestamp (Safari doesn't send this)
                true,  // key_share
                true,  // psk_key_exchange_modes
                true,  // supported_versions
                None,  // compress_certificate (Safari doesn't support cert compression)
                false, // application_settings (Safari doesn't use ALPS)
                false, // delegated_credentials (Safari doesn't use this)
                None,  // record_size_limit (Safari doesn't send this)
                // Safari extension order is distinctly different from Chrome
                vec![
                    ExtensionType::Grease(0xbaba),
                    ExtensionType::ServerName,
                    ExtensionType::ExtendedMasterSecret,
                    ExtensionType::RenegotiationInfo,
                    ExtensionType::SupportedGroups,
                    ExtensionType::EcPointFormats,
                    ExtensionType::ApplicationLayerProtocolNegotiation,
                    ExtensionType::StatusRequest,
                    ExtensionType::SignatureAlgorithms,
                    ExtensionType::KeyShare,
                    ExtensionType::PskKeyExchangeModes,
                    ExtensionType::SupportedVersions,
                    ExtensionType::Grease(0xbaba),
                    ExtensionType::Padding,
                ],
            )
            .with_session_ticket(false) // Safari doesn't send session_ticket
            .with_padding(true),        // Safari uses padding extension
            // ECH configuration (GREASE mode - Safari 17.0+ supports ECH GREASE)
            Some(EchConfig::new(
                EchMode::Grease {
                    hpke_suite: HpkeKemId::DhKemX25519HkdfSha256,
                },
                None,
            )),
            // ALPN protocols
            vec![b"h2".to_vec(), b"http/1.1".to_vec()],
        )
    }

    /// Safari 18.0 HTTP/2 fingerprint
    /// Safari uses different pseudo-header order than Chrome
    pub(crate) fn http2_fingerprint() -> Http2Fingerprint {
        Http2Fingerprint {
            pseudo_header_order: vec![
                ":method".to_string(),
                ":scheme".to_string(),
                ":path".to_string(),
                ":authority".to_string(),
            ],
            settings_header_table_size: Some(65536),
            settings_enable_push: Some(true),
            settings_max_concurrent_streams: Some(100),
            settings_initial_window_size: Some(4194304),
            settings_max_frame_size: Some(16384),
            settings_max_header_list_size: None,
            connection_window_size_increment: Some(10485760),
        }
    }

    /// Safari 18.0 HTTP headers (macOS Sequoia)
    fn headers() -> Vec<(String, String)> {
        vec![
            ("User-Agent".to_string(), "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Safari/605.1.15".to_string()),
            ("Accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".to_string()),
            ("Accept-Language".to_string(), "en-US,en;q=0.9".to_string()),
            ("Accept-Encoding".to_string(), "gzip, deflate, br".to_string()),
            ("sec-fetch-dest".to_string(), "document".to_string()),
            ("sec-fetch-mode".to_string(), "navigate".to_string()),
            ("sec-fetch-site".to_string(), "none".to_string()),
        ]
    }
}

/// Safari 18.4 fingerprint module (macOS Sequoia 15.4+)
pub mod safari_18_4 {
    use super::*;

    /// Returns the complete Safari 18.4 fingerprint
    pub fn fingerprint() -> BrowserFingerprint {
        BrowserFingerprint::new(
            "Safari",
            "18.4",
            safari_18_0::tls_fingerprint(), // Same TLS as 18.0
            safari_18_0::http2_fingerprint(),
            headers(),
        )
    }

    /// Returns Safari 18.4 fingerprint with macOS-version-specific headers.
    pub fn fingerprint_with_os(_os: &str, macos_version: Option<&str>) -> BrowserFingerprint {
        BrowserFingerprint::new(
            "Safari",
            "18.4",
            safari_18_0::tls_fingerprint(),
            safari_18_0::http2_fingerprint(),
            super::safari_headers_for_os("18.4", macos_version),
        )
    }

    /// Safari 18.4 HTTP headers
    fn headers() -> Vec<(String, String)> {
        vec![
            ("User-Agent".to_string(), "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.4 Safari/605.1.15".to_string()),
            ("Accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".to_string()),
            ("Accept-Language".to_string(), "en-US,en;q=0.9".to_string()),
            ("Accept-Encoding".to_string(), "gzip, deflate, br".to_string()),
            ("sec-fetch-dest".to_string(), "document".to_string()),
            ("sec-fetch-mode".to_string(), "navigate".to_string()),
            ("sec-fetch-site".to_string(), "none".to_string()),
        ]
    }
}

/// Safari 17.0 fingerprint module (macOS Sonoma)
pub mod safari_17_0 {
    use super::*;

    /// Returns the complete Safari 17.0 fingerprint
    pub fn fingerprint() -> BrowserFingerprint {
        BrowserFingerprint::new(
            "Safari",
            "17.0",
            tls_fingerprint(),
            http2_fingerprint(),
            headers(),
        )
    }

    /// Returns Safari 17.0 fingerprint with macOS-version-specific headers.
    pub fn fingerprint_with_os(_os: &str, macos_version: Option<&str>) -> BrowserFingerprint {
        BrowserFingerprint::new(
            "Safari",
            "17.0",
            tls_fingerprint(),
            http2_fingerprint(),
            super::safari_headers_for_os("17.0", macos_version),
        )
    }

    /// Safari 17.0 TLS fingerprint
    /// Slightly different from 18.0 - no ECH GREASE support
    fn tls_fingerprint() -> TlsFingerprint {
        TlsFingerprint::new(
            // Same cipher suite order as Safari 18.0
            vec![
                CipherSuite::TLS13_AES_128_GCM_SHA256,
                CipherSuite::TLS13_AES_256_GCM_SHA384,
                CipherSuite::TLS13_CHACHA20_POLY1305_SHA256,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
                CipherSuite::TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_CBC_SHA,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_CBC_SHA,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA,
                CipherSuite::TLS_RSA_WITH_AES_256_GCM_SHA384,
                CipherSuite::TLS_RSA_WITH_AES_128_GCM_SHA256,
                CipherSuite::TLS_RSA_WITH_AES_256_CBC_SHA,
                CipherSuite::TLS_RSA_WITH_AES_128_CBC_SHA,
            ],
            // Key exchange groups - same as 18.0
            vec![
                KeyExchangeGroup::X25519,
                KeyExchangeGroup::Secp256r1,
                KeyExchangeGroup::Secp384r1,
                KeyExchangeGroup::Secp521r1,
            ],
            // Signature algorithms
            vec![
                SignatureAlgorithm::EcdsaSecp256r1Sha256,
                SignatureAlgorithm::EcdsaSecp384r1Sha384,
                SignatureAlgorithm::EcdsaSecp521r1Sha512,
                SignatureAlgorithm::RsaPssRsaSha256,
                SignatureAlgorithm::RsaPssRsaSha384,
                SignatureAlgorithm::RsaPssRsaSha512,
                SignatureAlgorithm::RsaPkcs1Sha256,
                SignatureAlgorithm::RsaPkcs1Sha384,
                SignatureAlgorithm::RsaPkcs1Sha512,
                SignatureAlgorithm::EcdsaSha1Legacy,
                SignatureAlgorithm::RsaPkcs1Sha1,
            ],
            // TLS extensions - Safari 17 has GREASE and padding
            TlsExtensions::new(
                true,  // server_name
                true,  // status_request
                true,  // supported_groups
                true,  // signature_algorithms
                true,  // application_layer_protocol_negotiation
                false, // signed_certificate_timestamp
                true,  // key_share
                true,  // psk_key_exchange_modes
                true,  // supported_versions
                None,  // compress_certificate
                false, // application_settings
                false, // delegated_credentials
                None,  // record_size_limit
                vec![
                    ExtensionType::Grease(0xbaba),
                    ExtensionType::ServerName,
                    ExtensionType::ExtendedMasterSecret,
                    ExtensionType::RenegotiationInfo,
                    ExtensionType::SupportedGroups,
                    ExtensionType::EcPointFormats,
                    ExtensionType::ApplicationLayerProtocolNegotiation,
                    ExtensionType::StatusRequest,
                    ExtensionType::SignatureAlgorithms,
                    ExtensionType::KeyShare,
                    ExtensionType::PskKeyExchangeModes,
                    ExtensionType::SupportedVersions,
                    ExtensionType::Grease(0xbaba),
                    ExtensionType::Padding,
                ],
            )
            .with_session_ticket(false)
            .with_padding(true),
            // Safari 17.0 also uses ECH GREASE
            Some(EchConfig::new(
                EchMode::Grease {
                    hpke_suite: HpkeKemId::DhKemX25519HkdfSha256,
                },
                None,
            )),
            // ALPN protocols
            vec![b"h2".to_vec(), b"http/1.1".to_vec()],
        )
    }

    /// Safari 17.0 HTTP/2 fingerprint
    fn http2_fingerprint() -> Http2Fingerprint {
        Http2Fingerprint {
            pseudo_header_order: vec![
                ":method".to_string(),
                ":scheme".to_string(),
                ":path".to_string(),
                ":authority".to_string(),
            ],
            settings_header_table_size: Some(65536),
            settings_enable_push: Some(true),
            settings_max_concurrent_streams: Some(100),
            settings_initial_window_size: Some(4194304),
            settings_max_frame_size: Some(16384),
            settings_max_header_list_size: None,
            connection_window_size_increment: Some(10485760),
        }
    }

    /// Safari 17.0 HTTP headers (macOS Sonoma)
    fn headers() -> Vec<(String, String)> {
        vec![
            ("User-Agent".to_string(), "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15".to_string()),
            ("Accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".to_string()),
            ("Accept-Language".to_string(), "en-US,en;q=0.9".to_string()),
            ("Accept-Encoding".to_string(), "gzip, deflate, br".to_string()),
            ("sec-fetch-dest".to_string(), "document".to_string()),
            ("sec-fetch-mode".to_string(), "navigate".to_string()),
            ("sec-fetch-site".to_string(), "none".to_string()),
        ]
    }
}

/// Safari 17.2 iOS fingerprint module
pub mod safari_17_2_ios {
    use super::*;

    /// Returns the complete Safari 17.2 iOS fingerprint
    pub fn fingerprint() -> BrowserFingerprint {
        BrowserFingerprint::new(
            "Safari",
            "17.2",
            tls_fingerprint(),
            http2_fingerprint(),
            headers(),
        )
    }

    /// Returns Safari 17.2 iOS fingerprint with macOS-version-specific headers.
    /// Note: iOS Safari ignores the macos_version parameter and uses default iOS UA.
    pub fn fingerprint_with_os(_os: &str, _macos_version: Option<&str>) -> BrowserFingerprint {
        // iOS Safari always uses its own UA format
        fingerprint()
    }

    /// Safari 17.2 iOS TLS fingerprint
    /// iOS Safari has similar but not identical TLS to macOS Safari
    fn tls_fingerprint() -> TlsFingerprint {
        TlsFingerprint::new(
            // Same cipher suite order as macOS Safari
            vec![
                CipherSuite::TLS13_AES_128_GCM_SHA256,
                CipherSuite::TLS13_AES_256_GCM_SHA384,
                CipherSuite::TLS13_CHACHA20_POLY1305_SHA256,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
                CipherSuite::TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_CBC_SHA,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_CBC_SHA,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA,
                CipherSuite::TLS_RSA_WITH_AES_256_GCM_SHA384,
                CipherSuite::TLS_RSA_WITH_AES_128_GCM_SHA256,
                CipherSuite::TLS_RSA_WITH_AES_256_CBC_SHA,
                CipherSuite::TLS_RSA_WITH_AES_128_CBC_SHA,
            ],
            vec![
                KeyExchangeGroup::X25519,
                KeyExchangeGroup::Secp256r1,
                KeyExchangeGroup::Secp384r1,
                KeyExchangeGroup::Secp521r1,
            ],
            vec![
                SignatureAlgorithm::EcdsaSecp256r1Sha256,
                SignatureAlgorithm::EcdsaSecp384r1Sha384,
                SignatureAlgorithm::EcdsaSecp521r1Sha512,
                SignatureAlgorithm::RsaPssRsaSha256,
                SignatureAlgorithm::RsaPssRsaSha384,
                SignatureAlgorithm::RsaPssRsaSha512,
                SignatureAlgorithm::RsaPkcs1Sha256,
                SignatureAlgorithm::RsaPkcs1Sha384,
                SignatureAlgorithm::RsaPkcs1Sha512,
                SignatureAlgorithm::EcdsaSha1Legacy,
                SignatureAlgorithm::RsaPkcs1Sha1,
            ],
            TlsExtensions::new(
                true,  // server_name
                true,  // status_request
                true,  // supported_groups
                true,  // signature_algorithms
                true,  // application_layer_protocol_negotiation
                false, // signed_certificate_timestamp
                true,  // key_share
                true,  // psk_key_exchange_modes
                true,  // supported_versions
                None,  // compress_certificate
                false, // application_settings
                false, // delegated_credentials
                None,  // record_size_limit
                vec![
                    ExtensionType::Grease(0xbaba),
                    ExtensionType::ServerName,
                    ExtensionType::ExtendedMasterSecret,
                    ExtensionType::RenegotiationInfo,
                    ExtensionType::SupportedGroups,
                    ExtensionType::EcPointFormats,
                    ExtensionType::ApplicationLayerProtocolNegotiation,
                    ExtensionType::StatusRequest,
                    ExtensionType::SignatureAlgorithms,
                    ExtensionType::KeyShare,
                    ExtensionType::PskKeyExchangeModes,
                    ExtensionType::SupportedVersions,
                    ExtensionType::Grease(0xbaba),
                    ExtensionType::Padding,
                ],
            )
            .with_session_ticket(false)
            .with_padding(true),
            Some(EchConfig::new(
                EchMode::Grease {
                    hpke_suite: HpkeKemId::DhKemX25519HkdfSha256,
                },
                None,
            )),
            vec![b"h2".to_vec(), b"http/1.1".to_vec()],
        )
    }

    /// Safari 17.2 iOS HTTP/2 fingerprint
    fn http2_fingerprint() -> Http2Fingerprint {
        Http2Fingerprint {
            pseudo_header_order: vec![
                ":method".to_string(),
                ":scheme".to_string(),
                ":path".to_string(),
                ":authority".to_string(),
            ],
            settings_header_table_size: Some(65536),
            settings_enable_push: Some(true),
            settings_max_concurrent_streams: Some(100),
            settings_initial_window_size: Some(4194304),
            settings_max_frame_size: Some(16384),
            settings_max_header_list_size: None,
            connection_window_size_increment: Some(10485760),
        }
    }

    /// Safari 17.2 iOS HTTP headers
    fn headers() -> Vec<(String, String)> {
        vec![
            ("User-Agent".to_string(), "Mozilla/5.0 (iPhone; CPU iPhone OS 17_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Mobile/15E148 Safari/604.1".to_string()),
            ("Accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".to_string()),
            ("Accept-Language".to_string(), "en-US,en;q=0.9".to_string()),
            ("Accept-Encoding".to_string(), "gzip, deflate, br".to_string()),
            ("sec-fetch-dest".to_string(), "document".to_string()),
            ("sec-fetch-mode".to_string(), "navigate".to_string()),
            ("sec-fetch-site".to_string(), "none".to_string()),
        ]
    }
}
