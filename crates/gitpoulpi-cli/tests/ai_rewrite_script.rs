#![cfg(all(feature = "ai", feature = "history-rewrite"))]
//! `gitpoulpi ai rewrite-script`: the script a scripted local agent writes,
//! checked by the daemon, printed as JSON and applied nowhere; the sample of
//! commits goes along only with `--sample`.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_testkit::fake_agent::{agent_calls, ai_config_toml, write_fake_agent};
#[cfg(unix)]
use gitpoulpi_testkit::fake_agent::{agent_prompt, write_capturing_fake_agent};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::Value;

const GOOD: &[&str] = &[
    "```lua",
    "--! name: Old email",
    "--! kinds: commit",
    "function rewrite_commit(c, ctx)",
    "  if c.author.email == \"old@example.org\" then",
    "    return { author = { email = \"new@example.org\" } }",
    "  end",
    "end",
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
        .args(["ai", "rewrite-script"])
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

fn head(repo: &FixtureRepo) -> String {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo.root())
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn prints_the_script_and_its_check_and_changes_nothing() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), GOOD);
    std::fs::write(config.path().join("config.toml"), ai_config_toml(&agent)).unwrap();
    let before = head(&repo);

    let (code, value, stderr) = run(
        config.path(),
        &repo,
        "cli-rw-script-ai",
        &["replace the old email by the new one"],
    );
    assert_eq!(code, 0, "{value} {stderr}");
    assert_eq!(value["applied"], false);
    assert_eq!(value["retried"], false);
    assert_eq!(value["sample"], 0);
    let script = value["script"].as_str().unwrap();
    assert!(
        script.contains("function rewrite_commit(c, ctx)"),
        "{script}"
    );
    assert!(script.starts_with("--! name: Old email"), "{script}");
    // The daemon's check of that very script.
    assert_eq!(value["check"]["problems"], serde_json::json!([]), "{value}");
    assert_eq!(value["check"]["functions"][0], "rewrite_commit");
    assert_eq!(value["check"]["header"]["name"], "Old email");
    assert_eq!(agent_calls(&agent), 1);
    assert_eq!(head(&repo), before, "nothing was rewritten");
}

#[test]
fn a_script_that_does_not_compile_is_retried_once_then_reported() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    // The agent keeps answering the same broken script.
    let agent = write_fake_agent(
        tools.path(),
        &[
            "--! name: Broken",
            "--! kinds: commit",
            "function rewrite_commit(c, ctx)",
            "  return { author = { email = \"x\" } ",
            "end",
        ],
    );
    std::fs::write(config.path().join("config.toml"), ai_config_toml(&agent)).unwrap();

    let (code, value, _) = run(config.path(), &repo, "cli-rw-script-bad", &["fix emails"]);
    assert_eq!(code, 2, "a script with problems is exit 2: {value}");
    assert_eq!(value["retried"], true);
    assert_eq!(agent_calls(&agent), 2, "one retry, no more");
    let problems = value["check"]["problems"].as_array().unwrap();
    assert!(!problems.is_empty(), "{value}");
    assert!(value["script"].as_str().unwrap().contains("Broken"));
}

#[cfg(unix)]
#[test]
fn the_sample_of_commits_is_sent_only_with_sample_n() {
    let repo = fixture();
    let author = String::from_utf8(
        std::process::Command::new("git")
            .args(["log", "-1", "--format=%an"])
            .current_dir(repo.root())
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .to_string();
    assert!(!author.is_empty());
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let agent = write_capturing_fake_agent(tools.path(), GOOD);
    std::fs::write(config.path().join("config.toml"), ai_config_toml(&agent)).unwrap();

    let (code, value, stderr) = run(config.path(), &repo, "cli-rw-script-off", &["do it"]);
    assert_eq!(code, 0, "{value} {stderr}");
    let prompt = agent_prompt(&agent);
    assert!(prompt.contains("do it"), "{prompt}");
    for needle in [
        author.as_str(),
        "feat: first thing",
        "fix: second thing",
        "sample",
    ] {
        assert!(
            !prompt.contains(needle),
            "{needle:?} was sent without --sample\n{prompt}"
        );
    }

    let (code, value, stderr) = run(
        config.path(),
        &repo,
        "cli-rw-script-on",
        &["do it", "--sample", "1"],
    );
    assert_eq!(code, 0, "{value} {stderr}");
    assert_eq!(value["sample"], 1);
    let prompt = agent_prompt(&agent);
    assert!(prompt.contains(&author), "{prompt}");
    assert!(
        prompt.contains("fix: second thing"),
        "the newest commit\n{prompt}"
    );
    assert!(
        !prompt.contains("feat: first thing"),
        "only N commits\n{prompt}"
    );
}

#[test]
fn with_ai_off_it_refuses_and_asks_nothing() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config.path())
        .arg("--json")
        .args(["--session", "cli-rw-script-offcfg"])
        .args(["--repo", &repo.root().to_string_lossy()])
        .args(["ai", "rewrite-script", "x"])
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
