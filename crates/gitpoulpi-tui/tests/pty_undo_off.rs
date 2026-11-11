//! `general.undo_enabled` in the TUI, end to end through a pty: the
//! Settings checkbox asks before erasing the history, and with undo off the
//! toolbar has no Undo/Redo buttons (they come back when it is turned on).

mod common;

use std::time::{Duration, Instant};

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

const UNDO_BUTTON: &str = "\u{21B6} Undo";
const REDO_BUTTON: &str = "\u{21B7} Redo";

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .build()
}

fn config_has(path: &std::path::Path, needle: &str) -> bool {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .contains(needle)
}

fn wait_config(path: &std::path::Path, needle: &str) {
    let deadline = Instant::now() + STEP;
    while !config_has(path, needle) {
        assert!(Instant::now() < deadline, "{needle} never saved");
        std::thread::sleep(Duration::from_millis(200));
    }
}

fn open_undo_row(term: &mut Terminal) -> (u16, u16) {
    let (gc, gr) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(gc, gr).unwrap();
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // The Undo group sits below the fold of the General page.
    common::scroll_settings_to(term, "Undo enabled", STEP)
}

#[test]
fn undo_off_removes_the_buttons_after_a_confirmation() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let path = config.path().join("config.toml");
    let mut term = common::launch(repo.root())
        .session("pty-undo-off")
        .config_dir(config.path())
        .size(200, 60)
        .spawn();
    term.wait_for("the toolbar", STARTUP, |s| {
        s.contains(UNDO_BUTTON) && s.contains(REDO_BUTTON) && s.contains("initial commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Unticking asks first; Cancel keeps undo on.
    open_undo_row(&mut term);
    common::click_text_until(&mut term, "Undo enabled", "the question", |s| {
        s.contains("Turning Undo off erases")
    });
    term.send_keys("Enter").unwrap(); // Cancel is the default button
    term.wait_for("the question gone", STEP, |s| {
        !s.contains("Turning Undo off erases")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(!config_has(&path, "undo_enabled"), "Cancel keeps it on");

    // Turn off: Tab to the second button, Enter.
    common::click_text_until(&mut term, "Undo enabled", "the question", |s| {
        s.contains("Turning Undo off erases")
    });
    term.send_keys("Tab").unwrap();
    term.send_keys("Enter").unwrap();
    wait_config(&path, "undo_enabled = false");
    term.send_keys("Escape").unwrap();
    term.wait_for("the toolbar without Undo and Redo", STEP, |s| {
        !s.contains("Repo-specific settings")
            && !s.contains(UNDO_BUTTON)
            && !s.contains(REDO_BUTTON)
            && s.contains("Pull")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Ctrl+Z only says why, and the refresh that closing Settings sets off
    // must not wipe the message.
    term.send_keys("C-z").unwrap();
    term.wait_for("the status message", STEP, |s| s.contains("turned off"))
        .unwrap_or_else(|e| panic!("{e}"));

    // Back on: no question, the buttons return.
    open_undo_row(&mut term);
    // Retried (located afresh): a click placed on a torn frame lands nowhere.
    common::retry_until(
        &mut term,
        "undo_enabled = true saved",
        4,
        STEP,
        |t| {
            if !config_has(&path, "undo_enabled = true") {
                if let Some((col, row)) = t.screen().find("Undo enabled") {
                    t.click(col + 1, row).unwrap();
                }
            }
        },
        |_| config_has(&path, "undo_enabled = true"),
    );
    term.send_keys("Escape").unwrap();
    term.wait_for("the buttons back", STEP, |s| {
        !s.contains("Repo-specific settings") && s.contains(UNDO_BUTTON) && s.contains(REDO_BUTTON)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let _ = term.stop();
}

/// Started with undo off (the config file), the bar has no Undo/Redo and
/// Ctrl+Z / the palette only say why.
#[test]
fn started_with_undo_off_the_buttons_are_absent_and_ctrl_z_says_why() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        "[general]\nundo_enabled = false\n",
    )
    .unwrap();
    let mut term = common::launch(repo.root())
        .session("pty-undo-off-config")
        .config_dir(config.path())
        .size(200, 50)
        .spawn();
    term.wait_for("the toolbar", STARTUP, |s| {
        s.contains("Pull") && s.contains("initial commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // The bar's last frame, read once the window stopped redrawing.
    let screen = term
        .wait_for_stable(Duration::from_millis(400), STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !screen.contains(UNDO_BUTTON) && !screen.contains(REDO_BUTTON),
        "{}",
        screen.text()
    );
    term.send_keys("C-z").unwrap();
    term.wait_for("the reason", STEP, |s| s.contains("turned off"))
        .unwrap_or_else(|e| panic!("{e}"));
    let _ = term.stop();
}

/// No frame from the very first byte may carry the Undo/Redo buttons when the
/// config has undo off: the toolbar must not be drawn with the default (on)
/// and corrected once the daemon reports the journal. Reads the raw byte
/// stream (every frame ever written), not a settled screen.
#[test]
fn started_with_undo_off_no_frame_ever_draws_the_buttons() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        "[general]\nundo_enabled = false\n",
    )
    .unwrap();
    let term = common::launch(repo.root())
        .session("pty-undo-off-first-frame")
        .config_dir(config.path())
        .size(200, 50)
        .spawn();
    term.wait_for("the toolbar", STARTUP, |s| {
        s.contains("Pull") && s.contains("initial commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Let the daemon's journal state arrive and a few more frames pass.
    std::thread::sleep(Duration::from_millis(1500));
    let raw = String::from_utf8_lossy(&term.raw_output()).into_owned();
    for needle in [UNDO_BUTTON, REDO_BUTTON, "\u{21B6}", "\u{21B7}"] {
        assert!(
            !raw.contains(needle),
            "{needle:?} was drawn at some point with undo off"
        );
    }
    let _ = term.stop();
}

#[test]
fn started_with_undo_off_ctrl_z_and_palette_say_why() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        "[general]\nundo_enabled = false\n",
    )
    .unwrap();
    let mut term = common::launch(repo.root())
        .session("pty-undo-off-config-keys")
        .config_dir(config.path())
        .size(200, 50)
        .spawn();
    term.wait_for("the toolbar", STARTUP, |s| {
        s.contains("Pull") && s.contains("initial commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-z").unwrap();
    term.wait_for("the reason", STEP, |s| s.contains("turned off"))
        .unwrap_or_else(|e| panic!("{e}"));
    // The palette does not offer them either.
    term.send_keys("C-p").unwrap();
    term.send_text("undo").unwrap();
    std::thread::sleep(Duration::from_millis(500));
    assert!(!term.screen().contains("Edit: Undo"), "{}", term.text());
    let _ = term.stop();
}
