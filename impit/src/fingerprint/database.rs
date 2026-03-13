//! Pre-defined browser fingerprints
//!
//! This module contains fingerprint definitions for various browsers.

mod chrome;
mod firefox;
mod okhttp;

pub use chrome::{
    chrome_100, chrome_101, chrome_104, chrome_107, chrome_110, chrome_116, chrome_124, chrome_125,
    chrome_131, chrome_133, chrome_136, chrome_142,
};
pub use firefox::{firefox_128, firefox_133, firefox_135, firefox_144};
pub use okhttp::{okhttp3, okhttp4, okhttp5};
