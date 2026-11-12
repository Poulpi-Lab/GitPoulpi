#![cfg(feature = "radar")]
//! A press outside an open Radar popup closes it and runs nothing underneath:
//! the owner's rule for every menu and dropdown (`pty_click_away.rs` has the
//! rest of the app). The comboboxes (Scope, View, Show, Snooze) and the row
//! menu are checked with a left press on the toolbar's Back button (which
//! would leave the Radar if it ran) and with a right-click outside.

#![allow(dead_code)]

#[path = "common/mod.rs"]
mod common;
#[path = "pty_hosting/mock.rs"]
mod mock;
#[path = "pty_hosting/support.rs"]
mod support;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use mock::start_mock;
use support::{config_toml, COLS, ROWS, STARTUP, STEP};
use xtask::pty::{Screen, Terminal};

/// A second repository: nothing pushed, its own commit subject, its origin
/// on the mock under `team/tool`.
fn second(port: u16) -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("t.txt", "t\n", "second-repo-start")
        .unwrap()
        .build();
    repo.git(&[
        "remote",
        "add",
        "origin",
        &format!("http://127.0.0.1:{port}/team/tool.git"),
    ]);
    repo
}

/// Links the repository's pull requests and issues to the mock's project.
fn link(repo: &FixtureRepo, project: &str) {
    let dir = repo.state_dir();
    std::fs::create_dir_all(&dir).expect("state dir");
    std::fs::write(
        dir.join("repo-state.toml"),
        format!(
            "[pulls_source]\nintegration = \"corp\"\nproject = \"{project}\"\n\n[issues_source]\nintegration = \"corp\"\nproject = \"{project}\"\n"
        ),
    )
    .expect("seed repo-state.toml");
}

/// The chip of a repository: both fixtures are folders called `repo`, so
/// the Radar appends their parent folder.
fn name_of(repo: &FixtureRepo) -> String {
    let root = repo.root();
    format!(
        "repo ({})",
        root.parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy()
    )
}

struct Setup {
    term: Terminal,
    config: tempfile::TempDir,
    a: FixtureRepo,
    b: FixtureRepo,
    _mock: mock::Mock,
}

/// Both repositories are tabs of the session, the window is on the first.
fn setup(session: &str) -> Setup {
    let mock = start_mock();
    let a = support::fixture(mock.port);
    link(&a, "g/p");
    let b = second(mock.port);
    link(&b, "team/tool");
    let config = tempfile::tempdir().expect("temp config dir");
    std::fs::write(
        config.path().join("config.toml"),
        config_toml(mock.port, ""),
    )
    .expect("seed config.toml");
    std::fs::create_dir_all(config.path().join("sessions")).unwrap();
    std::fs::write(
        config.path().join("sessions").join(format!("{session}.toml")),
        format!(
            "[[tabs]]\nrepo_path = '{}'\nactive = true\n\n[[tabs]]\nrepo_path = '{}'\nactive = false\n",
            a.root().display(),
            b.root().display()
        ),
    )
    .expect("seed the session");
    let term = common::launch(a.root())
        .session(session)
        .config_dir(config.path())
        .size(COLS, ROWS)
        .env("GITPOULPI_NO_BROWSER", "1")
        .env("GITPOULPI_OPEN_LOG", config.path().join("opened.log"))
        .env("GITPOULPI_TOKEN_CORP", "test-token")
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .spawn();
    term.wait_for("the first repository loaded", STARTUP, |s| {
        s.contains("Add the feature")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    Setup {
        term,
        config,
        a,
        b,
        _mock: mock,
    }
}

/// Opens the Radar and waits for both repositories' rows.
fn open_radar(term: &mut Terminal) -> Screen {
    term.send_keys("M-r").expect("Alt+R");
    // Every repository answers on its own schedule (two repositories, several
    // requests each), so a list with the first rows is not the list: wait for
    // the whole fixture, each group with its count, or the next assertion or
    // click is made against a half-loaded screen (a row that is not there
    // yet, a count still short).
    term.wait_for("the Radar with both repositories' rows", STARTUP, |s| {
        s.contains("Radar")
            && s.contains("MY PULL REQUESTS  2")
            && s.contains("ASSIGNED TO ME  2")
            && s.contains("ISSUES CREATED BY ME  2")
            && s.contains("MENTIONING ME  4")
            && item_rows(s, "Add the parser").len() == 2
            && item_rows(s, "Docs for the parser").len() == 2
            && s.find_all("Please weigh in on the parser").len() == 2
    })
    .unwrap_or_else(|e| panic!("{e}"))
}

/// The list rows (not the status line) whose text holds `needle`.
fn item_rows(screen: &Screen, needle: &str) -> Vec<String> {
    screen
        .find_all(needle)
        .into_iter()
        .map(|(_, row)| screen.line(row))
        .filter(|line| line.contains('\u{21C4}') || line.contains('\u{25CF}'))
        .collect()
}

/// The cell of a list row holding `needle` (the status line may repeat it).
fn row_cell(screen: &Screen, needle: &str) -> (u16, u16) {
    screen
        .find_all(needle)
        .into_iter()
        .find(|&(_, row)| {
            let line = screen.line(row);
            line.contains('\u{21C4}') || line.contains('\u{25CF}')
        })
        .unwrap_or_else(|| panic!("no row {needle:?}:\n{}", screen.text()))
}

/// The Radar is still the Radar: its groups are listed.
fn still_radar(s: &Screen) -> bool {
    s.contains("MY PULL REQUESTS  2") && s.contains("ASSIGNED TO ME  2")
}

/// The cell of the Back button (on the toolbar line that holds Refresh).
fn back_button(s: &Screen) -> (u16, u16) {
    let (_, row) = s
        .find_all("Refresh")
        .into_iter()
        .find(|&(_, r)| s.line(r).contains("Back"))
        .unwrap_or_else(|| panic!("the toolbar:\n{}", s.text()));
    let line = s.line(row);
    let at = line.rfind("Back").unwrap();
    (line[..at].chars().count() as u16 + 1, row)
}

/// Each combobox: opened by its key, a left press on Back closes it and does
/// not leave the Radar; reopened, a right-click on a list row closes it and
/// opens no row menu.
fn combo_closes_outside(session: &str, key: &str, open: &str, row_first: bool) {
    let mut s = setup(session);
    open_radar(&mut s.term);
    if row_first {
        // The snooze list belongs to the selected row.
        let screen = s.term.screen();
        let (col, row) = row_cell(&screen, "Add the parser");
        s.term.click(col + 2, row).unwrap();
    }
    let opened = |scr: &Screen| scr.find_all(open).len() >= if open == "Open tabs" { 2 } else { 1 };
    s.term.send_text(key).unwrap();
    s.term
        .wait_for("the popup", STEP, |scr| opened(scr))
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    let (c, r) = back_button(&s.term.screen());
    s.term.click(c, r).unwrap();
    s.term
        .wait_for("the popup gone, still the Radar", STEP, |scr| {
            !opened(scr) && still_radar(scr)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));

    // The same popup, a right-click outside it, on a row that would open
    // the row menu.
    s.term.send_text(key).unwrap();
    s.term
        .wait_for("the popup again", STEP, |scr| opened(scr))
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    let (c, r) = row_cell(&s.term.screen(), "Docs for the parser");
    s.term.right_click(c + 2, r).unwrap();
    s.term
        .wait_for("the popup gone, no row menu", STEP, |scr| {
            !opened(scr) && !scr.contains("Pin to top") && still_radar(scr)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
}

#[test]
fn the_scope_combo_closes_on_a_press_outside_without_running_it() {
    combo_closes_outside("pty-radar-away-scope", "w", "Open tabs", false);
}

#[test]
fn the_view_combo_closes_on_a_press_outside_without_running_it() {
    combo_closes_outside("pty-radar-away-view", "v", "Issues", false);
}

#[test]
fn the_show_combo_closes_on_a_press_outside_without_running_it() {
    combo_closes_outside("pty-radar-away-who", "f", "Mentioning me", false);
}

#[test]
fn the_snooze_combo_closes_on_a_press_outside_without_running_it() {
    combo_closes_outside("pty-radar-away-snooze", "z", "Until tomorrow", true);
}

/// The row menu (a right-click on a row): a left press on Back closes it and
/// stays; a right-click on another row closes it and opens no new one.
#[test]
fn the_row_menu_closes_on_a_press_outside_without_running_it() {
    let mut s = setup("pty-radar-away-menu");
    open_radar(&mut s.term);
    let open_menu = |term: &mut Terminal| {
        common::retry_until(
            term,
            "the row menu",
            8,
            Duration::from_secs(4),
            |t| {
                let (c, r) = row_cell(&t.screen(), "Add the parser");
                t.right_click(c + 2, r).unwrap();
            },
            |scr| scr.contains("Pin to top"),
        );
    };
    open_menu(&mut s.term);
    let (c, r) = back_button(&s.term.screen());
    s.term.click(c, r).unwrap();
    s.term
        .wait_for("the menu gone, still the Radar", STEP, |scr| {
            !scr.contains("Pin to top") && still_radar(scr)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));

    open_menu(&mut s.term);
    let (c, r) = row_cell(&s.term.screen(), "Add the parser");
    s.term.right_click(c + 2, r).unwrap();
    s.term
        .wait_for(
            "the menu gone, none reopened (right-click on the row above it)",
            STEP,
            |scr| !scr.contains("Pin to top") && still_radar(scr),
        )
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
}
