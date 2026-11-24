//! Selectable, copyable text in read-only surfaces that are painted from
//! rich widgets (the conversation timeline, the commit details, the Activity
//! Log, the help and Markdown viewers): the same drag, the same highlight,
//! the same I-beam as `text_select`, for text that has no logical-line model.
//!
//! # The model: the painted rows
//!
//! These surfaces lay text out themselves (Markdown, wrapped paragraphs,
//! several styles per row), so there is no "logical line" to anchor to.
//! After a surface painted its content area, [`ScreenText::capture`] reads
//! the rows back from the frame buffer; each screen row is one line of a
//! `text_select::LineSource`, and the usual `pos_at`, `paint` and
//! `selected_text` work on it unchanged. What is copied is what is on the
//! screen, row by row (trailing blanks dropped, a wrapped paragraph copies as
//! its rows), so the area handed to `capture` must be the *content*: no
//! border, no scrollbar column.
//!
//! A selection lives as long as what it was made on. A scroll is followed:
//! when the new rows are the old ones moved up or down (the same rows, shifted
//! by some lines), the selection moves with the text and keeps what has
//! scrolled out of view (the rows it covered are remembered), so a wheel turn
//! does not lose it. Any other change of the painted rows or of the area (a
//! refresh that rewrites text, a resize) drops it.
//!
//! # One rule for the pointer
//!
//! [`ScreenText::register_pointer`] is fed by the same rows the selection
//! uses, so the I-beam appears exactly where a press can start a selection.
//! A surface calls it after [`ScreenText::capture`] and then registers its
//! controls (links, buttons) again on top: the last registered answers
//! first, so a link keeps the hand. (Painting helpers register `Plain` row
//! bands as they go, which would hide an I-beam registered earlier.)
//!
//! # A click keeps its meaning
//!
//! [`ScreenText::press`] only remembers where the button went down; the surface goes on to do what a
//! click there did. Only the first drag event turns the press into a
//! selection, and that drag (and the release that ends it) is the surface's
//! to swallow.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::text_select::{self, LineSource, RowSegs, Seg, Tabs, TextPos, TextSelection};

#[derive(Default)]
pub(crate) struct ScreenText {
    area: Rect,
    /// What the area shows now.
    rows: Vec<String>,
    /// The rows a selection or a press lives in: what the area showed, plus
    /// what a scroll took out of view while it was held. Row `top` of it is
    /// the first row shown.
    doc: Vec<String>,
    top: usize,
    segs: Vec<RowSegs>,
    sel: Option<TextSelection>,
    /// Where the left button went down, in the text, while it is held.
    press: Option<TextPos>,
    /// A drag turned the press into a selection: its release is not a click.
    dragged: bool,
    /// A press just went down: the first repaint after it may change the
    /// rows (the click selected an entry and painted its buttons) without
    /// that dropping the selection the press grows into.
    rebase: bool,
}

struct Rows<'a>(&'a [String]);

impl LineSource for Rows<'_> {
    fn raw(&self, line: usize) -> Option<&str> {
        self.0.get(line).map(String::as_str)
    }
    fn tabs(&self) -> Tabs {
        Tabs::OneSpace
    }
}

/// What row `y` of `area` shows, without its trailing blanks. A wide char's
/// second cell is not a char of the text.
fn row_text(buf: &Buffer, area: Rect, y: u16) -> String {
    let mut out = String::new();
    let mut skip = 0usize;
    for x in area.x..area.right() {
        if skip > 0 {
            skip -= 1;
            continue;
        }
        let Some(cell) = buf.cell((x, y)) else {
            break;
        };
        let symbol = cell.symbol();
        let width = crate::display_width(symbol);
        if width > 1 {
            skip = width - 1;
        }
        out.push_str(if symbol.is_empty() { " " } else { symbol });
    }
    out.truncate(out.trim_end().len());
    out
}

/// By how many rows `new` is `old` scrolled: `new[i] == old[i + k]` over the
/// rows both show. `None` when no shift explains the change (or only a blank
/// overlap does), the smallest one when several do.
fn scrolled_by(old: &[String], new: &[String]) -> Option<isize> {
    let h = old.len();
    if h != new.len() || h < 3 {
        return None;
    }
    let need = (h / 3).max(2);
    (1..h as isize).flat_map(|d| [d, -d]).find(|&k| {
        let (mut overlap, mut text) = (0usize, false);
        for (i, row) in new.iter().enumerate() {
            let Some(j) = usize::try_from(i as isize + k).ok().filter(|&j| j < h) else {
                continue;
            };
            if *row != old[j] {
                return false;
            }
            overlap += 1;
            text |= !row.is_empty();
        }
        overlap >= need && text
    })
}

impl ScreenText {
    /// An empty one, for a `static`.
    pub(crate) const fn new() -> Self {
        Self {
            area: Rect::new(0, 0, 0, 0),
            rows: Vec::new(),
            doc: Vec::new(),
            top: 0,
            segs: Vec::new(),
            sel: None,
            press: None,
            dragged: false,
            rebase: false,
        }
    }

    /// The rows moved by `k`: what is selected or pressed moves with its
    /// text, and the remembered rows grow to hold what came into view.
    fn follow_scroll(&mut self, k: isize, rows: &[String]) {
        let first = self.top as isize + k;
        if first < 0 {
            let add = (-first) as usize;
            self.doc
                .splice(0..0, std::iter::repeat_n(String::new(), add));
            for p in self
                .sel
                .iter_mut()
                .flat_map(|s| [&mut s.anchor, &mut s.head])
                .chain(self.press.as_mut())
            {
                p.line += add;
            }
            self.top = 0;
        } else {
            self.top = first as usize;
        }
        let end = self.top + rows.len();
        if self.doc.len() < end {
            self.doc.resize(end, String::new());
        }
        self.doc[self.top..end].clone_from_slice(rows);
    }

    /// Reads the rows `area` shows from the frame `buf`, after the surface
    /// painted them. A different area or different rows drop the selection.
    pub(crate) fn capture(&mut self, buf: &Buffer, area: Rect) {
        self.capture_spans(buf, area, |_| Some((area.x, area.right())));
    }

    /// Like [`capture`](Self::capture), for a surface where only some rows
    /// (and only part of them) are text: `span(i)` is the absolute columns
    /// `[from, to)` of row `i` of `area` that hold selectable text, `None`
    /// for a row that holds none (code, a gutter, a header).
    pub(crate) fn capture_spans(
        &mut self,
        buf: &Buffer,
        area: Rect,
        span: impl Fn(usize) -> Option<(u16, u16)>,
    ) {
        let cols: Vec<(u16, u16)> = (0..usize::from(area.height))
            .map(|i| {
                span(i).map_or((0, 0), |(from, to)| {
                    let from = from.max(area.x);
                    (from - area.x, to.min(area.right()).saturating_sub(from))
                })
            })
            .collect();
        let rows: Vec<String> = (area.y..area.bottom())
            .zip(&cols)
            .map(|(y, &(x, width))| {
                if width == 0 {
                    String::new()
                } else {
                    row_text(buf, Rect::new(area.x + x, y, width, 1), y)
                }
            })
            .collect();
        // A press survives the repaint it causes (a click that selects an
        // entry paints its buttons); a finished selection does not survive
        // different rows.
        if area != self.area {
            self.press = None;
            self.dragged = false;
            self.sel = None;
        } else if rows != self.rows {
            let live = self.sel.is_some() || self.press.is_some();
            match live.then(|| scrolled_by(&self.rows, &rows)).flatten() {
                Some(k) => self.follow_scroll(k, &rows),
                None if !self.dragged && !self.rebase => self.sel = None,
                None => {}
            }
        }
        if self.sel.is_some() || self.press.is_some() {
            let end = self.top + rows.len();
            if self.doc.len() < end {
                self.doc.resize(end, String::new());
            }
            self.doc[self.top..end].clone_from_slice(&rows);
        } else {
            self.doc = rows.clone();
            self.top = 0;
        }
        let top = self.top;
        self.segs = rows
            .iter()
            .zip(&cols)
            .map(|(row, &(x, width))| {
                let len = row.chars().count();
                let seg = (len > 0).then_some(Seg {
                    line: 0,
                    start: 0,
                    len,
                    x,
                    width,
                    last: true,
                });
                [seg, None]
            })
            .enumerate()
            .map(|(i, mut segs)| {
                if let Some(seg) = segs[0].as_mut() {
                    seg.line = top + i;
                }
                segs
            })
            .collect();
        self.rows = rows;
        self.area = area;
        self.rebase = false;
    }

    /// Paints the selection over the rows already drawn.
    pub(crate) fn paint(&self, buf: &mut Buffer) {
        if let Some(sel) = &self.sel {
            text_select::paint(buf, self.area, &self.segs, sel, &Rows(&self.doc));
        }
    }

    /// The I-beam over the cells of text last captured. Call it after
    /// `capture`, then register the surface's own controls over it.
    pub(crate) fn register_pointer(&self) {
        text_select::register_pointer(self.area, &self.segs);
    }

    /// A text position under the cell, `None` outside the area or past the
    /// last row of text.
    fn pos(&self, col: u16, row: u16) -> Option<TextPos> {
        if !self.area.contains((col, row).into()) {
            return None;
        }
        let r = usize::from(row - self.area.y);
        text_select::pos_at(&self.segs, r, 0, col - self.area.x, &Rows(&self.doc))
    }

    /// The text lies under this cell (a press there can start a selection).
    pub(crate) fn text_at(&self, col: u16, row: u16) -> bool {
        self.area.contains((col, row).into())
            && self
                .segs
                .get(usize::from(row.saturating_sub(self.area.y)))
                .is_some_and(|s| {
                    s[0].is_some_and(|s| (s.x..s.x + s.len as u16).contains(&(col - self.area.x)))
                })
    }

    /// The left button went down at the cell. Returns whether it landed on
    /// text; the caller goes on with what a click there does. (No
    /// double-click word selection: a double-click means something on some
    /// of these surfaces.)
    pub(crate) fn press(&mut self, col: u16, row: u16) -> bool {
        self.sel = None;
        self.press = None;
        self.dragged = false;
        let Some(pos) = self.pos(col, row).filter(|_| self.text_at(col, row)) else {
            return false;
        };
        self.press = Some(pos);
        self.rebase = true;
        true
    }

    /// The button is held and moved to the cell. `true`: this is a text
    /// selection now and the surface must not treat the move as its own.
    pub(crate) fn drag(&mut self, col: u16, row: u16) -> bool {
        let Some(anchor) = self.press else {
            return self.dragged;
        };
        let col = col.clamp(self.area.x, self.area.right().saturating_sub(1));
        let row = row.clamp(self.area.y, self.area.bottom().saturating_sub(1));
        let Some(head) = self.pos(col, row) else {
            return self.dragged;
        };
        self.dragged = true;
        self.sel = Some(TextSelection {
            side: 0,
            anchor,
            head,
        });
        true
    }

    /// The button went up. `true`: it ended a selection drag, so it is not a
    /// click.
    pub(crate) fn release(&mut self) -> bool {
        self.press = None;
        std::mem::take(&mut self.dragged)
    }

    /// Drops the selection (Esc, a click elsewhere). `true` when there was one.
    pub(crate) fn clear(&mut self) -> bool {
        self.press = None;
        self.dragged = false;
        self.sel.take().is_some()
    }

    /// A press is held or a selection is up: events outside the area still
    /// belong to it.
    #[cfg_attr(not(feature = "review"), allow(dead_code))]
    pub(crate) fn selecting(&self) -> bool {
        self.press.is_some() || self.sel.is_some()
    }

    pub(crate) fn has_selection(&self) -> bool {
        self.sel.is_some()
    }

    /// The selected text, as painted.
    pub(crate) fn selected(&self) -> Option<String> {
        let sel = self.sel.as_ref()?;
        let text = text_select::selected_text(sel, &Rows(&self.doc));
        (!text.is_empty()).then_some(text)
    }

    /// Everything the area shows (the keyboard's "copy it all").
    #[cfg(test)]
    pub(crate) fn all(&self) -> String {
        self.rows.join("\n").trim_end().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Style;

    fn painted(rows: &[&str], width: u16) -> (Buffer, Rect) {
        let area = Rect::new(2, 1, width, rows.len() as u16);
        let mut buf = Buffer::empty(Rect::new(0, 0, width + 4, rows.len() as u16 + 2));
        for (i, row) in rows.iter().enumerate() {
            buf.set_string(area.x, area.y + i as u16, row, Style::default());
        }
        (buf, area)
    }

    #[test]
    fn a_drag_over_two_rows_copies_what_is_painted_without_trailing_blanks() {
        let (buf, area) = painted(&["hello world", "second row"], 20);
        let mut st = ScreenText::default();
        st.capture(&buf, area);
        assert!(st.press(2 + 6, 1));
        assert!(st.drag(2 + 5, 2), "the first move is a selection");
        assert!(st.release(), "the release ends a drag, it is not a click");
        assert_eq!(st.selected().as_deref(), Some("world\nsecond"));
    }

    #[test]
    fn a_press_without_a_drag_selects_nothing_and_stays_a_click() {
        let (buf, area) = painted(&["hello world"], 20);
        let mut st = ScreenText::default();
        st.capture(&buf, area);
        assert!(st.press(3, 1));
        assert!(!st.release());
        assert_eq!(st.selected(), None);
    }

    #[test]
    fn a_press_on_blank_cells_starts_nothing() {
        let (buf, area) = painted(&["hi"], 20);
        let mut st = ScreenText::default();
        st.capture(&buf, area);
        assert!(!st.press(2 + 10, 1));
        assert!(!st.drag(2 + 11, 1));
    }

    #[test]
    fn changed_rows_drop_the_selection() {
        let (buf, area) = painted(&["one two"], 20);
        let mut st = ScreenText::default();
        st.capture(&buf, area);
        st.press(2, 1);
        st.drag(6, 1);
        st.release();
        st.capture(&buf, area);
        assert!(st.has_selection());
        let (other, area) = painted(&["new text"], 20);
        st.capture(&other, area);
        assert!(!st.has_selection());
        let (same, area) = painted(&["new text"], 20);
        st.capture(&same, area);
        st.press(2, 1);
        st.drag(6, 1);
        st.capture(&same, area);
        assert!(st.has_selection(), "the same rows keep it");
    }

    #[test]
    fn a_scroll_keeps_the_selection_with_its_text_even_out_of_view() {
        let all = ["aa", "bb", "cc", "dd", "ee", "ff", "gg", "hh"];
        let mut st = ScreenText::default();
        let (buf, area) = painted(&all[0..5], 20);
        st.capture(&buf, area);
        st.press(2, 2);
        st.drag(3, 3);
        st.release();
        assert_eq!(st.selected().as_deref(), Some("bb\ncc"));
        // Three rows down: the selection is above the view now.
        let (buf, area) = painted(&all[3..8], 20);
        st.capture(&buf, area);
        assert_eq!(st.selected().as_deref(), Some("bb\ncc"), "survives");
        // And back up past where it began, then to where it is on screen.
        let (buf, area) = painted(&all[0..5], 20);
        st.capture(&buf, area);
        assert_eq!(st.selected().as_deref(), Some("bb\ncc"));
        let (buf, area) = painted(&all[1..6], 20);
        st.capture(&buf, area);
        assert_eq!(st.selected().as_deref(), Some("bb\ncc"));
    }

    #[test]
    fn a_scroll_up_past_the_start_keeps_the_selection() {
        let all = ["aa", "bb", "cc", "dd", "ee", "ff", "gg", "hh"];
        let mut st = ScreenText::default();
        let (buf, area) = painted(&all[3..8], 20);
        st.capture(&buf, area);
        st.press(2, 1);
        st.drag(3, 1);
        st.release();
        assert_eq!(st.selected().as_deref(), Some("dd"));
        let (buf, area) = painted(&all[0..5], 20);
        st.capture(&buf, area);
        assert_eq!(st.selected().as_deref(), Some("dd"));
    }

    #[test]
    fn rewritten_rows_are_not_a_scroll() {
        let mut st = ScreenText::default();
        let (buf, area) = painted(&["aa", "bb", "cc", "dd"], 20);
        st.capture(&buf, area);
        st.press(2, 1);
        st.drag(3, 1);
        st.release();
        st.capture(&buf, area);
        assert!(st.has_selection());
        let (buf, area) = painted(&["xx", "yy", "zz", "ww"], 20);
        st.capture(&buf, area);
        assert!(!st.has_selection());
    }

    #[test]
    fn a_wide_char_is_one_char_of_the_text() {
        let (buf, area) = painted(&["a\u{4e2d}b"], 20);
        let mut st = ScreenText::default();
        st.capture(&buf, area);
        assert_eq!(st.all(), "a\u{4e2d}b");
        st.press(2, 1);
        st.drag(2 + 3, 1);
        assert_eq!(st.selected().as_deref(), Some("a\u{4e2d}b"));
    }
}
