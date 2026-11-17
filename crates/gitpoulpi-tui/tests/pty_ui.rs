//! End-to-end mouse verification of the real TUI binary, driven through a
//! pty by `xtask::pty`: terminal bytes in, the real event loop, hit-target
//! registry and daemon, and the painted characters and colors read back.
//! `TestBackend` buffer tests cannot prove a click at column 141 lands on
//! the row you think it does.
//!
//! Rules:
//! - No hard-coded coordinates: everything is located by its label.
//! - No sleeps: every step waits on a predicate about the screen.
//! - Every predicate describes the whole state it waits for. A terminal has
//!   no frame boundaries, so a partial predicate accepts a half-painted
//!   repaint (see `Terminal::wait_until`).
//!
//! `GITPOULPI_PTY_TEAR_MS=10 cargo test -p gitpoulpi-tui --test pty_ui`
//! makes the driver's reader crawl so every repaint is seen in pieces; use
//! it on any new test here.

mod common;

use common::git;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Color, Screen, Terminal};

/// Ceilings, not waits: each step returns the moment its predicate holds.
const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(20);

/// Separate from `STARTUP`: a program that has drawn nothing after this
/// long has not started, so a broken launch fails fast and specifically.
const FIRST_OUTPUT: Duration = Duration::from_secs(20);

/// A fixed size, so a layout assertion means something.
const COLS: u16 = 200;
const ROWS: u16 = 50;

/// `ACCENT` in `gitpoulpi_tui` - the color a live resize strip is
/// painted with.
const ACCENT: Color = Color::Rgb(64, 156, 255);

/// The collapsible-section arrows; `▾`/`▸` are easy to confuse on sight.
const EXPANDED_ARROW: char = '\u{25be}';
const COLLAPSED_ARROW: char = '\u{25b8}';

/// The Log pane's left border is the first pane divider, so finding the
/// pane finds the divider (no percentage arithmetic).
/// The Refs pane's top-right corner touching the Log's top-left one (the Log
/// has no title): the divider is the second of the two columns.
const LOG_PANE_CORNER: &str = "\u{256e}\u{256d}";

/// A repo with two committed files and three separate edits to one of
/// them, far enough apart that git emits three hunks.
fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    let mut lines: Vec<String> = (1..=30).map(|n| format!("l{n}")).collect();
    let original = lines.join("\n") + "\n";
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\ntwo\nthree\n", "initial")
        .unwrap()
        .commit_file("many.txt", &original, "add many")
        .unwrap();

    lines[1] = "CHANGED-TOP".to_string();
    lines[14] = "CHANGED-MIDDLE".to_string();
    lines[27] = "CHANGED-BOTTOM".to_string();
    let repo = repo
        .write_worktree("many.txt", (lines.join("\n") + "\n").as_bytes())
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\nthree\nfour\n")
        .unwrap();
    repo.build()
}

/// Launches the TUI binary in a pty against `repo` and waits until it has
/// drawn. The config dir and session name are per test, so tests never
/// share a daemon port file or the developer's live session.
fn launch(repo: &Path, session: &str) -> (Terminal, tempfile::TempDir) {
    let config = tempfile::tempdir().expect("temp config dir");
    let term = common::launch(repo)
        .session(session)
        .config_dir(config.path())
        .size(COLS, ROWS)
        .spawn();

    // Two waits, because they fail differently. First: did the program draw
    // anything? (Not a mode-setting sequence, which a pseudo-console may
    // consume instead of forwarding.)
    term.wait_for_first_output(FIRST_OUTPUT)
        .unwrap_or_else(|e| panic!("the TUI drew nothing: {e}"));

    // Second: did the repo load? The chrome draws before the daemon answers,
    // so wait for both commit messages (the log round trip completed).
    term.wait_for("the fixture's log", STARTUP, |s| {
        s.contains("add many") && s.contains("initial")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    (term, config)
}

/// Clicks the label `needle`, wherever it currently is.
fn click_label(term: &mut Terminal, needle: &str) {
    let (col, row) = term
        .wait_for_text(needle, STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    // A couple of columns in: the label padding is inside the hit rect.
    term.click(col + 1, row).expect("write the click");
}

/// Puts the third pane on the working-changes view and waits for it to be
/// fully painted. Selects the Log's WIP row by clicking its graph marker
/// `◌`, which selects without putting a caret in the message field.
fn open_working_changes(term: &mut Terminal) {
    let (col, row) = term
        .wait_for_text("\u{25CC}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col, row).expect("write the click");
    term.wait_for(
        "the working-changes pane, fully painted and with no diff open",
        STEP,
        |s| {
            s.contains("Unstaged Files")
                && s.contains("Staged Files")
                && s.contains("\u{270E} a.txt")
                && s.contains("\u{270E} many.txt")
                && !s.contains("CHANGED-MIDDLE")
        },
    )
    .unwrap_or_else(|e| panic!("{e}"));
}

fn hunk_header_rows(screen: &Screen) -> Vec<u16> {
    screen
        .find_all("@@ -")
        .into_iter()
        .map(|(_, row)| row)
        .collect()
}

/// Whether the resize strip is lit at this cell.
fn lit(screen: &Screen, col: u16, row: u16) -> bool {
    screen.cell(col, row).is_some_and(|c| c.bg == ACCENT)
}

/// The row a section header is on, and whether it currently shows
/// `arrow`.
fn section_arrow_is(screen: &Screen, section: &str, arrow: char) -> bool {
    screen
        .find(section)
        .is_some_and(|(_, row)| screen.line(row).contains(arrow))
}

/// A single click on a file row opens its diff.
#[test]
fn a_single_click_on_a_changed_file_opens_its_diff() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-single-click-diff");
    // Asserts the diff is not open yet: the precondition for "it opened".
    open_working_changes(&mut term);

    click_label(&mut term, "\u{270E} many.txt");

    // One predicate for the whole outcome: a split wait accepted the view's
    // "nothing left to show" placeholder drawn before the diff arrives.
    term.wait_for(
        "many.txt's unstaged diff, with all three of its edits",
        STEP,
        |s| {
            s.contains("many.txt - unstaged")
                && s.contains("CHANGED-TOP")
                && s.contains("CHANGED-MIDDLE")
                && s.contains("CHANGED-BOTTOM")
        },
    )
    .unwrap_or_else(|e| panic!("{e}"));
}

/// A click on a status section header collapses it; the click must reach
/// the header's hit rect.
#[test]
fn clicking_a_status_section_header_collapses_and_reexpands_it() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-collapse-section");
    open_working_changes(&mut term);

    term.wait_for("the Unstaged section expanded, with its files", STEP, |s| {
        section_arrow_is(s, "Unstaged Files", EXPANDED_ARROW)
            && s.contains("\u{270E} a.txt")
            && s.contains("\u{270E} many.txt")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    click_label(&mut term, "Unstaged Files");

    // The arrow flip and the rows disappearing are one repaint: one predicate.
    term.wait_for("the section collapsed, and its files gone", STEP, |s| {
        section_arrow_is(s, "Unstaged Files", COLLAPSED_ARROW)
            && !s.contains("\u{270E} a.txt")
            && !s.contains("\u{270E} many.txt")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    click_label(&mut term, "Unstaged Files");

    term.wait_for("the section expanded again, with its files", STEP, |s| {
        section_arrow_is(s, "Unstaged Files", EXPANDED_ARROW)
            && s.contains("\u{270E} a.txt")
            && s.contains("\u{270E} many.txt")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// Stage hunk on the second of three hunks stages only that hunk. The
/// final assertion reads `git diff --cached`: only the index says which
/// hunk was staged, and it is immune to torn frames.
#[test]
fn stage_hunk_on_the_second_of_three_hunks_stages_only_that_hunk() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-stage-second-hunk");
    open_working_changes(&mut term);
    click_label(&mut term, "\u{270E} many.txt");

    let opened = term
        .wait_for("the diff open with all three hunks", STEP, |s| {
            s.contains("many.txt - unstaged")
                && hunk_header_rows(s).len() == 3
                && s.contains("CHANGED-TOP")
                && s.contains("CHANGED-MIDDLE")
                && s.contains("CHANGED-BOTTOM")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let second = hunk_header_rows(&opened)[1];

    // The Stage hunk button is a hover affordance; a warped pointer delivers
    // no motion event, so move the pointer explicitly.
    term.move_to(60, second).unwrap();

    // Exactly one button, on the second hunk, in one predicate: mid-repaint
    // two are shown.
    let hovered = term
        .wait_for("Stage hunk offered on the second hunk alone", STEP, |s| {
            let rows = hunk_header_rows(s);
            let buttons = s.find_all("Stage hunk");
            rows.len() == 3 && buttons.len() == 1 && buttons[0].1 == rows[1]
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (button_col, button_row) = hovered.find("Stage hunk").expect("predicate just matched");
    assert_eq!(button_row, second, "guaranteed by the predicate above");

    term.click(button_col + 1, button_row).unwrap();

    // The file is staged and the unstaged diff lost exactly the middle hunk.
    term.wait_for(
        "the middle hunk staged and gone from the unstaged diff",
        STEP,
        |s| {
            s.contains("Staged Files (1)")
                && hunk_header_rows(s).len() == 2
                && !s.contains("CHANGED-MIDDLE")
                && s.contains("CHANGED-TOP")
                && s.contains("CHANGED-BOTTOM")
        },
    )
    .unwrap_or_else(|e| panic!("{e}"));

    let staged = git(repo.root(), &["diff", "--cached"]);
    assert!(
        staged.contains("CHANGED-MIDDLE"),
        "the second hunk should be staged:\n{staged}"
    );
    assert!(
        !staged.contains("CHANGED-TOP") && !staged.contains("CHANGED-BOTTOM"),
        "only the second hunk should be staged:\n{staged}"
    );

    let unstaged = git(repo.root(), &["diff"]);
    assert!(
        unstaged.contains("CHANGED-TOP") && unstaged.contains("CHANGED-BOTTOM"),
        "the other two hunks should still be unstaged:\n{unstaged}"
    );
    assert!(
        !unstaged.contains("CHANGED-MIDDLE"),
        "and the staged one should not still be pending:\n{unstaged}"
    );
}

/// Dragging a pane divider moves it and the highlight follows. Asserted
/// during the drag: a held button reports `Drag`, not `Moved`, so the
/// hover path that lights the strip does not run. Red without the fix
/// (81574de): the highlight froze at the drag's start column.
#[test]
fn dragging_a_pane_divider_moves_it_and_the_highlight_follows() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-drag-divider");

    let (divider, _) = term
        .wait_for_text(LOG_PANE_CORNER, STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    let divider = divider + 1;
    let middle = ROWS / 2;

    term.move_to(divider, middle).unwrap();
    // Lit on the divider and only there.
    term.wait_for("the divider lit, and only the divider", STEP, |s| {
        lit(s, divider, middle) && !lit(s, divider - 1, middle) && !lit(s, divider + 1, middle)
    })
    .unwrap_or_else(|e| panic!("{e}"));

    let target = divider + 20;
    term.press(divider, middle).unwrap();
    term.drag_to(target, middle).unwrap();

    // One predicate for the mid-drag state: divider moved, highlight on its
    // new column, nothing lit at the start.
    let dragging = term
        .wait_for(
            "the divider dragged right with its highlight following it",
            STEP,
            |s| match s.find(LOG_PANE_CORNER) {
                Some((moved, _)) => {
                    let moved = moved + 1;
                    moved > divider + 10 && lit(s, moved, middle) && !lit(s, divider, middle)
                }
                None => false,
            },
        )
        .unwrap_or_else(|e| panic!("{e}"));
    let moved_to = dragging
        .find(LOG_PANE_CORNER)
        .expect("predicate just matched")
        .0
        + 1;

    term.release(target, middle).unwrap();
    term.wait_for("the divider to stay where it was dropped", STEP, |s| {
        s.find(LOG_PANE_CORNER)
            .is_some_and(|(col, _)| col + 1 == moved_to)
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// The gear glyph in the toolbar's right-hand corner (`render_gear`).
const GEAR: &str = "\u{2699}";

/// Whether the settings row labeled `label` shows a ticked checkbox.
///
/// Row-scoped: `screen.contains("[x]")` would match any of the dozen
/// checkboxes.
fn checkbox_is_ticked(screen: &Screen, label: &str, ticked: bool) -> bool {
    match screen.find(label) {
        Some((_, row)) => screen
            .line(row)
            .contains(if ticked { "[x]" } else { "[ ]" }),
        None => false,
    }
}

/// Mouse-only on purpose: a click must redraw without a later keypress.
/// Red without the fix: the input drain loop re-polled at the bottom, so
/// mouse events ticked the checkbox but never repainted. A `send_keys`
/// here would hide the bug.
#[test]
fn clicking_a_preferences_checkbox_ticks_it_with_no_keystroke_anywhere() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-prefs-checkbox-click");

    let (gear_col, gear_row) = term
        .wait_for_text(GEAR, STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(gear_col, gear_row).expect("write the click");

    // One predicate for the opened state: title, sidebar, and the row in its
    // known-ticked start position (`general.auto_prune` defaults to `true`;
    // the click must untick it).
    term.wait_for("the Preferences screen open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(&mut term, "Synchronization");
    term.wait_for("Auto-prune ticked at start", STEP, |s| {
        checkbox_is_ticked(s, "Auto-prune", true)
    })
    .unwrap_or_else(|e| panic!("{e}"));

    let (label_col, label_row) = term
        .wait_for_text("Auto-prune", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(label_col + 1, label_row)
        .expect("write the click");

    term.wait_for("Auto-prune unticked by the click alone", STEP, |s| {
        checkbox_is_ticked(s, "Auto-prune", false)
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// Clicking a palette row runs that row's command through the palette's
/// own `Enter`, so a real run proves the translation. `View: Toggle Help
/// Overlay` is visible on screen and touches no repository.
#[test]
fn clicking_a_command_palette_row_runs_that_command() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-palette-click");

    term.send_keys("C-p").expect("open the palette");
    // Typed, not clicked: narrowing the list makes the target row unambiguous.
    term.send_text("Toggle Help").expect("filter the palette");

    // The predicate must pin the filter's final state: the unfiltered list and
    // every intermediate frame also show the command. The palette's title
    // counts matches, so `(1/` means the filter finished.
    let filtered = term
        .wait_for(
            "the palette filtered down to this one command, overlay not yet open",
            STEP,
            |s| {
                s.contains("Command Palette (1/")
                    && s.contains("View: Toggle Help Overlay")
                    && !s.contains("Keybindings")
            },
        )
        .unwrap_or_else(|e| panic!("{e}"));

    let (col, row) = filtered
        .find("View: Toggle Help Overlay")
        .expect("the predicate just matched");
    term.double_click(col + 1, row)
        .expect("write the double click");

    // Palette closed and overlay open, in one predicate.
    term.wait_for("the help overlay open and the palette gone", STEP, |s| {
        s.contains("Keybindings") && !s.contains("Command Palette")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// Clicking a Preferences sidebar section selects it and the body follows,
/// with no keystroke. Separate from the checkbox test: sidebar and body are
/// different branches of `PreferencesState::on_mouse`.
#[test]
fn clicking_a_preferences_sidebar_section_switches_the_body_with_no_keystroke() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-prefs-sidebar-click");

    let (gear_col, gear_row) = term
        .wait_for_text(GEAR, STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(gear_col, gear_row).expect("write the click");

    term.wait_for(
        "the Preferences screen open on General, showing a General-only setting",
        STEP,
        |s| s.contains("Remember tabs") && !s.contains("Author avatars"),
    )
    .unwrap_or_else(|e| panic!("{e}"));

    let (col, row) = term
        .wait_for_text("Appearance", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col + 1, row).expect("write the click");

    // Old page gone and the new page's setting present, in one predicate.
    term.wait_for(
        "the Appearance page, and General's settings gone",
        STEP,
        |s| s.contains("Author avatars") && !s.contains("Remember tabs"),
    )
    .unwrap_or_else(|e| panic!("{e}"));
}

/// The Home tab (Repository Management) shows the settings gear: its panel
/// must not paint over the gear glyph and its hit. Same real-click flow as
/// `clicking_a_preferences_checkbox_ticks_it_with_no_keystroke_anywhere`.
#[test]
fn clicking_the_gear_on_the_home_tab_opens_preferences() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-home-gear-click");

    term.send_keys("C-o").expect("go to the management tab");
    term.wait_for("the Repository Management tab", STEP, |s| {
        s.contains("Repository Management") && s.contains("Browse")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    let (gear_col, gear_row) = term
        .wait_for_text(GEAR, STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(gear_col, gear_row).expect("write the click");

    term.wait_for(
        "the Preferences screen open, the Repository Management tab's own body gone",
        STEP,
        |s| s.contains("Repo-specific settings") && !s.contains("Repository Management"),
    )
    .unwrap_or_else(|e| panic!("{e}"));
}

/// Opening a repository by typing its path: Browse (`b`) opens the folder
/// picker, `Tab` moves to the path field, `Enter` goes there, `Enter` again
/// chooses the folder and opens it as a tab.
#[test]
fn typing_a_path_in_the_folder_picker_and_pressing_enter_twice_opens_it_as_a_tab() {
    let repo = fixture();
    let second = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("second.txt", "hello\n", "the second repositorys own commit")
        .unwrap()
        .build();
    let (mut term, _config) = launch(repo.root(), "pty-open-path-enter");

    term.send_keys("C-o").expect("go to the management tab");
    term.wait_for("the Repository Management tab", STEP, |s| {
        s.contains("Repository Management") && s.contains("Browse")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("b").expect("Browse");
    term.wait_for("the folder picker", STEP, |s| {
        s.contains("Open a repository") && s.contains("Selected:") && s.contains("FOLDERS")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Tab").expect("to the path field");

    // Forward slashes: ConPTY's VT-input translation drops a synthetic `\`
    // before crossterm sees it.
    let second_path = second.root().display().to_string().replace('\\', "/");
    term.send_text(&second_path)
        .expect("type the second repository's path");
    // Wait for the path's last characters: `send_text` writes one key event
    // per character, so a partial match would let `Enter` fire early.
    let tail: String = second_path
        .chars()
        .rev()
        .take(12)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    let typed_tail = tail.clone();
    term.wait_for("the typed path in the field, fully typed", STEP, move |s| {
        s.contains(&typed_tail)
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("Enter").expect("go to the typed folder");
    term.wait_for(
        "the picker in the typed folder, which it knows is a repository",
        STEP,
        |s| s.contains("second.txt") && s.contains("git repository"),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").expect("choose the folder shown");

    // Picker gone and the second repository's commit on screen, in one
    // predicate.
    term.wait_for(
        "the picker closed and the second repository open as a tab",
        STEP,
        |s| s.contains("the second repositorys own commit") && !s.contains("Open a repository"),
    )
    .unwrap_or_else(|e| panic!("{e}"));
}

/// `Ctrl+N` opens the navigation prefix (hint in the footer), `Ctrl+N Shift+H`
/// toggles the Refs pane, and an unbound key cancels. Only the real
/// crossterm decoder proves `C-n` arrives as `Char('n')` with `CONTROL`
/// and is not intercepted by the terminal.
#[test]
fn ctrl_n_then_shift_h_collapses_the_refs_pane_and_a_second_ctrl_n_shift_h_restores_it() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-ctrl-n-collapse");

    term.wait_for(
        "the Refs sidebar's LOCAL section, before anything is collapsed",
        STEP,
        |s| s.contains("LOCAL"),
    )
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("C-n").expect("enter navigation mode");
    term.wait_for("the navigation prefix's footer hint", STEP, |s| {
        s.contains("Ctrl+N") && s.contains("collapse")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("H").expect("collapse the Refs pane");
    term.wait_for("the Refs pane collapsed to its restore strip", STEP, |s| {
        s.contains("Refs pane collapsed") && !s.contains("LOCAL")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Restores on a second `Ctrl+N Shift+H`.
    term.send_keys("C-n H").expect("restore the Refs pane");
    term.wait_for("the Refs pane restored, LOCAL back on screen", STEP, |s| {
        s.contains("Refs pane restored") && s.contains("LOCAL")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // An unbound key cancels; the Refs pane must still be there.
    term.send_keys("C-n").expect("enter navigation mode again");
    term.wait_for("the navigation prefix's footer hint, again", STEP, |s| {
        s.contains("Ctrl+N") && s.contains("collapse")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("z").expect("send an unbound key to cancel");
    term.wait_for("navigation canceled, LOCAL still on screen", STEP, |s| {
        s.contains("navigation canceled") && s.contains("LOCAL")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// The WIP row's inline message field: clicked, typed into, and the text
/// shows live in the composer's Summary (one buffer). The click must bring
/// up the working-changes pane; the precondition pins that the
/// `Commit details` header is on screen first.
#[test]
fn clicking_the_wip_field_and_typing_fills_the_composer_summary_too() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-wip-field-click");

    // The WIP row: one line, placeholder and counters.
    term.wait_for("the WIP row with its placeholder and counters", STEP, |s| {
        s.contains("// WIP") && s.contains("\u{270E} 2")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // A real commit first, so the pane starts on a commit's detail.
    click_label(&mut term, "add many");
    term.wait_for("a commit's detail on the right", STEP, |s| {
        s.contains("Commit details") && !s.contains("Unstaged Files")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    click_label(&mut term, "// WIP");
    term.wait_for(
        "the working-changes pane, reached by selecting the WIP row alone",
        STEP,
        |s| s.contains("Unstaged Files") && !s.contains("Commit details"),
    )
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_text("typed wip")
        .expect("type into the WIP field");
    // Both halves in one predicate: the Log's field (with its caret) and the
    // composer's Summary box, so the text appears twice.
    let screen = term
        .wait_for("the text in the WIP field and in the composer", STEP, |s| {
            s.contains("typed wip_") && s.text().matches("typed wip").count() >= 2
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !screen.text().contains("// WIP"),
        "the placeholder must give way to the text"
    );

    // Enter goes on to the Description: the caret leaves the Log, and typing
    // lands in the Description.
    term.send_keys("Enter")
        .expect("press Enter in the WIP field");
    term.send_text("desc text")
        .expect("type into the Description once focus has moved there");
    term.wait_for(
        "the Description shows the typed text, the Log's caret gone",
        STEP,
        |s| s.contains("desc text_") && !s.contains("typed wip_"),
    )
    .unwrap_or_else(|e| panic!("{e}"));
}

/// Keyboard parity: `c` in the Log selects the WIP row and puts the caret
/// in it; Esc leaves the field and keeps the text.
#[test]
fn c_in_the_log_edits_the_wip_message_without_the_mouse() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-wip-field-keyboard");
    term.wait_for("the WIP row", STEP, |s| s.contains("// WIP"))
        .unwrap_or_else(|e| panic!("{e}"));

    // The Log has the keyboard at start-up.
    term.send_keys("c").expect("press c in the Log");
    term.wait_for(
        "the WIP field focused and the working-changes pane up",
        STEP,
        |s| s.contains("Unstaged Files") && !s.contains("// WIP"),
    )
    .unwrap_or_else(|e| panic!("{e}"));

    // `j` and a space are *typed*, not taken as Log navigation.
    term.send_text("j k").expect("type into the WIP field");
    term.wait_for("the typed text in both fields", STEP, |s| {
        s.contains("j k_") && s.text().matches("j k").count() >= 2
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("Esc").expect("leave the field");
    term.wait_for("the field left, its text kept", STEP, |s| {
        s.contains("j k") && !s.contains("j k_")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// The palette's "Navigation: Focus Refs Filter" opens a collapsed Refs pane, so
/// the box being typed in is on screen, and the text never reaches the
/// status line. Red without the fix: the pane stayed collapsed and the draft
/// was visible only in the footer.
#[test]
fn the_palette_refs_filter_opens_a_collapsed_pane_and_types_in_its_box() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-palette-refs-filter");
    term.wait_for("the Refs sidebar", STEP, |s| s.contains("LOCAL"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-p").unwrap();
    term.send_text("toggle refs pane").unwrap();
    term.wait_for("the collapse row", STEP, |s| {
        s.contains("Toggle Refs Pane Collapsed")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the Refs pane collapsed", STEP, |s| !s.contains("LOCAL"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("C-p").unwrap();
    term.send_text("navigation: focus refs filter").unwrap();
    term.wait_for("the filter row", STEP, |s| {
        s.contains("Navigation: Focus Refs Filter")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the pane open again", STEP, |s| s.contains("LOCAL"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("zq9x").unwrap();
    let screen = term
        .wait_for("the draft in the sidebar box", STEP, |s| {
            s.contains(" / zq9x_")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        !common::status_line_text(&screen).contains("zq9x"),
        "the draft reached the status line: {}",
        term.text()
    );
    term.send_keys("Esc").unwrap();
}

/// `Ctrl+N` then a sub-pane's badge focuses it. The Refs filter field is
/// always the first badge target, so its badge is `1`.
#[test]
fn ctrl_n_then_a_badge_focuses_that_subpane() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-ctrl-n-badge-focus");

    term.wait_for("the Refs sidebar's LOCAL section", STEP, |s| {
        s.contains("LOCAL")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("C-n").expect("enter navigation mode");
    // (Not just "Ctrl+N" and "focus": the session name in the title is
    // `pty-ctrl-n-badge-focus` and the ordinary footer lists `Ctrl+N`, so that
    // held on the screen before the key had been read at all. And the badge
    // key is resolved against the last *painted* frame: a Unix pty delivers
    // `C-n` and `1` together, before the repaint that draws the badges.)
    term.wait_for("the navigation prefix's footer hint", STEP, |s| {
        s.contains("<badge>=focus that sub-pane")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Badge `1` is the Refs filter field.
    term.send_keys("1").expect("press badge 1");
    term.wait_for(
        "the Refs filter field focused (its footer shows only the hint)",
        STEP,
        |s| common::status_line_text(s).contains("Esc: cancel"),
    )
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("Esc").expect("cancel the filter");
}

/// `Ctrl+N Tab` and plain `t` open the tab picker listing every open tab;
/// typing narrows it, Esc closes it, Enter on a filtered row switches.
#[test]
fn ctrl_n_then_tab_opens_the_tab_picker_filters_and_switches_tabs() {
    let repo = fixture();
    let second = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("second.txt", "hello\n", "the second repositorys own commit")
        .unwrap()
        .build();
    // Every fixture root is a `repo` subdirectory, so both tabs would be
    // named "repo"; rename the folders so the filter can tell them apart.
    let first_root = repo
        .root()
        .parent()
        .expect("the first fixture's tempdir")
        .join("first-repo");
    std::fs::rename(repo.root(), &first_root)
        .expect("give the first fixture's tab a name of its own");
    let second_root = second
        .root()
        .parent()
        .expect("the second fixture's tempdir")
        .join("second-repo");
    std::fs::rename(second.root(), &second_root)
        .expect("give the second fixture's tab a name of its own");
    let (mut term, _config) = launch(&first_root, "pty-tab-picker");

    // Open the second repository as a tab (Browse-and-type-a-path).
    term.send_keys("C-o").expect("go to the management tab");
    term.wait_for("the Repository Management tab", STEP, |s| {
        s.contains("Repository Management") && s.contains("Browse")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("b").expect("Browse");
    term.wait_for("the folder picker", STEP, |s| {
        s.contains("Open a repository") && s.contains("Selected:") && s.contains("FOLDERS")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Tab").expect("to the path field");
    let second_path = second_root.display().to_string().replace('\\', "/");
    term.send_text(&second_path)
        .expect("type the second repository's path");
    let typed_tail: String = second_path
        .chars()
        .rev()
        .take(12)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    term.wait_for("the typed path in the field, fully typed", STEP, move |s| {
        s.contains(&typed_tail)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").expect("go to the typed folder");
    term.wait_for(
        "the picker in the typed folder, which it knows is a repository",
        STEP,
        |s| s.contains("second.txt") && s.contains("git repository"),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").expect("choose the folder shown");
    term.wait_for(
        "the second repository open as a tab and on screen",
        STEP,
        |s| s.contains("the second repositorys own commit") && !s.contains("Open a repository"),
    )
    .unwrap_or_else(|e| panic!("{e}"));

    // `Ctrl+N Tab` opens the picker over the second repository, listing both.
    term.send_keys("C-n").expect("enter navigation mode");
    term.wait_for("the navigation prefix's footer hint", STEP, |s| {
        s.contains("Ctrl+N") && s.contains("select tab")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Tab").expect("open the tab picker");
    term.wait_for("the tab picker, both tabs listed", STEP, |s| {
        s.contains("Select Tab (2/2)")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Esc closes it without switching.
    term.send_keys("Esc")
        .expect("close the picker without choosing");
    term.wait_for(
        "the picker closed, the second repository still on screen",
        STEP,
        |s| !s.contains("Select Tab") && s.contains("the second repositorys own commit"),
    )
    .unwrap_or_else(|e| panic!("{e}"));

    // Plain `t` (no leader) reopens it; filter to the first repository's tab.
    term.send_keys("t")
        .expect("reopen the tab picker with plain t");
    term.wait_for("the tab picker open again, both tabs listed", STEP, |s| {
        s.contains("Select Tab (2/2)")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("first")
        .expect("filter to the first repository's tab");
    term.wait_for("filtered down to the one matching tab", STEP, |s| {
        s.contains("Select Tab (1/2)")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Enter switches to the only match; the picker is gone.
    term.send_keys("Enter").expect("switch to the filtered tab");
    term.wait_for(
        "back on the first repository, the picker closed",
        STEP,
        |s| !s.contains("Select Tab") && s.contains("\u{270E} many.txt"),
    )
    .unwrap_or_else(|e| panic!("{e}"));

    // Plain `t` works from the Home tab too (owner), and switches from it.
    term.send_keys("C-o").expect("go to the management tab");
    term.wait_for("the Repository Management tab", STEP, |s| {
        s.contains("Repository Management") && s.contains("Browse")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("t").expect("open the tab picker from Home");
    term.wait_for("the tab picker over the Home tab", STEP, |s| {
        s.contains("Select Tab (")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("second")
        .expect("filter to the second repository's tab");
    term.send_keys("Enter").expect("switch from Home");
    term.wait_for(
        "the second repository on screen, Home and the picker gone",
        STEP,
        |s| !s.contains("Select Tab") && s.contains("the second repositorys own commit"),
    )
    .unwrap_or_else(|e| panic!("{e}"));
}

/// Digits typed into the Refs filter field are text, not a vim count: the
/// count prefix exists only in `InputMode::Normal`.
#[test]
fn a_digit_typed_in_the_refs_filter_goes_into_the_filter_not_a_count() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-digit-in-refs-filter");

    term.wait_for("the Refs sidebar", STEP, |s| s.contains("LOCAL"))
        .unwrap_or_else(|e| panic!("{e}"));

    // Badges resolve against the last painted frame; wait for Navigation mode
    // before the badge key (a Unix pty delivers both keys before the repaint).
    term.send_keys("C-n").expect("Ctrl+N: navigation mode");
    term.wait_for("the navigation badges", STEP, |s| {
        s.contains("<badge>=focus that sub-pane")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("1").expect("badge 1: the Refs filter");
    term.wait_for("the Refs filter field focused", STEP, |s| {
        common::status_line_text(s).contains("Esc: cancel")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_text("42").expect("type digits into the filter");
    term.wait_for(
        "the digits landed in the filter draft, not swallowed as a count",
        STEP,
        |s| {
            s.contains(" / 42_")
                && !s.contains("42\u{2026}")
                && !common::status_line_text(s).contains("42")
        },
    )
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("Esc").expect("cancel the filter");
}

/// A click outside the Refs filter releases its typing focus but keeps and
/// applies the text, as Enter would. Red without the fix: the click fell
/// through to the Log while the filter kept the keyboard.
#[test]
fn clicking_outside_the_refs_filter_releases_it_and_keeps_the_draft() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-outside-click-refs-filter");

    term.wait_for("the Refs sidebar", STEP, |s| s.contains("LOCAL"))
        .unwrap_or_else(|e| panic!("{e}"));

    // Badges resolve against the last painted frame; wait for Navigation mode
    // before the badge key.
    term.send_keys("C-n").expect("Ctrl+N: navigation mode");
    term.wait_for("the navigation badges", STEP, |s| {
        s.contains("<badge>=focus that sub-pane")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("1").expect("badge 1: the Refs filter");
    term.wait_for("the Refs filter field focused", STEP, |s| {
        common::status_line_text(s).contains("Esc: cancel")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_text("abc").expect("type into the filter");
    term.wait_for("the draft on screen with its caret", STEP, |s| {
        s.contains(" / abc_") && !common::status_line_text(s).contains("abc")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // A click on the Log releases the typing focus.
    click_label(&mut term, "add many");
    let screen = term
        .wait_for(
            "the filter field released, its draft applied and kept",
            STEP,
            |s| {
                !common::status_line_text(s).contains("Esc: cancel")
                    && s.contains("abc")
                    && !s.contains("abc_")
            },
        )
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !common::status_line_text(&screen).contains("Esc: cancel"),
        "the footer's editing hint must be gone, not just off-predicate"
    );

    // `j` now moves the Log selection instead of typing into the filter.
    term.send_keys("j")
        .expect("press j after the outside click");
    term.wait_for(
        "j moved the Log selection rather than editing the filter",
        STEP,
        |s| !common::status_line_text(s).contains("Esc: cancel") && !s.contains("abcj"),
    )
    .unwrap_or_else(|e| panic!("{e}"));
}

/// The same rule for the commit-detail file filter (`/` in the changed-files
/// list): a click elsewhere must release its focus.
#[test]
fn clicking_outside_the_commit_file_filter_releases_it_and_keeps_the_text() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-outside-click-commit-filter");

    click_label(&mut term, "add many");
    term.wait_for(
        "the commit's changed files, many.txt among them",
        STEP,
        |s| s.contains("many.txt") && s.contains("Commit details"),
    )
    .unwrap_or_else(|e| panic!("{e}"));

    // Focus the list with `Enter`, not a click: a click opens that file's diff
    // and swaps the commit list out of the central column. Then `/` opens the
    // filter.
    term.send_keys("Enter")
        .expect("focus the changed-files list without opening a diff");
    term.send_keys("/").expect("open the commit file filter");
    term.wait_for("the filter field focused, its caret up", STEP, |s| {
        s.contains("Filter files: _")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_text("xyz").expect("type into the filter");
    term.wait_for("the typed text with its caret", STEP, |s| {
        s.contains("Filter files: xyz_")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // A click elsewhere in the Log releases the focus, keeping the typed text.
    click_label(&mut term, "initial");
    term.wait_for(
        "the filter released, its text kept without the caret",
        STEP,
        |s| s.contains("Filter files: xyz") && !s.contains("Filter files: xyz_"),
    )
    .unwrap_or_else(|e| panic!("{e}"));
}

/// `<count>` then `j`/`k` moves the Log selection that many rows. Proven
/// comparatively: `initial` touched only `a.txt`, `add many` only
/// `many.txt`, and the dirty tree puts a WIP row above both. The WIP row's
/// `// WIP` label is not a marker (it is a permanent placeholder); the WIP
/// selection shows as the working-changes view (`Unstaged Files`).
#[test]
fn a_count_prefix_before_j_moves_the_log_selection_that_many_rows() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-count-prefix-log");

    term.wait_for("the WIP row on top of the Log", STEP, |s| {
        s.contains("// WIP") && s.contains("Unstaged Files")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Two single steps down (WIP, `add many`, `initial`): the reference for
    // the count motion.
    term.send_keys("j").expect("move down one row");
    term.send_keys("j").expect("move down a second row");
    term.wait_for(
        "two single `j` presses land on `initial`, showing only a.txt",
        STEP,
        |s| s.contains("a.txt") && !s.contains("many.txt") && !s.contains("Unstaged Files"),
    )
    .unwrap_or_else(|e| panic!("{e}"));

    // Back to the WIP row.
    term.send_keys("k").expect("move up one row");
    term.send_keys("k").expect("move up a second row");
    term.wait_for("back on the WIP row", STEP, |s| {
        s.contains("Unstaged Files")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // The count is shown pending in the footer before the motion lands.
    term.send_keys("2").expect("start a count of 2");
    term.wait_for("the pending count in the footer", STEP, |s| {
        s.contains("2\u{2026}")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("j")
        .expect("apply the count to a down motion");
    term.wait_for(
        "`2j` from the WIP row lands on `initial`, same as `j j` did",
        STEP,
        |s| {
            s.contains("a.txt")
                && !s.contains("many.txt")
                && !s.contains("Unstaged Files")
                && !s.contains("2\u{2026}")
        },
    )
    .unwrap_or_else(|e| panic!("{e}"));
}

/// Bare `?` over the plain graph opens the help overlay and any key closes
/// it; the baseline the Preferences case is contrasted against.
#[test]
fn bare_question_mark_opens_and_closes_the_help_overlay() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-help-bare");

    term.send_text("?").expect("send ?");
    term.wait_for("the help overlay open", STEP, |s| s.contains("Keybindings"))
        .unwrap_or_else(|e| panic!("{e}"));

    term.send_text("?").expect("send ? again to close it");
    term.wait_for("the help overlay closed", STEP, |s| {
        !s.contains("Keybindings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// `?` must open help while Preferences is open (`PreferencesState::passes_through`),
/// and the popup must draw over Preferences. A pty test because the bug
/// lived in the event loop's key dispatch.
#[test]
fn question_mark_opens_help_over_the_preferences_screen() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-help-over-prefs");

    let (gear_col, gear_row) = term
        .wait_for_text(GEAR, STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(gear_col, gear_row).expect("open Preferences");
    term.wait_for("Preferences open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_text("?")
        .expect("send ? while Preferences is open");
    term.wait_for("the help overlay open on top of Preferences", STEP, |s| {
        s.contains("Keybindings")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // `?` closes the overlay and lands back on Preferences, still open behind.
    term.send_text("?").expect("close the overlay");
    term.wait_for("back on Preferences, help closed", STEP, |s| {
        !s.contains("Keybindings") && s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// The help overlay's type-to-filter field: typing narrows the popup, `Esc`
/// clears the filter first, a second `Esc` closes it.
#[test]
fn help_overlay_filters_as_you_type_and_esc_clears_before_closing() {
    let repo = fixture();
    let (mut term, _config) = launch(repo.root(), "pty-help-filter");

    term.send_text("?").expect("open help");
    term.wait_for("the unfiltered help overlay", STEP, |s| {
        s.contains("Keybindings") && s.contains("Settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_text("settings").expect("type a filter");
    term.wait_for(
        "the overlay filtered down to the Settings paragraph",
        STEP,
        |s| {
            s.contains("filter \"settings\"")
                && s.contains("Settings")
                && !s.contains("Mouse basics")
        },
    )
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("Escape").expect("clear the filter");
    term.wait_for(
        "the filter cleared, overlay still open and unfiltered",
        STEP,
        |s| s.contains("Keybindings") && s.contains("type to filter") && !s.contains("filter \""),
    )
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("Escape").expect("close the overlay");
    term.wait_for("the overlay closed", STEP, |s| !s.contains("Keybindings"))
        .unwrap_or_else(|e| panic!("{e}"));
}

/// The mouse wheel over the Log scrolls its viewport only. Red without the
/// fix: each notch reselected whichever commit landed under the pointer.
///
/// 80 commits with zero-padded, mutually non-substring messages
/// (`commit-01`..`commit-80`). HEAD is selected by default, so its summary
/// shows in "Commit details"; wheel scrolling must not change it, clicking
/// a visible commit must.
#[test]
fn the_wheel_scrolls_the_log_without_moving_the_selection_and_a_click_after_still_selects_the_right_row(
) {
    // 80 commits: more than any Log viewport (50 rows total).
    let mut builder = FixtureRepoBuilder::new().unwrap();
    for n in 1..=80 {
        builder = builder
            .commit_file(
                &format!("f{n:02}.txt"),
                &format!("{n}\n"),
                &format!("commit-{n:02}"),
            )
            .unwrap();
    }
    let repo = builder.build();
    // Not the shared `launch()`: it waits for `fixture()`'s commit messages,
    // which this repo never prints.
    let config = tempfile::tempdir().expect("temp config dir");
    let mut term = common::launch(repo.root())
        .session("pty-log-wheel-scroll")
        .config_dir(config.path())
        .size(COLS, ROWS)
        .spawn();

    // HEAD is selected: the Log's top row and the bold summary show `commit-80`.
    term.wait_for(
        "HEAD (commit-80) selected, shown in both the Log and the details pane",
        STARTUP,
        |s| s.contains("commit-80") && s.contains("Commit details"),
    )
    .unwrap_or_else(|e| panic!("{e}"));

    // A point inside the Log body, below the pane's `LOG_PANE_CORNER`.
    let (log_col, log_row) = term
        .wait_for_text(LOG_PANE_CORNER, STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    let (wheel_col, wheel_row) = (log_col + 3, log_row + 4);

    // Scroll far enough to uncover `commit-40`: the viewport moved.
    for _ in 0..30 {
        term.scroll(true, wheel_col, wheel_row).unwrap();
    }
    // The selection must not have moved: `commit-80` is scrolled out of the
    // Log, so only the selection-driven "Commit details" summary can still
    // show it.
    term.wait_for(
        "the Log scrolled to show commit-40, while commit-80 is still \
         the selection (visible only in Commit details now, its own Log \
         row long gone)",
        STEP,
        |s| s.contains("commit-40") && s.contains("commit-80") && s.contains("Commit details"),
    )
    .unwrap_or_else(|e| panic!("{e}"));

    // Scroll further and click an old commit: only a click may select.
    for _ in 0..30 {
        term.scroll(true, wheel_col, wheel_row).unwrap();
    }
    let (c1, r1) = term
        .wait_for_text("commit-01", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c1 + 1, r1)
        .expect("click the row under the pointer");
    term.wait_for(
        "the click selected commit-01, not whatever the wheel had scrolled past",
        STEP,
        |s| s.contains("commit-01") && s.contains("Commit details") && !s.contains("commit-80"),
    )
    .unwrap_or_else(|e| panic!("{e}"));

    term.stop().unwrap();
}
