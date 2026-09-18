//! The Settings rows for the formerly "coming" keys are live: shown without
//! the "coming" tag, editable, and the choice reaches `config.toml`. Real
//! binary, temp config dir, driven by `xtask::pty`.
//! Rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, nothing sleeps.

mod common;

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

fn launch(repo: &FixtureRepo, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .size(200, 70)
        .ready_when("the fixture loaded", |s| s.contains("initial commit"))
        .start()
}

fn open_settings(term: &mut Terminal) {
    let (gear_col, gear_row) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(gear_col, gear_row).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

fn stored(config: &tempfile::TempDir) -> String {
    std::fs::read_to_string(config.path().join("config.toml")).unwrap_or_default()
}

#[test]
fn the_auto_stash_rows_are_live_and_a_pick_reaches_the_config() {
    let repo = fixture();
    let (mut term, config) = launch(&repo, "pty-settings-autostash");
    open_settings(&mut term);
    common::settings_page(&mut term, "Synchronization");

    let screen = term
        .wait_for("the Auto-stash rows", STEP, |s| {
            s.contains("Auto-stash before checkout,") && s.contains("Include untracked files in")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    for label in [
        // The label column wraps: the first line carries the row.
        "Auto-stash before checkout,",
        "Include untracked files in",
    ] {
        let (_, row) = screen.find(label).unwrap();
        assert!(
            !screen.line(row).contains("coming"),
            "{label} is still marked coming: {}",
            screen.line(row)
        );
    }

    let (col, row) = common::choice_at(&screen, "ask").expect("the choice shows `ask`");
    let (_, label_row) = screen.find("Auto-stash before checkout,").unwrap();
    assert_eq!(row, label_row, "the choice is on the label's first line");
    term.click(col + 1, row).expect("click the choice");
    let screen = term
        .wait_for("the popup listing the three modes", STEP, |s| {
            s.contains("\u{2713} ask") && s.contains("always") && s.contains("never")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = screen.find("\u{2713} ask").unwrap();
    term.click(c + 2, r + 1).expect("click `always`");

    let deadline = std::time::Instant::now() + STEP;
    while !stored(&config).contains("auto_stash = \"always\"") {
        assert!(
            std::time::Instant::now() < deadline,
            "the pick never reached config.toml: {}",
            stored(&config)
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}

#[test]
fn the_https_credentials_row_is_a_live_dropdown() {
    let repo = fixture();
    let (mut term, config) = launch(&repo, "pty-settings-https");
    open_settings(&mut term);
    common::settings_page(&mut term, "SSH & credentials");
    let screen = term
        .wait_for("the HTTPS credentials row", STEP, |s| {
            common::choice_at(s, "credential_helper").is_some()
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = common::choice_at(&screen, "credential_helper").unwrap();
    assert!(!screen.line(row).contains("coming"), "{}", screen.line(row));
    term.click(col + 1, row).expect("click the choice");
    let screen = term
        .wait_for("the popup listing the three modes", STEP, |s| {
            s.contains("\u{2713} credential_helper") && s.contains("prompt") && s.contains("none")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    // `none` is the third option, two rows under the ticked first one.
    let (c, r) = screen.find("\u{2713} credential_helper").unwrap();
    term.click(c + 2, r + 2).expect("click `none`");
    let deadline = std::time::Instant::now() + STEP;
    while !stored(&config).contains("https_mode = \"none\"") {
        assert!(
            std::time::Instant::now() < deadline,
            "the pick never reached config.toml: {}",
            stored(&config)
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}

#[test]
fn the_default_encoding_rows_are_live_dropdowns() {
    let repo = fixture();
    let (mut term, config) = launch(&repo, "pty-settings-encoding");
    open_settings(&mut term);
    common::settings_page(&mut term, "Git & encoding");
    let screen = term
        .wait_for("both Default encoding rows", STEP, |s| {
            s.find_all("Default encoding").len() >= 2 && common::choice_at(s, "utf-8").is_some()
        })
        .unwrap_or_else(|e| panic!("{e}"));
    for (_, row) in screen.find_all("Default encoding") {
        assert!(!screen.line(row).contains("coming"), "{}", screen.line(row));
    }
    let (col, row) = common::choice_at(&screen, "utf-8").unwrap();
    term.click(col + 1, row).expect("click the choice");
    let screen = term
        .wait_for("the popup listing the encodings", STEP, |s| {
            s.contains("\u{2713} utf-8") && s.contains("windows-1252")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    // `windows-1252` is the option under the ticked `utf-8`.
    let (c, r) = screen.find("\u{2713} utf-8").unwrap();
    term.double_click(c + 2, r + 1).expect("pick windows-1252");
    let deadline = std::time::Instant::now() + STEP;
    while !stored(&config).contains("default_encoding = \"windows-1252\"") {
        assert!(
            std::time::Instant::now() < deadline,
            "the pick never reached config.toml: {}",
            stored(&config)
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}

#[test]
fn keep_submodules_up_to_date_is_a_live_checkbox() {
    let repo = fixture();
    let (mut term, config) = launch(&repo, "pty-settings-submodules");
    open_settings(&mut term);
    common::settings_page(&mut term, "Synchronization");
    let screen = term
        .wait_for("the Keep submodules up to date row", STEP, |s| {
            s.contains("Keep submodules up to date")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (_, row) = screen.find("Keep submodules up to date").unwrap();
    let line = screen.line(row);
    assert!(!line.contains("coming"), "{line}");
    let box_at = line.find("[ ]").expect("an unchecked box");
    let col = line[..box_at].chars().count() as u16;
    term.click(col + 1, row).expect("click the checkbox");
    let deadline = std::time::Instant::now() + STEP;
    while !stored(&config).contains("keep_submodules_up_to_date = true") {
        assert!(
            std::time::Instant::now() < deadline,
            "the toggle never reached config.toml: {}",
            stored(&config)
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}
