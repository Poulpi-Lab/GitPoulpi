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
