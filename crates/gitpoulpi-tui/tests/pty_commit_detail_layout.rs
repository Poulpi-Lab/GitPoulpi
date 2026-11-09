//! The right pane's commit details layout, through a pty: the commit info
//! (author, parent, date, counts) sits above the message, and the file
//! options (sort, Path/Tree, View all files, filter) are the first rows of
//! the changed-files pane, right above the list. Nothing is located by a
//! hard-coded coordinate.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

fn fixture() -> (FixtureRepo, String) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("base commit")
        .unwrap()
        .build();
    std::fs::write(repo.root().join("target.txt"), "target").unwrap();
    repo.git(&["add", "target.txt"]);
    repo.git(&["commit", "-m", "target summary", "-m", "target body text"]);
    std::fs::write(repo.root().join("top.txt"), "top").unwrap();
    repo.git(&["add", "top.txt"]);
    repo.git(&["commit", "-m", "top commit"]);
    let author = repo.git(&["log", "-1", "--format=%an"]).trim().to_string();
    (repo, author)
}

fn start(repo: &FixtureRepo, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .ready("top commit")
        .mouse_capture()
        .start()
}

fn select_target(term: &mut Terminal) {
    let (col, row) = term.wait_for_text("target summary", STEP).unwrap();
    term.click(col + 1, row).unwrap();
    term.wait_for("the target's body in the detail pane", STEP, |s| {
        s.contains("target body text") && s.contains("target.txt")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// The rightmost hit of `needle` (the detail pane is right of the Log).
fn rightmost(term: &Terminal, needle: &str) -> Option<(u16, u16)> {
    let screen = term.screen();
    let mut best: Option<(u16, u16)> = None;
    for row in 0..screen.rows() {
        let line = screen.line(row);
        for (byte, _) in line.match_indices(needle) {
            let col = line[..byte].chars().count() as u16;
            if best.is_none_or(|(c, _)| col > c) {
                best = Some((col, row));
            }
        }
    }
    best
}

fn at(term: &Terminal, needle: &str) -> (u16, u16) {
    rightmost(term, needle).unwrap_or_else(|| panic!("{needle:?} on screen\n{}", term.text()))
}

/// The summary in the detail pane (the Log's copy is to its left).
fn summary_at(term: &Terminal) -> (u16, u16) {
    let screen = term.screen();
    screen
        .find_all("target summary")
        .into_iter()
        .rfind(|&(c, r)| c > 100 && !screen.line(r).contains("target.txt"))
        .expect("the summary in the detail pane")
}

fn author_at(term: &Terminal, author: &str) -> (u16, u16) {
    let screen = term.screen();
    screen
        .find_all(author)
        .into_iter()
        .find(|&(_, r)| screen.line(r).contains("parent:"))
        .expect("the author line of the detail pane")
}

/// Author, parent and date rows are above the summary; the options are in
/// the files pane, under its title and right above the first file row.
#[test]
fn the_info_block_is_above_the_message_and_the_options_open_the_files_pane() {
    let (repo, author) = fixture();
    let (mut term, _cfg) = start(&repo, "pty-detail-layout");
    select_target(&mut term);

    let summary = summary_at(&term).1;
    let author_row = author_at(&term, &author).1;
    let authored = at(&term, "authored").1;
    assert!(
        author_row < summary && authored < summary,
        "author {author_row} / authored {authored} must be above the summary {summary}\n{}",
        term.text()
    );
    let title = at(&term, "Changed files").1;
    let options = at(&term, "View all files").1;
    let filter = at(&term, "Filter files:").1;
    let first_file = at(&term, "target.txt").1;
    assert!(
        summary < title,
        "the message is above the files pane\n{}",
        term.text()
    );
    assert_eq!(options, title + 1, "options are the pane's first row");
    assert_eq!(filter, options + 1);
    assert_eq!(first_file, filter + 1, "the list starts under the filter");
    let _ = term.stop();
}

/// The options still work from their new place: View all files, Tree and
/// the filter.
#[test]
fn the_options_in_the_files_pane_still_work() {
    let (repo, _author) = fixture();
    let (mut term, _cfg) = start(&repo, "pty-detail-options");
    select_target(&mut term);

    // A click on the first file row opens its diff (the rows moved down by
    // the options: a wrong offset lands on the filter row or the next file).
    let (c, r) = at(&term, "target.txt");
    term.click(c + 1, r).unwrap();
    term.wait_for("the diff of target.txt in the centre", STEP, |s| {
        !s.contains("Graph  │Message")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for("the Log back", STEP, |s| s.contains("Graph  │Message"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // View all files: the pane lists the whole tree.
    let (c, r) = at(&term, "View all files");
    term.click(c + 1, r).unwrap();
    term.wait_for("the pane listing every file", STEP, |s| {
        s.contains("All files") && s.contains("base commit.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = at(&term, "View all files");
    term.click(c + 1, r).unwrap();
    term.wait_for("only the changed files again", STEP, |s| {
        s.contains("Changed files") && !s.contains("base commit.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Tree: the click reaches the toggle and the list keeps its file.
    let (c, r) = at(&term, "Tree");
    term.click(c + 1, r).unwrap();
    term.wait_for("the tree still lists the file", STEP, |s| {
        s.contains("target.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Filter: click the field, type a name that matches nothing.
    let (c, r) = at(&term, "Filter files:");
    term.click(c + 1, r).unwrap();
    term.send_text("zzzz").unwrap();
    term.wait_for("the filter emptying the list", STEP, |s| {
        s.contains("zzzz") && !s.contains("target.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let _ = term.stop();
}

/// The double click on the summary and on the author (the info block moved)
/// still opens the amend popup on that field.
#[test]
fn double_clicks_land_in_the_right_amend_field_after_the_move() {
    let (repo, author) = fixture();
    let (mut term, _cfg) = start(&repo, "pty-detail-dbl");
    select_target(&mut term);

    let (c, r) = summary_at(&term);
    term.double_click(c + 1, r).unwrap();
    term.wait_for("the summary popup", STEP, |s| {
        s.contains("Amend commit") && s.contains("Author email")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The caret is in the summary: a typed character lands there.
    term.send_text("!").unwrap();
    term.wait_for("the character in the summary", STEP, |s| {
        s.contains("target summary!")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    // The edited dialog asks before closing: a second Escape discards.
    term.wait_for_text("Discard your changes", STEP).unwrap();
    term.send_keys("Escape").unwrap();
    term.wait_for_text_to_go("Amend commit", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    let (c, r) = author_at(&term, &author);
    term.double_click(c + 1, r).unwrap();
    term.wait_for("the author popup", STEP, |s| {
        s.contains("Amend commit") && s.contains("Author email")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The caret is in the author name.
    term.send_text(" Jr").unwrap();
    term.wait_for("the suffix in the author name", STEP, |s| {
        s.contains(&format!("{author} Jr"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    let _ = term.stop();
}
