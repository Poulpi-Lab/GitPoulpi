//! "Mark then drop", the keyboard twin of dragging a ref onto another ref
//! (`ref_drop`), end to end through a pty (`xtask::pty`): the real binary,
//! the real daemon, real `git`.
//!
//! `m` marks the selected ref, `Shift+M` on another ref opens the very menu
//! a drag opens, `Esc` (or `m` on the marked ref) clears the mark.

mod common;

use common::git_trimmed as git;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(20);
const COLS: u16 = 200;
const ROWS: u16 = 50;
/// The Refs pane is 20% of 200 columns.
const SIDEBAR_RIGHT: u16 = 40;
/// The sidebar mark is `MARK_GLYPH` in `gitpoulpi_tui::ref_drop`: U+25C6.
const FF: &str = "Fast-forward other to feat";
const MERGE: &str = "Merge feat into other";
const REBASE: &str = "Rebase feat onto other";

/// `main` (checked out) and `feat` on the same commit, `other` one commit
/// behind: neither side of the pair `feat` -> `other` is the current branch.
fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial")
        .unwrap()
        .commit_file("b.txt", "two\n", "second")
        .unwrap()
        .build();
    let root = repo.root().to_path_buf();
    git(&root, &["branch", "feat"]);
    git(&root, &["branch", "other", "HEAD~1"]);
    repo
}

fn launch(repo: &Path, session: &str, config: &Path) -> Terminal {
    common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(COLS, ROWS)
        .ready_when("the fixture's log and its branches", |s| {
            s.contains("second") && s.contains("feat") && s.contains("other")
        })
        .spawn()
}

fn sidebar_below(screen: &Screen, needle: &str, from: u16) -> Option<(u16, u16)> {
    screen
        .find_all(needle)
        .into_iter()
        .filter(|(col, row)| *col < SIDEBAR_RIGHT && *row > from)
        .min_by_key(|(_, row)| *row)
}

/// The sidebar cell of the LOCAL branch `label`.
fn local_row(screen: &Screen, label: &str) -> Option<(u16, u16)> {
    let header = sidebar_below(screen, "LOCAL (", 0)?.1;
    sidebar_below(screen, label, header)
}

fn wait_local(term: &Terminal, label: &str) -> (u16, u16) {
    let screen = term
        .wait_for(&format!("{label:?} under LOCAL"), STEP, |s| {
            local_row(s, label).is_some()
        })
        .unwrap_or_else(|e| panic!("{e}"));
    local_row(&screen, label).unwrap()
}

/// Selects the LOCAL branch `label` (a click focuses the pane and selects).
fn select(term: &mut Terminal, label: &str) {
    let (c, r) = wait_local(term, label);
    term.click(c + 1, r).unwrap();
    term.wait_for(&format!("{label:?} selected"), STEP, |s| {
        local_row(s, label).is_some()
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

fn sidebar_marked(line: &str) -> bool {
    line.chars()
        .take(SIDEBAR_RIGHT as usize)
        .any(|c| c == '\u{25C6}')
}

fn marked_rows(s: &Screen) -> usize {
    (0..s.rows())
        .filter(|r| sidebar_marked(&s.line(*r)))
        .count()
}

fn mark_on(s: &Screen, label: &str) -> bool {
    local_row(s, label).is_some_and(|(_, row)| sidebar_marked(&s.line(row)))
}

/// A drag of `feat` onto `other`, for comparison: the menu rows it offers.
fn drag_menu(term: &mut Terminal) {
    let from = wait_local(term, "feat");
    let to = wait_local(term, "other");
    term.drag(from, to).unwrap();
    term.wait_for("the drag's drop menu", STEP, |s| s.contains(FF))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn mark_then_drop_opens_the_menu_a_drag_opens_and_runs_the_choice() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "mark-drop-menu", config.path());

    // What a drag offers for the pair.
    drag_menu(&mut term);
    let dragged = term.screen();
    for row in [MERGE, REBASE, FF] {
        assert!(
            dragged.contains(row),
            "drag menu lacks {row:?}\n{}",
            term.text()
        );
    }
    term.send_keys("Esc").unwrap();
    term.wait_for("the drag menu closed", STEP, |s| !s.contains(FF))
        .unwrap_or_else(|e| panic!("{e}"));

    // Mark `feat`.
    select(&mut term, "feat");
    term.send_keys("m").unwrap();
    term.wait_for("feat marked, status line says how to drop", STEP, |s| {
        mark_on(s, "feat")
            && s.contains("feat marked: go to a target and press")
            && marked_rows(s) == 1
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Another ref, then drop: the same rows, keyboard navigable.
    select(&mut term, "other");
    term.send_keys("M").unwrap();
    let marked = term
        .wait_for("the drop menu, the mark spent", STEP, |s| {
            s.contains(FF) && marked_rows(s) == 0
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    for row in [MERGE, REBASE, FF] {
        assert!(
            marked.contains(row),
            "marked menu lacks {row:?}\n{}",
            term.text()
        );
    }

    // Down to "Fast-forward other to feat" and run it with Enter.
    let order = [MERGE, REBASE, "Interactive rebase feat onto other...", FF];
    let steps = order.iter().position(|r| *r == FF).unwrap();
    for _ in 0..steps {
        term.send_keys("Down").unwrap();
    }
    term.send_keys("Enter").unwrap();
    term.wait_for("the menu gone", STEP, |s| !s.contains(FF))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let root = repo.root();
    let want = git(root, &["rev-parse", "feat"]);
    let deadline = std::time::Instant::now() + STEP;
    loop {
        if git(root, &["rev-parse", "other"]) == want {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "other did not fast-forward to feat\n{}",
            term.text()
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}

#[test]
fn esc_and_a_second_m_clear_the_mark() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "mark-drop-clear", config.path());

    select(&mut term, "feat");
    term.send_keys("m").unwrap();
    term.wait_for("feat marked", STEP, |s| mark_on(s, "feat"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Esc").unwrap();
    term.wait_for("the mark cleared by Esc", STEP, |s| marked_rows(s) == 0)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Marked again, then `m` on the marked ref itself.
    term.send_keys("m").unwrap();
    term.wait_for("feat marked again", STEP, |s| mark_on(s, "feat"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("m").unwrap();
    term.wait_for("the mark cleared by m", STEP, |s| marked_rows(s) == 0)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Nothing marked: the drop says so and opens nothing.
    select(&mut term, "other");
    term.send_keys("M").unwrap();
    term.wait_for("the no-mark notice", STEP, |s| s.contains("no ref marked"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!term.screen().contains(FF), "{}", term.text());
}

#[test]
fn the_palette_marks_and_drops_and_a_cancelled_menu_leaves_no_mark() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "mark-drop-palette", config.path());

    select(&mut term, "feat");
    term.send_keys("C-p").unwrap();
    term.send_text("Mark as Drop Source").unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("Refs: Mark as Drop Source")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("feat marked", STEP, |s| mark_on(s, "feat"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    select(&mut term, "other");
    term.send_keys("C-p").unwrap();
    term.send_text("Drop Marked Ref Here").unwrap();
    term.wait_for("the palette row", STEP, |s| {
        s.contains("Refs: Drop Marked Ref Here...")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the drop menu, the mark spent", STEP, |s| {
        s.contains(FF) && marked_rows(s) == 0
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Cancelling changes nothing and the mark stays gone.
    term.send_keys("Esc").unwrap();
    term.wait_for("the menu closed", STEP, |s| {
        !s.contains(FF) && marked_rows(s) == 0
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_ne!(
        git(repo.root(), &["rev-parse", "other"]),
        git(repo.root(), &["rev-parse", "feat"])
    );
}
