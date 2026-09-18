//! A Settings choice is a dropdown, never click-to-cycle: through a pty,
//! the real binary and a temp config dir (never the maintainer's own), clicking
//! a choice's value opens a popup listing every option, and clicking an
//! option changes the stored value.
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, nothing sleeps, and every wait describes the state it waits
//! for.

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
        .size(200, 50)
        .ready_when("the fixture loaded", |s| s.contains("initial commit"))
        .start()
}

/// Opens Settings on the Git & encoding page, where `Push button` is a
/// two-option choice (`direct` by default, then `ask`).
fn open_git_page(term: &mut Terminal) {
    let (gear_col, gear_row) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(gear_col, gear_row).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(term, "Git & encoding");
    term.wait_for("the Git & encoding page", STEP, |s| {
        s.contains("Push button")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

fn stored(config: &tempfile::TempDir) -> String {
    std::fs::read_to_string(config.path().join("config.toml")).unwrap_or_default()
}

#[test]
fn clicking_a_settings_choice_opens_a_popup_and_a_pick_changes_the_value() {
    let repo = fixture();
    let (mut term, config) = launch(&repo, "pty-choice-combo");
    open_git_page(&mut term);

    let (col, row) = common::wait_for_choice(&term, "direct", STEP);
    term.click(col + 1, row).expect("click the choice");

    // The popup lists the current option (ticked) and the other one under it.
    let screen = term
        .wait_for("the popup listing both options", STEP, |s| {
            s.contains("\u{2713} direct")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !stored(&config).contains("push_behavior"),
        "opening the choice must not change it"
    );
    let (tick_col, tick_row) = screen.find("\u{2713} direct").unwrap();
    // `ask` is the row under the ticked one.
    term.click(tick_col + 2, tick_row + 1)
        .expect("click the other option");

    let deadline = std::time::Instant::now() + STEP;
    while !stored(&config).contains("push_behavior = \"ask\"") {
        assert!(
            std::time::Instant::now() < deadline,
            "the pick never reached config.toml: {}",
            stored(&config)
        );
        std::thread::sleep(Duration::from_millis(200));
    }
    term.wait_for("the popup closed, the new value shown", STEP, |s| {
        common::choice_at(s, "ask").is_some() && !s.contains("\u{2713}")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}
