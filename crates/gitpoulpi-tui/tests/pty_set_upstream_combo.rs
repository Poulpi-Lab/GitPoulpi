//! The upstream of a branch is picked in the "Remotes of <branch>" dialog,
//! through a searchable list of remote branches (and a typed `remote/branch`),
//! never a menu with a footer prompt behind it: the palette's `Git: Remotes
//! of Branch...` (still found as "Git: Set Upstream") opens it, with the list
//! open when the branch has no upstream; typing filters, Enter picks, a
//! second Enter (or the Apply button) applies; Unset upstream is a button;
//! Esc closes the list first, then the dialog. The list's behaviour is the
//! one the Set upstream dialog had; the table of remotes under it has its
//! own test (`pty_branch_remotes_dialog.rs`).
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate and every wait describes the whole state it is waiting for.
//! Falsified: with the old menu back, the dialog title is never drawn.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);

/// `feature` is checked out with no upstream; `origin` has `main` and `feature`.
fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("base commit")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .branch("feature")
        .unwrap()
        .push("feature")
        .unwrap()
        .build()
}

fn launch(repo: &FixtureRepo, session: &str) -> Terminal {
    let (term, config) = common::launch(repo.root())
        .session(session)
        .size(160, 50)
        .no_exit_pause()
        .ready("base commit")
        .start();
    std::mem::forget(config);
    term
}

fn upstream(repo: &FixtureRepo) -> String {
    repo.git(&[
        "for-each-ref",
        "--format=%(upstream:short)",
        "refs/heads/feature",
    ])
    .trim()
    .to_string()
}

fn wait_upstream(repo: &FixtureRepo, expected: &str) {
    let deadline = std::time::Instant::now() + STEP;
    while upstream(repo) != expected {
        assert!(
            std::time::Instant::now() < deadline,
            "upstream is {:?}, wanted {expected:?}",
            upstream(repo)
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// Opens the dialog through the palette (typing the old name: it is still
/// an alias of the command), the list open: the branch has no upstream.
fn open_dialog(term: &mut Terminal) -> Screen {
    open_palette_row(term);
    // The whole state: the dialog with its list open, and no palette left
    // (a frame torn between the palette and the dialog fails this and is
    // polled again, instead of being returned half-painted).
    term.wait_for("the Remotes dialog with its list open", STEP, |s| {
        s.contains("Remotes of feature")
            && s.contains("origin/feature")
            && s.contains("origin/main")
            && !s.contains("Command Palette")
            && !s.contains("Enter to run")
    })
    .unwrap_or_else(|e| panic!("{e}"))
}

fn open_palette_row(term: &mut Terminal) {
    term.send_keys("C-p").expect("open the palette");
    term.send_text("Git: Set Upstream").unwrap();
    term.wait_for("the Remotes of Branch command", STEP, |s| {
        s.contains("Git: Remotes of Branch...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
}

/// Opens the dialog of a branch that has an upstream: no list, the table.
fn open_dialog_with_upstream(term: &mut Terminal) -> Screen {
    open_palette_row(term);
    term.wait_for("the Remotes dialog, rows read", STEP, |s| {
        s.contains("Remotes of feature")
            && s.contains("Unset upstream")
            && s.contains("Pull too")
            && !s.contains("Command Palette")
            && !s.contains("Enter to run")
    })
    .unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn the_list_filters_picks_and_the_set_button_applies() {
    let repo = fixture();
    let mut term = launch(&repo, "pty-upstream-pick");
    let open = open_dialog(&mut term);
    assert!(
        open.contains("origin/feature") && open.contains("origin/main"),
        "the list opens with every remote branch:\n{}",
        open.text()
    );
    // Typing filters the list; the text lives in the dialog, not the status line.
    term.send_text("zq9x").unwrap();
    let none = term
        .wait_for("the typed filter", STEP, |s| s.contains("zq9x"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!common::status_line_text(&none).contains("zq9x"));
    for _ in 0..4 {
        term.send_keys("BSpace").unwrap();
    }
    term.send_text("main").unwrap();
    term.wait_for("the list narrowed to origin/main", STEP, |s| {
        s.contains("origin/main") && !s.contains("origin/feature")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    // Picked: the list is closed (the table and the buttons are visible again).
    let screen = term
        .wait_for("the choice made, the list closed", STEP, |s| {
            s.contains("Remotes of feature")
                && s.contains("Cancel")
                && s.contains("Pull too")
                && s.contains("origin/main")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    // The Apply button applies it.
    // The hint line above the buttons says "Enter Apply" too: the button is
    // the lowest occurrence.
    let (col, row) = screen
        .find_all("Apply")
        .into_iter()
        .max_by_key(|(_, r)| *r)
        .expect("the Apply button");
    term.click(col + 1, row).unwrap();
    wait_upstream(&repo, "origin/main");
}

#[test]
fn unset_upstream_is_a_button_and_esc_closes_the_list_then_the_dialog() {
    let repo = fixture();
    repo.git(&["branch", "-u", "origin/main", "feature"]);
    let mut term = launch(&repo, "pty-upstream-unset");
    // With an upstream the list stays shut and the keyboard starts in the table.
    let open = open_dialog_with_upstream(&mut term);
    assert!(
        !open.contains("origin/feature"),
        "the list is closed:\n{}",
        open.text()
    );
    // Enter on the list field opens it; Esc closes the list first, the
    // dialog stays...
    term.send_keys("BTab").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the list open", STEP, |s| {
        s.contains("origin/feature") && s.contains("Remotes of feature")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for("the list closed, the dialog open", STEP, |s| {
        s.contains("Remotes of feature")
            && s.contains("Unset upstream")
            && !s.contains("origin/feature")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // ...then the dialog itself.
    term.send_keys("Escape").unwrap();
    term.wait_for("the dialog closed", STEP, |s| {
        !s.contains("Remotes of feature")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(upstream(&repo), "origin/main", "nothing changed");

    let screen = open_dialog_with_upstream(&mut term);
    let (col, row) = screen.find("Unset upstream").unwrap();
    term.click(col + 1, row).unwrap();
    wait_upstream(&repo, "");
}

#[test]
fn a_typed_value_must_name_a_known_remote() {
    let repo = fixture();
    let mut term = launch(&repo, "pty-upstream-custom");
    open_dialog(&mut term);
    term.send_text("nowhere/x").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the reason inside the dialog", STEP, |s| {
        s.contains("No remote named 'nowhere'") && s.contains("Remotes of feature")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(upstream(&repo), "", "nothing was set");
    // A known remote with a new branch name is accepted (git sets it up even
    // before the remote branch exists): list, type, Enter picks the custom row.
    term.send_keys("Enter").unwrap();
    term.send_text("origin/main").unwrap();
    term.send_keys("Enter").unwrap();
    term.send_keys("Enter").unwrap();
    wait_upstream(&repo, "origin/main");
}
