//! `hosting prs` / `issues` / `open` respect the repository's per-repo link
//! (`pulls_source` / `issues_source` in `.gitpoulpi/repo-state.toml`, the
//! one the TUI's sidebar sections save): nothing linked means nothing is
//! fetched and the error says how to link; a link decides the profile and
//! the project, whatever the remote says; `hosting link` sets, shows and
//! clears it.
//!
//! The host is a throwaway local mock (a GitLab), its request log is the
//! proof that "no fetch" really means no request. Assertions are on
//! deserialized JSON. Own config dir and fixture repository per test.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::sync::{Arc, Mutex};

use assert_cmd::Command;
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::Value;

struct Mock {
    port: u16,
    seen: Arc<Mutex<Vec<String>>>,
}

impl Mock {
    fn requests(&self) -> Vec<String> {
        self.seen.lock().unwrap().clone()
    }
}

fn start_mock() -> Mock {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind the mock");
    let port = listener.local_addr().unwrap().port();
    let seen: Arc<Mutex<Vec<String>>> = Arc::default();
    let log = Arc::clone(&seen);
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let log = Arc::clone(&log);
            std::thread::spawn(move || {
                let mut buf = Vec::new();
                let mut chunk = [0u8; 4096];
                loop {
                    let n = stream.read(&mut chunk).unwrap_or(0);
                    if n == 0 {
                        return;
                    }
                    buf.extend_from_slice(&chunk[..n]);
                    if buf.windows(4).any(|w| w == b"\r\n\r\n") {
                        break;
                    }
                }
                let head = String::from_utf8_lossy(&buf).to_string();
                let line = head.lines().next().unwrap_or_default().to_string();
                let target = line.split(' ').nth(1).unwrap_or_default().to_string();
                let path = target.split('?').next().unwrap_or_default().to_string();
                log.lock().unwrap().push(target.clone());
                let json = if path.ends_with("/merge_requests") {
                    r#"[{"iid":53,"title":"Add the parser","state":"opened","source_branch":"feature",
                        "target_branch":"main","source_project_id":1,"target_project_id":1,
                        "author":{"username":"kai"},"web_url":"http://127.0.0.1/x/-/merge_requests/53",
                        "updated_at":"2026-09-29T10:00:00Z","assignees":[],"reviewers":[]}]"#
                } else if path.ends_with("/issues") {
                    r#"[{"iid":12,"title":"Crash on startup","state":"opened",
                        "web_url":"http://127.0.0.1/x/-/issues/12","updated_at":"2026-09-29T10:00:00Z",
                        "author":{"username":"kai"},"assignees":[],"labels":[]}]"#
                } else {
                    "[]"
                };
                let reply = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{json}",
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

fn state(repo: &FixtureRepo) -> String {
    std::fs::read_to_string(repo.state_dir().join("repo-state.toml")).unwrap_or_default()
}

#[test]
fn an_unlinked_repository_fetches_nothing_and_says_how_to_link() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let cli = Cli::new("hosting-link-unlinked", mock.port);

    for (args, noun, link) in [
        (
            vec!["hosting", "prs"],
            "pull/merge request",
            "hosting link pulls",
        ),
        (
            vec!["hosting", "pr-for-branch", "main"],
            "pull/merge request",
            "hosting link pulls",
        ),
        (vec!["hosting", "issues"], "issue", "hosting link issues"),
        (
            vec!["hosting", "open", "issue", "12"],
            "issue",
            "hosting link issues",
        ),
    ] {
        let message = cli.error(repo.root(), &args);
        assert!(
            message.contains(&format!("no {noun} source linked"))
                && message.contains("nothing is fetched")
                && message.contains(link),
            "{args:?}: {message}"
        );
    }
    assert!(mock.requests().is_empty(), "{:?}", mock.requests());

    // `hosting link` with no flag shows the (absent) link.
    let shown = cli.json(repo.root(), &["hosting", "link", "pulls"]);
    assert_eq!(shown["linked"], Value::Bool(false));
    assert_eq!(shown["integration"], Value::Null);
}

#[test]
fn a_link_decides_the_profile_and_project_and_clearing_it_stops_the_fetch() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let cli = Cli::new("hosting-link-linked", mock.port);

    // Link the pull requests to a project the remote does NOT name.
    let linked = cli.json(
        repo.root(),
        &[
            "hosting",
            "link",
            "pulls",
            "--id",
            "corp",
            "--project",
            "other/proj",
        ],
    );
    assert_eq!(linked["linked"], Value::Bool(true));
    assert_eq!(linked["project"], "other/proj");
    assert_eq!(linked["changed"], Value::Bool(true));
    let saved = state(&repo);
    assert!(
        saved.contains("[pulls_source]")
            && saved.contains("integration = \"corp\"")
            && saved.contains("project = \"other/proj\""),
        "{saved}"
    );

    let prs = cli.json(repo.root(), &["hosting", "prs"]);
    assert_eq!(prs["integration"], "corp");
    assert_eq!(prs["project"], "other/proj");
    assert_eq!(prs["source"], "link");
    assert_eq!(prs["pull_requests"][0]["iid_label"], "!53");
    let requests = mock.requests();
    assert!(
        requests.iter().any(|r| r.contains("other%2Fproj")),
        "the link's project was fetched, not the remote's: {requests:?}"
    );
    assert!(
        !requests.iter().any(|r| r.contains("g%2Fp")),
        "{requests:?}"
    );

    // `open` follows the same link; issues are still unlinked.
    let opened = cli.json(repo.root(), &["hosting", "open", "pr", "53"]);
    assert_eq!(opened["project"], "other/proj");
    assert!(opened["url"]
        .as_str()
        .unwrap()
        .ends_with("other/proj/-/merge_requests/53"));
    let before = mock.requests().len();
    let message = cli.error(repo.root(), &["hosting", "issues"]);
    assert!(message.contains("no issue source linked"), "{message}");
    assert_eq!(mock.requests().len(), before);

    // Unlink: nothing is fetched again, the state forgets it.
    let cleared = cli.json(repo.root(), &["hosting", "link", "pulls", "--clear"]);
    assert_eq!(cleared["linked"], Value::Bool(false));
    assert!(!state(&repo).contains("pulls_source"), "{}", state(&repo));
    let message = cli.error(repo.root(), &["hosting", "prs"]);
    assert!(message.contains("nothing is fetched"), "{message}");
    assert_eq!(mock.requests().len(), before, "no request after unlinking");
}

#[test]
fn a_link_the_tui_saved_is_honoured_and_link_defaults_the_project_from_the_remote() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let cli = Cli::new("hosting-link-tui-state", mock.port);

    // What the TUI's ISSUES section writes.
    let dir = repo.state_dir();
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("repo-state.toml"),
        "[issues_source]\nintegration = \"corp\"\nproject = \"team/tool\"\n",
    )
    .unwrap();
    let issues = cli.json(repo.root(), &["hosting", "issues"]);
    assert_eq!(issues["project"], "team/tool");
    assert_eq!(issues["source"], "link");
    assert_eq!(issues["issues"][0]["iid_label"], "#12");

    // Pull requests are a separate link: still unlinked.
    let message = cli.error(repo.root(), &["hosting", "prs"]);
    assert!(message.contains("no pull/merge request source linked"));

    // No --project: the project the remote names on that profile.
    let linked = cli.json(repo.root(), &["hosting", "link", "pulls", "--id", "corp"]);
    assert_eq!(linked["project"], "g/p");

    // An explicit --id stays a one-off override that resolves by remote.
    cli.json(repo.root(), &["hosting", "link", "issues", "--clear"]);
    let by_id = cli.json(repo.root(), &["hosting", "issues", "--id", "corp"]);
    assert_eq!(by_id["project"], "g/p");
    assert_eq!(by_id["source"], "remote");
}

#[test]
fn edit_url_follows_the_link_for_a_number_and_for_a_branch() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let cli = Cli::new("hosting-edit-url", mock.port);

    // Unlinked: nothing is resolved, like `prs`.
    let message = cli.error(repo.root(), &["hosting", "edit-url", "53"]);
    assert!(
        message.contains("no pull/merge request source linked"),
        "{message}"
    );

    cli.json(
        repo.root(),
        &[
            "hosting",
            "link",
            "pulls",
            "--id",
            "corp",
            "--project",
            "other/proj",
        ],
    );
    let by_number = cli.json(repo.root(), &["hosting", "edit-url", "53"]);
    assert_eq!(by_number["project"], "other/proj");
    assert_eq!(by_number["number"], 53);
    assert_eq!(by_number["opened_in_browser"], Value::Bool(false));
    assert!(
        by_number["url"]
            .as_str()
            .unwrap()
            .ends_with("other/proj/-/merge_requests/53/edit"),
        "{by_number}"
    );

    let by_branch = cli.json(repo.root(), &["hosting", "edit-url", "--branch", "feature"]);
    assert_eq!(
        by_branch["url"],
        "http://127.0.0.1/x/-/merge_requests/53/edit"
    );
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
