//! The undo safety net from the command line (the undo-safety-net design):
//! `undo --list` with what can be restored, `undo restore`, `reflog`,
//! `reflog restore`, `lost-commits`. JSON is deserialized, never matched as
//! text. Each command is its own short-lived embedded daemon, so these also
//! prove the journal persists between processes.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_core::domain::{
    JournalListing, LostCommitsPage, LostSource, Oid, OperationKind, ReflogPage, RestoreItemKind,
    RestoreOutcome, UndoOutcome, UndoRefusal,
};
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

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build()
}

fn rev(repo: &FixtureRepo, spec: &str) -> Oid {
    Oid::from_hex(&repo.git(&["rev-parse", spec])).unwrap()
}

fn id_of(listing: &JournalListing, kind: OperationKind) -> u64 {
    listing
        .undo
        .iter()
        .rev()
        .find(|e| e.summary.kind == kind)
        .unwrap()
        .summary
        .id
}

#[test]
fn list_offers_restore_and_undo_restore_brings_a_deleted_branch_back() {
    let repo = fixture();
    let session = "net-restore";
    repo.git(&["branch", "feature"]);
    let tip = rev(&repo, "feature");
    ok(&repo, session, &["branch", "delete", "feature"]);
    std::fs::write(repo.root().join("x.txt"), "x").unwrap();
    ok(&repo, session, &["stage", "--file", "x.txt"]);
    ok(&repo, session, &["commit", "-m", "x"]);
    let main = rev(&repo, "main");

    let listing: JournalListing =
        serde_json::from_value(ok(&repo, session, &["undo", "--list"])).unwrap();
    let id = id_of(&listing, OperationKind::BranchDelete);
    let entry = listing.undo.iter().find(|e| e.summary.id == id).unwrap();
    assert_eq!(entry.restore.items[0].kind, RestoreItemKind::Branch);
    let commit = listing
        .undo
        .iter()
        .find(|e| e.summary.kind == OperationKind::Commit)
        .unwrap();
    assert_eq!(commit.restore.items[0].kind, RestoreItemKind::OldTip);
    let staging = listing
        .undo
        .iter()
        .find(|e| e.summary.kind == OperationKind::StageFile)
        .unwrap();
    assert!(!staging.restore.restorable());
    assert!(staging.restore.not_restorable.is_some());

    // Dry run first, with the two-word spelling.
    let dry: RestoreOutcome = serde_json::from_value(ok(
        &repo,
        session,
        &["undo", "restore", &id.to_string(), "--dry-run"],
    ))
    .unwrap();
    assert!(!dry.applied);
    assert!(repo.git(&["for-each-ref", "refs/heads/feature"]).is_empty());

    let done: RestoreOutcome =
        serde_json::from_value(ok(&repo, session, &["undo", "restore", &id.to_string()])).unwrap();
    assert!(done.applied);
    assert_eq!(done.refs, vec!["refs/heads/feature".to_string()]);
    assert_eq!(rev(&repo, "feature"), tip);
    assert_eq!(rev(&repo, "main"), main, "later actions are kept");

    // The restore is itself undoable.
    let undone: UndoOutcome = serde_json::from_value(ok(&repo, session, &["undo"])).unwrap();
    assert_eq!(undone.entry.kind, OperationKind::JournalRestore);
    assert!(repo.git(&["for-each-ref", "refs/heads/feature"]).is_empty());
}

#[test]
fn a_taken_name_is_exit_2_with_a_suggestion_and_name_resolves_it() {
    let repo = fixture();
    let session = "net-name-taken";
    repo.git(&["branch", "feature"]);
    let tip = rev(&repo, "feature");
    ok(&repo, session, &["branch", "delete", "feature"]);
    repo.git(&["branch", "feature", "HEAD"]);
    repo.git(&["commit", "--allow-empty", "-m", "elsewhere"]);
    repo.git(&["branch", "-f", "feature", "HEAD"]);
    let listing: JournalListing =
        serde_json::from_value(ok(&repo, session, &["undo", "--list"])).unwrap();
    let id = id_of(&listing, OperationKind::BranchDelete).to_string();

    let (code, json) = run(&repo, session, &["undo-restore", &id]);
    assert_eq!(code, 2);
    let reason: UndoRefusal = serde_json::from_value(json["refused"]["reason"].clone()).unwrap();
    assert_eq!(
        reason,
        UndoRefusal::NameTaken {
            name: "feature".into(),
            suggested: "feature-restored".into()
        }
    );
    ok(
        &repo,
        session,
        &["undo", "restore", &id, "--name", "feature-restored"],
    );
    assert_eq!(rev(&repo, "feature-restored"), tip);
}

#[test]
fn a_nothing_to_restore_entry_is_exit_2_with_its_reason() {
    let repo = fixture();
    let session = "net-not-restorable";
    ok(&repo, session, &["branch", "create", "side"]);
    let listing: JournalListing =
        serde_json::from_value(ok(&repo, session, &["undo", "--list"])).unwrap();
    let id = id_of(&listing, OperationKind::BranchCreate).to_string();
    let (code, json) = run(&repo, session, &["undo", "restore", &id]);
    assert_eq!(code, 2);
    let reason: UndoRefusal = serde_json::from_value(json["refused"]["reason"].clone()).unwrap();
    assert!(
        matches!(reason, UndoRefusal::NotRestorable { .. }),
        "{reason:?}"
    );
    let (code, json) = run(&repo, session, &["undo", "restore", "9999"]);
    assert_eq!(code, 2);
    let reason: UndoRefusal = serde_json::from_value(json["refused"]["reason"].clone()).unwrap();
    assert_eq!(reason, UndoRefusal::NoSuchEntry { id: 9999 });
}

#[test]
fn reflog_lists_a_shell_s_moves_and_reflog_restore_moves_a_branch_back() {
    let repo = fixture();
    let session = "net-reflog";
    let first = rev(&repo, "HEAD");
    repo.git(&["checkout", "-q", "-b", "work"]);
    repo.git(&["commit", "--allow-empty", "-m", "w1"]);
    let w1 = rev(&repo, "work");
    repo.git(&["checkout", "-q", "main"]);
    repo.git(&["branch", "-f", "work", &first.to_hex()]);

    let page: ReflogPage =
        serde_json::from_value(ok(&repo, session, &["reflog", "work", "--limit", "5"])).unwrap();
    assert_eq!(page.refname, "refs/heads/work");
    assert_eq!(page.entries[0].new, first);
    assert_eq!(page.entries[1].new, w1);
    assert_eq!(page.entries[1].message, "commit: w1");
    let head: ReflogPage = serde_json::from_value(ok(&repo, session, &["reflog"])).unwrap();
    assert_eq!(head.refname, "HEAD");
    let paged: ReflogPage = serde_json::from_value(ok(
        &repo,
        session,
        &["reflog", "work", "--skip", "1", "--limit", "1"],
    ))
    .unwrap();
    assert_eq!(paged.entries.len(), 1);
    assert_eq!(paged.entries[0].new, w1);

    let out: RestoreOutcome = serde_json::from_value(ok(
        &repo,
        session,
        &["reflog", "restore", "work", &w1.to_hex()],
    ))
    .unwrap();
    assert!(out.applied);
    assert_eq!(rev(&repo, "work"), w1);
    let undone: UndoOutcome = serde_json::from_value(ok(&repo, session, &["undo"])).unwrap();
    assert_eq!(undone.entry.kind, OperationKind::ReflogRestore);
    assert_eq!(rev(&repo, "work"), first);
}

#[test]
fn reflog_restore_of_a_dirty_checked_out_branch_needs_a_mode() {
    let repo = fixture();
    let session = "net-reflog-dirty";
    let first = rev(&repo, "HEAD");
    std::fs::write(repo.root().join("x.txt"), "x").unwrap();
    ok(&repo, session, &["stage", "--file", "x.txt"]);
    ok(&repo, session, &["commit", "-m", "x"]);
    std::fs::write(repo.root().join("x.txt"), "dirty").unwrap();
    let (code, json) = run(&repo, session, &["reflog-restore", "main", &first.to_hex()]);
    assert_eq!(code, 2);
    let reason: UndoRefusal = serde_json::from_value(json["refused"]["reason"].clone()).unwrap();
    assert!(
        matches!(reason, UndoRefusal::WouldOverwrite { .. }),
        "{reason:?}"
    );
    assert_ne!(rev(&repo, "main"), first);
    // keep: refuses too when the dirty file is in the way, nothing changes.
    let (code, _) = run(
        &repo,
        session,
        &["reflog-restore", "main", &first.to_hex(), "--mode", "keep"],
    );
    assert_eq!(code, 2);
    assert_eq!(
        std::fs::read_to_string(repo.root().join("x.txt")).unwrap(),
        "dirty"
    );
}

#[test]
fn lost_commits_lists_what_a_deleted_branch_left() {
    let repo = fixture();
    let session = "net-lost";
    repo.git(&["checkout", "-q", "-b", "feature"]);
    repo.git(&["commit", "--allow-empty", "-m", "feature one"]);
    repo.git(&["commit", "--allow-empty", "-m", "feature two"]);
    let tip = rev(&repo, "HEAD");
    repo.git(&["checkout", "-q", "main"]);
    repo.git(&["branch", "-D", "feature"]);

    let page: LostCommitsPage =
        serde_json::from_value(ok(&repo, session, &["lost-commits"])).unwrap();
    let summaries: Vec<&str> = page.commits.iter().map(|c| c.summary.as_str()).collect();
    assert_eq!(summaries, ["feature two", "feature one"]);
    assert_eq!(page.commits[0].oid, tip);
    assert_eq!(page.commits[0].source, LostSource::Reflog);
    assert!(!page.more && !page.partial);
    let one: LostCommitsPage = serde_json::from_value(ok(
        &repo,
        session,
        &["lost-commits", "--limit", "1", "--no-deep"],
    ))
    .unwrap();
    assert_eq!(one.commits.len(), 1);
    assert!(one.more);
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
