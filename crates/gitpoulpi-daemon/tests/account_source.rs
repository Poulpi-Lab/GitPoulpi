//! A GitPoulpi account as a repository's HTTPS source (ADR-0010): the token
//! reaches git through a credential grant, only for the account's own
//! address, only while the operation runs, and never through an argument or
//! an environment variable.
//!
//! Loopback servers only. The helper git runs is the stand-in binary of this
//! crate (`gitpoulpi-credential-stub`), copied under a directory with a space
//! in its name and logging its arguments and environment (no wrapper script,
//! so the same test runs on Windows).
#![cfg(feature = "hosting")]

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use gitpoulpi_cli_backend::SystemGitBackend;
use gitpoulpi_config::{ConnectedUser, Integration, IntegrationKind};
use gitpoulpi_core::domain::{
    FetchOptions, HttpsAuth, OperationKind, RemoteAccessReport, RepoAuthProfile, RepoAuthSetArgs,
    RepoId,
};
use gitpoulpi_core::TypedError;
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::git_credential::{
    CredentialAction, CredentialGrantAnswer, CredentialGrantRequest,
};
use gitpoulpi_ipc::{Client, Request, ResponseError};
use gitpoulpi_secrets::{InMemoryTokens, TokenStore};
use gitpoulpi_testkit::{AuthHttpRemote, FixtureRepo, FixtureRepoBuilder};
use serde_json::{json, Value};

gitpoulpi_testkit::isolate_binary!();

/// A token with no prefix any redaction rule knows.
const TOKEN: &str = "plainsecret-0123456789-abcdef";

fn config_dir() -> PathBuf {
    PathBuf::from(std::env::var("GITPOULPI_CONFIG_DIR").unwrap())
}

/// The helper git runs, and where its wrapper logs (Unix).
struct Helper {
    _dir: tempfile::TempDir,
    exe: PathBuf,
    log: PathBuf,
}

fn helper() -> &'static Helper {
    static HELPER: OnceLock<Helper> = OnceLock::new();
    HELPER.get_or_init(|| {
        let dir = tempfile::tempdir().unwrap();
        let stub = PathBuf::from(env!("CARGO_BIN_EXE_gitpoulpi-credential-stub"));
        // A directory with a space: the helper command must survive it under
        // the `sh` git runs it with (Git for Windows' on Windows).
        let spaced = dir.path().join("helper dir");
        std::fs::create_dir_all(&spaced).unwrap();
        let exe = spaced.join(format!("helper stub{}", std::env::consts::EXE_SUFFIX));
        std::fs::copy(&stub, &exe).unwrap();
        let log = dir.path().join("helper.log");
        std::env::set_var("GITPOULPI_STUB_LOG", &log);
        Helper {
            _dir: dir,
            exe,
            log,
        }
    })
}

fn setup() {
    static ONCE: OnceLock<()> = OnceLock::new();
    ONCE.get_or_init(|| {
        let store = InMemoryTokens::default();
        for id in ["acct-main", "acct-instead", "acct-edited"] {
            store.store(id, TOKEN).unwrap();
        }
        gitpoulpi_daemon::set_token_store(Some(Arc::new(store)));
        gitpoulpi_daemon::set_credential_helper_exe(Some(helper().exe.clone()));
    });
}

/// The tests share one `config.toml`: they take turns.
async fn exclusive() -> tokio::sync::MutexGuard<'static, ()> {
    static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
    LOCK.lock().await
}

/// Writes the account `id` pointing at `url`.
fn write_account(id: &str, url: &str) {
    let integration = Integration {
        id: id.into(),
        kind: IntegrationKind::Github,
        url: url.into(),
        token_ref: id.into(),
        name: String::new(),
        auth: Default::default(),
        account: String::new(),
        // Connected before: its trust record is written at first use.
        user: Some(ConnectedUser {
            login: "ada".into(),
            name: String::new(),
            id: "7".into(),
        }),
        ssh: None,
        enabled: true,
        local: None,
        radar: None,
        markup: Default::default(),
    };
    gitpoulpi_config::set_list_in_file(
        &config_dir().join("config.toml"),
        "integrations",
        &[integration],
    )
    .unwrap();
}

fn origin_of(server: &AuthHttpRemote) -> String {
    format!("http://{}", server.host())
}

fn account_profile(id: &str) -> RepoAuthProfile {
    RepoAuthProfile {
        https: Some(HttpsAuth::Integration {
            integration: id.into(),
            username: None,
        }),
        ssh: None,
    }
}

fn repo_on(server: &AuthHttpRemote) -> FixtureRepo {
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

struct Rig {
    handle: gitpoulpi_daemon::DaemonHandle,
    client: Client,
    repo: RepoId,
    next: u64,
}

impl Rig {
    async fn new(session: &str, fixture: &FixtureRepo) -> Self {
        let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
        let handle = Daemon::new(session).spawn(backend).await.unwrap();
        // The helper finds its daemon the way every real one does.
        let paths = gitpoulpi_config::ConfigPaths::discover().unwrap();
        gitpoulpi_ipc::discovery::publish(&paths.ipc_port_file(session), &handle.endpoint())
            .unwrap();
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
                // A client-written name is untrusted: the daemon stamps its own.
                session: "whatever-the-client-says".into(),
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

    async fn use_account(&mut self, id: &str) {
        let args = RepoAuthSetArgs {
            profile: account_profile(id),
            confirm: false,
        };
        self.call(
            OperationKind::RepoAuthSet,
            serde_json::to_value(args).unwrap(),
        )
        .await
        .expect("the account source is accepted");
    }

    /// What the helper would send for `grant`.
    async fn grant(
        &mut self,
        grant: &str,
        action: CredentialAction,
        host: &str,
    ) -> CredentialGrantAnswer {
        self.next += 1;
        let response = self
            .client
            .call(&Request {
                id: self.next,
                session: "x".into(),
                repo: None,
                command: OperationKind::CredentialGrant,
                args: serde_json::to_value(CredentialGrantRequest {
                    grant: grant.into(),
                    action,
                    protocol: "http".into(),
                    host: host.into(),
                    path: String::new(),
                })
                .unwrap(),
            })
            .await
            .unwrap();
        serde_json::from_value(response.result.unwrap()).unwrap()
    }
}

/// The token reaches the loopback server as `x-access-token:<token>`, a
/// connection test names the account that was accepted, nothing secret is in
/// what the helper process saw, and the grant is dead once the operation is.
#[tokio::test]
async fn the_accounts_token_reaches_its_own_server_and_nothing_else_sees_it() {
    let _turn = exclusive().await;
    setup();
    let server = AuthHttpRemote::start_accepting();
    write_account("acct-main", &origin_of(&server));
    let repo = repo_on(&server);
    let mut rig = Rig::new("account-source-main", &repo).await;
    rig.use_account("acct-main").await;

    rig.fetch().await.expect("the account's token is accepted");
    let received = server.basic_credentials();
    assert!(!received.is_empty());
    assert!(
        received
            .iter()
            .all(|c| *c == format!("x-access-token:{TOKEN}")),
        "{received:?}"
    );

    // The test names the identity git reported as accepted (`store`).
    let reports: Vec<RemoteAccessReport> = serde_json::from_value(
        rig.call(OperationKind::RepoAuthTest, json!({ "push": false }))
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(reports.len(), 1);
    assert!(reports[0].ok, "{reports:?}");
    assert_eq!(
        reports[0].accepted_identity.as_deref(),
        Some("account acct-main (token, user x-access-token)")
    );

    {
        let log = std::fs::read_to_string(&helper().log).unwrap();
        assert!(log.contains("ARGV [--gitpoulpi-git-credential] [get]"));
        assert!(
            log.contains("ARGV [--gitpoulpi-git-credential] [store]"),
            "git reports acceptance"
        );
        // Neither the helper's arguments nor its environment (git's own)
        // hold the token.
        assert!(!log.contains(TOKEN), "the token leaked into argv or env");
        let grant = log
            .lines()
            .find_map(|l| l.strip_prefix("GITPOULPI_CREDENTIAL_GRANT="))
            .expect("the helper was given a grant id")
            .to_string();
        assert!(log.contains("GITPOULPI_CREDENTIAL_SESSION=account-source-main"));
        // The operation is over: the grant answers nothing, for its own host too.
        let dead = rig
            .grant(&grant, CredentialAction::Get, &server.host())
            .await;
        assert_eq!(dead, CredentialGrantAnswer::default());
    }
    rig.handle.shutdown().await.unwrap();
}

/// A repository can rewrite a remote's URL to another server with its own
/// `url.<base>.insteadOf`. The grant is bound to the account's address, so
/// the other server never sees the token, and the failure says why.
#[tokio::test]
async fn a_rewritten_remote_url_gets_no_token() {
    let _turn = exclusive().await;
    setup();
    let account_server = AuthHttpRemote::start_accepting();
    let other = AuthHttpRemote::start_accepting();
    write_account("acct-instead", &origin_of(&account_server));
    let repo = repo_on(&account_server);
    repo.git(&[
        "config",
        &format!("url.http://{}/.insteadOf", other.host()),
        &format!("http://{}/", account_server.host()),
    ]);
    let mut rig = Rig::new("account-source-instead-of", &repo).await;
    rig.use_account("acct-instead").await;

    let error = rig.fetch().await.expect_err("the other server is refused");
    assert_eq!(
        error.kind,
        Some(TypedError::GrantRefused {
            host: other.host(),
            expected: account_server.host(),
        }),
        "{}",
        error.message
    );
    assert!(
        other.authorizations().is_empty(),
        "no credential may reach the other server: {:?}",
        other.authorizations()
    );
    assert!(account_server.authorizations().is_empty());
    assert!(!error.message.contains(TOKEN));
    rig.handle.shutdown().await.unwrap();
}

/// The address a token is bound to is the trust store's record: an account
/// whose URL was edited afterwards refuses the transfer instead of sending
/// the token to the new address. And one that is gone refuses too.
#[tokio::test]
async fn an_edited_account_address_refuses_the_transfer() {
    let _turn = exclusive().await;
    setup();
    let server = AuthHttpRemote::start_accepting();
    let elsewhere = AuthHttpRemote::start_accepting();
    write_account("acct-edited", &origin_of(&server));
    let repo = repo_on(&server);
    let mut rig = Rig::new("account-source-edited", &repo).await;
    rig.use_account("acct-edited").await;
    rig.fetch().await.expect("first use records the origin");

    write_account("acct-edited", &origin_of(&elsewhere));
    let error = rig.fetch().await.expect_err("the address changed");
    assert!(
        matches!(error.kind, Some(TypedError::RepoAuthUnavailable { .. })),
        "{error:?}"
    );
    assert!(elsewhere.authorizations().is_empty());

    gitpoulpi_config::set_list_in_file::<Integration>(
        &config_dir().join("config.toml"),
        "integrations",
        &[],
    )
    .unwrap();
    let error = rig.fetch().await.expect_err("the account is gone");
    assert!(
        matches!(&error.kind, Some(TypedError::RepoAuthUnavailable { reason }) if reason.contains("no longer exists")),
        "{error:?}"
    );
    rig.handle.shutdown().await.unwrap();
}
