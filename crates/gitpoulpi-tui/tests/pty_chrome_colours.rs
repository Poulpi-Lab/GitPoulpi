//! `ui.toolbar_bg` / `ui.statusbar_bg` / `ui.statusbar_fg`: with the VS Code
//! Dark preset the toolbar row and the selected tab share the toolbar color
//! and the footer is the blue status bar with white text. The pty has no
//! pixels, so cell colors are what is compared.

mod common;

use std::time::Duration;

use gitpoulpi_config::theme::{preset, ThemeColor};
use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Color;

fn rgb(c: ThemeColor) -> Color {
    match c {
        ThemeColor::Rgb(r, g, b) => Color::Rgb(r, g, b),
        other => panic!("{other:?}"),
    }
}

#[test]
fn toolbar_tab_and_status_bar_have_their_own_colors() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("chrome colors probe")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        "[ui]\ntheme = \"vscode-dark\"\n",
    )
    .unwrap();
    let spec = preset("vscode-dark").unwrap();
    let (toolbar, status_bg, status_fg, main) = (
        rgb(spec.ui.toolbar_bg),
        rgb(spec.ui.statusbar_bg),
        rgb(spec.ui.statusbar_fg),
        rgb(spec.ui.bg),
    );
    assert_eq!(status_bg, Color::Rgb(0, 0x7a, 0xcc));
    let term = common::launch(repo.root())
        .session("pty-chrome-colors")
        .config_dir(config.path())
        .size(200, 40)
        .no_exit_pause()
        .spawn();
    term.wait_for("the fixture loaded", Duration::from_secs(60), |s| {
        s.contains("chrome colors probe") && s.contains("LOCAL")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let screen = term.screen();
    let cell = |col: u16, row: u16| screen.cell(col, row).expect("a cell");
    // Row 0 is the window border, 1 the tab bar, 2 the toolbar; the footer is
    // the last row inside the border.
    let (rows, cols) = (40u16, 200u16);
    // The selected tab: bold name on the toolbar color.
    let tab_name = repo
        .root()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let (tc, tr) = screen
        .find_all(&tab_name)
        .into_iter()
        .find(|&(_, r)| r > 0 && r < 3)
        .unwrap_or_else(|| panic!("the tab name in the tab bar:\n{}", screen.text()));
    assert_eq!(
        cell(tc, tr).bg,
        toolbar,
        "the selected tab shares the toolbar color"
    );
    // A blank cell far right on the toolbar row (the row of "Undo": the
    // toolbar is at the bottom by default).
    let (_, toolbar_row) = screen.find("Undo").expect("the toolbar");
    assert_eq!(cell(cols - 30, toolbar_row).bg, toolbar, "the toolbar row");
    assert_ne!(toolbar, main);
    // The Log keeps the main background.
    let (lc, lr) = screen.find("chrome colors probe").unwrap();
    assert_eq!(cell(lc + 20, lr + 6).bg, main, "the Log is untouched");
    // The status bar.
    let footer = rows - 2;
    assert_eq!(cell(cols - 30, footer).bg, status_bg, "the status bar fill");
    let inked: Vec<_> = (1..cols - 1)
        .map(|c| cell(c, footer))
        .filter(|c| !c.text.trim().is_empty())
        .collect();
    assert!(!inked.is_empty(), "the footer has text:\n{}", screen.text());
    for c in inked {
        assert_eq!(c.fg, status_fg, "status bar text {:?} is white", c.text);
    }
    let _ = term.stop();
}
