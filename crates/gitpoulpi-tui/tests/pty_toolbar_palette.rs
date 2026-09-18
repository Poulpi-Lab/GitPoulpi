//! The toolbar's button group is centered (Search still clicks),
//! and the palette's key column shows full chords, never a bare prefix.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;

const STEP: Duration = Duration::from_secs(30);

fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("centered fixture")
        .unwrap()
        .build()
}

#[test]
fn the_centered_toolbar_search_button_still_opens_the_find_bar() {
    let repo = fixture();
    let (mut term, _config) = common::launch(repo.root())
        .session("pty-toolbar-centered")
        // The test is about the centered group: the default is start-aligned.
        .config("[ui]\ntoolbar_align = \"center\"\n")
        .size(200, 30)
        .ready_when("the toolbar", |s| {
            s.contains("Undo") && s.contains("Search")
        })
        .mouse_capture()
        .start();
    let screen = term.screen();
    let (undo_col, row) = screen.find("Undo").unwrap();
    assert!(
        undo_col > 30,
        "Undo should be centered, not left: {undo_col}"
    );
    let (search_col, _) = screen.find("\u{2315} Search").unwrap();
    assert!(
        search_col < 170,
        "Search should not hug the gear: {search_col}"
    );
    term.click(search_col + 3, row).expect("click Search");
    term.send_text("centered").unwrap();
    term.wait_for_text("1 of 1", STEP)
        .unwrap_or_else(|e| panic!("Search did not open the find bar: {e}"));
    term.stop().unwrap();
}

#[test]
fn the_palette_shows_the_full_chord_for_leader_commands() {
    let repo = fixture();
    let (mut term, _config) = common::launch(repo.root())
        .session("pty-palette-chords")
        .size(200, 40)
        .ready_when("the repository", |s| s.contains("centered fixture"))
        .start();
    term.send_keys("C-p").unwrap();
    term.send_text("Toggle Refs Pane Collapsed").unwrap();
    term.wait_for("the full chord", STEP, |s| s.contains("Ctrl+N h"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Esc").unwrap();
    term.wait_for_text_to_go("Command Palette", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-p").unwrap();
    term.send_text("Focus Log Pane").unwrap();
    term.wait_for("prefix plus badge", STEP, |s| s.contains("Ctrl+N <badge>"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

/// "Open in" has keys (Alt+O, Ctrl+Alt+O): the palette row shows them
/// right-aligned, so the palette teaches them like every other command.
#[test]
fn the_palette_shows_the_open_in_shortcuts() {
    let repo = fixture();
    let (mut term, _config) = common::launch(repo.root())
        .session("pty-palette-open-in")
        .size(200, 40)
        .ready_when("the repository", |s| s.contains("centered fixture"))
        .start();
    term.send_keys("C-p").unwrap();
    term.send_text("Open in Default").unwrap();
    term.wait_for("the default Open in key", STEP, |s| {
        s.contains("Tab: Open in Default") && s.contains("Ctrl+Alt+O")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}
