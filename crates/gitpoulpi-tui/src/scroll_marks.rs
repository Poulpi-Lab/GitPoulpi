//! The overview ruler drawn on a vertical scrollbar: one tick per place in
//! the document that matters (a change, the caret, another occurrence of the
//! selected text), projected onto the track so the whole file can be read at
//! a glance -- the strip editors put beside the scrollbar.
//!
//! Purely visual. The thumb's and the track's hit rects are registered from
//! `scrollbar::Track` and never look at this module, so a marker can not
//! steal a click or a drag: pressing on a tick is pressing on the track.
//!
//! Three steps, each a pure function so they are unit-tested without a
//! terminal: [`occurrences`] finds the rows to mark for the selected text,
//! [`project`] folds rows onto track cells, [`paint`] draws them.

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::Frame;

use crate::scrollbar::Track;
use crate::theme::th;

/// What a tick stands for. Declaration order is paint priority, lowest first:
/// when several land in one track cell the last one wins.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Mark {
    Add,
    Del,
    /// A deletion paired with an addition (a modified line).
    Modified,
    /// A review comment thread (or a pending comment, or a suggestion) sits
    /// on this row: a dot, so it reads apart from a change.
    #[cfg_attr(not(all(feature = "hosting", feature = "review")), allow(dead_code))]
    Thread,
    Occurrence,
    /// A merge conflict that is handled (picked or edited).
    ConflictDone,
    /// A merge conflict still waiting for a choice.
    ConflictOpen,
    /// The merge editor's current conflict.
    ConflictCurrent,
    Cursor,
}

/// The mark of each cell of a track `track_len` cells long, for `total`
/// document rows. `rows` are `(row, mark)` pairs; rows past `total` are
/// ignored. The result is always `track_len` long.
pub(crate) fn project(
    track_len: u16,
    total: usize,
    rows: impl IntoIterator<Item = (usize, Mark)>,
) -> Vec<Option<Mark>> {
    let len = track_len as usize;
    let mut cells: Vec<Option<Mark>> = vec![None; len];
    if len == 0 || total == 0 {
        return cells;
    }
    for (row, mark) in rows {
        if row >= total {
            continue;
        }
        let cell = (row * len / total).min(len - 1);
        cells[cell] = Some(match cells[cell] {
            // A cell holding both an addition and a deletion reads as a
            // modification, whatever order they arrive in.
            Some(Mark::Add) if mark == Mark::Del => Mark::Modified,
            Some(Mark::Del) if mark == Mark::Add => Mark::Modified,
            Some(old) => old.max(mark),
            None => mark,
        });
    }
    cells
}

/// Whether `needle` marks anything. Only a single-line, non-blank needle
/// does: a multi-line selection has no "every occurrence" to show, and a
/// blank one would light up the whole file. The one gate both the ruler
/// ([`occurrences`]) and the in-text tint ([`match_ranges`]) go through.
pub(crate) fn valid_needle(needle: &str) -> bool {
    !(needle.is_empty() || needle.contains('\n') || needle.trim().is_empty())
}

/// Every non-overlapping occurrence of `needle` in `line`, as byte ranges.
/// The single source of "where the selected text occurs": the ruler and the
/// text tint both read it, so they can never disagree.
pub(crate) fn match_ranges(needle: &str, line: &str) -> Vec<(usize, usize)> {
    if !valid_needle(needle) {
        return Vec::new();
    }
    line.match_indices(needle)
        .map(|(a, m)| (a, a + m.len()))
        .collect()
}

/// The document rows on which `needle` occurs, at most once per row.
pub(crate) fn occurrences<'a>(
    needle: &str,
    lines: impl IntoIterator<Item = &'a str>,
) -> Vec<usize> {
    if !valid_needle(needle) {
        return Vec::new();
    }
    lines
        .into_iter()
        .enumerate()
        .filter(|(_, line)| !match_ranges(needle, line).is_empty())
        .map(|(i, _)| i)
        .collect()
}

/// The soft background an occurrence of the selected text gets in the text
/// itself: the ruler tick's amber, blended into whatever background the
/// cell already has (a diff's add/remove tint stays readable underneath).
pub(crate) fn occurrence_bg(existing: Option<Color>) -> Color {
    let (r, g, b) = match existing {
        Some(Color::Rgb(r, g, b)) => (r, g, b),
        // The theme's minimap base (the dark base by default).
        _ => match th().minimap.bg {
            Color::Rgb(r, g, b) => (r, g, b),
            _ => (24, 26, 38),
        },
    };
    let (tr, tg, tb) = match th().scrollbar.marks_occurrence {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (240, 200, 90),
    };
    let mix = |a: u8, t: u8| ((a as u16 * 70 + t as u16 * 30) / 100) as u8;
    Color::Rgb(mix(r, tr), mix(g, tg), mix(b, tb))
}

/// The glyph and color a mark is painted with. A half block, so the
/// thumb's own color stays readable on the other half of the cell.
pub(crate) fn look(mark: Mark) -> (&'static str, Color) {
    let s = &th().scrollbar;
    match mark {
        Mark::Add => ("\u{2590}", s.marks_add),
        Mark::Del => ("\u{2590}", s.marks_del),
        Mark::Modified => ("\u{2590}", s.marks_modified),
        Mark::Thread => ("\u{25cf}", s.marks_conflict_current),
        Mark::Occurrence => ("\u{258c}", s.marks_occurrence),
        Mark::ConflictDone => ("\u{2590}", s.marks_add),
        Mark::ConflictOpen => ("\u{2590}", s.marks_conflict_open),
        Mark::ConflictCurrent => ("\u{25c6}", s.marks_conflict_current),
        Mark::Cursor => ("\u{25c6}", s.marks_cursor),
    }
}

/// Paints projected marks over an already-drawn vertical scrollbar,
/// keeping each cell's background (thumb or track).
pub(crate) fn paint(frame: &mut Frame, track: Track, cells: &[Option<Mark>]) {
    paint_buf(frame.buffer_mut(), track, cells);
}

/// [`paint`] on a bare buffer (the merge editor paints into one).
pub(crate) fn paint_buf(buf: &mut ratatui::buffer::Buffer, track: Track, cells: &[Option<Mark>]) {
    let area: Rect = buf.area;
    for (y, mark) in cells.iter().enumerate() {
        let Some(mark) = mark else { continue };
        // The cells span the inner track; the end arrows stay visible.
        let y = y as u16 + track.arrow;
        let row = track.rect.y + y;
        if y + track.arrow >= track.rect.height || row >= area.y + area.height {
            continue;
        }
        let (symbol, fg) = look(*mark);
        let cell = &mut buf[(track.rect.x, row)];
        cell.set_symbol(symbol);
        cell.set_style(Style::default().fg(fg).bg(cell.bg));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn rows_land_proportionally_on_the_track() {
        let cells = project(10, 100, [(0, Mark::Add), (55, Mark::Del), (99, Mark::Add)]);
        assert_eq!(cells.len(), 10);
        assert_eq!(cells[0], Some(Mark::Add));
        assert_eq!(cells[5], Some(Mark::Del));
        assert_eq!(cells[9], Some(Mark::Add));
        assert_eq!(cells.iter().flatten().count(), 3);
    }

    #[test]
    fn the_cursor_beats_everything_and_an_add_with_a_delete_is_a_modification() {
        let cells = project(
            4,
            8,
            [
                (0, Mark::Add),
                (1, Mark::Del),
                (2, Mark::Occurrence),
                (3, Mark::Cursor),
                (2, Mark::Add),
            ],
        );
        assert_eq!(cells[0], Some(Mark::Modified));
        assert_eq!(cells[1], Some(Mark::Cursor), "rows 2 and 3 share a cell");
    }

    #[test]
    fn out_of_range_rows_and_empty_tracks_are_harmless() {
        assert!(project(0, 10, [(1, Mark::Add)]).is_empty());
        assert_eq!(project(3, 0, [(0, Mark::Add)]), vec![None; 3]);
        assert_eq!(project(3, 5, [(5, Mark::Add)]), vec![None; 3]);
    }

    #[test]
    fn occurrences_find_every_row_holding_the_text_once_each() {
        let lines = ["let foo = 1;", "bar", "foo(foo)", "FOO"];
        assert_eq!(occurrences("foo", lines.iter().copied()), vec![0, 2]);
        assert!(occurrences("", lines.iter().copied()).is_empty());
        assert!(occurrences("  ", lines.iter().copied()).is_empty());
        assert!(occurrences("a\nb", lines.iter().copied()).is_empty());
    }

    #[test]
    fn painting_keeps_the_thumb_background_and_only_touches_marked_cells() {
        let mut terminal = Terminal::new(TestBackend::new(20, 12)).unwrap();
        let track = Track::vertical(Rect::new(0, 0, 20, 12), 30, 0).unwrap();
        let cells = project(track.inner_len(), 30, [(0, Mark::Add), (29, Mark::Cursor)]);
        terminal
            .draw(|f| {
                crate::scrollbar::render(f, track, true, false);
                paint(f, track, &cells);
            })
            .unwrap();
        let buf = terminal.backend().buffer();
        assert_eq!(
            buf[(track.rect.x, track.rect.y)].symbol(),
            "\u{2191}",
            "arrow stays"
        );
        let top = &buf[(track.rect.x, track.rect.y + 1)];
        assert_eq!(top.symbol(), "\u{2590}");
        assert_eq!(top.fg, th().scrollbar.marks_add);
        assert_eq!(top.bg, th().scrollbar.thumb, "the thumb stays visible");
        let last = &buf[(track.rect.x, track.rect.bottom() - 2)];
        assert_eq!(last.symbol(), "\u{25c6}");
        let mid = &buf[(track.rect.x, track.rect.y + 6)];
        assert_eq!(mid.symbol(), " ");
    }
}
