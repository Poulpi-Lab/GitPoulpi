//! `GitBackend::remote_access_check`: the connection test behind "Test
//! connection" (ADR-0010). Every check runs without a network: file remotes,
//! the loopback [`AuthHttpRemote`] and the stub ssh.

use super::block_on;
use crate::{AuthHttpRemote, FixtureRepo, FixtureRepoBuilder};
use gitpoulpi_core::domain::{RemoteAccessOptions, RemoteTransport};
use gitpoulpi_core::{FailureKind, GitBackend};

fn repo_with_pushed_origin() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .with_origin()
        .unwrap()
        .build();
    repo.git(&["push", "origin", "main"]);
    repo
}

/// Over an ssh remote the report counts the advertised branches and, asked
/// to, names the key the server accepted and the host that authenticated.
pub fn check_a_remote_access_check_over_ssh_names_the_accepted_key(backend: &dyn GitBackend) {
    #[cfg(unix)]
    {
        use crate::StubSsh;
        use gitpoulpi_core::domain::NetworkAuth;

        let repo = repo_with_pushed_origin();
        let bare = repo.remote_path().unwrap().to_path_buf();
        let url = StubSsh::url_for(&bare);
        repo.git(&["remote", "set-url", "origin", &url]);
        let stub = StubSsh::new();
        let keys = tempfile::tempdir().unwrap();
        let key = keys.path().join("work key");
        backend.set_network_auth(
            &repo.repo_ref(),
            Some(NetworkAuth {
                ssh_command: Some(stub.command_with_key(&key)),
                ..NetworkAuth::default()
            }),
        );

        let report = block_on(backend.remote_access_check(
            &repo.repo_ref(),
            "origin",
            &RemoteAccessOptions {
                push: false,
                ssh_verbose: true,
            },
        ))
        .unwrap();
        assert!(report.ok, "{report:?}");
        assert_eq!(report.remote, "origin");
        assert_eq!(report.transport, RemoteTransport::Ssh);
        assert_eq!(report.url, url);
        assert_eq!(report.ref_count, 1, "{report:?}");
        assert!(report.error.is_none());
        assert_eq!(
            report.ssh_accepted_key.as_deref(),
            Some(key.to_str().unwrap()),
            "{report:?}"
        );
        assert_eq!(report.ssh_authenticated_to.as_deref(), Some("stub.invalid"));
        assert!(
            stub.calls()[0].iter().any(|a| a == "-v"),
            "ssh was not asked for -v: {:?}",
            stub.calls()
        );

        // Without ssh_verbose the key is not reported and ssh is not verbose.
        let quiet = block_on(backend.remote_access_check(
            &repo.repo_ref(),
            "origin",
            &RemoteAccessOptions::default(),
        ))
        .unwrap();
        assert!(quiet.ok);
        assert_eq!(quiet.ssh_accepted_key, None);
        assert!(!stub.calls()[1].iter().any(|a| a == "-v"));
    }
    #[cfg(not(unix))]
    let _ = backend;
}

/// `push: true` tests the push URL, which is not the fetch URL when the
/// remote has a `pushurl`.
pub fn check_a_remote_access_check_with_push_uses_the_push_url(backend: &dyn GitBackend) {
    let repo = repo_with_pushed_origin();
    let good = repo
        .remote_path()
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/");
    let missing = repo
        .root()
        .join("no-such-remote.git")
        .to_string_lossy()
        .replace('\\', "/");
    repo.git(&["remote", "set-url", "origin", &missing]);
    repo.git(&["remote", "set-url", "--push", "origin", &good]);

    let fetch_report = block_on(backend.remote_access_check(
        &repo.repo_ref(),
        "origin",
        &RemoteAccessOptions::default(),
    ))
    .unwrap();
    assert!(!fetch_report.ok, "{fetch_report:?}");
    assert_eq!(fetch_report.url, missing);
    assert!(!fetch_report.push);

    let push_report = block_on(backend.remote_access_check(
        &repo.repo_ref(),
        "origin",
        &RemoteAccessOptions {
            push: true,
            ssh_verbose: false,
        },
    ))
    .unwrap();
    assert!(push_report.ok, "{push_report:?}");
    assert_eq!(push_report.url, good);
    assert!(push_report.push);
    assert_eq!(push_report.transport, RemoteTransport::File);
    assert_eq!(push_report.ref_count, 1);
}

/// A failed connection is a report, not an error: `ok: false` and an
/// [`gitpoulpi_core::ErrorDetail`] the frontends can classify. A server that
/// refuses the key reads as an authentication failure. The URL carries a
/// password the report must not repeat; a remote that does not exist is an
/// `Err`.
pub fn check_a_failed_remote_access_check_is_a_classified_report(backend: &dyn GitBackend) {
    // A password in the URL: the server answers 404 (it only asks for
    // credentials when none are given), the report hides the password.
    let server = AuthHttpRemote::start();
    let repo = repo_with_pushed_origin();
    repo.git(&[
        "remote",
        "set-url",
        "origin",
        &format!("http://me:hunter2@{}/r.git", server.host()),
    ]);
    let report = block_on(backend.remote_access_check(
        &repo.repo_ref(),
        "origin",
        &RemoteAccessOptions::default(),
    ))
    .unwrap();
    assert!(!report.ok, "{report:?}");
    assert_eq!(report.transport, RemoteTransport::Http);
    assert_eq!(report.ref_count, 0);
    assert!(!report.url.contains("hunter2"), "{}", report.url);
    assert!(!report.url.contains("me:"), "{}", report.url);
    let detail = report.error.expect("a failure carries its detail");
    assert!(!format!("{detail:?}").contains("hunter2"), "{detail:?}");

    let missing = block_on(backend.remote_access_check(
        &repo.repo_ref(),
        "no-such-remote",
        &RemoteAccessOptions::default(),
    ));
    assert!(missing.is_err(), "an unknown remote is an error");

    #[cfg(unix)]
    {
        use crate::StubSsh;
        let bare = repo.remote_path().unwrap().to_path_buf();
        repo.git(&["remote", "set-url", "origin", &StubSsh::url_for(&bare)]);
        let denying = StubSsh::denying();
        backend.set_network_auth(
            &repo.repo_ref(),
            Some(gitpoulpi_core::domain::NetworkAuth {
                ssh_command: Some(denying.command()),
                ..Default::default()
            }),
        );
        let report = block_on(backend.remote_access_check(
            &repo.repo_ref(),
            "origin",
            &RemoteAccessOptions::default(),
        ))
        .unwrap();
        assert!(!report.ok, "{report:?}");
        assert_eq!(report.transport, RemoteTransport::Ssh);
        let detail = report.error.expect("a failure carries its detail");
        assert_eq!(detail.kind(), FailureKind::AuthFailed, "{detail:?}");
        assert!(detail.stderr.contains("Permission denied"), "{detail:?}");
    }
}
