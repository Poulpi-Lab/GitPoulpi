//! Changed-word emphasis in the Diff view, end to end through a pty: a
//! changed line is painted with the theme's stronger added / removed
//! background on the words that differ only, in Inline and in Split, and
//! `ui.diff_intraline = false` leaves one flat tint per line.
//!
//! Colors are read from the cells the terminal emulator ended up with, never
//! guessed from the escape stream.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_config::theme::{preset, ThemeColor};
use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Color, Screen, Terminal};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

const OLD: &str = "Tire chaque branche locale qui a un amont : recupere tout";
const NEW: &str = "Met a jour toutes les branches locales qui ont un amont : recupere tout";

fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("notes.txt", &format!("{OLD}\nsecond line\n"), "first")
        .unwrap()
        .commit_file("notes.txt", &format!("{NEW}\nsecond line\n"), "reword")
        .unwrap()
        .build()
}

fn rgb(c: ThemeColor) -> Color {
    let ThemeColor::Rgb(r, g, b) = c else {
        panic!("the dark preset paints RGB");
    };
    Color::Rgb(r, g, b)
}

fn launch(repo: &Path, config_toml: &str, session: &str) -> (Terminal, tempfile::TempDir) {
    let config = tempfile::tempdir().unwrap();
    std::fs::write(config.path().join("config.toml"), config_toml).unwrap();
    let term = common::launch(repo)
        .session(session)
        .config_dir(config.path())
        .size(300, 50)
        .no_exit_pause()
        .spawn();
    term.wait_for("the commit detail of HEAD", STARTUP, |s| {
        s.contains("View all files") && s.contains("SHA")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.wait_for_mouse_capture(STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    (term, config)
}

fn open_the_diff(term: &mut Terminal) {
    let (c, r) = term
        .wait_for_text("notes.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    term.wait_for("the diff in the center", STEP, |s| {
        s.contains("Met a jour toutes") && s.contains("Tire chaque") && !s.contains("SHA")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// The background of the cell where `word` starts inside `line`, which
/// itself starts at `(col, row)` on the screen.
fn bg_of(screen: &Screen, (col, row): (u16, u16), line: &str, word: &str) -> Color {
    let at = line
        .find(word)
        .unwrap_or_else(|| panic!("{word:?} is not in {line:?}"));
    screen
        .cell(col + at as u16, row)
        .unwrap_or_else(|| panic!("no cell for {word:?}"))
        .bg
}

/// The two lines' words that stay put keep the flat tint; the words that
/// changed carry the stronger one.
fn assert_emphasis(screen: &Screen, old_at: (u16, u16), new_at: (u16, u16)) {
    let dark = preset("dark").expect("dark").diff;
    let (add, add_e) = (rgb(dark.add_bg), rgb(dark.add_emph_bg));
    let (del, del_e) = (rgb(dark.del_bg), rgb(dark.del_emph_bg));
    for word in ["Tire", "chaque", "branche", "locale", "a un"] {
        assert_eq!(bg_of(screen, old_at, OLD, word), del_e, "removed {word:?}");
    }
    for word in ["qui", "un", "amont", "recupere"] {
        assert_eq!(bg_of(screen, old_at, OLD, word), del, "removed {word:?}");
    }
    for word in ["Met", "jour", "branches", "locales", "ont"] {
        assert_eq!(bg_of(screen, new_at, NEW, word), add_e, "added {word:?}");
    }
    for word in ["qui", "un", "amont", "recupere"] {
        assert_eq!(bg_of(screen, new_at, NEW, word), add, "added {word:?}");
    }
}

#[test]
fn inline_paints_the_changed_words_stronger() {
    let repo = fixture();
    let (mut term, _config) = launch(
        repo.root(),
        "[ui]\ntheme = \"dark\"\ncolor_depth = \"truecolor\"\n",
        "pty-intraline-inline",
    );
    open_the_diff(&mut term);
    let screen = term.screen();
    let old_at = screen.find("Tire chaque").expect("the removed line");
    let new_at = screen.find("Met a jour").expect("the added line");
    assert_ne!(old_at.1, new_at.1, "Inline: one line under the other");
    assert_emphasis(&screen, old_at, new_at);
}

#[test]
fn split_paints_the_changed_words_stronger_on_both_halves() {
    let repo = fixture();
    let (mut term, _config) = launch(
        repo.root(),
        "[ui]\ntheme = \"dark\"\ncolor_depth = \"truecolor\"\ndiff_layout = \"split\"\n",
        "pty-intraline-split",
    );
    open_the_diff(&mut term);
    let screen = term.screen();
    let old_at = screen.find("Tire chaque").expect("the removed side");
    let new_at = screen.find("Met a jour").expect("the added side");
    assert_eq!(old_at.1, new_at.1, "Split: both sides on one row");
    assert!(old_at.0 < new_at.0);
    assert_emphasis(&screen, old_at, new_at);
}

#[test]
fn the_setting_off_leaves_one_flat_tint_per_line() {
    let repo = fixture();
    let (mut term, _config) = launch(
        repo.root(),
        "[ui]\ntheme = \"dark\"\ncolor_depth = \"truecolor\"\ndiff_intraline = false\n",
        "pty-intraline-off",
    );
    open_the_diff(&mut term);
    let screen = term.screen();
    let dark = preset("dark").expect("dark").diff;
    let old_at = screen.find("Tire chaque").expect("the removed line");
    let new_at = screen.find("Met a jour").expect("the added line");
    assert_eq!(bg_of(&screen, old_at, OLD, "Tire"), rgb(dark.del_bg));
    assert_eq!(bg_of(&screen, new_at, NEW, "Met"), rgb(dark.add_bg));
}
