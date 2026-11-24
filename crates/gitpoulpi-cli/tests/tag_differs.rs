//! `gitpoulpi tag take-remote` and `tag force-push` through the real binary,
//! asserted on deserialized JSON and exit codes. The remote is a local bare
//! repository (never a server).

use std::path::Path;

use assert_cmd::Command;
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

/// `v1` and `v2` are on `origin` at the first commit and known to the mirror;
/// here both are re-made at a second commit.
fn differing(session: &str) -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .build();
    ok(&repo, session, &["tag", "create", "v1"]);
    ok(&repo, session, &["tag", "create", "v2"]);
    ok(&repo, session, &["tag", "push", "v1", "v2"]);
    repo.git(&["commit", "--allow-empty", "-m", "second"]);
    repo.git(&["tag", "-f", "v1"]);
    repo.git(&["tag", "-f", "v2"]);
    repo
}

fn where_of(repo: &FixtureRepo, session: &str, name: &str) -> Value {
    let rows = ok(repo, session, &["tag", "list"]);
    rows.as_array()
        .unwrap()
        .iter()
        .find(|r| r["name"] == name)
        .cloned()
        .expect("the tag is listed")
}

#[test]
fn take_remote_moves_the_local_tag_to_the_remotes_commit() {
    let session = "cli-test-tag-take-remote";
    let repo = differing(session);
    let row = where_of(&repo, session, "v1");
    assert_eq!(row["where"], "differs");
    assert_eq!(row["differs_on"], serde_json::json!(["origin"]));

    let out = ok(&repo, session, &["tag", "take-remote", "v1"]);
    assert_eq!(out["took"], "v1");
    assert_eq!(out["remote"], "origin");
    assert_eq!(
        repo.git(&["rev-parse", "refs/tags/v1"]),
        repo.git_in_remote(&["rev-parse", "refs/tags/v1"])
    );
    assert_eq!(where_of(&repo, session, "v1")["where"], "both");

    // Not differing: nothing to take, and it says so.
    let (code, json) = run(&repo, session, &["tag", "take-remote", "v1"]);
    assert_ne!(code, 0, "{json}");
}

#[test]
fn force_push_needs_yes_then_moves_the_remote_tag() {
    let session = "cli-test-tag-force-push";
    let repo = differing(session);
    let theirs = repo.git_in_remote(&["rev-parse", "refs/tags/v1"]);

    let (code, json) = run(&repo, session, &["tag", "force-push", "v1"]);
    assert_eq!(code, 3, "{json}");
    assert!(
        json["needs_confirmation"]["prompt"]
            .as_str()
            .is_some_and(|p| p.contains("v1") && p.contains("origin")),
        "{json}"
    );
    assert_eq!(
        repo.git_in_remote(&["rev-parse", "refs/tags/v1"]),
        theirs,
        "nothing changed without --yes"
    );

    let out = ok(&repo, session, &["tag", "force-push", "v1", "--yes"]);
    assert_eq!(out["force_pushed"], "v1");
    assert_eq!(out["remote"], "origin");
    assert_eq!(
        repo.git_in_remote(&["rev-parse", "refs/tags/v1"]),
        repo.git(&["rev-parse", "refs/tags/v1"])
    );
    assert_eq!(where_of(&repo, session, "v1")["where"], "both");
}

#[test]
fn force_push_with_a_stale_lease_is_refused_and_leaves_the_remote_alone() {
    let session = "cli-test-tag-force-push-stale";
    let repo = differing(session);
    repo.push_from_elsewhere("main", "theirs");
    repo.git_in_remote(&["tag", "-f", "v2", "main"]);
    let moved = repo.git_in_remote(&["rev-parse", "refs/tags/v2"]);

    let (code, json) = run(&repo, session, &["tag", "force-push", "v2", "--yes"]);
    assert_ne!(code, 0, "{json}");
    assert_ne!(code, 3, "a refusal is not a confirmation request: {json}");
    assert!(json.to_string().contains("fetch first"), "{json}");
    assert_eq!(
        repo.git_in_remote(&["rev-parse", "refs/tags/v2"]),
        moved,
        "never overwritten"
    );

    ok(&repo, session, &["fetch"]);
    ok(&repo, session, &["tag", "force-push", "v2", "--yes"]);
    assert_eq!(
        repo.git_in_remote(&["rev-parse", "refs/tags/v2"]),
        repo.git(&["rev-parse", "refs/tags/v2"])
    );
}
