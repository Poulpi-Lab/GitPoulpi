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
    /// `host` or `host:port`, lowercase: the only host the grant answers.
    pub host: String,
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
            .field("host", &self.host)
            .finish()
    }
}

struct Entry {
    credential: GrantCredential,
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

impl Drop for GrantGuard {
    fn drop(&mut self) {
        let mut registry = lock();
        if let Some(entry) = registry.grants.get_mut(&self.id) {
            entry.holders = entry.holders.saturating_sub(1);
            if entry.holders == 0 {
                registry.grants.remove(&self.id);
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
            };
            entry.accepted = false;
            entry.token_refused = false;
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
            root: root.to_path_buf(),
            seq,
            holders: 1,
            accepted: false,
            token_refused: false,
            refused: None,
        },
    );
    Ok(GrantGuard { id })
}

fn host_matches(entry: &Entry, request: &CredentialGrantRequest) -> bool {
    request
        .protocol
        .eq_ignore_ascii_case(&entry.credential.scheme)
        && request
            .host
            .trim()
            .eq_ignore_ascii_case(&entry.credential.host)
}

/// The daemon's answer to one helper request. An unknown or revoked grant
/// answers nothing, exactly like a refused host.
pub(crate) fn answer(request: &CredentialGrantRequest) -> CredentialGrantAnswer {
    let mut registry = lock();
    let Some(entry) = registry.grants.get_mut(&request.grant) else {
        return CredentialGrantAnswer::default();
    };
    let matches = host_matches(entry, request);
    match request.action {
        CredentialAction::Get if matches => CredentialGrantAnswer {
            username: Some(entry.credential.username.clone()),
            password: Some(entry.credential.password.clone()),
        },
        CredentialAction::Get => {
            entry.refused = Some((
                request.host.trim().to_string(),
                entry.credential.host.clone(),
            ));
            CredentialGrantAnswer::default()
        }
        CredentialAction::Store => {
            if matches {
                entry.accepted = true;
            }
            CredentialGrantAnswer::default()
        }
        CredentialAction::Erase => {
            if matches {
                entry.token_refused = true;
            }
            CredentialGrantAnswer::default()
        }
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
            host: host.into(),
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

    #[test]
    fn a_grant_answers_only_its_protocol_and_host_and_only_while_alive() {
        let guard = issue(Path::new("/r"), credential("git.corp.example")).unwrap();
        let id = guard.id().to_string();
        assert_eq!(id.len(), 32);

        let ok = answer(&get(&id, "https", "GIT.corp.example"));
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
                answer(&get(&id, protocol, host)),
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
            answer(&get(&id, "https", "git.corp.example")),
            CredentialGrantAnswer::default(),
            "a revoked grant answers nothing"
        );
        assert_eq!(
            answer(&get(
                "0123456789abcdef0123456789abcdef",
                "https",
                "git.corp.example"
            )),
            CredentialGrantAnswer::default()
        );
    }

    #[test]
    fn store_and_erase_are_read_back_once_per_remote() {
        let root = Path::new("/outcome-root");
        let guard = issue(root, credential("h.example")).unwrap();
        let send = |action, host: &str| {
            let mut r = get(guard.id(), "https", host);
            r.action = action;
            answer(&r)
        };
        send(CredentialAction::Store, "h.example");
        assert_eq!(
            take_outcome(root),
            Outcome {
                accepted: Some("account work".into()),
                token_refused: false
            }
        );
        assert_eq!(take_outcome(root), Outcome::default());
        send(CredentialAction::Erase, "h.example");
        // Another host's store or erase says nothing about this account.
        send(CredentialAction::Store, "other.example");
        assert_eq!(
            take_outcome(root),
            Outcome {
                accepted: None,
                token_refused: true
            }
        );
    }

    #[test]
    fn debug_never_prints_the_password() {
        assert!(!format!("{:?}", credential("h")).contains("tok-0123456789"));
    }
}
