//! The column boundary, shared by the three sizable tables: the Log, the
//! Radar and the commit column of the diff, file and blame views.
//!
//! One boundary is one [`Edge`]: the **gap** cell between two columns and the
//! **next** cell (the first one of the column after it). `ui.column_resize`
//! picks what is grabbed:
//!
//! - `line` (default): the gap cell, over the **whole height** of the table
//!   (header row to last body row). While it is hovered or dragged a thin
//!   accent `│` is painted down it; at rest the same glyph, muted, sits in the header
//!   and the body shows the neighbouring cells through the gap.
//! - `header`: the next cell, **on the header row only**, lit as a block
//!   (`paint_resize_strip`), no tick.
//!
//! The rect a table registers, the cells that light and the pointer shape all
//! come from [`grab_rect`] and [`paint_grab`], so "if it highlights, dragging
//! it works" holds in every table and both modes.

use gitpoulpi_config::ColumnResize;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use crate::th;

/// The thin line drawn down a hovered boundary.
const LINE: &str = "\u{2502}";
/// The resting tick in the header's gap cell.
const TICK: &str = LINE;

/// The mode in force (`ui.column_resize`).
pub(crate) fn mode() -> ColumnResize {
    crate::live_settings::column_resize()
}

/// One boundary between two columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Edge {
    /// The cell between the two columns (the left column's separator).
    pub(crate) gap_x: u16,
    /// The first cell of the column after it.
    pub(crate) next_x: u16,
}

impl Edge {
    /// The boundary whose right-hand column starts at `next_x`; the gap is the
    /// cell before it.
    pub(crate) fn before(next_x: u16) -> Self {
        Self {
            gap_x: next_x.saturating_sub(1),
            next_x,
        }
    }

    /// The grip on the last column of a table whose columns end at `end_x`
    /// (exclusive): the column's own last cell, in both modes. Never the cell
    /// past it, which is the pane's border.
    pub(crate) fn last(end_x: u16) -> Self {
        let cell = end_x.saturating_sub(1);
        Self {
            gap_x: cell,
            next_x: cell,
        }
    }

    /// The boundary after a last column: no cell follows it, so the gap and
    /// the next cell are the same one (past the column's end).
    pub(crate) fn after(end_x: u16) -> Self {
        Self {
            gap_x: end_x,
            next_x: end_x,
        }
    }
}

/// The cells a press can grab for `edge`. `header_y` is the header row and
/// `bottom` the first row below the table body (exclusive).
pub(crate) fn grab_rect(mode: ColumnResize, edge: Edge, header_y: u16, bottom: u16) -> Rect {
    match mode {
        ColumnResize::Line => Rect::new(
            edge.gap_x,
            header_y,
            1,
            bottom.saturating_sub(header_y).max(1),
        ),
        ColumnResize::Header => Rect::new(edge.next_x, header_y, 1, 1),
    }
}

/// How far right of the edge the pointer sits when it holds the boundary:
/// `1` on the gap (`line`), `0` on the next column's first cell (`header`).
/// Drag arithmetic that wants "where the next column starts" adds it.
pub(crate) fn pointer_to_edge(mode: ColumnResize, edge_has_next: bool) -> u16 {
    match mode {
        ColumnResize::Line if edge_has_next => 1,
        _ => 0,
    }
}

/// Lights a grabbed boundary: the thin line in `line` mode, the block of the
/// resize strip in `header` mode.
pub(crate) fn paint_grab(buf: &mut Buffer, mode: ColumnResize, strip: Rect) {
    match mode {
        ColumnResize::Header => crate::pane_layout::paint_resize_strip(buf, strip),
        ColumnResize::Line => {
            let visible = strip.intersection(buf.area);
            for y in visible.y..visible.y + visible.height {
                for x in visible.x..visible.x + visible.width {
                    let cell = &mut buf[(x, y)];
                    cell.set_symbol(LINE);
                    cell.set_style(Style::default().fg(th().ui.accent));
                }
            }
        }
    }
}

/// The resting tick in a header's gap cell (`line` mode only). A cell that
/// already carries header text is left alone: the tick never eats a letter.
pub(crate) fn paint_tick(buf: &mut Buffer, mode: ColumnResize, gap_x: u16, header_y: u16) {
    if mode != ColumnResize::Line || !buf.area.contains((gap_x, header_y).into()) {
        return;
    }
    let cell = &mut buf[(gap_x, header_y)];
    if cell.symbol().trim().is_empty() {
        cell.set_symbol(TICK);
        cell.set_style(Style::default().fg(th().ui.muted));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_grabs_the_gap_down_the_table_and_header_the_next_cell_on_the_header() {
        let edge = Edge::before(10);
        assert_eq!(
            grab_rect(ColumnResize::Line, edge, 1, 9),
            Rect::new(9, 1, 1, 8)
        );
        assert_eq!(
            grab_rect(ColumnResize::Header, edge, 1, 9),
            Rect::new(10, 1, 1, 1)
        );
    }

    #[test]
    fn a_last_column_has_one_cell_for_both_modes() {
        let edge = Edge::after(20);
        assert_eq!(grab_rect(ColumnResize::Line, edge, 0, 5).x, 20);
        assert_eq!(grab_rect(ColumnResize::Header, edge, 0, 5).x, 20);
        assert_eq!(pointer_to_edge(ColumnResize::Line, false), 0);
        assert_eq!(pointer_to_edge(ColumnResize::Line, true), 1);
        assert_eq!(pointer_to_edge(ColumnResize::Header, true), 0);
    }

    #[test]
    fn the_line_is_thin_and_the_block_is_not() {
        let area = Rect::new(0, 0, 6, 4);
        let strip = Rect::new(2, 0, 1, 4);
        let mut buf = Buffer::empty(area);
        paint_grab(&mut buf, ColumnResize::Line, strip);
        for y in 0..4 {
            assert_eq!(buf[(2, y)].symbol(), "\u{2502}");
            assert_eq!(buf[(2, y)].style().fg, Some(th().ui.accent));
            assert_ne!(buf[(2, y)].style().bg, Some(th().ui.accent));
        }
        let mut buf = Buffer::empty(area);
        paint_grab(&mut buf, ColumnResize::Header, strip);
        assert_eq!(buf[(2, 1)].style().bg, Some(th().ui.accent));
    }

    #[test]
    fn a_tick_only_fills_a_blank_header_cell_in_line_mode() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 6, 1));
        buf[(1, 0)].set_symbol("A");
        paint_tick(&mut buf, ColumnResize::Line, 1, 0);
        assert_eq!(buf[(1, 0)].symbol(), "A");
        paint_tick(&mut buf, ColumnResize::Line, 2, 0);
        assert_eq!(buf[(2, 0)].symbol(), TICK);
        paint_tick(&mut buf, ColumnResize::Header, 3, 0);
        assert_eq!(buf[(3, 0)].symbol(), " ");
    }
}
