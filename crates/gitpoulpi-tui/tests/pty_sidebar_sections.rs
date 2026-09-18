//! The left pane's sections can be shown and hidden (per repository, in its state file):
//! the right-click checklist on a section header, the palette twins, the last
//! visible section that cannot be unchecked, "Reset to default", and the choice
//! surviving a restart. Real binary, daemon and system git in a pty; temp
//! config dirs only.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("sections base")
        .unwrap()
        .tag("v1.0")
        .unwrap()
        .build()
}

fn launch(repo: &FixtureRepo, session: &str, config: &Path) -> Terminal {
    common::launch(repo.root())
        .session(session)
        .config_dir(config)
        .size(200, 60)
        .ready("sections base")
        .spawn()
}

/// Whether a left-pane section header (`LOCAL`, `REMOTE`, `TAGS`) is on screen,
/// not counting a checklist row (those carry a box glyph before the name).
fn has_header(screen: &Screen, name: &str) -> bool {
    screen
        .text()
        .lines()
        .any(|l| l.contains(name) && !l.contains('\u{2611}') && !l.contains('\u{2610}'))
}

fn stored(config: &Path) -> String {
    std::fs::read_to_string(config).unwrap_or_default()
}

fn wait_stored(config: &Path, needle: &str) {
    let deadline = std::time::Instant::now() + STEP;
    while !stored(config).contains(needle) {
        assert!(
            std::time::Instant::now() < deadline,
            "{needle:?} never reached the repo state: {}",
            stored(config)
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}

fn open_checklist(term: &mut Terminal, header: &str) {
    let (c, r) = term
        .screen()
        .find(header)
        .unwrap_or_else(|| panic!("no {header} header\n{}", term.text()));
    term.right_click(c + 1, r).expect("right-click the header");
    term.wait_for("the checklist", STEP, |s| {
        s.contains("Sections (click to toggle") && s.contains("Reset to default")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

fn click_row(term: &mut Terminal, glyph: char, name: &str) {
    let screen = term.screen();
    let (c, r) = screen
        .text()
        .lines()
        .enumerate()
        .find_map(|(r, l)| {
            l.find(glyph).filter(|_| l.contains(name)).map(|byte| {
                let col = l[..byte].chars().count();
                (col as u16, r as u16)
            })
        })
        .unwrap_or_else(|| panic!("no {glyph} {name} row\n{}", term.text()));
    term.click(c + 2, r).expect("click the checklist row");
}

#[test]
fn the_checklist_hides_and_shows_sections_and_persists() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let state = repo.state_dir().join("repo-state.toml");
    let mut term = launch(&repo, "pty-sections", config.path());
    term.wait_for("the sections", STEP, |s| {
        has_header(s, "LOCAL") && has_header(s, "REMOTE") && has_header(s, "TAGS")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Right-click the TAGS header, uncheck TAGS: the section is not painted.
    open_checklist(&mut term, "TAGS");
    click_row(&mut term, '\u{2611}', "TAGS");
    term.wait_for("the TAGS box unchecked", STEP, |s| {
        s.text()
            .lines()
            .any(|l| l.contains('\u{2610}') && l.contains("TAGS"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    // The checklist must be gone too: the pane already hides TAGS while the
    // checklist is up, and an unread Escape meets the next C-p as Alt+Ctrl+P.
    term.wait_for("TAGS gone from the pane, checklist closed", STEP, |s| {
        !has_header(s, "TAGS") && has_header(s, "LOCAL") && !s.contains("Sections (click to toggle")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    wait_stored(&state, "sidebar_sections_hidden");
    assert!(stored(&state).contains("tags"), "{}", stored(&state));

    // The keyboard twin: the palette command shows it again.
    term.send_keys("C-p").unwrap();
    term.send_text("Show/Hide Tags").unwrap();
    term.wait_for("the palette entry", STEP, |s| {
        s.contains("Left pane: Show/Hide Tags")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("TAGS back", STEP, |s| has_header(s, "TAGS"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Hide TAGS and REMOTE with the keyboard on the checklist (palette opens
    // it, Down/Space-less: Enter toggles the row under the cursor).
    term.send_keys("C-p").unwrap();
    term.send_text("Show/Hide Sections").unwrap();
    term.wait_for("the menu entry", STEP, |s| {
        s.contains("Left pane: Show/Hide Sections")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the checklist", STEP, |s| {
        s.contains("Sections (click to toggle")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Cursor starts on LOCAL: Down once is REMOTE; Enter unchecks it.
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("REMOTE unchecked, menu still open", STEP, |s| {
        s.contains("Reset to default")
            && s.text()
                .lines()
                .any(|l| l.contains('\u{2610}') && l.contains("REMOTE"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Uncheck TAGS too: only LOCAL is left, and its row is grayed (always shown).
    click_row(&mut term, '\u{2611}', "TAGS");
    term.wait_for("LOCAL as the last section", STEP, |s| {
        s.contains("(always shown)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Clicking the grayed LOCAL row changes nothing.
    click_row(&mut term, '\u{2611}', "LOCAL");
    std::thread::sleep(Duration::from_millis(400));
    assert!(
        term.screen()
            .text()
            .lines()
            .any(|l| l.contains('\u{2611}') && l.contains("LOCAL")),
        "the last section stayed checked\n{}",
        term.text()
    );
    term.send_keys("Escape").unwrap();
    term.wait_for("only LOCAL painted", STEP, |s| {
        has_header(s, "LOCAL") && !has_header(s, "REMOTE") && !has_header(s, "TAGS")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    wait_stored(&state, "remote");
    let _ = term.stop();

    // A restart keeps the choice.
    let mut term = launch(&repo, "pty-sections", config.path());
    term.wait_for("LOCAL only after a restart", STEP, |s| {
        has_header(s, "LOCAL") && !has_header(s, "REMOTE") && !has_header(s, "TAGS")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Reset to default shows everything again.
    open_checklist(&mut term, "LOCAL");
    let (c, r) = term.screen().find("Reset to default").unwrap();
    term.click(c + 1, r).expect("click Reset");
    term.send_keys("Escape").unwrap();
    term.wait_for("every section back", STEP, |s| {
        has_header(s, "LOCAL") && has_header(s, "REMOTE") && has_header(s, "TAGS")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let _ = term.stop();
}
