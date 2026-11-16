//! `gitpoulpi stage --paths a b` / `unstage --paths a b`: several paths as
//! ONE journaled operation, so a moved file (its old and new path) is one
//! undo step. Real binary, JSON asserted by deserializing.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_core::domain::{JournalListing, OperationKind};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::Value;

fn config_dir() -> &'static Path {
    static DIR: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    DIR.get_or_init(|| tempfile::tempdir().expect("temp config dir"))
        .path()
}

fn ok(repo: &FixtureRepo, session: &str, args: &[&str]) -> Value {
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
    assert_eq!(
        output.status.code(),
        Some(0),
        "`gitpoulpi {}` failed: {stdout}",
        args.join(" ")
    );
    serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("not JSON ({e}): {stdout}"))
}

fn porcelain(repo: &FixtureRepo) -> String {
    repo.git(&["status", "--porcelain=v1", "--untracked-files=all"])
}

#[test]
fn staging_a_move_with_paths_is_one_undo_entry() {
    let body: String = (1..=12).map(|n| format!("line {n}\n")).collect();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("old.txt", &body, "first")
        .unwrap()
        .build();
    let session = "stage-paths";
    std::fs::rename(repo.root().join("old.txt"), repo.root().join("new.txt")).unwrap();

    ok(&repo, session, &["stage", "--paths", "new.txt", "old.txt"]);
    assert_eq!(porcelain(&repo).trim(), "R  old.txt -> new.txt");

    let listing: JournalListing =
        serde_json::from_value(ok(&repo, session, &["undo", "--list"])).unwrap();
    assert_eq!(listing.undo.len(), 1, "{:?}", listing.undo);
    assert_eq!(listing.undo[0].summary.kind, OperationKind::StageFile);
    assert_eq!(listing.undo[0].summary.label, "Stage new.txt and old.txt");

    // One undo puts both index entries back.
    ok(&repo, session, &["undo"]);
    let status = porcelain(&repo);
    assert!(status.contains("D old.txt"), "{status}");
    assert!(status.contains("?? new.txt"), "{status}");

    // And unstaging it is one entry too.
    ok(&repo, session, &["redo"]);
    ok(
        &repo,
        session,
        &["unstage", "--paths", "new.txt", "old.txt"],
    );
    let listing: JournalListing =
        serde_json::from_value(ok(&repo, session, &["undo", "--list"])).unwrap();
    assert_eq!(listing.undo.len(), 2, "{:?}", listing.undo);
    assert_eq!(listing.undo[1].summary.kind, OperationKind::UnstageFile);
    ok(&repo, session, &["undo"]);
    assert_eq!(porcelain(&repo).trim(), "R  old.txt -> new.txt");
}
