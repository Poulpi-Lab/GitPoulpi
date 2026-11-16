//! A file moved on disk but not staged is ONE row in the Unstaged list
//! ("new <- old"), its diff is the content against the old path under an
//! "old -> new" header (not an all-green new file), and staging the row
//! stages both sides, so git sees one rename.
//!
//! Same rules as `pty_ui.rs`: nothing located by a hard-coded coordinate,
//! and every wait describes the whole state it waits for.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

fn spawn(repo: &Path, config: &Path, session: &str) -> Terminal {
    common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(160, 50)
        .no_exit_pause()
        .spawn()
}

fn body() -> String {
    (1..=12).map(|n| format!("line {n}\n")).collect()
}

/// `old_name.txt` committed, then moved to `moved_name.txt` on disk.
fn moved_fixture(edit: bool) -> gitpoulpi_testkit::FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("old_name.txt", &body(), "add old_name")
        .unwrap()
        .build();
    std::fs::remove_file(repo.root().join("old_name.txt")).unwrap();
    let new = if edit {
        body().replace("line 6\n", "LINE-SIX-EDITED\n")
    } else {
        body()
    };
    std::fs::write(repo.root().join("moved_name.txt"), new).unwrap();
    repo
}

fn wait_row(term: &mut Terminal) {
    term.wait_for("the move as one row", STARTUP, |s| {
        s.contains("moved_name.txt <- old_name.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.wait_for_mouse_capture(STEP)
        .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn a_move_with_edits_is_one_row_and_diffs_against_the_old_file() {
    let repo = moved_fixture(true);
    let config = tempfile::tempdir().unwrap();
    let mut term = spawn(repo.root(), config.path(), "pty-unstaged-rename-edit");
    wait_row(&mut term);

    // One row: the deleted file is not listed on its own, so its name is
    // on the screen once (in the label).
    let screen = term.text();
    assert_eq!(
        screen.matches("old_name.txt").count(),
        1,
        "the deleted file listed beside the new one:\n{screen}"
    );

    let (c, r) = term
        .wait_for_text("moved_name.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    term.wait_for("the diff against the old path", STEP, |s| {
        s.contains("\u{2715}")
            && s.contains("old_name.txt -> moved_name.txt")
            && s.contains("% similar")
            && s.contains("LINE-SIX-EDITED")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn a_pure_move_says_there_is_no_content_change() {
    let repo = moved_fixture(false);
    let config = tempfile::tempdir().unwrap();
    let mut term = spawn(repo.root(), config.path(), "pty-unstaged-rename-pure");
    wait_row(&mut term);

    let (c, r) = term
        .wait_for_text("moved_name.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    term.wait_for("the rename header and no content", STEP, |s| {
        s.contains("old_name.txt -> moved_name.txt (100% similar)")
            && s.contains("No content change")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn staging_the_row_stages_both_sides_as_one_rename() {
    let repo = moved_fixture(true);
    let config = tempfile::tempdir().unwrap();
    let mut term = spawn(repo.root(), config.path(), "pty-unstaged-rename-stage");
    wait_row(&mut term);

    let (c, r) = term
        .wait_for_text("moved_name.txt", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c, r).unwrap();
    term.wait_for("the row open", STEP, |s| {
        s.contains("old_name.txt -> moved_name.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("C-g").unwrap();
    term.send_keys("s").unwrap();

    let deadline = std::time::Instant::now() + STEP;
    loop {
        let staged = repo.git(&["diff", "--cached", "--name-status", "-M"]);
        if staged.starts_with('R') && staged.contains("old_name.txt") {
            assert!(staged.contains("moved_name.txt"), "{staged}");
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "git never saw one staged rename: {staged:?}\n{}",
            term.text()
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}
