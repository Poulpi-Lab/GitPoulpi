use super::*;

/// ↑/↓ land on hunk headers measured from the cursor, and do not get
/// stuck when the scroll is clamped short of the last header.
/// Falsified: measuring from `scroll` (the old code) with scroll
/// clamped at 5 finds header 8 again and again.
#[test]
fn hunk_jumps_go_from_the_cursor_and_reach_the_last_hunk() {
    let starts = [2usize, 8, 9];
    let mut state = DiffViewState::for_working(DiffSource::Unstaged, PathBuf::from("a"));
    assert!(jump_to_hunk(&mut state, &starts, true, 20).is_ok());
    assert_eq!(state.cursor, 2);
    assert!(jump_to_hunk(&mut state, &starts, true, 20).is_ok());
    assert_eq!(state.cursor, 8);
    // The render clamps the scroll short of the last header.
    state.scroll = 5;
    assert!(jump_to_hunk(&mut state, &starts, true, 20).is_ok());
    assert_eq!(state.cursor, 9, "reached the last hunk");
    assert_eq!(
        jump_to_hunk(&mut state, &starts, true, 20),
        Err("no further hunk in this file")
    );
    state.scroll = 5;
    assert!(jump_to_hunk(&mut state, &starts, false, 20).is_ok());
    assert_eq!(state.cursor, 8);
    // Cursor scrolled out of view by the wheel: from the view's top.
    state.cursor = 0;
    state.scroll = 7;
    assert!(jump_to_hunk(&mut state, &starts, true, 1).is_ok());
    assert_eq!(state.cursor, 8);
}

// ---- screen row <-> diff row --------------------------------------

/// `body` is the BORDERED rect, so content starts at `body.y + 1` and
/// the last row is a border too. Getting this wrong by one stages the
/// hunk next to the one that was clicked - the same class of bug
/// `row_to_index`'s `top_offset` caused in the Log.
#[test]
fn a_screen_row_maps_through_the_border_and_the_scroll_offset() {
    let body = Rect::new(4, 10, 40, 12); // rows 10..21, borders at 10 and 21
    assert_eq!(diff_row_at(body, 0, 100, 10), None, "top border");
    assert_eq!(diff_row_at(body, 0, 100, 11), Some(0), "first content row");
    assert_eq!(diff_row_at(body, 0, 100, 20), Some(9), "last content row");
    assert_eq!(diff_row_at(body, 0, 100, 21), None, "bottom border");
    assert_eq!(diff_row_at(body, 0, 100, 22), None, "below the block");
    // The scroll offset shifts the whole mapping.
    assert_eq!(diff_row_at(body, 7, 100, 11), Some(7));
    assert_eq!(diff_row_at(body, 7, 100, 20), Some(16));
    // Past the end of the diff there is no row, however much blank
    // space the block still has.
    assert_eq!(diff_row_at(body, 0, 3, 14), None);
    assert_eq!(diff_row_at(body, 0, 3, 13), Some(2));
}

/// The cursor is a row index, and the view has to follow it rather
/// than the other way round.
#[test]
fn moving_the_cursor_scrolls_the_minimum_needed_to_keep_it_visible() {
    let mut state = DiffViewState::for_working(DiffSource::Unstaged, PathBuf::from("a.txt"));
    // A 5-row viewport over 20 rows.
    for _ in 0..4 {
        move_cursor(&mut state, 1, 20, 5);
    }
    assert_eq!((state.cursor, state.scroll), (4, 0), "still on the page");
    move_cursor(&mut state, 1, 20, 5);
    assert_eq!(
        (state.cursor, state.scroll),
        (5, 1),
        "one row down, not a whole page"
    );
    move_cursor(&mut state, 100, 20, 5);
    assert_eq!(state.cursor, 19, "clamped to the last row");
    assert_eq!(state.scroll, 15, "the last page, not past it");
    move_cursor(&mut state, -100, 20, 5);
    assert_eq!((state.cursor, state.scroll), (0, 0));
    // A diff with no rows at all must not panic or leave a phantom
    // cursor behind.
    move_cursor(&mut state, 1, 0, 5);
    assert_eq!(state.cursor, 0);
}

/// A relayout renumbers every row, so the cursor and the selection --
/// which are row indices - cannot survive it.
#[test]
fn a_relayout_drops_the_cursor_and_the_selection() {
    let mut state = DiffViewState::for_working(DiffSource::Unstaged, PathBuf::from("a.txt"));
    state.cursor = 12;
    state.scroll = 8;
    state.selection_anchor = Some(9);
    state.reset_position();
    assert_eq!((state.cursor, state.scroll), (0, 0));
    assert_eq!(state.selection_anchor, None);
}

/// With no selection, a line-level action covers the cursor row alone.
#[test]
fn the_selected_range_is_the_cursor_row_until_an_anchor_is_dropped() {
    let mut state = DiffViewState::for_working(DiffSource::Staged, PathBuf::from("a.txt"));
    state.cursor = 4;
    assert_eq!(state.selected_rows(), (4, 4));
    state.selection_anchor = Some(9);
    assert_eq!(state.selected_rows(), (4, 9), "normalized low..high");
    state.cursor = 12;
    assert_eq!(state.selected_rows(), (9, 12));
}
