//! The name box: when the Log cannot show the commit's row (a diff is open
//! in the central column), the new-branch and new-tag names are typed in a
//! small framed box, not in the status line. The box has a close cross and
//! buttons, shows a refused name inside itself, and creates on Enter.
//! Same rules as `pty_ui.rs`: nothing is located by a hard-coded
//! coordinate, temp config dirs and fixture repositories.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const SENTINEL: &str = "zq9x";

fn fixture() -> FixtureRepo {
    let before: String = (1..=40).map(|n| format!("row {n:02}\n")).collect();
    let after = before.replace("row 20\n", "row 20 changed\n");
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("big.txt", &before, "forty rows")
        .unwrap()
        .commit_file("big.txt", &after, "change row 20")
        .unwrap()
        .build()
}

/// Launches with a diff open: the Log is not on screen then.
fn launch_in_a_diff(repo: &FixtureRepo, session: &str) -> Terminal {
    let (mut term, _config) = common::launch(repo.root())
        .session(session)
        .size(160, 50)
        .ready_when("the commit detail", |s| s.contains("big.txt"))
        .start();
    let (c, r) = term.wait_for_text("big.txt", STEP).unwrap();
    term.click(c, r).unwrap();
    term.wait_for("the diff in the center", STEP, |s| {
        s.contains("row 20 changed") && !s.contains("SHA")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The temp config dir lives as long as the terminal does.
    std::mem::forget(_config);
    term
}

/// The status line and everything under the panes: the last three rows.
fn status_line(s: &Screen) -> String {
    (s.rows() - 3..s.rows())
        .map(|r| s.line(r))
        .collect::<Vec<_>>()
        .join("\n")
}

fn open_branch_box(term: &mut Terminal) {
    term.send_keys("C-g").unwrap();
    term.wait_for_text("Ctrl+G:", STEP).unwrap();
    term.send_text("b").unwrap();
    term.wait_for("the name box", STEP, |s| {
        s.contains("New branch") && s.contains("enter branch name")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// A new branch named from a diff view: a framed box with its cross and
/// buttons, a refused name written inside it, Enter creating the branch.
#[test]
fn a_new_branch_from_a_diff_is_named_in_the_box_and_created() {
    let repo = fixture();
    let mut term = launch_in_a_diff(&repo, "pty-name-box-branch");
    open_branch_box(&mut term);
    let screen = term.screen();
    assert!(
        screen.contains("\u{2715}") && screen.contains("Cancel") && screen.contains("Create"),
        "the box has no cross or buttons:\n{}",
        term.text()
    );

    // A name git refuses: the reason is inside the box, the box stays.
    term.send_text(&format!("{SENTINEL} bad")).unwrap();
    term.send_keys("Enter").unwrap();
    let screen = term
        .wait_for("the refusal inside the box", STEP, |s| {
            s.contains("can't contain spaces")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        !status_line(&screen).contains(SENTINEL) && !status_line(&screen).contains("spaces"),
        "the typed name or its refusal reached the status line: {:?}",
        status_line(&screen)
    );
    let (_, box_row) = screen.find("can't contain spaces").unwrap();
    let (_, typed_row) = screen.find(SENTINEL).unwrap();
    assert!(typed_row < box_row, "the refusal is not under the field");

    // Fix it and create.
    term.send_keys("Home").unwrap();
    term.send_keys("Delete").unwrap();
    for _ in 0..SENTINEL.len() {
        term.send_keys("Delete").unwrap();
    }
    term.send_keys("End").unwrap();
    term.send_text("-ok").unwrap();
    term.wait_for("the corrected name", STEP, |s| {
        s.contains("bad-ok") && !s.contains("can't contain spaces")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    for _ in 0..300 {
        if repo.git(&["branch", "--list"]).contains("bad-ok") {
            term.stop().unwrap();
            return;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    panic!("the branch was never created:\n{}", term.text());
}

/// The cross and Cancel close the box and create nothing.
#[test]
fn the_cross_and_cancel_close_the_box() {
    let repo = fixture();
    let mut term = launch_in_a_diff(&repo, "pty-name-box-close");
    open_branch_box(&mut term);
    term.send_text(&format!("{SENTINEL}-gone")).unwrap();
    let screen = term.screen();
    // The diff and the Commits pane have crosses too: the box's is on its
    // title row.
    let (_, title_row) = screen.find("New branch").unwrap();
    let (c, r) = screen
        .find_all("\u{2715}")
        .into_iter()
        .find(|(_, r)| *r == title_row)
        .unwrap_or_else(|| panic!("no cross on the box:\n{}", term.text()));
    term.click(c, r).unwrap();
    term.wait_for("the box closed", STEP, |s| !s.contains(SENTINEL))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    open_branch_box(&mut term);
    let (c, r) = term.screen().find("Cancel").unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("the box closed again", STEP, |s| {
        !s.contains("enter branch name")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!repo.git(&["branch", "--list"]).contains(SENTINEL));
    term.stop().unwrap();
}

/// A new tag is named in the same box, and its Create button works.
#[test]
fn a_new_tag_from_a_diff_is_named_in_the_box_and_the_button_creates_it() {
    let repo = fixture();
    let mut term = launch_in_a_diff(&repo, "pty-name-box-tag");
    term.send_keys("C-g").unwrap();
    term.wait_for_text("Ctrl+G:", STEP).unwrap();
    term.send_text("t").unwrap();
    term.wait_for("the tag box", STEP, |s| s.contains("enter tag name"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("v-box").unwrap();
    let screen = term
        .wait_for("the typed tag", STEP, |s| s.contains("v-box"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!status_line(&screen).contains("v-box"));
    let (c, r) = screen.find("Create").unwrap();
    term.click(c + 1, r).unwrap();
    for _ in 0..300 {
        if repo.git(&["tag", "--list"]).contains("v-box") {
            term.stop().unwrap();
            return;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    panic!("the tag was never created:\n{}", term.text());
}
