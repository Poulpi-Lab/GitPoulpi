//! Per-repository authentication through the daemon (ADR-0010): the profile
//! kept in the private `repo-auth.toml` is resolved into the repository's
//! `NetworkAuth` before each transfer.
//!
//! Everything runs against loopback servers and stub programs: an HTTP
//! server that asks for credentials and records what it is given, and a
//! stub `ssh` that logs its arguments and runs the remote command locally.
//! The binary is isolated (`isolate_binary!`): its global git config is a
//! temporary file, to which `setup` adds a global credential helper (it
//! answers only when git already knows the user name `globaluser`, like a
//! credential manager that has the account but needs to be told which) and a
//! global `core.sshCommand` naming the stub.

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use gitpoulpi_cli_backend::SystemGitBackend;
use gitpoulpi_core::domain::{
    FetchOptions, HttpsAuth, JournalListing, OperationKind, RemoteAccessReport, RepoAuthProfile,
    RepoAuthSetArgs, RepoAuthView, RepoId, SshAuth,
};
use gitpoulpi_core::{FailureKind, TypedError};
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::{Client, Event, Request, ResponseError};
use gitpoulpi_testkit::{AuthHttpRemote, FixtureRepo, FixtureRepoBuilder};
use serde_json::{json, Value};

gitpoulpi_testkit::isolate_binary!();

/// The stub ssh the global `core.sshCommand` names (Unix only: it is a `sh`
/// script).
#[cfg(unix)]
fn stub() -> &'static gitpoulpi_testkit::StubSsh {
    static STUB: OnceLock<gitpoulpi_testkit::StubSsh> = OnceLock::new();
    STUB.get_or_init(gitpoulpi_testkit::StubSsh::new)
}

/// Adds to the isolated global git config: a helper that answers a `get` that
/// names `globaluser` with the password `globalpw`, and (Unix) the stub as `core.sshCommand`. Once,
/// before any git of this binary runs.
fn setup() {
    static ONCE: OnceLock<()> = OnceLock::new();
    ONCE.get_or_init(|| {
        let global = PathBuf::from(std::env::var("GIT_CONFIG_GLOBAL").unwrap());
        let mut text = std::fs::read_to_string(&global).unwrap();
        text.push_str(
            "[credential]\n\thelper = \"!f() { if grep -q username=globaluser; then echo username=globaluser; echo password=globalpw; fi; }; f\"\n",
        );
        #[cfg(unix)]
        text.push_str(&format!(
            "[core]\n\tsshCommand = \"{}\"\n",
            stub().command().replace('\\', "\\\\").replace('"', "\\\"")
        ));
        std::fs::write(&global, text).unwrap();
    });
}

fn config_dir() -> PathBuf {
    PathBuf::from(std::env::var("GITPOULPI_CONFIG_DIR").unwrap())
}

struct Rig {
    handle: gitpoulpi_daemon::DaemonHandle,
    client: Client,
    repo: RepoId,
    next: u64,
}

impl Rig {
    async fn new(name: &str, fixture: &FixtureRepo) -> Self {
        let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
        let handle = Daemon::new(name).spawn(backend).await.unwrap();
        let client = Client::connect(&handle.endpoint()).await.unwrap();
        Self {
            handle,
            client,
            repo: fixture.repo_ref().id.clone(),
            next: 1,
        }
    }

    async fn call(&mut self, command: OperationKind, args: Value) -> Result<Value, ResponseError> {
        self.next += 1;
        self.client
            .call(&Request {
                id: self.next,
                session: "repo-auth-test".into(),
                repo: Some(self.repo.clone()),
                command,
                args,
            })
            .await
            .unwrap()
            .result
    }

    async fn ok(&mut self, command: OperationKind, args: Value) -> Value {
        self.call(command, args)
            .await
            .unwrap_or_else(|e| panic!("{command:?}: {}", e.message))
    }

    async fn fetch(&mut self) -> Result<Value, ResponseError> {
        self.call(
            OperationKind::Fetch,
            json!({ "remote": "origin", "opts": FetchOptions::default() }),
        )
        .await
    }

    async fn set(&mut self, profile: RepoAuthProfile) -> Result<Value, ResponseError> {
        let args = RepoAuthSetArgs {
            profile,
            confirm: false,
        };
        self.call(
            OperationKind::RepoAuthSet,
            serde_json::to_value(args).unwrap(),
        )
        .await
    }

    async fn view(&mut self) -> RepoAuthView {
        serde_json::from_value(self.ok(OperationKind::RepoAuthGet, json!({})).await).unwrap()
    }
}

fn kind_of(error: &ResponseError) -> Option<FailureKind> {
    error.detail.as_ref().map(|d| d.kind())
}

fn https_repo(server: &AuthHttpRemote) -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .with_origin()
        .unwrap()
        .build();
    repo.git(&["remote", "set-url", "origin", &server.url()]);
    repo
}

#[cfg(unix)]
fn ssh_repo() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .build();
    let bare = repo.remote_path().unwrap().to_path_buf();
    repo.git(&[
        "remote",
        "set-url",
        "origin",
        &gitpoulpi_testkit::StubSsh::url_for(&bare),
    ]);
    repo
}

fn system(username: Option<&str>, ignore_repo_helpers: bool) -> RepoAuthProfile {
    RepoAuthProfile {
        https: Some(HttpsAuth::System {
            username: username.map(str::to_string),
            ignore_repo_helpers,
        }),
        ssh: None,
    }
}

/// The reported case: the repository's own `credential.helper` names a script
/// that does not exist on this machine. git sends no credentials and the
/// failure reads as a helper failure. With the System source and "ignore the
/// repository's helpers" the machine's own (global) helper answers.
#[tokio::test]
async fn ignoring_the_repositorys_helpers_lets_the_machines_helper_answer() {
    setup();
    let server = AuthHttpRemote::start();
    let repo = https_repo(&server);
    repo.git(&["config", "credential.helper", "!/no/such/credential-script"]);
    let mut rig = Rig::new("repo-auth-owner", &repo).await;

    // Today's failure.
    let error = rig.fetch().await.expect_err("the missing helper breaks it");
    assert_eq!(
        kind_of(&error),
        Some(FailureKind::CredentialHelperFailed),
        "{}",
        error.message
    );
    assert!(server.basic_credentials().is_empty());

    // System, with the account the machine's helper knows, ignoring the
    // repository's helpers: the machine's credential reaches the server, and
    // the fetch no longer fails because of a helper.
    rig.set(system(Some("globaluser"), true)).await.unwrap();
    let outcome = rig.fetch().await;
    assert!(
        server
            .basic_credentials()
            .iter()
            .any(|c| c == "globaluser:globalpw"),
        "the machine's helper did not answer: {:?}",
        server.basic_credentials()
    );
    if let Err(error) = outcome {
        assert_ne!(
            kind_of(&error),
            Some(FailureKind::CredentialHelperFailed),
            "{}",
            error.message
        );
    }

    // The view says what it did: the local helper is listed, with its file,
    // and marked as ignored.
    let view = rig.view().await;
    assert!(view.configured);
    let local = view
        .overrides
        .iter()
        .find(|o| o.key == "credential.helper")
        .expect("the repository's helper is listed");
    assert_eq!(local.scope, "local");
    assert!(local.origin.contains("config"), "{local:?}");
    assert!(local.overridden);
    assert_eq!(
        view.summary.as_deref(),
        Some("HTTPS: globaluser, repo helpers ignored")
    );
    assert_eq!(view.unavailable, None);
    assert_eq!(view.remotes[0].remote, "origin");
    assert!(view.remotes[0].identity.starts_with("system git"));

    // The status carries the label for the push indicator, no spawn needed.
    let status = rig.ok(OperationKind::Status, json!({})).await;
    assert_eq!(
        status["repo_auth"],
        "HTTPS: globaluser, repo helpers ignored"
    );

    // Clear: back to the machine's defaults, and today's failure.
    rig.ok(OperationKind::RepoAuthClear, json!({})).await;
    let error = rig.fetch().await.expect_err("back to the broken helper");
    assert_eq!(kind_of(&error), Some(FailureKind::CredentialHelperFailed));
    let status = rig.ok(OperationKind::Status, json!({})).await;
    assert!(status.get("repo_auth").is_none());
    rig.handle.shutdown().await.unwrap();
}

/// "Ignore the repository's helpers" keeps them from running: without it a
/// helper of the repository's config runs (control), with it only the
/// machine's do.
#[tokio::test]
async fn the_repositorys_helpers_run_unless_they_are_ignored() {
    setup();
    let server = AuthHttpRemote::start();
    let repo = https_repo(&server);
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("local-helper.ran");
    repo.git(&[
        "config",
        "credential.helper",
        &format!(
            "!f() {{ cat >/dev/null; touch '{}'; }}; f",
            marker.to_string_lossy().replace('\\', "/")
        ),
    ]);
    let mut rig = Rig::new("repo-auth-markers", &repo).await;

    rig.set(system(None, false)).await.unwrap();
    let _ = rig.fetch().await;
    assert!(marker.exists(), "control: the repository's helper must run");
    std::fs::remove_file(&marker).unwrap();

    rig.set(system(None, true)).await.unwrap();
    let _ = rig.fetch().await;
    assert!(
        !marker.exists(),
        "the repository's helper ran although ignored"
    );
    rig.handle.shutdown().await.unwrap();
}

/// A repository that never had a profile behaves exactly as before: its own
/// helper runs.
#[tokio::test]
async fn a_repository_without_a_profile_keeps_the_global_behaviour() {
    setup();
    let server = AuthHttpRemote::start();
    let repo = https_repo(&server);
    repo.git(&["config", "credential.username", "globaluser"]);
    let mut rig = Rig::new("repo-auth-default", &repo).await;
    let _ = rig.fetch().await;
    assert!(server
        .basic_credentials()
        .iter()
        .any(|c| c == "globaluser:globalpw"));
    let view = rig.view().await;
    assert!(!view.configured);
    assert_eq!(view.summary, None);
    rig.handle.shutdown().await.unwrap();
}

/// A key file chosen for the repository beats the repository's own
/// `core.sshCommand`, through the program the global `core.sshCommand`
/// names (the stub), and the connection test names the accepted key.
#[cfg(unix)]
#[tokio::test]
async fn a_key_file_beats_the_repositorys_own_ssh_command() {
    setup();
    let repo = ssh_repo();
    repo.git(&["config", "core.sshCommand", "/bin/false"]);
    let keys = tempfile::tempdir().unwrap();
    let key = keys.path().join("work_ed25519");
    std::fs::write(&key, "not a real key").unwrap();
    let key_text = key.to_string_lossy().into_owned();
    let mut rig = Rig::new("repo-auth-key", &repo).await;

    // Control: the repository's own command is /bin/false.
    rig.fetch().await.expect_err("the local command fails");

    let profile = RepoAuthProfile {
        https: None,
        ssh: Some(SshAuth::KeyFile {
            key: key_text.clone(),
            known_hosts: None,
            strict_host_key: true,
        }),
    };
    rig.set(profile).await.unwrap();
    rig.fetch().await.expect("the key file command is used");
    let seen = stub().calls();
    let call = seen
        .iter()
        .find(|c| c.contains(&key_text))
        .expect("the stub saw the key");
    assert!(call.windows(2).any(|w| w == ["-o", "IdentitiesOnly=yes"]));
    assert!(call
        .windows(2)
        .any(|w| w == ["-o", "StrictHostKeyChecking=yes"]));

    // The connection test names the key the server accepted.
    let reports: Vec<RemoteAccessReport> = serde_json::from_value(
        rig.ok(OperationKind::RepoAuthTest, json!({ "push": false }))
            .await,
    )
    .unwrap();
    assert_eq!(reports.len(), 1);
    assert!(reports[0].ok, "{:?}", reports[0]);
    assert_eq!(reports[0].remote, "origin");
    assert_eq!(
        reports[0].ssh_accepted_key.as_deref(),
        Some(key_text.as_str())
    );

    // The view lists the repository's command as overridden, with its origin.
    let view = rig.view().await;
    let pin = view
        .overrides
        .iter()
        .find(|o| o.key == "core.sshcommand")
        .expect("the local core.sshCommand is listed");
    assert_eq!(pin.value, "/bin/false");
    assert_eq!(pin.scope, "local");
    assert!(pin.overridden);
    assert_eq!(view.remotes[0].identity, "ssh key work_ed25519");
    assert_eq!(view.summary.as_deref(), Some("SSH key: work_ed25519"));

    // A key that is gone refuses the transfer, typed, and runs nothing.
    std::fs::remove_file(&key).unwrap();
    let before = stub().calls().len();
    let error = rig.fetch().await.expect_err("a missing key refuses");
    assert!(
        matches!(error.kind, Some(TypedError::RepoAuthUnavailable { .. })),
        "{error:?}"
    );
    assert_eq!(stub().calls().len(), before, "nothing may have run");
    rig.handle.shutdown().await.unwrap();
}

/// The key of an integration profile is read live: a changed key applies to
/// the next transfer, and an integration that is gone refuses it.
#[cfg(unix)]
#[tokio::test]
async fn an_integrations_key_is_resolved_at_each_transfer() {
    use gitpoulpi_config::{Integration, IntegrationKind, IntegrationSsh, SshMode};
    setup();
    let repo = ssh_repo();
    let keys = tempfile::tempdir().unwrap();
    let (first, second) = (keys.path().join("first"), keys.path().join("second"));
    std::fs::write(&first, "k1").unwrap();
    std::fs::write(&second, "k2").unwrap();
    let write_integration = |key: Option<&std::path::Path>| {
        let integration = Integration {
            id: "repo-auth-work".into(),
            kind: IntegrationKind::Github,
            url: "https://github.invalid".into(),
            token_ref: "repo-auth-work".into(),
            name: String::new(),
            auth: Default::default(),
            account: String::new(),
            user: None,
            ssh: key.map(|k| IntegrationSsh {
                mode: SshMode::Custom,
                private_key_path: Some(k.to_string_lossy().into_owned()),
                public_key_path: None,
                title: None,
                remote_key_id: None,
            }),
            enabled: true,
            local: None,
            radar: None,
        };
        gitpoulpi_config::set_list_in_file(
            &config_dir().join("config.toml"),
            "integrations",
            &[integration],
        )
        .unwrap();
    };
    write_integration(Some(&first));
    let mut rig = Rig::new("repo-auth-integration-key", &repo).await;
    rig.set(RepoAuthProfile {
        https: None,
        ssh: Some(SshAuth::IntegrationKey {
            integration: "repo-auth-work".into(),
            known_hosts: None,
            strict_host_key: false,
        }),
    })
    .await
    .unwrap();

    rig.fetch().await.expect("the account's key is used");
    assert!(stub()
        .identity_files()
        .contains(&first.to_string_lossy().into_owned()));

    // Changed in Settings: the next transfer uses it.
    write_integration(Some(&second));
    rig.fetch().await.expect("the new key is used");
    assert!(stub()
        .identity_files()
        .contains(&second.to_string_lossy().into_owned()));
    assert_eq!(
        rig.view().await.summary.as_deref(),
        Some("SSH key of repo-auth-work")
    );

    // An account without a key, and one that is gone: refused, nothing runs,
    // and the view says why.
    for gone in [false, true] {
        if gone {
            gitpoulpi_config::set_list_in_file::<Integration>(
                &config_dir().join("config.toml"),
                "integrations",
                &[],
            )
            .unwrap();
        } else {
            write_integration(None);
        }
        let before = stub().calls().len();
        let error = rig.fetch().await.expect_err("an unusable account refuses");
        assert!(
            matches!(error.kind, Some(TypedError::RepoAuthUnavailable { .. })),
            "{error:?}"
        );
        assert_eq!(stub().calls().len(), before);
        assert!(rig.view().await.unavailable.is_some());
    }
    rig.handle.shutdown().await.unwrap();
}

/// Setting and clearing broadcast `RepoAuthChanged`, are not journaled (the
/// undo list is unchanged), and clearing nothing says nothing.
#[tokio::test]
async fn a_profile_change_is_announced_and_never_journaled() {
    setup();
    let server = AuthHttpRemote::start();
    let repo = https_repo(&server);
    let mut rig = Rig::new("repo-auth-event", &repo).await;
    let mut watcher = Client::connect(&rig.handle.endpoint())
        .await
        .unwrap()
        .into_stream();

    let target = rig.repo.clone();
    let listing = |v: Value| -> JournalListing { serde_json::from_value(v).unwrap() };
    let before = listing(rig.ok(OperationKind::JournalList, json!({})).await);

    rig.set(system(Some("alice"), true)).await.unwrap();
    wait_for_change(&mut watcher, &target).await;

    assert_eq!(
        rig.ok(OperationKind::RepoAuthClear, json!({})).await,
        json!(true)
    );
    wait_for_change(&mut watcher, &target).await;

    let after = listing(rig.ok(OperationKind::JournalList, json!({})).await);
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(&after).unwrap(),
        "a profile change must not be journaled"
    );
    assert!(!OperationKind::RepoAuthSet.traits().journaled);
    assert!(!OperationKind::RepoAuthClear.traits().journaled);

    // Nothing to clear: nothing announced, and no error.
    assert_eq!(
        rig.ok(OperationKind::RepoAuthClear, json!({})).await,
        json!(false)
    );
    rig.handle.shutdown().await.unwrap();
}

async fn wait_for_change(watcher: &mut tokio::net::TcpStream, target: &RepoId) {
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            let msg: gitpoulpi_ipc::ServerMessage = gitpoulpi_ipc::transport::read_message(watcher)
                .await
                .unwrap()
                .expect("closed before the event");
            if let gitpoulpi_ipc::ServerMessage::Event(Event::RepoAuthChanged { repo }) = msg {
                if repo == *target {
                    return;
                }
            }
        }
    })
    .await
    .expect("timed out waiting for RepoAuthChanged");
}
