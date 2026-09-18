//! End-to-end interactive rebase / cherry-pick flows through a pty: the
//! real binary, the real daemon, real `git`. Same rules as `pty_ui.rs`.
//!
//! "Interactively rebase from here" must send the clicked commit itself
//! (not its parent) as the upstream, or the backend refuses every Start
//! ("the plan is not exactly a reordering") and only the status bar says so.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

fn five_commits() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "c1 base")
        .unwrap()
        .commit_file("b.txt", "b\n", "c2 second")
        .unwrap()
        .commit_file("c.txt", "c\n", "c3 third")
        .unwrap()
        .commit_file("d.txt", "d\n", "c4 fourth")
        .unwrap()
        .commit_file("e.txt", "e\n", "c5 fifth")
        .unwrap()
        .build()
}

fn open(repo: &FixtureRepo, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .ready_when("the fixture loaded", |s| {
            s.contains("c5 fifth") && s.contains("c1 base")
        })
        .start()
}

fn subjects(repo: &FixtureRepo) -> Vec<String> {
    repo.git(&["log", "--format=%s"])
        .lines()
        .map(str::to_string)
        .collect()
}

/// Polls the repository (the rebase runs in the daemon, not on screen).
fn wait_subjects(repo: &FixtureRepo, want: &[&str]) {
    let want: Vec<String> = want.iter().map(|s| s.to_string()).collect();
    for _ in 0..300 {
        if subjects(repo) == want && !repo.root().join(".git/rebase-merge").exists() {
            return;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    assert_eq!(subjects(repo), want);
}

fn open_rebase_view(term: &mut Terminal, target: &str) {
    let (col, row) = term.wait_for_text(target, STEP).unwrap();
    term.right_click(col + 1, row).expect("right-click");
    let menu = term
        .wait_for("the commit menu", STEP, |s| {
            s.contains("Interactively rebase from here...")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("Interactively rebase from here...").unwrap();
    term.click(c + 1, r).expect("click the entry");
    term.wait_for("the rebase view", STEP, |s| s.contains("Start rebase"))
        .unwrap_or_else(|e| panic!("{e}"));
}

// Alt+Up/Down reordering is covered by `rebase_ui`'s unit tests: the pty
// harness's ESC-prefixed Alt chords reach the TUI as a bare Esc under
// ConPTY, which would just close the view.

/// (a) menu -> the view opens on the commits after the clicked one; drop,
/// reorder (Alt+Down) and squash, then Start with Ctrl+S.
#[test]
fn menu_entry_reorders_drops_and_squashes() {
    let repo = five_commits();
    let (mut term, _c) = open(&repo, "pty-rebase-a");
    open_rebase_view(&mut term, "c1 base");
    // Rows newest first: c5 c4 c3 c2.
    term.send_keys("d").unwrap(); // c5 dropped
    term.send_keys("Down").unwrap(); // cursor on c4
    term.send_keys("s").unwrap(); // c4 squashed into c3
    term.send_keys("C-s").unwrap();
    wait_subjects(&repo, &["c3 third", "c2 second", "c1 base"]);
    assert_eq!(repo.git(&["status", "--porcelain"]), "");
    term.wait_for("the view closed", STEP, |s| !s.contains("Start rebase"))
        .unwrap_or_else(|e| panic!("{e}"));
}

/// Start via the button click.
#[test]
fn start_button_click_runs_the_plan() {
    let repo = five_commits();
    let (mut term, _c) = open(&repo, "pty-rebase-b");
    open_rebase_view(&mut term, "c2 second");
    term.send_keys("d").unwrap();
    let (c, r) = term.wait_for_text("Start rebase", STEP).unwrap();
    term.click(c + 2, r).unwrap();
    wait_subjects(&repo, &["c4 fourth", "c3 third", "c2 second", "c1 base"]);
}

/// (b) palette entry, with the commit selected first; reword by typing.
#[test]
fn palette_entry_opens_the_view_and_reword_applies() {
    let repo = five_commits();
    let (mut term, _c) = open(&repo, "pty-rebase-c");
    let (col, row) = term.wait_for_text("c2 second", STEP).unwrap();
    term.click(col + 1, row).unwrap();
    term.send_keys("C-p").unwrap();
    term.send_text("Interactively Rebase").unwrap();
    term.wait_for("the palette narrowed", STEP, |s| {
        s.contains("Command Palette (1/")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the rebase view", STEP, |s| s.contains("Start rebase"))
        .unwrap_or_else(|e| panic!("{e}"));
    // Cursor on the newest row (c5): reword it.
    term.send_keys("r").unwrap(); // choosing Reword opens the editor
    term.wait_for("the message editor", STEP, |s| s.contains("c5 fifth"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text(" (renamed)").unwrap();
    term.send_keys("C-s").unwrap();
    term.send_keys("C-s").unwrap();
    wait_subjects(
        &repo,
        &[
            "c5 fifth (renamed)",
            "c4 fourth",
            "c3 third",
            "c2 second",
            "c1 base",
        ],
    );
}

/// Mouse drag reorders rows.
#[test]
fn dragging_a_row_reorders_the_plan() {
    let repo = five_commits();
    let (mut term, _c) = open(&repo, "pty-rebase-d");
    open_rebase_view(&mut term, "c1 base");
    let screen = term.screen();
    let (c4c, c4r) = screen.find("c4 fourth").unwrap();
    let (_, c3r) = screen.find("c3 third").unwrap();
    term.drag((c4c, c4r), (c4c, c3r)).unwrap();
    term.wait_for("c4 below c3", STEP, |s| {
        s.find("c4 fourth").map(|p| p.1) > s.find("c3 third").map(|p| p.1)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-s").unwrap();
    wait_subjects(
        &repo,
        &["c5 fifth", "c3 third", "c4 fourth", "c2 second", "c1 base"],
    );
}

/// Double-clicking a row opens its reword editor (the row becomes a
/// Reword); Enter is the keyboard twin, also on a row that is a plain Pick.
#[test]
fn double_click_and_enter_open_the_reword_editor_on_any_row() {
    let repo = five_commits();
    let (mut term, _c) = open(&repo, "pty-rebase-dblclick");
    open_rebase_view(&mut term, "c1 base");
    let (col, row) = term.wait_for_text("c4 fourth", STEP).unwrap();
    term.double_click(col + 1, row).unwrap();
    term.wait_for("the message editor", STEP, |s| {
        s.contains("Reword commit message")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text(" (dbl)").unwrap();
    term.send_keys("C-s").unwrap();
    term.wait_for("editor closed", STEP, |s| {
        !s.contains("Reword commit message") && s.contains("c4 fourth (dbl)")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Enter on the (Pick) first row.
    term.send_keys("Up").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the editor on the Pick row", STEP, |s| {
        s.contains("Reword commit message")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text(" (enter)").unwrap();
    term.send_keys("C-s").unwrap();
    term.wait_for("editor closed", STEP, |s| {
        !s.contains("Reword commit message")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-s").unwrap();
    wait_subjects(
        &repo,
        &[
            "c5 fifth (enter)",
            "c4 fourth (dbl)",
            "c3 third",
            "c2 second",
            "c1 base",
        ],
    );
}
