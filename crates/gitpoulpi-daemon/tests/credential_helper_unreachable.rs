//! A transfer whose grant the helper never asked about (it could not reach the
//! daemon) fails as `CredentialHelperUnreachable`, not as git's "terminal
//! prompts disabled" (an authentication failure). The helper here is the
//! stand-in binary told to stay deaf: it prints nothing and exits 0, which is
//! all a helper that cannot connect does. Loopback only.

use std::sync::Arc;

use gitpoulpi_cli_backend::SystemGitBackend;
use gitpoulpi_core::domain::{
    FetchOptions, HttpsAuth, OperationKind, RepoAuthProfile, RepoAuthSetArgs,
};
use gitpoulpi_core::TypedError;
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::{Client, Request};
use gitpoulpi_testkit::{AuthHttpRemote, FixtureRepoBuilder};
use serde_json::json;

gitpoulpi_testkit::isolate_binary!();

#[tokio::test]
async fn a_helper_that_never_reaches_the_daemon_is_named_as_such() {
    std::env::set_var("GITPOULPI_STUB_DEAF", "1");
    gitpoulpi_daemon::set_credential_helper_exe(Some(
        env!("CARGO_BIN_EXE_gitpoulpi-credential-stub").into(),
    ));
    let server = AuthHttpRemote::start_accepting();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .with_origin()
        .unwrap()
        .build();
    repo.git(&["remote", "set-url", "origin", &server.url()]);

    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
    let handle = Daemon::new("helper-unreachable")
        .spawn(backend)
        .await
        .unwrap();
    let mut client = Client::connect(&handle.endpoint()).await.unwrap();
    let repo_id = repo.repo_ref().id.clone();
    let mut next = 0;
    let mut call = |command: OperationKind, args: serde_json::Value| {
        next += 1;
        Request {
            id: next,
            session: "x".into(),
            repo: Some(repo_id.clone()),
            command,
            args,
        }
    };

    // The command is never run: the helper never asks for it.
    let profile = RepoAuthProfile {
        https: Some(HttpsAuth::TokenCommand {
            argv: vec!["a-token-tool-that-never-runs".into()],
            username: None,
            hosts: vec![server.host()],
            cache_minutes: None,
            use_for_hosting: false,
            confirmed_sha256: None,
            script_sha256: None,
            confirmed_at: None,
        }),
        ssh: None,
    };
    let set = client
        .call(&call(
            OperationKind::RepoAuthSet,
            serde_json::to_value(RepoAuthSetArgs {
                profile,
                confirm: true,
            })
            .unwrap(),
        ))
        .await
        .unwrap();
    set.result.expect("a confirmed command is stored");

    let fetch = client
        .call(&call(
            OperationKind::Fetch,
            json!({ "remote": "origin", "opts": FetchOptions::default() }),
        ))
        .await
        .unwrap();
    let error = fetch.result.expect_err("git had no credential");
    assert_eq!(
        error.kind,
        Some(TypedError::CredentialHelperUnreachable),
        "{}",
        error.message
    );
    assert!(
        error.message.contains("credential helper"),
        "{}",
        error.message
    );

    // The connection test reports it first in its stderr.
    let test = client
        .call(&call(OperationKind::RepoAuthTest, json!({ "push": false })))
        .await
        .unwrap();
    let reports = test.result.expect("a test is a report, not an error");
    let stderr = reports[0]["error"]["stderr"].as_str().unwrap_or_default();
    assert!(
        stderr.starts_with("GitPoulpi's credential helper could not reach the daemon"),
        "{reports}"
    );

    handle.shutdown().await.unwrap();
}
