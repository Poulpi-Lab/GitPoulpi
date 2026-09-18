//! `gitpoulpi launch -- <command>` with no GitPoulpi window open: it starts
//! the command (what a shortcut used to do) and says why. JSON assertions;
//! the config directory is a scratch one, so no real session is ever probed,
//! and the "terminal" is a script that leaves a marker file.

#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use assert_cmd::Command;
use serde_json::Value;

/// A script that creates the file it is given as its first argument.
fn marker_script(dir: &Path) -> PathBuf {
    let script = dir.join("terminal.sh");
    std::fs::write(&script, "#!/bin/sh\ntouch \"$1\"\n").unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    script
}

fn launch(config: &Path, args: &[&str]) -> (bool, Value) {
    let out = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config)
        .env("HOME", config)
        .env("XDG_CONFIG_HOME", config)
        .env("XDG_DATA_HOME", config)
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .arg("--json")
        .arg("launch")
        .args(args)
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let json = serde_json::from_str(&text).unwrap_or_else(|e| {
        panic!(
            "not JSON ({e}): {text} / {}",
            String::from_utf8_lossy(&out.stderr)
        )
    });
    (out.status.success(), json)
}

fn wait_for(file: &Path) -> bool {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if file.exists() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    false
}

#[test]
fn with_no_window_open_the_command_is_started_and_the_reason_is_said() {
    let tmp = tempfile::tempdir().unwrap();
    let script = marker_script(tmp.path());
    let marker = tmp.path().join("started");
    let (ok, json) = launch(
        tmp.path(),
        &["--", &script.to_string_lossy(), &marker.to_string_lossy()],
    );
    assert!(ok, "{json}");
    assert_eq!(json["action"], "started");
    assert_eq!(json["reason"], "no_window");
    assert_eq!(json["command"][0], script.to_string_lossy().as_ref());
    assert!(wait_for(&marker), "the command ran");
}

#[test]
fn the_setting_off_starts_the_command_without_looking_for_a_window() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(
        tmp.path().join("config.toml"),
        "[ui]\nreuse_window = false\n",
    )
    .unwrap();
    let script = marker_script(tmp.path());
    let marker = tmp.path().join("started");
    let (ok, json) = launch(
        tmp.path(),
        &["--", &script.to_string_lossy(), &marker.to_string_lossy()],
    );
    assert!(ok, "{json}");
    assert_eq!(json["action"], "started");
    assert_eq!(json["reason"], "disabled");
    assert!(wait_for(&marker));
}

#[test]
fn a_dead_daemons_port_file_does_not_block_the_start() {
    let tmp = tempfile::tempdir().unwrap();
    let sessions = tmp.path().join("sessions");
    std::fs::create_dir_all(&sessions).unwrap();
    // Nothing listens on port 9 here.
    std::fs::write(sessions.join("ghost.port"), "9").unwrap();
    let script = marker_script(tmp.path());
    let marker = tmp.path().join("started");
    let started = Instant::now();
    let (ok, json) = launch(
        tmp.path(),
        &["--", &script.to_string_lossy(), &marker.to_string_lossy()],
    );
    assert!(ok && json["reason"] == "no_window", "{json}");
    assert!(started.elapsed() < Duration::from_secs(5));
    assert!(wait_for(&marker));
}

#[test]
fn no_command_and_no_window_is_an_error_not_a_silent_success() {
    let tmp = tempfile::tempdir().unwrap();
    let (ok, json) = launch(tmp.path(), &[]);
    assert!(!ok, "{json}");
    assert_eq!(json["action"], "failed");
    assert!(json["error"].as_str().unwrap().contains("--"), "{json}");
}

#[test]
fn a_program_that_does_not_exist_fails_with_its_name() {
    let tmp = tempfile::tempdir().unwrap();
    let (ok, json) = launch(tmp.path(), &["--", "/nonexistent/no-such-terminal"]);
    assert!(!ok);
    assert!(
        json["error"].as_str().unwrap().contains("no-such-terminal"),
        "{json}"
    );
}

#[test]
fn new_window_starts_the_command_with_its_own_reason() {
    let tmp = tempfile::tempdir().unwrap();
    let script = marker_script(tmp.path());
    let marker = tmp.path().join("started");
    let (ok, json) = launch(
        tmp.path(),
        &[
            "--new-window",
            "--",
            &script.to_string_lossy(),
            &marker.to_string_lossy(),
        ],
    );
    assert!(ok, "{json}");
    assert_eq!(json["action"], "started");
    assert_eq!(json["reason"], "new_window");
    assert_eq!(json["command"][0], script.to_string_lossy().as_ref());
    assert!(wait_for(&marker));
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
