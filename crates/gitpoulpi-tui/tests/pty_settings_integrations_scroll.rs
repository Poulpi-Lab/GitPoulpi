//! Settings > Integrations on a short terminal: a connected card with its
//! SSH controls and the Radar block is taller than the page, so it scrolls.
//! The keyboard always brings the focused control into view, the wheel
//! scrolls, and the combobox at the bottom works from the keyboard. Real
//! binary through a pty; temp config dir only.
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, nothing sleeps, every wait describes the state it waits for.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const COMBO_LABEL: &str = "Request of a repository";

const PROFILE: &str = "[[integrations]]\nid = \"work\"\nkind = \"github\"\n\
    url = \"https://github.com\"\ntoken_ref = \"integration:work\"\nname = \"WorkAccount\"\n\n\
    [integrations.user]\nlogin = \"kai\"\nname = \"Kai\"\nid = \"7\"\n";

fn open_radar_block(
    session: &str,
) -> (Terminal, tempfile::TempDir, gitpoulpi_testkit::FixtureRepo) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("integrations scroll fixture")
        .unwrap()
        .build();
    let (mut term, config) = common::launch(repo.root())
        .session(session)
        .config(PROFILE)
        .size(100, 24)
        .ready_when("the fixture loaded", |s| {
            s.contains("integrations scroll fixture")
        })
        .start();
    term.send_keys("C-p").unwrap();
    term.send_text("Radar Options of a Profile").unwrap();
    term.wait_for("the palette narrowed", STEP, |s| {
        s.contains("Command Palette (1/")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    // The whole scrolled state: the block in view, the card's top and the
    // combobox scrolled out (a torn frame can show the block mid-scroll).
    term.wait_for("the Radar block scrolled into view", STEP, |s| {
        s.contains("[ ] Requests waiting for my review")
            && !s.contains("Personal access token")
            && !s.contains(COMBO_LABEL)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    (term, config, repo)
}

fn config_text(config: &tempfile::TempDir) -> String {
    std::fs::read_to_string(config.path().join("config.toml")).unwrap_or_default()
}

#[test]
fn tab_down_to_the_combobox_shows_it_and_a_key_changes_it() {
    let (mut term, config, _repo) = open_radar_block("pty-integrations-scroll-tab");
    // The card's top is scrolled away; the combobox is below the fold.
    let screen = term.screen();
    assert!(
        !screen.contains("Personal access token"),
        "the card's top should be scrolled out:\n{}",
        screen.text()
    );
    assert!(
        !screen.contains(COMBO_LABEL),
        "the combobox starts hidden:\n{}",
        screen.text()
    );

    // Review, Open, Drafts, Issues, Namespaces, Regular expressions, then the combobox.
    for _ in 0..6 {
        term.send_keys("Tab").unwrap();
    }
    let screen = term
        .wait_for("the combobox scrolled into view", STEP, |s| {
            s.contains(COMBO_LABEL) && s.contains("Ask")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(screen.contains("Ask"), "{}", screen.text());

    // Enter opens it, Down moves to "Clone it", Enter picks it.
    term.send_keys("Enter").unwrap();
    term.wait_for("the popup open", STEP, |s| {
        s.contains("Clone it") && s.contains("Review through the API")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    let deadline = std::time::Instant::now() + STEP;
    loop {
        let text = config_text(&config);
        if text.contains("review_mode = \"clone\"") {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "config.toml never held the mode:\n{text}\n{}",
            term.text()
        );
        std::thread::yield_now();
    }

    // Back up with Shift+Tab: the top of the block follows the focus.
    for _ in 0..6 {
        term.send_keys("BTab").unwrap();
    }
    term.wait_for("the first checkbox in view", STEP, |s| {
        s.contains("[ ] Requests waiting for my review")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn the_wheel_scrolls_the_card_both_ways() {
    let (mut term, _config, _repo) = open_radar_block("pty-integrations-scroll-wheel");
    let (col, row) = term
        .wait_for_text("[ ] Requests waiting for my review", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    for _ in 0..8 {
        term.scroll(false, col + 2, row).unwrap();
    }
    term.wait_for("the card's top back in view", STEP, |s| {
        s.contains("Personal access token")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    for _ in 0..12 {
        term.scroll(true, col + 2, row).unwrap();
    }
    term.wait_for("the end of the card", STEP, |s| s.contains(COMBO_LABEL))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}
