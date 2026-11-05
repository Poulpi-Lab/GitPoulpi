//! The lines above a tab's list (what was read, warnings, a result's totals)
//! never take the list's last rows: in a short pane the list keeps at least
//! [`MIN_LIST`] rows, the text above it is clipped, and the lines that only
//! inform (`optional`) go before the ones that warn or report an error.

use ratatui::layout::Rect;

/// Rows a list keeps in a short pane.
pub(super) const MIN_LIST: u16 = 4;

/// The y that header text must stay above, so `below` fixed rows (the tab's
/// button row, a toolbar, the actions row) and [`MIN_LIST`] list rows fit
/// under it. In a very short pane that is the top: the text goes, the
/// buttons and the list stay.
pub(super) fn cap(body: Rect, below: u16) -> u16 {
    body.bottom().saturating_sub(MIN_LIST + below)
}

/// The y an `optional` header line must stay above: the room the required
/// lines still to come (`reserve` rows) need is left free.
pub(super) fn limit(cap: u16, reserve: u16, optional: bool) -> u16 {
    if optional {
        cap.saturating_sub(reserve)
    } else {
        cap
    }
}
