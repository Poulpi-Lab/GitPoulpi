//! The LFS pre-flight of a pull or push (`git.lfs_autofix_filters`).
//!
//! A repository that uses LFS but never had `git lfs install --local` run
//! has no smudge filter (a pull leaves pointer files) and no `pre-push`
//! hook (a push uploads no LFS objects, so a later clone of the remote
//! fails). Pull and push are meant to be LFS-aware without thinking about
//! it, so the daemon repairs that, for this repository only, right before
//! the transfer and tells every frontend with [`Event::LfsFiltersPosed`].
//!
//! Silent and best-effort: any failure (no `git-lfs`, a read-only `.git`)
//! leaves the transfer to go ahead exactly as before. No extra process
//! spawn beyond `lfs_status`, whose `git lfs version` probe is cached.

use gitpoulpi_core::domain::{LfsBinary, LfsInstallScope, OperationKind, RepoRef};
use gitpoulpi_core::GitBackend;
use gitpoulpi_ipc::Event;
use tokio::sync::broadcast;

/// Whether the settings ask for the pre-flight: LFS on (the repository's
/// own choice first, then `git.lfs_enabled`) and `git.lfs_autofix_filters`.
fn wanted(repo: &RepoRef) -> bool {
    let Ok(settings) = gitpoulpi_config::Settings::resolve_ambient() else {
        return true;
    };
    let config = settings.config();
    if !config.git.lfs_autofix_filters {
        return false;
    }
    let state_path =
        gitpoulpi_session::RepoState::path_for(&repo.root, &config.session.per_repo_state_dir);
    let own = gitpoulpi_session::RepoState::load(&state_path)
        .ok()
        .and_then(|s| s.preferences.lfs_enabled);
    own.unwrap_or(config.git.lfs_enabled)
}

/// Runs the pre-flight for `operation` (`Pull`, `Push` or `RewritePush`; anything else
/// is a no-op).
pub(crate) fn run(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    operation: OperationKind,
    events: &broadcast::Sender<Event>,
) {
    if !matches!(
        operation,
        OperationKind::Pull | OperationKind::Push | OperationKind::RewritePush
    ) || !wanted(repo)
    {
        return;
    }
    let Ok(status) = backend.lfs_status(repo) else {
        return;
    };
    if !status.uses_lfs || !matches!(status.binary, LfsBinary::Installed { .. }) {
        return;
    }
    let missing = match operation {
        OperationKind::Pull => !status.filters_configured,
        _ => !status.hooks_installed,
    };
    if missing
        && backend
            .lfs_install(repo, LfsInstallScope::Repository)
            .is_ok()
    {
        let _ = events.send(Event::LfsFiltersPosed {
            repo: repo.id.clone(),
        });
    }
}
