//! The commit graph: lane layout and its vector painter, behind a deliberately small interface.
//!
//! The lane algorithm (`lanes.rs`) is a copy of the terminal UI's and will be replaced by the shared
//! `gitpoulpi-app` crate. Everything outside this folder talks to [`GraphLayout`], [`Entry`],
//! [`RowGraphic`] and [`Palette`] only; no lane, cell or colour-index type leaks out, so swapping the
//! implementation means rewriting `mod.rs` and `paint.rs`.

mod lanes;
mod paint;

use std::collections::HashSet;

use gitpoulpi_core::domain::{CommitNode, Oid, Stash};
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
    /// Commits some ref points at (they get a filled node).
    pub ref_targets: &'a HashSet<Oid>,
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
    grid: lanes::GraphGrid,
    entries: Vec<Entry>,
    /// For each input stash, its slot in the lane layout (stashes without a commit or base have none).
    slot_of_stash: Vec<Option<usize>>,
    max_cells: usize,
}

impl GraphLayout {
    pub fn build(input: &GraphInput<'_>) -> Self {
        let mut graph_stashes = Vec::new();
        let mut original = Vec::new();
        for (ix, s) in input.stashes.iter().enumerate() {
            if let (Some(commit), Some(base)) = (s.commit, s.base) {
                graph_stashes.push(lanes::GraphStash {
                    id: s.id.0,
                    commit,
                    base,
                    timestamp: s.timestamp,
                });
                original.push(ix);
            }
        }
        let wip = input.dirty.then_some(lanes::WipAnchor { head: input.head });
        let grid = lanes::build_graph_grid(
            input.nodes,
            input.ref_targets,
            wip,
            &graph_stashes,
            input.palette_len,
        );
        let mut slot_of_stash = vec![None; input.stashes.len()];
        for (slot, placed) in grid.stash_slots.iter().enumerate() {
            if let Some(&orig) = original.get(placed.stash) {
                slot_of_stash[orig] = Some(slot);
            }
        }
        let entries =
            lanes::display_order(grid.wip.is_some(), &grid.stash_slots, input.nodes.len())
                .into_iter()
                .filter_map(|e| match e {
                    lanes::LogEntry::Wip => Some(Entry::Wip),
                    lanes::LogEntry::Commit(i) => Some(Entry::Commit(i)),
                    lanes::LogEntry::Stash(slot) => {
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
pub struct RowGraphic(lanes::GraphRow);

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
            ref_targets: &HashSet::new(),
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
            ref_targets: &HashSet::new(),
            stashes: &[],
            dirty: true,
            head: None,
            palette_len: 8,
        });
        assert_eq!(layout.entries(), &[Entry::Wip]);
        assert!(layout.graphic(Entry::Wip).is_some());
    }
}
