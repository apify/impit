//! Chrome browser fingerprints

use crate::fingerprint::*;

/// Trust Anchor IDs (draft-ietf-tls-trust-anchor-ids) that the Chrome 155+ profiles advertise: the roots from
/// Chrome's list, in Chrome's order.
///
/// Real Chrome 155 and 156 send 28 IDs (October 2026): these 13 roots from its Root Store, and 15 intermediates
/// that Chrome ships itself (Let's Encrypt E7-E9 and R12-R14; Google WR1-WR4, WE1-WE4 and AE1). The
/// intermediates are left out on purpose. A server may drop a listed intermediate from its chain, and impit
/// doesn't have them, so it couldn't verify that chain. All 13 roots are in Mozilla's set (`webpki-root-certs`),
/// which impit trusts on every platform. JA3, JA4 and JA4_r only see that the extension is present, so the
/// shorter list doesn't change them. If a root leaves Mozilla's set, take it out here.
const CHROME_TRUST_ANCHOR_ROOTS: &[&[u8]] = &[
    &[0x82, 0xdf, 0x13, 0x02, 0x01],                   // ISRG Root X1
    &[0x82, 0xdf, 0x13, 0x02, 0x06],                   // ISRG Root X2
    &[0x83, 0x9a, 0x64, 0x8c, 0x9b, 0x2d, 0x01, 0x07], // QuoVadis Root CA 2 G3
    &[0x83, 0x9a, 0x64, 0x8c, 0x9b, 0x2d, 0x01, 0x08], // QuoVadis Root CA 3 G3
    &[0x83, 0x9a, 0x64, 0x8c, 0x9b, 0x2d, 0x01, 0x09], // DigiCert Global Root G2
    &[0x83, 0x9a, 0x64, 0x8c, 0x9b, 0x2d, 0x01, 0x0a], // DigiCert Global Root G3
    &[0x83, 0x9a, 0x64, 0x8c, 0x9b, 0x2d, 0x01, 0x0b], // DigiCert Trusted Root G4
    &[0x83, 0x9a, 0x64, 0x8c, 0x9b, 0x2d, 0x01, 0x0c], // DigiCert Assured ID Root G2
    &[0x83, 0x9a, 0x64, 0x8c, 0x9b, 0x2d, 0x01, 0x0d], // DigiCert Assured ID Root G3
    &[0x83, 0x9a, 0x64, 0x8c, 0x9b, 0x2d, 0x01, 0x12], // DigiCert TLS RSA4096 Root G5
    &[0x83, 0x9a, 0x64, 0x8c, 0x9b, 0x2d, 0x01, 0x13], // DigiCert TLS ECC P384 Root G5
    &[0xd6, 0x79, 0x09, 0x01],                         // GTS Root R1
    &[0xd6, 0x79, 0x09, 0x04],                         // GTS Root R4
];

/// Chrome 155 fingerprint module
///
/// Source: consumer Chrome stable 155.0.8059.80 on Windows 11, captured against
/// <https://tls.peet.ws> and in a decrypted Wireshark capture (October 2026).
///
/// Relative to Chrome 151 there are two TLS differences, both in every ClientHello
/// since Chrome 152: the Trust Anchor IDs extension (`0xca34`), and a GREASE value
/// leading the signature_algorithms list. The extension lists only Chrome's roots,
/// not its intermediates (see `CHROME_TRUST_ANCHOR_ROOTS`). In the headers,
/// `sec-ch-ua` and the user agent carry 155, and `accept` gains `image/jxl`.
pub mod chrome_155 {
    use super::*;

    /// Returns the complete Chrome 155 fingerprint
    pub fn fingerprint() -> BrowserFingerprint {
        BrowserFingerprint::new(
            "Chrome",
            "155",
            tls_fingerprint(),
            http2_fingerprint(),
            headers(),
        )
    }

    /// Chrome 155 TLS fingerprint
    fn tls_fingerprint() -> TlsFingerprint {
        TlsFingerprint::new(
            // Cipher suites in Chrome 155 preference order
            // GREASE cipher at position 1 (first) - same as Chrome 151
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
            // Key exchange groups (includes post-quantum hybrid X25519MLKEM768)
            // GREASE at position 1 (first) - same as Chrome 151
            vec![
                KeyExchangeGroup::Grease,
                KeyExchangeGroup::X25519MLKEM768,
                KeyExchangeGroup::X25519,
                KeyExchangeGroup::Secp256r1,
                KeyExchangeGroup::Secp384r1,
            ],
            // Signature algorithms
            // Chrome leads with a GREASE value (real Chrome picks a new one per
            // connection; this one is fixed), then exactly Chrome 151's list.
            vec![
                SignatureAlgorithm::Grease,
                SignatureAlgorithm::MlDsa44,
                SignatureAlgorithm::MlDsa65,
                SignatureAlgorithm::MlDsa87,
                SignatureAlgorithm::EcdsaSecp256r1Sha256,
                SignatureAlgorithm::RsaPssRsaSha256,
                SignatureAlgorithm::RsaPkcs1Sha256,
                SignatureAlgorithm::EcdsaSecp384r1Sha384,
                SignatureAlgorithm::RsaPssRsaSha384,
                SignatureAlgorithm::RsaPkcs1Sha384,
                SignatureAlgorithm::RsaPssRsaSha512,
                SignatureAlgorithm::RsaPkcs1Sha512,
            ],
            // TLS extensions configuration
            // Chrome 155 uses new ALPS codepoint (17613)
            TlsExtensions::new(
                true,                                                // server_name
                true,                                                // status_request
                true,                                                // supported_groups
                true,                                                // signature_algorithms
                true, // application_layer_protocol_negotiation
                true, // signed_certificate_timestamp
                true, // key_share
                true, // psk_key_exchange_modes
                true, // supported_versions
                Some(vec![CertificateCompressionAlgorithm::Brotli]), // compress_certificate
                true, // application_settings
                false, // delegated_credentials (Chrome doesn't use)
                None, // record_size_limit (Chrome doesn't use)
                // Extension order (critical for fingerprinting), same as Chrome 151.
                // TrustAnchors isn't listed: like the other unlisted extensions, it
                // goes before these, in random order.
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
            .with_trust_anchors(
                CHROME_TRUST_ANCHOR_ROOTS
                    .iter()
                    .map(|id| id.to_vec())
                    .collect(),
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

    /// Chrome 155 HTTP/2 fingerprint, same as Chrome 151
    fn http2_fingerprint() -> Http2Fingerprint {
        Http2Fingerprint {
            pseudo_header_order: vec![
                ":method".to_string(),
                ":authority".to_string(),
                ":scheme".to_string(),
                ":path".to_string(),
                ":protocol".to_string(),
                ":status".to_string(),
            ],
            initial_stream_window_size: Some(6_291_456),
            // Chrome's connection window is 15 MiB. This field is the window size,
            // not the increment, so the emitted WINDOW_UPDATE is 15728640 - 65535 =
            // 15663105, which is what the capture shows on the wire.
            initial_connection_window_size: Some(15_728_640),
            max_header_list_size: Some(262_144),
            header_table_size: Some(65_536),
            max_concurrent_streams: None,
        }
    }

    /// Chrome 155 HTTP headers
    fn headers() -> Vec<(String, String)> {
        vec![
            ("sec-ch-ua".to_string(), "\"Google Chrome\";v=\"155\", \"Chromium\";v=\"155\", \"Not(A:Brand\";v=\"24\"".to_string()),
            ("sec-ch-ua-mobile".to_string(), "?0".to_string()),
            ("sec-ch-ua-platform".to_string(), "\"Windows\"".to_string()),
            ("upgrade-insecure-requests".to_string(), "1".to_string()),
            ("user-agent".to_string(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/155.0.0.0 Safari/537.36".to_string()),
            ("accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,image/jxl,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7".to_string()),
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

/// Chrome 151 fingerprint module
///
/// Source: capture against <https://tls.peet.ws> from consumer Chrome stable
/// 151.0.7922.72 on Windows 11, cross-checked against Chrome for Testing
/// 151.0.7922.71 on the same host. The two agree on everything except
/// `sec-ch-ua`, which Chrome for Testing cannot supply because it is unbranded;
/// that header is taken from the consumer build and is stable across launches.
///
/// Relative to Chrome 142 the only wire-level TLS difference is that Chrome now
/// leads its signature_algorithms list with the three ML-DSA codepoints.
pub mod chrome_151 {
    use super::*;

    /// Returns the complete Chrome 151 fingerprint
    pub fn fingerprint() -> BrowserFingerprint {
        BrowserFingerprint::new(
            "Chrome",
            "151",
            tls_fingerprint(),
            http2_fingerprint(),
            headers(),
        )
    }

    /// Chrome 151 TLS fingerprint
    fn tls_fingerprint() -> TlsFingerprint {
        TlsFingerprint::new(
            // Cipher suites in Chrome 151 preference order
            // GREASE cipher at position 1 (first) - same as Chrome 142
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
            // Key exchange groups (includes post-quantum hybrid X25519MLKEM768)
            // GREASE at position 1 (first) - same as Chrome 142
            vec![
                KeyExchangeGroup::Grease,
                KeyExchangeGroup::X25519MLKEM768,
                KeyExchangeGroup::X25519,
                KeyExchangeGroup::Secp256r1,
                KeyExchangeGroup::Secp384r1,
            ],
            // Signature algorithms
            // Chrome leads with the three ML-DSA codepoints (0x0904/0x0905/0x0906);
            // the remainder is unchanged from Chrome 142.
            vec![
                SignatureAlgorithm::MlDsa44,
                SignatureAlgorithm::MlDsa65,
                SignatureAlgorithm::MlDsa87,
                SignatureAlgorithm::EcdsaSecp256r1Sha256,
                SignatureAlgorithm::RsaPssRsaSha256,
                SignatureAlgorithm::RsaPkcs1Sha256,
                SignatureAlgorithm::EcdsaSecp384r1Sha384,
                SignatureAlgorithm::RsaPssRsaSha384,
                SignatureAlgorithm::RsaPkcs1Sha384,
                SignatureAlgorithm::RsaPssRsaSha512,
                SignatureAlgorithm::RsaPkcs1Sha512,
            ],
            // TLS extensions configuration
            // Chrome 151 uses new ALPS codepoint (17613)
            TlsExtensions::new(
                true,                                                // server_name
                true,                                                // status_request
                true,                                                // supported_groups
                true,                                                // signature_algorithms
                true, // application_layer_protocol_negotiation
                true, // signed_certificate_timestamp
                true, // key_share
                true, // psk_key_exchange_modes
                true, // supported_versions
                Some(vec![CertificateCompressionAlgorithm::Brotli]), // compress_certificate
                true, // application_settings
                false, // delegated_credentials (Chrome doesn't use)
                None, // record_size_limit (Chrome doesn't use)
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
            )
            .with_new_alps_codepoint(true),
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

    /// Chrome 151 HTTP/2 fingerprint
    fn http2_fingerprint() -> Http2Fingerprint {
        Http2Fingerprint {
            pseudo_header_order: vec![
                ":method".to_string(),
                ":authority".to_string(),
                ":scheme".to_string(),
                ":path".to_string(),
                ":protocol".to_string(),
                ":status".to_string(),
            ],
            initial_stream_window_size: Some(6_291_456),
            // Chrome's connection window is 15 MiB. This field is the window size,
            // not the increment, so the emitted WINDOW_UPDATE is 15728640 - 65535 =
            // 15663105, which is what the capture shows on the wire.
            initial_connection_window_size: Some(15_728_640),
            max_header_list_size: Some(262_144),
            header_table_size: Some(65_536),
            max_concurrent_streams: None,
        }
    }

    /// Chrome 151 HTTP headers
    fn headers() -> Vec<(String, String)> {
        vec![
            ("sec-ch-ua".to_string(), "\"Not=A?Brand\";v=\"99\", \"Google Chrome\";v=\"151\", \"Chromium\";v=\"151\"".to_string()),
            ("sec-ch-ua-mobile".to_string(), "?0".to_string()),
            ("sec-ch-ua-platform".to_string(), "\"Windows\"".to_string()),
            ("upgrade-insecure-requests".to_string(), "1".to_string()),
            ("user-agent".to_string(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/151.0.0.0 Safari/537.36".to_string()),
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

/// Chrome 142 fingerprint module
pub mod chrome_142 {
    use super::*;

    /// Returns the complete Chrome 142 fingerprint
    pub fn fingerprint() -> BrowserFingerprint {
        BrowserFingerprint::new(
            "Chrome",
            "142",
            tls_fingerprint(),
            http2_fingerprint(),
            headers(),
        )
    }

    /// Chrome 142 TLS fingerprint
    fn tls_fingerprint() -> TlsFingerprint {
        TlsFingerprint::new(
            // Cipher suites in Chrome 142 preference order
            // GREASE cipher at position 1 (first) - same as Chrome 136
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
            // Key exchange groups (includes post-quantum hybrid X25519MLKEM768)
            // GREASE at position 1 (first) - same as Chrome 136
            vec![
                KeyExchangeGroup::Grease,
                KeyExchangeGroup::X25519MLKEM768,
                KeyExchangeGroup::X25519,
                KeyExchangeGroup::Secp256r1,
                KeyExchangeGroup::Secp384r1,
            ],
            // Signature algorithms
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
            // TLS extensions configuration
            // Chrome 142 uses new ALPS codepoint (17613)
            TlsExtensions::new(
                true,                                                // server_name
                true,                                                // status_request
                true,                                                // supported_groups
                true,                                                // signature_algorithms
                true, // application_layer_protocol_negotiation
                true, // signed_certificate_timestamp
                true, // key_share
                true, // psk_key_exchange_modes
                true, // supported_versions
                Some(vec![CertificateCompressionAlgorithm::Brotli]), // compress_certificate
                true, // application_settings
                false, // delegated_credentials (Chrome doesn't use)
                None, // record_size_limit (Chrome doesn't use)
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
            )
            .with_new_alps_codepoint(true),
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

    /// Chrome 142 HTTP/2 fingerprint
    fn http2_fingerprint() -> Http2Fingerprint {
        Http2Fingerprint {
            pseudo_header_order: vec![
                ":method".to_string(),
                ":authority".to_string(),
                ":scheme".to_string(),
                ":path".to_string(),
                ":protocol".to_string(),
                ":status".to_string(),
            ],
            initial_stream_window_size: Some(6_291_456),
            initial_connection_window_size: Some(15_728_640),
            max_header_list_size: Some(262_144),
            header_table_size: Some(65_536),
            max_concurrent_streams: None,
        }
    }

    /// Chrome 142 HTTP headers
    fn headers() -> Vec<(String, String)> {
        vec![
            ("sec-ch-ua".to_string(), "\"Chromium\";v=\"142\", \"Google Chrome\";v=\"142\", \"Not_A Brand\";v=\"99\"".to_string()),
            ("sec-ch-ua-mobile".to_string(), "?0".to_string()),
            ("sec-ch-ua-platform".to_string(), "\"macOS\"".to_string()),
            ("upgrade-insecure-requests".to_string(), "1".to_string()),
            ("user-agent".to_string(), "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36".to_string()),
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

/// Chrome 136 fingerprint module
pub mod chrome_136 {
    use super::*;

    /// Returns the complete Chrome 136 fingerprint
    pub fn fingerprint() -> BrowserFingerprint {
        BrowserFingerprint::new(
            "Chrome",
            "136",
            tls_fingerprint(),
            http2_fingerprint(),
            headers(),
        )
    }

    /// Chrome 136 TLS fingerprint
    fn tls_fingerprint() -> TlsFingerprint {
        TlsFingerprint::new(
            // Cipher suites in Chrome 136 preference order
            // GREASE cipher at position 1 (first) based on Wireshark capture
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
            // Key exchange groups (includes post-quantum hybrid X25519MLKEM768)
            // GREASE at position 1 (first) based on Wireshark capture
            vec![
                KeyExchangeGroup::Grease,
                KeyExchangeGroup::X25519MLKEM768,
                KeyExchangeGroup::X25519,
                KeyExchangeGroup::Secp256r1,
                KeyExchangeGroup::Secp384r1,
            ],
            // Signature algorithms
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
            // TLS extensions configuration
            // Chrome 136 uses new ALPS codepoint (17613)
            TlsExtensions::new(
                true,                                                // server_name
                true,                                                // status_request
                true,                                                // supported_groups
                true,                                                // signature_algorithms
                true, // application_layer_protocol_negotiation
                true, // signed_certificate_timestamp
                true, // key_share
                true, // psk_key_exchange_modes
                true, // supported_versions
                Some(vec![CertificateCompressionAlgorithm::Brotli]), // compress_certificate
                true, // application_settings
                false, // delegated_credentials (Chrome doesn't use)
                None, // record_size_limit (Chrome doesn't use)
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
            )
            .with_new_alps_codepoint(true),
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

    /// Chrome 136 HTTP/2 fingerprint
    fn http2_fingerprint() -> Http2Fingerprint {
        Http2Fingerprint {
            pseudo_header_order: vec![
                ":method".to_string(),
                ":authority".to_string(),
                ":scheme".to_string(),
                ":path".to_string(),
                ":protocol".to_string(),
                ":status".to_string(),
            ],
            initial_stream_window_size: Some(6_291_456),
            initial_connection_window_size: Some(15_728_640),
            max_header_list_size: Some(262_144),
            header_table_size: Some(65_536),
            max_concurrent_streams: None,
        }
    }

    /// Chrome 136 HTTP headers
    fn headers() -> Vec<(String, String)> {
        vec![
            ("sec-ch-ua".to_string(), "\"Chromium\";v=\"136\", \"Google Chrome\";v=\"136\", \"Not.A/Brand\";v=\"99\"".to_string()),
            ("sec-ch-ua-mobile".to_string(), "?0".to_string()),
            ("sec-ch-ua-platform".to_string(), "\"macOS\"".to_string()),
            ("upgrade-insecure-requests".to_string(), "1".to_string()),
            ("user-agent".to_string(), "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36".to_string()),
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

/// Chrome 133 fingerprint module
pub mod chrome_133 {
    use super::*;

    /// Returns the complete Chrome 133 fingerprint
    pub fn fingerprint() -> BrowserFingerprint {
        BrowserFingerprint::new(
            "Chrome",
            "133",
            tls_fingerprint(),
            http2_fingerprint(),
            headers(),
        )
    }

    /// Chrome 133 TLS fingerprint
    fn tls_fingerprint() -> TlsFingerprint {
        TlsFingerprint::new(
            // Cipher suites in Chrome 133 preference order
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
            // Key exchange groups (includes post-quantum hybrid X25519MLKEM768)
            vec![
                KeyExchangeGroup::X25519MLKEM768,
                KeyExchangeGroup::X25519,
                KeyExchangeGroup::Secp256r1,
                KeyExchangeGroup::Secp384r1,
            ],
            // Signature algorithms
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
            // TLS extensions configuration
            TlsExtensions::new(
                true,                                                // server_name
                true,                                                // status_request
                true,                                                // supported_groups
                true,                                                // signature_algorithms
                true, // application_layer_protocol_negotiation
                true, // signed_certificate_timestamp
                true, // key_share
                true, // psk_key_exchange_modes
                true, // supported_versions
                Some(vec![CertificateCompressionAlgorithm::Brotli]), // compress_certificate
                true, // application_settings
                false, // delegated_credentials (Chrome doesn't use)
                None, // record_size_limit (Chrome doesn't use)
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

    /// Chrome 133 HTTP/2 fingerprint
    fn http2_fingerprint() -> Http2Fingerprint {
        Http2Fingerprint {
            pseudo_header_order: vec![
                ":method".to_string(),
                ":authority".to_string(),
                ":scheme".to_string(),
                ":path".to_string(),
                ":protocol".to_string(),
                ":status".to_string(),
            ],
            initial_stream_window_size: Some(6_291_456),
            initial_connection_window_size: Some(15_728_640),
            max_header_list_size: Some(262_144),
            header_table_size: Some(65_536),
            max_concurrent_streams: None,
        }
    }

    /// Chrome 133 HTTP headers
    fn headers() -> Vec<(String, String)> {
        vec![
            ("sec-ch-ua".to_string(), "\"Not(A:Brand\";v=\"99\", \"Google Chrome\";v=\"133\", \"Chromium\";v=\"133\"".to_string()),
            ("sec-ch-ua-mobile".to_string(), "?0".to_string()),
            ("sec-ch-ua-platform".to_string(), "\"macOS\"".to_string()),
            ("upgrade-insecure-requests".to_string(), "1".to_string()),
            ("user-agent".to_string(), "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36".to_string()),
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

/// Chrome 124 fingerprint module
pub mod chrome_124 {
    use super::*;

    /// Returns the complete Chrome 124 fingerprint
    pub fn fingerprint() -> BrowserFingerprint {
        BrowserFingerprint::new(
            "Chrome",
            "124",
            tls_fingerprint(),
            http2_fingerprint(),
            headers(),
        )
    }

    /// Chrome 124 TLS fingerprint
    fn tls_fingerprint() -> TlsFingerprint {
        TlsFingerprint::new(
            // Cipher suites in Chrome 124 preference order
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
            // Key exchange groups (X25519Kyber768Draft00 was the pre-MLKEM version)
            vec![
                KeyExchangeGroup::X25519MLKEM768, // Maps to X25519Kyber768Draft00 in practice
                KeyExchangeGroup::X25519,
                KeyExchangeGroup::Secp256r1,
                KeyExchangeGroup::Secp384r1,
            ],
            // Signature algorithms
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
            // TLS extensions configuration
            TlsExtensions::new(
                true,                                                // server_name
                true,                                                // status_request
                true,                                                // supported_groups
                true,                                                // signature_algorithms
                true, // application_layer_protocol_negotiation
                true, // signed_certificate_timestamp
                true, // key_share
                true, // psk_key_exchange_modes
                true, // supported_versions
                Some(vec![CertificateCompressionAlgorithm::Brotli]), // compress_certificate
                true, // application_settings
                false, // delegated_credentials (Chrome doesn't use)
                None, // record_size_limit (Chrome doesn't use)
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

    /// Chrome 124 HTTP/2 fingerprint
    fn http2_fingerprint() -> Http2Fingerprint {
        Http2Fingerprint {
            pseudo_header_order: vec![
                ":method".to_string(),
                ":authority".to_string(),
                ":scheme".to_string(),
                ":path".to_string(),
                ":protocol".to_string(),
                ":status".to_string(),
            ],
            initial_stream_window_size: Some(6_291_456),
            initial_connection_window_size: Some(15_728_640),
            max_header_list_size: Some(262_144),
            header_table_size: Some(65_536),
            max_concurrent_streams: None,
        }
    }

    /// Chrome 124 HTTP headers
    fn headers() -> Vec<(String, String)> {
        vec![
            ("sec-ch-ua".to_string(), "\"Chromium\";v=\"124\", \"Google Chrome\";v=\"124\", \"Not-A.Brand\";v=\"99\"".to_string()),
            ("sec-ch-ua-mobile".to_string(), "?0".to_string()),
            ("sec-ch-ua-platform".to_string(), "\"macOS\"".to_string()),
            ("upgrade-insecure-requests".to_string(), "1".to_string()),
            ("user-agent".to_string(), "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36".to_string()),
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

/// Chrome 131 fingerprint module
pub mod chrome_131 {
    use super::*;

    /// Returns the complete Chrome 131 fingerprint
    pub fn fingerprint() -> BrowserFingerprint {
        BrowserFingerprint::new(
            "Chrome",
            "131",
            tls_fingerprint(),
            http2_fingerprint(),
            headers(),
        )
    }

    /// Chrome 131 TLS fingerprint
    fn tls_fingerprint() -> TlsFingerprint {
        TlsFingerprint::new(
            // Cipher suites in Chrome 131 preference order (matching CHROME_CIPHER_SUITES)
            vec![
                CipherSuite::TLS13_AES_128_GCM_SHA256,
                CipherSuite::TLS13_AES_256_GCM_SHA384,
                CipherSuite::TLS13_CHACHA20_POLY1305_SHA256,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
                CipherSuite::Grease,
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
            // Key exchange groups (includes post-quantum hybrid X25519MLKEM768)
            vec![
                KeyExchangeGroup::X25519MLKEM768,
                KeyExchangeGroup::X25519,
                KeyExchangeGroup::Secp256r1,
                KeyExchangeGroup::Secp384r1,
                KeyExchangeGroup::Grease,
            ],
            // Signature algorithms - order must match DEFAULT_SIGNATURE_VERIFICATION_ALGOS
            // Note: No SHA1 algorithms for Chrome (matches original implementation)
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
            // TLS extensions configuration
            TlsExtensions::new(
                true,                                                // server_name
                true,                                                // status_request
                true,                                                // supported_groups
                true,                                                // signature_algorithms
                true, // application_layer_protocol_negotiation
                true, // signed_certificate_timestamp
                true, // key_share
                true, // psk_key_exchange_modes
                true, // supported_versions
                Some(vec![CertificateCompressionAlgorithm::Brotli]), // compress_certificate
                true, // application_settings
                false, // delegated_credentials (Chrome doesn't use)
                None, // record_size_limit (Chrome doesn't use)
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

    /// Chrome 131 HTTP/2 fingerprint
    fn http2_fingerprint() -> Http2Fingerprint {
        Http2Fingerprint {
            pseudo_header_order: vec![
                ":method".to_string(),
                ":authority".to_string(),
                ":scheme".to_string(),
                ":path".to_string(),
                ":protocol".to_string(),
                ":status".to_string(),
            ],
            initial_stream_window_size: Some(6_291_456),
            initial_connection_window_size: Some(15_728_640),
            max_header_list_size: Some(262_144),
            header_table_size: Some(65_536),
            max_concurrent_streams: None,
        }
    }

    /// Chrome 131 HTTP headers
    fn headers() -> Vec<(String, String)> {
        vec![
            ("sec-ch-ua".to_string(), "\"Google Chrome\";v=\"131\", \"Chromium\";v=\"131\", \"Not_A Brand\";v=\"24\"".to_string()),
            ("sec-ch-ua-mobile".to_string(), "?0".to_string()),
            ("sec-ch-ua-platform".to_string(), "\"macOS\"".to_string()),
            ("upgrade-insecure-requests".to_string(), "1".to_string()),
            ("user-agent".to_string(), "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36".to_string()),
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

/// Chrome 100 fingerprint module
pub mod chrome_100 {
    use super::*;

    /// Returns the complete Chrome 100 fingerprint
    pub fn fingerprint() -> BrowserFingerprint {
        BrowserFingerprint::new(
            "Chrome",
            "100",
            tls_fingerprint(),
            http2_fingerprint(),
            headers(),
        )
    }

    /// Chrome 100 TLS fingerprint
    pub(crate) fn tls_fingerprint() -> TlsFingerprint {
        TlsFingerprint::new(
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
            vec![
                KeyExchangeGroup::Grease,
                KeyExchangeGroup::X25519,
                KeyExchangeGroup::Secp256r1,
                KeyExchangeGroup::Secp384r1,
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
                true,
                true,
                true,
                true,
                true,
                true,
                true,
                true,
                true,
                Some(vec![CertificateCompressionAlgorithm::Brotli]),
                true,
                false,
                None,
                vec![
                    ExtensionType::Grease,
                    ExtensionType::ServerName,
                    ExtensionType::ExtendedMasterSecret,
                    ExtensionType::RenegotiationInfo,
                    ExtensionType::SupportedGroups,
                    ExtensionType::EcPointFormats,
                    ExtensionType::SessionTicket,
                    ExtensionType::ApplicationLayerProtocolNegotiation,
                    ExtensionType::StatusRequest,
                    ExtensionType::SignatureAlgorithms,
                    ExtensionType::SignedCertificateTimestamp,
                    ExtensionType::KeyShare,
                    ExtensionType::PskKeyExchangeModes,
                    ExtensionType::SupportedVersions,
                    ExtensionType::CompressCertificate,
                    ExtensionType::ApplicationSettings,
                    // No trailing GREASE: rustls has one GREASE extension type, and repeating it
                    // makes BoringSSL servers reject the ClientHello (#575).
                    ExtensionType::Padding,
                ],
            )
            .with_padding(true), // Chrome 100 uses padding extension
            // No ECH in Chrome 100
            None,
            // ALPN protocols
            vec![b"h2".to_vec(), b"http/1.1".to_vec()],
        )
    }

    /// Chrome 100 HTTP/2 fingerprint
    pub(crate) fn http2_fingerprint() -> Http2Fingerprint {
        Http2Fingerprint {
            pseudo_header_order: vec![
                ":method".to_string(),
                ":authority".to_string(),
                ":scheme".to_string(),
                ":path".to_string(),
                ":protocol".to_string(),
                ":status".to_string(),
            ],
            initial_stream_window_size: Some(6_291_456),
            initial_connection_window_size: Some(15_728_640),
            max_header_list_size: Some(262_144),
            header_table_size: Some(65_536),
            max_concurrent_streams: Some(1_000),
        }
    }

    /// Chrome 100 HTTP headers
    fn headers() -> Vec<(String, String)> {
        vec![
            ("sec-ch-ua".to_string(), "\" Not A;Brand\";v=\"99\", \"Chromium\";v=\"100\", \"Google Chrome\";v=\"100\"".to_string()),
            ("sec-ch-ua-mobile".to_string(), "?0".to_string()),
            ("sec-ch-ua-platform".to_string(), "\"Windows\"".to_string()),
            ("upgrade-insecure-requests".to_string(), "1".to_string()),
            ("user-agent".to_string(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.0.4896.75 Safari/537.36".to_string()),
            ("accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9".to_string()),
            ("sec-fetch-site".to_string(), "none".to_string()),
            ("sec-fetch-mode".to_string(), "navigate".to_string()),
            ("sec-fetch-user".to_string(), "?1".to_string()),
            ("sec-fetch-dest".to_string(), "document".to_string()),
            ("accept-encoding".to_string(), "gzip, deflate, br".to_string()),
            ("accept-language".to_string(), "en-US,en;q=0.9".to_string()),
        ]
    }
}

/// Chrome 101 fingerprint module
pub mod chrome_101 {
    use super::*;

    /// Returns the complete Chrome 101 fingerprint
    pub fn fingerprint() -> BrowserFingerprint {
        BrowserFingerprint::new(
            "Chrome",
            "101",
            chrome_100::tls_fingerprint(),
            chrome_100::http2_fingerprint(),
            headers(),
        )
    }

    fn headers() -> Vec<(String, String)> {
        vec![
            ("sec-ch-ua".to_string(), "\" Not A;Brand\";v=\"99\", \"Chromium\";v=\"101\", \"Google Chrome\";v=\"101\"".to_string()),
            ("sec-ch-ua-mobile".to_string(), "?0".to_string()),
            ("sec-ch-ua-platform".to_string(), "\"Windows\"".to_string()),
            ("upgrade-insecure-requests".to_string(), "1".to_string()),
            ("user-agent".to_string(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/101.0.4951.67 Safari/537.36".to_string()),
            ("accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9".to_string()),
            ("sec-fetch-site".to_string(), "none".to_string()),
            ("sec-fetch-mode".to_string(), "navigate".to_string()),
            ("sec-fetch-user".to_string(), "?1".to_string()),
            ("sec-fetch-dest".to_string(), "document".to_string()),
            ("accept-encoding".to_string(), "gzip, deflate, br".to_string()),
            ("accept-language".to_string(), "en-US,en;q=0.9".to_string()),
        ]
    }
}

/// Chrome 104 fingerprint module
pub mod chrome_104 {
    use super::*;

    /// Returns the complete Chrome 104 fingerprint
    pub fn fingerprint() -> BrowserFingerprint {
        BrowserFingerprint::new(
            "Chrome",
            "104",
            chrome_100::tls_fingerprint(),
            chrome_100::http2_fingerprint(),
            headers(),
        )
    }

    fn headers() -> Vec<(String, String)> {
        vec![
            ("sec-ch-ua".to_string(), "\" Not A;Brand\";v=\"99\", \"Chromium\";v=\"104\", \"Google Chrome\";v=\"104\"".to_string()),
            ("sec-ch-ua-mobile".to_string(), "?0".to_string()),
            ("sec-ch-ua-platform".to_string(), "\"Windows\"".to_string()),
            ("upgrade-insecure-requests".to_string(), "1".to_string()),
            ("user-agent".to_string(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36".to_string()),
            ("accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9".to_string()),
            ("sec-fetch-site".to_string(), "none".to_string()),
            ("sec-fetch-mode".to_string(), "navigate".to_string()),
            ("sec-fetch-user".to_string(), "?1".to_string()),
            ("sec-fetch-dest".to_string(), "document".to_string()),
            ("accept-encoding".to_string(), "gzip, deflate, br".to_string()),
            ("accept-language".to_string(), "en-US,en;q=0.9".to_string()),
        ]
    }
}

/// Chrome 107 fingerprint module
pub mod chrome_107 {
    use super::*;

    /// Returns the complete Chrome 107 fingerprint
    pub fn fingerprint() -> BrowserFingerprint {
        BrowserFingerprint::new(
            "Chrome",
            "107",
            chrome_100::tls_fingerprint(),
            chrome_100::http2_fingerprint(),
            headers(),
        )
    }

    fn headers() -> Vec<(String, String)> {
        vec![
            ("sec-ch-ua".to_string(), "\" Not A;Brand\";v=\"99\", \"Chromium\";v=\"107\", \"Google Chrome\";v=\"107\"".to_string()),
            ("sec-ch-ua-mobile".to_string(), "?0".to_string()),
            ("sec-ch-ua-platform".to_string(), "\"Windows\"".to_string()),
            ("upgrade-insecure-requests".to_string(), "1".to_string()),
            ("user-agent".to_string(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/107.0.0.0 Safari/537.36".to_string()),
            ("accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9".to_string()),
            ("sec-fetch-site".to_string(), "none".to_string()),
            ("sec-fetch-mode".to_string(), "navigate".to_string()),
            ("sec-fetch-user".to_string(), "?1".to_string()),
            ("sec-fetch-dest".to_string(), "document".to_string()),
            ("accept-encoding".to_string(), "gzip, deflate, br".to_string()),
            ("accept-language".to_string(), "en-US,en;q=0.9".to_string()),
        ]
    }
}

/// Chrome 110 fingerprint module
pub mod chrome_110 {
    use super::*;

    /// Returns the complete Chrome 110 fingerprint
    pub fn fingerprint() -> BrowserFingerprint {
        BrowserFingerprint::new(
            "Chrome",
            "110",
            chrome_100::tls_fingerprint(),
            chrome_100::http2_fingerprint(),
            headers(),
        )
    }

    fn headers() -> Vec<(String, String)> {
        vec![
            ("sec-ch-ua".to_string(), "\"Chromium\";v=\"110\", \"Not A(Brand\";v=\"24\", \"Google Chrome\";v=\"110\"".to_string()),
            ("sec-ch-ua-mobile".to_string(), "?0".to_string()),
            ("sec-ch-ua-platform".to_string(), "\"Windows\"".to_string()),
            ("upgrade-insecure-requests".to_string(), "1".to_string()),
            ("user-agent".to_string(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.0.0.0 Safari/537.36".to_string()),
            ("accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9".to_string()),
            ("sec-fetch-site".to_string(), "none".to_string()),
            ("sec-fetch-mode".to_string(), "navigate".to_string()),
            ("sec-fetch-user".to_string(), "?1".to_string()),
            ("sec-fetch-dest".to_string(), "document".to_string()),
            ("accept-encoding".to_string(), "gzip, deflate, br".to_string()),
            ("accept-language".to_string(), "en-US,en;q=0.9".to_string()),
        ]
    }
}

/// Chrome 116 fingerprint module
pub mod chrome_116 {
    use super::*;

    /// Returns the complete Chrome 116 fingerprint
    pub fn fingerprint() -> BrowserFingerprint {
        BrowserFingerprint::new(
            "Chrome",
            "116",
            chrome_100::tls_fingerprint(),
            chrome_100::http2_fingerprint(),
            headers(),
        )
    }

    fn headers() -> Vec<(String, String)> {
        vec![
            ("sec-ch-ua".to_string(), "\"Chromium\";v=\"116\", \"Not)A;Brand\";v=\"24\", \"Google Chrome\";v=\"116\"".to_string()),
            ("sec-ch-ua-mobile".to_string(), "?0".to_string()),
            ("sec-ch-ua-platform".to_string(), "\"Windows\"".to_string()),
            ("upgrade-insecure-requests".to_string(), "1".to_string()),
            ("user-agent".to_string(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36".to_string()),
            ("accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9".to_string()),
            ("sec-fetch-site".to_string(), "none".to_string()),
            ("sec-fetch-mode".to_string(), "navigate".to_string()),
            ("sec-fetch-user".to_string(), "?1".to_string()),
            ("sec-fetch-dest".to_string(), "document".to_string()),
            ("accept-encoding".to_string(), "gzip, deflate, br".to_string()),
            ("accept-language".to_string(), "en-US,en;q=0.9".to_string()),
        ]
    }
}

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
            // Cipher suites in Chrome 125 preference order (matching CHROME_CIPHER_SUITES)
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
            // Key exchange groups (GREASE at the end for Chrome fingerprint)
            vec![
                KeyExchangeGroup::X25519,
                KeyExchangeGroup::Secp256r1,
                KeyExchangeGroup::Secp384r1,
                KeyExchangeGroup::Grease,
            ],
            // Signature algorithms - order must match DEFAULT_SIGNATURE_VERIFICATION_ALGOS
            // Note: No SHA1 algorithms for Chrome (matches original implementation)
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
            // TLS extensions configuration
            TlsExtensions::new(
                true,                                                // server_name
                true,                                                // status_request
                true,                                                // supported_groups
                true,                                                // signature_algorithms
                true, // application_layer_protocol_negotiation
                true, // signed_certificate_timestamp
                true, // key_share
                true, // psk_key_exchange_modes
                true, // supported_versions
                Some(vec![CertificateCompressionAlgorithm::Brotli]), // compress_certificate
                true, // application_settings
                false, // delegated_credentials (Chrome doesn't use)
                None, // record_size_limit (Chrome doesn't use)
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
        Http2Fingerprint {
            pseudo_header_order: vec![
                ":method".to_string(),
                ":authority".to_string(),
                ":scheme".to_string(),
                ":path".to_string(),
                ":protocol".to_string(),
                ":status".to_string(),
            ],
            initial_stream_window_size: Some(6_291_456),
            initial_connection_window_size: Some(15_728_640),
            max_header_list_size: Some(262_144),
            header_table_size: Some(65_536),
            max_concurrent_streams: None,
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fingerprint::database::tests::client_hello;

    // JA4_r of real Chrome 155's cold TCP ClientHello: consumer Chrome 155.0.8059.80 on Windows 11 against
    // tls.peet.ws (October 2026).
    const JA4_R_155: &str = "t13d1517h2_002f,0035,009c,009d,1301,1302,1303,c013,c014,c02b,c02c,c02f,c030,cca8,cca9_0005,000a,000b,000d,0012,0017,001b,0023,002b,002d,0033,44cd,ca34,fe0d,ff01_0904,0905,0906,0403,0804,0401,0503,0805,0501,0806,0601";

    // The trust_anchors (0xca34) body for the 13 roots in CHROME_TRUST_ANCHOR_ROOTS (105 bytes): a u16 length,
    // then each ID as a u8 length and its bytes.
    const CHROME_ROOTS_PAYLOAD: &str = "00670582df1302010582df13020608839a648c9b2d010708839a648c9b2d010808839a648c9b2d010908839a648c9b2d010a08839a648c9b2d010b08839a648c9b2d010c08839a648c9b2d010d08839a648c9b2d011208839a648c9b2d011304d679090104d6790904";

    fn hex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }

    fn is_grease(value: u16) -> bool {
        value & 0x0f0f == 0x0a0a
    }

    /// Big-endian u16 values, without GREASE.
    fn u16s(bytes: &[u8]) -> Vec<u16> {
        bytes
            .chunks(2)
            .map(|c| u16::from_be_bytes([c[0], c[1]]))
            .filter(|v| !is_grease(*v))
            .collect()
    }

    fn hex_list(values: &[u16]) -> String {
        values
            .iter()
            .map(|v| format!("{v:04x}"))
            .collect::<Vec<_>>()
            .join(",")
    }

    /// FoxIO's JA4_r (the raw, unhashed JA4). GREASE values are dropped everywhere.
    fn ja4_r(version: u16, ciphers: &[u16], extensions: &[(u16, Vec<u8>)]) -> String {
        let body = |typ: u16| {
            extensions
                .iter()
                .find(|(t, _)| *t == typ)
                .map(|(_, body)| body.as_slice())
        };
        // supported_versions: a u8 length, then the versions. The highest one counts.
        let version = body(0x002b).map_or(version, |b| u16s(&b[1..]).into_iter().max().unwrap());
        let version = match version {
            0x0304 => "13",
            0x0303 => "12",
            0x0302 => "11",
            0x0301 => "10",
            v => panic!("unexpected TLS version {v:#06x}"),
        };
        let sni = if body(0x0000).is_some() { 'd' } else { 'i' };
        let mut ciphers: Vec<u16> = ciphers.iter().copied().filter(|v| !is_grease(*v)).collect();
        ciphers.sort_unstable();
        let mut types: Vec<u16> = extensions
            .iter()
            .map(|(t, _)| *t)
            .filter(|t| !is_grease(*t))
            .collect();
        let count = types.len();
        types.retain(|t| *t != 0x0000 && *t != 0x0010);
        types.sort_unstable();
        // ALPN: a u16 list length, then the first protocol's u8 length and bytes.
        let alpn = body(0x0010).map_or("00".to_string(), |b| {
            let first = &b[3..3 + usize::from(b[2])];
            format!("{}{}", first[0] as char, first[first.len() - 1] as char)
        });
        // signature_algorithms: a u16 length, then the schemes in wire order.
        let sigalgs = body(0x000d).map_or(Vec::new(), |b| u16s(&b[2..]));
        format!(
            "t{version}{sni}{:02}{count:02}{alpn}_{}_{}_{}",
            ciphers.len(),
            hex_list(&ciphers),
            hex_list(&types),
            hex_list(&sigalgs)
        )
    }

    #[test]
    fn chrome_profiles_send_real_chrome_ja4_r() {
        for (name, fp, expected) in [("chrome_155", chrome_155::fingerprint(), JA4_R_155)] {
            let (version, ciphers, exts) = client_hello(fp);
            assert_eq!(ja4_r(version, &ciphers, &exts), expected, "{name}");
            let body = |t| exts.iter().find(|(typ, _)| *typ == t).unwrap().1.clone();
            assert_eq!(body(0xca34), hex(CHROME_ROOTS_PAYLOAD), "{name}"); // the 105-byte encoding
            assert_eq!(
                body(0x000d)[2..4],
                [0x1a, 0x1a],
                "{name}: GREASE first in sigalgs"
            );
        }
    }
}
