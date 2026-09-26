//! Assignees, reviewers, labels and milestones from the CLI (slice M):
//! `hosting members|labels|milestones`, `hosting pr-edit`, `hosting issue
//! edit` and the create flags. The host is a throwaway local GitLab mock
//! that records method, path and body; assertions are on deserialized JSON.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::sync::{Arc, Mutex};

use assert_cmd::Command;
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::Value;

type Seen = Arc<Mutex<Vec<(String, String, String)>>>;

struct Mock {
    port: u16,
    /// `(method, path, body)` of every request.
    seen: Seen,
}

impl Mock {
    fn requests(&self) -> Vec<(String, String, String)> {
        self.seen.lock().unwrap().clone()
    }

    fn sent(&self, method: &str, path_end: &str) -> Value {
        let found = self
            .requests()
            .into_iter()
            .find(|(m, p, _)| m == method && p.split('?').next().unwrap().ends_with(path_end))
            .unwrap_or_else(|| panic!("no {method} {path_end}: {:?}", self.requests()));
        serde_json::from_str(&found.2).unwrap_or(Value::Null)
    }
}

const MEMBERS: &str =
    r#"[{"id":11,"username":"ann","name":"Ann A"},{"id":12,"username":"bob","name":"Bob B"}]"#;
const LABELS: &str = r##"[{"name":"bug","color":"#d9534f","description":"Broken"},{"name":"ui","color":"#428bca"}]"##;
const MILESTONES: &str =
    r#"[{"id":77,"iid":1,"title":"v1","due_date":"2026-12-01","state":"active"}]"#;
const ISSUE: &str =
    r#"{"iid":5,"assignees":[{"username":"ann"}],"labels":["bug"],"milestone":null}"#;
const MR: &str = r#"{"iid":7,"assignees":[],"reviewers":[{"username":"bob"}],"labels":[],
    "milestone":{"id":77,"title":"v1","state":"active"}}"#;

/// Routes on the request line: the first `(method, fragment)` that fits.
fn answer(method: &str, path: &str) -> (u16, String) {
    let path = path.split('?').next().unwrap();
    let table: [(&str, &str, u16, &str); 9] = [
        ("GET", "/members/all", 200, MEMBERS),
        ("GET", "/labels", 200, LABELS),
        ("GET", "/milestones", 200, MILESTONES),
        ("GET", "/issues/5", 200, ISSUE),
        ("PUT", "/issues/5", 200, "{}"),
        ("GET", "/merge_requests/7", 200, MR),
        ("PUT", "/merge_requests/7", 200, "{}"),
        (
            "POST",
            "/issues",
            201,
            r#"{"iid":34,"web_url":"http://127.0.0.1/team/tool/-/issues/34"}"#,
        ),
        ("GET", "/nothing", 404, "{}"),
    ];
    table
        .iter()
        .find(|(m, frag, _, _)| *m == method && path.ends_with(frag))
        .map(|(_, _, status, body)| (*status, body.to_string()))
        .unwrap_or((404, r#"{"message":"404 Not Found"}"#.to_string()))
}

fn start_mock() -> Mock {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind the mock");
    let port = listener.local_addr().unwrap().port();
    let seen: Seen = Arc::default();
    let log = Arc::clone(&seen);
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let log = Arc::clone(&log);
            std::thread::spawn(move || {
                let mut buf = Vec::new();
                let mut chunk = [0u8; 4096];
                let (head_end, content_len) = loop {
                    let n = stream.read(&mut chunk).unwrap_or(0);
                    if n == 0 {
                        return;
                    }
                    buf.extend_from_slice(&chunk[..n]);
                    if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                        let head = String::from_utf8_lossy(&buf[..pos]).to_lowercase();
                        let len = head
                            .lines()
                            .find_map(|l| l.strip_prefix("content-length:"))
                            .and_then(|v| v.trim().parse::<usize>().ok())
                            .unwrap_or(0);
                        break (pos + 4, len);
                    }
                };
                while buf.len() < head_end + content_len {
                    let n = stream.read(&mut chunk).unwrap_or(0);
                    if n == 0 {
                        break;
                    }
                    buf.extend_from_slice(&chunk[..n]);
                }
                let head = String::from_utf8_lossy(&buf[..head_end]).to_string();
                let body = String::from_utf8_lossy(&buf[head_end..]).to_string();
                let line = head.lines().next().unwrap_or_default().to_string();
                let method = line.split(' ').next().unwrap_or_default().to_string();
                let path = line.split(' ').nth(1).unwrap_or_default().to_string();
                let (status, json) = answer(&method, &path);
                log.lock().unwrap().push((method, path, body));
                let reply = format!(
                    "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{json}",
                    json.len()
                );
                let _ = stream.write_all(reply.as_bytes());
            });
        }
    });
    Mock { port, seen }
}

struct Cli {
    config: tempfile::TempDir,
    session: String,
}

impl Cli {
    fn new(session: &str, port: u16) -> Self {
        let config = tempfile::tempdir().expect("temp config dir");
        std::fs::write(
            config.path().join("config.toml"),
            format!(
                "[[integrations]]\nid = \"corp\"\nkind = \"gitlab_self_managed\"\nurl = \"http://127.0.0.1:{port}\"\ntoken_ref = \"integration:corp\"\nname = \"corp\"\nuser = {{ login = \"kai\" }}\n"
            ),
        )
        .unwrap();
        Self {
            config,
            session: session.to_string(),
        }
    }

    fn output(&self, cwd: &Path, args: &[&str]) -> (bool, Value) {
        let output = Command::cargo_bin("gitpoulpi")
            .unwrap()
            .env("GITPOULPI_CONFIG_DIR", self.config.path())
            .env("GITPOULPI_TOKEN_CORP", "test-token")
            .arg("--json")
            .args(["--session", &self.session])
            .args(args)
            .current_dir(cwd)
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

    fn json(&self, cwd: &Path, args: &[&str]) -> Value {
        let (ok, json) = self.output(cwd, args);
        assert!(ok, "`{}` failed: {json}", args.join(" "));
        json
    }

    fn error(&self, cwd: &Path, args: &[&str]) -> String {
        let (ok, json) = self.output(cwd, args);
        assert!(!ok, "`{}` unexpectedly succeeded: {json}", args.join(" "));
        json["error"].as_str().expect("an error string").to_string()
    }
}

/// A repository whose `origin` names `g/p` on the mock, both lists linked
/// to `team/tool`.
fn fixture(port: u16) -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "start")
        .unwrap()
        .with_origin()
        .unwrap()
        .build();
    repo.git(&[
        "remote",
        "set-url",
        "origin",
        &format!("http://127.0.0.1:{port}/g/p.git"),
    ]);
    let dir = repo.state_dir();
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("repo-state.toml"),
        "[issues_source]\nintegration = \"corp\"\nproject = \"team/tool\"\n\n[pulls_source]\nintegration = \"corp\"\nproject = \"team/tool\"\n",
    )
    .unwrap();
    repo
}

#[test]
fn members_labels_and_milestones_are_listed_as_json() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let cli = Cli::new("hosting-meta-lists", mock.port);

    let members = cli.json(repo.root(), &["hosting", "members", "--query", "an"]);
    assert_eq!(members["members"][0]["login"], "ann");
    assert_eq!(members["members"][1]["id"], "12");

    let labels = cli.json(repo.root(), &["hosting", "labels", "--query", "BU"]);
    let labels = labels["labels"].as_array().unwrap();
    assert_eq!(labels.len(), 1, "filtered by the query: {labels:?}");
    assert_eq!(labels[0]["name"], "bug");
    assert_eq!(labels[0]["color"], "d9534f");

    let milestones = cli.json(
        repo.root(),
        &["hosting", "milestones", "--source", "issues"],
    );
    assert_eq!(milestones["milestones"][0]["title"], "v1");
    assert_eq!(milestones["milestones"][0]["id"], 77);
    assert_eq!(milestones["project"], "team/tool");

    let members_call = mock
        .requests()
        .into_iter()
        .find(|(_, p, _)| p.contains("/members/all"))
        .unwrap();
    assert!(members_call.1.contains("query=an"), "{}", members_call.1);
}

#[test]
fn editing_a_merge_request_adds_and_removes_and_reports_the_result() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let cli = Cli::new("hosting-meta-pr-edit", mock.port);

    let out = cli.json(
        repo.root(),
        &[
            "hosting",
            "pr-edit",
            "7",
            "--add-assignee",
            "@ann",
            "--remove-reviewer",
            "bob",
            "--add-reviewer",
            "ann",
            "--add-label",
            "bug,ui",
            "--no-milestone",
        ],
    );
    assert_eq!(out["number"], 7);
    assert_eq!(out["project"], "team/tool");
    assert!(out["reviewers"].is_array());

    let body = mock.sent("PUT", "/merge_requests/7");
    assert_eq!(body["assignee_ids"], serde_json::json!([11]));
    assert_eq!(body["reviewer_ids"], serde_json::json!([11]), "bob removed");
    assert_eq!(body["labels"], "bug,ui");
    assert_eq!(body["milestone_id"], 0);
}

#[test]
fn editing_an_issue_resolves_the_milestone_title() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let cli = Cli::new("hosting-meta-issue-edit", mock.port);

    let out = cli.json(
        repo.root(),
        &[
            "hosting",
            "issue",
            "edit",
            "5",
            "--remove-label",
            "bug",
            "--milestone",
            "V1",
        ],
    );
    assert_eq!(out["number"], 5);
    assert_eq!(out["assignees"], serde_json::json!(["ann"]));

    let body = mock.sent("PUT", "/issues/5");
    assert_eq!(body["labels"], "");
    assert_eq!(body["milestone_id"], 77);
    assert!(body.get("assignee_ids").is_none(), "untouched: {body}");
}

#[test]
fn an_unknown_milestone_lists_the_open_ones_and_sends_nothing() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let cli = Cli::new("hosting-meta-bad-milestone", mock.port);
    let message = cli.error(
        repo.root(),
        &["hosting", "issue", "edit", "5", "--milestone", "nope"],
    );
    assert!(message.contains("v1"), "{message}");
    assert!(mock.requests().iter().all(|(m, _, _)| m != "PUT"));
}

#[test]
fn an_edit_with_nothing_to_change_says_so() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let cli = Cli::new("hosting-meta-empty", mock.port);
    let message = cli.error(repo.root(), &["hosting", "issue", "edit", "5"]);
    assert!(message.contains("nothing to change"), "{message}");
    assert!(mock.requests().is_empty());
}

#[test]
fn issue_create_takes_assignees_and_a_milestone() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let cli = Cli::new("hosting-meta-create", mock.port);
    let created = cli.json(
        repo.root(),
        &[
            "hosting",
            "issue",
            "create",
            "--title",
            "Crash",
            "--assignee",
            "bob",
            "--milestone",
            "v1",
            "--label",
            "bug",
        ],
    );
    assert_eq!(created["number"], 34);
    let body = mock.sent("POST", "/issues");
    assert_eq!(body["assignee_ids"], serde_json::json!([12]));
    assert_eq!(body["milestone_id"], 77);
    assert_eq!(body["labels"], "bug");
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
