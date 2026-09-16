//! Dropping the future of a network operation is how a job is canceled
//! (the background-ops design`, B3): git and every helper it
//! started must die, and what a hard kill leaves behind must go.
//!
//! The "in flight" state is a [`SlowRemote`] that accepts the connection and
//! says nothing; the proof that git is dead is the server seeing the
//! connection close.

use std::future::Future;
use std::path::Path;
use std::time::Duration;

use super::block_on;
use crate::{FixtureRepo, FixtureRepoBuilder, SlowRemote};
use gitpoulpi_core::domain::{CloneOptions, FetchOptions, PushOptions};
use gitpoulpi_core::GitBackend;

const WAIT: Duration = Duration::from_secs(20);

/// Polls `started` while `operation` runs, then drops `operation`
/// (cancel). Panics if the operation ends on its own or `started` never
/// comes true.
fn cancel_when<T>(operation: impl Future<Output = T>, started: impl Fn() -> bool, what: &str) {
    block_on(async {
        let waiting = async {
            let deadline = std::time::Instant::now() + WAIT;
            while !started() {
                assert!(std::time::Instant::now() < deadline, "{what} never started");
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        };
        tokio::select! {
            _ = operation => panic!("{what} ended before it was canceled"),
            () = waiting => {}
        }
        // `operation` is dropped here: that is the cancel.
    });
}

fn repo_with_slow_origin(slow: &SlowRemote) -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .with_origin()
        .unwrap()
        .build();
    repo.git(&["remote", "set-url", "origin", &slow.url()]);
    repo
}

fn git_dir(repo: &FixtureRepo) -> std::path::PathBuf {
    repo.root().join(".git")
}

/// A canceled fetch: git is dead (the connection closed), a lock file git
/// would have left *since the fetch started* is removed, and one that
/// existed before it is not touched.
pub fn check_a_canceled_fetch_kills_git_and_removes_only_its_own_locks(backend: &dyn GitBackend) {
    let slow = SlowRemote::start();
    let repo = repo_with_slow_origin(&slow);
    let repo_ref = repo.repo_ref();
    let refs = git_dir(&repo).join("refs").join("remotes").join("origin");
    std::fs::create_dir_all(&refs).unwrap();
    let before = refs.join("older.lock");
    std::fs::write(&before, b"held by someone else").unwrap();
    // The fetch's own start must be strictly after this file's mtime.
    std::thread::sleep(Duration::from_millis(100));

    let ours = refs.join("main.lock");
    cancel_when(
        backend.fetch(&repo_ref, "origin", &FetchOptions::default()),
        || {
            // git is connected: now it "takes" a ref lock, as it would while
            // updating refs.
            let connected = slow.accepted() >= 1;
            if connected && !ours.exists() {
                std::fs::write(&ours, b"git was here").unwrap();
            }
            connected
        },
        "the fetch",
    );

    assert!(
        slow.wait_closed(1, WAIT),
        "git must be dead after a canceled fetch (its connection is still open)"
    );
    assert!(!ours.exists(), "the lock created during the fetch must go");
    assert!(before.exists(), "a lock older than the fetch is not ours");
}

/// A canceled push kills git too.
pub fn check_a_canceled_push_kills_git(backend: &dyn GitBackend) {
    let slow = SlowRemote::start();
    let repo = repo_with_slow_origin(&slow);
    let repo_ref = repo.repo_ref();
    cancel_when(
        backend.push(&repo_ref, "origin", &PushOptions::default()),
        || slow.accepted() >= 1,
        "the push",
    );
    assert!(
        slow.wait_closed(1, WAIT),
        "git must be dead after a canceled push"
    );
}

/// The helpers git started die with it, not only git: the `uploadpack` shell
/// of a fetch (a grandchild) must not live to write its marker.
pub fn check_a_canceled_fetch_kills_the_helpers_git_started(backend: &dyn GitBackend) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .with_origin()
        .unwrap()
        .build();
    let repo_ref = repo.repo_ref();
    let marks = tempfile::tempdir().unwrap();
    let path = |name: &str| marks.path().join(name).to_string_lossy().replace('\\', "/");
    let (started, survivor) = (path("started"), path("survivor"));
    // git runs this through the shell in place of `git-upload-pack`: a
    // subshell sleeping two seconds and then writing the marker, which only a
    // surviving process tree can do.
    repo.git(&[
        "config",
        "remote.origin.uploadpack",
        &format!(": > '{started}'; ( sleep 2; : > '{survivor}' ) & wait; git-upload-pack"),
    ]);

    cancel_when(
        backend.fetch(&repo_ref, "origin", &FetchOptions::default()),
        || Path::new(&started).exists(),
        "the fetch's upload-pack helper",
    );
    std::thread::sleep(Duration::from_millis(3500));
    assert!(
        !Path::new(&survivor).exists(),
        "a helper started by the canceled git was still running"
    );
}

/// A canceled clone: git is dead and the destination it was creating is gone.
pub fn check_a_canceled_clone_removes_the_destination_it_created(backend: &dyn GitBackend) {
    let slow = SlowRemote::start();
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("clone");
    cancel_when(
        backend.clone(&slow.url(), &dest, &CloneOptions::default()),
        || slow.accepted() >= 1,
        "the clone",
    );
    assert!(
        slow.wait_closed(1, WAIT),
        "git must be dead after a canceled clone"
    );
    assert!(
        !dest.exists(),
        "the destination the clone created must be removed"
    );
}

/// A destination that existed empty is emptied, not deleted; one that held
/// something is never touched.
pub fn check_a_canceled_clone_keeps_a_destination_that_existed(backend: &dyn GitBackend) {
    let slow = SlowRemote::start();
    let dir = tempfile::tempdir().unwrap();
    let empty = dir.path().join("empty");
    std::fs::create_dir(&empty).unwrap();
    cancel_when(
        backend.clone(&slow.url(), &empty, &CloneOptions::default()),
        || slow.accepted() >= 1,
        "the clone into an empty folder",
    );
    assert!(slow.wait_closed(1, WAIT), "git must be dead");
    assert!(empty.is_dir(), "the empty folder existed before: keep it");
    assert_eq!(std::fs::read_dir(&empty).unwrap().count(), 0);

    let full = dir.path().join("full");
    std::fs::create_dir(&full).unwrap();
    std::fs::write(full.join("mine.txt"), b"keep me").unwrap();
    let _ = block_on(async {
        tokio::time::timeout(
            Duration::from_millis(300),
            backend.clone(&slow.url(), &full, &CloneOptions::default()),
        )
        .await
    });
    assert_eq!(
        std::fs::read(full.join("mine.txt")).unwrap(),
        b"keep me",
        "a folder that had content is never touched"
    );
}

/// A pull dropped while it downloads (the job was canceled before the point of
/// no return): git is dead and `HEAD`, the index, the working tree (dirty
/// files included), the stash and the merge state are exactly as they were.
pub fn check_a_pull_canceled_during_its_download_changes_nothing(backend: &dyn GitBackend) {
    use gitpoulpi_core::domain::PullOptions;
    let slow = SlowRemote::start();
    let repo = repo_with_slow_origin(&slow);
    repo.git(&["config", "branch.main.remote", "origin"]);
    repo.git(&["config", "branch.main.merge", "refs/heads/main"]);
    std::fs::write(repo.root().join("initial.txt"), "edited\n").unwrap();
    std::fs::write(repo.root().join("staged.txt"), "staged\n").unwrap();
    repo.git(&["add", "staged.txt"]);
    let state = || {
        format!(
            "{}|{}|{}|{}|{}",
            repo.git(&["rev-parse", "HEAD"]).trim(),
            repo.git(&["ls-files", "-s"]),
            repo.git(&["status", "--porcelain"]),
            repo.git(&["stash", "list"]),
            git_dir(&repo).join("MERGE_HEAD").exists(),
        )
    };
    let before = state();
    let repo_ref = repo.repo_ref();
    let opts = PullOptions::default();
    cancel_when(
        backend.pull_fetch(&repo_ref, &opts),
        || slow.accepted() >= 1,
        "the pull's download",
    );
    assert!(
        slow.wait_closed(1, WAIT),
        "git must be dead after a canceled pull download"
    );
    assert_eq!(state(), before, "a canceled pull changed the checkout");
    assert!(!git_dir(&repo).join("index.lock").exists());
}
