//! The eight presets added after the first batch (One Light, Tokyo Night Day,
//! Rosé Pine Moon / Dawn, Catppuccin Macchiato / Frappé, Kanagawa Lotus,
//! Vesper) show in `App: Change Theme...` and apply (`ui.theme` is stored).

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const MESSAGE: &str = "more presets probe";

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

fn open_picker(term: &mut Terminal) {
    term.send_keys("C-p").unwrap();
    term.send_text("change theme").unwrap();
    term.wait_for("the palette entry", STEP, |s| s.contains("Change Theme"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the theme list", STEP, |s| {
        s.contains("System (follows terminal)")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn the_new_presets_are_listed_and_one_applies() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(repo.root())
        .session("theme-more-presets")
        .config_dir(config.path())
        .size(200, 60)
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| s.contains(MESSAGE))
        .unwrap_or_else(|e| panic!("{e}"));

    for (query, name) in [
        ("one light", "One Light"),
        ("tokyo night day", "Tokyo Night Day"),
        ("pine moon", "Moon"),
        ("pine dawn", "Dawn"),
        ("macchiato", "Catppuccin Macchiato"),
        ("frapp", "Catppuccin Frapp"),
        ("lotus", "Kanagawa Lotus"),
        ("vesper", "Vesper"),
    ] {
        open_picker(&mut term);
        term.send_text(query).unwrap();
        term.wait_for(name, STEP, |s| s.contains(name))
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        term.send_keys("Escape").unwrap();
        term.wait_for_text_to_go("Change Theme (", STEP)
            .unwrap_or_else(|e| panic!("{e}"));
    }
    assert!(
        !stored(config.path()).contains("theme"),
        "{}",
        stored(config.path())
    );

    open_picker(&mut term);
    term.send_text("vesper").unwrap();
    term.send_keys("Enter").unwrap();
    let deadline = std::time::Instant::now() + STEP;
    while !stored(config.path()).contains("theme = \"vesper\"") {
        assert!(
            std::time::Instant::now() < deadline,
            "not stored: {}",
            stored(config.path())
        );
        std::thread::sleep(Duration::from_millis(200));
    }
    let _ = term.stop();
}
