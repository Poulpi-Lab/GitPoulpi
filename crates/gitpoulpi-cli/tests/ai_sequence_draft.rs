#![cfg(feature = "ai")]
//! `gitpoulpi ai sequence-draft`: the sequence a scripted local agent
//! proposes, printed as JSON and saved nowhere; the names of the installation
//! go along only with `--context`, the edited sequence only with `--edit`.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_testkit::fake_agent::{agent_calls, ai_config_toml, write_fake_agent};
#[cfg(unix)]
use gitpoulpi_testkit::fake_agent::{agent_prompt, write_capturing_fake_agent};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::Value;

/// One good step, one destructive step (refused), one more good step.
const WITH_A_REFUSED_STEP: &[&str] = &[
    "```json",
    "{\"name\": \"Backup\", \"id\": \"backup\",",
    " \"description\": \"Stage, commit, push\",",
    " \"steps\": [",
    "  {\"run\": \"stage_all\"},",
    "  {\"run\": \"discard_all_changes\"},",
    "  {\"run\": \"push\"}",
    " ],",
    " \"shortcut\": \"Ctrl+G b\",",
    " \"notes\": [\"No way to skip hooks.\"]}",
    "```",
];

const ALL_GOOD: &[&str] = &[
    "```json",
    "{\"name\": \"Sync\", \"id\": \"sync\", \"steps\": [{\"run\": \"fetch\"}, {\"run\": \"pull\"}]}",
    "```",
];

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "feat: first thing")
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
        .args(["ai", "sequence-draft"])
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

#[test]
fn prints_the_sequence_and_what_it_left_out_and_saves_nothing() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), WITH_A_REFUSED_STEP);
    let config_file = config.path().join("config.toml");
    std::fs::write(&config_file, ai_config_toml(&agent)).unwrap();

    let (code, value, stderr) = run(
        config.path(),
        &repo,
        "cli-seq-draft",
        &["back everything up"],
    );
    // One step was refused: exit 2, the other two are kept.
    assert_eq!(code, 2, "{value} {stderr}");
    assert_eq!(value["applied"], false);
    assert_eq!(value["sequence"]["id"], "backup");
    assert_eq!(value["sequence"]["name"], "Backup");
    let steps = value["sequence"]["steps"].as_array().unwrap();
    let runs: Vec<&str> = steps.iter().map(|s| s["run"].as_str().unwrap()).collect();
    assert_eq!(runs, ["stage_all", "push"]);
    let problems = value["problems"].as_array().unwrap();
    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0]["position"], 2);
    assert_eq!(problems[0]["what"], "discard_all_changes");
    assert!(problems[0]["reason"]
        .as_str()
        .unwrap()
        .contains("throws work away"));
    assert_eq!(value["notes"][0], "No way to skip hooks.");
    assert_eq!(value["suggested_shortcut"], "Ctrl+G b");
    assert!(value["shortcut_conflict"].is_null());
    assert_eq!(value["retried"], false);
    assert_eq!(value["context"], false);
    // The TOML is what `sequence add --file` reads.
    let toml: toml::Table = toml::from_str(value["toml"].as_str().unwrap()).unwrap();
    assert_eq!(toml["id"].as_str(), Some("backup"));
    assert_eq!(agent_calls(&agent), 1);
    // The installation's own record may be written by any command; no
    // sequence is.
    let after: toml::Table =
        toml::from_str(&std::fs::read_to_string(&config_file).unwrap()).unwrap();
    assert!(!after.contains_key("sequences"), "{after:?}");
}

#[test]
fn a_clean_proposal_exits_zero() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), ALL_GOOD);
    std::fs::write(config.path().join("config.toml"), ai_config_toml(&agent)).unwrap();
    let (code, value, stderr) = run(config.path(), &repo, "cli-seq-draft-ok", &["sync it"]);
    assert_eq!(code, 0, "{value} {stderr}");
    assert!(value["problems"].as_array().unwrap().is_empty());
    assert_eq!(value["sequence"]["steps"].as_array().unwrap().len(), 2);
}

#[cfg(unix)]
#[test]
fn edit_sends_the_sequence_and_context_sends_names_only_when_asked() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let agent = write_capturing_fake_agent(tools.path(), ALL_GOOD);
    let mut text = ai_config_toml(&agent);
    text.push_str(
        "\n[[sequences]]\nid = \"nightly\"\nname = \"Nightly\"\nscope = \"machine\"\n\n[[sequences.steps]]\nrun = \"stash_push\"\n",
    );
    std::fs::write(config.path().join("config.toml"), text).unwrap();
    repo.git(&["remote", "add", "backup-nas", "/tmp/nowhere.git"]);

    // Creating: neither the sequence nor the remote name is sent.
    run(config.path(), &repo, "cli-seq-draft-new", &["do it"]);
    let prompt = agent_prompt(&agent);
    assert!(prompt.contains("do it"), "{prompt}");
    assert!(!prompt.contains("Nightly") && !prompt.contains("backup-nas"));

    // Editing: the current sequence is sent, the id is kept in the answer.
    let (_, value, stderr) = run(
        config.path(),
        &repo,
        "cli-seq-draft-edit",
        &["add a fetch", "--edit", "nightly"],
    );
    let prompt = agent_prompt(&agent);
    assert!(prompt.contains("The sequence to change"), "{prompt}");
    assert!(prompt.contains("\"id\": \"nightly\""), "{prompt}");
    assert!(prompt.contains("\"run\": \"stash_push\""), "{prompt}");
    assert!(!prompt.contains("backup-nas"));
    assert_eq!(value["sequence"]["id"], "nightly", "{value} {stderr}");
    assert_eq!(value["sequence"]["scope"], "machine");

    // An unknown id is refused before anything is asked.
    let calls = agent_calls(&agent);
    let (code, value, _) = run(
        config.path(),
        &repo,
        "cli-seq-draft-missing",
        &["x", "--edit", "nope"],
    );
    assert_ne!(code, 0);
    assert!(value["error"].as_str().unwrap().contains("no sequence"));
    assert_eq!(agent_calls(&agent), calls);

    // With --context the remote name goes along.
    let (_, value, _) = run(
        config.path(),
        &repo,
        "cli-seq-draft-ctx",
        &["do it", "--context"],
    );
    assert_eq!(value["context"], true);
    let prompt = agent_prompt(&agent);
    assert!(prompt.contains("backup-nas"), "{prompt}");
}

#[test]
fn with_ai_off_it_refuses_and_asks_nothing() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config.path())
        .arg("--json")
        .args(["--session", "cli-seq-draft-offcfg"])
        .args(["--repo", &repo.root().to_string_lossy()])
        .args(["ai", "sequence-draft", "x"])
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
