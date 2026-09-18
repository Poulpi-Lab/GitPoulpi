//! `packaging/install.sh` against a locally packed archive (the same code that
//! `xtask dist` packs with), in a scratch HOME, with `--archive` and `--yes`:
//! no network, and the real home, `~/.local` and shell files are never touched.

#![cfg(unix)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use xtask::dist::{checksum_line, marker_entry, sha256_hex, write_tar_gz, Entry};

const TARGET: &str = "x86_64-unknown-linux-gnu";

fn script() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packaging/install.sh")
}

/// A release-shaped archive whose `gitpoulpi` is the real CLI build.
fn make_archive(dir: &Path, version: &str) -> PathBuf {
    let cli = std::fs::read(assert_cmd::cargo::cargo_bin("gitpoulpi")).unwrap();
    let file = |name: &str, data: Vec<u8>, executable| Entry {
        name: name.into(),
        data,
        executable,
    };
    let entries = vec![
        file("gitpoulpi", cli, true),
        file("gitpoulpi-tui", b"#!/bin/sh\n".to_vec(), true),
        file("gitpoulpi-mcp", b"#!/bin/sh\n".to_vec(), true),
        file("gitpoulpi-launch", b"#!/bin/sh\n".to_vec(), true),
        marker_entry(version, TARGET),
        file("INSTALL.txt", b"hello".to_vec(), false),
        file("LICENSE", b"mit".to_vec(), false),
    ];
    let stem = format!("gitpoulpi-{version}-{TARGET}");
    let bytes = write_tar_gz(&stem, &entries).unwrap();
    let name = format!("{stem}.tar.gz");
    let path = dir.join(&name);
    std::fs::write(&path, &bytes).unwrap();
    std::fs::write(
        dir.join(format!("{name}.sha256")),
        checksum_line(&sha256_hex(&bytes), &name),
    )
    .unwrap();
    path
}

fn sh(home: &Path, args: &[&str]) -> Output {
    Command::new("sh")
        .arg(script())
        .args(args)
        .env("HOME", home)
        .env("XDG_DATA_HOME", home.join("data"))
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("GITPOULPI_CONFIG_DIR", home.join("cfg"))
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .env("PATH", "/usr/bin:/bin")
        .env_remove("LOCALAPPDATA")
        .output()
        .unwrap()
}

fn text(o: &Output) -> String {
    format!(
        "status {:?}\n{}\n{}",
        o.status.code(),
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}

#[test]
fn install_then_uninstall_round_trip() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("home");
    std::fs::create_dir_all(&home).unwrap();
    let rc = home.join(".bashrc");
    std::fs::write(&rc, "export KEEP=1\n").unwrap();
    let archive = make_archive(tmp.path(), "0.0.1");
    let prefix = home.join(".local/opt/gitpoulpi");
    let bin = home.join(".local/bin");

    let out = sh(
        &home,
        &[
            "--archive",
            archive.to_str().unwrap(),
            "--yes",
            "--alias",
            "gp",
        ],
    );
    assert!(out.status.success(), "{}", text(&out));
    for f in [
        "gitpoulpi",
        "gitpoulpi-tui",
        "gitpoulpi-mcp",
        ".gitpoulpi-dist",
    ] {
        assert!(prefix.join(f).is_file(), "{f} installed");
    }
    assert!(prefix.join("INSTALL.txt").is_file());
    for f in ["gitpoulpi", "gitpoulpi-mcp"] {
        assert_eq!(std::fs::read_link(bin.join(f)).unwrap(), prefix.join(f));
    }
    assert!(
        std::fs::symlink_metadata(bin.join("gitpoulpi-tui")).is_err(),
        "the TUI is not linked (it would read as an alias)"
    );
    assert_eq!(
        std::fs::read_link(bin.join("gp")).unwrap(),
        prefix.join("gitpoulpi-tui"),
        "the alias starts the TUI"
    );
    let rc_text = std::fs::read_to_string(&rc).unwrap();
    assert!(rc_text.starts_with("export KEEP=1\n"));
    assert!(rc_text.contains(&prefix.display().to_string()), "{rc_text}");

    // Installing again over the same prefix is fine.
    let again = sh(&home, &["--archive", archive.to_str().unwrap(), "--yes"]);
    assert!(again.status.success(), "{}", text(&again));

    let gone = sh(&home, &["--uninstall"]);
    assert!(gone.status.success(), "{}", text(&gone));
    assert!(!prefix.exists(), "the prefix is gone");
    for f in ["gitpoulpi", "gitpoulpi-mcp", "gp"] {
        assert!(std::fs::symlink_metadata(bin.join(f)).is_err(), "{f}");
    }
    assert_eq!(std::fs::read_to_string(&rc).unwrap(), "export KEEP=1\n");
}

#[test]
fn no_alias_and_no_path_leave_the_shell_alone() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("home");
    std::fs::create_dir_all(&home).unwrap();
    let rc = home.join(".bashrc");
    std::fs::write(&rc, "x\n").unwrap();
    let archive = make_archive(tmp.path(), "0.0.1");
    let prefix = tmp.path().join("elsewhere");

    let out = sh(
        &home,
        &[
            "--archive",
            archive.to_str().unwrap(),
            "--prefix",
            prefix.to_str().unwrap(),
            "--yes",
            "--no-alias",
            "--no-path",
        ],
    );
    assert!(out.status.success(), "{}", text(&out));
    assert!(prefix.join("gitpoulpi").is_file());
    assert!(std::fs::symlink_metadata(home.join(".local/bin/gp")).is_err());
    assert_eq!(std::fs::read_to_string(&rc).unwrap(), "x\n");
}

#[test]
fn a_tampered_archive_is_refused_and_nothing_is_installed() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("home");
    std::fs::create_dir_all(&home).unwrap();
    let archive = make_archive(tmp.path(), "0.0.1");
    let mut bytes = std::fs::read(&archive).unwrap();
    bytes.push(0);
    std::fs::write(&archive, bytes).unwrap();

    let out = sh(&home, &["--archive", archive.to_str().unwrap(), "--yes"]);
    assert!(!out.status.success());
    assert!(text(&out).contains("checksum mismatch"), "{}", text(&out));
    assert!(!home.join(".local/opt/gitpoulpi").exists());
}

#[test]
fn uninstall_refuses_a_folder_without_the_marker() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("home");
    let prefix = tmp.path().join("checkout");
    std::fs::create_dir_all(&home).unwrap();
    std::fs::create_dir_all(&prefix).unwrap();
    std::fs::copy(
        assert_cmd::cargo::cargo_bin("gitpoulpi"),
        prefix.join("gitpoulpi"),
    )
    .unwrap();

    let out = sh(
        &home,
        &["--uninstall", "--prefix", prefix.to_str().unwrap()],
    );
    assert!(!out.status.success(), "{}", text(&out));
    assert!(prefix.join("gitpoulpi").exists(), "nothing was deleted");
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
