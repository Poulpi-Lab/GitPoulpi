//! `gitpoulpi undo` of a stash drop and of a remote removal through the real binary: each command is its own short-lived
//! embedded daemon, so these also prove the journal and its pins persist
//! between processes. A push, which touches a server, is never undone.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_core::domain::{OperationKind, UndoOutcome, UndoRefusal};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::Value;

fn config_dir() -> &'static Path {
    static DIR: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    DIR.get_or_init(|| tempfile::tempdir().expect("temp config dir"))
        .path()
}

fn run(repo: &FixtureRepo, session: &str, args: &[&str]) -> (i32, Value) {
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config_dir())
        .env("GITPOULPI_GENERAL_UNDO_DEPTH", "0")
        .env("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false")
        .arg("--json")
        .args(["--session", session])
        .args(["--repo", &repo.root().to_string_lossy()])
        .args(args)
        .current_dir(repo.root())
        .output()
        .expect("failed to run gitpoulpi");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let json = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "not JSON ({e}): {stdout}\nstderr: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output.status.code().unwrap_or(-1), json)
}

fn ok(repo: &FixtureRepo, session: &str, args: &[&str]) -> Value {
    let (code, json) = run(repo, session, args);
    assert_eq!(code, 0, "`gitpoulpi {}` failed: {json}", args.join(" "));
    json
}

#[test]
fn a_stash_drop_is_undone_with_the_same_commit_and_redone() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "first")
        .unwrap()
        .build();
    let session = "undo-s6-stash";
    std::fs::write(repo.root().join("a.txt"), "one\nedited\n").unwrap();
    ok(&repo, session, &["stash", "push", "--message", "wip"]);
    let listed = repo.git(&["stash", "list", "--format=%H %gs"]);
    ok(&repo, session, &["stash", "drop", "0"]);
    assert!(repo.git(&["stash", "list"]).is_empty());

    let undone: UndoOutcome = serde_json::from_value(ok(&repo, session, &["undo"])).unwrap();
    assert!(undone.applied);
    assert_eq!(undone.entry.kind, OperationKind::StashDrop);
    assert!(undone.entry.label.starts_with("Drop stash \"wip\""));
    assert_eq!(
        repo.git(&["stash", "list", "--format=%H %gs"]),
        listed,
        "the very same stash commit, back as stash@{{0}}"
    );

    let redone: UndoOutcome = serde_json::from_value(ok(&repo, session, &["redo"])).unwrap();
    assert!(redone.applied);
    assert!(repo.git(&["stash", "list"]).is_empty());

    // A stash stored again outside makes the next undo refuse.
    ok(&repo, session, &["undo"]);
    ok(&repo, session, &["stash", "drop", "0"]);
    let commit = listed.split(' ').next().unwrap().to_string();
    repo.git(&["stash", "store", "-m", "by hand", &commit]);
    let (code, json) = run(&repo, session, &["undo"]);
    assert_eq!(code, 2, "{json}");
    let reason: UndoRefusal = serde_json::from_value(json["refused"]["reason"].clone()).unwrap();
    assert!(
        matches!(reason, UndoRefusal::StashChanged { .. }),
        "{reason:?}"
    );
}

#[test]
fn a_remote_remove_is_undone_with_its_tracking_branches_across_processes() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .build();
    let session = "undo-s6-remote";
    let refs = || {
        repo.git(&[
            "for-each-ref",
            "refs/remotes/origin/main",
            "--format=%(refname) %(objectname)",
        ])
    };
    let before = refs();
    assert!(before.starts_with("refs/remotes/origin/main "), "{before}");
    let url = repo.git(&["config", "remote.origin.url"]);

    ok(&repo, session, &["remote", "remove", "origin"]);
    assert!(refs().is_empty());
    assert!(repo.git(&["remote"]).is_empty());

    let undone: UndoOutcome = serde_json::from_value(ok(&repo, session, &["undo"])).unwrap();
    assert_eq!(undone.entry.kind, OperationKind::RemoteRemove);
    assert_eq!(refs(), before, "the remote-tracking branch is back");
    assert_eq!(repo.git(&["config", "remote.origin.url"]), url);
    assert_eq!(repo.git(&["config", "branch.main.remote"]), "origin");

    let redone: UndoOutcome = serde_json::from_value(ok(&repo, session, &["redo"])).unwrap();
    assert!(redone.applied);
    assert!(refs().is_empty());
}

#[test]
fn a_push_is_not_undoable_and_the_refusal_says_why() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .with_origin()
        .unwrap()
        .build();
    let session = "undo-s6-push";
    ok(&repo, session, &["push", "--remote", "origin"]);
    let (code, json) = run(&repo, session, &["undo"]);
    assert_eq!(code, 2, "{json}");
    let message = json["refused"]["message"].as_str().unwrap();
    assert!(
        message.contains("nothing to undo") && message.contains("remote server"),
        "{message}"
    );
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
