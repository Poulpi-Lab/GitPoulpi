#![cfg_attr(not(feature = "hosting"), allow(dead_code))]
//! Credential grants (ADR-0010): how an account's token reaches git without
//! sitting in a process argument or an environment variable.
//!
//! Before a network operation of a repository that authenticates with a
//! GitPoulpi account, the daemon issues a **grant**: a random 128-bit id bound
//! to one credential and one host. The git child gets only the id (and the
//! session name) in its environment, plus a credential helper that is a
//! GitPoulpi binary (`gitpoulpi_ipc::git_credential`). The helper asks the
//! daemon, over the authenticated IPC, to answer git's `get`.
//!
//! The daemon answers only when
//! - the grant is alive (the guard of the operation has not been dropped:
//!   success, failure and cancel all drop it),
//! - the protocol is the one the account's recorded origin has (so an `https`
//!   account never answers `http`), and
//! - `host[:port]` is the account's recorded origin, character for
//!   character after lowercasing.
//!
//! Anything else gets an empty answer and is remembered as a refusal, which is
//! how the transfer's failure is read afterwards (`Error::GrantRefused`): a
//! `url.<x>.insteadOf` in the repository's own config can point a remote at
//! any host, and this is what keeps the token from following it.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use gitpoulpi_ipc::git_credential::{
    CredentialAction, CredentialGrantAnswer, CredentialGrantRequest,
};

/// What a grant answers with, and for whom.
#[derive(PartialEq, Eq)]
pub(crate) struct GrantCredential {
    pub username: String,
    pub password: String,
    /// `http` or `https`: the only protocol the grant answers.
    pub scheme: String,
    /// `host` or `host:port`, lowercase: the only hosts the grant answers.
    pub hosts: Vec<String>,
    /// Whether `http` is answered too (only a token command whose hosts are
    /// all loopback: nothing crosses a network).
    pub allow_plain_http: bool,
    /// A token command that makes the credential when git asks (the
    /// `username` and `password` above are then empty).
    pub command: Option<crate::token_command::TokenCommand>,
    /// What a connection test says worked: `account work (token, user x)`.
    pub identity: String,
}

// By hand: a derived `Debug` would print the password.
impl std::fmt::Debug for GrantCredential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GrantCredential")
            .field("username", &self.username)
            .field("password", &"***")
            .field("scheme", &self.scheme)
            .field("hosts", &self.hosts)
            .field("command", &self.command)
            .finish()
    }
}

struct Entry {
    credential: GrantCredential,
    /// Whether the helper ever asked the daemon about this grant. A grant that
    /// was never asked about means the helper could not reach the daemon.
    asked: bool,
    /// The user-only file that tells the helper where this daemon is; removed
    /// with the grant.
    endpoint_file: Option<PathBuf>,
    root: PathBuf,
    /// Seq of the issue, so "the latest grant of a repository" is defined.
    seq: u64,
    /// Operations of this repository holding the grant right now (see
    /// [`issue`]); it is revoked when the last one lets go.
    holders: usize,
    accepted: bool,
    token_refused: bool,
    /// `(host git asked for, host the grant is bound to)` of the last `get`
    /// that was refused.
    refused: Option<(String, String)>,
    /// `(exit, stderr tail)` of the last token command that failed.
    command_failure: Option<(Option<i32>, String)>,
}

#[derive(Default)]
struct Registry {
    next: u64,
    grants: HashMap<String, Entry>,
}

fn registry() -> &'static Mutex<Registry> {
    static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();
    REGISTRY.get_or_init(Mutex::default)
}

fn lock() -> std::sync::MutexGuard<'static, Registry> {
    registry().lock().unwrap_or_else(|e| e.into_inner())
}

/// A live grant. Dropping it revokes it, so the grant is dead when the
/// operation that holds the guard ends, however it ends.
#[must_use = "the grant is revoked as soon as the guard is dropped"]
pub(crate) struct GrantGuard {
    id: String,
}

impl GrantGuard {
    /// The id git's child gets in its environment. Not a secret by itself:
    /// it answers only over the authenticated IPC, only while alive.
    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    /// `(host asked for, host expected)` of the last refused `get`.
    pub(crate) fn refused(&self) -> Option<(String, String)> {
        lock().grants.get(&self.id)?.refused.clone()
    }
}

impl GrantGuard {
    /// Why the grant's token command failed, if it did (the transfer then
    /// failed for want of a credential).
    pub(crate) fn command_failure(&self) -> Option<gitpoulpi_core::Error> {
        let (exit, stderr_tail) = lock().grants.get(&self.id)?.command_failure.clone()?;
        Some(gitpoulpi_core::Error::TokenCommandFailed { exit, stderr_tail })
    }
}

impl GrantGuard {
    /// `error` as the person should read it: when the transfer failed after
    /// the grant refused a host, or after the token command gave no token,
    /// that is the reason (not git's "terminal prompts disabled"). Every path
    /// that runs a transfer under a grant words its failure through this.
    pub(crate) fn explain(&self, error: gitpoulpi_core::Error) -> gitpoulpi_core::Error {
        self.reads_as(&error.to_string()).unwrap_or(error)
    }

    /// The reason behind a failure whose text is `message`, if the grant knows
    /// one: a refused host, a token command that gave no token, or a helper
    /// that never reached the daemon (git's "could not read Username ...
    /// terminal prompts disabled" and no request for this grant ever arrived).
    pub(crate) fn reads_as(&self, message: &str) -> Option<gitpoulpi_core::Error> {
        if let Some((host, expected)) = self.refused() {
            Some(gitpoulpi_core::Error::GrantRefused { host, expected })
        } else if let Some(command_error) = self.command_failure() {
            Some(command_error)
        } else if is_no_credential_message(message) && !self.asked() {
            Some(gitpoulpi_core::Error::CredentialHelperUnreachable)
        } else {
            None
        }
    }

    fn asked(&self) -> bool {
        lock().grants.get(&self.id).is_none_or(|e| e.asked)
    }

    /// Tells the helper where this grant's daemon is: writes the user-only
    /// endpoint file (once per grant) and returns its path. `None` when the
    /// daemon of `session` is unknown here or the file cannot be written; the
    /// helper then falls back to the session's published port file.
    pub(crate) fn endpoint_file(&self, session: &str) -> Option<PathBuf> {
        let endpoint = endpoints().get(session)?.1.clone();
        let paths = gitpoulpi_config::ConfigPaths::discover()?;
        let dir = paths.ipc_port_file(session).parent()?.to_path_buf();
        let mut registry = lock();
        let entry = registry.grants.get_mut(&self.id)?;
        if let Some(file) = &entry.endpoint_file {
            return Some(file.clone());
        }
        sweep_stale_endpoint_files(&dir);
        gitpoulpi_config::secure_fs::create_private_dir_all(&dir).ok()?;
        let file = dir.join(format!("{session}.grant-{}.cred", self.id));
        gitpoulpi_config::secure_fs::write_private(
            &file,
            gitpoulpi_ipc::git_credential::endpoint_file_text(&endpoint).as_bytes(),
        )
        .ok()?;
        entry.endpoint_file = Some(file.clone());
        Some(file)
    }
}

/// git's wording when it needed a credential, got none and may not prompt.
pub(crate) fn is_no_credential_message(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("terminal prompts disabled")
        || lower.contains("could not read username")
        || lower.contains("could not read password")
}

/// Endpoint files a crashed daemon left behind (a day old): they name a dead
/// daemon's token, useless but not worth keeping.
fn sweep_stale_endpoint_files(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !(name.contains(".grant-") && name.ends_with(".cred")) {
            continue;
        }
        let old = entry
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.elapsed().ok())
            .is_some_and(|age| age > std::time::Duration::from_secs(24 * 3600));
        if old {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

type EndpointMap = HashMap<String, (u64, gitpoulpi_ipc::Endpoint)>;

fn endpoints() -> std::sync::MutexGuard<'static, EndpointMap> {
    static ENDPOINTS: OnceLock<Mutex<EndpointMap>> = OnceLock::new();
    ENDPOINTS
        .get_or_init(Mutex::default)
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

/// Remembers where the daemon of `session` listens, so the grants it issues
/// can name it to the helper. Dropping the returned guard forgets it (unless a
/// later daemon of the same session in this process has replaced it).
#[must_use]
pub(crate) fn register_endpoint(session: &str, endpoint: gitpoulpi_ipc::Endpoint) -> EndpointGuard {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let seq = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    endpoints().insert(session.to_string(), (seq, endpoint));
    EndpointGuard {
        session: session.to_string(),
        seq,
    }
}

pub(crate) struct EndpointGuard {
    session: String,
    seq: u64,
}

impl Drop for EndpointGuard {
    fn drop(&mut self) {
        let mut map = endpoints();
        if map
            .get(&self.session)
            .is_some_and(|(seq, _)| *seq == self.seq)
        {
            map.remove(&self.session);
        }
    }
}

impl Drop for GrantGuard {
    fn drop(&mut self) {
        let mut registry = lock();
        if let Some(entry) = registry.grants.get_mut(&self.id) {
            entry.holders = entry.holders.saturating_sub(1);
            if entry.holders == 0 {
                if let Some(file) = registry
                    .grants
                    .remove(&self.id)
                    .and_then(|e| e.endpoint_file)
                {
                    let _ = std::fs::remove_file(file);
                }
            }
        }
    }
}

/// What `RepoAuthTest` reads back after one remote's check: the identity git
/// reported as accepted (`store`), if any, and whether the server refused it
/// (`erase`). Resets both, so the next remote starts clean.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Outcome {
    pub accepted: Option<String>,
    pub token_refused: bool,
    /// A grant is live and the helper never asked about it.
    pub unasked: bool,
    /// Why the grant's token command gave no token (its failure, as the
    /// person reads it), if it failed.
    pub command_failure: Option<String>,
}

/// [`Outcome`] of the newest live grant of the repository at `root`.
pub(crate) fn take_outcome(root: &Path) -> Outcome {
    let mut registry = lock();
    let newest = registry
        .grants
        .values_mut()
        .filter(|e| e.root == root)
        .max_by_key(|e| e.seq);
    match newest {
        Some(entry) => {
            let outcome = Outcome {
                accepted: entry.accepted.then(|| entry.credential.identity.clone()),
                token_refused: entry.token_refused,
                unasked: !entry.asked,
                command_failure: entry.command_failure.take().map(|(exit, stderr_tail)| {
                    gitpoulpi_core::Error::TokenCommandFailed { exit, stderr_tail }.to_string()
                }),
            };
            entry.accepted = false;
            entry.token_refused = false;
            entry.asked = false;
            outcome
        }
        None => Outcome::default(),
    }
}

/// A fresh 128-bit id from the operating system's random source.
fn new_id() -> Result<String, String> {
    let token = gitpoulpi_ipc::Token::generate().map_err(|e| e.to_string())?;
    Ok(token.to_hex()[..32].to_string())
}

/// Issues a grant for `credential`, for a network operation of the repository
/// at `root`. `Err` when the operating system has no random source: the
/// transfer is then refused rather than started with a guessable id.
///
/// Two transfers of one repository can overlap (a fetch and a push share the
/// transfer lock) and each of their git processes reads the repository's
/// current authentication when it starts, so the second would otherwise find
/// the first's grant, possibly already revoked. A grant for the same
/// repository and the very same credential is therefore shared, and revoked
/// when the last operation holding it ends.
pub(crate) fn issue(root: &Path, credential: GrantCredential) -> Result<GrantGuard, String> {
    let mut registry = lock();
    for (id, entry) in registry.grants.iter_mut() {
        if entry.root == root && entry.credential == credential {
            entry.holders += 1;
            return Ok(GrantGuard { id: id.clone() });
        }
    }
    let id = new_id()?;
    registry.next += 1;
    let seq = registry.next;
    registry.grants.insert(
        id.clone(),
        Entry {
            credential,
            asked: false,
            endpoint_file: None,
            root: root.to_path_buf(),
            seq,
            holders: 1,
            accepted: false,
            token_refused: false,
            refused: None,
            command_failure: None,
        },
    );
    Ok(GrantGuard { id })
}

fn host_matches(entry: &Entry, request: &CredentialGrantRequest) -> bool {
    let host = request.host.trim();
    let protocol_ok = request
        .protocol
        .eq_ignore_ascii_case(&entry.credential.scheme)
        || (entry.credential.allow_plain_http && request.protocol.eq_ignore_ascii_case("http"));
    protocol_ok
        && entry
            .credential
            .hosts
            .iter()
            .any(|h| host.eq_ignore_ascii_case(h))
}

/// What to do after the registry lock is released.
enum Next {
    Done(CredentialGrantAnswer),
    /// Run the token command (it takes a while: never under the lock).
    Command(crate::token_command::TokenCommand),
}

/// The daemon's answer to one helper request. An unknown or revoked grant
/// answers nothing, exactly like a refused host.
pub(crate) async fn answer(request: &CredentialGrantRequest) -> CredentialGrantAnswer {
    let next = {
        let mut registry = lock();
        let Some(entry) = registry.grants.get_mut(&request.grant) else {
            return CredentialGrantAnswer::default();
        };
        entry.asked = true;
        let matches = host_matches(entry, request);
        match request.action {
            CredentialAction::Get if matches => match &entry.credential.command {
                Some(command) => Next::Command(command.clone()),
                None => Next::Done(CredentialGrantAnswer {
                    username: Some(entry.credential.username.clone()),
                    password: Some(entry.credential.password.clone()),
                }),
            },
            CredentialAction::Get => {
                entry.refused = Some((
                    request.host.trim().to_string(),
                    entry.credential.hosts.join(", "),
                ));
                Next::Done(CredentialGrantAnswer::default())
            }
            CredentialAction::Store => {
                if matches {
                    entry.accepted = true;
                }
                Next::Done(CredentialGrantAnswer::default())
            }
            CredentialAction::Erase => {
                if matches {
                    entry.token_refused = true;
                    // The server refused the token: the next get must make a
                    // new one.
                    if let Some(command) = &entry.credential.command {
                        crate::token_command::invalidate(command);
                    }
                }
                Next::Done(CredentialGrantAnswer::default())
            }
        }
    };
    let command = match next {
        Next::Done(answer) => return answer,
        Next::Command(command) => command,
    };
    match crate::token_command::credential(&command).await {
        Ok(given) => {
            let (username, password) = crate::token_command::answer_of(&command, given);
            CredentialGrantAnswer {
                username: Some(username),
                password: Some(password),
            }
        }
        Err(gitpoulpi_core::Error::TokenCommandFailed { exit, stderr_tail }) => {
            if let Some(entry) = lock().grants.get_mut(&request.grant) {
                entry.command_failure = Some((exit, stderr_tail));
            }
            CredentialGrantAnswer::default()
        }
        Err(_) => CredentialGrantAnswer::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn credential(host: &str) -> GrantCredential {
        GrantCredential {
            username: "x-access-token".into(),
            password: "tok-0123456789".into(),
            scheme: "https".into(),
            hosts: vec![host.into()],
            allow_plain_http: false,
            command: None,
            identity: "account work".into(),
        }
    }

    fn get(grant: &str, protocol: &str, host: &str) -> CredentialGrantRequest {
        CredentialGrantRequest {
            grant: grant.into(),
            action: CredentialAction::Get,
            protocol: protocol.into(),
            host: host.into(),
            path: String::new(),
        }
    }

    #[tokio::test]
    async fn a_grant_answers_only_its_protocol_and_host_and_only_while_alive() {
        let guard = issue(Path::new("/r"), credential("git.corp.example")).unwrap();
        let id = guard.id().to_string();
        assert_eq!(id.len(), 32);

        let ok = answer(&get(&id, "https", "GIT.corp.example")).await;
        assert_eq!(ok.username.as_deref(), Some("x-access-token"));
        assert_eq!(ok.password.as_deref(), Some("tok-0123456789"));
        assert_eq!(guard.refused(), None);

        // Another host, a lookalike, another port, another protocol.
        for (protocol, host) in [
            ("https", "evil.example"),
            ("https", "git.corp.example.evil.example"),
            ("https", "git.corp.example:8443"),
            ("http", "git.corp.example"),
        ] {
            assert_eq!(
                answer(&get(&id, protocol, host)).await,
                CredentialGrantAnswer::default(),
                "{protocol}://{host}"
            );
        }
        assert_eq!(
            guard.refused(),
            Some(("git.corp.example".into(), "git.corp.example".into()))
        );

        drop(guard);
        assert_eq!(
            answer(&get(&id, "https", "git.corp.example")).await,
            CredentialGrantAnswer::default(),
            "a revoked grant answers nothing"
        );
        assert_eq!(
            answer(&get(
                "0123456789abcdef0123456789abcdef",
                "https",
                "git.corp.example"
            ))
            .await,
            CredentialGrantAnswer::default()
        );
    }

    #[tokio::test]
    async fn store_and_erase_are_read_back_once_per_remote() {
        let root = Path::new("/outcome-root");
        let guard = issue(root, credential("h.example")).unwrap();
        let send = |action, host: &str| {
            let mut r = get(guard.id(), "https", host);
            r.action = action;
            async move { answer(&r).await }
        };
        send(CredentialAction::Store, "h.example").await;
        assert_eq!(
            take_outcome(root),
            Outcome {
                accepted: Some("account work".into()),
                token_refused: false,
                unasked: false,
                ..Outcome::default()
            }
        );
        // Nothing asked since: the helper has not reached the daemon.
        assert_eq!(
            take_outcome(root),
            Outcome {
                unasked: true,
                ..Outcome::default()
            }
        );
        send(CredentialAction::Erase, "h.example").await;
        // Another host's store or erase says nothing about this account.
        send(CredentialAction::Store, "other.example").await;
        assert_eq!(
            take_outcome(root),
            Outcome {
                accepted: None,
                token_refused: true,
                unasked: false,
                ..Outcome::default()
            }
        );
    }

    #[tokio::test]
    async fn a_grant_nobody_asked_about_reads_git_s_missing_credential_as_unreachable() {
        let root = Path::new("/repo/unasked");
        let guard = issue(root, credential("h.example")).unwrap();
        let prompt =
            "fatal: could not read Username for 'https://h.example': terminal prompts disabled";
        assert!(matches!(
            guard.reads_as(prompt),
            Some(gitpoulpi_core::Error::CredentialHelperUnreachable)
        ));
        // Another failure of the same transfer is not about the helper.
        assert!(guard.reads_as("fatal: unable to access").is_none());
        // Once the helper has asked, the same words mean something else.
        let _ = answer(&get(guard.id(), "https", "h.example")).await;
        assert!(guard.reads_as(prompt).is_none());
    }

    #[test]
    fn debug_never_prints_the_password() {
        assert!(!format!("{:?}", credential("h")).contains("tok-0123456789"));
    }
}
