//! The ref snapshot diff that finds changes made outside GitPoulpi
//! (the outside-operations design). Ground truth is always
//! plain `git` run in the fixture; the snapshot is read through the backend
//! under test, so both backends must see the same moves.

use crate::TestRepoBuilder;
use gitpoulpi_core::domain::HeadState;
use gitpoulpi_core::journal::{diff_outside, OutsideChange, OutsideDiff, OutsideSnapshot};
use gitpoulpi_core::GitBackend;

fn snap(backend: &dyn GitBackend, repo: &crate::FixtureRepo) -> OutsideSnapshot {
    OutsideSnapshot::capture(backend, &repo.repo_ref()).unwrap()
}

fn diff(
    backend: &dyn GitBackend,
    repo: &crate::FixtureRepo,
    before: &OutsideSnapshot,
) -> (OutsideDiff, OutsideSnapshot) {
    let after = snap(backend, repo);
    let d = diff_outside(backend, &repo.repo_ref(), before, &after).unwrap();
    (d, after)
}

fn change_of<'a>(d: &'a OutsideDiff, needle: &str) -> &'a OutsideChange {
    d.changes
        .iter()
        .find(|c| c.message.contains(needle))
        .unwrap_or_else(|| panic!("no change mentioning {needle:?} in {:#?}", d.changes))
}

/// Nothing moved: nothing to report, and the snapshot is stable.
pub fn check_outside_diff_of_an_unchanged_repo_is_quiet(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .branch("topic")
        .build();
    let before = snap(backend, &repo);
    let (d, after) = diff(backend, &repo, &before);
    assert!(d.is_quiet(), "{d:?}");
    assert_eq!(before, after);
}

/// A commit, a deleted branch, a deleted tag and a hard reset made with
/// plain `git`: each is named, with the old and new values git holds.
pub fn check_outside_diff_names_commit_branch_delete_tag_delete_and_reset(
    backend: &dyn GitBackend,
) {
    let repo = TestRepoBuilder::new().with_initial().build();
    repo.git(&["branch", "doomed"]);
    repo.git(&["tag", "v1"]);
    let before = snap(backend, &repo);
    let main_before = repo.git(&["rev-parse", "main"]).trim().to_string();

    repo.git(&["commit", "--allow-empty", "-m", "outside work"]);
    repo.git(&["branch", "-D", "doomed"]);
    repo.git(&["tag", "-d", "v1"]);
    let (d, _) = diff(backend, &repo, &before);

    let commit = change_of(&d, "commit: outside work");
    assert_eq!(commit.refs.len(), 1);
    assert_eq!(commit.refs[0].name, "refs/heads/main");
    assert_eq!(commit.refs[0].old.unwrap().to_hex(), main_before);
    assert_eq!(
        commit.refs[0].new.unwrap().to_hex(),
        repo.git(&["rev-parse", "main"]).trim()
    );
    assert!(commit.committer.as_deref().is_some_and(|c| c.contains('<')));
    assert!(commit.head.is_none(), "HEAD stayed on main");

    let gone = change_of(&d, "deleted");
    let names: Vec<&str> = gone.refs.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(names, ["refs/heads/doomed", "refs/tags/v1"]);
    assert!(gone.refs.iter().all(|r| r.old.is_some() && r.new.is_none()));
    assert_eq!(d.changes.len(), 2, "{:#?}", d.changes);

    // A hard reset back: the branch moves, the reason says so.
    let mid = snap(backend, &repo);
    repo.git(&["reset", "--hard", &main_before]);
    let (d, _) = diff(backend, &repo, &mid);
    let reset = change_of(&d, "reset: moving to");
    assert_eq!(reset.refs[0].name, "refs/heads/main");
    assert_eq!(reset.refs[0].new.unwrap().to_hex(), main_before);
}

/// A rebase rewrites several commits and moves the branch once: one change,
/// not one per commit.
pub fn check_outside_diff_of_a_rebase_is_one_change(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .branch("feature")
        .build();
    for n in 1..=3 {
        repo.git(&["commit", "--allow-empty", "-m", &format!("f{n}")]);
    }
    repo.git(&["checkout", "main"]);
    repo.git(&["commit", "--allow-empty", "-m", "on main"]);
    repo.git(&["checkout", "feature"]);
    let before = snap(backend, &repo);

    repo.git(&["rebase", "main"]);
    let (d, _) = diff(backend, &repo, &before);
    assert_eq!(d.changes.len(), 1, "{:#?}", d.changes);
    let rebase = &d.changes[0];
    assert!(
        rebase.message.starts_with("rebase (finish)"),
        "{}",
        rebase.message
    );
    assert_eq!(rebase.refs.len(), 1);
    assert_eq!(rebase.refs[0].name, "refs/heads/feature");
    assert!(rebase.head.is_none(), "the rebase ended on the same branch");
}

/// Many commits made between two looks are one change from the first value to
/// the last.
pub fn check_outside_diff_folds_a_burst_of_commits_into_one_change(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let before = snap(backend, &repo);
    let first = repo.git(&["rev-parse", "main"]).trim().to_string();
    for n in 1..=5 {
        repo.git(&["commit", "--allow-empty", "-m", &format!("burst {n}")]);
    }
    let (d, _) = diff(backend, &repo, &before);
    assert_eq!(d.changes.len(), 1, "{:#?}", d.changes);
    assert_eq!(d.changes[0].refs[0].old.unwrap().to_hex(), first);
    assert!(d.changes[0].message.contains("burst 5"));
}

/// A new branch checked out (`checkout -b`) is one change that carries both
/// the new ref and the HEAD move; a plain checkout is a HEAD-only change.
pub fn check_outside_diff_joins_a_checkout_with_the_branch_it_created(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    repo.git(&["branch", "other"]);
    let before = snap(backend, &repo);
    repo.git(&["checkout", "-b", "fresh"]);
    let (d, mid) = diff(backend, &repo, &before);
    assert_eq!(d.changes.len(), 1, "{:#?}", d.changes);
    let c = &d.changes[0];
    assert_eq!(c.refs.len(), 1);
    assert_eq!(c.refs[0].name, "refs/heads/fresh");
    assert!(c.refs[0].old.is_none());
    let head = c.head.as_ref().expect("HEAD moved with it");
    assert_eq!(head.old, HeadState::Branch("main".into()));
    assert_eq!(head.new, HeadState::Branch("fresh".into()));

    repo.git(&["checkout", "other"]);
    let (d, _) = diff(backend, &repo, &mid);
    assert_eq!(d.changes.len(), 1, "{:#?}", d.changes);
    assert!(d.changes[0].refs.is_empty());
    assert_eq!(
        d.changes[0].head.as_ref().unwrap().new,
        HeadState::Branch("other".into())
    );
    assert!(d.changes[0].message.starts_with("checkout:"));
}

/// A detached HEAD moved by a commit is a HEAD-only change.
pub fn check_outside_diff_sees_a_detached_head_move(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    repo.git(&["checkout", "--detach"]);
    let before = snap(backend, &repo);
    repo.git(&["commit", "--allow-empty", "-m", "detached work"]);
    let (d, _) = diff(backend, &repo, &before);
    assert_eq!(d.changes.len(), 1, "{:#?}", d.changes);
    let head = d.changes[0].head.as_ref().unwrap();
    assert!(matches!(head.old, HeadState::Detached(_)));
    assert!(matches!(head.new, HeadState::Detached(_)));
    assert_ne!(head.old, head.new);
    assert!(d.changes[0].refs.is_empty());
}

/// A stash made and one dropped are found by commit, not by position.
pub fn check_outside_diff_sees_stashes_pushed_and_dropped(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    std::fs::write(repo.root().join("a.txt"), "wip").unwrap();
    repo.git(&["add", "a.txt"]);
    let before = snap(backend, &repo);
    repo.git(&["stash", "push", "-m", "outside stash"]);
    let (d, mid) = diff(backend, &repo, &before);
    assert_eq!(d.changes.len(), 1, "{:#?}", d.changes);
    let st = d.changes[0].stash.as_ref().unwrap();
    assert_eq!(st.added.len(), 1);
    assert!(st.removed.is_empty());
    assert!(st.added[0].message.contains("outside stash"));
    let commit = st.added[0].commit;

    repo.git(&["stash", "drop"]);
    let (d, _) = diff(backend, &repo, &mid);
    assert_eq!(d.changes.len(), 1, "{:#?}", d.changes);
    let st = d.changes[0].stash.as_ref().unwrap();
    assert!(st.added.is_empty());
    assert_eq!(st.removed[0].commit, commit);
    assert!(d.changes[0].message.starts_with("stash dropped"));
}

/// A linked worktree added with a new branch and removed again: the list
/// change is reported, and the label of the change that made the branch
/// says so (the worktree alone is not an entry).
pub fn check_outside_diff_sees_worktrees_added_and_removed(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let before = snap(backend, &repo);
    let wt = repo.root().parent().unwrap().join("agent-wt");
    let wt_str = wt.to_string_lossy().to_string();
    repo.git(&["worktree", "add", "-b", "agent-branch", &wt_str]);
    let (d, mid) = diff(backend, &repo, &before);
    assert_eq!(d.worktrees_added.len(), 1, "{d:?}");
    assert!(d.worktrees_added[0].ends_with("agent-wt"));
    let made = d
        .changes
        .iter()
        .find(|c| c.refs.iter().any(|r| r.name == "refs/heads/agent-branch"))
        .expect("the branch the worktree carries is a change");
    assert!(made.label().contains("agent-branch"), "{}", made.label());
    assert!(
        made.notes.iter().any(|n| n.starts_with("worktree added")),
        "{:?}",
        made.notes
    );

    repo.git(&["worktree", "remove", &wt_str]);
    let (d, _) = diff(backend, &repo, &mid);
    assert_eq!(d.worktrees_removed.len(), 1, "{d:?}");
    assert!(d.is_quiet(), "removing a worktree moves no ref: {d:?}");
}

/// A fetch moves remote-tracking refs: counted, never a change.
pub fn check_outside_diff_counts_fetches_without_reporting_them(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().with_origin().build();
    repo.git(&["push", "origin", "main"]);
    repo.git(&["fetch", "origin"]);
    let before = snap(backend, &repo);
    repo.push_from_elsewhere("main", "elsewhere");
    repo.git(&["fetch", "origin"]);
    let (d, _) = diff(backend, &repo, &before);
    assert!(d.is_quiet(), "{d:?}");
    assert_eq!(d.fetched, 1, "{d:?}");
}

/// Changes made while nothing was looking are still found: the diff only
/// needs the older snapshot, whenever it was taken.
pub fn check_outside_diff_spans_many_kinds_of_moves_at_once(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    repo.git(&["branch", "old"]);
    let before = snap(backend, &repo);
    repo.git(&["commit", "--allow-empty", "-m", "one"]);
    repo.git(&["branch", "-D", "old"]);
    repo.git(&["tag", "release"]);
    repo.git(&["branch", "-m", "main", "trunk"]);
    let (d, after) = diff(backend, &repo, &before);
    let all: Vec<&str> = d
        .changes
        .iter()
        .flat_map(|c| c.refs.iter().map(|r| r.name.as_str()))
        .collect();
    for want in [
        "refs/heads/old",
        "refs/tags/release",
        "refs/heads/main",
        "refs/heads/trunk",
    ] {
        assert!(all.contains(&want), "{want} missing from {all:?}");
    }
    assert_eq!(after.head, HeadState::Branch("trunk".into()));
}
