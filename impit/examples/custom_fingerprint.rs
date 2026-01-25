//! Example demonstrating custom browser fingerprint usage
//!
//! This example shows how to:
//! 1. Use pre-defined fingerprints from the database
//! 2. Create a completely custom fingerprint
//! 3. Mix and match components from different fingerprints

use impit::emulation::Browser;
use impit::fingerprint::database;
use impit::fingerprint::*;
use impit::impit::Impit;
use reqwest::cookie::Jar;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Using pre-defined fingerprint with the old API (backward compatible)
    println!("Example 1: Using Browser enum (backward compatible)");
    let impit1 = Impit::<Jar>::builder()
        .with_browser(Browser::Chrome)
        .build()?;

    let response = impit1.get("https://httpbin.org/headers".to_string(), None, None).await?;
    println!("Status: {}", response.status());
    println!("Response: {}\n", response.text().await?);

    // Example 2: Using pre-defined fingerprint with new API (explicit)
    println!("Example 2: Using pre-defined Chrome fingerprint explicitly");
    let impit2 = Impit::<Jar>::builder()
        .with_fingerprint(database::chrome_125::fingerprint())
        .build()?;

    let response = impit2.get("https://httpbin.org/headers".to_string(), None, None).await?;
    println!("Status: {}", response.status());
    println!("Response: {}\n", response.text().await?);

    // Example 3: Creating a completely custom fingerprint
    println!("Example 3: Creating a custom fingerprint");

    let custom_tls = TlsFingerprint::new(
        // Minimal cipher suite list
        vec![
            CipherSuite::TLS13_AES_128_GCM_SHA256,
            CipherSuite::TLS13_CHACHA20_POLY1305_SHA256,
        ],
        // Key exchange groups
        vec![
            KeyExchangeGroup::X25519,
            KeyExchangeGroup::Secp256r1,
        ],
        // Signature algorithms
        vec![
            SignatureAlgorithm::EcdsaSecp256r1Sha256,
            SignatureAlgorithm::RsaPssRsaSha256,
        ],
        // TLS extensions
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
            vec![
                ExtensionType::ServerName,
                ExtensionType::SupportedGroups,
                ExtensionType::SignatureAlgorithms,
                ExtensionType::KeyShare,
                ExtensionType::PskKeyExchangeModes,
                ExtensionType::SupportedVersions,
            ],
        ),
        // ECH configuration
        Some(EchConfig::new(
            EchMode::Grease {
                hpke_suite: HpkeKemId::DhKemX25519HkdfSha256,
            },
            None,
        )),
        // ALPN protocols
        vec![b"h2".to_vec(), b"http/1.1".to_vec()],
    );

    let custom_http2 = Http2Fingerprint::new(
        // Custom pseudo-header order
        vec![
            ":method".to_string(),
            ":path".to_string(),
            ":authority".to_string(),
            ":scheme".to_string(),
        ],
        // SETTINGS frame
        Http2Settings::new(
            Some(65536),
            Some(false),
            Some(1000),
            Some(6291456),
            Some(16384),
            Some(262144),
            vec![],
        ),
        // Window sizes
        Http2WindowSize::new(15728640, 6291456),
        // Priority
        Some(Http2Priority::new(255, 0, false)),
    );

    let custom_headers = vec![
        ("user-agent".to_string(), "CustomBrowser/1.0".to_string()),
        ("accept".to_string(), "text/html,application/xhtml+xml".to_string()),
        ("accept-encoding".to_string(), "gzip, deflate, br".to_string()),
        ("accept-language".to_string(), "en-US,en;q=0.9".to_string()),
    ];

    let custom_fp = BrowserFingerprint::new(
        "CustomBrowser",
        "1.0",
        custom_tls,
        custom_http2,
        custom_headers,
    );

    let impit3 = Impit::<Jar>::builder()
        .with_fingerprint(custom_fp)
        .build()?;

    let response = impit3.get("https://httpbin.org/headers".to_string(), None, None).await?;
    println!("Status: {}", response.status());
    println!("Response: {}\n", response.text().await?);

    // Example 4: Hybrid approach - use Chrome's TLS/HTTP2 but custom headers
    println!("Example 4: Hybrid fingerprint (Chrome TLS/HTTP2 + custom headers)");

    let base_fp = database::chrome_125::fingerprint();
    let custom_headers = vec![
        ("user-agent".to_string(), "MyApp/1.0 (Custom)".to_string()),
        ("x-custom-header".to_string(), "custom-value".to_string()),
    ];

    let hybrid_fp = BrowserFingerprint::new(
        "HybridBrowser",
        "1.0",
        base_fp.tls().clone(),      // Reuse Chrome's TLS fingerprint
        base_fp.http2().clone(),     // Reuse Chrome's HTTP/2 fingerprint
        custom_headers,              // Custom headers
    );

    let impit4 = Impit::<Jar>::builder()
        .with_fingerprint(hybrid_fp)
        .build()?;

    let response = impit4.get("https://httpbin.org/headers".to_string(), None, None).await?;
    println!("Status: {}", response.status());
    println!("Response: {}", response.text().await?);

    Ok(())
}
