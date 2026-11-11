use super::*;

/// The splitter is a pure render-time width: `Alt+Left`/`Alt+Right`
/// resize it and it is clamped to [`SPLIT_MIN_LEFT`] and to leaving
/// the content pane at least [`SPLIT_MIN_CONTENT`] wide.
#[test]
fn alt_left_and_right_resize_the_splitter_within_its_minimums() {
    let tmp = tree();
    let mut picker = picker_at(tmp.path());
    draw(&mut picker, 110, 30); // establishes panes_w
    let start = picker.left_area.width;

    picker.on_key(KeyEvent::new(KeyCode::Right, KeyModifiers::ALT));
    draw(&mut picker, 110, 30);
    assert_eq!(picker.left_area.width, start + SPLIT_STEP as u16);

    // Narrowing all the way down stops at the minimum, never
    // crushing the content pane to nothing.
    for _ in 0..40 {
        picker.on_key(KeyEvent::new(KeyCode::Left, KeyModifiers::ALT));
    }
    draw(&mut picker, 110, 30);
    assert_eq!(picker.left_area.width, SPLIT_MIN_LEFT);
    assert!(picker.content_area.width >= SPLIT_MIN_CONTENT);
}

/// A drag on the divider moves it live and a release ends the drag;
/// hover alone must not.
#[test]
fn dragging_the_splitter_resizes_it_and_release_ends_the_drag() {
    let tmp = tree();
    let mut picker = picker_at(tmp.path());
    draw(&mut picker, 110, 30);
    let (splitter_rect, _) = *picker
        .hits()
        .iter()
        .find(|(_, h)| *h == PickerHit::Splitter)
        .unwrap();
    picker.on_mouse(
        MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: splitter_rect.x,
            row: splitter_rect.y,
            modifiers: KeyModifiers::NONE,
        },
        false,
    );
    assert!(picker.dragging_splitter);
    picker.on_mouse(
        MouseEvent {
            kind: MouseEventKind::Drag(MouseButton::Left),
            column: splitter_rect.x + 5,
            row: splitter_rect.y,
            modifiers: KeyModifiers::NONE,
        },
        false,
    );
    draw(&mut picker, 110, 30);
    assert_eq!(
        picker.left_area.width,
        splitter_rect.x + 5 - picker.left_area.x
    );
    picker.on_mouse(
        MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: splitter_rect.x + 5,
            row: splitter_rect.y,
            modifiers: KeyModifiers::NONE,
        },
        false,
    );
    assert!(!picker.dragging_splitter);
}

/// The splitter is remembered as a share of the pane row: the next picker
/// opens with it, a narrower terminal scales it, and nothing is announced
/// for saving until the user actually moved it.
#[test]
fn the_splitter_share_is_remembered_by_the_next_picker() {
    install_split(0);
    let tmp = tree();
    let mut picker = picker_at(tmp.path());
    draw(&mut picker, 160, 30);
    assert_eq!(take_split_change(), None, "nothing moved, nothing to save");

    for _ in 0..4 {
        picker.on_key(KeyEvent::new(KeyCode::Right, KeyModifiers::ALT));
    }
    draw(&mut picker, 160, 30);
    let moved = picker.left_area.width;
    let share = take_split_change().expect("a keyboard nudge is announced");
    assert_eq!(take_split_change(), None, "announced once");

    // The next dialog (here: after the window re-installed the saved value)
    // opens with the same width, and scales on a narrower terminal.
    install_split(share);
    let mut next = picker_at(tmp.path());
    draw(&mut next, 160, 30);
    assert!(
        next.left_area.width.abs_diff(moved) <= 1,
        "{}",
        next.left_area.width
    );
    let mut narrow = picker_at(tmp.path());
    draw(&mut narrow, 80, 30);
    assert!(narrow.left_area.width < moved);
    assert!(narrow.content_area.width >= SPLIT_MIN_CONTENT);
    install_split(0);
}

fn splitter_rect(picker: &FolderPicker) -> Rect {
    picker
        .hits()
        .iter()
        .find(|(_, h)| *h == PickerHit::Splitter)
        .unwrap()
        .0
}

fn mouse_at(picker: &mut FolderPicker, kind: MouseEventKind, column: u16, row: u16) {
    picker.on_mouse(
        MouseEvent {
            kind,
            column,
            row,
            modifiers: KeyModifiers::NONE,
        },
        false,
    );
}

fn paint(picker: &mut FolderPicker) -> Buffer {
    let mut buf = Buffer::empty(Rect::new(0, 0, 110, 30));
    crate::pointer_map::begin_frame();
    picker.render(picker_rect(buf.area), &mut buf);
    buf
}

/// The splitter is the sideways resize pointer, lit with the accent like the
/// Log's and the Radar's resize lines, and unlit again on release.
#[test]
fn the_splitter_is_ew_resize_and_lights_with_the_shared_accent_until_release() {
    let tmp = tree();
    let mut picker = picker_at(tmp.path());
    let buf = paint(&mut picker);
    let r = splitter_rect(&picker);
    let rest = buf[(r.x, r.y + 1)].style().fg;
    assert_ne!(rest, Some(crate::th().ui.accent));
    assert_eq!(
        crate::pointer_map::shape_at(r.x, r.y + 1),
        crate::cursor::CursorShape::ColResize
    );

    mouse_at(&mut picker, MouseEventKind::Moved, r.x, r.y + 1);
    let buf = paint(&mut picker);
    assert_eq!(buf[(r.x, r.y + 1)].style().fg, Some(crate::th().ui.accent));
    assert_eq!(buf[(r.x, r.y + 1)].symbol(), "\u{2502}");

    mouse_at(
        &mut picker,
        MouseEventKind::Down(MouseButton::Left),
        r.x,
        r.y + 1,
    );
    mouse_at(
        &mut picker,
        MouseEventKind::Drag(MouseButton::Left),
        r.x + 4,
        r.y + 1,
    );
    paint(&mut picker);
    let r2 = splitter_rect(&picker);
    assert_eq!(r2.x, r.x + 4);
    // Released somewhere else: no `Moved` follows, the line must not stay lit.
    mouse_at(
        &mut picker,
        MouseEventKind::Up(MouseButton::Left),
        r2.x + 12,
        r2.y + 2,
    );
    let buf = paint(&mut picker);
    assert_ne!(
        buf[(r2.x, r2.y + 1)].style().fg,
        Some(crate::th().ui.accent)
    );
}

/// The selected folder row is shown by its highlight alone: no arrow glyph.
#[test]
fn the_selected_folder_row_has_no_arrow_marker() {
    let tmp = tree();
    let mut picker = picker_at(tmp.path());
    let text = draw(&mut picker, 110, 30);
    assert!(!text.contains('\u{25B8}'), "{text}");
    picker.on_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
    picker.on_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    let text = draw(&mut picker, 110, 30);
    assert!(!text.contains('\u{25B8}'), "{text}");
}

#[test]
fn the_picker_moves_resizes_and_arranges() {
    let tmp = tree();
    let mut picker = picker_at(tmp.path());
    crate::dialog_kit::frame::testing::check_gestures(true, || {
        let mut buf = Buffer::empty(Rect::new(0, 0, 160, 50));
        crate::pointer_map::begin_frame();
        picker.render(picker_rect(buf.area), &mut buf);
    });
}
