#![cfg(all(feature = "ai", feature = "worktree"))]
//! `gitpoulpi ai worktree-name --describe ...`: the names a scripted local
//! agent proposes, validated and printed as JSON; nothing is created.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_testkit::fake_agent::{agent_calls, ai_config_toml, write_fake_agent};
#[cfg(unix)]
use gitpoulpi_testkit::fake_agent::{agent_prompt, write_capturing_fake_agent};
use gitpoulpi_testkit::FixtureRepoBuilder;
use serde_json::Value;

fn run(config: &Path, root: &Path, session: &str, args: &[&str]) -> (i32, Value) {
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config)
        .arg("--json")
        .args(["--session", session])
        .args(["--repo", &root.to_string_lossy()])
        .args(["ai", "worktree-name"])
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let value =
        serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("not JSON ({e}): {stdout}"));
    (output.status.code().unwrap_or(-1), value)
}

fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "feat: first thing")
        .unwrap()
        .build()
}

#[test]
fn prints_the_worktree_and_branch_names_the_agent_proposes() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    #[cfg(unix)]
    let agent = write_capturing_fake_agent(
        tools.path(),
        &["{\"name\": \"Fix Empty Config\", \"branch\": \"fix/empty-config\"}"],
    );
    #[cfg(not(unix))]
    let agent = write_fake_agent(
        tools.path(),
        &["{\"name\": \"Fix Empty Config\", \"branch\": \"fix/empty-config\"}"],
    );
    std::fs::write(config.path().join("config.toml"), ai_config_toml(&agent)).unwrap();

    let (code, value) = run(
        config.path(),
        repo.root(),
        "cli-ai-wt-name",
        &["--describe", "fix the crash when the config is empty"],
    );
    assert_eq!(code, 0, "{value}");
    assert_eq!(value["name"], "fix-empty-config");
    assert_eq!(value["worktree"], "worktree-fix-empty-config");
    assert_eq!(value["branch"], "fix/empty-config");
    assert_eq!(agent_calls(&agent), 1);
    #[cfg(unix)]
    {
        let prompt = agent_prompt(&agent);
        assert!(prompt.contains("fix the crash when the config is empty"));
        // The repository's own branch names go along.
        assert!(prompt.contains("- "), "{prompt}");
    }
}

#[test]
fn an_answer_with_an_invalid_branch_is_an_error_not_a_name() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(
        tools.path(),
        &["{\"name\": \"x\", \"branch\": \"not a branch\"}"],
    );
    std::fs::write(config.path().join("config.toml"), ai_config_toml(&agent)).unwrap();

    let (code, value) = run(
        config.path(),
        repo.root(),
        "cli-ai-wt-name-bad",
        &["--describe", "anything"],
    );
    assert_ne!(code, 0, "{value}");
    assert!(value.get("branch").is_none(), "{value}");
}
