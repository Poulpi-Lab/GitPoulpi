//! Stand-in for `worktrees.rs` when the `worktree` feature is off (custom
//! builds): the same signatures, every operation answers
//! `NotBuilt { feature: "worktree" }`. The internal "checked out elsewhere"
//! guard reads `worktree_list` instead, which is always compiled.

use std::path::Path;

use gitpoulpi_core::domain::{RepoRef, Worktree, WorktreeAddOptions};
use gitpoulpi_core::{Error, Result};

fn not_built<T>() -> Result<T> {
    Err(Error::NotBuilt {
        feature: "worktree",
    })
}

pub fn worktrees(_repo: &RepoRef) -> Result<Vec<Worktree>> {
    not_built()
}

pub fn worktree_add(_repo: &RepoRef, _path: &Path, _opts: &WorktreeAddOptions) -> Result<Worktree> {
    not_built()
}
