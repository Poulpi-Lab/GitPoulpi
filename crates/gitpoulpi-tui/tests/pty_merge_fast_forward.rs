//! The merge fast-forward policy (`git.merge_fast_forward`, default `never`)
//! through a pty: a branch made from `dev` with two commits, `dev` untouched,
//! dragged onto `dev`. "Merge" makes a real merge commit (first parent the old
//! `dev`), the menu says so, and "Fast-forward dev to mybranch" stays one
//! item away. Real binary, real daemon, real `git`.

mod common;

use common::git_trimmed as git;

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(20);
const COLS: u16 = 200;
const ROWS: u16 = 50;
const SIDEBAR_RIGHT: u16 = 40;
const MERGE: &str = "Merge mybranch into dev (merge commit)";
const FF: &str = "Fast-forward dev to mybranch";

/// `dev` checked out, `mybranch` two commits ahead of it.
fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "pty-ff-base")
        .unwrap()
        .build();
    let root = repo.root().to_path_buf();
    git(&root, &["checkout", "-q", "-b", "dev"]);
    git(&root, &["checkout", "-q", "-b", "mybranch"]);
    for (file, message) in [("b.txt", "pty-ff-one"), ("c.txt", "pty-ff-two")] {
        std::fs::write(root.join(file), "x\n").unwrap();
        git(&root, &["add", file]);
        git(&root, &["commit", "-q", "-m", message]);
    }
    git(&root, &["checkout", "-q", "dev"]);
    repo
}

fn launch(repo: &Path, session: &str, config: &Path) -> Terminal {
    common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(COLS, ROWS)
        .ready_when("the log and the branches", |s| {
            s.contains("pty-ff-two") && s.contains("mybranch")
        })
        .spawn()
}

fn local_row(screen: &Screen, label: &str) -> Option<(u16, u16)> {
    let below = |needle: &str, from: u16| {
        screen
            .find_all(needle)
            .into_iter()
            .filter(|(col, row)| *col < SIDEBAR_RIGHT && *row > from)
            .min_by_key(|(_, row)| *row)
    };
    let header = below("LOCAL (", 0)?.1;
    below(label, header)
}

fn wait_local(term: &Terminal, label: &str) -> (u16, u16) {
    let screen = term
        .wait_for(&format!("{label:?} under LOCAL"), STEP, |s| {
            local_row(s, label).is_some()
        })
        .unwrap_or_else(|e| panic!("{e}"));
    local_row(&screen, label).unwrap()
}

/// Drags `mybranch` onto `dev` and waits for the drop menu.
fn drag_onto_dev(term: &mut Terminal) {
    let from = wait_local(term, "mybranch");
    let to = wait_local(term, "dev");
    term.drag(from, to).unwrap();
    term.wait_for("the drop menu", STEP, |s| s.contains(FF))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

fn parents(root: &Path, rev: &str) -> Vec<String> {
    git(root, &["rev-list", "--parents", "-n1", rev])
        .split_whitespace()
        .skip(1)
        .map(str::to_string)
        .collect()
}

fn wait_until(term: &Terminal, what: &str, mut ok: impl FnMut() -> bool) {
    let deadline = Instant::now() + STEP;
    while !ok() {
        assert!(Instant::now() < deadline, "never: {what}\n{}", term.text());
        std::thread::sleep(Duration::from_millis(200));
    }
}

#[test]
fn merging_a_fast_forwardable_branch_makes_a_merge_commit() {
    let repo = fixture();
    let root = repo.root();
    let old_dev = git(root, &["rev-parse", "dev"]);
    let tip = git(root, &["rev-parse", "mybranch"]);
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(root, "ff-policy-merge", config.path());

    drag_onto_dev(&mut term);
    let menu = term.screen();
    assert!(
        menu.contains(MERGE),
        "the Merge row says it makes a merge commit\n{}",
        term.text()
    );
    assert!(
        menu.contains(FF),
        "the Fast-forward row stays\n{}",
        term.text()
    );

    // The Merge row is the first one: Enter runs it.
    term.send_keys("Enter").unwrap();
    wait_until(&term, "dev is a merge commit", || {
        parents(root, "dev").len() == 2
    });
    let parents = parents(root, "dev");
    assert_eq!(parents[0], old_dev, "the first parent is the old dev");
    assert_eq!(parents[1], tip, "the second parent is the branch tip");
    assert_eq!(
        git(root, &["rev-parse", "HEAD"]),
        git(root, &["rev-parse", "dev"])
    );
}

#[test]
fn the_explicit_fast_forward_item_still_fast_forwards() {
    let repo = fixture();
    let root = repo.root();
    let old_dev = git(root, &["rev-parse", "dev"]);
    let tip = git(root, &["rev-parse", "mybranch"]);
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(root, "ff-policy-explicit", config.path());

    drag_onto_dev(&mut term);
    // Merge, Rebase, Interactive rebase, then Fast-forward.
    for _ in 0..3 {
        term.send_keys("Down").unwrap();
    }
    term.send_keys("Enter").unwrap();
    wait_until(&term, "dev fast-forwarded to mybranch", || {
        git(root, &["rev-parse", "dev"]) == tip
    });
    assert_ne!(old_dev, tip);
    assert_eq!(
        parents(root, "dev"),
        vec![git(root, &["rev-parse", "mybranch~1"])],
        "a plain fast-forward: no merge commit"
    );
}
