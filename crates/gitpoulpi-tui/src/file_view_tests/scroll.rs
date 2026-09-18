use super::*;

/// The scroll window has to follow the selection in *both*
/// directions. The downward case is the obvious one; the upward case
/// is where a `selected - viewport` on a `usize` wraps to a huge
/// number and throws the pane to its end.
#[test]
fn the_viewport_follows_the_selection_both_ways() {
    let mut insight = blame_insight(100);
    insight.select(60, 10);
    assert!(insight.offset <= 60 && 60 < insight.offset + 10);

    insight.move_selection(-100, 10);
    assert_eq!(insight.selected, 0);
    assert_eq!(insight.offset, 0);

    insight.move_selection(1000, 10);
    assert_eq!(insight.selected, 99);
    assert!(insight.offset <= 99 && 99 < insight.offset + 10);
}

/// The bug: a mouse-wheel `scroll()` moved `offset` but never touched
/// `selected` (still 0), and the render loop calls `clamp()` every
/// frame *before* drawing. `clamp()`'s job is to pull `offset` back to
/// `selected` when the selection has scrolled out of view - so with
/// `selected` stuck at 0, the very next `clamp()` undid the scroll
/// completely. The wheel looked inert. This reproduces exactly that
/// two-step sequence (scroll, then the clamp the next frame always
/// runs) and would have failed on the old `scroll(&mut self, delta)`.
#[test]
fn a_wheel_scroll_survives_the_clamp_the_next_frame_runs() {
    let mut insight = blame_insight(100);
    let viewport = 10;
    insight.scroll(9, viewport);
    assert_ne!(
        insight.offset, 0,
        "the wheel scroll itself must move the view"
    );
    insight.clamp(viewport);
    assert_eq!(
        insight.offset, 9,
        "the next frame's clamp() must not snap the offset back to the untouched selection"
    );
}

/// The other half of "cannot scroll to the bottom": the old bound let
/// `offset` run to `len - 1`, one screen past where the last line is
/// actually visible at the bottom of the viewport. The reachable range
/// is `[0, len - viewport]`.
#[test]
fn wheel_scrolling_far_enough_lands_exactly_on_the_last_line_no_further() {
    let mut insight = blame_insight(37);
    let viewport = 10;
    insight.scroll(1000, viewport);
    assert_eq!(insight.offset, 37 - viewport, "clamped to len - viewport");
    let last_visible = windowed(&insight, insight.len(), viewport).end - 1;
    assert_eq!(
        last_visible, 36,
        "the last line (index 36) must be on screen"
    );
}

/// Falsifiable the other direction too: scrolling up from the bottom
/// must be able to leave the last line again - offset isn't stuck at
/// its max either.
#[test]
fn wheel_scrolling_back_up_from_the_bottom_moves_the_view() {
    let mut insight = blame_insight(37);
    let viewport = 10;
    insight.scroll(1000, viewport);
    let at_bottom = insight.offset;
    insight.scroll(-5, viewport);
    assert_eq!(insight.offset, at_bottom - 5);
}

/// Keyboard `End`/`G` (via `move_selection`) must reach the true last
/// row too, the same guarantee the wheel now has.
#[test]
fn keyboard_end_reaches_the_last_line() {
    let mut insight = blame_insight(37);
    let viewport = 10;
    insight.move_selection(isize::MAX / 2, viewport);
    assert_eq!(insight.selected, 36);
    let last_visible = windowed(&insight, insight.len(), viewport).end - 1;
    assert_eq!(last_visible, 36);
}

/// Horizontal scroll clamps to the longest line: there is nothing past
/// it to scroll to, in either direction.
#[test]
fn horizontal_scroll_clamps_to_the_longest_line() {
    let mut insight = blame_insight(3); // each line is "line {i}" - 6-7 chars
    let longest = insight.longest_line();
    assert!(longest > 0);
    insight.scroll_h(1000);
    assert_eq!(insight.h_offset, longest - 1);
    insight.scroll_h(-1000);
    assert_eq!(insight.h_offset, 0);
}

/// The blame gutter (sha/author/date) must stay fixed while the text
/// column scrolls - only the content after the gutter should change
/// as `h_offset` grows.
#[test]
fn the_blame_gutter_stays_fixed_while_the_text_scrolls() {
    let mut insight = blame_insight(1);
    // Give the single line enough content to actually scroll through.
    if let FileInsightData::Blame(blame) = &mut insight.data {
        if let BlameContent::Lines(lines) = &mut blame.content {
            lines[0].content = "0123456789abcdefghij".to_string();
        }
    }
    let unscrolled: Vec<String> = insight_rows(&insight, 80, 10)[0]
        .spans
        .iter()
        .map(|s| s.content.to_string())
        .collect();
    insight.h_offset = 5;
    let scrolled: Vec<String> = insight_rows(&insight, 80, 10)[0]
        .spans
        .iter()
        .map(|s| s.content.to_string())
        .collect();

    // sha/space/author/space/date/space spans (indices 0..=5, before
    // the content span) are untouched by h_offset.
    for i in 0..6 {
        assert_eq!(
            unscrolled[i], scrolled[i],
            "gutter span {i} must not move when the text scrolls"
        );
    }
    // The content span (last one) must differ, and carry the
    // scrolled-indicator.
    assert!(scrolled[6].starts_with('\u{ab}'), "{:?}", scrolled[6]);
}
