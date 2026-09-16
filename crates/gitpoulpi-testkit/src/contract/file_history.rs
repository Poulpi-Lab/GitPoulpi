//! `file_history` (a file's commits, following renames) and the rename and
//! "previous revision" parts of `blame`.

use super::head_oid;
use crate::TestRepoBuilder;
use gitpoulpi_core::domain::{BlameContent, FileChangeKind, FileHistoryEntry, Oid};
use gitpoulpi_core::GitBackend;
use std::path::{Path, PathBuf};

fn commit_all(repo: &crate::FixtureRepo, message: &str) -> Oid {
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-q", "-m", message]);
    Oid::from_hex(&repo.git(&["rev-parse", "HEAD"])).unwrap()
}

fn numbered(n: u32) -> String {
    (1..=n).map(|i| format!("line {i}\n")).collect()
}

fn history(
    backend: &dyn GitBackend,
    repo: &crate::FixtureRepo,
    path: &str,
    skip: usize,
    limit: usize,
) -> gitpoulpi_core::domain::FileHistoryPage {
    let repo_ref = repo.repo_ref();
    let head = head_oid(backend, &repo_ref);
    backend
        .file_history(&repo_ref, Path::new(path), head, skip, limit)
        .unwrap()
}

fn summaries(entries: &[FileHistoryEntry]) -> Vec<&str> {
    entries.iter().map(|e| e.summary.as_str()).collect()
}

/// Plain edits: newest first, with the commit's identity, the change kind
/// and per-file line counts.
pub fn check_file_history_lists_a_files_commits_with_stats(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("f.txt", "a\nb\nc\n", "add f")
        .commit_file("other.txt", "unrelated\n", "touch other")
        .commit_file("f.txt", "a\nB\nc\nd\n", "edit f")
        .build();
    let page = history(backend, &repo, "f.txt", 0, 10);

    assert!(!page.more);
    assert_eq!(summaries(&page.entries), ["edit f", "add f"]);
    let edit = &page.entries[0];
    let add = &page.entries[1];
    assert_eq!(edit.change, FileChangeKind::Modified);
    assert_eq!((edit.added, edit.removed), (Some(2), Some(1)));
    assert_eq!(add.change, FileChangeKind::Added);
    assert_eq!((add.added, add.removed), (Some(3), Some(0)));
    assert_eq!(edit.path, PathBuf::from("f.txt"));
    assert_eq!(
        edit.parents.len(),
        1,
        "the parent is the unrelated commit between"
    );
    assert!(add.parents.is_empty());
    assert_eq!(edit.author_name, "GitPoulpi Testkit");
    assert_eq!(edit.author_email, "testkit@gitpoulpi.local");
    assert!(edit.timestamp > 0 && edit.committer_timestamp > 0);

    let none = history(backend, &repo, "never-existed.txt", 0, 10);
    assert!(none.entries.is_empty() && !none.more);
}

/// `git mv` plus an edit in one commit is a rename, and history carries on
/// under the old name: the path column differs across the rename.
pub fn check_file_history_follows_a_rename_with_changes(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("old.txt", &numbered(20), "add old")
        .commit_file("old.txt", &format!("{}extra\n", numbered(20)), "grow old")
        .build();
    repo.git(&["mv", "old.txt", "new.txt"]);
    std::fs::write(
        repo.root().join("new.txt"),
        format!("{}more\n", numbered(20)),
    )
    .unwrap();
    commit_all(&repo, "rename and edit");
    std::fs::write(repo.root().join("new.txt"), numbered(8)).unwrap();
    commit_all(&repo, "shrink new");

    let page = history(backend, &repo, "new.txt", 0, 10);
    assert_eq!(
        summaries(&page.entries),
        ["shrink new", "rename and edit", "grow old", "add old"]
    );
    assert_eq!(page.entries[0].path, PathBuf::from("new.txt"));
    assert_eq!(page.entries[0].change, FileChangeKind::Modified);
    assert_eq!(
        page.entries[1].change,
        FileChangeKind::Renamed {
            from: PathBuf::from("old.txt")
        }
    );
    assert_eq!(page.entries[1].path, PathBuf::from("new.txt"));
    assert_eq!(
        (page.entries[1].added, page.entries[1].removed),
        (Some(1), Some(1))
    );
    assert_eq!(page.entries[2].path, PathBuf::from("old.txt"));
    assert_eq!(page.entries[3].path, PathBuf::from("old.txt"));
    assert_eq!(page.entries[3].change, FileChangeKind::Added);

    // A page that starts after the rename still follows it (git's own
    // `--skip` would not: it skips before diffing, so the rename is missed).
    let tail = history(backend, &repo, "new.txt", 2, 5);
    assert_eq!(summaries(&tail.entries), ["grow old", "add old"]);
    assert!(!tail.more);

    // Asking by the old name from the old commit finds the old history.
    let repo_ref = repo.repo_ref();
    let at_grow = page.entries[2].commit;
    let old = backend
        .file_history(&repo_ref, Path::new("old.txt"), at_grow, 0, 10)
        .unwrap();
    assert_eq!(summaries(&old.entries), ["grow old", "add old"]);
}

/// Deleting and later re-adding a path: the deletion is an entry, the
/// re-adding commit is `Added`, and history continues before both.
pub fn check_file_history_across_delete_and_readd(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("f.txt", &numbered(12), "first life")
        .commit_removal("f.txt", "delete f")
        .commit_file("g.txt", "filler\n", "filler")
        .commit_file("f.txt", &numbered(3), "second life")
        .build();
    let page = history(backend, &repo, "f.txt", 0, 10);
    assert_eq!(
        summaries(&page.entries),
        ["second life", "delete f", "first life"]
    );
    assert_eq!(page.entries[0].change, FileChangeKind::Added);
    assert_eq!(page.entries[1].change, FileChangeKind::Deleted);
    assert_eq!(page.entries[1].path, PathBuf::from("f.txt"));
    assert_eq!(
        (page.entries[1].added, page.entries[1].removed),
        (Some(0), Some(12))
    );
    assert_eq!(page.entries[2].change, FileChangeKind::Added);
}

/// Merges: default simplified history, like `git log --follow`. A merge
/// that leaves the file identical to a parent is not an entry; one that
/// combines both sides' edits is, with its numbers against the first
/// parent, and the side branch's own commit is listed too.
pub fn check_file_history_of_merges_follows_git_log(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("f.txt", &numbered(30), "base")
        .build();
    repo.git(&["checkout", "-q", "-b", "side"]);
    std::fs::write(
        repo.root().join("f.txt"),
        numbered(30).replacen("line 3\n", "side three\n", 1),
    )
    .unwrap();
    let side = commit_all(&repo, "side edit");
    // A side commit that does not touch the file: its merge is TREESAME.
    std::fs::write(repo.root().join("unrelated.txt"), "x\n").unwrap();
    commit_all(&repo, "side unrelated");
    repo.git(&["checkout", "-q", "main"]);
    std::fs::write(
        repo.root().join("f.txt"),
        numbered(30).replacen("line 25\n", "main twenty-five\n", 1),
    )
    .unwrap();
    let main = commit_all(&repo, "main edit");
    repo.git(&["merge", "-q", "--no-ff", "-m", "merge side", "side"]);
    let merge = Oid::from_hex(&repo.git(&["rev-parse", "HEAD"])).unwrap();

    let page = history(backend, &repo, "f.txt", 0, 10);
    let sums = summaries(&page.entries);
    assert_eq!(sums[0], "merge side", "the merge is the newest entry");
    let mut rest = sums[1..].to_vec();
    rest.sort_unstable();
    assert_eq!(rest, ["base", "main edit", "side edit"]);

    let merge_entry = &page.entries[0];
    assert_eq!(merge_entry.commit, merge);
    assert_eq!(merge_entry.parents.len(), 2);
    assert_eq!(merge_entry.parents[0], main);
    assert_eq!(merge_entry.change, FileChangeKind::Modified);
    // Against the first parent (main): the side branch's one-line edit.
    assert_eq!((merge_entry.added, merge_entry.removed), (Some(1), Some(1)));
    assert!(page.entries.iter().any(|e| e.commit == side));

    // A merge that leaves the file as the first parent had it is dropped.
    repo.git(&["checkout", "-q", "-b", "quiet", "main~1"]);
    std::fs::write(repo.root().join("q.txt"), "q\n").unwrap();
    commit_all(&repo, "quiet commit");
    repo.git(&["checkout", "-q", "main"]);
    repo.git(&["merge", "-q", "--no-ff", "-m", "merge quiet", "quiet"]);
    let page = history(backend, &repo, "f.txt", 0, 10);
    assert!(
        !summaries(&page.entries).contains(&"merge quiet"),
        "a TREESAME merge is not part of the file's history"
    );
}

/// `skip` and `limit` page the history; `more` says whether older entries
/// exist; pages concatenate to the whole.
pub fn check_file_history_pages(backend: &dyn GitBackend) {
    let mut builder = TestRepoBuilder::new();
    for i in 1..=7 {
        builder = builder.commit_file("f.txt", &numbered(i), &format!("c{i}"));
    }
    let repo = builder.build();
    let all = history(backend, &repo, "f.txt", 0, 100);
    assert_eq!(all.entries.len(), 7);
    assert!(!all.more);

    let first = history(backend, &repo, "f.txt", 0, 3);
    assert_eq!(summaries(&first.entries), ["c7", "c6", "c5"]);
    assert!(first.more);
    let second = history(backend, &repo, "f.txt", 3, 3);
    assert_eq!(summaries(&second.entries), ["c4", "c3", "c2"]);
    assert!(second.more);
    let last = history(backend, &repo, "f.txt", 6, 3);
    assert_eq!(summaries(&last.entries), ["c1"]);
    assert!(!last.more);
    let exact = history(backend, &repo, "f.txt", 4, 3);
    assert_eq!(summaries(&exact.entries), ["c3", "c2", "c1"]);
    assert!(!exact.more, "exactly the last three is not 'more'");
    let beyond = history(backend, &repo, "f.txt", 20, 3);
    assert!(beyond.entries.is_empty() && !beyond.more);

    let mut joined = first.entries.clone();
    joined.extend(second.entries);
    joined.extend(last.entries);
    assert_eq!(joined, all.entries);
}

/// A binary file has no line counts.
pub fn check_file_history_binary_files_have_no_line_counts(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_bytes("b.bin", b"\x00\x01\x02\x03", "add bin")
        .commit_bytes("b.bin", b"\x00\x01\x02\x03\x04\x05", "change bin")
        .build();
    let page = history(backend, &repo, "b.bin", 0, 10);
    assert_eq!(summaries(&page.entries), ["change bin", "add bin"]);
    for entry in &page.entries {
        assert_eq!((entry.added, entry.removed), (None, None));
    }
}

/// Starting from an older commit answers as of that commit.
pub fn check_file_history_starts_at_the_given_commit(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("f.txt", "1\n", "one")
        .commit_file("f.txt", "2\n", "two")
        .commit_file("f.txt", "3\n", "three")
        .build();
    let repo_ref = repo.repo_ref();
    let two = Oid::from_hex(&repo.git(&["rev-parse", "HEAD~1"])).unwrap();
    let page = backend
        .file_history(&repo_ref, Path::new("f.txt"), two, 0, 10)
        .unwrap();
    assert_eq!(summaries(&page.entries), ["two", "one"]);
}

/// Blame follows a whole-file rename: lines from before it carry the old
/// path, `previous` points at the parent and the path to re-blame, and a
/// line born in its commit has no `previous`.
pub fn check_blame_follows_renames_and_reports_previous(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("old.txt", &numbered(10), "add old")
        .build();
    let add = Oid::from_hex(&repo.git(&["rev-parse", "HEAD"])).unwrap();
    repo.git(&["mv", "old.txt", "new.txt"]);
    let rename = commit_all(&repo, "pure rename");
    std::fs::write(
        repo.root().join("new.txt"),
        numbered(10).replacen("line 5\n", "FIVE\n", 1),
    )
    .unwrap();
    let edit = commit_all(&repo, "edit five");

    let repo_ref = repo.repo_ref();
    let blame = backend
        .blame(&repo_ref, Path::new("new.txt"), edit)
        .unwrap();
    let BlameContent::Lines(lines) = &blame.content else {
        panic!("expected lines");
    };
    assert_eq!(lines.len(), 10);

    // Line 1: born in `add` under the old name; the rename did not claim it.
    assert_eq!(lines[0].commit, add);
    assert_eq!(lines[0].orig_path, PathBuf::from("old.txt"));
    assert_eq!(lines[0].previous, None, "a root commit has no previous");
    assert!(lines[0].committer_time > 0);

    // Line 5: edited at the tip; previous is the rename commit, new name.
    assert_eq!(lines[4].commit, edit);
    assert_eq!(lines[4].orig_path, PathBuf::from("new.txt"));
    assert_eq!(
        lines[4].previous,
        Some((rename, PathBuf::from("new.txt"))),
        "previous names the parent and the path to re-blame there"
    );

    // Re-blaming at `previous` works and attributes line 5 to `add`.
    let again = backend
        .blame(&repo_ref, Path::new("new.txt"), rename)
        .unwrap();
    assert_eq!(again.content.lines()[4].commit, add);

    // A commit that renames *and* edits: its changed line's previous path
    // is the old name.
    repo.git(&["mv", "new.txt", "newer.txt"]);
    std::fs::write(
        repo.root().join("newer.txt"),
        numbered(10)
            .replacen("line 5\n", "FIVE\n", 1)
            .replacen("line 9\n", "NINE\n", 1),
    )
    .unwrap();
    let both = commit_all(&repo, "rename and edit nine");
    let blame = backend
        .blame(&repo_ref, Path::new("newer.txt"), both)
        .unwrap();
    let lines = blame.content.lines();
    assert_eq!(lines[8].commit, both);
    assert_eq!(lines[8].orig_path, PathBuf::from("newer.txt"));
    assert_eq!(
        lines[8].previous,
        Some((edit, PathBuf::from("new.txt"))),
        "previous follows the rename back to the old name"
    );
    assert_eq!(lines[4].commit, edit);
    assert_eq!(lines[4].orig_path, PathBuf::from("new.txt"));
}
