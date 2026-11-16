#![cfg(all(feature = "radar", feature = "review"))]
//! The review's Overview is the same conversation an issue has: the shared
//! timeline with its docked composer, the same keys (`r` reply, `Q` quote,
//! `m` load older), the same writes -- the real binary and daemon, driven
//! through a pty by `xtask::pty`, against `xtask mock-host` (a `TcpListener`
//! on 127.0.0.1; no other network).
//!
//! One GitLab profile (`lab`) with its review requests on: the review of
//! `!58` of `northwind/billing` is opened over the Radar through the API (the
//! repository is cloned nowhere). The mock pretends the merge request has
//! four pages of discussions: the newest window shows one remark, `m` reads
//! the first page (the code review's own discussions).

#![allow(dead_code)]

#[path = "common/mod.rs"]
mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::mock_host::{self, MockHost, Options};
use xtask::pty::{Screen, Terminal};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const COLS: u16 = 160;
const ROWS: u16 = 50;
const ROW: &str = "Round invoice totals";
/// What the mock's newest discussion says.
const NEWEST: &str = "Newest remark from the mock.";
/// What the mock's first page of discussions says (a plain comment).
const OLDER: &str = "Looks promising, one question inline.";

struct Setup {
    term: Terminal,
    mock: MockHost,
    port_file: std::path::PathBuf,
    _config: tempfile::TempDir,
    _repo: FixtureRepo,
}

fn setup(session: &str) -> Setup {
    let mock = mock_host::start(Options {
        port: 0,
        bare_root: None,
    })
    .expect("the mock host");
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "review-overview-start-commit")
        .unwrap()
        .build();
    let config = tempfile::tempdir().expect("temp config dir");
    std::fs::write(
        config.path().join("config.toml"),
        format!(
            "[[integrations]]\nid = \"lab\"\nkind = \"gitlab_self_managed\"\nurl = \"http://127.0.0.1:{port}\"\ntoken_ref = \"integration:lab\"\nname = \"lab\"\nuser = {{ login = \"{login}\" }}\n\n[integrations.radar]\nreview_requests = true\n",
            port = mock.port,
            login = mock_host::GITLAB_LOGIN,
        ),
    )
    .expect("seed config.toml");
    std::fs::create_dir_all(config.path().join("sessions")).unwrap();
    std::fs::write(
        config
            .path()
            .join("sessions")
            .join(format!("{session}.toml")),
        format!(
            "[[tabs]]\nrepo_path = '{}'\nactive = true\n",
            repo.root().display()
        ),
    )
    .expect("seed the session");
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(COLS, ROWS)
        .env("GITPOULPI_NO_BROWSER", "1")
        .env("GITPOULPI_TOKEN_LAB", "test-token")
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .spawn();
    term.wait_for("the repository loaded", STARTUP, |s| {
        s.contains("review-overview-start-commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    Setup {
        term,
        mock,
        port_file: config
            .path()
            .join("sessions")
            .join(format!("{session}.port")),
        _config: config,
        _repo: repo,
    }
}

fn wait_until(what: &str, mut ok: impl FnMut() -> bool) {
    let deadline = std::time::Instant::now() + STEP;
    while !ok() {
        assert!(std::time::Instant::now() < deadline, "timed out: {what}");
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// The review of `!58` over the Radar, on its Overview with the newest
/// window of the conversation in.
fn open_review(s: &mut Setup) -> Screen {
    s.term.send_keys("M-r").expect("Alt+R");
    let screen = s
        .term
        .wait_for("the far request", STARTUP, |s| {
            s.contains("Radar") && s.contains(ROW) && s.contains("Not cloned")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = screen.find(ROW).unwrap();
    s.term.click(col + 2, row).expect("select the row");
    s.term.send_keys("e").expect("e");
    s.term
        .wait_for("the question", STEP, |s| {
            s.contains("Review through the API")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("a").expect("a");
    s.term
        .wait_for("the Overview's timeline", STEP, |s| {
            s.contains("Review !58") && s.contains(NEWEST) && s.contains("New comment")
        })
        .unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn the_overview_reads_replies_quotes_and_loads_older_like_an_issue() {
    let mut s = setup("pty-review-overview");
    let screen = open_review(&mut s);
    // Read: the description is the timeline's first entry, the newest window
    // holds the latest remark, the composer is docked under it, and the
    // older page has not been read.
    assert!(
        screen.contains("Adds the moons column"),
        "{}",
        screen.text()
    );
    assert!(screen.contains("Write a comment"), "{}", screen.text());
    assert!(!screen.contains(OLDER), "not yet:\n{}", screen.text());

    // Down: the newest remark; `r` replies into its discussion.
    s.term.send_keys("Down").expect("select the remark");
    s.term.send_keys("r").expect("reply");
    s.term
        .wait_for("the reply box", STEP, |s| s.contains("Reply to tomas"))
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_text("agreed").unwrap();
    s.term.send_keys("C-s").expect("send");
    wait_until("the host got the reply", || {
        s.mock
            .saw("POST", "/merge_requests/58/discussions/d_late/notes")
    });
    let reply = s
        .mock
        .requests()
        .into_iter()
        .find(|r| r.method == "POST" && r.target.ends_with("/discussions/d_late/notes"))
        .expect("a reply");
    assert!(reply.body.contains("agreed"), "{}", reply.body);
    s.term
        .wait_for("the reply in the timeline", STEP, |s| {
            s.contains("agreed") && !s.contains("Reply to tomas")
        })
        .unwrap_or_else(|e| panic!("{e}"));

    // `Q`: the selected entry quoted into a new comment, sent as one.
    s.term.send_keys("Q").expect("quote");
    s.term
        .wait_for("the quote in the box", STEP, |s| {
            s.contains("> **tomas**") || s.contains("> Newest remark")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_text("same here").unwrap();
    s.term.send_keys("C-s").expect("send");
    wait_until("the host got the quote", || {
        s.mock.requests().iter().any(|r| {
            r.method == "POST"
                && r.target.ends_with("/merge_requests/58/notes")
                && r.body.contains("> ")
                && r.body.contains("same here")
        })
    });

    // `m`: the first page of discussions.
    // The host answered: the comment is in the timeline, the box is empty
    // and the keyboard is back on the timeline (a key typed while sending
    // would go into the box).
    s.term
        .wait_for("the quote sent", STEP, |s| {
            s.contains("same here")
                && !s.contains("Sending...")
                && s.contains("Write a comment (Markdown)...")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("m").expect("load older");
    s.term
        .wait_for("the older discussions", STEP, |s| s.contains(OLDER))
        .unwrap_or_else(|e| panic!("{e}"));
}

/// A write of the TUI reaches the other windows of the session: another IPC
/// client sees `HostingItemChanged` for the request with the writer's origin,
/// and the writing window does not read the request a second time because of
/// its own echo (the number of reads of the request stays what it was).
#[test]
fn a_comment_posted_in_the_overview_is_announced_to_the_other_windows() {
    use gitpoulpi_ipc::{Client, Event, HostingChange, HostingItemKind, ServerMessage};

    let mut s = setup("pty-review-overview-notify");
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut watcher = rt.block_on(async {
        Client::connect(
            &gitpoulpi_ipc::discovery::endpoint(&s.port_file)
                .expect("the session published its discovery files"),
        )
        .await
        .expect("connect")
        .into_stream()
    });
    open_review(&mut s);
    let reads_of_the_request = |s: &Setup| {
        s.mock
            .requests()
            .iter()
            .filter(|r| {
                r.method == "GET"
                    && r.target
                        .split('?')
                        .next()
                        .unwrap()
                        .ends_with("/merge_requests/58")
            })
            .count()
    };
    let before = reads_of_the_request(&s);
    s.term.send_keys("c").expect("new comment");
    s.term
        .wait_for("the box", STEP, |s| s.contains("Write a comment"))
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_text("hello from the first window").unwrap();
    s.term.send_keys("C-s").expect("send");
    let (_, change) = rt
        .block_on(async {
            tokio::time::timeout(STEP, async {
                loop {
                    let message: ServerMessage =
                        gitpoulpi_ipc::transport::read_message(&mut watcher)
                            .await
                            .unwrap()
                            .expect("the connection closed");
                    if let ServerMessage::Event(Event::HostingItemChanged { repo, change }) =
                        message
                    {
                        return (repo, change);
                    }
                }
            })
            .await
        })
        .expect("timed out waiting for HostingItemChanged");
    assert_eq!(change.change, HostingChange::Posted);
    assert_eq!(change.kind, HostingItemKind::PullRequest);
    assert_eq!(change.number, mock_host::review::GITLAB_MR);
    assert_eq!(change.project, "northwind/billing");
    assert_eq!(change.integration_id, "lab");
    assert!(change.origin.starts_with("tui-"), "{}", change.origin);
    // Give an echo time to cause a read, then check it did not.
    std::thread::sleep(Duration::from_millis(2500));
    assert_eq!(
        reads_of_the_request(&s),
        before,
        "the writing window read the request again for its own write"
    );
}
