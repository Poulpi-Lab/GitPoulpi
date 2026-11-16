//! A right-aligned row of dialog buttons that never drops one.
//!
//! When the dialog is narrow the row degrades: first every label is clipped
//! (with an ellipsis) so all the buttons still fit on the row; when even that
//! leaves less than [`MIN_LABEL`] columns per label and the dialog has spare
//! rows above the row, the buttons stack one per row (the first button on the
//! top row). A button is never left out: its hit rect always exists, so the
//! mouse keeps working next to the key and the palette.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::{button, ButtonState};
use crate::{display_width, fit};

/// The narrowest a clipped label gets (so "Cancel" reads "Ca…").
const MIN_LABEL: usize = 3;
/// Columns between two buttons on a row.
const GAP: u16 = 1;

/// One button of a row.
pub(crate) struct Btn<'a> {
    pub(crate) label: &'a str,
    pub(crate) state: ButtonState,
}

/// Where one button goes: `dx` columns from the row's left edge, `up` rows
/// above the bottom row, `cells` wide (padding included).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Slot {
    pub(crate) dx: u16,
    pub(crate) up: u16,
    pub(crate) cells: u16,
    /// The label is clipped to this many columns (`None`: whole).
    pub(crate) label_cols: Option<usize>,
}

/// The pure layout: `widths` are the labels' display widths, `room` the
/// columns the row has, `spare_rows` the free rows above it.
pub(crate) fn layout(widths: &[usize], room: u16, spare_rows: u16) -> Vec<Slot> {
    let n = widths.len();
    if n == 0 {
        return Vec::new();
    }
    let gaps = GAP * (n as u16 - 1);
    let full: u16 = widths.iter().map(|w| *w as u16 + 2).sum::<u16>() + gaps;
    if full <= room {
        let mut x = room - full;
        return widths
            .iter()
            .map(|w| {
                let cells = *w as u16 + 2;
                let slot = Slot {
                    dx: x,
                    up: 0,
                    cells,
                    label_cols: None,
                };
                x += cells + GAP;
                slot
            })
            .collect();
    }
    // Clip every label to an even share of what is left.
    let share = (room.saturating_sub(gaps + 2 * n as u16) as usize) / n;
    let stack_rows = n as u16 - 1;
    if share < MIN_LABEL && spare_rows >= stack_rows {
        // One button per row, each clipped to the whole row.
        return widths
            .iter()
            .enumerate()
            .map(|(i, w)| {
                let cells = (*w as u16 + 2).min(room);
                Slot {
                    dx: room - cells,
                    up: stack_rows - i as u16,
                    cells,
                    label_cols: (cells < *w as u16 + 2).then(|| (cells as usize).saturating_sub(2)),
                }
            })
            .collect();
    }
    let share = share.max(MIN_LABEL);
    let mut slots: Vec<Slot> = widths
        .iter()
        .map(|w| {
            let cols = (*w).min(share);
            Slot {
                dx: 0,
                up: 0,
                cells: cols as u16 + 2,
                label_cols: (cols < *w).then_some(cols),
            }
        })
        .collect();
    let total: u16 = slots.iter().map(|s| s.cells).sum::<u16>() + gaps;
    let mut x = room.saturating_sub(total);
    for slot in &mut slots {
        slot.dx = x;
        x += slot.cells + GAP;
    }
    slots
}

/// Paints the row: its right edge is `right`, its bottom row `y`, and it may
/// use `room` columns going left and `spare_rows` rows above `y`. Returns each
/// button's rect, in the order given.
pub(crate) fn paint(
    buf: &mut Buffer,
    right: u16,
    y: u16,
    room: u16,
    spare_rows: u16,
    items: &[Btn],
) -> Vec<Rect> {
    let widths: Vec<usize> = items.iter().map(|b| display_width(b.label)).collect();
    let left = right.saturating_sub(room);
    layout(&widths, room, spare_rows)
        .into_iter()
        .zip(items)
        .map(|(slot, item)| {
            let label = match slot.label_cols {
                Some(cols) => fit(item.label, cols),
                None => item.label.to_string(),
            };
            button(
                buf,
                left + slot.dx,
                y.saturating_sub(slot.up),
                &label,
                item.state,
            )
        })
        .collect()
}

/// Paints buttons left to right from `x` on one row of `room` columns; when
/// they do not fit, every label is clipped to an even share (never dropped).
#[allow(dead_code)] // used by the feature-gated views (review, items, board)
pub(crate) fn paint_left(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    room: u16,
    gap: u16,
    items: &[Btn],
) -> Vec<Rect> {
    let n = items.len() as u16;
    if n == 0 {
        return Vec::new();
    }
    let widths: Vec<usize> = items.iter().map(|b| display_width(b.label)).collect();
    let full: u16 = widths.iter().map(|w| *w as u16 + 2).sum::<u16>() + gap * (n - 1);
    let share = if full <= room {
        usize::MAX
    } else {
        ((room.saturating_sub(gap * (n - 1) + 2 * n) / n) as usize).max(MIN_LABEL)
    };
    let mut at = x;
    items
        .iter()
        .map(|item| {
            let label = fit(item.label, share);
            let rect = button(buf, at, y, &label, item.state);
            at = rect.right() + gap;
            rect
        })
        .collect()
}

/// A button right-aligned at `right` that is clipped, never dropped, when the
/// row is short: it keeps at most `avail - reserve` cells (and never fewer than
/// five), so the text it shares the row with keeps `reserve` columns.
#[allow(dead_code)] // used by the feature-gated views (review, items, board)
pub(crate) fn right_clipped(
    buf: &mut Buffer,
    right: u16,
    y: u16,
    avail: u16,
    reserve: u16,
    label: &str,
    state: ButtonState,
) -> Rect {
    let cells = (display_width(label) as u16 + 2)
        .min(avail.saturating_sub(reserve).max(MIN_LABEL as u16 + 2))
        .min(right.max(1));
    let label = fit(label, (cells as usize).saturating_sub(2));
    button(buf, right - cells.min(right), y, &label, state)
}

/// The pure layout of a left-to-right row of buttons that wraps: `cells` are
/// the buttons' widths (padding included), `gap` the columns between two on a
/// row, `room` the columns a row has. Returns `(dx, row, cells)` per button; a
/// button wider than the room is cut to it, never dropped.
#[allow(dead_code)] // used by the feature-gated views (review, items, board)
pub(crate) fn flow_layout(cells: &[u16], gap: u16, room: u16) -> Vec<(u16, u16, u16)> {
    let mut out = Vec::with_capacity(cells.len());
    let (mut x, mut row) = (0u16, 0u16);
    for &w in cells {
        let w = w.min(room.max(1));
        if x > 0 && x + w > room {
            x = 0;
            row += 1;
        }
        out.push((x, row, w));
        x += w + gap;
    }
    out
}

/// Paints a row of buttons from `x`, left to right, wrapping onto the rows
/// below `y` when they do not fit in `room` columns. Returns every button's
/// rect (in the order given) and how many rows the row took (at least 1).
#[allow(dead_code)] // used by the feature-gated views (review, items, board)
pub(crate) fn flow(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    room: u16,
    gap: u16,
    items: &[Btn],
) -> (Vec<Rect>, u16) {
    let cells: Vec<u16> = items
        .iter()
        .map(|b| display_width(b.label) as u16 + 2)
        .collect();
    let slots = flow_layout(&cells, gap, room);
    let rows = slots.iter().map(|s| s.1 + 1).max().unwrap_or(1);
    let rects = slots
        .into_iter()
        .zip(items)
        .map(|((dx, row, w), item)| {
            let label = fit(item.label, (w as usize).saturating_sub(2));
            button(buf, x + dx, y + row, &label, item.state)
        })
        .collect();
    (rects, rows)
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_flowing_row_wraps_instead_of_dropping() {
        use super::flow_layout;
        let slots = flow_layout(&[10, 10, 10], 1, 25);
        assert_eq!(slots, vec![(0, 0, 10), (11, 0, 10), (0, 1, 10)]);
        // A button wider than the room is cut to it.
        assert_eq!(flow_layout(&[30], 1, 12), vec![(0, 0, 12)]);
        // Roomy: one row.
        assert!(flow_layout(&[5, 5], 1, 40).iter().all(|s| s.1 == 0));
    }

    use super::*;

    #[test]
    fn a_row_that_fits_is_right_aligned_and_whole() {
        let slots = layout(&[6, 6], 40, 0);
        assert_eq!(slots.len(), 2);
        assert!(slots.iter().all(|s| s.up == 0 && s.label_cols.is_none()));
        assert_eq!(slots[1].dx + slots[1].cells, 40);
        assert_eq!(slots[0].dx + slots[0].cells + GAP, slots[1].dx);
    }

    #[test]
    fn a_narrow_row_clips_the_labels_but_keeps_every_button() {
        let slots = layout(&[6, 6], 14, 0);
        assert_eq!(slots.len(), 2);
        assert!(slots.iter().all(|s| s.label_cols.is_some()));
        let end = slots.last().map(|s| s.dx + s.cells).unwrap_or(0);
        assert!(end <= 14, "{slots:?}");
        assert!(slots[0].dx + slots[0].cells < slots[1].dx, "{slots:?}");
    }

    #[test]
    fn a_very_narrow_row_with_spare_rows_stacks_instead_of_dropping() {
        let slots = layout(&[6, 6, 6], 9, 2);
        assert_eq!(slots.len(), 3);
        let ups: Vec<u16> = slots.iter().map(|s| s.up).collect();
        assert_eq!(ups, vec![2, 1, 0]);
        assert!(slots.iter().all(|s| s.dx + s.cells <= 9), "{slots:?}");
    }

    #[test]
    fn a_very_narrow_row_without_spare_rows_still_keeps_all_buttons() {
        let slots = layout(&[6, 6], 9, 0);
        assert_eq!(slots.len(), 2);
    }
}
