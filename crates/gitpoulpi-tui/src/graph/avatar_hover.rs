//! Where the Graph column's commit avatars are on screen, for the hover
//! tooltip (`ui.avatars = "graph"`).
//!
//! The draw pass knows which rows carry a picture and where; the pointer
//! handler only has the geometry that survives between frames. So the draw
//! pass leaves the picture cells here and the handler
//! asks which commit's picture is under the pointer. Off, or a row without a
//! picture (a merge, a WIP row, a picture not fetched yet), there is no slot
//! and the plain dot has no tooltip.

use std::sync::Mutex;

use gitpoulpi_core::domain::Oid;
use ratatui::layout::Rect;

use super::raster::{avatar_left_pad, avatar_radius, RasterOptions};
use crate::terminal_image::CellSize;

/// A process-wide cell, not a thread-local: the frame is drawn on one thread
/// and the pointer is handled on another.
static SLOTS: Mutex<Vec<(Rect, Oid)>> = Mutex::new(Vec::new());

/// Replaces the picture cells of the frame just drawn (empty: none).
pub(crate) fn set_slots(slots: Vec<(Rect, Oid)>) {
    if let Ok(mut s) = SLOTS.lock() {
        *s = slots;
    }
}

/// The commit whose picture covers the cell `(col, row)`.
pub(crate) fn slot_at(col: u16, row: u16) -> Option<Oid> {
    SLOTS.lock().ok().and_then(|s| {
        s.iter()
            .find(|(r, _)| r.contains((col, row).into()))
            .map(|(_, id)| *id)
    })
}

/// The cells the picture of the node in cell `node` covers: `(first, count)`
/// relative to the first cell of the graph. The picture is as wide as two
/// cells, centred on the node cell and shifted right by the graph's left
/// pad (`raster::avatar_left_pad`); a sliver of a pixel or two in a third cell
/// does not count.
pub(crate) fn picture_cells(cell: CellSize, node: u16) -> (u16, u16) {
    let opt = RasterOptions::new(cell, 1).with_avatars(true);
    let cw = cell.width.max(1) as f32;
    let center = avatar_left_pad(&opt) as f32 + f32::from(node) * cw + cw / 2.0;
    let r = avatar_radius(&opt);
    let first = ((center - r) / cw).round().max(0.0) as u16;
    let end = ((center + r) / cw).round().max(0.0) as u16;
    (first, end.saturating_sub(first).max(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_picture_covers_two_cells_from_the_nodes_own() {
        for cell in [CellSize::FALLBACK, CellSize::HALF] {
            for node in [0u16, 1, 3] {
                assert_eq!(picture_cells(cell, node), (node, 2), "{cell:?} node {node}");
            }
        }
    }

    #[test]
    fn the_slots_are_found_by_cell() {
        let id = Oid([7; 20]);
        set_slots(vec![(Rect::new(4, 9, 2, 1), id)]);
        assert_eq!(slot_at(5, 9), Some(id));
        assert_eq!(slot_at(6, 9), None);
        assert_eq!(slot_at(5, 10), None);
        set_slots(Vec::new());
        assert_eq!(slot_at(5, 9), None);
    }
}
