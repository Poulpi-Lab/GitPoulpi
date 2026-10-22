//! A binary built without the `ai` feature (custom builds): no `ai`
//! subcommand, the `--ai` flags answer "not built", `config set ai.*` is
//! stored (the human output adds a note, covered by a unit test). Asserts on exit status and JSON, not prose.
#![cfg(not(feature = "ai"))]

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_testkit::FixtureRepoBuilder;
use serde_json::Value;

fn run(config: &Path, root: &Path, args: &[&str]) -> (bool, String, String) {
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config)
        .arg("--json")
        .args(["--session", "cli-ai-not-built"])
        .args(["--repo", &root.to_string_lossy()])
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn there_is_no_ai_subcommand_and_the_flags_say_not_built() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();

    let (success, _, _) = run(config.path(), repo.root(), &["ai", "commit-message"]);
    assert!(!success, "the ai subcommand must not exist");

    let (success, stdout, stderr) = run(config.path(), repo.root(), &["commit", "--ai"]);
    assert!(!success);
    assert!(
        format!("{stdout}{stderr}").contains("not built"),
        "{stdout} {stderr}"
    );

    let (success, stdout, _) = run(
        config.path(),
        repo.root(),
        &["config", "set", "ai.enabled", "true"],
    );
    assert!(success, "{stdout}");
    let value: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(value["key"], "ai.enabled");
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
