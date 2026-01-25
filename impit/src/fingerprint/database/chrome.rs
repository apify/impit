//! Chrome browser fingerprints

use crate::fingerprint::*;

/// Chrome 125 fingerprint module
pub mod chrome_125 {
    use super::*;

    /// Returns the complete Chrome 125 fingerprint
    pub fn fingerprint() -> BrowserFingerprint {
        BrowserFingerprint::new(
            "Chrome",
            "125",
            tls_fingerprint(),
            http2_fingerprint(),
            headers(),
        )
    }

    /// Chrome 125 TLS fingerprint
    fn tls_fingerprint() -> TlsFingerprint {
        TlsFingerprint::new(
            // Cipher suites in Chrome 125 preference order
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
            ],
            // Key exchange groups
            vec![
                KeyExchangeGroup::X25519,
                KeyExchangeGroup::Secp256r1,
                KeyExchangeGroup::Secp384r1,
            ],
            // Signature algorithms
            vec![
                SignatureAlgorithm::EcdsaSecp256r1Sha256,
                SignatureAlgorithm::RsaPssRsaSha256,
                SignatureAlgorithm::RsaPssRsaSha384,
                SignatureAlgorithm::RsaPssRsaSha512,
                SignatureAlgorithm::RsaPkcs1Sha256,
                SignatureAlgorithm::RsaPkcs1Sha384,
                SignatureAlgorithm::RsaPkcs1Sha512,
                SignatureAlgorithm::EcdsaSecp384r1Sha384,
                SignatureAlgorithm::EcdsaSecp521r1Sha512,
                SignatureAlgorithm::RsaPkcs1Sha1,
            ],
            // TLS extensions configuration
            TlsExtensions::new(
                true,  // server_name
                true,  // status_request
                true,  // supported_groups
                true,  // signature_algorithms
                true,  // application_layer_protocol_negotiation
                true,  // signed_certificate_timestamp
                true,  // key_share
                true,  // psk_key_exchange_modes
                true,  // supported_versions
                Some(vec![CertificateCompressionAlgorithm::Brotli]),  // compress_certificate
                true,  // application_settings
                // Extension order (critical for fingerprinting)
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
            ),
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

    /// Chrome 125 HTTP/2 fingerprint
    fn http2_fingerprint() -> Http2Fingerprint {
        Http2Fingerprint::new(
            // Pseudo-header ordering
            vec![
                ":method".to_string(),
                ":authority".to_string(),
                ":scheme".to_string(),
                ":path".to_string(),
            ],
            // SETTINGS frame values
            Http2Settings::new(
                Some(65536),   // header_table_size
                Some(false),   // enable_push
                Some(1000),    // max_concurrent_streams
                Some(6291456), // initial_window_size
                Some(16384),   // max_frame_size
                Some(262144),  // max_header_list_size
                vec![],        // custom settings
            ),
            // Window sizes
            Http2WindowSize::new(
                15728640,  // connection_window_size
                6291456,   // stream_window_size
            ),
            // Priority
            Some(Http2Priority::new(255, 0, false)),
        )
    }

    /// Chrome 125 HTTP headers
    fn headers() -> Vec<(String, String)> {
        vec![
            ("sec-ch-ua".to_string(), "\"Google Chrome\";v=\"125\", \"Chromium\";v=\"125\", \"Not.A/Brand\";v=\"24\"".to_string()),
            ("sec-ch-ua-mobile".to_string(), "?0".to_string()),
            ("sec-ch-ua-platform".to_string(), "\"Linux\"".to_string()),
            ("upgrade-insecure-requests".to_string(), "1".to_string()),
            ("user-agent".to_string(), "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36".to_string()),
            ("accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7".to_string()),
            ("sec-fetch-site".to_string(), "none".to_string()),
            ("sec-fetch-mode".to_string(), "navigate".to_string()),
            ("sec-fetch-user".to_string(), "?1".to_string()),
            ("sec-fetch-dest".to_string(), "document".to_string()),
            ("accept-encoding".to_string(), "gzip, deflate, br, zstd".to_string()),
            ("accept-language".to_string(), "en-US,en;q=0.9".to_string()),
        ]
    }
}
