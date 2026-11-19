//! The HTTP/2 SETTINGS env vars are process-wide, so this test has its own test binary: the lib
//! tests build fingerprinted clients in parallel and would race with it.

use impit::{cookie::Jar, fingerprint::database as fingerprints, impit::Impit};
use std::sync::Mutex;

const OMIT_SETTINGS: &str = "IMPIT_H2_OMIT_SETTINGS";
const NO_RFC7540_PRIORITIES: &str = "IMPIT_H2_NO_RFC7540_PRIORITIES";

/// Serializes the tests in this file: they all set the same process-wide env vars.
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn h2_settings_env() -> (String, String) {
    (
        std::env::var(OMIT_SETTINGS).unwrap(),
        std::env::var(NO_RFC7540_PRIORITIES).unwrap(),
    )
}

// A profile with nothing to leave out, built after one that leaves settings out, must not inherit
// them.
#[test]
fn later_profile_overrides_h2_settings_env() {
    let _guard = ENV_LOCK.lock().unwrap();

    Impit::<Jar>::builder()
        .with_fingerprint(fingerprints::firefox_128::fingerprint())
        .build()
        .unwrap();
    assert_eq!(h2_settings_env(), ("2,6".to_string(), "1".to_string()));

    // Without a fingerprint, impit leaves the vars alone, like IMPIT_H2_PSEUDOHEADERS_ORDER.
    Impit::<Jar>::builder().build().unwrap();
    assert_eq!(h2_settings_env(), ("2,6".to_string(), "1".to_string()));

    Impit::<Jar>::builder()
        .with_fingerprint(fingerprints::ios_18::fingerprint())
        .build()
        .unwrap();
    assert_eq!(h2_settings_env(), (String::new(), String::new()));
}
