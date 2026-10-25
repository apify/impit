//! Handshakes between impit's client configs and an in-memory rustls server, with and without the
//! certificate verification cache.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::client::WebPkiServerVerifier;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName, UnixTime};
use rustls::{
    ClientConfig, ClientConnection, DigitallySignedStruct, Error, RootCertStore, ServerConfig,
    ServerConnection, SignatureScheme,
};

use super::{CachingServerCertVerifier, TlsConfig};
use crate::fingerprint::{database, TlsFingerprint};

const ROOT: &[u8] = include_bytes!("../../tests/fixtures/certcache/root.der");
const INTERMEDIATE: &[u8] = include_bytes!("../../tests/fixtures/certcache/intermediate.der");
const LEAF_A: &[u8] = include_bytes!("../../tests/fixtures/certcache/leaf-a.der");
const LEAF_A_KEY: &[u8] = include_bytes!("../../tests/fixtures/certcache/leaf-a.key.der");

fn fingerprints() -> Vec<(&'static str, TlsFingerprint)> {
    vec![
        ("chrome_151", database::chrome_151::fingerprint().tls),
        ("firefox_144", database::firefox_144::fingerprint().tls),
        ("ios_18", database::ios_18::fingerprint().tls),
    ]
}

fn impit_config(
    fingerprint: Option<TlsFingerprint>,
    cert_verification_cache: bool,
) -> ClientConfig {
    let mut builder = TlsConfig::builder();
    if let Some(fingerprint) = fingerprint {
        builder.with_tls_fingerprint(fingerprint);
    }
    builder.with_cert_verification_cache(cert_verification_cache);
    builder.build()
}

/// The ClientHello's fields with the per-connection random parts (random, session ID, key
/// shares, GREASE values, the GREASE ECH payload) normalized, and the extensions sorted, since
/// some fingerprints permute them on every connection.
fn normalized_client_hello(config: ClientConfig) -> Vec<(u16, Vec<u8>)> {
    let mut conn =
        ClientConnection::new(Arc::new(config), ServerName::try_from("a.test").unwrap()).unwrap();
    let mut records = Vec::new();
    while conn.wants_write() {
        conn.write_tls(&mut records).unwrap();
    }

    let mut handshake = Vec::new();
    let mut rest = &records[..];
    while !rest.is_empty() {
        assert_eq!(rest[0], 22, "handshake record");
        let len = u16::from_be_bytes([rest[3], rest[4]]) as usize;
        handshake.extend_from_slice(&rest[5..5 + len]);
        rest = &rest[5 + len..];
    }
    assert_eq!(handshake[0], 1, "ClientHello");

    let grease = |v: u16| {
        if v & 0x0f0f == 0x0a0a && v >> 8 == v & 0xff {
            0x0a0a
        } else {
            v
        }
    };
    let u16_list = |data: &[u8]| -> Vec<u8> {
        data.chunks(2)
            .flat_map(|c| grease(u16::from_be_bytes([c[0], c[1]])).to_be_bytes())
            .collect()
    };

    let body = &handshake[4..];
    let mut out = vec![(0xffff_u16, body[..2].to_vec())]; // legacy_version
    let mut p = 2 + 32; // random
    p += 1 + body[p] as usize; // legacy_session_id
    let suites_len = u16::from_be_bytes([body[p], body[p + 1]]) as usize;
    out.push((0xfffe, u16_list(&body[p + 2..p + 2 + suites_len])));
    p += 2 + suites_len;
    out.push((0xfffd, body[p + 1..p + 1 + body[p] as usize].to_vec())); // compression
    p += 1 + body[p] as usize;
    let ext_end = p + 2 + u16::from_be_bytes([body[p], body[p + 1]]) as usize;
    p += 2;

    let mut extensions = Vec::new();
    while p < ext_end {
        let ty = grease(u16::from_be_bytes([body[p], body[p + 1]]));
        let len = u16::from_be_bytes([body[p + 2], body[p + 3]]) as usize;
        let data = &body[p + 4..p + 4 + len];
        let data = match ty {
            // supported_groups
            10 => u16_list(data),
            // supported_versions
            43 => [&data[..1], &u16_list(&data[1..])].concat(),
            // key_share: only the groups and key lengths
            51 => {
                let mut groups = Vec::new();
                let mut q = 2;
                while q < data.len() {
                    let group = grease(u16::from_be_bytes([data[q], data[q + 1]]));
                    let key_len = u16::from_be_bytes([data[q + 2], data[q + 3]]);
                    groups.extend(group.to_be_bytes());
                    groups.extend(key_len.to_be_bytes());
                    q += 4 + key_len as usize;
                }
                groups
            }
            // encrypted_client_hello (GREASE): random config ID, key and payload
            0xfe0d => Vec::new(),
            _ => data.to_vec(),
        };
        extensions.push((ty, data));
        p += 4 + len;
    }
    extensions.sort();
    out.extend(extensions);
    out
}

#[test]
fn client_hello_unaffected_by_cert_verification_cache() {
    let mut configs: Vec<(&str, Option<TlsFingerprint>)> = fingerprints()
        .into_iter()
        .map(|(name, fp)| (name, Some(fp)))
        .collect();
    configs.push(("vanilla", None));
    for (name, fingerprint) in configs {
        for _ in 0..5 {
            let off = normalized_client_hello(impit_config(fingerprint.clone(), false));
            let on = normalized_client_hello(impit_config(fingerprint.clone(), true));
            assert_eq!(off, on, "{name}");
            assert!(
                on.iter().any(|(ty, _)| *ty == 13),
                "{name}: signature_algorithms"
            );
        }
    }
}

#[test]
fn cert_verification_cache_is_on_by_default() {
    let config = TlsConfig::builder().build();
    assert!(format!("{config:?}").contains("CachingServerCertVerifier"));
    let config = impit_config(Some(database::chrome_151::fingerprint().tls), false);
    assert!(!format!("{config:?}").contains("CachingServerCertVerifier"));
    let mut builder = TlsConfig::builder();
    builder.with_ignore_tls_errors(true);
    assert!(!format!("{:?}", builder.build()).contains("CachingServerCertVerifier"));
}

/// Counts chain and handshake signature verifications reaching the real verifier.
#[derive(Debug)]
struct Counting {
    inner: Arc<WebPkiServerVerifier>,
    chain: AtomicUsize,
    signatures: AtomicUsize,
}

impl ServerCertVerifier for Counting {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        ocsp_response: &[u8],
        now: UnixTime,
    ) -> Result<ServerCertVerified, Error> {
        self.chain.fetch_add(1, Ordering::SeqCst);
        self.inner
            .verify_server_cert(end_entity, intermediates, server_name, ocsp_response, now)
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        self.signatures.fetch_add(1, Ordering::SeqCst);
        self.inner.verify_tls12_signature(message, cert, dss)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        self.signatures.fetch_add(1, Ordering::SeqCst);
        self.inner.verify_tls13_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.inner.supported_verify_schemes()
    }
}

fn test_root_verifier(config: &ClientConfig) -> Arc<WebPkiServerVerifier> {
    let mut roots = RootCertStore::empty();
    roots.add(CertificateDer::from(ROOT)).unwrap();
    WebPkiServerVerifier::builder_with_provider(roots.into(), config.crypto_provider().clone())
        .build()
        .unwrap()
}

/// A server presenting leaf-a (ECDSA P-256) and the RSA intermediate.
fn server_config() -> Arc<ServerConfig> {
    let config = ServerConfig::builder_with_provider(Arc::new(
        rustls::crypto::aws_lc_rs::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .unwrap()
    .with_no_client_auth()
    .with_single_cert(
        vec![
            CertificateDer::from(LEAF_A),
            CertificateDer::from(INTERMEDIATE),
        ],
        PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(LEAF_A_KEY)),
    )
    .unwrap();
    Arc::new(config)
}

/// Runs a handshake in memory; returns the time spent in the client's calls.
fn handshake(
    client: &Arc<ClientConfig>,
    server: &Arc<ServerConfig>,
    name: &str,
) -> Result<Duration, Error> {
    let mut buf = Vec::new();
    let start = Instant::now();
    let mut c = ClientConnection::new(
        client.clone(),
        ServerName::try_from(name.to_owned()).unwrap(),
    )?;
    let mut client_time = start.elapsed();
    let mut s = ServerConnection::new(server.clone())?;
    while c.is_handshaking() || s.is_handshaking() {
        buf.clear();
        let start = Instant::now();
        while c.wants_write() {
            c.write_tls(&mut buf).unwrap();
        }
        client_time += start.elapsed();
        if !buf.is_empty() {
            s.read_tls(&mut &buf[..]).unwrap();
            s.process_new_packets()?;
        }

        buf.clear();
        while s.wants_write() {
            s.write_tls(&mut buf).unwrap();
        }
        if buf.is_empty() && !c.wants_write() {
            break;
        }
        let start = Instant::now();
        if !buf.is_empty() {
            c.read_tls(&mut &buf[..]).unwrap();
            c.process_new_packets()?;
        }
        client_time += start.elapsed();
    }
    assert!(!c.is_handshaking());
    Ok(client_time)
}

fn client_with_verifier(
    fingerprint: TlsFingerprint,
    verifier: Arc<dyn ServerCertVerifier>,
) -> Arc<ClientConfig> {
    let mut config = impit_config(Some(fingerprint), false);
    config.dangerous().set_certificate_verifier(verifier);
    Arc::new(config)
}

#[test]
fn cached_handshakes_verify_chain_once_and_signature_every_time() {
    let server = server_config();
    // ios_18 sends its GREASE extension twice (0xbaba), which rustls servers reject.
    for (name, fingerprint) in fingerprints().into_iter().filter(|(n, _)| *n != "ios_18") {
        let base = impit_config(Some(fingerprint.clone()), false);
        let counting = Arc::new(Counting {
            inner: test_root_verifier(&base),
            chain: AtomicUsize::new(0),
            signatures: AtomicUsize::new(0),
        });
        let caching: Arc<dyn ServerCertVerifier> =
            Arc::new(CachingServerCertVerifier::new(counting.clone()));
        // A new client config per connection, as with a new client per proxy session.
        for _ in 0..3 {
            let client = client_with_verifier(fingerprint.clone(), caching.clone());
            handshake(&client, &server, "a.test").unwrap();
        }
        assert_eq!(counting.chain.load(Ordering::SeqCst), 1, "{name}");
        assert_eq!(counting.signatures.load(Ordering::SeqCst), 3, "{name}");

        // The cached chain doesn't make a name it isn't valid for acceptable.
        let client = client_with_verifier(fingerprint.clone(), caching.clone());
        handshake(&client, &server, "b.test").unwrap_err();
    }
}

/// Client-side CPU of a full handshake with and without the cache. Run with
/// `cargo test --release -p impit handshake_benchmark -- --ignored --nocapture`.
#[test]
#[ignore]
fn handshake_benchmark() {
    const N: u32 = 2000;
    let server = server_config();
    let fingerprint = database::chrome_151::fingerprint().tls;
    let base = impit_config(Some(fingerprint.clone()), false);
    let plain: Arc<dyn ServerCertVerifier> = test_root_verifier(&base);
    let caching: Arc<dyn ServerCertVerifier> =
        Arc::new(CachingServerCertVerifier::new(plain.clone()));

    let chain = [CertificateDer::from(INTERMEDIATE)];
    let leaf = CertificateDer::from(LEAF_A);
    let server_name = ServerName::try_from("a.test").unwrap();
    for (label, verifier) in [("off", &plain), ("on", &caching)] {
        // Every handshake is a full one, as with a new client per proxy session.
        let mut client = client_with_verifier(fingerprint.clone(), verifier.clone());
        Arc::make_mut(&mut client).resumption = rustls::client::Resumption::disabled();
        for _ in 0..200 {
            handshake(&client, &server, "a.test").unwrap();
        }
        let mut best = Duration::MAX;
        for _ in 0..5 {
            let mut total = Duration::ZERO;
            for _ in 0..N {
                total += handshake(&client, &server, "a.test").unwrap();
            }
            best = best.min(total / N);
        }

        let start = Instant::now();
        for _ in 0..N {
            verifier
                .verify_server_cert(&leaf, &chain, &server_name, &[], UnixTime::now())
                .unwrap();
        }
        let verify = start.elapsed() / N;
        println!(
            "cache {label}: client handshake {:.1} us, verify_server_cert {:.2} us",
            best.as_secs_f64() * 1e6,
            verify.as_secs_f64() * 1e6
        );
    }
}
