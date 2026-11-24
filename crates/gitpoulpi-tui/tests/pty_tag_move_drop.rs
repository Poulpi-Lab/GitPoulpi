//! Dropping a TAG on another ref (`ref_drop`): the menu it opens and what each
//! row does, end to end through a pty (`xtask::pty`): the real binary, the
//! real daemon, real `git`.
//!
//! The tag is carried from the TAGS section of the sidebar onto a ref pill
//! beside the commit graph. Nothing is located by a hard-coded coordinate.

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

/// `main`: initial, second, third. `side` branches off `initial` with one
/// commit. `v1` (annotated) is on `initial`, `late` (lightweight) on `third`.
fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial")
        .unwrap()
        .branch("side")
        .unwrap()
        .commit_file("side.txt", "side\n", "side work")
        .unwrap()
        .checkout("main")
        .unwrap()
        .commit_file("b.txt", "two\n", "second")
        .unwrap()
        .commit_file("c.txt", "three\n", "third")
        .unwrap()
        .build();
    let root = repo.root().to_path_buf();
    git(&root, &["tag", "-a", "v1", "-m", "release one", "HEAD~2"]);
    git(&root, &["tag", "late", "HEAD"]);
    repo
}

fn launch(repo: &Path, session: &str, config: &Path) -> Terminal {
    common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(COLS, ROWS)
        .ready_when("the fixture's log and its tags", |s| {
            s.contains("third") && s.contains("side work") && s.contains("v1")
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

fn tag_row(screen: &Screen, label: &str) -> Option<(u16, u16)> {
    let header = sidebar_below(screen, "TAGS (", 0)?.1;
    sidebar_below(screen, label, header)
}

fn wait_tag(term: &Terminal, label: &str) -> (u16, u16) {
    let screen = term
        .wait_for(&format!("{label:?} under TAGS"), STEP, |s| {
            tag_row(s, label).is_some()
        })
        .unwrap_or_else(|e| panic!("{e}"));
    tag_row(&screen, label).unwrap()
}

/// The Log row whose message is `commit` (not the sidebar).
fn log_row(screen: &Screen, commit: &str) -> Option<u16> {
    screen
        .find_all(commit)
        .into_iter()
        .find(|(col, _)| *col > SIDEBAR_RIGHT && *col < 140)
        .map(|(_, row)| row)
}

/// The Log pill `label` on the row of `commit`.
fn pill(screen: &Screen, commit: &str, label: &str) -> Option<(u16, u16)> {
    let row = log_row(screen, commit)?;
    screen
        .find_all(label)
        .into_iter()
        .find(|(col, r)| *r == row && *col > SIDEBAR_RIGHT)
}

/// Carries the sidebar tag `tag` onto the Log pill `label` of the row
/// `commit` and releases it there.
fn drop_tag_on(term: &mut Terminal, tag: &str, commit: &str, label: &str) {
    let target = term
        .wait_for(&format!("the {label} pill on {commit}"), STEP, |s| {
            pill(s, commit, label).is_some()
        })
        .map(|s| pill(&s, commit, label).unwrap())
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (col, row) = wait_tag(term, tag);
    term.press(col, row).unwrap();
    term.drag_to(col + 2, row).unwrap();
    term.drag_to(target.0, target.1).unwrap();
    term.wait_for("the drag banner naming the target", STEP, |s| {
        s.contains(&format!("{label} - release to drop"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.release(target.0, target.1).unwrap();
}

fn click_row(term: &mut Terminal, text: &str) {
    let (c, r) = term
        .screen()
        .find(text)
        .unwrap_or_else(|| panic!("no {text:?}\n{}", term.text()));
    term.click(c + 1, r).unwrap();
}

/// A fast-forward move: the plain row, both merges below it, and choosing
/// the move puts the (still annotated) tag on the target.
#[test]
fn dropping_a_tag_on_a_descendant_offers_a_plain_move_and_both_merges() {
    let repo = fixture();
    let root = repo.root().to_path_buf();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(&root, "tag-drop-ff", config.path());
    let tip = git(&root, &["rev-parse", "HEAD"]);
    let old_object = git(&root, &["rev-parse", "refs/tags/v1"]);

    drop_tag_on(&mut term, "v1", "third", "main");
    let menu = term
        .wait_for("the drop menu", STEP, |s| s.contains("Move tag v1 here"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        !menu.contains("not a fast-forward"),
        "a descendant is a plain move:\n{}",
        term.text()
    );
    assert!(menu.contains("Merge tag v1 into main"), "{}", term.text());
    assert!(
        menu.contains("Merge main into tag v1 and move the tag"),
        "{}",
        term.text()
    );

    click_row(&mut term, "Move tag v1 here");
    term.wait_for("the move reported", STEP, |s| {
        s.contains("moved tag v1 to main")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(git(&root, &["rev-parse", "v1^{commit}"]), tip);
    assert_eq!(git(&root, &["cat-file", "-t", "refs/tags/v1"]), "tag");
    assert_ne!(git(&root, &["rev-parse", "refs/tags/v1"]), old_object);
}

/// A move that is not a fast-forward is still offered, flagged, and the
/// owner's choice moves the tag.
#[test]
fn dropping_a_tag_on_an_ancestor_flags_the_move_as_not_a_fast_forward() {
    let repo = fixture();
    let root = repo.root().to_path_buf();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(&root, "tag-drop-non-ff", config.path());
    let first = git(&root, &["rev-parse", "HEAD~2"]);

    // `side` is `initial`'s child: `late` (on `third`) is not its ancestor.
    drop_tag_on(&mut term, "late", "side work", "side");
    term.wait_for("the flagged row", STEP, |s| {
        s.contains("Move tag late here (not a fast-forward")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    click_row(&mut term, "Move tag late here (not a fast-forward");
    let side = git(&root, &["rev-parse", "side"]);
    term.wait_for("the move reported", STEP, |s| {
        s.contains("moved tag late to side")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(git(&root, &["rev-parse", "late"]), side);
    assert_ne!(side, first);
}

/// "Merge <branch> into <tag> and move the tag": a merge commit of the tag's
/// commit and the branch, the tag on it, no branch moved.
#[test]
fn merging_a_branch_into_a_tag_makes_a_merge_commit_the_tag_moves_onto() {
    let repo = fixture();
    let root = repo.root().to_path_buf();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(&root, "tag-drop-merge-into-tag", config.path());
    let main = git(&root, &["rev-parse", "main"]);
    let side = git(&root, &["rev-parse", "side"]);

    drop_tag_on(&mut term, "late", "side work", "side");
    term.wait_for("the menu", STEP, |s| {
        s.contains("Merge side into tag late and move the tag")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    click_row(&mut term, "Merge side into tag late and move the tag");
    // The new merge commit heads the graph (its message is cut at the column)
    // and carries the tag's pill.
    term.wait_for("the merge commit with the tag on it", STEP, |s| {
        pill(s, "Merge branch 'side", "late").is_some()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    let merged = git(&root, &["rev-parse", "late^{commit}"]);
    assert_eq!(
        git(&root, &["rev-list", "--parents", "-n1", &merged]),
        format!("{merged} {main} {side}")
    );
    assert_eq!(git(&root, &["rev-parse", "main"]), main);
    assert_eq!(git(&root, &["rev-parse", "side"]), side);
}
