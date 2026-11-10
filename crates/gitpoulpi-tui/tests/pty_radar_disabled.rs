//! The Radar can be switched off (`general.radar_enabled`) and its
//! tab can be closed for a session (the tab's `x`, `Ctrl+W`, middle-click),
//! then reopened from the palette or the `+` menu.
//!
//! Real binary and daemon through a pty; temp config dirs and fixture
//! repositories only, no network.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "radar-off-commit")
        .unwrap()
        .build()
}

fn start(repo: &FixtureRepo, config: &Path, session: &str) -> Terminal {
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(config)
        .size(180, 45)
        .no_exit_pause()
        .mouse_capture()
        .spawn();
    term.wait_for("the first window", STARTUP, |s| {
        s.contains("radar-off-commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term
}

fn bar(screen: &Screen) -> String {
    screen.line(1)
}

fn col_in(line: &str, needle: &str) -> u16 {
    let at = line
        .find(needle)
        .unwrap_or_else(|| panic!("no {needle:?} in {line:?}"));
    line[..at].chars().count() as u16
}

fn radar_shows(s: &Screen) -> bool {
    s.contains("Scope") && s.contains("Open tabs") && s.contains("Refresh")
}

fn palette_lists_radar(term: &mut Terminal) -> bool {
    term.send_keys("C-p").unwrap();
    term.send_text("Radar").unwrap();
    // Something drawn after the query settled: the palette itself.
    term.wait_for("the palette", STEP, |s| {
        s.contains("Type to search") || s.contains("Radar") || s.contains("No command")
    })
    .ok();
    std::thread::sleep(Duration::from_millis(600));
    let shown = term.screen().contains("Radar: ");
    term.send_keys("Escape").unwrap();
    shown
}

/// Off in `config.toml`: no tab, no palette entries, `Alt+R` does nothing, the
/// tab picker lacks it.
#[test]
fn a_disabled_radar_has_no_tab_no_command_and_no_key() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        "[general]\nradar_enabled = false\n",
    )
    .unwrap();
    let mut term = start(&repo, config.path(), "pty-radar-off");
    let line = bar(&term.screen());
    assert!(!line.contains("Radar"), "no tab: {line}");
    assert!(!palette_lists_radar(&mut term), "no palette entry");
    term.send_keys("M-r").unwrap();
    std::thread::sleep(Duration::from_millis(800));
    let screen = term.screen();
    assert!(!radar_shows(&screen), "Alt+R is inert:\n{}", screen.text());
    assert!(!bar(&screen).contains("Radar"));
    // The tab picker lists repositories only.
    term.send_keys("C-n").unwrap();
    term.send_text("t").unwrap();
    term.wait_for("the tab picker", STEP, |s| s.contains("Select Tab"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!term.screen().text().contains("Radar"));
    term.send_keys("Escape").unwrap();
    term.stop().unwrap();
}

/// On (the default): the tab closes by its `x`, stays closed across a restart
/// of the session, and comes back from the palette right after Home.
#[test]
fn closing_the_radar_tab_persists_and_the_palette_reopens_it() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = start(&repo, config.path(), "pty-radar-close");
    // The cross shows on the hovered (or open) tab only, like the others.
    let at_rest = term
        .wait_for("the Radar tab", STEP, |s| bar(s).contains("Radar"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let line = bar(&at_rest);
    assert!(!line.contains("Radar x"), "no cross at rest: {line}");
    term.move_to(col_in(&line, "Radar") + 2, 1).unwrap();
    let hovered = term
        .wait_for("the cross on hover", STEP, |s| bar(s).contains("Radar x"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let line = bar(&hovered);
    let x = col_in(&line, "Radar x") + "Radar ".len() as u16;
    term.click(x, 1).unwrap();
    term.wait_for("the tab gone", STEP, |s| !bar(s).contains("Radar"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();

    // Restart: still closed. The setting itself was not touched.
    let mut term = start(&repo, config.path(), "pty-radar-close");
    // The bar drawn (its `+`), and without the Radar tab.
    let bar_up = term
        .wait_for("the tab bar", STEP, |s| bar(s).contains(" + "))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!bar(&bar_up).contains("Radar"), "stays closed");
    let cfg = std::fs::read_to_string(config.path().join("config.toml")).unwrap_or_default();
    assert!(!cfg.contains("radar_enabled"), "{cfg}");

    // The `+` menu offers it.
    let plus = col_in(&bar(&bar_up), " + ");
    term.click(plus + 1, 1).unwrap();
    term.wait_for("the + menu with a Radar row", STEP, |s| {
        s.contains("New tab") && s.text().contains("Radar")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for("the menu closed", STEP, |s| !s.contains("New tab"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The palette reopens it, right after the house and before the tabs.
    term.send_keys("C-p").unwrap();
    term.send_text("Radar: Open").unwrap();
    term.wait_for("the palette entry", STEP, |s| s.contains("Radar: Open"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    let screen = term
        .wait_for("the Radar", STEP, radar_shows)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let line = bar(&screen);
    assert!(
        col_in(&line, "\u{2302}") < col_in(&line, "Radar"),
        "after Home: {line}"
    );
    // Ctrl+W closes it again (no question), back on Home.
    term.send_keys("C-w").unwrap();
    term.wait_for("closed by Ctrl+W", STEP, |s| {
        !bar(s).contains("Radar") && !radar_shows(s)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
