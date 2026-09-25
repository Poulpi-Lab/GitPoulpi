//! Vertical scrolling of the profile view (the part under the profile
//! strip).
//!
//! The view is painted whole into an off-screen buffer as tall as any card
//! can get, anchored where it would sit unscrolled; the visible window of
//! it is then copied to the frame, the hit rects are shifted up by the
//! scroll and clipped to the window, and the scrollbar of the other Settings
//! pages (`crate::scrollbar`) is drawn in the right padding column - only
//! when the content is taller than the window. So every control keeps one
//! layout, scrolled or not, and nothing outside the window is clickable.
//!
//! The editor's field rects stay in the off-screen coordinates (the editor
//! only needs columns and, for a double click, rows): mouse events for it
//! are shifted down by the scroll instead.

use super::*;
use crate::scrollbar::{self, BarDrag, Track};
use ratatui::buffer::{Buffer, Cell};

/// Rows of the off-screen buffer: far more than the tallest card.
const VIRTUAL_ROWS: u16 = 160;
/// Rows a wheel notch scrolls.
const WHEEL_STEP: usize = 3;

/// Where the page paints: a buffer (the frame's, or the off-screen one).
pub(super) struct Canvas<'a> {
    buf: &'a mut Buffer,
}

impl<'a> Canvas<'a> {
    pub(super) fn new(buf: &'a mut Buffer) -> Self {
        Self { buf }
    }

    pub(super) fn buffer_mut(&mut self) -> &mut Buffer {
        self.buf
    }
}

/// What identifies the view being scrolled: another one starts at the top.
type ViewKey = (usize, usize, View);

/// The scroll state of the profile view.
#[derive(Default)]
pub(super) struct Scroll {
    /// First visible row of the view.
    pub(super) top: usize,
    /// Rows of content, as of the last frame.
    pub(super) content: usize,
    /// The window the view is shown in, as of the last frame.
    pub(super) window: Rect,
    /// Column where the right pane starts (the wheel is the pane's).
    pub(super) pane_x: u16,
    key: Option<ViewKey>,
    /// The control whose visibility was last ensured: a wheel scroll is not
    /// undone on the next frame, only a focus move scrolls.
    followed: Option<Item>,
    /// The Radar block's heading row, off-screen coordinates.
    pub(super) radar_top: Option<u16>,
    /// The combobox field, off-screen coordinates.
    pub(super) radar_field: Option<Rect>,
    /// The palette asked for the Radar block: show it next frame.
    pub(super) want_radar: bool,
    pub(super) bar: Option<Track>,
    pub(super) drag: BarDrag,
}

impl Scroll {
    fn max(&self) -> usize {
        self.content.saturating_sub(self.window.height as usize)
    }

    /// Brings rows `from..to` (relative to the window's top row in the
    /// unscrolled view) into the window with a line of context when there
    /// is room: the least scrolling that does it.
    fn reveal(&mut self, from: usize, to: usize) {
        let view = self.window.height as usize;
        let ctx = usize::from(view >= 4);
        if from < self.top + ctx {
            self.top = from.saturating_sub(ctx);
        } else if to + ctx > self.top + view {
            self.top = (to + ctx)
                .saturating_sub(view)
                .min(from.saturating_sub(ctx));
        }
        self.top = self.top.min(self.max());
    }
}

impl IntegrationsPage {
    /// Whether a press/drag/wheel is the page's even outside the body: a
    /// popup is open or the scrollbar thumb is held.
    pub(crate) fn owns_pointer(&self) -> bool {
        self.combo_open() || self.scroll.drag.held()
    }

    /// Scrolls the view by `delta` rows (clamped).
    fn scroll_by(&mut self, delta: isize) {
        let max = self.scroll.max();
        self.scroll.top = (self.scroll.top as isize + delta).clamp(0, max as isize) as usize;
    }

    /// Wheel and scrollbar events. Whether the page used the event.
    pub(super) fn scroll_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let event = crossterm::event::MouseEvent {
            kind,
            column: col,
            row,
            modifiers: KeyModifiers::NONE,
        };
        let (bar, content, top) = (self.scroll.bar, self.scroll.content, self.scroll.top);
        if let Some(top) = self.scroll.drag.on_mouse(bar, event, content, top) {
            self.scroll.top = top.min(self.scroll.max());
            return true;
        }
        if self.scroll.max() == 0 || col < self.scroll.pane_x {
            return false;
        }
        match kind {
            MouseEventKind::ScrollUp => self.scroll_by(-(WHEEL_STEP as isize)),
            MouseEventKind::ScrollDown => self.scroll_by(WHEEL_STEP as isize),
            _ => return false,
        }
        true
    }

    /// A row of the screen as the editor's off-screen rects know it.
    pub(super) fn editor_row(&self, row: u16) -> u16 {
        row.saturating_add(self.scroll.top as u16)
    }

    /// Paints the view of the selected profile in `window`, scrolled.
    pub(super) fn render_scrolled(
        &mut self,
        canvas: &mut Canvas,
        window: Rect,
        cfg: &Config,
        presence: &HashMap<String, Option<bool>>,
        focused: bool,
        current: Option<Item>,
    ) {
        self.scroll.window = window;
        self.scroll.pane_x = window.x;
        self.scroll.radar_top = None;
        self.scroll.radar_field = None;
        if window.height == 0 || window.width == 0 {
            return;
        }
        let key = (self.kind, self.sel, self.view(cfg));
        if self.scroll.key != Some(key) {
            self.scroll.key = Some(key);
            self.scroll.top = 0;
            self.scroll.followed = None;
        }

        let page = Rect::new(
            window.x,
            window.y,
            window.width,
            VIRTUAL_ROWS.max(window.height),
        );
        let mut buf = Buffer::empty(page);
        let mark = self.hits.len();
        {
            let mut off = Canvas::new(&mut buf);
            self.render_view(&mut off, page, cfg, presence, focused, current);
        }
        self.scroll.content = (page.y..page.bottom())
            .rev()
            .find(|y| (page.x..page.right()).any(|x| buf[(x, *y)] != Cell::default()))
            .map_or(0, |y| (y - page.y) as usize + 1);

        // Keep the focused control (or the block the palette asked for) in
        // sight; only a focus move scrolls, never a repaint.
        let max = self.scroll.max();
        self.scroll.top = self.scroll.top.min(max);
        if focused && self.zone == Zone::Body {
            let rect = current.and_then(|c| {
                self.hits[mark..]
                    .iter()
                    .find(|(_, h, _)| *h == Hit::Item(c))
                    .map(|(r, _, _)| *r)
            });
            if std::mem::take(&mut self.scroll.want_radar) {
                if let Some(top) = self.scroll.radar_top {
                    let from = (top - page.y) as usize;
                    let to = self.scroll.content.min(from + window.height as usize);
                    self.scroll.reveal(from, to);
                    self.scroll.followed = current;
                }
            } else if self.scroll.followed != current {
                if let Some(r) = rect {
                    self.scroll
                        .reveal((r.y - page.y) as usize, (r.bottom() - page.y) as usize);
                }
                self.scroll.followed = current;
            }
        } else {
            self.scroll.followed = None;
        }
        let top = self.scroll.top;

        // The visible window of the off-screen page onto the frame.
        let dst = canvas.buffer_mut();
        for dy in 0..window.height {
            let src_y = page.y + dy + top as u16;
            if src_y >= page.bottom() {
                break;
            }
            for x in window.x..window.right() {
                if let Some(cell) = dst.cell_mut((x, window.y + dy)) {
                    *cell = buf[(x, src_y)].clone();
                }
            }
        }

        // Hit rects: shifted by the scroll, clipped to the window.
        let shifted: Vec<_> = self
            .hits
            .split_off(mark)
            .into_iter()
            .filter_map(|(r, h, hover)| {
                let y0 = (i32::from(r.y) - top as i32).max(i32::from(window.y));
                let y1 = (i32::from(r.bottom()) - top as i32).min(i32::from(window.bottom()));
                (y1 > y0).then(|| {
                    (
                        Rect::new(r.x, y0 as u16, r.width, (y1 - y0) as u16),
                        h,
                        hover,
                    )
                })
            })
            .collect();
        self.hits.extend(shifted);

        // The popup hangs from where its field is drawn now.
        if let Some(field) = self.scroll.radar_field {
            let y = (i32::from(field.y) - top as i32)
                .clamp(i32::from(window.y), i32::from(window.bottom()) - 1);
            self.radar_combo
                .set_anchor(Rect::new(field.x, y as u16, field.width, 1));
        }

        // The scrollbar, in the padding column right of the page; only
        // when something is hidden and the column exists.
        let outer = Rect::new(
            window.right().saturating_sub(1),
            window.y.saturating_sub(1),
            2,
            window.height + 2,
        );
        self.scroll.bar = if dst.area.right() > window.right() {
            Track::vertical(outer, self.scroll.content, top)
        } else {
            None
        };
        if let Some(track) = self.scroll.bar {
            scrollbar::render_buf(dst, track, true, false);
        }
    }
}
