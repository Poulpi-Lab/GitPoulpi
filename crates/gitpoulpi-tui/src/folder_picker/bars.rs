//! Scrollbars of the folder picker's three lists: the left column, the
//! folder content and the path completion dropdown. Each bar is painted in
//! the last column of its list (the rows give up that column only while the
//! list overflows) and its `Track` is kept from the paint, so a press and a
//! drag hit exactly what the user sees.

use super::*;
use crate::scrollbar::{self, BarDrag, Track};

/// One list's bar as last painted, and the thumb held.
#[derive(Debug, Default, Clone, Copy)]
pub(super) struct PickerBar {
    pub(super) track: Option<Track>,
    drag: BarDrag,
    content: usize,
}

impl PickerBar {
    /// Records and paints the bar for `rows` lines in `area`, and returns the
    /// rect the rows keep: one column goes to the bar while the list overflows.
    pub(super) fn paint(&mut self, buf: &mut Buffer, area: Rect, rows: usize, top: usize) -> Rect {
        self.content = rows;
        self.track = if area.width > 2 {
            Track::column(
                Rect::new(area.right() - 1, area.y, 1, area.height),
                rows,
                area.height as usize,
                top,
            )
        } else {
            None
        };
        match self.track {
            Some(track) => {
                scrollbar::render_buf(buf, track, true, false);
                Rect {
                    width: area.width - 1,
                    ..area
                }
            }
            None => area,
        }
    }

    /// The list is not drawn: no bar to press.
    pub(super) fn hide(&mut self) {
        self.track = None;
    }

    /// Feeds a mouse event: the new first row when the bar took it.
    fn mouse(&mut self, mouse: MouseEvent, top: usize) -> Option<usize> {
        self.drag.on_mouse(self.track, mouse, self.content, top)
    }

    /// Whether the pointer is on the bar or its thumb is held.
    fn owns(&self, mouse: &MouseEvent) -> bool {
        self.drag.held()
            || self
                .track
                .is_some_and(|t| t.rect.contains((mouse.column, mouse.row).into()))
    }
}

impl FolderPicker {
    /// A mouse event for the lists' bars: `true` when one took it.
    pub(super) fn bar_mouse(&mut self, mouse: MouseEvent) -> bool {
        // The dropdown floats over the content list: its bar answers first.
        if let Some(top) = self.drop_bar.mouse(mouse, self.drop_top) {
            self.drop_top = top;
            return true;
        }
        if let Some(top) = self.left_bar.mouse(mouse, self.left_scroll) {
            self.left_scroll = top;
            return true;
        }
        if let Some(top) = self.content_bar.mouse(mouse, self.content_scroll) {
            self.content_scroll = top;
            return true;
        }
        // A release is never consumed by the drag; anything else on a bar
        // (a move over it, a right click) is the bar's too.
        !matches!(mouse.kind, MouseEventKind::Up(_))
            && (self.left_bar.owns(&mouse)
                || self.content_bar.owns(&mouse)
                || self.drop_bar.owns(&mouse))
    }
}
