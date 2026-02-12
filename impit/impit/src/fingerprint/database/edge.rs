//! Microsoft Edge browser fingerprints
//!
//! Edge is Chromium-based so the TLS fingerprint is identical to Chrome.
//! The key differences are in HTTP headers:
//! - `sec-ch-ua` includes "Microsoft Edge" brand instead of "Google Chrome"
//! - `User-Agent` includes "Edg/" suffix
//! - Different "Not A Brand" version strings

use crate::fingerprint::*;

/// Helper to create OS-specific Edge headers for a given version.
pub fn edge_headers_for_os(version: &str, os: &str) -> Vec<(String, String)> {
    let (ua_os, platform) = match os {
        "macos" => ("Macintosh; Intel Mac OS X 10_15_7", "\"macOS\""),
        "linux" => ("X11; Linux x86_64", "\"Linux\""),
        _ => ("Windows NT 10.0; Win64; x64", "\"Windows\""),
    };

    let sec_ch_ua = match version {
        "136" => "\"Microsoft Edge\";v=\"136\", \"Chromium\";v=\"136\", \"Not.A/Brand\";v=\"99\"".to_string(),
        "131" => "\"Microsoft Edge\";v=\"131\", \"Chromium\";v=\"131\", \"Not_A Brand\";v=\"24\"".to_string(),
        _ => format!("\"Microsoft Edge\";v=\"{version}\", \"Chromium\";v=\"{version}\", \"Not_A Brand\";v=\"99\""),
    };

    let ua = format!(
        "Mozilla/5.0 ({ua_os}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{version}.0.0.0 Safari/537.36 Edg/{version}.0.0.0"
    );

    vec![
        ("sec-ch-ua".to_string(), sec_ch_ua),
        ("sec-ch-ua-mobile".to_string(), "?0".to_string()),
        ("sec-ch-ua-platform".to_string(), platform.to_string()),
        ("upgrade-insecure-requests".to_string(), "1".to_string()),
        ("user-agent".to_string(), ua),
        ("accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7".to_string()),
        ("sec-fetch-site".to_string(), "none".to_string()),
        ("sec-fetch-mode".to_string(), "navigate".to_string()),
        ("sec-fetch-user".to_string(), "?1".to_string()),
        ("sec-fetch-dest".to_string(), "document".to_string()),
        ("accept-encoding".to_string(), "gzip, deflate, br, zstd".to_string()),
        ("accept-language".to_string(), "en-US,en;q=0.9".to_string()),
        ("priority".to_string(), "u=0, i".to_string()),
    ]
}

/// Edge 136 fingerprint module
/// Uses Chrome 136 TLS (same Chromium version) with Edge-specific headers
pub mod edge_136 {
    use super::*;

    /// Returns the complete Edge 136 fingerprint
    pub fn fingerprint() -> BrowserFingerprint {
        BrowserFingerprint::new(
            "Edge",
            "136",
            tls_fingerprint(),
            http2_fingerprint(),
            headers(),
        )
    }

    /// Returns Edge 136 fingerprint with OS-specific headers.
    pub fn fingerprint_with_os(os: &str) -> BrowserFingerprint {
        BrowserFingerprint::new(
            "Edge",
            "136",
            tls_fingerprint(),
            http2_fingerprint(),
            super::edge_headers_for_os("136", os),
        )
    }

    /// Edge 136 TLS fingerprint (identical to Chrome 136 - same Chromium engine)
    fn tls_fingerprint() -> TlsFingerprint {
        TlsFingerprint::new(
            // Same cipher suites as Chrome 136
            vec![
                CipherSuite::Grease,
                CipherSuite::TLS13_AES_128_GCM_SHA256,
                CipherSuite::TLS13_AES_256_GCM_SHA384,
                CipherSuite::TLS13_CHACHA20_POLY1305_SHA256,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256,
                CipherSuite::TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA,
                CipherSuite::TLS_RSA_WITH_AES_128_GCM_SHA256,
                CipherSuite::TLS_RSA_WITH_AES_256_GCM_SHA384,
                CipherSuite::TLS_RSA_WITH_AES_128_CBC_SHA,
                CipherSuite::TLS_RSA_WITH_AES_256_CBC_SHA,
            ],
            // Same key exchange groups as Chrome 136 (includes post-quantum)
            vec![
                KeyExchangeGroup::Grease,
                KeyExchangeGroup::X25519MLKEM768,
                KeyExchangeGroup::X25519,
                KeyExchangeGroup::Secp256r1,
                KeyExchangeGroup::Secp384r1,
            ],
            // Same signature algorithms as Chrome 136
            vec![
                SignatureAlgorithm::EcdsaSecp256r1Sha256,
                SignatureAlgorithm::RsaPssRsaSha256,
                SignatureAlgorithm::RsaPkcs1Sha256,
                SignatureAlgorithm::EcdsaSecp384r1Sha384,
                SignatureAlgorithm::RsaPssRsaSha384,
                SignatureAlgorithm::RsaPkcs1Sha384,
                SignatureAlgorithm::RsaPssRsaSha512,
                SignatureAlgorithm::RsaPkcs1Sha512,
            ],
            // Same extensions as Chrome 136 (with new ALPS codepoint)
            TlsExtensions::new(
                true,                                                // server_name
                true,                                                // status_request
                true,                                                // supported_groups
                true,                                                // signature_algorithms
                true,  // application_layer_protocol_negotiation
                true,  // signed_certificate_timestamp
                true,  // key_share
                true,  // psk_key_exchange_modes
                true,  // supported_versions
                Some(vec![CertificateCompressionAlgorithm::Brotli]), // compress_certificate
                true,  // application_settings
                false, // delegated_credentials
                None,  // record_size_limit
                // Same extension order as Chrome 136
                vec![
                    ExtensionType::ServerName,
                    ExtensionType::ExtendedMasterSecret,
                    ExtensionType::SessionTicket,
                    ExtensionType::SignatureAlgorithms,
                    ExtensionType::StatusRequest,
                    ExtensionType::SupportedGroups,
                    ExtensionType::ApplicationLayerProtocolNegotiation,
                    ExtensionType::SignedCertificateTimestamp,
                    ExtensionType::KeyShare,
                    ExtensionType::PskKeyExchangeModes,
                    ExtensionType::SupportedVersions,
                    ExtensionType::CompressCertificate,
                    ExtensionType::ApplicationSettings,
                ],
            )
            .with_new_alps_codepoint(true)
            .with_permute_extensions(true), // Chromium-based: randomizes extension order
            // ECH configuration (GREASE mode)
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

    /// Edge 136 HTTP/2 fingerprint (same as Chrome)
    fn http2_fingerprint() -> Http2Fingerprint {
        Http2Fingerprint {
            pseudo_header_order: vec![
                ":method".to_string(),
                ":authority".to_string(),
                ":scheme".to_string(),
                ":path".to_string(),
            ],
            settings_header_table_size: Some(65536),
            settings_enable_push: Some(false),
            settings_max_concurrent_streams: Some(1000),
            settings_initial_window_size: Some(6291456),
            settings_max_frame_size: None,
            settings_max_header_list_size: Some(262144),
            connection_window_size_increment: Some(15663105),
        }
    }

    /// Edge 136 HTTP headers - key difference: sec-ch-ua and User-Agent identify as Edge
    fn headers() -> Vec<(String, String)> {
        vec![
            ("sec-ch-ua".to_string(), "\"Microsoft Edge\";v=\"136\", \"Chromium\";v=\"136\", \"Not.A/Brand\";v=\"99\"".to_string()),
            ("sec-ch-ua-mobile".to_string(), "?0".to_string()),
            ("sec-ch-ua-platform".to_string(), "\"Windows\"".to_string()),
            ("upgrade-insecure-requests".to_string(), "1".to_string()),
            ("user-agent".to_string(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36 Edg/136.0.0.0".to_string()),
            ("accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7".to_string()),
            ("sec-fetch-site".to_string(), "none".to_string()),
            ("sec-fetch-mode".to_string(), "navigate".to_string()),
            ("sec-fetch-user".to_string(), "?1".to_string()),
            ("sec-fetch-dest".to_string(), "document".to_string()),
            ("accept-encoding".to_string(), "gzip, deflate, br, zstd".to_string()),
            ("accept-language".to_string(), "en-US,en;q=0.9".to_string()),
            ("priority".to_string(), "u=0, i".to_string()),
        ]
    }
}

/// Edge 131 fingerprint module
pub mod edge_131 {
    use super::*;

    /// Returns the complete Edge 131 fingerprint
    pub fn fingerprint() -> BrowserFingerprint {
        BrowserFingerprint::new(
            "Edge",
            "131",
            tls_fingerprint(),
            http2_fingerprint(),
            headers(),
        )
    }

    /// Returns Edge 131 fingerprint with OS-specific headers.
    pub fn fingerprint_with_os(os: &str) -> BrowserFingerprint {
        BrowserFingerprint::new(
            "Edge",
            "131",
            tls_fingerprint(),
            http2_fingerprint(),
            super::edge_headers_for_os("131", os),
        )
    }

    /// Edge 131 TLS fingerprint (based on Chrome 131 Chromium engine)
    fn tls_fingerprint() -> TlsFingerprint {
        TlsFingerprint::new(
            vec![
                CipherSuite::TLS13_AES_128_GCM_SHA256,
                CipherSuite::TLS13_AES_256_GCM_SHA384,
                CipherSuite::TLS13_CHACHA20_POLY1305_SHA256,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256,
                CipherSuite::TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA,
                CipherSuite::TLS_RSA_WITH_AES_128_GCM_SHA256,
                CipherSuite::TLS_RSA_WITH_AES_256_GCM_SHA384,
                CipherSuite::TLS_RSA_WITH_AES_128_CBC_SHA,
                CipherSuite::TLS_RSA_WITH_AES_256_CBC_SHA,
                CipherSuite::Grease,
            ],
            vec![
                KeyExchangeGroup::X25519MLKEM768,
                KeyExchangeGroup::X25519,
                KeyExchangeGroup::Secp256r1,
                KeyExchangeGroup::Secp384r1,
                KeyExchangeGroup::Grease,
            ],
            vec![
                SignatureAlgorithm::EcdsaSecp256r1Sha256,
                SignatureAlgorithm::RsaPssRsaSha256,
                SignatureAlgorithm::RsaPkcs1Sha256,
                SignatureAlgorithm::EcdsaSecp384r1Sha384,
                SignatureAlgorithm::RsaPssRsaSha384,
                SignatureAlgorithm::RsaPkcs1Sha384,
                SignatureAlgorithm::RsaPssRsaSha512,
                SignatureAlgorithm::RsaPkcs1Sha512,
            ],
            TlsExtensions::new(
                true,                                                // server_name
                true,                                                // status_request
                true,                                                // supported_groups
                true,                                                // signature_algorithms
                true,  // application_layer_protocol_negotiation
                true,  // signed_certificate_timestamp
                true,  // key_share
                true,  // psk_key_exchange_modes
                true,  // supported_versions
                Some(vec![CertificateCompressionAlgorithm::Brotli]), // compress_certificate
                true,  // application_settings
                false, // delegated_credentials
                None,  // record_size_limit
                vec![
                    ExtensionType::ServerName,
                    ExtensionType::ExtendedMasterSecret,
                    ExtensionType::SessionTicket,
                    ExtensionType::SignatureAlgorithms,
                    ExtensionType::StatusRequest,
                    ExtensionType::SupportedGroups,
                    ExtensionType::ApplicationLayerProtocolNegotiation,
                    ExtensionType::SignedCertificateTimestamp,
                    ExtensionType::KeyShare,
                    ExtensionType::PskKeyExchangeModes,
                    ExtensionType::SupportedVersions,
                    ExtensionType::CompressCertificate,
                    ExtensionType::ApplicationSettings,
                ],
            )
            .with_permute_extensions(true), // Chromium-based: randomizes extension order
            Some(EchConfig::new(
                EchMode::Grease {
                    hpke_suite: HpkeKemId::DhKemX25519HkdfSha256,
                },
                None,
            )),
            vec![b"h2".to_vec(), b"http/1.1".to_vec()],
        )
    }

    /// Edge 131 HTTP/2 fingerprint
    fn http2_fingerprint() -> Http2Fingerprint {
        Http2Fingerprint {
            pseudo_header_order: vec![
                ":method".to_string(),
                ":authority".to_string(),
                ":scheme".to_string(),
                ":path".to_string(),
            ],
            settings_header_table_size: Some(65536),
            settings_enable_push: Some(false),
            settings_max_concurrent_streams: Some(1000),
            settings_initial_window_size: Some(6291456),
            settings_max_frame_size: None,
            settings_max_header_list_size: Some(262144),
            connection_window_size_increment: Some(15663105),
        }
    }

    /// Edge 131 HTTP headers
    fn headers() -> Vec<(String, String)> {
        vec![
            ("sec-ch-ua".to_string(), "\"Microsoft Edge\";v=\"131\", \"Chromium\";v=\"131\", \"Not_A Brand\";v=\"24\"".to_string()),
            ("sec-ch-ua-mobile".to_string(), "?0".to_string()),
            ("sec-ch-ua-platform".to_string(), "\"Windows\"".to_string()),
            ("upgrade-insecure-requests".to_string(), "1".to_string()),
            ("user-agent".to_string(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36 Edg/131.0.0.0".to_string()),
            ("accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7".to_string()),
            ("sec-fetch-site".to_string(), "none".to_string()),
            ("sec-fetch-mode".to_string(), "navigate".to_string()),
            ("sec-fetch-user".to_string(), "?1".to_string()),
            ("sec-fetch-dest".to_string(), "document".to_string()),
            ("accept-encoding".to_string(), "gzip, deflate, br, zstd".to_string()),
            ("accept-language".to_string(), "en-US,en;q=0.9".to_string()),
            ("priority".to_string(), "u=0, i".to_string()),
        ]
    }
}
