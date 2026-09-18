use super::*;

// The wheel scrolls the WIP row's "All files" list's
// viewport, never its selection; a keyboard/click move re-arms
// following.

fn wip_all_with_rows(n: usize) -> WipAllFiles {
    WipAllFiles {
        rows: (0..n)
            .map(|i| TreeRow::File {
                depth: 0,
                label: format!("f{i}.rs"),
                index: i,
            })
            .collect(),
        area: Rect::new(0, 0, 30, 7), // 5 content rows
        ..WipAllFiles::default()
    }
}

#[test]
fn scroll_by_moves_the_offset_and_leaves_the_selection_alone() {
    let mut wip = wip_all_with_rows(20);
    wip.state.select(Some(2));

    wip.scroll_by(3);

    assert_eq!(
        wip.state.selected(),
        Some(2),
        "the wheel must never move the selection"
    );
    assert_eq!(wip.state.offset(), 3, "but it must move the offset");
    assert!(
        !wip.follow,
        "a wheel-scrolled list stops following its selection until \
         something else (keyboard, click) touches it again"
    );
}

#[test]
fn scroll_by_cannot_scroll_past_the_list_s_own_content() {
    let mut wip = wip_all_with_rows(3); // fewer rows than the visible area
    wip.scroll_by(50);
    assert_eq!(wip.state.offset(), 0);
}

#[test]
fn move_by_re_arms_following_after_a_wheel_scroll() {
    let mut wip = wip_all_with_rows(20);
    wip.state.select(Some(0));
    wip.scroll_by(3);
    assert!(!wip.follow);

    wip.move_by(1);

    assert_eq!(wip.state.selected(), Some(1));
    assert!(
        wip.follow,
        "a keyboard/click move must re-arm following so the view \
         snaps back onto the selection"
    );
}
