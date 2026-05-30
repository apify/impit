//! iOS system TLS fingerprints
//!
//! On iOS, Apple's App Store policy forces every app (Safari, Chrome iOS,
//! Firefox iOS, Edge iOS, and any native app using `NSURLSession` /
//! `Network.framework` / `WKWebView`) to use the OS networking stack, so a
//! single iOS TLS profile transparently covers the entire iOS browser and
//! native-app ecosystem.
//!
//! Source: capture against <https://tls.peet.ws> from an iPhone running
//! iOS 18.7 (Safari 26.5). Verified identical to a Chrome iOS 148 capture on
//! the same iOS version (same JA3, JA4, peetprint, and Akamai HTTP/2
//! fingerprints), confirming the fingerprint is the OS stack rather than the
//! browser.

use crate::fingerprint::*;

/// iOS 18 system TLS fingerprint module
pub mod ios_18 {
    use super::*;

    pub fn fingerprint() -> BrowserFingerprint {
        BrowserFingerprint::new(
            "Safari",
            "iOS 18",
            tls_fingerprint(),
            http2_fingerprint(),
            headers(),
        )
    }

    fn tls_fingerprint() -> TlsFingerprint {
        TlsFingerprint::new(
            vec![
                CipherSuite::Grease,
                CipherSuite::TLS13_AES_256_GCM_SHA384,
                CipherSuite::TLS13_CHACHA20_POLY1305_SHA256,
                CipherSuite::TLS13_AES_128_GCM_SHA256,
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
                CipherSuite::TLS_ECDHE_ECDSA_WITH_3DES_EDE_CBC_SHA,
                CipherSuite::TLS_ECDHE_RSA_WITH_3DES_EDE_CBC_SHA,
                CipherSuite::TLS_RSA_WITH_3DES_EDE_CBC_SHA,
            ],
            vec![
                KeyExchangeGroup::Grease,
                KeyExchangeGroup::X25519MLKEM768,
                KeyExchangeGroup::X25519,
                KeyExchangeGroup::Secp256r1,
                KeyExchangeGroup::Secp384r1,
                KeyExchangeGroup::Secp521r1,
            ],
            // The duplicate RsaPssRsaSha384 entry is intentional — every
            // observed iOS capture sends this list with the duplicate at
            // indexes 4 and 5.
            vec![
                SignatureAlgorithm::EcdsaSecp256r1Sha256,
                SignatureAlgorithm::RsaPssRsaSha256,
                SignatureAlgorithm::RsaPkcs1Sha256,
                SignatureAlgorithm::EcdsaSecp384r1Sha384,
                SignatureAlgorithm::RsaPssRsaSha384,
                SignatureAlgorithm::RsaPssRsaSha384,
                SignatureAlgorithm::RsaPkcs1Sha384,
                SignatureAlgorithm::RsaPssRsaSha512,
                SignatureAlgorithm::RsaPkcs1Sha512,
                SignatureAlgorithm::RsaPkcs1Sha1,
            ],
            TlsExtensions::new(
                true,                                              // server_name
                true,                                              // status_request
                true,                                              // supported_groups
                true,                                              // signature_algorithms
                true,                                              // application_layer_protocol_negotiation
                true,                                              // signed_certificate_timestamp
                true,                                              // key_share
                true,                                              // psk_key_exchange_modes
                true,                                              // supported_versions
                Some(vec![CertificateCompressionAlgorithm::Zlib]), // compress_certificate
                false,                                             // application_settings
                false,                                             // delegated_credentials
                None,                                              // record_size_limit
                vec![
                    ExtensionType::Grease,
                    ExtensionType::ServerName,
                    ExtensionType::ExtendedMasterSecret,
                    ExtensionType::RenegotiationInfo,
                    ExtensionType::SupportedGroups,
                    ExtensionType::EcPointFormats,
                    ExtensionType::ApplicationLayerProtocolNegotiation,
                    ExtensionType::StatusRequest,
                    ExtensionType::SignatureAlgorithms,
                    ExtensionType::SignedCertificateTimestamp,
                    ExtensionType::KeyShare,
                    ExtensionType::PskKeyExchangeModes,
                    ExtensionType::SupportedVersions,
                    ExtensionType::CompressCertificate,
                    ExtensionType::Grease,
                ],
            )
            .with_session_ticket(false),
            None,
            vec![b"h2".to_vec(), b"http/1.1".to_vec()],
        )
    }

    fn http2_fingerprint() -> Http2Fingerprint {
        Http2Fingerprint {
            // iOS sends :method :scheme :authority :path on requests.
            // :protocol (extended CONNECT) and :status (response) are
            // required by the impit h2 fork to be in the order list even
            // when not used on a given message.
            pseudo_header_order: vec![
                ":method".to_string(),
                ":scheme".to_string(),
                ":authority".to_string(),
                ":path".to_string(),
                ":protocol".to_string(),
                ":status".to_string(),
            ],
            initial_stream_window_size: Some(2_097_152),
            // 65_535 (h2 default) + 10_420_225 WINDOW_UPDATE = 10_485_760.
            initial_connection_window_size: Some(10_485_760),
            max_header_list_size: None,
        }
    }

    fn headers() -> Vec<(String, String)> {
        vec![
            (
                "user-agent".to_string(),
                "Mozilla/5.0 (iPhone; CPU iPhone OS 18_7 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.5 Mobile/15E148 Safari/604.1".to_string(),
            ),
            (
                "accept".to_string(),
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".to_string(),
            ),
            ("sec-fetch-site".to_string(), "none".to_string()),
            ("sec-fetch-mode".to_string(), "navigate".to_string()),
            ("sec-fetch-dest".to_string(), "document".to_string()),
            ("accept-language".to_string(), "en-US,en;q=0.9".to_string()),
            ("priority".to_string(), "u=0, i".to_string()),
            (
                "accept-encoding".to_string(),
                "gzip, deflate, br, zstd".to_string(),
            ),
        ]
    }
}
