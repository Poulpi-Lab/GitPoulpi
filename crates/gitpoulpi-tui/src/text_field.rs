//! The visible window of a single-line text input.
//!
//! Every type-into field in this UI (the Log's find bar, the command
//! palette, the Refs sidebar's filter, the commit-detail file filter) draws
//! its draft through here. A field's on-screen extent must not depend on
//! what was typed: a verbatim draft slid the find bar's counter and buttons
//! with every keystroke, and grew the other fields until ratatui clipped them
//! and took the caret off the right edge.
//!
//! A text input's width is chrome, not content. `field_window` reserves a
//! fixed number of columns and scrolls the draft inside them.
//!
//! With the caret anywhere and a selection, `edit_line` / `edit_lines` draw
//! a field from its text and its `text_edit::Caret` (scrolled to the caret,
//! selection highlighted), and `offset_at` maps a click back to a byte
//! offset through the same layout. With the caret at the end they draw
//! exactly what `field_window` + a caret glyph does.

use crate::th;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::display_width;
use crate::text_edit::Caret;

/// Left-elision marker. Text presentation, single column, no VS16 --
/// same rule as the find bar's own glyphs.
const ELLIPSIS: &str = "\u{2026}";

/// The slice of `text` to draw in a field `width` columns wide whose caret
/// (`caret_width` columns, 0 when the field is not focused) sits at the end
/// of the text, plus the number of blank columns to draw *after* the caret
/// so that text + caret + padding occupies exactly `width`.
///
/// Two things this has to get right:
///
/// - **Measure in terminal columns, not chars.** A CJK or emoji draft
///   char-counts as one and paints as two; scrolling by chars lets the field
///   overflow its own rect and shove whatever is beside it (the find bar's
///   buttons) off the edge. `display_width` is the measure the ref pills
///   use.
/// - **Keep the caret's column stable.** When the draft overflows, the
///   leftover column a double-width grapheme could not fill is spent right
///   after the ellipsis rather than after the caret, so the caret stays
///   flush against the field's right edge instead of twitching as wide
///   characters scroll past.
pub(crate) fn field_window(text: &str, width: usize, caret_width: usize) -> (String, usize) {
    let avail = width.saturating_sub(caret_width);
    let text_width = display_width(text);
    if text_width <= avail {
        return (text.to_string(), avail - text_width);
    }
    if avail <= 1 {
        // No room for both a marker and any text: show neither rather
        // than a bare ellipsis claiming to be the query.
        return (String::new(), avail);
    }
    // Take graphemes-as-chars from the *end*, i.e. from the caret
    // backwards, so the characters just typed are the ones that survive.
    let budget = avail - 1;
    let mut tail_width = 0usize;
    let mut take = 0usize;
    for c in text.chars().rev() {
        let w = display_width(&c.to_string());
        if tail_width + w > budget {
            break;
        }
        tail_width += w;
        take += 1;
    }
    let tail: String = {
        let chars: Vec<char> = text.chars().collect();
        chars[chars.len() - take..].iter().collect()
    };
    let slack = budget - tail_width;
    (format!("{ELLIPSIS}{}{tail}", " ".repeat(slack)), 0)
}

/// The scrollbar of a multi-line field whose text rows are `rows`
/// (drawn over the field's rightmost column): `None` when every line
/// fits. `top` is the first visible line.
pub(crate) fn field_track(rows: Rect, text: &str, top: usize) -> Option<crate::scrollbar::Track> {
    if rows.height < 2 || rows.y == 0 || rows.width < 3 {
        return None;
    }
    let body = Rect::new(rows.right() - 2, rows.y - 1, 2, rows.height + 2);
    crate::scrollbar::Track::vertical(body, text.split('\n').count(), top)
}

/// The spans of an unfocused single-line input: the text windowed into
/// `width` columns (`field_window`), padded so `style`'s background
/// covers the field's full width.
pub(crate) fn resting_spans(text: &str, width: usize, style: Style) -> Vec<Span<'static>> {
    let (shown, pad) = field_window(text, width, 0);
    let mut out = vec![Span::styled(shown, style)];
    if pad > 0 {
        out.push(Span::styled(" ".repeat(pad), style));
    }
    out
}

// ---------------------------------------------------------------------
// Drawing an editable field: caret anywhere, selection, scrolling
// ---------------------------------------------------------------------

/// The styles one editable field is drawn with.
#[derive(Debug, Clone, Copy)]
pub(crate) struct EditLook {
    /// The text.
    pub(crate) text: Style,
    /// The caret glyph drawn after the last character when the caret
    /// is at the end of its line (`_` in the composer, `█` in dialogs).
    pub(crate) caret_glyph: &'static str,
    /// The caret glyph's own style.
    pub(crate) caret: Style,
    /// The blank columns after the text (and the caret).
    pub(crate) pad: Style,
}

impl EditLook {
    /// Text, caret glyph and padding all in `text`'s style.
    pub(crate) fn new(text: Style, caret_glyph: &'static str) -> Self {
        Self {
            text,
            caret_glyph,
            caret: text,
            pad: text,
        }
    }

    pub(crate) fn caret_style(mut self, caret: Style) -> Self {
        self.caret = caret;
        self
    }

    fn selection(&self) -> Style {
        self.text
            .bg(th().ui.text_selection_bg)
            .fg(th().ui.text_selection_fg)
    }

    /// A caret in the middle of the text is the character under it in
    /// reverse video - a block caret, which takes no column of its own
    /// and so never shifts the text as it moves.
    fn caret_on_char(&self) -> Style {
        self.text.add_modifier(Modifier::REVERSED)
    }
}

/// One char of a line: its byte offset in the whole text, and columns.
#[derive(Debug, Clone, Copy)]
struct CharCell {
    byte: usize,
    ch: char,
    width: usize,
}

fn cells(text: &str, start: usize) -> Vec<CharCell> {
    text.char_indices()
        .map(|(i, ch)| CharCell {
            byte: start + i,
            ch,
            width: display_width(ch.encode_utf8(&mut [0; 4])),
        })
        .collect()
}

/// Which chars of one line are on screen: `start..end` (char indices),
/// drawn after `lead` columns (the left-scroll ellipsis, plus the odd
/// column a wide char couldn't fill when the tail is right-aligned).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LineWindow {
    start: usize,
    end: usize,
    lead: usize,
}

fn width_of(cells: &[CharCell]) -> usize {
    cells.iter().map(|c| c.width).sum()
}

/// Lays out the caret's line: scrolls (from last frame's `stored` scroll, as
/// little as possible) so the caret (the char under it, or the end) is
/// visible in `avail` columns. The same rules as [`field_window`] when the
/// caret is at the end: tail visible, flush right, the leftover column spent
/// after the ellipsis.
fn caret_line_window(cells: &[CharCell], caret: usize, stored: usize, avail: usize) -> LineWindow {
    let n = cells.len();
    if width_of(cells) <= avail {
        return LineWindow {
            start: 0,
            end: n,
            lead: 0,
        };
    }
    if avail <= 1 {
        return LineWindow {
            start: n,
            end: n,
            lead: 0,
        };
    }
    let cols = |s: usize| avail - usize::from(s > 0);
    let caret = caret.min(n);
    let mut s = stored.min(n);
    // Never leave blank columns on the right while text is cut off on
    // the left (the text got shorter since last frame).
    while s > 0 && width_of(&cells[s - 1..]) <= cols(s - 1) {
        s -= 1;
    }
    if caret < s {
        s = caret;
    }
    let need_end = if caret < n { caret + 1 } else { n };
    while s < caret && width_of(&cells[s..need_end]) > cols(s) {
        s += 1;
    }
    window_from(cells, s, avail)
}

/// The window of a line drawn from char `s` on.
fn window_from(cells: &[CharCell], s: usize, avail: usize) -> LineWindow {
    let n = cells.len();
    let s = s.min(n);
    let cols = avail.saturating_sub(usize::from(s > 0));
    let mut used = 0;
    let mut end = s;
    while end < n && used + cells[end].width <= cols {
        used += cells[end].width;
        end += 1;
    }
    let ellipsis = usize::from(s > 0);
    // A right-aligned tail spends the odd column after the ellipsis.
    let slack = if s > 0 && end == n { cols - used } else { 0 };
    LineWindow {
        start: s,
        end,
        lead: ellipsis + slack,
    }
}

/// The lines of `text` as (byte start, byte end) pairs.
fn line_ranges(text: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut start = 0;
    for (i, c) in text.char_indices() {
        if c == '\n' {
            out.push((start, i));
            start = i + 1;
        }
    }
    out.push((start, text.len()));
    out
}

/// Everything a render and a click both need to agree on.
struct Placement {
    lines: Vec<(usize, usize)>,
    caret_line: usize,
    caret_col: usize,
    top: usize,
    scroll: usize,
    avail: usize,
}

fn place(text: &str, caret: &Caret, width: usize, rows: usize, focused: bool) -> Placement {
    let lines = line_ranges(text);
    let cursor = caret.cursor(text);
    let caret_line = lines.iter().rposition(|(a, _)| *a <= cursor).unwrap_or(0);
    let caret_col = text[lines[caret_line].0..cursor].chars().count();
    let rows = rows.max(1);
    let mut top = caret.top.get().min(lines.len().saturating_sub(1));
    if lines.len() <= rows {
        top = 0;
    }
    if caret_line < top {
        top = caret_line;
    } else if caret_line >= top + rows {
        top = caret_line + 1 - rows;
    }
    let avail = width.saturating_sub(usize::from(focused));
    let (a, b) = lines[caret_line];
    let caret_cells = cells(&text[a..b], a);
    let scroll = caret_line_window(&caret_cells, caret_col, caret.scroll.get(), avail).start;
    Placement {
        lines,
        caret_line,
        caret_col,
        top,
        scroll,
        avail,
    }
}

impl Placement {
    fn window(&self, text: &str, line: usize) -> (Vec<CharCell>, LineWindow) {
        let (a, b) = self.lines[line];
        let line_cells = cells(&text[a..b], a);
        let window = if line == self.caret_line {
            caret_line_window(&line_cells, self.caret_col, self.scroll, self.avail)
        } else {
            window_from(&line_cells, self.scroll, self.avail)
        };
        (line_cells, window)
    }
}

/// Where the caret sits within the `width` x `rows` area `edit_lines` drew
/// (row, column), for anchoring a popup to it. Call after drawing, so the
/// caret's scroll is settled.
#[cfg_attr(not(feature = "ai"), allow(dead_code))] // only the AI instructions editor uses it
pub(crate) fn caret_cell(text: &str, caret: &Caret, width: usize, rows: usize) -> (usize, usize) {
    let p = place(text, caret, width, rows, true);
    let col = p.caret_col.saturating_sub(p.scroll) + usize::from(p.scroll > 0);
    (p.caret_line.saturating_sub(p.top), col)
}

/// Draws an editable field's text as up to `rows` lines of exactly `width`
/// columns each: scrolled so the caret is visible, the selection
/// highlighted, the caret drawn (when `focused`) as a reversed char mid-text
/// or as `look.caret_glyph` at a line's end. A single-line field passes
/// `rows == 1`.
///
/// `caret` is `None` for a field that does not have the keyboard: it is laid
/// out as if its caret were at the end with nothing selected, which for an
/// overflowing text shows its tail, as [`field_window`] does. The scroll
/// this settles on is written back into the caret, where a click on the
/// field reads it from ([`offset_at`]).
pub(crate) fn edit_lines(
    text: &str,
    caret: Option<&Caret>,
    width: usize,
    rows: usize,
    focused: bool,
    look: EditLook,
) -> Vec<Line<'static>> {
    let fallback = Caret::default();
    let caret = caret.unwrap_or(&fallback);
    let p = place(text, caret, width, rows, focused);
    caret.scroll.set(p.scroll);
    caret.top.set(p.top);
    let selection = caret.selection(text);
    let cursor = caret.cursor(text);
    let mut out = Vec::new();
    for line in p.top..(p.top + rows.max(1)).min(p.lines.len()) {
        let (line_cells, window) = p.window(text, line);
        let mut spans: Vec<Span<'static>> = Vec::new();
        let mut push = |s: &str, style: Style| {
            if s.is_empty() {
                return;
            }
            match spans.last_mut() {
                Some(last) if last.style == style => last.content.to_mut().push_str(s),
                _ => spans.push(Span::styled(s.to_string(), style)),
            }
        };
        let mut used = 0;
        if window.start > 0 && window.lead > 0 {
            push(ELLIPSIS, look.text);
            push(&" ".repeat(window.lead - 1), look.text);
            used += window.lead;
        }
        for c in &line_cells[window.start..window.end] {
            let style = if selection.is_some_and(|(a, b)| c.byte >= a && c.byte < b) {
                look.selection()
            } else if focused && c.byte == cursor {
                look.caret_on_char()
            } else {
                look.text
            };
            push(c.ch.encode_utf8(&mut [0; 4]), style);
            used += c.width;
        }
        let caret_at_end = focused && line == p.caret_line && p.caret_col == line_cells.len();
        if caret_at_end && used < width {
            push(look.caret_glyph, look.caret);
            used += display_width(look.caret_glyph);
        }
        push(&" ".repeat(width.saturating_sub(used)), look.pad);
        out.push(Line::from(spans));
    }
    out
}

/// [`edit_lines`] for a single-line field.
pub(crate) fn edit_line(
    text: &str,
    caret: Option<&Caret>,
    width: usize,
    focused: bool,
    look: EditLook,
) -> Line<'static> {
    edit_lines(text, caret, width, 1, focused, look)
        .pop()
        .unwrap_or_default()
}

/// The byte offset a click at (`col`, `row`), relative to the field's text
/// rect `width` columns wide, lands on, through the scroll the last
/// [`edit_lines`] drew with. A click on a char puts the caret before it (the
/// block caret then covers that char); a click past a line's end puts it at
/// that end.
pub(crate) fn offset_at(
    text: &str,
    caret: &Caret,
    width: usize,
    multiline: bool,
    col: usize,
    row: usize,
) -> usize {
    // Enough rows that the `top` the last render settled on is kept: a
    // click must never re-scroll what it is pointing into.
    let rows = caret.top.get() + row + 1;
    let p = place(text, caret, width, rows, true);
    let line = if multiline {
        (caret.top.get() + row).min(p.lines.len() - 1)
    } else {
        p.caret_line
    };
    let (line_cells, window) = p.window(text, line);
    let line_end = p.lines[line].1;
    if col < window.lead {
        return line_cells.get(window.start).map_or(line_end, |c| c.byte);
    }
    let mut x = window.lead;
    for c in &line_cells[window.start..window.end] {
        if col < x + c.width {
            return c.byte;
        }
        x += c.width;
    }
    line_cells.get(window.end).map_or(line_end, |c| c.byte)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The property the whole module exists for: whatever is typed, the
    /// field occupies the same number of columns.
    #[test]
    fn the_field_is_exactly_as_wide_as_asked_whatever_is_typed() {
        for text in [
            "",
            "a",
            "fix",
            &"x".repeat(59),
            &"y".repeat(400),
            "日本語のコミットメッセージ",
            "🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀🚀",
        ] {
            for width in [0usize, 1, 2, 3, 8, 20, 46] {
                let (shown, pad) = field_window(text, width, 1);
                let total = display_width(&shown) + 1usize.min(width) + pad;
                assert!(
                    total <= width || width == 0,
                    "{text:?} at width {width}: drew {total} columns ({shown:?} + caret + {pad})"
                );
                if width > 1 {
                    assert_eq!(
                        display_width(&shown) + 1 + pad,
                        width,
                        "{text:?} at width {width}: field must fill its rect exactly, \
                         got {shown:?} + caret + {pad} blanks"
                    );
                }
            }
        }
    }

    /// Overflow scrolls rather than grows, and what stays on screen is
    /// the end of the draft - where the caret is.
    #[test]
    fn an_overflowing_draft_keeps_its_tail_against_the_caret() {
        let (shown, pad) = field_window("abcdefghijklmnop", 6, 1);
        assert_eq!(shown, "\u{2026}mnop");
        assert_eq!(pad, 0, "a scrolled field has no slack left to pad with");
    }

    /// A short draft sits at the left with the caret right after it --
    /// the padding is what holds the field's width open, and it must come
    /// after the caret, not before it.
    #[test]
    fn a_short_draft_is_left_aligned_with_the_caret_against_it() {
        let (shown, pad) = field_window("fix", 10, 1);
        assert_eq!(shown, "fix");
        assert_eq!(pad, 6);
    }

    /// Wide graphemes are measured in columns. Char-counting here reports
    /// four columns for four emoji and overflows the rect by four cells.
    #[test]
    fn wide_graphemes_are_measured_in_columns_not_chars() {
        // 5 columns of window: ellipsis + two 2-column rockets, caret
        // flush right.
        let (shown, pad) = field_window("🚀🚀🚀🚀", 6, 1);
        assert_eq!(shown, "\u{2026}🚀🚀");
        assert_eq!(display_width(&shown) + 1 + pad, 6);
        // One more column than two rockets need: the odd column is spent
        // after the ellipsis, NOT after the caret, so the caret stays put
        // as wide characters scroll through the field.
        let (shown, pad) = field_window("🚀🚀🚀🚀", 7, 1);
        assert_eq!(shown, "\u{2026} 🚀🚀");
        assert_eq!(pad, 0);
        assert_eq!(display_width(&shown) + 1 + pad, 7);
    }

    /// An unfocused field has no caret and gives that column back to the
    /// text instead of leaving a hole.
    #[test]
    fn an_unfocused_field_spends_the_caret_column_on_text() {
        let (shown, pad) = field_window("abcdef", 4, 0);
        assert_eq!(shown, "\u{2026}def");
        assert_eq!(pad, 0);
    }

    #[test]
    fn a_degenerate_width_draws_nothing_rather_than_panicking() {
        assert_eq!(field_window("abc", 0, 1), (String::new(), 0));
        assert_eq!(field_window("abc", 1, 1), (String::new(), 0));
    }

    // -- edit_lines / offset_at --------------------------------------

    use crate::text_edit::Caret;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn look() -> EditLook {
        EditLook::new(Style::default(), "_")
    }

    fn flat(line: &Line) -> String {
        line.spans.iter().map(|s| s.content.as_ref()).collect()
    }

    fn press(caret: &mut Caret, text: &mut String, code: KeyCode, mods: KeyModifiers) {
        caret.on_key(text, &KeyEvent::new(code, mods), true);
    }

    /// With the caret at the end, the editable render is exactly the plain
    /// `field_window` + caret.
    #[test]
    fn a_caret_at_the_end_draws_like_field_window() {
        for (text, width) in [
            ("fix", 10),
            ("abcdefghijklmnop", 6),
            ("🚀🚀🚀🚀", 7),
            ("🚀🚀🚀🚀", 6),
        ] {
            let (shown, pad) = field_window(text, width, 1);
            let line = edit_line(text, None, width, true, look());
            assert_eq!(
                flat(&line),
                format!("{shown}_{}", " ".repeat(pad)),
                "{text:?}"
            );
            assert_eq!(display_width(&flat(&line)), width);
        }
    }

    #[test]
    fn a_caret_moved_left_scrolls_the_window_back_and_is_a_reversed_char() {
        let mut text = "abcdefghijklmnop".to_string();
        let mut caret = Caret::default();
        press(&mut caret, &mut text, KeyCode::Home, KeyModifiers::NONE);
        let line = edit_line(&text, Some(&caret), 6, true, look());
        assert_eq!(flat(&line), "abcde ");
        assert!(line.spans[0]
            .style
            .add_modifier
            .contains(Modifier::REVERSED));
        assert_eq!(line.spans[0].content, "a");
    }

    #[test]
    fn the_selection_is_highlighted() {
        let mut text = "hello".to_string();
        let mut caret = Caret::default();
        press(&mut caret, &mut text, KeyCode::Left, KeyModifiers::SHIFT);
        press(&mut caret, &mut text, KeyCode::Left, KeyModifiers::SHIFT);
        let line = edit_line(&text, Some(&caret), 10, true, look());
        let sel: String = line
            .spans
            .iter()
            .filter(|s| s.style.bg == Some(th().ui.text_selection_bg))
            .map(|s| s.content.as_ref())
            .collect();
        assert_eq!(sel, "lo");
    }

    #[test]
    fn a_click_maps_through_wide_chars_and_horizontal_scroll() {
        let text = "日本語abc";
        let caret = Caret::default();
        edit_line(text, Some(&caret), 20, true, look());
        // Columns 0-1 are 日, 2-3 本: both halves of a wide char map to it.
        assert_eq!(offset_at(text, &caret, 20, false, 0, 0), 0);
        assert_eq!(offset_at(text, &caret, 20, false, 3, 0), 3);
        assert_eq!(offset_at(text, &caret, 20, false, 6, 0), 9); // 'a'
        assert_eq!(offset_at(text, &caret, 20, false, 15, 0), text.len());
        // Scrolled: "…mnop_" in 6 columns, col 1 is 'm'.
        let text = "abcdefghijklmnop";
        let caret = Caret::default();
        edit_line(text, Some(&caret), 6, true, look());
        assert_eq!(offset_at(text, &caret, 6, false, 1, 0), 12);
    }

    #[test]
    fn multiline_draws_each_line_and_follows_the_caret_down() {
        let text = "one\ntwo\nthree\nfour";
        let caret = Caret::default();
        let lines = edit_lines(text, Some(&caret), 8, 2, true, look());
        assert_eq!(lines.len(), 2);
        assert_eq!(flat(&lines[0]), "three   ");
        assert_eq!(flat(&lines[1]), "four_   ");
        // Row 0 of the field is line 2 ("three").
        assert_eq!(offset_at(text, &caret, 8, true, 1, 0), 9);
        assert_eq!(offset_at(text, &caret, 8, true, 7, 1), text.len());
    }
}
