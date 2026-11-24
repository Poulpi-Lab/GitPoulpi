//! "Clone it, then review": what a clone started from the Radar owes the
//! request it was started for.
//!
//! The Clone dialog is the ordinary one, opened on the request's
//! connected account with its project selected (the URL prefill when the
//! profile cannot list repositories). Its Submit moves the
//! request into a process-wide slot **keyed by the destination folder** (the
//! clone outlives the dialog and the window, `clone_task`), and the window
//! that is current when the clone ends settles it:
//!
//! - success: the repository's `pulls_source` is written (and its
//!   `issues_source` when the host has issues and none is set), then the
//!   repository's tab opens with the review of that request carried
//!   (`radar::pending`);
//! - failure or cancel: the entry is taken and dropped, nothing is written.
//!
//! This slot has no expiry of its own (a clone can take minutes, unlike the
//! 20 s `radar::pending` keeps a carried action for); the carried action is
//! only set once the clone has landed.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use gitpoulpi_config::{ApiFamily, Integration};
use gitpoulpi_session::{IssuesSource, RepoState};

/// What the clone's destination owes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Carry {
    pub(crate) number: u64,
    /// `#12`, `!53`, for the status line.
    pub(crate) label: String,
    /// The profile the request came from.
    pub(crate) integration: String,
    /// `owner/name` on the host.
    pub(crate) project: String,
}

static SLOTS: Mutex<Vec<(PathBuf, Carry)>> = Mutex::new(Vec::new());

fn lock() -> std::sync::MutexGuard<'static, Vec<(PathBuf, Carry)>> {
    SLOTS.lock().unwrap_or_else(|e| e.into_inner())
}

fn same(a: &Path, b: &Path) -> bool {
    let (a, b) = (a.to_string_lossy(), b.to_string_lossy());
    if cfg!(windows) {
        a.eq_ignore_ascii_case(&b)
    } else {
        a == b
    }
}

/// Records that the clone into `dest` was started for `carry` (a second one
/// for the same folder replaces the first).
pub(crate) fn set(dest: &Path, carry: Carry) {
    let mut slots = lock();
    slots.retain(|(d, _)| !same(d, dest));
    slots.push((dest.to_path_buf(), carry));
}

/// Takes the entry of `dest`, if the clone was started for a request.
pub(crate) fn take(dest: &Path) -> Option<Carry> {
    let mut slots = lock();
    let at = slots.iter().position(|(d, _)| same(d, dest))?;
    Some(slots.remove(at).1)
}

/// Whether the host of `integration` has issues to link (Bitbucket and
/// Gerrit have none).
fn has_issues(integration: &Integration) -> bool {
    !matches!(
        integration.kind.family(),
        ApiFamily::Bitbucket | ApiFamily::Gerrit
    )
}

/// Links the new repository to the request's host: `pulls_source` always,
/// `issues_source` when the host has issues and none is set yet. `state` is
/// the repository's state file. An explicit user act (they chose to clone it
/// to review it), unlike the suggestions the remotes alone make.
pub(crate) fn write_sources(
    state: &Path,
    carry: &Carry,
    integrations: &[Integration],
) -> Result<(), String> {
    let integration = integrations.iter().find(|i| i.id == carry.integration);
    let mut repo = RepoState::load(state).unwrap_or_default();
    let source = || IssuesSource {
        integration: carry.integration.clone(),
        project: carry.project.clone(),
        filter: None,
    };
    repo.pulls_source = Some(source());
    if repo.issues_source.is_none() && integration.is_some_and(has_issues) {
        repo.issues_source = Some(source());
    }
    repo.save(state).map_err(|e| e.to_string())
}

/// Links the existing clone at `root` to `project` of `integration` when it
/// is not linked to any host for its pull requests yet (the Radar's review
/// through a clone it only knew by its remote). A repository with a link of
/// its own is left alone. Best effort: a failure leaves it unlinked and the
/// review says why it cannot start.
pub(crate) fn link_if_unlinked(root: &Path, integration: &str, project: &str) {
    let state = crate::remotes_ui::repo_state_path(root);
    if RepoState::load(&state).is_ok_and(|s| s.pulls_source.is_some()) {
        return;
    }
    let carry = Carry {
        number: 0,
        label: String::new(),
        integration: integration.to_string(),
        project: project.to_string(),
    };
    let all = crate::pull_request::load_integrations();
    let _ = write_sources(&state, &carry, &all);
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_config::IntegrationKind;

    fn carry() -> Carry {
        Carry {
            number: 58,
            label: "!58".into(),
            integration: "lab".into(),
            project: "northwind/billing".into(),
        }
    }

    fn profile(id: &str, kind: IntegrationKind) -> Integration {
        Integration::new(id, kind, "https://host.example")
    }

    #[test]
    fn the_slot_is_keyed_by_destination_and_taken_once() {
        let a = PathBuf::from("/tmp/after-clone-a");
        let b = PathBuf::from("/tmp/after-clone-b");
        set(&a, carry());
        assert_eq!(take(&b), None);
        assert_eq!(take(&a), Some(carry()));
        assert_eq!(take(&a), None, "taken once");
        // A second start for the same folder replaces the first.
        set(&a, carry());
        let mut other = carry();
        other.number = 7;
        set(&a, other.clone());
        assert_eq!(take(&a), Some(other));
    }

    #[test]
    fn a_clone_links_the_pull_requests_and_the_issues_when_none_is_set() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("repo-state.toml");
        let all = vec![profile("lab", IntegrationKind::GitlabSelfManaged)];
        write_sources(&path, &carry(), &all).unwrap();
        let state = RepoState::load(&path).unwrap();
        let want = IssuesSource {
            integration: "lab".into(),
            project: "northwind/billing".into(),
            filter: None,
        };
        assert_eq!(state.pulls_source, Some(want.clone()));
        assert_eq!(state.issues_source, Some(want));
    }

    #[test]
    fn an_issues_link_that_exists_is_kept_and_a_host_without_issues_gets_none() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("repo-state.toml");
        let mine = IssuesSource {
            integration: "jira".into(),
            project: "OPS".into(),
            filter: None,
        };
        let state = RepoState {
            issues_source: Some(mine.clone()),
            ..RepoState::default()
        };
        state.save(&path).unwrap();
        let all = vec![profile("lab", IntegrationKind::GitlabSelfManaged)];
        write_sources(&path, &carry(), &all).unwrap();
        let state = RepoState::load(&path).unwrap();
        assert_eq!(state.issues_source, Some(mine), "not overwritten");
        assert_eq!(state.pulls_source.unwrap().project, "northwind/billing");

        let other = dir.path().join("gerrit.toml");
        let gerrit = vec![profile("lab", IntegrationKind::Gerrit)];
        write_sources(&other, &carry(), &gerrit).unwrap();
        let state = RepoState::load(&other).unwrap();
        assert!(state.pulls_source.is_some());
        assert_eq!(state.issues_source, None, "Gerrit has no issues");
    }
}
