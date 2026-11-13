//! Renaming a branch in place: the name is typed over the branch's own row
//! in the Refs sidebar, or in the Ref cell of its tip's Log row, never in
//! the status line. A refusal is written on the row below with the box
//! still open. Same rules as `pty_ui.rs`: nothing is located by a
//! hard-coded coordinate, temp config dirs and fixture repositories.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const SENTINEL: &str = "zq9x";

/// Two commits; `hello` shares the newest one with the checked-out branch,
/// `to-delete` sits on the older one.
fn fixture() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("older target commit")
        .unwrap()
        .commit("newer head commit")
        .unwrap()
        .build();
    repo.git(&["branch", "hello"]);
    repo.git(&["branch", "to-delete", "HEAD~1"]);
    repo
}

fn launch(repo: &FixtureRepo, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .size(200, 60)
        .ready_when("the fixture loaded", |s| {
            s.contains("older target commit") && s.contains("newer head commit")
        })
        .start()
}

/// The sidebar row of `name` (its leftmost occurrence, left of the Log).
fn sidebar_cell(s: &Screen, name: &str) -> (u16, u16) {
    s.find_all(name)
        .into_iter()
        .filter(|(c, _)| *c < 40)
        .min_by_key(|(c, _)| *c)
        .unwrap_or_else(|| panic!("no `{name}` row in the sidebar:\n{}", s.text()))
}

fn branches(repo: &FixtureRepo) -> String {
    repo.git(&["branch", "--list"])
}

fn erase(term: &mut Terminal, chars: usize) {
    for _ in 0..chars {
        term.send_keys("BSpace").unwrap();
    }
}

fn open_rename_from_sidebar(term: &mut Terminal, name: &str) -> (u16, u16) {
    let (col, row) = sidebar_cell(&term.screen(), name);
    term.right_click(col + 1, row).unwrap();
    let label = format!("Rename {name}...");
    let menu = term
        .wait_for("the ref menu", STEP, |s| s.contains(&label))
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find(&label).unwrap();
    term.click(c + 1, r).unwrap();
    (col, row)
}

/// Right-click the pill of `name` on the Log row of `commit_text`, then
/// pick its Rename row.
fn open_rename_from_pill(term: &mut Terminal, commit_text: &str, name: &str) -> u16 {
    let (_, row) = term.wait_for_text(commit_text, STEP).unwrap();
    let col = term
        .screen()
        .find_all(name)
        .into_iter()
        .find(|(c, r)| *r == row && *c > 40)
        .unwrap_or_else(|| panic!("no `{name}` pill on the row of {commit_text}"))
        .0;
    term.right_click(col + 1, row).unwrap();
    let label = format!("Rename {name}...");
    let menu = term
        .wait_for("the pill menu", STEP, |s| s.contains(&label))
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find(&label).unwrap();
    term.click(c + 1, r).unwrap();
    row
}

/// Rename from a Refs row: the typed text is on that very row, the status
/// line stays clean, and Enter renames the branch.
#[test]
fn renaming_from_the_refs_row_types_over_that_row_and_renames() {
    let repo = fixture();
    let (mut term, _c) = launch(&repo, "pty-rename-refs");
    let (_, row) = open_rename_from_sidebar(&mut term, "to-delete");
    erase(&mut term, "to-delete".len());
    term.send_text(&format!("{SENTINEL}-new")).unwrap();
    let screen = term
        .wait_for("the typed name on the branch's row", STEP, |s| {
            s.line(row).contains(&format!("{SENTINEL}-new"))
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        !common::status_line_text(&screen).contains(SENTINEL),
        "the name was echoed in the status line: {:?}",
        common::status_line_text(&screen)
    );
    term.send_keys("Enter").unwrap();
    for _ in 0..300 {
        let list = branches(&repo);
        if list.contains(&format!("{SENTINEL}-new")) {
            assert!(!list.contains("to-delete"), "{list}");
            term.stop().unwrap();
            return;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    panic!("the branch was never renamed:\n{}", term.text());
}

/// A name that already exists is refused on the next row; the box stays
/// open and nothing is renamed.
#[test]
fn an_existing_name_is_refused_on_the_row_below_and_the_box_stays() {
    let repo = fixture();
    let (mut term, _c) = launch(&repo, "pty-rename-refs-clash");
    // `hello` sorts first, so the row below it is free for the refusal.
    let (_, row) = open_rename_from_sidebar(&mut term, "hello");
    erase(&mut term, "hello".len());
    term.send_text("to-delete").unwrap();
    term.send_keys("Enter").unwrap();
    let screen = term
        .wait_for("the refusal under the box", STEP, |s| {
            s.line(row + 1).contains("a branch named")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        screen.line(row).contains("to-delete"),
        "the box is closed: {:?}",
        screen.line(row)
    );
    assert!(
        !common::status_line_text(&screen).contains("a branch named"),
        "the refusal reached the status line"
    );
    assert!(branches(&repo).contains("hello"));
    // Esc closes it, nothing renamed.
    term.send_keys("Esc").unwrap();
    term.wait_for("the box gone", STEP, |s| !s.contains("a branch named"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(branches(&repo).contains("hello"));
    term.stop().unwrap();
}

/// Rename from the pill: the box is in the Ref cell of the tip's row.
#[test]
fn renaming_from_the_pill_types_in_the_log_row() {
    let repo = fixture();
    let (mut term, _c) = launch(&repo, "pty-rename-pill");
    let row = open_rename_from_pill(&mut term, "older target commit", "to-delete");
    erase(&mut term, "to-delete".len());
    term.send_text(&format!("{SENTINEL}-pill")).unwrap();
    let screen = term
        .wait_for("the typed name in the Log row", STEP, |s| {
            s.line(row).contains(&format!("{SENTINEL}-pill"))
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        screen
            .find(&format!("{SENTINEL}-pill"))
            .is_some_and(|(c, _)| c > 40),
        "the box is not in the Log: {:?}",
        screen.line(row)
    );
    assert!(
        !common::status_line_text(&screen).contains(SENTINEL),
        "the name was echoed in the status line"
    );
    term.send_keys("Enter").unwrap();
    for _ in 0..300 {
        if branches(&repo).contains(&format!("{SENTINEL}-pill")) {
            term.stop().unwrap();
            return;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    panic!("the branch was never renamed:\n{}", term.text());
}

/// The palette's rename with the Refs pane collapsed: the sidebar draws
/// nothing, so the box is in the branch's Log row.
#[test]
fn the_palette_rename_falls_back_to_the_pill_when_refs_is_collapsed() {
    let repo = fixture();
    let (mut term, _c) = launch(&repo, "pty-rename-palette");
    // Select the branch in the sidebar, then collapse the pane.
    let (col, row) = sidebar_cell(&term.screen(), "to-delete");
    term.click(col + 1, row).unwrap();
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text("Toggle Refs Pane").unwrap();
    term.wait_for_text("Toggle Refs Pane Collapsed", STEP)
        .unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the sidebar gone", STEP, |s| !s.contains("LOCAL"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text("Rename Selected Branch").unwrap();
    term.wait_for_text("Rename Selected Branch", STEP).unwrap();
    term.send_keys("Enter").unwrap();
    let (_, log_row) = term.wait_for_text("older target commit", STEP).unwrap();
    erase(&mut term, "to-delete".len());
    term.send_text(&format!("{SENTINEL}-pal")).unwrap();
    let screen = term
        .wait_for("the typed name in the Log row", STEP, |s| {
            s.line(log_row).contains(&format!("{SENTINEL}-pal"))
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        !common::status_line_text(&screen).contains(SENTINEL),
        "the name was echoed in the status line"
    );
    term.send_keys("Esc").unwrap();
    term.stop().unwrap();
}
