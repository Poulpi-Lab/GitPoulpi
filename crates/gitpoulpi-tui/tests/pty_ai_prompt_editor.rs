#![cfg(feature = "ai")]
//! The Settings > AI "Custom instructions" editor is saved and quit with
//! the mouse (Save / Cancel buttons), driven through a pty by
//! `xtask::pty`. A temp config dir only; the rules are `pty_ui.rs`'s.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

fn settled(term: &Terminal) -> Screen {
    term.wait_for_stable(Duration::from_millis(150), STEP)
        .unwrap_or_else(|e| panic!("{e}"))
}

/// Opens Settings > AI > Global "Custom instructions" editor.
fn open_editor(session: &str) -> (Terminal, tempfile::TempDir, gitpoulpi_testkit::FixtureRepo) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .build();
    let config = tempfile::tempdir().expect("temp config dir");
    std::fs::write(config.path().join("config.toml"), "[ai]\nenabled = true\n").unwrap();
    let mut term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(200, 50)
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| {
        s.contains("initial commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = term.wait_for_text("\u{2699}", STEP).unwrap();
    term.click(c, r).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(&mut term, "AI");
    term.wait_for("the AI page", STEP, |s| s.contains("Custom instructions"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = term.wait_for_text("Custom instructions", STEP).unwrap();
    term.click(c + 1, r).expect("select the row");
    settled(&term);
    term.send_keys("Enter").expect("open the editor");
    term.wait_for("the editor", STEP, |s| {
        s.contains("Save") && s.contains("Cancel")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    (term, config, repo)
}

/// The button's label position on the footer row (the row holding both).
fn button(s: &Screen, label: &str) -> (u16, u16) {
    s.find_all(label)
        .into_iter()
        .find(|&(_, r)| s.line(r).contains("Save") && s.line(r).contains("Cancel"))
        .unwrap_or_else(|| panic!("no {label} button. Screen:\n{}", s.text()))
}

fn config_text(config: &tempfile::TempDir) -> String {
    std::fs::read_to_string(config.path().join("config.toml")).unwrap_or_default()
}

#[test]
fn clicking_save_persists_the_text() {
    let (mut term, config, _repo) = open_editor("pty-ai-prompt-save");
    term.send_text("Always answer in haiku").unwrap();
    term.wait_for("the text typed", STEP, |s| {
        s.contains("Always answer in haiku")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = button(&settled(&term), "Save");
    term.click(c + 1, r).expect("click Save");
    term.wait_for("the editor closed", STEP, |s| !s.contains("Save"))
        .unwrap_or_else(|e| panic!("{e}"));
    let deadline = std::time::Instant::now() + STEP;
    while !config_text(&config).contains("Always answer in haiku") {
        assert!(
            std::time::Instant::now() < deadline,
            "never saved:\n{}",
            config_text(&config)
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}

#[test]
fn clicking_cancel_discards_the_text() {
    let (mut term, config, _repo) = open_editor("pty-ai-prompt-cancel");
    term.send_text("Never persisted words").unwrap();
    term.wait_for("the text typed", STEP, |s| {
        s.contains("Never persisted words")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = button(&settled(&term), "Cancel");
    term.click(c + 1, r).expect("click Cancel");
    // Typed text asks before it is thrown away.
    let discard_button = |s: &Screen| {
        s.find_all("Discard")
            .into_iter()
            .find(|&(_, row)| s.line(row).contains("Keep"))
    };
    let screen = term
        .wait_for("the discard question and its buttons", STEP, |s| {
            s.contains("Discard changes?") && discard_button(s).is_some()
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = discard_button(&screen).expect("the Discard button");
    term.click(c + 1, r).expect("click Discard");
    term.wait_for("the editor closed", STEP, |s| {
        !s.contains("Cancel") && !s.contains("Discard changes?")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    settled(&term);
    assert!(!config_text(&config).contains("Never persisted words"));
    assert!(!snap(&term).contains("Never persisted words"));
}

/// Hovering lights the button; a click outside the popup (on the page
/// behind, the Settings sidebar, and out in the frame) does nothing.
#[test]
fn hover_feedback_and_no_click_through() {
    let (mut term, config, _repo) = open_editor("pty-ai-prompt-modal");
    term.send_text("kept text").unwrap();
    let screen = settled(&term);
    let (c, r) = button(&screen, "Cancel");
    let rest = screen.cell(c + 1, r).unwrap().bg;
    term.move_to(c + 1, r).unwrap();
    let hovered = term
        .wait_for("Cancel lit under the pointer", STEP, |s| {
            s.cell(c + 1, r).unwrap().bg != rest
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert_ne!(hovered.cell(c + 1, r).unwrap().bg, rest);

    // On the sidebar's General entry (behind the popup's left edge area)
    // and in a far corner: the editor stays, text intact, page unchanged.
    let (gc, gr) = screen.find("Git & encoding").expect("sidebar entry");
    term.click(gc, gr).unwrap();
    term.click(0, 49).unwrap();
    let after = settled(&term);
    assert!(
        after.contains("Save")
            && after.contains("kept text")
            && after.contains("Custom instructions"),
        "the editor must survive outside clicks. Screen:\n{}",
        after.text()
    );
    assert!(!config_text(&config).contains("kept text"));
}

/// Typing `${re` offers `${repo}` (with its description); Enter inserts
/// it. `Ctrl+T` opens the Variables dropdown, whose pick lands at the
/// caret. Save writes both, as typed, to `config.toml`.
#[test]
fn the_autocomplete_and_the_variables_dropdown_insert_variables() {
    let (mut term, config, _repo) = open_editor("pty-ai-prompt-vars");
    term.send_text("${re").unwrap();
    term.wait_for("the popup listing ${repo}", STEP, |s| {
        s.contains("${repo}") && s.contains("Repository folder name")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the popup gone, the variable inserted", STEP, |s| {
        s.contains("${repo}") && !s.contains("Repository folder name")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("C-t").unwrap();
    term.wait_for("the dropdown", STEP, |s| {
        s.contains("${uilang}") && s.contains("UI language")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the dropdown closed", STEP, |s| {
        !s.contains("UI language (English name)")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // A made-up variable is warned about, not refused.
    term.send_text(" ${nope}").unwrap();
    term.wait_for("the warning", STEP, |s| {
        s.contains("Unknown variable: ${nope}")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    let (c, r) = button(&settled(&term), "Save");
    term.click(c + 1, r).expect("click Save");
    term.wait_for("the editor closed", STEP, |s| !s.contains("Save"))
        .unwrap_or_else(|e| panic!("{e}"));
    let deadline = std::time::Instant::now() + STEP;
    while !config_text(&config).contains("${repo}${uilang} ${nope}") {
        assert!(
            std::time::Instant::now() < deadline,
            "never saved:\n{}",
            config_text(&config)
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// The Variables button opens the dropdown with the mouse; the palette's
/// "AI: Insert Prompt Variable" reaches the same editor from anywhere.
#[test]
fn the_variables_button_and_the_palette_entry_open_the_dropdown() {
    let (mut term, _config, _repo) = open_editor("pty-ai-prompt-vars-button");
    let (c, r) = term
        .wait_for_text("Variables", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 1, r).expect("click Variables");
    term.wait_for("the dropdown", STEP, |s| s.contains("${branch}"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").unwrap();
    term.wait_for("the dropdown closed", STEP, |s| !s.contains("${branch}"))
        .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn the_palette_opens_the_editor_with_the_dropdown_ready() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .build();
    let config = tempfile::tempdir().expect("temp config dir");
    std::fs::write(config.path().join("config.toml"), "[ai]\nenabled = true\n").unwrap();
    let mut term = common::launch(repo.root())
        .session("pty-ai-prompt-vars-palette")
        .config_dir(config.path())
        .size(200, 50)
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| {
        s.contains("initial commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-p").unwrap();
    term.send_text("prompt variable").unwrap();
    term.wait_for("the entry", STEP, |s| {
        s.contains("AI: Insert Prompt Variable")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the editor with its dropdown", STEP, |s| {
        s.contains("Custom instructions") && s.contains("${uilang}") && s.contains("${email}")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the variable inserted", STEP, |s| {
        s.contains("${uilang}") && !s.contains("${email}")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// Polls the config file until it contains `needle`.
fn wait_saved(config: &tempfile::TempDir, needle: &str) {
    let deadline = std::time::Instant::now() + STEP;
    while !config_text(config).contains(needle) {
        assert!(
            std::time::Instant::now() < deadline,
            "never saved {needle:?}:\n{}",
            config_text(config)
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// The Quick insert chips live in the editor, above the text, and not on
/// the settings page behind it; a click inserts at the caret.
#[test]
fn the_quick_insert_row_is_in_the_editor_above_the_text_and_a_click_inserts_at_the_caret() {
    let (mut term, config, _repo) = open_editor("pty-ai-prompt-chips-click");
    let screen = settled(&term);
    assert!(
        screen.contains("Quick insert: [+ Respond in <language>]"),
        "{}",
        screen.text()
    );
    let (_, chips_row) = screen.find_all("Quick insert").into_iter().next().unwrap();
    term.send_text("AB").unwrap();
    term.wait_for("the text typed", STEP, |s| s.contains("AB"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (_, text_row) = settled(&term).find_all("AB").into_iter().next().unwrap();
    assert!(chips_row < text_row, "chips {chips_row}, text {text_row}");
    term.send_keys("Left").unwrap();
    let (c, r) = settled(&term)
        .find_all("[+ Keep responses concise]")
        .into_iter()
        .next()
        .expect("the second chip");
    term.click(c + 2, r).expect("click the chip");
    term.wait_for("the sentence inserted", STEP, |s| {
        s.contains("A Keep responses concise. B")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = button(&settled(&term), "Save");
    term.click(c + 1, r).expect("click Save");
    wait_saved(&config, "A Keep responses concise. B");
}

/// The keyboard twin: Tab moves to the row, Right chooses, Enter inserts
/// and returns to the text.
#[test]
fn the_keyboard_reaches_the_quick_insert_row_and_inserts() {
    let (mut term, config, _repo) = open_editor("pty-ai-prompt-chips-keys");
    assert!(settled(&term).contains("Tab: quick insert"));
    term.send_keys("Tab").unwrap();
    term.send_keys("Right").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the sentence inserted", STEP, |s| {
        s.contains("Keep responses concise.")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Back in the text: typing continues after the sentence.
    term.send_text("!").unwrap();
    term.wait_for("typed after it", STEP, |s| {
        s.contains("Keep responses concise.!")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-s").unwrap();
    wait_saved(&config, "Keep responses concise.!");
}

/// The settings page behind the editor no longer carries the chips.
#[test]
fn the_settings_page_has_no_quick_insert_row() {
    let (mut term, _config, _repo) = open_editor("pty-ai-prompt-chips-page");
    // The opening click may have left a line break: Esc then asks, and
    // Discard (the second button) closes.
    term.send_keys("Escape").unwrap();
    let _ = term.wait_for("the question or the page", STEP, |s| {
        s.contains("Discard changes?") || !s.contains("Ctrl+S: save")
    });
    if snap(&term).contains("Discard changes?") {
        term.send_keys("Right").unwrap();
        term.send_keys("Enter").unwrap();
    }
    term.wait_for("the editor closed", STEP, |s| !s.contains("Ctrl+S: save"))
        .unwrap_or_else(|e| panic!("{e}"));
    let screen = settled(&term);
    assert!(screen.contains("Custom instructions"), "{}", screen.text());
    assert!(!screen.contains("Quick insert"), "{}", screen.text());
    assert!(!screen.contains("[+ "), "{}", screen.text());
}

/// A click in the text places the caret there: typing goes in at the
/// clicked position.
#[test]
fn clicking_in_the_text_places_the_caret() {
    let (mut term, config, _repo) = open_editor("pty-ai-prompt-caret-click");
    term.send_text("hello world").unwrap();
    let (c, r) = term
        .wait_for_text("hello world", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 6, r).expect("click before the w");
    settled(&term);
    term.send_text("X").unwrap();
    term.wait_for("typed at the click", STEP, |s| s.contains("hello Xworld"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-s").unwrap();
    wait_saved(&config, "hello Xworld");
}

/// Dragging over the text selects it: the next character replaces the
/// selection.
#[test]
fn dragging_over_the_text_selects_it() {
    let (mut term, config, _repo) = open_editor("pty-ai-prompt-caret-drag");
    term.send_text("hello world").unwrap();
    let (c, r) = term
        .wait_for_text("hello world", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.drag((c, r), (c + 5, r)).expect("drag over hello");
    settled(&term);
    term.send_text("Z").unwrap();
    term.wait_for("the selection replaced", STEP, |s| s.contains("Z world"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!snap(&term).contains("hello"));
    term.send_keys("C-s").unwrap();
    wait_saved(&config, "Z world");
}

/// The screen once it has settled: a torn frame is not a place to locate a
/// cell or compare colors in.
fn snap(term: &xtask::pty::Terminal) -> xtask::pty::Screen {
    term.wait_for_stable(
        std::time::Duration::from_millis(300),
        std::time::Duration::from_secs(30),
    )
    .unwrap_or_else(|e| panic!("{e}"))
}
