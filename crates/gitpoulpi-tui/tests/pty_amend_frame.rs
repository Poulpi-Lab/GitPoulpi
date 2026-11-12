//! The Amend commit dialog is movable and resizable like the other dialogs:
//! the title drag moves it, the bottom corner resizes it (the description
//! editor grows with the height), `Ctrl+N m` arranges it. Whole-state waits.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const ATTEMPT: Duration = Duration::from_secs(5);
const TITLE: &str = "Amend commit";

fn title_at(s: &Screen) -> Option<(u16, u16)> {
    s.find_all(TITLE).into_iter().min_by_key(|(c, r)| (*r, *c))
}

fn grip(s: &Screen) -> Option<(u16, u16)> {
    s.find("\u{25e2}")
}

fn open_amend(term: &mut Terminal) {
    let (col, row) = term.wait_for_text("target summary", STEP).unwrap();
    term.right_click(col + 1, row).unwrap();
    let menu = term
        .wait_for("the commit menu", STEP, |s| s.contains("Amend..."))
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("Amend...").unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("the dialog with its grips", STEP, |s| {
        s.contains(TITLE) && s.contains("Author name") && grip(s).is_some()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// The rows between the dialog's title and its bottom border.
fn height(s: &Screen) -> u16 {
    grip(s).unwrap().1 - title_at(s).unwrap().1
}

/// Falsified: with the dialog's `Win` movable-only (`Win::movable("reword")`)
/// it has no corner grip and the corner drag changes nothing.
#[test]
fn the_amend_dialog_moves_resizes_and_arranges() {
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
    let (mut term, _config) = common::launch(repo.root())
        .session("pty-amend-frame")
        .size(200, 60)
        .ready("top commit")
        .mouse_capture()
        .start();
    open_amend(&mut term);
    let s = term.screen();
    let (tc, tr) = title_at(&s).unwrap();
    let (gx, gy) = grip(&s).unwrap();
    let h0 = height(&s);

    let (dx, dy) = (6u16, 2u16);
    term.act_until(
        "the dialog moved by its title",
        ATTEMPT,
        STEP,
        |t| {
            if title_at(&t.screen()) == Some((tc, tr)) {
                t.drag((tc + 1, tr), (tc + 1 + dx, tr + dy))?;
            }
            Ok(())
        },
        |s| title_at(s) == Some((tc + dx, tr + dy)) && grip(s) == Some((gx + dx, gy + dy)),
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    let g = (gx + dx, gy + dy);
    term.act_until(
        "the dialog taller by its corner",
        ATTEMPT,
        STEP,
        |t| {
            if grip(&t.screen()) == Some(g) {
                t.drag(g, (g.0 + 4, g.1 + 5))?;
            }
            Ok(())
        },
        |s| grip(s) == Some((g.0 + 4, g.1 + 5)) && height(s) == h0 + 5,
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    term.send_keys("C-n").unwrap();
    term.send_text("m").unwrap();
    term.wait_for("the arrange banner", STEP, |s| s.contains("Arrange:"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Left").unwrap();
    term.wait_for("two columns left", STEP, |s| {
        title_at(s) == Some((tc + dx - 2, tr + dy)) && s.contains("Arrange:")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    let _ = term.stop();
}
