//! Settings > Appearance > Theme: a live-previewing combo. Picking Light
//! repaints the Log with Light's background, Esc while previewing goes
//! back, a theme file dropped in the config directory is listed and
//! applies, and the lane palette combo offers "Follow theme" plus every
//! palette (the pty has no pixels, so this is the text renderer).

mod common;

use std::time::Duration;

use gitpoulpi_config::theme::{preset, ThemeColor};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Color, Screen, Terminal};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const MESSAGE: &str = "theme switch probe";
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

fn rgb(slug: &str, role: impl Fn(&gitpoulpi_config::theme::ThemeSpec) -> ThemeColor) -> Color {
    match role(&preset(slug).expect("a preset")) {
        ThemeColor::Rgb(r, g, b) => Color::Rgb(r, g, b),
        other => panic!("{slug}: {other:?} is not RGB"),
    }
}

/// `(bg, fg)` of the first letter of the older commit's message.
fn log_cell(screen: &Screen) -> (Color, Color) {
    let (col, row) = screen.find(ROW_TEXT).expect("the older commit row");
    let cell = screen.cell(col, row).expect("a cell");
    (cell.bg, cell.fg)
}

/// A cell of Settings' own chrome: the title's first letter.
/// `None` on a frame without the title (a torn frame).
fn settings_bg(screen: &Screen) -> Option<Color> {
    let (col, row) = screen.find("Settings - Tab")?;
    Some(screen.cell(col, row).expect("a cell").bg)
}

fn open_settings(term: &mut Terminal, page: &str) {
    let (gc, gr) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(gc, gr).unwrap();
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(term, page);
}

fn close_settings(term: &mut Terminal) {
    term.send_keys("Escape").unwrap();
    term.wait_for("Settings closed", STEP, |s| {
        !s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// Opens the Theme combo (its value ends in the arrow).
fn open_theme_combo(term: &mut Terminal, current: &str) {
    let (col, row) = common::wait_for_choice(term, current, STEP);
    term.click(col + 1, row).unwrap();
    term.wait_for("the popup", STEP, |s| {
        s.contains(&format!("\u{2713} {current}"))
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

fn stored(config: &std::path::Path) -> String {
    std::fs::read_to_string(config.join("config.toml")).unwrap_or_default()
}

fn wait_stored(config: &std::path::Path, needle: &str) {
    let deadline = std::time::Instant::now() + STEP;
    loop {
        let raw = stored(config);
        if raw.contains(needle) {
            return;
        }
        assert!(std::time::Instant::now() < deadline, "not stored: {raw}");
        std::thread::sleep(Duration::from_millis(200));
    }
}

#[test]
fn the_theme_combo_previews_commits_restores_and_lists_user_themes() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(config.path().join("themes")).unwrap();
    std::fs::write(
        config.path().join("themes/x.toml"),
        "name = \"My X\"\nbase = \"nord\"\n",
    )
    .unwrap();
    let mut term = common::launch(repo.root())
        .session("theme-switch")
        .config_dir(config.path())
        .size(200, 60)
        .spawn();
    let screen = term
        .wait_for("the fixture loaded, both rows", STARTUP, |s| {
            s.contains(MESSAGE) && s.find(ROW_TEXT).is_some()
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (dark_bg, dark_fg) = log_cell(&screen);
    assert_eq!(
        dark_bg,
        Color::Default,
        "Dark leaves the terminal background"
    );

    open_settings(&mut term, "Appearance");
    let dark_chrome = settings_bg(
        &term
            .wait_for("Settings' title", STEP, |s| settings_bg(s).is_some())
            .unwrap_or_else(|e| panic!("{e}")),
    )
    .expect("the title is drawn");
    open_theme_combo(&mut term, "System (follows terminal)");
    // Highlight Light: System, Dark, Light. The preview repaints at once.
    term.send_keys("Down").unwrap();
    term.send_keys("Down").unwrap();
    term.wait_for("Light previewed", STEP, |s| {
        settings_bg(s).is_some_and(|b| b != dark_chrome)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Esc closes the combo and restores Dark; nothing was stored.
    term.send_keys("Escape").unwrap();
    term.wait_for("Dark restored", STEP, |s| {
        settings_bg(s) == Some(dark_chrome)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !stored(config.path()).contains("theme = "),
        "{}",
        stored(config.path())
    );

    // Enter commits Light.
    open_theme_combo(&mut term, "System (follows terminal)");
    term.send_keys("Down").unwrap();
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    wait_stored(config.path(), "theme = \"light\"");
    close_settings(&mut term);
    let light_bg = rgb("light", |t| t.ui.bg);
    let screen = term
        .wait_for("the Log in Light", STEP, |s| {
            s.find(ROW_TEXT).is_some() && log_cell(s).0 == light_bg
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert_ne!(log_cell(&screen).1, dark_fg, "the text color changed too");

    // Back to Dark: the default background returns, with no stale cells.
    open_settings(&mut term, "Appearance");
    open_theme_combo(&mut term, "GitPoulpi Light");
    term.send_keys("Up").unwrap();
    term.send_keys("Enter").unwrap();
    wait_stored(config.path(), "theme = \"dark\"");
    close_settings(&mut term);
    term.wait_for("the Log in Dark", STEP, |s| {
        s.find(ROW_TEXT).is_some() && log_cell(s) == (dark_bg, dark_fg)
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // The user theme file is listed and applies.
    open_settings(&mut term, "Appearance");
    open_theme_combo(&mut term, "GitPoulpi Dark");
    // ~120 themes: narrow the list to the user theme (it is listed last).
    term.send_text("My X").unwrap();
    term.wait_for("My X listed", STEP, |s| s.contains("My X"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    wait_stored(config.path(), "theme = \"x\"");
    close_settings(&mut term);
    let nord_bg = rgb("nord", |t| t.ui.bg);
    term.wait_for("the Log in My X", STEP, |s| {
        s.find(ROW_TEXT).is_some() && log_cell(s).0 == nord_bg
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // The lane combo: Follow theme and every palette; picking one recolors.
    let nord_lane = lane_fg(
        &term
            .wait_for("a lane marker", STEP, |s| lane_fg(s).is_some())
            .unwrap_or_else(|e| panic!("{e}")),
    )
    .expect("a lane marker");
    open_settings(&mut term, "Appearance");
    // Input color also reads "Follow theme": take the lane row's combo.
    // The page is longer than the window: bring the lane row into view.
    common::scroll_settings_to(&mut term, "Graph color theme", STEP);
    let screen = term
        .wait_for("the lane row with its combo", STEP, |s| {
            s.find("Graph color theme").is_some_and(|(_, label_row)| {
                s.find_all("Follow theme")
                    .into_iter()
                    .any(|(_, r)| r == label_row)
            })
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (_, label_row) = screen.find("Graph color theme").unwrap();
    let (col, row) = screen
        .find_all("Follow theme")
        .into_iter()
        .find(|(_, r)| *r == label_row)
        .unwrap_or_else(|| panic!("the lane combo:\n{}", screen.text()));
    term.click(col + 1, row).unwrap();
    term.wait_for("the popup", STEP, |s| s.contains("\u{2713} Follow theme"))
        .unwrap_or_else(|e| panic!("{e}"));
    // The list is long, so it is searchable: every palette is reachable.
    for (query, label) in [
        ("gitpoulpi", "GitPoulpi"),
        ("solarized", "Solarized"),
        ("material", "Material"),
        ("nord", "Nord"),
        ("latte", "Catppuccin Latte"),
        ("serie", "Serie"),
    ] {
        term.send_text("\u{15}").unwrap();
        term.send_text(query).unwrap();
        term.wait_for(label, STEP, |s| s.contains(label))
            .unwrap_or_else(|e| panic!("{label} missing from the lane combo: {e}"));
    }
    term.send_keys("Enter").unwrap();
    wait_stored(config.path(), "graph_color_theme = \"serie\"");
    close_settings(&mut term);
    term.wait_for("the lanes recolored", STEP, |s| {
        lane_fg(s).is_some_and(|c| c != nord_lane)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let _ = term.stop();
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
