//! Settings > Agents & MCP: the MCP server switch writes `mcp.enabled`, the
//! port row writes the port of `mcp.bind` (host kept), and the port row is
//! editable while the server is off. Real binary, daemon and system git through
//! a pty; the config dir is a temp dir.
#![cfg(feature = "mcp")]

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

fn stored(dir: &Path) -> String {
    std::fs::read_to_string(dir.join("config.toml")).unwrap_or_default()
}

fn wait_stored(dir: &Path, needle: &str) {
    let deadline = std::time::Instant::now() + STEP;
    while !stored(dir).contains(needle) {
        assert!(
            std::time::Instant::now() < deadline,
            "{needle:?} never reached config.toml: {}",
            stored(dir)
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// Clicks the first cell of `text` on the page (scrolled into view first).
fn click_text(term: &mut Terminal, text: &str) {
    let (col, row) = common::scroll_settings_to(term, text, STEP);
    term.click(col + 1, row).expect("click the row");
}

#[test]
fn the_switch_and_the_port_are_saved_and_the_port_is_editable_while_off() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("mcp base")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        "[mcp]\nbind = \"127.0.0.2:7000\"\n",
    )
    .unwrap();
    let mut term = common::launch(repo.root())
        .session("pty-settings-mcp")
        .config_dir(config.path())
        .size(200, 100)
        .ready("mcp base")
        .spawn();
    let (gear_col, gear_row) = term.wait_for_text("\u{2699}", STEP).unwrap();
    term.click(gear_col, gear_row).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(&mut term, "Agents & MCP");

    // On by default, the port of the configured bind is shown.
    term.wait_for("the port of the bind", STEP, |s| {
        s.text()
            .lines()
            .any(|l| l.contains("MCP server port") && l.contains("7000"))
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Off: written, and the port row stays editable (no greying, no note).
    let (col, row) = common::scroll_settings_to(&mut term, "MCP server", STEP);
    term.click(col + 1, row).expect("click the switch");
    wait_stored(config.path(), "enabled = false");
    term.wait_for("the port row still showing its port", STEP, |s| {
        s.text()
            .lines()
            .any(|l| l.contains("MCP server port") && l.contains("7000"))
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(!term.screen().contains("turn it on to set the port"));

    // The port by mouse and keys while the server is off: the host stays.
    let screen = term.screen();
    let (_, port_row) = screen.find("MCP server port").expect("the port row");
    let (number_col, _) = screen
        .find_all("7000")
        .into_iter()
        .find(|&(_, r)| r == port_row)
        .expect("the number");
    term.click(number_col, port_row).expect("click the port");
    for _ in 0..6 {
        term.send_keys("Backspace").unwrap();
    }
    term.send_text("9100").unwrap();
    term.send_keys("Enter").unwrap();
    wait_stored(config.path(), "bind = \"127.0.0.2:9100\"");
    assert!(stored(config.path()).contains("enabled = false"));

    // On again: the port is kept.
    let (col, row) = common::scroll_settings_to(&mut term, "MCP server", STEP);
    term.click(col + 1, row).expect("click the switch");
    wait_stored(config.path(), "enabled = true");
    assert!(stored(config.path()).contains("bind = \"127.0.0.2:9100\""));

    // Out of range: refused, nothing more written.
    click_text(&mut term, "MCP server port");
    for _ in 0..6 {
        term.send_keys("Backspace").unwrap();
    }
    term.send_text("70000").unwrap();
    term.send_keys("Enter").unwrap();
    std::thread::sleep(Duration::from_millis(500));
    assert!(
        stored(config.path()).contains("bind = \"127.0.0.2:9100\""),
        "{}",
        stored(config.path())
    );
}
