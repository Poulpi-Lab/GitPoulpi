//! AI off hides every AI control, and switching it on in Settings shows
//! them at once. The real binary through a pty (rules of `pty_ui.rs`).
//! No AI request is ever made: only what is drawn is checked.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const SPARK: &str = "\u{2726}";

fn launch(
    session: &str,
    ai_on: bool,
) -> (Terminal, tempfile::TempDir, gitpoulpi_testkit::FixtureRepo) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .stage("a.txt")
        .unwrap()
        .build();
    let config = tempfile::tempdir().expect("temp config dir");
    if ai_on {
        std::fs::write(config.path().join("config.toml"), "[ai]\nenabled = true\n").unwrap();
    }
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(200, 50)
        .spawn();
    term.wait_for("the fixture loaded, WIP row and all", STARTUP, |s| {
        s.contains("initial commit") && s.contains("// WIP") && s.contains("Amend previous commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    (term, config, repo)
}

/// Every spark glyph on the screen.
fn sparks(s: &xtask::pty::Screen) -> usize {
    s.find_all(SPARK).len()
}

#[test]
fn with_ai_off_the_main_window_and_commit_panel_show_no_spark_and_the_palette_no_ai() {
    let (mut term, _config, _repo) = launch("pty-ai-off-main", false);
    let screen = term.screen();
    assert_eq!(sparks(&screen), 0, "{}", screen.text());
    assert!(!screen.contains("Compose commits with AI"));

    term.send_keys("C-p").unwrap();
    term.send_text("AI").unwrap();
    let screen = term
        .wait_for("the list narrowed by the query", STEP, |s| {
            s.text()
                .lines()
                .find(|l| l.contains("Command Palette ("))
                .is_some_and(|l| !l.contains("(324/") && !l.contains("(325/"))
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!screen.contains("AI: "), "{}", screen.text());
    assert_eq!(sparks(&screen), 0, "{}", screen.text());
    term.stop().unwrap();
}

/// A build without the `ai` feature: even a config that turns AI on shows
/// no spark and no AI command (the `[ai]` table still parses).
#[cfg(not(feature = "ai"))]
#[test]
fn a_build_without_ai_ignores_ai_enabled_in_the_config() {
    let (mut term, _config, _repo) = launch("pty-ai-off-unbuilt", true);
    let screen = term.screen();
    assert_eq!(sparks(&screen), 0, "{}", screen.text());
    assert!(!screen.contains("Compose commits with AI"));
    term.send_keys("C-p").unwrap();
    term.send_text("AI").unwrap();
    let screen = term
        .wait_for("the palette open", STEP, |s| {
            s.contains("Command Palette (")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!screen.contains("AI: "), "{}", screen.text());
    term.stop().unwrap();
}

#[cfg(feature = "ai")]
#[test]
fn turning_ai_on_in_settings_shows_the_controls_at_once() {
    let (mut term, _config, _repo) = launch("pty-ai-off-toggle", false);
    assert_eq!(sparks(&term.screen()), 0);

    let (c, r) = term.wait_for_text("\u{2699}", STEP).unwrap();
    term.click(c, r).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(&mut term, "AI");
    term.wait_for("the AI page", STEP, |s| s.contains("Enable AI"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = term.wait_for_text("Enable AI", STEP).unwrap();
    term.click(c + 1, r).expect("click the switch");
    let path = _config.path().join("config.toml");
    let deadline = std::time::Instant::now() + STEP;
    while !std::fs::read_to_string(&path)
        .unwrap_or_default()
        .contains("enabled = true")
    {
        assert!(
            std::time::Instant::now() < deadline,
            "ai.enabled never saved"
        );
        std::thread::sleep(Duration::from_millis(200));
    }
    term.send_keys("Escape").expect("close Settings");
    term.wait_for("the Generate sparkle back in the commit panel", STEP, |s| {
        s.find_all(SPARK)
            .into_iter()
            .any(|(_, r)| s.line(r).contains("Summary"))
            && s.contains("Compose commits with AI")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}
