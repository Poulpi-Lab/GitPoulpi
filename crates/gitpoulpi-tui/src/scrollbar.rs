//! Shared vertical/horizontal scrollbar geometry, rendering and hit-testing
//! for every scrollable central view (the diff, and the File Insight
//! overlay's File View/Blame/History): one source of truth so a scrollbar's
//! drawn thumb and its clickable rect never drift apart, as `diff_bar` /
//! `hunk_button` do for buttons (see `diff_view/`).
//!
//! A scrollbar is drawn *over the block's own border*, not inside the
//! content area: the vertical bar takes the right border column (between the
//! top and bottom corner glyphs), the horizontal one the bottom border row
//! (between the left and right corners). The content width/height is
//! therefore never shrunk, the track's length is exactly the block's inner
//! height/width, and `Track::vertical`/`Track::horizontal` take the outer
//! (bordered) rect and derive the track themselves.
//!
//! Cells are written straight into the frame's buffer (`Frame::buffer_mut`)
//! rather than through ratatui's `Scrollbar` widget, which computes its own
//! thumb position internally: a second, independently rounded computation
//! for hit-testing is the "two walks over the same thing" the conventions
//! warn against. `render_gear`/`render_close_app_button` in `chrome/` write
//! cells directly for the same reason.

use crate::th;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::Frame;

use crate::theme::{lighten, plain_button_style};

/// Which scrollable central view (and axis) a scrollbar belongs to --
/// carried by `HitTarget::ScrollbarThumb`/`ScrollbarTrack` so one mouse
/// handler can serve every view instead of each inventing its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ScrollBar {
    DiffVertical,
    DiffHorizontal,
    /// The Log's sideways bar on its bottom border (`scroll` column layout).
    LogHorizontal,
    /// The minimap beside the diff's vertical bar (`minimap.rs`): a track
    /// with no thumb - a click jumps to the line under it.
    DiffMinimap,
    FileInsightVertical,
    FileInsightHorizontal,
    /// The Blame view's minimap, colored by the age heat.
    BlameMinimap,
    /// The commit composer's Description box while it is focused (a
    /// `ui_kit::ListBar` drag).
    CommitDescription,
}

/// A scrollbar's track plus its thumb's position within it, both already
/// resolved for the current frame - the single computation `render` draws
/// from and the mouse handlers hit-test against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Track {
    /// The 1-cell-thick strip the bar occupies (a column for a vertical
    /// bar, a row for a horizontal one).
    pub(crate) rect: Rect,
    /// The track's own length along the scroll axis - `rect.height` for a
    /// vertical bar, `rect.width` for a horizontal one, kept alongside
    /// `rect` so callers never have to remember which field means which.
    pub(crate) track_len: u16,
    /// The thumb's offset and length within the track, in cells from the
    /// track's own start.
    pub(crate) thumb_start: u16,
    pub(crate) thumb_len: u16,
    /// 1 when the strip's first and last cells are the clickable end
    /// arrows (up/down, left/right), 0 for a strip too short for them;
    /// the thumb then lives in `arrow..track_len - arrow`.
    pub(crate) arrow: u16,
}

/// Strips shorter than this hide their end arrows.
const MIN_LEN_FOR_ARROWS: u16 = 3;

fn arrow_cells(track_len: u16) -> u16 {
    u16::from(track_len >= MIN_LEN_FOR_ARROWS)
}

/// Thumb `(start, len)` along a track of `track_len` cells, or `None` when
/// the content already fits the viewport - nothing to scroll, so no
/// thumb (and no scrollbar at all) is drawn.
fn thumb_geometry(track_len: u16, content_len: usize, viewport: usize) -> Option<(u16, u16)> {
    if track_len == 0 || viewport == 0 || content_len <= viewport {
        return None;
    }
    let track_len = track_len as usize;
    let len = ((viewport * track_len) / content_len).clamp(1, track_len);
    Some((0, len as u16))
}

fn thumb_start(
    track_len: u16,
    thumb_len: u16,
    content_len: usize,
    viewport: usize,
    offset: usize,
) -> u16 {
    let max_offset = content_len.saturating_sub(viewport);
    let max_start = track_len.saturating_sub(thumb_len);
    if max_offset == 0 || max_start == 0 {
        return 0;
    }
    (((offset.min(max_offset)) as u64 * max_start as u64) / max_offset as u64) as u16
}

/// The vertical bar's track: the right border column, exclusive of the
/// block's own top/bottom corner glyphs.
fn v_track_rect(body: Rect) -> Option<Rect> {
    if body.width < 2 || body.height < 3 {
        return None;
    }
    Some(Rect::new(
        body.x + body.width - 1,
        body.y + 1,
        1,
        body.height - 2,
    ))
}

/// The horizontal bar's track: the bottom border row, exclusive of the
/// block's own left/right corner glyphs.
fn h_track_rect(body: Rect) -> Option<Rect> {
    if body.width < 3 || body.height < 2 {
        return None;
    }
    Some(Rect::new(
        body.x + 1,
        body.y + body.height - 1,
        body.width - 2,
        1,
    ))
}

/// What [`Track::press`] decided.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TrackPress {
    /// An end arrow: scroll this many lines (the arrow repeat replays it).
    Arrow(isize),
    /// Hold the thumb at `grab`; with `jump` the content must first move so
    /// the thumb sits under the pointer (`drag_offset` with this grab).
    Grab { grab: i32, jump: bool },
}

impl Track {
    /// `body` is the scrollable view's **outer** (bordered) rect - the
    /// same "outer, not inner" convention `diff_row_at` already documents,
    /// because the track lives in the border, not the content.
    pub(crate) fn vertical(body: Rect, content_len: usize, offset: usize) -> Option<Self> {
        Self::vertical_with(body, content_len, offset)
    }

    fn vertical_with(body: Rect, content_len: usize, offset: usize) -> Option<Self> {
        let rect = v_track_rect(body)?;
        Self::column(rect, content_len, rect.height as usize, offset)
    }

    /// A vertical bar on an explicit one-column `rect` (a list's rows, not a
    /// whole bordered block), for a viewport of `viewport` content lines -
    /// usually `rect.height`, fewer when a row can be taller than a line.
    /// `None` while everything fits.
    pub(crate) fn column(
        rect: Rect,
        content_len: usize,
        viewport: usize,
        offset: usize,
    ) -> Option<Self> {
        if rect.width == 0 || rect.height == 0 {
            return None;
        }
        let arrow = arrow_cells(rect.height);
        let inner = rect.height - 2 * arrow;
        let (_, thumb_len) = thumb_geometry(inner, content_len, viewport)?;
        let start = arrow + thumb_start(inner, thumb_len, content_len, viewport, offset);
        Some(Track {
            rect,
            track_len: rect.height,
            thumb_start: start,
            thumb_len,
            arrow,
        })
    }

    pub(crate) fn horizontal(body: Rect, content_len: usize, offset: usize) -> Option<Self> {
        let rect = h_track_rect(body)?;
        let viewport = rect.width as usize;
        let arrow = arrow_cells(rect.width);
        let inner = rect.width - 2 * arrow;
        let (_, thumb_len) = thumb_geometry(inner, content_len, viewport)?;
        let start = arrow + thumb_start(inner, thumb_len, content_len, viewport, offset);
        Some(Track {
            rect,
            track_len: rect.width,
            thumb_start: start,
            thumb_len,
            arrow,
        })
    }

    /// Whether `pos` (a row for a vertical bar, a column for a horizontal
    /// one, always relative to `self.rect`'s own start) lands on the
    /// thumb.
    pub(crate) fn on_thumb(&self, pos: u16) -> bool {
        pos >= self.thumb_start && pos < self.thumb_start + self.thumb_len
    }

    /// The thumb's room: the track without its two end arrows.
    pub(crate) fn inner_len(&self) -> u16 {
        self.track_len - 2 * self.arrow
    }

    /// `Some(-1)` on the start arrow, `Some(1)` on the end arrow (`pos`
    /// track-relative), `None` anywhere else or when arrows are hidden.
    pub(crate) fn arrow_step(&self, pos: u16) -> Option<isize> {
        if self.arrow == 0 {
            None
        } else if pos == 0 {
            Some(-1)
        } else if pos + 1 == self.track_len {
            Some(1)
        } else {
            None
        }
    }

    /// What a mouse-down at `pos` (track-relative) starts, the one rule
    /// every bar shares: an end arrow steps, the thumb is grabbed where it
    /// is held, the bare track jumps the thumb under the pointer (centered,
    /// clamped by [`Self::drag_offset`]) and the press keeps dragging.
    pub(crate) fn press(&self, pos: u16) -> TrackPress {
        if let Some(step) = self.arrow_step(pos) {
            TrackPress::Arrow(step)
        } else if self.on_thumb(pos) {
            TrackPress::Grab {
                grab: i32::from(pos) - i32::from(self.thumb_start),
                jump: false,
            }
        } else {
            TrackPress::Grab {
                grab: i32::from(self.thumb_len / 2),
                jump: true,
            }
        }
    }

    /// New scroll offset while dragging the thumb. `grab` is the
    /// track-relative row/col where the thumb was first grabbed *within
    /// the thumb itself* (so the thumb doesn't jump to be centered under
    /// the pointer the instant the drag starts); `pos` is the current
    /// mouse row/col, track-relative, and may run negative or past the
    /// track's end if the drag leaves it - both are clamped.
    pub(crate) fn drag_offset(
        &self,
        content_len: usize,
        viewport: usize,
        grab: i32,
        pos: i32,
    ) -> usize {
        let max_offset = content_len.saturating_sub(viewport);
        let inner = self.track_len - 2 * self.arrow;
        let max_start = inner.saturating_sub(self.thumb_len);
        if max_offset == 0 || max_start == 0 {
            return 0;
        }
        let target = (pos - self.arrow as i32 - grab).clamp(0, max_start as i32) as u64;
        ((target * max_offset as u64) / max_start as u64) as usize
    }
}

/// The mouse half of a text field's scrollbar (the dialogs' Description
/// boxes): thumb grab and drag (even off the track), a click on the track
/// jumping the thumb under the pointer, end arrows stepping one line --
/// the same gestures `ui_kit::list_bar_press` gives the lists, kept here
/// so each dialog is one call, not its own copy.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(feature = "hosting"), allow(dead_code))]
pub(crate) struct BarDrag {
    grab: Option<i32>,
}

#[cfg_attr(not(feature = "hosting"), allow(dead_code))]
impl BarDrag {
    /// Whether the thumb is held (a drag in progress).
    pub(crate) fn held(&self) -> bool {
        self.grab.is_some()
    }

    /// Feeds one mouse event. `track` is the bar as last painted (`None`
    /// when it is not shown), `content` the line count and `top` the
    /// current first visible line. `Some(new_top)` means the bar consumed
    /// the event and the field must scroll to `new_top`; a release also
    /// ends the grab but is never consumed.
    pub(crate) fn on_mouse(
        &mut self,
        track: Option<Track>,
        mouse: crossterm::event::MouseEvent,
        content: usize,
        top: usize,
    ) -> Option<usize> {
        use crossterm::event::{MouseButton, MouseEventKind};
        let Some(track) = track else {
            self.grab = None;
            return None;
        };
        let rows = track.rect.height as usize;
        let max = content.saturating_sub(rows);
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left)
                if track.rect.contains((mouse.column, mouse.row).into()) =>
            {
                let pos = mouse.row - track.rect.y;
                if let Some(step) = track.arrow_step(pos) {
                    self.grab = None;
                    return Some((top as isize + step).clamp(0, max as isize) as usize);
                }
                if track.on_thumb(pos) {
                    self.grab = Some(i32::from(pos) - i32::from(track.thumb_start));
                    return Some(top);
                }
                self.grab = Some(i32::from(track.thumb_len / 2));
                self.drag_to(track, content, mouse.row)
            }
            MouseEventKind::Drag(MouseButton::Left) if self.grab.is_some() => {
                self.drag_to(track, content, mouse.row)
            }
            MouseEventKind::Up(MouseButton::Left) => {
                self.grab = None;
                None
            }
            _ => None,
        }
    }

    fn drag_to(&self, track: Track, content: usize, row: u16) -> Option<usize> {
        let grab = self.grab?;
        let rows = track.rect.height as usize;
        let pos = i32::from(row) - i32::from(track.rect.y);
        Some(track.drag_offset(content, rows, grab, pos))
    }
}

/// Draws one scrollbar from an already-resolved `Track`, straight into the
/// frame's buffer - see the module doc for why this bypasses ratatui's
/// own `Scrollbar` widget.
pub(crate) fn render(frame: &mut Frame, track: Track, vertical: bool, hovered: bool) {
    render_buf(frame.buffer_mut(), track, vertical, hovered);
}

thread_local! {
    /// Where the pointer rests, set by the event loop on every `Moved`
    /// (`None` while a popup or modal owns it). Scrollbars read it at paint
    /// time, so every bar - hit-target backed or `ListBar` region - lights
    /// its end arrows, thumb and track without each caller threading a hover
    /// flag through.
    static POINTER: std::cell::Cell<Option<(u16, u16)>> = const { std::cell::Cell::new(None) };
}

thread_local! {
    /// The end-arrow cells painted this frame (cleared by [`begin_frame`]):
    /// what a press-and-hold may auto-repeat on, whichever handler owns the
    /// bar's clicks.
    static ARROW_CELLS: std::cell::RefCell<Vec<(u16, u16)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

thread_local! {
    /// Arrow buttons outside a scrollbar that repeat like its end arrows
    /// while held (the tab bar's scroll arrows): each button's rect and the
    /// glyph painted in its middle cell (cleared by [`begin_frame`]).
    static REPEAT_BUTTONS: std::cell::RefCell<Vec<(ratatui::layout::Rect, &'static str)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Forgets last frame's arrow cells; called as a frame starts.
pub(crate) fn begin_frame() {
    ARROW_CELLS.with(|c| c.borrow_mut().clear());
    REPEAT_BUTTONS.with(|c| c.borrow_mut().clear());
}

/// Registers an arrow button (painted this frame, `glyph` in its middle
/// cell) whose held press auto-repeats like a scrollbar arrow.
pub(crate) fn note_repeat_button(rect: ratatui::layout::Rect, glyph: &'static str) {
    REPEAT_BUTTONS.with(|c| c.borrow_mut().push((rect, glyph)));
}

/// Whether a scrollbar end arrow (or a repeating arrow button) was painted
/// at this cell.
pub(crate) fn is_arrow_cell(col: u16, row: u16) -> bool {
    ARROW_CELLS.with(|c| c.borrow().contains(&(col, row)))
        || REPEAT_BUTTONS.with(|c| {
            c.borrow()
                .iter()
                .any(|(rect, _)| rect.contains((col, row).into()))
        })
}

/// Keeps only the arrow cells still showing an arrow once the whole frame
/// is painted: a popup drawn over a bar hides its arrows, and a held press
/// on the popup must not be replayed as an arrow repeat.
pub(crate) fn retain_visible_arrows(buf: &ratatui::buffer::Buffer) {
    ARROW_CELLS.with(|c| {
        c.borrow_mut().retain(|&(col, row)| {
            buf.cell((col, row)).is_some_and(|cell| {
                matches!(
                    cell.symbol(),
                    "\u{2191}" | "\u{2193}" | "\u{2190}" | "\u{2192}"
                )
            })
        })
    });
    REPEAT_BUTTONS.with(|c| {
        c.borrow_mut().retain(|(rect, glyph)| {
            buf.cell((rect.x + rect.width / 2, rect.y))
                .is_some_and(|cell| cell.symbol() == *glyph)
        })
    });
}

fn note_arrow(col: u16, row: u16) {
    ARROW_CELLS.with(|c| c.borrow_mut().push((col, row)));
}

/// Records the pointer cell scrollbars hover against (`None` clears it).
pub(crate) fn set_pointer(pos: Option<(u16, u16)>) {
    POINTER.with(|p| p.set(pos));
}

/// Runs `f` with the hover pointer set to `pos`, then puts the previous one
/// back: a popup lights its own bars from its own pointer without lighting
/// the bars painted behind it.
pub(crate) fn with_pointer<R>(pos: Option<(u16, u16)>, f: impl FnOnce() -> R) -> R {
    let prev = POINTER.with(|p| p.replace(pos));
    let out = f();
    POINTER.with(|p| p.set(prev));
    out
}

/// The track-relative cell of the pointer when it is over `track`.
fn pointer_pos(track: &Track, vertical: bool) -> Option<u16> {
    let (x, y) = POINTER.with(|p| p.get())?;
    if !track.rect.contains((x, y).into()) {
        return None;
    }
    Some(if vertical {
        y - track.rect.y
    } else {
        x - track.rect.x
    })
}

/// [`render`] on a bare buffer (popups painted without a `Frame`).
pub(crate) fn render_buf(
    buf: &mut ratatui::buffer::Buffer,
    track: Track,
    vertical: bool,
    hovered: bool,
) {
    let frame_area = buf.area;
    crate::pointer_map::hand(track.rect);
    let pos = pointer_pos(&track, vertical);
    let thumb_lit = hovered || pos.is_some_and(|p| track.on_thumb(p));
    let thumb_style = if thumb_lit {
        Style::default().bg(th().ui.accent).fg(th().ui.on_accent)
    } else {
        Style::default().bg(th().scrollbar.thumb).fg(th().ui.text)
    };
    let track_bg = th().scrollbar.track;
    let track_style = Style::default().bg(track_bg).fg(th().ui.muted);
    // The gutter lights lightly while the pointer is anywhere on the bar.
    let track_lit_style = Style::default().bg(lighten(track_bg, 14)).fg(th().ui.text);
    let cell_style = |i: u16| {
        if track.arrow_step(i).is_some() {
            if pos == Some(i) {
                plain_button_style(true)
            } else if pos.is_some() {
                track_lit_style
            } else {
                track_style
            }
        } else if track.on_thumb(i) {
            thumb_style
        } else if pos.is_some() {
            track_lit_style
        } else {
            track_style
        }
    };
    if vertical {
        for y in 0..track.rect.height {
            let row = track.rect.y + y;
            if row >= frame_area.y + frame_area.height {
                continue;
            }
            let glyph = match track.arrow_step(y) {
                Some(-1) => "\u{2191}",
                Some(_) => "\u{2193}",
                None => " ",
            };
            if track.arrow_step(y).is_some() {
                note_arrow(track.rect.x, row);
            }
            buf[(track.rect.x, row)]
                .set_symbol(glyph)
                .set_style(cell_style(y));
        }
    } else {
        for x in 0..track.rect.width {
            let col = track.rect.x + x;
            if col >= frame_area.x + frame_area.width {
                continue;
            }
            let glyph = match track.arrow_step(x) {
                Some(-1) => "\u{2190}",
                Some(_) => "\u{2192}",
                None => " ",
            };
            if track.arrow_step(x).is_some() {
                note_arrow(col, track.rect.y);
            }
            buf[(col, track.rect.y)]
                .set_symbol(glyph)
                .set_style(cell_style(x));
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn an_arrow_covered_by_a_popup_is_not_an_arrow_any_more() {
        use ratatui::buffer::Buffer;
        use ratatui::layout::Rect;
        begin_frame();
        let mut buf = Buffer::empty(Rect::new(0, 0, 4, 4));
        buf[(3, 0)].set_symbol("\u{2191}");
        buf[(3, 3)].set_symbol("\u{2193}");
        note_arrow(3, 0);
        note_arrow(3, 3);
        // A popup painted over the bottom arrow afterwards.
        buf[(3, 3)].set_symbol("x");
        retain_visible_arrows(&buf);
        assert!(is_arrow_cell(3, 0));
        assert!(!is_arrow_cell(3, 3));
    }

    use super::*;

    #[test]
    fn no_track_when_the_content_already_fits() {
        let body = Rect::new(0, 0, 20, 10);
        assert_eq!(Track::vertical(body, 5, 0), None);
        assert_eq!(Track::horizontal(body, 10, 0), None);
    }

    #[test]
    fn the_thumb_shrinks_as_content_grows_and_covers_the_whole_track_at_the_limit() {
        let body = Rect::new(0, 0, 20, 12); // vertical track height = 10
        let track = Track::vertical(body, 20, 0).expect("more content than viewport");
        assert_eq!(track.track_len, 10);
        assert_eq!(track.inner_len(), 8); // two cells go to the arrows
        assert_eq!(track.thumb_len, 4); // 10/20 of the 8-cell inner track
        assert!(track.thumb_len < track.track_len);

        // A much longer file -> a thumb still at least 1 cell, never 0.
        let track = Track::vertical(body, 10_000, 0).unwrap();
        assert_eq!(track.thumb_len, 1);
    }

    #[test]
    fn the_thumb_tracks_the_offset_from_top_to_bottom() {
        let body = Rect::new(0, 0, 20, 12); // track height 10, viewport 10
        let content_len = 30; // max_offset = 20
        let at_top = Track::vertical(body, content_len, 0).unwrap();
        assert_eq!(at_top.thumb_start, 1, "just under the up arrow");

        let at_bottom = Track::vertical(body, content_len, 20).unwrap();
        assert_eq!(
            at_bottom.thumb_start + at_bottom.thumb_len,
            at_bottom.track_len - 1,
            "scrolled to the end, the thumb must reach the down arrow"
        );

        let midway = Track::vertical(body, content_len, 10).unwrap();
        assert!(midway.thumb_start > at_top.thumb_start);
        assert!(midway.thumb_start < at_bottom.thumb_start);
    }

    #[test]
    fn pressing_the_bare_track_jumps_the_thumb_under_the_pointer() {
        let body = Rect::new(0, 0, 20, 12);
        let track = Track::vertical(body, 100, 0).unwrap();
        let TrackPress::Grab { grab, jump } = track.press(track.track_len - 2) else {
            panic!("a track press grabs");
        };
        assert!(jump);
        assert_eq!(grab, i32::from(track.thumb_len / 2), "centered");
        let off = track.drag_offset(100, 10, grab, i32::from(track.track_len - 2));
        assert!(
            off > 80,
            "far down the track lands far down the content: {off}"
        );
        assert_eq!(
            track.press(track.thumb_start),
            TrackPress::Grab {
                grab: 0,
                jump: false
            },
            "on the thumb: grabbed where held, no jump"
        );
        // Above the thumb jumps back, clamped at 0.
        let down = Track::vertical(body, 100, 90).unwrap();
        let TrackPress::Grab { grab, .. } = down.press(1) else {
            panic!()
        };
        assert_eq!(down.drag_offset(100, 10, grab, 1), 0);
    }

    #[test]
    fn the_end_arrows_step_one_row_and_short_strips_hide_them() {
        let body = Rect::new(0, 0, 20, 12);
        let track = Track::vertical(body, 30, 10).unwrap();
        assert_eq!(track.arrow_step(0), Some(-1));
        assert_eq!(track.arrow_step(track.track_len - 1), Some(1));
        assert_eq!(track.arrow_step(3), None);
        assert!(!track.on_thumb(0) && !track.on_thumb(track.track_len - 1));
        assert_eq!(track.press(0), TrackPress::Arrow(-1));
        assert_eq!(track.press(track.track_len - 1), TrackPress::Arrow(1));
        // A 2-cell strip (a 4-row block) has no room for arrows.
        let short = Track::vertical(Rect::new(0, 0, 20, 4), 30, 0).unwrap();
        assert_eq!((short.arrow, short.arrow_step(0)), (0, None));
        assert_eq!(short.thumb_start, 0);
        // Horizontal bars use the same rule.
        let h = Track::horizontal(Rect::new(0, 0, 40, 10), 100, 0).unwrap();
        assert_eq!(
            (h.arrow_step(0), h.arrow_step(h.track_len - 1)),
            (Some(-1), Some(1))
        );
    }

    #[test]
    fn arrows_are_drawn_at_both_ends() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;
        let mut terminal = Terminal::new(TestBackend::new(20, 12)).unwrap();
        let track = Track::vertical(Rect::new(0, 0, 20, 12), 30, 0).unwrap();
        terminal.draw(|f| render(f, track, true, false)).unwrap();
        let buf = terminal.backend().buffer();
        assert_eq!(buf[(track.rect.x, track.rect.y)].symbol(), "\u{2191}");
        assert_eq!(
            buf[(track.rect.x, track.rect.bottom() - 1)].symbol(),
            "\u{2193}"
        );
    }

    #[test]
    fn dragging_the_thumb_from_top_to_bottom_reaches_both_ends() {
        let body = Rect::new(0, 0, 20, 12);
        let content_len = 100;
        let viewport = 10;
        let track = Track::vertical(body, content_len, 0).unwrap();

        // Grabbed at the thumb's own first row (grab = 0).
        assert_eq!(track.drag_offset(content_len, viewport, 0, 0), 0);
        let bottom = track.drag_offset(content_len, viewport, 0, track.track_len as i32);
        assert_eq!(bottom, content_len - viewport, "clamped to the max offset");

        // A drag that leaves the track entirely (negative or past the end)
        // still clamps rather than panicking or wrapping.
        assert_eq!(track.drag_offset(content_len, viewport, 0, -50), 0);
        assert_eq!(
            track.drag_offset(content_len, viewport, 0, 500),
            content_len - viewport
        );
    }

    /// The bug this guards: computing `thumb_start` from the *unclamped*
    /// track_len - thumb_len when thumb_len == track_len (content barely
    /// over the viewport) would underflow a `u16` subtraction and panic in
    /// debug builds.
    #[test]
    fn a_thumb_as_long_as_the_track_never_panics_computing_its_start() {
        let body = Rect::new(0, 0, 20, 12); // track height 10
        let track = Track::vertical(body, 11, 5).unwrap(); // barely scrollable
        assert!(track.thumb_len <= track.track_len);
        assert!(track.thumb_start + track.thumb_len <= track.track_len);
    }

    #[test]
    fn horizontal_track_uses_width_not_height() {
        let body = Rect::new(0, 0, 40, 10);
        let track = Track::horizontal(body, 100, 0).unwrap();
        assert_eq!(track.track_len, 38); // width - 2 border columns (arrows inside)
        assert_eq!(track.rect.height, 1);
    }

    #[test]
    fn rendering_paints_the_thumb_cells_differently_from_the_track_and_lights_up_on_hover() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;

        let mut terminal = Terminal::new(TestBackend::new(20, 12)).unwrap();
        let body = Rect::new(0, 0, 20, 12);
        let track = Track::vertical(body, 30, 0).unwrap();

        terminal
            .draw(|frame| render(frame, track, true, false))
            .unwrap();
        let unhovered_bg = terminal.backend().buffer()[(track.rect.x, track.rect.y + 1)]
            .style()
            .bg;

        terminal
            .draw(|frame| render(frame, track, true, true))
            .unwrap();
        let hovered_bg = terminal.backend().buffer()[(track.rect.x, track.rect.y + 1)]
            .style()
            .bg;

        assert_ne!(
            unhovered_bg, hovered_bg,
            "hovering the thumb must change its rendered style"
        );

        // A cell below the thumb (still on the track) must differ from
        // the thumb's own color.
        let track_bg = terminal.backend().buffer()[(track.rect.x, track.rect.bottom() - 2)]
            .style()
            .bg;
        assert_ne!(track_bg, hovered_bg);
    }

    fn cell(term: &ratatui::Terminal<ratatui::backend::TestBackend>, x: u16, y: u16) -> Style {
        term.backend().buffer()[(x, y)].style()
    }

    /// The pointer over an end arrow lights that arrow alone; over the
    /// gutter it lights the track lightly; away from the bar nothing changes.
    #[test]
    fn the_end_arrows_and_the_track_light_on_pointer_hover() {
        use ratatui::backend::TestBackend;
        let body = Rect::new(0, 0, 20, 12);
        let track = Track::vertical(body, 100, 0).unwrap();
        let draw = |ptr: Option<(u16, u16)>| {
            set_pointer(ptr);
            let mut term = ratatui::Terminal::new(TestBackend::new(20, 12)).unwrap();
            term.draw(|f| render(f, track, true, false)).unwrap();
            set_pointer(None);
            term
        };
        let x = track.rect.x;
        let rest = draw(None);
        let last = track.rect.y + track.track_len - 1;
        let on_up = draw(Some((x, track.rect.y)));
        assert_ne!(cell(&on_up, x, track.rect.y), cell(&rest, x, track.rect.y));
        assert_eq!(
            cell(&on_up, x, last),
            cell(&rest, x, last)
                .bg(lighten(th().scrollbar.track, 14))
                .fg(th().ui.text)
        );
        let on_down = draw(Some((x, last)));
        assert_ne!(cell(&on_down, x, last), cell(&rest, x, last));
        let gutter = track.rect.y + track.track_len - 2;
        let on_track = draw(Some((x, gutter)));
        assert_ne!(cell(&on_track, x, gutter), cell(&rest, x, gutter));
        let on_thumb = draw(Some((x, track.rect.y + track.thumb_start)));
        assert_ne!(
            cell(&on_thumb, x, track.rect.y + track.thumb_start),
            cell(&rest, x, track.rect.y + track.thumb_start)
        );
        let away = draw(Some((3, 3)));
        assert_eq!(away.backend().buffer(), rest.backend().buffer());
    }
}

#[cfg(test)]
mod bar_drag_tests {
    use super::*;
    use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

    fn ev(kind: MouseEventKind, col: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind,
            column: col,
            row,
            modifiers: KeyModifiers::NONE,
        }
    }

    #[test]
    fn a_text_fields_bar_arrows_grabs_drags_off_the_track_and_jumps() {
        // 12 rows tall, so a 10-cell track (up arrow, 8 inner, down arrow).
        let body = Rect::new(0, 0, 20, 12);
        let content = 100;
        let track = Track::vertical(body, content, 0);
        let t = track.unwrap();
        let x = t.rect.x;
        let down = press_at(x, t.rect.y + 1);
        let mut bar = BarDrag::default();
        // The down arrow steps one line, the up arrow clamps at the top.
        let arrow = ev(
            MouseEventKind::Down(MouseButton::Left),
            x,
            t.rect.bottom() - 1,
        );
        assert_eq!(bar.on_mouse(track, arrow, content, 0), Some(1));
        let up = ev(MouseEventKind::Down(MouseButton::Left), x, t.rect.y);
        assert_eq!(bar.on_mouse(track, up, content, 0), Some(0));
        // A press on the thumb moves nothing, a drag far below (and off
        // the column) reaches the end, a release is not consumed.
        assert_eq!(bar.on_mouse(track, down, content, 0), Some(0));
        let drag = ev(MouseEventKind::Drag(MouseButton::Left), 0, 500);
        assert_eq!(
            bar.on_mouse(track, drag, content, 0),
            Some(content - t.rect.height as usize)
        );
        let release = ev(MouseEventKind::Up(MouseButton::Left), 0, 500);
        assert_eq!(bar.on_mouse(track, release, content, 0), None);
        // With nothing held a drag is not ours.
        assert_eq!(bar.on_mouse(track, drag, content, 0), None);
        // A press on the bare track jumps toward it and keeps the grab.
        let jump = ev(
            MouseEventKind::Down(MouseButton::Left),
            x,
            t.rect.bottom() - 3,
        );
        assert!(bar.on_mouse(track, jump, content, 0).unwrap() > 50);
        assert!(bar.on_mouse(track, drag, content, 0).is_some());
        // No bar shown: everything passes through.
        assert_eq!(bar.on_mouse(None, jump, content, 0), None);
    }

    fn press_at(col: u16, row: u16) -> MouseEvent {
        ev(MouseEventKind::Down(MouseButton::Left), col, row)
    }
}
