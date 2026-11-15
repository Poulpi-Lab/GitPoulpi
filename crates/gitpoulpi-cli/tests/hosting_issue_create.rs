//! `hosting issue create`: posts to the host the repository's ISSUES link
//! names, refuses when nothing is linked (no request at all), and
//! surfaces the host's refusal. The host is a throwaway local mock (a
//! GitLab) that records method, path and body; assertions are on
//! deserialized JSON.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::sync::{Arc, Mutex};

use assert_cmd::Command;
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::Value;

struct Mock {
    port: u16,
    /// `(method, path, body)` of every request.
    seen: Arc<Mutex<Vec<(String, String, String)>>>,
}

impl Mock {
    fn requests(&self) -> Vec<(String, String, String)> {
        self.seen.lock().unwrap().clone()
    }
}

/// Answers `POST .../issues` with 201 and a created issue, or with
/// `refuse` (status, body) when set.
fn start_mock(refuse: Option<(u16, &'static str)>) -> Mock {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind the mock");
    let port = listener.local_addr().unwrap().port();
    let seen: Arc<Mutex<Vec<(String, String, String)>>> = Arc::default();
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
                log.lock().unwrap().push((method, path, body));
                let (status, json) = refuse.unwrap_or((
                    201,
                    r#"{"iid":34,"web_url":"http://127.0.0.1/team/tool/-/issues/34"}"#,
                ));
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

/// A repository whose `origin` names `g/p` on the mock.
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
    repo
}

fn link_issues(repo: &FixtureRepo) {
    let dir = repo.state_dir();
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("repo-state.toml"),
        "[issues_source]\nintegration = \"corp\"\nproject = \"team/tool\"\n",
    )
    .unwrap();
}

#[test]
fn creating_an_issue_posts_title_body_and_labels_to_the_linked_project() {
    let mock = start_mock(None);
    let repo = fixture(mock.port);
    let cli = Cli::new("hosting-issue-create", mock.port);
    link_issues(&repo);

    let created = cli.json(
        repo.root(),
        &[
            "hosting", "issue", "create", "--title", "Crash", "--body", "Steps", "--label",
            "bug,ui", "--label", "p1",
        ],
    );
    assert_eq!(created["number"], 34);
    assert_eq!(created["web_url"], "http://127.0.0.1/team/tool/-/issues/34");
    assert_eq!(created["project"], "team/tool");

    let requests = mock.requests();
    let (method, path, body) = requests.last().unwrap();
    assert_eq!(method, "POST");
    assert_eq!(path, "/api/v4/projects/team%2Ftool/issues");
    let body: Value = serde_json::from_str(body).unwrap();
    assert_eq!(body["title"], "Crash");
    assert_eq!(body["description"], "Steps");
    assert_eq!(body["labels"], "bug,ui,p1");
}

#[test]
fn web_prints_the_prefilled_new_issue_page_and_calls_no_api() {
    let mock = start_mock(None);
    let repo = fixture(mock.port);
    let cli = Cli::new("hosting-issue-web", mock.port);
    link_issues(&repo);
    let out = cli.json(
        repo.root(),
        &[
            "hosting",
            "issue",
            "create",
            "--title",
            "Crash & burn",
            "--body",
            "Steps\nmore",
            "--label",
            "bug",
            "--web",
        ],
    );
    let url = out["url"].as_str().unwrap();
    assert!(url.contains("/team/tool/-/issues/new?"), "{url}");
    assert!(url.contains("issue%5Btitle%5D=Crash%20%26%20burn"), "{url}");
    assert!(url.contains("Steps%0Amore"), "{url}");
    assert!(
        !mock.requests().iter().any(|(m, _, _)| m == "POST"),
        "no POST"
    );
}

#[test]
fn the_body_can_come_from_a_file() {
    let mock = start_mock(None);
    let repo = fixture(mock.port);
    let cli = Cli::new("hosting-issue-create-file", mock.port);
    link_issues(&repo);
    let file = repo.root().join("body.md");
    std::fs::write(&file, "long\ntext\n").unwrap();
    cli.json(
        repo.root(),
        &[
            "hosting",
            "issue",
            "create",
            "--title",
            "T",
            "--body-file",
            file.to_str().unwrap(),
        ],
    );
    let requests = mock.requests();
    let body: Value = serde_json::from_str(&requests.last().unwrap().2).unwrap();
    assert_eq!(body["description"], "long\ntext\n");
    assert!(body.get("labels").is_none());
}

#[test]
fn an_unlinked_repository_creates_nothing_and_says_how_to_link() {
    let mock = start_mock(None);
    let repo = fixture(mock.port);
    let cli = Cli::new("hosting-issue-create-unlinked", mock.port);
    let message = cli.error(repo.root(), &["hosting", "issue", "create", "--title", "T"]);
    assert!(message.contains("hosting link issues"), "{message}");
    assert!(mock.requests().is_empty(), "no request when unlinked");
}

#[test]
fn a_host_refusal_is_surfaced_with_its_status_and_detail() {
    let mock = start_mock(Some((
        403,
        r#"{"message":"403 Forbidden - not allowed to create issues"}"#,
    )));
    let repo = fixture(mock.port);
    let cli = Cli::new("hosting-issue-create-403", mock.port);
    link_issues(&repo);
    let message = cli.error(repo.root(), &["hosting", "issue", "create", "--title", "T"]);
    assert!(message.contains("opening a GitLab issue"), "{message}");
    assert!(
        message.contains("not allowed to create issues"),
        "{message}"
    );
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();

impl Cli {
    /// Exit code and payload, success or not.
    fn code(&self, cwd: &Path, args: &[&str]) -> (i32, Value) {
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
        let json =
            serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("not JSON ({e}): {stdout}"));
        (output.status.code().unwrap_or(-1), json)
    }
}

#[test]
fn a_repository_that_blocks_assignment_refuses_an_issue_created_with_assignees() {
    let mock = start_mock(None);
    let repo = fixture(mock.port);
    let cli = Cli::new("hosting-issue-assignment-blocked", mock.port);
    link_issues(&repo);

    // Allowed by default.
    assert_eq!(
        cli.json(repo.root(), &["hosting", "assignment"])["assignment"],
        "allowed"
    );

    let shown = cli.json(repo.root(), &["hosting", "assignment", "--block"]);
    assert_eq!(shown["assignment"], "blocked");
    let before = mock.requests().len();
    let (code, json) = cli.code(
        repo.root(),
        &[
            "hosting",
            "issue",
            "create",
            "--title",
            "B",
            "--assignee",
            "kai",
        ],
    );
    assert_eq!(code, 2, "{json}");
    assert_eq!(json["refused"]["reason"], "assignment_blocked", "{json}");
    assert!(
        !mock
            .requests()
            .iter()
            .skip(before)
            .any(|(m, _, _)| m == "POST"),
        "nothing is created when the assignment is refused"
    );

    // Without an assignee the same repository still creates issues.
    let (code, _) = cli.code(repo.root(), &["hosting", "issue", "create", "--title", "C"]);
    assert_eq!(code, 0);

    // Allowing again is shown and persisted.
    let shown = cli.json(repo.root(), &["hosting", "assignment", "--allow"]);
    assert_eq!(shown["assignment"], "allowed");
    assert_eq!(
        cli.json(repo.root(), &["hosting", "assignment"])["assignment"],
        "allowed"
    );
}
