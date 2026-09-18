//! `Ctrl+N` badges over the Settings screen, end to end through a pty: the
//! prefix shows a badge on each control of the page, a badge acts like a click
//! on its control (search box, sub-tab, combobox, list button), `Insert` still
//! adds a list item, and `Esc` cancels. Temp config dir only.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .build()
}

fn launch(repo: &Path, config: &Path, session: &str) -> Terminal {
    common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(200, 60)
        .no_exit_pause()
        .ready_when("the log", |s| s.contains("Message"))
        .mouse_capture()
        .spawn()
}

fn open_settings(term: &mut Terminal) {
    let (c, r) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// `Ctrl+N`: waits for the frame that shows the prefix hint (the badge key is
/// looked up in what the last frame drew).
fn open_badges(term: &mut Terminal) {
    term.send_keys("C-n").unwrap();
    term.wait_for("the prefix hint and the badges", STEP, |s| {
        s.contains("<badge>=focus")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

fn hint_gone(term: &Terminal) {
    term.wait_for("the prefix hint gone", STEP, |s| {
        !s.contains("<badge>=focus")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// The badge printed at the start of the row that shows `label` (the badge
/// hides the label's first character, so pass the label without it): the
/// character two cells left of `label`.
fn badge_before(term: &Terminal, label: &str) -> char {
    let screen = term
        .wait_for("the label under a badge", STEP, |s| {
            s.find_all(label).iter().any(|(c, _)| *c >= 30)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = screen
        .find_all(label)
        .into_iter()
        .find(|(c, _)| *c >= 30)
        .unwrap();
    screen
        .line(r)
        .chars()
        .nth(usize::from(c) - 2)
        .unwrap_or_else(|| panic!("no badge before {label}\n{}", term.text()))
}

fn saved_profiles(config: &Path) -> usize {
    std::fs::read_to_string(config.join("config.toml"))
        .unwrap_or_default()
        .lines()
        .filter(|l| l.starts_with("id = \""))
        .count()
}

/// `Ctrl+N` over Settings shows the badges and the pending prefix's hint;
/// `Esc` cancels, and the key that would have been a badge then does nothing.
#[test]
fn ctrl_n_shows_the_badges_and_escape_cancels() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-nav-cancel");
    open_settings(&mut term);
    common::settings_page(&mut term, "Open in");
    open_badges(&mut term);
    term.send_keys("Escape").unwrap();
    hint_gone(&term);
    // `4` was the Entries tab's badge: with the prefix cancelled it is nothing.
    term.send_text("4").unwrap();
    std::thread::sleep(Duration::from_millis(400));
    let screen = term.screen();
    assert!(
        screen.contains("Repo-specific settings")
            && screen.contains("Default entry")
            && !screen.contains("+ Add"),
        "{}",
        term.text()
    );
}

/// The search box wears badge `1`: pressing it opens the search popup.
#[test]
fn a_badge_focuses_the_search_box() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-nav-search");
    open_settings(&mut term);
    open_badges(&mut term);
    term.send_text("1").unwrap();
    term.wait_for("the search popup", STEP, |s| s.contains("Enter: go"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// A sub-tab badge switches to that tab (`4` is the list tab here, `3` the
/// General tab).
#[test]
fn a_badge_switches_the_sub_tab() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-nav-tab");
    open_settings(&mut term);
    common::settings_page(&mut term, "Open in");
    open_badges(&mut term);
    term.send_text("4").unwrap();
    term.wait_for("the list tab", STEP, |s| {
        s.contains("+ Add") && s.contains("Editing entry")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    hint_gone(&term);
    open_badges(&mut term);
    term.send_text("3").unwrap();
    term.wait_for("the General tab", STEP, |s| {
        s.contains("Default entry") && !s.contains("+ Add")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// A combobox badge opens that combobox (`6` is the Default entry choice on
/// Open in's General tab); an option only the open list shows proves it.
#[test]
fn a_badge_opens_a_combobox() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-nav-combo");
    open_settings(&mut term);
    common::settings_page(&mut term, "Open in");
    // The kitty entry is on every Linux list; Windows and macOS list the detected apps
    // only, and the open list marks the chosen one with a tick.
    let option = if cfg!(target_os = "macos") {
        "\u{2713} Finder"
    } else if cfg!(windows) {
        "\u{2713} File explorer"
    } else {
        "kitty: new tab (remote control)"
    };
    assert!(!term.screen().contains(option));
    open_badges(&mut term);
    term.send_text("6").unwrap();
    term.wait_for("the open choice list", STEP, |s| s.contains(option))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// The close cross wears the last badge here (`8`): it closes Settings.
#[test]
fn a_badge_presses_the_close_cross() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-nav-close");
    open_settings(&mut term);
    common::settings_page(&mut term, "Open in");
    open_badges(&mut term);
    term.send_text("8").unwrap();
    term.wait_for("Settings closed", STEP, |s| {
        !s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// A list badge presses its button: on the empty Profiles list only `+ Add`
/// is enabled, so it is the first badge after the tabs (`5`). `Insert` is
/// still the key that adds.
#[test]
fn a_badge_presses_a_list_button_and_insert_still_adds() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-nav-list");
    open_settings(&mut term);
    common::settings_page(&mut term, "Profiles");
    common::settings_tab(&mut term, "Profiles");
    term.wait_for("the list", STEP, |s| {
        s.contains("+ Add") && s.contains("No profiles yet")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(saved_profiles(config.path()), 0);

    open_badges(&mut term);
    term.send_text("5").unwrap();
    term.wait_for("a profile added", STEP, |_| {
        saved_profiles(config.path()) == 1
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    hint_gone(&term);

    term.send_keys("Insert").unwrap();
    term.wait_for("a second profile added", STEP, |_| {
        saved_profiles(config.path()) == 2
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// A text field's badge opens it for editing: what is typed lands in the
/// field and in `config.toml`.
#[test]
fn a_badge_opens_a_text_field_for_editing() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-nav-text");
    open_settings(&mut term);
    common::settings_page(&mut term, "Profiles");
    common::settings_tab(&mut term, "Profiles");
    term.wait_for("the list", STEP, |s| s.contains("+ Add"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = term.screen().find("+ Add").unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("a profile", STEP, |_| saved_profiles(config.path()) == 1)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    open_badges(&mut term);
    let badge = badge_before(&term, "uthor email");
    term.send_text(&badge.to_string()).unwrap();
    term.wait_for("the field open", STEP, |s| s.contains("\u{2588}"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("nav@example.test").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the email saved", STEP, |_| {
        std::fs::read_to_string(config.path().join("config.toml"))
            .unwrap_or_default()
            .contains("nav@example.test")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}
