//! The saved state of a stopped flow: `<git-dir>/gitpoulpi/flow-state.json`.
//! In the git directory (not the working tree) so it is per worktree, never
//! committed and never shown by status.

use std::path::PathBuf;

use gitpoulpi_core::domain::RepoRef;
use gitpoulpi_core::workflow::FlowState;

fn path(repo: &RepoRef) -> Option<PathBuf> {
    crate::journal::git_dir(&repo.root).map(|d| d.join("gitpoulpi").join("flow-state.json"))
}

pub(super) fn load(repo: &RepoRef) -> Option<FlowState> {
    let text = std::fs::read_to_string(path(repo)?).ok()?;
    match serde_json::from_str(&text) {
        Ok(state) => Some(state),
        Err(e) => {
            tracing::warn!(target: "daemon", "the saved flow state is unreadable: {e}");
            None
        }
    }
}

pub(super) fn save(repo: &RepoRef, state: &FlowState) {
    let Some(path) = path(repo) else {
        return;
    };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    match serde_json::to_string_pretty(state) {
        Ok(text) => {
            let tmp = path.with_extension("json.tmp");
            if std::fs::write(&tmp, text).is_ok() {
                if let Err(e) = std::fs::rename(&tmp, &path) {
                    tracing::warn!(target: "daemon", "could not save the flow state: {e}");
                }
            }
        }
        Err(e) => tracing::warn!(target: "daemon", "could not encode the flow state: {e}"),
    }
}

pub(super) fn clear(repo: &RepoRef) {
    if let Some(path) = path(repo) {
        let _ = std::fs::remove_file(path);
    }
}
