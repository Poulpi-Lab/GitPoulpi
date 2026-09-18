//! `ui.graph_color_theme`: picking a theme in the Settings combo recolors
//! the text renderer's lanes once Preferences closes (the pty has no
//! pixels, so this is the text renderer; the picture is built from the
//! same grid colors and its cache key hashes them).

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Color, Screen};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const MESSAGE: &str = "theme probe commit";
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

/// The foreground of the lane marker left of `ROW_TEXT`'s message.
fn lane_fg(screen: &Screen) -> Option<Color> {
    let (msg_col, row) = screen.find(ROW_TEXT)?;
    (msg_col.saturating_sub(8)..msg_col)
        .filter_map(|c| screen.cell(c, row))
        .find(|cell| {
            "\u{2502}\u{25cf}\u{25cb}\u{25c9}".contains(cell.text.as_str()) && !cell.text.is_empty()
        })
        .map(|cell| cell.fg)
}

#[test]
fn picking_a_graph_color_theme_recolors_the_lanes() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(repo.root())
        .session("graph-color-theme")
        .config_dir(config.path())
        .size(200, 60)
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| s.contains(MESSAGE))
        .unwrap_or_else(|e| panic!("{e}"));
    let before = lane_fg(&term.screen()).expect("a lane marker");
    assert!(matches!(before, Color::Rgb(..)), "{before:?}");

    let (gc, gr) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(gc, gr).unwrap();
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(&mut term, "Appearance");
    // Input color also reads "Follow theme": take the combo on the
    // Graph color theme row.
    common::scroll_settings_to(&mut term, "Graph color theme", STEP);
    let screen = term.screen();
    let (_, label_row) = screen.find("Graph color theme").unwrap();
    let (col, row) = screen
        .find_all("Follow theme")
        .into_iter()
        .find(|(_, r)| *r == label_row)
        .unwrap_or_else(|| panic!("the theme combo:\n{}", screen.text()));
    term.click(col + 1, row).unwrap();
    let screen = term
        .wait_for("the popup", STEP, |s| s.contains("\u{2713} Follow theme"))
        .unwrap_or_else(|e| panic!("{e}"));
    let _ = screen;
    // ~120 palettes: narrow the list to Dracula (first of the matches), pick it.
    term.send_text("Dracula").unwrap();
    term.send_keys("Enter").unwrap();

    let deadline = std::time::Instant::now() + STEP;
    loop {
        let stored = std::fs::read_to_string(config.path().join("config.toml")).unwrap_or_default();
        if stored.contains("graph_color_theme = \"dracula\"") {
            break;
        }
        assert!(std::time::Instant::now() < deadline, "not stored: {stored}");
        std::thread::sleep(Duration::from_millis(200));
    }
    term.send_keys("Escape").unwrap();
    let screen = term
        .wait_for("the lanes recolored", STEP, |s| {
            lane_fg(s).is_some_and(|c| c != before)
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(lane_fg(&screen), Some(Color::Rgb(0xbd, 0x93, 0xf9)));
    let _ = term.stop();
}
