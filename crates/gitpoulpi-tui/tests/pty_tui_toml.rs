//! The terminal UI's settings live in `tui.toml`, copied from `config.toml`:
//! an old-style `config.toml` (a theme and a key binding) is honoured and
//! stays byte-identical while `tui.toml` appears, and a `gitpoulpi config set
//! ui.theme` from a shell repaints the running window through `tui.toml`.
//! Scratch config directories only.

mod common;

use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Color, Screen};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const MESSAGE: &str = "tui toml probe";
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

fn cli() -> PathBuf {
    let tui = PathBuf::from(env!("CARGO_BIN_EXE_gitpoulpi-tui"));
    let path = tui.with_file_name(format!("gitpoulpi{}", std::env::consts::EXE_SUFFIX));
    if !path.exists() {
        let status = Command::new(env!("CARGO"))
            .args(["build", "-p", "gitpoulpi-cli"])
            .status()
            .expect("run cargo build");
        assert!(status.success(), "building the gitpoulpi CLI failed");
    }
    path
}

/// An old-style file: a theme and one custom key binding, no `tui.toml`.
const OLD_STYLE: &str = "[ui]\ntheme = \"light\"\n\n[keybindings.status]\nstage_toggle = \"q\"\n";

#[test]
fn an_old_style_config_is_honoured_copied_to_tui_toml_and_left_untouched() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let file = config.path().join("config.toml");
    std::fs::write(&file, OLD_STYLE).unwrap();
    let mut term = common::launch(repo.root())
        .session("tui-toml-copy")
        .config_dir(config.path())
        .size(200, 60)
        .spawn();
    // The theme is honoured: Light draws a background, Dark leaves it alone.
    term.wait_for("the fixture loaded under Light", STARTUP, |s| {
        s.contains(MESSAGE) && log_bg(s).is_some_and(|c| c != Color::Default)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // The copy appears, complete.
    let tui = config.path().join("tui.toml");
    let deadline = Instant::now() + STEP;
    while !tui.exists() {
        assert!(Instant::now() < deadline, "tui.toml never appeared");
        std::thread::sleep(Duration::from_millis(100));
    }
    let copy: toml::Value = toml::from_str(&std::fs::read_to_string(&tui).unwrap()).unwrap();
    assert_eq!(copy["ui"]["theme"].as_str(), Some("light"));
    assert_eq!(
        copy["keybindings"]["status"]["stage_toggle"].as_str(),
        Some("q")
    );
    // The key binding is honoured: the Keyboard & mouse page lists `q`.
    term.send_keys("C-p").unwrap();
    term.send_text("App: Keybindings").unwrap();
    term.wait_for("the palette entry", STEP, |s| {
        s.contains("App: Keybindings...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the page", STEP, |s| s.contains("Keyboard & mouse"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("/").unwrap();
    term.send_text("status.stage_toggle").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the custom key", STEP, |s| s.contains("[q]"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let _ = term.stop();
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        OLD_STYLE,
        "config.toml is never modified by the terminal UI"
    );
}

#[test]
fn config_set_ui_theme_from_a_shell_repaints_a_running_window() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let file = config.path().join("config.toml");
    // The window's installation is the primary, so the colour is `ui.theme`.
    let before = "[ui]\ntheme = \"dark\"\n\n[installation]\nprimary = \"p-pty\"\n";
    std::fs::write(&file, before).unwrap();
    let term = common::launch(repo.root())
        .session("tui-toml-set")
        .config_dir(config.path())
        .size(200, 60)
        .spawn();
    term.wait_for("the fixture loaded under Dark", STARTUP, |s| {
        s.contains(MESSAGE) && log_bg(s) == Some(Color::Default)
    })
    .unwrap_or_else(|e| panic!("{e}"));

    let out = Command::new(cli())
        .env("GITPOULPI_CONFIG_DIR", config.path())
        .env("GITPOULPI_INSTALL_ID", "pty")
        .arg("--json")
        .args(["config", "set", "ui.theme", "light"])
        .current_dir(repo.root())
        .output()
        .expect("run gitpoulpi");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(
        PathBuf::from(json["file"].as_str().unwrap()),
        config.path().join("tui.toml")
    );

    term.wait_for("Light applied by the watcher", STEP, |s| {
        log_bg(s).is_some_and(|c| c != Color::Default)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let _ = term.stop();
    assert_eq!(std::fs::read_to_string(&file).unwrap(), before);
}
