//! `--autostash` on checkout/merge/rebase/pull, and the `ask` default's
//! refusal, through the real binary and daemon.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::Value;

fn config_dir() -> &'static Path {
    static DIR: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    DIR.get_or_init(|| tempfile::tempdir().unwrap()).path()
}

fn run(root: &Path, session: &str, args: &[&str]) -> std::process::Output {
    Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config_dir())
        .arg("--json")
        .args(["--session", session])
        .args(["--repo", &root.to_string_lossy()])
        .args(args)
        .current_dir(root)
        .output()
        .unwrap()
}

fn blocked_repo() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "1\n2\n3\n4\n5\n", "base")
        .unwrap()
        .branch("other")
        .unwrap()
        .checkout("other")
        .unwrap()
        .commit_file("a.txt", "1 other\n2\n3\n4\n5\n", "other edit")
        .unwrap()
        .checkout("main")
        .unwrap()
        .build();
    std::fs::write(repo.root().join("a.txt"), "1\n2\n3\n4\n5 local\n").unwrap();
    repo
}

#[test]
fn a_blocked_checkout_is_refused_by_default_and_stashed_with_the_flag() {
    let repo = blocked_repo();
    let session = "cli-test-autostash";

    let refused = run(repo.root(), session, &["checkout", "other"]);
    assert!(!refused.status.success());
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&refused.stdout),
        String::from_utf8_lossy(&refused.stderr)
    );
    assert!(said.contains("--autostash"), "{said}");
    assert_eq!(
        repo.git(&["rev-parse", "--abbrev-ref", "HEAD"]).trim(),
        "main"
    );

    let done = run(repo.root(), session, &["checkout", "other", "--autostash"]);
    assert!(
        done.status.success(),
        "{}",
        String::from_utf8_lossy(&done.stderr)
    );
    let json: Value = serde_json::from_slice(&done.stdout).unwrap();
    assert!(json.get("checked_out").is_some(), "{json}");
    assert_eq!(
        repo.git(&["rev-parse", "--abbrev-ref", "HEAD"]).trim(),
        "other"
    );
    assert_eq!(
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap(),
        "1 other\n2\n3\n4\n5 local\n"
    );
    assert!(repo.git(&["stash", "list"]).is_empty());
}

#[test]
fn merge_and_rebase_take_the_flag_too() {
    let repo = blocked_repo();
    let session = "cli-test-autostash-merge";
    let done = run(repo.root(), session, &["merge", "other", "--autostash"]);
    assert!(
        done.status.success(),
        "{}",
        String::from_utf8_lossy(&done.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap(),
        "1 other\n2\n3\n4\n5 local\n"
    );

    let repo = blocked_repo();
    repo.git(&["stash", "-q"]);
    repo.git(&["checkout", "-q", "-b", "feature"]);
    std::fs::write(repo.root().join("f.txt"), "f\n").unwrap();
    repo.git(&["add", "f.txt"]);
    repo.git(&["commit", "-q", "-m", "feature work"]);
    repo.git(&["stash", "pop", "-q"]);
    let done = run(
        repo.root(),
        "cli-test-autostash-rebase",
        &["rebase", "other", "--autostash"],
    );
    assert!(
        done.status.success(),
        "{}",
        String::from_utf8_lossy(&done.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap(),
        "1 other\n2\n3\n4\n5 local\n"
    );
}

/// `origin/main` has moved on with `remote_edit` of `a.txt`; the local
/// `main` is on the base commit with its last line edited, so a pull is
/// blocked by the local change.
fn blocked_pull_repo(remote_edit: &str) -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "1\n2\n3\n4\n5\n", "base")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .build();
    let scratch = repo.root().parent().unwrap().join("scratch-clone");
    let remote = repo.remote_path().unwrap().to_string_lossy().to_string();
    let scratch_str = scratch.to_string_lossy().to_string();
    repo.git_at(repo.root(), &["clone", "-q", &remote, &scratch_str]);
    std::fs::write(scratch.join("a.txt"), remote_edit).unwrap();
    repo.git_at(&scratch, &["add", "a.txt"]);
    repo.git_at(
        &scratch,
        &[
            "-c",
            "user.name=Else",
            "-c",
            "user.email=else@x.test",
            "commit",
            "-q",
            "-m",
            "remote edit",
        ],
    );
    repo.git_at(&scratch, &["push", "-q", "origin", "main"]);
    std::fs::write(repo.root().join("a.txt"), "1\n2\n3\n4\n5 local\n").unwrap();
    repo
}

#[test]
fn a_blocked_pull_is_refused_by_default_and_stashed_with_the_flag() {
    let repo = blocked_pull_repo("1 remote\n2\n3\n4\n5\n");
    let session = "cli-test-autostash-pull";

    let refused = run(repo.root(), session, &["pull"]);
    assert!(!refused.status.success());
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&refused.stdout),
        String::from_utf8_lossy(&refused.stderr)
    );
    assert!(said.contains("--autostash"), "{said}");
    assert!(repo.git(&["stash", "list"]).is_empty());

    let done = run(repo.root(), session, &["pull", "--autostash"]);
    assert!(
        done.status.success(),
        "{}",
        String::from_utf8_lossy(&done.stderr)
    );
    let _: Value = serde_json::from_slice(&done.stdout).unwrap();
    assert_eq!(
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap(),
        "1 remote\n2\n3\n4\n5 local\n"
    );
    assert!(repo.git(&["stash", "list"]).is_empty());
    let state = std::fs::read_to_string(repo.root().join(".git/gitpoulpi/repo-state.toml"))
        .unwrap_or_default();
    assert!(!state.contains("pending_auto_stash"), "{state}");
}

#[test]
fn a_pull_whose_pop_conflicts_keeps_the_stash_and_says_so() {
    let repo = blocked_pull_repo("1\n2\n3\n4\n5 remote\n");
    let done = run(
        repo.root(),
        "cli-test-autostash-pull-conflict",
        &["pull", "--autostash"],
    );
    assert!(!done.status.success());
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&done.stdout),
        String::from_utf8_lossy(&done.stderr)
    );
    assert!(said.contains("stash"), "{said}");
    assert!(!repo.git(&["stash", "list"]).is_empty());
    let state =
        std::fs::read_to_string(repo.root().join(".git/gitpoulpi/repo-state.toml")).unwrap();
    assert!(state.contains("Conflicted"), "{state}");
}

#[test]
fn status_json_reports_the_pending_auto_stash_until_it_is_dropped() {
    let repo = blocked_repo();
    let session = "cli-test-autostash-status";
    let status = |repo: &FixtureRepo| -> Value {
        let out = run(repo.root(), session, &["status"]);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        serde_json::from_slice(&out.stdout).unwrap()
    };
    // The key is always there; `null` without a pending stash.
    let clean = status(&repo);
    assert!(
        clean.get("pending_auto_stash").is_some_and(Value::is_null),
        "{clean}"
    );

    // A local edit on the very line `other` changed: the checkout goes
    // through, putting the changes back conflicts, the record stays.
    std::fs::write(repo.root().join("a.txt"), "1 local\n2\n3\n4\n5\n").unwrap();
    let done = run(repo.root(), session, &["checkout", "other", "--autostash"]);
    assert!(!done.status.success(), "the pop must conflict");
    let json = status(&repo);
    let pending = &json["pending_auto_stash"];
    assert_eq!(pending["state"], "Conflicted", "{json}");
    assert_eq!(pending["reason"], "Checkout", "{json}");
    assert_eq!(pending["stash_id"], 0, "{json}");

    // Dropping the stash clears it.
    repo.git(&["reset", "-q", "--hard"]);
    let dropped = run(repo.root(), session, &["stash", "drop", "0"]);
    assert!(
        dropped.status.success(),
        "{}",
        String::from_utf8_lossy(&dropped.stderr)
    );
    let json = status(&repo);
    assert!(json["pending_auto_stash"].is_null(), "{json}");
}

#[test]
fn a_pending_auto_stash_refuses_checkout_and_pull_until_popped() {
    let repo = blocked_repo();
    let session = "cli-test-autostash-block";
    std::fs::write(repo.root().join("a.txt"), "1 local\n2\n3\n4\n5\n").unwrap();
    let done = run(repo.root(), session, &["checkout", "other", "--autostash"]);
    assert!(!done.status.success(), "the pop must conflict");
    repo.git(&["reset", "-q", "--hard"]);
    repo.git(&["checkout", "-q", "main"]);

    for args in [
        &["checkout", "other"][..],
        &["pull"][..],
        &["merge", "other"][..],
    ] {
        let out = run(repo.root(), session, args);
        assert_eq!(out.status.code(), Some(1), "{args:?}");
        // `--json`: the refusal is a JSON object on stdout whose `error`
        // names the operation, the stash and the way out.
        let json: Value = serde_json::from_slice(&out.stdout)
            .unwrap_or_else(|e| panic!("{args:?}: not JSON ({e}): {:?}", out.stdout));
        let said = json["error"].as_str().unwrap_or_default();
        assert!(said.contains("is blocked"), "{args:?}: {json}");
        assert!(said.contains("stash@{0}"), "{args:?}: {json}");
        assert!(said.contains("gitpoulpi stash pop"), "{args:?}: {json}");
    }
    assert_eq!(
        repo.git(&["rev-parse", "--abbrev-ref", "HEAD"]).trim(),
        "main"
    );

    let popped = run(repo.root(), session, &["stash", "pop", "0"]);
    assert!(popped.status.success());
    repo.git(&["checkout", "-q", "--", "a.txt"]);
    let again = run(repo.root(), session, &["checkout", "other"]);
    assert!(
        again.status.success(),
        "{}",
        String::from_utf8_lossy(&again.stderr)
    );
}

#[test]
fn autostash_start_over_takes_a_conflicted_pull_back() {
    let repo = blocked_pull_repo("1\n2\n3\n4\n5 remote\n");
    let session = "cli-test-autostash-start-over";
    let before = repo.git(&["rev-parse", "HEAD"]).trim().to_string();
    let done = run(repo.root(), session, &["pull", "--autostash"]);
    assert!(!done.status.success(), "the pop must conflict");
    let conflicted = std::fs::read_to_string(repo.root().join("a.txt")).unwrap();

    // Dry run: says what it would do, changes nothing.
    let dry = run(
        repo.root(),
        session,
        &["autostash", "start-over", "--dry-run"],
    );
    assert!(
        dry.status.success(),
        "{}",
        String::from_utf8_lossy(&dry.stderr)
    );
    let json: Value = serde_json::from_slice(&dry.stdout).unwrap();
    assert_eq!(json["applied"], false, "{json}");
    assert!(json["steps"].as_array().unwrap().len() >= 4, "{json}");
    assert_eq!(
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap(),
        conflicted
    );

    // Without --yes: exit code 3, nothing changed.
    let asked = run(repo.root(), session, &["autostash", "start-over"]);
    assert_eq!(asked.status.code(), Some(3));
    let json: Value = serde_json::from_slice(&asked.stdout).unwrap();
    assert!(json["needs_confirmation"]["prompt"].is_string(), "{json}");
    assert!(!repo.git(&["stash", "list"]).is_empty());

    let done = run(repo.root(), session, &["autostash", "start-over", "--yes"]);
    assert!(
        done.status.success(),
        "{}{}",
        String::from_utf8_lossy(&done.stdout),
        String::from_utf8_lossy(&done.stderr)
    );
    let json: Value = serde_json::from_slice(&done.stdout).unwrap();
    assert_eq!(json["applied"], true, "{json}");
    assert_eq!(repo.git(&["rev-parse", "HEAD"]).trim(), before);
    assert_eq!(
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap(),
        "1\n2\n3\n4\n5 local\n"
    );
    assert!(repo.git(&["stash", "list"]).is_empty());
    let state = std::fs::read_to_string(repo.root().join(".git/gitpoulpi/repo-state.toml"))
        .unwrap_or_default();
    assert!(!state.contains("pending_auto_stash"), "{state}");

    // Nothing pending any more: refused, exit code 1.
    let again = run(repo.root(), session, &["autostash", "start-over", "--yes"]);
    assert_eq!(again.status.code(), Some(1));
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
