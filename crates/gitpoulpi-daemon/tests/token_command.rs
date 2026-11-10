//! The helper and token-command HTTPS sources (ADR-0010): a command the
//! person confirmed makes the credential, only for its hosts, cached until the
//! server refuses it; a command that is not the confirmed one, or whose
//! in-tree script changed, refuses the transfer.
//!
//! Loopback servers only. The token tools are `sh` scripts on Unix and
//! PowerShell scripts (`pwsh -NoProfile -File x.ps1`) on Windows: the tests
//! say so and skip when `pwsh` is not installed. The helper git runs for the
//! token command is the stand-in binary of this crate, copied under a
//! directory with a space and logging its arguments and environment.

use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use gitpoulpi_cli_backend::SystemGitBackend;
use gitpoulpi_core::domain::{
    FetchOptions, HttpsAuth, OperationKind, RepoAuthProfile, RepoAuthSetArgs, RepoAuthView, RepoId,
};
use gitpoulpi_core::TypedError;
use gitpoulpi_daemon::Daemon;
use gitpoulpi_ipc::{Client, Request, ResponseError};
use gitpoulpi_testkit::{AuthHttpRemote, FixtureRepo, FixtureRepoBuilder};
use serde_json::{json, Value};

gitpoulpi_testkit::isolate_binary!();

/// A token with no prefix any redaction rule knows.
const TOKEN: &str = "plainsecret-0123456789-abcdef";

/// The tests share one private directory (`repo-auth.toml`): they take turns,
/// because one of them rewrites the file by hand.
async fn exclusive() -> tokio::sync::MutexGuard<'static, ()> {
    static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
    LOCK.lock().await
}

/// Whether the scripting language of the token tools is installed: always on
/// Unix (`sh`), `pwsh` on Windows. A test that needs it returns early with a
/// message otherwise.
fn tools_available() -> bool {
    #[cfg(windows)]
    {
        let found = std::process::Command::new("pwsh")
            .args(["-NoProfile", "-Command", "exit 0"])
            .output()
            .is_ok_and(|o| o.status.success());
        if !found {
            eprintln!("SKIPPED: pwsh is not installed, the token tools cannot run");
        }
        found
    }
    #[cfg(not(windows))]
    {
        true
    }
}

macro_rules! need_tools {
    () => {
        if !tools_available() {
            return;
        }
    };
}

struct Helper {
    _dir: tempfile::TempDir,
    log: PathBuf,
}

/// The helper git runs for the token command: a copy of the stub under a
/// directory with a space, which logs its arguments and environment.
fn helper() -> &'static Helper {
    static HELPER: OnceLock<Helper> = OnceLock::new();
    HELPER.get_or_init(|| {
        let dir = tempfile::tempdir().unwrap();
        let stub = PathBuf::from(env!("CARGO_BIN_EXE_gitpoulpi-credential-stub"));
        let spaced = dir.path().join("helper dir");
        std::fs::create_dir_all(&spaced).unwrap();
        let exe = spaced.join(format!("helper stub{}", std::env::consts::EXE_SUFFIX));
        std::fs::copy(&stub, &exe).unwrap();
        let log = dir.path().join("helper.log");
        std::env::set_var("GITPOULPI_STUB_LOG", &log);
        gitpoulpi_daemon::set_credential_helper_exe(Some(exe));
        Helper { _dir: dir, log }
    })
}

/// What a token tool does, in order.
enum Step<'a> {
    /// Appends a line to the file (counts the runs).
    Append(&'a Path),
    /// Writes its working directory to the file.
    Cwd(&'a Path),
    /// Prints on stdout.
    Print(&'a str),
    /// Prints on stderr.
    Stderr(&'a str),
    Exit(i32),
}

/// The text of a tool in the platform's scripting language.
fn script_text(steps: &[Step]) -> String {
    let mut text = String::new();
    for step in steps {
        #[cfg(unix)]
        let line = match step {
            Step::Append(p) => format!("echo x >> '{}'", p.display()),
            Step::Cwd(p) => format!("pwd > '{}'", p.display()),
            Step::Print(t) => format!("echo '{t}'"),
            Step::Stderr(t) => format!("echo '{t}' >&2"),
            Step::Exit(n) => format!("exit {n}"),
        };
        #[cfg(windows)]
        let line = match step {
            Step::Append(p) => format!("Add-Content -LiteralPath '{}' -Value x", p.display()),
            Step::Cwd(p) => format!(
                "(Get-Location).Path | Set-Content -LiteralPath '{}'",
                p.display()
            ),
            Step::Print(t) => format!("Write-Output '{t}'"),
            Step::Stderr(t) => format!("[Console]::Error.WriteLine('{t}')"),
            Step::Exit(n) => format!("exit {n}"),
        };
        text.push_str(&line);
        text.push('\n');
    }
    text
}

/// The extension of a tool script.
const EXT: &str = if cfg!(windows) { "ps1" } else { "sh" };

/// The command line that runs the script file `file` (an absolute path or one
/// relative to the repository's root).
fn run_script_argv(file: &str) -> Vec<String> {
    if cfg!(windows) {
        ["pwsh", "-NoProfile", "-File", file]
            .map(String::from)
            .to_vec()
    } else if Path::new(file).is_absolute() {
        // Executable, run directly.
        vec![file.to_string()]
    } else {
        vec!["sh".to_string(), file.to_string()]
    }
}

/// Writes the tool `dir/<name>.<ext>` and returns the argv that runs it by
/// its absolute path.
fn tool(dir: &Path, name: &str, steps: &[Step]) -> Vec<String> {
    let path = write_script(&dir.join(format!("{name}.{EXT}")), steps);
    run_script_argv(&path.to_string_lossy())
}

fn write_script(path: &Path, steps: &[Step]) -> PathBuf {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::write(path, format!("#!/bin/sh\n{}", script_text(steps))).unwrap();
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    #[cfg(not(unix))]
    std::fs::write(path, script_text(steps)).unwrap();
    path.to_path_buf()
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

fn token_command(argv: &[impl AsRef<str>], hosts: &[&str]) -> RepoAuthProfile {
    RepoAuthProfile {
        https: Some(HttpsAuth::TokenCommand {
            argv: argv.iter().map(|s| s.as_ref().to_string()).collect(),
            username: None,
            hosts: hosts.iter().map(|s| s.to_string()).collect(),
            cache_minutes: None,
            confirmed_sha256: None,
            script_sha256: None,
            confirmed_at: None,
        }),
        ssh: None,
    }
}

fn helper_source(helper: &str) -> RepoAuthProfile {
    RepoAuthProfile {
        https: Some(HttpsAuth::Helper {
            helper: helper.into(),
            username: None,
            hosts: Vec::new(),
            confirmed_sha256: None,
            script_sha256: None,
            confirmed_at: None,
        }),
        ssh: None,
    }
}

struct Rig {
    handle: gitpoulpi_daemon::DaemonHandle,
    client: Client,
    repo: RepoId,
    next: u64,
}

impl Rig {
    async fn new(session: &str, fixture: &FixtureRepo) -> Self {
        helper();
        let backend: Arc<dyn gitpoulpi_core::GitBackend> = Arc::new(SystemGitBackend::new());
        let handle = Daemon::new(session).spawn(backend).await.unwrap();
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

    async fn set(
        &mut self,
        profile: RepoAuthProfile,
        confirm: bool,
    ) -> Result<Value, ResponseError> {
        let args = RepoAuthSetArgs { profile, confirm };
        self.call(
            OperationKind::RepoAuthSet,
            serde_json::to_value(args).unwrap(),
        )
        .await
    }

    async fn view(&mut self) -> RepoAuthView {
        serde_json::from_value(
            self.call(OperationKind::RepoAuthGet, Value::Null)
                .await
                .unwrap(),
        )
        .unwrap()
    }
}

fn lines(path: &Path) -> usize {
    std::fs::read_to_string(path).map_or(0, |t| t.lines().count())
}

/// The script prints a token: it reaches the server as the password, the
/// command runs once for two fetches (cached), in the repository's root, and
/// the token is in neither the helper's arguments nor its environment.
#[tokio::test]
async fn a_token_command_serves_its_server_and_is_cached() {
    need_tools!();
    let _turn = exclusive().await;
    let server = AuthHttpRemote::start_accepting();
    let repo = repo_on(&server);
    let scripts = tempfile::tempdir().unwrap();
    let counter = scripts.path().join("count");
    let cwd = scripts.path().join("cwd");
    let tool = tool(
        scripts.path(),
        "token",
        &[Step::Append(&counter), Step::Cwd(&cwd), Step::Print(TOKEN)],
    );
    let mut rig = Rig::new("token-command-main", &repo).await;
    rig.set(token_command(&tool, &[]), true)
        .await
        .expect("a confirmed command is stored");

    rig.fetch().await.expect("the token is accepted");
    rig.fetch().await.expect("the second fetch too");
    let received = server.basic_credentials();
    assert!(!received.is_empty());
    assert!(
        received
            .iter()
            .all(|c| *c == format!("x-access-token:{TOKEN}")),
        "{received:?}"
    );
    assert_eq!(lines(&counter), 1, "the token is cached between operations");
    assert_eq!(
        std::fs::canonicalize(std::fs::read_to_string(&cwd).unwrap().trim()).unwrap(),
        std::fs::canonicalize(repo.root()).unwrap(),
        "the command runs in the repository's root"
    );

    let log = std::fs::read_to_string(&helper().log).unwrap();
    assert!(log.contains("ARGV [--gitpoulpi-git-credential] [get]"));
    assert!(!log.contains(TOKEN), "the token leaked into argv or env");

    // A view shows no command and no secret.
    let view = rig.view().await;
    assert!(view.configured);
    match view.profile.https {
        Some(HttpsAuth::TokenCommand {
            argv,
            confirmed_sha256,
            ..
        }) => {
            assert!(argv.is_empty(), "the command is blanked in a view");
            assert!(confirmed_sha256.is_some());
        }
        other => panic!("{other:?}"),
    }
    rig.handle.shutdown().await.unwrap();
}

/// A server that refuses the token (401) makes git report it rejected
/// (`erase`): the cached token is dropped, so the next transfer runs the
/// command again.
#[tokio::test]
async fn a_refused_token_is_erased_from_the_cache() {
    need_tools!();
    let _turn = exclusive().await;
    let server = AuthHttpRemote::start_rejecting();
    let repo = repo_on(&server);
    let scripts = tempfile::tempdir().unwrap();
    let counter = scripts.path().join("count");
    let tool = tool(
        scripts.path(),
        "token",
        &[Step::Append(&counter), Step::Print(TOKEN)],
    );
    let mut rig = Rig::new("token-command-erase", &repo).await;
    rig.set(token_command(&tool, &[]), true).await.unwrap();

    rig.fetch().await.expect_err("the server refuses the token");
    // Git may ask twice in one transfer (two processes): after an erase the
    // next get runs the command again, so the count is not exactly one.
    let after_first = lines(&counter);
    assert!(after_first >= 1);
    rig.fetch().await.expect_err("and again");
    assert!(
        lines(&counter) > after_first,
        "an erase made the next get run the command"
    );
    rig.handle.shutdown().await.unwrap();
}

/// The command's hosts bind the grant: a remote on another host gets no
/// token, the command is never run for it, and the failure says why.
#[tokio::test]
async fn the_command_serves_only_its_hosts() {
    need_tools!();
    let _turn = exclusive().await;
    let server = AuthHttpRemote::start_accepting();
    let repo = repo_on(&server);
    let scripts = tempfile::tempdir().unwrap();
    let counter = scripts.path().join("count");
    let tool = tool(
        scripts.path(),
        "token",
        &[Step::Append(&counter), Step::Print(TOKEN)],
    );
    let mut rig = Rig::new("token-command-hosts", &repo).await;
    rig.set(token_command(&tool, &["git.corp.example"]), true)
        .await
        .unwrap();
    let error = rig.fetch().await.expect_err("the server is not its host");
    assert_eq!(
        error.kind,
        Some(TypedError::GrantRefused {
            host: server.host(),
            expected: "git.corp.example".into(),
        }),
        "{}",
        error.message
    );
    assert_eq!(lines(&counter), 0, "the command did not run");
    assert!(server.authorizations().is_empty());
    rig.handle.shutdown().await.unwrap();
}

/// A script inside the work tree is hash-pinned: a pull (here, an edit) that
/// changes it refuses the transfer until the person confirms again.
#[tokio::test]
async fn an_edited_in_tree_script_is_refused_until_confirmed_again() {
    need_tools!();
    let _turn = exclusive().await;
    let server = AuthHttpRemote::start_accepting();
    let repo = repo_on(&server);
    let name = format!("get-token.{EXT}");
    let tracked = repo.root().join(&name);
    write_script(&tracked, &[Step::Print(TOKEN)]);
    let argv = run_script_argv(&name);
    let mut rig = Rig::new("token-command-in-tree", &repo).await;
    rig.set(token_command(&argv, &[]), true).await.unwrap();
    rig.fetch().await.expect("the confirmed script runs");

    write_script(&tracked, &[Step::Print("evil-token-998877")]);
    let error = rig.fetch().await.expect_err("the script changed");
    assert_eq!(
        error.kind,
        Some(TypedError::RepoAuthCommandChanged { path: name.clone() }),
        "{}",
        error.message
    );
    assert!(view_says_unavailable(&mut rig).await);
    assert!(
        !server
            .basic_credentials()
            .iter()
            .any(|c| c.contains("evil-token")),
        "the changed script never ran"
    );

    // Confirming the command again pins the new content.
    rig.set(token_command(&argv, &[]), true).await.unwrap();
    rig.fetch().await.expect("confirmed again");
    assert!(server
        .basic_credentials()
        .iter()
        .any(|c| c.contains("evil-token-998877")));
    rig.handle.shutdown().await.unwrap();
}

/// Paths with spaces: a script outside the work tree and one inside it (named
/// relative, in a directory with a space) both run, and the in-tree one is
/// still hash-pinned.
#[tokio::test]
async fn script_paths_with_spaces_run_and_the_in_tree_one_is_pinned() {
    need_tools!();
    let _turn = exclusive().await;
    let server = AuthHttpRemote::start_accepting();
    let repo = repo_on(&server);
    let outside = tempfile::tempdir().unwrap();
    let spaced = outside.path().join("my tools");
    let absolute = tool(&spaced, "get my token", &[Step::Print(TOKEN)]);
    let mut rig = Rig::new("token-command-spaces", &repo).await;
    rig.set(token_command(&absolute, &[]), true).await.unwrap();
    rig.fetch().await.expect("a script path with spaces runs");
    assert!(server
        .basic_credentials()
        .iter()
        .any(|c| *c == format!("x-access-token:{TOKEN}")));

    let name = format!("my scripts/get token.{EXT}");
    let tracked = repo.root().join(&name);
    write_script(&tracked, &[Step::Print(TOKEN)]);
    let relative = run_script_argv(&name);
    rig.set(token_command(&relative, &[]), true).await.unwrap();
    rig.fetch().await.expect("an in-tree path with spaces runs");
    write_script(&tracked, &[Step::Print("evil-token-998877")]);
    let error = rig.fetch().await.expect_err("the script changed");
    assert_eq!(
        error.kind,
        Some(TypedError::RepoAuthCommandChanged { path: name }),
        "{}",
        error.message
    );
    rig.handle.shutdown().await.unwrap();
}

/// A `.cmd` file: through `cmd /c` it runs (CreateProcess starts executables,
/// a batch file needs the interpreter), and named bare as the program it runs
/// too (the standard library starts a batch file through `cmd.exe`).
#[cfg(windows)]
#[tokio::test]
async fn a_cmd_script_runs_through_cmd_or_bare() {
    let _turn = exclusive().await;
    let server = AuthHttpRemote::start_accepting();
    let repo = repo_on(&server);
    let dir = tempfile::tempdir().unwrap();
    let spaced = dir.path().join("cmd tools");
    std::fs::create_dir_all(&spaced).unwrap();
    let script = spaced.join("get token.cmd");
    std::fs::write(&script, format!("@echo off\r\necho {TOKEN}\r\n")).unwrap();
    let mut rig = Rig::new("token-command-cmd", &repo).await;

    let through_cmd = vec![
        "cmd".to_string(),
        "/c".to_string(),
        script.to_string_lossy().into_owned(),
    ];
    rig.set(token_command(&through_cmd, &[]), true)
        .await
        .unwrap();
    rig.fetch().await.expect("cmd /c <script.cmd> runs");
    assert!(server
        .basic_credentials()
        .iter()
        .any(|c| *c == format!("x-access-token:{TOKEN}")));

    let bare = vec![script.to_string_lossy().into_owned()];
    rig.set(token_command(&bare, &[]), true).await.unwrap();
    rig.fetch().await.expect("a bare .cmd runs");
    rig.handle.shutdown().await.unwrap();
}

async fn view_says_unavailable(rig: &mut Rig) -> bool {
    rig.view().await.unavailable.is_some()
}

/// A non-zero exit refuses the transfer with the command's stderr, redacted:
/// neither a token-shaped value nor the exact registered one survives.
#[tokio::test]
async fn a_failing_command_reads_with_a_redacted_stderr() {
    need_tools!();
    let _turn = exclusive().await;
    let server = AuthHttpRemote::start_accepting();
    let repo = repo_on(&server);
    let scripts = tempfile::tempdir().unwrap();
    let login_failed = format!("login failed for {TOKEN}");
    let tool = tool(
        scripts.path(),
        "token",
        &[
            Step::Stderr(&login_failed),
            Step::Stderr("Authorization: Bearer abcdef1234567890xyz"),
            Step::Stderr("vault is sealed"),
            Step::Exit(3),
        ],
    );
    gitpoulpi_core::redact_register_secret(TOKEN);
    let mut rig = Rig::new("token-command-failing", &repo).await;
    rig.set(token_command(&tool, &[]), true).await.unwrap();
    let error = rig.fetch().await.expect_err("the command failed");
    match &error.kind {
        Some(TypedError::TokenCommandFailed { exit, stderr_tail }) => {
            assert_eq!(*exit, Some(3));
            assert!(stderr_tail.contains("vault is sealed"), "{stderr_tail}");
            assert!(!stderr_tail.contains(TOKEN), "{stderr_tail}");
            assert!(
                !stderr_tail.contains("abcdef1234567890xyz"),
                "{stderr_tail}"
            );
        }
        other => panic!("{other:?}: {}", error.message),
    }
    assert!(!error.message.contains(TOKEN) && !error.message.contains("abcdef1234567890xyz"));
    assert!(server.authorizations().is_empty());
    rig.handle.shutdown().await.unwrap();
}

/// The submodule update runs under the same grant: a token command that fails
/// reads as that there too, not as git's "terminal prompts disabled".
#[tokio::test]
async fn a_failing_command_reads_the_same_in_a_submodule_update() {
    need_tools!();
    let _turn = exclusive().await;
    let server = AuthHttpRemote::start_accepting();
    let repo = repo_on(&server);
    let head = repo.git(&["rev-parse", "HEAD"]);
    std::fs::write(
        repo.root().join(".gitmodules"),
        format!(
            "[submodule \"sub\"]\n\tpath = sub\n\turl = {}\n",
            server.url()
        ),
    )
    .unwrap();
    repo.git(&[
        "update-index",
        "--add",
        "--cacheinfo",
        &format!("160000,{},sub", head.trim()),
    ]);
    let scripts = tempfile::tempdir().unwrap();
    let tool = tool(
        scripts.path(),
        "token",
        &[Step::Stderr("vault is sealed"), Step::Exit(3)],
    );
    let mut rig = Rig::new("token-command-submodule", &repo).await;
    rig.set(token_command(&tool, &[]), true).await.unwrap();
    let error = rig
        .call(
            OperationKind::SubmoduleUpdate,
            json!({ "opts": { "paths": [], "init": true, "recursive": false } }),
        )
        .await
        .expect_err("the command failed");
    match &error.kind {
        Some(TypedError::TokenCommandFailed { exit, stderr_tail }) => {
            assert_eq!(*exit, Some(3));
            assert!(stderr_tail.contains("vault is sealed"), "{stderr_tail}");
        }
        other => panic!("{other:?}: {}", error.message),
    }
    rig.handle.shutdown().await.unwrap();
}

/// A helper source: git runs the confirmed helper as is. A stored hash that no
/// longer matches the helper refuses the transfer, as does a changed in-tree
/// helper script.
#[tokio::test]
async fn a_helper_source_needs_its_confirmed_hash() {
    need_tools!();
    let _turn = exclusive().await;
    let server = AuthHttpRemote::start_accepting();
    let repo = repo_on(&server);
    let name = format!("cred.{EXT}");
    let tracked = repo.root().join(&name);
    #[cfg(unix)]
    let (text, helper_text) = (
        "cat >/dev/null; echo username=hh; echo password=pp-123456\n",
        format!("!sh {name}"),
    );
    #[cfg(windows)]
    let (text, helper_text) = (
        "$null = [Console]::In.ReadToEnd(); Write-Output 'username=hh'; Write-Output 'password=pp-123456'\n",
        format!("!pwsh -NoProfile -File {name}"),
    );
    std::fs::write(&tracked, text).unwrap();
    let helper_text = helper_text.as_str();
    let mut rig = Rig::new("token-command-helper", &repo).await;
    rig.set(helper_source(helper_text), true).await.unwrap();
    rig.fetch().await.expect("the helper answers");
    assert!(server
        .basic_credentials()
        .iter()
        .any(|c| c == "hh:pp-123456"));

    // The stored hash is not the helper's any more (a hand-edited store).
    let file = gitpoulpi_config::ConfigPaths::discover()
        .unwrap()
        .private_dir()
        .join("repo-auth.toml");
    let original = std::fs::read_to_string(&file).unwrap();
    let forged: String = original
        .lines()
        .map(|l| {
            if l.trim_start().starts_with("confirmed_sha256") {
                "confirmed_sha256 = \"00\"".to_string()
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&file, forged).unwrap();
    let error = rig.fetch().await.expect_err("the hash does not match");
    assert!(
        matches!(&error.kind, Some(TypedError::RepoAuthUnavailable { reason }) if reason.contains("confirm")),
        "{error:?}"
    );
    std::fs::write(&file, &original).unwrap();
    rig.fetch().await.expect("the original store works again");

    std::fs::write(&tracked, "echo password=other\n").unwrap();
    let error = rig.fetch().await.expect_err("the helper script changed");
    assert!(
        matches!(&error.kind, Some(TypedError::RepoAuthCommandChanged { path }) if *path == name),
        "{error:?}"
    );
    rig.handle.shutdown().await.unwrap();
}

/// Only a confirmed request stores a command. Editing the other fields of the
/// confirmed command needs no new confirmation, changing the command does.
#[tokio::test]
async fn an_unconfirmed_request_cannot_store_a_command() {
    let _turn = exclusive().await;
    let server = AuthHttpRemote::start_accepting();
    let repo = repo_on(&server);
    let mut rig = Rig::new("token-command-unconfirmed", &repo).await;

    let error = rig
        .set(token_command(&["tool"], &[]), false)
        .await
        .expect_err("an agent cannot confirm");
    assert!(
        matches!(error.kind, Some(TypedError::NeedsConfirmation(_))),
        "{error:?}"
    );
    let error = rig
        .set(helper_source("!tool"), false)
        .await
        .expect_err("nor a helper");
    assert!(matches!(error.kind, Some(TypedError::NeedsConfirmation(_))));
    assert!(!rig.view().await.configured, "nothing was stored");

    // Hashes a client writes are not believed: the daemon computes its own.
    let mut forged = token_command(&["tool"], &[]);
    if let Some(HttpsAuth::TokenCommand {
        confirmed_sha256, ..
    }) = forged.https.as_mut()
    {
        *confirmed_sha256 = Some("00".into());
    }
    rig.set(forged, false).await.expect_err("a forged hash");
    assert!(!rig.view().await.configured);

    rig.set(token_command(&["tool"], &[]), true).await.unwrap();
    // Same command, another host list: no confirmation needed.
    rig.set(token_command(&["tool"], &["a.example"]), false)
        .await
        .expect("the confirmed command is kept");
    // Another command: it needs one.
    rig.set(token_command(&["other-tool"], &["a.example"]), false)
        .await
        .expect_err("a changed command needs a person");
    rig.handle.shutdown().await.unwrap();
}
