use super::*;

/// `longest_line` is what `DiffViewState::scroll_h`'s clamp (applied by
/// the render loop) bounds against.
#[test]
fn longest_line_is_the_longest_raw_line_in_the_file() {
    assert_eq!(longest_line(&wide_sample(), None), 40); // "...xyz-old"/"-new"
    assert_eq!(longest_line(&file(vec![]), None), 0);
    // The whole-file layout also draws the unchanged lines.
    let full = vec!["x".repeat(55)];
    assert_eq!(longest_line(&wide_sample(), Some(&full)), 55);
}

/// `scroll_h` is a no-op while word wrap is on - a wrapped line
/// already fits the width, so there is nothing to its right to reveal.
#[test]
fn scroll_h_does_nothing_while_word_wrap_is_on() {
    let mut state = DiffViewState::for_commit(test_oid(), 0, PathBuf::from("f.rs"));
    state.word_wrap = true;
    state.scroll_h(10);
    assert_eq!(state.h_offset, 0);
}

/// `scroll_h`'s own lower bound: it never goes negative, independent
/// of whatever the render loop's upper clamp against `longest_line`
/// does.
#[test]
fn scroll_h_never_goes_negative() {
    let mut state = DiffViewState::for_commit(test_oid(), 0, PathBuf::from("f.rs"));
    state.scroll_h(-100);
    assert_eq!(state.h_offset, 0);
}

/// The bug this mirrors from the File Insight overlay: with word wrap
/// off and the content column truncated to the pane width, there was
/// no way to see what a long line held past its right edge.
/// `render_diff_scrolled` with a nonzero `h_offset` must show
/// *different* text than `h_offset == 0`, and the gutter (line
/// numbers, +/- marker) must be identical either way.
#[test]
fn horizontal_scroll_reveals_different_text_without_moving_the_gutter() {
    let f = wide_sample();
    let width = 30; // narrow enough that the sample line doesn't fit
    let unscrolled = render_diff_scrolled(&f, DiffViewMode::Inline, true, false, width, 0);
    let scrolled = render_diff_scrolled(&f, DiffViewMode::Inline, true, false, width, 10);

    let u = plain(&unscrolled.lines);
    let s = plain(&scrolled.lines);
    assert_eq!(u.len(), s.len());

    // Row 0 is the hunk header (unaffected); row 1 is the first body
    // line -- content must differ once scrolled.
    assert_ne!(
        u[1], s[1],
        "scrolling must change what's shown: {u:?} vs {s:?}"
    );

    // The gutter -- everything up to and including the +/- marker,
    // `GUTTER_WIDTH + 1` characters -- must be byte-identical.
    let gutter_len = GUTTER_WIDTH + 1;
    let u_gutter: String = u[1].chars().take(gutter_len).collect();
    let s_gutter: String = s[1].chars().take(gutter_len).collect();
    assert_eq!(u_gutter, s_gutter, "the gutter must stay fixed");
}

/// Scrolling past the end of every line in the file must still render
/// well-formed rows (the scrolled-indicator alone), never panic --
/// `render_diff_scrolled` doesn't clamp `h_offset` itself; that's the
/// render loop's job (against `longest_line`), so this function has to
/// cope with an out-of-range value handed to it directly.
#[test]
fn scrolling_past_every_line_s_end_does_not_panic() {
    let f = wide_sample();
    let rendered = render_diff_scrolled(&f, DiffViewMode::Inline, true, false, 30, 10_000);
    assert!(!rendered.lines.is_empty());
}

/// Split mode gets the same horizontal scroll as inline/hunk mode.
#[test]
fn horizontal_scroll_also_works_in_split_mode() {
    let f = wide_sample();
    let unscrolled = render_diff_scrolled(&f, DiffViewMode::Split, true, false, 60, 0);
    let scrolled = render_diff_scrolled(&f, DiffViewMode::Split, true, false, 60, 10);
    assert_ne!(plain(&unscrolled.lines), plain(&scrolled.lines));
}

// -- horizontal scroll extent ------------------------------------------

/// The bug: the bar's content length was the longest line against the
/// pane's *full* width, so a line overflowing the (narrower) text column
/// showed no bar, and the offset the thumb could reach stopped short of
/// the end of the line.
#[test]
fn the_h_scroll_range_is_measured_against_the_text_column_not_the_pane() {
    // Inline at width 40: gutter 13, text column 27.
    assert_eq!(text_viewport(DiffViewMode::Inline, 40), 27);
    assert_eq!(h_scroll_max(27, DiffViewMode::Inline, 40), 0, "just fits");
    assert!(
        h_scroll_max(28, DiffViewMode::Inline, 40) > 0,
        "one column too long, well inside the pane's 40: the bar must show"
    );
    // Split shows one side at a time: much narrower again.
    assert!(text_viewport(DiffViewMode::Split, 40) < 27);
    assert!(h_scroll_max(27, DiffViewMode::Split, 40) > 0);
    // No overflow, no bar.
    assert_eq!(h_track_content(20, DiffViewMode::Inline, 40, 40), 0);
}

/// At the largest offset the bar's thumb reaches, the last character of
/// the longest line is on screen, whole (no trailing ellipsis).
#[test]
fn the_end_of_the_thumb_travel_shows_the_last_character_of_the_longest_line() {
    for (mode, width) in [(DiffViewMode::Inline, 40u16), (DiffViewMode::Split, 60u16)] {
        let f = wide_sample(); // longest line: 40 chars, ends in "-new"/"-old"
        let longest = longest_line(&f, None);
        let max = h_scroll_max(longest, mode, width);
        assert!(max > 0, "{mode:?}: 40 chars overflow the text column");
        let r = render_diff_scrolled(&f, mode, true, false, width, max);
        let rows = plain(&r.lines);
        assert!(
            rows.iter()
                .any(|l| l.contains("-new") || l.contains("-old")),
            "{mode:?}: {rows:?}"
        );
        assert!(!rows.iter().any(|l| l.contains('\u{2026}')), "{rows:?}");
    }
}

/// The track's content length is chosen so the thumb's full travel is
/// exactly `0..=h_scroll_max`.
#[test]
fn the_track_content_length_gives_the_thumb_exactly_the_scroll_range() {
    let (longest, width) = (200usize, 100u16);
    let track_w = width; // no minimap
    let max = h_scroll_max(longest, DiffViewMode::Inline, width);
    let content = h_track_content(longest, DiffViewMode::Inline, width, track_w);
    assert_eq!(content - track_w as usize, max);
    let body = Rect::new(0, 0, track_w + 2, 10);
    let track = crate::scrollbar::Track::horizontal(body, content, 0).unwrap();
    assert_eq!(
        track.drag_offset(content, track_w as usize, 0, 10_000),
        max,
        "dragging to the far end reaches the very end of the longest line"
    );
}
