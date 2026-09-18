//! `gitpoulpi desktop shortcut status|install|remove`. JSON assertions; HOME,
//! XDG_DATA_HOME and XDG_CONFIG_HOME point at a scratch directory so the real
//! launcher is never touched, and no terminal is ever started (the entry is
//! only written).

#![cfg(target_os = "linux")]

use std::path::Path;

use assert_cmd::Command;
use serde_json::Value;

fn run(scratch: &Path, args: &[&str]) -> (bool, Value) {
    let config = scratch.join("cfg");
    std::fs::create_dir_all(&config).unwrap();
    std::fs::write(
        config.join("config.toml"),
        "[ui]\nlaunch_terminal = \"kitty\"\n",
    )
    .unwrap();
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", &config)
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
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

fn state_of(status: &Value, place: &str) -> String {
    status["shortcuts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["place"] == place)
        .unwrap_or_else(|| panic!("no {place} in {status}"))["state"]
        .as_str()
        .unwrap()
        .to_string()
}

#[test]
fn install_status_remove_round_trip_in_a_scratch_home() {
    let tmp = tempfile::tempdir().unwrap();
    let entry = tmp.path().join("data/applications/gitpoulpi.desktop");

    let (ok, status) = run(tmp.path(), &["desktop", "shortcut", "status"]);
    assert!(ok, "{status}");
    assert_eq!(status["terminal"], "kitty");
    assert_eq!(state_of(&status, "launcher"), "not_installed");

    let (ok, done) = run(tmp.path(), &["desktop", "shortcut", "install"]);
    assert!(ok, "{done}");
    assert_eq!(done["done"][0], "installed launcher");
    let text = std::fs::read_to_string(&entry).unwrap();
    assert!(
        text.contains(" launch -- kitty --class gitpoulpi "),
        "{text}"
    );
    let icon = text.lines().find_map(|l| l.strip_prefix("Icon=")).unwrap();
    assert!(tmp
        .path()
        .join(format!("data/icons/hicolor/256x256/apps/{icon}.png"))
        .is_file());

    let (_, status) = run(tmp.path(), &["desktop", "shortcut", "status"]);
    assert_eq!(state_of(&status, "launcher"), "installed");

    let (ok, done) = run(tmp.path(), &["desktop", "shortcut", "remove"]);
    assert!(ok, "{done}");
    assert!(!entry.exists());
    let (_, status) = run(tmp.path(), &["desktop", "shortcut", "status"]);
    assert_eq!(state_of(&status, "launcher"), "not_installed");
}

#[test]
fn a_foreign_file_is_reported_and_never_removed() {
    let tmp = tempfile::tempdir().unwrap();
    let apps = tmp.path().join("data/applications");
    std::fs::create_dir_all(&apps).unwrap();
    std::fs::write(
        apps.join("gitpoulpi.desktop"),
        "[Desktop Entry]\nName=mine\n",
    )
    .unwrap();
    let (_, status) = run(tmp.path(), &["desktop", "shortcut", "status"]);
    assert_eq!(state_of(&status, "launcher"), "foreign");
    let (ok, err) = run(tmp.path(), &["desktop", "shortcut", "remove"]);
    assert!(!ok);
    assert!(err["error"].as_str().unwrap().contains("not ours"), "{err}");
    assert!(apps.join("gitpoulpi.desktop").is_file());
}

#[test]
fn an_unknown_place_is_an_error() {
    let tmp = tempfile::tempdir().unwrap();
    let (ok, err) = run(
        tmp.path(),
        &["desktop", "shortcut", "install", "--where", "moon"],
    );
    assert!(!ok);
    assert!(err["error"].as_str().unwrap().contains("unknown place"));
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
