use encoding::Encoding;
use mime::{Mime, TEXT_PLAIN};

/// Implements the BOM sniffing algorithm to detect the encoding of the response.
/// If the BOM sniffing algorithm fails, the function returns `None`.
///
/// See more details at https://encoding.spec.whatwg.org/#bom-sniff
fn bom_sniffing(bytes: &[u8]) -> Option<encoding::EncodingRef> {
    if bytes.len() < 3 {
        return None;
    }

    if [0xEF, 0xBB, 0xBF].to_vec() == bytes[0..3].to_vec() {
        return Some(encoding::all::UTF_8);
    }

    if [0xFE, 0xFF].to_vec() == bytes[0..2].to_vec() {
        return Some(encoding::all::UTF_16BE);
    }

    if [0xFF, 0xFE].to_vec() == bytes[0..2].to_vec() {
        return Some(encoding::all::UTF_16LE);
    }

    None
}

/// A lazy implementation of the prescan algorithm, using `lol_html` to parse the HTML and extract the encoding.
///
/// See more details at https://html.spec.whatwg.org/#prescan-a-byte-stream-to-determine-its-encoding
fn prescan_bytestream(bytes: &[u8]) -> Option<encoding::EncodingRef> {
    if bytes.len() < 4 {
        return None;
    }

    let limit = std::cmp::min(1024, bytes.len());

    let ascii_body = encoding::all::ASCII
        .decode(&bytes[0..limit], encoding::DecoderTrap::Replace)
        .unwrap();

    let found = std::rc::Rc::new(std::cell::RefCell::new(None::<encoding::EncodingRef>));
    let found_charset = std::rc::Rc::clone(&found);
    let found_http_equiv = std::rc::Rc::clone(&found);

    let mut rewriter = lol_html::HtmlRewriter::new(
        lol_html::Settings {
            element_content_handlers: vec![
                lol_html::element!("meta[charset]", move |el| {
                    if found_charset.borrow().is_none() {
                        if let Some(charset) = el.get_attribute("charset") {
                            *found_charset.borrow_mut() =
                                encoding::label::encoding_from_whatwg_label(&charset);
                        }
                    }
                    Ok(())
                }),
                lol_html::element!("meta[http-equiv]", move |el| {
                    if found_http_equiv.borrow().is_none() {
                        let is_content_type = el
                            .get_attribute("http-equiv")
                            .map(|v| v.eq_ignore_ascii_case("content-type"))
                            .unwrap_or(false);
                        if is_content_type {
                            if let Some(content) = el.get_attribute("content") {
                                if let Ok(ct) = ContentType::from(&content) {
                                    *found_http_equiv.borrow_mut() = ct.into();
                                }
                            }
                        }
                    }
                    Ok(())
                }),
            ],
            ..lol_html::Settings::default()
        },
        |_: &[u8]| {},
    );

    rewriter.write(ascii_body.as_bytes()).ok()?;
    rewriter.end().ok()?;

    let result = *found.borrow();
    result
}

/// Converts a vector of bytes to a [`String`] using the provided encoding.
///
/// If the encoding is not provided, the function tries to detect it using the BOM sniffing algorithm
/// and the byte stream prescanning algorithm.
///
/// ### Example
///
/// ```rust
/// use impit::utils::decode;
///
/// let bytes = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F];
/// let string = decode(&bytes, None);
///
/// assert_eq!(string, "Hello"); // By default, the function uses the UTF-8 encoding.
///
/// let bytes = vec![0xFE, 0xFF, 0x00, 0x48, 0x00, 0x65, 0x00, 0x6C, 0x00, 0x6C, 0x00, 0x6F];
/// let string = decode(&bytes, None);
///
/// assert_eq!(string, "\u{feff}Hello"); // The function detects the UTF-16BE encoding using the BOM sniffing algorithm.
///
/// let bytes = vec![0x9e, 0x6c, 0x75, 0x9d, 0x6f, 0x75, 0xe8, 0x6b, 0xfd, 0x20, 0x6b, 0xf9, 0xf2];
/// let string = decode(&bytes, Some(impit::utils::encodings::WINDOWS_1250));
///
/// assert_eq!(string, "žluťoučký kůň"); // The function uses the Windows-1250 encoding.
/// ```
pub fn decode(bytes: &[u8], preferred_encoding: Option<encoding::EncodingRef>) -> String {
    let encoding = match preferred_encoding {
        Some(encoding) => encoding,
        None => determine_encoding(bytes).unwrap_or(encoding::all::UTF_8),
    };

    encoding
        .decode(bytes, encoding::DecoderTrap::Replace)
        .unwrap()
}

/// Determines the encoding of a byte stream.
///
/// If the checks fail, the function returns `None`.
pub fn determine_encoding(bytes: &[u8]) -> Option<encoding::EncodingRef> {
    if let Some(enc) = bom_sniffing(bytes) {
        return Some(enc);
    } else if let Some(enc) = prescan_bytestream(bytes) {
        return Some(enc);
    }

    None
}

/// Decodes an HTTP header value into a [`String`].
///
/// Header values arrive as raw bytes with no charset declaration. Per RFC 9110 §5.5 they are
/// nominally ISO-8859-1 (the `obs-text` range), but in practice modern servers routinely send
/// UTF-8 (for example `Content-Disposition: attachment; filename="naïve.pdf"`).
///
/// This function decodes the bytes as UTF-8 when they form valid UTF-8, and otherwise falls back
/// to a byte-for-byte ISO-8859-1 decode (each byte `0x00..=0xFF` maps to the code point
/// `U+0000..=U+00FF`). This fixes the common UTF-8 case without corrupting genuine ISO-8859-1
/// values, never fails, and never emits `U+FFFD` replacement characters — so no header value can
/// crash a caller or come back empty.
///
/// ### Example
///
/// ```rust
/// use impit::utils::decode_header_value;
///
/// // Valid UTF-8 is decoded as UTF-8 (the ï is the two UTF-8 bytes 0xC3 0xAF).
/// assert_eq!(decode_header_value(&[b'n', b'a', 0xC3, 0xAF, b'v', b'e']), "naïve");
///
/// // A lone 0xE4 is not valid UTF-8, so it falls back to ISO-8859-1 ('ä').
/// assert_eq!(decode_header_value(&[b'M', 0xE4, b'r', b'z']), "März");
/// ```
pub fn decode_header_value(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(valid) => valid.to_owned(),
        Err(_) => bytes.iter().map(|&b| b as char).collect(),
    }
}

/// A struct that represents the contents of the `Content-Type` header.
///
/// The struct is used to extract the charset from the `Content-Type` header and convert it to an [`encoding::EncodingRef`].
///
/// ### Example
/// ```rust
/// use impit::utils::{ContentType, decode};
///
/// let bytes = vec![0x9e, 0x6c, 0x75, 0x9d, 0x6f, 0x75, 0xe8, 0x6b, 0xfd];
/// let content_type = ContentType::from("text/html; charset=cp1250").ok().unwrap();
///
/// let decoded = decode(&bytes, content_type.into());
/// ```
pub struct ContentType {
    pub charset: String,
}

/// Error enum for the `ContentType` struct operations.
pub enum ContentTypeError {
    InvalidContentType,
}

impl ContentType {
    pub fn from(content_type: &str) -> Result<Self, ContentTypeError> {
        let mime: Mime = content_type.parse().unwrap_or(TEXT_PLAIN);

        match mime.get_param("charset") {
            Some(encoding) => Ok(ContentType {
                charset: encoding.to_string(),
            }),
            None => Err(ContentTypeError::InvalidContentType),
        }
    }
}

impl From<ContentType> for Option<encoding::EncodingRef> {
    fn from(val: ContentType) -> Self {
        encoding::label::encoding_from_whatwg_label(val.charset.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::decode_header_value;

    #[test]
    fn ascii_is_unchanged() {
        assert_eq!(decode_header_value(b"application/json"), "application/json");
    }

    #[test]
    fn empty_is_empty() {
        assert_eq!(decode_header_value(b""), "");
    }

    #[test]
    fn utf8_is_decoded_as_utf8() {
        // "naïve.pdf" — the ï is UTF-8 bytes 0xC3 0xAF (issue #479).
        let bytes = "attachment; filename=\"naïve.pdf\"".as_bytes();
        assert_eq!(
            decode_header_value(bytes),
            "attachment; filename=\"naïve.pdf\""
        );
    }

    #[test]
    fn invalid_utf8_falls_back_to_iso_8859_1() {
        // Lone 0xE4 ('ä' in ISO-8859-1) is not valid UTF-8 (PR #434 / issue #430).
        let bytes = [
            b'D', b'i', b'e', b'n', b's', b't', b'a', b'g', b',', b' ', b'3', b'1', b'.', b' ',
            b'M', 0xE4, b'r', b'z', b' ', b'2', b'0', b'2', b'6',
        ];
        assert_eq!(decode_header_value(&bytes), "Dienstag, 31. März 2026");
    }

    #[test]
    fn iso_8859_1_fallback_never_produces_replacement_char() {
        // Every non-UTF-8 byte maps to exactly one char, so the result round-trips back to bytes.
        let bytes = [0xE4, 0xF6, 0xFC, 0xFF];
        let decoded = decode_header_value(&bytes);
        assert!(!decoded.contains('\u{FFFD}'));
        let roundtrip: Vec<u8> = decoded.chars().map(|c| c as u8).collect();
        assert_eq!(roundtrip, bytes);
    }
}
