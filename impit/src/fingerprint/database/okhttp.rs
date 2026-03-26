//! OkHttp client fingerprints
//!
//! OkHttp is the most popular HTTP client for Android applications.
//! These fingerprints cover OkHttp 3.x through 5.x using Conscrypt (BoringSSL)
//! as the TLS provider, which is the default on modern Android devices.
//!
//! Key differences from browser fingerprints:
//! - No GREASE values (Conscrypt doesn't use GREASE)
//! - No post-quantum key exchange (no X25519MLKEM768)
//! - No ECH/ESNI support
//! - No certificate compression
//! - No ALPS/application_settings
//! - Includes ec_point_formats extension (browsers have dropped this)
//! - Simpler cipher suite list (ConnectionSpec.MODERN_TLS)
//! - HTTP/2 window sizes set to 16 MiB (vs 64 KiB browser defaults)

use crate::fingerprint::*;

/// Shared HTTP/2 fingerprint for all OkHttp versions.
///
/// OkHttp's HTTP/2 implementation (okhttp2 codec) uses the same settings
/// across versions 3.x through 5.x.
fn shared_http2_fingerprint() -> Http2Fingerprint {
    Http2Fingerprint {
        // OkHttp pseudo-header order (from Http2ExchangeCodec)
        pseudo_header_order: vec![
            ":method".to_string(),
            ":path".to_string(),
            ":authority".to_string(),
            ":scheme".to_string(),
            ":protocol".to_string(),
            ":status".to_string(),
        ],
        // OkHttp sets INITIAL_WINDOW_SIZE to 16 MiB
        // (okHttpClientWindowSize = 16 * 1024 * 1024)
        initial_stream_window_size: Some(16_777_216),
        // Connection window is also increased to 16 MiB via WINDOW_UPDATE
        initial_connection_window_size: Some(16_777_216),
        max_header_list_size: None,
    }
}

/// Shared signature algorithms for all OkHttp versions with Conscrypt.
fn shared_signature_algorithms() -> Vec<SignatureAlgorithm> {
    vec![
        SignatureAlgorithm::EcdsaSecp256r1Sha256,
        SignatureAlgorithm::RsaPssRsaSha256,
        SignatureAlgorithm::RsaPkcs1Sha256,
        SignatureAlgorithm::EcdsaSecp384r1Sha384,
        SignatureAlgorithm::RsaPssRsaSha384,
        SignatureAlgorithm::RsaPkcs1Sha384,
        SignatureAlgorithm::RsaPssRsaSha512,
        SignatureAlgorithm::RsaPkcs1Sha512,
        SignatureAlgorithm::RsaPkcs1Sha1,
    ]
}

/// OkHttp 3 fingerprint module (pre-TLS 1.3, Conscrypt on Android)
///
/// Represents OkHttp 3.9-3.11 before TLS 1.3 support was added in 3.12.
/// Common in older Android apps that haven't updated their OkHttp dependency.
/// TLS 1.2 only — no key_share, psk_key_exchange_modes, or supported_versions.
/// Includes CBC and RSA cipher suites from ConnectionSpec.MODERN_TLS.
pub mod okhttp3 {
    use super::*;

    pub fn fingerprint() -> BrowserFingerprint {
        BrowserFingerprint::new(
            "OkHttp",
            "3",
            tls_fingerprint(),
            shared_http2_fingerprint(),
            headers(),
        )
    }

    pub(crate) fn tls_fingerprint() -> TlsFingerprint {
        TlsFingerprint::new(
            // OkHttp 3 cipher suites — TLS 1.2 only, no TLS 1.3 ciphers
            // Includes CBC and RSA suites from MODERN_TLS
            vec![
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
            shared_signature_algorithms(),
            // TLS extensions — no TLS 1.3 extensions
            TlsExtensions::new(
                true,  // server_name
                true,  // status_request
                true,  // supported_groups
                true,  // signature_algorithms
                true,  // application_layer_protocol_negotiation
                false, // signed_certificate_timestamp
                false, // key_share (TLS 1.3 only)
                false, // psk_key_exchange_modes (TLS 1.3 only)
                false, // supported_versions (TLS 1.3 only)
                None,  // compress_certificate
                false, // application_settings
                false, // delegated_credentials
                None,  // record_size_limit
                // Extension order for Conscrypt TLS 1.2
                vec![
                    ExtensionType::ServerName,
                    ExtensionType::ExtendedMasterSecret,
                    ExtensionType::RenegotiationInfo,
                    ExtensionType::SupportedGroups,
                    ExtensionType::EcPointFormats,
                    ExtensionType::SessionTicket,
                    ExtensionType::ApplicationLayerProtocolNegotiation,
                    ExtensionType::StatusRequest,
                    ExtensionType::SignatureAlgorithms,
                ],
            ),
            None, // No ECH
            vec![b"h2".to_vec(), b"http/1.1".to_vec()],
        )
    }

    fn headers() -> Vec<(String, String)> {
        vec![
            ("user-agent".to_string(), "okhttp/3.14.9".to_string()),
            ("accept-encoding".to_string(), "gzip".to_string()),
        ]
    }
}

/// OkHttp 4 fingerprint module (Conscrypt on Android 13+)
///
/// Represents OkHttp 4.12.x with Conscrypt TLS provider on Android 13-14.
/// This covers the vast majority of Android app traffic in production.
pub mod okhttp4 {
    use super::*;

    pub fn fingerprint() -> BrowserFingerprint {
        BrowserFingerprint::new(
            "OkHttp",
            "4",
            tls_fingerprint(),
            shared_http2_fingerprint(),
            headers(),
        )
    }

    /// OkHttp 4 TLS fingerprint (Conscrypt/BoringSSL on Android)
    pub(crate) fn tls_fingerprint() -> TlsFingerprint {
        TlsFingerprint::new(
            // Cipher suites in OkHttp 4 preference order (Conscrypt)
            // No GREASE — Conscrypt doesn't use GREASE values
            // Filtered to OkHttp's ConnectionSpec.MODERN_TLS approved list
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
            ],
            // Key exchange groups (Conscrypt on Android 13+)
            // No post-quantum, no GREASE
            vec![
                KeyExchangeGroup::X25519,
                KeyExchangeGroup::Secp256r1,
                KeyExchangeGroup::Secp384r1,
            ],
            shared_signature_algorithms(),
            // TLS extensions configuration
            // OkHttp/Conscrypt has a simpler extension set than browsers
            TlsExtensions::new(
                true,  // server_name
                true,  // status_request
                true,  // supported_groups
                true,  // signature_algorithms
                true,  // application_layer_protocol_negotiation
                false, // signed_certificate_timestamp (not used by OkHttp)
                true,  // key_share
                true,  // psk_key_exchange_modes
                true,  // supported_versions
                None,  // compress_certificate (not used by OkHttp)
                false, // application_settings (not used by OkHttp)
                false, // delegated_credentials (not used by OkHttp)
                None,  // record_size_limit (not used by OkHttp)
                // Extension order (critical for fingerprinting)
                // Conscrypt sends extensions in this order (from JA3 capture)
                vec![
                    ExtensionType::ServerName,
                    ExtensionType::ExtendedMasterSecret,
                    ExtensionType::RenegotiationInfo,
                    ExtensionType::SupportedGroups,
                    ExtensionType::EcPointFormats,
                    ExtensionType::SessionTicket,
                    ExtensionType::ApplicationLayerProtocolNegotiation,
                    ExtensionType::StatusRequest,
                    ExtensionType::SignatureAlgorithms,
                    ExtensionType::KeyShare,
                    ExtensionType::PskKeyExchangeModes,
                    ExtensionType::SupportedVersions,
                ],
            ),
            // No ECH — Conscrypt doesn't support ECH/GREASE
            None,
            // ALPN protocols
            vec![b"h2".to_vec(), b"http/1.1".to_vec()],
        )
    }

    fn headers() -> Vec<(String, String)> {
        vec![
            ("user-agent".to_string(), "okhttp/4.12.0".to_string()),
            ("accept-encoding".to_string(), "gzip".to_string()),
        ]
    }
}

/// OkHttp 5 fingerprint module (Conscrypt on Android)
///
/// Represents OkHttp 5.x with Conscrypt TLS provider.
/// TLS fingerprint is identical to OkHttp 4.x on Android/Conscrypt.
/// The main behavioral difference is that OkHttp 5 enforces its own
/// cipher suite preference order rather than deferring to the JVM.
pub mod okhttp5 {
    use super::*;

    pub fn fingerprint() -> BrowserFingerprint {
        BrowserFingerprint::new(
            "OkHttp",
            "5",
            // Same TLS fingerprint as OkHttp 4 on Conscrypt
            okhttp4::tls_fingerprint(),
            shared_http2_fingerprint(),
            headers(),
        )
    }

    fn headers() -> Vec<(String, String)> {
        vec![
            ("user-agent".to_string(), "okhttp/5.0.0".to_string()),
            ("accept-encoding".to_string(), "gzip".to_string()),
        ]
    }
}
