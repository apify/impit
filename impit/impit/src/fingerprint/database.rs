//! Pre-defined browser fingerprints
//!
//! This module contains fingerprint definitions for various browsers.

mod chrome;
mod edge;
mod firefox;
mod safari;

pub use chrome::{
    chrome_100, chrome_101, chrome_104, chrome_107, chrome_110, chrome_116, chrome_124, chrome_125,
    chrome_131, chrome_133, chrome_136, chrome_142,
};
pub use chrome::chrome_headers_for_os;
pub use edge::{edge_131, edge_136};
pub use edge::edge_headers_for_os;
pub use firefox::{firefox_128, firefox_133, firefox_135, firefox_144};
pub use firefox::firefox_headers_for_os;
pub use safari::{safari_17_0, safari_17_2_ios, safari_18_0, safari_18_4};
pub use safari::safari_headers_for_os;
