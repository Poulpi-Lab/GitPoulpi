//! What the Log holds once loaded: commits, refs, stashes, working changes and the graph layout.

use std::collections::HashMap;
use std::sync::Arc;

use gitpoulpi_core::domain::{CommitNode, Oid, Ref, RefKind, Stash, StatusEntry};

use crate::daemon::HistoryPlan;
use crate::graph::{Entry, GraphInput, GraphLayout};

/// What a selected Log row stands for.
#[derive(Clone)]
pub enum Selection {
    Wip,
    Commit(CommitNode),
    Stash(Stash),
}

#[derive(Clone, Default)]
pub struct LogData {
    pub nodes: Arc<Vec<CommitNode>>,
    pub refs: Arc<Vec<Ref>>,
    /// Indices into `refs`, by the commit they point at.
    pub refs_by_commit: Arc<HashMap<Oid, Vec<usize>>>,
    pub stashes: Arc<Vec<Stash>>,
    pub status: Arc<Vec<StatusEntry>>,
    pub head_branch: Option<String>,
    pub layout: Arc<GraphLayout>,
    pub plan: Option<HistoryPlan>,
    /// The commit HEAD points at (also when HEAD is detached and no ref says so).
    pub head: Option<Oid>,
}

impl LogData {
    pub fn new(
        nodes: Arc<Vec<CommitNode>>,
        refs: Arc<Vec<Ref>>,
        stashes: Arc<Vec<Stash>>,
        status: Vec<StatusEntry>,
        plan: HistoryPlan,
        head: Option<Oid>,
        layout: GraphLayout,
    ) -> Self {
        let mut data = Self {
            nodes,
            refs,
            stashes,
            status: Arc::new(status),
            plan: Some(plan),
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

    /// Display index of a commit.
    pub fn position_of(&self, oid: Oid) -> Option<usize> {
        let ci = self.nodes.iter().position(|n| n.id == oid)?;
        self.layout
            .entries()
            .iter()
            .position(|e| *e == Entry::Commit(ci))
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
