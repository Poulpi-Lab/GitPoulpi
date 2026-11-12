//! End-to-end checks of the responsive button bars:
//! at a narrow width the toolbar's least important buttons collapse into
//! one `⋯`, clicking it opens the menu of the hidden buttons, and
//! choosing a row -- by mouse or by keyboard -- runs the hidden button's
//! own action. Driven through a pty by `xtask::pty`: the real binary, the
//! real daemon, real `git`.
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, nothing sleeps, and every wait describes the state it is
//! waiting for.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
/// Narrow enough that Tag, Stash, Pop and Search do not fit (see
/// `chrome::tests::the_toolbar_at_a_narrow_width_...` for the layout).
const COLS: u16 = 60;
const ROWS: u16 = 30;

const OVERFLOW: &str = "\u{22EF}";

fn launch(repo: &Path, session: &str) -> (Terminal, tempfile::TempDir) {
    launch_with(repo, session, "")
}

fn launch_with(repo: &Path, session: &str, config: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .session(session)
        .config(config)
        .size(COLS, ROWS)
        .ready_when("the narrow toolbar", |s| {
            s.contains(OVERFLOW) && s.contains("Pull")
        })
        .mouse_capture()
        .start()
}

fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("overflow fixture")
        .unwrap()
        .build()
}

/// Click the `⋯`, click `⌕ Search` in its menu: the Log's find bar opens,
/// exactly as the Search button (or `f`) opens it.
#[test]
fn clicking_a_hidden_toolbar_button_in_the_overflow_menu_runs_it() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-overflow-mouse");

    // The toolbar row is above every pane, so the first `⋯` on screen
    // is the toolbar's.
    let (col, row) = term.wait_for_text(OVERFLOW, STEP).unwrap();
    assert!(
        !term.text().contains("Search"),
        "Search should be hidden at {COLS} columns:\n{}",
        term.text()
    );
    term.click(col, row).expect("click the overflow button");
    let (c, r) = term
        .wait_for_text("\u{2315} Search", STEP)
        .unwrap_or_else(|e| panic!("the overflow menu did not open: {e}"));
    term.wait_for_text("Pop", STEP)
        .unwrap_or_else(|e| panic!("the menu lists every hidden button: {e}"));
    term.click(c + 2, r).expect("choose Search");

    // The find bar: its counter, and typing narrows to the one commit.
    term.wait_for_text_to_go("Pop", STEP)
        .unwrap_or_else(|e| panic!("the menu should close on a choice: {e}"));
    term.send_text("overflow").unwrap();
    term.wait_for_text("1 of 1", STEP)
        .unwrap_or_else(|e| panic!("Search did not open the find bar: {e}"));
    term.stop().unwrap();
}

/// The keyboard half: the menu takes the arrows and Enter like every
/// other popup menu, and Enter runs the row's button - here Tag, which
/// opens the new-tag prompt.
#[test]
fn the_overflow_menu_is_driven_by_the_keyboard_too() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-overflow-keys");

    let (col, row) = term.wait_for_text(OVERFLOW, STEP).unwrap();
    term.click(col, row).expect("click the overflow button");
    term.wait_for_text("Tag", STEP)
        .unwrap_or_else(|e| panic!("the overflow menu did not open: {e}"));
    // Rows: Tag, Stash, Pop, Search -- the cursor starts on Tag (Undo and
    // Redo outrank them and stay on the bar).
    term.send_keys("Enter").unwrap();
    // The window is narrow here, so the Ref column is squeezed away and the
    // prompt falls back to the status-bar field ("New tag name"); wide
    // enough, it is the inline box ("enter tag name").
    term.wait_for("the new-tag prompt", STEP, |s| {
        s.contains("New tag name") || s.contains("enter tag name")
    })
    .unwrap_or_else(|e| panic!("Enter on Tag did not run the Tag button: {e}"));
    term.stop().unwrap();
}

/// Opening the overflow menu closes every context menu, including the
/// commit-file menu.
#[test]
fn the_overflow_menu_closes_the_commit_file_context_menu() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "overflow subject")
        .unwrap()
        .build();
    // Pinned to the top toolbar (the old layout): the commit-file menu opens
    // at the bottom of this small window and would cover a bottom toolbar,
    // so the `⋯` could not be clicked while the menu is open.
    let (mut term, _config) = launch_with(
        repo.root(),
        "pty-overflow-closes-menus",
        "[ui]\ntoolbar_position = \"top\"\n",
    );
    let (c, r) = term.wait_for_text("overflow subject", STEP).unwrap();
    term.click(c + 1, r).unwrap();
    let (c, r) = term.wait_for_text("a.txt", STEP).unwrap();
    term.right_click(c + 1, r).unwrap();
    term.wait_for_text("Open Diff", STEP)
        .unwrap_or_else(|e| panic!("the commit-file menu did not open: {e}"));
    // A press outside an open menu only closes it (the click-away rule):
    // the first click on `⋯` takes the commit-file menu away, the next one
    // opens the overflow menu. Never two menus at once.
    let (c, r) = term.wait_for_text(OVERFLOW, STEP).unwrap();
    term.click(c, r).expect("click the overflow button");
    term.wait_for("the commit-file menu closed", STEP, |s| {
        !s.contains("Open Diff") && !s.contains("\u{2315} Search")
    })
    .unwrap_or_else(|e| panic!("the press outside did not only close the menu: {e}"));
    common::retry_until(
        &mut term,
        "the overflow menu",
        4,
        Duration::from_secs(8),
        |t| {
            if let Some((c, r)) = t.screen().find(OVERFLOW) {
                t.click(c, r).expect("click the overflow button");
            }
        },
        |s| s.contains("\u{2315} Search") && !s.contains("Open Diff"),
    );
    term.stop().unwrap();
}
