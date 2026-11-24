//! `ui.graph_color_theme`: picking a custom (user) theme in the Settings
//! combo recolors the text renderer's lanes with that theme's own
//! `graph_palette` once Preferences closes.

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
fn picking_a_custom_theme_as_graph_colors_recolors_the_lanes() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(config.path().join("themes")).unwrap();
    // Named like a built-in lane-only palette (`gruvbox`): the user's file wins.
    std::fs::write(
        config.path().join("themes/gruvbox.toml"),
        "name = \"Zz Lanes\"\ngraph_palette = [\"#12ab34\", \"#12ab35\", \"#12ab36\"]\n",
    )
    .unwrap();
    let mut term = common::launch(repo.root())
        .session("graph-color-theme")
        .config_dir(config.path())
        .size(200, 60)
        .spawn();
    let loaded = term
        .wait_for("the fixture loaded with its lane marker", STARTUP, |s| {
            s.contains(MESSAGE) && lane_fg(s).is_some()
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let before = lane_fg(&loaded).expect("a lane marker");
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
    // Retried: a click aimed at a torn or still-scrolling frame opens nothing.
    common::retry_until(
        &mut term,
        "the popup",
        4,
        Duration::from_secs(10),
        |t| {
            common::scroll_settings_to(t, "Graph color theme", STEP);
            let screen = t
                .wait_for_stable(Duration::from_millis(300), STEP)
                .unwrap_or_else(|e| panic!("{e}"));
            let (_, label_row) = screen.find("Graph color theme").unwrap();
            let (col, row) = screen
                .find_all("Follow theme")
                .into_iter()
                .find(|(_, r)| *r == label_row)
                .unwrap_or_else(|| panic!("the theme combo:\n{}", screen.text()));
            t.click(col + 1, row).unwrap();
        },
        |s| s.contains("\u{2713} Follow theme"),
    );
    term.send_text("Zz Lanes").unwrap();
    term.send_keys("Enter").unwrap();

    let deadline = std::time::Instant::now() + STEP;
    loop {
        let stored = std::fs::read_to_string(config.path().join("config.toml")).unwrap_or_default();
        if stored.contains("graph_color_theme = \"gruvbox\"") {
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
    assert_eq!(lane_fg(&screen), Some(Color::Rgb(0x12, 0xab, 0x34)));
    let _ = term.stop();
}
