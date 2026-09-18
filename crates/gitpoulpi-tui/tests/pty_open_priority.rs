//! Opening a repository: the graph and the refs come first, the host
//! (pull/merge requests and issues) loads in the background afterwards, and
//! says so while it does. Hosting is opt-in per repo: an unlinked repository
//! never talks to a host.
//!
//! The real binary, the real daemon and real `git`, driven through a pty by
//! `xtask::pty`, against a local mock of a GitLab instance whose answers
//! are slow on purpose (a `TcpListener` in this process, no network beyond
//! 127.0.0.1). The rules are `pty_ui.rs`'s: nothing is located by a
//! hard-coded coordinate and every wait describes the whole state it is
//! waiting for.

mod common;

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const COLS: u16 = 160;
const ROWS: u16 = 60;
/// How long the mock holds every answer back.
const HOST_DELAY: Duration = Duration::from_secs(6);

struct SlowHost {
    port: u16,
    /// Requests received (counted on arrival).
    received: Arc<AtomicUsize>,
    /// Requests answered (counted once the delay has passed).
    answered: Arc<AtomicUsize>,
}

fn start_slow_host() -> SlowHost {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind the mock");
    let port = listener.local_addr().unwrap().port();
    let received = Arc::new(AtomicUsize::new(0));
    let answered = Arc::new(AtomicUsize::new(0));
    let (rec, ans) = (Arc::clone(&received), Arc::clone(&answered));
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let (rec, ans) = (Arc::clone(&rec), Arc::clone(&ans));
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
                rec.fetch_add(1, Ordering::SeqCst);
                let head = String::from_utf8_lossy(&buf).to_string();
                let target = head
                    .lines()
                    .next()
                    .and_then(|l| l.split(' ').nth(1))
                    .unwrap_or_default()
                    .split('?')
                    .next()
                    .unwrap_or_default()
                    .to_string();
                std::thread::sleep(HOST_DELAY);
                let (status, json) = if target.ends_with("/merge_requests") {
                    (
                        200,
                        r#"[{"iid":53,"title":"Add the parser","state":"opened","source_branch":"feature","target_branch":"main","source_project_id":1,"target_project_id":1,"author":{"username":"kai"},"web_url":"http://127.0.0.1/g/p/-/merge_requests/53","updated_at":"2026-09-29T10:00:00Z","assignees":[],"reviewers":[]}]"#,
                    )
                } else if target.ends_with("/issues") {
                    (
                        200,
                        r#"[{"iid":12,"title":"Crash on startup","state":"opened","web_url":"http://127.0.0.1/g/p/-/issues/12","updated_at":"2026-09-29T10:00:00Z","author":{"username":"kai"},"assignees":[],"labels":[]}]"#,
                    )
                } else if target == "/api/v4/user" {
                    (
                        200,
                        r#"{"id":9,"username":"kai","name":"Kai","web_url":"http://127.0.0.1/kai"}"#,
                    )
                } else {
                    (404, r#"{"message":"404 Not Found"}"#)
                };
                let reply = format!(
                    "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{json}",
                    json.len()
                );
                let _ = stream.write_all(reply.as_bytes());
                let _ = stream.flush();
                ans.fetch_add(1, Ordering::SeqCst);
            });
        }
    });
    SlowHost {
        port,
        received,
        answered,
    }
}

fn fixture(port: u16) -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "pty-open-start")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .branch("feature")
        .unwrap()
        .commit_file("f.txt", "f\n", "Add the feature")
        .unwrap()
        .push("feature")
        .unwrap()
        .track("feature", "origin")
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

fn config_toml(port: u16) -> String {
    format!(
        "[general]\nhosting_refresh_minutes = 0\n\n[[integrations]]\nid = \"corp\"\nkind = \"gitlab_self_managed\"\nurl = \"http://127.0.0.1:{port}\"\ntoken_ref = \"integration:corp\"\nname = \"corp\"\nuser = {{ login = \"kai\" }}\n"
    )
}

fn link_both(repo: &FixtureRepo) {
    let dir = repo.state_dir();
    std::fs::create_dir_all(&dir).expect("state dir");
    std::fs::write(
        dir.join("repo-state.toml"),
        "[pulls_source]\nintegration = \"corp\"\nproject = \"g/p\"\n\n[issues_source]\nintegration = \"corp\"\nproject = \"g/p\"\n",
    )
    .expect("seed repo-state.toml");
}

fn launch(repo: &FixtureRepo, port: u16, session: &str) -> (Terminal, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("temp config dir");
    std::fs::write(dir.path().join("config.toml"), config_toml(port)).expect("seed config.toml");
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(dir.path())
        .size(COLS, ROWS)
        .env("GITPOULPI_NO_BROWSER", "1")
        .env("GITPOULPI_TOKEN_CORP", "test-token")
        .spawn();
    (term, dir)
}

/// The graph (and the refs beside it) paint while the host is still
/// holding its answers back, and the sections say they are loading instead
/// of being blank; the lists fill in once the host answers.
///
/// Falsified by making `HostingUi::loading` return `None`: the section
/// never shows "Loading" and the middle wait times out.
#[test]
fn the_graph_paints_first_and_the_host_sections_load_after_it_with_feedback() {
    let host = start_slow_host();
    let repo = fixture(host.port);
    link_both(&repo);
    let (term, _config) = launch(&repo, host.port, "pty-open-priority");

    term.wait_for(
        "the graph and the refs, with the host still silent",
        STARTUP,
        |s| s.contains("Add the feature") && s.contains("pty-open-start"),
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    assert_eq!(
        host.answered.load(Ordering::SeqCst),
        0,
        "the graph appeared only after the host answered"
    );
    assert!(
        !term.screen().contains("REQUESTS (1)"),
        "the pull requests were on screen before the host answered"
    );

    term.wait_for(
        "in-flight feedback in the PULL REQUESTS and ISSUES sections",
        Duration::from_secs(4),
        |s| {
            (s.contains("Loading pull requests") || s.contains("Loading merge requests"))
                && s.contains("Loading issues")
        },
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    assert_eq!(host.answered.load(Ordering::SeqCst), 0);

    term.wait_for(
        "the pull request and the issue, loaded in the background",
        HOST_DELAY * 4,
        |s| s.contains("REQUESTS (1)") && s.contains("ISSUES (1)") && !s.contains("Loading"),
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    term.stop().unwrap();
}

/// Hosting is opt-in per repository: nothing linked, nothing fetched, and
/// no "Loading" anywhere.
#[test]
fn an_unlinked_repository_never_talks_to_the_host() {
    let host = start_slow_host();
    let repo = fixture(host.port);
    let (term, _config) = launch(&repo, host.port, "pty-open-unlinked");
    term.wait_for("the graph", STARTUP, |s| s.contains("Add the feature"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.screen().text()));
    let settled = term
        .wait_for_stable(Duration::from_millis(1500), STARTUP)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!settled.contains("Loading"), "{}", settled.text());
    assert_eq!(
        host.received.load(Ordering::SeqCst),
        0,
        "an unlinked repository reached the host"
    );
    term.stop().unwrap();
}
