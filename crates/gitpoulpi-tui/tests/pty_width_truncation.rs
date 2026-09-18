//! A CJK commit subject in a narrow Log column is cut by display width
//! (2 cells per ideograph), not by character count: the row never
//! overflows into the next column and the columns after it stay aligned
//! with an ASCII row's.

mod common;

use gitpoulpi_testkit::FixtureRepoBuilder;

#[test]
fn a_cjk_subject_is_truncated_by_width_and_keeps_the_columns_aligned() {
    let cjk = "日本語のコミットメッセージはとても長い件名ですので切り詰められます";
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("ascii subject row")
        .unwrap()
        .commit(cjk)
        .unwrap()
        .build();
    let sha_top = repo
        .git(&["rev-parse", "--short", "HEAD"])
        .trim()
        .to_string();
    let sha_below = repo
        .git(&["rev-parse", "--short", "HEAD~1"])
        .trim()
        .to_string();

    // The screen model renders a wide glyph's continuation cell as a
    // blank, so the CJK text is located by its first ideograph.
    let (top, below) = (sha_top.clone(), sha_below.clone());
    let (term, _config) = common::launch(repo.root())
        .session("pty-width-truncation")
        .size(90, 30)
        .ready_when("both subjects", move |s| {
            s.contains("ascii subject") && s.contains(&top) && s.contains(&below)
        })
        .start();

    let screen = term.screen();
    let cjk_row = screen.find("日").unwrap().1;
    let ascii_row = screen.find("ascii subject").unwrap().1;
    let cjk_line = screen.line(cjk_row);
    // The author is "GitPoulpi Test", itself cut with an ellipsis, so
    // only what precedes it is the Message cell.
    let message_cell = cjk_line.split("GitP").next().unwrap();
    assert!(
        message_cell.contains('\u{2026}'),
        "no ellipsis:\n{}",
        screen.text()
    );
    assert!(
        !cjk_line.contains('切'),
        "the tail should be cut:\n{}",
        screen.text()
    );

    // Every column after Message starts at the same cell on both rows: a
    // cut that counted chars would have pushed the SHA (and the pane's
    // right border) out on the CJK row.
    let sha_col = |sha: &str, row: u16| {
        screen
            .find_all(sha)
            .into_iter()
            .find(|&(_, r)| r == row)
            .map(|(c, _)| c)
    };
    let (a, c) = (sha_col(&sha_below, ascii_row), sha_col(&sha_top, cjk_row));
    assert!(
        a.is_some() && a == c,
        "SHA column {a:?} vs {c:?}:\n{}",
        screen.text()
    );
    // The Log's columns only (from its header to the right pane's title):
    // the side panes draw their own borders on whichever rows they happen
    // to share with the Log rows.
    let log_start = screen.find("Graph").map_or(0, |(c, _)| c as usize);
    let log_end = screen.find("Commit details").map_or(0, |(c, _)| c as usize);
    let border = |row: u16| {
        screen
            .line(row)
            .chars()
            .take(log_end)
            .skip(log_start)
            .filter(|&ch| ch == '\u{2502}')
            .count()
    };
    assert_eq!(border(cjk_row), border(ascii_row), "{}", screen.text());
}
