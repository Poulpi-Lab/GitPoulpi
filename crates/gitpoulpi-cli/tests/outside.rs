//! Changes made outside GitPoulpi in `gitpoulpi undo --list`, end to end
//! through the real binary (the outside-operations design).
//! Each command is its own short-lived embedded daemon, so these also prove
//! the refs snapshot persists between processes and finds what happened
//! while none ran.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_core::domain::{JournalListing, OperationKind, UndoOutcome};
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
        .env("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "true")
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
fn undo_list_shows_what_a_shell_did_as_an_outside_entry_and_undo_takes_it_back() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let session = "outside-list";
    repo.git(&["branch", "agent-branch"]);
    // The first command is the baseline of this repository.
    ok(&repo, session, &["undo", "--list"]);
    let tip = repo.git(&["rev-parse", "agent-branch"]).trim().to_string();

    repo.git(&["branch", "-D", "agent-branch"]);
    let listed = ok(&repo, session, &["undo", "--list"]);

    // The raw JSON says who made it.
    let entry = &listed["undo"][0];
    assert_eq!(entry["source"], "outside", "{listed}");
    assert_eq!(entry["summary"]["state"], "Done", "{listed}");
    assert!(
        entry["summary"]["label"]
            .as_str()
            .unwrap()
            .contains("deleted branch agent-branch"),
        "{listed}"
    );
    assert!(entry["outside"]["message"].is_string(), "{listed}");
    // Restorable: the deleted branch comes back.
    assert_eq!(entry["restore"]["items"][0]["kind"], "Branch", "{listed}");
    let typed: JournalListing = serde_json::from_value(listed).unwrap();
    assert_eq!(typed.undo[0].summary.kind, OperationKind::Outside);

    // Our own operation is "gitpoulpi"; it is not recorded twice.
    ok(&repo, session, &["undo"]);
    assert_eq!(repo.git(&["rev-parse", "agent-branch"]).trim(), tip);
    let after = ok(&repo, session, &["undo", "--list"]);
    assert!(after["undo"].as_array().unwrap().is_empty(), "{after}");
    assert_eq!(after["redo"][0]["source"], "outside", "{after}");

    let redone: UndoOutcome = serde_json::from_value(ok(&repo, session, &["redo"])).unwrap();
    assert!(redone.applied);
    assert!(repo.try_git(&["rev-parse", "agent-branch"]).is_none());
}

#[test]
fn a_gitpoulpi_entry_is_marked_gitpoulpi_and_an_outside_one_after_it_is_marked_outside() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let session = "outside-mixed";
    let head = repo.git(&["rev-parse", "HEAD"]).trim().to_string();
    ok(&repo, session, &["branch", "create", "mine", &head]);
    repo.git(&["commit", "--allow-empty", "-m", "agent commit"]);
    let listed = ok(&repo, session, &["undo", "--list"]);
    let entries = listed["undo"].as_array().unwrap();
    assert_eq!(entries.len(), 2, "{listed}");
    assert_eq!(entries[0]["source"], "gitpoulpi", "{listed}");
    assert_eq!(entries[1]["source"], "outside", "{listed}");
    assert!(entries[1]["outside"]["committer"].is_string(), "{listed}");
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
