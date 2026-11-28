//! The commit graph: lane layout (from the shared `gitpoulpi-app` crate) and its vector painter,
//! behind a deliberately small interface.
//!
//! Everything outside this folder talks to [`GraphLayout`], [`Entry`], [`RowGraphic`] and
//! [`Palette`] only; no lane, cell or colour-index type leaks out.

#[cfg(test)]
mod fixtures;
#[cfg(test)]
mod golden_tests;
mod paint;

use std::collections::HashSet;

use gitpoulpi_app::graph as app;
use gitpoulpi_core::domain::{CommitNode, Oid, Ref, Stash};
use gpui_kit::{Bounds, Hsla, Pixels, Window};

pub use paint::{Palette, CH, CW};

/// One row of the Log, in display order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Entry {
    /// The working-tree pseudo commit on top.
    Wip,
    /// Index into the `nodes` the layout was built from.
    Commit(usize),
    /// Index into the `stashes` the layout was built from.
    Stash(usize),
}

/// Everything the layout is computed from.
pub struct GraphInput<'a> {
    pub nodes: &'a [CommitNode],
    /// Every ref; the commits they point at get a filled node.
    pub refs: &'a [Ref],
    pub stashes: &'a [Stash],
    /// A dirty working tree gets the WIP row.
    pub dirty: bool,
    /// The commit HEAD points at, where the WIP row hangs.
    pub head: Option<Oid>,
    /// Number of colours in the active lane palette.
    pub palette_len: usize,
}

/// The computed graph: display order plus one drawable row per entry.
#[derive(Clone, Default)]
pub struct GraphLayout {
    grid: app::GraphGrid,
    entries: Vec<Entry>,
    /// For each input stash, its slot in the lane layout (stashes without a commit or base have none).
    slot_of_stash: Vec<Option<usize>>,
    max_cells: usize,
}

impl GraphLayout {
    pub fn build(input: &GraphInput<'_>) -> Self {
        let mut stashes = Vec::new();
        let mut original = Vec::new();
        for (ix, s) in input.stashes.iter().enumerate() {
            if let (Some(commit), Some(base)) = (s.commit, s.base) {
                stashes.push(app::GraphStash {
                    id: s.id.0,
                    commit,
                    base,
                    timestamp: s.timestamp,
                });
                original.push(ix);
            }
        }
        let wip = input.dirty.then_some(app::WipAnchor { head: input.head });
        // The shared layout scans the refs once per commit; one ref per target keeps that scan short
        // on repositories with thousands of refs.
        let mut seen = HashSet::new();
        let refs: Vec<Ref> = input
            .refs
            .iter()
            .filter(|r| seen.insert(r.target))
            .cloned()
            .collect();
        let grid =
            app::build_graph_grid_themed(input.nodes, &refs, wip, &stashes, input.palette_len);
        let mut slot_of_stash = vec![None; input.stashes.len()];
        for (slot, placed) in grid.stash_slots.iter().enumerate() {
            if let Some(&orig) = original.get(placed.stash) {
                slot_of_stash[orig] = Some(slot);
            }
        }
        let order = grid.log_rows();
        let entries = (0..order.len(input.nodes.len()))
            .filter_map(|display| match order.entry(display) {
                app::LogEntry::Wip => Some(Entry::Wip),
                app::LogEntry::Commit(i) => Some(Entry::Commit(i)),
                app::LogEntry::Stash(slot) => {
                    let placed = grid.stash_slots.get(slot)?;
                    original.get(placed.stash).map(|&orig| Entry::Stash(orig))
                }
            })
            .collect();
        let max_cells = grid.rows.iter().map(|r| r.cells.len()).max().unwrap_or(0);
        Self {
            grid,
            entries,
            slot_of_stash,
            max_cells,
        }
    }

    /// The rows in display order.
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Widest row, in cells (two cells per lane): what "fit the Graph column" measures.
    pub fn max_cells(&self) -> usize {
        self.max_cells
    }

    /// The drawable row of an entry.
    pub fn graphic(&self, entry: Entry) -> Option<RowGraphic> {
        let row = match entry {
            Entry::Wip => self.grid.wip.clone(),
            Entry::Commit(i) => self.grid.rows.get(i).cloned(),
            Entry::Stash(orig) => {
                let slot = (*self.slot_of_stash.get(orig)?)?;
                self.grid.stashes.get(slot).cloned()
            }
        };
        row.map(RowGraphic)
    }
}

/// One row's graph cells, ready to paint.
#[derive(Clone)]
pub struct RowGraphic(app::GraphRow);

impl RowGraphic {
    /// The colour of this row's own lane (what its ref pills are tinted with).
    pub fn lane_color(&self, palette: &Palette) -> Hsla {
        palette.color(self.0.color)
    }

    /// Paints the row into `bounds`. With `connector`, the line from the Ref column's pill to the node.
    pub fn paint(
        &self,
        window: &mut Window,
        bounds: Bounds<Pixels>,
        row_index: usize,
        connector: bool,
        palette: &Palette,
    ) {
        paint::paint_row(window, bounds, &self.0, row_index, connector, palette);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_has_no_rows() {
        let layout = GraphLayout::build(&GraphInput {
            nodes: &[],
            refs: &[],
            stashes: &[],
            dirty: false,
            head: None,
            palette_len: 8,
        });
        assert!(layout.entries().is_empty());
        assert_eq!(layout.max_cells(), 0);
    }

    #[test]
    fn dirty_tree_adds_a_wip_row() {
        let layout = GraphLayout::build(&GraphInput {
            nodes: &[],
            refs: &[],
            stashes: &[],
            dirty: true,
            head: None,
            palette_len: 8,
        });
        assert_eq!(layout.entries(), &[Entry::Wip]);
        assert!(layout.graphic(Entry::Wip).is_some());
    }
}
