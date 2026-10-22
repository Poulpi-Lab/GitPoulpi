#![cfg(feature = "radar")]
//! The Radar's account-wide rows (`Alt+R`): the requests that wait for your
//! review in every repository of a host, cloned or not -- the real binary and
//! daemon, driven through a pty by `xtask::pty`, against `xtask mock-host`
//! (a `TcpListener` on 127.0.0.1; no other network).
//!
//! One profile (`hub`, GitHub Enterprise) with no list turned on. The mock
//! answers the review-requested search with three requests: two in
//! repositories that exist nowhere on this machine, one in `lumen-labs/lumen-core`,
//! which the fixture repository (the only tab) has as its `origin`.

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

struct Setup {
    term: Terminal,
    config: tempfile::TempDir,
    _mock: MockHost,
    _repo: FixtureRepo,
}

/// The mock, a `hub` profile with `radar` as its `[integrations.radar]`
/// table (empty: all lists off), and the TUI on the fixture repository.
fn setup(session: &str, radar: &str) -> Setup {
    let mock = mock_host::start(Options {
        port: 0,
        bare_root: None,
    })
    .expect("the mock host");
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "account-start-commit")
        .unwrap()
        .build();
    repo.git(&[
        "remote",
        "add",
        "origin",
        &format!("http://127.0.0.1:{}/lumen-labs/lumen-core.git", mock.port),
    ]);
    let config = tempfile::tempdir().expect("temp config dir");
    std::fs::write(
        config.path().join("config.toml"),
        format!(
            "[[integrations]]\nid = \"hub\"\nkind = \"github_enterprise\"\nurl = \"http://127.0.0.1:{port}\"\ntoken_ref = \"integration:hub\"\nname = \"hub\"\nuser = {{ login = \"{login}\" }}\n{radar}",
            port = mock.port,
            login = mock_host::GITHUB_LOGIN,
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
        .env("GITPOULPI_TOKEN_HUB", "test-token")
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .spawn();
    term.wait_for("the repository loaded", STARTUP, |s| {
        s.contains("account-start-commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    Setup {
        term,
        config,
        _mock: mock,
        _repo: repo,
    }
}

fn open_radar(term: &mut Terminal) -> Screen {
    term.send_keys("M-r").expect("Alt+R");
    term.wait_for("the Radar", STARTUP, |s| {
        s.contains("Radar") && s.contains("Scope")
    })
    .unwrap_or_else(|e| panic!("{e}"))
}

/// The list line (not the status line) holding `needle`.
fn row_of(screen: &Screen, needle: &str) -> Option<String> {
    screen
        .find_all(needle)
        .into_iter()
        .map(|(_, row)| screen.line(row))
        .find(|line| line.contains('\u{21C4}'))
}

fn config_of(dir: &Path) -> String {
    std::fs::read_to_string(dir.join("config.toml")).unwrap_or_default()
}

fn rows_ready(s: &Screen) -> bool {
    s.contains("Harden the token build")
        && s.contains("Lower the render budget")
        && s.contains("Batch the metric flushes")
}

/// Off by default: the Radar offers the lists on one line. `T` writes the
/// setting and the rows of repositories that are not cloned arrive; the one
/// whose repository is cloned carries it, the others say so.
#[test]
fn the_hint_turns_the_account_lists_on_and_not_cloned_rows_say_so() {
    let mut s = setup("pty-radar-account-on", "");
    open_radar(&mut s.term);
    let screen = s
        .term
        .wait_for("the hint row", STARTUP, |s| s.contains("Turn on"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        screen.contains("requests waiting for your review"),
        "{}",
        screen.text()
    );
    assert!(!screen.contains("Harden the token build"), "off by default");
    assert!(!config_of(s.config.path()).contains("review_requests"));

    s.term.send_text("T").expect("T");
    let screen = s
        .term
        .wait_for("the account rows", STARTUP, rows_ready)
        .unwrap_or_else(|e| panic!("{e}"));
    let saved = config_of(s.config.path());
    assert!(saved.contains("review_requests = true"), "{saved}");
    assert!(!screen.contains("Turn on"), "the hint is gone");
    assert!(screen.contains("[x] Not cloned") && screen.contains("[ ] Drafts"));

    // A row of a repository that is not cloned: dimmed chip with the project.
    let far = row_of(&screen, "Harden the token build").expect("the row");
    assert!(far.contains("harbor-guild"), "{far}");
    // The one whose project is the fixture's origin carries the clone.
    let near = row_of(&screen, "Lower the render budget").expect("the row");
    assert!(!near.contains("lumen-labs"), "{near}");

    // Select the far row: the status line says it, and a branch action
    // answers instead of opening anything.
    let (col, row) = screen.find("Harden the token build").unwrap();
    s.term.click(col + 2, row).expect("select the row");
    s.term.send_text("c").expect("c");
    s.term
        .wait_for("the clone-first message", STEP, |s| {
            s.contains("clone the repository first")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(s.term.screen().contains("Not cloned"));
}

/// `a` hides the rows of repositories with no known clone and keeps the
/// clone's; `d` brings the drafts; both are kept in `radar.toml`.
#[test]
fn a_and_d_filter_the_account_rows_and_radar_toml_remembers() {
    let mut s = setup(
        "pty-radar-account-filters",
        "\n[integrations.radar]\nreview_requests = true\nopen_requests = true\n",
    );
    open_radar(&mut s.term);
    s.term
        .wait_for("the account rows", STARTUP, |s| {
            rows_ready(s) && s.contains("Rename the spacing scale")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !s.term.screen().contains("WIP sampling rework"),
        "drafts are hidden"
    );

    s.term.send_text("d").expect("d");
    s.term
        .wait_for("the draft", STEP, |s| s.contains("WIP sampling rework"))
        .unwrap_or_else(|e| panic!("{e}"));

    s.term.send_text("a").expect("a");
    let screen = s
        .term
        .wait_for("only the clone's rows left", STEP, |s| {
            !s.contains("Harden the token build") && s.contains("[ ] Not cloned")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        screen.contains("Lower the render budget"),
        "{}",
        screen.text()
    );
    assert!(
        !screen.contains("Rename the spacing scale"),
        "{}",
        screen.text()
    );

    let file = std::fs::read_to_string(s.config.path().join("radar.toml")).unwrap_or_default();
    assert!(
        file.contains("show_not_cloned = false") && file.contains("show_drafts = true"),
        "{file}"
    );
    // The same keys flip them back.
    s.term.send_text("a").expect("a");
    s.term
        .wait_for("the far rows back", STEP, |s| {
            s.contains("Harden the token build") && s.contains("[x] Not cloned")
        })
        .unwrap_or_else(|e| panic!("{e}"));
}

/// Clicking the "Not cloned" checkbox while the last list row (a far one) is
/// selected hides rows that the selection pointed at: the Radar stays up and
/// shows the clone's rows.
#[test]
fn unticking_not_cloned_by_mouse_with_a_far_row_selected_does_not_crash() {
    let mut s = setup(
        "pty-radar-account-untick",
        "\n[integrations.radar]\nreview_requests = true\nopen_requests = true\n",
    );
    open_radar(&mut s.term);
    let screen = s
        .term
        .wait_for("the account rows", STARTUP, |s| {
            rows_ready(s) && s.contains("Rename the spacing scale")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    // The oldest request sorts last: the highest board index, and a far one.
    let (col, row) = screen.find("Batch the metric flushes").expect("the row");
    s.term.click(col + 2, row).expect("select the last row");
    let (col, row) = screen.find("[x] Not cloned").expect("the checkbox");
    s.term.click(col + 1, row).expect("untick");
    let screen = s
        .term
        .wait_for("the checkbox unticked", STEP, |s| {
            s.contains("[ ] Not cloned")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !screen.contains("Harden the token build"),
        "{}",
        screen.text()
    );
    assert!(
        screen.contains("Lower the render budget"),
        "{}",
        screen.text()
    );
    assert!(!s.term.exited(), "the TUI is still running");
}
