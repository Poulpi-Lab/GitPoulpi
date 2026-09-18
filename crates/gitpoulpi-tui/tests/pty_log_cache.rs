//! The Log's layout and the search matches are cached between frames
//! (`graph::LogLayoutCache`); a real change must still show up. Driven
//! through the real binary: a commit made behind the TUI's back, a
//! resize, and a search that has to follow the new log.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(60);

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("first zulu commit")
        .unwrap()
        .commit("second plain commit")
        .unwrap()
        .build()
}

fn launch(repo: &FixtureRepo, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .size(160, 40)
        .ready_when("the fixture loaded", |s| s.contains("second pl"))
        .start()
}

#[test]
fn a_new_commit_and_a_resize_reach_the_cached_log_and_its_search() {
    let repo = fixture();
    let (mut term, _config) = launch(&repo, "pty-log-cache");

    // Open the find bar and search: one hit among the two commits.
    term.send_keys("f").unwrap();
    term.send_text("zulu").unwrap();
    term.wait_for_text("1 of 1", STEP)
        .unwrap_or_else(|e| panic!("search did not find the commit: {e}"));

    // A commit made outside the TUI: the cached rows and the cached hits
    // must both be rebuilt for it.
    repo.git(&["commit", "--allow-empty", "-m", "third zulu arrives"]);
    term.wait_for_text("third zu", STEP)
        .unwrap_or_else(|e| panic!("the new commit never reached the Log: {e}"));
    term.wait_for_text("of 2", STEP)
        .unwrap_or_else(|e| panic!("the search hits are stale after the refresh: {e}"));

    // A resize re-lays the table out; the content must survive it.
    term.resize(120, 30).unwrap();
    term.wait_for("rows still drawn after a resize", STEP, |s| {
        s.contains("third zu") && s.contains("second pl")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.resize(200, 50).unwrap();
    // The rows alone are on the 120-column frame too, so the wait is for the
    // 200-column layout itself (the right-hand pane starts past column 130
    // only then); keys sent before it are typed into a relayout in flight.
    term.wait_for("rows still drawn after growing", STEP, |s| {
        s.contains("third zu")
            && s.contains("first zu")
            && s.find("Commit details").is_some_and(|(col, _)| col > 130)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Editing the query changes the hits at once.
    term.send_keys("BSpace").unwrap();
    term.send_text("x").unwrap();
    term.wait_for_text("0 of 0", STEP)
        .unwrap_or_else(|e| panic!("a changed query kept the old hits: {e}"));
    term.stop().unwrap();
}
