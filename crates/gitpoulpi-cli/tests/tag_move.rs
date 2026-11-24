//! `gitpoulpi tag move` and `tag merge-branch` through the real binary: the
//! JSON they print, the refusals, and the one undo entry each leaves.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_core::domain::UndoOutcome;
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

/// `initial`, an annotated tag `v1` on it, then `second` on `main`.
fn repo_with_tag() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial")
        .unwrap()
        .annotated_tag("v1")
        .unwrap()
        .commit_file("b.txt", "two\n", "second")
        .unwrap()
        .build()
}

#[test]
fn tag_move_fast_forwards_an_annotated_tag_and_is_one_undo_entry() {
    let repo = repo_with_tag();
    let session = "tag-move-ff";
    let tip = repo.git(&["rev-parse", "HEAD"]);
    let old_object = repo.git(&["rev-parse", "refs/tags/v1"]);

    let json = ok(&repo, session, &["tag", "move", "v1", "main"]);
    assert_eq!(json["moved"], "v1");
    assert_eq!(json["to"], tip.as_str());
    assert_eq!(json["forced"], false);
    assert_eq!(repo.git(&["rev-parse", "v1^{commit}"]), tip);
    assert_eq!(
        repo.git(&["cat-file", "-t", "refs/tags/v1"]),
        "tag",
        "still annotated"
    );
    assert!(repo
        .git(&["tag", "-l", "--format=%(contents)", "v1"])
        .contains("v1"));

    let undone: UndoOutcome = serde_json::from_value(ok(&repo, session, &["undo"])).unwrap();
    assert!(undone.applied);
    assert_eq!(undone.entry.label, "Move tag v1");
    assert_eq!(
        repo.git(&["rev-parse", "refs/tags/v1"]),
        old_object,
        "undo restores the old tag object"
    );
    let redone: UndoOutcome = serde_json::from_value(ok(&repo, session, &["redo"])).unwrap();
    assert!(redone.applied);
    assert_eq!(repo.git(&["rev-parse", "v1^{commit}"]), tip);
}

#[test]
fn tag_move_refuses_a_non_fast_forward_unless_forced() {
    let repo = repo_with_tag();
    let session = "tag-move-non-ff";
    repo.git(&["tag", "later", "HEAD"]);
    let tip = repo.git(&["rev-parse", "HEAD"]);

    let (code, json) = run(&repo, session, &["tag", "move", "later", "HEAD~1"]);
    assert_ne!(code, 0, "{json}");
    assert!(json.get("error").is_some(), "{json}");
    assert_eq!(repo.git(&["rev-parse", "later"]), tip, "nothing moved");

    let json = ok(
        &repo,
        session,
        &["tag", "move", "later", "HEAD~1", "--force-non-ff"],
    );
    assert_eq!(json["forced"], true);
    assert_eq!(
        repo.git(&["rev-parse", "later"]),
        repo.git(&["rev-parse", "HEAD~1"])
    );
}

#[test]
fn tag_merge_branch_makes_a_merge_commit_and_moves_only_the_tag() {
    let repo = repo_with_tag();
    let session = "tag-merge-branch";
    repo.git(&["checkout", "-q", "-b", "side", "v1^{commit}"]);
    std::fs::write(repo.root().join("side.txt"), "side\n").unwrap();
    repo.git(&["add", "side.txt"]);
    repo.git(&["commit", "-q", "-m", "side work"]);
    let side = repo.git(&["rev-parse", "side"]);
    repo.git(&["checkout", "-q", "main"]);
    let main = repo.git(&["rev-parse", "main"]);
    repo.git(&["tag", "-f", "v1", "main"]);
    let old_object = repo.git(&["rev-parse", "refs/tags/v1"]);

    let json = ok(&repo, session, &["tag", "merge-branch", "v1", "side"]);
    let merged = json["commit"].as_str().unwrap().to_string();
    assert_eq!(json["tag"], "v1");
    assert_eq!(
        repo.git(&["rev-list", "--parents", "-n1", &merged]),
        format!("{merged} {main} {side}")
    );
    assert_eq!(repo.git(&["rev-parse", "v1^{commit}"]), merged);
    assert_eq!(repo.git(&["rev-parse", "main"]), main, "no branch moved");

    let undone: UndoOutcome = serde_json::from_value(ok(&repo, session, &["undo"])).unwrap();
    assert_eq!(undone.entry.label, "Merge side into tag v1");
    assert_eq!(repo.git(&["rev-parse", "refs/tags/v1"]), old_object);
}

#[test]
fn tag_merge_branch_refuses_a_conflict_and_changes_nothing() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "base\n", "base")
        .unwrap()
        .build();
    let session = "tag-merge-conflict";
    repo.git(&["checkout", "-q", "-b", "side"]);
    std::fs::write(repo.root().join("a.txt"), "two\n").unwrap();
    repo.git(&["commit", "-q", "-am", "side edit"]);
    repo.git(&["checkout", "-q", "main"]);
    std::fs::write(repo.root().join("a.txt"), "one\n").unwrap();
    repo.git(&["commit", "-q", "-am", "main edit"]);
    repo.git(&["tag", "v1"]);
    let tip = repo.git(&["rev-parse", "HEAD"]);

    let (code, json) = run(&repo, session, &["tag", "merge-branch", "v1", "side"]);
    assert_ne!(code, 0, "{json}");
    assert!(json.get("error").is_some(), "{json}");
    assert_eq!(repo.git(&["rev-parse", "v1"]), tip);
    assert!(repo.git(&["status", "--porcelain"]).is_empty());
}
