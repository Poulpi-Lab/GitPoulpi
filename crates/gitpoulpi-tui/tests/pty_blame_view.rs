//! End to end through a pty: the Blame view - annotations grouped per hunk
//! on the right (or the left), the detail card, its actions, hunk keys and
//! the width of the annotation column.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const LOG_HEADER: &str = "Message";
const BLAME_TITLE: &str = "Blame - notes.txt @";

/// `notes.txt`, six lines. The first commit (by the testkit author) wrote all
/// of them; the second, by Bob Builder, rewrote lines 3 and 4. So the blame
/// has three hunks: 1-2 (first), 3-4 (Bob's), 5-6 (first).
fn repo() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file(
            "notes.txt",
            "row one\nrow two\nrow three\nrow four\nrow five\nrow six\n",
            "add notes",
        )
        .unwrap()
        .build();
    std::fs::write(
        repo.root().join("notes.txt"),
        "row one\nrow two\nrow three!\nrow four!\nrow five\nrow six\n",
    )
    .unwrap();
    repo.git(&["add", "notes.txt"]);
    repo.git(&[
        "-c",
        "user.name=Bob Builder",
        "-c",
        "user.email=bob@example.com",
        "commit",
        "-q",
        "-m",
        "edit middle",
    ]);
    repo
}

fn launch(repo: &Path, session: &str, config: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .no_commit_column()
        .session(session)
        .config(config)
        .size(200, 40)
        .no_exit_pause()
        .ready_when("the commit detail of HEAD", |s| {
            s.contains("View all files") && s.contains(LOG_HEADER)
        })
        .mouse_capture()
        .start()
}

/// Opens notes.txt's Blame from the commit's file list and the diff bar.
fn open_blame(term: &mut Terminal) {
    let (c, r) = term
        .wait_for_text("notes.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    let (bc, br) = term
        .wait_for_text(" Blame ", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(bc + 2, br).unwrap();
    term.wait_for("the Blame view", STEP, |s| {
        s.contains(BLAME_TITLE) && s.contains("row six") && s.contains("Bob Builder")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// The blame annotation's author: not the commit details' author line (the
/// one that also carries `parent:`), which the right pane now shows first.
fn annotation_author(screen: &xtask::pty::Screen) -> Option<(u16, u16)> {
    screen
        .find_all("Bob Builder")
        .into_iter()
        .find(|&(_, r)| !screen.line(r).contains("parent:"))
}

fn lines_with(term: &Terminal, needle: &str) -> Vec<String> {
    term.text()
        .lines()
        .filter(|l| l.contains(needle))
        .map(str::to_string)
        .collect()
}

/// A summary is printed once per hunk, not on every line of it, and the
/// annotation column is to the right of the code by default.
#[test]
fn annotations_are_grouped_per_hunk_and_sit_on_the_right() {
    let repo = repo();
    let (mut term, _config) = launch(repo.root(), "pty-blame-group", "");
    open_blame(&mut term);

    // Two hunks of the first commit, one line carrying its summary each
    // (the second line of each: "row two", "row six"); Bob's hunk has it once.
    // Only the central column: the right pane shows HEAD's message too.
    let rows = |needle: &str| {
        lines_with(&term, needle)
            .into_iter()
            .map(|l| l.chars().take(125).collect::<String>())
            .filter(|l| l.contains("row ") && l.contains(needle))
            .count()
    };
    assert_eq!(rows("add notes"), 2, "{}", term.text());
    assert_eq!(rows("edit middle"), 1, "{}", term.text());
    // Bob's name is on the first row of his hunk, right of that row's code.
    let line = lines_with(&term, "row three!")
        .into_iter()
        .find(|l| l.contains("Bob Builder"))
        .unwrap_or_else(|| panic!("{}", term.text()));
    assert!(
        line.find("Bob Builder").unwrap() > line.find("row three!").unwrap(),
        "annotation on the right: {line}"
    );
    term.stop().unwrap();
}

#[test]
fn ui_blame_side_left_puts_the_annotations_before_the_code() {
    let repo = repo();
    let (mut term, _config) = launch(
        repo.root(),
        "pty-blame-left",
        "[ui]\nblame_side = \"left\"\n",
    );
    open_blame(&mut term);
    let line = lines_with(&term, "row three!")
        .into_iter()
        .find(|l| l.contains("Bob Builder"))
        .unwrap_or_else(|| panic!("{}", term.text()));
    assert!(
        line.find("Bob Builder").unwrap() < line.find("row three!").unwrap(),
        "annotation on the left: {line}"
    );
    term.stop().unwrap();
}

/// `n` / `p` move between hunks; the card (`i`) tells which one is selected,
/// and "Show commit in Log" leaves the view for the commit in the Log.
#[test]
fn hunk_keys_move_and_the_card_acts_on_the_commit() {
    let repo = repo();
    let head = repo.git(&["rev-parse", "HEAD"]);
    let head = head.trim();
    let (mut term, _config) = launch(repo.root(), "pty-blame-card", "");
    open_blame(&mut term);

    // Line 1 is in the first commit's hunk; `n` goes to Bob's.
    term.send_keys("n").unwrap();
    term.send_keys("i").unwrap();
    term.wait_for("Bob's card", STEP, |s| {
        s.contains(head) && s.contains("Bob Builder <bob@example.com>")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // `p` goes back to the start of the first hunk: the card follows.
    term.send_keys("p").unwrap();
    term.wait_for("the first commit's card", STEP, |s| {
        s.contains("<testkit@gitpoulpi.local>") && !s.contains("bob@example.com")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("n").unwrap();
    term.wait_for("Bob's card again", STEP, |s| s.contains("bob@example.com"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Esc closes the card first; the view stays.
    term.send_keys("Escape").unwrap();
    term.wait_for("the card gone", STEP, |s| {
        !s.contains("bob@example.com") && s.contains(BLAME_TITLE)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Open it again and click "Show commit in Log".
    term.send_keys("i").unwrap();
    let (c, r) = term
        .wait_for_text("Show commit in Log", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c + 2, r).unwrap();
    term.wait_for("the Log on Bob's commit", STEP, |s| {
        s.contains(LOG_HEADER) && !s.contains(BLAME_TITLE)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// The same card opens by itself when the pointer rests on an annotation.
#[test]
fn the_card_opens_when_the_pointer_rests_on_an_annotation() {
    let repo = repo();
    let (mut term, _config) = launch(repo.root(), "pty-blame-hover", "");
    open_blame(&mut term);
    let (c, r) = annotation_author(&term.screen()).unwrap();
    assert!(!term.text().contains("bob@example.com"));
    term.move_to(c + 2, r).unwrap();
    term.wait_for("the hover card", STEP, |s| {
        s.contains("Bob Builder <bob@example.com>") && s.contains("Show commit in Log")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Moving away closes it.
    term.move_to(2, 3).unwrap();
    term.wait_for("the card gone", STEP, |s| !s.contains("bob@example.com"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// `b` blames the file again as of the selected commit's parent.
#[test]
fn blame_at_the_parent_reblames_the_file_before_the_commit() {
    let repo = repo();
    let first = repo.git(&["rev-parse", "HEAD^"]);
    let first = first.trim();
    let (mut term, _config) = launch(repo.root(), "pty-blame-parent", "");
    open_blame(&mut term);
    term.send_keys("n").unwrap();
    term.send_keys("b").unwrap();
    term.wait_for("the blame at the parent", STEP, |s| {
        s.contains(&format!("{BLAME_TITLE} {}", &first[..8]))
            && s.contains("row three")
            && !s.contains("row three!")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Before Bob's commit every line is the first commit's.
    let central: String = term
        .text()
        .lines()
        .map(|l| l.chars().take(125).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!central.contains("Bob Builder"), "{}", term.text());
    term.stop().unwrap();
}

/// `d` opens the selected commit's diff of the file.
#[test]
fn show_the_commits_diff_of_the_file_opens_the_diff() {
    let repo = repo();
    let (mut term, _config) = launch(repo.root(), "pty-blame-diff", "");
    open_blame(&mut term);
    term.send_keys("n").unwrap();
    term.send_keys("d").unwrap();
    term.wait_for("the commit's diff of notes.txt", STEP, |s| {
        !s.contains(BLAME_TITLE)
            && !s.contains(LOG_HEADER)
            && s.contains("row three!")
            && s.contains("\u{2715}")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// Right-click on a hunk opens its menu; Esc closes it.
#[test]
fn right_click_on_a_hunk_opens_its_menu() {
    let repo = repo();
    let (mut term, _config) = launch(repo.root(), "pty-blame-menu", "");
    open_blame(&mut term);
    let (c, r) = annotation_author(&term.screen()).unwrap();
    term.right_click(c + 2, r).unwrap();
    term.wait_for("the menu", STEP, |s| {
        s.contains("Blame at parent") && s.contains("Copy commit SHA")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for("the menu gone", STEP, |s| {
        !s.contains("Blame at parent") && s.contains(BLAME_TITLE)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// `>` widens the annotation column; so does dragging its border, which is
/// saved to `ui.blame_width`.
#[test]
fn the_annotation_column_widens_with_its_key_and_by_dragging_its_border() {
    let repo = repo();
    let (mut term, config) = launch(repo.root(), "pty-blame-width", "[ui]\nblame_width = 30\n");
    open_blame(&mut term);
    let x0 = annotation_author(&term.screen()).unwrap().0;
    term.send_keys(">").unwrap();
    term.wait_for("a wider column", STEP, |s| {
        annotation_author(s).is_some_and(|(c, _)| c < x0)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (x1, row) = annotation_author(&term.screen()).unwrap();
    assert_eq!(x0 - x1, 4);
    // The border is three cells left of the annotation text.
    term.drag((x1 - 3, row), (x1 - 3 - 8, row)).unwrap();
    term.wait_for("the dragged column", STEP, |s| {
        annotation_author(s).is_some_and(|(c, _)| c == x1 - 8)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let saved = std::fs::read_to_string(config.path().join("config.toml")).unwrap_or_default();
    assert!(saved.contains("blame_width = 42"), "{saved}");
    term.stop().unwrap();
}

/// Blame bare-letter keys (n, p, g...) also match under Ctrl; the palette Ctrl+P must still reach the window.
#[test]
fn ctrl_chords_still_reach_the_window_from_the_blame_view() {
    let repo = repo();
    let (mut term, _config) = launch(repo.root(), "pty-blame-ctrl", "");
    open_blame(&mut term);
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}

/// A file of `lines` lines, all written by one commit: one tall hunk.
fn tall_hunk_repo(lines: usize) -> FixtureRepo {
    let body: String = (1..=lines).map(|i| format!("line {i:03}\n")).collect();
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("notes.txt", &body, "write the whole file")
        .unwrap()
        .build()
}

/// A hunk of three or more lines fills the room under its summary with the
/// short sha and the full date, muted, instead of leaving it empty.
/// Red when the third annotation line stays empty.
#[test]
fn a_tall_hunk_shows_its_sha_and_full_date_under_the_summary() {
    let repo = tall_hunk_repo(8);
    let (mut term, _config) = launch(repo.root(), "pty-blame-meta", "");
    let (c, r) = term
        .wait_for_text("notes.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    let (bc, br) = term
        .wait_for_text(" Blame ", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(bc + 2, br).unwrap();
    term.wait_for("the Blame view", STEP, |s| {
        s.contains(BLAME_TITLE) && s.contains("line 008")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    let sha = repo.git(&["rev-parse", "--short=7", "HEAD"]);
    let sha = sha.trim();
    let (_, summary_row) = term
        .screen()
        .find("write the whole file")
        .unwrap_or_else(|| panic!("{}", term.text()));
    // The central column only (the right pane shows the commit too).
    let meta: String = term
        .screen()
        .line(summary_row + 1)
        .chars()
        .take(150)
        .collect();
    assert!(meta.contains(sha), "the sha under the summary: {meta:?}");
    let year = 1970
        + std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            / 31_556_952;
    assert!(
        meta.contains(&year.to_string()),
        "the full date under the summary: {meta:?}"
    );
    term.stop().unwrap();
}

/// With `ui.minimap` on, the Blame view draws the minimap beside the code,
/// colored by the age heat, and a click on it scrolls the file there.
/// Red when the minimap is missing, plain-inked, or a click does nothing.
#[test]
fn the_blame_minimap_is_heat_colored_and_a_click_scrolls() {
    let repo = tall_hunk_repo(150);
    let (mut term, _config) = launch(repo.root(), "pty-blame-minimap", "[ui]\nminimap = true\n");
    let (c, r) = term
        .wait_for_text("notes.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    let (bc, br) = term
        .wait_for_text(" Blame ", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(bc + 2, br).unwrap();
    term.wait_for("the Blame view", STEP, |s| {
        s.contains(BLAME_TITLE) && s.contains("line 001")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        !term.text().contains("line 150"),
        "the file is longer than the pane"
    );

    // The braille cells of the minimap, and their color: the heat ramp's
    // middle (one commit, no age spread), not the plain ink.
    let screen = term.screen();
    let mut braille = Vec::new();
    for row in 0..screen.rows() {
        for col in 0..screen.cols() {
            let cell = screen.cell(col, row).unwrap();
            if cell
                .text
                .chars()
                .next()
                .is_some_and(|ch| ('\u{2801}'..='\u{28ff}').contains(&ch))
            {
                braille.push((col, row, cell.fg));
            }
        }
    }
    assert!(!braille.is_empty(), "a minimap is drawn:\n{}", term.text());
    let heat = [(60, 110, 220), (230, 200, 70), (240, 95, 60)]
        .map(|(r, g, b)| xtask::pty::Color::Rgb(r, g, b));
    assert!(
        braille.iter().any(|(_, _, fg)| heat.contains(fg)),
        "colored by the age heat: {:?}",
        braille.iter().map(|b| b.2).collect::<Vec<_>>()
    );

    // A click near the bottom of the minimap scrolls the code there.
    let col = braille.last().map(|(c, _, _)| *c).unwrap();
    let bottom = braille.iter().map(|(_, r, _)| *r).max().unwrap();
    term.click(col, bottom).unwrap();
    term.wait_for("the end of the file", STEP, |s| {
        s.contains("line 150") && !s.contains("line 001")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
