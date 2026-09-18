//! `ui.side_bg`: with the Catppuccin Mocha preset the Refs pane and the right
//! panel have their own background, the Log keeps the main one. The pty has
//! no pixels, so cell colors are what is compared.

mod common;

use std::time::Duration;

use gitpoulpi_config::theme::{preset, ThemeColor};
use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Color;

const MESSAGE: &str = "side bg probe";

fn rgb(c: ThemeColor) -> Color {
    match c {
        ThemeColor::Rgb(r, g, b) => Color::Rgb(r, g, b),
        other => panic!("{other:?}"),
    }
}

#[test]
fn the_side_panes_have_their_own_background() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit(MESSAGE)
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        "[ui]\ntheme = \"catppuccin-mocha\"\n",
    )
    .unwrap();
    let spec = preset("catppuccin-mocha").unwrap();
    let (side, main) = (rgb(spec.ui.side_bg), rgb(spec.ui.bg));
    assert_ne!(side, main);
    let term = common::launch(repo.root())
        .session("pty-side-bg")
        .config_dir(config.path())
        .size(200, 40)
        .no_exit_pause()
        .spawn();
    term.wait_for("the fixture loaded", Duration::from_secs(60), |s| {
        s.contains(MESSAGE) && s.contains("LOCAL")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let screen = term.screen();
    let bg = |col: u16, row: u16| screen.cell(col, row).expect("a cell").bg;
    let (lc, lr) = screen.find(MESSAGE).unwrap();
    assert_eq!(
        bg(lc + 20, lr + 6),
        main,
        "a Log cell keeps the main background"
    );
    let (rc, rr) = screen.find("LOCAL").unwrap();
    // A blank cell well inside the Refs pane, below its first row.
    assert_eq!(bg(rc + 1, rr + 1), side, "inside the Refs pane");
    // The right panel: its right-most interior column on a body row.
    assert_eq!(bg(190, 30), side, "inside the right panel");
    let _ = term.stop();
}
