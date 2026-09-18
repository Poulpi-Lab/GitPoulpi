//! Pointer motion lights up exactly the thing under it and lets go when it
//! leaves; the wheel scrolls the diff under the pointer.
//!
//! Same rules as `pty_ui.rs`: nothing located by a hard-coded coordinate,
//! temp config dirs and fixture repositories.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Cell, Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);

fn launch(
    repo: &FixtureRepo,
    session: &str,
    config: &str,
    ready: &str,
) -> (Terminal, tempfile::TempDir) {
    let ready = ready.to_string();
    common::launch(repo.root())
        .session(session)
        .config(config)
        .size(200, 50)
        .ready_when("the fixture loaded", move |s| s.contains(&ready))
        .mouse_capture()
        .start()
}

fn cell_at(s: &Screen, c: u16, r: u16) -> Cell {
    s.cell(c, r).expect("a cell on screen").clone()
}

/// Hovering a toolbar button changes its look, and moving away restores
/// it; hovering another button moves the highlight with the pointer.
#[test]
fn a_toolbar_button_lights_up_under_the_pointer_and_lets_go() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("hover fixture")
        .unwrap()
        .build();
    let (mut term, _c) = launch(&repo, "pty-hover-toolbar", "", "hover fixture");
    let screen = term.screen();
    let (bx, by) = screen.find("Branch").unwrap();
    let (tx, ty) = screen.find("Tag").unwrap();
    let rest_branch = cell_at(&screen, bx + 1, by);
    let rest_tag = cell_at(&screen, tx + 1, ty);

    term.move_to(bx + 1, by).unwrap();
    term.wait_for("Branch lit", STEP, |s| {
        cell_at(s, bx + 1, by) != rest_branch && cell_at(s, tx + 1, ty) == rest_tag
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.move_to(tx + 1, ty).unwrap();
    term.wait_for("the highlight followed to Tag", STEP, |s| {
        cell_at(s, tx + 1, ty) != rest_tag && cell_at(s, bx + 1, by) == rest_branch
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Off everything: the status bar's far corner.
    term.move_to(screen.cols() - 3, screen.rows() - 2).unwrap();
    term.wait_for("both back at rest", STEP, |s| {
        cell_at(s, bx + 1, by) == rest_branch && cell_at(s, tx + 1, ty) == rest_tag
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

/// Hovering a Log row tints that row and no other; leaving clears it.
#[test]
fn a_log_row_is_tinted_under_the_pointer_only() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("older hover row")
        .unwrap()
        .commit("newer hover row")
        .unwrap()
        .build();
    let (mut term, _c) = launch(&repo, "pty-hover-log", "", "older hover row");
    let screen = term.screen();
    let (c, older) = screen.find("older hover row").unwrap();
    let (_, newer) = screen.find("newer hover row").unwrap();
    let rest_older = cell_at(&screen, c + 2, older);
    let rest_newer = cell_at(&screen, c + 2, newer);
    term.move_to(c + 2, older).unwrap();
    term.wait_for("only the older row tinted", STEP, |s| {
        cell_at(s, c + 2, older) != rest_older && cell_at(s, c + 2, newer) == rest_newer
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.move_to(screen.cols() - 3, screen.rows() - 2).unwrap();
    term.wait_for("the tint gone", STEP, |s| {
        cell_at(s, c + 2, older) == rest_older
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

/// The wheel over a long working-file diff scrolls the diff: the first
/// lines leave the top, the last come in, and the Log selection is not
/// what moved.
#[test]
fn the_wheel_scrolls_a_long_diff_under_the_pointer() {
    let original: String = (1..=150).map(|n| format!("line{n:03}\n")).collect();
    let changed = original.replace("line001", "CHANGED-FIRST");
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("long.txt", &original, "add long")
        .unwrap()
        .write_worktree("long.txt", changed.as_bytes())
        .unwrap()
        .build();
    let (mut term, _c) = launch(
        &repo,
        "pty-scroll-diff",
        "[ui]\ndiff_hunks_only = false\n",
        "add long",
    );
    let (c, r) = term.wait_for_text("\u{25CC}", STEP).unwrap();
    term.click(c, r).unwrap();
    let (c, r) = term.wait_for_text("\u{270E} long.txt", STEP).unwrap();
    term.click(c + 1, r).unwrap();
    let screen = term
        .wait_for("the whole file in the diff", STEP, |s| {
            s.contains("CHANGED-FIRST") && s.contains("line020")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (dc, dr) = screen.find("line010").unwrap();
    for _ in 0..8 {
        term.scroll(true, dc + 2, dr).unwrap();
    }
    term.wait_for("the diff scrolled down", STEP, |s| {
        !s.contains("CHANGED-FIRST") && s.contains("line060")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    for _ in 0..8 {
        term.scroll(false, dc + 2, dr).unwrap();
    }
    term.wait_for("the diff scrolled back to the top", STEP, |s| {
        s.contains("CHANGED-FIRST")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
