//! Intra-line ("changed words") emphasis for the diff views, terminal side.
//!
//! The pairing, the token diff and its budgets live in
//! `gitpoulpi_app::diff::intraline` and answer in chars. This file turns
//! those char ranges into terminal **cells** of the tab-expanded text (what
//! the row painter slices, scrolls and wraps by) and paints them.

use super::layout::expand_tabs;
use crate::cell_text::char_cells;
use gitpoulpi_app::diff::intraline as app;
use gitpoulpi_core::domain::{DiffLine, LineOrigin};
use ratatui::prelude::*;

/// `[start, end)` in cells.
pub(super) type CellRange = (usize, usize);

/// The cells the char ranges of `text` cover. A range of zero-width chars
/// only covers no cell and is dropped; ranges that touch are joined.
fn to_cells(text: &str, chars: &[app::CharRange]) -> Vec<CellRange> {
    if chars.is_empty() {
        return Vec::new();
    }
    let mut cum = Vec::with_capacity(text.len() + 1);
    cum.push(0usize);
    for c in text.chars() {
        cum.push(cum[cum.len() - 1] + char_cells(c));
    }
    let mut out: Vec<CellRange> = Vec::new();
    for &(s, e) in chars {
        let (from, to) = (cum[s], cum[e]);
        if to > from {
            match out.last_mut() {
                Some(last) if last.1 == from => last.1 = to,
                _ => out.push((from, to)),
            }
        }
    }
    out
}

/// The emphasis of every line of a hunk, by line index, in cells of the
/// tab-expanded content (see `gitpoulpi_app::diff::intraline::hunk_emphasis`).
pub(super) fn hunk_emphasis(lines: &[DiffLine]) -> Vec<Vec<CellRange>> {
    let chars = app::hunk_emphasis(lines, expand_tabs);
    chars
        .into_iter()
        .zip(lines)
        .map(|(ranges, line)| {
            if ranges.is_empty() || line.origin == LineOrigin::Context {
                Vec::new()
            } else {
                to_cells(&expand_tabs(&line.content), &ranges)
            }
        })
        .collect()
}

/// `piece` (what one drawn row shows of a line) as spans: the cells inside
/// `marked` in `emph`, the rest in `base`.
///
/// Column `c` of the piece is cell `cell_base + c - lead` of the line's
/// tab-expanded text; the first `lead` columns (a scroll marker) are never
/// marked. A zero-width char takes the mark of the char before it.
pub(super) fn emph_spans(
    piece: &str,
    (cell_base, lead): (usize, usize),
    marked: &[CellRange],
    base: Style,
    emph: Style,
) -> Vec<Span<'static>> {
    if marked.is_empty() {
        return vec![Span::styled(piece.to_string(), base)];
    }
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut run = String::new();
    let mut run_marked = false;
    let mut col = 0usize;
    let mut at = 0usize; // first range that may still contain the cell
    for c in piece.chars() {
        let w = char_cells(c);
        let is_marked = if w == 0 {
            run_marked
        } else if col < lead {
            false
        } else {
            let cell = cell_base + col - lead;
            while at < marked.len() && marked[at].1 <= cell {
                at += 1;
            }
            at < marked.len() && marked[at].0 <= cell
        };
        if is_marked != run_marked && !run.is_empty() {
            spans.push(Span::styled(
                std::mem::take(&mut run),
                if run_marked { emph } else { base },
            ));
        }
        run_marked = is_marked;
        run.push(c);
        col += w;
    }
    if !run.is_empty() {
        spans.push(Span::styled(run, if run_marked { emph } else { base }));
    }
    spans
}

/// The style of the marked words of an added or removed line: `base` with the
/// stronger background `emph_bg`. When the terminal's color depth would draw
/// `emph_bg` the same as the line's own `line_bg` (16 colors, or a theme with
/// no background of its own), bold and underline mark the words instead.
pub(super) fn emph_style(base: Style, line_bg: Color, emph_bg: Color) -> Style {
    emph_style_at(base, line_bg, emph_bg, crate::color_depth::active())
}

fn emph_style_at(
    base: Style,
    line_bg: Color,
    emph_bg: Color,
    depth: crate::color_depth::Depth,
) -> Style {
    if crate::color_depth::distinct_at(line_bg, emph_bg, depth) {
        base.bg(emph_bg)
    } else {
        base.add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    }
}

/// The marked-words style for a line of this origin, `None` for context.
pub(super) fn emph_for(origin: &LineOrigin, base: Style) -> Option<Style> {
    let diff = &crate::th().diff;
    match origin {
        LineOrigin::Add => Some(emph_style(base, diff.add_bg, diff.add_emph_bg)),
        LineOrigin::Delete => Some(emph_style(base, diff.del_bg, diff.del_emph_bg)),
        LineOrigin::Context => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_distinct_background_falls_back_to_bold_and_underline() {
        use crate::color_depth::Depth;
        let base = Style::default().fg(Color::Green);
        let (flat, strong) = (Color::Rgb(24, 52, 38), Color::Rgb(36, 88, 60));
        // Truecolor: the stronger background itself.
        assert_eq!(
            emph_style_at(base, flat, strong, Depth::True),
            base.bg(strong)
        );
        // A theme with no background of its own (the Terminal preset).
        let marked = emph_style_at(base, Color::Reset, Color::Reset, Depth::True);
        assert_eq!(marked.bg, None);
        assert!(marked
            .add_modifier
            .contains(Modifier::BOLD | Modifier::UNDERLINED));
        // Two tints the 16 colors draw alike: still told apart.
        assert!(!crate::color_depth::distinct_at(flat, strong, Depth::X16));
        let marked = emph_style_at(base, flat, strong, Depth::X16);
        assert!(marked
            .add_modifier
            .contains(Modifier::BOLD | Modifier::UNDERLINED));
    }

    #[test]
    fn spans_split_at_the_marked_cells() {
        let base = Style::default().fg(Color::Green);
        let emph = base.bg(Color::Red);
        let spans = emph_spans("abcdef", (0, 0), &[(2, 4)], base, emph);
        let parts: Vec<(&str, bool)> = spans
            .iter()
            .map(|s| (s.content.as_ref(), s.style == emph))
            .collect();
        assert_eq!(parts, [("ab", false), ("cd", true), ("ef", false)]);
        // A scrolled row: one marker column, then the line from cell 3.
        let spans = emph_spans("\u{ab}defg", (3, 1), &[(4, 5)], base, emph);
        let parts: Vec<(&str, bool)> = spans
            .iter()
            .map(|s| (s.content.as_ref(), s.style == emph))
            .collect();
        assert_eq!(parts, [("\u{ab}d", false), ("e", true), ("fg", false)]);
    }
}

#[cfg(test)]
mod cell_tests {
    use super::*;

    #[test]
    fn char_ranges_become_cell_ranges() {
        // Two wide chars take four cells: "aaa" at chars 3..6 is cells 5..8.
        let text = "\u{65e5}\u{672c} aaa end";
        assert_eq!(to_cells(text, &[(3, 6)]), vec![(5, 8)]);
        // A range over a lone combining mark covers no cell.
        assert_eq!(to_cells("\u{301}a", &[(0, 1)]), Vec::<CellRange>::new());
        // Touching ranges are joined.
        assert_eq!(to_cells("abcd", &[(0, 2), (2, 4)]), vec![(0, 4)]);
    }

    #[test]
    fn a_tab_is_expanded_before_the_ranges_are_taken() {
        let lines = vec![
            DiffLine {
                origin: LineOrigin::Delete,
                content: "\tfoo(a)".into(),
                staged: false,
            },
            DiffLine {
                origin: LineOrigin::Add,
                content: "\tfoo(b)".into(),
                staged: false,
            },
        ];
        let e = hunk_emphasis(&lines);
        let tab = crate::editor_buffer::tab_width();
        assert_eq!(e[0], vec![(tab + 4, tab + 5)]);
        assert_eq!(e[1], vec![(tab + 4, tab + 5)]);
    }
}
