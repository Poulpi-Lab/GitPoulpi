//! `gitpoulpi alias status|create|remove` and `gitpoulpi path status|add|remove`.
//! JSON assertions; HOME, XDG_* and PATH point into a scratch directory so the
//! real home, shell files and `~/.local/bin` are never touched.

#![cfg(unix)]

use std::path::Path;

use assert_cmd::Command;
use serde_json::Value;

fn run(scratch: &Path, args: &[&str]) -> (bool, Value) {
    let config = scratch.join("cfg");
    std::fs::create_dir_all(&config).unwrap();
    let home = scratch.join("home");
    std::fs::create_dir_all(&home).unwrap();
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", &config)
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .env("HOME", &home)
        .env("XDG_DATA_HOME", scratch.join("data"))
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("PATH", "/usr/bin:/bin")
        .env_remove("LOCALAPPDATA")
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

#[test]
fn create_status_remove_round_trip_in_a_scratch_home() {
    let tmp = tempfile::tempdir().unwrap();
    let link = tmp.path().join("home/.local/bin/gp");

    let (ok, status) = run(tmp.path(), &["alias", "status"]);
    assert!(ok, "{status}");
    assert_eq!(status["aliases"].as_array().unwrap().len(), 0);
    assert_eq!(
        status["bin_dir"],
        tmp.path().join("home/.local/bin").display().to_string()
    );

    let (ok, created) = run(tmp.path(), &["alias", "create", "gp"]);
    assert!(ok, "{created}");
    assert_eq!(created["created"], "gp");
    assert_eq!(created["path"], link.display().to_string());
    assert_eq!(
        std::fs::read_link(&link).unwrap().display().to_string(),
        created["target"].as_str().unwrap(),
        "the link points at the reported exe"
    );
    assert!(created["target"].as_str().unwrap().contains("gitpoulpi"));
    assert_eq!(created["bin_dir_on_path"], false);

    let (_, status) = run(tmp.path(), &["alias", "status"]);
    assert_eq!(status["aliases"][0]["name"], "gp");
    assert_eq!(status["aliases"][0]["state"], "installed");

    let (ok, removed) = run(tmp.path(), &["alias", "remove"]);
    assert!(ok, "{removed}");
    assert_eq!(removed["removed"][0], "gp");
    assert!(std::fs::symlink_metadata(&link).is_err());
    let (_, again) = run(tmp.path(), &["alias", "remove", "gp"]);
    assert_eq!(again["removed"].as_array().unwrap().len(), 0);
}

#[test]
fn a_bad_name_and_a_file_that_is_not_ours_are_refused() {
    let tmp = tempfile::tempdir().unwrap();
    let (ok, out) = run(tmp.path(), &["alias", "create", "a/b"]);
    assert!(!ok);
    assert!(
        out["error"].as_str().unwrap().contains("separator"),
        "{out}"
    );
    assert!(!tmp.path().join("home/.local/bin").exists());

    let bin = tmp.path().join("home/.local/bin");
    std::fs::create_dir_all(&bin).unwrap();
    std::fs::write(bin.join("gp"), "#!/bin/sh\n").unwrap();
    let (ok, out) = run(tmp.path(), &["alias", "create", "gp"]);
    assert!(!ok);
    assert!(out["error"].as_str().unwrap().contains("not ours"), "{out}");
    let (ok, _) = run(tmp.path(), &["alias", "remove", "gp"]);
    assert!(!ok);
    assert_eq!(
        std::fs::read_to_string(bin.join("gp")).unwrap(),
        "#!/bin/sh\n"
    );
}

#[test]
fn path_add_status_remove_touch_only_the_marked_block() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("home");
    std::fs::create_dir_all(&home).unwrap();
    let rc = home.join(".bashrc");
    std::fs::write(&rc, "export A=1\n").unwrap();

    let (ok, status) = run(tmp.path(), &["path", "status"]);
    assert!(ok, "{status}");
    assert_eq!(status["state"], "not_added");
    assert!(!status["wanted"].as_array().unwrap().is_empty());

    let (ok, added) = run(tmp.path(), &["path", "add"]);
    assert!(ok, "{added}");
    assert_eq!(added["changed"][0], rc.display().to_string());
    let text = std::fs::read_to_string(&rc).unwrap();
    assert!(text.starts_with("export A=1\n") && text.contains("X-GitPoulpi=1"));
    assert!(text.contains(&home.join(".local/bin").display().to_string()));
    let (_, status) = run(tmp.path(), &["path", "status"]);
    assert_eq!(status["state"], "added");

    // Idempotent: a second add leaves the file byte for byte.
    run(tmp.path(), &["path", "add"]);
    assert_eq!(std::fs::read_to_string(&rc).unwrap(), text);

    let (ok, removed) = run(tmp.path(), &["path", "remove"]);
    assert!(ok, "{removed}");
    assert_eq!(removed["changed"][0], rc.display().to_string());
    assert_eq!(std::fs::read_to_string(&rc).unwrap(), "export A=1\n");
    let (_, removed) = run(tmp.path(), &["path", "remove"]);
    assert_eq!(removed["changed"].as_array().unwrap().len(), 0);
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
