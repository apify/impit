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
            // Cipher suites in real Firefox 128's order (17 suites)
            vec![
                CipherSuite::TLS13_AES_128_GCM_SHA256,
                CipherSuite::TLS13_CHACHA20_POLY1305_SHA256,
                CipherSuite::TLS13_AES_256_GCM_SHA384,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256,
                CipherSuite::TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_CBC_SHA,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_CBC_SHA,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA,
                CipherSuite::TLS_RSA_WITH_AES_128_GCM_SHA256,
                CipherSuite::TLS_RSA_WITH_AES_256_GCM_SHA384,
                CipherSuite::TLS_RSA_WITH_AES_128_CBC_SHA,
                CipherSuite::TLS_RSA_WITH_AES_256_CBC_SHA,
            ],
            // Key exchange groups (Firefox includes FFDHE groups)
            vec![
                KeyExchangeGroup::X25519,
                KeyExchangeGroup::Secp256r1,
                KeyExchangeGroup::Secp384r1,
                KeyExchangeGroup::Secp521r1,
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
                    ExtensionType::RenegotiationInfo,
                    ExtensionType::SupportedGroups,
                    ExtensionType::EcPointFormats,
                    ExtensionType::SessionTicket,
                    ExtensionType::ApplicationLayerProtocolNegotiation,
                    ExtensionType::StatusRequest,
                    ExtensionType::DelegatedCredentials,
                    ExtensionType::KeyShare,
                    ExtensionType::SupportedVersions,
                    ExtensionType::SignatureAlgorithms,
                    ExtensionType::PskKeyExchangeModes,
                    ExtensionType::RecordSizeLimit,
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
                ":protocol".to_string(),
                ":status".to_string(),
            ],
            initial_stream_window_size: Some(131_072),
            initial_connection_window_size: Some(12_582_912),
            max_header_list_size: None,
            header_table_size: Some(65_536),
            max_concurrent_streams: None,
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

/// Firefox 133 fingerprint module
pub mod firefox_133 {
    use super::*;

    /// Returns the complete Firefox 133 fingerprint
    pub fn fingerprint() -> BrowserFingerprint {
        BrowserFingerprint::new(
            "Firefox",
            "133",
            tls_fingerprint(),
            http2_fingerprint(),
            headers(),
        )
    }

    /// Firefox 133 TLS fingerprint
    fn tls_fingerprint() -> TlsFingerprint {
        TlsFingerprint::new(
            // Cipher suites in Firefox 133 preference order
            vec![
                CipherSuite::TLS13_AES_128_GCM_SHA256,
                CipherSuite::TLS13_CHACHA20_POLY1305_SHA256,
                CipherSuite::TLS13_AES_256_GCM_SHA384,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256,
                CipherSuite::TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_CBC_SHA,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_CBC_SHA,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA,
                CipherSuite::TLS_RSA_WITH_AES_128_GCM_SHA256,
                CipherSuite::TLS_RSA_WITH_AES_256_GCM_SHA384,
                CipherSuite::TLS_RSA_WITH_AES_128_CBC_SHA,
                CipherSuite::TLS_RSA_WITH_AES_256_CBC_SHA,
            ],
            // Key exchange groups (Firefox includes FFDHE groups and post-quantum hybrid)
            vec![
                KeyExchangeGroup::X25519MLKEM768,
                KeyExchangeGroup::X25519,
                KeyExchangeGroup::Secp256r1,
                KeyExchangeGroup::Secp384r1,
                KeyExchangeGroup::Secp521r1,
                KeyExchangeGroup::Ffdhe2048,
                KeyExchangeGroup::Ffdhe3072,
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
            // TLS extensions configuration
            TlsExtensions::new(
                true,  // server_name
                true,  // status_request
                true,  // supported_groups
                true,  // signature_algorithms
                true,  // application_layer_protocol_negotiation
                false, // signed_certificate_timestamp (Firefox 133 doesn't use)
                true,  // key_share
                true,  // psk_key_exchange_modes
                true,  // supported_versions
                Some(vec![
                    CertificateCompressionAlgorithm::Zlib,
                    CertificateCompressionAlgorithm::Brotli,
                    CertificateCompressionAlgorithm::Zstd,
                ]), // compress_certificate (all three algorithms)
                false, // application_settings
                true,  // delegated_credentials (Firefox uses this)
                Some(16385), // record_size_limit (0x4001)
                // Extension order (critical for fingerprinting)
                vec![
                    ExtensionType::ServerName,
                    ExtensionType::ExtendedMasterSecret,
                    ExtensionType::RenegotiationInfo,
                    ExtensionType::SupportedGroups,
                    ExtensionType::EcPointFormats,
                    ExtensionType::SessionTicket,
                    ExtensionType::ApplicationLayerProtocolNegotiation,
                    ExtensionType::StatusRequest,
                    ExtensionType::DelegatedCredentials,
                    ExtensionType::KeyShare,
                    ExtensionType::SupportedVersions,
                    ExtensionType::SignatureAlgorithms,
                    ExtensionType::PskKeyExchangeModes,
                    ExtensionType::RecordSizeLimit,
                    ExtensionType::CompressCertificate,
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

    /// Firefox 133 HTTP/2 fingerprint
    fn http2_fingerprint() -> Http2Fingerprint {
        Http2Fingerprint {
            pseudo_header_order: vec![
                ":method".to_string(),
                ":path".to_string(),
                ":authority".to_string(),
                ":scheme".to_string(),
                ":protocol".to_string(),
                ":status".to_string(),
            ],
            initial_stream_window_size: Some(131_072),
            initial_connection_window_size: Some(12_582_912),
            max_header_list_size: None,
            header_table_size: Some(65_536),
            max_concurrent_streams: None,
        }
    }

    /// Firefox 133 HTTP headers
    fn headers() -> Vec<(String, String)> {
        vec![
            ("user-agent".to_string(), "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:133.0) Gecko/20100101 Firefox/133.0".to_string()),
            ("accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".to_string()),
            ("accept-language".to_string(), "en-US,en;q=0.5".to_string()),
            ("accept-encoding".to_string(), "gzip, deflate, br, zstd".to_string()),
            ("upgrade-insecure-requests".to_string(), "1".to_string()),
            ("sec-fetch-dest".to_string(), "document".to_string()),
            ("sec-fetch-mode".to_string(), "navigate".to_string()),
            ("sec-fetch-site".to_string(), "none".to_string()),
            ("sec-fetch-user".to_string(), "?1".to_string()),
            ("priority".to_string(), "u=0, i".to_string()),
            ("te".to_string(), "trailers".to_string()),
        ]
    }
}

/// Firefox 135 fingerprint module
pub mod firefox_135 {
    use super::*;

    /// Returns the complete Firefox 135 fingerprint
    pub fn fingerprint() -> BrowserFingerprint {
        BrowserFingerprint::new(
            "Firefox",
            "135",
            tls_fingerprint(),
            http2_fingerprint(),
            headers(),
        )
    }

    /// Firefox 135 TLS fingerprint
    pub(crate) fn tls_fingerprint() -> TlsFingerprint {
        TlsFingerprint::new(
            // Cipher suites in Firefox 135 preference order
            // TLS 1.3 cipher suites first, then TLS 1.2
            vec![
                // Real TLS 1.3 cipher suites
                CipherSuite::TLS13_AES_128_GCM_SHA256,
                CipherSuite::TLS13_CHACHA20_POLY1305_SHA256,
                CipherSuite::TLS13_AES_256_GCM_SHA384,
                // Real TLS 1.2 cipher suites
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256,
                CipherSuite::TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_CBC_SHA,
                CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_CBC_SHA,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA,
                CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA,
                CipherSuite::TLS_RSA_WITH_AES_128_GCM_SHA256,
                CipherSuite::TLS_RSA_WITH_AES_256_GCM_SHA384,
                CipherSuite::TLS_RSA_WITH_AES_128_CBC_SHA,
                CipherSuite::TLS_RSA_WITH_AES_256_CBC_SHA,
            ],
            // Key exchange groups (Firefox includes FFDHE groups and post-quantum hybrid)
            vec![
                KeyExchangeGroup::X25519MLKEM768,
                KeyExchangeGroup::X25519,
                KeyExchangeGroup::Secp256r1,
                KeyExchangeGroup::Secp384r1,
                KeyExchangeGroup::Secp521r1,
                KeyExchangeGroup::Ffdhe2048,
                KeyExchangeGroup::Ffdhe3072,
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
            // TLS extensions configuration
            TlsExtensions::new(
                true, // server_name
                true, // status_request
                true, // supported_groups
                true, // signature_algorithms
                true, // application_layer_protocol_negotiation
                true, // signed_certificate_timestamp (enabled in Firefox 135)
                true, // key_share
                true, // psk_key_exchange_modes
                true, // supported_versions
                Some(vec![
                    CertificateCompressionAlgorithm::Zlib,
                    CertificateCompressionAlgorithm::Brotli,
                    CertificateCompressionAlgorithm::Zstd,
                ]), // compress_certificate (all three algorithms)
                false, // application_settings
                true, // delegated_credentials (Firefox uses this)
                Some(16385), // record_size_limit (0x4001)
                // Extension order (critical for fingerprinting)
                vec![
                    ExtensionType::ServerName,
                    ExtensionType::ExtendedMasterSecret,
                    ExtensionType::RenegotiationInfo,
                    ExtensionType::SupportedGroups,
                    ExtensionType::EcPointFormats,
                    ExtensionType::SessionTicket,
                    ExtensionType::ApplicationLayerProtocolNegotiation,
                    ExtensionType::StatusRequest,
                    ExtensionType::DelegatedCredentials,
                    ExtensionType::SignedCertificateTimestamp,
                    ExtensionType::KeyShare,
                    ExtensionType::SupportedVersions,
                    ExtensionType::SignatureAlgorithms,
                    ExtensionType::PskKeyExchangeModes,
                    ExtensionType::RecordSizeLimit,
                    ExtensionType::CompressCertificate,
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

    /// Firefox 135 HTTP/2 fingerprint
    pub(crate) fn http2_fingerprint() -> Http2Fingerprint {
        Http2Fingerprint {
            pseudo_header_order: vec![
                ":method".to_string(),
                ":path".to_string(),
                ":authority".to_string(),
                ":scheme".to_string(),
                ":protocol".to_string(),
                ":status".to_string(),
            ],
            initial_stream_window_size: Some(131_072),
            initial_connection_window_size: Some(12_582_912),
            max_header_list_size: None,
            header_table_size: Some(65_536),
            max_concurrent_streams: None,
        }
    }

    /// Firefox 135 HTTP headers
    fn headers() -> Vec<(String, String)> {
        vec![
            ("user-agent".to_string(), "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:135.0) Gecko/20100101 Firefox/135.0".to_string()),
            ("accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".to_string()),
            ("accept-language".to_string(), "en-US,en;q=0.5".to_string()),
            ("accept-encoding".to_string(), "gzip, deflate, br, zstd".to_string()),
            ("upgrade-insecure-requests".to_string(), "1".to_string()),
            ("sec-fetch-dest".to_string(), "document".to_string()),
            ("sec-fetch-mode".to_string(), "navigate".to_string()),
            ("sec-fetch-site".to_string(), "none".to_string()),
            ("sec-fetch-user".to_string(), "?1".to_string()),
            ("priority".to_string(), "u=0, i".to_string()),
            ("te".to_string(), "trailers".to_string()),
        ]
    }
}

/// Firefox 144 fingerprint module (matches curl_firefox144)
/// Based on Firefox 135 pattern which has the same TLS fingerprint
pub mod firefox_144 {
    use super::*;

    /// Returns the complete Firefox 144 fingerprint
    pub fn fingerprint() -> BrowserFingerprint {
        BrowserFingerprint::new(
            "Firefox",
            "144",
            firefox_135::tls_fingerprint(),
            firefox_135::http2_fingerprint(),
            headers(),
        )
    }

    /// Firefox 144 HTTP headers
    fn headers() -> Vec<(String, String)> {
        vec![
            ("user-agent".to_string(), "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:144.0) Gecko/20100101 Firefox/144.0".to_string()),
            ("accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".to_string()),
            ("accept-language".to_string(), "en-US,en;q=0.5".to_string()),
            ("accept-encoding".to_string(), "gzip, deflate, br, zstd".to_string()),
            ("upgrade-insecure-requests".to_string(), "1".to_string()),
            ("sec-fetch-dest".to_string(), "document".to_string()),
            ("sec-fetch-mode".to_string(), "navigate".to_string()),
            ("sec-fetch-site".to_string(), "none".to_string()),
            ("sec-fetch-user".to_string(), "?1".to_string()),
            ("priority".to_string(), "u=0, i".to_string()),
            ("te".to_string(), "trailers".to_string()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tls::TlsConfig;
    use std::sync::Arc;

    // JA3 strings (before hashing) of real Firefox's cold TCP ClientHello, from decrypted Wireshark captures of
    // Firefox 128, 133, 135 and 144 on Windows (October 2026). Firefox 144's is the same as 135's.
    const JA3_128: &str = "771,4865-4867-4866-49195-49199-52393-52392-49196-49200-49162-49161-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-34-51-43-13-45-28-65037,29-23-24-25-256-257,0";
    const JA3_133: &str = "771,4865-4867-4866-49195-49199-52393-52392-49196-49200-49162-49161-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-34-51-43-13-45-28-27-65037,4588-29-23-24-25-256-257,0";
    const JA3_135: &str = "771,4865-4867-4866-49195-49199-52393-52392-49196-49200-49162-49161-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-34-18-51-43-13-45-28-27-65037,4588-29-23-24-25-256-257,0";

    /// The ClientHello that impit sends for `fingerprint`: legacy version, cipher suites, and each extension's type
    /// and body in wire order.
    fn client_hello(fingerprint: BrowserFingerprint) -> (u16, Vec<u16>, Vec<(u16, Vec<u8>)>) {
        let mut builder = TlsConfig::builder();
        builder.with_tls_fingerprint(fingerprint.tls);
        let server_name = rustls::pki_types::ServerName::try_from("example.com").unwrap();
        let mut conn =
            rustls::ClientConnection::new(Arc::new(builder.build()), server_name).unwrap();
        let mut record = Vec::new();
        conn.write_tls(&mut record).unwrap();

        // Skip the record (5) and handshake (4) headers.
        let hello = &record[9..];
        let u16_at = |i: usize| u16::from_be_bytes([hello[i], hello[i + 1]]);
        let version = u16_at(0);
        let mut i = 2 + 32; // legacy_version, random
        i += 1 + usize::from(hello[i]); // legacy_session_id
        let ciphers_end = i + 2 + usize::from(u16_at(i));
        let ciphers = (i + 2..ciphers_end).step_by(2).map(u16_at).collect();
        i = ciphers_end;
        i += 1 + usize::from(hello[i]); // legacy_compression_methods
        let end = i + 2 + usize::from(u16_at(i));
        i += 2;
        let mut extensions = Vec::new();
        while i < end {
            let body_end = i + 4 + usize::from(u16_at(i + 2));
            extensions.push((u16_at(i), hello[i + 4..body_end].to_vec()));
            i = body_end;
        }
        assert_eq!(i, end, "ClientHello extensions not parsed");
        assert!(
            extensions.iter().any(|(typ, _)| *typ == 0x0000),
            "no server_name, ClientHello not parsed"
        );
        (version, ciphers, extensions)
    }

    /// The JA3 string (before hashing). Firefox sends no GREASE, so no value is skipped.
    fn ja3(version: u16, ciphers: &[u16], extensions: &[(u16, Vec<u8>)]) -> String {
        fn join(values: impl IntoIterator<Item = u16>) -> String {
            values
                .into_iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join("-")
        }
        let body = |typ: u16| {
            extensions
                .iter()
                .find(|(t, _)| *t == typ)
                .map_or(&[][..], |(_, body)| body.as_slice())
        };
        let groups = body(0x000a)
            .get(2..)
            .unwrap_or_default()
            .chunks(2)
            .map(|c| u16::from_be_bytes([c[0], c[1]]));
        let point_formats = body(0x000b)
            .get(1..)
            .unwrap_or_default()
            .iter()
            .map(|&f| u16::from(f));
        format!(
            "{version},{},{},{},{}",
            join(ciphers.iter().copied()),
            join(extensions.iter().map(|(typ, _)| *typ)),
            join(groups),
            join(point_formats)
        )
    }

    // Real Firefox sends its extensions in one fixed order (over TCP), so its JA3 is the same on every cold
    // connection.
    #[test]
    fn firefox_profiles_send_real_firefox_ja3() {
        let profiles = [
            ("firefox_128", firefox_128::fingerprint(), JA3_128),
            ("firefox_133", firefox_133::fingerprint(), JA3_133),
            ("firefox_135", firefox_135::fingerprint(), JA3_135),
            ("firefox_144", firefox_144::fingerprint(), JA3_135),
        ];

        let mut mismatches = Vec::new();
        for (name, fingerprint, expected) in profiles {
            for _ in 0..8 {
                let (version, ciphers, extensions) = client_hello(fingerprint.clone());
                let got = ja3(version, &ciphers, &extensions);
                if got != expected {
                    mismatches.push(format!("{name}: {got}"));
                }
            }
        }
        assert!(
            mismatches.is_empty(),
            "ClientHellos whose JA3 differs from real Firefox's:\n{}",
            mismatches.join("\n")
        );
    }

    #[test]
    fn firefox_profiles_send_record_size_limit_16385() {
        let profiles = [
            ("firefox_128", firefox_128::fingerprint()),
            ("firefox_133", firefox_133::fingerprint()),
            ("firefox_135", firefox_135::fingerprint()),
            ("firefox_144", firefox_144::fingerprint()),
        ];

        let mut wrong = Vec::new();
        for (name, fingerprint) in profiles {
            let (_, _, extensions) = client_hello(fingerprint);
            let body = extensions
                .into_iter()
                .find(|(typ, _)| *typ == 0x001c)
                .map(|(_, body)| body);
            if body.as_deref() != Some(&[0x40, 0x01][..]) {
                wrong.push(format!("{name}: {body:02x?}"));
            }
        }
        assert!(
            wrong.is_empty(),
            "record_size_limit is not 16385 (0x4001): {wrong:?}"
        );
    }
}
