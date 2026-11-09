//! `gitpoulpi undo` of a stash drop and of a remote removal through the real binary: each command is its own short-lived
//! embedded daemon, so these also prove the journal and its pins persist
//! between processes. A push, which touches a server, is never undone; deleting a remote branch or tag is.

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

/// A repository whose local bare `origin` has `main`, the branch `topic` and
/// the annotated tag `v1`.
fn with_remote_refs() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .build();
    repo.git(&["branch", "topic"]);
    repo.git(&["tag", "-a", "v1", "-m", "release one"]);
    repo.git(&["push", "origin", "topic", "refs/tags/v1"]);
    repo
}

#[test]
fn deleting_a_remote_branch_is_undone_after_a_confirmation_and_redone() {
    let repo = with_remote_refs();
    let session = "undo-s6-remote-branch";
    let tip = repo.git(&["rev-parse", "topic"]);
    ok(
        &repo,
        session,
        &["branch", "delete-remote", "topic", "--remote", "origin"],
    );
    assert_eq!(repo.git_in_remote(&["branch", "--list", "topic"]), "");

    // Re-creating a branch on the server asks first (exit 3), says so, and
    // changes nothing.
    let (code, json) = run(&repo, session, &["undo"]);
    assert_eq!(code, 3, "{json}");
    let prompt = json["needs_confirmation"]["prompt"].as_str().unwrap();
    assert!(
        prompt.contains("re-creates branch topic on origin"),
        "{prompt}"
    );
    assert_eq!(repo.git_in_remote(&["branch", "--list", "topic"]), "");

    let undone: UndoOutcome =
        serde_json::from_value(ok(&repo, session, &["undo", "--yes"])).unwrap();
    assert!(undone.applied);
    assert_eq!(undone.entry.label, "Delete branch topic on origin");
    assert!(
        undone
            .steps
            .iter()
            .any(|s| s.contains("re-create topic on origin")),
        "{:?}",
        undone.steps
    );
    assert_eq!(repo.git_in_remote(&["rev-parse", "refs/heads/topic"]), tip);

    let redone: UndoOutcome =
        serde_json::from_value(ok(&repo, session, &["redo", "--yes"])).unwrap();
    assert!(redone.applied);
    assert_eq!(repo.git_in_remote(&["branch", "--list", "topic"]), "");
}

#[test]
fn deleting_a_remote_tag_is_undone_to_the_same_annotated_object() {
    let repo = with_remote_refs();
    let session = "undo-s6-remote-tag";
    let object = repo.git(&["rev-parse", "refs/tags/v1"]);
    ok(
        &repo,
        session,
        &["tag", "delete-remote", "v1", "--remote", "origin"],
    );
    assert_eq!(repo.git_in_remote(&["tag", "--list"]), "");
    ok(&repo, session, &["undo", "--yes"]);
    assert_eq!(
        repo.git_in_remote(&["rev-parse", "refs/tags/v1"]),
        object,
        "the tag object itself, so the annotation is kept"
    );
    assert_eq!(
        repo.git_in_remote(&["cat-file", "-t", "refs/tags/v1"]),
        "tag"
    );
}

#[test]
fn undoing_a_remote_deletion_refuses_when_the_branch_was_recreated() {
    let repo = with_remote_refs();
    let session = "undo-s6-remote-moved";
    ok(
        &repo,
        session,
        &["branch", "delete-remote", "topic", "--remote", "origin"],
    );
    repo.git(&["commit", "--allow-empty", "-m", "elsewhere"]);
    repo.git(&["push", "origin", "HEAD:refs/heads/topic"]);
    let theirs = repo.git(&["rev-parse", "HEAD"]);

    let (code, json) = run(&repo, session, &["undo", "--yes"]);
    assert_eq!(code, 2, "{json}");
    let reason: UndoRefusal = serde_json::from_value(json["refused"]["reason"].clone()).unwrap();
    assert!(
        matches!(&reason, UndoRefusal::RemoteRefMoved(m)
            if m.remote == "origin" && m.name == "refs/heads/topic"),
        "{reason:?}"
    );
    assert_eq!(
        repo.git_in_remote(&["rev-parse", "refs/heads/topic"]),
        theirs,
        "nothing was pushed over it"
    );
}

#[test]
fn tag_delete_both_is_one_entry_undone_in_both_places() {
    let repo = with_remote_refs();
    let session = "undo-s6-tag-both";
    let object = repo.git(&["rev-parse", "refs/tags/v1"]);
    // The remote-tag mirror learns the remote has it (a plain push leaves it unknown).
    ok(&repo, session, &["tag", "push", "v1", "--remote", "origin"]);
    ok(&repo, session, &["tag", "delete", "v1", "--both"]);
    assert_eq!(repo.git(&["tag", "--list"]), "");
    assert_eq!(repo.git_in_remote(&["tag", "--list"]), "");
    ok(&repo, session, &["undo", "--yes"]);
    assert_eq!(repo.git(&["rev-parse", "refs/tags/v1"]), object);
    assert_eq!(repo.git_in_remote(&["rev-parse", "refs/tags/v1"]), object);
    let listing = ok(&repo, session, &["undo", "--list"]);
    assert_eq!(
        listing["redo"].as_array().map(Vec::len),
        Some(1),
        "{listing}"
    );
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
