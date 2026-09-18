//! The global Activity Log (see the background-ops design, B13): a clone that fails
//! from the Home tab has no repository tab to carry its entry, so the failure
//! card's Details opens the Activity Log on "All repositories" with the entry open,
//! and that view is reachable from Home by button, key and palette, with a Scope box
//! to narrow it.
//!
//! Real binary and daemon through a pty; temp config dirs and fixture
//! repositories only, nothing is cloned from the network (the URL is a closed
//! local port).

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

struct Setup {
    term: Terminal,
    _root: tempfile::TempDir,
    _start: FixtureRepo,
    _config: tempfile::TempDir,
}

fn launch(session: &str) -> Setup {
    let start = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "global-log-start-commit")
        .unwrap()
        .build();
    let root = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        format!("[repos]\nclone_root = '{}'\n", root.path().display()),
    )
    .unwrap();
    let term = common::launch(start.root())
        .session(session)
        .config_dir(config.path())
        .size(180, 45)
        .no_exit_pause()
        .mouse_capture()
        .spawn();
    term.wait_for("the first window", STARTUP, |s| {
        s.contains("global-log-start-commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    Setup {
        term,
        _root: root,
        _start: start,
        _config: config,
    }
}

/// Home, the Clone dialog, `url` typed, Enter: a clone that fails at once.
/// It is the `nth` failure (the Activity button counts the unread ones).
fn failing_clone(term: &mut Terminal, name: &str, nth: usize) {
    term.send_keys("C-o").expect("the management tab");
    term.wait_for("the management tab", STEP, |s| {
        s.contains("Repository Management") && s.contains("Clone")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("c").expect("Clone");
    term.wait_for("the Clone dialog", STEP, |s| {
        s.contains("Clone a repository") && s.contains("Clone with URL")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text(&format!("git://127.0.0.1:1/{name}.git"))
        .expect("the URL");
    let n = name.to_string();
    term.wait_for("the full path", STEP, move |s| {
        s.contains("Full path:") && s.contains(&n)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    // No card: the status line says it (over Home's bar) and counts it.
    term.wait_for("the failure in the status line", STEP, |s| {
        s.contains("error: Clone failed") && s.contains(&format!("Activity ({nth})"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

fn global_tab_shows(s: &Screen) -> bool {
    s.contains("Scope:") && s.contains("All repositories")
}

/// A click on the status line's error opens the Global tab on the entry,
/// which is listed under the clone's destination.
#[test]
fn a_failed_clone_from_home_is_in_the_global_log_and_the_error_line_opens_it() {
    let mut s = launch("pty-global-details");
    failing_clone(&mut s.term, "orphan", 1);
    let screen = s.term.screen();
    let (row, line) = screen
        .text()
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains("error: Clone failed"))
        .map(|(i, l)| (i, l.to_string()))
        .unwrap();
    let col = line[..line.find("error:").unwrap()].chars().count();
    s.term.click(col as u16 + 2, row as u16).unwrap();
    let screen = s
        .term
        .wait_for("the global log on the entry", STEP, |x| {
            global_tab_shows(x) && x.contains("[orphan]")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    // Opened on the entry: its details (the failure) are expanded.
    assert!(screen.contains("Clone failed"), "{}", screen.text());
    s.term.send_keys("Escape").unwrap();
    s.term.stop().unwrap();
}

/// Two failed clones are two repositories in the box: picking one by click
/// narrows the list to it.
#[test]
fn the_repository_box_narrows_the_global_log_by_click() {
    let mut s = launch("pty-global-filter");
    failing_clone(&mut s.term, "alpha-gone", 1);
    // Leave Home for the repository tab (`Esc` is Home's), then clone again:
    // each step is waited for by what only its result draws.
    s.term.send_keys("Escape").unwrap();
    s.term
        .wait_for("Home left for the repository tab", STEP, |x| {
            x.contains("global-log-start-commit") && !x.contains("Repository Management")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    failing_clone(&mut s.term, "beta-gone", 2);
    s.term.send_keys("C-n").unwrap();
    s.term.send_keys("G").unwrap();
    s.term
        .wait_for("both failures", STEP, |x| {
            global_tab_shows(x) && x.contains("[alpha-gone]") && x.contains("[beta-gone]")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));

    // The repository box: click it, click the row for alpha-gone.
    let (col, row) = s
        .term
        .screen()
        .find("All repositories")
        .expect("the repository box");
    s.term.click(col + 1, row).unwrap();
    let screen = s
        .term
        .wait_for("the list", STEP, |x| {
            x.text()
                .lines()
                .any(|l| l.contains("alpha-gone") && !l.contains("[alpha-gone]"))
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    let pick_row = screen
        .text()
        .lines()
        .position(|l| l.contains("alpha-gone") && !l.contains("[alpha-gone]"))
        .unwrap() as u16;
    let line = s.term.screen().line(pick_row);
    let pick_col = line.find("alpha-gone").unwrap();
    let pick_col = line[..pick_col].chars().count() as u16;
    s.term.click(pick_col + 1, pick_row).unwrap();
    // The box now says alpha-gone: the same frame lists only its entries.
    let screen = s
        .term
        .wait_for("the box shows the pick", STEP, |x| {
            x.text()
                .lines()
                .any(|l| l.contains("Scope:") && l.contains("alpha-gone"))
        })
        .unwrap_or_else(|e| {
            panic!(
                "{e}
{}",
                s.term.text()
            )
        });
    // One clone failure row is left (a narrowed scope names no repository per
    // row); under "All repositories" there were two.
    let rows = screen
        .text()
        .lines()
        .filter(|l| l.contains("Clone failed - "))
        .count();
    assert_eq!(rows, 1, "{}", screen.text());
    s.term.stop().unwrap();
}

/// From Home: the status bar's Activity button, the key (`Ctrl+N Shift+G`) and the palette entry
/// each open the Global tab.
#[test]
fn home_opens_the_global_log_by_button_key_and_palette() {
    let mut s = launch("pty-global-home");
    s.term.send_keys("C-o").unwrap();
    s.term
        .wait_for("Home with its button", STEP, |x| {
            x.contains("Repository Management") && x.contains("Activity")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    // The status bar's Activity button, bottom right, like a repository
    // tab's; it opens the Global tab here.
    let screen = s.term.screen();
    let last = screen.rows() - 2;
    let line = screen.line(last);
    let col = line[..line.find("Activity").expect("the button")]
        .chars()
        .count() as u16;
    s.term.click(col + 1, last).unwrap();
    s.term
        .wait_for("the global log by button", STEP, global_tab_shows)
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.send_keys("Escape").unwrap();
    s.term
        .wait_for("closed", STEP, |x| !global_tab_shows(x))
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));

    s.term.send_keys("C-n").unwrap();
    s.term.send_keys("G").unwrap();
    s.term
        .wait_for("the global log by key", STEP, global_tab_shows)
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.send_keys("Escape").unwrap();
    s.term
        .wait_for("closed again", STEP, |x| !global_tab_shows(x))
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));

    s.term.send_keys("C-p").unwrap();
    s.term.send_text("Global Activity Log").unwrap();
    s.term
        .wait_for("the palette row", STEP, |x| {
            x.contains("View: Global Activity Log")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.send_keys("Enter").unwrap();
    s.term
        .wait_for("the global log by palette", STEP, global_tab_shows)
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.stop().unwrap();
}

/// `Alt+R` is the Radar's key from the Log (the commit graph of a repository
/// window); it is also `merge.reset_all` and `composer.insert_issue_ref`,
/// which win in their own contexts.
#[cfg(feature = "radar")]
#[test]
fn alt_r_opens_the_radar_from_the_log() {
    let mut s = launch("pty-global-alt-r");
    s.term.send_keys("M-r").unwrap();
    s.term
        .wait_for("the Radar", STEP, |x| {
            x.contains("Scope") && x.contains("Open tabs") && x.contains("Refresh")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", s.term.text()));
    s.term.stop().unwrap();
}
