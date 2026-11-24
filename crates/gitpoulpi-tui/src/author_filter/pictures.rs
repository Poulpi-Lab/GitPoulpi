//! The popup's real avatars: the Log column's round picture in the two cells
//! where the initials badge is, through the Log's own `LogAvatarImages`.
//!
//! The popup is a text popup that paints the badge itself (`render`); this
//! runs after it, once per frame, and claims the badge's cells for the
//! picture of every visible author whose picture is ready. Nothing waits: an
//! author without a picture yet keeps the badge and gets the picture on the
//! frame after it lands. Clearing is the Log pictures' rule: `finish_frame`
//! (after every popup) erases or re-sends rows that scrolled, were filtered
//! out, or whose cells the popup no longer covers, and the popup closing
//! simply draws no slot, so the pictures it left are withdrawn.

use super::{AuthorFilterPopup, AuthorRow};
use crate::avatar::{self, AvatarService, LogAvatarImages, LogAvatarMode};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

/// The visible author rows that get a picture slot: `(slot, entry index,
/// row index)`. Only with `mode` showing pictures and the popup's avatars on.
pub(crate) fn slots(
    popup: &AuthorFilterPopup,
    list: Rect,
    window_start: usize,
    mode: LogAvatarMode,
) -> Vec<(Rect, usize, usize)> {
    if mode != LogAvatarMode::Picture || !popup.avatars {
        return Vec::new();
    }
    let rows = popup.rows();
    // Left border, then the tick and its space: the badge starts at +3.
    let x = list.x + 3;
    let right = list.right().saturating_sub(1);
    rows.iter()
        .enumerate()
        .filter_map(|(i, row)| {
            // The scrolled window the popup's own list shows (its bar is
            // painted by `render`).
            let line = i.checked_sub(window_start)?;
            if line >= usize::from(list.height) {
                return None;
            }
            let AuthorRow::Author(e) = row else {
                return None;
            };
            let entry = popup.entry(*e)?;
            if entry.email.trim().is_empty() {
                return None;
            }
            let y = list.y + u16::try_from(line).ok()?;
            let area = avatar::slot_area(x, avatar::SLOT_COLS + 1, y, right)?;
            Some((area, *e, i))
        })
        .collect()
}

/// Claims the badge cells of every visible author whose picture is ready.
/// Call after the popup is drawn and before the pictures' `finish_frame`.
pub(crate) fn paint(
    buf: &mut Buffer,
    popup: &AuthorFilterPopup,
    rects: &crate::chrome::PalettePopup,
    service: &mut AvatarService,
    images: &mut LogAvatarImages,
    mode: LogAvatarMode,
) {
    let wanted = slots(popup, rects.list, rects.window_start, mode);
    if wanted.is_empty() {
        return;
    }
    let entries: Vec<&super::AuthorEntry> = wanted
        .iter()
        .filter_map(|(_, e, _)| popup.entry(*e))
        .collect();
    let queries: Vec<avatar::AvatarQuery> = entries
        .iter()
        .map(|e| avatar::AvatarQuery {
            email: e.email.as_str(),
            sha: e.sha.as_str(),
        })
        .collect();
    let found = service.pictures(&queries);
    let draw: Vec<avatar::AvatarSlot> = wanted
        .iter()
        .zip(found)
        .filter_map(|((area, _, row), picture)| {
            let (author, bitmap) = picture?;
            Some(avatar::AvatarSlot {
                area: *area,
                key: avatar::PictureKey {
                    author: author.to_string(),
                    tint: Some(avatar::RowTint::of(*row == popup.selected, false)),
                    dimmed: false,
                },
                bitmap,
            })
        })
        .collect();
    images.render(buf, &draw);
}
