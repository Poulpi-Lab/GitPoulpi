//! The ref-to-commit connector (`ui.ref_connector`): a thin line from the
//! end of a ref pill in the Log's Ref column to its commit's node in the
//! Graph column.
//!
//! **It never moves anything.** The connector is laid only over cells that
//! are already blank; it never replaces a lane glyph and never adds or
//! removes a cell, so the graph's centring, the Log's column arithmetic
//! and every hit target are what they were without it.
//!
//! **Crossing a lane: skipped, not a crossing glyph.** Where the path from
//! the column's left edge to the node passes a live lane, that cell keeps
//! its own `│` (or corner, or run) and the connector resumes on the other
//! side; the image renderer does the same by drawing the line *under* the
//! lanes. `┼` was rejected on purpose: it is [`TreePart::Crossing`], meaning
//! *a branch or merge run crosses a live lane*, so borrowing it would make a
//! ref look like history, overwrite the lane's own cell, and ink at full
//! lane weight instead of thinner than the lanes.
//!
//! The Ref column's half -- `─` padding after the pill -- is drawn by
//! `log_table::commit_row`; the image renderer's 1-px line by
//! [`super::raster::rasterize`] from [`super::RasterRow::ref_connector`].

use ratatui::text::{Line, Span};

use super::layout::GraphRow;
use super::part::{ref_connector_span, TreeCell, TreePart};

/// The cell holding a row's commit node, if the row has one.
pub(crate) fn node_column(cells: &[TreeCell]) -> Option<usize> {
    cells
        .iter()
        .position(|c| matches!(c.part, TreePart::Node { .. }))
}

/// The Graph cell of a highlighted row (`band` = selection or hover
/// color): the band starts at the commit's node and runs
/// to the column's right edge; the lane cells left of the node keep the
/// plain background. A span that already has a background (the
/// `row_tint` band) keeps it. `node == None` bands the whole cell. With no
/// `band` this is just the line.
pub(crate) fn banded_graph_cell(
    line: Line<'static>,
    node: Option<usize>,
    width: usize,
    band: Option<ratatui::style::Color>,
) -> ratatui::widgets::Cell<'static> {
    use ratatui::style::Style;
    let Some(bg) = band else {
        return ratatui::widgets::Cell::from(line);
    };
    let node = node.unwrap_or(0);
    let mut used = 0usize;
    let mut spans: Vec<Span<'static>> = Vec::with_capacity(line.spans.len() + 1);
    for (i, span) in line.spans.into_iter().enumerate() {
        used += span.width();
        spans.push(if i >= node && span.style.bg.is_none() {
            span.patch_style(Style::default().bg(bg))
        } else {
            span
        });
    }
    if used < width {
        spans.push(Span::styled(
            " ".repeat(width - used),
            Style::default().bg(bg),
        ));
    }
    ratatui::widgets::Cell::from(Line::from(spans))
}

/// `line` - one row's text Graph cell, one span per [`TreeCell`] as
/// `build_graph_lines_with` makes it - with every **blank** cell left of
/// the node replaced by the connector in the row's lane color.
///
/// Returned unchanged when the row has no node, or when `line` is not one
/// span per cell (it would then be something this function does not
/// understand, and a connector in the wrong cell is worse than none).
pub(crate) fn with_ref_connector(line: Line<'static>, row: &GraphRow) -> Line<'static> {
    let Some(node) = node_column(&row.cells) else {
        return line;
    };
    if line.spans.len() != row.cells.len() {
        return line;
    }
    let spans: Vec<Span<'static>> = line
        .spans
        .into_iter()
        .zip(&row.cells)
        .enumerate()
        .map(|(j, (span, cell))| {
            if j < node && cell.part == TreePart::Empty {
                ref_connector_span(row.color, 1)
            } else {
                span
            }
        })
        .collect();
    Line::from(spans).style(line.style)
}
