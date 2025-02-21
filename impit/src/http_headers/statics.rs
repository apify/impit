// [TODO!]
// Note that not all requests are made the same:
//  - on forced (Ctrl+R) reloads, Chrome sets Cache-Control: max-age=0
//  - when the URL is in the address bar (but not submitted yet), Chrome sets `Purpose: prefetch` and `Sec-Purpose: prefetch`
pub static CHROME_HEADERS: &[(&str, &str)] = &[
    ("sec-ch-ua", "\"Google Chrome\";v=\"125\", \"Chromium\";v=\"125\", \"Not.A/Brand\";v=\"24\""),
    ("sec-ch-ua-mobile", "?0"),
    ("sec-ch-ua-platform", "Linux"),
    ("upgrade-insecure-requests", "1"),
    ("user-agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36"),
    ("accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7"),
    ("sec-fetch-site", "none"),
    ("sec-fetch-mode", "navigate"),
    ("sec-fetch-user", "?1"),
    ("sec-fetch-dest", "document"),
    ("accept-encoding", "gzip, deflate, br, zstd"),
    ("accept-language", "en-US,en;q=0.9"),
];

pub static CHROME_PSEUDOHEADERS_ORDER: [&str; 6] = [
    ":method",
    ":authority",
    ":scheme",
    ":path",
    ":protocol",
    ":status",
];

pub static FIREFOX_HEADERS: &[(&str, &str)] = &[
    ("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0"),
    ("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/png,image/svg+xml,*/*;q=0.8"),
    ("Accept-Language", "en,cs;q=0.7,en-US;q=0.3"),
    ("Accept-Encoding", "gzip, deflate, br, zstd"),
    ("sec-fetch-dest", "document"),
    ("sec-fetch-mode", "navigate"),
    ("sec-fetch-site", "none"),
    ("sec-fetch-user", "?1"),
    ("Upgrade-Insecure-Requests", "1"),
    ("Priority", "u=0, i"),
];

pub static FIREFOX_PSEUDOHEADERS_ORDER: [&str; 6] = [
    ":method",
    ":path",
    ":authority",
    ":scheme",
    ":protocol",
    ":status",
];
