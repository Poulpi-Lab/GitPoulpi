//! Settings' scroll arrows: hovering an end arrow lights it (the event loop
//! clears the scrollbars' pointer while Settings owns the mouse, so Settings
//! hands its own), and a click on the bottom arrow scrolls the page instead
//! of reaching the status bar's Activity button underneath (which opened the
//! Activity Log).

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const UP: &str = "\u{2191}";
const DOWN: &str = "\u{2193}";

fn open_settings() -> (Terminal, tempfile::TempDir, gitpoulpi_testkit::FixtureRepo) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "x\n", "one")
        .unwrap()
        .build();
    let (mut term, config) = common::launch(repo.root())
        .no_commit_column()
        .session("pty-settings-scroll")
        .size(120, 30)
        .no_exit_pause()
        .ready("one")
        .mouse_capture()
        .start();
    let (c, r) = term.wait_for_text("\u{2699}", STEP).unwrap();
    term.click(c, r).unwrap();
    term.wait_for("Settings", STEP, |s| s.contains("Repo-specific settings"))
        .unwrap();
    common::settings_page(&mut term, "General");
    (term, config, repo)
}

fn arrow(term: &Terminal, glyph: &str) -> (u16, u16) {
    // The bar is the right-most column: the glyph with the largest column.
    // Waited for: a torn frame can lack the bar.
    let screen = term
        .wait_for(
            &format!("the {glyph} arrow"),
            Duration::from_secs(30),
            |s| !s.find_all(glyph).is_empty(),
        )
        .unwrap_or_else(|e| panic!("no {glyph} arrow: {e}\n{}", term.text()));
    screen
        .find_all(glyph)
        .into_iter()
        .max_by_key(|(c, _)| *c)
        .expect("the predicate just matched")
}

fn bg(term: &Terminal, (c, r): (u16, u16)) -> String {
    format!("{:?}", term.screen().cell(c, r).unwrap().bg)
}

#[test]
fn hovering_either_settings_arrow_lights_it() {
    let (mut term, _config, _repo) = open_settings();
    // Scroll down so both arrows are live.
    let down = arrow(&term, DOWN);
    term.click(down.0, down.1).unwrap();
    // The scroll's final frame: read once the page stopped redrawing.
    term.wait_for_stable(Duration::from_millis(400), Duration::from_secs(30))
        .unwrap_or_else(|e| panic!("{e}"));
    let up = arrow(&term, UP);
    let down = arrow(&term, DOWN);
    term.move_to(up.0 - 8, up.1 + 2).unwrap();
    term.wait_for_stable(Duration::from_millis(300), Duration::from_secs(30))
        .unwrap_or_else(|e| panic!("{e}"));
    let (up_rest, down_rest) = (bg(&term, up), bg(&term, down));
    term.move_to(up.0, up.1).unwrap();
    term.wait_for("the up arrow lit", Duration::from_secs(30), |s| {
        format!("{:?}", s.cell(up.0, up.1).unwrap().bg) != up_rest
    })
    .unwrap_or_else(|e| panic!("up arrow not lit: {e}\n{}", term.text()));
    term.move_to(down.0, down.1).unwrap();
    term.wait_for("the down arrow lit", Duration::from_secs(30), |s| {
        format!("{:?}", s.cell(down.0, down.1).unwrap().bg) != down_rest
    })
    .unwrap_or_else(|e| panic!("down arrow not lit: {e}\n{}", term.text()));
}

#[test]
fn the_bottom_arrow_scrolls_and_never_opens_the_activity_log() {
    let (mut term, _config, _repo) = open_settings();
    for _ in 0..40 {
        let down = arrow(&term, DOWN);
        term.click(down.0, down.1).unwrap();
        std::thread::sleep(Duration::from_millis(30));
    }
    std::thread::sleep(Duration::from_millis(300));
    let text = term.text();
    assert!(
        text.contains("Repo-specific settings") && !text.contains("Git operations"),
        "left Settings\n{text}"
    );
}

/// kitty paints U+2699 as an empty cell, so under kitty the gear is U+26ED
/// (not emoji-capable); anywhere else it stays U+2699. The gear opens
/// Settings either way.
#[test]
fn the_gear_is_a_glyph_kitty_draws_and_still_opens_settings() {
    for (kitty, want, other) in [
        (true, "\u{26ED}", "\u{2699}"),
        (false, "\u{2699}", "\u{26ED}"),
    ] {
        let repo = FixtureRepoBuilder::new()
            .unwrap()
            .commit_file("a.txt", "x\n", "one")
            .unwrap()
            .build();
        let mut launch = common::launch(repo.root())
            .no_commit_column()
            .session("pty-gear-kitty")
            .size(120, 30)
            .no_exit_pause()
            .ready("one")
            .mouse_capture();
        if kitty {
            launch = launch.env("KITTY_WINDOW_ID", "1");
        }
        let (mut term, _config) = launch.start();
        let (c, r) = term.wait_for_text(want, STEP).unwrap();
        let screen = term.screen();
        assert!(!screen.contains(other), "kitty={kitty}\n{}", term.text());
        term.click(c, r).unwrap();
        term.wait_for("Settings", STEP, |s| s.contains("Repo-specific settings"))
            .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    }
}

/// Settings remembers each page's scroll: another page and back, and a
/// close and reopen, both land where the page was left, not at its top.
#[test]
fn a_page_keeps_its_scroll_across_pages_and_a_reopen() {
    let (mut term, _config, _repo) = open_settings();
    common::settings_page(&mut term, "Appearance");
    term.wait_for("the Appearance page from its top", STEP, |s| {
        s.contains("Colors of the whole interface")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    for _ in 0..40 {
        let down = arrow(&term, DOWN);
        term.click(down.0, down.1).unwrap();
        std::thread::sleep(Duration::from_millis(30));
    }
    term.wait_for("the page scrolled", STEP, |s| {
        !s.contains("Colors of the whole interface")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Another page, then back.
    common::settings_page(&mut term, "Synchronization");
    common::settings_page(&mut term, "Appearance");
    std::thread::sleep(Duration::from_millis(300));
    assert!(
        !term.screen().contains("Colors of the whole interface"),
        "back at the top\n{}",
        term.text()
    );

    // Close and reopen Settings.
    term.send_keys("Escape").unwrap();
    term.wait_for("Settings closed", STEP, |s| {
        !s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = term.wait_for_text("\u{2699}", STEP).unwrap();
    term.click(c, r).unwrap();
    term.wait_for("Settings reopened", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    std::thread::sleep(Duration::from_millis(300));
    let text = term.text();
    assert!(
        text.contains("Appearance - global") && !text.contains("Colors of the whole interface"),
        "reopened at the top\n{text}"
    );
}
