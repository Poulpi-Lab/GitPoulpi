use super::*;

/// `Alt+N` opens the inline field; an invalid (Windows-reserved)
/// name is refused and stays open; a valid one is handed to the
/// owner as [`PickerOutcome::CreateFolder`], and once the owner
/// reports it created ([`FolderPicker::folder_created`]) the fresh
/// listing selects it.
#[test]
fn alt_n_opens_new_folder_validates_and_selects_the_created_folder() {
    let tmp = tree();
    let mut picker = picker_at(tmp.path());
    assert_eq!(
        picker.on_key(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::ALT)),
        PickerOutcome::Stay
    );
    assert!(picker.new_folder_draft.is_some());

    for c in "CON".chars() {
        picker.on_key(key(KeyCode::Char(c)));
    }
    assert_eq!(picker.on_key(key(KeyCode::Enter)), PickerOutcome::Stay);
    assert!(
        picker.message.as_deref().unwrap().contains("reserved"),
        "{:?}",
        picker.message
    );
    assert!(
        picker.new_folder_draft.is_some(),
        "stays open on an invalid name"
    );

    // Esc cancels; Ctrl+N reopens with a clean draft.
    picker.on_key(key(KeyCode::Esc));
    assert!(picker.new_folder_draft.is_none());
    picker.on_key(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::ALT));
    for c in "sub".chars() {
        picker.on_key(key(KeyCode::Char(c)));
    }
    let outcome = picker.on_key(key(KeyCode::Enter));
    assert_eq!(outcome, PickerOutcome::CreateFolder(tmp.path().join("sub")));
    assert!(picker.new_folder_draft.is_none());

    // The maintainer's IO, then its report back.
    std::fs::create_dir(tmp.path().join("sub")).unwrap();
    picker.folder_created("sub");
    settle(&mut picker);
    let index = content_index(&picker, "sub");
    assert_eq!(picker.content_sel, Some(index));
}

/// A folder the OS refuses (already exists, here) reports through
/// [`FolderPicker::folder_create_failed`] as a message, not a panic.
#[test]
fn a_failed_creation_is_reported_not_swallowed() {
    let tmp = tree();
    let mut picker = picker_at(tmp.path());
    picker.folder_create_failed("already exists");
    assert!(picker
        .message
        .as_deref()
        .unwrap()
        .contains("already exists"));
}
