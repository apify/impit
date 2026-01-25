//! Pre-defined browser fingerprints
//!
//! This module contains fingerprint definitions for various browsers.

mod chrome;
mod firefox;

use crate::emulation::Browser;
use super::BrowserFingerprint;

pub use chrome::chrome_125;
pub use firefox::firefox_128;

/// Returns a fingerprint for the specified browser.
///
/// This function provides a convenient way to get a fingerprint using the Browser enum.
pub fn get_fingerprint(browser: Browser) -> BrowserFingerprint {
    match browser {
        Browser::Chrome => chrome_125::fingerprint(),
        Browser::Firefox => firefox_128::fingerprint(),
    }
}
