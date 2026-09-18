//! `gitpoulpi conflicts am-*` and `gitpoulpi index-lock ...` through the real
//! binary and daemon (plan `stuck-states.md`, slice S8).

use std::path::Path;
use std::time::{Duration, SystemTime};

use assert_cmd::Command;
use gitpoulpi_testkit::contract::am_fixture;
use gitpoulpi_testkit::FixtureRepoBuilder;
use serde_json::Value;

fn config_dir() -> &'static Path {
    static DIR: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    DIR.get_or_init(|| tempfile::tempdir().unwrap()).path()
}

fn run(root: &Path, session: &str, args: &[&str]) -> (i32, Value) {
    let out = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config_dir())
        .arg("--json")
        .args(["--session", session])
        .args(["--repo", &root.to_string_lossy()])
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let json = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "`{}` gave no JSON ({e}): {stdout}\n{}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr)
        )
    });
    (out.status.code().unwrap_or(-1), json)
}

fn ok(root: &Path, session: &str, args: &[&str]) -> Value {
    let (code, json) = run(root, session, args);
    assert_eq!(code, 0, "`{}` failed: {json}", args.join(" "));
    json
}

#[test]
fn an_am_in_progress_is_reported_and_aborted() {
    let (f, _) = am_fixture(true);
    let s = "cli-am-abort";
    let state = ok(f.root(), s, &["conflicts", "state"]);
    assert_eq!(state["operation"], "am");
    assert_eq!(state["state"]["Am"]["total"], 2);
    let head = f.git(&["rev-parse", "HEAD"]);
    let aborted = ok(f.root(), s, &["conflicts", "am-abort"]);
    assert_eq!(aborted["aborted"], "am");
    assert!(ok(f.root(), s, &["conflicts", "state"]).is_null());
    assert_eq!(f.git(&["rev-parse", "HEAD"]), head);
}

#[test]
fn am_skip_and_continue_report_their_outcome() {
    let (f, _) = am_fixture(true);
    let s = "cli-am-skip";
    let out = ok(f.root(), s, &["conflicts", "am-skip"]);
    assert_eq!(out["outcome"], "Completed");
    assert_eq!(f.git(&["log", "-1", "--format=%s"]), "second patch");

    let (f, _) = am_fixture(false);
    let s = "cli-am-continue";
    std::fs::write(f.root().join("a.txt"), "resolved\n").unwrap();
    f.git(&["add", "a.txt"]);
    let out = ok(f.root(), s, &["conflicts", "am-continue"]);
    assert_eq!(out["outcome"], "Completed");
    assert_eq!(f.git(&["log", "-1", "--format=%s"]), "feature change");
}

#[test]
fn am_without_an_am_is_refused() {
    let f = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "x\n", "c")
        .unwrap()
        .build();
    let (code, json) = run(f.root(), "cli-am-none", &["conflicts", "am-skip"]);
    assert_eq!(code, 2, "{json}");
    assert_eq!(json["refused"]["reason"], "no_operation");
}

#[test]
fn a_stale_index_lock_is_shown_and_removed() {
    let f = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "x\n", "c")
        .unwrap()
        .build();
    let s = "cli-index-lock";
    assert!(ok(f.root(), s, &["index-lock", "show"]).is_null());
    let lock = f.root().join(".git/index.lock");
    std::fs::write(&lock, "").unwrap();
    let fresh = ok(f.root(), s, &["index-lock", "show"]);
    assert_eq!(fresh["stale"], false);
    let (code, json) = run(f.root(), s, &["index-lock", "remove"]);
    assert_ne!(code, 0, "a fresh lock is not removed: {json}");
    assert!(lock.exists());

    let file = std::fs::OpenOptions::new().write(true).open(&lock).unwrap();
    file.set_modified(SystemTime::now() - Duration::from_secs(120))
        .unwrap();
    let stale = ok(f.root(), s, &["index-lock", "show"]);
    assert_eq!(stale["stale"], true);
    assert_eq!(ok(f.root(), s, &["index-lock", "remove"])["removed"], true);
    assert!(!lock.exists());
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
