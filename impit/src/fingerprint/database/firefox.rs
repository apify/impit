//! Firefox browser fingerprints

use crate::fingerprint::*;

/// Firefox 128 fingerprint module
pub mod firefox_128 {
    use super::*;

    /// Returns the complete Firefox 128 fingerprint
    pub fn fingerprint() -> BrowserFingerprint {
        BrowserFingerprint::new(
            "Firefox",
            "128",
            tls_fingerprint(),
            http2_fingerprint(),
            headers(),
        )
    }

    /// Firefox 128 TLS fingerprint
    fn tls_fingerprint() -> TlsFingerprint {
        TlsFingerprint::new(
            // Cipher suites in Firefox 128 preference order (17 suites)
            // TLS 1.3 cipher suites first (including fake ones for fingerprinting), then TLS 1.2
            vec![
                // Real TLS 1.3 cipher suites
                CipherSuite::TLS13_AES_128_GCM_SHA256,
                CipherSuite::TLS13_CHACHA20_POLY1305_SHA256,
                CipherSuite::TLS13_AES_256_GCM_SHA384,
                // Fake cipher suites for TLS 1.3 fingerprinting (advertised but not used)
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_CBC_SHA,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_CBC_SHA,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA,
                CipherSuite::TLS_RSA_WITH_AES_128_GCM_SHA256,
                CipherSuite::TLS_RSA_WITH_AES_256_GCM_SHA384,
                CipherSuite::TLS_RSA_WITH_AES_128_CBC_SHA,
                CipherSuite::TLS_RSA_WITH_AES_256_CBC_SHA,
                // Real TLS 1.2 cipher suites
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256,
                CipherSuite::TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
            ],
            // Key exchange groups (Firefox includes FFDHE groups)
            vec![
                KeyExchangeGroup::X25519,
                KeyExchangeGroup::Secp256r1,
                KeyExchangeGroup::Secp384r1,
                KeyExchangeGroup::Ffdhe2048,
                KeyExchangeGroup::Ffdhe3072,
            ],
            // Signature algorithms - order must match FIREFOX_SIGNATURE_VERIFICATION_ALGOS mapping
            // Note: Ed25519 is included in verification but not in the ClientHello extension
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
            TlsExtensions::new(
                true,        // server_name
                true,        // status_request
                true,        // supported_groups
                true,        // signature_algorithms
                true,        // application_layer_protocol_negotiation
                false,       // signed_certificate_timestamp (Firefox doesn't use this, only Chrome)
                true,        // key_share
                true,        // psk_key_exchange_modes
                true,        // supported_versions
                None,        // compress_certificate (Firefox doesn't use this)
                false,       // application_settings
                true,        // delegated_credentials (Firefox uses this)
                Some(16385), // record_size_limit (Firefox uses this)
                // Extension order (critical for fingerprinting)
                // Note: Firefox doesn't send SignedCertificateTimestamp
                vec![
                    ExtensionType::ServerName,
                    ExtensionType::ExtendedMasterSecret,
                    ExtensionType::SessionTicket,
                    ExtensionType::SignatureAlgorithms,
                    ExtensionType::StatusRequest,
                    ExtensionType::SupportedGroups,
                    ExtensionType::ApplicationLayerProtocolNegotiation,
                    ExtensionType::KeyShare,
                    ExtensionType::PskKeyExchangeModes,
                    ExtensionType::SupportedVersions,
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

    /// Firefox 128 HTTP/2 fingerprint
    fn http2_fingerprint() -> Http2Fingerprint {
        Http2Fingerprint {
            pseudo_header_order: vec![
                ":method".to_string(),
                ":path".to_string(),
                ":authority".to_string(),
                ":scheme".to_string(),
            ]
        }
    }

    /// Firefox 128 HTTP headers
    fn headers() -> Vec<(String, String)> {
        vec![
            ("User-Agent".to_string(), "Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0".to_string()),
            ("Accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/png,image/svg+xml,*/*;q=0.8".to_string()),
            ("Accept-Language".to_string(), "en,cs;q=0.7,en-US;q=0.3".to_string()),
            ("Accept-Encoding".to_string(), "gzip, deflate, br, zstd".to_string()),
            ("sec-fetch-dest".to_string(), "document".to_string()),
            ("sec-fetch-mode".to_string(), "navigate".to_string()),
            ("sec-fetch-site".to_string(), "none".to_string()),
            ("sec-fetch-user".to_string(), "?1".to_string()),
            ("Upgrade-Insecure-Requests".to_string(), "1".to_string()),
            ("Priority".to_string(), "u=0, i".to_string()),
        ]
    }
}
