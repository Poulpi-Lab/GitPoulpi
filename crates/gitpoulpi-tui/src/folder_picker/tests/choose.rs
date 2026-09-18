use super::*;

/// `PickerMode::File` is the opposite: a file row is selectable and
/// choosing it (Enter, or a double click) picks the file, while a
/// folder-mode picker on the same tree never lets a file become the
/// choice at all.
#[test]
fn a_file_picker_chooses_a_file_and_a_folder_picker_never_does() {
    let tmp = tree();
    let mut file_picker = FolderPicker::for_file("Apply patch", "Apply", tmp.path().to_path_buf());
    settle(&mut file_picker);
    let _ = draw(&mut file_picker, 110, 30);
    let file_index = file_picker
        .content_rows()
        .iter()
        .position(|r| matches!(r, ContentRow::File(_)))
        .expect("notes.txt should be a content row");
    file_picker.content_sel = Some(file_index);
    let outcome = file_picker.on_key(key(KeyCode::Enter));
    assert_eq!(outcome, PickerOutcome::Choose(tmp.path().join("notes.txt")));

    let mut folder_picker = picker_at(tmp.path());
    let folder_file_index = folder_picker
        .content_rows()
        .iter()
        .position(|r| matches!(r, ContentRow::File(_)));
    assert!(
        folder_file_index.is_none()
            || folder_picker.content_key(key(KeyCode::End), false) == PickerOutcome::Stay
                && !matches!(
                    folder_picker
                        .content_rows()
                        .get(folder_picker.content_sel.unwrap_or(usize::MAX)),
                    Some(ContentRow::File(_))
                ),
        "a folder picker's selection must never land on a file row"
    );
}

#[test]
fn choosing_picks_the_selected_subfolder_or_else_the_folder_shown() {
    let tmp = tree();
    let mut picker = picker_at(tmp.path());
    let ctrl_s = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
    assert_eq!(
        picker.on_key(ctrl_s),
        PickerOutcome::Choose(tmp.path().to_path_buf())
    );
    picker.content_sel = Some(content_index(&picker, "alpha"));
    assert!(picker.chosen_is_repo());
    assert_eq!(
        picker.on_key(ctrl_s),
        PickerOutcome::Choose(tmp.path().join("alpha"))
    );
    // Enter with nothing selected chooses the folder shown.
    picker.content_sel = None;
    assert_eq!(
        picker.on_key(key(KeyCode::Enter)),
        PickerOutcome::Choose(tmp.path().to_path_buf())
    );
    assert_eq!(picker.on_key(key(KeyCode::Esc)), PickerOutcome::Cancel);
}
