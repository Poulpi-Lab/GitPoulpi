//! The saved state of a grouped run that stopped (on conflicts, canceled, or
//! with targets to retry): `<git-dir>/gitpoulpi/remote-group.json`. In the
//! git directory (not the working tree), so it is per worktree, never
//! committed and never shown by status, like `flow-state.json`. It is bound
//! to the merge or rebase in progress and to HEAD, which are per worktree.
//! Written atomically (temporary file, rename) through
//! `gitpoulpi_config::secure_fs`.

use std::path::PathBuf;

use gitpoulpi_core::domain::{
    GroupMode, Oid, RepoRef, StateTarget, StoppedAt, REMOTE_GROUP_STATE_VERSION,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct GroupState {
    pub version: u32,
    pub run_id: String,
    pub mode: GroupMode,
    pub branch: String,
    pub started_tip: Oid,
    #[serde(default)]
    pub tags: bool,
    /// `git.pull_mode` is rebase (see `RemoteGroupRunRequest`).
    #[serde(default)]
    pub prefer_rebase: bool,
    pub targets: Vec<StateTarget>,
    #[serde(default)]
    pub stopped: Option<StoppedAt>,
}

impl GroupState {
    pub(super) fn new(
        mode: GroupMode,
        branch: &str,
        started_tip: Oid,
        tags: bool,
        targets: Vec<StateTarget>,
    ) -> Self {
        Self {
            version: REMOTE_GROUP_STATE_VERSION,
            run_id: new_run_id(),
            mode,
            branch: branch.to_string(),
            started_tip,
            tags,
            prefer_rebase: false,
            targets,
            stopped: None,
        }
    }
}

fn new_run_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("rg-{nanos:x}-{}", COUNTER.fetch_add(1, Ordering::Relaxed))
}

fn path(repo: &RepoRef) -> Option<PathBuf> {
    crate::journal::git_dir(&repo.root).map(|d| d.join("gitpoulpi").join("remote-group.json"))
}

pub(super) fn load(repo: &RepoRef) -> Option<GroupState> {
    let text = std::fs::read_to_string(path(repo)?).ok()?;
    match serde_json::from_str::<GroupState>(&text) {
        Ok(state) if state.version <= REMOTE_GROUP_STATE_VERSION => Some(state),
        Ok(_) => {
            tracing::warn!(target: "daemon", "the saved grouped run is from a newer version");
            None
        }
        Err(e) => {
            tracing::warn!(target: "daemon", "the saved grouped run is unreadable: {e}");
            None
        }
    }
}

pub(super) fn save(repo: &RepoRef, state: &GroupState) {
    let Some(path) = path(repo) else {
        return;
    };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    match serde_json::to_vec_pretty(state) {
        Ok(bytes) => {
            if let Err(e) = gitpoulpi_config::secure_fs::write_private(&path, &bytes) {
                tracing::warn!(target: "daemon", "could not save the grouped run: {e}");
            }
        }
        Err(e) => tracing::warn!(target: "daemon", "could not encode the grouped run: {e}"),
    }
}

pub(super) fn clear(repo: &RepoRef) {
    if let Some(path) = path(repo) {
        let _ = std::fs::remove_file(path);
    }
}
