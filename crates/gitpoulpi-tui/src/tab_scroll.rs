//! Horizontal scrolling of the repository tab bar.
//!
//! When the tabs do not fit, the bar shows a window of them between two arrow
//! buttons (`‹` at its left end, `›` at its right end, dimmed when there is
//! nothing more on that side). A click on an arrow, a held press (the
//! scrollbar arrows' auto-repeat), the wheel over the bar and a tab dragged
//! onto an arrow all move the window by one tab. Whenever the active tab, the
//! number of tabs or the bar's width changes, the window moves just enough to
//! show the active tab whole (`window_start`); a scroll by the user is never
//! undone by the next frame.
//!
//! The layout itself is the bar's own paint walk (`repo_switcher::tab_bar_with`,
//! hit rects from the same walk); this module only keeps the process-wide
//! state, like `tab_column` does for the vertical column: the first visible
//! tab, what the last follow saw, where the bar and its arrows were drawn.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use ratatui::layout::Rect;

/// The left arrow's glyph. U+2039 (single left-pointing angle quotation
/// mark) is text-presentation, single width and in every monospace font.
pub(crate) const LEFT_GLYPH: &str = "\u{2039}";
/// The right arrow's glyph (U+203A).
pub(crate) const RIGHT_GLYPH: &str = "\u{203A}";
/// The left arrow button: its glyph with a column of padding either side.
pub(crate) const LEFT_BUTTON: &str = " \u{2039} ";
/// The right arrow button.
pub(crate) const RIGHT_BUTTON: &str = " \u{203A} ";
/// The width of either arrow button.
pub(crate) const ARROW_W: u16 = 3;

static OFFSET: AtomicUsize = AtomicUsize::new(0);
static FOLLOWED: Mutex<Option<(usize, usize, u16)>> = Mutex::new(None);
static DRAWN: Mutex<Drawn> = Mutex::new(Drawn {
    bar: None,
    left: None,
    right: None,
});
static DRAG_HOLD: Mutex<Option<(isize, Instant)>> = Mutex::new(None);

/// Where the bar and its enabled arrows were painted last frame.
#[derive(Clone, Copy)]
struct Drawn {
    bar: Option<Rect>,
    left: Option<Rect>,
    right: Option<Rect>,
}

fn drawn() -> std::sync::MutexGuard<'static, Drawn> {
    DRAWN.lock().unwrap_or_else(|e| e.into_inner())
}

/// The offset to draw with, and the tab to bring into view: the active one,
/// on the frame where it, the number of tabs or the bar's width changed
/// (`None` on Home, where no repository tab is selected).
pub(crate) fn scroll_for_draw(
    active: Option<usize>,
    tabs: usize,
    width: u16,
) -> (usize, Option<usize>) {
    let mut last = FOLLOWED.lock().unwrap_or_else(|e| e.into_inner());
    let follow = active.filter(|a| {
        let key = (*a, tabs, width);
        let changed = *last != Some(key);
        *last = Some(key);
        changed
    });
    if active.is_none() {
        *last = None;
    }
    (OFFSET.load(Ordering::Relaxed), follow)
}

/// Stores what one draw of the bar settled on: its first visible tab, the
/// bar's rect (`None`: the horizontal bar is not on screen) and its enabled
/// arrows.
pub(crate) fn store(offset: usize, bar: Option<Rect>, left: Option<Rect>, right: Option<Rect>) {
    OFFSET.store(offset, Ordering::Relaxed);
    *drawn() = Drawn { bar, left, right };
}

/// Moves the window by `delta` tabs (clamped at the next draw).
pub(crate) fn scroll_by(delta: isize) {
    let now = OFFSET.load(Ordering::Relaxed) as isize;
    OFFSET.store((now + delta).max(0) as usize, Ordering::Relaxed);
}

/// The wheel over the horizontal bar: `true` when it was over it (and so
/// scrolled it, or would have, had there been anything to scroll).
pub(crate) fn wheel(col: u16, row: u16, delta: isize) -> bool {
    let over = drawn().bar.is_some_and(|b| b.contains((col, row).into()));
    if over {
        scroll_by(delta);
    }
    over
}

/// The direction of the enabled arrow under `(col, row)`, if any.
pub(crate) fn arrow_at(col: u16, row: u16) -> Option<isize> {
    let d = *drawn();
    let on = |r: Option<Rect>| r.is_some_and(|r| r.contains((col, row).into()));
    if on(d.left) {
        Some(-1)
    } else if on(d.right) {
        Some(1)
    } else {
        None
    }
}

/// A tab dragged along the bar is over `(col, row)`: over an enabled arrow
/// the bar scrolls one tab at once, then keeps scrolling while the pointer
/// stays there (`drag_due`); anywhere else the hold ends.
pub(crate) fn drag_over(col: u16, row: u16, now: Instant) {
    let mut hold = DRAG_HOLD.lock().unwrap_or_else(|e| e.into_inner());
    match arrow_at(col, row) {
        Some(dir) if hold.is_none_or(|(d, _)| d != dir) => {
            scroll_by(dir);
            *hold = Some((dir, now + crate::scrollbar_repeat::DELAY));
        }
        Some(_) => {}
        None => *hold = None,
    }
}

/// The drag ended (dropped or canceled).
pub(crate) fn drag_end() {
    *DRAG_HOLD.lock().unwrap_or_else(|e| e.into_inner()) = None;
}

/// The pace of the scroll while a dragged tab rests on an arrow: slower than
/// a scrollbar's, one tab is a long way.
const DRAG_RATE: Duration = Duration::from_millis(250);

/// One more scroll step of a drag held on an arrow when it is due; `true`
/// when the bar moved.
pub(crate) fn drag_due(now: Instant) -> bool {
    let mut hold = DRAG_HOLD.lock().unwrap_or_else(|e| e.into_inner());
    match *hold {
        Some((dir, next)) if now >= next => {
            scroll_by(dir);
            *hold = Some((dir, now + DRAG_RATE));
            true
        }
        _ => false,
    }
}

/// When the loop must wake for the next drag scroll step.
pub(crate) fn drag_deadline() -> Option<Instant> {
    DRAG_HOLD
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .map(|(_, next)| next)
}

/// The first visible tab for tabs of `widths` cells in a strip of `room`
/// cells, starting from the requested `offset` and bringing `follow` into
/// view with the least movement: a tab left of the window becomes its first,
/// a tab right of it its last. The window never scrolls past the point where
/// the last tab is flush with the strip's end.
pub(crate) fn window_start(
    widths: &[u16],
    room: u16,
    offset: usize,
    follow: Option<usize>,
) -> usize {
    let n = widths.len();
    if n == 0 {
        return 0;
    }
    let room = u32::from(room);
    let mut max = n - 1;
    let mut acc = 0u32;
    for i in (0..n).rev() {
        acc += u32::from(widths[i]);
        if acc > room {
            break;
        }
        max = i;
    }
    let mut start = offset.min(max);
    if let Some(a) = follow.filter(|a| *a < n) {
        if a < start {
            start = a;
        } else {
            let mut acc: u32 = widths[start..=a].iter().map(|w| u32::from(*w)).sum();
            while acc > room && start < a {
                acc -= u32::from(widths[start]);
                start += 1;
            }
        }
    }
    start
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tab_right_of_the_window_becomes_its_last() {
        // Five tabs of 10 cells in 25 cells: two fit.
        let w = [10; 5];
        assert_eq!(window_start(&w, 25, 0, Some(4)), 3);
        assert_eq!(window_start(&w, 25, 0, Some(2)), 1);
    }

    #[test]
    fn a_tab_left_of_the_window_becomes_its_first() {
        let w = [10; 5];
        assert_eq!(window_start(&w, 25, 3, Some(1)), 1);
    }

    #[test]
    fn a_visible_tab_does_not_move_the_window() {
        let w = [10; 5];
        assert_eq!(window_start(&w, 25, 1, Some(2)), 1);
        assert_eq!(window_start(&w, 25, 1, Some(1)), 1);
    }

    #[test]
    fn the_window_never_scrolls_past_the_last_tab() {
        let w = [10; 5];
        assert_eq!(window_start(&w, 25, 99, None), 3);
        assert_eq!(window_start(&w, 100, 2, None), 0);
    }

    #[test]
    fn a_tab_wider_than_the_strip_starts_the_window() {
        let w = [10, 40, 10];
        assert_eq!(window_start(&w, 25, 0, Some(1)), 1);
        assert_eq!(window_start(&[], 25, 4, Some(1)), 0);
    }
}
