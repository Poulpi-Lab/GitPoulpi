//! Settings search and the Editor page, end to end through a pty: the real
//! binary, the real daemon, a temp config dir (never the developer's own).
//!
//! - the search popup finds a setting by label or by description and
//!   jumps to it (keyboard `/` and `Ctrl+F`, the mouse button, a click on
//!   a result row);
//! - the Editor page's settings change what the editor and the diff
//!   actually draw.
//!
//! Same rules as `pty_ui.rs`: nothing located by a hard-coded coordinate,
//! every wait describes the whole state it waits for.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
/// The popup border title: shown only while the sidebar search box is focused.
const POPUP: &str = "Enter: go";
const LOG_HEADER: &str = "Message";

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .write_worktree("notes.txt", b"\talpha\nbeta\n")
        .unwrap()
        .stage(".")
        .unwrap()
        .commit("base")
        .unwrap()
        .write_worktree("notes.txt", b"\talpha\nbeta\ngamma\n")
        .unwrap()
        .build()
}

fn launch(repo: &Path, config: &Path, session: &str) -> Terminal {
    common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(200, 50)
        .no_exit_pause()
        .ready_when("the working-changes pane", |s| {
            s.contains("Unstaged") && s.contains(LOG_HEADER)
        })
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

fn stored(config: &Path) -> String {
    std::fs::read_to_string(config.join("config.toml")).unwrap_or_default()
}

/// The screen row holding `needle`.
fn row_of(term: &Terminal, needle: &str) -> Option<String> {
    let screen = term.screen();
    let (_, row) = screen.find(needle)?;
    Some(screen.line(row))
}

#[test]
fn search_finds_a_setting_by_label_and_by_description_and_jumps_to_it() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-settings-search");
    open_settings(&mut term);

    // Keyboard: `/` opens the popup, typing narrows it, Enter jumps.
    term.send_keys("/").unwrap();
    term.wait_for("the search popup", STEP, |s| s.contains(POPUP))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("tab size").unwrap();
    term.wait_for("the Tab size hit, with its page", STEP, |s| {
        s.contains("Tab size") && s.contains("\u{00B7}  Diff & editor")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the Editor page on its Tab size row", STEP, |s| {
        s.contains("Diff & editor - global")
            && s.contains("Syntax highlighting")
            && !s.contains(POPUP)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // A word of a *description*: the Hunks-only help says "changed hunks".
    term.send_keys("C-f").unwrap();
    term.wait_for("the search popup again", STEP, |s| s.contains(POPUP))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("whole file hunks").unwrap();
    term.wait_for("Hunks only, found through its description", STEP, |s| {
        s.contains("Hunks only") && s.contains("(description)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the Diff & editor page", STEP, |s| {
        s.contains("Diff & editor - global") && s.contains("Hunks only") && !s.contains(POPUP)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Esc closes the popup alone, not Settings.
    term.send_keys("/").unwrap();
    term.wait_for("the popup", STEP, |s| s.contains(POPUP))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").unwrap();
    term.wait_for("the popup gone, Settings still open", STEP, |s| {
        !s.contains(POPUP) && s.contains("Diff & editor - global")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn the_search_box_anchors_the_popup_under_itself_and_works_with_the_mouse() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-settings-search-mouse");
    open_settings(&mut term);

    // The title-row button is gone; the box is at the top of the sidebar.
    assert!(
        !term.screen().contains("/ Search settings"),
        "{}",
        term.text()
    );
    let (bc, br) = term
        .wait_for_text("Search settings...", STEP)
        .unwrap_or_else(|e| panic!("the box placeholder is drawn: {e}\n{}", term.text()));
    assert!(!term.screen().contains(POPUP), "closed until focused");

    // Click-to-focus: the popup opens directly under the box, left-aligned.
    term.click(bc + 2, br).unwrap();
    term.wait_for("the popup opened by the box", STEP, |s| s.contains(POPUP))
        .unwrap_or_else(|e| panic!("{e}"));
    let (pc, pr) = term.screen().find(POPUP).unwrap();
    assert_eq!(pr, br + 1, "popup top border is the row under the box");
    let border = term.screen().line(pr);
    let corner = border.chars().position(|ch| ch == '\u{256D}').unwrap();
    assert!(pc > corner as u16, "title is inside the border");
    assert_eq!(
        corner as u16 + 1,
        bc,
        "popup is left-aligned with the box (text starts one cell in)"
    );

    term.send_text("line numbers").unwrap();
    let (rc, rr) = term
        .wait_for_text("\u{00B7}  Diff & editor", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(rr > pr, "results are inside the popup, under the box");
    // A click on the result row selects it, a double click jumps.
    term.double_click(rc, rr).unwrap();
    term.wait_for("the Editor page", STEP, |s| {
        s.contains("Diff & editor - global") && s.contains("Line numbers") && !s.contains(POPUP)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn an_outside_click_closes_the_search_popup_without_falling_through() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-settings-search-outside");
    open_settings(&mut term);
    term.send_keys("/").unwrap();
    term.wait_for("the popup", STEP, |s| s.contains(POPUP))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("tab").unwrap();
    term.wait_for("results", STEP, |s| s.contains("\u{00B7}"))
        .unwrap_or_else(|e| panic!("{e}"));
    // Click a sidebar page row that is outside the popup (Git & encoding
    // sits under the repo header, below the popup's ten rows).
    let (c, r) = term
        .wait_for_text("Repo-specific settings", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r + 2).unwrap();
    term.wait_for("the popup gone, Settings open", STEP, |s| {
        !s.contains(POPUP) && s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The click did not select that page: the General page is still shown.
    assert!(
        !term.screen().contains("Git & encoding -"),
        "{}",
        term.text()
    );
}

#[test]
fn up_down_move_in_the_results_and_esc_clears_the_box() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-settings-search-nav");
    open_settings(&mut term);
    term.send_keys("C-f").unwrap();
    term.wait_for("the popup", STEP, |s| s.contains(POPUP))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("size").unwrap();
    term.wait_for("results", STEP, |s| s.contains("\u{00B7}"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Down").unwrap();
    term.send_keys("Up").unwrap();
    term.send_keys("Escape").unwrap();
    term.wait_for("closed, box empty again", STEP, |s| {
        !s.contains(POPUP)
            && s.contains("Search settings...")
            && s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn the_palette_reaches_the_settings_search() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-settings-search-palette");
    term.send_keys("C-p").unwrap();
    term.wait_for("the palette", STEP, |s| s.contains("Command"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("Search Settings").unwrap();
    term.wait_for("the command listed", STEP, |s| {
        s.contains("App: Search Settings...")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("Settings open on the search popup", STEP, |s| {
        s.contains(POPUP) && s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// Opens `notes.txt`'s unstaged diff, then the editor over it.
fn open_the_editor(term: &mut Terminal) {
    let (c, r) = term
        .wait_for_text("notes.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    term.wait_for("the unstaged diff", STEP, |s| {
        s.contains("notes.txt - unstaged") && s.contains("gamma") && s.contains(" Edit ")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (ec, er) = term.screen().find(" Edit ").unwrap();
    term.click(ec + 2, er).unwrap();
    term.wait_for("the editor", STEP, |s| {
        s.contains("Edit - notes.txt @ working tree") && s.contains("alpha")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn editor_settings_change_the_editor_tab_stops_gutter_status_and_saved_endings() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        "[editor]\ntab_size = 8\neol = \"crlf\"\nline_numbers = false\n",
    )
    .unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-settings-editor-cfg");
    open_the_editor(&mut term);

    // The tab is eight cells wide and there is no number column: the
    // border, then eight blanks, then the text.
    let row = row_of(&term, "alpha").expect("alpha is on screen");
    assert!(
        row.contains("\u{2502}        alpha"),
        "tab_size = 8, no gutter: {row:?}"
    );
    // The status bar says what the next save writes: CRLF, for an LF file.
    assert!(term.screen().contains("CRLF"), "{}", term.text());

    term.send_text("X").unwrap();
    term.send_keys("C-s").unwrap();
    term.wait_for("the save", STEP, |s| s.contains("saved notes.txt"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(
        std::fs::read(repo.root().join("notes.txt")).unwrap(),
        b"X\talpha\r\nbeta\r\ngamma\r\n",
        "editor.eol = crlf converts the LF file on save"
    );
}

#[test]
fn defaults_draw_the_gutter_and_four_wide_tabs() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-settings-editor-default");
    open_the_editor(&mut term);
    let row = row_of(&term, "alpha").expect("alpha is on screen");
    assert!(
        row.contains("\u{2502}1     alpha"),
        "number, one space, four tab cells: {row:?}"
    );
    assert!(term.screen().contains("LF"), "{}", term.text());
}

#[test]
fn a_toggle_on_the_editor_page_is_saved_and_changes_the_diff() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), config.path(), "pty-settings-editor-live");

    // Settings first: find Line numbers by search and switch it off.
    open_settings(&mut term);
    term.send_keys("/").unwrap();
    term.wait_for("the popup", STEP, |s| s.contains(POPUP))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("line numbers").unwrap();
    term.wait_for("the hit", STEP, |s| s.contains("\u{00B7}  Diff & editor"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the Editor page", STEP, |s| {
        s.contains("Diff & editor - global") && !s.contains(POPUP)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Space").unwrap();
    term.wait_for("the toggle saved", STEP, |_| {
        stored(config.path()).contains("line_numbers = false")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", stored(config.path())));
    term.send_keys("Escape").unwrap();
    term.wait_for_text_to_go("Repo-specific settings", STEP)
        .unwrap_or_else(|e| panic!("{e}"));

    // The diff now has a blank number column: the added line `gamma` is
    // its third line, and no `3` stands before its marker.
    let (c, r) = term
        .wait_for_text("notes.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).unwrap();
    term.wait_for("the unstaged diff", STEP, |s| {
        s.contains("notes.txt - unstaged") && s.contains("+gamma")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let row = row_of(&term, "+gamma").unwrap();
    assert!(!row.contains("3 +gamma"), "number column blanked: {row:?}");
}
