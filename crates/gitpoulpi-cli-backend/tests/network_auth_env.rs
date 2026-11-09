//! What the contract suite cannot check because it must not touch the process
//! environment: a repository's `NetworkAuth` ssh command beats a
//! `GIT_SSH_COMMAND` the user exported, and the process-wide `ssh.private_key_path`
//! setting does not apply to a repository that has an entry (the entry applies
//! instead of it, ADR-0010). Everything runs in one test: the environment and
//! the key are process-wide.
#![cfg(unix)]

use gitpoulpi_cli_backend::{set_ssh_key, SystemGitBackend};
use gitpoulpi_core::domain::{FetchOptions, NetworkAuth};
use gitpoulpi_core::GitBackend;
use gitpoulpi_testkit::{FixtureRepoBuilder, StubSsh};

#[tokio::test]
async fn an_entrys_ssh_command_beats_the_environment_and_the_global_key() {
    let exported = StubSsh::new();
    let chosen = StubSsh::new();
    std::env::set_var("GIT_SSH_COMMAND", exported.command());
    std::env::set_var("GIT_TERMINAL_PROMPT", "0");
    std::env::remove_var("GIT_SSH");

    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .with_origin()
        .unwrap()
        .build();
    repo.git(&["push", "origin", "main"]);
    let bare = repo.remote_path().unwrap().to_path_buf();
    repo.git(&["remote", "set-url", "origin", &StubSsh::url_for(&bare)]);
    let backend = SystemGitBackend::new();
    let fetch = || async {
        backend
            .fetch(&repo.repo_ref(), "origin", &FetchOptions::default())
            .await
            .expect("the stub ssh reaches the bare repository");
    };

    // No entry: the user's own GIT_SSH_COMMAND is what runs.
    fetch().await;
    assert!(!exported.calls().is_empty());
    assert!(chosen.calls().is_empty());

    // An entry: its command wins over the environment, and the global key
    // (which only fills a gap when nothing was chosen) is not added to it.
    set_ssh_key(Some("/keys/global".to_string()));
    let seen = exported.calls().len();
    backend.set_network_auth(
        &repo.repo_ref(),
        Some(NetworkAuth {
            ssh_command: Some(chosen.command()),
            ..NetworkAuth::default()
        }),
    );
    fetch().await;
    assert!(
        !chosen.calls().is_empty(),
        "the entry's command did not run"
    );
    assert_eq!(exported.calls().len(), seen, "GIT_SSH_COMMAND still ran");
    assert!(
        chosen.identity_files().is_empty(),
        "the global key leaked into the entry's command: {:?}",
        chosen.calls()
    );

    // No entry again: back to the environment.
    backend.set_network_auth(&repo.repo_ref(), None);
    set_ssh_key(None);
    fetch().await;
    assert!(exported.calls().len() > seen);
    std::env::remove_var("GIT_SSH_COMMAND");
}
