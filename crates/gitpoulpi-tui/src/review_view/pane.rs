//! A scrolling diff pane's cursor, selection and scroll position. The rows
//! themselves come from `diffrows`; the Files page has one pane (a file
//! with its threads), the Commits page another (a commit's files).

use std::rc::Rc;

use ratatui::layout::Rect;

use super::diffrows::{kind_of, RowKind};
use crate::diff_view::{DiffRender, DiffRow};

/// What the rows were built for; a different key rebuilds them.
pub(super) type BuildKey = (String, u16, u64);

#[derive(Default)]
pub(super) struct DiffPane {
    /// The Diff view's own render of what the pane shows, with the review's
    /// rows in it.
    pub(super) render: Option<Rc<DiffRender>>,
    built_for: Option<BuildKey>,
    pub(super) cursor: usize,
    /// The other end of a multi-line selection, when there is one.
    pub(super) anchor: Option<usize>,
    pub(super) offset: usize,
    /// Where the rows were painted (set by the paint walk).
    pub(super) area: Rect,
    pub(super) bar: Option<crate::scrollbar::Track>,
    pub(super) drag: crate::scrollbar::BarDrag,
    /// A mouse press on a row that may turn into a drag selection.
    pub(super) press: Option<usize>,
    /// Split only: the side (`0` old, `1` new) a comment goes on when the
    /// cursor row shows a deletion and an addition. The side last clicked.
    pub(super) side: usize,
    /// The furthest the text can scroll sideways (the last render's
    /// `h_scroll_max`; 0 while it fits or wraps).
    pub(super) h_max: usize,
    /// The widest line of the render, in cells.
    pub(super) longest: usize,
    /// The horizontal bar as last painted, and where its thumb was grabbed.
    pub(super) h_bar: Option<crate::scrollbar::Track>,
    pub(super) h_grab: Option<i32>,
    /// A mouse text selection (anchored in the text, as the Diff view's) and
    /// the press that may still grow into one.
    pub(super) text_sel: Option<crate::text_select::TextSelection>,
    pub(super) text_drag: Option<crate::text_select::TextDrag>,
    /// The words of the inline thread, pending and suggestion rows, read
    /// back from the frame: selectable with a drag like the code is.
    pub(super) notes: crate::screen_text::ScreenText,
    /// The minimap's rect as last painted, and whether a press on it is held.
    pub(super) map: Option<Rect>,
    pub(super) map_drag: bool,
    pub(super) minimap: crate::minimap::Minimap,
}

impl DiffPane {
    /// Whether `key` differs from what the rows were built for.
    pub(super) fn stale(&self, key: &BuildKey) -> bool {
        self.built_for.as_ref() != Some(key)
    }

    /// The rows of the render (none before the first one).
    pub(super) fn rows(&self) -> &[DiffRow] {
        self.render.as_ref().map_or(&[], |r| &r.rows[..])
    }

    pub(super) fn len(&self) -> usize {
        self.rows().len()
    }

    pub(super) fn is_empty(&self) -> bool {
        self.rows().is_empty()
    }

    /// What row `i` is.
    pub(super) fn kind(&self, i: usize) -> Option<RowKind> {
        self.rows().get(i).map(kind_of)
    }

    /// The text row `i` shows: the code, or the annotation's words.
    #[cfg(test)]
    pub(super) fn text(&self, i: usize) -> String {
        self.render
            .as_ref()
            .and_then(|r| r.lines.get(i))
            .map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect())
            .unwrap_or_default()
    }

    pub(super) fn set_render(&mut self, key: BuildKey, render: Option<Rc<DiffRender>>) {
        let new_file = self
            .built_for
            .as_ref()
            .is_none_or(|(name, _, _)| *name != key.0);
        self.built_for = Some(key);
        self.render = render;
        if new_file {
            self.cursor = 0;
            self.offset = 0;
            self.anchor = None;
            self.text_sel = None;
            self.text_drag = None;
        }
        self.clamp();
    }

    /// The Diff view's state as this pane stands in it (its toggles, this
    /// pane's cursor, scroll and selections): what the shared copy and
    /// selection painting functions read.
    pub(super) fn view_state(
        &self,
        base: &crate::diff_view::DiffViewState,
    ) -> crate::diff_view::DiffViewState {
        let mut s = base.clone();
        s.scroll = self.offset;
        s.cursor = self.cursor;
        s.selection_anchor = self.anchor;
        s.text_sel = self.text_sel;
        s
    }

    /// What `Track::horizontal` is drawn and dragged for: the scroll range
    /// plus the track's own width (so the thumb spans exactly `0..=h_max`).
    pub(super) fn h_content_len(&self) -> usize {
        match (self.h_max, self.h_bar) {
            (0, _) | (_, None) => 0,
            (max, Some(track)) => max + track.rect.width as usize,
        }
    }

    pub(super) fn visible(&self) -> usize {
        self.area.height.max(1) as usize
    }

    /// `clamp` for a position restored from a marker.
    pub(super) fn clamp_to_rows(&mut self) {
        self.clamp();
    }

    fn clamp(&mut self) {
        let last = self.len().saturating_sub(1);
        self.cursor = self.cursor.min(last);
        self.anchor = self.anchor.map(|a| a.min(last));
        let max = self.len().saturating_sub(self.visible());
        self.offset = self.offset.min(max);
    }

    /// Keeps the cursor on screen.
    pub(super) fn reveal(&mut self) {
        let rows = self.visible();
        if self.cursor < self.offset {
            self.offset = self.cursor;
        } else if self.cursor >= self.offset + rows {
            self.offset = self.cursor + 1 - rows;
        }
    }

    pub(super) fn move_cursor(&mut self, delta: isize, extend: bool) {
        if self.is_empty() {
            return;
        }
        if extend {
            self.anchor.get_or_insert(self.cursor);
        } else {
            self.anchor = None;
        }
        let last = self.len() - 1;
        self.cursor = (self.cursor as isize + delta).clamp(0, last as isize) as usize;
        self.reveal();
    }

    pub(super) fn jump(&mut self, to: usize) {
        if self.is_empty() {
            return;
        }
        self.anchor = None;
        self.cursor = to.min(self.len() - 1);
        self.reveal();
    }

    pub(super) fn scroll(&mut self, delta: isize) {
        let max = self.len().saturating_sub(self.visible());
        self.offset = (self.offset as isize + delta).clamp(0, max as isize) as usize;
    }

    /// First and last row of the selection (the cursor alone without one).
    pub(super) fn selection(&self) -> (usize, usize) {
        match self.anchor {
            Some(a) => (a.min(self.cursor), a.max(self.cursor)),
            None => (self.cursor, self.cursor),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text_select::RowSegs;

    fn pane(n: usize) -> DiffPane {
        let mut p = DiffPane {
            area: Rect::new(0, 5, 40, 4),
            ..DiffPane::default()
        };
        p.set_render(("a".into(), 40, 0), Some(rows(n)));
        p
    }

    fn rows(n: usize) -> Rc<DiffRender> {
        let segs: Vec<RowSegs> = vec![[None, None]; n];
        Rc::new(DiffRender {
            lines: vec![ratatui::text::Line::default(); n],
            hunk_starts: Vec::new(),
            rows: Rc::from(vec![DiffRow::default(); n]),
            segs: Rc::from(segs),
            text: Rc::default(),
            split_x: None,
            annot_text: Rc::default(),
        })
    }

    #[test]
    fn the_cursor_scrolls_the_view_and_a_selection_remembers_where_it_started() {
        let mut p = pane(10);
        p.move_cursor(5, false);
        assert_eq!((p.cursor, p.offset), (5, 2), "the view follows the cursor");
        p.move_cursor(-1, true);
        p.move_cursor(-1, true);
        assert_eq!(p.selection(), (3, 5));
        p.move_cursor(1, false);
        assert_eq!(p.anchor, None, "a plain move drops the selection");
    }

    #[test]
    fn rows_rebuilt_for_the_same_file_keep_the_cursor_a_new_file_resets_it() {
        let mut p = pane(10);
        p.jump(7);
        p.set_render(("a".into(), 40, 1), Some(rows(10)));
        assert_eq!(p.cursor, 7);
        p.set_render(("b".into(), 40, 1), Some(rows(10)));
        assert_eq!(p.cursor, 0);
        assert!(p.stale(&("b".into(), 41, 1)));
    }
}
