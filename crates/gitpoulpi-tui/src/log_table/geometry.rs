//! The Log's row arithmetic: top row, window, index to screen row and back.

use super::*;

/// First terminal row of the Log table's content (past its top border and
/// its header row). Both directions of the Log's row arithmetic key off
/// this, so it lives in one place rather than being re-derived as
/// `log_area.y + 2` at each site.
pub(crate) fn log_top_y(area: Rect) -> u16 {
    area.y + 2
}

/// How many Log rows the window can show: exactly the band
/// `log_row_to_index` accepts, i.e. everything between the header row and
/// the bottom border.
///
/// Derived from the same two bounds that function uses rather than from
/// `area.height` minus a hand-counted constant. That constant is the
/// Log's row arithmetic, which `docs/developer/where-we-are.md` names as
/// this file's repeat offender, and it has no business being written down
/// twice: the lazy loader asks "how much is left below the window" and a
/// click asks "which commit is at this row", and the two must agree about
/// which rows exist.
pub(crate) fn log_visible_rows(area: Rect) -> usize {
    (area.y + area.height)
        .saturating_sub(1)
        .saturating_sub(log_top_y(area)) as usize
}

/// The slice of display rows the Log paints this frame, and where the
/// selection sits inside it.
///
/// The Log only builds `start..end` (a screenful), not every loaded commit:
/// building 5,000 rows per frame was the TUI's whole idle cost. `start` is
/// the offset ratatui's `Table` would settle on - `Table::get_row_bounds`
/// for rows one line tall - so the window is exactly the rows it would have
/// drawn, and the row arithmetic (`log_index_to_row`, `log_row_to_index`)
/// keeps using absolute display indices with `start` as its offset. The
/// table is then rendered over just those rows with offset 0 and the
/// selection made relative (`selected`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LogWindow {
    /// First display index drawn; the Log's scroll offset after this frame.
    pub start: usize,
    /// One past the last display index drawn.
    pub end: usize,
    /// The selection relative to `start`, when it is inside the window.
    pub selected: Option<usize>,
}

/// `selected` is the render-state selection (`table_render_state`: `None`
/// when it is scrolled out of view and the Log is not following it).
pub(crate) fn log_window(
    offset: usize,
    selected: Option<usize>,
    visible: usize,
    total: usize,
) -> LogWindow {
    if total == 0 {
        return LogWindow {
            start: 0,
            end: 0,
            selected: None,
        };
    }
    let mut start = offset.min(total - 1);
    let selected = selected.map(|s| s.min(total - 1));
    if let Some(sel) = selected {
        if sel < start || visible == 0 {
            start = sel;
        } else if sel >= start + visible {
            start = sel + 1 - visible;
        }
    }
    let end = (start + visible).min(total);
    LogWindow {
        start,
        end,
        selected: selected
            .filter(|s| (start..end).contains(s))
            .map(|s| s - start),
    }
}

/// How far `PageUp`/`PageDown` move the Log's selection: one visible
/// screen of rows - "page down/up c'est monter/descendre d'un écran
/// visible". Floored at 1 so a pane too short to show even one full row
/// (mid-resize, or a pathologically small terminal) still moves the
/// selection rather than becoming a dead key.
pub(crate) fn log_page_delta(area: Rect) -> i32 {
    (log_visible_rows(area) as i32).max(1)
}

/// Screen row where Log display-index `index` is drawn.
///
/// **Every Log row is one terminal line, the WIP row included**, so this
/// is plain arithmetic - and the one place it is written, so it can never
/// drift from `log_row_to_index`. Pinned by
/// `log_index_to_row_round_trips_with_log_row_to_index`.
pub(crate) fn log_index_to_row(top_y: u16, offset: usize, index: usize) -> Option<u16> {
    let rel = index.checked_sub(offset)?;
    Some(top_y + rel as u16)
}

/// Inverse of `log_index_to_row`: maps a click's screen row back to a Log
/// display index, or `None` on the border, the header row, or below the
/// last row.
pub(crate) fn log_row_to_index(area: Rect, offset: usize, row: u16, len: usize) -> Option<usize> {
    let top = log_top_y(area);
    if row < top || row >= area.y + area.height - 1 {
        return None; // border and header rows
    }
    let index = (row - top) as usize + offset;
    (index < len).then_some(index)
}
