//! Pre-defined browser fingerprints
//!
//! This module contains fingerprint definitions for various browsers.

mod chrome;
mod firefox;

pub use chrome::{chrome_124, chrome_125, chrome_131, chrome_133, chrome_136, chrome_142};
pub use firefox::{firefox_133, firefox_135, firefox_144};
