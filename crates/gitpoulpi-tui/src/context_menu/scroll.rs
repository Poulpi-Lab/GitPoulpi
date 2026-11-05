//! The scroll state of the one context menu on screen.
//!
//! A menu taller than the screen shows `top..top + visible` rows and draws the
//! shared scrollbar in its right border. Exactly one menu is open at a time (a
//! new one closes the others), and `Menu::render` takes `&self`, so the state
//! lives here, keyed by the menu's id and open submenu: opening another menu
//! or entering a submenu starts again at the top.
//!
//! The selection keeps being followed: `top` moves only when the selection
//! does (a key, a click), so a wheel turn or a thumb drag that scrolled the
//! window away is not undone by the next frame. The registry is cleared as a
//! frame starts and refilled by the paint, so a closed menu takes no click.

use std::cell::RefCell;

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

use crate::scrollbar::{BarDrag, Track};

/// Rows one wheel notch moves a list.
const WHEEL_ROWS: usize = 3;

#[derive(Default)]
struct Scroll {
    id: u64,
    path: Vec<usize>,
    top: usize,
    /// The selection `top` last followed.
    followed: Option<usize>,
    rows: usize,
    visible: usize,
    /// The popup as last painted, for the wheel.
    popup: Option<Rect>,
    track: Option<Track>,
    drag: BarDrag,
}

thread_local! {
    static SCROLL: RefCell<Scroll> = RefCell::new(Scroll::default());
}

/// Forgets last frame's popup and bar; called as a frame starts.
pub(crate) fn begin_frame() {
    SCROLL.with(|s| {
        let mut s = s.borrow_mut();
        s.popup = None;
        s.track = None;
    });
}

/// The first row to draw for menu `id` at submenu `path`, with `rows` rows of
/// which `visible` fit and `selected` highlighted; records the bar's track for
/// the popup whose bordered rect is `popup`. Returns the offset and the bar.
pub(super) fn frame(
    id: u64,
    path: &[usize],
    rows: usize,
    visible: usize,
    selected: usize,
    popup: Rect,
) -> (usize, Option<Track>) {
    SCROLL.with(|s| {
        let mut s = s.borrow_mut();
        if s.id != id || s.path != path {
            *s = Scroll {
                id,
                path: path.to_vec(),
                ..Scroll::default()
            };
        }
        if s.followed != Some(selected) {
            s.followed = Some(selected);
            s.top = follow(s.top, selected, visible);
        }
        s.top = if visible == 0 || rows <= visible {
            0
        } else {
            s.top.min(rows - visible)
        };
        s.rows = rows;
        s.visible = visible;
        s.popup = Some(popup);
        s.track = Track::column(
            Rect::new(
                popup.right().saturating_sub(1),
                popup.y + 1,
                1,
                visible as u16,
            ),
            rows,
            visible,
            s.top,
        );
        (s.top, s.track)
    })
}

/// The smallest move of `top` that brings `selected` into view.
pub(super) fn follow(top: usize, selected: usize, visible: usize) -> usize {
    if visible == 0 || selected < top {
        selected
    } else if selected >= top + visible {
        selected + 1 - visible
    } else {
        top
    }
}

/// A mouse event for the open menu's scrollbar or wheel: `true` when it took
/// it (the event goes no further).
pub(crate) fn on_mouse(mouse: MouseEvent) -> bool {
    SCROLL.with(|s| {
        let mut s = s.borrow_mut();
        let Some(popup) = s.popup else {
            return false;
        };
        let scrollable = s.rows > s.visible;
        match mouse.kind {
            MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
                if !scrollable || !popup.contains((mouse.column, mouse.row).into()) {
                    return false;
                }
                let max = s.rows - s.visible;
                s.top = if mouse.kind == MouseEventKind::ScrollDown {
                    (s.top + WHEEL_ROWS).min(max)
                } else {
                    s.top.saturating_sub(WHEEL_ROWS)
                };
                true
            }
            MouseEventKind::Down(MouseButton::Left)
            | MouseEventKind::Drag(MouseButton::Left)
            | MouseEventKind::Up(MouseButton::Left) => {
                let held = s.drag.held();
                let (track, rows, top) = (s.track, s.rows, s.top);
                if let Some(new) = s.drag.on_mouse(track, mouse, rows, top) {
                    s.top = new;
                    true
                } else {
                    // The release of a held thumb ends the drag here.
                    held && matches!(mouse.kind, MouseEventKind::Up(_))
                }
            }
            _ => false,
        }
    })
}

/// The first row drawn now (tests).
#[cfg(test)]
pub(crate) fn top() -> usize {
    SCROLL.with(|s| s.borrow().top)
}

/// The bar as last painted (tests).
#[cfg(test)]
pub(crate) fn track() -> Option<Track> {
    SCROLL.with(|s| s.borrow().track)
}
