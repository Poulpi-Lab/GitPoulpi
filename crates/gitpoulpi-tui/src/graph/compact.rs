//! The compact graph: at the Graph column's minimum width every commit sits
//! on one lane, one under the other, with no branch or merge lines.
//!
//! **A function of the width, nothing stored.** [`is_compact`] is the whole
//! rule: a column [`COMPACT_W`] cells wide or less is compact, one cell wider
//! draws the tree again. Both graphs (the Log's Graph column and the commit
//! column's Graph part) ask it with the width they are about to draw into, and
//! both renderers (characters and the picture) go through the same cells here,
//! so they cannot disagree about what a compact row is.

use ratatui::text::{Line, Span};

use super::connector::node_column;
use super::part::{NodeSet, Sides, TreeCell, TreePart};
use super::raster::RasterRow;

/// The Graph column's least width, in cells, and the widest one that is
/// compact: the cells of one commit dot (an avatar circle is two cells
/// across).
pub(crate) const COMPACT_W: usize = 2;

/// Whether a Graph column `width` cells wide draws the compact graph.
pub(crate) fn is_compact(width: usize) -> bool {
    width <= COMPACT_W
}

/// A row's cells reduced to its commit dot, alone in the first cell: no
/// strokes in any direction. A row with no node (it has none to show) is blank.
pub(crate) fn compact_cells(cells: &[TreeCell]) -> Vec<TreeCell> {
    let Some(node) = node_column(cells) else {
        return vec![TreeCell::empty()];
    };
    let mut cell = cells[node];
    if let TreePart::Node { kind, .. } = cell.part {
        cell.part = TreePart::Node {
            kind,
            sides: Sides::NONE,
        };
    }
    cell.dashed = Sides::NONE;
    vec![cell]
}

/// [`compact_cells`] as the characters the text renderer draws.
pub(crate) fn compact_line(cells: &[TreeCell], set: NodeSet) -> Line<'static> {
    Line::from(
        compact_cells(cells)
            .into_iter()
            .map(|c| c.span(set))
            .collect::<Vec<Span<'static>>>(),
    )
}

/// The picture's rows reduced the same way (the avatar and the selection
/// stay, the ref connector has no lane to cross and goes).
pub(crate) fn compact_raster_rows(rows: &mut [RasterRow]) {
    for row in rows {
        row.cells = compact_cells(&row.cells);
        row.ref_connector = false;
    }
}
