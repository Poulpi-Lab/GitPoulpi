use super::*;

fn type_into_search(state: &mut PreferencesState, text: &str) {
    for c in text.chars() {
        state.on_key(KeyEvent::from(KeyCode::Char(c)));
    }
}

#[test]
fn search_jumps_to_a_setting_by_label_and_by_description() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    state.on_key(KeyEvent::from(KeyCode::Char('/')));
    assert!(state.search.is_some(), "/ opens the search");
    type_into_search(&mut state, "tab size");
    state.on_key(KeyEvent::from(KeyCode::Enter));
    assert!(state.search.is_none());
    assert_eq!(state.current_section(), PrefSection::DiffEditor);
    assert_eq!(state.fields()[state.field].label, "Tab size");
    assert_eq!(state.focus, PrefFocus::Fields);

    // Ctrl+F, and a word that only the help text has.
    state.on_key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL));
    assert!(state.search.is_some());
    type_into_search(&mut state, "whole file hunks");
    state.on_key(KeyEvent::from(KeyCode::Enter));
    assert_eq!(state.current_section(), PrefSection::DiffEditor);
    assert_eq!(state.fields()[state.field].label, "Hunks only");
}

#[test]
fn search_keeps_typed_slashes_out_of_text_fields_and_esc_closes_only_it() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    state.on_key(KeyEvent::from(KeyCode::Char('/')));
    type_into_search(&mut state, "no such setting anywhere");
    assert!(state.search.as_ref().unwrap().hits.is_empty());
    assert!(
        state.on_key(KeyEvent::from(KeyCode::Esc)),
        "Esc stays in Settings"
    );
    assert!(state.search.is_none());
    // Typing a text value: `/` is a character, not the search.
    state.section = PrefSection::ALL
        .iter()
        .position(|s| *s == PrefSection::Appearance)
        .unwrap();
    state.focus = PrefFocus::Fields;
    state.field = state
        .fields()
        .iter()
        .position(|f| f.label == "Date format")
        .unwrap();
    state.activate();
    assert!(state.edit.is_some());
    state.on_key(KeyEvent::from(KeyCode::Char('/')));
    assert!(state.search.is_none());
}
