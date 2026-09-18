//! Stashes in the commit graph: where each stash row goes
//! among the commits, and the one mapping between a Log display index and
//! what is drawn there.
//!
//! A stash is laid out as a **pseudo-commit**, the way the WIP row is: its
//! only parent is its base (the stash commit's first parent), so the lane
//! walk in [`super::layout`] draws the connector down to the base with the
//! same code that joins every other lane, and marks it dashed. This file
//! decides *where* the rows go; the layout decides how they are drawn.
//!
//! # Placement
//!
//! A stash sits at its **date** among the commits, and never below its
//! base: it goes right above the first commit, from the top down to the
//! base itself, that is not newer than the stash. A stash made on an old
//! commit today is therefore near the top, with a long dashed lane down to
//! its base. Several stashes above the same
//! commit keep the stash list's own order (`stash@{0}` first).
//!
//! **A stash whose base is not loaded is not drawn** -- lazy loading has
//! not reached it yet, or solo filtered it out. It appears when the page
//! holding its base arrives. Placement reads `nodes[..=base]` only, so a
//! page appended further down never moves a stash already placed.
//!
//! # The Log's row arithmetic
//!
//! [`LogRows`] is the one place a display index becomes "the WIP row",
//! "commit `i`" or "stash row `k`", and back. A stash row between two
//! commits shifts every commit below it by one, so an open-coded
//! `+ has_wip` would put clicks, menus and the selection on the wrong row:
//! the Log's most repeated bug, see `docs/developer/where-we-are.md`.
use gitpoulpi_core::domain::{CommitNode, Oid};

/// A stash, as the graph needs it: its own commit, the commit it was made
/// on, when, and its `stash@{n}` index to find it again.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GraphStash {
    /// `stash@{n}`'s `n`.
    pub id: u32,
    pub commit: Oid,
    pub base: Oid,
    pub timestamp: i64,
}

impl GraphStash {
    /// The stashes the graph can draw: the ones whose commit and base the
    /// backend reported (always, from a current daemon; see
    /// `gitpoulpi_core::domain::Stash`).
    pub(crate) fn from_stashes(stashes: &[gitpoulpi_core::domain::Stash]) -> Vec<GraphStash> {
        stashes
            .iter()
            .filter_map(|s| {
                Some(GraphStash {
                    id: s.id.0,
                    commit: s.commit?,
                    base: s.base?,
                    timestamp: s.timestamp,
                })
            })
            .collect()
    }
}

/// One placed stash row: it sits directly above commit `before`, and it is
/// `stashes[stash]` of the slice handed to [`place_stashes`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StashSlot {
    pub before: usize,
    pub stash: usize,
}

/// Where each stash row goes. Sorted by `before`, then by the stash list's
/// own order, which is display order. See the module header for the rule.
pub(crate) fn place_stashes(nodes: &[CommitNode], stashes: &[GraphStash]) -> Vec<StashSlot> {
    let mut slots: Vec<StashSlot> = stashes
        .iter()
        .enumerate()
        .filter_map(|(k, s)| {
            let base = nodes.iter().position(|n| n.id == s.base)?;
            let before = nodes[..=base]
                .iter()
                .position(|n| n.timestamp <= s.timestamp)
                .unwrap_or(base);
            Some(StashSlot { before, stash: k })
        })
        .collect();
    slots.sort_by_key(|s| (s.before, s.stash));
    slots
}

/// What one Log display index shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum LogEntry {
    /// The WIP row, display index 0 on a dirty tree.
    Wip,
    /// `nodes[i]`. Not bounds-checked: past the end of the history is
    /// whatever the caller's `nodes.get(i)` says it is.
    Commit(usize),
    /// Stash row `k`: `slots[k]`, in display order.
    Stash(usize),
}

/// The Log's display order: the WIP row (if any), then the commits with
/// the stash rows between them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct LogRows {
    wip: bool,
    slots: Vec<StashSlot>,
}

impl LogRows {
    pub(crate) fn new(wip: bool, slots: Vec<StashSlot>) -> LogRows {
        LogRows { wip, slots }
    }

    /// Whether the WIP row is the first row.
    pub(crate) fn has_wip(&self) -> bool {
        self.wip
    }

    /// The placed stash rows, in display order.
    pub(crate) fn slots(&self) -> &[StashSlot] {
        &self.slots
    }

    /// How many rows the Log has for `commits` commits.
    pub(crate) fn len(&self, commits: usize) -> usize {
        usize::from(self.wip) + self.slots.len() + commits
    }

    pub(crate) fn entry(&self, display: usize) -> LogEntry {
        let d = if self.wip {
            match display.checked_sub(1) {
                Some(d) => d,
                None => return LogEntry::Wip,
            }
        } else {
            display
        };
        // Stash row `k` is drawn after the `before` commits above it and
        // the `k` stash rows before it.
        for (k, slot) in self.slots.iter().enumerate() {
            let at = slot.before + k;
            if at == d {
                return LogEntry::Stash(k);
            }
            if at > d {
                return LogEntry::Commit(d - k);
            }
        }
        LogEntry::Commit(d - self.slots.len())
    }

    /// The display index of `nodes[i]`.
    pub(crate) fn commit_display_index(&self, i: usize) -> usize {
        usize::from(self.wip) + i + self.slots.iter().filter(|s| s.before <= i).count()
    }

    /// The display index of stash row `k`.
    pub(crate) fn stash_display_index(&self, k: usize) -> Option<usize> {
        let slot = self.slots.get(k)?;
        Some(usize::from(self.wip) + slot.before + k)
    }
}
