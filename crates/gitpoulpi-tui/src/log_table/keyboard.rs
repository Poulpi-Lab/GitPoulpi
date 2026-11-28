//! The Log columns by keyboard: pick one, move it, size it. The pure parts
//! (the commit column's `Tab`, `<` / `>`, `[` / `]`, `0`, for the Log's own
//! header); the event loop wraps them in `commands/log_columns.rs`.

use super::*;
use crate::commit_column::Side;

pub(crate) use super::vocab::LogColumnCmd;

/// One row of the Log columns chooser (right-click the header, or the
/// palette): the column checkboxes, then the actions below a separator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChooserRow {
    /// A column's checkbox (`Message` is listed but never actionable).
    Column(LogColumn),
    /// The dim line between the checkboxes and the actions.
    Separator,
    /// The checkbox that switches to the `scroll` layout (columns keep their
    /// widths and the Log scrolls sideways).
    Layout,
    /// Fit this column to its content (only with a target column).
    FitColumn(LogColumn),
    FitAll,
    Reset,
}

impl ChooserRow {
    /// Whether the cursor can stop on it and a click does something.
    pub(crate) fn actionable(self, layout: gitpoulpi_config::LogColumnsLayout) -> bool {
        match self {
            ChooserRow::Column(c) => c.hideable(),
            ChooserRow::Separator => false,
            // In the `fit` layout `Message` is the flexible column: it has no
            // width of its own to fit, so the row is shown dimmed. In `scroll`
            // it has one like the others.
            ChooserRow::FitColumn(c) => {
                c != LogColumn::Message || layout == gitpoulpi_config::LogColumnsLayout::Scroll
            }
            ChooserRow::Layout | ChooserRow::FitAll | ChooserRow::Reset => true,
        }
    }

    /// Rows that run and close the chooser (the checkboxes keep it open).
    pub(crate) fn closes(self) -> bool {
        !matches!(self, ChooserRow::Column(_) | ChooserRow::Layout)
    }
}

/// The open chooser: its keyboard cursor (an index into [`chooser_rows`])
/// and the column the "fit" row acts on (the one right-clicked, or the one
/// the keyboard picked).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ColumnMenu {
    pub(crate) cursor: usize,
    pub(crate) target: Option<LogColumn>,
}

impl ColumnMenu {
    /// Opens on the first checkbox that can be toggled.
    pub(crate) fn new(target: Option<LogColumn>) -> Self {
        Self {
            cursor: first_hideable(),
            target,
        }
    }
}

/// The chooser's rows, top to bottom.
pub(crate) fn chooser_rows(target: Option<LogColumn>) -> Vec<ChooserRow> {
    let mut rows: Vec<ChooserRow> = LogColumn::ALL.into_iter().map(ChooserRow::Column).collect();
    rows.push(ChooserRow::Separator);
    rows.push(ChooserRow::Layout);
    if let Some(col) = target {
        rows.push(ChooserRow::FitColumn(col));
    }
    rows.push(ChooserRow::FitAll);
    rows.push(ChooserRow::Reset);
    rows
}

/// The first column the chooser can toggle (the cursor's start): an index
/// into [`chooser_rows`].
pub(crate) fn first_hideable() -> usize {
    LogColumn::ALL
        .iter()
        .position(|c| c.hideable())
        .unwrap_or(0)
}

/// The chooser's cursor one step down or up, skipping rows that cannot be
/// acted on and wrapping.
pub(crate) fn step_chooser(
    rows: &[ChooserRow],
    layout: gitpoulpi_config::LogColumnsLayout,
    cursor: usize,
    down: bool,
) -> usize {
    let n = rows.len();
    (1..=n)
        .map(|k| {
            if down {
                (cursor + k) % n
            } else {
                (cursor + n * k - k) % n
            }
        })
        .find(|i| rows[*i].actionable(layout))
        .unwrap_or(cursor)
}

/// What one `[` / `]` changes a width by, in cells.
pub(crate) const KEY_STEP: u16 = 2;

/// What one `Shift+Left` / `Shift+Right` (or sideways wheel notch) scrolls the
/// columns by, in cells.
pub(crate) const SCROLL_STEP: u16 = 4;

/// `col` made `delta` cells wider (narrower when negative) in the `scroll`
/// layout, where only that column changes: its width now, clamped to what a
/// column can be.
pub(crate) fn scroll_resized(geo: &LogColumnsGeo, col: LogColumn, delta: i32) -> u16 {
    let now = i32::from(geo.widths.width_of(col));
    let wanted = (now + delta).clamp(i32::from(col.drag_min_width()), i32::from(MAX_COLUMN_CELLS));
    u16::try_from(wanted).unwrap_or(col.drag_min_width())
}

/// The scroll offset `delta` cells from `now`, kept inside what the content
/// allows.
pub(crate) fn scrolled(geo: &LogColumnsGeo, delta: i32) -> u16 {
    let wanted = (i32::from(geo.h_offset) + delta).clamp(0, i32::from(geo.max_h_offset()));
    u16::try_from(wanted).unwrap_or(0)
}

/// The offset that brings `col` fully into view with the least movement (its
/// start when it is wider than the viewport); unchanged in the `fit` layout.
pub(crate) fn offset_to_show(geo: &LogColumnsGeo, col: LogColumn) -> u16 {
    let Some((x, w)) = geo.span_of(col) else {
        return geo.h_offset;
    };
    let right = geo.h_offset + geo.inner.width;
    let wanted = if x < geo.h_offset || w >= geo.inner.width {
        x
    } else if x + w > right {
        x + w - geo.inner.width
    } else {
        geo.h_offset
    };
    wanted.min(geo.max_h_offset())
}

/// Flips `ui.log_columns_layout` (the chooser's checkbox, the palette and `s`
/// while a column is picked), publishes it for the paint and writes it back to
/// `config.toml` on a short-lived thread (the input loop never waits on the
/// disk; the write shares the column lock with the width saves). Returns the
/// status-line message.
pub(crate) fn toggle_layout(config_path: &Option<std::path::PathBuf>) -> String {
    use gitpoulpi_config::LogColumnsLayout;
    let next = match crate::live_settings::log_columns_layout() {
        LogColumnsLayout::Fit => LogColumnsLayout::Scroll,
        LogColumnsLayout::Scroll => LogColumnsLayout::Fit,
    };
    crate::live_settings::install_log_columns_layout(next);
    if let Some(path) = config_path.clone() {
        std::thread::spawn(move || {
            let _ = gitpoulpi_config::set_log_columns_layout_in_file(&path, next);
        });
    }
    match next {
        LogColumnsLayout::Fit => gitpoulpi_i18n::msg::log_table::LAYOUT_NOW_FIT.get().into(),
        LogColumnsLayout::Scroll => gitpoulpi_i18n::msg::log_table::LAYOUT_NOW_SCROLL
            .get()
            .into(),
    }
}

/// The visible columns in display order.
pub(crate) fn visible_in_order(
    order: &[LogColumn],
    hidden: &std::collections::HashSet<LogColumn>,
) -> Vec<LogColumn> {
    order
        .iter()
        .copied()
        .filter(|c| !hidden.contains(c))
        .collect()
}

/// The visible column after (or before) `current`, wrapping; the first one
/// when nothing is picked yet or the picked one went away.
pub(crate) fn step_pick(
    visible: &[LogColumn],
    current: Option<LogColumn>,
    forward: bool,
) -> Option<LogColumn> {
    let n = visible.len();
    if n == 0 {
        return None;
    }
    let Some(i) = current.and_then(|c| visible.iter().position(|v| *v == c)) else {
        return Some(visible[0]);
    };
    Some(
        visible[if forward {
            (i + 1) % n
        } else {
            (i + n - 1) % n
        }],
    )
}

/// Swaps `col` with its visible neighbour on `side` in the full order (hidden
/// columns keep their places). `false` at the edge.
pub(crate) fn move_column(
    order: &mut [LogColumn],
    hidden: &std::collections::HashSet<LogColumn>,
    col: LogColumn,
    side: Side,
) -> bool {
    let visible = visible_in_order(order, hidden);
    let Some(i) = visible.iter().position(|c| *c == col) else {
        return false;
    };
    let other = match side {
        Side::Left => i.checked_sub(1),
        Side::Right => (i + 1 < visible.len()).then_some(i + 1),
    }
    .map(|j| visible[j]);
    let Some(other) = other else {
        return false;
    };
    let (Some(a), Some(b)) = (
        order.iter().position(|c| *c == col),
        order.iter().position(|c| *c == other),
    ) else {
        return false;
    };
    order.swap(a, b);
    true
}

/// The width overrides that make `col` `delta` cells wider (narrower when
/// negative): its right boundary moves, and the last visible column moves
/// its left boundary instead. Empty when nothing can change (a floor, a lone
/// column).
pub(crate) fn resize_widths(
    offsets: &[(LogColumn, u16, u16)],
    col: LogColumn,
    delta: i32,
) -> Vec<(LogColumn, u16)> {
    let Some(i) = offsets.iter().position(|(c, _, _)| *c == col) else {
        return Vec::new();
    };
    let shift = |x: u16, by: i32| u16::try_from((i32::from(x) + by).max(0)).unwrap_or(u16::MAX);
    let (_, x, w) = offsets[i];
    let pair = if i + 1 < offsets.len() {
        boundary_drag_widths(offsets, col, shift(x + w, delta))
    } else if i > 0 {
        // The last column has no boundary on its right: its left one moves
        // the other way.
        boundary_drag_widths(offsets, offsets[i - 1].0, shift(x, -delta))
    } else {
        None
    };
    pair.unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn offsets() -> Vec<(LogColumn, u16, u16)> {
        vec![
            (LogColumn::Ref, 1, 14),
            (LogColumn::Graph, 15, 8),
            (LogColumn::Message, 23, 30),
            (LogColumn::Author, 53, 10),
        ]
    }

    #[test]
    fn pick_wraps_and_starts_at_the_first() {
        let v = [LogColumn::Ref, LogColumn::Graph, LogColumn::Sha];
        assert_eq!(step_pick(&v, None, true), Some(LogColumn::Ref));
        assert_eq!(
            step_pick(&v, Some(LogColumn::Sha), true),
            Some(LogColumn::Ref)
        );
        assert_eq!(
            step_pick(&v, Some(LogColumn::Ref), false),
            Some(LogColumn::Sha)
        );
        assert_eq!(step_pick(&[], None, true), None);
    }

    #[test]
    fn move_swaps_with_the_visible_neighbour() {
        let mut order = LogColumn::ALL.to_vec();
        let hidden: std::collections::HashSet<_> = [LogColumn::Changes].into();
        assert!(move_column(
            &mut order,
            &hidden,
            LogColumn::Date,
            Side::Right
        ));
        let at = |c| order.iter().position(|x| *x == c).unwrap();
        assert!(at(LogColumn::Sha) < at(LogColumn::Date));
        assert!(!move_column(
            &mut order,
            &hidden,
            LogColumn::Date,
            Side::Right
        ));
        assert!(!move_column(
            &mut order,
            &hidden,
            LogColumn::Ref,
            Side::Left
        ));
    }

    #[test]
    fn resize_moves_the_right_boundary_and_the_last_column_its_left() {
        let w = resize_widths(&offsets(), LogColumn::Ref, 2);
        assert_eq!(w, vec![(LogColumn::Ref, 16), (LogColumn::Graph, 6)]);
        // Message is flexible, never stored: the other side carries it.
        let w = resize_widths(&offsets(), LogColumn::Author, 2);
        assert_eq!(w, vec![(LogColumn::Author, 12)]);
        // A floor: nothing below the least width.
        let w = resize_widths(&offsets(), LogColumn::Graph, -20);
        assert!(w
            .iter()
            .all(|(c, n)| *c != LogColumn::Graph || *n >= LogColumn::Graph.drag_min_width()));
    }
}
