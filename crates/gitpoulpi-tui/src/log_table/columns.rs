//! The Log columns: identity, width allocation, offsets, header hits and boundary drags.

use super::*;

/// Identifies one of the Log table's six columns - one enum for every
/// concern that needs to name a column: width allocation, on-screen
/// display order (drag-to-reorder), show/hide (right-click header menu),
/// and which column a hover/click lands in (drives per-column tooltip
/// content and header/boundary hit-testing). A single shared identity
/// instead of a near-duplicate enum per concern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum LogColumn {
    Ref,
    Graph,
    Message,
    Author,
    Date,
    /// Files changed and lines added/removed (`3 files +12 -4`). Hidden by
    /// default, and filled lazily for the rows on screen only.
    Changes,
    Sha,
}

/// The widest a column can be fitted to its content (a pasted novel in a
/// commit summary must not make a column a thousand cells wide).
pub(crate) const MAX_COLUMN_CELLS: u16 = 300;

impl LogColumn {
    pub(crate) const ALL: [LogColumn; 7] = [
        LogColumn::Ref,
        LogColumn::Graph,
        LogColumn::Message,
        LogColumn::Author,
        LogColumn::Date,
        LogColumn::Changes,
        LogColumn::Sha,
    ];

    /// The stable name written to `config.toml` (`ui.log_columns_*`).
    pub(crate) fn config_name(self) -> &'static str {
        match self {
            LogColumn::Ref => "ref",
            LogColumn::Graph => "graph",
            LogColumn::Message => "message",
            LogColumn::Author => "author",
            LogColumn::Date => "date",
            LogColumn::Changes => "changes",
            LogColumn::Sha => "sha",
        }
    }

    /// Inverse of [`LogColumn::config_name`]; `None` for an unknown name.
    pub(crate) fn from_config_name(name: &str) -> Option<LogColumn> {
        LogColumn::ALL
            .iter()
            .copied()
            .find(|c| c.config_name() == name)
    }

    /// The header text in the current language (display only; the stable
    /// name is [`LogColumn::config_name`]).
    pub(crate) fn title(self) -> &'static str {
        use gitpoulpi_i18n::msg::log_table as m;
        match self {
            LogColumn::Ref => m::COL_REF,
            LogColumn::Graph => m::COL_GRAPH,
            LogColumn::Message => m::COL_MESSAGE,
            LogColumn::Author => m::COL_AUTHOR,
            LogColumn::Date => m::COL_DATE,
            LogColumn::Changes => m::COL_CHANGES,
            LogColumn::Sha => m::COL_SHA,
        }
        .get()
    }

    /// `Message` is the one column the show/hide menu must never offer: a
    /// Log pane without commit summaries shows nothing meaningful.
    ///
    /// Hiding is the only thing special-cased for `Message`: every column,
    /// `Message` included, resizes from its own right-hand boundary
    /// (`for_area_width` honours a `Message` override and
    /// `boundary_drag_widths` pins both sides of a drag).
    pub(crate) fn hideable(self) -> bool {
        !matches!(self, LogColumn::Message)
    }

    pub(crate) fn min_width(self) -> u16 {
        match self {
            LogColumn::Ref => 10,
            LogColumn::Graph => 4,
            LogColumn::Message => 8,
            LogColumn::Author => 4,
            LogColumn::Date => 10,
            LogColumn::Changes => 6,
            LogColumn::Sha => 4,
        }
    }

    /// The least a column can be **dragged** to. The Graph column goes down
    /// to one commit dot (`graph::COMPACT_W`), where the graph is compact;
    /// the automatic layout still keeps [`LogColumn::min_width`] for it.
    pub(crate) fn drag_min_width(self) -> u16 {
        match self {
            LogColumn::Graph => crate::graph::COMPACT_W as u16,
            other => other.min_width(),
        }
    }

    /// The order columns are given up in when the pane is too narrow for
    /// every visible column's `min_width` - first entry sacrificed first.
    /// `Message` is absent on purpose: it is the one column that has to
    /// survive any width, and everything here is spent buying it room.
    ///
    /// This list is a product decision, not an artefact of the allocation
    /// loop. The reasoning, cheapest sacrifice first:
    ///
    /// - **Changes** is an optional, off-by-default extra: the first to go.
    /// - **Ref** costs ten cells for pills the sidebar already lists in
    ///   full, and a pill is the first thing that stops being readable
    ///   when it is truncated.
    /// - **Date** costs ten more, and the log is already in date order:
    ///   a row's *position* carries most of what its timestamp says.
    /// - **Graph** is what this client is for, but with two or three live
    ///   lanes its column is mostly whitespace, and the topology is still
    ///   legible in the ordering.
    /// - **Author** costs four cells and is the cheapest way to tell two
    ///   similar commits apart.
    /// - **Sha** costs four cells and is the handle you need to *act* on a
    ///   commit from the CLI, so it is the last thing to go.
    ///
    /// A sacrificed column is not *hidden* - the show/hide menu still
    /// lists it as visible and it comes back the moment the pane is wide
    /// enough. It simply renders at zero width for this frame.
    pub(crate) const SACRIFICE_ORDER: [LogColumn; 6] = [
        LogColumn::Changes,
        LogColumn::Ref,
        LogColumn::Date,
        LogColumn::Graph,
        LogColumn::Author,
        LogColumn::Sha,
    ];

    /// How much of a narrow pane is reserved for `Message` before any
    /// other column grows past its own floor.
    ///
    /// Capped at 16 cells deliberately: `Message` naturally gets more than
    /// that from the leftover budget at every pane width the app is
    /// normally used at, so the reservation only ever binds when the pane
    /// is genuinely cramped. That is what keeps this fix from quietly
    /// re-proportioning the columns at widths where they were already
    /// right.
    fn message_reservation(inner: u16) -> u16 {
        ((inner as u32 * 45 / 100) as u16).clamp(LogColumn::Message.min_width(), 16)
    }
}

/// The Log table's six column widths, derived from the pane's *actual*
/// available width every frame rather than fixed constants - the ref-pill
/// column in particular needs real room (icon + optional checkmark +
/// name, not just a bare branch name), and every *visible* column must add
/// up to exactly `inner_width` so nothing is left to ratatui's own
/// no-ellipsis clipping. Percentage-based with floors, so it degrades
/// gracefully (still no overlap, just tighter truncation) at a narrow pane
/// width instead of only working at whatever width it was eyeballed
/// against. `hidden` columns get 0 width and don't consume budget;
/// `overrides` (from dragging a column boundary) replace a column's
/// percentage-derived desired width but are still clamped to its
/// `min_width` and to whatever budget remains.
#[derive(Debug, Default)]
pub(crate) struct LogColumnWidths {
    pub(crate) ref_w: u16,
    pub(crate) graph_w: u16,
    pub(crate) message_w: u16,
    pub(crate) author_w: u16,
    pub(crate) date_w: u16,
    pub(crate) changes_w: u16,
    pub(crate) sha_w: u16,
}

impl LogColumnWidths {
    pub(crate) fn for_area_width(
        area_width: u16,
        hidden: &std::collections::HashSet<LogColumn>,
        overrides: &std::collections::HashMap<LogColumn, u16>,
    ) -> Self {
        let inner = area_width.saturating_sub(2); // left+right border

        // Every column's ideal width, before any budget is considered.
        let desired = |col: LogColumn| -> u16 {
            match col {
                // Short SHAs are always exactly 7 hex chars, plus the one cell
                // the last column's resize grip sits on (`Edge::last`), so it
                // never covers the 7th digit.
                LogColumn::Sha => 8,
                // "YYYY-MM-DD HH:MM" is always 16 chars, plus the one-cell
                // gutter `commit_row` keeps at the right of every text
                // column. Asking for only the 16 it prints made the date
                // abut the SHA next to it, the columns having tiled the
                // pane exactly.
                LogColumn::Date => 17,
                // `12 files +345 -67` and its gutter cell.
                LogColumn::Changes => 19,
                // Pills truncate badly, so Ref gets real room before
                // author/graph/message.
                //
                // Ref, Graph and Author are fixed in terminal cells, never a
                // share of the pane: a window maximize/restore must not
                // move them, only `Message` absorbs the difference.
                LogColumn::Ref => 22,
                LogColumn::Author => 12,
                // A merge's fan-out connector needs *some* horizontal room
                // to be visible once there are more than 2-3 concurrent
                // lanes, but not so much it starves Message -- the merge
                // glyph itself (`GRAPH_NODE_MERGE` ╋, distinctly heavier
                // than the plain/ref-pointed knots) is what actually fixes
                // "on ne voit pas les merges"; this column just needs to
                // not immediately clip a 2-3-lane fan-out.
                LogColumn::Graph => 8,
                // Elastic: it takes the leftover at the end, so its
                // "desire" is only ever its reservation (see below).
                LogColumn::Message => LogColumn::message_reservation(inner),
            }
        };
        let want = |col: LogColumn| -> u16 {
            overrides
                .get(&col)
                .copied()
                .unwrap_or_else(|| desired(col))
                .max(col.drag_min_width())
        };
        let floor = |col: LogColumn| -> u16 { column_floor(col, overrides) };

        // Who survives this pane width at all: `message_reservation` is
        // taken out of the budget first, then, when the floors no longer
        // fit, whole columns are given up in `SACRIFICE_ORDER` rather than
        // shaving everyone below its `min_width`. Both passes only move
        // width between columns, so the visible widths sum to exactly
        // `inner` -- which the Log table depends on, its
        // `column_spacing(0)` leaving ratatui no slack for a rounding error.
        let mut live: Vec<LogColumn> = LogColumn::SACRIFICE_ORDER
            .into_iter()
            .filter(|c| !hidden.contains(c))
            .collect();
        let floors = |live: &[LogColumn]| -> u16 { live.iter().map(|c| floor(*c)).sum() };

        // `Message` is never pinned: it is the one flexible column and
        // takes whatever the fixed columns leave. (A width stored for it
        // by an older version is dropped at load, `LogColumnPrefs`.)
        let message_floor = LogColumn::message_reservation(inner);
        while !live.is_empty() && floors(&live) + message_floor > inner {
            live.remove(0);
        }

        // Everyone alive starts at its floor, then grows.
        let mut out = Self::default();
        for &col in &live {
            *out.width_mut(col) = floor(col);
        }
        // A pane too narrow even for `Message` alone: it takes what there
        // is, which is still an exact tiling.
        *out.width_mut(LogColumn::Message) = message_floor.min(inner);
        let mut spare = inner
            .saturating_sub(message_floor)
            .saturating_sub(floors(&live));

        // Growth order: pinned by the width tests at the sizes the app is
        // used at. A column with a width of its own (dragged, or fitted to
        // its content) is served first, so what it asks for is what it gets
        // up to `fit_max`: the others fall back to their floors, never the
        // reverse.
        let growth = [
            LogColumn::Sha,
            LogColumn::Date,
            LogColumn::Changes,
            LogColumn::Ref,
            LogColumn::Graph,
            LogColumn::Author,
        ];
        let pinned_first = growth
            .into_iter()
            .filter(|c| overrides.contains_key(c))
            .chain(growth.into_iter().filter(|c| !overrides.contains_key(c)));
        for col in pinned_first {
            if spare == 0 {
                break;
            }
            if !live.contains(&col) {
                continue;
            }
            let extra = want(col).saturating_sub(floor(col)).min(spare);
            *out.width_mut(col) += extra;
            spare -= extra;
        }

        // `Message` is the flexible column: it absorbs the whole remainder
        // (and, when the pane is too narrow, is the first to give), so every
        // visible width sums to exactly `inner`. The user's widths for the
        // other columns are only ever read here, never rewritten by a
        // resize: a window that grows back gets them back exactly.
        *out.width_mut(LogColumn::Message) += spare;

        out
    }

    /// The most `col` can be made in the fit layout: the pane's inner width
    /// less the room reserved for `Message` and the floor of every other
    /// column still on screen. Past it the column would take `Message`'s
    /// reservation or push another column off the pane. `widths` is what the
    /// pane shows now (a sacrificed column has none and costs nothing).
    pub(crate) fn fit_max(
        &self,
        area_width: u16,
        order: &[LogColumn],
        overrides: &std::collections::HashMap<LogColumn, u16>,
        col: LogColumn,
    ) -> u16 {
        let inner = area_width.saturating_sub(2);
        let others: u16 = order
            .iter()
            .filter(|c| **c != col && **c != LogColumn::Message && self.width_of(**c) > 0)
            .map(|c| column_floor(*c, overrides))
            .sum();
        inner
            .saturating_sub(LogColumn::message_reservation(inner))
            .saturating_sub(others)
            .max(col.drag_min_width())
    }

    fn width_mut(&mut self, col: LogColumn) -> &mut u16 {
        match col {
            LogColumn::Ref => &mut self.ref_w,
            LogColumn::Graph => &mut self.graph_w,
            LogColumn::Message => &mut self.message_w,
            LogColumn::Author => &mut self.author_w,
            LogColumn::Date => &mut self.date_w,
            LogColumn::Changes => &mut self.changes_w,
            LogColumn::Sha => &mut self.sha_w,
        }
    }

    pub(crate) fn width_of(&self, col: LogColumn) -> u16 {
        match col {
            LogColumn::Ref => self.ref_w,
            LogColumn::Graph => self.graph_w,
            LogColumn::Message => self.message_w,
            LogColumn::Author => self.author_w,
            LogColumn::Date => self.date_w,
            LogColumn::Changes => self.changes_w,
            LogColumn::Sha => self.sha_w,
        }
    }
}

/// A column's floor: `min_width`, or the width the user set when that is
/// smaller (a width below the automatic floor is its own floor).
fn column_floor(col: LogColumn, overrides: &std::collections::HashMap<LogColumn, u16>) -> u16 {
    overrides.get(&col).map_or(col.min_width(), |o| {
        (*o).clamp(col.drag_min_width(), col.min_width())
    })
}

/// Left-to-right screen-column offsets for each currently *visible* column
/// in display `order` - the one place that computes "where is column X on
/// screen", reused by header-cell/boundary hit-testing, the Ref-pill hit
/// rect, and per-column hover-tooltip detection, instead of four separate
/// cumulative-sum implementations that could drift out of sync with each
/// other.
pub(crate) fn column_x_offsets(
    area_x: u16,
    visible_order: &[LogColumn],
    widths: &LogColumnWidths,
) -> Vec<(LogColumn, u16, u16)> {
    let mut x = area_x + 1; // past the left border
    visible_order
        .iter()
        .map(|&col| {
            let w = widths.width_of(col);
            let entry = (col, x, w);
            x += w;
            entry
        })
        .collect()
}

/// The Log table widget, built in one place so the rendered columns and
/// the hit targets registered from `column_x_offsets` can never disagree
/// about where a column edge is.
///
/// `column_spacing(0)` is the load-bearing part. ratatui's `Table` defaults
/// to a spacing of **1**, and `LogColumnWidths` already hands it widths
/// that sum to exactly the pane's inner width - so the default asked the
/// layout solver for `inner + 5` cells of a pane `inner` cells wide, and it
/// answered by quietly shrinking the `Length` constraints. Every rendered
/// edge then sat one or two cells away from the edge we had computed, in
/// either direction (`Graph` +1, `Message` +2, `Author` -2, `Date` -1 at an
/// 80-column pane): the resize grab zones drifted into the middle of
/// columns, and the widest one landed nowhere near a visible boundary at
/// all. It also cost the `Date` column the cell its 16-character timestamp
/// needs, so the date was ellipsised at full width.
pub(crate) fn log_table<'a>(
    rows: Vec<Row<'a>>,
    header_cells: Vec<Cell<'a>>,
    visible_order: &[LogColumn],
    widths: &LogColumnWidths,
    border_style: Style,
) -> Table<'a> {
    Table::new(
        rows,
        visible_order
            .iter()
            .map(|&c| Constraint::Length(widths.width_of(c)))
            .collect::<Vec<_>>(),
    )
    .column_spacing(0)
    .header(
        Row::new(header_cells).style(
            Style::default()
                .fg(th().ui.muted)
                .add_modifier(Modifier::BOLD),
        ),
    )
    .block(
        Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(border_style),
    )
    // No `.bg()` here, deliberately -- background for a selected row is
    // applied per-cell in `commit_row`/the WIP row, skipping the Ref column
    // on purpose so its pill keeps its own lane color instead of the
    // table's highlight flattening it to `SELECTION_BG`, matching
    // other clients. `BOLD` alone is still a real, if subtler, selection cue
    // for the row as a whole.
    .row_highlight_style(Style::default().add_modifier(Modifier::BOLD))
}

/// Which column (if any) `col` falls inside, from a `column_x_offsets`
/// result - drives per-column tooltip content (full message vs. full SHA
/// vs. full author vs. full date vs. the ref-reveal list) instead of one
/// universal full-message tooltip regardless of which cell is hovered.
pub(crate) fn log_hover_column(offsets: &[(LogColumn, u16, u16)], col: u16) -> Option<LogColumn> {
    offsets
        .iter()
        .find(|(_, x, w)| col >= *x && col < *x + *w)
        .map(|(c, _, _)| *c)
}

/// The Log header row's hit targets, in registration order: every column
/// boundary first, then every header cell.
///
/// That order is load-bearing, the same way the status pane's row buttons
/// are: `find_hit_target` returns the first registered rect containing the
/// point, and a boundary strip straddles the edge between two header cells.
/// Registered cell-first, the half of the strip lying inside the left-hand
/// cell was swallowed by that cell's drag-to-reorder target, leaving a grab
/// zone one cell narrower than it looks and off-center from the edge it
/// advertises.
///
/// Both kinds of target live on the header row alone (`log_area.y` is the
/// pane's top border, `+ 1` the header), like a spreadsheet's: a boundary
/// strip down the whole height would turn a plain click on a commit near a
/// column edge into a resize drag.
///
/// The boundary rect comes from `log_boundary_grab_rect`, the same builder
/// `resize_grab_at` uses, so the cell that lights up, the cell that shows
/// the resize pointer and the cell that starts a drag are one cell.
pub(crate) fn log_header_hit_targets(
    log_area: Rect,
    offsets: &[(LogColumn, u16, u16)],
) -> Vec<(Rect, HitTarget)> {
    let mut hits = Vec::with_capacity(offsets.len() * 2 + 1);
    for &(col, _, _) in offsets {
        // A boundary always resizes the column on its left -- the one whose edge the user is
        // actually pointing at -- against its right-hand neighbor; see
        // `boundary_drag_widths`. The last column's grip is its own last cell
        // (`log_grip_rect`), never the pane's border, which belongs to the
        // Log/Status pane-divider drag: grabbing "the pane divider" and
        // getting a column resize instead, with the whole Log pane appearing
        // to shift, was exactly that collision.
        if let Some(rect) = log_grip_rect(log_area, offsets, col) {
            hits.push((rect, HitTarget::LogColumnBoundary(col)));
        }
    }
    for &(col, x, w) in offsets {
        hits.push((
            Rect::new(x, log_area.y + 1, w, 1),
            HitTarget::LogColumnHeader(col),
        ));
    }
    hits
}

/// What a press on a column's grip drags: the column, where the pointer
/// went down and the column's width then. The last column's grip has no
/// neighbour to trade width with (`Message` absorbs what it frees), so it
/// moves by how far the pointer travelled from the press, not to wherever
/// the pointer is: its grip sits on the column's own last cell, and an
/// absolute position would jump the column by the cell offset on the first
/// move.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LogBoundaryDrag {
    pub(crate) col: LogColumn,
    pub(crate) press_x: u16,
    pub(crate) start_w: u16,
    /// The grip after the last visible column.
    pub(crate) last: bool,
}

/// The width the last column's grip gives it with the pointer at `pointer_x`:
/// its width at the press plus the pointer's travel, between the least a
/// column can be dragged to and `room` (`LogColumnWidths::fit_max`).
pub(crate) fn last_grip_width(drag: &LogBoundaryDrag, pointer_x: u16, room: u16) -> u16 {
    let least = drag.col.drag_min_width();
    let wanted = i32::from(drag.start_w) + i32::from(pointer_x) - i32::from(drag.press_x);
    u16::try_from(wanted.max(i32::from(least)))
        .unwrap_or(u16::MAX)
        .min(room.max(least))
}

/// The cells a press can grab to resize `col`: the gap after it for an
/// internal boundary, its own last cell for the last visible column (not
/// when that one is `Message`, which is what absorbs the change, or when it
/// is under two cells wide). `None` when `col` is not visible.
pub(crate) fn log_grip_rect(
    log_area: Rect,
    offsets: &[(LogColumn, u16, u16)],
    col: LogColumn,
) -> Option<Rect> {
    let i = offsets.iter().position(|(c, _, _)| *c == col)?;
    let (_, x, w) = offsets[i];
    let bottom = (log_area.y + log_area.height).saturating_sub(1);
    if i + 1 < offsets.len() {
        return Some(log_boundary_grab_rect(log_area, x + w));
    }
    (w >= 2 && col != LogColumn::Message).then(|| {
        crate::column_resize::grab_rect(
            crate::column_resize::mode(),
            crate::column_resize::Edge::last(x + w),
            log_area.y + 1,
            bottom,
        )
    })
}

/// Screen column of the draggable boundary at the right edge of `col`, from
/// a `column_x_offsets` result - `None` when `col` isn't visible, or is the
/// last visible column (whose right edge is the pane's own border and so
/// carries no drag target; see the registration site).
///
/// The resize highlight is drawn here rather than at the pointer's own
/// column: mid-drag the two part company by however far the pointer has
/// travelled past the edge's clamp.
#[cfg(test)]
pub(crate) fn log_boundary_x(offsets: &[(LogColumn, u16, u16)], col: LogColumn) -> Option<u16> {
    let i = offsets.iter().position(|(c, _, _)| *c == col)?;
    (i + 1 < offsets.len()).then(|| offsets[i].1 + offsets[i].2)
}

/// The resting ticks in the header's gap cells (`ui.column_resize = "line"`;
/// nothing in `header` mode): one per internal boundary, where the grab
/// strip of `log_boundary_grab_rect` starts.
pub(crate) fn paint_log_ticks(
    buf: &mut ratatui::buffer::Buffer,
    log_area: Rect,
    offsets: &[(LogColumn, u16, u16)],
) {
    for &(col, _, _) in offsets {
        // The grip's resting tick is its gap cell: the column's own last
        // cell in both layouts (the last column's grip included).
        if let Some(grip) = log_grip_rect(log_area, offsets, col) {
            let gap = match crate::column_resize::mode() {
                gitpoulpi_config::ColumnResize::Line => grip.x,
                gitpoulpi_config::ColumnResize::Header => grip.x.saturating_sub(1),
            };
            crate::column_resize::paint_tick(
                buf,
                crate::column_resize::mode(),
                gap,
                log_area.y + 1,
            );
        }
    }
}

/// The width overrides one step of a boundary drag writes: the edge at the
/// right of `left` moves to screen column `pointer`, so `left` takes the
/// width up to the pointer and its right-hand neighbor gives up - or
/// takes back - exactly as much, conserving the pair's combined width.
///
/// Conserving the pair is what makes the grabbed edge actually move.
/// Overriding `left` alone works only for the boundaries to the LEFT of
/// `Message`. `Message` is the elastic column
/// that absorbs whatever the others don't claim, so every column after it
/// has its right edge fixed by the columns that follow - `Author`'s, for
/// instance, is `inner - date_w - sha_w` no matter how wide `Author` is.
/// Widening `Author` there took the width out of `Message`, slid `Author`
/// bodily to the left, and left the edge under the pointer exactly where it
/// started: a boundary that cannot be dragged, while the columns on the
/// other side of the pane visibly jump.
///
/// Both widths are clamped so neither neighbor is crushed below its
/// `min_width`; the boundary simply stops there.
pub(crate) fn boundary_drag_widths(
    offsets: &[(LogColumn, u16, u16)],
    left: LogColumn,
    pointer: u16,
) -> Option<Vec<(LogColumn, u16)>> {
    let i = offsets.iter().position(|(c, _, _)| *c == left)?;
    let (_, x, w) = offsets[i];
    let (right, _, right_w) = *offsets.get(i + 1)?;
    let total = w + right_w;
    let max_left = total
        .saturating_sub(right.drag_min_width())
        .max(left.drag_min_width());
    let new_left = pointer
        .saturating_sub(x)
        .clamp(left.drag_min_width(), max_left);
    // `Message` is flexible and never stored: the side that is not
    // `Message` carries the width.
    Some(
        [(left, new_left), (right, total.saturating_sub(new_left))]
            .into_iter()
            .filter(|(c, _)| *c != LogColumn::Message)
            .collect(),
    )
}
