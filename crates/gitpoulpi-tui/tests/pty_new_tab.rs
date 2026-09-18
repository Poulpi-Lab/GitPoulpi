//! The tab bar's `+` and the transient "New tab": `+` / `Alt+T` open it,
//! `Esc` returns to the previous tab, picking a repository adds its tab at
//! the end of the bar, and a middle-click closes a repository tab.

mod common;

use std::path::{Path, PathBuf};
use std::time::Duration;

use gitpoulpi_session::GlobalSession;
use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Button, Terminal};

const STEP: Duration = Duration::from_secs(30);
const SESSION: &str = "pty-new-tab";

/// A fixture repository whose folder is called `name` (the tab's label).
fn repo(name: &str, file: &str, subject: &str) -> (gitpoulpi_testkit::FixtureRepo, PathBuf) {
    let fx = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file(file, "x\n", subject)
        .unwrap()
        .build();
    let root = fx.root().parent().unwrap().join(name);
    std::fs::rename(fx.root(), &root).unwrap();
    (fx, root)
}

/// Launches in `alpha` with `others` in the recent list (so Home lists them).
fn start(alpha: &Path, others: &[&Path], config: &Path) -> Terminal {
    let mut toml = String::new();
    for path in others {
        toml.push_str(&format!(
            "[[repos]]\npath = '{}'\nname = '{}'\nlast_opened = 1\n\n",
            path.display(),
            path.file_name().unwrap().to_string_lossy()
        ));
    }
    std::fs::write(config.join("recent.toml"), toml).unwrap();
    common::launch(alpha)
        .session(SESSION)
        .config_dir(config)
        .size(200, 50)
        .no_exit_pause()
        .ready("alpha-only-commit")
        .mouse_capture()
        .spawn()
}

fn bar(term: &Terminal) -> String {
    term.text().lines().nth(1).unwrap_or("").to_string()
}

#[test]
fn clicking_plus_opens_a_new_tab_and_escape_returns_to_the_previous_tab() {
    let (_a, alpha) = repo("orbit", "a.txt", "alpha-only-commit");
    let (_b, bravo) = repo("moonbase", "b.txt", "bravo-only-commit");
    let config = tempfile::tempdir().unwrap();
    let mut term = start(&alpha, &[&bravo], config.path());
    let screen = term.screen();
    let (col, row) = screen
        .find(" + ")
        .unwrap_or_else(|| panic!("no + in the tab bar\n{}", term.text()));
    assert_eq!(row, 1, "the + sits on the tab bar");
    assert!(
        bar(&term).find("orbit").unwrap() < bar(&term).find(" + ").unwrap(),
        "the + follows the last tab: {}",
        bar(&term)
    );
    // The `+` is a menu (New tab, Radar, Workspaces); its first row is New tab.
    term.click(col + 1, row).unwrap();
    term.wait_for("the + menu", STEP, |s| s.contains("New tab"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the New tab", STEP, |s| {
        s.contains("OPEN REPOSITORIES") && s.text().lines().nth(1).unwrap_or("").contains("New tab")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for("back on the repository tab", STEP, |s| {
        s.contains("alpha-only-commit")
            && !s.contains("OPEN REPOSITORIES")
            && !s.text().lines().nth(1).unwrap_or("").contains("New tab")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

#[test]
fn the_new_tab_key_then_a_recent_repository_opens_it_as_the_last_tab() {
    let (_a, alpha) = repo("orbit", "a.txt", "alpha-only-commit");
    let (_b, bravo) = repo("moonbase", "b.txt", "bravo-only-commit");
    let config = tempfile::tempdir().unwrap();
    let mut term = start(&alpha, &[&bravo], config.path());
    term.send_keys("M-t").unwrap();
    term.wait_for("the New tab", STEP, |s| {
        s.contains("OPEN REPOSITORIES") && s.text().lines().nth(1).unwrap_or("").contains("New tab")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The search field has the focus already: typing filters at once.
    term.send_text("moon").unwrap();
    term.wait_for("the filter", STEP, |s| s.contains("Enter open best match"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("moonbase shown", STEP, |s| {
        s.contains("bravo-only-commit") && !s.contains("OPEN REPOSITORIES")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let bar = bar(&term);
    assert!(
        bar.find("orbit").unwrap() < bar.find("moonbase").unwrap(),
        "moonbase is the last tab: {bar}"
    );
    assert!(!bar.contains("New tab"), "{bar}");
    term.stop().unwrap();
}

#[test]
fn a_middle_click_closes_a_repository_tab() {
    let (_a, alpha) = repo("orbit", "a.txt", "alpha-only-commit");
    let (_b, bravo) = repo("moonbase", "b.txt", "bravo-only-commit");
    let config = tempfile::tempdir().unwrap();
    let mut term = start(&alpha, &[], config.path());
    // Open the second tab from outside, as `gitpoulpi repo open` does.
    let file = config
        .path()
        .join("sessions")
        .join(format!("{SESSION}.toml"));
    let mut session = GlobalSession::load(&file).unwrap();
    session.open(&bravo);
    session.save(&file).unwrap();
    let mut tries = 0;
    loop {
        term.send_keys("Down").unwrap();
        std::thread::sleep(Duration::from_millis(400));
        if bar(&term).contains("moonbase") {
            break;
        }
        tries += 1;
        assert!(tries < 20, "{}", term.text());
    }
    // The window still shows orbit (the session file's active tab is
    // untouched by the outside open); middle-click the *other* tab.
    let (col, row) = term.screen().find("moonbase").unwrap();
    term.click_with(Button::Middle, col, row).unwrap();
    term.wait_for("the confirmation", STEP, |s| s.contains("Close the tab"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the tab gone", STEP, |s| {
        !s.text().lines().nth(1).unwrap_or("").contains("moonbase")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(bar(&term).contains("orbit"), "{}", bar(&term));
    term.stop().unwrap();
}
