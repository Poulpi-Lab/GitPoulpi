#![cfg(all(feature = "radar", feature = "review"))]
//! The Radar's row actions on the host -- Merge, Close, Mark as ready, Clone --
//! the branches shown on a request row, the repository on an issue row, and
//! the review through the local clone. The real binary and daemon, driven
//! through a pty by `xtask::pty`, against `xtask mock-host` (a `TcpListener`
//! on 127.0.0.1; no other network).
//!
//! One GitLab profile (`lab`). With `open_requests` the mock lists `!52` of
//! `northwind/ledger-ui` (source `feature/52`, target `main`), `!58` of
//! `northwind/billing` and the draft `!6` of `northwind/infra-notes`, in
//! repositories that exist nowhere on this machine, so their rows are not
//! cloned. The mock keeps the writes: a merged or closed request leaves its
//! lists, a request whose title lost `Draft:` is no draft any more.

#![allow(dead_code)]

#[path = "common/mod.rs"]
mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::mock_host::{self, MockHost, Options};
use xtask::pty::{Screen, Terminal};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const COLS: u16 = 160;
const ROWS: u16 = 50;
const GRID: &str = "Keyboard shortcuts for the grid";
const BILLING: &str = "Round invoice totals";
const RUNBOOK: &str = "runbook for restores";
const ISSUE: &str = "Ledger export drops the last row";

struct Setup {
    term: Terminal,
    mock: MockHost,
    config: tempfile::TempDir,
    clone_root: tempfile::TempDir,
    _bare: tempfile::TempDir,
    repo: FixtureRepo,
}

/// The mock, a `lab` profile with `radar` as its extra lines, and the TUI on
/// a fixture repository (the only tab). `origin` is the project the fixture's
/// `origin` remote names, when it is a clone of one.
fn setup(session: &str, radar: &str, origin: Option<&str>, cols: u16) -> Setup {
    let bare = tempfile::tempdir().unwrap();
    mock_host::seed_bare_repos(bare.path()).unwrap();
    let mock = mock_host::start(Options {
        port: 0,
        bare_root: Some(bare.path().to_path_buf()),
    })
    .expect("the mock host");
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "radar-acts-start-commit")
        .unwrap()
        .build();
    if let Some(project) = origin {
        repo.git(&[
            "remote",
            "add",
            "origin",
            &format!("http://127.0.0.1:{}/{project}.git", mock.port),
        ]);
    }
    let clone_root = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().expect("temp config dir");
    std::fs::write(
        config.path().join("config.toml"),
        format!(
            "[repos]\nclone_root = '{root}'\n\n[[integrations]]\nid = \"lab\"\nkind = \"gitlab_self_managed\"\nurl = \"http://127.0.0.1:{port}\"\ntoken_ref = \"integration:lab\"\nname = \"lab\"\nuser = {{ login = \"{login}\" }}\n\n[integrations.radar]\n{radar}\n",
            root = clone_root.path().display(),
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
        .size(cols, ROWS)
        .mouse_capture()
        .env("GITPOULPI_NO_BROWSER", "1")
        .env("GITPOULPI_OPEN_LOG", config.path().join("opened.log"))
        .env("GITPOULPI_TOKEN_LAB", "test-token")
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .spawn();
    term.wait_for("the repository loaded", STARTUP, |s| {
        s.contains("radar-acts-")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    Setup {
        term,
        mock,
        config,
        clone_root,
        _bare: bare,
        repo,
    }
}

const OPEN: &str = "open_requests = true\ninclude_drafts = true\nassigned_issues = true\n";

/// The Radar with `text` listed.
fn open_radar(term: &mut Terminal, text: &str) -> Screen {
    term.send_keys("M-r").expect("Alt+R");
    term.wait_for(text, STARTUP, |s| s.contains("Radar") && s.contains(text))
        .unwrap_or_else(|e| panic!("{e}"))
}

/// The Radar with `text` listed and its row selected.
fn select(term: &mut Terminal, text: &str) -> Screen {
    let screen = open_radar(term, text);
    let (col, row) = screen.find(text).unwrap();
    term.click(col + 2, row).expect("select the row");
    screen
}

fn line_of(screen: &Screen, text: &str) -> String {
    let (_, row) = screen
        .find(text)
        .unwrap_or_else(|| panic!("{text}:\n{}", screen.text()));
    screen.line(row)
}

fn wait_until(what: &str, mut ok: impl FnMut() -> bool) {
    let deadline = std::time::Instant::now() + STEP;
    while !ok() {
        assert!(std::time::Instant::now() < deadline, "timed out: {what}");
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

/// A request row says `source -> target` after its title and an issue row
/// names its repository, like the request rows do.
#[test]
fn a_request_row_shows_its_branches_and_an_issue_row_its_repository() {
    let mut s = setup("pty-acts-rows", OPEN, None, COLS);
    let screen = open_radar(&mut s.term, ISSUE);
    let request = line_of(&screen, GRID);
    assert!(
        request.contains("feature/52 \u{2192} main"),
        "branches after the title: {request}"
    );
    assert!(request.contains("northwind/ledger-ui"), "{request}");
    let issue = line_of(&screen, ISSUE);
    assert!(issue.contains("northwind/ledger-ui"), "{issue}");
    // The title stays whole, the branches follow it.
    assert!(
        request.find(GRID).unwrap() < request.find("feature/52").unwrap(),
        "{request}"
    );
}

/// Narrow, the branches are cut (never the Actions column), and below the
/// room they need they are not painted at all.
#[test]
fn the_branches_give_way_when_the_window_narrows() {
    let mut s = setup("pty-acts-narrow", OPEN, None, 100);
    let screen = open_radar(&mut s.term, "Keyboard");
    let line = line_of(&screen, "Keyboard");
    assert!(
        line.contains('\u{22EE}'),
        "the row menu button is still there: {line}"
    );
}

/// Merge asks first (naming the request, its branches and the method), then
/// calls the host, and the row leaves the list.
#[test]
fn merge_asks_confirmation_then_merges_on_the_host_and_the_row_goes() {
    let mut s = setup("pty-acts-merge", OPEN, None, COLS);
    select(&mut s.term, GRID);
    s.term.send_keys("m").expect("m");
    let screen = s
        .term
        .wait_for("the confirmation", STEP, |s| {
            s.contains("feature/52 \u{2192} main") && s.contains("Squash and")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(screen.contains("!52"), "{}", screen.text());
    assert!(screen.contains("cannot be undone"), "{}", screen.text());
    assert!(!s.mock.saw("PUT", "/merge"), "nothing sent yet");
    // Esc cancels: nothing is sent.
    s.term.send_keys("Escape").expect("Esc");
    s.term
        .wait_for("the dialog closed", STEP, |s| !s.contains("Squash and"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!s.mock.saw("PUT", "/merge"));
    s.term.send_keys("m").expect("m again");
    s.term
        .wait_for("the confirmation", STEP, |s| s.contains("Squash and"))
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("Down").expect("squash");
    s.term.send_keys("Enter").expect("confirm");
    s.term
        .wait_for("the row gone", STEP, |s| !s.contains(GRID))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(s.mock.saw("PUT", "/merge_requests/52/merge"));
}

/// Close asks first, then closes on the host.
#[test]
fn close_asks_confirmation_then_closes_on_the_host() {
    let mut s = setup("pty-acts-close", OPEN, None, COLS);
    select(&mut s.term, GRID);
    s.term.send_keys("x").expect("x");
    let screen = s
        .term
        .wait_for("the confirmation", STEP, |s| s.contains("Close it"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(screen.contains("without a"), "{}", screen.text());
    assert!(!s.mock.saw("PUT", "/merge_requests/52"));
    s.term.send_keys("Enter").expect("confirm");
    s.term
        .wait_for("the row gone", STEP, |s| !s.contains(GRID))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(s.mock.saw("PUT", "/merge_requests/52"));
}

/// Mark as ready on a draft: no question (it is reversible on the host), the
/// host drops the draft marker and the row loses its chip.
#[test]
fn mark_as_ready_takes_a_draft_out_of_draft() {
    let mut s = setup("pty-acts-ready", OPEN, None, COLS);
    open_radar(&mut s.term, GRID);
    // The Drafts checkbox is a local filter.
    s.term.send_keys("d").expect("d");
    let screen = s
        .term
        .wait_for("the draft", STEP, |s| s.contains(RUNBOOK))
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = screen.find(RUNBOOK).unwrap();
    s.term.click(col + 2, row).expect("select the row");
    assert!(line_of(&screen, RUNBOOK).contains("draft"));
    s.term.send_keys("y").expect("y");
    s.term
        .wait_for("the chip gone", STEP, |s| {
            s.contains(RUNBOOK)
                && !s
                    .find(RUNBOOK)
                    .is_some_and(|(_, row)| s.line(row).contains("draft"))
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(s.mock.saw("PUT", "/merge_requests/6"));
}

/// A request that is not a draft refuses Mark as ready with its reason.
#[test]
fn mark_as_ready_on_a_request_that_is_not_a_draft_says_why() {
    let mut s = setup("pty-acts-notdraft", OPEN, None, COLS);
    select(&mut s.term, GRID);
    s.term.send_keys("y").expect("y");
    s.term
        .wait_for("the reason", STEP, |s| s.contains("It is not a draft"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!s.mock.saw("PUT", "/merge_requests/6"));
}

/// The palette reaches Merge with the row selected.
#[test]
fn the_palette_reaches_merge_and_close() {
    let mut s = setup("pty-acts-palette", OPEN, None, COLS);
    select(&mut s.term, GRID);
    s.term.send_keys("C-p").expect("the palette");
    s.term
        .wait_for("the palette", STEP, |s| s.contains("Command Palette"))
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_text("Radar: Merge").unwrap();
    s.term.send_keys("Enter").expect("run");
    s.term
        .wait_for("the confirmation", STEP, |s| s.contains("Squash and"))
        .unwrap_or_else(|e| panic!("{e}"));
}

/// Clone on a row that is not cloned opens the Clone dialog on its project;
/// once cloned the row says so and Clone refuses.
#[test]
fn clone_opens_the_clone_dialog_and_the_radar_finds_the_clone() {
    let mut s = setup("pty-acts-clone", OPEN, None, COLS);
    select(&mut s.term, BILLING);
    s.term.send_keys("g").expect("g");
    s.term
        .wait_for("the Clone dialog", STEP, |s| {
            s.contains("Clone a repository") && s.contains("northwind/billing")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("Enter").expect("clone");
    wait_until("the clone on disk", || {
        s.clone_root.path().join("billing").join(".git").exists()
    });
    s.term
        .wait_for("the clone's tab", STARTUP, |s| s.contains("/billing -"))
        .unwrap_or_else(|e| panic!("{e}"));
    // The mock clones from a local path; a real clone's remote names the
    // host, which is what the Radar places a row by.
    let url = format!("http://127.0.0.1:{}/northwind/billing.git", s.mock.port);
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(s.clone_root.path().join("billing"))
        .args(["remote", "set-url", "origin", &url])
        .output()
        .expect("git");
    assert!(out.status.success(), "{out:?}");
    s.term.send_keys("M-r").expect("Alt+R");
    // The row is placed on the clone once the Radar has read its remote.
    let screen = s
        .term
        .wait_for("the row on the clone", STARTUP, |s| {
            s.find(BILLING)
                .is_some_and(|(_, row)| !s.line(row).contains("northwind/billing"))
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = screen.find(BILLING).unwrap();
    s.term.click(col + 2, row).expect("select the row");
    s.term.send_keys("g").expect("g again");
    s.term
        .wait_for("already cloned", STEP, |s| s.contains("Already cloned at"))
        .unwrap_or_else(|e| panic!("{e}"));
}

/// Review through the local clone: the repository's tab opens AND the review
/// of the request shows. The clone is known (it is the fixture's `origin`)
/// but was never linked to the host, so the window has to link it first.
#[test]
fn review_through_the_local_clone_opens_the_tab_and_the_review() {
    let radar = "review_requests = true\nreview_mode = \"clone\"\n";
    let mut s = setup(
        "pty-acts-local-review",
        radar,
        Some("northwind/billing"),
        COLS,
    );
    let screen = select(&mut s.term, BILLING);
    // The chip is the clone's folder name, not the project path.
    assert!(
        !line_of(&screen, BILLING).contains("northwind/billing"),
        "the row is placed on the clone: {}",
        screen.text()
    );
    s.term.send_keys("e").expect("e");
    s.term
        .wait_for("the review in the clone's tab", STARTUP, |s| {
            s.contains("Review !58") && s.contains("Add moons to the planet table")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let state = s
        .repo
        .root()
        .join(".git")
        .join("gitpoulpi")
        .join("repo-state.toml");
    let written = read(&state);
    assert!(
        written.contains("project = \"northwind/billing\""),
        "{written}"
    );
}
