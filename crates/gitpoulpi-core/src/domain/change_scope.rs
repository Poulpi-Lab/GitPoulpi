use serde::{Deserialize, Serialize};

/// What a mutation can have changed, so a client re-reads only that.
///
/// A set of flags in one `u16` on the wire. `Event::RepoChanged` carries it
/// with `#[serde(default)]`, and the default is [`ChangeScope::ALL`]: a
/// daemon that predates the field sends none, and a client then does a full
/// refresh, exactly as before. The status read is not a flag; a client
/// always re-reads it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ChangeScope(u16);

impl ChangeScope {
    pub const NONE: Self = Self(0);
    /// The index (staged side of the status).
    pub const INDEX: Self = Self(1);
    /// The working tree files.
    pub const WORKTREE: Self = Self(1 << 1);
    /// Branches, tags, remote-tracking refs and HEAD (the `Refs` read).
    pub const REFS: Self = Self(1 << 2);
    /// The commit graph.
    pub const LOG: Self = Self(1 << 3);
    pub const STASHES: Self = Self(1 << 4);
    /// The linked worktree list.
    pub const WORKTREES: Self = Self(1 << 5);
    /// Remotes and `core.sshCommand`.
    pub const CONFIG: Self = Self(1 << 6);
    /// Git LFS state.
    pub const LFS: Self = Self(1 << 7);
    /// Unmerged paths and the rebase in progress.
    pub const CONFLICTS: Self = Self(1 << 8);
    pub const ALL: Self = Self((1 << 9) - 1);

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Whether every flag of `other` is set.
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Whether any flag of `other` is set.
    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    /// Whether the LFS state can have changed: an LFS or config scope, or
    /// a checkout (refs and worktree files together).
    pub const fn touches_lfs(self) -> bool {
        self.intersects(Self::LFS.union(Self::CONFIG))
            || self.contains(Self::REFS.union(Self::WORKTREE))
    }
}

impl Default for ChangeScope {
    fn default() -> Self {
        Self::ALL
    }
}

impl std::ops::BitOr for ChangeScope {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        self.union(rhs)
    }
}

impl std::ops::BitOrAssign for ChangeScope {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = self.union(rhs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lfs_relevance() {
        let stage = ChangeScope::INDEX | ChangeScope::WORKTREE | ChangeScope::CONFLICTS;
        assert!(!stage.touches_lfs());
        assert!(ChangeScope::CONFIG.touches_lfs());
        assert!(ChangeScope::ALL.touches_lfs());
    }
}
