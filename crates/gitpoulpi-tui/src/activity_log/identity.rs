//! Who each repository signs in as, when its SSH & credentials profile is not
//! all default: what the push menu says and what the Activity Log appends to
//! the entries of fetch, pull and push.
//!
//! The daemon puts the typed [`ActiveIdentity`] in the `Status` payload (from
//! its private file, no spawn); the window notes it here when a refresh lands,
//! so the link's worker threads and the render pass only read a map. Nothing
//! here reads a file or runs a process.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use gitpoulpi_core::domain::{ActiveIdentity, IdentityVia};
use gitpoulpi_i18n::msg::repo_auth as t;

use crate::daemon_link::Action;

type Map = HashMap<PathBuf, ActiveIdentity>;

#[cfg(not(test))]
static IDENTITIES: std::sync::Mutex<Option<Map>> = std::sync::Mutex::new(None);

#[cfg(test)]
thread_local! {
    static IDENTITIES: std::cell::RefCell<Option<Map>> = const { std::cell::RefCell::new(None) };
}

#[cfg(not(test))]
fn with<R>(f: impl FnOnce(&mut Map) -> R) -> R {
    let mut guard = IDENTITIES.lock().unwrap_or_else(|e| e.into_inner());
    f(guard.get_or_insert_with(Map::new))
}

#[cfg(test)]
fn with<R>(f: impl FnOnce(&mut Map) -> R) -> R {
    IDENTITIES.with(|cell| f(cell.borrow_mut().get_or_insert_with(Map::new)))
}

/// Remembers the identity of `repo` as the last status said it (`None`:
/// everything default).
pub(crate) fn note(repo: &Path, identity: Option<&ActiveIdentity>) {
    with(|map| match identity {
        Some(id) => {
            map.insert(repo.to_path_buf(), id.clone());
        }
        None => {
            map.remove(repo);
        }
    });
}

/// The identity of `repo`, when its profile is not all default.
pub(crate) fn get(repo: &Path) -> Option<ActiveIdentity> {
    with(|map| map.get(repo).cloned())
}

/// "Signs in as: github-corp (token)" / "SSH key: work_ed25519" / both
/// joined.
pub(crate) fn text(identity: &ActiveIdentity) -> String {
    let https = identity.https.as_ref().map(|h| match h.via {
        IdentityVia::Token => t::identity_token(&h.name),
        IdentityVia::Username => t::identity_user(&h.name),
        IdentityVia::CredentialHelper => t::IDENTITY_HELPER.get().to_string(),
        IdentityVia::TokenCommand => t::IDENTITY_COMMAND.get().to_string(),
        IdentityVia::HelpersIgnored => t::IDENTITY_IGNORED.get().to_string(),
    });
    let ssh = identity.ssh.as_ref().map(|s| {
        if s.of_account {
            t::identity_ssh_account(&s.name)
        } else {
            t::identity_ssh(&s.name)
        }
    });
    match (https, ssh) {
        (Some(h), Some(s)) => t::identity_both(h, s),
        (Some(line), None) | (None, Some(line)) => line,
        (None, None) => String::new(),
    }
}

/// The line for `repo`, when there is one.
pub(crate) fn line_for(repo: &Path) -> Option<String> {
    get(repo).map(|id| text(&id)).filter(|s| !s.is_empty())
}

/// Whether `action` is a fetch, pull or push: the operations that sign in.
pub(crate) fn uses_network(action: &Action) -> bool {
    let action = match action {
        Action::AutoStashed(inner) | Action::AllowSensitive(inner) => inner,
        other => other,
    };
    matches!(
        action,
        Action::Fetch(_)
            | Action::FetchAll
            | Action::Pull(_)
            | Action::Push { .. }
            | Action::PushForReview { .. }
            | Action::PullThenPush { .. }
            | Action::FetchThenForcePush { .. }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::{HttpsIdentity, SshIdentity};

    #[test]
    fn the_line_names_the_account_and_the_key() {
        let both = ActiveIdentity {
            https: Some(HttpsIdentity {
                via: IdentityVia::Token,
                name: "github-corp".into(),
            }),
            ssh: Some(SshIdentity {
                name: "work_ed25519".into(),
                of_account: false,
            }),
            integration: true,
        };
        assert_eq!(
            text(&both),
            "Signs in as: github-corp (token); SSH key: work_ed25519"
        );
        let key = ActiveIdentity {
            ssh: both.ssh.clone(),
            ..ActiveIdentity::default()
        };
        assert_eq!(text(&key), "SSH key: work_ed25519");
        let repo = Path::new("/r");
        assert_eq!(line_for(repo), None);
        note(repo, Some(&key));
        assert_eq!(line_for(repo).as_deref(), Some("SSH key: work_ed25519"));
        note(repo, None);
        assert_eq!(line_for(repo), None, "back to default: no line");
    }

    /// The Activity Log entry of a pull or push appends who it signed in as,
    /// only for a repository whose profile is not default and only for the
    /// operations that sign in.
    #[test]
    fn the_entry_of_a_pull_says_who_it_signed_in_as() {
        use crate::activity_log::{feed, report_success};
        use gitpoulpi_core::domain::PullOptions;
        let repo = Path::new("/repos/work");
        let pull = Action::Pull(PullOptions::default());
        let took = std::time::Duration::from_millis(5);

        report_success(&pull, repo, took);
        let plain = feed::snapshot().remove(0);
        assert!(!plain.message.contains("Signs in"), "{}", plain.message);

        let account = ActiveIdentity {
            https: Some(HttpsIdentity {
                via: IdentityVia::Token,
                name: "github-corp".into(),
            }),
            ..ActiveIdentity::default()
        };
        note(repo, Some(&account));
        report_success(&pull, repo, took);
        let entry = feed::snapshot().remove(0);
        assert!(
            entry.message.contains("Signs in as: github-corp (token)"),
            "{}",
            entry.message
        );
        // Another repository, and an operation that never signs in: nothing.
        report_success(&pull, Path::new("/repos/other"), took);
        assert!(!feed::snapshot()[0].message.contains("Signs in"));
        assert!(!uses_network(&Action::StageAll));
        assert!(uses_network(&Action::FetchAll));
        assert!(uses_network(&pull));
    }
}
