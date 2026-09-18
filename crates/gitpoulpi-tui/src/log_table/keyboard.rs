//! The Log columns by keyboard: pick one, move it, size it. The pure parts
//! (the commit column's `Tab`, `<` / `>`, `[` / `]`, `0`, for the Log's own
//! header); the event loop wraps them in `commands/log_columns.rs`.

use super::*;
use crate::commit_column::Side;

/// What the Log column keys and palette commands do to the picked column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum LogColumnCmd {
    /// Starts picking: the first visible column, or the one already picked.
    Pick,
    NextColumn,
    PrevColumn,
    /// Moves the picked column one place among the visible ones (the twin of
    /// dragging its header).
    Move(Side),
    /// Widens or narrows the picked column (the twin of dragging its
    /// boundary).
    Wider,
    Narrower,
    /// Every column back to its automatic width (the commit column's `0`).
    ResetWidth,
    /// Opens the columns chooser (the header's right-click menu).
    Chooser,
}

/// The first column the chooser can toggle (the cursor's start): an index
/// into `LogColumn::ALL`.
pub(crate) fn first_hideable() -> usize {
    LogColumn::ALL
        .iter()
        .position(|c| c.hideable())
        .unwrap_or(0)
}

/// The chooser's cursor one step down or up, skipping the always-shown
/// column and wrapping.
pub(crate) fn step_chooser(cursor: usize, down: bool) -> usize {
    let n = LogColumn::ALL.len();
    (1..=n)
        .map(|k| {
            if down {
                (cursor + k) % n
            } else {
                (cursor + n * k - k) % n
            }
        })
        .find(|i| LogColumn::ALL[*i].hideable())
        .unwrap_or(cursor)
}

/// What one `[` / `]` changes a width by, in cells.
pub(crate) const KEY_STEP: u16 = 2;

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
