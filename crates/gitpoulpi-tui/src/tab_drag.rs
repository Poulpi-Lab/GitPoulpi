//! Reordering repository tabs by dragging: from the tab bar, and from the
//! rows of the tab picker (`Ctrl+N t`).
//!
//! The keyboard twins are the picker's `Alt+Up/Down` and the palette's
//! `Tab: Move Left/Right`; all three end in `tabs::reorder_tab`.
//!
//! Like `RefDrag`, a press only *arms* the gesture: it becomes a drag once the
//! pointer has left the cell it went down on, so a press-release without
//! movement stays the plain click it always was. The state carries everything
//! the paint needs (the dimmed source, the insertion marker or lit row, the
//! ghost), computed from the hit rects of the frame the pointer moved over, so
//! painting never re-derives geometry.

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;

use crate::repo_switcher::TabHit;
use crate::th;

/// Cells the pointer must travel before a press on a tab is a drag, so a
/// click that wobbles by one cell still selects.
pub(crate) const DRAG_THRESHOLD: u16 = 2;

/// Where a tab drag was picked up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DragFrom {
    Bar,
    Picker,
}

/// The place the tab would land if released now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DropAt {
    /// The tab's index once it has moved.
    pub(crate) to: usize,
    /// The insertion marker (bar) or the lit row (picker).
    pub(crate) mark: Rect,
    /// The marker of a vertical tab column is a bar across, not along.
    pub(crate) vertical: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TabDrag {
    pub(crate) from: usize,
    pub(crate) name: String,
    pub(crate) origin_kind: DragFrom,
    pub(crate) started: bool,
    pub(crate) origin: (u16, u16),
    pub(crate) at: (u16, u16),
    pub(crate) source_rect: Option<Rect>,
    pub(crate) over: Option<DropAt>,
}

impl TabDrag {
    pub(crate) fn armed(
        from: usize,
        name: String,
        origin_kind: DragFrom,
        col: u16,
        row: u16,
    ) -> Self {
        TabDrag {
            from,
            name,
            origin_kind,
            started: false,
            origin: (col, row),
            at: (col, row),
            source_rect: None,
            over: None,
        }
    }

    /// Whether the pointer at `(col, row)` has gone far enough to start it.
    pub(crate) fn passes_threshold(&self, col: u16, row: u16) -> bool {
        col.abs_diff(self.origin.0).max(row.abs_diff(self.origin.1)) >= DRAG_THRESHOLD
    }
}

/// The slot (0..=count) a pointer at `(col, row)` points at: the number of
/// tabs whose centre lies before it (left of it on the bar, above it in the
/// column; a pointer on a tab's own row drops after it when the tab being
/// carried (`vertical` holds its index) comes before). `hits` are the
/// repository tabs only.
pub(crate) fn slot_at(hits: &[TabHit], col: u16, row: u16, vertical: Option<usize>) -> usize {
    hits.iter()
        .take_while(|h| {
            if let Some(from) = vertical {
                row > h.tab.y || (row == h.tab.y && h.index >= from)
            } else {
                col >= h.tab.x + h.tab.width / 2
            }
        })
        .count()
}

/// The index a tab picked up at `from` ends at when dropped on slot `slot`
/// (a position between tabs of the *original* order).
pub(crate) fn index_after_drop(from: usize, slot: usize) -> usize {
    if slot > from {
        slot - 1
    } else {
        slot
    }
}

/// The insertion marker for `slot` on the tab bar: one cell in the tab's
/// own padding, so it never covers a label or a close button.
pub(crate) fn marker_for_slot(hits: &[TabHit], slot: usize, vertical: bool) -> Option<Rect> {
    if vertical {
        // The first cell of the row the tab would take; past the last tab,
        // of the row below it.
        return match hits.get(slot) {
            Some(h) => Some(Rect::new(h.tab.x, h.tab.y, 1, 1)),
            None => hits.last().map(|h| Rect::new(h.tab.x, h.tab.y + 1, 1, 1)),
        };
    }
    match hits.get(slot) {
        Some(h) => Some(Rect::new(h.tab.x, h.tab.y, 1, 1)),
        None => hits
            .last()
            .map(|h| Rect::new(h.tab.x + h.tab.width.saturating_sub(1), h.tab.y, 1, 1)),
    }
}

/// The cells of the picker's filtered row `pos`, when it is in the visible
/// window (the same walk `palette_row_at` resolves clicks with).
pub(crate) fn row_rect(popup: &crate::chrome::PalettePopup, pos: usize) -> Option<Rect> {
    let rows = popup.list.height as usize;
    if pos < popup.window_start || pos >= popup.window_start + rows || popup.list.width < 3 {
        return None;
    }
    Some(Rect::new(
        popup.list.x + 1,
        popup.list.y + (pos - popup.window_start) as u16,
        popup.list.width - 2,
        1,
    ))
}

/// Updates the drop target of a bar drag for a pointer at column `col`.
pub(crate) fn update_bar_target(
    drag: &mut TabDrag,
    hits: &[TabHit],
    tabs: usize,
    (col, row): (u16, u16),
    vertical: bool,
) {
    let repo_hits: Vec<TabHit> = hits.iter().filter(|h| h.index < tabs).cloned().collect();
    // The column shows a window of the tabs: a slot counts from its first.
    let first = repo_hits.first().map_or(0, |h| h.index);
    let slot = first + slot_at(&repo_hits, col, row, vertical.then_some(drag.from));
    drag.over = marker_for_slot(&repo_hits, slot - first, vertical).map(|mark| DropAt {
        to: index_after_drop(drag.from, slot),
        mark,
        vertical,
    });
}

/// Paints the drag's feedback over an already-drawn frame: the source tab
/// dimmed, the insertion marker or lit row, and the ghost label following the
/// pointer (the same look as the ref drag's). Only touches the current
/// frame's cells, so nothing survives the drag.
pub(crate) fn paint(buf: &mut ratatui::buffer::Buffer, drag: &TabDrag) {
    if !drag.started {
        return;
    }
    let area = buf.area;
    if area.width == 0 || area.height == 0 {
        return;
    }
    let inside = |x: u16, y: u16| x < area.x + area.width && y < area.y + area.height;
    if let Some(r) = drag.source_rect {
        for y in r.y..r.y.saturating_add(r.height) {
            for x in r.x..r.x.saturating_add(r.width) {
                if inside(x, y) {
                    buf[(x, y)].set_style(Style::default().add_modifier(Modifier::DIM));
                }
            }
        }
    }
    if let Some(over) = &drag.over {
        match drag.origin_kind {
            DragFrom::Bar => {
                let (x, y) = (over.mark.x, over.mark.y);
                if inside(x, y) {
                    let bar = if over.vertical {
                        "\u{2501}"
                    } else {
                        "\u{2503}"
                    };
                    buf[(x, y)].set_symbol(bar).set_style(
                        Style::default()
                            .fg(th().ui.accent)
                            .add_modifier(Modifier::BOLD),
                    );
                }
            }
            DragFrom::Picker => {
                let r = over.mark;
                for y in r.y..r.y.saturating_add(r.height) {
                    for x in r.x..r.x.saturating_add(r.width) {
                        if inside(x, y) {
                            buf[(x, y)].set_style(crate::theme::plain_button_style(true));
                        }
                    }
                }
            }
        }
    }
    let label = format!(" \u{21C4} {} ", drag.name);
    let width = Line::from(label.as_str()).width() as u16;
    let (px, py) = drag.at;
    let mut x = px.saturating_add(2);
    if x.saturating_add(width) > area.x + area.width {
        x = (area.x + area.width).saturating_sub(width).max(area.x);
    }
    let y = if py + 1 < area.y + area.height {
        py + 1
    } else {
        py.saturating_sub(1).max(area.y)
    };
    let max = (area.x + area.width).saturating_sub(x) as usize;
    buf.set_stringn(
        x,
        y,
        label,
        max,
        Style::default()
            .fg(th().chrome.on_bright)
            .bg(th().ui.accent)
            .add_modifier(Modifier::BOLD),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hits(n: usize) -> Vec<TabHit> {
        (0..n)
            .map(|i| TabHit {
                index: i,
                tab: Rect::new(10 + 10 * i as u16, 0, 10, 1),
                close: Rect::new(18 + 10 * i as u16, 0, 1, 1),
            })
            .collect()
    }

    #[test]
    fn the_slot_is_the_number_of_tab_centres_left_of_the_pointer() {
        let h = hits(3);
        assert_eq!(slot_at(&h, 0, 0, None), 0);
        assert_eq!(slot_at(&h, 14, 0, None), 0);
        assert_eq!(slot_at(&h, 15, 0, None), 1);
        assert_eq!(slot_at(&h, 29, 0, None), 2);
        assert_eq!(slot_at(&h, 80, 0, None), 3);
    }

    #[test]
    fn dropping_after_the_source_accounts_for_its_own_removal() {
        assert_eq!(index_after_drop(0, 2), 1);
        assert_eq!(index_after_drop(2, 0), 0);
        assert_eq!(index_after_drop(1, 1), 1);
        assert_eq!(index_after_drop(1, 2), 1, "right after itself is a no-op");
        assert_eq!(index_after_drop(0, 3), 2);
    }

    #[test]
    fn the_marker_sits_in_a_tab_padding_cell() {
        let h = hits(2);
        assert_eq!(marker_for_slot(&h, 0, false), Some(Rect::new(10, 0, 1, 1)));
        assert_eq!(marker_for_slot(&h, 1, false), Some(Rect::new(20, 0, 1, 1)));
        assert_eq!(marker_for_slot(&h, 2, false), Some(Rect::new(29, 0, 1, 1)));
        assert_eq!(marker_for_slot(&[], 0, false), None);
    }

    fn column(n: usize) -> Vec<TabHit> {
        (0..n)
            .map(|i| TabHit {
                index: i,
                tab: Rect::new(1, 3 + i as u16, 24, 1),
                close: Rect::new(23, 3 + i as u16, 1, 1),
            })
            .collect()
    }

    #[test]
    fn in_a_column_the_pointer_row_picks_the_slot_by_rows_not_columns() {
        let h = column(4);
        // Carrying tab 1: on its own row, the drop is a no-op.
        assert_eq!(slot_at(&h, 5, 4, Some(1)), 2);
        assert_eq!(index_after_drop(1, 2), 1);
        // Down onto tab 3's row lands there.
        assert_eq!(slot_at(&h, 5, 6, Some(1)), 4);
        assert_eq!(index_after_drop(1, 4), 3);
        // Up onto tab 0's row lands there.
        assert_eq!(slot_at(&h, 5, 3, Some(2)), 0);
        assert_eq!(index_after_drop(2, 0), 0);
        // Above the list is the first slot, the column does not care about x.
        assert_eq!(slot_at(&h, 99, 0, Some(2)), 0);
    }

    #[test]
    fn the_column_marker_is_the_first_cell_of_a_row() {
        let h = column(2);
        assert_eq!(marker_for_slot(&h, 0, true), Some(Rect::new(1, 3, 1, 1)));
        assert_eq!(marker_for_slot(&h, 1, true), Some(Rect::new(1, 4, 1, 1)));
        assert_eq!(marker_for_slot(&h, 2, true), Some(Rect::new(1, 5, 1, 1)));
    }

    #[test]
    fn a_scrolled_column_counts_slots_from_its_first_visible_tab() {
        let mut h = column(3);
        for hit in &mut h {
            hit.index += 10;
        }
        let mut drag = TabDrag::armed(11, "x".into(), DragFrom::Bar, 5, 4);
        update_bar_target(&mut drag, &h, 20, (5, 5), true);
        let over = drag.over.unwrap();
        assert_eq!(over.to, 12);
        assert!(over.vertical);
    }

    #[test]
    fn a_drag_starts_only_past_the_threshold() {
        let d = TabDrag::armed(0, "a".into(), DragFrom::Bar, 10, 0);
        assert!(!d.passes_threshold(11, 0));
        assert!(d.passes_threshold(12, 0));
        assert!(d.passes_threshold(10, 2));
    }
}
