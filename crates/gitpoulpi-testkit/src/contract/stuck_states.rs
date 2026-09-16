//! Checks for the states a repository gets stuck in (plan `stuck-states.md`
//! slice S8): a `git am` stopped on a patch, and a stale `index.lock`.

use std::time::{Duration, SystemTime};

use gitpoulpi_core::domain::{
    AmOutcome, ConflictRefusal, OperationState, RepoRef, STALE_INDEX_LOCK_SECS,
};
use gitpoulpi_core::{Error, GitBackend};

use crate::{FixtureRepo, FixtureRepoBuilder};

/// A repository with a `git am` stopped on a patch: `main` and `feature` both rewrote `a.txt`; `git am` of the feature commit
/// onto `main` stops on it. Two patches when `two` (the second is clean).
pub fn am_fixture(two: bool) -> (FixtureRepo, String) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "base\n", "base")
        .unwrap()
        .branch("feature")
        .unwrap()
        .commit_file("a.txt", "feature\n", "feature change")
        .unwrap()
        .commit_file("b.txt", "second\n", "second patch")
        .unwrap()
        .checkout("main")
        .unwrap()
        .commit_file("a.txt", "main\n", "main change")
        .unwrap()
        .build();
    let dir = repo.root().join(".git").join("test-patches");
    let dir_arg = dir.to_string_lossy().to_string();
    let listed = if two {
        repo.git(&["format-patch", "-o", &dir_arg, "main..feature"])
    } else {
        repo.git(&["format-patch", "-o", &dir_arg, "-1", "feature~1"])
    };
    let files: Vec<String> = listed.lines().map(str::to_string).collect();
    let path = files[0].clone();
    let mut am = vec!["am"];
    am.extend(files.iter().map(String::as_str));
    assert!(repo.try_git(&am).is_none(), "the patch must stop git am");
    (repo, path)
}

fn expect_am(backend: &dyn GitBackend, repo: &RepoRef, step: u32, total: u32) {
    match backend.operation_state(repo).unwrap() {
        Some(OperationState::Am { step: s, total: t }) => assert_eq!((s, t), (step, total)),
        other => panic!("expected an am in progress, got {other:?}"),
    }
}

pub fn check_am_in_progress_is_its_own_operation_state(backend: &dyn GitBackend) {
    let (repo, _) = am_fixture(true);
    let r = repo.repo_ref();
    expect_am(backend, &r, 1, 2);
    // Not a rebase: the rebase read says so and rebase --abort refuses.
    assert_eq!(backend.rebase_state(&r).unwrap(), None);
    assert!(backend.rebase_abort(&r).is_err());
    assert!(repo.root().join(".git/rebase-apply/applying").exists());
}

pub fn check_am_abort_restores_the_branch_tip(backend: &dyn GitBackend) {
    let (repo, _) = am_fixture(true);
    let r = repo.repo_ref();
    let head = repo.git(&["rev-parse", "main"]);
    backend.am_abort(&r).unwrap();
    assert_eq!(backend.operation_state(&r).unwrap(), None);
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), head);
    assert_eq!(repo.git(&["status", "--porcelain"]), "");
}

pub fn check_am_skip_drops_the_stopped_patch_and_goes_on(backend: &dyn GitBackend) {
    let (repo, _) = am_fixture(true);
    let r = repo.repo_ref();
    // Skipping the conflicting patch applies the clean second one.
    assert_eq!(backend.am_skip(&r).unwrap(), AmOutcome::Completed);
    assert_eq!(backend.operation_state(&r).unwrap(), None);
    assert_eq!(repo.git(&["log", "-1", "--format=%s"]), "second patch");
    assert_eq!(repo.git(&["show", "HEAD~1:a.txt"]), "main");
}

pub fn check_am_continue_after_resolving_applies_the_patch(backend: &dyn GitBackend) {
    let (repo, _) = am_fixture(false);
    let r = repo.repo_ref();
    expect_am(backend, &r, 1, 1);
    std::fs::write(repo.root().join("a.txt"), "resolved\n").unwrap();
    repo.git(&["add", "a.txt"]);
    assert_eq!(backend.am_continue(&r).unwrap(), AmOutcome::Completed);
    assert_eq!(backend.operation_state(&r).unwrap(), None);
    assert_eq!(repo.git(&["log", "-1", "--format=%s"]), "feature change");
    assert_eq!(repo.git(&["show", "HEAD:a.txt"]), "resolved");
}

pub fn check_am_without_an_am_is_a_typed_refusal(backend: &dyn GitBackend) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "x\n", "c")
        .unwrap()
        .build();
    let r = repo.repo_ref();
    for result in [
        backend.am_continue(&r).map(|_| ()),
        backend.am_skip(&r).map(|_| ()),
        backend.am_abort(&r),
    ] {
        match result {
            Err(Error::ConflictRefused(ConflictRefusal::NoOperation)) => {}
            other => panic!("expected NoOperation, got {other:?}"),
        }
    }
}

fn age_lock(path: &std::path::Path, secs: u64) {
    let file = std::fs::OpenOptions::new().write(true).open(path).unwrap();
    file.set_modified(SystemTime::now() - Duration::from_secs(secs))
        .unwrap();
}

pub fn check_index_lock_is_detected_when_old_and_removable(backend: &dyn GitBackend) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "x\n", "c")
        .unwrap()
        .build();
    let r = repo.repo_ref();
    assert_eq!(backend.index_lock(&r).unwrap(), None);
    assert!(backend.index_lock_remove(&r).is_err(), "nothing to remove");

    let lock = repo.root().join(".git/index.lock");
    std::fs::write(&lock, "").unwrap();
    // A write fails while it is there: the trap.
    assert!(repo.try_git(&["add", "-A"]).is_none());
    let seen = backend.index_lock(&r).unwrap().expect("the lock is seen");
    assert!(!seen.stale, "a fresh lock may belong to a running git");
    assert!(backend.index_lock_remove(&r).is_err());
    assert!(lock.exists(), "a fresh lock is left alone");

    age_lock(&lock, STALE_INDEX_LOCK_SECS + 50);
    let seen = backend.index_lock(&r).unwrap().expect("the lock is seen");
    assert!(seen.stale && seen.age_secs >= STALE_INDEX_LOCK_SECS);
    backend.index_lock_remove(&r).unwrap();
    assert!(!lock.exists());
    assert_eq!(backend.index_lock(&r).unwrap(), None);
    assert!(repo.try_git(&["add", "-A"]).is_some(), "writes work again");
}

pub fn check_index_lock_of_a_linked_worktree_is_the_worktrees_own(backend: &dyn GitBackend) {
    let (builder, wt) = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "x\n", "c")
        .unwrap()
        .worktree("wt", "side")
        .unwrap();
    let repo = builder.build();
    let main_ref = repo.repo_ref();
    let wt_ref = RepoRef::from_root(&wt);
    let wt_dir = {
        let out = std::process::Command::new("git")
            .args(["rev-parse", "--absolute-git-dir"])
            .current_dir(&wt)
            .output()
            .unwrap();
        std::path::PathBuf::from(String::from_utf8_lossy(&out.stdout).trim())
    };
    let lock = wt_dir.join("index.lock");
    std::fs::write(&lock, "").unwrap();
    age_lock(&lock, STALE_INDEX_LOCK_SECS + 50);
    assert_eq!(backend.index_lock(&main_ref).unwrap(), None);
    let seen = backend.index_lock(&wt_ref).unwrap().expect("seen");
    assert_eq!(seen.path, lock);
    backend.index_lock_remove(&wt_ref).unwrap();
    assert!(!lock.exists());
}
