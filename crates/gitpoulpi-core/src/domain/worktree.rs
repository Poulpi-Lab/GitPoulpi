use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// One entry of `git worktree list`: the repository's own working
/// directory plus every linked worktree added with `git worktree add`.
///
/// Note the deliberate contract both backends converge on: **the main
/// worktree is always included, first, with `is_main: true`**. That
/// isn't free - `git worktree list` reports it but
/// `git2::Repository::worktrees()` lists *only* linked worktrees, so the
/// git2 backend has to synthesize it. Listing it matters because a repo
/// with no linked worktrees still has one worktree, and a UI that hides
/// the main entry would show an empty pane for the very repo you're
/// looking at.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Worktree {
    pub path: PathBuf,
    /// Short branch name (`refs/heads/` stripped), or `None` when this
    /// worktree has a detached HEAD.
    pub branch: Option<String>,
    pub is_main: bool,
    pub is_locked: bool,
}
