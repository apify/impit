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

/// A lazy implementation of the BOM sniffing algorithm, using `scraper` to parse the HTML and extract the encoding.
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
    let dom = scraper::Html::parse_document(&ascii_body);

    let meta = dom
        .select(&scraper::Selector::parse("meta[charset]").unwrap())
        .next();

    if let Some(meta) = meta {
        let charset = meta.value().attr("charset").unwrap();
        return encoding::label::encoding_from_whatwg_label(charset);
    }

    let meta = dom
        .select(&scraper::Selector::parse("meta[http-equiv=content-type]").unwrap())
        .next();

    if let Some(meta) = meta {
        let content = meta.value().attr("content").unwrap();
        let content_type = ContentType::from(content);

        return match content_type {
            Ok(content_type) => content_type.into(),
            Err(_) => None,
        };
    }

    None
}

/// Converts a vector of bytes to a [`String`] using the provided encoding.
///
/// If the encoding is not provided, the function tries to detect it using the BOM sniffing algorithm
/// and the byte stream prescanning algorithm.
///
/// ### Example
///
/// ```rust
/// let bytes = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F];
/// let string = decode(&bytes, None);
///
/// assert_eq!(string, "Hello"); // By default, the function uses the UTF-8 encoding.
///
/// let bytes = vec![0xFE, 0xFF, 0x00, 0x48, 0x00, 0x65, 0x00, 0x6C, 0x00, 0x6C, 0x00, 0x6F];
/// let string = decode(&bytes, None);
///
/// assert_eq!(string, "Hello"); // The function detects the UTF-16BE encoding using the BOM sniffing algorithm.
///
/// let bytes = vec![0x9e, 0x6c, 0x75, 0x9d, 0x6f, 0x75, 0xe8, 0x6b, 0xfd, 0x20, 0x6b, 0xf9, 0xf2];
/// let string = decode(&bytes, Some(encoding::all::WINDOWS_1250));
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
/// let content_type = ContentType::from("text/html; charset=cp1250").unwrap();
///
/// decode(&bytes, content_type.into());
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
