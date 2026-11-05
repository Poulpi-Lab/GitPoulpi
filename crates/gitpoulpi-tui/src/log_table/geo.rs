//! The Log table's column geometry, computed once.
//!
//! Every site that needs "where is column X on screen" (paint, header hits,
//! hover, boundary drags, the keyboard resize, the ref popup anchor, the
//! sideways scroll) asks one [`LogColumnsGeo`] instead of rebuilding widths,
//! visible order and offsets by hand. Two layouts share it: `fit`, where the
//! columns tile the pane exactly, and `scroll`, where they keep their own
//! widths, the table can be wider than the pane and `h_offset` says how far
//! it is scrolled.

use super::*;
use gitpoulpi_config::LogColumnsLayout;

/// The two width maps the layouts keep apart (`ui.log_column_widths` for
/// `fit`, `ui.log_column_widths_scroll` for `scroll`).
#[derive(Debug, Clone, Copy)]
pub(crate) struct LogWidthMaps<'a> {
    pub(crate) fit: &'a std::collections::HashMap<LogColumn, u16>,
    pub(crate) scroll: &'a std::collections::HashMap<LogColumn, u16>,
}

/// The Log's columns for one pane rectangle.
#[derive(Debug, Default)]
pub(crate) struct LogColumnsGeo {
    pub(crate) layout: LogColumnsLayout,
    pub(crate) widths: LogColumnWidths,
    /// Visible columns, in display order.
    pub(crate) order: Vec<LogColumn>,
    /// `(column, x from the table's start, full width)` per visible column:
    /// where each sits in the content, whatever is on screen.
    pub(crate) spans: Vec<(LogColumn, u16, u16)>,
    /// `(column, screen x, width)` of what is on screen, clipped to the
    /// viewport; a column fully scrolled out is left out. In the `fit` layout
    /// this is every visible column (a sacrificed one at width 0).
    pub(crate) offsets: Vec<(LogColumn, u16, u16)>,
    /// The pane inside its border: the viewport the content scrolls in.
    pub(crate) inner: Rect,
    /// The table's full width (the `fit` layout: the inner width).
    pub(crate) content_w: u16,
    /// How far the content is scrolled, already clamped (0 in `fit`).
    pub(crate) h_offset: u16,
}

impl LogColumnsGeo {
    /// `all_order` is the full display order (hidden columns included), as
    /// stored; hidden ones are dropped here. `view` is the layout and the
    /// requested scroll offset, which is clamped to what the content allows.
    pub(crate) fn compute(
        log_area: Rect,
        all_order: &[LogColumn],
        hidden: &std::collections::HashSet<LogColumn>,
        maps: &LogWidthMaps<'_>,
        view: (LogColumnsLayout, u16),
    ) -> Self {
        let (layout, want_offset) = view;
        let order = visible_in_order(all_order, hidden);
        let inner = Rect::new(
            log_area.x + 1,
            log_area.y + 1,
            log_area.width.saturating_sub(2),
            log_area.height.saturating_sub(2),
        );
        let widths = match layout {
            LogColumnsLayout::Fit => {
                LogColumnWidths::for_area_width(log_area.width, hidden, maps.fit)
            }
            LogColumnsLayout::Scroll => {
                LogColumnWidths::for_scroll(log_area.width, hidden, maps.scroll, maps.fit)
            }
        };
        let mut x = 0u16;
        let spans: Vec<(LogColumn, u16, u16)> = order
            .iter()
            .map(|&col| {
                let w = widths.width_of(col);
                let span = (col, x, w);
                x = x.saturating_add(w);
                span
            })
            .collect();
        let content_w = x;
        let h_offset = match layout {
            LogColumnsLayout::Fit => 0,
            LogColumnsLayout::Scroll => want_offset.min(content_w.saturating_sub(inner.width)),
        };
        let offsets = match layout {
            // The columns tile the pane: exactly the hand-built chain this
            // type replaced, a sacrificed column included at width 0.
            LogColumnsLayout::Fit => column_x_offsets(log_area.x, &order, &widths),
            LogColumnsLayout::Scroll => spans
                .iter()
                .filter_map(|&(col, cx, w)| {
                    let from = cx.max(h_offset);
                    let to = (cx + w).min(h_offset + inner.width);
                    (to > from).then(|| (col, inner.x + (from - h_offset), to - from))
                })
                .collect(),
        };
        Self {
            layout,
            widths,
            order,
            spans,
            offsets,
            inner,
            content_w,
            h_offset,
        }
    }

    /// The `fit` geometry a test hand-built as `column_x_offsets` offsets (the
    /// spans are the offsets, relative to the pane's inner start).
    #[cfg(test)]
    pub(crate) fn from_fit_offsets(log_area: Rect, offsets: &[(LogColumn, u16, u16)]) -> Self {
        let inner = Rect::new(
            log_area.x + 1,
            log_area.y + 1,
            log_area.width.saturating_sub(2),
            log_area.height.saturating_sub(2),
        );
        Self {
            layout: LogColumnsLayout::Fit,
            order: offsets.iter().map(|(c, _, _)| *c).collect(),
            spans: offsets
                .iter()
                .map(|&(c, x, w)| (c, x.saturating_sub(inner.x), w))
                .collect(),
            offsets: offsets.to_vec(),
            inner,
            content_w: inner.width,
            h_offset: 0,
            ..Self::default()
        }
    }

    /// Screen column of the content cell `content_x`, if it is in the
    /// viewport.
    pub(crate) fn screen_x(&self, content_x: u16) -> Option<u16> {
        (content_x >= self.h_offset && content_x < self.h_offset + self.inner.width)
            .then(|| self.inner.x + (content_x - self.h_offset))
    }

    /// The content cell under screen column `screen_col` (the inverse of
    /// [`LogColumnsGeo::screen_x`] inside the viewport).
    pub(crate) fn content_x(&self, screen_col: u16) -> u16 {
        screen_col.saturating_sub(self.inner.x) + self.h_offset
    }

    /// The furthest the content can be scrolled.
    pub(crate) fn max_h_offset(&self) -> u16 {
        match self.layout {
            LogColumnsLayout::Fit => 0,
            LogColumnsLayout::Scroll => self.content_w.saturating_sub(self.inner.width),
        }
    }

    /// Whether the content is wider than the viewport (the bar's condition).
    pub(crate) fn scrolls(&self) -> bool {
        self.max_h_offset() > 0
    }

    /// The column's full span `(x from the table's start, width)`.
    pub(crate) fn span_of(&self, col: LogColumn) -> Option<(u16, u16)> {
        self.spans
            .iter()
            .find(|(c, _, _)| *c == col)
            .map(|(_, x, w)| (*x, *w))
    }

    /// Whether `col` is on screen in full (not cut at either edge).
    pub(crate) fn fully_visible(&self, col: LogColumn) -> bool {
        match (
            self.span_of(col),
            self.offsets.iter().find(|(c, _, _)| *c == col),
        ) {
            (Some((_, w)), Some((_, _, shown))) => w == *shown,
            _ => false,
        }
    }

    /// The cells a press can grab to resize `col`: the gap after it for an
    /// internal boundary, its own last cell for the last visible column, both
    /// only while that cell is in the viewport. Not for a last column under
    /// two cells wide, nor for `Message` in the `fit` layout (it absorbs the
    /// change).
    pub(crate) fn grip_rect(&self, log_area: Rect, col: LogColumn) -> Option<Rect> {
        let i = self.spans.iter().position(|(c, _, _)| *c == col)?;
        let (_, x, w) = self.spans[i];
        let last = i + 1 == self.spans.len();
        if last && (w < 2 || (col == LogColumn::Message && self.layout == LogColumnsLayout::Fit)) {
            return None;
        }
        let mode = crate::column_resize::mode();
        let end = x + w;
        // `line` grabs the gap (the column's own last cell); `header` grabs
        // the first cell of the next column, which a last column does not have.
        let cell = match (mode, last) {
            (gitpoulpi_config::ColumnResize::Header, false) => end,
            _ => end.saturating_sub(1),
        };
        let screen = self.screen_x(cell)?;
        let edge = if last {
            crate::column_resize::Edge::last(screen + 1)
        } else {
            crate::column_resize::Edge {
                gap_x: screen,
                next_x: screen,
            }
        };
        Some(crate::column_resize::grab_rect(
            mode,
            edge,
            log_area.y + 1,
            (log_area.y + log_area.height).saturating_sub(1),
        ))
    }
}

/// The width the `scroll` layout's drag gives `col` with the pointer at
/// screen column `pointer`: the pointer's place in the content less where the
/// column starts, plus the cells between the pointer and the edge it moves
/// (the pointer holds the gap in `line` mode, one cell before the edge).
/// Absolute, since the other columns do not move: only `col` changes.
pub(crate) fn scroll_drag_width(geo: &LogColumnsGeo, col: LogColumn, pointer: u16) -> u16 {
    let Some((start, _)) = geo.span_of(col) else {
        return col.drag_min_width();
    };
    let last = geo.order.last() == Some(&col);
    let to_edge = match (crate::column_resize::mode(), last) {
        (gitpoulpi_config::ColumnResize::Header, false) => 0,
        _ => 1,
    };
    let wanted = u32::from(geo.content_x(pointer)) + to_edge;
    u16::try_from(wanted.saturating_sub(u32::from(start)))
        .unwrap_or(u16::MAX)
        .clamp(col.drag_min_width(), MAX_COLUMN_CELLS)
}

#[cfg(test)]
mod tests {
    use super::*;

    type Map = std::collections::HashMap<LogColumn, u16>;

    fn geo(width: u16, hidden: &std::collections::HashSet<LogColumn>, fit: &Map) -> LogColumnsGeo {
        LogColumnsGeo::compute(
            Rect::new(3, 2, width, 20),
            &LogColumn::ALL,
            hidden,
            &LogWidthMaps {
                fit,
                scroll: &Map::new(),
            },
            (LogColumnsLayout::Fit, 0),
        )
    }

    /// Widths that make the content wider than a 60 column pane.
    fn wide() -> Map {
        Map::from([
            (LogColumn::Ref, 20),
            (LogColumn::Graph, 10),
            (LogColumn::Message, 50),
            (LogColumn::Author, 20),
            (LogColumn::Date, 20),
            (LogColumn::Sha, 10),
        ])
    }

    fn scroll_geo(width: u16, scroll: &Map, h: u16) -> LogColumnsGeo {
        let hidden: std::collections::HashSet<LogColumn> = [LogColumn::Changes].into();
        LogColumnsGeo::compute(
            Rect::new(3, 2, width, 20),
            &LogColumn::ALL,
            &hidden,
            &LogWidthMaps {
                fit: &Map::new(),
                scroll,
            },
            (LogColumnsLayout::Scroll, h),
        )
    }

    /// In the fit layout the geometry is the hand-rolled chain it replaced,
    /// at every width and with columns hidden.
    #[test]
    fn fit_geo_equals_the_hand_built_chain() {
        let overrides = Map::from([(LogColumn::Author, 15)]);
        for width in [20u16, 40, 80, 120, 200] {
            for hidden in [
                std::collections::HashSet::new(),
                [LogColumn::Changes].into(),
                [LogColumn::Ref, LogColumn::Date].into(),
            ] {
                let g = geo(width, &hidden, &overrides);
                let widths = LogColumnWidths::for_area_width(width, &hidden, &overrides);
                let visible = visible_in_order(&LogColumn::ALL, &hidden);
                assert_eq!(g.order, visible);
                assert_eq!(g.offsets, column_x_offsets(3, &visible, &widths));
                let sum: u16 = g.offsets.iter().map(|(_, _, w)| *w).sum();
                assert_eq!(sum, width.saturating_sub(2), "tiles the inner width");
                assert_eq!(g.h_offset, 0);
                assert_eq!(g.max_h_offset(), 0);
            }
        }
    }

    /// Nothing is given up in the scroll layout: at 40 columns every visible
    /// column keeps a width (the fit layout would sacrifice some), and the
    /// content is wider than the pane.
    #[test]
    fn scroll_sacrifices_nothing_at_forty_columns() {
        let g = scroll_geo(40, &Map::new(), 0);
        assert!(g
            .order
            .iter()
            .all(|c| g.widths.width_of(*c) >= c.min_width()));
        assert!(g.content_w > g.inner.width, "wider than the 38 cells");
        assert!(g.scrolls());
        // Fit at the same width gives some columns up.
        let fit = geo(40, &[LogColumn::Changes].into(), &Map::new());
        assert!(fit.order.iter().any(|c| fit.widths.width_of(*c) == 0));
    }

    /// A stored scroll width is honoured (clamped), `Message` has one of its
    /// own and keeps it when the pane grows.
    #[test]
    fn message_keeps_its_width_when_the_pane_grows() {
        let scroll = Map::from([(LogColumn::Message, 60), (LogColumn::Author, 400)]);
        let narrow = scroll_geo(80, &scroll, 0);
        let wide = scroll_geo(200, &scroll, 0);
        assert_eq!(narrow.widths.width_of(LogColumn::Message), 60);
        assert_eq!(wide.widths.width_of(LogColumn::Message), 60);
        assert_eq!(
            narrow.widths.width_of(LogColumn::Author),
            MAX_COLUMN_CELLS,
            "clamped"
        );
    }

    /// The offsets are the on-screen part of each column: a column cut at the
    /// left edge starts at the viewport and is narrower; one scrolled out is
    /// gone; the visible widths never exceed the viewport.
    #[test]
    fn clipped_offsets_cover_the_viewport_and_drop_columns_scrolled_out() {
        let scroll = Map::from([
            (LogColumn::Ref, 20),
            (LogColumn::Graph, 10),
            (LogColumn::Message, 50),
            (LogColumn::Author, 20),
            (LogColumn::Date, 20),
            (LogColumn::Sha, 10),
        ]);
        let g = scroll_geo(60, &scroll, 0);
        assert_eq!(g.offsets[0], (LogColumn::Ref, 4, 20));
        // Scrolled 25 cells: Ref is gone, Graph is cut to 5 cells at the edge.
        let g = scroll_geo(60, &scroll, 25);
        assert_eq!(g.h_offset, 25);
        assert!(g.offsets.iter().all(|(c, _, _)| *c != LogColumn::Ref));
        assert_eq!(g.offsets[0], (LogColumn::Graph, g.inner.x, 5));
        let shown: u16 = g.offsets.iter().map(|(_, _, w)| *w).sum();
        assert_eq!(shown, g.inner.width);
        // The offset is clamped to what the content allows.
        let g = scroll_geo(60, &scroll, 9999);
        assert_eq!(g.h_offset, g.max_h_offset());
        assert_eq!(
            g.offsets.last().map(|(_, x, w)| x + w),
            Some(g.inner.x + g.inner.width)
        );
    }

    /// `screen_x` and `content_x` are inverses inside the viewport, and
    /// `None` outside it.
    #[test]
    fn screen_x_and_content_x_round_trip() {
        let g = scroll_geo(60, &wide(), 7);
        for sx in g.inner.x..g.inner.x + g.inner.width {
            assert_eq!(g.screen_x(g.content_x(sx)), Some(sx));
        }
        assert_eq!(g.screen_x(6), None, "left of the viewport");
        assert_eq!(g.screen_x(7 + g.inner.width), None, "right of it");
    }

    /// A grip is only there while its cell is on screen.
    #[test]
    fn a_grip_exists_only_while_its_cell_is_in_the_viewport() {
        crate::live_settings::install_column_resize(gitpoulpi_config::ColumnResize::Line);
        let area = Rect::new(3, 2, 60, 20);
        let g = scroll_geo(60, &wide(), 0);
        let (first, ..) = g.spans[0];
        assert!(g.grip_rect(area, first).is_some());
        let far = scroll_geo(60, &wide(), 9999);
        assert!(far.grip_rect(area, first).is_none(), "scrolled out");
        let (last, ..) = *far.spans.last().unwrap();
        assert!(
            far.grip_rect(area, last).is_some(),
            "the last grip is in view"
        );
        crate::live_settings::install_column_resize(gitpoulpi_config::ColumnResize::Header);
    }

    /// With the table scrolled, a drag sets `col`'s width from the pointer's
    /// place in the content (red when `h_offset` is ignored).
    #[test]
    fn a_scroll_drag_width_counts_the_scroll_offset() {
        crate::live_settings::install_column_resize(gitpoulpi_config::ColumnResize::Line);
        let scroll = Map::from([(LogColumn::Graph, 10), (LogColumn::Message, 80)]);
        let g = scroll_geo(60, &scroll, 30);
        let (start, _) = g.span_of(LogColumn::Message).unwrap();
        // The pointer on screen column inner.x + 20 is content cell 50; the
        // pointer holds the gap, one cell before the edge it moves.
        let w = scroll_drag_width(&g, LogColumn::Message, g.inner.x + 20);
        assert_eq!(w, 30 + 20 + 1 - start);
        let at_start = scroll_geo(60, &scroll, 0);
        assert_eq!(
            scroll_drag_width(&at_start, LogColumn::Message, 0),
            LogColumn::Message.drag_min_width(),
            "never below the least width"
        );
        crate::live_settings::install_column_resize(gitpoulpi_config::ColumnResize::Header);
    }
}
