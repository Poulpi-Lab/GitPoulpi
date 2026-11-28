//! What the Log holds once loaded: commits, refs, stashes, working changes and the graph layout.

use std::collections::HashMap;
use std::sync::Arc;

use gitpoulpi_app::graph::{solo_filter_nodes, solo_target_oid};
use gitpoulpi_core::domain::{CommitNode, Oid, Ref, RefKind, Stash, StatusEntry};

use std::collections::HashSet;

use crate::daemon::Walk;
use crate::graph::{Entry, GraphInput, GraphLayout};

/// What a selected Log row stands for.
#[derive(Clone)]
pub enum Selection {
    Wip,
    Commit(CommitNode),
    Stash(Stash),
}

/// What a selected row is, independent of where it sits: a reload keeps the selection on it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RowKey {
    Wip,
    Commit(Oid),
    Stash(Option<Oid>, String),
}

impl RowKey {
    fn of(selection: &Selection) -> Self {
        match selection {
            Selection::Wip => RowKey::Wip,
            Selection::Commit(n) => RowKey::Commit(n.id),
            Selection::Stash(s) => RowKey::Stash(s.commit, s.message.clone()),
        }
    }
}

#[derive(Clone, Default)]
pub struct LogData {
    /// Every commit held, in walk order: what paging appends to.
    pub held: Arc<Vec<CommitNode>>,
    /// The commits the Log shows: `held`, or only what the soloed refs reach.
    pub nodes: Arc<Vec<CommitNode>>,
    pub refs: Arc<Vec<Ref>>,
    /// Indices into `refs`, by the commit they point at.
    pub refs_by_commit: Arc<HashMap<Oid, Vec<usize>>>,
    pub stashes: Arc<Vec<Stash>>,
    pub status: Arc<Vec<StatusEntry>>,
    pub head_branch: Option<String>,
    pub layout: Arc<GraphLayout>,
    /// What the next page continues.
    pub walk: Option<Walk>,
    /// The commit HEAD points at (also when HEAD is detached and no ref says so).
    pub head: Option<Oid>,
}

impl LogData {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        held: Arc<Vec<CommitNode>>,
        nodes: Arc<Vec<CommitNode>>,
        refs: Arc<Vec<Ref>>,
        stashes: Arc<Vec<Stash>>,
        status: Vec<StatusEntry>,
        walk: Option<Walk>,
        head: Option<Oid>,
        layout: GraphLayout,
    ) -> Self {
        let mut data = Self {
            held,
            nodes,
            refs,
            stashes,
            status: Arc::new(status),
            walk,
            head,
            layout: Arc::new(layout),
            ..Self::default()
        };
        data.index_refs();
        data
    }

    fn index_refs(&mut self) {
        let mut by: HashMap<Oid, Vec<usize>> = HashMap::new();
        for (i, r) in self.refs.iter().enumerate() {
            by.entry(r.target).or_default().push(i);
        }
        self.refs_by_commit = Arc::new(by);
        self.head_branch = self
            .refs
            .iter()
            .find(|r| r.is_head)
            .and_then(|r| match &r.kind {
                RefKind::LocalBranch { name } => Some(name.clone()),
                _ => None,
            });
    }

    /// The commit the WIP row hangs from.
    pub fn head_target(&self) -> Option<Oid> {
        self.head
            .or_else(|| self.refs.iter().find(|r| r.is_head).map(|r| r.target))
    }

    pub fn dirty(&self) -> bool {
        !self.status.is_empty()
    }

    pub fn entry_count(&self) -> usize {
        self.layout.entries().len()
    }

    pub fn selection_at(&self, ix: usize) -> Option<Selection> {
        match *self.layout.entries().get(ix)? {
            Entry::Wip => Some(Selection::Wip),
            Entry::Commit(i) => self.nodes.get(i).cloned().map(Selection::Commit),
            Entry::Stash(i) => self.stashes.get(i).cloned().map(Selection::Stash),
        }
    }

    /// The identity of the row at `ix`.
    pub fn key_at(&self, ix: usize) -> Option<RowKey> {
        self.selection_at(ix).map(|s| RowKey::of(&s))
    }

    /// Display index of the row `key` names, if it is still there.
    pub fn position_of_key(&self, key: &RowKey) -> Option<usize> {
        (0..self.entry_count()).find(|&ix| self.key_at(ix).as_ref() == Some(key))
    }

    /// Display index of a commit.
    pub fn position_of(&self, oid: Oid) -> Option<usize> {
        let ci = self.nodes.iter().position(|n| n.id == oid)?;
        self.layout
            .entries()
            .iter()
            .position(|e| *e == Entry::Commit(ci))
    }
}

/// The commits the Log shows for the soloed refs: everything they reach (first parents and merge
/// parents alike), in held order (`gitpoulpi_app::graph::solo_filter_nodes`). Soloed refs that no longer
/// exist are skipped, and when nothing is soloed or nothing resolves the whole list is shown (solo
/// silently doing nothing beats a blank graph), the terminal UI's rule. The same list (`Arc`) is
/// returned when nothing is filtered.
pub fn shown_nodes(
    held: &Arc<Vec<CommitNode>>,
    refs: &[Ref],
    solo: &HashSet<String>,
) -> Arc<Vec<CommitNode>> {
    if solo.is_empty() {
        return held.clone();
    }
    let targets: Vec<Oid> = solo
        .iter()
        .filter_map(|name| solo_target_oid(name, refs))
        .collect();
    let kept = solo_filter_nodes(held, &targets);
    if kept.is_empty() {
        held.clone()
    } else {
        Arc::new(kept)
    }
}

/// The graph layout for a set of loaded commits. Run on a background thread: it is the one heavy step
/// of loading a page.
pub fn compute_layout(
    nodes: &[CommitNode],
    refs: &[Ref],
    stashes: &[Stash],
    dirty: bool,
    head: Option<Oid>,
    palette_len: usize,
) -> GraphLayout {
    GraphLayout::build(&GraphInput {
        nodes,
        refs,
        stashes,
        dirty,
        head,
        palette_len,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::RefVisibility;

    fn node(n: u8, parents: &[u8]) -> CommitNode {
        CommitNode {
            id: Oid([n; 20]),
            parents: parents.iter().map(|p| Oid([*p; 20])).collect(),
            summary: format!("c{n}"),
            message: String::new(),
            author_name: String::new(),
            author_email: String::new(),
            timestamp: 0,
            refs: vec![],
        }
    }

    fn branch(name: &str, n: u8) -> Ref {
        Ref {
            kind: RefKind::LocalBranch { name: name.into() },
            target: Oid([n; 20]),
            visibility: RefVisibility::Shown,
            is_head: false,
            upstream: None,
            tag_sync: None,
            multi_remote: None,
        }
    }

    /// 4 -> 3 -> 1 (main) and 2 -> 1 (side): a merge-free fork.
    fn held() -> Arc<Vec<CommitNode>> {
        Arc::new(vec![
            node(4, &[3]),
            node(2, &[1]),
            node(3, &[1]),
            node(1, &[]),
        ])
    }

    fn solo(names: &[&str]) -> HashSet<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn solo_keeps_what_the_soloed_refs_reach_in_held_order() {
        let refs = vec![branch("main", 4), branch("side", 2)];
        let shown = shown_nodes(&held(), &refs, &solo(&["side"]));
        let ids: Vec<u8> = shown.iter().map(|n| n.id.0[0]).collect();
        assert_eq!(ids, vec![2, 1]);
        let shown = shown_nodes(&held(), &refs, &solo(&["side", "main"]));
        assert_eq!(shown.len(), 4, "soloing a second ref adds its ancestry");
    }

    #[test]
    fn solo_that_resolves_to_nothing_shows_everything() {
        let refs = vec![branch("main", 4)];
        let h = held();
        assert!(Arc::ptr_eq(&shown_nodes(&h, &refs, &solo(&[])), &h));
        let shown = shown_nodes(&h, &refs, &solo(&["deleted-meanwhile"]));
        assert_eq!(shown.len(), 4);
    }
}
