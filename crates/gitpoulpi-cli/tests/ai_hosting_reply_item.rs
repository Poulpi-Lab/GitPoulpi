#![cfg(all(feature = "ai", feature = "github", unix))]
//! `gitpoulpi ai hosting-reply` over the conversation the `hosting` commands
//! print (`hosting issue comments 7 --json | ai hosting-reply --conversation -`)
//! and over `--item`, which reads it from the host itself. Against
//! `xtask mock-host`; the agent is a script that records its prompt. The AI
//! only drafts: every answer says `posted: false`.

use assert_cmd::Command;
use gitpoulpi_testkit::fake_agent::{
    agent_calls, agent_prompt, ai_config_toml, write_capturing_fake_agent,
};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::Value;
use xtask::mock_host::{self, MockHost, Options};

struct Rig {
    config: tempfile::TempDir,
    repo: FixtureRepo,
    mock: MockHost,
    agent: std::path::PathBuf,
    _tools: tempfile::TempDir,
    session: &'static str,
}

fn rig(session: &'static str) -> Rig {
    let mock = mock_host::start(Options::default()).expect("start the mock host");
    let tools = tempfile::tempdir().unwrap();
    let agent = write_capturing_fake_agent(tools.path(), &["Thanks, looking at it."]);
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        format!(
            "{}\n[[integrations]]\nid = \"h\"\nkind = \"github_enterprise\"\nurl = \"http://127.0.0.1:{}\"\ntoken_ref = \"integration:h\"\nname = \"h\"\naccount = \"mara\"\nuser = {{ login = \"mara-quinn\" }}\n",
            ai_config_toml(&agent),
            mock.port
        ),
    )
    .unwrap();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "start")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .build();
    let rig = Rig {
        config,
        repo,
        mock,
        agent,
        _tools: tools,
        session,
    };
    for list in ["pulls", "issues"] {
        let (ok, out, _) = rig.run(
            &[
                "hosting",
                "link",
                list,
                "--id",
                "h",
                "--project",
                "owner/repo",
            ],
            "",
        );
        assert!(ok, "{out}");
    }
    rig
}

impl Rig {
    fn run(&self, args: &[&str], stdin: &str) -> (bool, Value, String) {
        let output = Command::cargo_bin("gitpoulpi")
            .unwrap()
            .env("GITPOULPI_CONFIG_DIR", self.config.path())
            .env("GITPOULPI_TOKEN_H", "test-token")
            .arg("--json")
            .args(["--session", self.session])
            .args(args)
            .write_stdin(stdin.to_string())
            .current_dir(self.repo.root())
            .output()
            .expect("run gitpoulpi");
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        // A refused command line (clap) prints no JSON, only a failure.
        let json = if output.status.success() {
            serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("not JSON ({e}): {stdout}"))
        } else {
            serde_json::from_str(&stdout).unwrap_or(Value::Null)
        };
        (output.status.success(), json, stdout)
    }
}

fn assert_draft(rig: &Rig, value: &Value, ok: bool) {
    assert!(ok, "{value}");
    assert_eq!(value["draft"], "Thanks, looking at it.");
    assert_eq!(value["posted"], false, "the AI only drafts");
    assert_eq!(value["context"]["entries_sent"], 2, "{value}");
    assert_eq!(agent_calls(&rig.agent), 1);
    let prompt = agent_prompt(&rig.agent);
    assert!(prompt.contains("Issue #7"), "{prompt}");
    assert!(
        prompt.contains("Crash when the planet list is empty"),
        "{prompt}"
    );
    assert!(prompt.contains("I can reproduce this on #9."), "{prompt}");
    assert!(prompt.contains("GitHub Flavored Markdown"), "{prompt}");
}

#[test]
fn what_hosting_prints_is_what_the_reply_reads() {
    let rig = rig("ai-reply-pipe");
    let (ok, _, printed) = rig.run(&["hosting", "issue", "comments", "7"], "");
    assert!(ok, "{printed}");
    let (ok, value, _) = rig.run(
        &[
            "ai",
            "hosting-reply",
            "--conversation",
            "-",
            "--hint",
            "ack",
        ],
        &printed,
    );
    assert_draft(&rig, &value, ok);
    assert!(
        rig.mock.requests().iter().all(|r| r.method == "GET"),
        "reading and drafting post nothing"
    );
}

#[test]
fn item_reads_the_conversation_from_the_host_itself() {
    let rig = rig("ai-reply-item");
    let (ok, value, _) = rig.run(
        &["ai", "hosting-reply", "--item", "#7", "--hint", "ack"],
        "",
    );
    assert_draft(&rig, &value, ok);
    assert!(
        rig.mock
            .requests()
            .iter()
            .any(|r| r.method == "GET" && r.target.contains("issues/7")),
        "{:?}",
        rig.mock.requests()
    );
    assert!(rig.mock.requests().iter().all(|r| r.method == "GET"));
}

#[test]
fn item_and_conversation_exclude_each_other_and_a_bad_shape_is_an_error() {
    let rig = rig("ai-reply-item-bad");
    let (ok, value, _) = rig.run(
        &["ai", "hosting-reply", "--item", "7", "--conversation", "{}"],
        "",
    );
    assert!(!ok, "{value}");
    let (ok, value, _) = rig.run(&["ai", "hosting-reply", "--conversation", "{\"x\": 1}"], "");
    assert!(!ok);
    assert!(
        value["error"].as_str().unwrap().contains("`item`"),
        "{value}"
    );
}
