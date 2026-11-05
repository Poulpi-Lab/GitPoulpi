use super::*;

/// A folder with `n` sub-folders `d00`, `d01`, ...
fn many(n: usize) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    for i in 0..n {
        std::fs::create_dir_all(tmp.path().join(format!("d{i:02}"))).unwrap();
    }
    tmp
}

fn buffer(picker: &mut FolderPicker, width: u16, height: u16) -> Buffer {
    let mut buf = Buffer::empty(Rect::new(0, 0, width, height));
    picker.render(picker_rect(buf.area), &mut buf);
    buf
}

fn glyph(buf: &Buffer, x: u16, y: u16) -> String {
    buf[(x, y)].symbol().to_string()
}

/// Whether the column at `x` draws the bar's two end arrows over `height` rows from `y0`.
fn has_arrows(buf: &Buffer, x: u16, y0: u16, height: u16) -> bool {
    glyph(buf, x, y0) == "\u{2191}" && glyph(buf, x, y0 + height - 1) == "\u{2193}"
}

fn mouse(kind: MouseEventKind, col: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind,
        column: col,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

#[test]
fn the_content_list_draws_a_bar_only_while_it_overflows() {
    let long = many(60);
    let mut picker = picker_at(long.path());
    let buf = buffer(&mut picker, 110, 24);
    let area = picker.content_area;
    let (x, y) = (area.right() - 1, area.y + 1);
    assert!(has_arrows(&buf, x, y, area.height - 1), "bar missing");

    let short = many(3);
    let mut picker = picker_at(short.path());
    let buf = buffer(&mut picker, 110, 24);
    let area = picker.content_area;
    assert!(!has_arrows(
        &buf,
        area.right() - 1,
        area.y + 1,
        area.height - 1
    ));
}

#[test]
fn the_content_bar_drags_and_the_wheel_and_keys_still_work() {
    let tmp = many(60);
    let mut picker = picker_at(tmp.path());
    buffer(&mut picker, 110, 24);
    let area = picker.content_area;
    let track = picker.content_bar.track.expect("overflowing");
    let x = track.rect.x;
    // Press the thumb, drag it past the bottom: the list is at its end.
    let down = track.rect.y + track.thumb_start;
    picker.on_mouse(
        mouse(MouseEventKind::Down(MouseButton::Left), x, down),
        false,
    );
    let low = track.rect.bottom() + 3;
    picker.on_mouse(
        mouse(MouseEventKind::Drag(MouseButton::Left), x, low),
        false,
    );
    picker.on_mouse(mouse(MouseEventKind::Up(MouseButton::Left), x, low), false);
    buffer(&mut picker, 110, 24);
    let height = area.height as usize - 1;
    assert_eq!(picker.content_scroll, picker.content_rows().len() - height);
    assert_eq!(picker.content_sel, None, "dragging selects nothing");
    // The wheel over the list scrolls it back up.
    picker.on_mouse(
        mouse(MouseEventKind::ScrollUp, area.x + 2, area.y + 3),
        false,
    );
    assert_eq!(
        picker.content_scroll,
        picker.content_rows().len() - height - 3
    );
    // A key moves the selection and the list follows it.
    picker.focus(Zone::Content);
    picker.on_key(key(KeyCode::End));
    buffer(&mut picker, 110, 24);
    let last = picker.content_rows().len() - 1;
    assert_eq!(picker.content_sel, Some(last));
    assert!(picker.content_scroll + height > last);
}

#[test]
fn the_left_list_draws_a_bar_and_drags() {
    let tmp = many(2);
    let mut picker = picker_at(tmp.path());
    // Many quick places overflow the left column.
    picker.set_places(
        (0..40)
            .map(|i| Place {
                label: format!("Place {i}"),
                path: tmp.path().join("d00"),
                kind: PlaceKind::Standard,
            })
            .collect(),
    );
    let buf = buffer(&mut picker, 110, 24);
    let area = picker.left_area;
    assert!(has_arrows(&buf, area.right() - 1, area.y, area.height));
    let track = picker.left_bar.track.unwrap();
    let x = track.rect.x;
    picker.on_mouse(
        mouse(
            MouseEventKind::Down(MouseButton::Left),
            x,
            track.rect.bottom() - 2,
        ),
        false,
    );
    assert!(picker.left_scroll > 0, "a press on the track jumps there");
    picker.on_mouse(
        mouse(MouseEventKind::Up(MouseButton::Left), x, track.rect.y),
        false,
    );

    let tmp = many(2);
    let mut picker = picker_at(tmp.path());
    let buf = buffer(&mut picker, 110, 24);
    let area = picker.left_area;
    assert!(!has_arrows(&buf, area.right() - 1, area.y, area.height));
}

#[test]
fn the_completion_dropdown_draws_a_bar_and_keeps_a_dragged_window() {
    let tmp = many(30);
    let mut picker = picker_at(tmp.path());
    picker.focus(Zone::Path);
    picker.path_draft = format!("{}{}d", tmp.path().display(), std::path::MAIN_SEPARATOR);
    picker.typing = true;
    settle(&mut picker);
    let buf = buffer(&mut picker, 110, 30);
    let track = picker.drop_bar.track.expect("30 completions, 8 rows");
    assert!(has_arrows(
        &buf,
        track.rect.x,
        track.rect.y,
        track.rect.height
    ));
    let x = track.rect.x;
    let y = track.rect.bottom() - 2;
    picker.on_mouse(mouse(MouseEventKind::Down(MouseButton::Left), x, y), false);
    picker.on_mouse(mouse(MouseEventKind::Up(MouseButton::Left), x, y), false);
    assert!(picker.drop_top > 0);
    let dragged = picker.drop_top;
    buffer(&mut picker, 110, 30);
    assert_eq!(picker.drop_top, dragged, "the next frame keeps the window");
    // The wheel over the dropdown scrolls it, not the list below.
    let before = picker.content_scroll;
    picker.on_mouse(mouse(MouseEventKind::ScrollUp, x - 2, track.rect.y), false);
    buffer(&mut picker, 110, 30);
    assert!(picker.drop_top < dragged);
    assert_eq!(picker.content_scroll, before);
}
