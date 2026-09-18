//! The editor's status-bar chips (encoding, line breaks, indentation), end
//! to end through a pty: the real binary opens a file in its own encoding,
//! the chip opens a dropdown, a choice converts, a save writes the bytes.
//!
//! Same rules as `pty_ui.rs`: nothing located by a hard-coded coordinate
//! (rows are found from their own text), and every wait describes the whole
//! state it waits for.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const LOG_HEADER: &str = "Message";

/// A repository whose `file` holds `base` in the last commit and `bytes` in
/// the working tree (so it is a changed file the editor can be opened from).
fn repo_with(file: &str, base: &[u8], bytes: &[u8]) -> gitpoulpi_testkit::FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .write_worktree(file, base)
        .unwrap()
        .stage(".")
        .unwrap()
        .commit("base")
        .unwrap()
        .write_worktree(file, bytes)
        .unwrap()
        .build()
}

fn launch(repo: &Path, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .session(session)
        .size(180, 40)
        .no_exit_pause()
        .ready_when("the working-changes pane", |s| {
            s.contains("Unstaged") && s.contains(LOG_HEADER)
        })
        .mouse_capture()
        .start()
}

/// Opens `file` in the editor from its unstaged diff, once `ready` shows.
fn open_editor(term: &mut Terminal, file: &str, ready: &'static str) {
    let (c, r) = term
        .wait_for_text(file, STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    term.wait_for("the unstaged diff", STEP, |s| {
        s.contains(&format!("{file} - unstaged")) && s.contains(" Edit ")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (ec, er) = term.screen().find(" Edit ").unwrap();
    term.click(ec + 2, er).unwrap();
    term.wait_for("the editor", STEP, |s| {
        s.contains(&format!("Edit - {file} @ working tree")) && s.contains(ready)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// The row (and first column) of the status-bar chip `label`: the editor's
/// bar is the line right above the block, so it is the first line holding
/// the chip text next to the position `Ln`.
fn chip(s: &Screen, label: &str) -> (u16, u16) {
    let (_, bar) = s
        .find("Ln 1, Col")
        .unwrap_or_else(|| panic!("{}", s.text()));
    let line = s.line(bar);
    let at = line
        .find(label)
        .unwrap_or_else(|| panic!("{label} on the bar: {line:?}"));
    (line[..at].chars().count() as u16, bar)
}

/// Clicks the dropdown row whose text starts with `label`.
fn click_row(term: &mut Terminal, label: &str) {
    let s = term.screen();
    let (c, r) = s
        .find(label)
        .unwrap_or_else(|| panic!("{label} in the menu\n{}", s.text()));
    term.click(c + 1, r).unwrap();
}

fn wait_message(term: &mut Terminal, what: &str, needle: &'static str) {
    term.wait_for(what, STEP, |s| s.contains(needle))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn a_windows_1252_file_opens_readable_and_the_chip_converts_it_to_utf8() {
    let repo = repo_with("latin.txt", b"x\r\n", b"caf\xE9 cr\xE8me\r\n");
    let path = repo.root().join("latin.txt");
    let (mut term, _config) = launch(repo.root(), "pty-format-encoding");
    open_editor(&mut term, "latin.txt", "caf\u{e9} cr\u{e8}me");
    let s = term.screen();
    assert!(
        s.contains("Windows-1252") && s.contains("CRLF"),
        "the chips show the detected format\n{}",
        s.text()
    );
    assert!(!s.contains("modified"));

    // The chip: a dropdown with the two ways to change the encoding.
    let (c, r) = chip(&s, "Windows-1252");
    term.click(c + 2, r).unwrap();
    term.wait_for("the encoding menu", STEP, |s| {
        s.contains("Reopen with encoding") && s.contains("Convert to encoding")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    click_row(&mut term, "Convert to encoding");
    term.wait_for(
        "the list of encodings with the current one marked",
        STEP,
        |s| s.contains("Shift_JIS") && s.contains("KOI8-R") && s.contains("\u{2713} Windows-1252"),
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The plain "UTF-8" row (not "UTF-8 BOM").
    let s = term.screen();
    let (uc, ur) = s
        .find_all("UTF-8")
        .into_iter()
        .find(|&(c, r)| {
            !s.line(r)
                .chars()
                .skip(c as usize)
                .collect::<String>()
                .starts_with("UTF-8 BOM")
        })
        .unwrap_or_else(|| panic!("a UTF-8 row\n{}", s.text()));
    term.click(uc + 1, ur).unwrap();
    wait_message(
        &mut term,
        "the conversion told in the status line",
        "will be saved as UTF-8",
    );
    term.wait_for("the chip changed and the file is dirty", STEP, |s| {
        s.contains("\u{25CF} modified")
            && s.contains("caf\u{e9} cr\u{e8}me")
            && !s.contains("Reopen with")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(
        std::fs::read(&path).unwrap(),
        b"caf\xE9 cr\xE8me\r\n",
        "nothing is written before the save"
    );

    term.send_keys("C-s").unwrap();
    wait_message(&mut term, "the save", "saved latin.txt");
    assert_eq!(
        std::fs::read(&path).unwrap(),
        "caf\u{e9} cr\u{e8}me\r\n".as_bytes(),
        "UTF-8 on disk, CRLF kept"
    );
    assert!(!term.exited());
    term.stop().unwrap();
}

#[test]
fn the_line_break_chip_converts_crlf_to_lf_and_the_save_writes_it() {
    let repo = repo_with("lines.txt", b"x\n", b"one\r\ntwo\r\nthree\r\n");
    let path = repo.root().join("lines.txt");
    let (mut term, _config) = launch(repo.root(), "pty-format-eol");
    open_editor(&mut term, "lines.txt", "three");
    let s = term.screen();
    assert!(s.contains("UTF-8") && s.contains("CRLF"), "{}", s.text());

    let (c, r) = chip(&s, "CRLF");
    term.click(c + 1, r).unwrap();
    term.wait_for("the three conversions", STEP, |s| {
        s.contains("Convert to LF")
            && s.contains("Convert to CRLF")
            && s.contains("Convert to CR (classic")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    click_row(&mut term, "Convert to LF");
    wait_message(&mut term, "the conversion", "line breaks converted to LF");
    term.wait_for("the chip says LF and the menu is gone", STEP, |s| {
        s.contains("\u{25CF} modified") && !s.contains("Convert to LF") && !s.contains("CRLF")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    term.send_keys("C-s").unwrap();
    wait_message(&mut term, "the save", "saved lines.txt");
    assert_eq!(std::fs::read(&path).unwrap(), b"one\ntwo\nthree\n");

    // Undo takes the conversion back, in the buffer.
    term.send_keys("C-z").unwrap();
    term.wait_for("CRLF back", STEP, |s| {
        s.contains("CRLF") && s.contains("\u{25CF} modified")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

#[test]
fn the_indentation_chip_converts_tabs_to_four_spaces() {
    let repo = repo_with("code.txt", b"x\n", b"a\n\tb\n\t\tc\n");
    let path = repo.root().join("code.txt");
    let (mut term, _config) = launch(repo.root(), "pty-format-indent");
    open_editor(&mut term, "code.txt", "Tabs: 4");

    let s = term.screen();
    let (c, r) = chip(&s, "Tabs: 4");
    term.click(c + 1, r).unwrap();
    term.wait_for("the indentation menu", STEP, |s| {
        s.contains("Indent using spaces")
            && s.contains("Convert indentation to spaces")
            && s.contains("Size: 8")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    click_row(&mut term, "Convert indentation to spaces");
    wait_message(
        &mut term,
        "the conversion",
        "indentation converted (lines changed: 2)",
    );
    term.wait_for("the chip says Spaces: 4", STEP, |s| {
        s.contains("Spaces: 4") && !s.contains("Tabs: 4") && s.contains("\u{25CF} modified")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    term.send_keys("C-s").unwrap();
    wait_message(&mut term, "the save", "saved code.txt");
    assert_eq!(std::fs::read(&path).unwrap(), b"a\n    b\n        c\n");
    term.stop().unwrap();
}

#[test]
fn the_palette_opens_the_menus_and_they_are_driven_by_the_keyboard() {
    let repo = repo_with("latin.txt", b"x\r\n", b"caf\xE9 cr\xE8me\r\n");
    let path = repo.root().join("latin.txt");
    let (mut term, _config) = launch(repo.root(), "pty-format-palette");
    open_editor(&mut term, "latin.txt", "caf\u{e9} cr\u{e8}me");

    // Reopen with an encoding: palette, type to filter, Enter.
    term.send_keys("C-p").unwrap();
    term.send_text("Reopen with encoding").unwrap();
    term.wait_for("the palette filtered to the command", STEP, |s| {
        s.contains("Editor: Reopen with Encoding")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the list of encodings", STEP, |s| {
        s.contains("Shift_JIS") && s.contains("\u{2713} Windows-1252")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("koi8").unwrap();
    term.wait_for("the list narrowed to the typed text", STEP, |s| {
        s.contains("KOI8-R") && !s.contains("Shift_JIS")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    wait_message(
        &mut term,
        "the file read again",
        "reopened latin.txt as KOI8-R",
    );
    term.wait_for("the text decoded as KOI8-R", STEP, |s| {
        s.contains("KOI8-R") && !s.contains("caf\u{e9}") && !s.contains("\u{25CF} modified")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Line breaks, by keyboard: the current row (CRLF) is highlighted, Down
    // moves to CR, Enter converts.
    term.send_keys("C-p").unwrap();
    term.send_text("Convert Line Endings").unwrap();
    term.wait_for("the palette filtered to the command", STEP, |s| {
        s.contains("Editor: Convert Line Endings")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the three rows", STEP, |s| {
        s.contains("Convert to CR (classic")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Down Enter").unwrap();
    wait_message(&mut term, "the conversion", "line breaks converted to CR");
    term.send_keys("C-s").unwrap();
    wait_message(&mut term, "the save", "saved latin.txt");
    assert_eq!(
        std::fs::read(&path).unwrap(),
        b"caf\xE9 cr\xE8me\r",
        "the bytes keep their KOI8-R reading, the break is a lone CR"
    );
    // Esc closes a menu, not the editor.
    term.send_keys("C-p").unwrap();
    term.send_text("Change Indentation").unwrap();
    term.wait_for("the palette filtered to the command", STEP, |s| {
        s.contains("Editor: Change Indentation")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the indentation menu", STEP, |s| {
        s.contains("Indent using tabs")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for("the menu closed, the editor still open", STEP, |s| {
        !s.contains("Indent using tabs") && s.contains("Edit - latin.txt @ working tree")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
