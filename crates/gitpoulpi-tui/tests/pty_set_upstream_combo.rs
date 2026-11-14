//! Set upstream is a dialog with a searchable list of remote branches (and a
//! typed `remote/branch`), never a menu with a footer prompt behind it: the
//! palette's `Git: Set Upstream...` opens it with the list open, typing
//! filters, Enter picks, a second Enter (or the Set button) applies; Unset
//! upstream is a button; Esc closes the list first, then the dialog.
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

/// Opens the dialog through the palette.
fn open_dialog(term: &mut Terminal) -> Screen {
    term.send_keys("C-p").expect("open the palette");
    term.send_text("Git: Set Upstream").unwrap();
    term.wait_for("the Set Upstream command", STEP, |s| {
        s.contains("Git: Set Upstream...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    // The whole state: the dialog with its list open, and no palette left
    // (a frame torn between the palette and the dialog fails this and is
    // polled again, instead of being returned half-painted).
    term.wait_for("the Set upstream dialog with its list open", STEP, |s| {
        s.contains("Set upstream of feature")
            && s.contains("origin/feature")
            && s.contains("origin/main")
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
    // Picked: the list is closed (the buttons are visible again).
    let screen = term
        .wait_for("the choice made, the list closed", STEP, |s| {
            s.contains("Set upstream of feature")
                && s.contains("Cancel")
                && s.contains("origin/main")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    // The Set button applies it.
    let (_, row) = screen.find("Cancel").unwrap();
    let (col, _) = screen
        .find_all("Set")
        .into_iter()
        .filter(|(_, r)| *r == row)
        .max_by_key(|(c, _)| *c)
        .expect("the Set button");
    term.click(col + 1, row).unwrap();
    wait_upstream(&repo, "origin/main");
}

#[test]
fn unset_upstream_is_a_button_and_esc_closes_the_list_then_the_dialog() {
    let repo = fixture();
    repo.git(&["branch", "-u", "origin/main", "feature"]);
    let mut term = launch(&repo, "pty-upstream-unset");
    let open = open_dialog(&mut term);
    assert!(
        open.contains("origin/feature"),
        "the list is open:\n{}",
        open.text()
    );
    // Esc: the list first (the Unset button shows again), the dialog stays...
    term.send_keys("Escape").unwrap();
    term.wait_for("the list closed, the dialog open", STEP, |s| {
        s.contains("Set upstream of feature")
            && s.contains("Unset upstream")
            && !s.contains("origin/feature")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // ...then the dialog itself.
    term.send_keys("Escape").unwrap();
    term.wait_for("the dialog closed", STEP, |s| {
        !s.contains("Set upstream of feature")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(upstream(&repo), "origin/main", "nothing changed");

    open_dialog(&mut term);
    term.send_keys("Escape").unwrap();
    let screen = term
        .wait_for("the list closed, the Unset button back", STEP, |s| {
            s.contains("Set upstream of feature")
                && s.contains("Unset upstream")
                && !s.contains("origin/feature")
        })
        .unwrap_or_else(|e| panic!("{e}"));
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
        s.contains("No remote named 'nowhere'") && s.contains("Set upstream of feature")
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
