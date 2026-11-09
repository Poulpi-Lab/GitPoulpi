//! Per-repository network authentication at the git boundary (ADR-0010):
//! what `GitBackend::set_network_auth` changes about the git processes of one
//! repository, and what it leaves alone.
//!
//! Nothing here touches the process environment (the suite runs its checks in
//! parallel): a credential reaches the loopback [`AuthHttpRemote`] through a
//! helper `-c credential.helper=` names, an ssh key reaches a stub `ssh`. The
//! checks that need `GIT_SSH_COMMAND` in the environment live in
//! `gitpoulpi-cli-backend`'s `ssh_key` test.

use std::path::Path;

use super::block_on;
use crate::{AuthHttpRemote, FixtureRepo, FixtureRepoBuilder};
use gitpoulpi_core::domain::{FetchOptions, NetworkAuth};
use gitpoulpi_core::GitBackend;

/// A helper that answers every `get` with `user` and `password`.
fn answering(user: &str, password: &str) -> String {
    format!("!f() {{ echo username={user}; echo password={password}; }}; f")
}

/// A helper that only leaves `marker` behind, to show that it ran.
fn marking(marker: &Path) -> String {
    let marker = marker.to_string_lossy().replace('\\', "/");
    format!("!f() {{ cat >/dev/null; touch '{marker}'; }}; f")
}

fn repo_on(remote_url: &str) -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .with_origin()
        .unwrap()
        .build();
    repo.git(&["remote", "set-url", "origin", remote_url]);
    repo
}

fn fetch(backend: &dyn GitBackend, repo: &FixtureRepo) -> gitpoulpi_core::Result<()> {
    block_on(backend.fetch(&repo.repo_ref(), "origin", &FetchOptions::default())).map(|_| ())
}

fn credentials(user: &str, password: &str) -> NetworkAuth {
    NetworkAuth {
        credential_helpers: Some(vec![answering(user, password)]),
        no_prompt: true,
        reset_extra_header: true,
        ..NetworkAuth::default()
    }
}

/// The credential a helper named by [`NetworkAuth`] gives reaches the server,
/// and the helpers the repository's own configuration names (its config file,
/// a `credential.<url>.helper` entry, an included file) do not run: the reset
/// that precedes ours clears all of them. A header the repository's config
/// would add (`http.extraHeader`) is not sent either.
///
/// The control comes first: with no entry the same three helpers do run, so
/// the markers prove the setup is live rather than the helpers being broken.
pub fn check_a_helpers_credential_reaches_the_remote_and_the_repositorys_own_helpers_do_not_run(
    backend: &dyn GitBackend,
) {
    let server = AuthHttpRemote::start();
    let repo = repo_on(&server.url());
    let dir = tempfile::tempdir().unwrap();
    let (local, scoped, included) = (
        dir.path().join("local.ran"),
        dir.path().join("scoped.ran"),
        dir.path().join("included.ran"),
    );
    repo.git(&["config", "credential.helper", &marking(&local)]);
    repo.git(&[
        "config",
        &format!("credential.http://{}.helper", server.host()),
        &marking(&scoped),
    ]);
    let include = dir.path().join("include.gitconfig");
    std::fs::write(
        &include,
        format!(
            // Quoted: an unquoted `;` starts a comment in a config file.
            "[credential]\n\thelper = \"{}\"\n",
            marking(&included)
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
        ),
    )
    .unwrap();
    repo.git(&[
        "config",
        "include.path",
        &include.to_string_lossy().replace('\\', "/"),
    ]);
    repo.git(&["config", "http.extraHeader", "Authorization: Bearer LEAKED"]);

    // No entry: the repository's helpers are git's business and they run.
    let _ = fetch(backend, &repo);
    for (marker, what) in [
        (&local, "the repository's helper"),
        (&scoped, "the URL-scoped helper"),
        (&included, "the included helper"),
    ] {
        assert!(marker.exists(), "without an entry {what} must run");
        std::fs::remove_file(marker).unwrap();
    }

    // An entry: ours answers, theirs are gone. (The control above sent the
    // repository's own header; only what arrives from here on counts.)
    let before = server.authorizations().len();
    backend.set_network_auth(&repo.repo_ref(), Some(credentials("alice", "pw-one")));
    let _ = fetch(backend, &repo);
    let seen: Vec<String> = server.authorizations().split_off(before);
    let basic: Vec<&String> = seen.iter().filter(|a| a.starts_with("Basic ")).collect();
    assert!(
        !basic.is_empty(),
        "no credential reached the server: {seen:?}"
    );
    assert!(
        seen.iter().all(|a| a == "Basic YWxpY2U6cHctb25l"),
        "the server saw something but alice:pw-one: {seen:?}"
    );
    for (marker, what) in [
        (&local, "the repository's helper"),
        (&scoped, "the URL-scoped helper"),
        (&included, "the included helper"),
    ] {
        assert!(!marker.exists(), "{what} ran although the entry reset it");
    }
}

/// Two repositories with different entries, fetching at the same time (as one
/// daemon serves them), each present only its own credential. This is the
/// reason the entry is keyed by repository and not process-wide.
pub fn check_two_repositories_with_different_auth_do_not_cross(backend: &dyn GitBackend) {
    let (server_a, server_b) = (AuthHttpRemote::start(), AuthHttpRemote::start());
    let (repo_a, repo_b) = (repo_on(&server_a.url()), repo_on(&server_b.url()));
    backend.set_network_auth(&repo_a.repo_ref(), Some(credentials("alice", "pw-a")));
    backend.set_network_auth(&repo_b.repo_ref(), Some(credentials("bob", "pw-b")));
    let (ref_a, ref_b) = (repo_a.repo_ref(), repo_b.repo_ref());
    let opts = FetchOptions::default();
    for _ in 0..3 {
        block_on(async {
            let _ = tokio::join!(
                backend.fetch(&ref_a, "origin", &opts),
                backend.fetch(&ref_b, "origin", &opts),
            );
        });
    }
    let only = |server: &AuthHttpRemote, expected: &str| {
        let seen = server.basic_credentials();
        assert!(!seen.is_empty(), "no credential reached the server");
        assert!(
            seen.iter().all(|c| c == expected),
            "expected only {expected}, the server saw {seen:?}"
        );
    };
    only(&server_a, "alice:pw-a");
    only(&server_b, "bob:pw-b");
}

/// Clearing the entry (`None`) restores the unchanged behaviour: git's own
/// helpers are consulted again, and a repository that never had an entry is
/// not affected by another's.
pub fn check_no_network_auth_entry_means_unchanged_behaviour(backend: &dyn GitBackend) {
    let server = AuthHttpRemote::start();
    let repo = repo_on(&server.url());
    let other = repo_on(&server.url());
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("helper.ran");
    repo.git(&["config", "credential.helper", &marking(&marker)]);
    other.git(&["config", "credential.helper", &marking(&marker)]);

    backend.set_network_auth(&repo.repo_ref(), Some(credentials("alice", "pw")));
    let _ = fetch(backend, &other);
    assert!(
        marker.exists(),
        "another repository's entry changed this one"
    );
    std::fs::remove_file(&marker).unwrap();

    let _ = fetch(backend, &repo);
    assert!(!marker.exists(), "the entry did not apply");

    backend.set_network_auth(&repo.repo_ref(), None);
    let _ = fetch(backend, &repo);
    assert!(
        marker.exists(),
        "clearing the entry did not restore git's helpers"
    );
}

/// The key-file command an entry carries is what ssh runs: it beats the
/// repository's own `core.sshCommand`, and a fetch over the stub ssh reaches
/// the bare repository through it. (That it also beats an exported
/// `GIT_SSH_COMMAND` is checked where the process environment may be set.)
pub fn check_an_entrys_ssh_command_beats_core_ssh_command(backend: &dyn GitBackend) {
    #[cfg(unix)]
    ssh_command_beats_core_ssh_command(backend);
    #[cfg(not(unix))]
    let _ = backend;
}

#[cfg(unix)]
fn ssh_command_beats_core_ssh_command(backend: &dyn GitBackend) {
    use crate::StubSsh;

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

    let chosen = StubSsh::new();
    let pinned = StubSsh::new();
    repo.git(&["config", "core.sshCommand", &pinned.command()]);
    let keys = tempfile::tempdir().unwrap();
    let key = keys.path().join("my keys").join("id work");

    // The repository's own pin is what runs without an entry.
    fetch(backend, &repo).expect("the stub ssh reaches the bare repository");
    assert!(!pinned.calls().is_empty(), "core.sshCommand did not run");
    assert!(chosen.calls().is_empty());

    backend.set_network_auth(
        &repo.repo_ref(),
        Some(NetworkAuth {
            ssh_command: Some(chosen.command_with_key(&key)),
            ..NetworkAuth::default()
        }),
    );
    let before = pinned.calls().len();
    fetch(backend, &repo).expect("the stub ssh reaches the bare repository");
    let keys_used = chosen.identity_files();
    assert!(
        !keys_used.is_empty() && keys_used.iter().all(|k| *k == key.display().to_string()),
        "the entry's key command did not run: {:?}",
        chosen.calls()
    );
    assert_eq!(pinned.calls().len(), before, "core.sshCommand still ran");
}
