//! A repository's token command as the token of its hosting API calls, and
//! its assignment block: the daemon's side of the "Acting as a GitHub App bot"
//! setup. Unix only (the token tool is an `sh` script).
#![cfg(unix)]

use std::path::{Path, PathBuf};
use std::sync::Arc;

use gitpoulpi_cli_backend::SystemGitBackend;
use gitpoulpi_core::domain::{HttpsAuth, OperationKind, RepoAuthProfile, RepoAuthSetArgs, RepoId};
use gitpoulpi_daemon::{hosting_scope, Daemon};
use gitpoulpi_ipc::{Client, Request, ResponseError};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::Value;

gitpoulpi_testkit::isolate_binary!();

const TOKEN: &str = "ghs_plainsecret-0123456789-abcdef";

/// The tests share one private directory: they take turns.
async fn exclusive() -> tokio::sync::MutexGuard<'static, ()> {
    static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
    LOCK.lock().await
}

/// A tool that counts its runs in `counter` and prints the token.
fn tool(dir: &Path, counter: &Path) -> Vec<String> {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("token.sh");
    std::fs::write(
        &path,
        format!(
            "#!/bin/sh\necho x >> '{}'\necho '{TOKEN}'\n",
            counter.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    vec![path.to_string_lossy().into_owned()]
}

fn profile(argv: Vec<String>, use_for_hosting: bool) -> RepoAuthProfile {
    RepoAuthProfile {
        https: Some(HttpsAuth::TokenCommand {
            argv,
            username: None,
            hosts: vec!["github.com".into()],
            cache_minutes: None,
            use_for_hosting,
            confirmed_sha256: None,
            script_sha256: None,
            confirmed_at: None,
        }),
        ssh: None,
    }
}

struct Rig {
    _handle: gitpoulpi_daemon::DaemonHandle,
    client: Client,
    repo: RepoId,
    next: u64,
}

impl Rig {
    async fn new(session: &str, fixture: &FixtureRepo) -> Self {
        let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
        let handle = Daemon::new(session).spawn(backend).await.unwrap();
        let client = Client::connect(&handle.endpoint()).await.unwrap();
        Self {
            _handle: handle,
            client,
            repo: fixture.repo_ref().id.clone(),
            next: 1,
        }
    }

    async fn set(
        &mut self,
        profile: RepoAuthProfile,
        confirm: bool,
    ) -> Result<Value, ResponseError> {
        self.next += 1;
        self.client
            .call(&Request {
                id: self.next,
                session: "s".into(),
                repo: Some(self.repo.clone()),
                command: OperationKind::RepoAuthSet,
                args: serde_json::to_value(RepoAuthSetArgs { profile, confirm }).unwrap(),
            })
            .await
            .unwrap()
            .result
    }
}

fn repo() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build()
}

fn runs(counter: &Path) -> usize {
    std::fs::read_to_string(counter).map_or(0, |t| t.lines().count())
}

#[tokio::test]
async fn the_command_gives_the_hosting_token_cached_and_refreshed_on_demand() {
    let _turn = exclusive().await;
    let fixture = repo();
    let scripts = tempfile::tempdir().unwrap();
    let counter: PathBuf = scripts.path().join("count");
    let argv = tool(scripts.path(), &counter);
    let mut rig = Rig::new("hosting-scope-token", &fixture).await;
    rig.set(profile(argv, true), true).await.unwrap();

    let root = fixture.root();
    assert_eq!(
        hosting_scope::hosting_token(root, false)
            .await
            .unwrap()
            .unwrap(),
        TOKEN
    );
    assert_eq!(
        hosting_scope::hosting_token(root, false)
            .await
            .unwrap()
            .unwrap(),
        TOKEN
    );
    assert_eq!(runs(&counter), 1, "the token is cached between calls");
    hosting_scope::hosting_token(root, true)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(runs(&counter), 2, "a refusal asks the command again");
}

#[tokio::test]
async fn a_profile_not_marked_for_hosting_gives_no_token() {
    let _turn = exclusive().await;
    let fixture = repo();
    let scripts = tempfile::tempdir().unwrap();
    let counter = scripts.path().join("count");
    let argv = tool(scripts.path(), &counter);
    let mut rig = Rig::new("hosting-scope-off", &fixture).await;
    rig.set(profile(argv, false), true).await.unwrap();
    assert!(hosting_scope::hosting_token(fixture.root(), false)
        .await
        .is_none());
    assert_eq!(runs(&counter), 0, "the command is not run for nothing");
}

#[tokio::test]
async fn only_a_person_turns_the_hosting_use_on() {
    let _turn = exclusive().await;
    let fixture = repo();
    let scripts = tempfile::tempdir().unwrap();
    let counter = scripts.path().join("count");
    let argv = tool(scripts.path(), &counter);
    let mut rig = Rig::new("hosting-scope-confirm", &fixture).await;
    rig.set(profile(argv.clone(), false), true).await.unwrap();
    let refused = rig.set(profile(argv.clone(), true), false).await;
    assert!(
        refused.is_err(),
        "an agent cannot send the calls through the command"
    );
    rig.set(profile(argv, true), true).await.unwrap();
}

#[tokio::test]
async fn an_unreadable_command_refuses_the_call_instead_of_falling_back() {
    let _turn = exclusive().await;
    let fixture = repo();
    let scripts = tempfile::tempdir().unwrap();
    let counter = scripts.path().join("count");
    let argv = tool(fixture.root(), &counter);
    let mut rig = Rig::new("hosting-scope-broken", &fixture).await;
    rig.set(profile(argv.clone(), true), true).await.unwrap();
    // The confirmed script (it lives in the repository) changes: the call is refused, not made with the
    // personal token.
    std::fs::write(&argv[0], "#!/bin/sh\necho changed\n").unwrap();
    let answer = hosting_scope::hosting_token(fixture.root(), false).await;
    assert!(matches!(answer, Some(Err(_))), "{answer:?}");
}

#[test]
fn assignment_is_allowed_until_the_repository_blocks_it() {
    let fixture = repo();
    let root = fixture.root();
    assert!(!hosting_scope::assignment_blocked(root));
    let state_dir = gitpoulpi_config::Settings::resolve_ambient()
        .map(|s| s.config().session.per_repo_state_dir.clone())
        .unwrap_or_else(|_| gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION.to_string());
    let path = gitpoulpi_session::RepoState::path_for(root, &state_dir);
    let state = gitpoulpi_session::RepoState {
        assignment_blocked: true,
        ..Default::default()
    };
    state.save(&path).unwrap();
    assert!(hosting_scope::assignment_blocked(root));
}
