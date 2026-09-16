//! The `search` part of the config schema: the workspace search's commit
//! index (the workspaces design).

use serde::{Deserialize, Serialize};

/// Commit messages indexed per repository unless the config says otherwise.
pub const DEFAULT_SEARCH_COMMIT_LIMIT: u32 = 200_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SearchConfig {
    /// Live: how many commits (newest first, over every branch and tag) the
    /// search index keeps per repository. The index lives in the daemon's
    /// memory (about 100 bytes plus the message per commit), so a huge
    /// history is cut here; a search then says its answer is partial. `0`
    /// turns commit search off.
    pub commit_limit: u32,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            commit_limit: DEFAULT_SEARCH_COMMIT_LIMIT,
        }
    }
}
