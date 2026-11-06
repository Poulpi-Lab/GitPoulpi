//! `config.toml` is written user-only by every writer, through
//! `secure_fs::write_private`: a mode test per writer, so a path that goes
//! back to `std::fs::write` is red.

#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use gitpoulpi_config::Config;

fn mode(path: &Path) -> u32 {
    std::fs::metadata(path).unwrap().permissions().mode() & 0o777
}

#[test]
fn save_writes_a_user_only_file_in_a_user_only_directory() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cfg").join("config.toml");
    Config::default().save(&path).unwrap();
    assert_eq!(mode(&path), 0o600);
    assert_eq!(mode(path.parent().unwrap()), 0o700);
}

#[test]
fn a_key_set_in_an_existing_loose_file_leaves_it_user_only() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "[general]\n").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
    gitpoulpi_config::set_in_file(&path, "general.auto_prune", "true").unwrap();
    assert_eq!(mode(&path), 0o600);
}
