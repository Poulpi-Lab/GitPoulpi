#![cfg(all(feature = "ai", unix))]
//! The AI input caps (`[ai.limits]`) through the real binary, the daemon and a
//! capturing fake agent. Nobody can answer a dialog here: an input over a cap
//! is truncated like it always was, but never silently (`truncated` names the
//! part, its size, the cap and the setting), and `--send-all` sends it whole.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_testkit::fake_agent::{
    agent_calls, agent_prompt, ai_config_toml, write_capturing_fake_agent,
};
use gitpoulpi_testkit::FixtureRepoBuilder;
use serde_json::Value;

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

type Setup = (
    gitpoulpi_testkit::FixtureRepo,
    tempfile::TempDir,
    tempfile::TempDir,
    std::path::PathBuf,
    String,
);

/// A repository with one staged file of about 90 KB of changed lines.
fn setup(name: &str, extra_config: &str) -> Setup {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let agent = write_capturing_fake_agent(tools.path(), &["feat: add things"]);
    std::fs::write(
        config.path().join("config.toml"),
        format!("{}\n{extra_config}", ai_config_toml(&agent)),
    )
    .unwrap();
    let body: String = (0..3_000)
        .map(|i| format!("let value_{i} = compute({i});\n"))
        .collect();
    std::fs::write(repo.root().join("big.rs"), body).unwrap();
    let out = std::process::Command::new("git")
        .args(["add", "big.rs"])
        .current_dir(repo.root())
        .output()
        .unwrap();
    assert!(out.status.success());
    (repo, tools, config, agent, format!("cli-ailimits-{name}"))
}

#[test]
fn an_oversized_diff_is_truncated_and_reported_never_silently() {
    let (repo, _tools, config, agent, session) = setup("report", "");
    let (code, json, stderr) = run(
        config.path(),
        repo.root(),
        &session,
        &["ai", "commit-message"],
    );
    assert_eq!(code, Some(0), "{json} {stderr}");
    assert_eq!(json["summary"], "feat: add things");
    let truncated = json["truncated"].as_array().expect("truncated").clone();
    assert_eq!(truncated.len(), 1, "{json}");
    let cut = &truncated[0];
    assert_eq!(cut["part"], "diff");
    assert_eq!(cut["limit"], 60_000);
    assert!(cut["bytes"].as_u64().unwrap() > 60_000, "{json}");
    assert_eq!(cut["setting"], "ai.limits.diff_bytes");
    let prompt = agent_prompt(&agent);
    assert!(prompt.contains("[diff truncated: showing"), "{prompt}");
    assert!(!prompt.contains("value_2999"), "the tail was cut");
}

#[test]
fn send_all_sends_the_whole_diff_and_reports_nothing() {
    let (repo, _tools, config, agent, session) = setup("all", "");
    let (code, json, stderr) = run(
        config.path(),
        repo.root(),
        &session,
        &["ai", "commit-message", "--send-all"],
    );
    assert_eq!(code, Some(0), "{json} {stderr}");
    assert!(json.get("truncated").is_none(), "{json}");
    let prompt = agent_prompt(&agent);
    assert!(prompt.contains("value_2999"), "the whole diff");
    assert!(!prompt.contains("diff truncated"), "{prompt}");
    assert_eq!(agent_calls(&agent), 1);
}

#[test]
fn a_raised_limit_sends_the_whole_diff_without_the_flag() {
    let (repo, _tools, config, agent, session) =
        setup("raised", "[ai.limits]\ndiff_bytes = 200000\n");
    let (code, json, stderr) = run(
        config.path(),
        repo.root(),
        &session,
        &["ai", "commit-message"],
    );
    assert_eq!(code, Some(0), "{json} {stderr}");
    assert!(json.get("truncated").is_none(), "{json}");
    assert!(agent_prompt(&agent).contains("value_2999"));
}

#[test]
fn explain_reports_the_same_way_and_the_flag_works_after_the_command() {
    let (repo, _tools, config, agent, session) = setup("explain", "");
    let (code, json, stderr) = run(
        config.path(),
        repo.root(),
        &session,
        &["ai", "explain", "--wip"],
    );
    assert_eq!(code, Some(0), "{json} {stderr}");
    assert_eq!(json["truncated"][0]["part"], "diff", "{json}");
    assert!(json["truncated"][0]["bytes"].as_u64().unwrap() > 60_000);
    // `--no-truncate` is the same flag, and it is global to `ai`.
    let (code, json, stderr) = run(
        config.path(),
        repo.root(),
        &session,
        &["ai", "explain", "--wip", "--no-truncate"],
    );
    assert_eq!(code, Some(0), "{json} {stderr}");
    assert!(json.get("truncated").is_none(), "{json}");
    assert!(agent_prompt(&agent).contains("value_2999"));
}

#[test]
fn commit_ai_and_stash_ai_report_and_take_the_flag() {
    let (repo, _tools, config, agent, session) = setup("commit", "");
    let (code, json, stderr) = run(config.path(), repo.root(), &session, &["commit", "--ai"]);
    assert_eq!(code, Some(0), "{json} {stderr}");
    assert!(json["commit"].is_string(), "{json}");
    assert_eq!(json["truncated"][0]["setting"], "ai.limits.diff_bytes");
    assert!(!agent_prompt(&agent).contains("value_2999"));

    // `--send-all` needs `--ai`.
    let (code, _, _) = run(
        config.path(),
        repo.root(),
        &session,
        &["commit", "-m", "x", "--send-all"],
    );
    assert_ne!(code, Some(0));

    // The next change, stashed with a name from the AI.
    let body: String = (0..3_000)
        .map(|i| format!("let other_{i} = compute({i});\n"))
        .collect();
    std::fs::write(repo.root().join("big.rs"), body).unwrap();
    let (code, json, stderr) = run(
        config.path(),
        repo.root(),
        &session,
        &["stash", "push", "--ai", "--send-all"],
    );
    assert_eq!(code, Some(0), "{json} {stderr}");
    assert!(json.get("truncated").is_none(), "{json}");
    assert!(
        agent_prompt(&agent).contains("other_2999"),
        "the whole diff"
    );
}

/// A conflict is refused over its cap (never cut): the error names the
/// setting, and `--send-all` sends it whole.
#[test]
fn a_conflict_over_its_cap_is_refused_unless_send_all() {
    let fixture = gitpoulpi_testkit::ConflictFixture::large_text(25_000, "ours");
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let agent = write_capturing_fake_agent(tools.path(), &["```", "resolved", "```"]);
    std::fs::write(config.path().join("config.toml"), ai_config_toml(&agent)).unwrap();
    let session = "cli-ailimits-conflict";
    let (code, json, _) = run(
        config.path(),
        fixture.root(),
        session,
        &["ai", "resolve-conflict", "notes.txt"],
    );
    assert_ne!(code, Some(0), "{json}");
    assert!(
        json.to_string().contains("ai.limits.conflict_bytes"),
        "{json}"
    );
    assert_eq!(agent_calls(&agent), 0);
    let (code, json, stderr) = run(
        config.path(),
        fixture.root(),
        session,
        &["ai", "resolve-conflict", "notes.txt", "--send-all"],
    );
    assert_eq!(code, Some(0), "{json} {stderr}");
    assert_eq!(json["resolution"][0], "resolved", "{json}");
    assert_eq!(agent_calls(&agent), 1);
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
