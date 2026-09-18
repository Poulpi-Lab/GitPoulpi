//! Following the active tab's folder. While the active tab is not the
//! folder GitPoulpi was started in, the tab bar shows "Quit & cd here" (only
//! when started with `--cwd-file`, the wrapper's flag) and the "Open in"
//! combobox;
//! `ui.tab_dir_follow = "auto"` removes the buttons and makes every quit
//! write the folder and every newly opened tab open a terminal.
//!
//! The terminal is a fake launcher script recording its arguments: no real
//! terminal is ever started. Temp config dir and fixture repos only.

#![cfg(unix)]

mod common;

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
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
        let script = work.path().join("fake-term.sh");
        std::fs::write(
            &script,
            format!(
                "#!/bin/sh\necho \"$@\" >> '{}'\n",
                work.path().join("launches.log").display()
            ),
        )
        .unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        Self {
            alpha,
            bravo,
            config,
            work,
        }
    }

    fn write_config(&self, follow: &str) {
        std::fs::write(
            self.config.path().join("config.toml"),
            format!(
                "[ui]\ngraph_renderer = \"text\"\ntab_dir_follow = \"{follow}\"\n\
                 terminal_launch = \"custom\"\nterminal_launch_command = \"{fake} --here {{repo}}\"\n\
                 [[ui.open_in]]\nlabel = \"Fake terminal\"\ncommand = \"{fake} --here ${{cwd}}\"\n",
                fake = self.work.path().join("fake-term.sh").display()
            ),
        )
        .unwrap();
    }

    fn cwd_file(&self) -> PathBuf {
        self.work.path().join("cwd-out")
    }

    fn launches(&self) -> String {
        std::fs::read_to_string(self.work.path().join("launches.log")).unwrap_or_default()
    }

    fn start(&self, with_cwd_file: bool) -> Terminal {
        let mut launch = common::launch(self.alpha.root())
            .session("tab-dir")
            .config_dir(self.config.path())
            .size(200, 50)
            .no_exit_pause()
            .ready("alpha-only-commit");
        if with_cwd_file {
            launch = launch.arg("--cwd-file").arg(self.cwd_file());
        }
        launch.spawn()
    }
}

/// Opens bravo as a second tab through the management tab.
fn open_bravo(term: &mut Terminal) {
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
}

fn wait_exit(term: &Terminal) {
    for _ in 0..200 {
        if term.exited() {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    panic!("the TUI did not exit:\n{}", term.text());
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

#[test]
fn quit_and_cd_here_writes_the_active_tabs_folder_to_cwd_file() {
    let setup = Setup::new();
    setup.write_config("buttons");
    let mut term = setup.start(true);
    // On the launch folder: Open in yes, Quit & cd no.
    assert!(term.screen().contains("Open in"), "{}", term.text());
    assert!(!term.screen().contains("Quit & cd here"), "{}", term.text());
    open_bravo(&mut term);
    term.wait_for("both buttons", STEP, |s| {
        s.contains("Quit & cd here") && s.contains("Open in")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    click_text(&mut term, "Quit & cd here");
    wait_exit(&term);
    let written = std::fs::read_to_string(setup.cwd_file()).expect("--cwd-file written");
    assert!(
        same_dir(Path::new(&written), setup.bravo.root()),
        "{written}"
    );
}

#[test]
fn a_plain_quit_leaves_cwd_file_empty() {
    let setup = Setup::new();
    setup.write_config("buttons");
    let mut term = setup.start(true);
    open_bravo(&mut term);
    term.send_keys("q").unwrap();
    wait_exit(&term);
    let written = std::fs::read_to_string(setup.cwd_file()).unwrap_or_default();
    assert!(written.is_empty(), "{written:?}");
}

#[test]
fn open_in_runs_the_picked_entry_in_the_tabs_folder() {
    let setup = Setup::new();
    setup.write_config("buttons");
    let mut term = setup.start(false);
    open_bravo(&mut term);
    term.wait_for("the Open in button", STEP, |s| s.contains("Open in"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Without the wrapper there is no cd button, only the terminal one.
    assert!(!term.screen().contains("Quit & cd here"), "{}", term.text());
    assert!(setup.launches().is_empty());
    click_text(&mut term, "Open in");
    term.wait_for("the list", STEP, |s| s.contains("Fake terminal"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    click_text(&mut term, "Fake terminal");
    for _ in 0..100 {
        if !setup.launches().is_empty() {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let log = setup.launches();
    let arg = log.trim().strip_prefix("--here ").expect(&log);
    assert!(same_dir(Path::new(arg), setup.bravo.root()), "{log}");
    term.stop().unwrap();
}

#[test]
fn the_palette_twin_explains_the_wrapper_when_it_is_missing() {
    let setup = Setup::new();
    setup.write_config("buttons");
    let mut term = setup.start(false);
    open_bravo(&mut term);
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text("quit and cd").unwrap();
    term.wait_for("the entry", STEP, |s| s.contains("Tab: Quit and cd Here"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the setup hint", STEP, |s| s.contains("shell-init"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!term.exited());
    term.stop().unwrap();
}

#[test]
fn auto_mode_keeps_open_in_hides_quit_cd_cds_on_every_quit_and_opens_a_terminal_per_new_tab() {
    let setup = Setup::new();
    setup.write_config("auto");
    let mut term = setup.start(true);
    // The launch tab (and restored ones) never open a terminal.
    assert!(setup.launches().is_empty());
    open_bravo(&mut term);
    for _ in 0..100 {
        if !setup.launches().is_empty() {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let log = setup.launches();
    let arg = log.trim().strip_prefix("--here ").expect(&log);
    assert!(same_dir(Path::new(arg), setup.bravo.root()), "{log}");
    assert_eq!(log.lines().count(), 1, "{log}");
    let screen = term.screen();
    assert!(
        !screen.contains("Quit & cd here") && screen.contains("Open in"),
        "{}",
        term.text()
    );
    // A plain quit now writes the active tab's folder.
    term.send_keys("q").unwrap();
    wait_exit(&term);
    let written = std::fs::read_to_string(setup.cwd_file()).expect("--cwd-file written");
    assert!(
        same_dir(Path::new(&written), setup.bravo.root()),
        "{written}"
    );
}
