//! Sideways scrolling of the list panes: the Refs sections, the Unstaged and
//! Staged lists, the commit's changed files (and a comparison's) and the
//! commit-details info lines.
//!
//! One mechanism, the one the Diff view and the Log's `scroll` layout use:
//!
//! - a horizontal wheel (`ScrollLeft`/`ScrollRight`) or Shift+wheel over a pane
//!   that overflows scrolls it sideways, [`STEP`] cells a notch;
//! - a horizontal scrollbar on the pane's bottom border (the same bar, the
//!   same hand pointer as the vertical one: end arrows, a thumb to drag, a
//!   click on the track jumps there) that exists only while the content is
//!   wider than the pane, so it never takes a content row;
//! - Alt+Left / Alt+Right (the `Scroll ... Left/Right` actions) for the pane
//!   that has the keyboard, the palette's `Scroll the Pane Left/Right`.
//!
//! **What scrolls and what stays.** The whole row content scrolls (indent,
//! status marker, chevron and name together); the pane's title, its border,
//! both scrollbars and the Stage/Unstage button a hovered Status row grows at
//! its right end do not. A name is never cut with an ellipsis any more: the
//! tail is one scroll away. The offset is per pane, clamped to the widest row
//! in view, and forgotten when the repository changes.
//!
//! The panes' geometry (where a pane is, how wide its content is, how far it
//! is scrolled) is recorded here while a frame is drawn and read by the mouse
//! and key handlers, the way `scrollbar` keeps its arrow cells: one thread
//! (the UI's), cleared as a frame starts, so a pane that was not drawn is not
//! scrollable.

use crate::cell_text::{cells, slice_cells};
use crate::chrome::HitTarget;
use crate::refs_sidebar::SidebarSection;
use crate::scrollbar::{self, ScrollBar, Track, TrackPress};
use crate::status_pane::StatusSection;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState, StatefulWidget};
use ratatui::Frame;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// Which way a key or palette command scrolls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Dir {
    Left,
    Right,
}

impl HPane {
    /// The registry action that scrolls this pane `dir` from the keyboard.
    pub(crate) fn key(self, dir: Dir) -> crate::keymap::Action {
        use crate::keymap::Action as A;
        match (self, dir) {
            (HPane::Refs(_), Dir::Left) => A::RefsScrollLeft,
            (HPane::Refs(_), Dir::Right) => A::RefsScrollRight,
            (HPane::Status(_), Dir::Left) => A::StatusScrollLeft,
            (HPane::Status(_), Dir::Right) => A::StatusScrollRight,
            (HPane::CommitFiles, Dir::Left) => A::LogFilesScrollLeft,
            (HPane::CommitFiles, Dir::Right) => A::LogFilesScrollRight,
            (HPane::CommitInfo, Dir::Left) => A::LogDetailsScrollLeft,
            (HPane::CommitInfo, Dir::Right) => A::LogDetailsScrollRight,
        }
    }
}

/// Cells one wheel notch or one key press moves a pane.
pub(crate) const STEP: usize = 4;

/// The widest content a pane scrolls over: a name longer than this is cut
/// (the off-screen row buffer has this many columns at most).
const MAX_CONTENT: usize = 2048;

/// A pane that scrolls sideways.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum HPane {
    /// One Refs section (Branches, Remotes, Tags, Stashes, Worktrees, Submodules).
    Refs(SidebarSection),
    /// The Unstaged or Staged list.
    Status(StatusSection),
    /// The commit's (or the comparison's) changed files.
    CommitFiles,
    /// The commit-details info lines: id, author, authored date, counts.
    CommitInfo,
}

#[derive(Debug, Clone, Copy, Default)]
struct Entry {
    /// The pane's outer (bordered) rect: the bar lives on its bottom border.
    area: Rect,
    /// The widest row in view, in cells.
    content: usize,
    offset: usize,
    /// The frame that last drew the pane.
    frame: u64,
}

#[derive(Default)]
struct Store {
    frame: u64,
    repo: u64,
    panes: HashMap<HPane, Entry>,
}

thread_local! {
    static STORE: RefCell<Store> = RefCell::new(Store::default());
}

fn with<R>(f: impl FnOnce(&mut Store) -> R) -> R {
    STORE.with(|s| f(&mut s.borrow_mut()))
}

/// Called as a frame starts: panes not drawn again are no longer scrollable,
/// and a different repository forgets every offset.
pub(crate) fn begin_frame(repo: &std::path::Path) {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    repo.hash(&mut h);
    let repo = h.finish();
    with(|s| {
        s.frame += 1;
        if s.repo != repo {
            s.repo = repo;
            s.panes.clear();
        }
    });
}

/// Forgets every pane (tests start from nothing).
#[cfg(test)]
pub(crate) fn reset_all() {
    with(|s| *s = Store::default());
}

/// A pane's view width: its outer width less the two border columns.
fn view_of(area: Rect) -> usize {
    usize::from(area.width.saturating_sub(2))
}

/// Records that `pane` is drawn in `area` this frame with rows `content`
/// cells wide at most, and returns the offset to draw at: the stored one
/// clamped to what the content allows.
pub(crate) fn claim(pane: HPane, area: Rect, content: usize) -> usize {
    with(|s| {
        let frame = s.frame;
        let e = s.panes.entry(pane).or_default();
        e.area = area;
        e.content = content;
        e.frame = frame;
        e.offset = e.offset.min(content.saturating_sub(view_of(area)));
        e.offset
    })
}

/// The pane's offset as of the last frame (0 when it was not drawn).
pub(crate) fn offset(pane: HPane) -> usize {
    live(pane).map_or(0, |e| e.offset)
}

fn live(pane: HPane) -> Option<Entry> {
    with(|s| s.panes.get(&pane).copied().filter(|e| e.frame == s.frame))
}

/// Whether the pane is drawn and wider than its view.
#[cfg(test)]
pub(crate) fn overflows(pane: HPane) -> bool {
    live(pane).is_some_and(|e| e.content > view_of(e.area))
}

/// Moves a pane by `delta` cells; `true` when it overflows (so the gesture
/// was for it), moved or not.
pub(crate) fn scroll_by(pane: HPane, delta: isize) -> bool {
    with(|s| {
        let frame = s.frame;
        let Some(e) = s.panes.get_mut(&pane).filter(|e| e.frame == frame) else {
            return false;
        };
        let max = e.content.saturating_sub(view_of(e.area));
        if max == 0 {
            return false;
        }
        e.offset = (e.offset as isize + delta).clamp(0, max as isize) as usize;
        true
    })
}

fn set_offset(pane: HPane, to: usize) {
    with(|s| {
        if let Some(e) = s.panes.get_mut(&pane) {
            e.offset = to.min(e.content.saturating_sub(view_of(e.area)));
        }
    });
}

/// The overflowing pane under `(col, row)`, if any.
pub(crate) fn pane_at(col: u16, row: u16) -> Option<HPane> {
    with(|s| {
        let frame = s.frame;
        s.panes
            .iter()
            .filter(|(_, e)| e.frame == frame && e.content > view_of(e.area))
            .find(|(_, e)| e.area.contains((col, row).into()))
            .map(|(p, _)| *p)
    })
}

/// The wheel over `(col, row)`: `true` when an overflowing pane took it.
pub(crate) fn wheel(col: u16, row: u16, notches: isize) -> bool {
    pane_at(col, row).is_some_and(|p| scroll_by(p, notches * STEP as isize))
}

/// The bar's track, while the pane overflows.
pub(crate) fn track(pane: HPane) -> Option<Track> {
    let e = live(pane)?;
    Track::horizontal(e.area, e.content, e.offset)
}

/// Draws the pane's bar on its bottom border and registers its hit targets
/// (the thumb, then the track). `at` is where in `hits` they go: a pane whose
/// bottom border already carries a handle (the Refs sections' resize strip)
/// puts the bar ahead of it.
pub(crate) fn paint_bar(
    frame: &mut Frame,
    pane: HPane,
    hits: &mut Vec<(Rect, HitTarget)>,
    at: Option<usize>,
) {
    let Some(track) = track(pane) else { return };
    scrollbar::render(frame, track, false, false);
    let thumb = Rect::new(
        track.rect.x + track.thumb_start,
        track.rect.y,
        track.thumb_len.max(1),
        track.rect.height,
    );
    let new = [
        (thumb, HitTarget::ScrollbarThumb(ScrollBar::Pane(pane))),
        (track.rect, HitTarget::ScrollbarTrack(ScrollBar::Pane(pane))),
    ];
    let at = at.unwrap_or(hits.len()).min(hits.len());
    for (i, hit) in new.into_iter().enumerate() {
        hits.insert(at + i, hit);
    }
}

/// A press on the thumb: where in it the pointer holds it.
pub(crate) fn thumb_grab(pane: HPane, col: u16) -> Option<i32> {
    track(pane).map(|t| i32::from(col) - i32::from(t.rect.x) - i32::from(t.thumb_start))
}

/// A press on the bare track or an end arrow: steps or jumps now; `Some(grab)`
/// when the press keeps dragging.
pub(crate) fn press_track(pane: HPane, col: u16) -> Option<i32> {
    let t = track(pane)?;
    let e = live(pane)?;
    let pos = (i32::from(col) - i32::from(t.rect.x)).max(0) as u16;
    match t.press(pos) {
        TrackPress::Arrow(step) => {
            scroll_by(pane, step * STEP as isize);
            None
        }
        TrackPress::Grab { grab, .. } => {
            let to = t.drag_offset(e.content, view_of(e.area), grab, i32::from(pos));
            set_offset(pane, to);
            Some(grab)
        }
    }
}

/// The thumb held at `grab` while the pointer is on column `col`.
pub(crate) fn drag_to(pane: HPane, grab: i32, col: u16) {
    let (Some(t), Some(e)) = (track(pane), live(pane)) else {
        return;
    };
    let pos = i32::from(col) - i32::from(t.rect.x);
    set_offset(pane, t.drag_offset(e.content, view_of(e.area), grab, pos));
}

/// `spans` without their first `skip` cells (a wide char cut in half becomes
/// a space).
pub(crate) fn cut_spans<'a>(spans: Vec<Span<'a>>, skip: usize) -> Vec<Span<'a>> {
    if skip == 0 {
        return spans;
    }
    let mut left = skip;
    let mut out = Vec::with_capacity(spans.len());
    for span in spans {
        if left == 0 {
            out.push(span);
            continue;
        }
        let w = cells(&span.content);
        if w <= left {
            left -= w;
            continue;
        }
        let rest = slice_cells(&span.content, left, w - left);
        left = 0;
        out.push(Span::styled(rest, span.style));
    }
    out
}

/// `line` without its first `skip` cells.
pub(crate) fn cut_line(line: Line<'_>, skip: usize) -> Line<'_> {
    if skip == 0 {
        return line;
    }
    Line {
        spans: cut_spans(line.spans, skip),
        ..line
    }
}

/// Renders a list of rows that may be wider than `inner` (the list's own
/// rect, inside its border): the rows as they are when they fit or the pane
/// is not scrolled, else drawn on a buffer as wide as the widest row and
/// copied from the offset, so the selection fill, the pills and every style
/// scroll with the text. `outer` is the pane's bordered rect, `pane` its id.
/// Returns the offset drawn at.
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_list(
    frame: &mut Frame,
    items: Vec<ListItem<'_>>,
    highlight: Style,
    inner: Rect,
    state: &mut ListState,
    pane: HPane,
    outer: Rect,
    pinned: bool,
) -> usize {
    // The widest row in view: the rows the list shows from its offset.
    let content = items
        .iter()
        .skip(state.offset())
        .take(usize::from(inner.height).max(1))
        .map(ListItem::width)
        .max()
        .unwrap_or(0)
        .min(MAX_CONTENT);
    let off = claim(pane, outer, content);
    let off = if pinned { 0 } else { off };
    let inner = inner.intersection(frame.area());
    if off == 0 || inner.is_empty() {
        crate::ui_kit::ClippedRender::render_stateful_clipped(
            frame,
            List::new(items).highlight_style(highlight),
            inner,
            state,
        );
        return 0;
    }
    let wide = Rect::new(0, 0, content.max(view_of(outer)) as u16, inner.height);
    let mut scratch = Buffer::empty(wide);
    StatefulWidget::render(
        List::new(items).highlight_style(highlight),
        wide,
        &mut scratch,
        state,
    );
    copy_window(&scratch, off, frame.buffer_mut(), inner);
    off
}

/// Copies `src`'s columns `from..from + dst.width` into `dst` at `dst_rect`.
/// A wide glyph that would end past the right edge is replaced by a blank, as
/// is the cut half of one at the left edge.
fn copy_window(src: &Buffer, from: usize, dst: &mut Buffer, dst_rect: Rect) {
    let last = dst_rect.width.saturating_sub(1);
    for dy in 0..dst_rect.height.min(src.area.height) {
        for dx in 0..dst_rect.width {
            let sx = from + usize::from(dx);
            if sx >= usize::from(src.area.width) {
                break;
            }
            let mut cell = src[(sx as u16, dy)].clone();
            if dx == last && cells(cell.symbol()) > 1 {
                cell.set_symbol(" ");
            }
            dst[(dst_rect.x + dx, dst_rect.y + dy)] = cell;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spans(parts: &[&str]) -> Vec<Span<'static>> {
        parts.iter().map(|p| Span::raw((*p).to_string())).collect()
    }

    fn text(spans: &[Span<'_>]) -> String {
        spans.iter().map(|s| s.content.as_ref()).collect()
    }

    #[test]
    fn cut_spans_drops_whole_spans_and_part_of_the_next() {
        assert_eq!(text(&cut_spans(spans(&["ab", "cdef"]), 3)), "def");
        assert_eq!(text(&cut_spans(spans(&["ab", "cd"]), 0)), "abcd");
        assert_eq!(text(&cut_spans(spans(&["ab"]), 9)), "");
    }

    #[test]
    fn cut_spans_turns_a_cut_wide_char_into_a_blank() {
        assert_eq!(text(&cut_spans(spans(&["\u{4e2d}x"]), 1)), " x");
    }

    #[test]
    fn claim_clamps_the_offset_to_the_widest_row() {
        reset_all();
        let area = Rect::new(0, 0, 12, 5);
        assert_eq!(claim(HPane::CommitFiles, area, 30), 0);
        assert!(scroll_by(HPane::CommitFiles, 100));
        assert_eq!(offset(HPane::CommitFiles), 20);
        // The content shrank: the offset follows it back.
        assert_eq!(claim(HPane::CommitFiles, area, 15), 5);
        // It fits now: nothing scrolls.
        assert_eq!(claim(HPane::CommitFiles, area, 8), 0);
        assert!(!scroll_by(HPane::CommitFiles, 4));
        reset_all();
    }

    #[test]
    fn a_pane_not_drawn_this_frame_is_not_scrollable() {
        reset_all();
        let repo = std::path::Path::new("/r");
        begin_frame(repo);
        claim(HPane::CommitInfo, Rect::new(0, 0, 10, 4), 40);
        assert!(overflows(HPane::CommitInfo));
        begin_frame(repo);
        assert!(!overflows(HPane::CommitInfo));
        assert_eq!(pane_at(1, 1), None);
        reset_all();
    }

    #[test]
    fn a_new_repository_forgets_the_offsets() {
        reset_all();
        begin_frame(std::path::Path::new("/a"));
        claim(HPane::CommitInfo, Rect::new(0, 0, 10, 4), 40);
        scroll_by(HPane::CommitInfo, 8);
        begin_frame(std::path::Path::new("/b"));
        assert_eq!(claim(HPane::CommitInfo, Rect::new(0, 0, 10, 4), 40), 0);
        reset_all();
    }

    fn drawn(items: Vec<ListItem<'static>>, w: u16, scroll_to: usize) -> (Buffer, usize) {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;
        let mut term = Terminal::new(TestBackend::new(w, 4)).unwrap();
        let outer = Rect::new(0, 0, w, 4);
        let inner = Rect::new(1, 1, w - 2, 2);
        let mut state = ListState::default();
        state.select(Some(0));
        let pane = HPane::CommitFiles;
        let mut used = 0;
        let mut frame = |term: &mut Terminal<TestBackend>, used: &mut usize| {
            term.draw(|f| {
                *used = render_list(
                    f,
                    items.clone(),
                    Style::default(),
                    inner,
                    &mut state,
                    pane,
                    outer,
                    false,
                );
            })
            .unwrap();
        };
        // The first frame learns the width, then the pane scrolls and the
        // next frame draws at the offset.
        frame(&mut term, &mut used);
        scroll_by(pane, scroll_to as isize);
        frame(&mut term, &mut used);
        (term.backend().buffer().clone(), used)
    }

    fn row(buf: &Buffer, y: u16) -> String {
        (0..buf.area.width)
            .map(|x| buf[(x, y)].symbol().to_string())
            .collect()
    }

    #[test]
    fn a_wide_row_is_drawn_from_its_offset_and_the_border_stays() {
        reset_all();
        let long = "0123456789abcdefghijTAIL";
        let items = vec![ListItem::new(long), ListItem::new("short")];
        let (buf, used) = drawn(items.clone(), 12, 8);
        assert_eq!(used, 8, "it scrolled");
        // The 10 inner cells hold the text from the offset on; the column
        // outside the list on each side is untouched.
        assert_eq!(&row(&buf, 1)[1..11], &long[used..used + 10]);
        assert_eq!(&row(&buf, 1)[0..1], " ");
        reset_all();
        let (buf, used) = drawn(items, 12, 0);
        assert_eq!(used, 0);
        assert_eq!(&row(&buf, 1)[1..11], &long[..10]);
        reset_all();
    }

    #[test]
    fn a_row_that_fits_has_no_bar_and_no_offset() {
        reset_all();
        let (_, used) = drawn(vec![ListItem::new("fits")], 12, 8);
        assert_eq!(used, 0);
        assert!(track(HPane::CommitFiles).is_none(), "no bar");
        reset_all();
    }

    #[test]
    fn a_wide_char_at_the_right_edge_is_blanked_not_split() {
        reset_all();
        // The wide char lands on the last inner cell and its tail would
        // spill into the border column.
        let items = vec![ListItem::new("abcdefghijk\u{4e2d}tail")];
        let (buf, used) = drawn(items, 12, 2);
        assert_eq!(used, 2);
        assert_eq!(buf[(10, 1)].symbol(), " ", "{}", row(&buf, 1));
        reset_all();
    }
}
