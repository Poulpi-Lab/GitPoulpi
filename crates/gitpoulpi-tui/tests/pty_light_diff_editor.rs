//! Light themes in the Diff bar and the built-in editor (maintainer report
//! 2026-10-06): the bar's buttons stayed dark (dark text on the dark
//! `ui.button` fill) and, switching theme with the editor open, the text
//! stayed unreadable until the page was left and re-entered. The pty has no
//! pixels, so cell colors are what is compared.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_config::theme::{preset, ThemeColor};
use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Color, Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const LOG_HEADER: &str = "Message";

#[allow(dead_code)]
fn light_bg() -> (u8, u8, u8) {
    match preset("light").expect("light").ui.bg {
        ThemeColor::Rgb(r, g, b) => (r, g, b),
        other => panic!("{other:?}"),
    }
}

fn lum((r, g, b): (u8, u8, u8)) -> f64 {
    let ch = |v: u8| {
        let v = f64::from(v) / 255.0;
        if v <= 0.03928 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * ch(r) + 0.7152 * ch(g) + 0.0722 * ch(b)
}

fn ratio(a: (u8, u8, u8), b: (u8, u8, u8)) -> f64 {
    let (la, lb) = (lum(a), lum(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// Contrast of the cell at `(col, row)`: its foreground against its
/// background (`Default` meaning Light's own, painted, background).
fn cell_ratio(screen: &Screen, col: u16, row: u16) -> f64 {
    let cell = screen.cell(col, row).expect("a cell");
    let Color::Rgb(r, g, b) = cell.fg else {
        return 21.0; // not measurable
    };
    let bg = match cell.bg {
        Color::Rgb(r, g, b) => (r, g, b),
        // Light paints its own background everywhere; a cell left on the
        // terminal's (dark) one under Light's dark text is the bug.
        _ => (0, 0, 0),
    };
    ratio((r, g, b), bg)
}

/// The first cell of `needle` must read (3.0) on its background.
fn assert_readable(screen: &Screen, needle: &str) {
    let (c, r) = screen
        .find(needle)
        .unwrap_or_else(|| panic!("no {needle:?} on\n{}", screen.text()));
    let k = cell_ratio(screen, c, r);
    let cell = screen.cell(c, r).unwrap();
    assert!(
        k >= 3.0,
        "{needle:?} has contrast {k:.2} (fg {:?} on bg {:?})",
        cell.fg,
        cell.bg
    );
}

fn launch_light(repo: &Path, config: &Path, session: &str) -> Terminal {
    std::fs::write(config.join("config.toml"), "[ui]\ntheme = \"light\"\n").unwrap();
    common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(200, 40)
        .no_exit_pause()
        .mouse_capture()
        .spawn()
}

fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .write_worktree("notes.rs", b"fn alpha() {\n    let beta = 1;\n}\n")
        .unwrap()
        .stage(".")
        .unwrap()
        .commit("base")
        .unwrap()
        .write_worktree(
            "notes.rs",
            b"fn alpha() {\n    let beta = 1;\n    let gamma = 2;\n}\n",
        )
        .unwrap()
        .build()
}

fn open_diff(term: &mut Terminal) {
    term.wait_for("the working-changes pane", Duration::from_secs(60), |s| {
        s.contains("Unstaged") && s.contains(LOG_HEADER)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = term
        .wait_for_text("notes.rs", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).unwrap();
    term.wait_for("the unstaged diff", STEP, |s| {
        s.contains("notes.rs - unstaged") && s.contains("gamma") && s.contains(" Edit ")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

fn open_editor(term: &mut Terminal) {
    let (ec, er) = term.screen().find(" Edit ").unwrap();
    term.click(ec + 2, er).unwrap();
    term.wait_for("the editor", STEP, |s| {
        s.contains("Edit - notes.rs @ working tree") && s.contains("alpha")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn the_diff_bar_and_the_editor_read_on_a_light_theme() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch_light(repo.root(), config.path(), "pty-light-diff");
    open_diff(&mut term);
    let screen = term.screen();
    for label in [" Edit ", " Stage ", " Discard "] {
        if screen.contains(label) {
            assert_readable(&screen, label.trim());
        }
    }
    assert_readable(&screen, "Edit");
    open_editor(&mut term);
    let screen = term.screen();
    assert_readable(&screen, "alpha");
    assert_readable(&screen, "Save");
    let _ = term.stop();
}

#[test]
fn switching_to_a_light_theme_with_the_editor_open_recolors_it_at_once() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(repo.root())
        .session("pty-light-editor-switch")
        .config_dir(config.path())
        .size(200, 40)
        .no_exit_pause()
        .mouse_capture()
        .spawn();
    open_diff(&mut term);
    open_editor(&mut term);
    term.send_keys("C-p").unwrap();
    term.send_text("change theme").unwrap();
    term.wait_for("the palette entry", STEP, |s| s.contains("Change Theme"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the theme list", STEP, |s| {
        s.contains("System (follows terminal)")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("GitPoulpi Light").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("Light applied over the open editor", STEP, |s| {
        !s.contains("System (follows")
            && s.find("alpha")
                .is_some_and(|(c, r)| matches!(s.cell(c, r).map(|x| x.bg), Some(Color::Rgb(..))))
            && s.contains("Edit - notes.rs")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let screen = term.screen();
    assert_readable(&screen, "alpha");
    assert_readable(&screen, "Save");
    let _ = term.stop();
}

/// The same switch through Settings > Appearance (the page the maintainer uses),
/// then back to the editor: its text reads without leaving the page.
#[test]
fn switching_theme_in_settings_then_returning_to_the_editor_reads_at_once() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(repo.root())
        .session("pty-light-editor-settings")
        .config_dir(config.path())
        .size(200, 40)
        .no_exit_pause()
        .mouse_capture()
        .spawn();
    open_diff(&mut term);
    open_editor(&mut term);
    let (gc, gr) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(gc, gr).unwrap();
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(&mut term, "Appearance");
    let (col, row) = common::wait_for_choice(&term, "System (follows terminal)", STEP);
    term.click(col + 1, row).unwrap();
    term.wait_for("the popup", STEP, |s| {
        s.contains("\u{2713} System (follows terminal)")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Down").unwrap();
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    let deadline = std::time::Instant::now() + STEP;
    while !std::fs::read_to_string(config.path().join("config.toml"))
        .unwrap_or_default()
        .contains("theme = \"light\"")
    {
        assert!(std::time::Instant::now() < deadline, "theme not stored");
        std::thread::sleep(Duration::from_millis(200));
    }
    term.send_keys("Escape").unwrap();
    term.wait_for("back on the editor in Light", STEP, |s| {
        !s.contains("Repo-specific settings")
            && s.contains("Edit - notes.rs")
            && s.find("alpha")
                .is_some_and(|(c, r)| matches!(s.cell(c, r).map(|x| x.bg), Some(Color::Rgb(..))))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let screen = term.screen();
    assert_readable(&screen, "alpha");
    assert_readable(&screen, "Save");
    let _ = term.stop();
}
