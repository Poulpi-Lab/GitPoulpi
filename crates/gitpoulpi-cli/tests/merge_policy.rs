//! `gitpoulpi merge --ff | --no-ff | --ff-only` and the setting they override
//! (`git.merge_fast_forward`, default `never`), end to end through the real
//! binary: the JSON says whether the merge fast-forwarded (`FastForwarded`)
//! or made a merge commit (`Merged`), and what lands in the repository.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_core::domain::Oid;
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::Value;

fn config_dir() -> &'static Path {
    static DIR: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    DIR.get_or_init(|| tempfile::tempdir().expect("temp config dir"))
        .path()
}

/// `(exit code, parsed stdout)` of `gitpoulpi --json ...`; `setting` is the
/// global policy (the env override of `git.merge_fast_forward`) when given.
fn run(repo: &FixtureRepo, session: &str, setting: Option<&str>, args: &[&str]) -> (i32, Value) {
    let mut command = Command::cargo_bin("gitpoulpi").unwrap();
    command
        .env("GITPOULPI_CONFIG_DIR", config_dir())
        .env("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false")
        .env_remove("GITPOULPI_GIT_MERGE_FAST_FORWARD");
    if let Some(setting) = setting {
        command.env("GITPOULPI_GIT_MERGE_FAST_FORWARD", setting);
    }
    let output = command
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

fn parents(repo: &FixtureRepo) -> Vec<String> {
    repo.git(&["rev-list", "--parents", "-n1", "HEAD"])
        .split_whitespace()
        .skip(1)
        .map(str::to_string)
        .collect()
}

/// `main` untouched since `feature` was made from it; `feature` has two commits.
fn fast_forwardable() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .branch("feature")
        .unwrap()
        .commit_file("f1.txt", "1\n", "feature one")
        .unwrap()
        .commit_file("f2.txt", "2\n", "feature two")
        .unwrap()
        .checkout("main")
        .unwrap()
        .build()
}

#[test]
fn a_plain_merge_makes_a_merge_commit_by_default() {
    let repo = fast_forwardable();
    let old_main = head(&repo);
    let (code, json) = run(&repo, "cli-mff-default", None, &["merge", "feature"]);
    assert_eq!(code, 0, "{json}");
    assert!(json.get("Merged").is_some(), "a merge commit: {json}");
    assert!(json.get("FastForwarded").is_none(), "{json}");
    let parents = parents(&repo);
    assert_eq!(parents.len(), 2);
    assert_eq!(
        parents[0],
        old_main.to_hex(),
        "first parent is the old main"
    );
}

#[test]
fn ff_fast_forwards_and_overrides_the_setting() {
    let repo = fast_forwardable();
    let (code, json) = run(&repo, "cli-mff-ff", None, &["merge", "feature", "--ff"]);
    assert_eq!(code, 0, "{json}");
    assert!(json.get("FastForwarded").is_some(), "{json}");
    assert_eq!(
        head(&repo).to_hex(),
        repo.git(&["rev-parse", "feature"]).trim(),
        "main now is the feature tip"
    );
}

#[test]
fn the_setting_is_followed_when_no_flag_is_given_and_no_ff_overrides_it() {
    let repo = fast_forwardable();
    let (code, json) = run(
        &repo,
        "cli-mff-setting",
        Some("allowed"),
        &["merge", "feature"],
    );
    assert_eq!(code, 0, "{json}");
    assert!(json.get("FastForwarded").is_some(), "{json}");

    let repo = fast_forwardable();
    let (code, json) = run(
        &repo,
        "cli-mff-setting-no-ff",
        Some("allowed"),
        &["merge", "feature", "--no-ff"],
    );
    assert_eq!(code, 0, "{json}");
    assert!(json.get("Merged").is_some(), "{json}");
    assert_eq!(parents(&repo).len(), 2);
}

#[test]
fn ff_only_refuses_a_diverged_branch_and_moves_nothing() {
    let repo = fast_forwardable();
    std::fs::write(repo.root().join("m.txt"), "m\n").unwrap();
    repo.git(&["add", "m.txt"]);
    repo.git(&["commit", "-q", "-m", "main moved on"]);
    let before = head(&repo);
    let (code, json) = run(
        &repo,
        "cli-mff-only",
        None,
        &["merge", "feature", "--ff-only"],
    );
    assert_ne!(code, 0, "{json}");
    assert_eq!(head(&repo), before, "nothing moved");
    assert!(
        json["error"]
            .as_str()
            .unwrap_or_default()
            .to_lowercase()
            .contains("fast-forward"),
        "{json}"
    );

    // The same branch merges fine when the policy is not `only`.
    let (code, json) = run(&repo, "cli-mff-only-2", None, &["merge", "feature"]);
    assert_eq!(code, 0, "{json}");
    assert_eq!(parents(&repo).len(), 2);
}

#[test]
fn the_flags_exclude_each_other() {
    let repo = fast_forwardable();
    let before = head(&repo);
    for pair in [
        ["--ff", "--no-ff"],
        ["--ff", "--ff-only"],
        ["--no-ff", "--ff-only"],
        ["--squash", "--no-ff"],
    ] {
        let (code, _) = run(
            &repo,
            "cli-mff-excl",
            None,
            &["merge", "feature", pair[0], pair[1]],
        );
        assert_ne!(code, 0, "{pair:?}");
    }
    assert_eq!(head(&repo), before);
}
