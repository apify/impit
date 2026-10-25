//! A [`ServerCertVerifier`] that remembers successful certificate-chain verifications.
//!
//! Every new connection to a server verifies the same chain again: parsing the certificates,
//! building the path to a root and checking each signature (RSA intermediates are the most
//! expensive part). Clients that open many short-lived connections, e.g. one per proxy session,
//! spend a noticeable share of their CPU on that. [`CachingServerCertVerifier`] skips the chain
//! check when the same server name presented the same certificates (and stapled OCSP response)
//! recently, and the earlier verification succeeded.
//!
//! Nothing changes on the wire: the ClientHello is the same and no session state is kept for
//! the server to recognize. The handshake signature (`verify_tls12_signature` /
//! `verify_tls13_signature`), which proves the server holds the certificate's key, is checked on
//! every connection.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use aws_lc_rs::digest::{Context, SHA256, SHA256_OUTPUT_LEN};
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{DigitallySignedStruct, DistinguishedName, Error, SignatureScheme};

/// Default number of verified chains to remember.
pub(crate) const DEFAULT_CAPACITY: usize = 256;

/// Default for how long a verification is reused, unless a certificate expires sooner.
///
/// Bounds how long a chain stays trusted after the inner verifier would have started rejecting
/// it for a reason other than expiry (e.g. a changed root store).
pub(crate) const DEFAULT_TTL: Duration = Duration::from_secs(10 * 60);

type CacheKey = (ServerName<'static>, [u8; SHA256_OUTPUT_LEN]);

struct CacheEntry {
    /// When the chain was verified (Unix seconds); a clock that went back before it misses.
    verified_at: u64,
    /// The entry is valid while `now < expires_at` (Unix seconds): the earlier of the TTL and
    /// the first `notAfter` in the chain.
    expires_at: u64,
    /// Value of [`Cache::tick`] at the last hit, for LRU eviction.
    last_used: u64,
}

struct Cache {
    entries: HashMap<CacheKey, CacheEntry>,
    tick: u64,
}

/// Wraps a [`ServerCertVerifier`] and caches its successful `verify_server_cert` results in a
/// bounded LRU, keyed by the server name and a SHA-256 hash of the end-entity certificate,
/// the intermediates and the stapled OCSP response.
///
/// A result is reused until the TTL passes or any certificate the server sent reaches its
/// `notAfter`, whichever comes first. Failed verifications are never cached. All other trait
/// methods, including the handshake signature checks, go to the inner verifier every time.
pub struct CachingServerCertVerifier {
    inner: Arc<dyn ServerCertVerifier>,
    capacity: usize,
    ttl_secs: u64,
    cache: Mutex<Cache>,
}

impl CachingServerCertVerifier {
    /// Wraps `inner` with the default capacity (256 chains) and TTL (10 minutes).
    pub fn new(inner: Arc<dyn ServerCertVerifier>) -> Self {
        Self::with_limits(inner, DEFAULT_CAPACITY, DEFAULT_TTL)
    }

    /// Wraps `inner`, remembering at most `capacity` chains for at most `ttl` each.
    pub fn with_limits(inner: Arc<dyn ServerCertVerifier>, capacity: usize, ttl: Duration) -> Self {
        Self {
            inner,
            capacity: capacity.max(1),
            ttl_secs: ttl.as_secs(),
            cache: Mutex::new(Cache {
                entries: HashMap::new(),
                tick: 0,
            }),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Cache> {
        self.cache.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Returns whether a still-valid successful verification of this chain is cached.
    fn lookup(&self, key: &CacheKey, now: u64) -> bool {
        let mut cache = self.lock();
        cache.tick += 1;
        let tick = cache.tick;
        match cache.entries.get_mut(key) {
            Some(entry) if entry.verified_at <= now && now < entry.expires_at => {
                entry.last_used = tick;
                true
            }
            Some(_) => {
                cache.entries.remove(key);
                false
            }
            None => false,
        }
    }

    fn insert(&self, key: CacheKey, verified_at: u64, expires_at: u64) {
        let mut cache = self.lock();
        if cache.entries.len() >= self.capacity && !cache.entries.contains_key(&key) {
            cache.entries.retain(|_, e| verified_at < e.expires_at);
            if cache.entries.len() >= self.capacity {
                let oldest = cache
                    .entries
                    .iter()
                    .min_by_key(|(_, e)| e.last_used)
                    .map(|(k, _)| k.clone());
                if let Some(oldest) = oldest {
                    cache.entries.remove(&oldest);
                }
            }
        }
        cache.tick += 1;
        let last_used = cache.tick;
        cache.entries.insert(
            key,
            CacheEntry {
                verified_at,
                expires_at,
                last_used,
            },
        );
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.lock().entries.len()
    }
}

impl fmt::Debug for CachingServerCertVerifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CachingServerCertVerifier")
            .field("inner", &self.inner)
            .field("capacity", &self.capacity)
            .field("ttl_secs", &self.ttl_secs)
            .finish_non_exhaustive()
    }
}

fn chain_hash(
    end_entity: &CertificateDer<'_>,
    intermediates: &[CertificateDer<'_>],
    ocsp_response: &[u8],
) -> [u8; SHA256_OUTPUT_LEN] {
    let mut ctx = Context::new(&SHA256);
    // Length prefixes keep the encoding unambiguous.
    let mut add = |bytes: &[u8]| {
        ctx.update(&(bytes.len() as u64).to_be_bytes());
        ctx.update(bytes);
    };
    add(end_entity);
    for cert in intermediates {
        add(cert);
    }
    add(ocsp_response);
    let mut out = [0; SHA256_OUTPUT_LEN];
    out.copy_from_slice(ctx.finish().as_ref());
    out
}

impl ServerCertVerifier for CachingServerCertVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        ocsp_response: &[u8],
        now: UnixTime,
    ) -> Result<ServerCertVerified, Error> {
        let key = (
            server_name.to_owned(),
            chain_hash(end_entity, intermediates, ocsp_response),
        );
        let now_secs = now.as_secs();
        if self.lookup(&key, now_secs) {
            return Ok(ServerCertVerified::assertion());
        }

        let verified = self.inner.verify_server_cert(
            end_entity,
            intermediates,
            server_name,
            ocsp_response,
            now,
        )?;

        // Not cached if any certificate's notAfter can't be read.
        let not_after = std::iter::once(end_entity)
            .chain(intermediates)
            .map(|cert| not_after(cert))
            .try_fold(u64::MAX, |min, t| t.map(|t| min.min(t)));
        if let Some(not_after) = not_after {
            let expires_at = not_after.min(now_secs.saturating_add(self.ttl_secs));
            if now_secs < expires_at {
                self.insert(key, now_secs, expires_at);
            }
        }
        Ok(verified)
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        self.inner.verify_tls12_signature(message, cert, dss)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        self.inner.verify_tls13_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.inner.supported_verify_schemes()
    }

    fn requires_raw_public_keys(&self) -> bool {
        self.inner.requires_raw_public_keys()
    }

    fn root_hint_subjects(&self) -> Option<&[DistinguishedName]> {
        self.inner.root_hint_subjects()
    }
}

/// Reads one DER TLV: returns the tag, the contents and the rest of the input.
fn der_tlv(input: &[u8]) -> Option<(u8, &[u8], &[u8])> {
    let (&tag, rest) = input.split_first()?;
    let (&first, mut rest) = rest.split_first()?;
    let len = if first < 0x80 {
        first as usize
    } else {
        let n = (first & 0x7f) as usize;
        if n == 0 || n > 4 || rest.len() < n {
            return None;
        }
        let len = rest[..n]
            .iter()
            .fold(0usize, |len, &b| (len << 8) | b as usize);
        rest = &rest[n..];
        len
    };
    (rest.len() >= len).then(|| (tag, &rest[..len], &rest[len..]))
}

/// The certificate's `notAfter` as Unix seconds, or `None` if it can't be parsed.
fn not_after(cert: &[u8]) -> Option<u64> {
    const SEQUENCE: u8 = 0x30;
    const INTEGER: u8 = 0x02;
    const VERSION: u8 = 0xa0;

    let expect = |input, tag| der_tlv(input).filter(|(t, _, _)| *t == tag);
    let (_, cert, _) = expect(cert, SEQUENCE)?;
    let (_, mut tbs, _) = expect(cert, SEQUENCE)?;
    if tbs.first() == Some(&VERSION) {
        tbs = der_tlv(tbs)?.2;
    }
    // serialNumber, signature, issuer
    for tag in [INTEGER, SEQUENCE, SEQUENCE] {
        tbs = expect(tbs, tag)?.2;
    }
    let (_, validity, _) = expect(tbs, SEQUENCE)?;
    let (_, _, validity) = der_tlv(validity)?; // notBefore
    let (tag, time, _) = der_tlv(validity)?;
    parse_time(tag, time)
}

/// Parses an X.509 `UTCTime` or `GeneralizedTime` (RFC 5280 forms only) to Unix seconds.
fn parse_time(tag: u8, value: &[u8]) -> Option<u64> {
    const UTC_TIME: u8 = 0x17;
    const GENERALIZED_TIME: u8 = 0x18;

    fn num(digits: &[u8]) -> Option<u64> {
        digits.iter().try_fold(0, |n, &d| {
            d.is_ascii_digit().then(|| n * 10 + u64::from(d - b'0'))
        })
    }

    let (year, rest) = match (tag, value.len()) {
        (UTC_TIME, 13) => {
            let yy = num(&value[..2])?;
            (if yy >= 50 { 1900 + yy } else { 2000 + yy }, &value[2..])
        }
        (GENERALIZED_TIME, 15) => (num(&value[..4])?, &value[4..]),
        _ => return None,
    };
    if rest[10] != b'Z' {
        return None;
    }
    let [month, day, hour, minute, second] = [0, 2, 4, 6, 8].map(|i| num(&rest[i..i + 2]));
    let (month, day, hour, minute, second) = (month?, day?, hour?, minute?, second?);
    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || hour > 23
        || minute > 59
        || second > 59
        || year < 1970
    {
        return None;
    }

    // Days since 1970-01-01 in the proleptic Gregorian calendar (years before 1970 rejected above).
    let y = if month <= 2 { year - 1 } else { year };
    let (era, yoe) = (y / 400, y % 400);
    let doy = (153 * ((month + 9) % 12) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = (era * 146_097 + doe).checked_sub(719_468)?;
    Some(days * 86_400 + hour * 3_600 + minute * 60 + second)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    const ROOT: &[u8] = include_bytes!("../../tests/fixtures/certcache/root.der");
    const INTERMEDIATE: &[u8] = include_bytes!("../../tests/fixtures/certcache/intermediate.der");
    const LEAF_A: &[u8] = include_bytes!("../../tests/fixtures/certcache/leaf-a.der");
    const LEAF_B: &[u8] = include_bytes!("../../tests/fixtures/certcache/leaf-b.der");
    const LEAF_SHORT: &[u8] = include_bytes!("../../tests/fixtures/certcache/leaf-short.der");

    /// notAfter of the fixtures (see `gen.sh`).
    const LEAF_A_NOT_AFTER: u64 = 3_520_961_356; // 2081-07-28T20:49:16Z, GeneralizedTime
    const LEAF_SHORT_NOT_AFTER: u64 = 1_879_361_356; // 2029-07-21T20:49:16Z, UTCTime
    const INTERMEDIATE_NOT_AFTER: u64 = 4_384_961_355; // 2108-12-14T20:49:15Z

    /// 2027-01-01T00:00:00Z, after the fixtures' notBefore.
    const T0: u64 = 1_798_761_600;

    /// Counts `verify_server_cert` calls and succeeds unless told to fail.
    #[derive(Debug, Default)]
    struct CountingVerifier {
        calls: AtomicUsize,
        fail: std::sync::atomic::AtomicBool,
    }

    impl CountingVerifier {
        fn calls(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    impl ServerCertVerifier for CountingVerifier {
        fn verify_server_cert(
            &self,
            _: &CertificateDer<'_>,
            _: &[CertificateDer<'_>],
            _: &ServerName<'_>,
            _: &[u8],
            _: UnixTime,
        ) -> Result<ServerCertVerified, Error> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if self.fail.load(Ordering::SeqCst) {
                Err(Error::InvalidCertificate(
                    rustls::CertificateError::UnknownIssuer,
                ))
            } else {
                Ok(ServerCertVerified::assertion())
            }
        }

        fn verify_tls12_signature(
            &self,
            _: &[u8],
            _: &CertificateDer<'_>,
            _: &DigitallySignedStruct,
        ) -> Result<HandshakeSignatureValid, Error> {
            unimplemented!()
        }

        fn verify_tls13_signature(
            &self,
            _: &[u8],
            _: &CertificateDer<'_>,
            _: &DigitallySignedStruct,
        ) -> Result<HandshakeSignatureValid, Error> {
            unimplemented!()
        }

        fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
            vec![SignatureScheme::ECDSA_NISTP256_SHA256]
        }
    }

    fn setup() -> (Arc<CountingVerifier>, CachingServerCertVerifier) {
        let inner = Arc::new(CountingVerifier::default());
        let caching = CachingServerCertVerifier::new(inner.clone());
        (inner, caching)
    }

    fn name(s: &str) -> ServerName<'static> {
        ServerName::try_from(s.to_owned()).unwrap()
    }

    fn at(secs: u64) -> UnixTime {
        UnixTime::since_unix_epoch(Duration::from_secs(secs))
    }

    fn verify(
        v: &CachingServerCertVerifier,
        leaf: &[u8],
        intermediates: &[&[u8]],
        host: &str,
        ocsp: &[u8],
        now: u64,
    ) -> Result<ServerCertVerified, Error> {
        let intermediates: Vec<_> = intermediates
            .iter()
            .map(|c| CertificateDer::from(*c))
            .collect();
        v.verify_server_cert(
            &CertificateDer::from(leaf),
            &intermediates,
            &name(host),
            ocsp,
            at(now),
        )
    }

    #[test]
    fn parses_not_after_in_both_time_formats() {
        assert_eq!(not_after(LEAF_A), Some(LEAF_A_NOT_AFTER));
        assert_eq!(not_after(LEAF_SHORT), Some(LEAF_SHORT_NOT_AFTER));
        assert_eq!(not_after(INTERMEDIATE), Some(INTERMEDIATE_NOT_AFTER));
        assert_eq!(not_after(ROOT), Some(4_946_561_355)); // 2126-10-01T20:49:15Z
        assert_eq!(not_after(b"\x30\x03\x02\x01\x00"), None);
        assert_eq!(not_after(&LEAF_A[..LEAF_A.len() / 2]), None);
        assert_eq!(parse_time(0x17, b"700101000000Z"), Some(0));
        assert_eq!(parse_time(0x17, b"491231235959Z"), Some(2_524_607_999));
        assert_eq!(parse_time(0x18, b"20000229120000Z"), Some(951_825_600));
        assert_eq!(parse_time(0x17, b"700101000000+0100"), None);
        assert_eq!(parse_time(0x18, b"20001301000000Z"), None);
    }

    #[test]
    fn hit_skips_inner_verifier() {
        let (inner, v) = setup();
        verify(&v, LEAF_A, &[INTERMEDIATE], "a.test", b"", T0).unwrap();
        verify(&v, LEAF_A, &[INTERMEDIATE], "a.test", b"", T0 + 60).unwrap();
        verify(&v, LEAF_A, &[INTERMEDIATE], "a.test", b"", T0 + 120).unwrap();
        assert_eq!(inner.calls(), 1);
    }

    // The handshake signature checks are covered by `handshake_tests`, since
    // `DigitallySignedStruct` can't be built outside rustls.
    #[test]
    fn delegates_other_methods() {
        let (inner, v) = setup();
        assert_eq!(
            v.supported_verify_schemes(),
            inner.supported_verify_schemes()
        );
        assert_eq!(
            v.requires_raw_public_keys(),
            inner.requires_raw_public_keys()
        );
        assert!(v.root_hint_subjects().is_none());
    }

    #[test]
    fn different_chain_name_or_ocsp_misses() {
        let (inner, v) = setup();
        verify(&v, LEAF_A, &[INTERMEDIATE], "a.test", b"", T0).unwrap();
        assert_eq!(inner.calls(), 1);
        // Different end-entity certificate.
        verify(&v, LEAF_B, &[INTERMEDIATE], "a.test", b"", T0).unwrap();
        assert_eq!(inner.calls(), 2);
        // Same end-entity certificate, different intermediates.
        verify(&v, LEAF_A, &[], "a.test", b"", T0).unwrap();
        assert_eq!(inner.calls(), 3);
        verify(&v, LEAF_A, &[INTERMEDIATE, ROOT], "a.test", b"", T0).unwrap();
        assert_eq!(inner.calls(), 4);
        // Different server name.
        verify(&v, LEAF_A, &[INTERMEDIATE], "localhost", b"", T0).unwrap();
        assert_eq!(inner.calls(), 5);
        // Different stapled OCSP response.
        verify(&v, LEAF_A, &[INTERMEDIATE], "a.test", b"ocsp", T0).unwrap();
        assert_eq!(inner.calls(), 6);
        // All of them are cached now.
        verify(&v, LEAF_B, &[INTERMEDIATE], "a.test", b"", T0).unwrap();
        verify(&v, LEAF_A, &[INTERMEDIATE], "localhost", b"", T0).unwrap();
        verify(&v, LEAF_A, &[INTERMEDIATE], "a.test", b"ocsp", T0).unwrap();
        assert_eq!(inner.calls(), 6);
    }

    #[test]
    fn entry_expires_after_ttl() {
        let inner = Arc::new(CountingVerifier::default());
        let v = CachingServerCertVerifier::with_limits(inner.clone(), 16, Duration::from_secs(600));
        verify(&v, LEAF_A, &[INTERMEDIATE], "a.test", b"", T0).unwrap();
        verify(&v, LEAF_A, &[INTERMEDIATE], "a.test", b"", T0 + 599).unwrap();
        assert_eq!(inner.calls(), 1);
        verify(&v, LEAF_A, &[INTERMEDIATE], "a.test", b"", T0 + 600).unwrap();
        assert_eq!(inner.calls(), 2);
        // Re-cached from the new verification.
        verify(&v, LEAF_A, &[INTERMEDIATE], "a.test", b"", T0 + 1199).unwrap();
        assert_eq!(inner.calls(), 2);
    }

    #[test]
    fn entry_expires_at_not_after() {
        let (inner, v) = setup();
        let t = LEAF_SHORT_NOT_AFTER - 30;
        verify(&v, LEAF_SHORT, &[INTERMEDIATE], "a.test", b"", t).unwrap();
        verify(&v, LEAF_SHORT, &[INTERMEDIATE], "a.test", b"", t + 29).unwrap();
        assert_eq!(inner.calls(), 1);
        // The TTL hasn't passed, but the certificate has expired: back to the inner verifier.
        verify(
            &v,
            LEAF_SHORT,
            &[INTERMEDIATE],
            "a.test",
            b"",
            LEAF_SHORT_NOT_AFTER,
        )
        .unwrap();
        assert_eq!(inner.calls(), 2);
        verify(
            &v,
            LEAF_SHORT,
            &[INTERMEDIATE],
            "a.test",
            b"",
            LEAF_SHORT_NOT_AFTER + 1,
        )
        .unwrap();
        assert_eq!(inner.calls(), 3);
    }

    #[test]
    fn clock_going_back_misses() {
        let (inner, v) = setup();
        verify(&v, LEAF_A, &[INTERMEDIATE], "a.test", b"", T0).unwrap();
        verify(&v, LEAF_A, &[INTERMEDIATE], "a.test", b"", T0 - 1).unwrap();
        assert_eq!(inner.calls(), 2);
    }

    #[test]
    fn failure_is_never_cached() {
        let (inner, v) = setup();
        inner.fail.store(true, Ordering::SeqCst);
        verify(&v, LEAF_A, &[INTERMEDIATE], "a.test", b"", T0).unwrap_err();
        verify(&v, LEAF_A, &[INTERMEDIATE], "a.test", b"", T0).unwrap_err();
        assert_eq!(inner.calls(), 2);
        assert_eq!(v.len(), 0);
        inner.fail.store(false, Ordering::SeqCst);
        verify(&v, LEAF_A, &[INTERMEDIATE], "a.test", b"", T0).unwrap();
        verify(&v, LEAF_A, &[INTERMEDIATE], "a.test", b"", T0).unwrap();
        assert_eq!(inner.calls(), 3);
    }

    #[test]
    fn unparsable_certificate_is_not_cached() {
        let (inner, v) = setup();
        verify(&v, b"not a certificate", &[INTERMEDIATE], "a.test", b"", T0).unwrap();
        verify(&v, b"not a certificate", &[INTERMEDIATE], "a.test", b"", T0).unwrap();
        assert_eq!(inner.calls(), 2);
        assert_eq!(v.len(), 0);
    }

    #[test]
    fn evicts_least_recently_used() {
        let inner = Arc::new(CountingVerifier::default());
        let v = CachingServerCertVerifier::with_limits(inner.clone(), 2, DEFAULT_TTL);
        verify(&v, LEAF_A, &[INTERMEDIATE], "a.test", b"", T0).unwrap(); // miss: a
        verify(&v, LEAF_B, &[INTERMEDIATE], "a.test", b"", T0).unwrap(); // miss: a, b
        verify(&v, LEAF_A, &[INTERMEDIATE], "a.test", b"", T0).unwrap(); // hit a
        verify(&v, LEAF_A, &[INTERMEDIATE], "localhost", b"", T0).unwrap(); // miss, evicts b
        assert_eq!(inner.calls(), 3);
        assert_eq!(v.len(), 2);
        verify(&v, LEAF_A, &[INTERMEDIATE], "a.test", b"", T0).unwrap(); // hit
        verify(&v, LEAF_A, &[INTERMEDIATE], "localhost", b"", T0).unwrap(); // hit
        assert_eq!(inner.calls(), 3);
        verify(&v, LEAF_B, &[INTERMEDIATE], "a.test", b"", T0).unwrap(); // evicted: miss
        assert_eq!(inner.calls(), 4);
    }

    #[test]
    fn real_verifier_hit_and_rejection() {
        let mut roots = rustls::RootCertStore::empty();
        roots.add(CertificateDer::from(ROOT)).unwrap();
        let inner = rustls::client::WebPkiServerVerifier::builder_with_provider(
            roots.into(),
            Arc::new(rustls::crypto::aws_lc_rs::default_provider()),
        )
        .build()
        .unwrap();
        let v = CachingServerCertVerifier::new(inner);
        verify(&v, LEAF_A, &[INTERMEDIATE], "a.test", b"", T0).unwrap();
        verify(&v, LEAF_A, &[INTERMEDIATE], "a.test", b"", T0).unwrap();
        assert_eq!(v.len(), 1);
        // A name the certificate isn't valid for is rejected, also with its chain cached.
        verify(&v, LEAF_A, &[INTERMEDIATE], "b.test", b"", T0).unwrap_err();
        // Missing intermediate.
        verify(&v, LEAF_A, &[], "a.test", b"", T0).unwrap_err();
        // Expired.
        verify(
            &v,
            LEAF_SHORT,
            &[INTERMEDIATE],
            "a.test",
            b"",
            LEAF_SHORT_NOT_AFTER + 1,
        )
        .unwrap_err();
        assert_eq!(v.len(), 1);
    }
}
