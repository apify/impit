#![allow(non_camel_case_types)]
//! Type definitions for browser fingerprints
//!
//! This module contains enum types used to configure TLS and HTTP/2 fingerprints
//! in a type-safe manner.

/// TLS cipher suites
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CipherSuite {
    // TLS 1.3 cipher suites
    TLS13_AES_128_GCM_SHA256,
    TLS13_AES_256_GCM_SHA384,
    TLS13_CHACHA20_POLY1305_SHA256,
    // TLS 1.2 cipher suites
    TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
    TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
    TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
    TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
    TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256,
    TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256,
    TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA,
    TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA,
    TLS_ECDHE_ECDSA_WITH_AES_128_CBC_SHA,
    TLS_ECDHE_ECDSA_WITH_AES_256_CBC_SHA,
    TLS_RSA_WITH_AES_128_GCM_SHA256,
    TLS_RSA_WITH_AES_256_GCM_SHA384,
    TLS_RSA_WITH_AES_128_CBC_SHA,
    TLS_RSA_WITH_AES_256_CBC_SHA,
    /// GREASE cipher suite for fingerprinting
    Grease,
}

/// Key exchange groups for TLS
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeyExchangeGroup {
    X25519,
    /// X25519 with MLKEM768 (post-quantum hybrid)
    X25519MLKEM768,
    Secp256r1,
    Secp384r1,
    Secp521r1,
    Ffdhe2048,
    Ffdhe3072,
    Ffdhe4096,
    Ffdhe6144,
    Ffdhe8192,
    /// GREASE key exchange group for fingerprinting
    Grease,
}

/// Signature algorithms for TLS
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SignatureAlgorithm {
    // RSASSA-PSS algorithms
    RsaPssRsaSha256,
    RsaPssRsaSha384,
    RsaPssRsaSha512,
    // ECDSA algorithms
    EcdsaSecp256r1Sha256,
    EcdsaSecp384r1Sha384,
    EcdsaSecp521r1Sha512,
    // Legacy RSA PKCS#1 v1.5 algorithms
    RsaPkcs1Sha256,
    RsaPkcs1Sha384,
    RsaPkcs1Sha512,
    RsaPkcs1Sha1,
    // EdDSA algorithms
    Ed25519,
    Ed448,
    // Legacy ECDSA with SHA-1 (for backwards compatibility)
    EcdsaSha1Legacy,
}

/// TLS extension types
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExtensionType {
    ServerName,
    MaxFragmentLength,
    StatusRequest,
    SupportedGroups,
    EcPointFormats,
    SignatureAlgorithms,
    UseSrtp,
    Heartbeat,
    ApplicationLayerProtocolNegotiation,
    SignedCertificateTimestamp,
    ClientCertificateType,
    ServerCertificateType,
    Padding,
    PreSharedKey,
    EarlyData,
    SupportedVersions,
    Cookie,
    PskKeyExchangeModes,
    CertificateAuthorities,
    OidFilters,
    PostHandshakeAuth,
    SignatureAlgorithmsCert,
    KeyShare,
    ExtendedMasterSecret,
    RenegotiationInfo,
    SessionTicket,
    CompressCertificate,
    ApplicationSettings,
    EarlyDataExtension,
    Grease,
}

/// Certificate compression algorithms
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CertificateCompressionAlgorithm {
    Zlib,
    Brotli,
    Zstd,
}

/// HPKE KEM identifiers for ECH
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HpkeKemId {
    DhKemP256HkdfSha256,
    DhKemP384HkdfSha384,
    DhKemP521HkdfSha512,
    DhKemX25519HkdfSha256,
    DhKemX448HkdfSha512,
}
