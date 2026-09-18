//! Comparing the Log's multi-selection through the real binary: Ctrl+click /
//! Space / Shift+click two or more rows (commits, the WIP row, a stash) and
//! the right pane lists the files that differ between them; a click on a
//! file opens that comparison's diff in the center; Esc / a plain click
//! returns to the single commit.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);

/// Left of this column is the Log, right of it the commit-detail pane
/// (200 columns wide terminal).
const LOG_COLS: u16 = 110;

fn fixture(dirty: bool) -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "1\n2\n3\n", "c1 base")
        .unwrap()
        .commit_file("a.txt", "1\nTWO\n3\n", "c2 edit a")
        .unwrap()
        .commit_file("b.txt", "bee\n", "c3 add b")
        .unwrap()
        .commit_file("c.txt", "see\n", "c4 add c")
        .unwrap()
        .build();
    if dirty {
        std::fs::write(repo.root().join("a.txt"), "1\nTWO\n3\nWIPLINE\n").unwrap();
    }
    repo
}

fn launch(repo: &FixtureRepo, session: &str) -> Terminal {
    let (term, config) = common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .no_exit_pause()
        .ready_when("the log", |s| s.contains("c4 add c"))
        .mouse_capture()
        .start();
    // The config dir lives as long as the process; keep it for the test.
    std::mem::forget(config);
    term
}

/// `(col, row)` of `summary` in the Log (not the detail pane, which repeats
/// the summary of a comparison's ends).
fn log_cell(s: &Screen, summary: &str) -> Option<(u16, u16)> {
    s.find_all(summary).into_iter().find(|(c, _)| *c < LOG_COLS)
}

fn wait_log_cell(term: &Terminal, summary: &str) -> (u16, u16) {
    let mut found = None;
    term.wait_for(summary, STEP, |s| {
        found = log_cell(s, summary);
        found.is_some()
    })
    .unwrap_or_else(|e| panic!("{e}"));
    found.unwrap()
}

/// The right pane's text only (the part right of the Log).
fn detail_text(s: &Screen) -> String {
    (0..s.rows())
        .map(|r| {
            s.line(r)
                .chars()
                .skip(LOG_COLS as usize)
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn wait_detail(term: &Terminal, what: &str, ok: impl Fn(&str) -> bool) -> Screen {
    term.wait_for(what, STEP, |s| ok(&detail_text(s)))
        .unwrap_or_else(|e| panic!("{e}"))
}

/// The detail line holding `summary` (a card's second line, ending with its
/// From / To tag), trimmed.
fn card_line(d: &str, summary: &str) -> String {
    d.lines()
        .find(|l| l.contains(summary))
        .unwrap_or_default()
        .trim_end_matches([' ', '\u{2502}'])
        .to_string()
}

fn detail_cell(s: &Screen, needle: &str) -> (u16, u16) {
    s.find_all(needle)
        .into_iter()
        .find(|(c, _)| *c >= LOG_COLS)
        .unwrap_or_else(|| panic!("{needle} not in the detail pane:\n{}", s.text()))
}

/// Ctrl+click on two commits: the right pane is a comparison listing exactly
/// the files that differ between them, in older-to-newer order, whatever the
/// click order. A click on a file opens the comparison's diff; Esc closes
/// it, Esc again returns to the single commit.
#[test]
fn ctrl_click_two_commits_lists_the_files_between_them() {
    let repo = fixture(false);
    let mut term = launch(&repo, "pty-compare-two");
    let (col, row4) = wait_log_cell(&term, "c4 add c");
    let (_, row2) = wait_log_cell(&term, "c2 edit a");
    term.click(col, row4).unwrap();
    term.ctrl_click(col, row2).unwrap();

    let s = wait_detail(&term, "the comparison lists b.txt and c.txt", |d| {
        d.contains("Comparing") && d.contains("b.txt") && d.contains("c.txt")
    });
    let d = detail_text(&s);
    assert!(!d.contains("a.txt"), "a.txt is the same at c2 and c4:\n{d}");
    assert!(card_line(&d, "c2 edit a").ends_with("From"), "{d}");
    assert!(card_line(&d, "c4 add c").ends_with("To"), "{d}");

    // The file opens its comparison diff in the center.
    let (fc, fr) = detail_cell(&s, "b.txt");
    term.click(fc, fr).unwrap();
    term.wait_for("the diff of b.txt between the two commits", STEP, |s| {
        s.contains("+bee")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Esc closes the diff, a second Esc clears the selection.
    term.send_keys("Escape").unwrap();
    term.wait_for("the Log is back", STEP, |s| !s.contains("+bee"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").unwrap();
    wait_detail(&term, "the single commit's detail", |d| {
        !d.contains("Comparing") && d.contains("Changed files")
    });
}

/// The WIP row and a commit: the commit against what is on disk.
#[test]
fn wip_and_a_commit_compare_the_commit_with_the_working_tree() {
    let repo = fixture(true);
    let mut term = launch(&repo, "pty-compare-wip");
    let (col, row2) = wait_log_cell(&term, "c2 edit a");
    let (_, row4) = wait_log_cell(&term, "c4 add c");
    let wip_row = row4 - 1;
    term.click(col, row2).unwrap();
    term.ctrl_click(col, wip_row).unwrap();

    let s = wait_detail(&term, "the comparison with the working tree", |d| {
        d.contains("Comparing") && d.contains("a.txt") && d.contains("b.txt")
    });
    let d = detail_text(&s);
    assert!(d.contains("Working tree"), "the WIP card:\n{d}");
    assert!(
        card_line(&d, "Uncommitted changes").ends_with("To"),
        "WIP is the newer end:\n{d}"
    );
    let (fc, fr) = detail_cell(&s, "a.txt");
    term.click(fc, fr).unwrap();
    term.wait_for("the unsaved line in the diff", STEP, |s| {
        s.contains("+WIPLINE")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// Shift+click a range of three commits: their changes combined, from just
/// before the oldest, so the oldest commit's own edit is in the list.
#[test]
fn shift_click_three_commits_combines_the_range() {
    let repo = fixture(false);
    let mut term = launch(&repo, "pty-compare-range");
    let (col, row4) = wait_log_cell(&term, "c4 add c");
    let (_, row2) = wait_log_cell(&term, "c2 edit a");
    term.click(col, row4).unwrap();
    term.shift_click(col, row2).unwrap();

    let s = wait_detail(&term, "the combined range", |d| {
        d.contains("Comparing") && d.contains("a.txt") && d.contains("b.txt") && d.contains("c.txt")
    });
    let d = detail_text(&s);
    assert!(
        !d.contains("base"),
        "the range starts before c2, not at c1:\n{d}"
    );
    let (fc, fr) = detail_cell(&s, "a.txt");
    term.click(fc, fr).unwrap();
    term.wait_for("c2's edit in the combined diff", STEP, |s| {
        s.contains("+TWO")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// Alt+click is Shift+click for terminals that keep Shift+click for their
/// own text selection (kitty, Windows Terminal), and Alt+Down is Shift+Down.
#[test]
fn alt_click_and_alt_arrows_take_the_range_like_shift() {
    let repo = fixture(false);
    let mut term = launch(&repo, "pty-compare-alt");
    let (col, row4) = wait_log_cell(&term, "c4 add c");
    let (_, row2) = wait_log_cell(&term, "c2 edit a");
    term.click(col, row4).unwrap();
    term.alt_click(col, row2).unwrap();
    wait_detail(&term, "the range by Alt+click", |d| {
        d.contains("Comparing") && d.contains("a.txt") && d.contains("b.txt") && d.contains("c.txt")
    });

    term.click(col, row4).unwrap();
    wait_detail(&term, "a plain click starts over", |d| {
        !d.contains("Comparing")
    });
    term.send_keys("M-Down").unwrap();
    // The From tag is on the card's line in the same frame as the range.
    let s = wait_detail(&term, "the range by Alt+Down", |d| {
        d.contains("Comparing") && d.contains("c.txt") && card_line(d, "c3 add b").ends_with("From")
    });
    let d = detail_text(&s);
    assert!(card_line(&d, "c3 add b").ends_with("From"), "{d}");
}

/// The keyboard twins: Space + Down + Space selects, `x` swaps the ends,
/// Esc clears.
#[test]
fn keyboard_selects_swaps_and_clears() {
    let repo = fixture(false);
    let mut term = launch(&repo, "pty-compare-keys");
    let (col, row4) = wait_log_cell(&term, "c4 add c");
    term.click(col, row4).unwrap();
    term.send_text(" ").unwrap();
    term.send_keys("Down Down").unwrap();
    term.send_text(" ").unwrap();
    let s = wait_detail(&term, "the comparison of c4 and c2", |d| {
        d.contains("Comparing") && card_line(d, "c2 edit a").ends_with("From")
    });
    let from_line = |s: &Screen| {
        detail_text(s)
            .lines()
            .find(|l| l.trim_end_matches([' ', '\u{2502}']).ends_with("From"))
            .unwrap_or_default()
            .to_string()
    };
    assert!(from_line(&s).contains("c2 edit a"), "{}", from_line(&s));

    term.send_keys("x").unwrap();
    let s = wait_detail(&term, "the ends swapped", |d| {
        card_line(d, "c4 add c").ends_with("From")
    });
    assert!(from_line(&s).contains("c4 add c"));

    // The header's Swap button is the mouse twin.
    let (bc, br) = detail_cell(&s, "Swap");
    term.click(bc, br).unwrap();
    wait_detail(&term, "swapped back", |d| {
        card_line(d, "c2 edit a").ends_with("From")
    });

    term.send_keys("Escape").unwrap();
    wait_detail(&term, "back to the single commit", |d| {
        !d.contains("Comparing")
    });
}

/// A stash and a commit: the stash's saved change against that commit.
#[test]
fn a_stash_and_a_commit_compare() {
    let repo = fixture(true);
    repo.git(&["stash", "push", "-m", "stashmsg"]);
    let mut term = launch(&repo, "pty-compare-stash");
    let (col, stash_row) = wait_log_cell(&term, "stashmsg");
    let (_, row2) = wait_log_cell(&term, "c2 edit a");
    term.click(col, row2).unwrap();
    term.ctrl_click(col, stash_row).unwrap();
    let s = wait_detail(&term, "the stash compared with c2", |d| {
        d.contains("stash@{0}") && d.contains("a.txt") && d.contains("b.txt")
    });
    let d = detail_text(&s);
    assert!(d.contains("a.txt") && d.contains("b.txt"), "{d}");
}

/// Three Ctrl+clicked commits are three cards (author, summary, the From and
/// To ends); a click on the middle card moves the Log cursor to its row
/// without breaking the selection (Space there then deselects exactly that
/// commit), and `n` walks the cursor across the compared rows.
#[test]
fn the_panel_lists_a_card_per_selected_commit_and_a_click_goes_to_its_row() {
    let repo = fixture(false);
    let mut term = launch(&repo, "pty-compare-cards");
    let (col, row4) = wait_log_cell(&term, "c4 add c");
    let (_, row3) = wait_log_cell(&term, "c3 add b");
    let (_, row2) = wait_log_cell(&term, "c2 edit a");
    term.click(col, row4).unwrap();
    term.ctrl_click(col, row3).unwrap();
    term.ctrl_click(col, row2).unwrap();

    let s = wait_detail(&term, "three cards", |d| {
        d.contains("c4 add c") && d.contains("c3 add b") && d.contains("c2 edit a")
    });
    let d = detail_text(&s);
    assert!(
        d.matches("Testkit").count() >= 3,
        "the author of each card:\n{d}"
    );
    assert!(card_line(&d, "c4 add c").ends_with("To"), "{d}");
    assert!(card_line(&d, "c2 edit a").ends_with("From"), "{d}");
    assert!(
        !card_line(&d, "c3 add b").ends_with("To") && !card_line(&d, "c3 add b").ends_with("From"),
        "the middle card is no end:\n{d}"
    );

    // The cursor is on c2 (the last Ctrl+click). Click the c3 card.
    let (cc, cr) = detail_cell(&s, "c3 add b");
    term.click(cc, cr).unwrap();
    // Space toggles the row under the Log cursor: it is c3 now.
    term.send_text(" ").unwrap();
    wait_detail(&term, "c3 left the selection", |d| {
        d.contains("c4 add c") && d.contains("c2 edit a") && !d.contains("c3 add b")
    });

    // `n`: the next compared row below the cursor, wrapping to the first.
    term.send_text(" ").unwrap();
    wait_detail(&term, "c3 is back", |d| d.contains("c3 add b"));
    term.send_text("n").unwrap();
    term.send_text(" ").unwrap();
    wait_detail(&term, "the card after c3 went", |d| {
        d.contains("c4 add c") && d.contains("c3 add b") && !d.contains("c2 edit a")
    });
}
