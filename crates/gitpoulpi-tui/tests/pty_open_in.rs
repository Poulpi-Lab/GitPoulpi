//! The tab bar's "Open in" combobox, through a real pty. A click always opens
//! the list (entries in order, the default marked, "Configure..." last); the
//! palette's "Tab: Open in..." opens it focused, "Tab: Open in Default" runs
//! the default entry; the popup is modal, so a click away only closes it.
//!
//! The launcher is a fake script recording its arguments: no real terminal or
//! editor is ever started. Temp config dir and fixture repos only.

#![cfg(unix)]

mod common;

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const STARTUP: Duration = Duration::from_secs(60);

struct Setup {
    alpha: gitpoulpi_testkit::FixtureRepo,
    bravo: gitpoulpi_testkit::FixtureRepo,
    config: tempfile::TempDir,
    work: tempfile::TempDir,
}

impl Setup {
    fn new() -> Self {
        let alpha = FixtureRepoBuilder::new()
            .unwrap()
            .commit_file("a.txt", "a\n", "alpha-only-commit")
            .unwrap()
            .build();
        let bravo = FixtureRepoBuilder::new()
            .unwrap()
            .commit_file("b.txt", "b\n", "bravo-only-commit")
            .unwrap()
            .build();
        let config = tempfile::tempdir().unwrap();
        std::fs::write(
            config.path().join("recent.toml"),
            format!(
                "[[repos]]\npath = '{}'\nname = 'bravo-repo'\nlast_opened = 1\n",
                bravo.root().display()
            ),
        )
        .unwrap();
        let work = tempfile::tempdir().unwrap();
        let script = work.path().join("fake-launcher.sh");
        std::fs::write(
            &script,
            format!(
                "#!/bin/sh\necho \"$@\" >> '{}'\n",
                work.path().join("launches.log").display()
            ),
        )
        .unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        let fake = script.display();
        std::fs::write(
            config.path().join("config.toml"),
            format!(
                "[ui]\ngraph_renderer = \"text\"\nopen_in_default = \"Second entry\"\n\
                 [[ui.open_in]]\nlabel = \"First entry\"\ncommand = \"{fake} --first ${{cwd}}\"\n\
                 [[ui.open_in]]\nlabel = \"Second entry\"\ncommand = \"{fake} --second ${{cwd}}\"\n\
                 [[ui.open_in]]\nlabel = \"Third entry\"\ncommand = \"{fake} --third ${{cwd}}\"\n"
            ),
        )
        .unwrap();
        Self {
            alpha,
            bravo,
            config,
            work,
        }
    }

    fn launches(&self) -> String {
        std::fs::read_to_string(self.work.path().join("launches.log")).unwrap_or_default()
    }

    /// Waits for the launcher's log to hold a line.
    fn launched(&self) -> String {
        for _ in 0..200 {
            if !self.launches().is_empty() {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        self.launches()
    }

    /// GitPoulpi in alpha with bravo open as the active tab (the bar hides the
    /// button on the launch folder).
    fn start_on_bravo(&self) -> Terminal {
        let mut term = common::launch(self.alpha.root())
            .session("open-in")
            .config_dir(self.config.path())
            .size(200, 50)
            .no_exit_pause()
            .ready("alpha-only-commit")
            .spawn();
        term.send_keys("C-o").unwrap();
        term.wait_for("the management tab", STEP, |s| {
            s.contains("Repository Management") && s.contains("bravo-repo")
        })
        .unwrap_or_else(|e| panic!("{e}"));
        term.send_keys("/").unwrap();
        term.send_text("bravo").unwrap();
        term.wait_for("the filter applied", STEP, |s| {
            s.contains("Enter open best match")
        })
        .unwrap_or_else(|e| panic!("{e}"));
        term.send_keys("Enter").unwrap();
        term.wait_for("bravo open as a tab", STARTUP, |s| {
            s.contains("bravo-only-commit") && !s.contains("Repository Management")
        })
        .unwrap_or_else(|e| panic!("{e}"));
        term.wait_for("the Open in button", STEP, |s| s.contains("Open in"))
            .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
        term
    }
}

fn click_text(term: &mut Terminal, text: &str) {
    let (col, row) = term
        .screen()
        .find_all(text)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("{text:?} not on screen:\n{}", term.text()));
    term.click(col + 1, row).unwrap();
}

fn same_dir(a: &Path, b: &Path) -> bool {
    std::fs::canonicalize(a).unwrap() == std::fs::canonicalize(b).unwrap()
}

fn wait_popup(term: &Terminal) {
    term.wait_for("the Open in list", STEP, |s| {
        s.contains("First entry") && s.contains("Third entry") && s.contains("Configure")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// The folder a recorded launch ran for, after its `--flag`.
fn launched_folder(log: &str, flag: &str) -> String {
    log.trim()
        .strip_prefix(flag)
        .unwrap_or_else(|| panic!("{flag} not in {log:?}"))
        .trim()
        .to_string()
}

#[test]
fn clicking_open_in_lists_the_entries_and_picking_one_runs_it_in_the_tabs_folder() {
    let setup = Setup::new();
    let mut term = setup.start_on_bravo();
    assert!(setup.launches().is_empty());
    click_text(&mut term, "Open in");
    wait_popup(&term);
    // The default is marked (in the same frame as the list; waited for, not
    // asserted on the first frame that has the entries).
    term.wait_for("the default entry marked", STEP, |s| {
        s.contains("\u{2022} Second entry")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    click_text(&mut term, "Second entry");
    let log = setup.launched();
    let folder = launched_folder(&log, "--second");
    assert!(same_dir(Path::new(&folder), setup.bravo.root()), "{log}");
    // The pick closes the list: the launch is recorded before the next frame
    // is drawn, so the list is waited out, not read off the current screen.
    term.wait_for("the list closed", STEP, |s| !s.contains("Third entry"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

#[test]
fn the_palette_runs_the_default_entry() {
    let setup = Setup::new();
    let mut term = setup.start_on_bravo();
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text("Open in Default").unwrap();
    term.wait_for("the entry", STEP, |s| s.contains("Tab: Open in Default"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    let log = setup.launched();
    let folder = launched_folder(&log, "--second");
    assert!(same_dir(Path::new(&folder), setup.bravo.root()), "{log}");
    term.stop().unwrap();
}

#[test]
fn ctrl_alt_o_runs_the_default_entry_instead_of_going_home() {
    // `Ctrl+O` (Home) matches leniently and also fires on `Ctrl+Alt+O`.
    let setup = Setup::new();
    let mut term = setup.start_on_bravo();
    term.send_keys("M-C-o").unwrap();
    let log = setup.launched();
    assert!(!log.is_empty(), "nothing launched:\n{}", term.text());
    let folder = launched_folder(&log, "--second");
    assert!(same_dir(Path::new(&folder), setup.bravo.root()), "{log}");
    term.stop().unwrap();
}

#[test]
fn the_keyboard_path_opens_the_list_focused_and_enter_picks() {
    let setup = Setup::new();
    let mut term = setup.start_on_bravo();
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text("Open in...").unwrap();
    term.wait_for("the entry", STEP, |s| s.contains("Tab: Open In..."))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    wait_popup(&term);
    // Starts on the default (second); Down moves to the third.
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    let log = setup.launched();
    let folder = launched_folder(&log, "--third");
    assert!(same_dir(Path::new(&folder), setup.bravo.root()), "{log}");

    // Esc closes it without running anything.
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text("Open in...").unwrap();
    term.send_keys("Enter").unwrap();
    wait_popup(&term);
    term.send_keys("Escape").unwrap();
    term.wait_for("the list closed", STEP, |s| !s.contains("Third entry"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(setup.launches().lines().count(), 1);
    term.stop().unwrap();
}

#[test]
fn a_click_away_only_closes_the_popup_and_never_reaches_the_tab_under_it() {
    let setup = Setup::new();
    let mut term = setup.start_on_bravo();
    click_text(&mut term, "Open in");
    wait_popup(&term);
    // The first tab (alpha) is a plain click away: it closes the list and is
    // consumed, so the active tab stays bravo.
    let alpha_tab = setup
        .alpha
        .root()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    click_text(&mut term, &alpha_tab);
    term.wait_for("the list closed", STEP, |s| !s.contains("Third entry"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let screen = term.screen();
    assert!(
        screen.contains("bravo-only-commit") && !screen.contains("alpha-only-commit"),
        "{}",
        term.text()
    );
    assert!(setup.launches().is_empty());
    term.stop().unwrap();
}

#[test]
fn configure_opens_settings() {
    let setup = Setup::new();
    let mut term = setup.start_on_bravo();
    click_text(&mut term, "Open in");
    wait_popup(&term);
    click_text(&mut term, "Configure");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

#[test]
fn open_in_sits_in_the_title_bar_next_to_the_path_and_the_gear_left_of_the_quit_button() {
    let setup = Setup::new();
    let mut term = setup.start_on_bravo();
    let screen = term.screen();
    let (col, row) = screen.find("Open in").expect("the button");
    assert_eq!(row, 0, "Open in is on the title row:\n{}", term.text());
    let (path_col, path_row) = screen.find("GitPoulpi").expect("the title");
    assert_eq!(path_row, 0);
    assert!(col > path_col, "after the title text");
    // The settings gear: the title row too, just left of the quit button,
    // with blank cells between them so a click on the gear never quits.
    let (gear_col, gear_row) = screen.find("\u{2699}").expect("the gear");
    assert_eq!(
        gear_row,
        0,
        "the gear is on the title row:\n{}",
        term.text()
    );
    let (quit_col, quit_row) = screen.find("\u{D7}").expect("the quit button");
    assert_eq!(quit_row, 0);
    assert!(
        gear_col + 3 < quit_col,
        "a gap between the gear ({gear_col}) and the quit button ({quit_col})"
    );
    assert!(
        quit_col - gear_col < 8,
        "right next to it: {gear_col} {quit_col}"
    );
    assert!(col < gear_col, "Open in is left of the gear");
    // Nothing of the old spot is left on the tab row.
    let tab_row: String = term.text().lines().nth(1).unwrap_or("").to_string();
    assert!(!tab_row.contains('\u{2699}'), "{tab_row}");
    // Still opens the list from there, hanging under the button (not at
    // the bar's right end, where it used to be).
    term.click(col + 1, row).unwrap();
    wait_popup(&term);
    let (list_col, _) = term.screen().find("First entry").expect("a list row");
    assert!(
        list_col.abs_diff(col) < 12,
        "the list ({list_col}) hangs under the button ({col}):\n{}",
        term.text()
    );
    term.stop().unwrap();
}

/// The title bar can no longer be hidden: `Shift+T` is free and does
/// nothing, and the palette has no "Toggle Title Bar". Falsified by
/// restoring the toggle (the title row disappears).
#[test]
fn the_title_bar_cannot_be_hidden() {
    let setup = Setup::new();
    let mut term = setup.start_on_bravo();
    term.send_text("T").unwrap();
    std::thread::sleep(Duration::from_millis(400));
    let screen = term.screen();
    assert!(
        screen.line(0).contains("GitPoulpi") && screen.line(0).contains('\u{D7}'),
        "{}",
        term.text()
    );
    term.send_text("\x10").unwrap();
    term.send_text("Toggle Title").unwrap();
    std::thread::sleep(Duration::from_millis(400));
    assert!(
        !term.screen().contains("Toggle Title Bar"),
        "{}",
        term.text()
    );
    term.stop().unwrap();
}

/// The version chip is in the title bar between the app name and the path,
/// and a click on it opens the Release notes tab.
#[test]
fn the_version_chip_is_in_the_title_bar_and_opens_the_release_notes() {
    let setup = Setup::new();
    let mut term = setup.start_on_bravo();
    let screen = term.screen();
    let (chip_col, chip_row) = screen
        .find(concat!("v", env!("CARGO_PKG_VERSION")))
        .expect("the version chip");
    assert_eq!(chip_row, 0, "{}", term.text());
    let (name_col, _) = screen.find("GitPoulpi").expect("the name");
    let (dash_col, _) = screen.find("--").expect("the path separator");
    assert!(name_col < chip_col && chip_col < dash_col);
    assert!(
        screen
            .find_all(concat!("v", env!("CARGO_PKG_VERSION")))
            .iter()
            .all(|(_, r)| *r == 0),
        "the status line has no version:\n{}",
        term.text()
    );
    term.click(chip_col + 1, chip_row).unwrap();
    term.wait_for("the Release notes tab", Duration::from_secs(10), |s| {
        s.find_all("Release notes").iter().any(|(_, r)| *r <= 2)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
