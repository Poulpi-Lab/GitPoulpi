//! Wide chars (CJK, emoji) must not move the Split diff's separator: a
//! column is built by cells, not chars.

use super::*;

fn cjk_file() -> FileDiff {
    file(vec![hunk(vec![
        line(LineOrigin::Context, "日本語のテキスト"),
        line(LineOrigin::Delete, "abc 日本語のテキスト def 😀 end"),
        line(LineOrigin::Delete, "plain ascii line"),
        line(LineOrigin::Add, "日本語のテキスト mixed ascii と日本語"),
        line(LineOrigin::Add, "e\u{301}e\u{301}e\u{301} combining"),
        line(LineOrigin::Context, "x"),
    ])])
}

/// The separator's cell column on every row that has one.
fn separator_columns(r: &DiffRender) -> Vec<usize> {
    plain(&r.lines)
        .iter()
        .filter_map(|l| {
            l.split_once('\u{2502}')
                .map(|(left, _)| display_width(left))
        })
        .collect()
}

fn assert_stable(wrap: bool, width: u16, h: usize) {
    let r = render_diff_scrolled(&cjk_file(), DiffViewMode::Split, true, wrap, width, h);
    let cols = separator_columns(&r);
    assert!(cols.len() >= 4, "rows with a separator: {cols:?}");
    assert!(
        cols.iter().all(|&c| c == cols[0]),
        "wrap={wrap} width={width} h={h}: separator moved {cols:?}: {:#?}",
        plain(&r.lines)
    );
    for line in plain(&r.lines).iter().filter(|l| l.contains('\u{2502}')) {
        assert!(display_width(line) <= width as usize, "{line:?} overflows");
    }
}

#[test]
fn the_split_separator_stays_put_with_wide_chars() {
    for width in [41, 40, 61, 80] {
        assert_stable(false, width, 0);
    }
}

#[test]
fn the_split_separator_stays_put_with_wide_chars_and_word_wrap() {
    for width in [41, 40, 61, 80] {
        assert_stable(true, width, 0);
    }
}

#[test]
fn the_split_separator_stays_put_when_scrolled_sideways() {
    for h in 1..12 {
        assert_stable(false, 41, h);
        assert_stable(false, 40, h);
    }
}

#[test]
fn a_scroll_inside_a_wide_char_shows_a_space_for_its_second_half() {
    let r = render_diff_scrolled(&cjk_file(), DiffViewMode::Split, true, false, 80, 1);
    let rows = plain(&r.lines);
    let first = rows.iter().find(|l| l.contains('\u{2502}')).unwrap();
    // "«" then the second half of the first wide char as a blank.
    assert!(first.contains("\u{ab} 本語"), "{first:?}");
}

#[test]
fn horizontal_scroll_limit_counts_cells() {
    let f = file(vec![hunk(vec![line(
        LineOrigin::Add,
        "日本語日本語日本語",
    )])]);
    assert_eq!(longest_line(&f, None), 18);
}

#[test]
fn unified_rows_stay_within_the_width_with_wide_chars() {
    for h in 0..6 {
        let r = render_diff_scrolled(&cjk_file(), DiffViewMode::Inline, true, false, 30, h);
        for l in plain(&r.lines) {
            assert!(display_width(&l) <= 30, "{l:?}");
        }
    }
}

#[test]
fn a_scrolled_wide_row_keeps_its_segment_on_the_char_under_the_marker() {
    let r = render_diff_scrolled(&cjk_file(), DiffViewMode::Inline, true, false, 40, 2);
    let seg = r.segs[1][0].expect("body row");
    // Two cells scrolled = the first wide char gone whole, so the row
    // starts at the char after it (the marker cell is not mapped).
    assert_eq!(seg.start, 1);
    assert_eq!(seg.x as usize, GUTTER_WIDTH + 1 + 1);
}
