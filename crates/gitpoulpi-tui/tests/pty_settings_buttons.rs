//! Settings inline buttons through the real binary: the alias [Create]
//! button runs by a click and by Enter, and a destructive button asks first
//! (Cancel changes nothing). HOME and the config dir are scratch directories.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const MESSAGE: &str = "buttons probe commit";

fn open_settings(page: &str, session: &str) -> (Terminal, tempfile::TempDir, tempfile::TempDir) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit(MESSAGE)
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        "[ui]\ngraph_renderer = \"text\"\ncommand_alias = \"gp\"\n\n[installation]\nprimary = \"p-pty\"\n",
    )
    .unwrap();
    let mut term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .scratch_home(home.path())
        .size(200, 70)
        .no_exit_pause()
        .ready(MESSAGE)
        .spawn();
    let (col, row) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col, row).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(&mut term, page);
    // Keep the repo alive as long as the terminal: leak it into the tempdirs.
    std::mem::forget(repo);
    (term, config, home)
}

/// Scrolls the page body until `text` is on screen.
fn scroll_to(term: &mut Terminal, text: &str) {
    common::scroll_settings_to(term, text, Duration::from_secs(30));
}

fn wait_link(term: &Terminal, link: &std::path::Path) {
    let deadline = std::time::Instant::now() + Duration::from_secs(60);
    while std::fs::symlink_metadata(link).is_err() {
        assert!(
            std::time::Instant::now() < deadline,
            "the alias was never created\n{}",
            term.text()
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// Where the Create button holds still. `scroll_to` returns as soon as the
/// text shows, but the wheel events it sent while waiting are still being
/// read under load and keep moving the page, so a click at the first
/// position lands on whatever scrolled there.
fn settled_create(term: &Terminal) -> (u16, u16) {
    term.wait_for_settled_text("Create: command alias", Duration::from_millis(400), STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()))
}

#[test]
fn clicking_the_alias_create_button_creates_it() {
    let (mut term, _config, home) = open_settings("Launcher", "settings-buttons-click");
    scroll_to(&mut term, "Create: command alias");
    let link = common::alias_file(home.path(), "gp");
    // A click that raced a late scroll hits nothing, so press again, located
    // afresh, until the link exists (it is only pressed again while absent).
    let deadline = std::time::Instant::now() + xtask::pty::scaled(STEP);
    while std::fs::symlink_metadata(&link).is_err() {
        assert!(
            std::time::Instant::now() < deadline,
            "the alias was never created\n{}",
            term.text()
        );
        let (col, row) = settled_create(&term);
        assert!(!term.text().contains("\u{2192} run"));
        term.click(col + 1, row).unwrap();
        let wait = std::time::Instant::now() + Duration::from_secs(3);
        while std::fs::symlink_metadata(&link).is_err() && std::time::Instant::now() < wait {
            std::thread::sleep(Duration::from_millis(100));
        }
    }
    let _ = term.stop();
}

#[test]
fn enter_on_the_selected_alias_create_row_presses_its_button() {
    let (mut term, _config, home) = open_settings("Launcher", "settings-buttons-enter");
    scroll_to(&mut term, "Create: command alias");
    let link = common::alias_file(home.path(), "gp");
    // A click left of the button (the empty label column) only selects the
    // row, then Enter presses the button. A click lost to a torn frame
    // selects nothing and Enter then does nothing: try again, located afresh.
    let deadline = std::time::Instant::now() + xtask::pty::scaled(STEP);
    while std::fs::symlink_metadata(&link).is_err() {
        assert!(
            std::time::Instant::now() < deadline,
            "the alias was never created\n{}",
            term.text()
        );
        let (col, row) = settled_create(&term);
        term.click(col - 3, row).unwrap();
        std::thread::sleep(Duration::from_millis(500));
        assert!(
            std::fs::symlink_metadata(&link).is_err(),
            "selecting the row must not press the button"
        );
        term.send_keys("Enter").unwrap();
        let wait_until = std::time::Instant::now() + Duration::from_secs(5);
        while std::fs::symlink_metadata(&link).is_err() && std::time::Instant::now() < wait_until {
            std::thread::sleep(Duration::from_millis(100));
        }
    }
    wait_link(&term, &link);
    let _ = term.stop();
}

#[test]
fn forget_credentials_asks_and_cancel_closes_the_question() {
    let (mut term, _config, _home) = open_settings("SSH & credentials", "settings-buttons-forget");
    let screen = term
        .wait_for("the Forget button", STEP, |s| {
            s.contains("Forget all usernames and passwords")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (col, row) = screen.find("Forget all usernames and passwords").unwrap();
    term.click(col + 1, row).unwrap();
    term.wait_for("the question", STEP, |s| s.contains("undone"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap(); // Cancel is focused
    term.wait_for("the question closed, Settings still open", STEP, |s| {
        !s.contains("undone") && s.contains("SSH & credentials - global")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let _ = term.stop();
}
