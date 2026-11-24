//! Two installations share one `config.toml`; the theme is each one's own.
//! The running window (installation `p-pty`, pinned by the harness) must not
//! recolour when the shared file changes because ANOTHER installation picked
//! a theme, and must when its own entry changes. Scratch config only.

mod common;

use std::time::{Duration, Instant};

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Color, Screen, Terminal};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const MESSAGE: &str = "installation theme probe";
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

/// Background of the older commit's row; `Default` under Dark.
fn log_bg(screen: &Screen) -> Option<Color> {
    let (col, row) = screen.find(ROW_TEXT)?;
    Some(screen.cell(col, row)?.bg)
}

/// A config where `p-pty` (this window) wears Dark in its own entry while the
/// shared keys and another installation say `shared`.
fn config_text(own: &str, shared: &str, other: &str) -> String {
    format!(
        "[ui]\ntheme = \"{shared}\"\n\n\
         [installation.settings.p-pty]\nkind = \"pinned\"\n\n\
         [installation.settings.p-pty.look]\ntheme = \"{own}\"\n\n\
         [installation.settings.p-other]\nkind = \"pinned\"\n\n\
         [installation.settings.p-other.look]\ntheme = \"{other}\"\n"
    )
}

fn wait_bg(term: &mut Terminal, what: &str, ok: impl Fn(Color) -> bool) {
    term.wait_for(what, STEP, |s| log_bg(s).is_some_and(&ok))
        .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn another_installations_theme_never_recolours_this_window() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let file = config.path().join("config.toml");
    std::fs::write(&file, config_text("dark", "dark", "dark")).unwrap();
    let mut term = common::launch(repo.root())
        .session("installation-theme")
        .config_dir(config.path())
        .size(200, 60)
        .spawn();
    term.wait_for("the fixture loaded under Dark", STARTUP, |s| {
        s.contains(MESSAGE) && log_bg(s) == Some(Color::Default)
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Another installation (and an older build writing `ui.theme`) pick Light.
    std::fs::write(&file, config_text("dark", "light", "light")).unwrap();
    // Give the watcher (750 ms tick) several looks at the changed file.
    let until = Instant::now() + Duration::from_secs(6);
    while Instant::now() < until {
        let screen = term
            .wait_for_stable(Duration::from_millis(300), STEP)
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(
            log_bg(&screen),
            Some(Color::Default),
            "this window recoloured for another installation:\n{}",
            screen.text()
        );
    }

    // Positive control: its own entry changing does recolour it.
    std::fs::write(&file, config_text("light", "light", "light")).unwrap();
    wait_bg(&mut term, "own theme Light applied", |c| {
        c != Color::Default
    });
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

#[test]
fn the_graph_colours_are_this_installations_own_too() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    // The shared key (an older build, another installation) says nord; this
    // installation's entry says dracula, so dracula's lanes are drawn.
    std::fs::write(
        config.path().join("config.toml"),
        "[ui]\ngraph_color_theme = \"nord\"\n\n\
         [installation.settings.p-pty]\nkind = \"pinned\"\n\n\
         [installation.settings.p-pty.look]\ngraph_color_theme = \"dracula\"\n",
    )
    .unwrap();
    let term = common::launch(repo.root())
        .session("installation-graph-colours")
        .config_dir(config.path())
        .size(200, 60)
        .spawn();
    let screen = term
        .wait_for("the fixture loaded with its lane marker", STARTUP, |s| {
            s.contains(MESSAGE) && lane_fg(s).is_some()
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(lane_fg(&screen), Some(Color::Rgb(0xbd, 0x93, 0xf9)));
    let _ = term.stop();
}
