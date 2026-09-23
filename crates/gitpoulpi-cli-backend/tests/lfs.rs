//! LFS behavior that is specific to `SystemGitBackend`'s plumbing (the
//! shared behavior is in the contract suite).

#![cfg(feature = "lfs")]

use std::sync::{Arc, Mutex};

use gitpoulpi_cli_backend::SystemGitBackend;
use gitpoulpi_core::domain::{LfsDirection, LfsProgress};
use gitpoulpi_core::GitBackend;
use gitpoulpi_testkit::lfs::{hermetic_git, LfsLockServer};
use gitpoulpi_testkit::{contract, require_git_lfs, FixtureRepoBuilder};

#[test]
fn lfs_mutations_without_the_program_are_typed_unavailable() {
    contract::assert_lfs_mutations_without_binary_are_typed_unavailable(
        &SystemGitBackend::without_lfs(),
    );
}

fn collecting() -> (
    Arc<Mutex<Vec<LfsProgress>>>,
    impl Fn(LfsProgress) + Send + Sync,
) {
    let seen: Arc<Mutex<Vec<LfsProgress>>> = Arc::default();
    let writer = Arc::clone(&seen);
    (seen, move |p| writer.lock().unwrap().push(p))
}

/// A real HTTP transfer (git-lfs's built-in `file://` adapter reports no
/// progress at all, so only this exercises the `GIT_LFS_PROGRESS` parser
/// against git-lfs's own output): the pushed and pulled object's progress
/// arrives in intermediate reports carrying the real size, then a final
/// `done` one. Also proves `lfs_pull` installs the filters when a
/// repository has none, or the checkout step would be skipped.
#[tokio::test]
async fn a_real_http_transfer_reports_intermediate_progress_with_the_real_size() {
    require_git_lfs!("a_real_http_transfer_reports_intermediate_progress_with_the_real_size");
    let server = LfsLockServer::start();
    let payload = vec![7u8; 3 * 1024 * 1024];
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_origin()
        .unwrap()
        .with_lfs()
        .unwrap()
        .lfs_track("*.bin")
        .unwrap()
        .commit_lfs_file("big.bin", &payload, "add big")
        .unwrap()
        .config("lfs.url", &server.url())
        .unwrap()
        .config("credential.helper", "")
        .unwrap()
        .build();
    let backend = SystemGitBackend::new();

    let (seen, sink) = collecting();
    backend
        .lfs_push(&repo.repo_ref(), "origin", &Default::default(), &sink)
        .await
        .unwrap();
    assert_eq!(server.stored_objects().len(), 1);
    {
        let seen = seen.lock().unwrap();
        let intermediate: Vec<_> = seen.iter().filter(|p| !p.done).collect();
        assert!(
            intermediate
                .iter()
                .any(|p| p.direction == LfsDirection::Upload
                    && p.bytes_total == payload.len() as u64
                    && p.bytes_done > 0),
            "no upload progress line was parsed: {seen:?}"
        );
        assert!(seen.last().unwrap().done);
    }

    // The other direction: a clone that holds pointers pulls the object.
    repo.git(&["push", "origin", "main"]);
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("clone");
    gitpoulpi_testkit::lfs::clone_leaving_pointers(repo.remote_path().unwrap(), &dest);
    hermetic_git(&dest, &["config", "lfs.url", &server.url()]);
    hermetic_git(&dest, &["config", "credential.helper", ""]);
    let clone = gitpoulpi_core::domain::RepoRef::from_root(&dest);

    let (seen, sink) = collecting();
    backend.lfs_pull(&clone, "origin", &sink).await.unwrap();
    assert_eq!(std::fs::read(dest.join("big.bin")).unwrap(), payload);
    let seen = seen.lock().unwrap();
    assert!(
        seen.iter().any(|p| !p.done
            && p.direction == LfsDirection::Download
            && p.bytes_total == payload.len() as u64),
        "no download progress line was parsed: {seen:?}"
    );
    assert!(seen.last().unwrap().done);
}

#[test]
fn a_clone_without_the_program_succeeds_and_keeps_pointers() {
    contract::assert_clone_without_lfs_program_succeeds_and_keeps_pointers(
        &SystemGitBackend::without_lfs(),
    );
}
