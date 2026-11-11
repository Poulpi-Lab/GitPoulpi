//! Where a tag lives, and a delete that does what it says (owner, 2026-10-22:
//! the red Delete only deleted locally, a fetch brought the tag back, and it
//! was never clear which tags were local, remote or both).
//!
//! Real binary, real daemon, real `git`, a bare `origin` inside the
//! fixture's temp dir (no network). A fetch is what teaches the app the
//! remote's tags, so each test fetches once before looking at the markers.
//!
//! Markers (see `tag_where_ui`): up arrow = only here, down arrow = only on a
//! remote, left-right = both, up-down = both at different commits.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

const LOCAL: &str = "\u{2B06}";
const REMOTE: &str = "\u{2B07}";
const BOTH: &str = "\u{2194}";

/// `v1` is on origin too; `mine` was never pushed.
fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("base commit")
        .unwrap()
        .with_origin()
        .unwrap()
        .tag("v1")
        .unwrap()
        .tag("mine")
        .unwrap()
        .push("main")
        .unwrap()
        .push("refs/tags/v1")
        .unwrap()
        .build()
}

fn launch(repo: &Path, session: &str) -> (Terminal, tempfile::TempDir) {
    let config = tempfile::tempdir().expect("temp config dir");
    let term = common::launch(repo)
        .session(session)
        .config_dir(config.path())
        .size(200, 50)
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| s.contains("base commit"))
        .unwrap_or_else(|e| panic!("{e}"));
    (term, config)
}

/// The app learns a remote's tags from a fetch (there are no remote-tracking
/// refs for tags), so every test starts by fetching.
fn launch_fetched(repo: &Path, session: &str) -> (Terminal, tempfile::TempDir) {
    let (mut term, config) = launch(repo, session);
    fetch_all(&mut term);
    (term, config)
}

/// The sidebar row of tag `name` with its marker: ` name <marker>`. The Log
/// pill of the same tag reads `name <tag icon> <marker>`, so this needle only
/// matches the sidebar.
fn row_with(name: &str, marker: &str) -> String {
    format!("{name} {marker}")
}

fn wait_marked(term: &Terminal, name: &str, marker: &str) -> (u16, u16) {
    let needle = row_with(name, marker);
    term.wait_for(&format!("{needle} in the TAGS section"), STEP, |s| {
        s.contains(&needle)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()))
    .find(&needle)
    .expect("just seen")
}

fn open_delete_dialog(term: &mut Terminal, name: &str, marker: &str) {
    let (col, row) = wait_marked(term, name, marker);
    term.right_click(col + 1, row).expect("right-click the tag");
    let label = format!("Delete {name}...");
    let menu = term
        .wait_for("the tag's menu with its Delete", STEP, |s| {
            s.contains(&label)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = menu.find(&label).expect("the Delete row");
    term.click(c + 1, r).expect("click Delete");
    term.wait_for("the scope dialog", STEP, |s| {
        s.contains(&format!("Where should {name} be deleted?"))
            && s.contains("1  Delete locally")
            && s.contains("2  Delete on origin")
            && s.contains("3  Delete locally and on origin")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

fn local_has(repo: &FixtureRepo, tag: &str) -> bool {
    repo.try_git(&["rev-parse", "--verify", "-q", &format!("refs/tags/{tag}")])
        .is_some()
}

fn fetch_all(term: &mut Terminal) {
    // Retried: under load the chord's second key can arrive before the git
    // mode is armed, and a second fetch is harmless.
    common::retry_until(
        term,
        "the fetch's result",
        3,
        Duration::from_secs(15),
        |t| {
            t.send_keys("C-g").unwrap();
            t.send_keys("f").unwrap();
        },
        |s| s.contains("fetched every remote"),
    );
}

/// The three states side by side: pushed (both), never pushed (only here),
/// and the tooltip says so in words.
#[test]
fn the_tags_section_marks_where_each_tag_lives() {
    let repo = fixture();
    let (mut term, _config) = launch_fetched(repo.root(), "pty-tag-where-markers");
    let (col, row) = wait_marked(&term, "v1", BOTH);
    wait_marked(&term, "mine", LOCAL);

    // Hovering the pushed tag explains its marker in words, and what a
    // fetch does with it.
    term.move_to(col, row).unwrap();
    term.wait_for("the tooltip", STEP, |s| {
        s.contains("Here and on origin, at the same commit")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// Delete on both sides through the dialog; a refetch does not bring the
/// tag back (the bug the maintainer reported, fixed end to end).
#[test]
fn deleting_locally_and_on_the_remote_sticks_through_a_refetch() {
    let repo = fixture();
    let (mut term, _config) = launch_fetched(repo.root(), "pty-tag-where-both");
    open_delete_dialog(&mut term, "v1", BOTH);

    term.send_keys("3").unwrap();
    // Outward-facing: a confirm that names the remote.
    term.wait_for("the confirm naming origin", STEP, |s| {
        s.contains("here AND on origin")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        remote_tags(&repo).contains("v1"),
        "nothing is deleted before the confirm"
    );
    term.send_keys("y").unwrap();
    term.wait_for("the result line", STEP, |s| {
        s.contains("Deleted v1 here and on origin")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!local_has(&repo, "v1"), "gone locally");
    assert!(!remote_tags(&repo).contains("v1"), "gone on origin");

    fetch_all(&mut term);
    assert!(
        !local_has(&repo, "v1"),
        "the refetch must not bring it back"
    );
    term.wait_for("the TAGS section without v1", STEP, |s| {
        !s.contains(&row_with("v1", BOTH)) && s.contains(&row_with("mine", LOCAL))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// The combined delete is one Undo entry: Ctrl+Z asks (it re-creates the tag
/// on the server), and after the confirm the tag is back here and on origin.
#[test]
fn undoing_a_delete_here_and_on_the_remote_brings_the_tag_back_in_both_places() {
    let repo = fixture();
    let object = repo.git(&["rev-parse", "refs/tags/v1"]);
    let (mut term, _config) = launch_fetched(repo.root(), "pty-tag-where-undo");
    open_delete_dialog(&mut term, "v1", BOTH);
    term.send_keys("3").unwrap();
    term.wait_for("the confirm naming origin", STEP, |s| {
        s.contains("here AND on origin")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("y").unwrap();
    term.wait_for("the result line", STEP, |s| {
        s.contains("Deleted v1 here and on origin")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!local_has(&repo, "v1") && !remote_tags(&repo).contains("v1"));

    let mut asked = false;
    for _ in 0..60 {
        term.send_keys("C-z").unwrap();
        if term
            .wait_for("the undo question", Duration::from_secs(1), |s| {
                s.contains("re-creates tag v1 on origin")
            })
            .is_ok()
        {
            asked = true;
            break;
        }
    }
    assert!(asked, "the undo never asked:\n{}", term.text());
    assert!(
        !remote_tags(&repo).contains("v1"),
        "asking changes nothing on the server"
    );
    term.send_keys("y").unwrap();
    // Both places, in one predicate: the undo restores the remote tag and
    // the local one as two steps, in no guaranteed order.
    term.wait_for("the tag back on origin and here", STEP, |_| {
        remote_tags(&repo).contains("v1") && local_has(&repo, "v1")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(
        repo.git_in_remote(&["rev-parse", "refs/tags/v1"]),
        object,
        "the same tag object"
    );
}

/// The local-only delete of a tag a remote has: it works, it says the remote
/// still has it, and the next fetch does bring it back (git's behaviour,
/// now announced instead of surprising).
#[test]
fn a_local_only_delete_warns_that_the_next_fetch_brings_it_back() {
    let repo = fixture();
    let (mut term, _config) = launch_fetched(repo.root(), "pty-tag-where-local");
    open_delete_dialog(&mut term, "v1", BOTH);
    // The highlighted first row carries the warning before anything runs.
    term.wait_for("the warning on the first row", STEP, |s| {
        s.contains("brings it back")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    term.send_keys("1").unwrap();
    term.wait_for("the warning after the delete", STEP, |s| {
        s.contains("It is still on origin: the next fetch brings it back")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!local_has(&repo, "v1"));
    assert!(remote_tags(&repo).contains("v1"), "the remote is untouched");

    fetch_all(&mut term);
    assert!(local_has(&repo, "v1"), "git follows the remote's tags");
}

/// A tag deleted here but still on the remote is shown as remote only (down
/// arrow, dimmed) with a Fetch row that brings it back on demand.
#[test]
fn a_remote_only_tag_is_marked_and_can_be_fetched() {
    let repo = fixture();
    let (mut term, _config) = launch_fetched(repo.root(), "pty-tag-where-remote-only");
    wait_marked(&term, "v1", BOTH);
    // Deleted behind the app's back: the mirror still knows origin has it.
    repo.git(&["tag", "-d", "v1"]);
    let (col, row) = wait_marked(&term, "v1", REMOTE);

    term.right_click(col + 1, row).unwrap();
    let menu = term
        .wait_for("the remote-only tag's menu", STEP, |s| {
            s.contains("Fetch v1 from origin") && s.contains("Delete v1 on origin...")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = menu.find("Fetch v1 from origin").unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("the tag fetched", STEP, |s| {
        s.contains("Downloaded v1 from origin")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(local_has(&repo, "v1"));
    wait_marked(&term, "v1", BOTH);
}

fn remote_tags(repo: &FixtureRepo) -> String {
    repo.git_in_remote(&["tag", "--list"])
}
