//! Browser fingerprint data structures
//!
//! This module contains all the types needed to define a complete browser fingerprint,
//! including TLS, HTTP/2, and HTTP header configurations.

pub mod database;
mod types;

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

    /// Produces a deterministically-mutated clone of this fingerprint based on
    /// the given `seed`.  Uses a seeded PRNG (StdRng) for unlimited randomization
    /// dimensions while remaining fully deterministic for the same seed.
    ///
    /// ## Randomization dimensions
    ///
    /// | Dimension                      | Choices  | JA4 impact       | JA3 impact |
    /// |-------------------------------|----------|------------------|------------|
    /// | GREASE cipher value           | 16       | No (excluded)    | Yes        |
    /// | GREASE key exchange value     | 16       | No (excluded)    | Yes        |
    /// | GREASE extension value        | 16       | No (excluded)    | Yes        |
    /// | Optional TLS 1.2 ciphers (8)  | 2^8=256  | Yes              | Yes        |
    /// | Optional extensions (6)       | 2^6=64   | Yes              | Yes        |
    /// | Optional sig algs (6)         | 2^6=64   | Yes (extHash)    | Yes        |
    /// | Key exchange group toggle (2) | 2^2=4    | Yes              | Yes        |
    /// | ALPS codepoint (old/new)      | 2        | Yes              | Yes        |
    /// | ECH GREASE toggle             | 2        | Yes              | Yes        |
    /// | TLS 1.2 cipher order shuffle  | many     | No               | Yes        |
    /// | Header micro-variations       | many     | N/A              | N/A        |
    ///
    /// **JA4 diversity per profile**: ~16.7 million unique hashes
    /// **JA3 diversity**: practically unlimited (GREASE + order permutations)
    /// **With 15 profiles**: ~250 million unique JA4 hashes
    pub fn randomize(&self, seed: u64) -> BrowserFingerprint {
        use rand::rngs::StdRng;
        use rand::seq::SliceRandom;
        use rand::{Rng, SeedableRng};

        let mut rng = StdRng::seed_from_u64(seed);
        let mut fp = self.clone();

        // ── RFC 8701 GREASE values ──────────────────────────────────────
        const GREASE_VALUES: [u16; 16] = [
            0x0a0a, 0x1a1a, 0x2a2a, 0x3a3a, 0x4a4a, 0x5a5a, 0x6a6a, 0x7a7a,
            0x8a8a, 0x9a9a, 0xaaaa, 0xbaba, 0xcaca, 0xdada, 0xeaea, 0xfafa,
        ];

        // ── 1. Randomize GREASE values ──────────────────────────────────
        // Each GREASE slot gets a random value from the 16 RFC 8701 values.
        // This affects JA3 (which includes GREASE) but not JA4 (which excludes it).
        let grease_cipher_val = GREASE_VALUES[rng.gen_range(0..16)];
        let grease_kex_val = GREASE_VALUES[rng.gen_range(0..16)];
        let grease_ext_val = GREASE_VALUES[rng.gen_range(0..16)];

        // Replace GREASE cipher suite values
        for cs in &mut fp.tls.cipher_suites {
            if matches!(cs, CipherSuite::Grease(_)) {
                *cs = CipherSuite::Grease(grease_cipher_val);
            }
        }

        // Replace GREASE key exchange group values
        for kg in &mut fp.tls.key_exchange_groups {
            if matches!(kg, KeyExchangeGroup::Grease(_)) {
                *kg = KeyExchangeGroup::Grease(grease_kex_val);
            }
        }

        // Replace GREASE extension type values
        for ext in &mut fp.tls.extensions.extension_order {
            if matches!(ext, ExtensionType::Grease(_)) {
                *ext = ExtensionType::Grease(grease_ext_val);
            }
        }

        // ── 2. Optional TLS 1.2 cipher suites (8 toggleable) ───────────
        // These legacy/CBC/RSA-only suites can be independently toggled.
        // Core TLS 1.3 + ECDHE-GCM/ChaCha20 suites always remain.
        let optional_ciphers: &[CipherSuite] = &[
            CipherSuite::TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA,
            CipherSuite::TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA,
            CipherSuite::TLS_RSA_WITH_AES_128_GCM_SHA256,
            CipherSuite::TLS_RSA_WITH_AES_256_GCM_SHA384,
            CipherSuite::TLS_RSA_WITH_AES_128_CBC_SHA,
            CipherSuite::TLS_RSA_WITH_AES_256_CBC_SHA,
            CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_128_CBC_SHA,
            CipherSuite::TLS_ECDHE_ECDSA_WITH_AES_256_CBC_SHA,
        ];

        for cipher in optional_ciphers {
            if rng.gen_bool(0.5) {
                fp.tls.cipher_suites.retain(|c| c != cipher);
            }
        }

        // ── 3. Optional TLS extensions (6 toggleable) ───────────────────
        // compress_certificate
        if rng.gen_bool(0.5) {
            if fp.tls.extensions.compress_certificate.is_some() {
                fp.tls.extensions.compress_certificate = None;
                fp.tls.extensions.extension_order
                    .retain(|e| *e != ExtensionType::CompressCertificate);
            } else {
                fp.tls.extensions.compress_certificate =
                    Some(vec![CertificateCompressionAlgorithm::Brotli]);
                if !fp.tls.extensions.extension_order
                    .contains(&ExtensionType::CompressCertificate)
                {
                    let len = fp.tls.extensions.extension_order.len();
                    fp.tls.extensions.extension_order
                        .insert(len.saturating_sub(1), ExtensionType::CompressCertificate);
                }
            }
        }

        // application_settings (ALPS)
        if rng.gen_bool(0.5) {
            fp.tls.extensions.application_settings = !fp.tls.extensions.application_settings;
            if !fp.tls.extensions.application_settings {
                fp.tls.extensions.extension_order
                    .retain(|e| *e != ExtensionType::ApplicationSettings);
            } else if !fp.tls.extensions.extension_order
                .contains(&ExtensionType::ApplicationSettings)
            {
                let len = fp.tls.extensions.extension_order.len();
                fp.tls.extensions.extension_order
                    .insert(len.saturating_sub(1), ExtensionType::ApplicationSettings);
            }
        }

        // signed_certificate_timestamp
        if rng.gen_bool(0.5) {
            fp.tls.extensions.signed_certificate_timestamp =
                !fp.tls.extensions.signed_certificate_timestamp;
            if !fp.tls.extensions.signed_certificate_timestamp {
                fp.tls.extensions.extension_order
                    .retain(|e| *e != ExtensionType::SignedCertificateTimestamp);
            } else if !fp.tls.extensions.extension_order
                .contains(&ExtensionType::SignedCertificateTimestamp)
            {
                let len = fp.tls.extensions.extension_order.len();
                fp.tls.extensions.extension_order
                    .insert(len.saturating_sub(1), ExtensionType::SignedCertificateTimestamp);
            }
        }

        // delegated_credentials
        if rng.gen_bool(0.5) {
            fp.tls.extensions.delegated_credentials = !fp.tls.extensions.delegated_credentials;
        }

        // padding extension
        if rng.gen_bool(0.5) {
            fp.tls.extensions.padding = !fp.tls.extensions.padding;
            if fp.tls.extensions.padding {
                if !fp.tls.extensions.extension_order.contains(&ExtensionType::Padding) {
                    fp.tls.extensions.extension_order.push(ExtensionType::Padding);
                }
            } else {
                fp.tls.extensions.extension_order
                    .retain(|e| *e != ExtensionType::Padding);
            }
        }

        // record_size_limit
        if rng.gen_bool(0.5) {
            if fp.tls.extensions.record_size_limit.is_some() {
                fp.tls.extensions.record_size_limit = None;
            } else {
                // Firefox uses 16385, some use 4096
                fp.tls.extensions.record_size_limit = Some(if rng.gen_bool(0.7) { 16385 } else { 4096 });
            }
        }

        // ── 4. Optional signature algorithms (6 toggleable) ─────────────
        let optional_sigalgs: &[SignatureAlgorithm] = &[
            SignatureAlgorithm::RsaPkcs1Sha384,
            SignatureAlgorithm::RsaPssRsaSha512,
            SignatureAlgorithm::RsaPkcs1Sha512,
            SignatureAlgorithm::Ed25519,
            SignatureAlgorithm::EcdsaSha1Legacy,
            SignatureAlgorithm::RsaPkcs1Sha1,
        ];

        for sigalg in optional_sigalgs {
            if rng.gen_bool(0.5) {
                fp.tls.signature_algorithms.retain(|s| s != sigalg);
            }
        }

        // ── 5. Key exchange group variations ────────────────────────────
        // Toggle post-quantum hybrid (X25519MLKEM768) - only present in Chrome 142+
        if rng.gen_bool(0.5) {
            if fp.tls.key_exchange_groups.contains(&KeyExchangeGroup::X25519MLKEM768) {
                fp.tls.key_exchange_groups.retain(|g| *g != KeyExchangeGroup::X25519MLKEM768);
            }
        }

        // Toggle Secp384r1
        if rng.gen_bool(0.5) {
            if fp.tls.key_exchange_groups.contains(&KeyExchangeGroup::Secp384r1) {
                fp.tls.key_exchange_groups.retain(|g| *g != KeyExchangeGroup::Secp384r1);
            }
        }

        // ── 6. ALPS codepoint toggle (old 17513 vs new 17613) ───────────
        // Chrome 136+ uses new, older versions use old.  Toggling changes
        // the extension type code in JA4.
        if rng.gen_bool(0.5) && fp.tls.extensions.application_settings {
            fp.tls.extensions.use_new_alps_codepoint = !fp.tls.extensions.use_new_alps_codepoint;
        }

        // ── 7. ECH GREASE toggle ────────────────────────────────────────
        // Toggle Encrypted Client Hello GREASE on/off
        if rng.gen_bool(0.5) {
            if fp.tls.ech_config.is_some() {
                fp.tls.ech_config = None;
            } else {
                fp.tls.ech_config = Some(EchConfig::new(
                    EchMode::Grease {
                        hpke_suite: HpkeKemId::DhKemX25519HkdfSha256,
                    },
                    None,
                ));
            }
        }

        // ── 8. TLS 1.2 cipher suite order shuffle (JA3 only) ───────────
        // JA4 sorts cipher suites, so order doesn't matter.  But JA3 is
        // order-sensitive, so shuffling TLS 1.2 suites creates massive
        // JA3 diversity without affecting JA4.
        // Find the TLS 1.2 block (everything after TLS 1.3 suites and GREASE)
        let tls12_start = fp.tls.cipher_suites.iter().position(|cs| {
            !matches!(
                cs,
                CipherSuite::TLS13_AES_128_GCM_SHA256
                    | CipherSuite::TLS13_AES_256_GCM_SHA384
                    | CipherSuite::TLS13_CHACHA20_POLY1305_SHA256
                    | CipherSuite::Grease(_)
            )
        });
        if let Some(start) = tls12_start {
            fp.tls.cipher_suites[start..].shuffle(&mut rng);
        }

        // ── 9. Header micro-variations ──────────────────────────────────
        // Vary the User-Agent patch version and Accept-Language quality factors
        // to create subtle header diversity across sessions.
        Self::randomize_headers(&mut fp.headers, &mut rng);

        fp
    }

    /// Apply micro-variations to HTTP headers for fingerprint diversity.
    fn randomize_headers(headers: &mut Vec<(String, String)>, rng: &mut impl rand::Rng) {
        for (key, value) in headers.iter_mut() {
            let key_lower = key.to_lowercase();

            // Vary Chrome/Edge UA patch version: Chrome/142.0.0.0 → Chrome/142.0.XXXX.YY
            if key_lower == "user-agent" {
                if let Some(chrome_pos) = value.find("Chrome/") {
                    let after = &value[chrome_pos + 7..];
                    // Find the major version (e.g., "142")
                    if let Some(dot_pos) = after.find('.') {
                        let major = &after[..dot_pos];
                        if let Ok(ver) = major.parse::<u32>() {
                            if ver >= 125 {
                                // Generate realistic build numbers
                                let build = rng.gen_range(6700..7300);
                                let patch = rng.gen_range(0..256);
                                let new_version = format!("{}.0.{}.{}", ver, build, patch);

                                // Find the end of the version string
                                let version_start = chrome_pos + 7;
                                // Find next space or end
                                let version_end = value[version_start..]
                                    .find(' ')
                                    .map(|p| version_start + p)
                                    .unwrap_or(value.len());

                                *value = format!(
                                    "{}{}{}",
                                    &value[..version_start],
                                    new_version,
                                    &value[version_end..]
                                );
                            }
                        }
                    }
                }
            }

            // Vary Accept-Language quality factor: en;q=0.9 → en;q=0.{7-9}
            if key_lower == "accept-language" {
                let q_values = ["0.9", "0.8", "0.7"];
                let chosen = q_values[rng.gen_range(0..q_values.len())];
                if value.contains("en;q=0.9") {
                    *value = value.replace("en;q=0.9", &format!("en;q={}", chosen));
                } else if value.contains("en;q=0.5") {
                    // Firefox uses q=0.5
                    let ff_q = ["0.5", "0.3", "0.7"];
                    let ff_chosen = ff_q[rng.gen_range(0..ff_q.len())];
                    *value = value.replace("en;q=0.5", &format!("en;q={}", ff_chosen));
                }
            }
        }
    }
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

/// HTTP/2 fingerprint configuration.
///
/// Controls the SETTINGS frame parameters, WINDOW_UPDATE increment, and
/// pseudo-header ordering sent during the HTTP/2 handshake. These values
/// are used by anti-bot systems (Cloudflare, Akamai) for fingerprinting.
#[derive(Clone, Debug)]
pub struct Http2Fingerprint {
    /// Order of HTTP/2 pseudo-headers in HEADERS frames.
    /// Chrome: `:method, :authority, :scheme, :path`
    /// Firefox: `:method, :path, :authority, :scheme`
    pub pseudo_header_order: Vec<String>,
    /// SETTINGS_HEADER_TABLE_SIZE (0x1). Chrome/Firefox: 65536. Default: 4096.
    pub settings_header_table_size: Option<u32>,
    /// SETTINGS_ENABLE_PUSH (0x2). Chrome: false. Default: true.
    pub settings_enable_push: Option<bool>,
    /// SETTINGS_MAX_CONCURRENT_STREAMS (0x3). Chrome: 1000. Firefox: not sent.
    pub settings_max_concurrent_streams: Option<u32>,
    /// SETTINGS_INITIAL_WINDOW_SIZE (0x4). Chrome: 6291456. Firefox: 131072. Default: 65535.
    pub settings_initial_window_size: Option<u32>,
    /// SETTINGS_MAX_FRAME_SIZE (0x5). Firefox: 16384. Default: 16384.
    pub settings_max_frame_size: Option<u32>,
    /// SETTINGS_MAX_HEADER_LIST_SIZE (0x6). Chrome: 262144. Firefox: not sent.
    pub settings_max_header_list_size: Option<u32>,
    /// Connection-level WINDOW_UPDATE increment sent after SETTINGS.
    /// Chrome: 15663105 (total window = 65535 + 15663105 = ~15MB).
    /// Firefox: 12517377 (total window = 65535 + 12517377 = ~12MB).
    pub connection_window_size_increment: Option<u32>,
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
    /// Whether to randomly permute TLS extension order on each client build.
    /// Chrome 110+ randomizes extension order per-connection, producing different
    /// JA3 hashes each time (while JA4 remains stable due to sorted hashing).
    /// This makes the client indistinguishable from a real Chrome browser.
    /// GREASE extensions at the start/end of the list are kept in place.
    pub permute_extensions: bool,
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
            permute_extensions: false,
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

    /// Enable TLS extension order permutation (Chrome 110+ behavior).
    /// When enabled, the extension order is randomly shuffled when building the
    /// TLS fingerprint. GREASE extensions at the boundaries are kept in place.
    /// This produces different JA3 hashes per client instance while maintaining
    /// the same JA4 hash (since JA4 sorts extensions before hashing).
    pub fn with_permute_extensions(mut self, enabled: bool) -> Self {
        self.permute_extensions = enabled;
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
    ///
    /// If `permute_extensions` is enabled on the extensions config, the extension
    /// order will be randomly shuffled (mimicking Chrome 110+ behavior). GREASE
    /// extensions at the start/end are kept in place, while all other extensions
    /// are shuffled. This produces different JA3 hashes per call while JA4
    /// (which sorts extensions) remains stable.
    pub fn to_rustls_fingerprint(&self) -> rustls::client::TlsFingerprint {
        use rand::seq::SliceRandom;
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
                CipherSuite::Grease(_) => FingerprintCipherSuite::Grease,
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
                KeyExchangeGroup::Grease(_) => FingerprintKeyExchangeGroup::Grease,
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

        // Apply extension permutation if enabled (Chrome 110+ behavior)
        // This shuffles non-GREASE extensions while keeping GREASE at boundaries
        let extension_order = if self.extensions.permute_extensions {
            let mut order = self.extensions.extension_order.clone();
            // Find the range of non-GREASE extensions to shuffle
            // GREASE can appear at the start and/or end - keep those in place
            let start = if matches!(order.first(), Some(ExtensionType::Grease(_))) { 1 } else { 0 };
            let end = if order.len() > 1 && matches!(order.last(), Some(ExtensionType::Grease(_))) && start < order.len() - 1 {
                order.len() - 1
            } else {
                order.len()
            };
            // Also keep Padding at the end if present (it must be last before GREASE)
            let shuffle_end = if end > start && order.get(end - 1) == Some(&ExtensionType::Padding) {
                end - 1
            } else {
                end
            };
            if shuffle_end > start + 1 {
                let mut rng = rand::thread_rng();
                order[start..shuffle_end].shuffle(&mut rng);
            }
            order
        } else {
            self.extensions.extension_order.clone()
        };

        // Find the GREASE extension value (if any) from the extension order
        let grease_ext_val = extension_order
            .iter()
            .find_map(|e| match e {
                ExtensionType::Grease(val) => Some(*val),
                _ => None,
            });

        // Map impit ExtensionType → rustls ExtensionType for extension ordering
        use rustls::internal::msgs::enums::ExtensionType as RustlsExtType;
        let rustls_extension_order: Vec<RustlsExtType> = extension_order
            .iter()
            .filter_map(|ext| match ext {
                ExtensionType::ServerName => Some(RustlsExtType::ServerName),
                ExtensionType::StatusRequest => Some(RustlsExtType::StatusRequest),
                ExtensionType::SupportedGroups => Some(RustlsExtType::EllipticCurves),
                ExtensionType::EcPointFormats => Some(RustlsExtType::ECPointFormats),
                ExtensionType::SignatureAlgorithms => Some(RustlsExtType::SignatureAlgorithms),
                ExtensionType::ApplicationLayerProtocolNegotiation => {
                    Some(RustlsExtType::ALProtocolNegotiation)
                }
                ExtensionType::SignedCertificateTimestamp => Some(RustlsExtType::SCT),
                ExtensionType::Padding => Some(RustlsExtType::Padding),
                ExtensionType::SupportedVersions => Some(RustlsExtType::SupportedVersions),
                ExtensionType::PskKeyExchangeModes => Some(RustlsExtType::PSKKeyExchangeModes),
                ExtensionType::KeyShare => Some(RustlsExtType::KeyShare),
                ExtensionType::ExtendedMasterSecret => Some(RustlsExtType::ExtendedMasterSecret),
                ExtensionType::RenegotiationInfo => Some(RustlsExtType::RenegotiationInfo),
                ExtensionType::SessionTicket => Some(RustlsExtType::SessionTicket),
                ExtensionType::CompressCertificate => Some(RustlsExtType::CompressCertificate),
                ExtensionType::ApplicationSettings => {
                    if self.extensions.use_new_alps_codepoint {
                        Some(RustlsExtType::ApplicationSettingsNew)
                    } else {
                        Some(RustlsExtType::ApplicationSettings)
                    }
                }
                ExtensionType::PreSharedKey => Some(RustlsExtType::PreSharedKey),
                ExtensionType::EarlyData => Some(RustlsExtType::EarlyData),
                ExtensionType::PostHandshakeAuth => Some(RustlsExtType::PostHandshakeAuth),
                ExtensionType::Grease(_) => Some(RustlsExtType::ReservedGrease),
                _ => None,
            })
            .collect();

        let extensions_config = TlsExtensionsConfig {
            grease: grease_ext_val.is_some(),
            signed_certificate_timestamp: self.extensions.signed_certificate_timestamp,
            application_settings: self.extensions.application_settings,
            use_new_alps_codepoint: self.extensions.use_new_alps_codepoint,
            delegated_credentials: self.extensions.delegated_credentials,
            record_size_limit: self.extensions.record_size_limit,
            renegotiation_info: true, // Common for both browsers
            padding: self.extensions.padding,
            supported_versions: true,
            extension_order: rustls_extension_order,
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
