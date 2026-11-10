//! Pushing a rewrite back, end to end through a pty (`xtask::pty`): the real
//! binary, daemon and git, with a local bare repository as the remote.
//!
//! - Apply on a pushed branch asks, "Rewrite anyway" rewrites it, and
//!   "Push rewritten branches" sends the ticked refs only: the unticked tag
//!   stays as the remote has it. The note for collaborators is copied whole.
//! - A remote that moved after the preview is a rejected row, the remote is
//!   not overwritten, and "Fetch and review" closes the workbench and fetches.
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, nothing sleeps, every wait describes the state it waits for.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

fn old_commit(repo: &FixtureRepo, message: &str) {
    repo.git(&[
        "-c",
        "user.name=Old Name",
        "-c",
        "user.email=old@example.com",
        "commit",
        "--allow-empty",
        "-q",
        "-m",
        message,
    ]);
}

/// `main` with a base commit, and a pushed `feature` with two commits by
/// old@example.com and a pushed tag `v1` on its tip.
fn fixture() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_origin()
        .unwrap()
        .commit_file("a.txt", "one\n", "base")
        .unwrap()
        .build();
    repo.git(&["push", "-q", "origin", "main"]);
    repo.git(&["checkout", "-q", "-b", "feature"]);
    old_commit(&repo, "first by old");
    old_commit(&repo, "second by old");
    repo.git(&["tag", "v1"]);
    repo.git(&["push", "-q", "origin", "feature", "v1"]);
    let tip = repo.git(&["rev-parse", "HEAD"]);
    // The tag mirror a fetch keeps; it is what says the remote has the tag.
    repo.git(&["update-ref", "refs/gitpoulpi/remote-tags/origin/v1", &tip]);
    repo
}

fn origin_rev(repo: &FixtureRepo, name: &str) -> String {
    repo.git_in_remote(&["rev-parse", name])
}

fn launch(repo: &FixtureRepo, session: &str, dump: &std::path::Path) -> Terminal {
    let config = tempfile::tempdir().expect("temp config dir");
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .env("GITPOULPI_CLIPBOARD_DUMP", dump)
        .size(200, 50)
        .spawn();
    // The config dir lives as long as the process: keep it for the test.
    std::mem::forget(config);
    term.wait_for("the fixture loaded", STARTUP, |s| {
        s.contains("second by old") && s.contains("first by old")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term
}

fn open_by_chord(term: &mut Terminal) {
    term.send_keys("C-g").unwrap();
    term.send_text("H").unwrap();
    term.wait_for("the workbench", STEP, |s| {
        s.contains("Rewrite history") && s.contains("Add a rule to start")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// An identity rule by keys: old@example.com becomes new@example.org.
fn fill_identity_by_keys(term: &mut Terminal) {
    term.send_text("a").unwrap();
    term.wait_for("the Add menu", STEP, |s| s.contains("Replace an author"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the rule's fields", STEP, |s| {
        s.contains("Email is") && s.contains("New email")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    term.send_text("old@example.com").unwrap();
    term.send_keys("Enter").unwrap();
    for _ in 0..3 {
        term.send_keys("Down").unwrap();
    }
    term.send_keys("Enter").unwrap();
    term.send_text("new@example.org").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("both emails in the fields", STEP, |s| {
        s.contains("old@example.com") && s.contains("new@example.org")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

fn wait_until(what: &str, mut ok: impl FnMut() -> bool) {
    let deadline = std::time::Instant::now() + STEP;
    while !ok() {
        assert!(std::time::Instant::now() < deadline, "timed out: {what}");
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Preview, then Apply, answering the question a pushed branch raises.
fn apply_pushed(term: &mut Terminal, by_mouse: bool) {
    term.send_keys("F5").unwrap();
    term.wait_for("the summary", STEP, |s| s.contains("keep their id"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-s").unwrap();
    term.wait_for("the pushed-history question", STEP, |s| {
        s.contains("Rewrite pushed history?") && s.contains("Rewrite anyway")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    if by_mouse {
        answer_rewrite_anyway(term);
    } else {
        term.send_keys("1").unwrap();
        term.wait_for("the result panel with the push list", STEP, |s| {
            s.contains("Rewrote 2") && s.contains("Push rewritten branches")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    }
}

/// Apply asks, the push sends the ticked branch and not the unticked tag,
/// and the note is copied whole.
#[test]
fn the_rewritten_branch_is_pushed_and_an_unticked_tag_stays_as_it_was() {
    let repo = fixture();
    let old_feature = origin_rev(&repo, "refs/heads/feature");
    let old_tag = origin_rev(&repo, "refs/tags/v1");
    let dump_dir = tempfile::tempdir().unwrap();
    let dump = dump_dir.path().join("clipboard.txt");
    let mut term = launch(&repo, "pty-rewrite-push", &dump);
    open_by_chord(&mut term);
    fill_identity_by_keys(&mut term);
    apply_pushed(&mut term, false);

    term.wait_for("the push list in full", STEP, |s| {
        s.contains("[x] feature")
            && s.contains("[x] v1")
            && s.contains("The remote keeps the new history even if you undo")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        origin_rev(&repo, "refs/heads/feature"),
        old_feature,
        "applying rewrites locally; the remote is untouched until Push"
    );

    // Untick the tag with a click on its box.
    // Retried: a box located on a torn frame can take the click nowhere.
    common::retry_until(
        &mut term,
        "the tag unticked",
        4,
        Duration::from_secs(10),
        |t| {
            if let Some((c, r)) = t.screen().find("[x] v1") {
                t.click(c + 1, r).expect("untick v1");
            }
        },
        |s| s.contains("[ ] v1"),
    );

    // The note first: it reads the applied run, not the push.
    term.send_keys("c").unwrap();
    wait_until("the note on the clipboard", || {
        std::fs::read_to_string(&dump).is_ok_and(|t| t.contains("git reset --hard origin/feature"))
    });
    let note = std::fs::read_to_string(&dump).unwrap();
    assert!(note.contains("git fetch origin"), "{note}");
    assert!(note.contains("git rebase --onto origin/feature"), "{note}");
    assert!(!note.contains('\u{2014}'), "{note}");

    term.send_keys("p").unwrap();
    term.wait_for("the pushed row", STEP, |s| {
        s.contains("feature  origin  pushed")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        origin_rev(&repo, "refs/heads/feature"),
        repo.git(&["rev-parse", "feature"]),
        "the remote has the rewritten branch"
    );
    assert_ne!(origin_rev(&repo, "refs/heads/feature"), old_feature);
    assert_eq!(
        origin_rev(&repo, "refs/tags/v1"),
        old_tag,
        "the unticked tag was not sent"
    );
    assert!(
        !term.screen().contains("v1  origin  pushed"),
        "{}",
        term.screen().text()
    );
    let log = repo.git_in_remote(&["log", "--format=%ae", "feature"]);
    assert!(
        log.contains("new@example.org") && !log.contains("old@example.com"),
        "{log}"
    );
}

/// The remote moved after the preview: the row is rejected, the remote keeps
/// the collaborator's commit, and "Fetch and review" closes the workbench and
/// fetches.
#[test]
fn a_remote_that_moved_is_a_rejected_row_and_fetch_and_review_fetches() {
    let repo = fixture();
    let dump_dir = tempfile::tempdir().unwrap();
    let mut term = launch(
        &repo,
        "pty-rewrite-push-moved",
        &dump_dir.path().join("c.txt"),
    );
    open_by_chord(&mut term);
    fill_identity_by_keys(&mut term);
    term.send_keys("F5").unwrap();
    term.wait_for("the summary", STEP, |s| s.contains("keep their id"))
        .unwrap_or_else(|e| panic!("{e}"));
    // A collaborator pushes to feature after the preview recorded the lease.
    repo.push_from_elsewhere("feature", "rival");
    let rival = origin_rev(&repo, "refs/heads/feature");
    apply_pushed_after_preview(&mut term);

    // Push by clicking the button.
    common::retry_until(
        &mut term,
        "the rejected row with its button",
        4,
        Duration::from_secs(10),
        |t| {
            if let Some((c, r)) = t.screen().find("Push rewritten branches") {
                t.click(c + 1, r).expect("click Push");
            }
        },
        |s| {
            s.contains("rejected: the remote moved since the preview")
                && s.contains("Fetch and review")
        },
    );
    assert_eq!(
        origin_rev(&repo, "refs/heads/feature"),
        rival,
        "the collaborator's commit is not overwritten"
    );
    assert_ne!(
        repo.git(&["rev-parse", "refs/remotes/origin/feature"]),
        rival,
        "nothing fetched yet"
    );

    common::retry_until(
        &mut term,
        "the workbench closed",
        4,
        Duration::from_secs(10),
        |t| {
            if let Some((c, r)) = t.screen().find("Fetch and review") {
                t.click(c + 1, r).expect("click Fetch and review");
            }
        },
        |s| !s.contains("Rewrite history"),
    );
    wait_until("origin/feature fetched", || {
        repo.try_git(&["rev-parse", "refs/remotes/origin/feature"])
            .as_deref()
            == Some(rival.as_str())
    });
}

/// Apply from a preview that is already on screen.
fn apply_pushed_after_preview(term: &mut Terminal) {
    term.send_keys("C-s").unwrap();
    term.wait_for("the pushed-history question", STEP, |s| {
        s.contains("Rewrite pushed history?") && s.contains("Rewrite anyway")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    answer_rewrite_anyway(term);
}

/// Clicks "Rewrite anyway" until the result panel with the push list shows;
/// a button located on a torn frame can take the click nowhere.
fn answer_rewrite_anyway(term: &mut Terminal) {
    common::retry_until(
        term,
        "the result panel with the push list",
        4,
        Duration::from_secs(10),
        |t| {
            if let Some((c, r)) = t.screen().find("Rewrite anyway") {
                t.click(c + 1, r).expect("click Rewrite anyway");
            }
        },
        |s| s.contains("Rewrote 2") && s.contains("Push rewritten branches"),
    );
}
