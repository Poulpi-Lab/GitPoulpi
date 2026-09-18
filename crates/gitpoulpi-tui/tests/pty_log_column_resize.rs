//! A terminal resize (maximize / restore) must not move the Ref and Graph
//! column widths: only `Message` absorbs the difference.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(60);

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("first commit")
        .unwrap()
        .commit("second commit")
        .unwrap()
        .build()
}

/// `(ref width, graph width)` read off the header row: the distance between
/// consecutive header titles.
fn widths(term: &Terminal) -> Option<(u16, u16)> {
    let screen = term.screen();
    let (g, row) = screen.find("Graph")?;
    // "Refs" is also the sidebar's title: take the one on the Graph row.
    let (r, _) = screen
        .find_all("Ref")
        .into_iter()
        .find(|&(c, y)| y == row && c < g)?;
    let (m, _) = screen
        .find_all("Message")
        .into_iter()
        .find(|&(c, y)| y == row && c > g)?;
    Some((g - r, m - g))
}

#[test]
fn a_resize_keeps_the_ref_and_graph_widths() {
    let repo = fixture();
    let (mut term, _config) = common::launch(repo.root())
        .session("pty-log-column-resize")
        .size(200, 40)
        .ready_when("the fixture loaded", |s| s.contains("second co"))
        .start();
    let before = widths(&term).unwrap_or_else(|| panic!("no header\n{}", term.text()));

    for (cols, rows) in [(240, 50), (220, 30), (200, 40)] {
        term.resize(cols, rows).unwrap();
        term.wait_for("the relaid-out header", STEP, |s| {
            s.cols() == cols && s.contains("Graph")
        })
        .unwrap_or_else(|e| panic!("{e}"));
        // Let the relayout settle before measuring.
        term.wait_for("stable widths", STEP, |_| widths(&term) == Some(before))
            .unwrap_or_else(|e| panic!("widths drifted at {cols}x{rows}: {e}\n{}", term.text()));
    }
    term.stop().unwrap();
}
