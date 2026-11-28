//! Multi-select of commits directly in the Log (the commit tree): Ctrl+click
//! or Space toggles a commit, Shift+click or Shift+Up/Down selects a range
//! from the anchor, Esc clears. With more than one commit selected,
//! Cherry-pick opens the interactive view (`rebase_ui`) directly.
//!
//! The selection is a set of `Oid`s kept *next to* the Log's own single
//! cursor (`TableState::selected`), never instead of it: the cursor keeps
//! driving the detail panel, and a plain click or plain arrow keys leave
//! this set alone or reset it.

use std::collections::HashSet;

use super::*;

/// What a Log row stands for in the multi-select: the WIP row, or a commit
/// (a stash row is its stash commit, an `Oid` of its own that no history
/// node shares).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RowKey {
    Wip,
    Id(Oid),
}

#[derive(Debug, Default, Clone)]
pub(crate) struct LogSelection {
    set: HashSet<Oid>,
    wip: bool,
    /// Where a Shift range starts: the last row toggled or clicked.
    anchor: Option<RowKey>,
}

impl LogSelection {
    pub(crate) fn len(&self) -> usize {
        self.set.len() + usize::from(self.wip)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub(crate) fn contains(&self, key: &RowKey) -> bool {
        match key {
            RowKey::Wip => self.wip,
            RowKey::Id(id) => self.set.contains(id),
        }
    }

    /// Whether the commit (or stash commit) `id` is selected.
    pub(crate) fn contains_id(&self, id: &Oid) -> bool {
        self.set.contains(id)
    }

    pub(crate) fn clear(&mut self) {
        self.set.clear();
        self.wip = false;
        self.anchor = None;
    }

    fn insert(&mut self, key: RowKey) {
        match key {
            RowKey::Wip => self.wip = true,
            RowKey::Id(id) => {
                self.set.insert(id);
            }
        }
    }

    fn remove(&mut self, key: RowKey) -> bool {
        match key {
            RowKey::Wip => std::mem::take(&mut self.wip),
            RowKey::Id(id) => self.set.remove(&id),
        }
    }

    /// A plain click: forget the selection, remember where a following
    /// Shift+click should start from.
    pub(crate) fn reset_to(&mut self, anchor: Option<RowKey>) {
        self.set.clear();
        self.wip = false;
        self.anchor = anchor;
    }

    /// Ctrl+click / Space: flips `key`. The row the cursor was on is part
    /// of the selection the moment a second one is added, the way
    /// Ctrl+click extends what is already highlighted.
    pub(crate) fn toggle(&mut self, key: RowKey, cursor: Option<RowKey>) {
        if self.is_empty() {
            if let Some(cursor) = cursor.filter(|c| *c != key) {
                self.insert(cursor);
            }
        }
        if !self.remove(key) {
            self.insert(key);
        }
        self.anchor = Some(key);
    }

    /// Shift+click / Shift+Up/Down: selects every row between the anchor
    /// (or `cursor_index` when there is none yet) and `target`, both
    /// display indexes into `display`.
    pub(crate) fn extend_to(
        &mut self,
        display: &[Option<RowKey>],
        target: usize,
        cursor_index: Option<usize>,
    ) {
        let anchor_index = self
            .anchor
            .and_then(|a| display.iter().position(|d| *d == Some(a)))
            .or(cursor_index);
        let Some(anchor_index) = anchor_index else {
            return;
        };
        if self.anchor.is_none() {
            self.anchor = display.get(anchor_index).copied().flatten();
        }
        let (lo, hi) = if anchor_index <= target {
            (anchor_index, target)
        } else {
            (target, anchor_index)
        };
        self.set.clear();
        self.wip = false;
        for key in display.iter().skip(lo).take(hi - lo + 1).flatten() {
            self.insert(*key);
        }
    }

    /// The selection in the Log's own order, newest (top row) first: what
    /// the comparison reads its two ends from. Rows that scrolled out of
    /// `display` are left out.
    pub(crate) fn in_display_order(&self, display: &[Option<RowKey>]) -> Vec<RowKey> {
        display
            .iter()
            .flatten()
            .filter(|k| self.contains(k))
            .copied()
            .collect()
    }

    /// The selected *commits* in the graph's own order, newest first -
    /// what the interactive view lists top to bottom. The WIP row and
    /// stashes are not commits to cherry-pick and are left out; so are
    /// commits that scrolled out of the loaded page (the caller compares
    /// the length).
    pub(crate) fn newest_first(
        &self,
        log_result: &gitpoulpi_core::Result<gitpoulpi_core::domain::CommitGraph>,
    ) -> Vec<Oid> {
        match log_result {
            Ok(graph) => graph
                .nodes
                .iter()
                .filter(|n| self.set.contains(&n.id))
                .map(|n| n.id)
                .collect(),
            Err(_) => Vec::new(),
        }
    }
}

/// What every display row of the Log stands for: `None` only for a row
/// that is nothing selectable (a stash whose commit is unknown).
pub(crate) fn display_commits(
    log_result: &gitpoulpi_core::Result<gitpoulpi_core::domain::CommitGraph>,
    stashes_result: &gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Stash>>,
    rows: &graph::LogRows,
    log_len: usize,
) -> Vec<Option<RowKey>> {
    let Ok(graph) = log_result else {
        return Vec::new();
    };
    (0..log_len)
        .map(|i| match rows.entry(i) {
            graph::LogEntry::Commit(n) => graph.nodes.get(n).map(|c| RowKey::Id(c.id)),
            graph::LogEntry::Wip => Some(RowKey::Wip),
            graph::LogEntry::Stash(_) => log_stash_at(stashes_result, rows, i)
                .and_then(|s| s.commit)
                .map(RowKey::Id),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::*;

    fn id(n: u8) -> RowKey {
        RowKey::Id(oid(n))
    }

    fn display() -> Vec<Option<RowKey>> {
        // A WIP row, then commits 3, 2, 1 (newest first).
        vec![Some(RowKey::Wip), Some(id(3)), Some(id(2)), Some(id(1))]
    }

    #[test]
    fn ctrl_click_adds_the_cursor_commit_then_toggles() {
        let mut sel = LogSelection::default();
        sel.toggle(id(1), Some(id(3)));
        assert_eq!(sel.len(), 2);
        assert!(sel.contains(&id(3)) && sel.contains(&id(1)));
        sel.toggle(id(1), Some(id(3)));
        assert!(!sel.contains(&id(1)));
        assert_eq!(sel.len(), 1);
    }

    #[test]
    fn space_on_the_cursor_commit_selects_just_it() {
        let mut sel = LogSelection::default();
        sel.toggle(id(2), Some(id(2)));
        assert_eq!(sel.len(), 1);
        assert!(sel.contains(&id(2)));
    }

    #[test]
    fn the_wip_row_joins_the_selection_like_a_commit() {
        let mut sel = LogSelection::default();
        sel.toggle(RowKey::Wip, Some(id(2)));
        assert_eq!(sel.len(), 2);
        assert!(sel.contains(&RowKey::Wip) && sel.contains(&id(2)));
        assert_eq!(
            sel.in_display_order(&display()),
            vec![RowKey::Wip, id(2)],
            "newest (top row) first, WIP on top"
        );
        sel.toggle(RowKey::Wip, Some(id(2)));
        assert!(!sel.contains(&RowKey::Wip));
    }

    #[test]
    fn shift_selects_the_range_from_the_anchor_wip_row_included() {
        let mut sel = LogSelection::default();
        sel.reset_to(Some(id(3)));
        sel.extend_to(&display(), 3, Some(1));
        assert_eq!(sel.len(), 3);
        // Extending back up to the WIP row takes it too.
        sel.extend_to(&display(), 0, Some(3));
        assert_eq!(sel.len(), 2);
        assert!(sel.contains(&RowKey::Wip) && sel.contains(&id(3)));
        assert!(!sel.contains(&id(1)));
    }

    #[test]
    fn shift_without_an_anchor_starts_at_the_cursor_row() {
        let mut sel = LogSelection::default();
        sel.extend_to(&display(), 2, Some(1));
        assert!(sel.contains(&id(3)) && sel.contains(&id(2)));
        assert_eq!(sel.len(), 2);
    }

    #[test]
    fn newest_first_follows_the_graph_order_and_leaves_the_wip_out() {
        let graph = Ok(gitpoulpi_core::domain::CommitGraph {
            nodes: vec![commit(3, &[2]), commit(2, &[1]), commit(1, &[])],
        });
        let mut sel = LogSelection::default();
        sel.toggle(id(1), None);
        sel.toggle(id(3), None);
        sel.toggle(RowKey::Wip, None);
        assert_eq!(sel.newest_first(&graph), vec![oid(3), oid(1)]);
    }

    #[test]
    fn a_plain_click_clears_the_selection() {
        let mut sel = LogSelection::default();
        sel.toggle(RowKey::Wip, Some(id(2)));
        sel.reset_to(Some(id(3)));
        assert!(sel.is_empty());
    }
}
