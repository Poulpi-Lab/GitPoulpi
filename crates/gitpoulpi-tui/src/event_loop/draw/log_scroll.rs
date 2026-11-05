//! The Log table in the `scroll` layout.
//!
//! The columns keep their own widths, so the table can be wider than the
//! pane. ratatui's `Table` cannot start part-way through a column, so the
//! table is drawn whole (only the columns that reach the viewport, each at
//! its full width) into a scratch buffer and the viewport's window of it is
//! copied over the pane. Everything the paint walk registers afterwards (hit
//! rects, the graph picture, avatars) comes from the clipped
//! [`LogColumnsGeo::offsets`], never from the scratch buffer.

use super::super::*;
use ratatui::buffer::{Buffer, Cell as BufCell};
use ratatui::widgets::StatefulWidget;

/// What `render` draws: the rows and header the paint walk built for the
/// columns in `order` (the ones that reach the viewport), their widths and the
/// border style.
pub(in crate::event_loop) struct ScrolledTable<'a> {
    pub(in crate::event_loop) rows: Vec<Row<'a>>,
    pub(in crate::event_loop) header: Vec<Cell<'a>>,
    pub(in crate::event_loop) order: &'a [LogColumn],
    pub(in crate::event_loop) widths: &'a LogColumnWidths,
    pub(in crate::event_loop) border_style: Style,
}

/// Draws the block, then the table through the viewport of `geo`.
pub(in crate::event_loop) fn render(
    frame: &mut Frame,
    log_area: Rect,
    geo: &LogColumnsGeo,
    table: ScrolledTable<'_>,
    state: &mut TableState,
) {
    frame.render_clipped(log_block(table.border_style), log_area);
    let inner = clip_to_frame(frame.area(), geo.inner);
    if inner.width == 0 || inner.height == 0 || table.order.is_empty() {
        return;
    }
    // Where the drawn columns start in the content, and how wide they are in all.
    let first_x = table
        .order
        .first()
        .and_then(|c| geo.span_of(*c))
        .map_or(0, |(x, _)| x);
    let total: u16 = table.order.iter().map(|c| table.widths.width_of(*c)).sum();
    let scratch_area = Rect::new(0, 0, total, geo.inner.height);
    let mut scratch = Buffer::empty(scratch_area);
    // The scratch buffer starts as a copy of what the pane already shows
    // there, so a cell the table leaves untouched keeps the theme's
    // background instead of the terminal default.
    let to_screen = |c: u16| -> Option<u16> {
        let content = i32::from(first_x) + i32::from(c);
        let sx = i32::from(geo.inner.x) + content - i32::from(geo.h_offset);
        (sx >= i32::from(inner.x) && sx < i32::from(inner.x + inner.width))
            .then(|| u16::try_from(sx).unwrap_or(0))
    };
    for ry in 0..inner.height {
        for c in 0..total {
            if let Some(sx) = to_screen(c) {
                scratch[(c, ry)] = frame.buffer_mut()[(sx, inner.y + ry)].clone();
            }
        }
    }
    StatefulWidget::render(
        log_table_body(table.rows, table.header, table.order, table.widths),
        scratch_area,
        &mut scratch,
        state,
    );
    let right_edge = inner.x + inner.width - 1;
    for ry in 0..inner.height {
        for c in 0..total {
            let Some(sx) = to_screen(c) else {
                continue;
            };
            let mut cell: BufCell = scratch[(c, ry)].clone();
            // A wide glyph on the last cell would spill over the border: it is
            // cut to a blank. (One cut at the left edge has already lost its
            // first half; what is left, its blank second half, is drawn as
            // the blank it is.)
            if sx == right_edge && display_width(cell.symbol()) > 1 {
                cell.set_symbol(" ");
            }
            frame.buffer_mut()[(sx, inner.y + ry)] = cell;
        }
    }
    paint_tail(frame.buffer_mut(), inner, first_x + total, geo.h_offset);
}

/// The selected or hovered row's band runs to the border: past the end of a
/// table narrower than the pane the cells keep the theme's blank, so the last
/// drawn cell's background is carried over them when it is a row band.
fn paint_tail(buf: &mut Buffer, inner: Rect, content_end: u16, h_offset: u16) {
    let Some(end) = content_end.checked_sub(h_offset) else {
        return;
    };
    if end == 0 || end >= inner.width {
        return;
    }
    let last_drawn = inner.x + end - 1;
    for y in inner.y + 1..inner.y + inner.height {
        let bg = buf[(last_drawn, y)].bg;
        if bg == th().ui.selection || bg == th().ui.hover {
            for x in last_drawn + 1..inner.x + inner.width {
                buf[(x, y)].set_bg(bg);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn render_rows(
        width: u16,
        h_offset: u16,
        message: &str,
        selected: Option<usize>,
    ) -> (Buffer, LogColumnsGeo) {
        let scroll: std::collections::HashMap<LogColumn, u16> = [
            (LogColumn::Message, 30),
            (LogColumn::Author, 12),
            (LogColumn::Sha, 8),
        ]
        .into();
        let hidden: std::collections::HashSet<LogColumn> = [
            LogColumn::Ref,
            LogColumn::Graph,
            LogColumn::Date,
            LogColumn::Changes,
        ]
        .into();
        let log_area = Rect::new(0, 0, width, 6);
        let geo = LogColumnsGeo::compute(
            log_area,
            &LogColumn::ALL,
            &hidden,
            &LogWidthMaps {
                fit: &Default::default(),
                scroll: &scroll,
            },
            (gitpoulpi_config::LogColumnsLayout::Scroll, h_offset),
        );
        let order: Vec<LogColumn> = geo.offsets.iter().map(|(c, _, _)| *c).collect();
        let rows: Vec<Row> = (0..3)
            .map(|i| {
                let row = Row::new(
                    order
                        .iter()
                        .map(|c| match c {
                            LogColumn::Message => Cell::from(message.to_string()),
                            LogColumn::Author => Cell::from("Ada Lovelace"),
                            _ => Cell::from("abc1234"),
                        })
                        .collect::<Vec<_>>(),
                );
                // The selected row carries the band, as `commit_row` styles it.
                if selected == Some(i) {
                    row.style(Style::default().bg(th().ui.selection))
                } else {
                    row
                }
            })
            .collect();
        let header: Vec<Cell> = order.iter().map(|c| Cell::from(c.title())).collect();
        let mut terminal = Terminal::new(TestBackend::new(width, 6)).unwrap();
        terminal
            .draw(|frame| {
                // A themed background the scratch buffer must not lose.
                let area = frame.area();
                frame
                    .buffer_mut()
                    .set_style(area, Style::default().bg(th().ui.panel));
                let mut state = TableState::default();
                *state.selected_mut() = selected;
                render(
                    frame,
                    log_area,
                    &geo,
                    ScrolledTable {
                        rows,
                        header,
                        order: &order,
                        widths: &geo.widths,
                        border_style: Style::default(),
                    },
                    &mut state,
                );
            })
            .unwrap();
        (terminal.backend().buffer().clone(), geo)
    }

    fn row_text(buf: &Buffer, y: u16) -> String {
        (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect()
    }

    /// The header titles sit at the clipped offsets the geometry registers, at
    /// every scroll offset (red when the blit or the hit rects ignore the
    /// offset).
    #[test]
    fn header_titles_sit_where_the_geometry_says() {
        for h in [0u16, 3, 9, 20] {
            let (buf, geo) = render_rows(34, h, "a commit summary", None);
            for &(col, x, w) in &geo.offsets {
                let (cx, _) = geo.span_of(col).unwrap();
                // The title starts at the column's start: on screen only when
                // that start is not cut off at the left.
                if cx >= geo.h_offset {
                    let title = col.title();
                    let got: String = (x..x + w.min(title.len() as u16))
                        .map(|i| buf[(i, 1)].symbol())
                        .collect();
                    assert_eq!(got, title[..got.len()], "{col:?} at h={h}");
                }
            }
        }
    }

    /// The block stays intact and a wide glyph on the last inner cell is cut
    /// to a blank instead of spilling over the border; a wide glyph cut at the
    /// left edge leaves no half glyph.
    #[test]
    fn wide_glyphs_are_cut_cleanly_at_both_edges_and_the_border_stays() {
        for h in 0..14u16 {
            let (buf, _) = render_rows(
                24,
                h,
                "\u{5C71}\u{7530}\u{592A}\u{90CE} \u{1F419} done",
                None,
            );
            for y in 0..6 {
                let line = row_text(&buf, y);
                let cells: Vec<&str> = (0..24).map(|x| buf[(x, y)].symbol()).collect();
                assert!(
                    matches!(cells[0], "\u{2502}" | "\u{256D}" | "\u{2570}"),
                    "left border at h={h} y={y}: {line:?}"
                );
                assert!(
                    matches!(cells[23], "\u{2502}" | "\u{256E}" | "\u{256F}"),
                    "right border at h={h} y={y}: {line:?}"
                );
                // No wide glyph starts on the last inner cell.
                assert!(display_width(cells[22]) <= 1, "h={h} y={y}: {line:?}");
            }
        }
    }

    /// A table narrower than the pane leaves blank space at the right, and a
    /// selected row's band runs through it to the border.
    #[test]
    fn a_narrow_table_leaves_blank_space_and_the_selection_band_reaches_the_border() {
        // 12 + 8 + the minimum widths of the three columns: well under 80.
        let (buf, geo) = render_rows(80, 0, "x", Some(0));
        let content = geo.content_w;
        assert!(content < geo.inner.width);
        let y = 2; // first body row
        let end = geo.inner.x + content;
        assert_eq!(
            buf[(end + 3, 1)].symbol(),
            " ",
            "blank space right of the header"
        );
        assert_eq!(
            buf[(end + 3, y)].bg,
            th().ui.selection,
            "the band runs to the border"
        );
        assert_eq!(
            buf[(geo.inner.x + geo.inner.width - 1, y)].bg,
            th().ui.selection
        );
        assert_eq!(
            buf[(end + 3, y + 1)].bg,
            th().ui.panel,
            "an unselected row keeps the theme's blank, not the terminal's"
        );
    }
}
