//! What a delivered refresh changed, as plain flags.
//!
//! A frontend that keeps its own copy of the repository's state wants to know
//! which parts of it to replace without hashing the status itself. The client
//! already hashes every read it makes (for the idle-poll fingerprint and the
//! Log layout cache), so the answer costs one comparison per part:
//! [`SummaryTracker`] remembers the hashes of the last outcome it saw and
//! [`RefreshSummary`] says which parts of the next one differ.

use std::hash::{Hash, Hasher};

use super::*;

/// The hashes of the parts of one refresh that [`RefreshSummary`] compares.
/// `None` is a part the scope left out (nothing was read, so nothing moved).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(super) struct PartHashes {
    /// The status is read by every refresh.
    pub(super) status: u64,
    /// Which local branch HEAD is on and where it points. Part of the refs read.
    pub(super) head: Option<u64>,
    pub(super) worktrees: Option<u64>,
    pub(super) submodules: Option<u64>,
    /// Conflicts, the operation in progress and the stale index lock.
    pub(super) conflicts: Option<u64>,
    pub(super) remotes: Option<u64>,
    /// The ssh pin, the sparse checkout state and the repository config.
    pub(super) config: Option<u64>,
}

/// Which parts of the repository's state a delivered [`RefreshOutcome`]
/// changed, compared with the outcome delivered before it (see
/// [`RefreshOutcome::summary`]). A part the refresh's scope did not re-read is
/// never reported as changed.
///
/// The undo journal is not a refresh read: it arrives on its own
/// ([`LinkEvent::JournalChanged`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RefreshSummary {
    /// The first outcome of the link: every part it read is new.
    pub first: bool,
    /// The commit graph (the Log).
    pub log: bool,
    /// The local branch HEAD is on, or the commit it points to, moved. A
    /// detached HEAD is seen through the Log and refs instead.
    pub head: bool,
    /// The branches and tags.
    pub refs: bool,
    /// The working tree and index (the status).
    pub working_tree: bool,
    pub stashes: bool,
    pub worktrees: bool,
    pub submodules: bool,
    /// The conflicts, the merge/rebase/cherry-pick in progress or the index lock.
    pub conflicts: bool,
    pub remotes: bool,
    /// The ssh pin, the sparse checkout or the repository's own config.
    pub config: bool,
    /// A fresh LFS snapshot differing from the last one.
    pub lfs: bool,
}

impl RefreshSummary {
    /// Folds `other` into `self` (each flag is the OR of the two): for a
    /// frontend that coalesces several delivered refreshes into one redraw and
    /// must still replace every part any of them changed.
    pub fn merge(&mut self, other: RefreshSummary) {
        self.first |= other.first;
        self.log |= other.log;
        self.head |= other.head;
        self.refs |= other.refs;
        self.working_tree |= other.working_tree;
        self.stashes |= other.stashes;
        self.worktrees |= other.worktrees;
        self.submodules |= other.submodules;
        self.conflicts |= other.conflicts;
        self.remotes |= other.remotes;
        self.config |= other.config;
        self.lfs |= other.lfs;
    }

    /// Whether any part changed.
    pub fn any(&self) -> bool {
        self.log
            || self.head
            || self.refs
            || self.working_tree
            || self.stashes
            || self.worktrees
            || self.submodules
            || self.conflicts
            || self.remotes
            || self.config
            || self.lfs
    }
}

/// Remembers the hashes of the last outcome delivered and compares the next.
#[derive(Debug, Default)]
pub(super) struct SummaryTracker {
    seen: bool,
    parts: PartHashes,
    reads: ReadHashes,
}

impl SummaryTracker {
    /// Records `outcome` as delivered and says what differs from the one
    /// before it.
    pub(super) fn observe(&mut self, outcome: &RefreshOutcome) -> RefreshSummary {
        fn step(held: &mut Option<u64>, new: Option<u64>) -> bool {
            match new {
                None => false,
                Some(h) if *held == Some(h) => false,
                Some(h) => {
                    *held = Some(h);
                    true
                }
            }
        }
        let new = outcome.parts;
        let reads = outcome.read_hashes;
        let first = !self.seen;
        self.seen = true;
        let working_tree = self.parts.status != new.status || first;
        self.parts.status = new.status;
        RefreshSummary {
            first,
            working_tree,
            head: step(&mut self.parts.head, new.head),
            worktrees: step(&mut self.parts.worktrees, new.worktrees),
            submodules: step(&mut self.parts.submodules, new.submodules),
            conflicts: step(&mut self.parts.conflicts, new.conflicts),
            remotes: step(&mut self.parts.remotes, new.remotes),
            config: step(&mut self.parts.config, new.config),
            log: step(&mut self.reads.log, reads.log),
            refs: step(&mut self.reads.refs, reads.refs),
            stashes: step(&mut self.reads.stashes, reads.stashes),
            lfs: step(&mut self.reads.lfs, reads.lfs),
        }
    }
}

/// The hash of where HEAD is: the branch it is on and the commit it points to.
pub(super) fn head_hash(refs: &GitResult<Vec<Ref>>) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    if let Ok(refs) = refs {
        if let Some(head) = refs.iter().find(|r| r.is_head) {
            format!("{:?}", head.kind).hash(&mut hasher);
            head.target.0.hash(&mut hasher);
        }
    }
    hasher.finish()
}

/// Folds several optional hashes into one (the parts of a group).
pub(super) fn combine(parts: &[u64]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    parts.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod merge_tests {
    use super::*;

    #[test]
    fn merging_ors_every_flag() {
        let mut a = RefreshSummary {
            log: true,
            first: true,
            ..RefreshSummary::default()
        };
        a.merge(RefreshSummary {
            refs: true,
            lfs: true,
            ..RefreshSummary::default()
        });
        assert!(a.first && a.log && a.refs && a.lfs);
        assert!(!a.working_tree && !a.stashes && !a.config);
        let mut quiet = RefreshSummary::default();
        quiet.merge(RefreshSummary::default());
        assert!(!quiet.any());
    }
}
