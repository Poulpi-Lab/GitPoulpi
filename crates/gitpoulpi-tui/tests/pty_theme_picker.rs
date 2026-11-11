//! The palette's `App: Change Theme...`: a second-level list of System, the
//! presets and the user's theme files. Moving the highlight previews the
//! theme, Enter applies it and writes `ui.theme`, Esc puts the previous one
//! back (the pty has no pixels, so the text renderer's cell backgrounds are
//! what is compared).

mod common;

use std::time::Duration;

use gitpoulpi_config::theme::{preset, ThemeColor};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Color, Screen, Terminal};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const MESSAGE: &str = "theme picker probe";
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

fn log_bg(screen: &Screen) -> Color {
    let (col, row) = screen.find(ROW_TEXT).expect("the older commit row");
    screen.cell(col, row).expect("a cell").bg
}

fn light_bg() -> Color {
    match preset("light").expect("light").ui.bg {
        ThemeColor::Rgb(r, g, b) => Color::Rgb(r, g, b),
        other => panic!("{other:?}"),
    }
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
        s.contains("System (follows terminal)") && s.contains("Light")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn the_palette_changes_the_theme_previews_and_escapes() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(config.path().join("themes")).unwrap();
    std::fs::write(
        config.path().join("themes/x.toml"),
        "name = \"My X\"\nbase = \"nord\"\n",
    )
    .unwrap();
    let mut term = common::launch(repo.root())
        .session("theme-picker")
        .config_dir(config.path())
        .size(200, 70)
        .spawn();
    let screen = term
        .wait_for("the fixture loaded, both rows", STARTUP, |s| {
            s.contains(MESSAGE) && s.find(ROW_TEXT).is_some()
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let dark_bg = log_bg(&screen);

    // The list: System first, a tick on the current theme, user themes in.
    open_picker(&mut term);
    let screen = term
        .wait_for("the current theme ticked", STEP, |s| s.contains("\u{2713}"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(screen.contains("\u{2713}"), "{}", screen.text());
    // ~120 themes: the user theme is far down, so narrow the list to it.
    term.send_text("My X").unwrap();
    term.wait_for("My X listed", STEP, |s| s.contains("My X"))
        .unwrap_or_else(|e| panic!("{e}"));
    for _ in 0.."My X".len() {
        term.send_keys("Backspace").unwrap();
    }

    // Narrowing to GitPoulpi Light previews it at once (plain "Light" also
    // matches Gruvbox Light first); Esc restores Dark, stores nothing.
    term.send_text("GitPoulpi Light").unwrap();
    term.wait_for("Light previewed", STEP, |s| {
        s.find(ROW_TEXT).is_some() && log_bg(s) == light_bg()
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").unwrap();
    term.wait_for("Dark restored", STEP, |s| {
        s.find(ROW_TEXT).is_some() && log_bg(s) == dark_bg && !s.contains("System (follows")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !stored(config.path()).contains("theme = "),
        "{}",
        stored(config.path())
    );

    // Enter applies Light and writes ui.theme.
    open_picker(&mut term);
    term.send_text("GitPoulpi Light").unwrap();
    term.send_keys("Enter").unwrap();
    let deadline = std::time::Instant::now() + STEP;
    while !stored(config.path()).contains("theme = \"light\"") {
        assert!(
            std::time::Instant::now() < deadline,
            "not stored: {}",
            stored(config.path())
        );
        std::thread::sleep(Duration::from_millis(200));
    }
    term.wait_for("Light stays", STEP, |s| {
        s.find(ROW_TEXT).is_some() && log_bg(s) == light_bg() && !s.contains("System (follows")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let _ = term.stop();
}

#[test]
fn ctrl_n_y_opens_the_picker_and_escape_closes_it() {
    let repo = fixture();
    let (mut term, _config) = common::launch(repo.root())
        .session("theme-picker-key")
        .size(200, 60)
        .ready_when("the fixture loaded", |s| s.contains(MESSAGE))
        .start();
    term.send_keys("C-n").unwrap();
    term.send_text("y").unwrap();
    term.wait_for("the theme list", STEP, |s| {
        s.contains("System (follows terminal)")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").unwrap();
    term.wait_for_text_to_go("System (follows terminal)", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    let _ = term.stop();
}
