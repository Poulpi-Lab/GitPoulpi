//! Cutting and padding text by terminal **cells**, never by chars.
//!
//! A CJK or emoji char takes two cells, a combining mark none. Anything
//! that builds a fixed-width column (the Split diff's two halves, a
//! horizontally scrolled row) must count cells, or the column edge, and
//! so the separator after it, lands somewhere else on every row that
//! holds a wide char. These helpers are the one place that does it: a
//! wide char that straddles a cut edge becomes a space, so every row is
//! exactly as wide as asked.

/// The cells one char takes (0 for a combining mark or control char).
pub(crate) fn char_cells(c: char) -> usize {
    let mut buf = [0u8; 4];
    crate::display_width(c.encode_utf8(&mut buf))
}

/// The cells `s` takes.
pub(crate) fn cells(s: &str) -> usize {
    crate::display_width(s)
}

/// `s` followed by spaces up to exactly `width` cells; unchanged when it
/// is already as wide or wider.
pub(crate) fn pad_to(s: &str, width: usize) -> String {
    let mut out = String::from(s);
    out.extend(std::iter::repeat(' ').take(width.saturating_sub(cells(s))));
    out
}

/// Where cell `cell` falls in `s`: the index of the first char that
/// starts at or after it, and whether a wide char straddles it (starts
/// before, ends after; the index is then that char's). `s.chars().count()`
/// when `cell` is past the end.
pub(crate) fn char_at_cell(s: &str, cell: usize) -> (usize, bool) {
    let mut col = 0usize;
    for (i, c) in s.chars().enumerate() {
        let w = char_cells(c);
        if col >= cell && w > 0 {
            return (i, false);
        }
        if col < cell && col + w > cell {
            return (i, true);
        }
        col += w;
    }
    (s.chars().count(), false)
}

/// The cells `skip..skip + width` of `s`. A wide char cut by either edge
/// is replaced by a space for the cells of it that are inside, so the
/// result is as wide as the text it came from allows and never wider than
/// `width`. A combining mark whose base was cut off goes with it.
pub(crate) fn slice_cells(s: &str, skip: usize, width: usize) -> String {
    let end = skip.saturating_add(width);
    let mut out = String::new();
    let mut col = 0usize;
    let mut kept_prev = false;
    for c in s.chars() {
        let w = char_cells(c);
        if w == 0 {
            if kept_prev {
                out.push(c);
            }
            continue;
        }
        let (a, b) = (col, col + w);
        col = b;
        if b <= skip {
            kept_prev = false;
            continue;
        }
        if a >= end {
            break;
        }
        if a >= skip && b <= end {
            out.push(c);
            kept_prev = true;
        } else {
            let inside = b.min(end) - a.max(skip);
            out.extend(std::iter::repeat(' ').take(inside));
            kept_prev = false;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cells_count_wide_chars_twice_and_marks_never() {
        assert_eq!(cells("abc"), 3);
        assert_eq!(cells("日本語"), 6);
        assert_eq!(cells("e\u{301}"), 1);
        assert_eq!(cells("a😀"), 3);
    }

    #[test]
    fn pad_to_counts_cells_not_chars() {
        assert_eq!(pad_to("日本", 6), "日本  ");
        assert_eq!(cells(&pad_to("a日", 8)), 8);
        assert_eq!(pad_to("日本語", 4), "日本語");
    }

    #[test]
    fn slice_takes_whole_wide_chars() {
        assert_eq!(slice_cells("日本語のテキスト", 0, 6), "日本語");
        assert_eq!(slice_cells("a日本b", 1, 4), "日本");
    }

    #[test]
    fn a_wide_char_at_the_right_edge_becomes_a_space() {
        assert_eq!(slice_cells("日本語", 0, 5), "日本 ");
        assert_eq!(cells(&slice_cells("ab日本", 0, 3)), 3);
    }

    #[test]
    fn an_offset_inside_a_wide_char_leaves_a_space() {
        assert_eq!(slice_cells("日本語", 1, 10), " 本語");
        assert_eq!(slice_cells("日本語", 3, 10), " 語");
    }

    #[test]
    fn emoji_are_two_cells_and_cut_like_cjk() {
        assert_eq!(slice_cells("a😀b", 0, 2), "a ");
        assert_eq!(slice_cells("a😀b", 2, 5), " b");
    }

    #[test]
    fn combining_marks_follow_their_base_and_vanish_with_it() {
        assert_eq!(slice_cells("e\u{301}x", 0, 1), "e\u{301}");
        assert_eq!(slice_cells("e\u{301}x", 1, 5), "x");
    }

    #[test]
    fn char_at_cell_reports_the_straddler() {
        assert_eq!(char_at_cell("日本語", 2), (1, false));
        assert_eq!(char_at_cell("日本語", 3), (1, true));
        assert_eq!(char_at_cell("ab", 9), (2, false));
    }
}
