//! The reflog read, the lost-commit scan and raw blob reads
//! (the undo-safety-net design). Ground truth always comes from
//! plain `git` run in the fixture, never from the backend under test.

use crate::TestRepoBuilder;
use gitpoulpi_core::domain::{
    CommitOptions, LostCommitOptions, LostSource, Oid, ResetMode, StageTarget, ZERO_OID,
};
use gitpoulpi_core::GitBackend;

fn oid(hex: &str) -> Oid {
    Oid::from_hex(hex.trim()).unwrap()
}

/// `git log -g`'s view of a reflog: `(new oid, reason)`, newest first.
fn truth(repo: &crate::FixtureRepo, refname: &str) -> Vec<(Oid, String)> {
    repo.git(&["log", "-g", "--format=%H%x09%gs", refname])
        .lines()
        .filter_map(|l| l.split_once('\t'))
        .map(|(h, s)| (oid(h), s.to_string()))
        .collect()
}

/// Moves made through the backend and moves made with plain `git` outside
/// show up alike, newest first, with git's own reason; `old` chains.
pub fn check_reflog_shows_moves_made_inside_and_outside(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    std::fs::write(repo.root().join("a.txt"), "a").unwrap();
    backend
        .stage(&repo_ref, &StageTarget::File("a.txt".into()))
        .unwrap();
    let second = backend
        .commit(&repo_ref, "second", &CommitOptions::default())
        .unwrap();
    // Outside: a commit, an amend and a reset in a shell.
    repo.git(&["commit", "--allow-empty", "-m", "third"]);
    repo.git(&["commit", "--allow-empty", "--amend", "-m", "third, amended"]);
    repo.git(&["reset", "--soft", &second.to_hex()]);

    for name in ["HEAD", "main", "refs/heads/main"] {
        let page = backend.reflog(&repo_ref, name, 0, 100).unwrap();
        let want = truth(&repo, name);
        assert_eq!(page.entries.len(), want.len(), "{name}");
        assert!(!page.more, "{name}");
        for (entry, (new, reason)) in page.entries.iter().zip(&want) {
            assert_eq!(&entry.new, new, "{name}: {entry:?}");
            assert_eq!(&entry.message, reason, "{name}");
            assert!(entry.time > 1_000_000_000, "{name}: {entry:?}");
            assert!(entry.committer.contains('<'), "{name}: {entry:?}");
        }
        // Each entry's `old` is where the previous (older) entry left the ref.
        for pair in page.entries.windows(2) {
            assert_eq!(pair[0].old, pair[1].new, "{name}: the chain is unbroken");
        }
        let oldest = page.entries.last().unwrap();
        assert_eq!(
            oldest.old, ZERO_OID,
            "{name}: the first move has no old value"
        );
        assert_eq!(
            page.entries[0].new, second,
            "{name}: the reset is the newest"
        );
        assert!(page.entries[0].message.starts_with("reset: moving to"));
    }
    let amend = backend.reflog(&repo_ref, "main", 1, 1).unwrap();
    assert!(
        amend.entries[0].message.starts_with("commit (amend)"),
        "{:?}",
        amend.entries
    );
}

/// `skip` and `limit` page the reflog, `more` says whether there is a next
/// page, and a window past the end is empty.
pub fn check_reflog_pages(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    for n in 0..5 {
        repo.git(&["commit", "--allow-empty", "-m", &format!("c{n}")]);
    }
    let all = backend.reflog(&repo_ref, "HEAD", 0, 100).unwrap();
    assert_eq!(all.entries.len(), 6);
    let first = backend.reflog(&repo_ref, "HEAD", 0, 4).unwrap();
    assert_eq!(first.entries, all.entries[..4]);
    assert!(first.more);
    let rest = backend.reflog(&repo_ref, "HEAD", 4, 4).unwrap();
    assert_eq!(rest.entries, all.entries[4..]);
    assert!(!rest.more);
    let past = backend.reflog(&repo_ref, "HEAD", 10, 4).unwrap();
    assert!(past.entries.is_empty() && !past.more);
    let exact = backend.reflog(&repo_ref, "HEAD", 0, 6).unwrap();
    assert!(
        !exact.more,
        "a page that ends at the last entry has no more"
    );
}

/// A branch that was deleted took its reflog with it: an empty page, not an
/// error. A tag, and a branch that never existed, have none either.
pub fn check_reflog_of_a_deleted_or_unknown_ref_is_empty(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    repo.git(&["branch", "feature"]);
    repo.git(&["tag", "v1"]);
    assert_eq!(
        backend
            .reflog(&repo_ref, "feature", 0, 10)
            .unwrap()
            .entries
            .len(),
        1,
        "the branch has a creation entry while it exists"
    );
    repo.git(&["branch", "-D", "feature"]);
    for name in ["feature", "refs/heads/feature", "refs/tags/v1", "nope"] {
        let page = backend.reflog(&repo_ref, name, 0, 10).unwrap();
        assert!(page.entries.is_empty() && !page.more, "{name}: {page:?}");
    }
}

/// A detached HEAD and a reset through the backend are in the HEAD reflog.
pub fn check_reflog_records_backend_resets_and_checkouts(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .commit("second")
        .build();
    let repo_ref = repo.repo_ref();
    let tip = oid(&repo.git(&["rev-parse", "HEAD"]));
    let first = oid(&repo.git(&["rev-parse", "HEAD~1"]));
    backend.reset(&repo_ref, first, ResetMode::Hard).unwrap();
    let page = backend.reflog(&repo_ref, "HEAD", 0, 3).unwrap();
    assert_eq!(page.entries[0].new, first);
    assert_eq!(page.entries[0].old, tip);
    let head = backend.reflog(&repo_ref, "", 0, 1).unwrap();
    assert_eq!(head.refname, "HEAD");
    assert_eq!(head.entries[0].new, first);
}

fn lost(
    backend: &dyn GitBackend,
    repo: &crate::FixtureRepo,
    deep: bool,
) -> Vec<(String, LostSource)> {
    backend
        .lost_commits(
            &repo.repo_ref(),
            &LostCommitOptions {
                deep,
                ..LostCommitOptions::default()
            },
        )
        .unwrap()
        .commits
        .into_iter()
        .map(|c| (c.summary, c.source))
        .collect()
}

/// A repository where nothing was lost lists nothing, even with the
/// undo pins (trees and blobs) around.
pub fn check_lost_commits_none_in_a_clean_repo(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .with_initial()
        .commit("second")
        .build();
    repo.git(&["branch", "side"]);
    repo.git(&["tag", "t"]);
    repo.git(&["update-ref", "refs/gitpoulpi/undo/1/0", "HEAD^{tree}"]);
    for deep in [false, true] {
        let page = backend
            .lost_commits(
                &repo.repo_ref(),
                &LostCommitOptions {
                    deep,
                    ..LostCommitOptions::default()
                },
            )
            .unwrap();
        assert!(
            page.commits.is_empty() && !page.more && !page.partial,
            "{page:?}"
        );
    }
}

/// Deleting a branch leaves its commits reachable only from a reflog: they
/// are listed newest first with summary, author and date, are not the
/// commits that are still on a branch, and page.
pub fn check_lost_commits_after_a_branch_delete(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    repo.git(&["checkout", "-q", "-b", "feature"]);
    repo.git(&["commit", "--allow-empty", "-m", "feature one"]);
    repo.git(&["commit", "--allow-empty", "-m", "feature two"]);
    let tip = oid(&repo.git(&["rev-parse", "HEAD"]));
    repo.git(&["checkout", "-q", "main"]);
    repo.git(&["branch", "-D", "feature"]);

    for deep in [false, true] {
        let got = lost(backend, &repo, deep);
        assert_eq!(
            got,
            vec![
                ("feature two".to_string(), LostSource::Reflog),
                ("feature one".to_string(), LostSource::Reflog)
            ],
            "deep={deep}"
        );
    }
    let page = backend
        .lost_commits(
            &repo.repo_ref(),
            &LostCommitOptions {
                limit: 1,
                ..LostCommitOptions::default()
            },
        )
        .unwrap();
    assert_eq!(page.commits.len(), 1);
    assert!(page.more);
    let c = &page.commits[0];
    assert_eq!(c.oid, tip);
    assert_eq!(c.summary, "feature two");
    assert!(!c.author.is_empty());
    assert!(c.time > 1_000_000_000);
    assert_eq!(c.parents.len(), 1);
    let second = backend
        .lost_commits(
            &repo.repo_ref(),
            &LostCommitOptions {
                skip: 1,
                limit: 1,
                ..LostCommitOptions::default()
            },
        )
        .unwrap();
    assert_eq!(second.commits[0].summary, "feature one");
    assert!(!second.more);
    // Bringing a branch back makes them found again.
    repo.git(&["branch", "feature", &tip.to_hex()]);
    assert!(lost(backend, &repo, true).is_empty());
}

/// Once the reflogs are gone, a commit only an undo pin keeps is `Pin`, and
/// one nothing names is dangling and found by the deep scan only.
pub fn check_lost_commits_pinned_and_dangling(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    repo.git(&["checkout", "-q", "-b", "feature"]);
    repo.git(&["commit", "--allow-empty", "-m", "pinned"]);
    let pinned = repo.git(&["rev-parse", "HEAD"]);
    repo.git(&["commit", "--allow-empty", "-m", "dangling"]);
    repo.git(&["checkout", "-q", "main"]);
    repo.git(&["branch", "-D", "feature"]);
    repo.git(&["update-ref", "refs/gitpoulpi/undo/3/0", pinned.trim()]);
    repo.git(&["reflog", "expire", "--expire=now", "--all"]);

    // The pin keeps "pinned" (and so its parent chain); the commit above it
    // is named by nothing at all.
    let shallow = lost(backend, &repo, false);
    assert_eq!(shallow, vec![("pinned".to_string(), LostSource::Pin)]);
    let deep = lost(backend, &repo, true);
    assert_eq!(
        deep,
        vec![
            ("dangling".to_string(), LostSource::Dangling),
            ("pinned".to_string(), LostSource::Pin)
        ]
    );
}

/// The scan stops at its time budget instead of running on, and says so.
/// (A budget of 1 ms cannot finish a deep scan of a real object database;
/// whichever step it stops in, the answer is `partial` or complete, never
/// an error.)
pub fn check_lost_commits_respects_a_tiny_budget(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    repo.git(&["checkout", "-q", "-b", "feature"]);
    repo.git(&["commit", "--allow-empty", "-m", "x"]);
    repo.git(&["checkout", "-q", "main"]);
    repo.git(&["branch", "-D", "feature"]);
    let page = backend
        .lost_commits(
            &repo.repo_ref(),
            &LostCommitOptions {
                budget_ms: 1,
                ..LostCommitOptions::default()
            },
        )
        .unwrap();
    assert!(page.partial || page.commits.len() == 1, "{page:?}");
}

/// A blob's raw bytes by id, binary and CRLF included; a missing id is an
/// error.
pub fn check_blob_bytes_reads_a_blob_by_id(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new().with_initial().build();
    let repo_ref = repo.repo_ref();
    let bytes: Vec<u8> = vec![0, 1, 2, 13, 10, 255, b'x'];
    std::fs::write(repo.root().join("blob.bin"), &bytes).unwrap();
    let id = oid(&repo.git(&["hash-object", "-w", "blob.bin"]));
    assert_eq!(backend.blob_bytes(&repo_ref, id).unwrap(), bytes);
    assert!(backend.blob_bytes(&repo_ref, Oid([7; 20])).is_err());
}

/// A reftable repository (git 2.45+) has no `logs/` files and no loose refs:
/// its reflog and the lost-commit scan still work, identically on both
/// backends. Skips, with a note, when the installed git cannot make one.
pub fn check_reflog_and_lost_commits_in_a_reftable_repository(backend: &dyn GitBackend) {
    let Some(builder) = TestRepoBuilder::reftable() else {
        return;
    };
    let repo = builder.with_initial().build();
    let repo_ref = repo.repo_ref();
    repo.git(&["commit", "--allow-empty", "-m", "second"]);
    repo.git(&["checkout", "-q", "-b", "feature"]);
    repo.git(&["commit", "--allow-empty", "-m", "feature one"]);
    let tip = oid(&repo.git(&["rev-parse", "HEAD"]));
    repo.git(&["reset", "-q", "--hard", "HEAD~1"]);

    for name in ["HEAD", "feature", "refs/heads/feature"] {
        let page = backend.reflog(&repo_ref, name, 0, 100).unwrap();
        let want = truth(&repo, name);
        assert!(!want.is_empty(), "{name}: the fixture has a reflog");
        assert_eq!(page.entries.len(), want.len(), "{name}: {page:?}");
        for (entry, (new, reason)) in page.entries.iter().zip(&want) {
            assert_eq!(&entry.new, new, "{name}: {entry:?}");
            assert_eq!(&entry.message, reason, "{name}");
            assert!(entry.time > 1_000_000_000, "{name}: {entry:?}");
            assert!(entry.committer.contains('<'), "{name}: {entry:?}");
        }
        for pair in page.entries.windows(2) {
            assert_eq!(pair[0].old, pair[1].new, "{name}: the chain is unbroken");
        }
        assert_eq!(page.entries.last().unwrap().old, ZERO_OID, "{name}");
        assert!(!page.more, "{name}");
    }
    let first = backend.reflog(&repo_ref, "HEAD", 0, 2).unwrap();
    assert_eq!(first.entries.len(), 2);
    assert!(first.more);
    let rest = backend.reflog(&repo_ref, "HEAD", 2, 100).unwrap();
    assert!(!rest.more);
    assert_eq!(first.entries[1].old, rest.entries[0].new);
    let past = backend.reflog(&repo_ref, "HEAD", 50, 5).unwrap();
    assert!(past.entries.is_empty() && !past.more);

    // A gone branch: an empty page, and its commits are found as lost.
    repo.git(&["checkout", "-q", "main"]);
    repo.git(&["branch", "-D", "feature"]);
    assert!(backend
        .reflog(&repo_ref, "feature", 0, 10)
        .unwrap()
        .entries
        .is_empty());
    for deep in [false, true] {
        let got = lost(backend, &repo, deep);
        assert_eq!(
            got,
            vec![("feature one".to_string(), LostSource::Reflog)],
            "deep={deep}"
        );
    }
    let page = backend
        .lost_commits(&repo_ref, &LostCommitOptions::default())
        .unwrap();
    assert_eq!(page.commits[0].oid, tip);
}

// -- restoring a lost commit's changes -----------------------------------

/// Bytes of `path` as `commit` holds them.
fn blob_of(repo: &crate::FixtureRepo, commit: Oid, path: &str) -> Vec<u8> {
    let out = std::process::Command::new("git")
        .args(["cat-file", "blob", &format!("{}:{path}", commit.to_hex())])
        .current_dir(repo.root())
        .output()
        .unwrap();
    assert!(out.status.success(), "cat-file {path}");
    out.stdout
}

/// A repo whose `main` tip `lost` modified a file, added one, deleted one
/// and changed a binary one, then moved back to `base`: `lost` is only in
/// the reflog, as a lost commit is.
fn lost_change_set() -> (crate::FixtureRepo, Oid, Oid) {
    let repo = TestRepoBuilder::new()
        .commit_file("keep.txt", "one\ntwo\nthree\n", "base one")
        .commit_file("gone.txt", "bye\n", "base two")
        .commit_bytes("bin.dat", &[0, 1, 2, 3, 0, 255], "base bin")
        .build();
    let base = oid(&repo.git(&["rev-parse", "HEAD"]));
    std::fs::write(repo.root().join("keep.txt"), "one\n2\nthree\nfour\n").unwrap();
    std::fs::write(repo.root().join("new.txt"), "new file\n").unwrap();
    std::fs::write(repo.root().join("bin.dat"), [9u8, 0, 0, 1, 200, 7, 0]).unwrap();
    repo.git(&["rm", "-q", "gone.txt"]);
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-q", "-m", "the lost work"]);
    let lost = oid(&repo.git(&["rev-parse", "HEAD"]));
    repo.git(&["reset", "-q", "--hard", &base.to_hex()]);
    (repo, base, lost)
}

/// `apply_commit_to_worktree` replays one commit's changes (modify, add,
/// delete, binary) into the working tree only: the index and HEAD stay.
pub fn check_apply_commit_to_worktree_changes_the_worktree_only(backend: &dyn GitBackend) {
    let (repo, base, lost) = lost_change_set();
    let repo_ref = repo.repo_ref();
    let applied = backend.apply_commit_to_worktree(&repo_ref, lost).unwrap();
    let mut files = applied.files.clone();
    files.sort();
    assert_eq!(files, ["bin.dat", "gone.txt", "keep.txt", "new.txt"]);

    for path in ["keep.txt", "new.txt", "bin.dat"] {
        assert_eq!(
            std::fs::read(repo.root().join(path)).unwrap(),
            blob_of(&repo, lost, path),
            "{path}"
        );
    }
    assert!(!repo.root().join("gone.txt").exists());
    assert_eq!(oid(&repo.git(&["rev-parse", "HEAD"])), base, "HEAD stays");
    assert_eq!(
        repo.git(&["diff", "--cached", "--name-only"]).trim(),
        "",
        "the index stays"
    );
}

/// A root commit applies against the empty tree.
pub fn check_apply_commit_to_worktree_of_a_root_commit(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("a.txt", "root a\n", "root")
        .build();
    let repo_ref = repo.repo_ref();
    let root = oid(&repo.git(&["rev-parse", "HEAD"]));
    repo.git(&["checkout", "-q", "--orphan", "elsewhere"]);
    repo.git(&["rm", "-rfq", "."]);
    assert!(!repo.root().join("a.txt").exists());

    let applied = backend.apply_commit_to_worktree(&repo_ref, root).unwrap();
    assert_eq!(applied.files, ["a.txt"]);
    assert_eq!(
        std::fs::read(repo.root().join("a.txt")).unwrap(),
        b"root a\n"
    );
}

/// A merge commit has no single set of changes: a typed refusal, nothing
/// touched.
pub fn check_apply_commit_to_worktree_refuses_a_merge_commit(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("a.txt", "a\n", "base")
        .branch("side")
        .commit_file("main.txt", "m\n", "on main")
        .checkout("side")
        .commit_file("side.txt", "s\n", "on side")
        .checkout("main")
        .build();
    let repo_ref = repo.repo_ref();
    repo.git(&["merge", "-q", "--no-ff", "-m", "merge side", "side"]);
    let merge = oid(&repo.git(&["rev-parse", "HEAD"]));
    repo.git(&["reset", "-q", "--hard", "HEAD~1"]);

    match backend.apply_commit_to_worktree(&repo_ref, merge) {
        Err(gitpoulpi_core::Error::ApplyRefused(
            gitpoulpi_core::domain::ApplyRefusal::MergeCommit,
        )) => {}
        other => panic!("expected the merge refusal, got {other:?}"),
    }
    assert!(!repo.root().join("side.txt").exists());
    assert_eq!(repo.git(&["status", "--porcelain"]).trim(), "");
}

/// All or nothing: when one file of the commit does not apply, the files
/// that would have applied are not touched either.
pub fn check_apply_commit_to_worktree_that_does_not_apply_changes_nothing(
    backend: &dyn GitBackend,
) {
    let (repo, _base, lost) = lost_change_set();
    let repo_ref = repo.repo_ref();
    // keep.txt diverges so its hunk no longer matches; the others would apply.
    std::fs::write(repo.root().join("keep.txt"), "other\ncontent\n").unwrap();
    let before = repo.git(&["status", "--porcelain"]);

    assert!(backend.apply_commit_to_worktree(&repo_ref, lost).is_err());

    assert_eq!(
        std::fs::read(repo.root().join("keep.txt")).unwrap(),
        b"other\ncontent\n"
    );
    assert!(!repo.root().join("new.txt").exists(), "nothing was written");
    assert!(repo.root().join("gone.txt").exists(), "nothing was removed");
    assert_eq!(repo.git(&["status", "--porcelain"]), before);
    assert!(backend
        .apply_commit_to_worktree(&repo_ref, Oid::from_hex(&"1".repeat(40)).unwrap())
        .is_err());
}

/// Binary bytes and CRLF text come out byte for byte, with the user's
/// line-ending conversion off and on (Git for Windows defaults to on).
pub fn check_apply_commit_to_worktree_keeps_binary_and_crlf_bytes(backend: &dyn GitBackend) {
    for autocrlf in ["false", "true"] {
        let repo = TestRepoBuilder::new()
            .commit_bytes("win.txt", b"a\r\nb\r\nc\r\n", "base crlf")
            .commit_bytes("img.bin", &[0, 13, 10, 0, 1], "base bin")
            .build();
        let repo_ref = repo.repo_ref();
        let base = oid(&repo.git(&["rev-parse", "HEAD"]));
        std::fs::write(repo.root().join("win.txt"), b"a\r\nB\r\nc\r\nd\r\n").unwrap();
        std::fs::write(repo.root().join("img.bin"), [0u8, 13, 10, 0, 2, 3]).unwrap();
        repo.git(&["add", "-A"]);
        repo.git(&["commit", "-q", "-m", "edit"]);
        let lost = oid(&repo.git(&["rev-parse", "HEAD"]));
        repo.git(&["reset", "-q", "--hard", &base.to_hex()]);
        repo.git(&["config", "core.autocrlf", autocrlf]);

        backend.apply_commit_to_worktree(&repo_ref, lost).unwrap();
        assert_eq!(
            std::fs::read(repo.root().join("img.bin")).unwrap(),
            [0u8, 13, 10, 0, 2, 3],
            "autocrlf={autocrlf}"
        );
        // Whatever git's own conversion made of the text, adding it back
        // gives exactly the commit's tree.
        repo.git(&["add", "-A"]);
        assert_eq!(
            repo.git(&["diff", "--cached", "--stat", &lost.to_hex()])
                .trim(),
            "",
            "autocrlf={autocrlf}"
        );
    }
}

/// `stash_from_commit` builds a stash holding exactly the commit's changes:
/// the same patch as the commit, applicable by the ordinary stash apply, on
/// top of the stash list, with a given or a default message.
pub fn check_stash_from_commit_builds_a_stash_of_the_commits_changes(backend: &dyn GitBackend) {
    let (repo, base, lost) = lost_change_set();
    let repo_ref = repo.repo_ref();
    let want_patch = repo.git(&["diff", &format!("{}^", lost.to_hex()), &lost.to_hex()]);

    let stash_commit = backend
        .stash_from_commit(&repo_ref, lost, Some("rescued"))
        .unwrap();
    let list = backend.stashes(&repo_ref).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, gitpoulpi_core::domain::StashId(0));
    assert_eq!(list[0].commit, Some(stash_commit));
    assert!(list[0].message.contains("rescued"), "{}", list[0].message);
    assert_eq!(oid(&repo.git(&["rev-parse", "HEAD"])), base, "HEAD stays");
    assert_eq!(
        repo.git(&["status", "--porcelain"]).trim(),
        "",
        "tree stays"
    );

    assert_eq!(
        repo.git(&["stash", "show", "-p", "stash@{0}"]),
        want_patch,
        "the stash shows the commit's own change"
    );
    backend
        .stash_apply(&repo_ref, gitpoulpi_core::domain::StashId(0))
        .unwrap();
    for path in ["keep.txt", "bin.dat"] {
        assert_eq!(
            std::fs::read(repo.root().join(path)).unwrap(),
            blob_of(&repo, lost, path),
            "{path}"
        );
    }
    assert!(!repo.root().join("gone.txt").exists());

    // A default message names the commit; a second stash goes on top.
    backend.stash_from_commit(&repo_ref, lost, None).unwrap();
    let list = backend.stashes(&repo_ref).unwrap();
    assert_eq!(list.len(), 2);
    assert!(
        list[0].message.contains(&lost.to_hex()[..7]),
        "{}",
        list[0].message
    );
    assert!(backend
        .stash_from_commit(&repo_ref, Oid::from_hex(&"2".repeat(40)).unwrap(), None)
        .is_err());
    assert_eq!(backend.stashes(&repo_ref).unwrap().len(), 2);
}

/// A root commit stashes against the empty tree, and a merge commit
/// against its first parent.
pub fn check_stash_from_commit_handles_root_and_merge_commits(backend: &dyn GitBackend) {
    let repo = TestRepoBuilder::new()
        .commit_file("a.txt", "a\n", "root")
        .branch("side")
        .commit_file("main.txt", "m\n", "on main")
        .checkout("side")
        .commit_file("side.txt", "s\n", "on side")
        .checkout("main")
        .build();
    let repo_ref = repo.repo_ref();
    let root = oid(&repo.git(&["rev-list", "--max-parents=0", "HEAD"]));
    repo.git(&["merge", "-q", "--no-ff", "-m", "merge side", "side"]);
    let merge = oid(&repo.git(&["rev-parse", "HEAD"]));

    backend.stash_from_commit(&repo_ref, root, None).unwrap();
    assert_eq!(
        repo.git(&["stash", "show", "--name-only", "stash@{0}"])
            .trim(),
        "a.txt"
    );
    backend.stash_from_commit(&repo_ref, merge, None).unwrap();
    assert_eq!(
        repo.git(&["stash", "show", "--name-only", "stash@{0}"])
            .trim(),
        repo.git(&[
            "diff",
            "--name-only",
            &format!("{}^1", merge.to_hex()),
            &merge.to_hex()
        ])
        .trim()
    );
    repo.git(&["stash", "drop", "-q"]);
    repo.git(&["stash", "drop", "-q"]);
    assert!(backend.stashes(&repo_ref).unwrap().is_empty());
}
