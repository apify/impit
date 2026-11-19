//! Pre-defined browser fingerprints
//!
//! This module contains fingerprint definitions for various browsers.

mod chrome;
mod firefox;
mod okhttp;
mod safari;

pub use chrome::{
    chrome_100, chrome_101, chrome_104, chrome_107, chrome_110, chrome_116, chrome_124, chrome_125,
    chrome_131, chrome_133, chrome_136, chrome_142, chrome_151, chrome_155, chrome_156, chrome_157,
};
pub use firefox::{firefox_128, firefox_133, firefox_135, firefox_144, firefox_158};
pub use okhttp::{okhttp3, okhttp4, okhttp5};
pub use safari::ios_18;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fingerprint::BrowserFingerprint;
    use crate::tls::TlsConfig;
    use std::sync::Arc;

    /// The ClientHello that impit sends for `fingerprint`: legacy version, cipher suites, and each extension's type
    /// and body in wire order.
    pub(super) fn client_hello(
        fingerprint: BrowserFingerprint,
    ) -> (u16, Vec<u16>, Vec<(u16, Vec<u8>)>) {
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

    /// Extension types of the ClientHello that impit sends for `fingerprint`, in wire order.
    fn client_hello_extension_types(fingerprint: BrowserFingerprint) -> Vec<u16> {
        let (_, _, extensions) = client_hello(fingerprint);
        extensions.into_iter().map(|(typ, _)| typ).collect()
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
            ("chrome_155", chrome_155::fingerprint()),
            ("chrome_156", chrome_156::fingerprint()),
            ("chrome_157", chrome_157::fingerprint()),
            ("firefox_128", firefox_128::fingerprint()),
            ("firefox_133", firefox_133::fingerprint()),
            ("firefox_135", firefox_135::fingerprint()),
            ("firefox_144", firefox_144::fingerprint()),
            ("firefox_158", firefox_158::fingerprint()),
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

    // Chrome leaves out MAX_FRAME_SIZE (5), Firefox MAX_HEADER_LIST_SIZE (6), and Firefox 128 also
    // ENABLE_PUSH (2) while adding setting 9 = 1. The values are measured on the real browsers
    // (chrome_116 on Chrome for Testing 116), except chrome_100-110's: only their leaving out 5
    // comes from Chromium's source, and their `2:0` contradicts that source for 100, 101 and 104.
    // The unmeasured ios_18 and okhttp keep what they sent before: `5:16384` and `6:16384`,
    // hyper's defaults.
    #[test]
    fn every_profile_has_the_expected_h2_settings() {
        // (max_frame_size, max_header_list_size,
        //  IMPIT_H2_OMIT_SETTINGS, IMPIT_H2_NO_RFC7540_PRIORITIES)
        let chrome = (None, Some(262_144), "5", "");
        let firefox = (Some(16_384), None, "6", "");
        let firefox128 = (Some(16_384), None, "2,6", "1");
        let unchanged = (Some(16_384), Some(16_384), "", "");
        let profiles = [
            ("chrome_100", chrome_100::fingerprint(), chrome),
            ("chrome_101", chrome_101::fingerprint(), chrome),
            ("chrome_104", chrome_104::fingerprint(), chrome),
            ("chrome_107", chrome_107::fingerprint(), chrome),
            ("chrome_110", chrome_110::fingerprint(), chrome),
            ("chrome_116", chrome_116::fingerprint(), chrome),
            ("chrome_124", chrome_124::fingerprint(), chrome),
            ("chrome_125", chrome_125::fingerprint(), chrome),
            ("chrome_131", chrome_131::fingerprint(), chrome),
            ("chrome_133", chrome_133::fingerprint(), chrome),
            ("chrome_136", chrome_136::fingerprint(), chrome),
            ("chrome_142", chrome_142::fingerprint(), chrome),
            ("chrome_151", chrome_151::fingerprint(), chrome),
            ("chrome_155", chrome_155::fingerprint(), chrome),
            ("chrome_156", chrome_156::fingerprint(), chrome),
            ("chrome_157", chrome_157::fingerprint(), chrome),
            ("firefox_128", firefox_128::fingerprint(), firefox128),
            ("firefox_133", firefox_133::fingerprint(), firefox),
            ("firefox_135", firefox_135::fingerprint(), firefox),
            ("firefox_144", firefox_144::fingerprint(), firefox),
            ("firefox_158", firefox_158::fingerprint(), firefox),
            ("okhttp3", okhttp3::fingerprint(), unchanged),
            ("okhttp4", okhttp4::fingerprint(), unchanged),
            ("okhttp5", okhttp5::fingerprint(), unchanged),
            ("ios_18", ios_18::fingerprint(), unchanged),
        ];

        let mut wrong = Vec::new();
        for (name, fingerprint, settings) in profiles {
            let (max_frame_size, max_header_list_size, omit, no_rfc7540) = settings;
            let http2 = &fingerprint.http2;
            let actual = (
                http2.max_frame_size,
                http2.max_header_list_size,
                http2.h2_settings_env(),
            );
            let expected = (
                max_frame_size,
                max_header_list_size,
                [
                    ("IMPIT_H2_OMIT_SETTINGS", omit.to_string()),
                    ("IMPIT_H2_NO_RFC7540_PRIORITIES", no_rfc7540.to_string()),
                ],
            );
            if actual != expected {
                wrong.push(format!("{name}: got {actual:?}, expected {expected:?}"));
            }
        }
        assert!(wrong.is_empty(), "wrong HTTP/2 SETTINGS: {wrong:#?}");
    }
}
