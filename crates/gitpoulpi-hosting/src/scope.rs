//! Per-repository decisions the hosting clients honour without every caller
//! passing a repository: which token a repository's host calls use (a token
//! command, for a GitHub App bot) and whether assignment is blocked.
//!
//! The hosting crate cannot read the repository's stored settings (those live
//! in crates that depend on it), so a higher crate installs one
//! [`RepoScopes`] for the process. A client then asks it, per call, by the
//! `(integration id, project)` pair every call already carries: no call site
//! needs a repository handed to it, and a surface that forgets to register a
//! repository simply gets the plain behaviour (personal token, assignment
//! allowed) rather than a failure.

use std::sync::{Arc, OnceLock};

use crate::{Error, Result};

/// The reason shown wherever an assignment is refused or an assignee field is
/// hidden. Also the marker a CLI answer is recognised by (exit code 2).
pub const ASSIGNMENT_BLOCKED: &str = "assignment is blocked for this repository";

/// What a repository's stored settings say about its host calls.
#[async_trait::async_trait]
pub trait RepoScopes: Send + Sync {
    /// Whether nothing may be assigned in `project` of `integration_id`.
    fn assignment_blocked(&self, integration_id: &str, project: &str) -> bool;

    /// The token the repository's token command gives for its host calls,
    /// `None` when the repository does not use one (the personal token
    /// applies). `refresh`: the host refused the previous one, so ask the
    /// command again. An `Err` refuses the call: it never falls back to the
    /// personal token.
    async fn token(
        &self,
        integration_id: &str,
        project: &str,
        refresh: bool,
    ) -> Option<Result<String>>;

    /// Whether the repository's host calls are made by an App bot.
    fn acts_as_bot(&self, integration_id: &str, project: &str) -> bool;

    /// The "Post as" name the user configured for `project`: the
    /// repository's own, else the global `general.hosting_post_as`.
    fn post_as(&self, _integration_id: &str, _project: &str) -> Option<String> {
        None
    }

    /// The login learned from the author of our first post, kept in the
    /// repository's state.
    fn learned_login(&self, _integration_id: &str, _project: &str) -> Option<String> {
        None
    }

    /// Remembers `login` as who the repository's token posts as.
    fn learn_login(&self, _integration_id: &str, _project: &str, _login: &str) {}
}

static SCOPES: OnceLock<Arc<dyn RepoScopes>> = OnceLock::new();

/// Installs the process's scopes (the first call wins).
pub fn install(scopes: Arc<dyn RepoScopes>) {
    let _ = SCOPES.set(scopes);
}

pub(crate) fn installed() -> Option<&'static Arc<dyn RepoScopes>> {
    SCOPES.get()
}

/// Whether assignment is blocked for `project` of `integration_id`.
pub fn assignment_blocked(integration_id: &str, project: &str) -> bool {
    installed().is_some_and(|s| s.assignment_blocked(integration_id, project))
}

/// Whether the repository's host calls are made by an App bot.
pub fn acts_as_bot(integration_id: &str, project: &str) -> bool {
    installed().is_some_and(|s| s.acts_as_bot(integration_id, project))
}

/// The configured "Post as" name of `project`, if any.
pub fn post_as(integration_id: &str, project: &str) -> Option<String> {
    installed()
        .and_then(|s| s.post_as(integration_id, project))
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty())
}

/// The login learned from our first post in `project`, if any.
pub fn learned_login(integration_id: &str, project: &str) -> Option<String> {
    installed().and_then(|s| s.learned_login(integration_id, project))
}

/// Persists `login` as who the token posts as in `project`.
pub fn learn_login(integration_id: &str, project: &str, login: &str) {
    if let Some(s) = installed() {
        s.learn_login(integration_id, project, login);
    }
}

/// The refusal of an assignment into a blocked repository.
#[cfg_attr(not(feature = "hosting"), allow(dead_code))]
pub(crate) fn refuse() -> Error {
    Error::AssignmentBlocked
}

/// Whether `token` is a GitHub App installation token (`ghs_`): it has no
/// "me" and cannot call `GET /user`.
pub fn is_app_installation_token(token: &str) -> bool {
    token.trim().starts_with("ghs_")
}

/// `owner/repo` of an API URL of the form `<api>/repos/<owner>/<repo>/...`.
#[cfg_attr(not(feature = "github"), allow(dead_code))]
pub(crate) fn github_project_of(url: &str) -> Option<String> {
    let path = url.split(['?', '#']).next()?;
    let (_, rest) = path.split_once("/repos/")?;
    let mut parts = rest.split('/');
    let owner = parts.next().filter(|p| !p.is_empty())?;
    let repo = parts.next().filter(|p| !p.is_empty())?;
    Some(format!("{owner}/{repo}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_project_of_an_api_url_is_owner_and_repo() {
        assert_eq!(
            github_project_of("https://api.example.com/repos/acme/app/issues/3/comments?x=1"),
            Some("acme/app".to_string())
        );
        assert_eq!(github_project_of("https://api.example.com/user"), None);
        assert_eq!(
            github_project_of("https://api.example.com/repos/acme"),
            None
        );
    }

    #[test]
    fn an_installation_token_is_recognised_by_its_prefix() {
        assert!(is_app_installation_token("ghs_abc"));
        assert!(!is_app_installation_token("ghp_abc"));
        assert!(!is_app_installation_token("github_pat_abc"));
    }
}
