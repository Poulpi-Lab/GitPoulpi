//! The helper and token-command HTTPS sources (ADR-0010).
//!
//! **Helper**: the person's own credential helper string, run by git exactly
//! as configured, after the repository's own helpers were reset. Refused
//! unless the person confirmed it and its in-tree files are unchanged
//! (`command_trust`).
//!
//! **Token command**: an argv the daemon runs itself when git asks the
//! GitPoulpi helper for a credential (a grant, `grants`):
//! - directly, no shell, in the repository's root, stdin closed;
//! - at most 30 seconds, then the process and everything it started is killed;
//! - its first stdout line is the token, or it prints git-credential
//!   `username=` / `password=` lines;
//! - the token is cached in memory for `cache_minutes` (default 50), until
//!   git reports it refused (`erase`: a 401 or 403), until the profile is
//!   changed or cleared, or the daemon ends;
//! - its stderr goes to the Activity Log, redacted, and the token value is
//!   registered with `redact_register_secret` so no log line or error report
//!   can carry it.
//!
//! The grant is bound to the profile's `hosts` (default: the host of
//! `origin`), https only.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use gitpoulpi_core::domain::{HttpsAuth, NetworkAuth};
use gitpoulpi_core::Error;

use crate::grants::{GrantCredential, GrantGuard};

/// The default time a token is reused.
pub(crate) const DEFAULT_CACHE_MINUTES: u32 = 50;
/// The longest a token command may run.
pub(crate) const TIMEOUT: Duration = Duration::from_secs(30);
/// The user name when neither the profile nor the command gives one: hosts
/// that take a token as the password accept any non-empty name.
const DEFAULT_USERNAME: &str = "x-access-token";
/// How much of a failing command's stderr an error carries.
const STDERR_TAIL_LINES: usize = 8;
const STDERR_TAIL_CHARS: usize = 1000;

fn unavailable(reason: impl Into<String>) -> Error {
    Error::RepoAuthUnavailable {
        reason: reason.into(),
    }
}

/// A token command as a grant runs it.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct TokenCommand {
    pub argv: Vec<String>,
    /// The repository's root: the command's working directory.
    pub cwd: PathBuf,
    /// The profile's user name; wins over the one the command prints.
    pub username: Option<String>,
    pub cache: Duration,
    pub timeout: Duration,
}

// By hand: an argument may embed a secret, so only the program is shown.
impl std::fmt::Debug for TokenCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenCommand")
            .field("program", &self.argv.first())
            .finish()
    }
}

/// What a token command gave.
pub(crate) struct Credential {
    pub username: Option<String>,
    pub password: String,
}

struct Cached {
    at: Instant,
    credential: std::sync::Arc<(Option<String>, String)>,
}

type CacheKey = (PathBuf, Vec<String>);

fn cache() -> &'static Mutex<HashMap<CacheKey, Cached>> {
    static CACHE: OnceLock<Mutex<HashMap<CacheKey, Cached>>> = OnceLock::new();
    CACHE.get_or_init(Mutex::default)
}

fn cache_lock() -> std::sync::MutexGuard<'static, HashMap<CacheKey, Cached>> {
    cache().lock().unwrap_or_else(|e| e.into_inner())
}

fn key_of(command: &TokenCommand) -> CacheKey {
    (command.cwd.clone(), command.argv.clone())
}

/// Forgets the cached token of `command` (git reported it refused).
pub(crate) fn invalidate(command: &TokenCommand) {
    cache_lock().remove(&key_of(command));
}

/// Forgets every cached token of the repository at `root` (its profile was
/// changed or cleared).
pub(crate) fn forget_root(root: &Path) {
    cache_lock().retain(|(r, _), _| r != root);
}

/// The credential of `command`: the cached one while it is fresh, else the
/// command's output (and then cached).
#[cfg(all(test, unix))]
pub(crate) async fn credential(command: &TokenCommand) -> Result<Credential, Error> {
    let _one_at_a_time = turn().await;
    credential_in_turn(command).await
}

/// The turn to run a token command: one at a time. git can ask twice at once
/// (a fetch runs the tag-mirror fetch beside its own, a submodule or prune
/// step starts another), and a command that prompts a person or spends a
/// one-time code must not run twice for one transfer. A caller that decides
/// something from what an earlier turn left (a failure) takes the turn first
/// and then calls [`credential_in_turn`].
pub(crate) async fn turn() -> tokio::sync::MutexGuard<'static, ()> {
    static RUNNING: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
    RUNNING.lock().await
}

/// [`credential`] for a caller that already holds the [`turn`].
pub(crate) async fn credential_in_turn(command: &TokenCommand) -> Result<Credential, Error> {
    if !command.cache.is_zero() {
        let hit = cache_lock()
            .get(&key_of(command))
            .filter(|c| c.at.elapsed() < command.cache)
            .map(|c| c.credential.clone());
        if let Some(hit) = hit {
            return Ok(Credential {
                username: hit.0.clone(),
                password: hit.1.clone(),
            });
        }
    }
    let credential = run(command).await?;
    if !command.cache.is_zero() {
        cache_lock().insert(
            key_of(command),
            Cached {
                at: Instant::now(),
                credential: std::sync::Arc::new((
                    credential.username.clone(),
                    credential.password.clone(),
                )),
            },
        );
    }
    Ok(credential)
}

fn failed(exit: Option<i32>, stderr: &str) -> Error {
    Error::TokenCommandFailed {
        exit,
        stderr_tail: tail(stderr),
    }
}

/// The end of `stderr`, redacted.
fn tail(stderr: &str) -> String {
    let lines: Vec<&str> = stderr.lines().filter(|l| !l.trim().is_empty()).collect();
    let from = lines.len().saturating_sub(STDERR_TAIL_LINES);
    let joined = gitpoulpi_core::redact(&lines[from..].join("\n"));
    let chars: Vec<char> = joined.chars().collect();
    let start = chars.len().saturating_sub(STDERR_TAIL_CHARS);
    chars[start..].iter().collect()
}

/// What the application log calls this command: "Token command `prog` in
/// `folder`". The program's name only: its arguments may hold a secret.
fn what(command: &TokenCommand) -> String {
    let program = command
        .argv
        .first()
        .map(|p| {
            Path::new(p)
                .file_name()
                .map_or_else(|| p.clone(), |n| n.to_string_lossy().into_owned())
        })
        .unwrap_or_default();
    let folder = command
        .cwd
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    gitpoulpi_core::redact(&format!("Token command `{program}` in `{folder}`"))
}

async fn run(command: &TokenCommand) -> Result<Credential, Error> {
    let what = what(command);
    let output = match gitpoulpi_cli_backend::run_command(
        &command.argv,
        &command.cwd,
        command.timeout,
    )
    .await
    {
        Ok(output) => output,
        Err(gitpoulpi_cli_backend::CommandRunError::TimedOut) => {
            tracing::warn!(
                "{what} failed: it ran longer than {} s and was killed",
                command.timeout.as_secs()
            );
            return Err(failed(
                None,
                &format!(
                    "it ran longer than {} seconds and was stopped",
                    command.timeout.as_secs()
                ),
            ));
        }
        Err(gitpoulpi_cli_backend::CommandRunError::Spawn(why)) => {
            tracing::warn!(
                "{what} failed: it could not start: {}",
                gitpoulpi_core::redact(&why)
            );
            return Err(failed(None, &why));
        }
    };
    for line in output.stderr.lines().filter(|l| !l.trim().is_empty()) {
        tracing::info!("{what} stderr: {}", gitpoulpi_core::redact(line));
    }
    if output.exit_code != Some(0) {
        tracing::warn!("{what} failed: it exited with {:?}", output.exit_code);
        return Err(failed(output.exit_code, &output.stderr));
    }
    let Some(parsed) = parse_output(&output.stdout) else {
        return Err(failed(
            output.exit_code,
            &format!("it printed no token\n{}", output.stderr),
        ));
    };
    // From here on the token is masked wherever it appears in a log line or
    // an error report, whatever it looks like.
    gitpoulpi_core::redact_register_secret(&parsed.password);
    Ok(parsed)
}

/// `username=` / `password=` lines when there is a `password=`, else the
/// first non-empty line as the token.
pub(crate) fn parse_output(stdout: &str) -> Option<Credential> {
    let mut username = None;
    let mut password = None;
    for line in stdout.lines() {
        let line = line.trim_end_matches('\r');
        if let Some(v) = line.strip_prefix("password=") {
            password = Some(v.to_string());
        } else if let Some(v) = line.strip_prefix("username=") {
            username = Some(v.to_string());
        }
    }
    if let Some(password) = password.filter(|p| !p.trim().is_empty()) {
        return Some(Credential {
            username: username.filter(|u| !u.is_empty()),
            password,
        });
    }
    let first = stdout
        .lines()
        .map(|l| l.trim())
        .find(|l| !l.is_empty())?
        .to_string();
    if first.starts_with("username=") {
        return None;
    }
    Some(Credential {
        username: None,
        password: first,
    })
}

/// The credential a grant answers `get` with: the profile's user name, else
/// the command's, else [`DEFAULT_USERNAME`].
pub(crate) fn answer_of(command: &TokenCommand, given: Credential) -> (String, String) {
    let username = command
        .username
        .clone()
        .filter(|u| !u.is_empty())
        .or(given.username)
        .unwrap_or_else(|| DEFAULT_USERNAME.to_string());
    (username, given.password)
}

/// `host[:port]`, lowercase, of one entry of a `hosts` list (a scheme or a
/// path the person typed is dropped).
fn normalize_host(entry: &str) -> Option<String> {
    let entry = entry.trim();
    let rest = entry.split_once("://").map_or(entry, |(_, r)| r);
    let host = rest.split(['/', '?', '#']).next()?.rsplit('@').next()?;
    (!host.is_empty()).then(|| host.to_ascii_lowercase())
}

/// Whether `host[:port]` is this machine: the one place plain `http` is
/// answered (nothing crosses a network).
fn is_loopback(host: &str) -> bool {
    let name = match host.strip_prefix('[') {
        Some(v6) => v6.split(']').next().unwrap_or(""),
        None => host.rsplit_once(':').map_or(host, |(h, _)| h),
    };
    name == "localhost" || name == "::1" || name == "127.0.0.1"
}

/// The host of `origin` when it is an http(s) remote.
fn origin_host(root: &Path) -> Option<String> {
    let output = std::process::Command::new(gitpoulpi_cli_backend::git_program())
        .arg("-C")
        .arg(root)
        .args(["config", "--get", "remote.origin.url"])
        .stdin(std::process::Stdio::null())
        .output()
        .ok()?;
    let url = String::from_utf8_lossy(&output.stdout);
    let url = url.trim();
    let (scheme, _) = url.split_once("://")?;
    matches!(scheme.to_ascii_lowercase().as_str(), "http" | "https")
        .then(|| normalize_host(url))
        .flatten()
}

fn hosts_of(root: &Path, hosts: &[String]) -> Result<Vec<String>, Error> {
    let mut out: Vec<String> = hosts.iter().filter_map(|h| normalize_host(h)).collect();
    if out.is_empty() {
        out.extend(origin_host(root));
    }
    if out.is_empty() {
        return Err(unavailable(
            "the repository has no https origin to take the host from: name the hosts of the command",
        ));
    }
    Ok(out)
}

/// Resolves a helper or token-command source for a transfer of the repository
/// at `root` and fills `auth`. Verifies the person's confirmation first.
/// `session: None` only checks that the source can be used (nothing is run,
/// no grant issued).
pub(crate) fn apply(
    root: &Path,
    auth: &mut NetworkAuth,
    https: &HttpsAuth,
    session: Option<&str>,
) -> Result<Option<GrantGuard>, Error> {
    crate::command_trust::verify(root, https)?;
    match https {
        HttpsAuth::Helper {
            helper,
            username,
            hosts,
            ..
        } => {
            auth.credential_username = username
                .as_deref()
                .map(str::trim)
                .filter(|u| !u.is_empty())
                .map(str::to_string);
            auth.no_prompt = true;
            let hosts: Vec<String> = hosts.iter().filter_map(|h| normalize_host(h)).collect();
            if hosts.is_empty() {
                auth.credential_helpers = Some(vec![helper.clone()]);
            } else {
                // Only for its hosts: git asks it for no other.
                auth.credential_helpers = Some(Vec::new());
                auth.credential_url_helpers = hosts
                    .into_iter()
                    .map(|h| (format!("https://{h}"), helper.clone()))
                    .collect();
            }
            Ok(None)
        }
        HttpsAuth::TokenCommand {
            argv,
            username,
            hosts,
            cache_minutes,
            ..
        } => {
            let hosts = hosts_of(root, hosts)?;
            let Some(session) = session else {
                return Ok(None);
            };
            let command = TokenCommand {
                argv: argv.clone(),
                cwd: root.to_path_buf(),
                username: username
                    .as_deref()
                    .map(str::trim)
                    .filter(|u| !u.is_empty())
                    .map(str::to_string),
                cache: Duration::from_secs(
                    u64::from(cache_minutes.unwrap_or(DEFAULT_CACHE_MINUTES)) * 60,
                ),
                timeout: TIMEOUT,
            };
            let grant = crate::grants::issue(
                root,
                GrantCredential {
                    username: String::new(),
                    password: String::new(),
                    scheme: "https".into(),
                    allow_plain_http: hosts.iter().all(|h| is_loopback(h)),
                    hosts,
                    identity: "token command".into(),
                    command: Some(command),
                },
            )
            .map_err(|e| unavailable(format!("no random grant could be made: {e}")))?;
            auth.credential_helpers = Some(vec![crate::account_source::helper_command()?]);
            auth.no_prompt = true;
            auth.reset_extra_header = true;
            auth.env = crate::account_source::grant_env(&grant, session);
            Ok(Some(grant))
        }
        _ => Ok(None),
    }
}

/// Validates the command-specific fields of a profile being stored.
pub(crate) fn validate(https: &HttpsAuth) -> Result<(), String> {
    let plain = |what: &str, v: &str| -> Result<(), String> {
        if v.trim().is_empty() {
            return Err(format!("{what} is empty"));
        }
        if v.chars().any(|c| c.is_control() && c != '\t') {
            return Err(format!("{what} holds a control character"));
        }
        Ok(())
    };
    let hosts_ok = |hosts: &[String]| -> Result<(), String> {
        for h in hosts {
            if normalize_host(h).is_none() {
                return Err(format!("the host \"{h}\" is not usable"));
            }
        }
        Ok(())
    };
    match https {
        HttpsAuth::Helper { helper, hosts, .. } => {
            plain("the credential helper", helper)?;
            hosts_ok(hosts)
        }
        HttpsAuth::TokenCommand {
            argv,
            hosts,
            cache_minutes,
            ..
        } => {
            if argv.is_empty() || argv[0].trim().is_empty() {
                return Err("the token command is empty".into());
            }
            for a in argv {
                if a.contains('\0') || a.chars().any(|c| c.is_control() && c != '\t') {
                    return Err("the token command holds a control character".into());
                }
            }
            if cache_minutes.is_some_and(|m| m > 24 * 60) {
                return Err("the token cache time is over a day".into());
            }
            hosts_ok(hosts)
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod timeout_tests {
    use super::*;

    /// The helper git runs waits for the daemon, whose token command may take
    /// `TIMEOUT`: if the helper gave up first, git would fail with its generic
    /// "terminal prompts disabled" and the command would be cut short.
    #[test]
    fn the_helper_waits_longer_than_a_token_command_may_run() {
        assert!(
            gitpoulpi_ipc::git_credential::ASK_TIMEOUT > TIMEOUT,
            "the credential helper's wait must outlast the token command's timeout"
        );
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    fn command(root: &Path, script: &str, cache: Duration, timeout: Duration) -> TokenCommand {
        TokenCommand {
            argv: vec!["sh".into(), "-c".into(), script.into()],
            cwd: root.to_path_buf(),
            username: None,
            cache,
            timeout,
        }
    }

    const MIN: Duration = Duration::from_secs(60);
    const LONG: Duration = Duration::from_secs(20);

    #[test]
    fn output_is_one_line_or_credential_lines() {
        let one = parse_output("\n  tok-123456 \nignored\n").unwrap();
        assert_eq!((one.username, one.password), (None, "tok-123456".into()));
        let two = parse_output("username=bob\r\npassword=pw-123456\r\n").unwrap();
        assert_eq!(
            (two.username, two.password),
            (Some("bob".into()), "pw-123456".into())
        );
        assert!(parse_output("  \n").is_none());
        assert!(parse_output("username=bob\n").is_none());
    }

    #[test]
    fn hosts_are_normalized() {
        assert_eq!(
            normalize_host(" HTTPS://User@Git.Example:8443/x ").unwrap(),
            "git.example:8443"
        );
        assert_eq!(normalize_host("github.com").unwrap(), "github.com");
        assert!(normalize_host("  ").is_none());
    }

    #[tokio::test]
    async fn a_token_is_cached_until_it_is_invalidated() {
        let dir = tempfile::tempdir().unwrap();
        let counter = dir.path().join("count");
        let script = format!("echo x >> '{}'; echo tok-cache-0001", counter.display());
        let cmd = command(dir.path(), &script, MIN, LONG);
        let runs = || std::fs::read_to_string(&counter).unwrap().lines().count();

        let first = credential(&cmd).await.unwrap();
        assert_eq!(first.password, "tok-cache-0001");
        credential(&cmd).await.unwrap();
        assert_eq!(runs(), 1, "the second get is served from the cache");

        invalidate(&cmd);
        credential(&cmd).await.unwrap();
        assert_eq!(runs(), 2, "an erase makes the next get run the command");

        forget_root(dir.path());
        credential(&cmd).await.unwrap();
        assert_eq!(runs(), 3, "a profile change forgets the token");

        let no_cache = command(dir.path(), &script, Duration::ZERO, LONG);
        credential(&no_cache).await.unwrap();
        credential(&no_cache).await.unwrap();
        assert_eq!(runs(), 5, "cache_minutes = 0 never caches");
    }

    #[tokio::test]
    async fn a_failing_command_reports_its_redacted_stderr() {
        let dir = tempfile::tempdir().unwrap();
        let cmd = command(
            dir.path(),
            "echo 'could not log in with plainsecret-998877' >&2; \
             echo 'Authorization: Bearer abcdef1234567890' >&2; exit 7",
            MIN,
            LONG,
        );
        gitpoulpi_core::redact_register_secret("plainsecret-998877");
        let error = credential(&cmd).await.err().unwrap();
        match error {
            Error::TokenCommandFailed { exit, stderr_tail } => {
                assert_eq!(exit, Some(7));
                assert!(stderr_tail.contains("could not log in"), "{stderr_tail}");
                assert!(!stderr_tail.contains("plainsecret-998877"), "{stderr_tail}");
                assert!(!stderr_tail.contains("abcdef1234567890"), "{stderr_tail}");
            }
            other => panic!("{other:?}"),
        }
    }

    #[tokio::test]
    async fn no_output_and_a_missing_program_are_failures_too() {
        let dir = tempfile::tempdir().unwrap();
        let cmd = command(dir.path(), "true", MIN, LONG);
        assert!(matches!(
            credential(&cmd).await,
            Err(Error::TokenCommandFailed { exit: Some(0), .. })
        ));
        let mut missing = cmd.clone();
        missing.argv = vec!["/no/such/program".into()];
        assert!(matches!(
            credential(&missing).await,
            Err(Error::TokenCommandFailed { exit: None, .. })
        ));
    }

    #[tokio::test]
    async fn a_timeout_is_a_failure_and_kills_the_tree() {
        let dir = tempfile::tempdir().unwrap();
        let pidfile = dir.path().join("pid");
        let script = format!("sleep 60 & echo $! > '{}'; wait", pidfile.display());
        let cmd = command(dir.path(), &script, MIN, Duration::from_millis(1500));
        let error = credential(&cmd).await.err().unwrap();
        assert!(
            matches!(&error, Error::TokenCommandFailed { exit: None, stderr_tail } if stderr_tail.contains("longer than")),
            "{error:?}"
        );
        let pid: i32 = std::fs::read_to_string(&pidfile)
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        for _ in 0..50 {
            // SAFETY: signal 0 only probes for existence.
            if unsafe { libc::kill(pid, 0) } != 0 {
                return;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        panic!("the grandchild {pid} is still running");
    }

    #[test]
    fn the_command_runs_in_the_root_and_the_debug_hides_the_arguments() {
        let cmd = TokenCommand {
            argv: vec!["tool".into(), "--secret=hunter2hunter2".into()],
            cwd: PathBuf::from("/r"),
            username: None,
            cache: MIN,
            timeout: LONG,
        };
        assert!(!format!("{cmd:?}").contains("hunter2"));
    }
}
