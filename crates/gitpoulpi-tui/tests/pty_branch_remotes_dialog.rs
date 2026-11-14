//! "Remotes of <branch>": the dialog every "Set upstream..." entry opens. One
//! row per remote of the repository with a Push and a Pull too checkbox; Apply
//! writes the branch's extra push targets and pull sources in one step, and
//! one Ctrl+Z takes it back.
//!
//! The fixture has three bare remotes: `origin` (the upstream of `feature`),
//! `gitlab` (has a copy of `feature`) and `backup` (never pushed to).
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded coordinate
//! (rows are found by the remote's name, controls by their text) and every
//! wait describes the whole state it is waiting for. Falsified: with the
//! Space toggle, the `p` toggle, the Apply request or the nested rename
//! prompt broken, the matching test below goes red.

mod common;

use std::time::{Duration, Instant};

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const PUSH_TO: &str = "branch.feature.gitpoulpi-push-to";
const PULL_FROM: &str = "branch.feature.gitpoulpi-pull-from";

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("base commit")
        .unwrap()
        .with_origin()
        .unwrap()
        .with_remote("gitlab")
        .unwrap()
        .with_remote("backup")
        .unwrap()
        .push("main")
        .unwrap()
        .branch("feature")
        .unwrap()
        .commit("feature commit")
        .unwrap()
        .push("feature")
        .unwrap()
        .push_to("gitlab", "feature")
        .unwrap()
        .track("feature", "origin")
        .unwrap()
        .build()
}

fn launch(repo: &FixtureRepo, session: &str) -> Terminal {
    let (term, config) = common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .no_exit_pause()
        .ready("feature commit")
        .start();
    std::mem::forget(config);
    term
}

fn values(repo: &FixtureRepo, key: &str) -> Vec<String> {
    repo.try_git(&["config", "--local", "--get-all", key])
        .unwrap_or_default()
        .lines()
        .map(str::to_string)
        .collect()
}

/// Waits for the key to hold `expected` (as a set: the file order follows the
/// table's rows, which is the remotes' order, not what is being checked).
fn wait_values(repo: &FixtureRepo, key: &str, expected: &[&str]) {
    let sorted = |mut v: Vec<String>| {
        v.sort();
        v
    };
    let wanted = sorted(expected.iter().map(|s| s.to_string()).collect());
    let deadline = Instant::now() + STEP;
    while sorted(values(repo, key)) != wanted {
        assert!(
            Instant::now() < deadline,
            "{key} is {:?}, wanted {expected:?}",
            values(repo, key)
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// The dialog with its rows read, and no palette or menu left over.
fn wait_dialog(term: &Terminal) -> Screen {
    term.wait_for("the Remotes dialog with its rows", STEP, |s| {
        s.contains("Remotes of feature")
            && s.contains("Pull too")
            && s.contains("Unset upstream")
            && s.contains("Rename on remote...")
            && !table_rows(s).is_empty()
            && !s.contains("Command Palette")
            && !s.contains("Enter to run")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()))
}

/// The checkboxes on screen row `y`, left to right: the column of each and
/// whether it is checked. Read cell by cell (`find_all` reports one hit per
/// row, and a row holds two boxes).
fn boxes_on(screen: &Screen, y: u16) -> Vec<(u16, bool)> {
    let text = |col: u16| {
        screen
            .cell(col, y)
            .map(|c| c.display().to_string())
            .unwrap_or_default()
    };
    (0..screen.cols().saturating_sub(2))
        .filter_map(|col| {
            let mid = text(col + 1);
            (text(col) == "[" && text(col + 2) == "]" && (mid == "x" || mid == " "))
                .then_some((col, mid == "x"))
        })
        .collect()
}

/// The table's rows, top to bottom: the remote's name and its checkbox marks
/// (`true` = checked) from left to right. A row of the table is the one with a
/// checkbox on it and the remote's name, so the sidebar's remote names never
/// count.
fn table_rows(screen: &Screen) -> Vec<(String, Vec<(u16, bool)>)> {
    let mut rows = Vec::new();
    for y in 0..screen.rows() {
        let boxes = boxes_on(screen, y);
        if boxes.is_empty() {
            continue;
        }
        let line = screen.line(y);
        if let Some(name) = ["origin", "gitlab", "backup"]
            .into_iter()
            .find(|n| line.contains(n))
        {
            rows.push((name.to_string(), boxes));
        }
    }
    rows
}

fn marks_of(screen: &Screen, remote: &str) -> Vec<(u16, bool)> {
    table_rows(screen)
        .into_iter()
        .find(|(n, _)| n == remote)
        .map(|(_, m)| m)
        .unwrap_or_default()
}

/// How many rows down `remote` is from the first row of the table.
fn index_of(screen: &Screen, remote: &str) -> usize {
    table_rows(screen)
        .iter()
        .position(|(n, _)| n == remote)
        .unwrap_or_else(|| panic!("no row for {remote}:\n{}", screen.text()))
}

/// Moves the table's selection from row `from` to row `to`.
fn move_rows(term: &mut Terminal, from: usize, to: usize) {
    let key = if to >= from { "Down" } else { "Up" };
    for _ in 0..from.abs_diff(to) {
        term.send_keys(key).unwrap();
    }
}

/// The Apply button: the hint line above the buttons says "Enter Apply" too,
/// so the button is the lowest occurrence.
fn apply_button(screen: &Screen) -> (u16, u16) {
    screen
        .find_all("Apply")
        .into_iter()
        .max_by_key(|(_, r)| *r)
        .expect("the Apply button")
}

fn ctrl_z_restores(term: &mut Terminal, repo: &FixtureRepo) {
    term.send_keys("C-z").unwrap();
    wait_values(repo, PUSH_TO, &[]);
    wait_values(repo, PULL_FROM, &[]);
}

/// From the palette, by keyboard alone: Space on a row checks Push, `p` checks
/// Pull too, Enter applies, Ctrl+Z restores.
#[test]
fn the_keyboard_toggles_push_and_pull_too_and_enter_applies_in_one_step() {
    let repo = fixture();
    let mut term = launch(&repo, "pty-remotes-keys");
    term.send_keys("C-p").unwrap();
    term.send_text("Git: Remotes of Branch").unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("Git: Remotes of Branch...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    let screen = wait_dialog(&term);
    // The upstream's own row has nothing to toggle but is the first row.
    let gitlab = index_of(&screen, "gitlab");
    let backup = index_of(&screen, "backup");
    move_rows(&mut term, 0, gitlab);
    term.send_keys("Space").unwrap();
    term.send_text("p").unwrap();
    let after = term
        .wait_for("gitlab's two boxes checked", STEP, |s| {
            marks_of(s, "gitlab").iter().all(|(_, on)| *on) && marks_of(s, "gitlab").len() == 2
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        marks_of(&after, "backup").iter().all(|(_, on)| !*on),
        "only the selected row changed:\n{}",
        after.text()
    );
    move_rows(&mut term, gitlab, backup);
    term.send_keys("Space").unwrap();
    term.wait_for("backup pushes too", STEP, |s| {
        marks_of(s, "backup").first().is_some_and(|(_, on)| *on)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the dialog closed", STEP, |s| {
        !s.contains("Remotes of feature")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    wait_values(&repo, PUSH_TO, &["gitlab", "backup"]);
    wait_values(&repo, PULL_FROM, &["gitlab"]);
    assert_eq!(
        values(&repo, "branch.feature.remote"),
        ["origin"],
        "the upstream is untouched"
    );
    ctrl_z_restores(&mut term, &repo);
}

/// From the Refs row's menu, by mouse alone: a click on a Push checkbox and on
/// a Pull too checkbox, then the Apply button.
#[test]
fn the_refs_row_menu_opens_it_and_the_mouse_toggles_both_boxes() {
    let repo = fixture();
    let mut term = launch(&repo, "pty-remotes-mouse");
    let sidebar = term
        .wait_for("the branch in the sidebar", STEP, |s| {
            s.contains("\u{2713}   feature")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    // The sidebar's own row (the status bar names the branch too).
    let (col, row) = sidebar
        .find("\u{2713}   feature")
        .unwrap_or_else(|| panic!("no LOCAL row:\n{}", sidebar.text()));
    term.right_click(col + 6, row).unwrap();
    let menu = term
        .wait_for("the Refs row menu", STEP, |s| {
            s.contains("Set upstream of feature...")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("Set upstream of feature...").unwrap();
    term.click(c + 1, r).unwrap();
    let screen = wait_dialog(&term);
    let marks = marks_of(&screen, "gitlab");
    assert_eq!(marks.len(), 2, "Push and Pull too:\n{}", screen.text());
    let (push_col, y) = (marks[0].0, table_row_y(&screen, "gitlab"));
    term.click(push_col + 1, y).unwrap();
    term.wait_for("gitlab's Push checked", STEP, |s| {
        marks_of(s, "gitlab").first().is_some_and(|(_, on)| *on)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (pull_col, y) = (marks[1].0, table_row_y(&screen, "gitlab"));
    term.click(pull_col + 1, y).unwrap();
    let both = term
        .wait_for("gitlab's two boxes checked", STEP, |s| {
            let m = marks_of(s, "gitlab");
            m.len() == 2 && m.iter().all(|(_, on)| *on)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = apply_button(&both);
    term.click(c + 1, r).unwrap();
    wait_values(&repo, PUSH_TO, &["gitlab"]);
    wait_values(&repo, PULL_FROM, &["gitlab"]);
    ctrl_z_restores(&mut term, &repo);
}

fn table_row_y(screen: &Screen, remote: &str) -> u16 {
    (0..screen.rows())
        .find(|y| !boxes_on(screen, *y).is_empty() && screen.line(*y).contains(remote))
        .unwrap_or_else(|| panic!("no table row for {remote}:\n{}", screen.text()))
}

/// From the branch's pill in the Log: `r` asks the branch name on the
/// selected remote in a prompt dialog, and Apply writes `remote:name`.
#[test]
fn the_pill_menu_opens_it_and_rename_on_remote_sets_the_branch_name() {
    let repo = fixture();
    let mut term = launch(&repo, "pty-remotes-pill");
    let (msg_col, row) = term.wait_for_text("feature commit", STEP).unwrap();
    let screen = term.screen();
    // The pill sits before the message on its row; the sidebar's own `feature`
    // may share that row but lies further left.
    let col = screen
        .find_all("feature")
        .into_iter()
        .filter(|(c, r)| *r == row && *c < msg_col)
        .map(|(c, _)| c)
        .max()
        .unwrap_or_else(|| panic!("no pill on the row:\n{}", screen.text()));
    term.right_click(col + 1, row).unwrap();
    let menu = term
        .wait_for("the pill menu", STEP, |s| {
            s.contains("Set upstream of feature...")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("Set upstream of feature...").unwrap();
    term.click(c + 1, r).unwrap();
    let screen = wait_dialog(&term);
    move_rows(&mut term, 0, index_of(&screen, "backup"));
    term.send_text("r").unwrap();
    let prompt = term
        .wait_for("the branch name prompt", STEP, |s| {
            s.contains("Branch name on backup") && s.contains("Set")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        !common::status_line_text(&prompt).contains("feature"),
        "the name is asked in a dialog, not the status line"
    );
    for _ in 0.."feature".len() {
        term.send_keys("BSpace").unwrap();
    }
    term.send_text("feat-x").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("back in the table, backup pushing", STEP, |s| {
        !s.contains("Branch name on backup")
            && s.contains("feat-x")
            && marks_of(s, "backup").first().is_some_and(|(_, on)| *on)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    wait_values(&repo, PUSH_TO, &["backup:feat-x"]);
    wait_values(&repo, PULL_FROM, &[]);
    ctrl_z_restores(&mut term, &repo);
}
