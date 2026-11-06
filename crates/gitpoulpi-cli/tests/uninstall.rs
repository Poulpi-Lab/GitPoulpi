//! `gitpoulpi uninstall`: JSON assertions in a scratch HOME. The exe is
//! copied next to a marker so it counts as a release install; the dev build
//! (no marker) is refused.

#![cfg(unix)]

use std::path::Path;

use assert_cmd::Command;
use serde_json::Value;

fn run(exe: &Path, scratch: &Path, args: &[&str]) -> (bool, Value) {
    let home = scratch.join("home");
    std::fs::create_dir_all(&home).unwrap();
    // The exe was just copied by this process: while a sibling test thread
    // forks, the child briefly inherits the still-open write fd and an exec
    // of the copy fails with ETXTBSY ("Text file busy"). That is the harness
    // racing itself, not the product, so only that error is retried.
    let mut attempt = 0;
    let output = loop {
        let mut cmd = Command::new(exe);
        cmd.env("GITPOULPI_CONFIG_DIR", scratch.join("cfg"))
            .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
            .env("HOME", &home)
            .env("XDG_DATA_HOME", scratch.join("data"))
            .env("XDG_CONFIG_HOME", home.join(".config"))
            .env("PATH", "/usr/bin:/bin")
            .env_remove("LOCALAPPDATA")
            .current_dir(scratch)
            .arg("--json")
            .args(args);
        match cmd.output() {
            Ok(output) => break output,
            Err(e) if e.raw_os_error() == Some(26) && attempt < 50 => {
                attempt += 1;
                std::thread::sleep(std::time::Duration::from_millis(40));
            }
            Err(e) => panic!("running {}: {e}", exe.display()),
        }
    };
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let json = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "not JSON ({e}): {stdout} / {}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output.status.success(), json)
}

fn built() -> std::path::PathBuf {
    assert_cmd::cargo::cargo_bin("gitpoulpi")
}

#[test]
fn a_development_build_is_refused() {
    let tmp = tempfile::tempdir().unwrap();
    let (ok, out) = run(&built(), tmp.path(), &["uninstall"]);
    assert!(!ok);
    assert!(out["error"].as_str().unwrap().contains("not a release"));
}

#[test]
fn a_release_install_loses_its_alias_path_block_and_files() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("opt");
    std::fs::create_dir_all(&dir).unwrap();
    let exe = dir.join("gitpoulpi");
    std::fs::copy(built(), &exe).unwrap();
    std::fs::write(dir.join("gitpoulpi-tui"), b"tui").unwrap();
    std::fs::write(dir.join("gitpoulpi.old"), b"old").unwrap();
    std::fs::write(dir.join("notes.txt"), b"mine").unwrap();
    std::fs::write(
        dir.join(".gitpoulpi-dist"),
        gitpoulpi_config::release::Marker::new("0.0.1", gitpoulpi_config::release::host_target())
            .render(),
    )
    .unwrap();
    let rc = tmp.path().join("home/.bashrc");
    std::fs::create_dir_all(rc.parent().unwrap()).unwrap();
    std::fs::write(&rc, "export A=1\n").unwrap();

    let (ok, made) = run(&exe, tmp.path(), &["alias", "create", "gp"]);
    assert!(ok, "{made}");
    let (ok, added) = run(&exe, tmp.path(), &["path", "add"]);
    assert!(ok, "{added}");
    assert!(std::fs::read_to_string(&rc).unwrap().len() > "export A=1\n".len());
    let link = tmp.path().join("home/.local/bin/gp");
    assert!(std::fs::symlink_metadata(&link).is_ok());

    let (ok, out) = run(&exe, tmp.path(), &["uninstall"]);
    assert!(ok, "{out}");
    assert_eq!(out["aliases_removed"][0], "gp");
    assert!(!out["path_changed"].as_array().unwrap().is_empty());
    assert_eq!(out["kept_files"], false);
    let removed: Vec<&str> = out["files_removed"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    for name in [
        "gitpoulpi",
        "gitpoulpi-tui",
        "gitpoulpi.old",
        ".gitpoulpi-dist",
    ] {
        assert!(removed.contains(&name), "{name} in {removed:?}");
    }
    assert_eq!(out["dir_removed"], false, "notes.txt keeps the folder");
    assert!(dir.join("notes.txt").is_file());
    assert!(!exe.exists());
    assert!(std::fs::symlink_metadata(&link).is_err());
    assert_eq!(std::fs::read_to_string(&rc).unwrap(), "export A=1\n");
}

#[test]
fn keep_files_leaves_the_program_alone() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("opt");
    std::fs::create_dir_all(&dir).unwrap();
    let exe = dir.join("gitpoulpi");
    std::fs::copy(built(), &exe).unwrap();
    std::fs::write(dir.join(".gitpoulpi-dist"), "version=0.0.1\ntarget=x\n").unwrap();
    let (ok, out) = run(&exe, tmp.path(), &["uninstall", "--keep-files"]);
    assert!(ok, "{out}");
    assert_eq!(out["kept_files"], true);
    assert!(exe.exists());
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
