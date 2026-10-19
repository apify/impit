//! Pre-defined browser fingerprints
//!
//! This module contains fingerprint definitions for various browsers.

mod chrome;
mod firefox;
mod okhttp;
mod safari;

pub use chrome::{
    chrome_100, chrome_101, chrome_104, chrome_107, chrome_110, chrome_116, chrome_124, chrome_125,
    chrome_131, chrome_133, chrome_136, chrome_142, chrome_151,
};
pub use firefox::{firefox_128, firefox_133, firefox_135, firefox_144};
pub use okhttp::{okhttp3, okhttp4, okhttp5};
pub use safari::ios_18;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fingerprint::BrowserFingerprint;
    use crate::tls::TlsConfig;
    use std::sync::Arc;

    /// Extension types of the ClientHello that impit sends for `fingerprint`, in wire order.
    fn client_hello_extension_types(fingerprint: BrowserFingerprint) -> Vec<u16> {
        let mut builder = TlsConfig::builder();
        builder.with_tls_fingerprint(fingerprint.tls);
        let server_name = rustls::pki_types::ServerName::try_from("example.com").unwrap();
        let mut conn =
            rustls::ClientConnection::new(Arc::new(builder.build()), server_name).unwrap();
        let mut record = Vec::new();
        conn.write_tls(&mut record).unwrap();

        // Skip the record (5) and handshake (4) headers, legacy_version (2) and random (32).
        let hello = &record[9..];
        let u16_at = |i: usize| usize::from(u16::from_be_bytes([hello[i], hello[i + 1]]));
        let mut i = 2 + 32;
        i += 1 + usize::from(hello[i]); // legacy_session_id
        i += 2 + u16_at(i); // cipher_suites
        i += 1 + usize::from(hello[i]); // legacy_compression_methods
        let end = i + 2 + u16_at(i);
        i += 2;
        let mut types = Vec::new();
        while i < end {
            types.push(u16_at(i) as u16);
            i += 4 + u16_at(i + 2);
        }
        types
    }

    // TLS 1.3 forbids repeating an extension type, and BoringSSL-based servers (Google, Cloudflare)
    // reject such a ClientHello with `decode_error` (#575).
    #[test]
    fn no_profile_sends_an_extension_type_twice() {
        let profiles = [
            ("chrome_100", chrome_100::fingerprint()),
            ("chrome_101", chrome_101::fingerprint()),
            ("chrome_104", chrome_104::fingerprint()),
            ("chrome_107", chrome_107::fingerprint()),
            ("chrome_110", chrome_110::fingerprint()),
            ("chrome_116", chrome_116::fingerprint()),
            ("chrome_124", chrome_124::fingerprint()),
            ("chrome_125", chrome_125::fingerprint()),
            ("chrome_131", chrome_131::fingerprint()),
            ("chrome_133", chrome_133::fingerprint()),
            ("chrome_136", chrome_136::fingerprint()),
            ("chrome_142", chrome_142::fingerprint()),
            ("chrome_151", chrome_151::fingerprint()),
            ("firefox_128", firefox_128::fingerprint()),
            ("firefox_133", firefox_133::fingerprint()),
            ("firefox_135", firefox_135::fingerprint()),
            ("firefox_144", firefox_144::fingerprint()),
            ("okhttp3", okhttp3::fingerprint()),
            ("okhttp4", okhttp4::fingerprint()),
            ("okhttp5", okhttp5::fingerprint()),
            ("ios_18", ios_18::fingerprint()),
        ];

        let mut duplicates = Vec::new();
        for (name, fingerprint) in profiles {
            let types = client_hello_extension_types(fingerprint);
            assert!(
                types.contains(&0x0000),
                "{name}: no server_name, ClientHello not parsed"
            );
            for (i, typ) in types.iter().enumerate() {
                if types[..i].contains(typ) {
                    duplicates.push(format!("{name}: {typ:#06x}"));
                }
            }
        }
        assert!(
            duplicates.is_empty(),
            "repeated ClientHello extensions: {duplicates:?}"
        );
    }
}
