//! The Reflog view and the blocked-undo offer, end to end through a pty:
//! the real binary, daemon and `git`. Moves made with plain `git` outside
//! the app are exactly what the reflog exists for.

mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

struct Rig {
    term: Terminal,
    _config: tempfile::TempDir,
}

fn launch(repo: &FixtureRepo, session: &str) -> Rig {
    let (term, config) = common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .no_exit_pause()
        .ready_when("the toolbar and the log", |s| {
            s.contains("Undo") && s.contains("initial commit")
        })
        .mouse_capture()
        // The blocked-undo offer is what these tests are about: an outside
        // commit is not recorded as an entry here (pty_outside_undo.rs
        // covers it recorded).
        .env("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false")
        .start();
    Rig {
        term,
        _config: config,
    }
}

fn git(repo: &Path, args: &[&str]) -> String {
    common::git(repo, args).trim().to_string()
}

fn wait_until(what: &str, mut f: impl FnMut() -> bool) {
    let deadline = Instant::now() + STEP;
    while !f() {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// A non-checked-out branch moved with plain git: its reflog lists both
/// positions, "Restore topic to this position" moves it back (after a
/// confirmation), and one Ctrl+Z moves it forward again.
#[test]
fn restoring_a_branch_to_an_earlier_reflog_position_is_undoable() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .branch("topic")
        .unwrap()
        .checkout("main")
        .unwrap()
        .commit_file("b.txt", "two\n", "second commit")
        .unwrap()
        .build();
    let older = git(repo.root(), &["rev-parse", "topic"]);
    git(repo.root(), &["branch", "-f", "topic", "main"]);
    let newer = git(repo.root(), &["rev-parse", "topic"]);
    assert_ne!(older, newer);

    let mut rig = launch(&repo, "pty-reflog-branch");
    let (col, row) = rig
        .term
        .wait_for("the topic row", STEP, |s| s.contains("topic"))
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()))
        .find_all("topic")
        .into_iter()
        .min_by_key(|(col, _)| *col)
        .unwrap_or_else(|| panic!("no topic row:\n{}", rig.term.text()));
    rig.term.click(col, row).unwrap();
    rig.term.send_keys("C-p").unwrap();
    rig.term.send_text("reflog selected").unwrap();
    rig.term
        .wait_for("the palette row", STEP, |s| {
            s.contains("Reflog of the Selected Branch")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    rig.term.send_keys("Enter").unwrap();
    rig.term
        .wait_for("both positions of topic", STEP, |s| {
            s.contains("Past positions of topic")
                && s.contains("branch: Created")
                && s.contains("branch: Reset")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));

    rig.term.send_keys("Down r").unwrap();
    rig.term
        .wait_for("the confirmation", STEP, |s| {
            s.contains(&format!("Move topic to {}", &older[..7])) && s.contains("Cancel")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    assert_eq!(git(repo.root(), &["rev-parse", "topic"]), newer);
    rig.term.send_keys("Enter").unwrap();
    wait_until("topic to move back", || {
        git(repo.root(), &["rev-parse", "topic"]) == older
    });

    rig.term.send_keys("Escape").unwrap();
    rig.term
        .wait_for("the panel closed", STEP, |s| {
            !s.contains("Past positions of topic")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    rig.term.send_keys("C-z").unwrap();
    wait_until("Ctrl+Z to move topic forward again", || {
        git(repo.root(), &["rev-parse", "topic"]) == newer
    });
    rig.term.stop().unwrap();
}

/// A commit made with plain git blocks the Undo of the commit made in the
/// app; the panel offers "Restore main to where Commit left it", and taking
/// it puts main back on the app's commit.
#[test]
fn an_outside_commit_blocks_undo_and_the_offered_reflog_restore_works() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .build();
    let mut rig = launch(&repo, "pty-reflog-offer");
    std::fs::write(repo.root().join("mine.txt"), "mine").unwrap();
    git(repo.root(), &["add", "-A"]);
    rig.term
        .wait_for("the staged file", STEP, |s| s.contains("mine.txt"))
        .unwrap_or_else(|e| panic!("{e}"));
    rig.term.send_keys("c").unwrap();
    rig.term
        .wait_for("the Summary field", STEP, |s| {
            s.contains("Staged Files") && s.text().contains("\u{2502}_")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    rig.term.send_text("mine").unwrap();
    rig.term.send_keys("Enter C-j").unwrap();
    wait_until("the app's commit", || {
        git(repo.root(), &["log", "-1", "--format=%s"]) == "mine"
    });
    let mine = git(repo.root(), &["rev-parse", "HEAD"]);
    // Outside the app: a commit on top.
    git(repo.root(), &["commit", "--allow-empty", "-m", "outside"]);
    assert_ne!(git(repo.root(), &["rev-parse", "HEAD"]), mine);

    rig.term.send_keys("M-z").unwrap();
    rig.term
        .wait_for("the row blocked, with the offer", STEP, |s| {
            s.contains("Commit \"mine\"") && s.contains("blocked") && s.contains("to where")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    rig.term.send_keys("o").unwrap();
    rig.term
        .wait_for("the confirmation", STEP, |s| {
            s.contains("Move main to") && s.contains("Cancel")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    rig.term.send_keys("Enter").unwrap();
    wait_until("main back on the app's commit", || {
        git(repo.root(), &["rev-parse", "HEAD"]) == mine
    });
    rig.term.stop().unwrap();
}

/// A repository whose HEAD reflog holds 450 positions, "step 000" (oldest)
/// to "step 449": more than two pages of 200. The log file is written
/// directly, so the fixture costs no process per entry.
fn long_reflog_repo() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .build();
    let tip = git(repo.root(), &["rev-parse", "HEAD"]);
    let zero = "0".repeat(40);
    let mut log = String::new();
    for n in 0..450 {
        let old = if n == 0 { zero.as_str() } else { tip.as_str() };
        log.push_str(&format!(
            "{old} {tip} Tester <t@example.com> {} +0000\tcommit: step {n:03}\n",
            1_700_000_000 + n
        ));
    }
    std::fs::write(repo.root().join(".git").join("logs").join("HEAD"), log).unwrap();
    repo
}

/// Opens the Undo history panel on its Reflog view (HEAD).
fn open_head_reflog(rig: &mut Rig) {
    rig.term.send_keys("M-z").unwrap();
    rig.term
        .wait_for("the Undo history panel", STEP, |s| {
            s.contains("Nothing to undo yet") || s.contains("can be undone")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    rig.term.send_keys("2").unwrap();
    rig.term
        .wait_for("the newest position", STEP, |s| s.contains("step 449"))
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
}

/// The reflog is read a page (200) at a time. Moving the cursor to the end
/// fetches the next page by itself, so the list just keeps going until the
/// oldest position is reached.
#[test]
fn the_reflog_list_loads_the_next_page_when_the_cursor_reaches_the_end() {
    let repo = long_reflog_repo();
    let mut rig = launch(&repo, "pty-reflog-auto-more");
    open_head_reflog(&mut rig);
    assert!(!rig.term.screen().contains("step 000"), "one page only");

    // Page two (positions 249..50): the row after the last position of page
    // one becomes position 249. Then page three (the last 50).
    rig.term.send_keys("End").unwrap();
    rig.term
        .wait_for("the second page", STEP, |s| s.contains("step 249"))
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    // The cursor is on position 249, 200 rows down: End reaches the end of
    // page two, which fetches page three. The row that was "Load more" is
    // then position 49, and End goes to the oldest.
    rig.term.send_keys("End").unwrap();
    rig.term
        .wait_for("the third page", STEP, |s| s.contains("step 049"))
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    rig.term.send_keys("End").unwrap();
    rig.term
        .wait_for("the oldest position", STEP, |s| s.contains("step 000"))
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    // The whole reflog is here: nothing left to ask for.
    assert!(
        !rig.term.screen().contains("Load more"),
        "{}",
        rig.term.text()
    );
    rig.term.stop().unwrap();
}

/// The "Load more" button (key `m`) and its palette twin fetch the next page
/// without moving the cursor.
#[test]
fn load_more_has_a_key_a_button_and_a_palette_command() {
    let repo = long_reflog_repo();
    let mut rig = launch(&repo, "pty-reflog-manual-more");
    open_head_reflog(&mut rig);
    rig.term
        .wait_for("the Load more button", STEP, |s| s.contains("Load more"))
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));

    rig.term.send_keys("m").unwrap();
    // Page two is in, and said so (only a manual load announces itself).
    rig.term
        .wait_for("page two", STEP, |s| s.contains("positions loaded: 400"))
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));

    // The palette twin does the same for the last page.
    rig.term.send_keys("C-p").unwrap();
    rig.term
        .wait_for("the command palette over the panel", STEP, |s| {
            s.contains("Command Palette") || s.contains("commands)")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    rig.term.send_text("Reflog: Load More").unwrap();
    rig.term
        .wait_for("the row in the list", STEP, |s| {
            s.contains("Reflog: Load More")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    rig.term.send_keys("Enter").unwrap();
    rig.term
        .wait_for("the last page", STEP, |s| {
            s.contains("positions loaded: 450")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    rig.term.send_keys("End").unwrap();
    rig.term
        .wait_for("the oldest position", STEP, |s| s.contains("step 000"))
        .unwrap_or_else(|e| panic!("{e}\n{}", rig.term.text()));
    rig.term.stop().unwrap();
}
