//! Where a repository's own state file lives (pure: config and session
//! only). Re-exported by `remotes_ui`.

use std::path::{Path, PathBuf};

/// The per-repository state file of `repo_root` under the configured data dir.
pub fn repo_state_path(repo_root: &Path) -> PathBuf {
    let state_dir = gitpoulpi_config::Settings::resolve_ambient()
        .map(|s| s.config().session.per_repo_state_dir.clone())
        .unwrap_or_else(|_| gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION.to_string());
    gitpoulpi_session::RepoState::path_for(repo_root, &state_dir)
}
