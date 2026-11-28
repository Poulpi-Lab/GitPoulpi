//! Where stash rows sit among the commits (the terminal UI's `stash.rs`).

use gitpoulpi_core::domain::{CommitNode, Oid};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GraphStash {
    pub id: u32,
    pub commit: Oid,
    pub base: Oid,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StashSlot {
    pub before: usize,
    pub stash: usize,
}

pub fn place_stashes(nodes: &[CommitNode], stashes: &[GraphStash]) -> Vec<StashSlot> {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogEntry {
    Wip,
    Commit(usize),
    Stash(usize),
}

/// The Log's display order: the WIP row (if any), then the commits with the stash rows between.
pub fn display_order(wip: bool, slots: &[StashSlot], commits: usize) -> Vec<LogEntry> {
    let mut out = Vec::with_capacity(commits + slots.len() + 1);
    if wip {
        out.push(LogEntry::Wip);
    }
    let mut next = 0;
    for i in 0..commits {
        while let Some(s) = slots.get(next).filter(|s| s.before == i) {
            out.push(LogEntry::Stash(next));
            let _ = s;
            next += 1;
        }
        out.push(LogEntry::Commit(i));
    }
    out
}
