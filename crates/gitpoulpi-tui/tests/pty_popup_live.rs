//! Palette pickers paint over the live app and preview what they pick: the
//! graph color theme and the language change the app behind the popup as the
//! highlight moves (Esc restores), the theme picker has an All / Dark / Light
//! / Colored prefilter (click and keyboard), and a picker never takes the
//! Graph column's picture away.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Color, Screen, Terminal};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const MESSAGE: &str = "popup live probe";
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

fn stored(config: &std::path::Path) -> String {
    std::fs::read_to_string(config.join("config.toml")).unwrap_or_default()
}

fn lane_fg(screen: &Screen) -> Option<Color> {
    let (msg_col, row) = screen.find(ROW_TEXT)?;
    (msg_col.saturating_sub(8)..msg_col)
        .filter_map(|c| screen.cell(c, row))
        .find(|cell| {
            "\u{2502}\u{25cf}\u{25cb}\u{25c9}".contains(cell.text.as_str()) && !cell.text.is_empty()
        })
        .map(|cell| cell.fg)
}

fn open_from_palette(term: &mut Terminal, query: &str, entry: &str, title: &str) {
    term.send_keys("C-p").unwrap();
    term.send_text(query).unwrap();
    term.wait_for("the palette entry", STEP, |s| s.contains(entry))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the picker", STEP, |s| s.contains(title))
        .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn the_graph_color_picker_previews_live_and_escape_restores() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(repo.root())
        .session("popup-live-graph")
        .config_dir(config.path())
        .size(200, 60)
        .spawn();
    term.wait_for("loaded", STARTUP, |s| s.contains(MESSAGE))
        .unwrap_or_else(|e| panic!("{e}"));
    let before = lane_fg(&term.screen()).expect("a lane marker");

    open_from_palette(
        &mut term,
        "graph color theme",
        "Graph Color",
        "Change Graph Color",
    );
    term.send_text("Dracula").unwrap();
    // Still open, and the lanes behind it are already Dracula's.
    term.wait_for("Dracula previewed", STEP, |s| {
        s.contains("Change Graph Color") && lane_fg(s) == Some(Color::Rgb(0xbd, 0x93, 0xf9))
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").unwrap();
    term.wait_for("lanes restored", STEP, |s| {
        !s.contains("Change Graph Color") && lane_fg(s) == Some(before)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(!stored(config.path()).contains("graph_color_theme"));

    // Enter keeps it.
    open_from_palette(
        &mut term,
        "graph color theme",
        "Graph Color",
        "Change Graph Color",
    );
    term.send_text("Dracula").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("lanes kept", STEP, |s| {
        !s.contains("Change Graph Color") && lane_fg(s) == Some(Color::Rgb(0xbd, 0x93, 0xf9))
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(stored(config.path()).contains("graph_color_theme = \"dracula\""));
    let _ = term.stop();
}

#[test]
fn the_language_picker_previews_live_and_escape_restores() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        "[ui]\nlanguage = \"en\"\n",
    )
    .unwrap();
    let mut term = common::launch(repo.root())
        .session("popup-live-lang")
        .env("GITPOULPI_LANG", "")
        .config_dir(config.path())
        .size(200, 60)
        .spawn();
    term.wait_for("loaded", STARTUP, |s| s.contains(MESSAGE))
        .unwrap_or_else(|e| panic!("{e}"));
    open_from_palette(
        &mut term,
        "change language",
        "Change Language",
        "Change Language (",
    );
    term.send_text("fran").unwrap();
    // The app behind the popup (the toolbar) reads French while it is open.
    term.wait_for("French preview", STEP, |s| s.contains("Annuler"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").unwrap();
    term.wait_for_text_to_go("Annuler", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!stored(config.path()).contains("\"fr\""));
    let _ = term.stop();
}

fn shown(screen: &Screen) -> Option<usize> {
    let text = screen.text();
    let at = text.find("Change Theme (")? + "Change Theme (".len();
    text[at..].split('/').next()?.parse().ok()
}

#[test]
fn the_theme_picker_prefilter_works_by_keyboard_and_click() {
    let repo = fixture();
    let (mut term, _config) = common::launch(repo.root())
        .session("popup-live-prefilter")
        .size(200, 70)
        .ready_when("loaded", |s| s.contains(MESSAGE))
        .start();
    open_from_palette(
        &mut term,
        "change theme",
        "Change Theme",
        "System (follows terminal)",
    );
    let screen = term.screen();
    for label in ["All", "Dark", "Light", "Colored"] {
        assert!(screen.contains(label), "{}", screen.text());
    }
    let all = shown(&screen).expect("count");

    // Tab: Dark. System (All only) is gone.
    term.send_keys("Tab").unwrap();
    let dark = term
        .wait_for("dark only", STEP, |s| {
            !s.contains("System (follows terminal)")
        })
        .map(|s| shown(&s).unwrap())
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(dark < all);
    // Alt+3: Light; the light themes are a different set.
    term.send_keys("M-3").unwrap();
    let light = term
        .wait_for("light only", STEP, |s| shown(s).is_some_and(|n| n != dark))
        .map(|s| shown(&s).unwrap())
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(light < all);
    // Click Colored; typing combines with it.
    let (col, row) = term.screen().find("Colored").expect("the segment");
    term.click(col + 1, row).unwrap();
    let colored = term
        .wait_for("colored only", STEP, |s| {
            shown(s).is_some_and(|n| n != light)
        })
        .map(|s| shown(&s).unwrap())
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(colored < all);
    term.send_text("solarized dark").unwrap();
    term.wait_for("Solarized Dark listed", STEP, |s| {
        s.contains("Solarized Dark")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Back to All with Alt+1 brings System back for an empty draft.
    for _ in 0.."solarized dark".len() {
        term.send_keys("Backspace").unwrap();
    }
    term.send_keys("M-1").unwrap();
    term.wait_for("all again", STEP, |s| {
        s.contains("System (follows terminal)")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let _ = term.stop();
}

#[test]
fn a_palette_picker_does_not_take_the_graph_picture_away() {
    let repo = fixture();
    let (mut term, _config) = common::launch(repo.root())
        .session("popup-live-kitty")
        .config("[ui]\ngraph_renderer = \"image\"\nimage_protocol = \"kitty\"\n")
        .terminal_image()
        .env("GITPOULPI_TERMINAL_CELL_PX", "9x21")
        .size(70, 40)
        .ready_when("loaded", |s| s.contains(MESSAGE))
        .start();
    term.wait_for_stable(Duration::from_millis(800), STEP)
        .unwrap();
    let before = term.raw_output().len();
    open_from_palette(
        &mut term,
        "change language",
        "Change Language",
        "Change Language (",
    );
    term.wait_for_stable(Duration::from_millis(800), STEP)
        .unwrap();
    let raw = term.raw_output();
    let written = String::from_utf8_lossy(&raw[before..]);
    assert!(
        !written.contains("a=d"),
        "the picker deleted the Graph picture: {written:?}"
    );
    let _ = term.stop();
}
