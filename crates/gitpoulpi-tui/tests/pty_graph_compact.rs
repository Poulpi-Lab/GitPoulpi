//! The Log's Graph column at its least width (one commit dot) draws every
//! commit on a single lane, one under the other, with no branch or merge
//! lines; one cell wider the tree is back. A function of the width alone.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Screen;

const STEP: Duration = Duration::from_secs(30);
/// The strokes a lane or a branch / merge junction is made of.
const STROKES: &str =
    "\u{2502}\u{256d}\u{256e}\u{256f}\u{2570}\u{2500}\u{253c}\u{251c}\u{2524}\u{252c}\u{2534}";
/// The commit markers of the circle set.
const DOTS: &str = "\u{25cb}\u{25cf}\u{25ce}";

/// main and a side branch that is merged back with a merge commit: lanes,
/// a fork and a join.
fn fixture() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "base commit")
        .unwrap()
        .branch("topic")
        .unwrap()
        .commit_file("b.txt", "b\n", "topic work")
        .unwrap()
        .checkout("main")
        .unwrap()
        .commit_file("c.txt", "c\n", "main work")
        .unwrap()
        .build();
    repo.git(&["merge", "--no-ff", "topic", "-m", "merge topic"]);
    repo
}

fn header(s: &Screen, title: &str) -> Option<(u16, u16)> {
    let (_, row) = s.find_all("Message").into_iter().next()?;
    s.find_all(title).into_iter().find(|(_, r)| *r == row)
}

/// The Graph column's `width` cells of each commit row under the header.
fn graph_cells(s: &Screen, gx: u16, width: u16, header_row: u16) -> Vec<String> {
    // The row's message starts right after the Graph column.
    let message_x = header(s, "Message").map_or(gx + width, |(x, _)| x);
    ((header_row + 1)..s.rows())
        .filter(|&y| {
            let message: String = (message_x..message_x + 12)
                .map(|x| s.cell(x, y).map_or(" ", |c| c.display()).to_string())
                .collect();
            ["base commit", "topic work", "main work", "merge topic"]
                .iter()
                .any(|m| message.starts_with(m))
        })
        .map(|y| {
            (gx..gx + width)
                .map(|x| s.cell(x, y).map_or(" ", |c| c.display()).to_string())
                .collect::<String>()
        })
        .collect()
}

/// The screen once it has settled (a torn frame has half a header).
fn snap(term: &xtask::pty::Terminal) -> Screen {
    term.wait_for_stable(
        std::time::Duration::from_millis(300),
        std::time::Duration::from_secs(30),
    )
    .unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn the_graph_at_its_least_width_is_one_dot_per_row_and_one_cell_more_brings_the_tree_back() {
    let repo = fixture();
    let (mut term, _config) = common::launch(repo.root())
        .session("pty-graph-compact")
        .size(200, 40)
        .ready_when("the Log header", |s| {
            s.contains("merge topic") && header(s, "Graph").is_some()
        })
        .mouse_capture()
        .start();
    let (gx, row) = header(&snap(&term), "Graph").unwrap();
    let (mx, _) = header(&snap(&term), "Message").unwrap();
    assert!(mx - gx > 3, "the Graph column starts wide");
    let wide = graph_cells(&snap(&term), gx, mx - gx, row);
    assert_eq!(wide.len(), 4, "{}", term.text());
    assert!(
        wide.iter().any(|r| r.chars().any(|c| STROKES.contains(c))),
        "the fixture has lanes: {wide:?}"
    );

    // To its least: two cells, the width of one dot.
    // `line` mode: the pointer holds the gap, one cell before the edge.
    term.drag((mx - 1, row), (gx + 1, row)).unwrap();
    term.wait_for("the narrowest Graph", STEP, |s| {
        header(s, "Message").is_some_and(|(x, _)| x == gx + 2)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let compact = graph_cells(&snap(&term), gx, 2, row);
    assert_eq!(compact.len(), 4);
    for cells in &compact {
        assert!(
            cells.chars().next().is_some_and(|c| DOTS.contains(c)),
            "one dot in the first cell: {compact:?}"
        );
        assert!(
            cells.chars().all(|c| !STROKES.contains(c)),
            "no branch or merge line at the least width: {compact:?}"
        );
    }

    // One cell wider: the tree is back.
    term.drag((gx + 1, row), (gx + 2, row)).unwrap();
    term.wait_for("one cell wider", STEP, |s| {
        header(s, "Message").is_some_and(|(x, _)| x == gx + 3)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let three = graph_cells(&snap(&term), gx, 3, row);
    assert!(
        three.iter().any(|r| r.chars().any(|c| STROKES.contains(c))),
        "lanes are back one cell wider: {three:?}\n{}",
        term.text()
    );
    term.stop().unwrap();
}
