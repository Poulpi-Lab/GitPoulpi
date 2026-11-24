#![cfg(feature = "radar")]
//! The Radar (`Alt+R`): the pull requests, issues and WIP of the open
//! tabs in one list -- the real binary, the real daemon, driven through a pty
//! by `xtask::pty`, against the local mock of a GitLab instance the hosting
//! tests use (a `TcpListener` on 127.0.0.1; no other network).
//!
//! Two repositories, `g/p` and `team/tool`, both linked to the mock. It
//! answers the same merge requests for either, so rows are told apart by
//! their repository chip.

#![allow(dead_code)]

#[path = "common/mod.rs"]
mod common;
#[path = "pty_hosting/mock.rs"]
mod mock;
#[path = "pty_hosting/support.rs"]
mod support;

use std::path::Path;

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

/// The cell of `needle` on the line that also holds `also`.
fn find_on_line(screen: &Screen, needle: &str, also: &str) -> (u16, u16) {
    screen
        .find_all(needle)
        .into_iter()
        .find(|&(_, row)| screen.line(row).contains(also))
        .unwrap_or_else(|| panic!("no {needle:?} on a line with {also:?}:\n{}", screen.text()))
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

fn radar_file(config: &Path) -> String {
    std::fs::read_to_string(config.join("radar.toml")).unwrap_or_default()
}

/// `Alt+R` opens the view; the rows of both repositories are there, grouped
/// and counted, each with its repository chip.
#[test]
fn the_radar_lists_the_rows_of_both_repositories() {
    let mut s = setup("pty-radar-rows");
    let screen = open_radar(&mut s.term);
    let (a, b) = (name_of(&s.a), name_of(&s.b));
    // "My" holds the request kai wrote in each repository, "Assigned" the
    // one assigned to him, the issue he created is under "Issues created".
    assert!(screen.contains("MY PULL REQUESTS  2"), "{}", screen.text());
    assert!(screen.contains("ASSIGNED TO ME  2"), "{}", screen.text());
    assert!(
        screen.contains("ISSUES CREATED BY ME  2"),
        "{}",
        screen.text()
    );
    for name in [&a, &b] {
        let row = screen
            .find_all("Add the parser")
            .into_iter()
            .map(|(_, r)| screen.line(r))
            .find(|l| l.contains(name.as_str()));
        assert!(row.is_some(), "a row for {name}:\n{}", screen.text());
    }
    // The three comboboxes and the filter.
    assert!(screen.contains("Open tabs") && screen.contains("Everyone"));
    // Only GitHub and GitLab are ever named.
    for word in ["Bitbucket", "Azure", "Jira", "Trello", "OAuth"] {
        assert!(!screen.contains(word), "{word}");
    }
}

/// Pinning moves a row to the top, in a PINNED group, and the pin is in
/// `radar.toml`: coming back to the Radar shows it still on top.
#[test]
fn a_pinned_row_goes_to_the_top_and_stays_there_when_reopened() {
    let mut s = setup("pty-radar-pin");
    let screen = open_radar(&mut s.term);
    let b = name_of(&s.b);
    let (col, row) = find_on_line(&screen, "Docs for the parser", &b);
    s.term.click(col + 2, row).expect("select the row");
    s.term.send_text("p").expect("pin it");
    let screen = s
        .term
        .wait_for("the PINNED group on top", STEP, |s| {
            let pinned = s.find_all("PINNED").first().map(|&(_, r)| r);
            let mine = s.find_all("MY PULL REQUESTS").first().map(|&(_, r)| r);
            matches!((pinned, mine), (Some(p), Some(m)) if p < m)
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (_, pinned_row) = screen.find("PINNED").unwrap();
    assert!(
        screen.line(pinned_row + 1).contains("Docs for the parser"),
        "the pinned row sits right under its heading:\n{}",
        screen.text()
    );
    let file = radar_file(s.config.path());
    assert!(
        file.contains("team/tool") && file.contains("number = 54"),
        "{file}"
    );

    // Back to Home and into the Radar again: still pinned.
    s.term.send_keys("Escape").unwrap();
    s.term
        .wait_for("Home", STEP, |s| s.contains("Repository Management"))
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("M-r").unwrap();
    s.term
        .wait_for("the pin kept", STEP, |s| {
            s.contains("PINNED") && s.contains("Radar")
        })
        .unwrap_or_else(|e| panic!("{e}"));
}

/// Snoozing hides the row (the Snoozed group keeps a count) and writes the
/// snooze to the file.
#[test]
fn a_snoozed_row_is_hidden() {
    let mut s = setup("pty-radar-snooze");
    let screen = open_radar(&mut s.term);
    let a = name_of(&s.a);
    let (col, row) = find_on_line(&screen, "Docs for the parser", &a);
    s.term.click(col + 2, row).expect("select the row");
    // `z` opens the duration combobox; Enter picks 1 hour.
    s.term.send_text("z").unwrap();
    s.term
        .wait_for("the snooze durations", STEP, |s| {
            s.contains("Until tomorrow")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("Enter").unwrap();
    let screen = s
        .term
        .wait_for("the row hidden", STEP, |s| {
            item_rows(s, "Docs for the parser").len() == 1 && s.contains("SNOOZED  1")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !item_rows(&screen, "Docs for the parser")
            .iter()
            .any(|l| l.contains(a.as_str())),
        "the snoozed row is the one of {a}:\n{}",
        screen.text()
    );
    let file = radar_file(s.config.path());
    assert!(
        file.contains("[[snoozes]]") && file.contains("number = 54"),
        "{file}"
    );
}

/// Enter on a row opens its repository: the window moves to that tab.
#[test]
fn enter_on_a_row_opens_its_repository() {
    let mut s = setup("pty-radar-open");
    let screen = open_radar(&mut s.term);
    let b = name_of(&s.b);
    let (col, row) = find_on_line(&screen, "Add the parser", &b);
    s.term.click(col + 2, row).expect("select the row");
    s.term.send_keys("Enter").unwrap();
    s.term
        .wait_for("the second repository's window", STEP, |s| {
            s.contains("second-repo-start") && !s.contains("MY PULL REQUESTS")
        })
        .unwrap_or_else(|e| panic!("{e}"));
}

/// The palette reaches it too: `Radar: Open` from a repository window.
#[test]
fn the_palette_opens_the_radar() {
    let mut s = setup("pty-radar-palette");
    s.term.send_keys("C-p").unwrap();
    s.term.send_text("Radar: Open").unwrap();
    s.term
        .wait_for("the palette entry", STEP, |s| s.contains("Radar: Open"))
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("Enter").unwrap();
    s.term
        .wait_for("the Radar", STEP, |s| {
            s.contains("MY PULL REQUESTS") && s.contains("Open tabs")
        })
        .unwrap_or_else(|e| panic!("{e}"));
}

/// The host's answers about checks and conflicts show as glyphs on the row,
/// in words on the status line when the row is selected, and in a tooltip
/// after a dwell on the glyphs; what mentions kai is its own group.
#[test]
fn checks_conflicts_and_mentions_show_on_the_rows() {
    let mut s = setup("pty-radar-checks");
    let screen = open_radar(&mut s.term);
    let a = name_of(&s.a);
    // "Mentioning me": the request and the issue of someone else, in both
    // repositories (the to-do list answers both projects).
    assert!(screen.contains("MENTIONING ME  4"), "{}", screen.text());
    assert_eq!(
        screen.find_all("Please weigh in on the parser").len(),
        2,
        "{}",
        screen.text()
    );
    // A failed pipeline with a conflict: a cross and a mark. A passing one: a
    // tick and no mark.
    let (col, row) = find_on_line(&screen, "Add the parser", &a);
    let line = screen.line(row);
    assert!(line.contains("\u{2717}!"), "{line}");
    let (_, ok_row) = find_on_line(&screen, "Docs for the parser", &a);
    let ok_line = screen.line(ok_row);
    assert!(
        ok_line.contains("\u{2713} ") && !ok_line.contains("\u{2713}!"),
        "{ok_line}"
    );

    // Selecting the row says it in words on the status line (the keyboard's
    // twin of the tooltip).
    s.term.click(col + 2, row).expect("select the row");
    s.term
        .wait_for("the detail line", STEP, |s| {
            s.contains("Checks failed, Conflicts with main")
        })
        .unwrap_or_else(|e| panic!("{e}"));

    // Resting the pointer on the glyphs shows the tooltip after the dwell.
    let glyph = screen.line(row).find('\u{2717}').expect("the cross");
    let glyph_col = screen.line(row)[..glyph].chars().count() as u16;
    s.term.move_to(glyph_col, row).unwrap();
    let screen = s
        .term
        .wait_for("the tooltip", STEP, |s| {
            s.find_all("Checks failed")
                .into_iter()
                .any(|(_, r)| r > row)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    assert!(screen.contains("Checks failed"), "{}", screen.text());
}

/// `Ctrl+N` over the Radar: waits for the frame that shows the prefix
/// hint (the badge key is looked up in what the last frame drew).
fn open_badges(term: &mut Terminal) {
    term.send_keys("C-n").unwrap();
    term.wait_for("the prefix hint and the badges", STEP, |s| {
        s.contains("<badge>=focus")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// Badges over the Radar, in this order: the filter `1`, the Scope,
/// View and Show comboboxes `2`-`4`, Refresh `5`, Back `6`, then the section
/// headings on screen `7`... The filter badge puts the keyboard in the
/// filter: what is typed narrows the list.
#[test]
fn ctrl_n_badge_focuses_the_radar_filter() {
    let mut s = setup("pty-radar-nav-filter");
    open_radar(&mut s.term);
    open_badges(&mut s.term);
    s.term.send_text("1").unwrap();
    s.term
        .wait_for("the prefix hint gone", STEP, |s| {
            !s.contains("<badge>=focus")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    // `q` would quit and `w` open the Scope if the list had the keyboard.
    s.term.send_text("zzzz").unwrap();
    s.term
        .wait_for("the typed text in the filter", STEP, |s| {
            s.contains("zzzz") && !s.contains("Add the parser")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
}

/// A combobox badge opens that combobox (here Scope `2`: its first option is
/// the open tabs, its last the folder scopes).
#[test]
fn ctrl_n_badge_opens_a_radar_combobox() {
    let mut s = setup("pty-radar-nav-combo");
    open_radar(&mut s.term);
    open_badges(&mut s.term);
    s.term.send_text("4").unwrap();
    // The Show combobox lists who the rows are about.
    s.term
        .wait_for("the Show options", STEP, |s| {
            s.contains("Review requested") && s.contains("Mentioning me")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
}

/// The Back badge (`7`) acts like its button: back to the Home tab.
#[test]
fn ctrl_n_badge_presses_the_radar_back_button() {
    let mut s = setup("pty-radar-nav-back");
    open_radar(&mut s.term);
    open_badges(&mut s.term);
    s.term.send_text("7").unwrap();
    s.term
        .wait_for("Home", STEP, |s| s.contains("Repository Management"))
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
}

/// A heading badge (`8` is the first heading on screen, `9` the second)
/// lands the selection on that heading: `Enter` then folds exactly it.
#[test]
fn ctrl_n_badge_selects_a_radar_section() {
    let mut s = setup("pty-radar-nav-heading");
    let screen = open_radar(&mut s.term);
    // MY PULL REQUESTS is the first heading, ASSIGNED TO ME the second, and
    // the only rows that say "Docs for the parser" are under it.
    let (_, mine) = screen.find("MY PULL REQUESTS").unwrap();
    let (_, assigned) = screen.find("ASSIGNED TO ME").unwrap();
    assert!(mine < assigned, "{}", screen.text());
    let key = "Docs for the parser".to_string();
    open_badges(&mut s.term);
    s.term.send_text("9").unwrap();
    s.term
        .wait_for("the prefix hint gone", STEP, |s| {
            !s.contains("<badge>=focus")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let before = s.term.screen().find_all(&key).len();
    s.term.send_keys("Enter").unwrap();
    s.term
        .wait_for("the section folded", STEP, |s| {
            s.find_all(&key).len() < before
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
}

/// `Esc` cancels the badges: nothing is focused and the Radar stays
/// open (the key that would have pressed Back then does nothing).
#[test]
fn ctrl_n_escape_cancels_the_radar_badges() {
    let mut s = setup("pty-radar-nav-cancel");
    open_radar(&mut s.term);
    open_badges(&mut s.term);
    s.term.send_keys("Escape").unwrap();
    s.term
        .wait_for("the hint gone", STEP, |s| !s.contains("<badge>=focus"))
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_text("7").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(400));
    let screen = s.term.screen();
    assert!(
        screen.contains("Radar") && !screen.contains("Repository Management"),
        "{}",
        screen.text()
    );
}

/// `Ctrl+O` on the open Radar goes back Home (the Radar closes, the list's
/// search is focused): it does not duplicate the Radar's own `/` and
/// `Ctrl+F` by focusing the Radar's filter.
#[test]
fn ctrl_o_on_the_radar_goes_home_instead_of_searching() {
    let mut s = setup("pty-radar-ctrl-o");
    open_radar(&mut s.term);
    s.term.send_keys("C-o").expect("Ctrl+O");
    s.term
        .wait_for("Home without the Radar", STEP, |s| {
            s.contains("Repository Management") && !s.contains("MY PULL REQUESTS")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    // The typed text goes to the Home search (the Radar is gone, so it
    // cannot be in its filter) and the Radar stays closed.
    s.term.send_text("zqx").expect("type");
    let screen = s
        .term
        .wait_for("the text in the Home search", STEP, |s| s.contains("zqx"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !screen.contains("MY PULL REQUESTS") && screen.contains("Repository Management"),
        "{}",
        screen.text()
    );
}
