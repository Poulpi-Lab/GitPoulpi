//! A reusable checker for finished `ratatui` buffers: the invariants a CJK
//! (double-width) UI must keep, written once and used by every layout test
//! and by the CJK render sweep (`cjk_sweep_tests`).
//!
//! ratatui stores a wide glyph in cell N and a blank continuation in N+1, and
//! its diff never emits N+1. Whatever sits in N+1 is therefore invisible, and
//! whatever was *meant* to be shown there (a border, a letter) is lost, so
//! the rules are about what a finished buffer must never contain:
//!
//! * **(a) wide cells are whole** ([`wide_violations`]): a wide glyph is
//!   followed by its blank continuation (a horizontal rule `─` under a title glyph
//!   is the one tolerated leftover: ratatui's `Span` keeps the cell it covers) and never starts in the last column;
//! * **(b) a box is intact** ([`box_violations_of`], for a popup's known
//!   rectangle): four corners, a side on every row, a rule along the
//!   bottom;
//! * **(c) nothing outside the widget** ([`changed_outside`]): rendering over
//!   a sentinel background must leave every cell outside the widget's area
//!   untouched;
//! * **(d) a column is aligned** ([`right_aligned_at`]): the cells of a column
//!   that ends flush at the same x (a menu's shortcut column).

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::display_width;

/// One row of `buf` as text; a wide glyph's continuation cell is skipped.
pub(crate) fn row_text(buf: &Buffer, y: u16) -> String {
    let mut out = String::new();
    let mut x = buf.area.x;
    while x < buf.area.right() {
        let symbol = buf[(x, y)].symbol();
        out.push_str(symbol);
        x += display_width(symbol).max(1) as u16;
    }
    out
}

/// Row `y` of `buf` restricted to the columns of `cols`, continuation cells
/// skipped (a glyph starting left of the range is not included).
pub(crate) fn row_text_in(buf: &Buffer, y: u16, cols: Rect) -> String {
    let mut out = String::new();
    let mut x = cols.x;
    while x < cols.right().min(buf.area.right()) {
        let symbol = buf[(x, y)].symbol();
        out.push_str(symbol);
        x += display_width(symbol).max(1) as u16;
    }
    out
}

/// The whole buffer as text, for a failure message.
pub(crate) fn dump(buf: &Buffer) -> String {
    (buf.area.y..buf.area.bottom())
        .map(|y| row_text(buf, y))
        .collect::<Vec<_>>()
        .join("\n")
}

fn is_blank(symbol: &str) -> bool {
    symbol.trim().is_empty()
}

fn is_rule(symbol: &str) -> bool {
    symbol
        .chars()
        .all(|c| ('\u{2500}'..='\u{257F}').contains(&c))
        && !symbol.is_empty()
}

/// Rule (a): a wide glyph keeps its blank continuation and never starts in
/// the last column. Messages are `(x, y): ...`.
pub(crate) fn wide_violations(buf: &Buffer) -> Vec<String> {
    let mut out = Vec::new();
    let right = buf.area.right();
    for y in buf.area.y..buf.area.bottom() {
        for x in buf.area.x..right {
            let cell = &buf[(x, y)];
            if cell.diff_option == ratatui::buffer::CellDiffOption::Skip {
                continue;
            }
            let symbol = cell.symbol();
            let w = display_width(symbol);
            if w < 2 {
                continue;
            }
            if x + 1 >= right {
                out.push(format!(
                    "({x},{y}): wide glyph {symbol:?} in the last column"
                ));
                continue;
            }
            for k in 1..w as u16 {
                if x + k >= right {
                    out.push(format!("({x},{y}): wide glyph {symbol:?} cut by the edge"));
                    break;
                }
                let next = buf[(x + k, y)].symbol();
                if !is_blank(next) && next != "\u{2500}" {
                    out.push(format!(
                        "({x},{y}): wide glyph {symbol:?} followed by {next:?}, not its blank continuation"
                    ));
                }
            }
        }
    }
    out
}

/// Cells a box side may run through: border glyphs, joints, a scrollbar's
/// track, thumb and arrows, the close cross.
fn is_side_like(symbol: &str) -> bool {
    matches!(
        symbol,
        "\u{2502}"
            | "\u{2503}"
            | "\u{251C}"
            | "\u{2524}"
            | "\u{253C}"
            | "\u{256D}"
            | "\u{256E}"
            | "\u{2570}"
            | "\u{256F}"
            | "\u{2191}"
            | "\u{2193}"
            | "\u{25B2}"
            | "\u{25BC}"
            | "\u{2591}"
            | "\u{2592}"
            | "\u{2588}"
            | "\u{25A0}"
            | "\u{2595}"
            | "\u{258F}"
            | "\u{2599}"
    ) || is_rule(symbol)
}

/// Rule (b): the rounded box that fills `rect` is whole: four corners, a
/// rule along the bottom, a side on every row. A side may be a scrollbar's
/// track, thumb or arrow (the bar replaces the right border where the list is
/// taller than its box), recognised by an arrow in the same column. Messages
/// are `(x, y): ...`.
pub(crate) fn box_violations_of(buf: &Buffer, rect: Rect) -> Vec<String> {
    let mut out = Vec::new();
    let rect = rect.intersection(buf.area);
    if rect.width < 2 || rect.height < 2 {
        return out;
    }
    let (x0, y0, x1, y1) = (rect.x, rect.y, rect.right() - 1, rect.bottom() - 1);
    for (x, y, want) in [
        (x0, y0, "\u{256D}"),
        (x1, y0, "\u{256E}"),
        (x0, y1, "\u{2570}"),
        (x1, y1, "\u{256F}"),
    ] {
        let s = buf[(x, y)].symbol();
        if s != want {
            out.push(format!(
                "({x},{y}): the corner of {rect:?} is {s:?}, not {want:?}"
            ));
        }
    }
    for x in x0 + 1..x1 {
        let s = buf[(x, y1)].symbol();
        if !is_rule(s) {
            out.push(format!("({x},{y1}): the bottom of {rect:?} is {s:?}"));
            break;
        }
    }
    let has_bar = (y0 + 1..y1).any(|y| {
        matches!(
            buf[(x1, y)].symbol(),
            "\u{2191}" | "\u{25B2}" | "\u{2193}" | "\u{25BC}"
        )
    });
    for y in y0 + 1..y1 {
        let l = buf[(x0, y)].symbol();
        if !is_side_like(l) {
            out.push(format!("({x0},{y}): the left side of {rect:?} is {l:?}"));
        }
        let r = buf[(x1, y)].symbol();
        if !is_side_like(r) && !(has_bar && is_blank(r)) {
            out.push(format!("({x1},{y}): the right side of {rect:?} is {r:?}"));
        }
    }
    out
}

/// Rule (b) for an overlay built of several stacked boxes (the palette's
/// input, list and hint band): the first and last column of `rect` hold a
/// border (or a scrollbar) on every row. Messages are `(x, y): ...`.
pub(crate) fn edges_violations(buf: &Buffer, rect: Rect) -> Vec<String> {
    let mut out = Vec::new();
    let rect = rect.intersection(buf.area);
    if rect.is_empty() {
        return out;
    }
    let (x0, x1) = (rect.x, rect.right() - 1);
    let has_bar = (rect.y..rect.bottom()).any(|y| {
        matches!(
            buf[(x1, y)].symbol(),
            "\u{2191}" | "\u{25B2}" | "\u{2193}" | "\u{25BC}"
        )
    });
    for y in rect.y..rect.bottom() {
        let l = buf[(x0, y)].symbol();
        if !is_side_like(l) {
            out.push(format!("({x0},{y}): the left edge of {rect:?} is {l:?}"));
        }
        let r = buf[(x1, y)].symbol();
        if !is_side_like(r) && !(has_bar && is_blank(r)) {
            out.push(format!("({x1},{y}): the right edge of {rect:?} is {r:?}"));
        }
    }
    out
}

/// Rule (a) over the whole buffer (rule (b) needs the box's rectangle, see
/// [`box_violations_of`]); panics with the screen on any violation.
pub(crate) fn assert_clean(what: &str, buf: &Buffer) {
    let all = wide_violations(buf);
    assert!(
        all.is_empty(),
        "{what}: {} render violation(s):\n  {}\n{}",
        all.len(),
        all.join("\n  "),
        dump(buf)
    );
}

/// Rule (c): the cells outside `area` that differ between `before` and
/// `after` (same size). A widget that wrote outside its area shows up here.
pub(crate) fn changed_outside(before: &Buffer, after: &Buffer, area: Rect) -> Vec<(u16, u16)> {
    let mut out = Vec::new();
    for y in after.area.y..after.area.bottom() {
        for x in after.area.x..after.area.right() {
            if area.contains((x, y).into()) {
                continue;
            }
            if before[(x, y)].symbol() != after[(x, y)].symbol() {
                out.push((x, y));
            }
        }
    }
    out
}

/// Rule (d): the display column at which `needle` ends in row `y`, or `None`
/// when the row does not hold it. Lines whose end columns differ are
/// misaligned.
pub(crate) fn ends_at(buf: &Buffer, y: u16, needle: &str) -> Option<u16> {
    let mut text = String::new();
    let mut ends = Vec::new();
    let mut x = buf.area.x;
    while x < buf.area.right() {
        let symbol = buf[(x, y)].symbol();
        let w = display_width(symbol).max(1) as u16;
        text.push_str(symbol);
        x += w;
        ends.push((text.len(), x));
    }
    let at = text.rfind(needle)? + needle.len();
    ends.iter().find(|(len, _)| *len == at).map(|(_, x)| *x)
}

/// Rule (d) as an assertion over several rows: every `(y, needle)` ends at
/// the same column.
pub(crate) fn right_aligned_at(buf: &Buffer, rows: &[(u16, String)]) -> Result<u16, String> {
    let mut col = None;
    for (y, needle) in rows {
        let Some(e) = ends_at(buf, *y, needle) else {
            return Err(format!("row {y} does not hold {needle:?}"));
        };
        match col {
            None => col = Some(e),
            Some(c) if c == e => {}
            Some(c) => return Err(format!("{needle:?} ends at {e} on row {y}, others at {c}")),
        }
    }
    col.ok_or_else(|| "no rows".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Style;

    fn buf(w: u16, h: u16) -> Buffer {
        Buffer::empty(Rect::new(0, 0, w, h))
    }

    #[test]
    fn an_overwritten_continuation_is_reported() {
        let mut b = buf(10, 1);
        b.set_stringn(0, 0, "将拣", 10, Style::default());
        assert!(wide_violations(&b).is_empty());
        // The terminal would never show the "x": the glyph is not whole.
        b[(1, 0)].set_symbol("x");
        assert_eq!(wide_violations(&b).len(), 1);
        // A wide glyph in the last column spills.
        let mut b = buf(4, 1);
        b[(3, 0)].set_symbol("将");
        assert_eq!(wide_violations(&b).len(), 1);
    }

    #[test]
    fn a_broken_box_is_reported() {
        let mut b = buf(8, 4);
        let rows = [
            "\u{256D}\u{2500}\u{2500}\u{2500}\u{2500}\u{256E}  ",
            "\u{2502}    \u{2502}  ",
            "\u{2502}    \u{2502}  ",
            "\u{2570}\u{2500}\u{2500}\u{2500}\u{2500}\u{256F}  ",
        ];
        for (y, r) in rows.iter().enumerate() {
            b.set_stringn(0, y as u16, r, 8, Style::default());
        }
        let rect = Rect::new(0, 0, 6, 4);
        assert!(box_violations_of(&b, rect).is_empty());
        b[(5, 1)].set_symbol("x");
        assert_eq!(box_violations_of(&b, rect).len(), 1);
        b[(5, 1)].set_symbol("\u{2502}");
        b[(0, 2)].set_symbol("a");
        assert_eq!(box_violations_of(&b, rect).len(), 1);
        b[(0, 2)].set_symbol("\u{2502}");
        b[(3, 3)].set_symbol("q");
        assert_eq!(box_violations_of(&b, rect).len(), 1);
        b[(3, 3)].set_symbol("\u{2500}");
        // A scrollbar track (blank) is a side only under an arrow.
        b[(5, 1)].set_symbol(" ");
        assert_eq!(box_violations_of(&b, rect).len(), 1);
        b[(5, 1)].set_symbol("\u{2191}");
        b[(5, 2)].set_symbol(" ");
        assert!(box_violations_of(&b, rect).is_empty());
    }

    #[test]
    fn alignment_is_measured_in_cells() {
        let mut b = buf(12, 2);
        b.set_stringn(0, 0, "将拣 Ctrl+G", 12, Style::default());
        b.set_stringn(0, 1, "a Ctrl+G", 12, Style::default());
        assert_eq!(ends_at(&b, 0, "Ctrl+G"), Some(11));
        assert!(right_aligned_at(&b, &[(0, "Ctrl+G".into()), (1, "Ctrl+G".into())]).is_err());
    }
}
