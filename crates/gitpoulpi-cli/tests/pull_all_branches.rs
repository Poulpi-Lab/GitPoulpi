//! `gitpoulpi pull --all-branches`, through the real binary, the daemon and
//! the backend. Assertions are on deserialized JSON (the domain types), never
//! on text.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_core::domain::{BranchSyncStatus, PullAllBranchesOutcome, PullMode};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::Value;

fn config_dir() -> &'static Path {
    static DIR: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    DIR.get_or_init(|| tempfile::tempdir().unwrap()).path()
}

fn run(repo: &FixtureRepo, session: &str, args: &[&str]) -> (i32, Value) {
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config_dir())
        .env("GITPOULPI_GENERAL_AUTO_STASH", "ask")
        .args(["--json", "--session", session])
        .args(["--repo", &repo.root().to_string_lossy()])
        .args(args)
        .current_dir(repo.root())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "{args:?}: not JSON ({e}): {stdout}\n{}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output.status.code().unwrap_or(-1), value)
}

fn outcome(value: &Value) -> PullAllBranchesOutcome {
    serde_json::from_value(value.clone())
        .unwrap_or_else(|e| panic!("not an outcome ({e}): {value}"))
}

fn status_of(out: &PullAllBranchesOutcome, branch: &str) -> BranchSyncStatus {
    out.branches
        .iter()
        .find(|b| b.branch == branch)
        .unwrap_or_else(|| panic!("no item for {branch}: {out:?}"))
        .status
}

fn tip(repo: &FixtureRepo, rev: &str) -> String {
    repo.git(&["rev-parse", rev]).trim().to_string()
}

/// `main` (checked out) and `feature/x`, pushed to origin, tracking it, then
/// both moved forward on origin by someone else.
fn behind() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("base.txt", "base\n", "initial")
        .unwrap()
        .commit_file("a.txt", "1\n2\n3\n", "add a")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .branch("feature/x")
        .unwrap()
        .push("feature/x")
        .unwrap()
        .track("feature/x", "origin")
        .unwrap()
        .checkout("main")
        .unwrap()
        .build();
    repo.push_from_elsewhere("main", "main moves");
    repo.push_from_elsewhere("feature/x", "feature moves");
    repo
}

#[test]
fn pulls_every_branch_and_prints_the_outcome_as_json() {
    let repo = behind();
    let (code, value) = run(&repo, "cli-pab-all", &["pull", "--all-branches"]);
    assert_eq!(code, 0, "{value}");
    let out = outcome(&value);
    assert_eq!(out.fetched, ["origin"]);
    assert_eq!(status_of(&out, "main"), BranchSyncStatus::Updated);
    assert_eq!(status_of(&out, "feature/x"), BranchSyncStatus::Updated);
    assert_eq!(out.updated(), 2);
    assert_eq!(out.head_mode, Some(PullMode::Merge));
    assert_eq!(
        tip(&repo, "main"),
        repo.git_in_remote(&["rev-parse", "main"]).trim()
    );
    assert_eq!(
        tip(&repo, "feature/x"),
        repo.git_in_remote(&["rev-parse", "feature/x"]).trim()
    );
}

#[test]
fn undo_takes_the_whole_run_back_in_one_step() {
    let repo = behind();
    let (main_before, feature_before) = (tip(&repo, "main"), tip(&repo, "feature/x"));
    let (code, _) = run(&repo, "cli-pab-undo", &["pull", "--all-branches"]);
    assert_eq!(code, 0);
    assert_ne!(tip(&repo, "main"), main_before);

    let (code, value) = run(&repo, "cli-pab-undo", &["undo", "--yes"]);
    assert_eq!(code, 0, "{value}");
    assert_eq!(tip(&repo, "main"), main_before);
    assert_eq!(tip(&repo, "feature/x"), feature_before);
    assert!(!repo.root().join("main moves.txt").exists());
}

#[test]
fn mode_applies_to_the_checked_out_branch_and_worktrees_is_a_choice() {
    let repo = behind();
    let (code, value) = run(
        &repo,
        "cli-pab-mode",
        &[
            "pull",
            "--all-branches",
            "--mode",
            "ff-only",
            "--worktrees",
            "update",
        ],
    );
    assert_eq!(code, 0, "{value}");
    let out = outcome(&value);
    assert_eq!(out.head_mode, Some(PullMode::FastForwardOnly));
    assert_eq!(status_of(&out, "main"), BranchSyncStatus::Updated);
}

#[test]
fn no_fetch_works_from_the_tracking_refs_as_they_are() {
    let repo = behind();
    let before = tip(&repo, "main");
    let (code, value) = run(
        &repo,
        "cli-pab-nofetch",
        &["pull", "--all-branches", "--no-fetch"],
    );
    assert_eq!(code, 0, "{value}");
    let out = outcome(&value);
    assert!(out.fetched.is_empty());
    assert_eq!(out.updated(), 0);
    assert_eq!(tip(&repo, "main"), before);
}

#[test]
fn a_failing_fetch_exits_4_and_names_the_remote() {
    let repo = behind();
    let dead = repo.root().join("nowhere.git");
    repo.git(&["remote", "add", "dead", dead.to_str().unwrap()]);
    repo.git(&["branch", "stale", "origin/main"]);
    repo.git(&["config", "branch.stale.remote", "dead"]);
    repo.git(&["config", "branch.stale.merge", "refs/heads/main"]);
    let (code, value) = run(&repo, "cli-pab-failed", &["pull", "--all-branches"]);
    assert_eq!(code, 4, "{value}");
    let out = outcome(&value);
    assert_eq!(out.fetch_failed.len(), 1);
    assert_eq!(out.fetch_failed[0].remote, "dead");
    assert_eq!(status_of(&out, "stale"), BranchSyncStatus::FetchFailed);
    // The healthy remote's branches still came up to date.
    assert_eq!(status_of(&out, "main"), BranchSyncStatus::Updated);
}

#[test]
fn local_changes_in_the_way_are_a_refusal_with_exit_2_and_autostash_goes_through() {
    let repo = behind();
    // Someone else edits line 1 of `a.txt` on main; the user edits it too.
    let scratch = tempfile::tempdir().unwrap();
    let bare = repo.git_in_remote(&["rev-parse", "--absolute-git-dir"]);
    let dir = scratch.path().join("w");
    let git = |cwd: &Path, args: &[&str]| {
        let out = std::process::Command::new("git")
            .args([
                "-c",
                "user.name=Else",
                "-c",
                "user.email=else@gitpoulpi.local",
            ])
            .args(args)
            .current_dir(cwd)
            .output()
            .unwrap();
        assert!(out.status.success(), "git {args:?}");
    };
    git(
        scratch.path(),
        &[
            "clone",
            "--branch",
            "main",
            bare.trim(),
            dir.to_str().unwrap(),
        ],
    );
    std::fs::write(dir.join("a.txt"), "1 theirs\n2\n3\n").unwrap();
    git(&dir, &["commit", "-qam", "theirs"]);
    git(&dir, &["push", "-q", "origin", "main"]);
    std::fs::write(repo.root().join("a.txt"), "1 mine\n2\n3\n").unwrap();

    let feature_before = tip(&repo, "feature/x");
    // The setting is `ask` here: the CLI is told to pass `--autostash`.
    let (code, value) = run(&repo, "cli-pab-refused", &["pull", "--all-branches"]);
    assert_eq!(code, 2, "{value}");
    assert_eq!(value["refused"]["reason"], "needs_autostash", "{value}");
    assert_eq!(tip(&repo, "feature/x"), feature_before, "nothing moved");

    // With `--autostash` it goes through; the edit (moved to a line the
    // remote did not touch) is back afterwards.
    std::fs::write(repo.root().join("a.txt"), "1\n2\n3 mine\n").unwrap();
    let (code, value) = run(
        &repo,
        "cli-pab-refused",
        &["pull", "--all-branches", "--autostash"],
    );
    assert_eq!(code, 0, "{value}");
    let out = outcome(&value);
    assert_eq!(status_of(&out, "feature/x"), BranchSyncStatus::Updated);
    assert_eq!(
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap(),
        "1 theirs\n2\n3 mine\n"
    );
}

#[test]
fn it_conflicts_with_remote_and_all_sources() {
    let repo = behind();
    for other in [["--remote", "origin"], ["--all-sources", ""]] {
        let mut args = vec!["pull", "--all-branches", other[0]];
        if !other[1].is_empty() {
            args.push(other[1]);
        }
        let output = Command::cargo_bin("gitpoulpi")
            .unwrap()
            .env("GITPOULPI_CONFIG_DIR", config_dir())
            .args(["--json", "--session", "cli-pab-conflict"])
            .args(["--repo", &repo.root().to_string_lossy()])
            .args(&args)
            .output()
            .unwrap();
        assert!(!output.status.success(), "{args:?}");
    }
}
