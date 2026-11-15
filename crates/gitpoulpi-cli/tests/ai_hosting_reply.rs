#![cfg(all(feature = "ai", feature = "github"))]
//! `gitpoulpi ai hosting-reply`: the draft a scripted local agent writes for
//! a conversation, printed as JSON. The AI only drafts: every answer says
//! `posted: false`. `--read-repo` lets it read the repository (off by default):
//! a local agent in read-only mode, or a cleaned context for an API profile.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_testkit::fake_agent::{agent_calls, ai_config_toml, write_fake_agent};
#[cfg(unix)]
use gitpoulpi_testkit::fake_agent::{
    agent_prompt, agent_runs, write_capturing_fake_agent, write_recording_fake_agent,
};
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

/// An agent the harness table knows (Claude Code's flags), whose command is
/// the fake script.
#[cfg(unix)]
fn claude_like_config(dir: &Path, agent: &Path, extra: &str) {
    let body = format!(
        "[ai]\nenabled = true\n\n[[ai.profiles]]\nid = \"fake\"\nname = \"Fake agent\"\n\
         kind = \"local_agent\"\nharness = \"claude\"\ncommand = '{}'\n\
         args = [\"-p\", \"--no-session-persistence\"]\n{extra}\n",
        agent.display()
    );
    std::fs::write(dir.join("config.toml"), body).unwrap();
}

#[cfg(unix)]
fn commit_more(repo: &gitpoulpi_testkit::FixtureRepo, name: &str) {
    std::fs::write(repo.root().join(name), "more\n").unwrap();
    repo.git(&["add", name]);
    repo.git(&["commit", "-q", "-m", &format!("feat: {name}")]);
}

fn flag_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

/// The default is off: without `--read-repo` (and with the setting off) the
/// agent gets the plain arguments, no read-only flags and no session.
#[test]
#[cfg(unix)]
fn the_repository_is_not_read_by_default() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let agent = write_recording_fake_agent(tools.path(), &["Fine."]);
    claude_like_config(config.path(), &agent, "");
    let file = write_page(tools.path(), vec![comment("c1", "bob", "hello")]);
    let (code, value) = run(
        config.path(),
        repo.root(),
        "cli-ai-reply-off",
        &["--conversation", &file],
    );
    assert_eq!(code, 0, "{value}");
    assert_eq!(value["context"]["repository"], false);
    assert!(value["context"].get("repo").is_none(), "{value}");
    let runs = agent_runs(&agent);
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].args, ["-p", "--no-session-persistence"]);
}

/// `--read-repo` with a local agent: it runs in the repository, with
/// read-only tools, no hosting credentials and a session it keeps between
/// replies; a commit, or `--refresh-context`, starts a new one.
#[test]
#[cfg(unix)]
fn read_repo_runs_the_agent_in_the_repository_read_only_and_keeps_its_session() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let agent = write_recording_fake_agent(tools.path(), &["A draft."]);
    claude_like_config(config.path(), &agent, "");
    let file = write_page(tools.path(), vec![comment("c1", "bob", "hello")]);
    let ask = |extra: &[&str]| {
        let mut args = vec!["--conversation", file.as_str(), "--read-repo"];
        args.extend_from_slice(extra);
        let output = Command::cargo_bin("gitpoulpi")
            .unwrap()
            .env("GITPOULPI_CONFIG_DIR", config.path())
            .env("GH_TOKEN", "must-not-reach-the-agent")
            .env("GITLAB_TOKEN", "nor-this")
            .arg("--json")
            .args(["--session", "cli-ai-reply-read"])
            .args(["--repo", &repo.root().to_string_lossy()])
            .args(["ai", "hosting-reply"])
            .args(&args)
            .current_dir(repo.root())
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let value: Value = serde_json::from_str(&stdout)
            .unwrap_or_else(|e| panic!("not JSON ({e}): {stdout} {:?}", output.stderr));
        assert!(output.status.success(), "{value}");
        value
    };

    let first = ask(&[]);
    assert_eq!(first["posted"], false, "the AI only drafts");
    assert_eq!(first["context"]["repository"], true);
    assert_eq!(first["context"]["repo"]["route"], "agent");
    assert_eq!(first["context"]["repo"]["agent_session"], "started");
    let run = &agent_runs(&agent)[0];
    assert_eq!(
        run.cwd.canonicalize().unwrap(),
        repo.root().canonicalize().unwrap(),
        "the agent runs in the repository"
    );
    assert_eq!(
        flag_value(&run.args, "--tools").as_deref(),
        Some("Read,Grep,Glob")
    );
    let denied = flag_value(&run.args, "--disallowedTools").unwrap();
    for tool in ["Bash", "Edit", "Write", "WebFetch"] {
        assert!(denied.contains(tool), "{tool}: {:?}", run.args);
    }
    assert!(run.args.contains(&"--strict-mcp-config".to_string()));
    assert!(
        !run.args.contains(&"--no-session-persistence".to_string()),
        "a session is kept: {:?}",
        run.args
    );
    assert!(
        run.credential_env.is_empty(),
        "no hosting credential reaches the agent: {:?}",
        run.credential_env
    );
    let id = flag_value(&run.args, "--session-id").expect("the session is started");

    let second = ask(&[]);
    assert_eq!(second["context"]["repo"]["agent_session"], "continued");
    let run = &agent_runs(&agent)[1];
    assert_eq!(
        flag_value(&run.args, "--resume").as_deref(),
        Some(id.as_str())
    );
    assert!(!run.args.contains(&"--session-id".to_string()));

    // A new commit moves HEAD: a new session.
    commit_more(&repo, "b.txt");
    let third = ask(&[]);
    assert_eq!(third["context"]["repo"]["agent_session"], "started");
    let run = &agent_runs(&agent)[2];
    let new_id = flag_value(&run.args, "--session-id").expect("a new session");
    assert_ne!(new_id, id);

    // "Refresh context" starts over too.
    let again = ask(&["--refresh-context"]);
    assert_eq!(again["context"]["repo"]["agent_session"], "started");
    let run = &agent_runs(&agent)[3];
    assert_ne!(flag_value(&run.args, "--session-id").unwrap(), new_id);
}

/// The setting turns reading on without the flag.
#[test]
#[cfg(unix)]
fn the_setting_turns_reading_on_without_the_flag() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let agent = write_recording_fake_agent(tools.path(), &["Fine."]);
    claude_like_config(
        config.path(),
        &agent,
        "[ai.hosting_reply]\nread_repository = true\n",
    );
    let file = write_page(tools.path(), vec![comment("c1", "bob", "hello")]);
    let (code, value) = run(
        config.path(),
        repo.root(),
        "cli-ai-reply-setting",
        &["--conversation", &file],
    );
    assert_eq!(code, 0, "{value}");
    assert_eq!(value["context"]["repo"]["route"], "agent");
}

/// An agent whose read-only mode GitPoulpi does not trust cannot read the
/// repository: refused before anything runs.
#[test]
fn read_repo_with_an_agent_without_a_trusted_read_only_mode_is_refused() {
    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(tools.path(), &["never"]);
    config_with(config.path(), &agent, "");
    let file = write_page(tools.path(), vec![comment("c1", "bob", "hello")]);
    let (code, value) = run(
        config.path(),
        repo.root(),
        "cli-ai-reply-refused",
        &["--conversation", &file, "--read-repo"],
    );
    assert_ne!(code, 0, "{value}");
    assert!(value.get("draft").is_none(), "{value}");
    assert!(
        value.to_string().contains("no read-only mode"),
        "the refusal says why: {value}"
    );
    assert_eq!(agent_calls(&agent), 0, "the agent was never run");
}

/// The API route: the context GitPoulpi builds goes after the instructions,
/// secrets masked and sensitive files left out, and is kept privately for
/// the next reply. `--print-sent` shows the request without sending it.
#[test]
#[cfg(unix)]
fn read_repo_over_an_api_profile_sends_a_cleaned_context_and_keeps_it_privately() {
    let token = "ghp_q7Xk2LmZp9RtV4bN8wYcJ3hGfD5sAe6U1oIx";
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file(
            "src/crash.rs",
            &format!("fn crash_on_empty() {{}}\nconst KEY: &str = \"{token}\";\n"),
            "feat: crash",
        )
        .unwrap()
        .commit_file(".env", "DB_PASSWORD=hunter2hunter2\n", "chore: env")
        .unwrap()
        .build();
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        "[ai]\nenabled = true\n\n[[ai.profiles]]\nid = \"api\"\nname = \"API\"\nkind = \"anthropic\"\n",
    )
    .unwrap();
    let file = write_page(
        tools.path(),
        vec![comment(
            "c1",
            "bob",
            "It dies in src/crash.rs:1 and also reads .env",
        )],
    );
    let (code, value) = run(
        config.path(),
        repo.root(),
        "cli-ai-reply-api",
        &["--conversation", &file, "--read-repo", "--print-sent"],
    );
    assert_eq!(code, 0, "{value}");
    assert_eq!(value["dry_run"], true);
    let system = value["sent"][0]["system"].as_str().unwrap();
    assert!(system.contains("fn crash_on_empty"), "{system}");
    assert!(
        system.contains("Repository files for the reply"),
        "{system}"
    );
    assert!(!system.contains(token), "the token leaked: {system}");
    assert!(system.contains("[REDACTED:"), "{system}");
    assert!(!system.contains("hunter2"), "{system}");

    // Kept for the next reply, readable by the user only.
    let dir = config
        .path()
        .join("data")
        .join("private")
        .join("ai-context");
    let kept: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|e| e.unwrap())
        .collect();
    assert!(!kept.is_empty(), "the context was kept");
    {
        use std::os::unix::fs::PermissionsExt;
        for entry in &kept {
            let mode = entry.metadata().unwrap().permissions().mode();
            assert_eq!(mode & 0o077, 0, "{:?} is private to the user", entry.path());
        }
    }
    let text = std::fs::read_to_string(kept[0].path()).unwrap();
    assert!(
        !text.contains(token) && !text.contains("hunter2"),
        "the cache holds no secret"
    );
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
