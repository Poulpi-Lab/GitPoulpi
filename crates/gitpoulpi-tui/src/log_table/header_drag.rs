//! Dragging a Log column header to another place: the drop slot under the
//! pointer, the marker where the column would land, and the drop itself.
//! The columns reorder on release only; while the button is down the dragged
//! header's ghost follows the pointer (`column_drag::paint_ghost`) and an
//! insertion bar marks the slot (`column_drag::paint_marker`).

use super::*;

/// A header drag in flight.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LogHeaderDrag {
    pub(crate) col: LogColumn,
    /// Where the button went down; a press that never leaves it is no drag.
    pub(crate) origin: (u16, u16),
    /// Where the pointer is now.
    pub(crate) at: (u16, u16),
    pub(crate) started: bool,
    /// The insertion slot among the visible columns (`0..=n`), once started.
    pub(crate) slot: Option<usize>,
    /// The dragged column's header cells, dimmed while it travels.
    pub(crate) source: Option<ratatui::layout::Rect>,
    /// The screen cell of the insertion bar.
    pub(crate) marker: Option<(u16, u16)>,
}

impl LogHeaderDrag {
    pub(crate) fn armed(col: LogColumn, at: (u16, u16)) -> Self {
        Self {
            col,
            origin: at,
            at,
            started: false,
            slot: None,
            source: None,
            marker: None,
        }
    }
}

/// The insertion slot for a pointer at screen column `x`: before the visible
/// column whose left half holds it, else after it.
pub(crate) fn drop_slot(offsets: &[(LogColumn, u16, u16)], x: u16) -> usize {
    for (i, (_, cx, w)) in offsets.iter().enumerate() {
        if x < cx + (w / 2).max(1) {
            return i;
        }
    }
    offsets.len()
}

/// The screen column of the insertion bar for `slot`: the separator cell that
/// ends the column before it (so no header letter is covered), else the last
/// cell of the last column.
pub(crate) fn marker_x(offsets: &[(LogColumn, u16, u16)], slot: usize) -> Option<u16> {
    match offsets.get(slot) {
        Some((_, x, _)) => Some(x.saturating_sub(1)),
        None => offsets.last().map(|(_, x, w)| (x + w).saturating_sub(1)),
    }
}

/// Moves `dragged` to the visible slot `slot` of the full `order` (hidden
/// columns keep their places). `false` when that is where it already is.
pub(crate) fn drop_column(
    order: &mut Vec<LogColumn>,
    hidden: &std::collections::HashSet<LogColumn>,
    dragged: LogColumn,
    slot: usize,
) -> bool {
    let visible = visible_in_order(order, hidden);
    let Some(from) = visible.iter().position(|c| *c == dragged) else {
        return false;
    };
    if slot == from || slot == from + 1 {
        return false;
    }
    let before = visible.get(slot).copied();
    let Some(pos) = order.iter().position(|c| *c == dragged) else {
        return false;
    };
    order.remove(pos);
    let at = match before {
        Some(b) => order.iter().position(|c| *c == b),
        None => visible
            .iter()
            .rev()
            .find(|c| **c != dragged)
            .and_then(|last| order.iter().position(|c| c == last))
            .map(|i| i + 1),
    }
    .unwrap_or(order.len());
    order.insert(at, dragged);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn order() -> Vec<LogColumn> {
        LogColumn::ALL.to_vec()
    }

    #[test]
    fn dropping_before_a_column_lands_just_before_it() {
        let mut o = order();
        let none = HashSet::new();
        let last = *o.last().unwrap();
        assert!(drop_column(&mut o, &none, last, 0));
        assert_eq!(o[0], last);
    }

    #[test]
    fn dropping_past_the_end_lands_last_and_the_own_place_is_a_no_op() {
        let mut o = order();
        let none = HashSet::new();
        let first = o[0];
        assert!(!drop_column(&mut o, &none, first, 0));
        assert!(!drop_column(&mut o, &none, first, 1));
        let n = o.len();
        assert!(drop_column(&mut o, &none, first, n));
        assert_eq!(*o.last().unwrap(), first);
    }
}
