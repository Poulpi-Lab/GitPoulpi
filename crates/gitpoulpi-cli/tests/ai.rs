#![cfg(feature = "ai")]
//! `gitpoulpi commit --ai`, `gitpoulpi ai commit-message` and `ai explain`,
//! end to end through the real binary, the daemon and a **fake local
//! agent** (a script printing a canned answer): no socket is ever opened.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_testkit::fake_agent::{agent_calls, ai_config_toml, write_fake_agent};
#[cfg(unix)]
use gitpoulpi_testkit::fake_agent::{agent_prompt, write_capturing_fake_agent};
use gitpoulpi_testkit::FixtureRepoBuilder;
use serde_json::Value;

fn run(config: &Path, root: &Path, session: &str, args: &[&str]) -> (bool, String, String) {
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
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

fn ok_json(config: &Path, root: &Path, session: &str, args: &[&str]) -> Value {
    let (success, stdout, stderr) = run(config, root, session, args);
    assert!(success, "{args:?} failed: {stdout} {stderr}");
    serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("not JSON ({e}): {stdout}"))
}

fn git(root: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(out.status.success(), "git {args:?}");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn commit_ai_commits_with_the_agents_message() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(
        tools.path(),
        &[
            "feat: describe the new file",
            "",
            "Adds new.txt with a greeting.",
        ],
    );
    std::fs::write(config.path().join("config.toml"), ai_config_toml(&agent)).unwrap();
    std::fs::write(repo.root().join("new.txt"), "hello\n").unwrap();
    git(repo.root(), &["add", "new.txt"]);

    ok_json(
        config.path(),
        repo.root(),
        "cli-ai-commit",
        &["commit", "--ai"],
    );

    let message = git(repo.root(), &["log", "-1", "--format=%B"]);
    assert!(
        message.starts_with("feat: describe the new file"),
        "{message}"
    );
    assert!(
        message.contains("Adds new.txt with a greeting."),
        "{message}"
    );
    assert_eq!(agent_calls(&agent), 1);
}

#[test]
fn ai_commit_message_prints_and_commits_nothing() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &["fix: only print me"]);
    std::fs::write(config.path().join("config.toml"), ai_config_toml(&agent)).unwrap();
    std::fs::write(repo.root().join("a.txt"), "x\n").unwrap();
    git(repo.root(), &["add", "a.txt"]);
    let before = git(repo.root(), &["rev-parse", "HEAD"]);

    let json = ok_json(
        config.path(),
        repo.root(),
        "cli-ai-message",
        &["ai", "commit-message"],
    );
    assert_eq!(json["summary"], "fix: only print me");
    assert_eq!(git(repo.root(), &["rev-parse", "HEAD"]), before);
}

#[test]
fn ai_explain_returns_the_explanation() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &["This commit sets things up."]);
    std::fs::write(config.path().join("config.toml"), ai_config_toml(&agent)).unwrap();

    let json = ok_json(
        config.path(),
        repo.root(),
        "cli-ai-explain",
        &["ai", "explain"],
    );
    assert_eq!(json["explanation"], "This commit sets things up.");
}

#[test]
fn with_ai_off_the_flag_refuses_and_no_agent_runs() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &["never used"]);
    // Profiles configured, but the general switch is off (the default).
    let toml = ai_config_toml(&agent).replace("enabled = true", "enabled = false");
    std::fs::write(config.path().join("config.toml"), toml).unwrap();
    std::fs::write(repo.root().join("a.txt"), "x\n").unwrap();
    git(repo.root(), &["add", "a.txt"]);

    let (success, stdout, stderr) = run(
        config.path(),
        repo.root(),
        "cli-ai-off",
        &["commit", "--ai"],
    );
    assert!(!success);
    assert!(
        format!("{stdout}{stderr}").contains("turned off"),
        "{stdout} {stderr}"
    );
    assert_eq!(agent_calls(&agent), 0);
}

#[test]
fn stash_push_ai_names_the_stash_with_the_agents_answer() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .build();
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &["WIP: second line"]);
    std::fs::write(config.path().join("config.toml"), ai_config_toml(&agent)).unwrap();

    ok_json(
        config.path(),
        repo.root(),
        "cli-ai-stash",
        &["stash", "push", "--ai"],
    );
    let list = git(repo.root(), &["stash", "list"]);
    assert!(list.contains("WIP: second line"), "{list}");
}

#[test]
fn commit_needs_a_message_or_ai() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let (success, _, _) = run(config.path(), repo.root(), "cli-ai-none", &["commit"]);
    assert!(!success);
}

/// A config and a repo-state file in the shape written before the AI
/// redesign (presets + defaults + a per-repo preset choice) still drive
/// the CLI: the global default preset runs on a broken profile, so the
/// command can only succeed through the repository's migrated override.
#[test]
fn an_old_preset_shaped_config_and_repo_choice_still_work() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &["fix: from the repo preset"]);
    let old = format!(
        "{}\n[[ai.profiles]]\nid = \"broken\"\nname = \"Broken\"\nkind = \"local_agent\"\ncommand = 'no-such-agent-binary'\nargs = []\n\n\
         [[ai.presets]]\nid = \"global\"\naction = \"commit_message\"\nprofile = \"broken\"\nsystem_prompt = \"Global.\"\n\n\
         [[ai.presets]]\nid = \"repo\"\naction = \"commit_message\"\nprofile = \"fake\"\nsystem_prompt = \"Repo.\"\n\n\
         [ai.defaults]\ncommit_message = \"global\"\n",
        ai_config_toml(&agent)
    );
    std::fs::write(config.path().join("config.toml"), old).unwrap();
    std::fs::create_dir_all(repo.state_dir()).unwrap();
    std::fs::write(
        repo.state_dir().join("repo-state.toml"),
        "[preferences.ai_presets]\ncommit_message = \"repo\"\n",
    )
    .unwrap();
    std::fs::write(repo.root().join("a.txt"), "x\n").unwrap();
    git(repo.root(), &["add", "a.txt"]);

    let json = ok_json(
        config.path(),
        repo.root(),
        "cli-ai-old-shape",
        &["ai", "commit-message"],
    );
    assert_eq!(json["summary"], "fix: from the repo preset");

    // Without the repo choice the migrated global action is used, and it
    // points at the broken profile.
    std::fs::remove_file(repo.state_dir().join("repo-state.toml")).unwrap();
    let (success, stdout, stderr) = run(
        config.path(),
        repo.root(),
        "cli-ai-old-shape",
        &["ai", "commit-message"],
    );
    assert!(!success, "{stdout} {stderr}");
    assert_eq!(agent_calls(&agent), 1);
}

#[test]
fn ai_resolve_conflict_prints_the_suggestion_and_writes_nothing() {
    use gitpoulpi_testkit::ConflictFixture;
    let fixture = ConflictFixture::text();
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    #[cfg(unix)]
    let agent = write_capturing_fake_agent(
        tools.path(),
        &["```", "    kept both", "second line", "```"],
    );
    #[cfg(not(unix))]
    let agent = write_fake_agent(tools.path(), &["```", "kept both", "second line", "```"]);
    std::fs::write(config.path().join("config.toml"), ai_config_toml(&agent)).unwrap();
    let before = std::fs::read(fixture.root().join("notes.txt")).unwrap();

    let json = ok_json(
        config.path(),
        fixture.root(),
        "cli-ai-resolve",
        &["ai", "resolve-conflict", "notes.txt"],
    );
    assert_eq!(json["path"], "notes.txt");
    assert_eq!(json["conflict"], 1);
    assert_eq!(json["applied"], false);
    let lines: Vec<&str> = json["resolution"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l.as_str().unwrap())
        .collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].ends_with("kept both") && lines[1] == "second line");
    assert_eq!(agent_calls(&agent), 1);
    #[cfg(unix)]
    {
        let prompt = agent_prompt(&agent);
        assert!(prompt.contains("`notes.txt`"), "{prompt}");
        assert!(prompt.contains("three (ours)") && prompt.contains("three (theirs)"));
    }
    assert_eq!(
        std::fs::read(fixture.root().join("notes.txt")).unwrap(),
        before,
        "nothing is written"
    );

    // A conflict the file does not have, and AI off.
    let (success, stdout, stderr) = run(
        config.path(),
        fixture.root(),
        "cli-ai-resolve",
        &["ai", "resolve-conflict", "notes.txt", "--conflict", "3"],
    );
    assert!(!success, "{stdout}");
    assert!(
        format!("{stdout}{stderr}").contains("there is no conflict 3"),
        "{stdout}{stderr}"
    );
    let off = tempfile::tempdir().unwrap();
    let (success, stdout, stderr) = run(
        off.path(),
        fixture.root(),
        "cli-ai-resolve-off",
        &["ai", "resolve-conflict", "notes.txt"],
    );
    assert!(!success, "{stdout}");
    assert!(
        format!("{stdout}{stderr}").contains("turned off"),
        "{stdout}{stderr}"
    );
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
