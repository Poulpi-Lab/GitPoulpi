//! One list behavior for every filterable popup: the keys that move the
//! highlight ([`nav`]), and, for the popups painted by the main window (the
//! command palette, the tab / theme / language pickers), the scroll window,
//! the scrollbar and the click semantics ([`View`]).
//!
//! Keys: `Up`/`Down` move one row (never a `Ctrl+letter` pair: `Ctrl+N` is the
//! navigation prefix and `Ctrl+P` the command palette everywhere, and a filter
//! field owns the letters, `Ctrl+K` included);
//! `Home`/`End` jump to the first/last row; `PageUp`/`PageDown` move one
//! visible page. These win over the filter field's own caret keys (`Home`,
//! `End`, `Ctrl+K` kill-to-end): the filter is a short query, and the list is
//! what those keys are for in a popup. `Ctrl+A`/`Ctrl+E` stay caret keys.
//!
//! Mouse: the wheel scrolls the window without moving the highlight, a single
//! click selects, a second click on the same row (double click) validates, and
//! a list taller than its window paints the shared scrollbar.

use crate::scrollbar::{self, Track};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::Rect;
use std::cell::RefCell;
use std::time::Instant;

/// Rows one wheel notch scrolls.
pub(crate) const WHEEL_STEP: usize = 3;

/// The new highlight when `key` is a list-navigation key, `None` when it is
/// not one (the caller goes on to its field / other keys). `len` is the row
/// count, `page` the number of rows visible at once.
pub(crate) fn nav(key: &KeyEvent, selected: usize, len: usize, page: usize) -> Option<usize> {
    if key.kind == KeyEventKind::Release {
        return None;
    }
    let alt = key.modifiers.contains(KeyModifiers::ALT);
    let last = len.saturating_sub(1);
    let page = page.saturating_sub(1).max(1);
    let to = match key.code {
        KeyCode::Up if !alt => selected.saturating_sub(1),
        KeyCode::Down if !alt => selected + 1,
        KeyCode::Home if !alt => 0,
        KeyCode::End if !alt => last,
        KeyCode::PageUp if !alt => selected.saturating_sub(page),
        KeyCode::PageDown if !alt => selected + page,
        _ => return None,
    };
    Some(to.min(last))
}

#[derive(Default)]
struct State {
    /// A wheel/scrollbar scroll in force: the window offset, and the
    /// highlight it was made for (any move of the highlight follows it again).
    free: Option<(usize, usize)>,
    /// The window the last paint showed.
    offset: usize,
    rows: usize,
    count: usize,
    painted: bool,
    track: Option<Track>,
    drag_grab: Option<i32>,
    click: Option<(Instant, usize)>,
}

/// The "second click on the same row within the window" memory of a popup
/// that owns its state (a [`crate::combo::Combo`]), as opposed to [`View`]'s
/// thread-local one: `View::begin_frame` wipes that when the popup did not
/// paint through `View`, which a combobox never does, so a frame drawn between
/// the two clicks of a real double-click (the pair never shares a drain
/// outside a test) erased the first one.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ClickPair(Option<(Instant, usize)>);

impl ClickPair {
    /// Records a click on row `index`; `true` when it completes a double click
    /// (and consumes the pair).
    pub(crate) fn register(&mut self, index: usize) -> bool {
        let now = Instant::now();
        let double = self.0.is_some_and(|(t, i)| {
            i == index && now.duration_since(t) < crate::pane_layout::DOUBLE_CLICK_WINDOW
        });
        self.0 = (!double).then_some((now, index));
        double
    }

    pub(crate) fn reset(&mut self) {
        self.0 = None;
    }
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

/// Window / scrollbar / click state of the one popup list on screen.
pub(crate) struct View;

impl View {
    /// Called as a frame starts: a popup that did not paint last frame is
    /// gone, and its scroll state with it.
    pub(crate) fn begin_frame() {
        STATE.with(|s| {
            let mut s = s.borrow_mut();
            if !s.painted {
                *s = State::default();
            }
            s.painted = false;
        });
    }

    /// The first visible row for a list of `count` rows in a window of
    /// `rows`, with `selected` highlighted: the highlight is kept in view
    /// unless the wheel or the scrollbar scrolled the window away from it.
    pub(crate) fn window_start(selected: usize, rows: usize, count: usize) -> usize {
        STATE.with(|s| {
            let mut s = s.borrow_mut();
            if s.count != count {
                s.free = None;
            }
            let max = count.saturating_sub(rows);
            let start = match s.free {
                Some((offset, anchor)) if anchor == selected => offset.min(max),
                _ => {
                    s.free = None;
                    if rows == 0 || selected < rows {
                        0
                    } else {
                        selected + 1 - rows
                    }
                }
            };
            s.offset = start;
            s.rows = rows;
            s.count = count;
            s.painted = true;
            start
        })
    }

    /// Records the window of a scrolled body that is not a highlight list
    /// (the help): its offset, visible rows and length, for the scrollbar.
    pub(crate) fn set_window(offset: usize, rows: usize, count: usize) {
        STATE.with(|s| {
            let mut s = s.borrow_mut();
            s.offset = offset;
            s.rows = rows;
            s.count = count;
            s.free = None;
            s.painted = true;
        });
    }

    /// The offset the wheel, a bar press or a thumb drag asked for since the
    /// last paint (a body without a highlight reads it back after the event).
    pub(crate) fn free_offset() -> Option<usize> {
        STATE.with(|s| s.borrow().free.map(|(offset, _)| offset))
    }

    /// Rows the popup list showed last frame (the `PgUp`/`PgDn` step).
    pub(crate) fn page() -> usize {
        STATE.with(|s| s.borrow().rows).max(2)
    }

    /// Paints the scrollbar over the right border of `list` (the block with
    /// LEFT|RIGHT borders whose every row is a content row) when `count`
    /// overflows its height, and remembers its geometry for the mouse.
    pub(crate) fn paint_bar(buf: &mut ratatui::buffer::Buffer, list: Rect, count: usize) {
        let offset = STATE.with(|s| s.borrow().offset);
        // `Track::vertical` takes the bordered rect and spans the rows
        // between its corners: stretch the list by one row each way.
        let body = Rect::new(
            list.x,
            list.y.saturating_sub(1),
            list.width,
            list.height + 2,
        );
        let track = Track::vertical(body, count, offset);
        STATE.with(|s| s.borrow_mut().track = track);
        if let Some(track) = track {
            let dragging = STATE.with(|s| s.borrow().drag_grab.is_some());
            scrollbar::render_buf(buf, track, true, dragging);
        }
    }

    fn scroll_to(offset: usize, selected: usize) {
        STATE.with(|s| {
            let mut s = s.borrow_mut();
            let max = s.count.saturating_sub(s.rows);
            s.free = Some((offset.min(max), selected));
        });
    }

    /// The wheel: scrolls the window `delta` rows, the highlight stays.
    pub(crate) fn wheel(delta: isize, selected: usize) {
        let offset = STATE.with(|s| s.borrow().offset);
        Self::scroll_to((offset as isize + delta).max(0) as usize, selected);
    }

    /// A press on the scrollbar: grabs the thumb, steps an arrow or jumps the thumb under the pointer.
    /// `false` when `(col, row)` is not on the bar.
    pub(crate) fn bar_press(col: u16, row: u16, selected: usize) -> bool {
        let (track, offset, rows) = STATE.with(|s| {
            let s = s.borrow();
            (s.track, s.offset, s.rows)
        });
        let Some(track) = track.filter(|t| t.rect.contains((col, row).into())) else {
            return false;
        };
        let pos = row - track.rect.y;
        match track.press(pos) {
            crate::scrollbar::TrackPress::Arrow(step) => {
                Self::scroll_to((offset as isize + step).max(0) as usize, selected);
            }
            crate::scrollbar::TrackPress::Grab { grab, jump } => {
                STATE.with(|s| s.borrow_mut().drag_grab = Some(grab));
                if jump {
                    let count = STATE.with(|s| s.borrow().count);
                    let to = track.drag_offset(count, rows, grab, i32::from(pos));
                    Self::scroll_to(to, selected);
                }
            }
        }
        true
    }

    /// A drag with the thumb held.
    pub(crate) fn bar_drag(row: u16, selected: usize) {
        let (track, grab, rows, count) = STATE.with(|s| {
            let s = s.borrow();
            (s.track, s.drag_grab, s.rows, s.count)
        });
        if let (Some(track), Some(grab)) = (track, grab) {
            let to = track.drag_offset(count, rows, grab, row as i32 - track.rect.y as i32);
            Self::scroll_to(to, selected);
        }
    }

    pub(crate) fn bar_release() {
        STATE.with(|s| s.borrow_mut().drag_grab = None);
    }

    /// A left click on row `index`: `true` when it is the second click on
    /// the same row within the double-click window (validate), else it only
    /// selects.
    pub(crate) fn click(index: usize) -> bool {
        let now = Instant::now();
        STATE.with(|s| {
            let mut s = s.borrow_mut();
            let double = s.click.is_some_and(|(t, i)| {
                i == index && now.duration_since(t) < crate::pane_layout::DOUBLE_CLICK_WINDOW
            });
            s.click = (!double).then_some((now, index));
            double
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn k(code: KeyCode, mods: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, mods)
    }

    #[test]
    fn nav_keys_move_one_row_a_page_or_to_the_ends() {
        let none = KeyModifiers::NONE;
        let ctrl = KeyModifiers::CONTROL;
        assert_eq!(nav(&k(KeyCode::Down, none), 2, 10, 5), Some(3));
        // No `Ctrl+letter` pair moves a list: `Ctrl+N` / `Ctrl+P` are the
        // navigation prefix and the palette, the letters belong to the filter.
        assert_eq!(nav(&k(KeyCode::Char('n'), ctrl), 2, 10, 5), None);
        assert_eq!(nav(&k(KeyCode::Char('p'), ctrl), 2, 10, 5), None);
        assert_eq!(nav(&k(KeyCode::Char('j'), ctrl), 2, 10, 5), None);
        assert_eq!(nav(&k(KeyCode::Up, none), 2, 10, 5), Some(1));
        assert_eq!(nav(&k(KeyCode::Char('k'), ctrl), 2, 10, 5), None);
        assert_eq!(nav(&k(KeyCode::Home, none), 7, 10, 5), Some(0));
        assert_eq!(nav(&k(KeyCode::End, none), 2, 10, 5), Some(9));
        assert_eq!(nav(&k(KeyCode::PageDown, none), 2, 10, 5), Some(6));
        assert_eq!(nav(&k(KeyCode::PageUp, none), 6, 10, 5), Some(2));
    }

    #[test]
    fn nav_clamps_and_ignores_other_keys() {
        let none = KeyModifiers::NONE;
        assert_eq!(nav(&k(KeyCode::Up, none), 0, 10, 5), Some(0));
        assert_eq!(nav(&k(KeyCode::Down, none), 9, 10, 5), Some(9));
        assert_eq!(nav(&k(KeyCode::PageDown, none), 8, 10, 5), Some(9));
        assert_eq!(nav(&k(KeyCode::Down, none), 0, 0, 5), Some(0), "empty list");
        assert_eq!(
            nav(&k(KeyCode::Char('j'), none), 0, 10, 5),
            None,
            "plain j types"
        );
        assert_eq!(
            nav(&k(KeyCode::Char('a'), KeyModifiers::CONTROL), 0, 10, 5),
            None
        );
        assert_eq!(nav(&k(KeyCode::Enter, none), 0, 10, 5), None);
    }

    #[test]
    fn the_window_follows_the_highlight_until_the_wheel_scrolls_it() {
        View::begin_frame();
        assert_eq!(View::window_start(0, 5, 20), 0);
        assert_eq!(View::window_start(7, 5, 20), 3);
        View::wheel(3, 7);
        assert_eq!(
            View::window_start(7, 5, 20),
            6,
            "wheel moved the window only"
        );
        assert_eq!(View::window_start(8, 5, 20), 4, "a key move follows again");
        View::wheel(100, 8);
        assert_eq!(View::window_start(8, 5, 20), 15, "clamped to the end");
    }

    #[test]
    fn a_click_pair_survives_the_frame_reset_between_its_clicks() {
        let mut pair = ClickPair::default();
        assert!(!pair.register(2));
        View::begin_frame();
        View::begin_frame();
        assert!(pair.register(2), "a frame in between must not erase it");
        assert!(!pair.register(2), "the pair is consumed");
    }

    #[test]
    fn a_second_click_on_the_same_row_validates() {
        View::begin_frame();
        assert!(!View::click(3));
        assert!(View::click(3));
        assert!(!View::click(3), "the pair is consumed");
        assert!(!View::click(4));
        assert!(!View::click(3));
    }
}
