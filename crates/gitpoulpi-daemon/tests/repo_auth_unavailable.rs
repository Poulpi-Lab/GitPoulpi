//! A profile that cannot be used refuses the transfer (owner decision 3): it
//! never falls back to the machine's own credentials. Its own test binary:
//! it corrupts and rewrites the shared `repo-auth.toml`, which the other
//! binary's tests must not see.

use std::sync::Arc;

use gitpoulpi_cli_backend::SystemGitBackend;
use gitpoulpi_core::domain::{
    FetchOptions, HttpsAuth, OperationKind, RepoAuthProfile, RepoAuthSetArgs, RepoAuthView,
};
use gitpoulpi_core::TypedError;
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::{Client, Request, ResponseError};
use gitpoulpi_session::{repo_auth_key, RepoAuthStore};
use gitpoulpi_testkit::{AuthHttpRemote, FixtureRepoBuilder};
use serde_json::{json, Value};

gitpoulpi_testkit::isolate_binary!();

struct Rig {
    client: Client,
    repo: gitpoulpi_core::domain::RepoId,
    next: u64,
}

impl Rig {
    async fn call(&mut self, command: OperationKind, args: Value) -> Result<Value, ResponseError> {
        self.next += 1;
        self.client
            .call(&Request {
                id: self.next,
                session: "repo-auth-unavailable".into(),
                repo: Some(self.repo.clone()),
                command,
                args,
            })
            .await
            .unwrap()
            .result
    }

    async fn fetch(&mut self) -> Result<Value, ResponseError> {
        self.call(
            OperationKind::Fetch,
            json!({ "remote": "origin", "opts": FetchOptions::default() }),
        )
        .await
    }

    async fn view(&mut self) -> RepoAuthView {
        serde_json::from_value(
            self.call(OperationKind::RepoAuthGet, json!({}))
                .await
                .unwrap(),
        )
        .unwrap()
    }
}

fn unavailable_reason(error: &ResponseError) -> String {
    match &error.kind {
        Some(TypedError::RepoAuthUnavailable { reason }) => reason.clone(),
        other => panic!(
            "expected RepoAuthUnavailable, got {other:?}: {}",
            error.message
        ),
    }
}

#[tokio::test]
async fn a_profile_that_cannot_be_used_refuses_the_transfer() {
    let server = AuthHttpRemote::start();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .with_origin()
        .unwrap()
        .build();
    repo.git(&["remote", "set-url", "origin", &server.url()]);
    let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
    let handle = Daemon::new("repo-auth-unavailable")
        .spawn(backend)
        .await
        .unwrap();
    let mut rig = Rig {
        client: Client::connect(&handle.endpoint()).await.unwrap(),
        repo: repo.repo_ref().id.clone(),
        next: 1,
    };
    let paths = gitpoulpi_config::ConfigPaths::discover().unwrap();
    let file = RepoAuthStore::file_in(&paths);
    let set = |profile: RepoAuthProfile| {
        serde_json::to_value(RepoAuthSetArgs {
            profile,
            confirm: true,
        })
        .unwrap()
    };

    // Helper and token command sources are not accepted yet, whatever
    // `confirm` says, and nothing is stored.
    for https in [
        HttpsAuth::Helper {
            helper: "!/x/helper".into(),
            username: None,
            hosts: vec![],
            confirmed_sha256: None,
            script_sha256: None,
            confirmed_at: None,
        },
        HttpsAuth::TokenCommand {
            argv: vec!["/x/token".into()],
            username: None,
            hosts: vec![],
            cache_minutes: None,
            confirmed_sha256: None,
            script_sha256: None,
            confirmed_at: None,
        },
    ] {
        let error = rig
            .call(
                OperationKind::RepoAuthSet,
                set(RepoAuthProfile {
                    https: Some(https),
                    ssh: None,
                }),
            )
            .await
            .expect_err("a command source is not available yet");
        assert!(
            unavailable_reason(&error).contains("not available yet"),
            "{error:?}"
        );
    }
    assert!(!rig.view().await.configured);

    // An invalid profile is a plain error, not a stored one.
    let error = rig
        .call(
            OperationKind::RepoAuthSet,
            set(RepoAuthProfile {
                https: Some(HttpsAuth::System {
                    username: Some("a\nb".into()),
                    ignore_repo_helpers: false,
                }),
                ssh: None,
            }),
        )
        .await
        .expect_err("a control character in the user name");
    assert!(
        error.message.contains("control character"),
        "{}",
        error.message
    );

    // An account source that names an account this machine does not have is
    // stored, but refuses the transfer, and the view says so.
    rig.call(
        OperationKind::RepoAuthSet,
        set(RepoAuthProfile {
            https: Some(HttpsAuth::Integration {
                integration: "work".into(),
                username: None,
            }),
            ssh: None,
        }),
    )
    .await
    .unwrap();
    let error = rig.fetch().await.expect_err("the account source refuses");
    assert!(unavailable_reason(&error).contains("no longer exists"), "{error:?}");
    let view = rig.view().await;
    assert!(view.configured);
    assert!(view
        .unavailable
        .as_deref()
        .unwrap()
        .contains("no longer exists"));
    assert_eq!(view.summary.as_deref(), Some("HTTPS: work"));

    // Hand-written helper and token command entries (the file is the source
    // of truth) are parsed and kept, and refuse the same way.
    let key = repo_auth_key(&repo.repo_ref().root);
    for body in [
        "source = \"helper\"\nhelper = \"!/x/helper\"\n",
        "source = \"token_command\"\nargv = [\"/x/token\"]\nhosts = [\"127.0.0.1\"]\n",
    ] {
        std::fs::write(
            &file,
            format!("[[repo]]\nkey = \"{key}\"\ngit_common_dir = \"x\"\nupdated_at = 1\n\n[repo.https]\n{body}"),
        )
        .unwrap();
        let error = rig.fetch().await.expect_err("a command source refuses");
        assert!(unavailable_reason(&error).contains("not available yet"));
        // The view never carries the command.
        let view = rig.view().await;
        let text = serde_json::to_string(&view).unwrap();
        assert!(
            !text.contains("/x/helper") && !text.contains("/x/token"),
            "{text}"
        );
    }

    // An unreadable file refuses too, and is not mistaken for "no profile".
    std::fs::write(&file, "[[repo]\nthis is not toml").unwrap();
    let error = rig
        .fetch()
        .await
        .expect_err("an unreadable profile refuses");
    assert!(
        unavailable_reason(&error).contains("unreadable"),
        "{error:?}"
    );
    let error = rig
        .call(OperationKind::Pull, json!({ "opts": {} }))
        .await
        .expect_err("a pull is refused too");
    assert!(matches!(
        error.kind,
        Some(TypedError::RepoAuthUnavailable { .. })
    ));
    let error = rig
        .call(OperationKind::RepoAuthTest, json!({}))
        .await
        .expect_err("so is the connection test");
    assert!(matches!(
        error.kind,
        Some(TypedError::RepoAuthUnavailable { .. })
    ));
    // Neither fell back to the machine's credentials: the server was never
    // even contacted.
    assert_eq!(server.requests(), 0, "a refused transfer must not connect");
    assert!(rig.view().await.unavailable.unwrap().contains("unreadable"));
    // The status keeps working (no label, no failure).
    rig.call(OperationKind::Status, json!({})).await.unwrap();

    // Once the file is readable and the repository has no profile, the
    // transfer goes ahead.
    std::fs::remove_file(&file).unwrap();
    let _ = rig.fetch().await;
    assert!(server.requests() > 0);
    handle.shutdown().await.unwrap();
}
