//! Changing the logo color in the About window rewrites the icon of a
//! launcher shortcut GitPoulpi made, and leaves one it did not make alone.
//!
//! XDG_DATA_HOME is a scratch directory (the harness points it into the
//! config dir), so the developer's real launcher entry is out of reach.

// XDG only: the icon lives in the `.desktop` entry's hicolor theme; the macOS
// bundle icon is covered by the config unit tests.
#![cfg(all(unix, not(target_os = "macos")))]

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;

const STEP: Duration = Duration::from_secs(30);

/// The icon file the launcher entry names: its name carries the color, so
/// launchers that cache icons by name or path pick up a new one.
fn named_icon(data: &std::path::Path) -> Option<std::path::PathBuf> {
    let text = std::fs::read_to_string(data.join("applications/gitpoulpi.desktop")).ok()?;
    let name = text.lines().find_map(|l| l.strip_prefix("Icon="))?;
    Some(data.join(format!("icons/hicolor/256x256/apps/{name}.png")))
}

fn wait_for_icon(data: &std::path::Path, what: &str, ok: impl Fn(&[u8]) -> bool) -> Vec<u8> {
    for _ in 0..200 {
        if let Some(Ok(bytes)) = named_icon(data).map(std::fs::read) {
            if ok(&bytes) {
                return bytes;
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    panic!("{what}: the icon never became right");
}

/// The terminal, its config dir, the placeholder icon and the XDG data dir.
fn start(
    ours: bool,
) -> (
    xtask::pty::Terminal,
    tempfile::TempDir,
    std::path::PathBuf,
    std::path::PathBuf,
) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "first-commit")
        .unwrap()
        .build();
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("config.toml"),
        "[ui]\ngraph_renderer = \"text\"\nlogo_color = \"sun\"\n",
    )
    .unwrap();
    let data = dir.path().join("xdg-data");
    let apps = data.join("applications");
    std::fs::create_dir_all(&apps).unwrap();
    let marker = if ours { "X-GitPoulpi=1\n" } else { "" };
    std::fs::write(
        apps.join("gitpoulpi.desktop"),
        format!("[Desktop Entry]\nName=GitPoulpi\n{marker}"),
    )
    .unwrap();
    let icon = data.join("icons/hicolor/256x256/apps/gitpoulpi.png");
    std::fs::create_dir_all(icon.parent().unwrap()).unwrap();
    std::fs::write(&icon, b"placeholder").unwrap();
    let term = common::launch(repo.root())
        .session("launcher-icon")
        .config_dir(dir.path())
        .size(200, 50)
        .no_exit_pause()
        .ready("first-commit")
        .spawn();
    (term, dir, icon, data)
}

fn pick_mint(term: &mut xtask::pty::Terminal) {
    let (col, row) = term.screen().find("\u{1F419}").expect("the octopus");
    term.click(col, row).unwrap();
    term.wait_for("the About window", STEP, |s| s.contains("License: MIT"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("c").unwrap();
    term.wait_for("the color list", STEP, |s| s.contains("Mint"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("Mint").unwrap();
    term.send_keys("Enter").unwrap();
}

#[test]
fn a_new_logo_color_rewrites_the_icon_of_our_shortcut() {
    let (mut term, _dir, placeholder, data) = start(true);
    // Start-up already brings a stale icon to the configured color.
    let sun = wait_for_icon(&data, "start-up", |b| b.starts_with(b"\x89PNG"));
    let sun_path = named_icon(&data).unwrap();
    assert!(!placeholder.exists(), "the icon of the old name is gone");
    pick_mint(&mut term);
    let mint = wait_for_icon(&data, "after the pick", |b| b != sun.as_slice());
    assert!(mint.starts_with(b"\x89PNG"));
    assert_ne!(
        named_icon(&data).unwrap(),
        sun_path,
        "a new color is a new name"
    );
    assert!(!sun_path.exists(), "the previous color's icon is gone");
    term.wait_for("the footer note", STEP, |s| {
        s.contains("launcher shortcuts updated")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

#[test]
fn a_desktop_file_that_is_not_ours_keeps_its_icon() {
    let (mut term, _dir, icon, _data) = start(false);
    pick_mint(&mut term);
    std::thread::sleep(Duration::from_secs(3));
    assert_eq!(std::fs::read(&icon).unwrap(), b"placeholder");
    term.stop().unwrap();
}
