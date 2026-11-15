#![cfg(all(feature = "ai", feature = "github"))]
//! `gitpoulpi ai hosting-reply`: the draft a scripted local agent writes for
//! a conversation, printed as JSON. The AI only drafts: every answer says
//! `posted: false`, and `--read-repo` is refused until reading the
//! repository is built.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_testkit::fake_agent::{agent_calls, ai_config_toml, write_fake_agent};
#[cfg(unix)]
use gitpoulpi_testkit::fake_agent::{agent_prompt, write_capturing_fake_agent};
use gitpoulpi_testkit::FixtureRepoBuilder;
use serde_json::{json, Value};

fn run_with(config: &Path, root: &Path, session: &str, args: &[&str], stdin: &str) -> (i32, Value) {
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config)
        .arg("--json")
        .args(["--session", session])
        .args(["--repo", &root.to_string_lossy()])
        .args(["ai", "hosting-reply"])
        .args(args)
        .write_stdin(stdin.to_string())
        .current_dir(root)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let value =
        serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("not JSON ({e}): {stdout}"));
    (output.status.code().unwrap_or(-1), value)
}

fn run(config: &Path, root: &Path, session: &str, args: &[&str]) -> (i32, Value) {
    run_with(config, root, session, args, "")
}

fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "feat: first thing")
        .unwrap()
        .build()
}

fn actor(login: &str) -> Value {
    json!({ "login": login, "display": login, "kind": "user", "web_url": null })
}

fn comment(id: &str, who: &str, text: &str) -> Value {
    json!({
        "entry": "comment", "id": id, "author": actor(who),
        "body": { "text": text, "flavor": "gfm" },
        "created_at": "2026-11-02", "updated_at": null, "edited": false,
        "reactions": [], "in_reply_to": null, "discussion_id": null,
        "internal": false, "can_edit": null, "can_delete": null, "web_url": null
    })
}

/// A conversation page as `hosting` prints it: `header` and `entries`.
fn page(entries: Vec<Value>) -> Value {
    json!({
        "header": {
            "key": { "kind": "pull_request", "number": 7, "id": "" },
            "reference": "#7", "title": "Fix the crash", "state_label": "open",
            "open": true, "author": actor("alice"),
            "body": { "text": "It fixes the empty config crash.", "flavor": "gfm" },
            "created_at": "2026-11-01", "web_url": "https://example.test/7",
            "labels": ["bug"]
        },
        "entries": entries,
        "older": null,
        "total_hint": null
    })
}

fn event() -> Value {
    json!({ "entry": "event", "id": "e1", "actor": null, "at": "2026-11-03",
            "kind": { "kind": "closed" } })
}

/// A config turning AI on with the fake agent, plus `extra` lines.
fn config_with(dir: &Path, agent: &Path, extra: &str) {
    let body = format!("{}\n{extra}\n", ai_config_toml(agent));
    std::fs::write(dir.join("config.toml"), body).unwrap();
}

fn write_page(dir: &Path, entries: Vec<Value>) -> String {
    let path = dir.join("conversation.json");
    std::fs::write(&path, page(entries).to_string()).unwrap();
    path.to_string_lossy().into_owned()
}

#[test]
fn prints_the_draft_and_says_nothing_was_posted() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    #[cfg(unix)]
    let agent = write_capturing_fake_agent(tools.path(), &["Thanks, I will look tomorrow."]);
    #[cfg(not(unix))]
    let agent = write_fake_agent(tools.path(), &["Thanks, I will look tomorrow."]);
    config_with(config.path(), &agent, "");
    let file = write_page(
        tools.path(),
        vec![
            comment("c1", "bob", "Can you take a look?"),
            event(),
            comment("c2", "carol", "Same crash here."),
        ],
    );

    let (code, value) = run(
        config.path(),
        repo.root(),
        "cli-ai-reply",
        &[
            "--conversation",
            &file,
            "--hint",
            "say I will look tomorrow",
        ],
    );
    assert_eq!(code, 0, "{value}");
    assert_eq!(value["draft"], "Thanks, I will look tomorrow.");
    assert_eq!(value["action"], "hosting_reply");
    assert_eq!(value["target"], "comment");
    assert_eq!(value["improved"], false);
    assert_eq!(value["posted"], false, "the AI only drafts");
    assert_eq!(
        value["context"],
        json!({ "entries_sent": 2, "entries_omitted": 0, "repository": false }),
        "the system event is not an entry"
    );
    assert!(value.get("truncated").is_none(), "{value}");
    assert_eq!(agent_calls(&agent), 1);
    #[cfg(unix)]
    {
        let prompt = agent_prompt(&agent);
        assert!(prompt.contains("You help the user write text"), "{prompt}");
        assert!(prompt.contains("Can you take a look?"), "{prompt}");
        assert!(prompt.contains("Same crash here."), "{prompt}");
        assert!(prompt.contains("say I will look tomorrow"), "{prompt}");
        assert!(prompt.contains("Pull request #7"), "{prompt}");
        assert!(prompt.contains("GitHub Flavored Markdown"), "{prompt}");
        assert!(prompt.contains("<untrusted>"), "{prompt}");
    }
}

#[test]
fn a_thread_answer_that_improves_a_draft_says_so() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    #[cfg(unix)]
    let agent = write_capturing_fake_agent(tools.path(), &["Better wording."]);
    #[cfg(not(unix))]
    let agent = write_fake_agent(tools.path(), &["Better wording."]);
    config_with(config.path(), &agent, "");
    let mut root = comment("t1", "bob", "THREADROOT");
    root["discussion_id"] = json!("d9");
    let file = write_page(tools.path(), vec![root, comment("o1", "dave", "OTHERTALK")]);

    let (code, value) = run(
        config.path(),
        repo.root(),
        "cli-ai-reply-thread",
        &[
            "--conversation",
            &file,
            "--reply-to",
            "d9",
            "--draft",
            "thx will do",
        ],
    );
    assert_eq!(code, 0, "{value}");
    assert_eq!(value["target"], "thread_reply");
    assert_eq!(value["improved"], true);
    assert_eq!(value["draft"], "Better wording.");
    #[cfg(unix)]
    {
        let prompt = agent_prompt(&agent);
        assert!(prompt.contains("Thread being answered"), "{prompt}");
        assert!(prompt.contains("thx will do"), "{prompt}");
    }
}

/// Owner requirement: the base prompt is the user's to edit.
#[test]
#[cfg(unix)]
fn the_users_base_prompt_replaces_the_built_in_one() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let agent = write_capturing_fake_agent(tools.path(), &["Arr."]);
    config_with(
        config.path(),
        &agent,
        "[ai.hosting_reply]\nbase_prompt = \"Reply like a pirate.\"\n",
    );
    let file = write_page(tools.path(), vec![comment("c1", "bob", "hello")]);
    let (code, value) = run(
        config.path(),
        repo.root(),
        "cli-ai-reply-base",
        &["--conversation", &file],
    );
    assert_eq!(code, 0, "{value}");
    let prompt = agent_prompt(&agent);
    assert!(prompt.contains("Reply like a pirate."), "{prompt}");
    assert!(!prompt.contains("You help the user write text"), "{prompt}");
    // The request still marks other people's text as untrusted.
    assert!(prompt.contains("<untrusted>"), "{prompt}");
}

#[test]
fn a_new_issue_description_needs_no_conversation() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &["Steps to reproduce."]);
    config_with(config.path(), &agent, "");
    let (code, value) = run(
        config.path(),
        repo.root(),
        "cli-ai-reply-issue",
        &[
            "--issue-title",
            "Crash on empty config",
            "--flavor",
            "gitlab_md",
        ],
    );
    assert_eq!(code, 0, "{value}");
    assert_eq!(value["action"], "issue_body");
    assert_eq!(value["target"], "issue_body");
    assert_eq!(value["posted"], false);
    assert_eq!(value["context"]["entries_sent"], 0);
}

#[test]
fn reading_the_repository_is_refused_and_nothing_is_asked() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &["never"]);
    config_with(config.path(), &agent, "");
    let file = write_page(tools.path(), vec![comment("c1", "bob", "hello")]);
    let (code, value) = run(
        config.path(),
        repo.root(),
        "cli-ai-reply-read",
        &["--conversation", &file, "--read-repo"],
    );
    assert_ne!(code, 0, "{value}");
    assert!(value.get("draft").is_none(), "{value}");
    assert!(
        value.to_string().contains("not built yet"),
        "the refusal says why: {value}"
    );
    assert_eq!(agent_calls(&agent), 0, "the agent was never run");
}

#[test]
fn a_conversation_over_its_limit_is_truncated_and_reported_and_send_all_lifts_it() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &["ok"]);
    config_with(
        config.path(),
        &agent,
        "[ai.limits]\nhosting_reply_context_bytes = 2000\n",
    );
    let many: Vec<Value> = (0..20)
        .map(|i| {
            comment(
                &format!("c{i:02}"),
                "bob",
                &format!("{i:02} {}", "x".repeat(400)),
            )
        })
        .collect();
    let file = write_page(tools.path(), many);

    let (code, value) = run(
        config.path(),
        repo.root(),
        "cli-ai-reply-limit",
        &["--conversation", &file],
    );
    assert_eq!(code, 0, "{value}");
    assert!(value["context"]["entries_omitted"].as_u64().unwrap() > 0);
    let cuts = value["truncated"]
        .as_array()
        .expect("truncated is reported");
    assert!(!cuts.is_empty());
    assert_eq!(cuts[0]["part"], "reply_conversation");
    assert_eq!(cuts[0]["setting"], "ai.limits.hosting_reply_context_bytes");
    assert_eq!(cuts[0]["limit"], 2000);

    let (code, value) = run(
        config.path(),
        repo.root(),
        "cli-ai-reply-limit",
        &["--conversation", &file, "--send-all"],
    );
    assert_eq!(code, 0, "{value}");
    assert_eq!(value["context"]["entries_omitted"], 0);
    assert!(value.get("truncated").is_none(), "{value}");
}

#[test]
fn the_conversation_can_come_from_stdin_or_inline_and_a_bad_one_is_an_error() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &["ok"]);
    config_with(config.path(), &agent, "");
    let text = page(vec![comment("c1", "bob", "hello")]).to_string();

    let (code, value) = run_with(
        config.path(),
        repo.root(),
        "cli-ai-reply-stdin",
        &["--conversation", "-"],
        &text,
    );
    assert_eq!(code, 0, "{value}");
    assert_eq!(value["context"]["entries_sent"], 1);

    let (code, value) = run(
        config.path(),
        repo.root(),
        "cli-ai-reply-inline",
        &["--conversation", &text],
    );
    assert_eq!(code, 0, "{value}");

    let (code, value) = run(
        config.path(),
        repo.root(),
        "cli-ai-reply-bad",
        &["--conversation", "{\"nope\": 1}"],
    );
    assert_ne!(code, 0, "{value}");
    assert!(value.get("draft").is_none());
}

#[test]
fn with_ai_off_nothing_is_drafted() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &["never"]);
    std::fs::write(
        config.path().join("config.toml"),
        ai_config_toml(&agent).replace("enabled = true", "enabled = false"),
    )
    .unwrap();
    let file = write_page(tools.path(), vec![comment("c1", "bob", "hello")]);
    let (code, value) = run(
        config.path(),
        repo.root(),
        "cli-ai-reply-off",
        &["--conversation", &file],
    );
    assert_ne!(code, 0, "{value}");
    assert_eq!(agent_calls(&agent), 0);
}

#[test]
fn the_targets_exclude_each_other() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config.path())
        .args(["--session", "cli-ai-reply-excl"])
        .args(["--repo", &repo.root().to_string_lossy()])
        .args([
            "ai",
            "hosting-reply",
            "--issue-title",
            "T",
            "--reply-to",
            "x",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("cannot be used with"), "{stderr}");
}
