//! The Activity Log window opened from the status bar's button (the way a
//! user opens it): the title drag moves it, the bottom corner resizes it and
//! its list shows more rows, `Ctrl+N m` arranges it. Whole-state waits.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const ATTEMPT: Duration = Duration::from_secs(5);

fn title_at(s: &Screen, title: &str) -> Option<(u16, u16)> {
    s.find_all(title).into_iter().min_by_key(|(c, r)| (*r, *c))
}

fn grip(s: &Screen) -> Option<(u16, u16)> {
    s.find("\u{25e2}")
}

/// The rows of the window between its title and its bottom border that hold
/// text (what the list shows).
fn height_of_window(s: &Screen) -> Option<u16> {
    let (_, tr) = title_at(s, "Activity Log")?;
    let (_, gy) = grip(s)?;
    Some(gy - tr)
}

#[test]
fn the_activity_window_opened_from_the_status_button_moves_and_resizes() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "frames-commit")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(repo.root())
        .session("pty-activity-frame")
        .config_dir(config.path())
        .size(140, 40)
        .no_exit_pause()
        .ready("frames-commit")
        .mouse_capture()
        .spawn();
    // The status bar's button (the last row's right end).
    let (bx, by) = term
        .screen()
        .find_all("Activity")
        .into_iter()
        .max_by_key(|(c, r)| (*r, *c))
        .expect("the Activity button");
    term.act_until(
        "the Activity window with its grips",
        ATTEMPT,
        STEP,
        |t| t.click(bx + 1, by),
        |s| title_at(s, "Activity Log").is_some_and(|(_, r)| r < by) && grip(s).is_some(),
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let s = term.screen();
    let (tc, tr) = title_at(&s, "Activity Log").unwrap();
    let (gx, gy) = grip(&s).unwrap();
    let h0 = height_of_window(&s).unwrap();

    // Shrink first (the default fills the screen), then drag the title.
    term.act_until(
        "the window narrowed by its corner",
        ATTEMPT,
        STEP,
        |t| {
            if grip(&t.screen()) == Some((gx, gy)) {
                t.drag((gx, gy), (gx - 20, gy - 8))?;
            }
            Ok(())
        },
        |s| grip(s) == Some((gx - 20, gy - 8)) && title_at(s, "Activity Log") == Some((tc, tr)),
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (dx, dy) = (6u16, 3u16);
    term.act_until(
        "the window moved by its title",
        ATTEMPT,
        STEP,
        |t| {
            if let Some((c, r)) = title_at(&t.screen(), "Activity Log") {
                if (c, r) == (tc, tr) {
                    t.drag((c + 1, r), (c + 1 + dx, r + dy))?;
                }
            }
            Ok(())
        },
        |s| {
            title_at(s, "Activity Log") == Some((tc + dx, tr + dy))
                && grip(s) == Some((gx - 20 + dx, gy - 8 + dy))
        },
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The corner again: the window grows and the content follows its height.
    let g = (gx - 20 + dx, gy - 8 + dy);
    term.act_until(
        "the window taller by its corner",
        ATTEMPT,
        STEP,
        |t| {
            if grip(&t.screen()) == Some(g) {
                t.drag(g, (g.0 + 6, g.1 + 4))?;
            }
            Ok(())
        },
        |s| grip(s) == Some((g.0 + 6, g.1 + 4)),
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let h1 = height_of_window(&term.screen()).unwrap();
    assert!(h1 + 8 - 4 > h0 - 8 && h1 > h0 - 8, "grew: {h0} -> {h1}");

    // `Ctrl+N m`: the arrow moves it.
    term.send_keys("C-n").unwrap();
    term.send_text("m").unwrap();
    term.wait_for("the arrange banner", STEP, |s| s.contains("Arrange:"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Left").unwrap();
    term.wait_for("two columns left", STEP, |s| {
        title_at(s, "Activity Log") == Some((tc + dx - 2, tr + dy)) && s.contains("Arrange:")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.stop().unwrap();
}
