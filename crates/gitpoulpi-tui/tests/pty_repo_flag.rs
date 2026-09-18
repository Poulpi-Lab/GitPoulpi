//! `--repo <path>` on a TUI launch opens that repository, not the current
//! directory's; a path that is not a repository opens Home with a message.
//! Launched from a folder that is no repository at all.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;

const STEP: Duration = Duration::from_secs(30);

#[test]
fn repo_flag_opens_that_repository_from_anywhere() {
    let fixture = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "flagged-repo-commit")
        .unwrap()
        .build();
    let elsewhere = tempfile::tempdir().unwrap();
    let (term, _config) = common::launch(elsewhere.path())
        .session("pty-repo-flag")
        .arg("--repo")
        .arg(fixture.root())
        .no_exit_pause()
        .ready("flagged-repo-commit")
        .start();
    drop(term);
}

#[test]
fn repo_flag_with_a_non_repository_opens_home_with_a_message() {
    let elsewhere = tempfile::tempdir().unwrap();
    let not_a_repo = tempfile::tempdir().unwrap();
    let (mut term, _config) = common::launch(elsewhere.path())
        .session("pty-repo-flag-bad")
        .arg("--repo")
        .arg(not_a_repo.path())
        .no_exit_pause()
        .ready("not a git repository")
        .start();
    // Home, not a crash: the process is alive and the keyboard works.
    assert!(!term.exited());
    term.send_keys("C-p").unwrap();
    term.wait_for("the palette", STEP, |s| {
        s.contains("Command") || s.contains("Rebase")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}
