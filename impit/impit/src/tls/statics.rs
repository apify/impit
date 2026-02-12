use rustls::crypto::{aws_lc_rs, hpke::Hpke};

pub static GREASE_HPKE_SUITE: &dyn Hpke = aws_lc_rs::hpke::DH_KEM_X25519_HKDF_SHA256_AES_128;
