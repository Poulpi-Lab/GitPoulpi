//! `hosting issue comments|comment|comment-edit|comment-delete`, `hosting
//! review conversation|comment-edit|comment-delete` and `hosting markdown`
//! against `xtask mock-host`'s conversation endpoints. Assertions are on
//! deserialized JSON and on the requests the mock recorded; no network, own
//! config dir and fixture per test.

use assert_cmd::Command;
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::Value;
use xtask::mock_host::{self, conversation, MockHost, Options};

struct Rig {
    config: tempfile::TempDir,
    repo: FixtureRepo,
    /// Where `--project` commands run: not a repository.
    cwd: tempfile::TempDir,
    mock: MockHost,
    session: String,
    token: String,
}

/// `user`: the login the profile recorded at sign-in; an App installation
/// token has none (it cannot ask `/user`).
fn config_for(mock: &MockHost, kind: &str, url_suffix: &str, extra: &str, user: &str) -> String {
    let user = if user.is_empty() {
        String::new()
    } else {
        format!("user = {{ login = \"{user}\" }}\n")
    };
    format!(
        "{extra}[[integrations]]\nid = \"h\"\nkind = \"{kind}\"\nurl = \"http://127.0.0.1:{}{url_suffix}\"\ntoken_ref = \"integration:h\"\nname = \"h\"\naccount = \"mara\"\n{user}",
        mock.port
    )
}

/// `link`: the repository's issue and pull lists point at `project`.
fn rig(kind: &str, project: &str, session: &str, token: &str, extra: &str) -> Rig {
    let mock = mock_host::start(Options::default()).expect("start the mock host");
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        config_for(
            &mock,
            kind,
            "",
            extra,
            if token.starts_with("ghs_") {
                ""
            } else if kind.starts_with("github") {
                "mara-quinn"
            } else {
                "mara"
            },
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
        cwd: tempfile::tempdir().unwrap(),
        mock,
        session: session.to_string(),
        token: token.to_string(),
    };
    for list in ["pulls", "issues"] {
        rig.json(&["hosting", "link", list, "--id", "h", "--project", project]);
    }
    rig
}

impl Rig {
    fn run(&self, dir: &std::path::Path, args: &[&str]) -> (bool, Value) {
        let output = Command::cargo_bin("gitpoulpi")
            .unwrap()
            .env("GITPOULPI_CONFIG_DIR", self.config.path())
            .env("GITPOULPI_TOKEN_H", &self.token)
            .arg("--json")
            .args(["--session", &self.session])
            .args(args)
            .current_dir(dir)
            .output()
            .expect("run gitpoulpi");
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let json = serde_json::from_str(&stdout).unwrap_or_else(|e| {
            panic!(
                "not JSON ({e}): {stdout} / {}",
                String::from_utf8_lossy(&output.stderr)
            )
        });
        (output.status.success(), json)
    }

    /// In the repository.
    fn json(&self, args: &[&str]) -> Value {
        let (ok, json) = self.run(self.repo.root(), args);
        assert!(ok, "`{}` failed: {json}", args.join(" "));
        json
    }

    fn error(&self, args: &[&str]) -> String {
        let (ok, json) = self.run(self.repo.root(), args);
        assert!(!ok, "`{}` unexpectedly succeeded: {json}", args.join(" "));
        json["error"].as_str().expect("an error string").to_string()
    }

    /// Outside any repository (`--project`).
    fn project_json(&self, args: &[&str]) -> Value {
        let (ok, json) = self.run(self.cwd.path(), args);
        assert!(ok, "`{}` failed: {json}", args.join(" "));
        json
    }

    /// `(target, body)` of the requests of `method`.
    fn sent(&self, method: &str) -> Vec<(String, String)> {
        self.mock
            .requests()
            .iter()
            .filter(|r| r.method == method)
            .map(|r| (r.target.clone(), r.body.clone()))
            .collect()
    }
}

fn types(out: &Value) -> Vec<&str> {
    out["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["type"].as_str().unwrap())
        .collect()
}

#[test]
fn github_issue_comments_read_with_references_identity_and_capabilities() {
    let rig = rig(
        "github_enterprise",
        "owner/repo",
        "conv-gh-read",
        "test-token",
        "",
    );
    let out = rig.json(&["hosting", "issue", "comments", "7"]);
    assert_eq!(out["item"]["kind"], "issue");
    assert_eq!(out["item"]["number"], 7);
    assert_eq!(out["item"]["title"], "Crash when the planet list is empty");
    assert_eq!(out["item"]["body_flavor"], "gfm");
    assert_eq!(types(&out), ["comment", "event", "comment"]);
    assert_eq!(out["entries"][0]["author"]["login"], "tomas");
    assert_eq!(out["entries"][0]["body"], "I can reproduce this on #9.");
    assert_eq!(out["entries"][0]["refs"][0]["text"], "#9");
    assert_eq!(out["entries"][0]["refs"][0]["kind"], "issue");
    assert_eq!(out["entries"][0]["refs"][0]["in_app"], true);
    assert_eq!(out["entries"][1]["kind"], "labeled");
    assert_eq!(out["entries"][1]["added"][0], "bug");
    assert_eq!(out["entries"][2]["edited"], true);
    assert_eq!(out["identity"]["login"], "mara-quinn");
    assert_eq!(out["identity"]["kind"], "personal_token");
    assert_eq!(out["capabilities"]["comment"], true);
    assert_eq!(out["capabilities"]["reply_in_thread"], false);
    assert_eq!(out["capabilities"]["write_flavor"], "gfm");
    assert!(out["older"].is_null());
    assert_eq!(out["target"]["integration"], "h");
    assert!(rig.sent("POST").is_empty(), "a read posts nothing");

    let no_events = rig.json(&["hosting", "issue", "comments", "7", "--no-events"]);
    assert_eq!(types(&no_events), ["comment", "comment"]);
    let limited = rig.json(&["hosting", "issue", "comments", "#7", "--limit", "1"]);
    assert_eq!(types(&limited), ["comment"]);
    assert_eq!(limited["entries"][0]["id"], "2003");
    assert_eq!(limited["trimmed_older"], 2);
}

#[test]
fn posting_editing_and_deleting_send_the_exact_requests_and_tell_the_daemon() {
    let rig = rig(
        "github_enterprise",
        "owner/repo",
        "conv-gh-write",
        "test-token",
        "",
    );
    let posted = rig.json(&[
        "hosting",
        "issue",
        "comment",
        "7",
        "--body",
        "Fixed in #9\nsecond line",
    ]);
    assert_eq!(posted["comment"]["id"], "2100");
    assert_eq!(posted["comment"]["body"], "Fixed in #9\nsecond line");
    assert_eq!(posted["posted_as"], "mara-quinn");
    assert_eq!(posted["identity"]["kind"], "personal_token");
    assert_eq!(posted["notified"], true, "the daemon was told");
    let sent = rig.sent("POST");
    let (target, body) = sent.last().unwrap();
    assert_eq!(target, "/api/v3/repos/owner/repo/issues/7/comments");
    assert_eq!(
        serde_json::from_str::<Value>(body).unwrap(),
        serde_json::json!({ "body": "Fixed in #9\nsecond line" })
    );

    let file = rig.repo.root().join("edit.md");
    std::fs::write(&file, "edited\n").unwrap();
    let edited = rig.json(&[
        "hosting",
        "issue",
        "comment-edit",
        "7",
        "2100",
        "--body-file",
        file.to_str().unwrap(),
    ]);
    assert_eq!(edited["comment"]["body"], "edited\n");
    let patched = rig.sent("PATCH");
    assert_eq!(
        patched[0].0,
        "/api/v3/repos/owner/repo/issues/comments/2100"
    );
    assert_eq!(
        serde_json::from_str::<Value>(&patched[0].1).unwrap()["body"],
        "edited\n"
    );

    let deleted = rig.json(&["hosting", "issue", "comment-delete", "7", "2100"]);
    assert_eq!(deleted["deleted"], "2100");
    assert_eq!(deleted["notified"], true);
    assert_eq!(
        rig.sent("DELETE")[0].0,
        "/api/v3/repos/owner/repo/issues/comments/2100"
    );
}

#[test]
fn an_app_installation_token_posts_as_the_bot_and_post_as_names_it() {
    let rig = rig(
        "github_enterprise",
        "owner/repo",
        "conv-gh-bot",
        "ghs_installation",
        "",
    );
    let posted = rig.json(&["hosting", "issue", "comment", "7", "--body", "from the bot"]);
    assert_eq!(posted["identity"]["kind"], "app_bot");
    assert_eq!(
        posted["comment"]["author"]["login"],
        conversation::BOT_LOGIN
    );
    assert_eq!(posted["comment"]["author"]["kind"], "bot");
    assert_eq!(posted["posted_as"], conversation::BOT_LOGIN);

    // `general.hosting_post_as` names the bot before anything is posted.
    let named = rig_named("conv-gh-bot-named");
    let read = named.json(&["hosting", "issue", "comments", "7"]);
    assert_eq!(read["identity"]["kind"], "app_bot");
    assert_eq!(read["identity"]["login"], "my-bot[bot]");
    let posted = named.json(&["hosting", "issue", "comment", "7", "--body", "hi"]);
    assert_eq!(posted["posted_as"], "my-bot[bot]");
}

fn rig_named(session: &str) -> Rig {
    rig(
        "github_enterprise",
        "owner/repo",
        session,
        "ghs_installation",
        "[general]\nhosting_post_as = \"my-bot[bot]\"\n\n",
    )
}

#[test]
fn what_a_provider_cannot_do_is_refused_before_anything_is_sent() {
    let rig = rig(
        "github_enterprise",
        "owner/repo",
        "conv-gh-refuse",
        "test-token",
        "",
    );
    let message = rig.error(&[
        "hosting",
        "issue",
        "comment",
        "7",
        "--body",
        "x",
        "--reply-to",
        "d1",
    ]);
    assert!(
        message.contains("does not support replying in a thread"),
        "{message}"
    );
    let message = rig.error(&[
        "hosting",
        "issue",
        "comment",
        "7",
        "--body",
        "x",
        "--internal",
    ]);
    assert!(message.contains("internal comments"), "{message}");
    let message = rig.error(&["hosting", "issue", "comment", "ABC-12", "--body", "x"]);
    assert!(message.contains("not an issue number"), "{message}");
    let message = rig.error(&["hosting", "issue", "comment", "7", "--body", "  "]);
    assert!(message.contains("the text is empty"), "{message}");
    assert!(rig.sent("POST").is_empty() && rig.sent("PATCH").is_empty());
}

#[test]
fn a_gitlab_issue_reply_goes_to_the_discussion_notes() {
    let rig = rig(
        "gitlab_self_managed",
        "orbit-co/orbit",
        "conv-gl",
        "test-token",
        "",
    );
    let out = rig.json(&["hosting", "issue", "comments", "31"]);
    assert_eq!(out["item"]["body_flavor"], "gitlab_md");
    let kinds = types(&out);
    assert!(
        kinds.contains(&"comment") && kinds.contains(&"event"),
        "{kinds:?}"
    );
    let discussion = out["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|e| e["discussion_id"].as_str())
        .expect("a discussion id")
        .to_string();
    assert_eq!(discussion, "d_issue1");
    assert_eq!(out["capabilities"]["reply_in_thread"], true);
    assert_eq!(out["capabilities"]["internal_notes"], true);

    let reply = rig.json(&[
        "hosting",
        "issue",
        "comment",
        "31",
        "--body",
        "On it",
        "--reply-to",
        &discussion,
    ]);
    assert_eq!(reply["reply_to"], "d_issue1");
    assert_eq!(reply["posted_as"], "mara");
    let sent = rig.sent("POST");
    let (target, body) = sent.last().unwrap();
    assert_eq!(
        target,
        "/api/v4/projects/orbit-co%2Forbit/issues/31/discussions/d_issue1/notes"
    );
    assert_eq!(
        serde_json::from_str::<Value>(body).unwrap()["body"],
        "On it"
    );

    rig.json(&[
        "hosting",
        "issue",
        "comment",
        "31",
        "--body",
        "private",
        "--internal",
    ]);
    let sent = rig.sent("POST");
    let (target, body) = sent.last().unwrap();
    assert_eq!(target, "/api/v4/projects/orbit-co%2Forbit/issues/31/notes");
    assert_eq!(
        serde_json::from_str::<Value>(body).unwrap()["internal"],
        true
    );
}

#[test]
fn a_github_pull_request_conversation_has_code_threads_and_edits_its_comments() {
    let rig = rig(
        "github_enterprise",
        "owner/repo",
        "conv-gh-pr",
        "test-token",
        "",
    );
    let out = rig.json(&["hosting", "review", "conversation", "9"]);
    assert_eq!(out["item"]["kind"], "pull_request");
    let kinds = types(&out);
    assert_eq!(
        kinds.iter().filter(|k| **k == "thread").count(),
        2,
        "{kinds:?}"
    );
    let thread = out["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["id"] == "PRRT_open")
        .unwrap();
    assert_eq!(thread["path"], "src/planets.rs");
    assert_eq!(thread["resolved"], false);
    assert_eq!(thread["comments"].as_array().unwrap().len(), 2);
    assert_eq!(out["entries"][0]["refs"][0]["text"], "#9");

    let edited = rig.json(&[
        "hosting",
        "review",
        "comment-edit",
        "9",
        "3003",
        "--body",
        "better",
    ]);
    assert_eq!(edited["comment"]["body"], "better");
    assert_eq!(edited["item"]["kind"], "pull_request");
    assert_eq!(
        rig.sent("PATCH")[0].0,
        "/api/v3/repos/owner/repo/issues/comments/3003"
    );
    rig.json(&["hosting", "review", "comment-delete", "9", "3003"]);
    assert_eq!(rig.sent("DELETE").len(), 1);
}

#[test]
fn azure_devops_pull_requests_have_a_conversation_and_review_comment_reaches_it() {
    let mock = mock_host::start(Options::default()).expect("start the mock host");
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        config_for(&mock, "azure_devops_server", "/tfs/Collection", "", "mara"),
    )
    .unwrap();
    let rig = Rig {
        config,
        repo: FixtureRepoBuilder::new()
            .unwrap()
            .commit_file("a.txt", "a\n", "start")
            .unwrap()
            .build(),
        cwd: tempfile::tempdir().unwrap(),
        mock,
        session: "conv-az".to_string(),
        token: "test-token".to_string(),
    };
    let out = rig.project_json(&[
        "hosting",
        "review",
        "conversation",
        "9",
        "--project",
        "Proj/repo",
    ]);
    assert_eq!(out["item"]["title"], "Add retries");
    assert_eq!(out["item"]["reference"], "!9");
    let kinds = types(&out);
    assert!(
        kinds.contains(&"thread") && kinds.contains(&"comment"),
        "{kinds:?}"
    );
    assert!(
        out["entries"].as_array().unwrap().iter().any(|e| e["path"]
            .as_str()
            .is_some_and(|p| p.ends_with("src/net.rs"))),
        "the code thread is there"
    );
    assert_eq!(out["capabilities"]["comment"], true);
    assert_eq!(out["target"]["source"], "project");

    // The review commands had no Azure comment method: they ride on the
    // conversation one, through the capabilities.
    let posted = rig.project_json(&[
        "hosting",
        "review",
        "comment",
        "9",
        "--project",
        "Proj/repo",
        "--body",
        "hello",
    ]);
    assert_eq!(posted["comment"]["id"], "8.1");
    assert_eq!(posted["notified"], false, "no repository, nobody to tell");
    let sent = rig.sent("POST");
    let (target, body) = sent.last().unwrap();
    assert!(
        target.contains("/Proj/_apis/git/repositories/repo-guid-1/pullRequests/9/threads"),
        "{target}"
    );
    assert_eq!(
        serde_json::from_str::<Value>(body).unwrap()["comments"][0]["content"],
        "hello"
    );

    let replied = rig.project_json(&[
        "hosting",
        "review",
        "reply",
        "9",
        "1",
        "--project",
        "Proj/repo",
        "--body",
        "answer",
    ]);
    assert_eq!(replied["reply_to"], "1");
    assert!(rig
        .sent("POST")
        .last()
        .unwrap()
        .0
        .contains("/threads/1/comments"));
}

#[test]
fn issue_comments_can_name_a_project_with_no_repository() {
    let rig = rig(
        "github_enterprise",
        "owner/repo",
        "conv-gh-project",
        "test-token",
        "",
    );
    let out = rig.project_json(&[
        "hosting",
        "issue",
        "comments",
        "7",
        "--project",
        "other/thing",
    ]);
    assert_eq!(out["target"]["source"], "project");
    assert_eq!(out["target"]["project"], "other/thing");
    let posted = rig.project_json(&[
        "hosting",
        "issue",
        "comment",
        "7",
        "--project",
        "other/thing",
        "--body",
        "hi",
    ]);
    assert_eq!(posted["notified"], false);
    assert_eq!(
        rig.sent("POST").last().unwrap().0,
        "/api/v3/repos/other/thing/issues/7/comments"
    );
}

#[test]
fn markdown_cleans_the_text_finds_references_and_wraps() {
    let rig = rig(
        "github_enterprise",
        "owner/repo",
        "conv-markdown",
        "test-token",
        "",
    );
    let out = rig.project_json(&[
        "hosting",
        "markdown",
        "--id",
        "h",
        "--project",
        "owner/repo",
        "--body",
        "see #12 and @mara\u{1b}[31m red\n\nthe quick brown fox",
        "--width",
        "12",
    ]);
    let text = out["text"].as_str().unwrap();
    assert!(!text.contains('\u{1b}'), "{text:?}");
    let refs: Vec<(&str, &str)> = out["refs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| (r["text"].as_str().unwrap(), r["kind"].as_str().unwrap()))
        .collect();
    assert!(refs.contains(&("#12", "web")), "{refs:?}");
    assert!(refs.contains(&("@mara", "mention")), "{refs:?}");
    assert_eq!(out["width"], 12);
    for line in out["lines"].as_array().unwrap() {
        assert!(line.as_str().unwrap().chars().count() <= 12, "{line}");
    }
    assert!(rig.mock.requests().iter().all(|r| r.method != "POST"));
}
