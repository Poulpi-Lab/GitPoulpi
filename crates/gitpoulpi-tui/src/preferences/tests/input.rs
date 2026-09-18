use super::*;

/// Esc closes; every other key leaves it open. The screen is the
/// only thing on screen while it is up, so a key that fell through
/// would act on the pane underneath.
#[test]
fn escape_closes_and_other_keys_do_not() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    assert!(state.on_key(KeyEvent::from(KeyCode::Down)));
    assert!(state.on_key(KeyEvent::from(KeyCode::Tab)));
    assert!(!state.on_key(KeyEvent::from(KeyCode::Esc)));
}

#[test]
fn tab_moves_between_the_section_list_and_the_settings() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    assert_eq!(state.focus, PrefFocus::Sections);
    state.on_key(KeyEvent::from(KeyCode::Tab));
    assert_eq!(state.focus, PrefFocus::Fields);
    state.on_key(KeyEvent::from(KeyCode::Tab));
    assert_eq!(state.focus, PrefFocus::Sections);
}

#[test]
fn the_section_list_walks_every_page_and_stops_at_both_ends() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    state.on_key(KeyEvent::from(KeyCode::Up));
    assert_eq!(state.current_section(), PrefSection::General);
    for _ in 0..PrefSection::ALL.len() + 3 {
        state.on_key(KeyEvent::from(KeyCode::Down));
    }
    let last = PrefSection::ALL
        .iter()
        .rev()
        .find(|s| s.is_built())
        .unwrap();
    assert_eq!(state.current_section(), *last);
}

/// Typing into a field must not be interpreted as navigation: `j`
/// belongs in a branch name.
#[test]
fn keys_typed_into_an_open_editor_go_into_the_draft() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    state.focus = PrefFocus::Fields;
    state.field = state
        .general_fields()
        .iter()
        .position(|f| f.label == "Default branch name")
        .unwrap();
    state.activate();
    assert!(state.edit.is_some());
    for c in "jkq".chars() {
        state.on_key(KeyEvent::from(KeyCode::Char(c)));
    }
    assert_eq!(state.edit.as_ref().unwrap().1, "mainjkq");
    state.on_key(KeyEvent::from(KeyCode::Enter));
    assert_eq!(state.config.general.default_branch_name, "mainjkq");
}

/// Walks every `Control::Number` row (General's four, UI's one), not one
/// picked by name. The vim-count arm in `event_loop` once swallowed digits
/// before they reached `PreferencesState::on_key`; that routing is covered by
/// `chrome::count_prefix_may_not_apply_while_preferences_is_open`, and this
/// proves the screen handles a digit once it arrives.
#[test]
fn every_numeric_field_accepts_typed_digits_and_commits_them() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());

    let mut numeric_fields: Vec<(usize, usize, String, i64)> = Vec::new();
    for section_index in 0..PrefSection::ALL.len() {
        state.section = section_index;
        for (field_index, field) in state.fields().iter().enumerate() {
            if let Control::Number { min, .. } = &field.control {
                numeric_fields.push((section_index, field_index, field.label.clone(), *min));
            }
        }
    }
    assert!(
        numeric_fields.len() >= 5,
        "expected every known numeric field (General's four, UI's Mouse wheel \
         lines) to be found, got {numeric_fields:?}"
    );

    for (section_index, field_index, label, min) in numeric_fields {
        state.section = section_index;
        state.focus = PrefFocus::Fields;
        state.field = field_index;
        state.activate();
        assert!(
            state.edit.is_some(),
            "{label}: activating a numeric field must open its editor"
        );
        // Clear the pre-filled current value first -- Backspace, the
        // one key that worked before this fix, still must.
        let current_len = state.edit.as_ref().unwrap().1.chars().count();
        for _ in 0..current_len {
            assert!(state.on_key(KeyEvent::from(KeyCode::Backspace)));
        }
        assert_eq!(
            state.edit.as_ref().unwrap().1,
            "",
            "{label}: Backspace didn't clear it"
        );
        // Then every digit of `min`, one `on_key` call each -- the
        // exact shape a real keystroke arrives in.
        for c in min.to_string().chars() {
            assert!(
                state.on_key(KeyEvent::from(KeyCode::Char(c))),
                "{label}: on_key returned false for digit {c:?}"
            );
        }
        assert_eq!(
            state.edit.as_ref().unwrap().1,
            min.to_string(),
            "{label}: typed digits did not reach the draft"
        );
        state.on_key(KeyEvent::from(KeyCode::Enter));
        assert!(state.edit.is_none(), "{label}: Enter must close the editor");
        let committed = state
            .fields()
            .into_iter()
            .find(|f| f.label == label)
            .and_then(|f| match f.control {
                Control::Number { value, .. } => Some(value),
                _ => None,
            });
        assert_eq!(
            committed,
            Some(min),
            "{label}: the typed value was not committed (message: {:?})",
            state.message
        );
    }
}

#[test]
fn escape_in_an_editor_abandons_the_draft_without_closing_the_screen() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    state.focus = PrefFocus::Fields;
    state.field = state
        .general_fields()
        .iter()
        .position(|f| f.label == "Default branch name")
        .unwrap();
    state.activate();
    state.on_key(KeyEvent::from(KeyCode::Char('x')));
    assert!(state.on_key(KeyEvent::from(KeyCode::Esc)));
    assert!(state.edit.is_none());
    assert_eq!(state.config.general.default_branch_name, "main");
}

#[test]
fn a_click_outside_the_screen_closes_it_and_one_inside_does_not() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    state.area = Rect::new(10, 5, 40, 20);
    assert!(!state.on_mouse(MouseEventKind::Down(MouseButton::Left), 0, 0));
    assert!(state.on_mouse(MouseEventKind::Down(MouseButton::Left), 20, 10));
}
