#![cfg(all(feature = "ai", unix))]
//! What the AI is given and what staging asks, through the real binary, the
//! daemon and a capturing fake agent: the prompt never holds a token or a
//! `.env`, the answer reports what was taken out, and `--print-sent` sends
//! nothing.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_testkit::fake_agent::{
    agent_calls, agent_prompt, ai_config_toml, write_capturing_fake_agent,
};
use gitpoulpi_testkit::FixtureRepoBuilder;
use serde_json::Value;

const BODY: &str = "q7Xk2LmZp9RtV4bN8wYcJ3hGfD5sAe6U1oIx";

fn run(config: &Path, root: &Path, session: &str, args: &[&str]) -> (Option<i32>, Value, String) {
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config)
        .arg("--json")
        .args(["--session", session])
        .args(["--repo", &root.to_string_lossy()])
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let json = serde_json::from_str(&stdout).unwrap_or(Value::Null);
    (
        output.status.code(),
        json,
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

fn git(root: &Path, args: &[&str]) {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(out.status.success(), "git {args:?}");
}

/// A repository with a staged source file holding a token and a staged `.env`.
fn setup(
    name: &str,
) -> (
    gitpoulpi_testkit::FixtureRepo,
    tempfile::TempDir,
    tempfile::TempDir,
    std::path::PathBuf,
    String,
) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let agent = write_capturing_fake_agent(tools.path(), &["feat: add things"]);
    std::fs::write(config.path().join("config.toml"), ai_config_toml(&agent)).unwrap();
    std::fs::write(
        repo.root().join("client.txt"),
        format!("token = ghp_{BODY}\nhello\n"),
    )
    .unwrap();
    std::fs::write(repo.root().join(".env"), "DB_PASSWORD=hunter2hunter2\n").unwrap();
    // `git add` is not guarded: only GitPoulpi's own stage is.
    git(repo.root(), &["add", "client.txt", ".env"]);
    (repo, tools, config, agent, format!("cli-privacy-{name}"))
}

#[test]
fn the_prompt_has_no_token_and_no_env_and_the_answer_says_what_was_removed() {
    let (repo, _tools, config, agent, session) = setup("prompt");
    let (code, json, stderr) = run(
        config.path(),
        repo.root(),
        &session,
        &["ai", "commit-message"],
    );
    assert_eq!(code, Some(0), "{json} {stderr}");
    assert_eq!(json["summary"], "feat: add things");
    assert!(
        json["privacy"]["redactions"].as_u64().unwrap() >= 1,
        "{json}"
    );
    let excluded = json["privacy"]["excluded_files"].as_array().unwrap();
    assert_eq!(excluded, &vec![Value::String(".env".into())], "{json}");

    let prompt = agent_prompt(&agent);
    assert!(prompt.contains("client.txt"), "{prompt}");
    assert!(prompt.contains("[REDACTED:github-token]"), "{prompt}");
    assert!(!prompt.contains(BODY), "{prompt}");
    assert!(!prompt.contains("hunter2"), "{prompt}");
    assert!(!prompt.contains("DB_PASSWORD"), "{prompt}");
    // The JSON the CLI printed holds no value either.
    assert!(!json.to_string().contains(BODY));
}

#[test]
fn print_sent_prints_the_prepared_text_and_calls_nobody() {
    let (repo, _tools, config, agent, session) = setup("print");
    let (code, json, stderr) = run(
        config.path(),
        repo.root(),
        &session,
        &["ai", "commit-message", "--print-sent"],
    );
    assert_eq!(code, Some(0), "{json} {stderr}");
    assert_eq!(agent_calls(&agent), 0, "nothing was sent");
    assert_eq!(json["dry_run"], true);
    let user = json["sent"][0]["user"].as_str().unwrap();
    assert!(user.contains("[REDACTED:github-token]"), "{user}");
    assert!(!user.contains(BODY) && !user.contains("hunter2"), "{user}");
    assert_eq!(json["privacy"]["excluded_files"][0], ".env");
}
