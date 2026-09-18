//! `Ctrl+N` badges over the amend dialog and the history rewrite workbench:
//! `Ctrl+N` shows a one-character badge on each field and button, and
//! pressing the character clicks it. Real binary and daemon through a pty
//! (`xtask::pty`); temp config dirs only.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);

/// Which side of its label a control's badge is on: a button's is on its own
/// left edge; a field's is left of the field, so right of the label.
#[derive(Clone, Copy)]
enum Side {
    Left,
    Right,
}

/// The badge character drawn next to `label`, on the row that holds it.
fn badge_of(screen: &Screen, label: &str, side: Side) -> Option<char> {
    let (col, row) = screen.find(label)?;
    let line: Vec<char> = screen.line(row).chars().collect();
    let col = usize::from(col).min(line.len());
    let is_badge = |i: usize| {
        i + 2 < line.len()
            && line[i] == ' '
            && line[i + 2] == ' '
            && line[i + 1].is_ascii_alphanumeric()
    };
    let at = match side {
        Side::Left => (col.saturating_sub(7)..col).rev().find(|&i| is_badge(i)),
        Side::Right => {
            let end = col + label.chars().count();
            (end..end + 24).find(|&i| is_badge(i))
        }
    };
    at.map(|i| line[i + 1])
}

fn wait(term: &mut Terminal, what: &str, f: impl Fn(&Screen) -> bool) {
    term.wait_for(what, STEP, f)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// `Ctrl+N`, then the badge of the control next to `label`.
fn press_badge(term: &mut Terminal, label: &str, side: Side) {
    term.send_keys("C-n").unwrap();
    let l = label.to_string();
    wait(term, "the badge on the control", move |s| {
        badge_of(s, &l, side).is_some()
    });
    let badge = badge_of(&term.screen(), label, side).expect("badge");
    term.send_text(&badge.to_string()).unwrap();
}

/// base <- target <- top; the author the fixture commits with.
fn amend_fixture() -> (FixtureRepo, String) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("base commit")
        .unwrap()
        .commit("target summary")
        .unwrap()
        .build();
    std::fs::write(repo.root().join("top.txt"), "top").unwrap();
    repo.git(&["add", "top.txt"]);
    repo.git(&["commit", "-m", "top commit"]);
    let author = repo.git(&["log", "-1", "--format=%an"]).trim().to_string();
    (repo, author)
}

fn open_amend(term: &mut Terminal) {
    let (col, row) = term.wait_for_text("target summary", STEP).unwrap();
    term.right_click(col + 1, row).unwrap();
    let menu = term
        .wait_for("the commit menu", STEP, |s| s.contains("Amend..."))
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("Amend...").unwrap();
    term.click(c + 1, r).unwrap();
    wait(term, "the dialog", |s| {
        s.contains("Amend commit") && s.contains("Author name") && s.contains("Apply")
    });
}

#[test]
fn amend_badges_focus_a_field_and_press_apply() {
    let (repo, author) = amend_fixture();
    let (mut term, _cfg) = common::launch(repo.root())
        .session("pty-badges-amend")
        .size(200, 50)
        .ready("top commit")
        .mouse_capture()
        .start();
    open_amend(&mut term);
    // The Author name field: typing after its badge lands in it.
    press_badge(&mut term, "Author name", Side::Right);
    term.send_text("Zed").unwrap();
    wait(&mut term, "the text in the author field", |s| {
        s.contains(&format!("Zed{author}"))
    });
    // Apply presses the button.
    press_badge(&mut term, "ply", Side::Left);
    wait(&mut term, "the amend reported", |s| {
        s.contains("amended") && !s.contains("Author email")
    });
    assert_eq!(
        repo.git(&["log", "-1", "--format=%an", "HEAD~1"]),
        format!("Zed{author}")
    );
    let _ = term.stop();
}

#[test]
fn amend_badge_mode_is_cancelled_by_escape_and_cancel_closes() {
    let (repo, _author) = amend_fixture();
    let (mut term, _cfg) = common::launch(repo.root())
        .session("pty-badges-amend-esc")
        .size(200, 50)
        .ready("top commit")
        .mouse_capture()
        .start();
    open_amend(&mut term);
    term.send_keys("C-n").unwrap();
    wait(&mut term, "badges", |s| {
        badge_of(s, "ncel", Side::Left).is_some()
    });
    term.send_keys("Escape").unwrap();
    wait(&mut term, "no badges, dialog still open", |s| {
        badge_of(s, "ncel", Side::Left).is_none() && s.contains("Amend commit")
    });
    press_badge(&mut term, "ncel", Side::Left);
    wait(&mut term, "the dialog closed", |s| {
        !s.contains("Author email")
    });
    let _ = term.stop();
}

fn rewrite_fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "base")
        .unwrap()
        .commit_file("b.txt", "two\n", "second")
        .unwrap()
        .build()
}

fn open_workbench(repo: &FixtureRepo, session: &str) -> (Terminal, tempfile::TempDir) {
    let config = tempfile::tempdir().expect("temp config dir");
    let mut term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(200, 50)
        .spawn();
    term.wait_for("the fixture loaded", Duration::from_secs(60), |s| {
        s.contains("second")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-g").unwrap();
    term.send_text("H").unwrap();
    wait(&mut term, "the workbench", |s| {
        s.contains("Rewrite history") && s.contains("Add a rule to start")
    });
    (term, config)
}

#[test]
fn workbench_badges_open_the_add_menu_and_press_load() {
    let repo = rewrite_fixture();
    let (mut term, _config) = open_workbench(&repo, "pty-badges-rewrite");
    // "+ Add rule" opens its menu.
    press_badge(&mut term, "dd rule", Side::Left);
    wait(&mut term, "the Add menu", |s| {
        s.contains("Replace an author") && s.contains("Authors")
    });
    // A dropdown is open: Ctrl+N gives no badges.
    term.send_keys("C-n").unwrap();
    wait(&mut term, "the menu still open, no badge", |s| {
        s.contains("Replace an author") && badge_of(s, "Replace an author", Side::Left).is_none()
    });
    term.send_keys("Escape").unwrap();
    wait(&mut term, "the menu closed", |s| {
        !s.contains("Replace an author")
    });
    // Esc on the badges leaves the workbench open.
    term.send_keys("C-n").unwrap();
    wait(&mut term, "badges", |s| {
        badge_of(s, "dd rule", Side::Left).is_some()
    });
    term.send_keys("Escape").unwrap();
    wait(&mut term, "no badges, workbench open", |s| {
        badge_of(s, "dd rule", Side::Left).is_none() && s.contains("Rewrite history")
    });
    // Load is a button like the others: its badge presses it.
    press_badge(&mut term, "d ▾", Side::Left);
    wait(&mut term, "the answer of Load", |s| {
        s.contains("No saved recipe yet")
    });
    let _ = term.stop();
}
