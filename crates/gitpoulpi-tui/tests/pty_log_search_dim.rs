//! Log search: every row that is not a hit is dimmed in the first frame
//! after typing, with no mouse movement ("c'est pas live" on Windows, where
//! `DIM` next to `BOLD` is folded into one intensity). The emulator has no
//! DIM attribute, so the test asserts what the fix guarantees on any
//! terminal: a non-matching Message cell is neither bold nor full color.
//! Temp config dir and fixture repo only.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

#[test]
fn non_matching_rows_are_dimmed_in_the_first_frame_after_typing() {
    let config = tempfile::tempdir().unwrap();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "1\n", "alpha one")
        .unwrap()
        .commit_file("b.txt", "1\n", "beta two")
        .unwrap()
        .commit_file("c.txt", "1\n", "alpha three")
        .unwrap()
        .commit_file("d.txt", "1\n", "gamma four")
        .unwrap()
        .build();
    let mut term = common::launch(repo.root())
        .session("search-dim")
        .config_dir(config.path())
        .size(200, 50)
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| s.contains("gamma four"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("f").unwrap();
    term.send_text("alpha").unwrap();
    // The frame where the hit count is shown; no mouse event was sent.
    term.wait_for("the search is live", STEP, |s| {
        let find = |t: &str| s.find(t).and_then(|(c, r)| s.cell(c, r).cloned());
        match (find("alpha one"), find("beta two"), find("gamma four")) {
            (Some(hit), Some(miss), Some(miss2)) => {
                hit.bold && !miss.bold && !miss2.bold && hit.fg != miss.fg && miss.fg == miss2.fg
            }
            _ => false,
        }
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}
