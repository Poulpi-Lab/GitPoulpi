//! The palette's `App: Change Graph Color Theme...` (a picker over the lane
//! palettes, Enter stores `ui.graph_color_theme`) and `App: Toggle Graph Row
//! Tints` (flips `ui.graph_lane_background` between `off` and `row_tint`).

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const MESSAGE: &str = "graph palette commands probe";

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit(MESSAGE)
        .unwrap()
        .build()
}

fn stored(config: &std::path::Path) -> String {
    std::fs::read_to_string(config.join("config.toml")).unwrap_or_default()
}

fn wait_stored(config: &std::path::Path, needle: &str) {
    let deadline = std::time::Instant::now() + STEP;
    while !stored(config).contains(needle) {
        assert!(
            std::time::Instant::now() < deadline,
            "{needle:?} not stored: {}",
            stored(config)
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}

fn run_palette(term: &mut Terminal, query: &str, entry: &str) {
    term.send_keys("C-p").unwrap();
    term.send_text(query).unwrap();
    term.wait_for("the palette entry", STEP, |s| s.contains(entry))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
}

#[test]
fn the_palette_changes_the_graph_color_theme() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(repo.root())
        .session("graph-palette-pick")
        .config_dir(config.path())
        .size(200, 60)
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| s.contains(MESSAGE))
        .unwrap_or_else(|e| panic!("{e}"));

    // Esc stores nothing.
    run_palette(&mut term, "graph color theme", "Change Graph Color Theme");
    term.wait_for("the list", STEP, |s| {
        s.contains("Change Graph Color Theme (") && s.contains("\u{2713} Follow theme")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("dracula").unwrap();
    term.send_keys("Escape").unwrap();
    term.wait_for_text_to_go("Change Graph Color Theme (", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !stored(config.path()).contains("graph_color_theme"),
        "{}",
        stored(config.path())
    );

    // Enter stores the chosen palette.
    run_palette(&mut term, "graph color theme", "Change Graph Color Theme");
    term.wait_for("the list", STEP, |s| {
        s.contains("Change Graph Color Theme (")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("dracula").unwrap();
    term.send_keys("Enter").unwrap();
    wait_stored(config.path(), "graph_color_theme = \"dracula\"");
    let _ = term.stop();
}

#[test]
fn the_palette_toggles_the_graph_row_tints() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(repo.root())
        .session("graph-palette-tints")
        .config_dir(config.path())
        .size(200, 60)
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| s.contains(MESSAGE))
        .unwrap_or_else(|e| panic!("{e}"));

    run_palette(&mut term, "row tints", "Toggle Graph Row Tints");
    wait_stored(config.path(), "graph_lane_background = \"row_tint\"");
    run_palette(&mut term, "row tints", "Toggle Graph Row Tints");
    wait_stored(config.path(), "graph_lane_background = \"off\"");
    let _ = term.stop();
}
