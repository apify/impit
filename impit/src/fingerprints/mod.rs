use crate::fingerprint::{Fingerprint, H2Fingerprint, TlsFingerprint};

pub(crate) static CHROME_136_FINGERPRINT: Fingerprint = Fingerprint {
    headers: &[
        ("sec-ch-ua", "\"Chromium\";v=\"136\", \"Google Chrome\";v=\"136\", \"Not.A/Brand\";v=\"99\""),
        ("sec-ch-ua-mobile", "?0"),
        ("sec-ch-ua-platform", "\"macOS\""),
        ("Upgrade-Insecure-Requests", "1"),
        ("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36"),
        ("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7"),
        ("Sec-Fetch-Site", "none"),
        ("Sec-Fetch-Mode", "navigate"),
        ("Sec-Fetch-User", "?1"),
        ("Sec-Fetch-Dest", "document"),
        ("Accept-Encoding", "gzip, deflate, br, zstd"),
        ("Accept-Language", "en-US,en;q=0.9"),
        ("Priority", "u=0, i"),
    ],
    h2_fingerprint: H2Fingerprint {
        window_size: 6291456,
        max_frame_size: 16384,
        initial_conn_window_size: 15728640,
        settings: &[
            (0x1, 0x100),       // SETTINGS_HEADER_TABLE_SIZE
            (0x2, 0),           // SETTINGS_ENABLE_PUSH
            (0x3, 0x4000),      // SETTINGS_MAX_CONCURRENT_STREAMS
            (0x4, 0x10000),     // SETTINGS_INITIAL_WINDOW_SIZE
            (0x5, 0),           // SETTINGS_MAX_FRAME_SIZE
            (0x6, 0xffff),      // SETTINGS_MAX_HEADER_LIST_SIZE
        ],
        pseudo_headers_order: &[
            ":method",
            ":authority",
            ":scheme",
            ":path",
            ":protocol",
            ":status",
        ],
    },
    tls_fingerprint: TlsFingerprint {
        ciphers: &[
            0x1301, 0x1302, 0x1303, 0xcca9, 0xcca8, 0xcca7, 0xc02b, 0xc02f, 0xc02c, 0xc030,
            0x009e, 0x009d, 0x002f, 0x0035, 0x000a,
        ],
        extensions: &[
            0x0000, 0x000b, 0x000a, 0x0015, 0x0017, 0x0023, 0x002b, 0x0033,
            0x000d, 0x001c, 0x002d, 0x0005, 0x0010,
        ],
        groups: &[
            0x001d, 0x0017, 0x001e, 0x0100,
        ],
        sig_algs: &[
            0x0403, 0x0503, 0x0603, 0x0804, 0x0805, 0x0806,
            0x0201, 0x0401, 0x0501, 0x0601,
        ],
    }
};

