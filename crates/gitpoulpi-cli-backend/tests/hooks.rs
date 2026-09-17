//! The resolved `git` binary (on Windows, `mingw64\bin\git.exe` instead of the
//! `cmd\git.exe` shim) must still run shell hooks, which need `sh` and the
//! `usr\bin` coreutils on `PATH`.

use gitpoulpi_cli_backend::SystemGitBackend;
use gitpoulpi_core::domain::{CommitOptions, StageTarget};
use gitpoulpi_core::GitBackend;
use gitpoulpi_testkit::FixtureRepoBuilder;

fn install_hook(root: &std::path::Path, body: &str) {
    let hook = root.join(".git").join("hooks").join("pre-commit");
    std::fs::write(&hook, body).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
}

fn staged_new_file() -> (gitpoulpi_testkit::FixtureRepo, SystemGitBackend) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    std::fs::write(repo.root().join("new.txt"), "hello\n").unwrap();
    (repo, SystemGitBackend::new())
}

#[test]
fn pre_commit_shell_hook_runs_through_the_system_backend() {
    let (repo, backend) = staged_new_file();
    // `uname` and `touch` come from `usr\bin` on Windows: a hook only gets
    // them if the child's PATH is right.
    install_hook(
        repo.root(),
        "#!/bin/sh\nuname > hook-ran.txt\ntouch hook-touched\n",
    );
    let repo_ref = repo.repo_ref();
    backend
        .stage(&repo_ref, &StageTarget::File("new.txt".into()))
        .unwrap();

    backend
        .commit(&repo_ref, "with hook", &CommitOptions::default())
        .unwrap();

    assert!(
        repo.root().join("hook-touched").exists(),
        "hook did not run"
    );
    let uname = std::fs::read_to_string(repo.root().join("hook-ran.txt")).unwrap();
    assert!(!uname.trim().is_empty());
}

#[test]
fn failing_pre_commit_hook_blocks_the_commit() {
    let (repo, backend) = staged_new_file();
    install_hook(repo.root(), "#!/bin/sh\nexit 1\n");
    let repo_ref = repo.repo_ref();
    backend
        .stage(&repo_ref, &StageTarget::File("new.txt".into()))
        .unwrap();

    assert!(backend
        .commit(&repo_ref, "blocked", &CommitOptions::default())
        .is_err());
}
