//! Selectable, copyable text in the read-only file-content views: the
//! Diff view, File View, Blame and File History.
//!
//! # The model: logical positions, not screen cells
//!
//! A selection is kept as two positions in the *text* -- `(line, char
//! offset)` in one column of text -- never as screen cells. The views
//! wrap, scroll sideways, truncate with `…` and mark a scrolled row with
//! `«`, so a cell range read back off the screen would copy exactly those
//! decorations and lose whatever did not fit. Anchored in the text, the
//! selection survives a vertical or horizontal scroll unchanged, and
//! copying it yields the real line content: no line numbers, no `+`/`-`
//! markers, no blame gutter, no `@@` header (those rows carry no text at
//! all here), and whole lines even where the screen showed only a part.
//!
//! Every view describes what each screen row shows as a [`Seg`]: which
//! logical line, from which char offset, drawn from which column. Those
//! are produced by the *same* walk that draws the rows (`diff_view`'s
//! emitters, `file_view::insight_segs`), the "one walk, never two"
//! discipline the diff's `hunk_starts` and row map already follow. This
//! module then does the three things a selection needs, all pure:
//! [`pos_at`] (a mouse cell to a text position), [`paint`] (the
//! highlight, written straight into the frame buffer after the rows were
//! drawn) and [`selected_text`] (what `Ctrl+C` / `y` / Copy put on the
//! clipboard).
//!
//! # Offsets are display chars; copies are raw
//!
//! Offsets count chars of the line *as displayed*: the diff expands a tab
//! to the next 4-column stop, the File Insight views replace a tab with
//! one space. [`LineSource::tabs`] says which, and [`selected_text`] maps
//! the offsets back onto the raw line, so a copied tab is a tab again.
//!
//! # Columns are cells
//!
//! A mouse column becomes an offset by walking the displayed chars and
//! summing their cell widths, so a wide (CJK, emoji) char is one offset
//! covering two cells, both ends of it mapping to the same char.
//!
//! # Split
//!
//! The Split diff has two columns of text, `side` 0 (before) and 1
//! (after). A selection belongs to the side its drag started on and never
//! crosses the separator: dragging onto the other half keeps extending
//! the original side, row by row.

use crate::th;
use std::borrow::Cow;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

/// Which view a text drag belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextView {
    Diff,
    File,
}

/// A left-button press that may turn into a text selection: where it went
/// down, in the text, and on which side. A press alone selects nothing --
/// a single click keeps doing what it always did (move the cursor, pick
/// the row); only the first drag event turns this into a selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TextDrag {
    pub(crate) view: TextView,
    pub(crate) side: usize,
    pub(crate) anchor: TextPos,
}

impl TextDrag {
    pub(crate) fn to(self, head: TextPos) -> TextSelection {
        TextSelection {
            side: self.side,
            anchor: self.anchor,
            head,
        }
    }
}

/// The press half of the gesture, shared by every view: a double-click
/// selects the word under the pointer at once; any other press clears
/// the old selection and returns the drag to start.
pub(crate) fn press(
    view: TextView,
    side: usize,
    pos: Option<TextPos>,
    double: bool,
    src: &dyn LineSource,
    sel: &mut Option<TextSelection>,
) -> Option<TextDrag> {
    *sel = None;
    let pos = pos?;
    if double {
        *sel = word_at(pos, src).map(|(anchor, head)| TextSelection { side, anchor, head });
        return None;
    }
    Some(TextDrag {
        view,
        side,
        anchor: pos,
    })
}

/// A position in one column of text: the logical line and the char
/// offset within its *displayed* form. `offset == len` is the position
/// just past the end of the line (a click beyond the text).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct TextPos {
    pub(crate) line: usize,
    pub(crate) offset: usize,
}

/// A text selection. Both ends are *inclusive*, the way a terminal's own
/// selection is: dragging from a char to another selects both of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TextSelection {
    /// 0 for the Inline diff and the File Insight views; in Split, 0 is
    /// the before column and 1 the after column.
    pub(crate) side: usize,
    pub(crate) anchor: TextPos,
    pub(crate) head: TextPos,
}

impl TextSelection {
    pub(crate) fn ordered(&self) -> (TextPos, TextPos) {
        if self.anchor <= self.head {
            (self.anchor, self.head)
        } else {
            (self.head, self.anchor)
        }
    }

    /// The half-open char range of `line` (whose displayed length is
    /// `len`) this selection covers, and whether it also covers the line
    /// break after it. `None` when the line is outside the selection.
    pub(crate) fn span_on(&self, line: usize, len: usize) -> Option<(usize, usize, bool)> {
        let (from, to) = self.ordered();
        if line < from.line || line > to.line {
            return None;
        }
        let start = if line == from.line {
            from.offset.min(len)
        } else {
            0
        };
        let end = if line == to.line {
            (to.offset + 1).min(len)
        } else {
            len
        };
        Some((start, end.max(start), line < to.line))
    }
}

/// What one screen row shows of one logical line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Seg {
    /// Index of the logical line in its side's text.
    pub(crate) line: usize,
    /// Displayed-char offset of what is drawn at content column 0. With a
    /// horizontal scroll this is `h_offset - 1`: that char's cell holds
    /// the `«` marker, and treating it as that char keeps every column to
    /// its right on its real offset.
    pub(crate) start: usize,
    /// How many chars of the line this row stands for, from `start`. A
    /// word-wrapped row includes the blank its break swallowed; the last
    /// row of a line runs to the end of the line, even where the screen
    /// cut it short with `…`.
    pub(crate) len: usize,
    /// First content column, relative to the view's inner rect.
    pub(crate) x: u16,
    /// Content cells available on the row.
    pub(crate) width: u16,
    /// This is the line's last row (or its only one).
    pub(crate) last: bool,
}

/// A row's segments, by side (Inline and File Insight only use side 0).
pub(crate) type RowSegs = [Option<Seg>; 2];

/// How a view turns a raw line into what it draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tabs {
    /// A tab advances to the next multiple of this many columns (the
    /// diff).
    Expand(usize),
    /// A tab is drawn as a single space (the File Insight views).
    OneSpace,
}

/// The raw lines of one column of text.
pub(crate) trait LineSource {
    fn raw(&self, line: usize) -> Option<&str>;
    fn tabs(&self) -> Tabs;
}

/// A plain list of lines - the diff's two sides.
pub(crate) struct Lines<'a> {
    pub(crate) lines: &'a [String],
    pub(crate) tabs: Tabs,
}

impl LineSource for Lines<'_> {
    fn raw(&self, line: usize) -> Option<&str> {
        self.lines.get(line).map(String::as_str)
    }
    fn tabs(&self) -> Tabs {
        self.tabs
    }
}

/// The line as drawn.
pub(crate) fn display(raw: &str, tabs: Tabs) -> Cow<'_, str> {
    if !raw.contains('\t') {
        return Cow::Borrowed(raw);
    }
    let mut out = String::with_capacity(raw.len());
    let mut col = 0usize;
    for ch in raw.chars() {
        if ch == '\t' {
            let pad = match tabs {
                Tabs::Expand(n) => n - (col % n),
                Tabs::OneSpace => 1,
            };
            out.extend(std::iter::repeat(' ').take(pad));
            col += pad;
        } else {
            out.push(ch);
            col += 1;
        }
    }
    Cow::Owned(out)
}

/// The cells one char takes on screen. Control chars and combining marks
/// are 0 in Unicode's table; they still occupy a slot in the offset
/// numbering, so they are counted as one cell here rather than making
/// two offsets share a column.
fn cell_width(c: char) -> usize {
    let mut buf = [0u8; 4];
    crate::display_width(c.encode_utf8(&mut buf)).max(1)
}

/// Displayed-char offset -> raw-char offset, rounding *down* (a start
/// inside an expanded tab takes the tab).
fn raw_floor(raw: &str, tabs: Tabs, offset: usize) -> usize {
    raw_map(raw, tabs, offset, false)
}

/// Displayed-char offset -> raw-char offset, rounding *up* (an end inside
/// an expanded tab takes the tab).
fn raw_ceil(raw: &str, tabs: Tabs, offset: usize) -> usize {
    raw_map(raw, tabs, offset, true)
}

fn raw_map(raw: &str, tabs: Tabs, offset: usize, ceil: bool) -> usize {
    let mut col = 0usize;
    for (i, ch) in raw.chars().enumerate() {
        let w = match (ch, tabs) {
            ('\t', Tabs::Expand(n)) => n - (col % n),
            _ => 1,
        };
        let covers = if ceil {
            col < offset
        } else {
            col + w <= offset
        };
        if !covers {
            return i;
        }
        col += w;
    }
    raw.chars().count()
}

/// Which side a content column belongs to: everything left of
/// `split_x` (the after column's first cell) is side 0.
pub(crate) fn side_at(split_x: Option<u16>, x: u16) -> usize {
    match split_x {
        Some(split) if x >= split => 1,
        _ => 0,
    }
}

/// The text position under content column `x` (relative to the inner
/// rect) of row `row` on `side`.
///
/// A row that shows nothing of that side - a hunk header, a notice, the
/// blank half of a Split row - resolves to the nearest row below that
/// does (at its start), else the nearest above (at its end), so a drag
/// that crosses a header keeps extending instead of stalling on it.
pub(crate) fn pos_at(
    rows: &[RowSegs],
    row: usize,
    side: usize,
    x: u16,
    src: &dyn LineSource,
) -> Option<TextPos> {
    let seg_of = |r: usize| rows.get(r).and_then(|segs| segs[side]);
    if let Some(seg) = seg_of(row) {
        return Some(pos_in_seg(seg, x, src));
    }
    if let Some(seg) = (row + 1..rows.len()).find_map(seg_of) {
        return Some(TextPos {
            line: seg.line,
            offset: seg.start,
        });
    }
    let seg = (0..row.min(rows.len())).rev().find_map(seg_of)?;
    let len = src
        .raw(seg.line)
        .map_or(0, |raw| display(raw, src.tabs()).chars().count());
    Some(TextPos {
        line: seg.line,
        offset: len,
    })
}

fn pos_in_seg(seg: Seg, x: u16, src: &dyn LineSource) -> TextPos {
    let raw = src.raw(seg.line).unwrap_or("");
    let shown = display(raw, src.tabs());
    let len = shown.chars().count();
    let col = x.saturating_sub(seg.x) as usize;
    let at = |offset| TextPos {
        line: seg.line,
        offset,
    };
    // The last cell of a row cut short with `…` stands for everything
    // that did not fit: dragging onto it takes the line to its end.
    let tail_cells: usize = shown.chars().skip(seg.start).map(cell_width).sum();
    let truncated = seg.last && tail_cells > seg.width as usize;
    if truncated && col + 1 >= seg.width as usize {
        return at(len);
    }
    let mut acc = 0usize;
    let end = (seg.start + seg.len).min(len);
    for (i, ch) in shown
        .chars()
        .enumerate()
        .skip(seg.start)
        .take(end.saturating_sub(seg.start))
    {
        let w = cell_width(ch);
        if col < acc + w {
            return at(i);
        }
        acc += w;
    }
    // Past the drawn text. The last row of a line means "the end of the
    // line"; an earlier wrapped row means its own last char (the blank
    // its break swallowed), never the next row's first.
    if seg.last {
        at(len)
    } else {
        at(end.saturating_sub(1).max(seg.start))
    }
}

/// The word (or run of blanks, or of punctuation) under `pos` - the
/// double-click rule, with the same three classes the text fields use.
/// `None` past the end of the line.
pub(crate) fn word_at(pos: TextPos, src: &dyn LineSource) -> Option<(TextPos, TextPos)> {
    let raw = src.raw(pos.line)?;
    let chars: Vec<char> = display(raw, src.tabs()).chars().collect();
    let c = *chars.get(pos.offset)?;
    let class = crate::text_edit::char_class(c);
    let same = |c: &char| crate::text_edit::char_class(*c) == class;
    let start = pos.offset
        - chars[..pos.offset]
            .iter()
            .rev()
            .take_while(|c| same(c))
            .count();
    let end = pos.offset + chars[pos.offset..].iter().take_while(|c| same(c)).count();
    let at = |offset| TextPos {
        line: pos.line,
        offset,
    };
    Some((at(start), at(end - 1)))
}

/// The selected text, as the raw lines hold it: tabs are tabs again, a
/// line break joins every two lines, and nothing else - no gutter, no
/// marker, no header - was ever part of a line to begin with.
pub(crate) fn selected_text(sel: &TextSelection, src: &dyn LineSource) -> String {
    let (from, to) = sel.ordered();
    let mut out = String::new();
    for line in from.line..=to.line {
        let Some(raw) = src.raw(line) else {
            continue;
        };
        let len = display(raw, src.tabs()).chars().count();
        let Some((start, end, _)) = sel.span_on(line, len) else {
            continue;
        };
        if line > from.line {
            out.push('\n');
        }
        let a = raw_floor(raw, src.tabs(), start);
        let b = raw_ceil(raw, src.tabs(), end).max(a);
        out.extend(raw.chars().skip(a).take(b - a));
    }
    out
}

/// Softly tints every occurrence of `needle` on the rows already drawn into
/// `buf`, on `side` (the selection's own side). Matches come from
/// `scroll_marks::match_ranges` - the ruler's source - and are mapped from
/// raw bytes to displayed chars so an expanded tab does not shift them. Only
/// the background changes, blended into the cell's own, so a diff's
/// add/remove colors stay visible; the selection itself is painted over
/// this afterwards.
pub(crate) fn paint_occurrences(
    buf: &mut Buffer,
    inner: Rect,
    rows: &[RowSegs],
    side: usize,
    needle: &str,
    src: &dyn LineSource,
) {
    if !crate::scroll_marks::valid_needle(needle) {
        return;
    }
    for (i, segs) in rows.iter().enumerate().take(inner.height as usize) {
        let Some(seg) = segs[side.min(1)] else {
            continue;
        };
        let Some(raw) = src.raw(seg.line) else {
            continue;
        };
        let ranges = crate::scroll_marks::match_ranges(needle, raw);
        if ranges.is_empty() {
            continue;
        }
        // (byte offset, displayed char offset) at every raw char boundary.
        let mut table: Vec<(usize, usize)> = Vec::new();
        let mut shown_off = 0usize;
        for (b, ch) in raw.char_indices() {
            table.push((b, shown_off));
            shown_off += if ch == '\t' {
                match src.tabs() {
                    Tabs::Expand(n) => n - (shown_off % n),
                    Tabs::OneSpace => 1,
                }
            } else {
                1
            };
        }
        table.push((raw.len(), shown_off));
        let shown_at = |byte: usize| {
            table
                .iter()
                .find(|(b, _)| *b == byte)
                .map_or(shown_off, |(_, s)| *s)
        };
        let spans: Vec<(usize, usize)> = ranges
            .iter()
            .map(|&(a, b)| (shown_at(a), shown_at(b)))
            .collect();
        let shown = display(raw, src.tabs());
        let len = shown.chars().count();
        let row_end = (seg.start + seg.len).min(len);
        let y = inner.y + i as u16;
        let mut acc = 0usize;
        for (k, ch) in shown
            .chars()
            .enumerate()
            .skip(seg.start)
            .take(row_end.saturating_sub(seg.start))
        {
            let w = cell_width(ch);
            if spans.iter().any(|&(a, b)| k >= a && k < b) {
                for dx in acc..acc + w {
                    let x = inner.x + seg.x + dx as u16;
                    if dx >= seg.width as usize || x >= inner.x + inner.width {
                        break;
                    }
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        let bg = match cell.bg {
                            ratatui::style::Color::Reset => None,
                            c => Some(c),
                        };
                        cell.set_bg(crate::scroll_marks::occurrence_bg(bg));
                    }
                }
            }
            acc += w;
        }
    }
}

/// Whole lines, joined - what "Copy line" and "Copy hunk" put on the
/// clipboard.
pub(crate) fn lines_text(lines: &[usize], src: &dyn LineSource) -> String {
    lines
        .iter()
        .filter_map(|&l| src.raw(l))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Paints the selection over rows already drawn into `buf`. `rows[i]` is
/// the screen row `inner.y + i`. Only cells that show selected text get
/// the highlight, plus one cell past a line's end where the selection
/// runs on to the next line - the same cue an editor gives a selected
/// line break, and what keeps a selected *empty* line visible at all.
pub(crate) fn paint(
    buf: &mut Buffer,
    inner: Rect,
    rows: &[RowSegs],
    sel: &TextSelection,
    src: &dyn LineSource,
) {
    for (i, segs) in rows.iter().enumerate().take(inner.height as usize) {
        let Some(seg) = segs[sel.side.min(1)] else {
            continue;
        };
        let Some(raw) = src.raw(seg.line) else {
            continue;
        };
        let shown = display(raw, src.tabs());
        let len = shown.chars().count();
        let Some((start, end, newline)) = sel.span_on(seg.line, len) else {
            continue;
        };
        let row_end = (seg.start + seg.len).min(len);
        let mut acc = 0usize;
        let mut x0: Option<usize> = None;
        let mut x1 = 0usize;
        for (k, ch) in shown
            .chars()
            .enumerate()
            .skip(seg.start)
            .take(row_end.saturating_sub(seg.start))
        {
            let w = cell_width(ch);
            if k >= start && k < end {
                x0.get_or_insert(acc);
                x1 = acc + w;
            }
            acc += w;
        }
        if newline && seg.last {
            x0.get_or_insert(acc);
            x1 = acc + 1;
        }
        let Some(x0) = x0 else {
            continue;
        };
        let x1 = x1.min(seg.width as usize);
        if x0 >= x1 {
            continue;
        }
        let y = inner.y + i as u16;
        for dx in x0..x1 {
            let x = inner.x + seg.x + dx as u16;
            if x >= inner.x + inner.width {
                break;
            }
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_bg(th().ui.text_selection_bg);
                cell.set_fg(th().ui.text_selection_fg);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn src(lines: &[&str], _tabs: Tabs) -> Vec<String> {
        lines.iter().map(|s| s.to_string()).collect()
    }

    fn seg(line: usize, start: usize, len: usize, x: u16, width: u16, last: bool) -> Seg {
        Seg {
            line,
            start,
            len,
            x,
            width,
            last,
        }
    }

    fn pos(line: usize, offset: usize) -> TextPos {
        TextPos { line, offset }
    }

    fn sel(side: usize, a: TextPos, h: TextPos) -> TextSelection {
        TextSelection {
            side,
            anchor: a,
            head: h,
        }
    }

    #[test]
    fn a_column_maps_to_the_char_under_it_past_the_gutter() {
        let lines = src(&["hello world"], Tabs::OneSpace);
        let s = Lines {
            lines: &lines,
            tabs: Tabs::OneSpace,
        };
        let rows = [[Some(seg(0, 0, 11, 5, 40, true)), None]];
        assert_eq!(pos_at(&rows, 0, 0, 5, &s), Some(pos(0, 0)));
        assert_eq!(pos_at(&rows, 0, 0, 11, &s), Some(pos(0, 6)));
        // The gutter is before the text: the line's first char.
        assert_eq!(pos_at(&rows, 0, 0, 1, &s), Some(pos(0, 0)));
        // Past the text: the end of the line.
        assert_eq!(pos_at(&rows, 0, 0, 30, &s), Some(pos(0, 11)));
    }

    #[test]
    fn a_horizontal_scroll_shifts_every_column_onto_its_real_offset() {
        // h_offset 4: column 0 is the `«` standing in for char 3.
        let lines = src(&["0123456789"], Tabs::OneSpace);
        let s = Lines {
            lines: &lines,
            tabs: Tabs::OneSpace,
        };
        let rows = [[Some(seg(0, 3, 7, 0, 20, true)), None]];
        assert_eq!(pos_at(&rows, 0, 0, 1, &s), Some(pos(0, 4)));
        let sel = sel(0, pos(0, 4), pos(0, 6));
        assert_eq!(selected_text(&sel, &s), "456");
    }

    #[test]
    fn wrapped_rows_map_into_the_same_line_and_copy_it_joined() {
        // "alpha beta gamma" wrapped at 6: "alpha" / "beta" / "gamma",
        // each row covering the blank its break swallowed.
        let lines = src(&["alpha beta gamma"], Tabs::OneSpace);
        let s = Lines {
            lines: &lines,
            tabs: Tabs::OneSpace,
        };
        let rows = [
            [Some(seg(0, 0, 6, 0, 6, false)), None],
            [Some(seg(0, 6, 5, 0, 6, false)), None],
            [Some(seg(0, 11, 5, 0, 6, true)), None],
        ];
        let a = pos_at(&rows, 0, 0, 2, &s).unwrap();
        let h = pos_at(&rows, 2, 0, 1, &s).unwrap();
        assert_eq!((a, h), (pos(0, 2), pos(0, 12)));
        assert_eq!(selected_text(&sel(0, a, h), &s), "pha beta ga");
        // Past the text of a non-last row: that row's own last char.
        assert_eq!(pos_at(&rows, 1, 0, 5, &s), Some(pos(0, 10)));
    }

    #[test]
    fn a_selection_over_several_lines_copies_whole_middle_lines() {
        let lines = src(&["first", "second", "third"], Tabs::OneSpace);
        let s = Lines {
            lines: &lines,
            tabs: Tabs::OneSpace,
        };
        let t = selected_text(&sel(0, pos(2, 1), pos(0, 3)), &s);
        assert_eq!(t, "st\nsecond\nth");
        // From past the end of a line: the break, then the next line.
        let t = selected_text(&sel(0, pos(0, 5), pos(1, 2)), &s);
        assert_eq!(t, "\nsec");
    }

    #[test]
    fn expanded_tabs_are_copied_as_tabs() {
        let lines = src(&["\tx = 1;"], Tabs::Expand(4));
        let s = Lines {
            lines: &lines,
            tabs: Tabs::Expand(4),
        };
        assert_eq!(display(&lines[0], Tabs::Expand(4)), "    x = 1;");
        // A selection starting inside the expanded tab takes the tab.
        assert_eq!(selected_text(&sel(0, pos(0, 2), pos(0, 4)), &s), "\tx");
        assert_eq!(selected_text(&sel(0, pos(0, 4), pos(0, 9)), &s), "x = 1;");
    }

    #[test]
    fn a_wide_char_is_one_offset_covering_two_cells() {
        let lines = src(&["a\u{4e2d}b"], Tabs::OneSpace);
        let s = Lines {
            lines: &lines,
            tabs: Tabs::OneSpace,
        };
        let rows = [[Some(seg(0, 0, 3, 0, 20, true)), None]];
        assert_eq!(pos_at(&rows, 0, 0, 1, &s), Some(pos(0, 1)));
        assert_eq!(pos_at(&rows, 0, 0, 2, &s), Some(pos(0, 1)));
        assert_eq!(pos_at(&rows, 0, 0, 3, &s), Some(pos(0, 2)));
        assert_eq!(
            selected_text(&sel(0, pos(0, 1), pos(0, 2)), &s),
            "\u{4e2d}b"
        );
    }

    #[test]
    fn the_ellipsis_cell_stands_for_the_rest_of_the_line() {
        let lines = src(&["abcdefghijklmnop"], Tabs::OneSpace);
        let s = Lines {
            lines: &lines,
            tabs: Tabs::OneSpace,
        };
        // Drawn in 6 cells: "abcde…".
        let rows = [[Some(seg(0, 0, 16, 0, 6, true)), None]];
        let h = pos_at(&rows, 0, 0, 5, &s).unwrap();
        assert_eq!(h, pos(0, 16));
        assert_eq!(selected_text(&sel(0, pos(0, 2), h), &s), "cdefghijklmnop");
    }

    #[test]
    fn a_row_without_text_on_the_side_resolves_to_its_neighbors() {
        let lines = src(&["one", "two"], Tabs::OneSpace);
        let s = Lines {
            lines: &lines,
            tabs: Tabs::OneSpace,
        };
        // A header row between two lines; the second row is blank on the
        // before side.
        let rows = [
            [Some(seg(0, 0, 3, 0, 10, true)), None],
            [None, None],
            [Some(seg(1, 0, 3, 0, 10, true)), None],
        ];
        assert_eq!(pos_at(&rows, 1, 0, 4, &s), Some(pos(1, 0)));
        let tail = [rows[0], [None, None]];
        assert_eq!(pos_at(&tail, 1, 0, 4, &s), Some(pos(0, 3)));
    }

    #[test]
    fn split_sides_are_told_apart_by_the_separator_column() {
        assert_eq!(side_at(None, 50), 0);
        assert_eq!(side_at(Some(30), 29), 0);
        assert_eq!(side_at(Some(30), 30), 1);
        let before = src(&["old"], Tabs::OneSpace);
        let after = src(&["new"], Tabs::OneSpace);
        let rows = [[
            Some(seg(0, 0, 3, 7, 20, true)),
            Some(seg(0, 0, 3, 37, 20, true)),
        ]];
        let b = Lines {
            lines: &before,
            tabs: Tabs::OneSpace,
        };
        let a = Lines {
            lines: &after,
            tabs: Tabs::OneSpace,
        };
        // A column on the after half, read against the after side.
        assert_eq!(pos_at(&rows, 0, 1, 38, &a), Some(pos(0, 1)));
        assert_eq!(selected_text(&sel(1, pos(0, 0), pos(0, 2)), &a), "new");
        assert_eq!(selected_text(&sel(0, pos(0, 0), pos(0, 2)), &b), "old");
    }

    #[test]
    fn double_click_takes_the_word_under_the_pointer() {
        let lines = src(&["let foo_bar = 1;"], Tabs::OneSpace);
        let s = Lines {
            lines: &lines,
            tabs: Tabs::OneSpace,
        };
        assert_eq!(word_at(pos(0, 6), &s), Some((pos(0, 4), pos(0, 10))));
        assert_eq!(word_at(pos(0, 40), &s), None);
        let (a, h) = word_at(pos(0, 5), &s).unwrap();
        assert_eq!(selected_text(&sel(0, a, h), &s), "foo_bar");
    }

    #[test]
    fn paint_highlights_only_the_selected_cells_of_the_selected_side() {
        let lines = src(&["abcdef", "gh"], Tabs::OneSpace);
        let s = Lines {
            lines: &lines,
            tabs: Tabs::OneSpace,
        };
        let inner = Rect::new(0, 0, 20, 2);
        let mut buf = Buffer::empty(inner);
        let rows = [
            [Some(seg(0, 0, 6, 2, 10, true)), None],
            [Some(seg(1, 0, 2, 2, 10, true)), None],
        ];
        paint(&mut buf, inner, &rows, &sel(0, pos(0, 3), pos(1, 0)), &s);
        let lit = |x: u16, y: u16| buf[(x, y)].bg == th().ui.text_selection_bg;
        // Row 0: "def" at cells 5..8, plus the line break at 8.
        let row0: Vec<u16> = (0..20).filter(|&x| lit(x, 0)).collect();
        assert_eq!(row0, vec![5, 6, 7, 8]);
        // Row 1: only "g".
        let row1: Vec<u16> = (0..20).filter(|&x| lit(x, 1)).collect();
        assert_eq!(row1, vec![2]);
        // The other side's selection paints nothing here.
        let mut other = Buffer::empty(inner);
        paint(&mut other, inner, &rows, &sel(1, pos(0, 0), pos(1, 1)), &s);
        assert!((0..20).all(|x| other[(x, 0)].bg != th().ui.text_selection_bg));
    }
}
