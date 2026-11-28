//! What a background probe found out about a repository, shared by every
//! frontend's repository lists (the Radar, the Home list).

/// What a background probe found out about a repository, filled in after
/// the row is already on screen.
///
/// Every field costs a `git` subprocess, so none is read on the render path:
/// `repo_probe` runs off the input loop and the row is drawn the moment the
/// overlay opens, with `facts: None` meaning *still looking* (see "never
/// block the input loop on a git subprocess call" in
/// `docs/developer/where-we-are.md`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RepoFacts {
    /// The branch actually checked out now, which may differ from the one
    /// the recent list remembered.
    pub branch: Option<String>,
    /// Anything at all in the working tree or the index.
    pub dirty: bool,
    pub ahead: u32,
    pub behind: u32,
    /// Work-in-progress counters, `✎ n  + n`: files changed
    /// (modified, deleted, renamed, conflicted - anything tracked) and
    /// files added (untracked, or new in the index).
    pub modified: u32,
    pub added: u32,
    /// Who the repository belongs to: the owner or namespace from the default
    /// remote's URL (`Poulpi-Lab`, `group/sub`), `None` when there is no
    /// remote to read one from (the caller falls back to the parent folder's
    /// name).
    pub owner: Option<String>,
}
