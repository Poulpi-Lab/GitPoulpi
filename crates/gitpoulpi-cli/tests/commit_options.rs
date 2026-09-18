//! `gitpoulpi commit --no-verify` and `gitpoulpi merge --squash`, end to end
//! through the real binary (the same flags the MCP tools take): the JSON
//! outcomes, exit codes, and what lands in the repository.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_core::domain::{MergeOutcome, Oid};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::Value;

fn config_dir() -> &'static Path {
    static DIR: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    DIR.get_or_init(|| tempfile::tempdir().expect("temp config dir"))
        .path()
}

/// `(exit code, parsed stdout)` of `gitpoulpi --json ...`.
fn run(repo: &FixtureRepo, session: &str, args: &[&str]) -> (i32, Value) {
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config_dir())
        .env("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false")
        .arg("--json")
        .args(["--session", session])
        .args(["--repo", &repo.root().to_string_lossy()])
        .args(args)
        .current_dir(repo.root())
        .output()
        .expect("failed to run gitpoulpi");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let json = serde_json::from_str(&stdout).unwrap_or(Value::Null);
    (output.status.code().unwrap_or(-1), json)
}

fn head(repo: &FixtureRepo) -> Oid {
    Oid::from_hex(&repo.git(&["rev-parse", "HEAD"])).unwrap()
}

#[cfg(unix)]
#[test]
fn commit_no_verify_skips_a_failing_pre_commit_hook() {
    use std::os::unix::fs::PermissionsExt;
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let hook = repo.root().join(".git/hooks/pre-commit");
    std::fs::create_dir_all(hook.parent().unwrap()).unwrap();
    std::fs::write(&hook, "#!/bin/sh\necho refused >&2\nexit 1\n").unwrap();
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
    let session = "cli-no-verify";
    let before = head(&repo);
    std::fs::write(repo.root().join("x.txt"), "x").unwrap();
    assert_eq!(
        run(&repo, session, &["stage", "--file", "x.txt"]).0,
        0,
        "staged"
    );

    let (code, _) = run(&repo, session, &["commit", "-m", "blocked"]);
    assert_ne!(code, 0, "the hook blocks a plain commit");
    assert_eq!(head(&repo), before);

    let (code, json) = run(&repo, session, &["commit", "-m", "past it", "--no-verify"]);
    assert_eq!(code, 0, "{json}");
    let commit: Oid = serde_json::from_value(json["commit"].clone()).unwrap();
    assert_eq!(head(&repo), commit);
    assert_eq!(repo.git(&["log", "-1", "--format=%s"]).trim(), "past it");
}

#[test]
fn merge_squash_stages_the_changes_and_makes_no_commit() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .branch("feature")
        .unwrap()
        .checkout("feature")
        .unwrap()
        .commit_file("f.txt", "f\n", "feature work")
        .unwrap()
        .checkout("main")
        .unwrap()
        .commit_file("m.txt", "m\n", "main work")
        .unwrap()
        .build();
    let session = "cli-squash";
    let before = head(&repo);
    let (code, json) = run(&repo, session, &["merge", "feature", "--squash"]);
    assert_eq!(code, 0, "{json}");
    let outcome: MergeOutcome = serde_json::from_value(json).unwrap();
    assert!(matches!(outcome, MergeOutcome::Merged(_)), "{outcome:?}");
    assert_eq!(head(&repo), before, "no commit");
    assert_eq!(
        repo.git(&["diff", "--cached", "--name-only"]).trim(),
        "f.txt"
    );
    assert!(!repo.root().join(".git/MERGE_HEAD").exists());

    // The commit that follows is an ordinary one-parent commit.
    let (code, json) = run(&repo, session, &["commit", "-m", "squashed feature"]);
    assert_eq!(code, 0, "{json}");
    assert_eq!(
        repo.git(&["rev-list", "--parents", "-n1", "HEAD"])
            .split_whitespace()
            .count(),
        2
    );
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
