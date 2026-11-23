//! `gitpoulpi installation set name|name_with_version` renames the shortcuts
//! the installation already has, in place, instead of leaving the old name
//! until `desktop shortcut install` runs. HOME, XDG_DATA_HOME and the config
//! folder point into a scratch directory; the terminal is never started.

#![cfg(target_os = "linux")]

use std::path::Path;

use assert_cmd::Command;
use serde_json::Value;

fn run(scratch: &Path, id: &str, args: &[&str]) -> (bool, Value) {
    let config = scratch.join("cfg");
    std::fs::create_dir_all(&config).unwrap();
    let file = config.join("config.toml");
    if !file.exists() {
        std::fs::write(&file, "[ui]\nlaunch_terminal = \"kitty\"\n").unwrap();
    }
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", &config)
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .env("GITPOULPI_INSTALL_ID", id)
        .env_remove("GITPOULPI_ENV")
        .env("HOME", scratch)
        .env("XDG_DATA_HOME", scratch.join("data"))
        .env("XDG_CONFIG_HOME", scratch.join("xdg-config"))
        .current_dir(scratch)
        .arg("--json")
        .args(args)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let json = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "not JSON ({e}): {stdout} / {}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output.status.success(), json)
}

/// The file name and `Name=` of the one entry under the scratch applications
/// folder.
fn entry(scratch: &Path) -> (String, String) {
    let apps = scratch.join("data/applications");
    let mut files: Vec<_> = std::fs::read_dir(&apps)
        .map(|d| d.flatten().map(|e| e.path()).collect())
        .unwrap_or_default();
    files.sort();
    assert_eq!(files.len(), 1, "one entry expected in {apps:?}: {files:?}");
    let text = std::fs::read_to_string(&files[0]).unwrap();
    let name = text
        .lines()
        .find_map(|l| l.strip_prefix("Name="))
        .unwrap_or_default()
        .to_string();
    (files[0].file_name().unwrap().to_string_lossy().into(), name)
}

#[test]
fn setting_the_name_or_the_version_renames_the_existing_shortcut_in_place() {
    let tmp = tempfile::tempdir().unwrap();
    let (ok, made) = run(tmp.path(), "tester", &["desktop", "shortcut", "install"]);
    assert!(ok, "{made}");
    let (file, name) = entry(tmp.path());
    assert!(name.starts_with("GitPoulpi ("), "{name}");

    let (ok, set) = run(
        tmp.path(),
        "tester",
        &["installation", "set", "name", "Renamed Box"],
    );
    assert!(ok, "{set}");
    assert_eq!(entry(tmp.path()), (file.clone(), "Renamed Box".to_string()));

    let (ok, set) = run(
        tmp.path(),
        "tester",
        &["installation", "set", "name_with_version", "on"],
    );
    assert!(ok, "{set}");
    let version = {
        let mut parts = env!("CARGO_PKG_VERSION").split('.');
        format!("{}.{}", parts.next().unwrap(), parts.next().unwrap())
    };
    assert_eq!(
        entry(tmp.path()),
        (file.clone(), format!("Renamed Box {version}"))
    );

    // Another installation's rename never touches this one's shortcut.
    let (ok, set) = run(
        tmp.path(),
        "other",
        &["installation", "set", "name", "Elsewhere"],
    );
    assert!(ok, "{set}");
    assert_eq!(entry(tmp.path()).0, file);
    assert!(entry(tmp.path()).1.starts_with("Renamed Box"));
}

#[test]
fn a_name_change_creates_no_shortcut_that_was_not_there() {
    let tmp = tempfile::tempdir().unwrap();
    let (ok, set) = run(
        tmp.path(),
        "tester",
        &["installation", "set", "name", "Nothing Yet"],
    );
    assert!(ok, "{set}");
    assert!(
        !tmp.path().join("data/applications").exists()
            || std::fs::read_dir(tmp.path().join("data/applications"))
                .unwrap()
                .next()
                .is_none()
    );
}
