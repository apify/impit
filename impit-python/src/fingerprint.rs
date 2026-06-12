use impit::fingerprint::{database, BrowserFingerprint};
use pyo3::exceptions::PyValueError;
use pyo3::PyResult;

/// Resolves a browser identifier string to its [`BrowserFingerprint`].
///
/// Shared by the sync and async clients so the two bindings cannot drift apart.
pub(crate) fn fingerprint_by_name(browser: &str) -> PyResult<BrowserFingerprint> {
    Ok(match browser.to_lowercase().as_str() {
        "chrome" | "chrome125" => database::chrome_125::fingerprint(),
        "chrome100" => database::chrome_100::fingerprint(),
        "chrome101" => database::chrome_101::fingerprint(),
        "chrome104" => database::chrome_104::fingerprint(),
        "chrome107" => database::chrome_107::fingerprint(),
        "chrome110" => database::chrome_110::fingerprint(),
        "chrome116" => database::chrome_116::fingerprint(),
        "chrome124" => database::chrome_124::fingerprint(),
        "chrome131" => database::chrome_131::fingerprint(),
        "chrome136" => database::chrome_136::fingerprint(),
        "chrome142" => database::chrome_142::fingerprint(),
        "firefox" | "firefox128" => database::firefox_128::fingerprint(),
        "firefox133" => database::firefox_133::fingerprint(),
        "firefox135" => database::firefox_135::fingerprint(),
        "firefox144" => database::firefox_144::fingerprint(),
        "okhttp3" => database::okhttp3::fingerprint(),
        "okhttp" | "okhttp4" => database::okhttp4::fingerprint(),
        "okhttp5" => database::okhttp5::fingerprint(),
        "ios18" => database::ios_18::fingerprint(),
        other => {
            return Err(PyValueError::new_err(format!(
                "Unsupported browser: {other}"
            )))
        }
    })
}
