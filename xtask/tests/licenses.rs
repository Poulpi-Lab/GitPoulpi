//! The committed `THIRD-PARTY-LICENSES.txt` must match `Cargo.lock`: adding
//! or bumping a dependency without regenerating the notices fails here
//! (offline: it compares a fingerprint, it does not run cargo).

use std::path::{Path, PathBuf};

use xtask::licenses::{is_fresh, lock_fingerprint, recorded_fingerprint, OUTPUT};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

#[test]
fn the_notices_match_the_lock_file() {
    assert!(
        is_fresh(&root()).unwrap(),
        "{OUTPUT} is stale: run `cargo run -p xtask -- licenses` and commit it"
    );
}

#[test]
fn a_changed_lock_file_is_detected() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("Cargo.lock"), "version = 4\n").unwrap();
    let fp = lock_fingerprint(dir.path()).unwrap();
    std::fs::write(
        dir.path().join(OUTPUT),
        format!("head\nCargo.lock sha256: {fp}\n"),
    )
    .unwrap();
    assert!(is_fresh(dir.path()).unwrap());
    // CRLF checkouts hash the same.
    std::fs::write(dir.path().join("Cargo.lock"), "version = 4\r\n").unwrap();
    assert!(is_fresh(dir.path()).unwrap());
    std::fs::write(dir.path().join("Cargo.lock"), "version = 4\n[[package]]\n").unwrap();
    assert!(!is_fresh(dir.path()).unwrap());
}

#[test]
fn the_notices_carry_what_the_licenses_demand() {
    let text = std::fs::read_to_string(root().join(OUTPUT)).unwrap();
    assert!(recorded_fingerprint(&text).is_some());
    for needle in [
        "libgit2",
        "PCRE",
        "Community Data License Agreement",
        "Mozilla Public License",
        "option-ext",
        "syntect",
        "Catppuccin",
        "not affiliated with",
    ] {
        assert!(text.contains(needle), "missing: {needle}");
    }
}
