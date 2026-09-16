//! Comparing arbitrary revisions: the targets behind the Log's multi-select
//! (two commits, a commit and the working tree, a stash and a commit, a run
//! of commits as one).

use crate::TestRepoBuilder;
use gitpoulpi_core::domain::{DiffTarget, FileDiff, FileStatus, LineOrigin};
use gitpoulpi_core::GitBackend;

fn paths_and_statuses(files: &[FileDiff]) -> Vec<(String, FileStatus)> {
    files
        .iter()
        .map(|f| (f.path.to_string_lossy().replace('\\', "/"), f.status))
        .collect()
}

/// Two commits go older to newer (`CommitRange`), swapping them swaps the
/// statuses, a rename is a delete plus an add on both backends (the way
/// the commit file list always showed it), and a commit against itself is
/// empty. `diff_files` agrees with `diff` on every target.
pub fn check_compare_two_commits_both_ways(backend: &dyn GitBackend) {
    use FileStatus::{Added, Deleted, Modified};
    let repo = TestRepoBuilder::new()
        .commit_file("a.txt", "1\n2\n3\n", "c1")
        .commit_file("b.txt", "b\n", "c1b")
        .commit_file("a.txt", "1\nTWO\n3\n", "c2")
        .commit_file("c.txt", "c\n", "c2b")
        .build();
    repo.git(&["mv", "b.txt", "moved.txt"]);
    repo.git(&["commit", "-q", "-m", "c3 rename"]);
    let repo_ref = repo.repo_ref();
    let at = |rev: &str| backend.resolve(&repo_ref, rev).unwrap();
    let (c1, c3) = (at("HEAD~3"), at("HEAD"));
    let diff = |target: DiffTarget| {
        let full = backend.diff(&repo_ref, target.clone()).unwrap();
        let listed = backend.diff_files(&repo_ref, target.clone()).unwrap();
        assert_eq!(
            paths_and_statuses(&listed.files),
            paths_and_statuses(&full),
            "{target:?}: the list is the diff's"
        );
        full
    };

    let forward = diff(DiffTarget::CommitRange(c1, c3));
    assert_eq!(
        paths_and_statuses(&forward),
        vec![
            ("a.txt".into(), Modified),
            ("b.txt".into(), Deleted),
            ("c.txt".into(), Added),
            ("moved.txt".into(), Added),
        ]
    );
    let backward = diff(DiffTarget::CommitRange(c3, c1));
    assert_eq!(
        paths_and_statuses(&backward),
        vec![
            ("a.txt".into(), Modified),
            ("b.txt".into(), Added),
            ("c.txt".into(), Deleted),
            ("moved.txt".into(), Deleted),
        ]
    );
    assert!(
        diff(DiffTarget::CommitRange(c3, c3)).is_empty(),
        "a commit against itself changes nothing"
    );
}

/// A run of commits as one: from the first parent of the oldest to the
/// newest; a root commit as the oldest starts from the empty tree.
pub fn check_compare_a_span_starts_at_the_oldests_parent(backend: &dyn GitBackend) {
    use FileStatus::Added;
    let repo = TestRepoBuilder::new()
        .commit_file("a.txt", "1\n2\n3\n", "c1")
        .commit_file("a.txt", "1\nTWO\n3\n", "c2")
        .commit_file("c.txt", "c\n", "c3")
        .build();
    let repo_ref = repo.repo_ref();
    let at = |rev: &str| backend.resolve(&repo_ref, rev).unwrap();
    let (c1, c2, c3) = (at("HEAD~2"), at("HEAD~1"), at("HEAD"));
    let diff = |target: DiffTarget| backend.diff(&repo_ref, target).unwrap();

    let span = diff(DiffTarget::CommitSpan(c2, c3));
    assert_eq!(
        paths_and_statuses(&span),
        paths_and_statuses(&diff(DiffTarget::CommitRange(c1, c3))),
        "span = the oldest's parent .. the newest"
    );
    assert!(
        span.iter().any(|f| f.path.to_str() == Some("a.txt")),
        "the oldest commit's own edit is inside the span"
    );
    let from_root = diff(DiffTarget::CommitSpan(c1, c3));
    assert_eq!(
        paths_and_statuses(&from_root),
        vec![("a.txt".into(), Added), ("c.txt".into(), Added)],
        "a root commit's span is everything, added"
    );
}

/// A commit against the working tree: unstaged and staged edits of tracked
/// files count, an untracked file is not part of any diff, and
/// `diff_paths` restricts it like every other target.
pub fn check_compare_a_commit_with_the_working_tree(backend: &dyn GitBackend) {
    use FileStatus::{Added, Deleted, Modified};
    let repo = TestRepoBuilder::new()
        .commit_file("a.txt", "1\n2\n3\n", "c1")
        .commit_file("gone.txt", "g\n", "c2")
        .commit_removal("gone.txt", "c3")
        .build();
    let repo_ref = repo.repo_ref();
    let c2 = backend.resolve(&repo_ref, "HEAD~1").unwrap();
    let head = backend.resolve(&repo_ref, "HEAD").unwrap();
    std::fs::write(repo.root().join("a.txt"), "1\n2\n3\nfour\n").unwrap();
    std::fs::write(repo.root().join("staged.txt"), "s\n").unwrap();
    repo.git(&["add", "staged.txt"]);
    std::fs::write(repo.root().join("untracked.txt"), "u\n").unwrap();

    let diff = |target: DiffTarget| {
        let full = backend.diff(&repo_ref, target.clone()).unwrap();
        let listed = backend.diff_files(&repo_ref, target.clone()).unwrap();
        assert_eq!(
            paths_and_statuses(&listed.files),
            paths_and_statuses(&full),
            "{target:?}"
        );
        full
    };
    let wip = diff(DiffTarget::CommitVsWorkingTree(head));
    assert_eq!(
        paths_and_statuses(&wip),
        vec![("a.txt".into(), Modified), ("staged.txt".into(), Added)],
        "the unstaged edit and the staged add; the untracked file is no diff"
    );
    let added: Vec<&str> = wip[0]
        .hunks
        .iter()
        .flat_map(|h| &h.lines)
        .filter(|l| l.origin == LineOrigin::Add)
        .map(|l| l.content.as_str())
        .collect();
    assert_eq!(added, vec!["four"]);

    let older = diff(DiffTarget::CommitVsWorkingTree(c2));
    assert_eq!(
        paths_and_statuses(&older),
        vec![
            ("a.txt".into(), Modified),
            ("gone.txt".into(), Deleted),
            ("staged.txt".into(), Added),
        ],
        "against c2, which still had gone.txt, the tree has it deleted"
    );
}

/// A stash is a commit whose tree is the working tree it saved, so a stash
/// against a commit is `CommitRange` again, in either direction.
pub fn check_compare_a_stash_with_a_commit(backend: &dyn GitBackend) {
    use FileStatus::{Added, Modified};
    let repo = TestRepoBuilder::new()
        .commit_file("a.txt", "1\n2\n3\n", "c1")
        .commit_file("b.txt", "b\n", "c2")
        .build();
    let repo_ref = repo.repo_ref();
    let c1 = backend.resolve(&repo_ref, "HEAD~1").unwrap();
    let c2 = backend.resolve(&repo_ref, "HEAD").unwrap();
    std::fs::write(repo.root().join("a.txt"), "1\n2\n3\nfour\n").unwrap();
    backend.stash_push(&repo_ref, Some("wip"), false).unwrap();
    let stash = backend.stashes(&repo_ref).unwrap()[0].commit.unwrap();

    let diff = |from, to| {
        backend
            .diff(&repo_ref, DiffTarget::CommitRange(from, to))
            .unwrap()
    };
    assert_eq!(
        paths_and_statuses(&diff(c2, stash)),
        vec![("a.txt".into(), Modified)]
    );
    assert_eq!(
        paths_and_statuses(&diff(c1, stash)),
        vec![("a.txt".into(), Modified), ("b.txt".into(), Added)]
    );
    assert_eq!(
        paths_and_statuses(&diff(stash, c1)),
        vec![
            ("a.txt".into(), Modified),
            ("b.txt".into(), FileStatus::Deleted)
        ]
    );
}
