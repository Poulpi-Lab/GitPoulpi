//! Renamed and moved files: `status` and `diff` pair a deletion with an
//! addition of similar content (git's 50% default) into one rename - staged
//! (git pairs them), unstaged (a deleted tracked file plus an untracked one)
//! and in a commit - and say where it came from.

use super::head_oid;
use crate::TestRepoBuilder;
use gitpoulpi_core::domain::{DiffTarget, FileStatus, RepoRef, StatusEntry};
use gitpoulpi_core::GitBackend;

fn body() -> String {
    (1..=12).map(|n| format!("line {n}\n")).collect()
}

fn unrelated() -> String {
    (1..=12).map(|n| format!("other {n} text\n")).collect()
}

fn entry<'a>(files: &'a [StatusEntry], path: &str) -> Option<&'a StatusEntry> {
    files.iter().find(|f| f.path.to_str() == Some(path))
}

fn status(backend: &dyn GitBackend, repo: &RepoRef) -> Vec<StatusEntry> {
    backend.status(repo).unwrap().files
}

/// A rename in the index: one entry at the new path, from the old one.
pub fn check_staged_rename_is_one_entry_and_one_diff(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("old.txt", &body(), "add")
        .build();
    let repo_ref = repo.repo_ref();
    repo.git(&["mv", "old.txt", "moved.txt"]);

    let files = status(backend, &repo_ref);
    assert_eq!(files.len(), 1, "{files:?}");
    let moved = entry(&files, "moved.txt").expect("the new path is listed");
    assert_eq!(moved.index, Some(FileStatus::Renamed));
    assert_eq!(moved.renamed_from.as_deref(), Some("old.txt".as_ref()));

    let diffs = backend.diff(&repo_ref, DiffTarget::IndexVsHead).unwrap();
    assert_eq!(diffs.len(), 1, "{diffs:?}");
    assert_eq!(diffs[0].path.to_str(), Some("moved.txt"));
    assert_eq!(diffs[0].status, FileStatus::Renamed);
    assert_eq!(diffs[0].renamed_from.as_deref(), Some("old.txt".as_ref()));
    assert_eq!(diffs[0].similarity, Some(100));
    assert!(diffs[0].hunks.is_empty(), "a pure rename has no content");
}

/// A rename plus edits, staged: the diff is the edit, against the old file.
pub fn check_staged_rename_with_edits_diffs_against_the_old_file(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("old.txt", &body(), "add")
        .build();
    let repo_ref = repo.repo_ref();
    repo.git(&["mv", "old.txt", "moved.txt"]);
    std::fs::write(
        repo.root().join("moved.txt"),
        body().replace("line 6\n", "line six\n"),
    )
    .unwrap();
    repo.git(&["add", "moved.txt"]);

    let diffs = backend.diff(&repo_ref, DiffTarget::IndexVsHead).unwrap();
    assert_eq!(diffs.len(), 1, "{diffs:?}");
    assert_eq!(diffs[0].status, FileStatus::Renamed);
    assert_eq!(diffs[0].renamed_from.as_deref(), Some("old.txt".as_ref()));
    let similarity = diffs[0]
        .similarity
        .expect("a rename carries its similarity");
    assert!((50..100).contains(&similarity), "{similarity}");
    let changed: Vec<&str> = diffs[0]
        .hunks
        .iter()
        .flat_map(|h| h.lines.iter())
        .filter(|l| l.origin != gitpoulpi_core::domain::LineOrigin::Context)
        .map(|l| l.content.as_str())
        .collect();
    assert_eq!(changed, vec!["line 6", "line six"], "only the edit");
}

/// A move on disk, not staged: the deleted file and the untracked one are
/// one entry at the new path. The deleted file stays listed as it was.
pub fn check_unstaged_rename_pairs_the_deleted_and_the_untracked_file(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("old.txt", &body(), "add")
        .commit_file("keep.txt", "keep\n", "keep")
        .build();
    let repo_ref = repo.repo_ref();
    std::fs::create_dir_all(repo.root().join("sub")).unwrap();
    std::fs::rename(
        repo.root().join("old.txt"),
        repo.root().join("sub/moved.txt"),
    )
    .unwrap();
    // An untracked file with nothing in common stays alone.
    std::fs::write(repo.root().join("alone.txt"), unrelated()).unwrap();

    let files = status(backend, &repo_ref);
    let moved = entry(&files, "sub/moved.txt").expect("the new path is listed");
    assert_eq!(moved.index, None);
    assert_eq!(moved.worktree, Some(FileStatus::Untracked));
    assert_eq!(moved.renamed_from.as_deref(), Some("old.txt".as_ref()));
    let old = entry(&files, "old.txt").expect("the old path keeps its row");
    assert_eq!(old.worktree, Some(FileStatus::Deleted));
    assert_eq!(old.renamed_from, None);
    let alone = entry(&files, "alone.txt").unwrap();
    assert_eq!(alone.renamed_from, None);
}

/// The same with edits to the moved file, still above the threshold.
pub fn check_unstaged_rename_with_edits_still_pairs(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("old.txt", &body(), "add")
        .build();
    let repo_ref = repo.repo_ref();
    std::fs::remove_file(repo.root().join("old.txt")).unwrap();
    std::fs::write(
        repo.root().join("moved.txt"),
        body().replace("line 3\n", "line three\n"),
    )
    .unwrap();

    let files = status(backend, &repo_ref);
    let moved = entry(&files, "moved.txt").unwrap();
    assert_eq!(moved.renamed_from.as_deref(), Some("old.txt".as_ref()));
}

/// Below the threshold a "move" is a deletion plus a new file, staged or not.
pub fn check_dissimilar_files_are_not_a_rename(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("old.txt", &body(), "add")
        .build();
    let repo_ref = repo.repo_ref();
    std::fs::remove_file(repo.root().join("old.txt")).unwrap();
    std::fs::write(repo.root().join("new.txt"), unrelated()).unwrap();

    let files = status(backend, &repo_ref);
    assert_eq!(files.len(), 2, "{files:?}");
    assert!(files.iter().all(|f| f.renamed_from.is_none()), "{files:?}");

    repo.git(&["add", "-A"]);
    let files = status(backend, &repo_ref);
    assert_eq!(files.len(), 2, "{files:?}");
    assert!(files.iter().all(|f| f.renamed_from.is_none()), "{files:?}");
    let diffs = backend.diff(&repo_ref, DiffTarget::IndexVsHead).unwrap();
    let statuses: Vec<(String, FileStatus)> = diffs
        .iter()
        .map(|d| (d.path.to_string_lossy().into_owned(), d.status))
        .collect();
    assert_eq!(
        statuses,
        vec![
            ("new.txt".to_string(), FileStatus::Added),
            ("old.txt".to_string(), FileStatus::Deleted),
        ]
    );
    assert!(diffs.iter().all(|d| d.renamed_from.is_none()));
}

/// A commit that moves a file shows it as a rename, and `diff_files` lists
/// the same.
pub fn check_commit_diff_shows_a_rename(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("old.txt", &body(), "add")
        .build();
    let repo_ref = repo.repo_ref();
    repo.git(&["mv", "old.txt", "moved.txt"]);
    std::fs::write(
        repo.root().join("moved.txt"),
        body().replace("line 9\n", "line nine\n"),
    )
    .unwrap();
    repo.git(&["add", "moved.txt"]);
    repo.git(&["commit", "-m", "move"]);
    let head = head_oid(backend, &repo_ref);

    let diffs = backend
        .diff(&repo_ref, DiffTarget::CommitVsParent(head))
        .unwrap();
    assert_eq!(diffs.len(), 1, "{diffs:?}");
    assert_eq!(diffs[0].path.to_str(), Some("moved.txt"));
    assert_eq!(diffs[0].status, FileStatus::Renamed);
    assert_eq!(diffs[0].renamed_from.as_deref(), Some("old.txt".as_ref()));
    assert!(diffs[0].similarity.is_some_and(|s| s >= 50));
    assert_eq!(diffs[0].hunks.len(), 1);

    let listed = backend
        .diff_files(&repo_ref, DiffTarget::CommitVsParent(head))
        .unwrap();
    assert_eq!(listed.files.len(), 1);
    assert_eq!(listed.files[0].path, diffs[0].path);
    assert_eq!(listed.files[0].status, FileStatus::Renamed);
    assert_eq!(listed.files[0].renamed_from, diffs[0].renamed_from);
}
