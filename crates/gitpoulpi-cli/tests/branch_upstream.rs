//! `gitpoulpi branch upstream list|add|remove|pull|primary` and
//! `remote-group status`, through the real binary, the daemon and the
//! backend, on fixtures with three bare remotes. Assertions are on
//! deserialized JSON (the domain types), never on text.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_core::domain::{RemoteGroupStatus, RemoteRelation, TargetRef, TargetRole};
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
        .args(["--json", "--session", session])
        .args(["--repo", &repo.root().to_string_lossy()])
        .args(args)
        .current_dir(repo.root())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "not JSON ({e}): {stdout}\n{}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output.status.code().unwrap_or(-1), value)
}

fn status_of(repo: &FixtureRepo, session: &str, args: &[&str]) -> RemoteGroupStatus {
    let (code, value) = run(repo, session, args);
    assert_eq!(code, 0, "{args:?}: {value}");
    serde_json::from_value(value).unwrap_or_else(|e| panic!("{args:?}: not a status ({e})"))
}

fn fixture() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .with_origin()
        .unwrap()
        .with_remote("gitlab")
        .unwrap()
        .with_remote("backup")
        .unwrap()
        .push_to("origin", "main")
        .unwrap()
        .push_to("gitlab", "main")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .build();
    repo.git(&["fetch", "-q", "--all"]);
    repo
}

fn target(remote: &str, branch: &str) -> TargetRef {
    TargetRef {
        remote: remote.into(),
        branch: branch.into(),
    }
}

fn values(repo: &FixtureRepo, variable: &str) -> Vec<String> {
    let key = format!("branch.main.{variable}");
    repo.git(&["config", "--get-all", &key])
        .lines()
        .map(str::to_string)
        .collect()
}

#[test]
fn list_and_remote_group_status_report_the_primary_and_nothing_else_at_first() {
    let repo = fixture();
    let s = "bu-list";
    let listed = status_of(&repo, s, &["branch", "upstream", "list"]);
    assert_eq!(listed.remotes.primary, Some(target("origin", "main")));
    assert!(listed.remotes.push_to.is_empty() && listed.remotes.pull_from.is_empty());
    assert_eq!(listed.targets.len(), 1);
    assert_eq!(listed.targets[0].role, TargetRole::Primary);
    assert_eq!(listed.targets[0].relation, RemoteRelation::UpToDate);
    let group = status_of(&repo, s, &["remote-group", "status"]);
    assert_eq!(group, listed);
    let named = status_of(&repo, s, &["remote-group", "status", "--branch", "main"]);
    assert_eq!(named, listed);
}

#[test]
fn add_pull_and_remove_edit_the_targets_and_the_config() {
    let repo = fixture();
    let s = "bu-edit";
    let added = status_of(&repo, s, &["branch", "upstream", "add", "gitlab", "--pull"]);
    assert_eq!(added.remotes.push_to, vec![target("gitlab", "main")]);
    assert_eq!(added.remotes.pull_from, vec![target("gitlab", "main")]);
    let gitlab = added.targets.iter().find(|t| t.remote == "gitlab").unwrap();
    assert!(gitlab.pull);
    assert_eq!(gitlab.relation, RemoteRelation::UpToDate);
    assert_eq!(values(&repo, "gitpoulpi-push-to"), ["gitlab"]);

    // Another branch name on the remote; never pushed there yet.
    let added = status_of(&repo, s, &["branch", "upstream", "add", "backup:feat-x"]);
    assert_eq!(
        added.remotes.push_to,
        vec![target("gitlab", "main"), target("backup", "feat-x")]
    );
    let backup = added.targets.iter().find(|t| t.remote == "backup").unwrap();
    assert_eq!(backup.relation, RemoteRelation::Absent);
    assert_eq!(
        values(&repo, "gitpoulpi-push-to"),
        ["gitlab", "backup:feat-x"]
    );

    // Pull sources on and off, independent of the push targets.
    let on = status_of(
        &repo,
        s,
        &["branch", "upstream", "pull", "backup:feat-x", "--on"],
    );
    assert_eq!(on.remotes.pull_from.len(), 2);
    let off = status_of(
        &repo,
        s,
        &["branch", "upstream", "pull", "backup:feat-x", "--off"],
    );
    assert_eq!(off.remotes.pull_from, vec![target("gitlab", "main")]);
    assert_eq!(off.remotes.push_to.len(), 2);

    let removed = status_of(&repo, s, &["branch", "upstream", "remove", "gitlab"]);
    assert_eq!(removed.remotes.push_to, vec![target("backup", "feat-x")]);
    assert!(removed.remotes.pull_from.is_empty());
    assert_eq!(removed.remotes.primary, Some(target("origin", "main")));
}

#[test]
fn undoing_an_add_restores_the_config() {
    let repo = fixture();
    let s = "bu-undo";
    let before = repo.git(&["config", "--local", "--list"]);
    status_of(&repo, s, &["branch", "upstream", "add", "gitlab"]);
    assert_ne!(repo.git(&["config", "--local", "--list"]), before);
    let (code, value) = run(&repo, s, &["undo"]);
    assert_eq!(code, 0, "{value}");
    assert_eq!(repo.git(&["config", "--local", "--list"]), before);
}

#[test]
fn refusals_exit_2_with_a_reason() {
    let repo = fixture();
    let s = "bu-refuse";
    for args in [
        vec!["branch", "upstream", "add", "origin"],
        vec!["branch", "upstream", "remove", "origin"],
        vec!["branch", "upstream", "remove", "backup"],
        vec!["branch", "upstream", "pull", "backup", "--off"],
        vec!["branch", "upstream", "pull", "origin", "--on"],
    ] {
        let (code, value) = run(&repo, s, &args);
        assert_eq!(code, 2, "{args:?}: {value}");
        assert!(value["refused"]["reason"].is_string(), "{args:?}: {value}");
    }
    // A remote that does not exist is a failure from the daemon, not ok.
    let (code, _) = run(&repo, s, &["branch", "upstream", "add", "nowhere"]);
    assert_ne!(code, 0);
    assert!(values_absent(&repo));
}

fn values_absent(repo: &FixtureRepo) -> bool {
    let out = std::process::Command::new("git")
        .args(["config", "--get-all", "branch.main.gitpoulpi-push-to"])
        .current_dir(repo.root())
        .output()
        .unwrap();
    out.stdout.is_empty()
}

#[test]
fn primary_is_set_upstream_and_unset_upstream_still_works() {
    let repo = fixture();
    let s = "bu-primary";
    let (code, value) = run(&repo, s, &["branch", "upstream", "primary", "gitlab/main"]);
    assert_eq!(code, 0, "{value}");
    assert_eq!(repo.git(&["config", "branch.main.remote"]).trim(), "gitlab");
    let (code, value) = run(&repo, s, &["branch", "set-upstream", "main", "origin/main"]);
    assert_eq!(code, 0, "{value}");
    assert_eq!(repo.git(&["config", "branch.main.remote"]).trim(), "origin");
    let (code, value) = run(&repo, s, &["branch", "unset-upstream", "main"]);
    assert_eq!(code, 0, "{value}");
    let listed = status_of(&repo, s, &["branch", "upstream", "list"]);
    assert_eq!(listed.remotes.primary, None);
}
