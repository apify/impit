#[derive(Debug, Clone, Copy, Default)]
pub struct H2Fingerprint {
    pub window_size: u32,
    pub max_frame_size: u32,
    pub initial_conn_window_size: u32,
    pub settings: &'static [(u16, u32)],
    pub pseudo_headers_order: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TlsFingerprint {
    pub ciphers: &'static [u16],
    pub extensions: &'static [u16],
    pub groups: &'static [u16],
    pub sig_algs: &'static [u16],
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Fingerprint {
    pub headers: &'static[(&'static str, &'static str)],
    pub tls_fingerprint: TlsFingerprint,
    pub h2_fingerprint: H2Fingerprint,
}
