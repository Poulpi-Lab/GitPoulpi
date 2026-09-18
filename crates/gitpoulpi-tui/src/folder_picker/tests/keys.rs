use super::*;

#[test]
fn enter_goes_into_a_folder_and_backspace_and_alt_up_come_back() {
    let tmp = tree();
    let mut picker = picker_at(tmp.path());
    picker.content_sel = Some(content_index(&picker, "beta"));
    assert_eq!(picker.on_key(key(KeyCode::Enter)), PickerOutcome::Stay);
    assert_eq!(picker.dir, tmp.path().join("beta"));
    settle(&mut picker);
    assert!(draw(&mut picker, 100, 30).contains("inner"));

    picker.on_key(key(KeyCode::Backspace));
    assert_eq!(picker.dir, tmp.path());
    // Back on the folder just left.
    settle(&mut picker);
    assert_eq!(picker.chosen(), tmp.path().join("beta"));

    picker.navigate(tmp.path().join("beta"));
    settle(&mut picker);
    picker.on_key(KeyEvent::new(KeyCode::Up, KeyModifiers::ALT));
    assert_eq!(picker.dir, tmp.path());
}

#[test]
fn the_path_field_completes_navigates_and_then_chooses() {
    let tmp = tree();
    let mut picker = picker_at(tmp.path());
    picker.on_key(key(KeyCode::Tab));
    assert_eq!(picker.zone, Zone::Path);
    // The first key replaces the folder's own path.
    for c in "be".chars() {
        picker.on_key(key(KeyCode::Char(c)));
    }
    assert_eq!(picker.path_draft, "be");
    settle(&mut picker);
    let names: Vec<String> = picker.completions().into_iter().map(|e| e.name).collect();
    assert_eq!(names, ["beta"]);
    assert!(draw(&mut picker, 100, 30).contains("beta"));
    // Tab takes the only candidate.
    picker.on_key(key(KeyCode::Tab));
    assert!(picker.path_draft.ends_with(std::path::MAIN_SEPARATOR));
    // Enter goes there; Enter again chooses it.
    picker.on_key(key(KeyCode::Enter));
    assert_eq!(picker.dir, tmp.path().join("beta"));
    settle(&mut picker);
    assert_eq!(
        picker.on_key(key(KeyCode::Enter)),
        PickerOutcome::Choose(tmp.path().join("beta"))
    );
}

#[test]
fn a_typed_folder_that_does_not_exist_says_so() {
    let tmp = tree();
    let mut picker = picker_at(tmp.path());
    picker.focus(Zone::Path);
    picker.paste(&tmp.path().join("nowhere").display().to_string());
    picker.on_key(key(KeyCode::Enter));
    settle(&mut picker);
    assert!(draw(&mut picker, 100, 30).contains("no such folder"));
}
