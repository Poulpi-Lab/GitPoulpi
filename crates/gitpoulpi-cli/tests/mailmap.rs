//! `gitpoulpi mailmap list|check|add`, end to end through the real binary,
//! the daemon and the backend. Assertions are on deserialized JSON.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::{json, Value};

fn config_dir() -> &'static Path {
    static DIR: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    DIR.get_or_init(|| tempfile::tempdir().unwrap()).path()
}

fn run(repo: &FixtureRepo, args: &[&str]) -> (i32, Value) {
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config_dir())
        .args(["--json", "--session", "mailmap-cli-test"])
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

fn ok(repo: &FixtureRepo, args: &[&str]) -> Value {
    let (code, value) = run(repo, args);
    assert_eq!(code, 0, "{args:?}: {value}");
    value
}

fn commit_as(repo: &FixtureRepo, file: &str, who: &str) {
    std::fs::write(repo.root().join(file), format!("{who}\n")).unwrap();
    repo.git(&["add", file]);
    repo.git(&["commit", "-q", "-m", file, &format!("--author={who}")]);
}

fn authors(log: &Value) -> Vec<String> {
    log["commits"]
        .as_array()
        .or_else(|| log.as_array())
        .or_else(|| log["nodes"].as_array())
        .unwrap_or_else(|| panic!("no commits in {log}"))
        .iter()
        .map(|c| {
            format!(
                "{} <{}>",
                c["author_name"].as_str().unwrap(),
                c["author_email"].as_str().unwrap()
            )
        })
        .collect()
}

#[test]
fn list_check_and_add_work_on_json_and_log_follows_the_mailmap() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .build();
    commit_as(&repo, "a.txt", "alex <alex@old.example.com>");
    commit_as(&repo, "b.txt", "Alex Doe <alex@example.org>");

    // Nothing mapped yet.
    let list = ok(&repo, &["mailmap", "list"]);
    assert_eq!(list["has_mailmap"], json!(false));
    assert_eq!(list["commits"], json!(3));
    let rows = list["identities"].as_array().unwrap();
    let old = rows
        .iter()
        .find(|r| r["email"] == "alex@old.example.com")
        .unwrap();
    assert_eq!(old["name"], "alex");
    assert_eq!(old["as_author"], json!(1));
    assert!(old.get("mapped_to").is_none());
    assert!(!authors(&ok(&repo, &["log"])).is_empty());

    // Add: the file is written and nothing is committed.
    let head = repo.git(&["rev-parse", "HEAD"]);
    let added = ok(
        &repo,
        &[
            "mailmap",
            "add",
            "--canonical",
            "Alex Doe <alex@example.org>",
            "--from",
            "alex <alex@old.example.com>",
        ],
    );
    assert_eq!(added["mailmap"]["added"], json!(1));
    assert_eq!(added["mailmap"]["file"], json!(".mailmap"));
    assert_eq!(
        std::fs::read_to_string(repo.root().join(".mailmap")).unwrap(),
        "Alex Doe <alex@example.org> alex <alex@old.example.com>\n"
    );
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), head);

    // The list, check and the log all see it.
    let mapped = ok(&repo, &["mailmap", "list", "--mapped"]);
    let rows = mapped["identities"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0]["mapped_to"],
        json!({ "name": "Alex Doe", "email": "alex@example.org" })
    );
    let unmapped = ok(&repo, &["mailmap", "list", "--unmapped"]);
    assert!(unmapped["identities"]
        .as_array()
        .unwrap()
        .iter()
        .all(|r| r.get("mapped_to").is_none()));
    let check = ok(
        &repo,
        &[
            "mailmap",
            "check",
            "alex <alex@old.example.com>",
            "Zed <zed@example.org>",
        ],
    );
    let rows = check["identities"].as_array().unwrap();
    assert_eq!(rows[0]["mapped"], json!(true));
    assert_eq!(
        rows[0]["shown_as"],
        json!({ "name": "Alex Doe", "email": "alex@example.org" })
    );
    assert_eq!(rows[1]["mapped"], json!(false));
    let shown = authors(&ok(&repo, &["log"]));
    assert!(
        shown.iter().all(|a| !a.contains("alex@old.example.com")),
        "{shown:?}"
    );

    // Adding again writes nothing; undo puts the file away.
    let again = ok(
        &repo,
        &[
            "mailmap",
            "add",
            "--canonical",
            "Alex Doe <alex@example.org>",
            "--from",
            "alex <alex@old.example.com>",
        ],
    );
    assert_eq!(again["mailmap"]["added"], json!(0));
    ok(&repo, &["undo"]);
    assert!(!repo.root().join(".mailmap").exists());

    // A malformed identity is an error, not a panic.
    let (code, _) = run(&repo, &["mailmap", "check", "no brackets"]);
    assert_ne!(code, 0);
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
