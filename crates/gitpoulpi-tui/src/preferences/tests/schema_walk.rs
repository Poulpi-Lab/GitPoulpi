use super::*;

/// Every boolean setting renders as a checkbox, never as a text field, a
/// number or a dropdown: walks all pages and looks each global key up in
/// the serialized default config.
#[test]
fn every_boolean_setting_is_a_checkbox() {
    let dir = tempfile::tempdir().unwrap();
    let state = state(dir.path());
    let defaults = toml::Value::try_from(Config::default()).unwrap();
    let mut bools = 0;
    for section in PrefSection::ALL {
        for field in state.fields_of(*section) {
            let Target::Global(key) = field.target else {
                continue;
            };
            let value = key.split('.').try_fold(&defaults, |v, part| v.get(part));
            if matches!(value, Some(toml::Value::Boolean(_))) {
                bools += 1;
                assert!(
                    matches!(field.control, Control::Toggle(_)),
                    "{key} is a boolean but its row is {:?}",
                    field.control
                );
            }
        }
    }
    assert!(bools > 10, "the walk found the booleans ({bools})");
}

/// The Editor page keeps related settings together and the merge editor's
/// last.
#[test]
fn the_editor_page_ends_with_the_merge_editor_settings() {
    let dir = tempfile::tempdir().unwrap();
    let state = state(dir.path());
    let keys: Vec<&str> = state
        .editor_rows()
        .iter()
        .filter_map(|f| match f.target {
            Target::Global(k) => Some(k),
            _ => None,
        })
        .collect();
    let first_merge = keys.iter().position(|k| k.starts_with("merge.")).unwrap();
    assert!(
        keys[first_merge..].iter().all(|k| k.starts_with("merge.")),
        "{keys:?}"
    );
    let wrap = keys.iter().position(|k| *k == "ui.diff_word_wrap").unwrap();
    let numbers = keys
        .iter()
        .position(|k| *k == "editor.line_numbers")
        .unwrap();
    assert!(
        wrap.abs_diff(numbers) <= 2,
        "word wrap sits with the editor rows: {keys:?}"
    );
}

/// Leaving a text field commits it; a refused value keeps the text, red;
/// Esc cancels.
#[test]
fn leaving_a_field_commits_refusals_stay_and_esc_cancels() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    go(&mut state, PrefSection::General);
    state.focus = PrefFocus::Fields;
    let key = |code| KeyEvent::new(code, KeyModifiers::NONE);
    let at = |s: &PreferencesState, label: &str| {
        s.fields().iter().position(|f| f.label == label).unwrap()
    };
    // Text: type, Down commits and moves on.
    state.field = at(&state, "Default branch name");
    state.on_key(key(KeyCode::Enter));
    for c in "trunk".chars() {
        state.on_key(key(KeyCode::Char(c)));
    }
    state.on_key(key(KeyCode::Down));
    assert!(state.edit.is_none());
    assert!(state.config.general.default_branch_name.ends_with("trunk"));
    // Esc cancels.
    state.field = at(&state, "Default branch name");
    state.on_key(key(KeyCode::Enter));
    state.on_key(key(KeyCode::Char('Z')));
    state.on_key(key(KeyCode::Esc));
    assert!(state.edit.is_none());
    assert!(state.config.general.default_branch_name.ends_with("trunk"));
    // A refused number stays in the field, with its text.
    go(&mut state, PrefSection::LogGraph);
    state.field = at(&state, "Initial commits in graph");
    state.on_key(key(KeyCode::Enter));
    for c in "abc".chars() {
        state.on_key(key(KeyCode::Char(c)));
    }
    state.on_key(key(KeyCode::Tab));
    assert!(state
        .edit
        .as_ref()
        .is_some_and(|(_, d, _)| d.ends_with("abc")));
    assert!(state.edit_invalid.is_some());
}
