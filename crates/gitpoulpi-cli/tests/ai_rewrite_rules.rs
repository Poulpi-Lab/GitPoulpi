#![cfg(all(feature = "ai", feature = "history-rewrite"))]
//! `gitpoulpi ai rewrite-rules`: the recipe a scripted local agent proposes,
//! printed as JSON and applied nowhere; the repository context goes along
//! only with `--context`.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_testkit::fake_agent::{agent_calls, ai_config_toml, write_fake_agent};
#[cfg(unix)]
use gitpoulpi_testkit::fake_agent::{agent_prompt, write_capturing_fake_agent};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::Value;

const GOOD: &[&str] = &[
    "```json",
    "{\"scope\": {\"scope\": \"all_local\"},",
    " \"rules\": [",
    "  {\"label\": \"Old email\", \"kind\": {\"type\": \"identity\", \"when\": {\"email\": \"old@example.org\"}, \"set\": {\"email\": \"new@example.org\"}}},",
    "  {\"kind\": {\"type\": \"date_shift\", \"by_seconds\": 0}}",
    " ],",
    " \"notes\": [\"No date limit exists.\"]}",
    "```",
];

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "feat: first thing")
        .unwrap()
        .commit_file("b.txt", "two\n", "fix: second thing")
        .unwrap()
        .build()
}

fn run(config: &Path, repo: &FixtureRepo, session: &str, args: &[&str]) -> (i32, Value, String) {
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config)
        .arg("--json")
        .args(["--session", session])
        .args(["--repo", &repo.root().to_string_lossy()])
        .args(["ai", "rewrite-rules"])
        .args(args)
        .current_dir(repo.root())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let value =
        serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("not JSON ({e}): {stdout}"));
    (
        output.status.code().unwrap_or(-1),
        value,
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

fn refs(repo: &FixtureRepo) -> String {
    repo.git(&["for-each-ref", "--format=%(refname) %(objectname)"])
}

#[test]
fn prints_the_recipe_and_what_it_left_out_and_changes_nothing() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), GOOD);
    std::fs::write(config.path().join("config.toml"), ai_config_toml(&agent)).unwrap();
    let before = refs(&repo);

    let (code, value, stderr) = run(
        config.path(),
        &repo,
        "cli-rw-rules-ai",
        &["replace the old email"],
    );
    // One rule was refused (a shift of zero): exit 2, the other is kept.
    assert_eq!(code, 2, "{value} {stderr}");
    assert_eq!(value["applied"], false);
    assert_eq!(value["recipe"]["scope"]["scope"], "all_local");
    let rules = value["recipe"]["rules"].as_array().unwrap();
    assert_eq!(rules.len(), 1, "{value}");
    assert_eq!(rules[0]["kind"]["type"], "identity");
    assert_eq!(rules[0]["label"], "Old email");
    let problems = value["problems"].as_array().unwrap();
    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0]["position"], 2);
    assert_eq!(value["notes"][0], "No date limit exists.");
    assert_eq!(value["context"], false);
    assert_eq!(agent_calls(&agent), 1);
    assert_eq!(refs(&repo), before, "nothing was rewritten");
}

#[cfg(unix)]
#[test]
fn the_context_is_sent_only_with_context() {
    let repo = fixture();
    let author = repo.git(&["log", "-1", "--format=%an"]).trim().to_string();
    assert!(!author.is_empty());
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let agent = write_capturing_fake_agent(tools.path(), GOOD);
    std::fs::write(config.path().join("config.toml"), ai_config_toml(&agent)).unwrap();

    run(config.path(), &repo, "cli-rw-rules-off", &["do it"]);
    let prompt = agent_prompt(&agent);
    assert!(prompt.contains("do it"), "{prompt}");
    assert!(
        !prompt.contains(&author),
        "author sent without --context\n{prompt}"
    );

    let (_, value, _) = run(
        config.path(),
        &repo,
        "cli-rw-rules-on",
        &["do it", "--context"],
    );
    assert_eq!(value["context"], true);
    let prompt = agent_prompt(&agent);
    assert!(prompt.contains(&author), "{prompt}");
    assert!(prompt.contains("current branch"), "{prompt}");
}

#[test]
fn with_ai_off_it_refuses_and_asks_nothing() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config.path())
        .arg("--json")
        .args(["--session", "cli-rw-rules-offcfg"])
        .args(["--repo", &repo.root().to_string_lossy()])
        .args(["ai", "rewrite-rules", "x"])
        .current_dir(repo.root())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        value["error"].as_str().unwrap().contains("turned off"),
        "{value}"
    );
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
