use super::*;

#[test]
fn a_quick_place_and_a_tree_node_navigate_on_one_click() {
    let tmp = tree();
    let mut picker = picker_at(tmp.path());
    picker.set_places(vec![Place {
        label: "Home".into(),
        path: tmp.path().join("beta"),
        kind: PlaceKind::Standard,
    }]);
    draw(&mut picker, 110, 30);
    let place = picker
        .left_rows()
        .iter()
        .position(|r| matches!(r, LeftRow::Place(0)))
        .unwrap();
    click(&mut picker, PickerHit::Left(place), false);
    assert_eq!(picker.dir, tmp.path().join("beta"));
    settle(&mut picker);

    // The tree holds the ancestors: clicking the parent's node goes
    // back up.
    draw(&mut picker, 110, 30);
    let root_node = picker
        .left_rows()
        .iter()
        .position(|r| matches!(r, LeftRow::Tree { path, .. } if path == tmp.path()))
        .unwrap();
    click(&mut picker, PickerHit::Left(root_node), false);
    assert_eq!(picker.dir, tmp.path());
}

#[test]
fn a_click_selects_a_double_click_opens_and_the_buttons_act() {
    let tmp = tree();
    let mut picker = picker_at(tmp.path());
    draw(&mut picker, 110, 30);
    let beta = content_index(&picker, "beta");
    click(&mut picker, PickerHit::Content(beta), false);
    assert_eq!(picker.content_sel, Some(beta));
    assert_eq!(picker.dir, tmp.path());
    click(&mut picker, PickerHit::Content(beta), true);
    assert_eq!(picker.dir, tmp.path().join("beta"));
    settle(&mut picker);
    draw(&mut picker, 110, 30);
    click(&mut picker, PickerHit::UpButton, false);
    assert_eq!(picker.dir, tmp.path());
    settle(&mut picker);
    draw(&mut picker, 110, 30);
    assert_eq!(
        click(&mut picker, PickerHit::Choose, false),
        PickerOutcome::Choose(tmp.path().join("beta")),
        "the folder just left is selected"
    );
    assert_eq!(
        click(&mut picker, PickerHit::Cancel, false),
        PickerOutcome::Cancel
    );
}

#[test]
fn the_wheel_scrolls_and_hover_follows_the_pointer() {
    let tmp = tempfile::tempdir().unwrap();
    for i in 0..60 {
        std::fs::create_dir(tmp.path().join(format!("d{i:02}"))).unwrap();
    }
    let mut picker = picker_at(tmp.path());
    let before = draw(&mut picker, 100, 24);
    assert!(before.contains("d00"));
    let area = picker.content_area;
    let wheel = |kind| MouseEvent {
        kind,
        column: area.x + 2,
        row: area.y + 2,
        modifiers: KeyModifiers::NONE,
    };
    for _ in 0..4 {
        picker.on_mouse(wheel(MouseEventKind::ScrollDown), false);
    }
    draw(&mut picker, 100, 24);
    assert!(picker.content_scroll >= 10, "{}", picker.content_scroll);
    picker.on_mouse(wheel(MouseEventKind::Moved), false);
    assert!(matches!(picker.hover, Some(PickerHit::Content(_))));
}
