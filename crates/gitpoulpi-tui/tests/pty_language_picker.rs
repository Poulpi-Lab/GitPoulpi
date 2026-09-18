//! The palette's `App: Change Language...`: a second-level list of the built
//! languages; Enter applies the language live and writes `ui.language`, Esc
//! stores nothing.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const MESSAGE: &str = "language picker probe";

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
    term.send_text("change language").unwrap();
    term.wait_for("the palette entry", STEP, |s| s.contains("Change Language"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the language list", STEP, |s| {
        s.contains("Change Language (") && s.contains("Fran\u{e7}ais")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn the_palette_changes_the_language_and_escapes() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    // Start in English whatever the machine's locale is.
    std::fs::write(
        config.path().join("config.toml"),
        "[ui]\nlanguage = \"en\"\n",
    )
    .unwrap();
    let mut term = common::launch(repo.root())
        .session("language-picker")
        // The harness pins GITPOULPI_LANG=en; empty is ignored, so the config rules.
        .env("GITPOULPI_LANG", "")
        .config_dir(config.path())
        .size(200, 60)
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| s.contains(MESSAGE))
        .unwrap_or_else(|e| panic!("{e}"));

    // Esc: nothing stored.
    open_picker(&mut term);
    term.send_text("fran").unwrap();
    term.send_keys("Escape").unwrap();
    term.wait_for_text_to_go("Change Language (", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !stored(config.path()).contains("fr"),
        "{}",
        stored(config.path())
    );

    // Enter: ui.language = "fr" is stored.
    open_picker(&mut term);
    term.send_text("fran").unwrap();
    term.send_keys("Enter").unwrap();
    let deadline = std::time::Instant::now() + STEP;
    while !stored(config.path()).contains("language = \"fr\"") {
        assert!(
            std::time::Instant::now() < deadline,
            "not stored: {}",
            stored(config.path())
        );
        std::thread::sleep(Duration::from_millis(200));
    }
    // The UI is French at once: the toolbar reads in French.
    term.wait_for("French palette text", STEP, |s| s.contains("Annuler"))
        .unwrap_or_else(|e| panic!("{e}"));
    let _ = term.stop();
}

#[test]
fn ctrl_n_g_opens_the_picker_and_escape_closes_it() {
    let repo = fixture();
    let (mut term, _config) = common::launch(repo.root())
        .session("language-picker-key")
        .size(200, 60)
        .ready_when("the fixture loaded", |s| s.contains(MESSAGE))
        .start();
    term.send_keys("C-n").unwrap();
    term.send_text("g").unwrap();
    term.wait_for("the language list", STEP, |s| {
        s.contains("Change Language (")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").unwrap();
    term.wait_for_text_to_go("Change Language (", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    let _ = term.stop();
}
