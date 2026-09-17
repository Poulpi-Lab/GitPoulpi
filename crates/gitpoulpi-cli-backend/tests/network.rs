use gitpoulpi_cli_backend::SystemGitBackend;
use gitpoulpi_core::domain::{
    CloneOptions, FetchOptions, PullOptions, PullOutcome, PushOptions, StageTarget,
};
use gitpoulpi_core::GitBackend;
use gitpoulpi_testkit::FixtureRepoBuilder;
use std::process::Command;

fn clone_bare(source_root: &std::path::Path, dest: &std::path::Path) {
    let status = Command::new("git")
        .args([
            "clone",
            "--bare",
            source_root.to_str().unwrap(),
            dest.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());
}

#[tokio::test]
async fn fetch_reports_the_updated_remote_tracking_refs() {
    let origin = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let bare_dir = tempfile::tempdir().unwrap();
    let bare_path = bare_dir.path().join("origin.git");
    clone_bare(origin.root(), &bare_path);

    let clone_dir = tempfile::tempdir().unwrap();
    let clone_path = clone_dir.path().join("clone");
    let backend = SystemGitBackend::new();
    let repo_ref = backend
        .clone(
            bare_path.to_str().unwrap(),
            &clone_path,
            &CloneOptions::default(),
        )
        .await
        .unwrap();

    // A new commit lands on the bare "origin" after the clone, so `fetch`
    // has something new to report.
    std::fs::write(origin.root().join("second.txt"), "second").unwrap();
    Command::new("git")
        .args(["add", "second.txt"])
        .current_dir(origin.root())
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "second"])
        .current_dir(origin.root())
        .status()
        .unwrap();
    Command::new("git")
        .args(["push", bare_path.to_str().unwrap(), "main"])
        .current_dir(origin.root())
        .status()
        .unwrap();

    let outcome = backend
        .fetch(&repo_ref, "origin", &FetchOptions::default())
        .await
        .unwrap();

    assert!(!outcome.updated_refs.is_empty());
}

#[tokio::test]
async fn pull_fast_forwards_from_the_remote() {
    let origin = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let bare_dir = tempfile::tempdir().unwrap();
    let bare_path = bare_dir.path().join("origin.git");
    clone_bare(origin.root(), &bare_path);

    let clone_dir = tempfile::tempdir().unwrap();
    let clone_path = clone_dir.path().join("clone");
    let backend = SystemGitBackend::new();
    let repo_ref = backend
        .clone(
            bare_path.to_str().unwrap(),
            &clone_path,
            &CloneOptions::default(),
        )
        .await
        .unwrap();

    std::fs::write(origin.root().join("second.txt"), "second").unwrap();
    Command::new("git")
        .args(["add", "second.txt"])
        .current_dir(origin.root())
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "second"])
        .current_dir(origin.root())
        .status()
        .unwrap();
    Command::new("git")
        .args(["push", bare_path.to_str().unwrap(), "main"])
        .current_dir(origin.root())
        .status()
        .unwrap();

    let outcome = backend
        .pull(&repo_ref, &PullOptions::default())
        .await
        .unwrap();

    assert!(matches!(outcome, PullOutcome::FastForwarded(_)));
    assert!(clone_path.join("second.txt").exists());
}

#[tokio::test]
async fn push_uploads_the_local_commit_to_the_remote() {
    let origin = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let bare_dir = tempfile::tempdir().unwrap();
    let bare_path = bare_dir.path().join("origin.git");
    clone_bare(origin.root(), &bare_path);

    let clone_dir = tempfile::tempdir().unwrap();
    let clone_path = clone_dir.path().join("clone");
    let backend = SystemGitBackend::new();
    let repo_ref = backend
        .clone(
            bare_path.to_str().unwrap(),
            &clone_path,
            &CloneOptions::default(),
        )
        .await
        .unwrap();
    // A fresh clone has no local git identity, and CI runners have no
    // global one configured either (unlike a dev machine) -- set it
    // locally so `commit()` below doesn't fail on "please tell me who
    // you are".
    Command::new("git")
        .args(["config", "user.name", "GitPoulpi Testkit"])
        .current_dir(&clone_path)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "testkit@gitpoulpi.local"])
        .current_dir(&clone_path)
        .status()
        .unwrap();

    std::fs::write(clone_path.join("local.txt"), "local").unwrap();
    backend
        .stage(&repo_ref, &StageTarget::File("local.txt".into()))
        .unwrap();
    backend
        .commit(
            &repo_ref,
            "local commit",
            &gitpoulpi_core::domain::CommitOptions::default(),
        )
        .unwrap();

    let outcome = backend
        .push(&repo_ref, "origin", &PushOptions::default())
        .await
        .unwrap();

    assert_eq!(outcome.pushed_refs, vec!["main".to_string()]);

    let verify_dir = tempfile::tempdir().unwrap();
    let verify_path = verify_dir.path().join("verify");
    clone_bare(&bare_path, &verify_path);
    // bare clone of a bare repo just to confirm the ref moved -- check
    // for the pushed commit via `git log` on the bare repo directly.
    let log = Command::new("git")
        .args(["log", "--oneline", "main"])
        .current_dir(&bare_path)
        .output()
        .unwrap();
    let log = String::from_utf8_lossy(&log.stdout);
    assert!(log.contains("local commit"));
}

#[test]
fn remote_add_and_remove_round_trip() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let backend = SystemGitBackend::new();
    let repo_ref = repo.repo_ref();

    backend
        .remote_add(&repo_ref, "upstream", "https://example.invalid/repo.git")
        .unwrap();
    let remotes = backend.remotes(&repo_ref).unwrap();
    assert!(remotes.iter().any(|r| r.name == "upstream"));

    backend.remote_remove(&repo_ref, "upstream").unwrap();
    let remotes = backend.remotes(&repo_ref).unwrap();
    assert!(!remotes.iter().any(|r| r.name == "upstream"));
}
