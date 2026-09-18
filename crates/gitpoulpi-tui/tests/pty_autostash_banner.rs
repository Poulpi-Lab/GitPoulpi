//! The pending auto-stash banner: a checkout whose stash cannot be put
//! back (the pop conflicts) leaves a record; the TUI shows a strip with
//! "Pop now" / "Show stash" / "Drop", each also a palette command, and the
//! strip vanishes live once the stash is popped or dropped.

mod common;

use common::{git_ok, git_try};

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const SIDEBAR_RIGHT: u16 = 40;

/// `topic` edits line 1; the working tree edits line 1 too: the checkout
/// is blocked, the stash is made, and putting it back conflicts.
fn conflicting_fixture() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "1\n2\n3\n4\n5\n", "base commit")
        .unwrap()
        .branch("topic")
        .unwrap()
        .checkout("topic")
        .unwrap()
        .commit_file("a.txt", "1 topic\n2\n3\n4\n5\n", "topic edit")
        .unwrap()
        .checkout("main")
        .unwrap()
        .build();
    std::fs::write(repo.root().join("a.txt"), "1 local\n2\n3\n4\n5\n").unwrap();
    repo
}

fn sidebar_topic(screen: &Screen) -> Option<(u16, u16)> {
    screen
        .find_all("topic")
        .into_iter()
        .find(|(col, _)| *col < SIDEBAR_RIGHT)
}

/// Double-clicks `topic` in the Refs sidebar until `done` holds, finding the
/// row afresh for every attempt.
///
/// A double-click is two events the TUI times itself, from when it *handles*
/// each (`DOUBLE_CLICK_WINDOW`, 400 ms): on a machine busy enough to stall
/// it between the two, the pair is read as two single clicks, which only
/// select the row. Repeating is safe here (checking out `topic` again is a
/// no-op or a refusal, never a second effect).
fn double_click_topic_until(term: &mut Terminal, what: &str, done: impl FnMut(&Screen) -> bool) {
    term.act_until(
        what,
        Duration::from_secs(5),
        STEP,
        |t| {
            let screen = t.wait_for("`topic` in the Refs sidebar", STEP, |s| {
                sidebar_topic(s).is_some()
            })?;
            let (col, row) = sidebar_topic(&screen).unwrap();
            t.double_click(col + 1, row)
        },
        done,
    )
    .unwrap_or_else(|e| panic!("{e}"));
}

/// Launches, checks `topic` out (auto-stash `always`) and waits for the banner.
fn launch_with_banner(repo: &Path, session: &str) -> (Terminal, tempfile::TempDir) {
    let (mut term, config) = common::launch(repo)
        .session(session)
        .config("[general]\nauto_stash = \"always\"\n")
        .size(240, 50)
        .no_exit_pause()
        .ready_when("the fixture's log", |s| {
            s.contains("topic edit") && s.contains("base commit")
        })
        .start();
    // The banner (a state event) and the "Checkout failed" dialog (the
    // request's answer) arrive independently, in either order. The dialog is
    // modal: left up, it swallows the next click or key of the test, which on
    // a loaded machine is exactly when it is still on its way. Wait for both
    // and dismiss it, so the test starts from a quiet screen.
    double_click_topic_until(&mut term, "the banner and the failure dialog", |s| {
        s.contains("Auto-stash pending")
            && s.contains("Pop now")
            && s.contains("Drop")
            && s.contains("Checkout failed")
    });
    term.send_keys("Escape").unwrap();
    term.wait_for("the dialog closed", STEP, |s| {
        !s.contains("Checkout failed") && s.contains("Pop now")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    (term, config)
}

/// Back to a tree the stash applies to cleanly, so "Pop now" can succeed.
fn make_poppable(repo: &Path) {
    git_ok(repo, &["reset", "-q", "--hard"]);
    git_ok(repo, &["checkout", "-q", "main"]);
}

#[test]
fn pop_now_restores_the_changes_and_the_banner_vanishes() {
    let repo = conflicting_fixture();
    let (mut term, _config) = launch_with_banner(repo.root(), "pty-autostash-banner-pop");
    assert!(git_try(repo.root(), &["stash", "list"]).is_some_and(|l| !l.is_empty()));
    make_poppable(repo.root());

    let (col, row) = settled(&term, "Pop now");
    term.click(col + 1, row).unwrap();
    term.wait_for("the banner gone", STEP, |s| {
        !s.contains("Auto-stash pending")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap(),
        "1 local\n2\n3\n4\n5\n"
    );
    assert!(git_try(repo.root(), &["stash", "list"]).is_some_and(|l| l.is_empty()));
}

#[test]
fn drop_asks_then_drops_and_the_banner_vanishes() {
    let repo = conflicting_fixture();
    let (mut term, _config) = launch_with_banner(repo.root(), "pty-autostash-banner-drop");
    make_poppable(repo.root());

    let (col, row) = term
        .screen()
        .find_all("Drop")
        .into_iter()
        .next()
        .expect("the Drop button");
    term.click(col + 1, row).unwrap();
    term.wait_for("the confirmation", STEP, |s| s.contains("auto-stash"))
        .unwrap_or_else(|e| panic!("{e}"));
    // Nothing dropped before the answer.
    assert!(git_try(repo.root(), &["stash", "list"]).is_some_and(|l| !l.is_empty()));
    term.send_keys("y").unwrap();
    term.wait_for("the banner gone", STEP, |s| {
        !s.contains("Auto-stash pending")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(git_try(repo.root(), &["stash", "list"]).is_some_and(|l| l.is_empty()));
}

#[test]
fn the_palette_reaches_show_stash_and_pop() {
    let repo = conflicting_fixture();
    let (mut term, _config) = launch_with_banner(repo.root(), "pty-autostash-banner-palette");

    term.send_keys("C-p").unwrap();
    term.send_text("Show pending").unwrap();
    term.wait_for("the palette entry", STEP, |s| {
        s.contains("Banner: Show Pending Auto-Stash")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the stash selected", STEP, |s| {
        s.contains("auto-stash: stash@{0}")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    make_poppable(repo.root());
    term.send_keys("C-p").unwrap();
    term.send_text("Pop pending").unwrap();
    term.wait_for("the palette entry", STEP, |s| {
        s.contains("Banner: Pop Pending Auto-Stash")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the banner gone", STEP, |s| {
        !s.contains("Auto-stash pending")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(git_try(repo.root(), &["stash", "list"]).is_some_and(|l| l.is_empty()));
}

#[test]
fn a_checkout_is_refused_while_the_banner_shows_and_works_again_after_the_pop() {
    let repo = conflicting_fixture();
    let (mut term, _config) = launch_with_banner(repo.root(), "pty-autostash-banner-block");
    make_poppable(repo.root());

    // Checking `topic` out again is refused, pointing at the banner.
    double_click_topic_until(&mut term, "the refusal", |s| {
        s.contains("is blocked: auto-stash")
    });
    assert!(term.screen().contains("Auto-stash pending"));
    assert_eq!(
        git_try(repo.root(), &["rev-parse", "--abbrev-ref", "HEAD"])
            .unwrap()
            .trim(),
        "main"
    );

    // The palette grays the checkout with the reason.
    term.send_keys("C-p").unwrap();
    term.send_text("Refs: Checkout").unwrap();
    term.wait_for("the reason", STEP, |s| {
        s.contains("an auto-stash is pending")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").unwrap();
    // The click below must not race the palette's close (a click that lands
    // while the palette is still up is swallowed by it).
    term.wait_for("the palette closed", STEP, |s| {
        !s.contains("an auto-stash is pending")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Pop it; the block lifts.
    let (col, row) = settled(&term, "Pop now");
    term.click(col + 1, row).unwrap();
    term.wait_for("the banner gone", STEP, |s| {
        !s.contains("Auto-stash pending")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    git_ok(repo.root(), &["reset", "-q", "--hard"]);
    let deadline = std::time::Instant::now() + xtask::pty::scaled(STEP);
    loop {
        let (col, row) = sidebar_topic(&term.screen()).expect("topic in the sidebar");
        term.double_click(col + 1, row).unwrap();
        // Give this attempt a few seconds before repeating it (a repeat once
        // `topic` is checked out is a no-op).
        let attempt = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            let head = git_try(repo.root(), &["rev-parse", "--abbrev-ref", "HEAD"]).unwrap();
            if head.trim() == "topic" {
                return;
            }
            if std::time::Instant::now() >= attempt {
                assert!(std::time::Instant::now() < deadline, "still on {head}");
                break;
            }
            std::thread::sleep(Duration::from_millis(200));
        }
    }
}

/// Stuck state #4: the stash is dropped with plain git while the banner
/// shows. The banner goes away on its own and the status line says why.
#[test]
fn a_stash_dropped_outside_clears_the_banner_and_says_so() {
    let repo = conflicting_fixture();
    let (mut term, _config) = launch_with_banner(repo.root(), "pty-autostash-banner-external");
    git_ok(repo.root(), &["stash", "drop", "-q"]);
    // Whatever refreshes next (the file watcher, or this key) re-reads the
    // stash list.
    term.send_keys("r").unwrap();
    term.wait_for("the banner gone and the reason", STEP, |s| {
        !s.contains("Auto-stash pending") && s.contains("outside GitPoulpi")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// Stuck state #2: another stash is pushed on top, shifting the index. The
/// banner's "Pop now" still restores the auto-stash, not the other one.
#[test]
fn pop_now_takes_the_right_stash_after_the_index_shifted() {
    let repo = conflicting_fixture();
    let (mut term, _config) = launch_with_banner(repo.root(), "pty-autostash-banner-shift");
    make_poppable(repo.root());
    std::fs::write(repo.root().join("a.txt"), "1\n2\n3\n4\n5 foreign\n").unwrap();
    git_ok(repo.root(), &["stash", "push", "-q", "-m", "foreign"]);

    let (col, row) = settled(&term, "Pop now");
    term.click(col + 1, row).unwrap();
    term.wait_for("the banner gone", STEP, |s| {
        !s.contains("Auto-stash pending")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap(),
        "1 local\n2\n3\n4\n5\n"
    );
    let list = git_try(repo.root(), &["stash", "list"]).unwrap();
    assert!(
        list.contains("foreign") && list.lines().count() == 1,
        "{list}"
    );
}

/// Stuck state #1: a checkout blocked by an untracked file; the auto-stash
/// takes it, and putting it back fails ("already exists"), applying nothing.
/// The banner says so with git's reason and offers *Retry pop*, which works
/// once the obstacle is gone.
#[test]
fn a_failed_pop_with_nothing_applied_says_so_and_retry_pop_restores() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "1\n2\n3\n4\n5\n", "base commit")
        .unwrap()
        .branch("topic")
        .unwrap()
        .checkout("topic")
        .unwrap()
        .commit_file("u.txt", "tracked on topic\n", "topic adds u")
        .unwrap()
        .checkout("main")
        .unwrap()
        .build();
    std::fs::write(repo.root().join("u.txt"), "mine\n").unwrap();
    let (mut term, _config) = common::launch(repo.root())
        .session("pty-autostash-banner-popfailed")
        .config("[general]\nauto_stash = \"always\"\ninclude_untracked_in_autostash = true\n")
        .size(240, 50)
        .no_exit_pause()
        .ready_when("the fixture's log", |s| {
            s.contains("topic adds u") && s.contains("base commit")
        })
        .start();
    double_click_topic_until(&mut term, "the failed-pop banner", |s| {
        s.contains("Auto-stash pending")
            && s.contains("failed")
            && s.contains("Retry pop")
            && s.contains("already exists")
    });
    let screen = term.screen();
    assert!(!screen.contains("Keep result") && !screen.contains("Resolve conflicts"));
    assert!(git_try(repo.root(), &["stash", "list"]).is_some_and(|l| !l.is_empty()));

    // Drop warns that the stash is the only copy, and nothing is dropped
    // before the answer.
    let (col, row) = screen
        .find_all("Drop")
        .into_iter()
        .next()
        .expect("the Drop button");
    term.click(col + 1, row).unwrap();
    term.wait_for("the strong warning", STEP, |s| s.contains("ONLY"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("n").unwrap();
    assert!(git_try(repo.root(), &["stash", "list"]).is_some_and(|l| !l.is_empty()));

    // Remove the obstacle with plain git (u.txt is tracked on topic only).
    git_ok(repo.root(), &["checkout", "-q", "main"]);
    let (col, row) = settled(&term, "Retry pop");
    term.click(col + 1, row).unwrap();
    term.wait_for("the banner gone", STEP, |s| {
        !s.contains("Auto-stash pending")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        std::fs::read_to_string(repo.root().join("u.txt")).unwrap(),
        "mine\n"
    );
    assert!(git_try(repo.root(), &["stash", "list"]).is_some_and(|l| l.is_empty()));
}

/// Stuck states #3 / #5: the banner's *Start over* asks first, then takes
/// the checkout back (HEAD on `main` again) and pops the stash: the local
/// edit is restored exactly, the stash and the banner are gone.
#[test]
fn start_over_asks_then_takes_the_checkout_back_and_pops_the_stash() {
    let repo = conflicting_fixture();
    let (mut term, _config) = launch_with_banner(repo.root(), "pty-autostash-banner-startover");
    assert_eq!(
        git_try(repo.root(), &["rev-parse", "--abbrev-ref", "HEAD"])
            .unwrap_or_default()
            .trim(),
        "topic"
    );
    let (col, row) = term
        .screen()
        .find("Start over")
        .expect("the Start over button");
    term.click(col + 1, row).unwrap();
    term.wait_for("the confirmation", STEP, |s| s.contains("Start over?"))
        .unwrap_or_else(|e| panic!("{e}"));
    // Nothing changed before the answer.
    assert!(git_try(repo.root(), &["stash", "list"]).is_some_and(|l| !l.is_empty()));
    term.send_keys("n").unwrap();
    assert_eq!(
        git_try(repo.root(), &["rev-parse", "--abbrev-ref", "HEAD"])
            .unwrap_or_default()
            .trim(),
        "topic"
    );

    // The palette is the keyboard twin.
    term.send_keys("C-p").unwrap();
    term.send_text("Start over").unwrap();
    term.wait_for("the palette entry", STEP, |s| {
        s.contains("Banner: Start Over")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the confirmation", STEP, |s| s.contains("Start over?"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("y").unwrap();
    term.wait_for("the banner gone", STEP, |s| {
        !s.contains("Auto-stash pending")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        git_try(repo.root(), &["rev-parse", "--abbrev-ref", "HEAD"])
            .unwrap_or_default()
            .trim(),
        "main"
    );
    assert_eq!(
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap(),
        "1 local\n2\n3\n4\n5\n"
    );
    assert!(git_try(repo.root(), &["stash", "list"]).is_some_and(|l| l.is_empty()));
}

/// Undo while the banner shows routes to Start over, with a question that
/// says what will happen (it used to be refused).
#[test]
fn undo_with_a_pending_auto_stash_routes_to_start_over() {
    let repo = conflicting_fixture();
    let (mut term, _config) = launch_with_banner(repo.root(), "pty-autostash-banner-undo");

    term.send_keys("C-p").unwrap();
    term.send_text("Edit: Undo").unwrap();
    term.wait_for("the palette entry", STEP, |s| s.contains("Edit: Undo"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the question", STEP, |s| s.contains("Start over:"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(git_try(repo.root(), &["stash", "list"]).is_some_and(|l| !l.is_empty()));
    term.send_keys("y").unwrap();
    term.wait_for("the banner gone", STEP, |s| {
        !s.contains("Auto-stash pending")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        git_try(repo.root(), &["rev-parse", "--abbrev-ref", "HEAD"])
            .unwrap_or_default()
            .trim(),
        "main"
    );
    assert_eq!(
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap(),
        "1 local\n2\n3\n4\n5\n"
    );
}

/// Stuck state #7: the record of an auto-stash waiting for a stopped
/// operation (`after_operation`), the operation ended with plain git (no
/// marker left): the next refresh pops the stash and the banner goes.
#[test]
fn a_stash_waiting_for_an_operation_that_ended_outside_is_popped_on_refresh() {
    let repo = conflicting_fixture();
    let (mut term, _config) = launch_with_banner(repo.root(), "pty-autostash-banner-ended");
    make_poppable(repo.root());
    let state_file = repo.state_dir().join("repo-state.toml");
    let text = std::fs::read_to_string(&state_file).unwrap();
    let text = text.replace("Conflicted", "Applied");
    let text = text.replace("state = ", "after_operation = true\nstate = ");
    std::fs::write(&state_file, text).unwrap();

    term.send_keys("r").unwrap();
    term.wait_for("the banner gone", STEP, |s| {
        !s.contains("Auto-stash pending")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap(),
        "1 local\n2\n3\n4\n5\n"
    );
    assert!(git_try(repo.root(), &["stash", "list"]).is_some_and(|l| l.is_empty()));
}

/// Where `text` holds still, for a click that cannot be repeated (Pop, Retry:
/// a slow success repeated would be a second effect). The banner sits under
/// rows the daemon's refreshes (the HEAD change a test makes outside) can
/// still move.
fn settled(term: &Terminal, text: &str) -> (u16, u16) {
    term.wait_for_settled_text(text, Duration::from_millis(300), STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()))
}
