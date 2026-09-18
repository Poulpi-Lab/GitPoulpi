//! `ui.graph_row_tint_strength`: picking Subtle in the Settings combo (shown
//! once the lane background is Row tint) fades the text renderer's row-tint
//! band once Preferences closes, and a stored `strong` is stronger than the
//! default `medium`. The pty has no pixels; the picture takes the same
//! strength as its alpha (unit-tested on the bitmap).

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Color, Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const MESSAGE: &str = "tint strength probe commit";
const ROW_TEXT: &str = "an older commit";

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("an older commit under the probe")
        .unwrap()
        .commit(MESSAGE)
        .unwrap()
        .build()
}

fn launch(repo: &FixtureRepo, session: &str, toml: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .config(toml)
        .size(200, 60)
        .ready_when("the fixture loaded", |s| s.contains(MESSAGE))
        .start()
}

/// The background of the commit's node cell on `ROW_TEXT`'s row.
fn band_bg(screen: &Screen) -> Option<(u8, u8, u8)> {
    let (msg_col, row) = screen.find(ROW_TEXT)?;
    (msg_col.saturating_sub(8)..msg_col)
        .filter_map(|c| screen.cell(c, row))
        .find(|cell| {
            "\u{2502}\u{25cf}\u{25cb}\u{25c9}".contains(cell.text.as_str()) && !cell.text.is_empty()
        })
        .and_then(|cell| match cell.bg {
            Color::Rgb(r, g, b) => Some((r, g, b)),
            _ => None,
        })
}

/// Distance from the pane color (29, 32, 34): how present the band is.
fn presence((r, g, b): (u8, u8, u8)) -> i32 {
    (i32::from(r) - 29).abs() + (i32::from(g) - 32).abs() + (i32::from(b) - 34).abs()
}

const TINT: &str = "[ui]\ngraph_lane_background = \"row_tint\"\n";

#[test]
fn a_stored_strong_is_more_present_than_the_default_medium() {
    let repo = fixture();
    let (term, _c) = launch(&repo, "tint-strength-a", TINT);
    let medium = band_bg(&term.screen()).expect("a tinted node");
    let _ = term.stop();
    let (term, _c) = launch(
        &repo,
        "tint-strength-b",
        &format!("{TINT}graph_row_tint_strength = \"strong\"\n"),
    );
    let strong = band_bg(&term.screen()).expect("a tinted node");
    let _ = term.stop();
    assert!(
        presence(strong) > presence(medium),
        "{strong:?} vs {medium:?}"
    );
}

#[test]
fn picking_subtle_fades_the_band() {
    let repo = fixture();
    let (mut term, config) = launch(&repo, "tint-strength-c", TINT);
    let before = band_bg(&term.screen()).expect("a tinted node");

    let (gc, gr) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(gc, gr).unwrap();
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(&mut term, "Appearance");
    let (col, row) = common::scroll_settings_to_choice(&mut term, "Medium", STEP);
    term.click(col + 1, row).unwrap();
    let screen = term
        .wait_for("the popup", STEP, |s| s.contains("\u{2713} Medium"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (tc, tr) = screen.find("\u{2713} Medium").unwrap();
    // Strong, the ticked Medium, then Subtle one row below.
    term.click(tc + 2, tr + 1).unwrap();

    let deadline = std::time::Instant::now() + STEP;
    loop {
        let stored = std::fs::read_to_string(config.path().join("config.toml")).unwrap_or_default();
        if stored.contains("graph_row_tint_strength = \"subtle\"") {
            break;
        }
        assert!(std::time::Instant::now() < deadline, "not stored: {stored}");
        std::thread::sleep(Duration::from_millis(200));
    }
    term.send_keys("Escape").unwrap();
    let screen = term
        .wait_for("the band faded", STEP, |s| {
            band_bg(s).is_some_and(|c| c != before)
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let after = band_bg(&screen).unwrap();
    assert!(
        presence(after) < presence(before),
        "{after:?} vs {before:?}"
    );
    let _ = term.stop();
}
