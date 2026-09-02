use encoding::Encoding;
use lol_html::{ElementContentHandlers, HandlerResult, Selector};
use mime::{Mime, TEXT_PLAIN};
use std::{borrow::Cow, sync::LazyLock};

/// Implements the BOM sniffing algorithm to detect the encoding of the response.
/// If the BOM sniffing algorithm fails, the function returns `None`.
///
/// See more details at https://encoding.spec.whatwg.org/#bom-sniff
fn bom_sniffing(bytes: &[u8]) -> Option<encoding::EncodingRef> {
    if bytes.len() < 3 {
        return None;
    }

    match bytes {
        [0xEF, 0xBB, 0xBF, ..] => Some(encoding::all::UTF_8),
        [0xFE, 0xFF, ..] => Some(encoding::all::UTF_16BE),
        [0xFF, 0xFE, ..] => Some(encoding::all::UTF_16LE),
        _ => None,
    }
}

const META_TAG_START: &[u8] = b"<meta";

/// The prescan selectors are parsed once - `lol_html::element!` re-parses them on every call.
static CHARSET_SELECTOR: LazyLock<Selector> = LazyLock::new(|| "meta[charset]".parse().unwrap());
static HTTP_EQUIV_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| "meta[http-equiv]".parse().unwrap());

fn on_element<'h>(
    selector: &'static Selector,
    handler: impl FnMut(&mut lol_html::html_content::Element<'_, '_>) -> HandlerResult + 'h,
) -> (Cow<'static, Selector>, ElementContentHandlers<'h>) {
    (
        Cow::Borrowed(selector),
        ElementContentHandlers::default().element(handler),
    )
}

/// A lazy implementation of the prescan algorithm, using `lol_html` to parse the HTML and extract the encoding.
///
/// See more details at https://html.spec.whatwg.org/#prescan-a-byte-stream-to-determine-its-encoding
fn prescan_bytestream(bytes: &[u8]) -> Option<encoding::EncodingRef> {
    if bytes.len() < 4 {
        return None;
    }

    let limit = std::cmp::min(1024, bytes.len());

    // Both handlers below can only fire on a `<meta` start tag, so responses without one in the
    // prescanned prefix (JSON, plain text, binaries, ...) don't need the HTML parser at all.
    if !bytes[0..limit]
        .windows(META_TAG_START.len())
        .any(|window| window.eq_ignore_ascii_case(META_TAG_START))
    {
        return None;
    }

    let ascii_body = encoding::all::ASCII
        .decode(&bytes[0..limit], encoding::DecoderTrap::Replace)
        .unwrap();

    let found = std::rc::Rc::new(std::cell::RefCell::new(None::<encoding::EncodingRef>));
    let found_charset = std::rc::Rc::clone(&found);
    let found_http_equiv = std::rc::Rc::clone(&found);

    let mut rewriter = lol_html::HtmlRewriter::new(
        lol_html::Settings {
            element_content_handlers: vec![
                on_element(&CHARSET_SELECTOR, move |el| {
                    if found_charset.borrow().is_none() {
                        if let Some(charset) = el.get_attribute("charset") {
                            *found_charset.borrow_mut() =
                                encoding::label::encoding_from_whatwg_label(&charset);
                        }
                    }
                    Ok(())
                }),
                on_element(&HTTP_EQUIV_SELECTOR, move |el| {
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
    use super::*;

    #[test]
    fn prescan_finds_the_meta_charset() {
        let html = br#"<!doctype html><html><head><meta charset="windows-1250"></head></html>"#;

        assert_eq!(
            prescan_bytestream(html).map(|encoding| encoding.name()),
            Some("windows-1250")
        );
    }

    #[test]
    fn prescan_matches_the_meta_tag_case_insensitively() {
        let html = br#"<!DOCTYPE HTML><HTML><HEAD><META CHARSET="windows-1250"></HEAD></HTML>"#;

        assert_eq!(
            prescan_bytestream(html).map(|encoding| encoding.name()),
            Some("windows-1250")
        );
    }

    #[test]
    fn prescan_reads_the_charset_from_the_content_type_meta() {
        let html = br#"<meta http-equiv="Content-Type" content="text/html; charset=windows-1250">"#;

        assert_eq!(
            prescan_bytestream(html).map(|encoding| encoding.name()),
            Some("windows-1250")
        );
    }

    #[test]
    fn prescan_ignores_meta_tags_past_the_first_kilobyte() {
        let mut html = vec![b' '; 1024];
        html.extend_from_slice(br#"<meta charset="windows-1250">"#);

        assert!(prescan_bytestream(&html).is_none());
    }

    #[test]
    fn prescan_skips_documents_without_a_meta_tag() {
        assert!(prescan_bytestream(br#"{"hello":"world"}"#).is_none());
    }
}
