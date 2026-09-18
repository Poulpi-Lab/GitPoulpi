use super::*;

/// A state on the Git & Encoding page with `Push button` selected.
fn push_button_state(dir: &Path) -> PreferencesState {
    let mut state = state(dir);
    state.section = PrefSection::ALL
        .iter()
        .position(|s| *s == PrefSection::GitAndEncoding)
        .unwrap();
    state.focus = PrefFocus::Fields;
    state.field = state
        .fields()
        .iter()
        .position(|f| f.label == "Push button")
        .unwrap();
    state
}

fn key_of(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

#[test]
fn enter_on_a_choice_opens_a_dropdown_and_does_not_change_the_value() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = push_button_state(dir.path());
    let before = state.config.git.push_behavior;
    state.on_key(key_of(KeyCode::Enter));
    assert!(state.choice.is_some(), "Enter opens the popup");
    assert_eq!(state.config.git.push_behavior, before, "nothing cycled");
    // Another Enter picks the highlighted (current) option: still same.
    state.on_key(key_of(KeyCode::Enter));
    assert!(state.choice.is_none());
    assert_eq!(state.config.git.push_behavior, before);
}

#[test]
fn a_keyboard_pick_from_the_dropdown_commits_and_esc_abandons() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = push_button_state(dir.path());
    let before = state.config.git.push_behavior;
    state.on_key(key_of(KeyCode::Char(' ')));
    state.on_key(key_of(KeyCode::Down));
    state.on_key(key_of(KeyCode::Esc));
    assert!(state.choice.is_none(), "Esc closes only the popup");
    assert_eq!(state.config.git.push_behavior, before);
    // `q` must not close the screen while the popup is open.
    state.on_key(key_of(KeyCode::Enter));
    assert!(state.on_key(key_of(KeyCode::Char('q'))), "modal popup");
    state.on_key(key_of(KeyCode::Esc));

    state.on_key(key_of(KeyCode::Enter));
    state.on_key(key_of(KeyCode::Down));
    state.on_key(key_of(KeyCode::Enter));
    assert_ne!(state.config.git.push_behavior, before);
    let raw = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert!(raw.contains("push_behavior"), "{raw}");
}

#[test]
fn clicking_a_choice_opens_a_popup_and_clicking_an_option_picks_it() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = push_button_state(dir.path());
    let before = state.config.git.push_behavior;
    let screen = draw_screen(&mut state);
    assert!(screen.contains('\u{25BE}'), "the value carries the arrow");
    let (y, _) = *state
        .field_rows
        .iter()
        .find(|(_, i)| *i == state.field)
        .unwrap();
    let x = state.body_area.right() - 4;
    state.on_mouse(MouseEventKind::Down(MouseButton::Left), x, y);
    assert!(state.choice.is_some(), "a click opens, it does not cycle");
    assert_eq!(state.config.git.push_behavior, before);
    draw_screen(&mut state);
    let items = state.choice.as_ref().unwrap().1.item_rects();
    assert_eq!(items.len(), PushBehavior::ALL.len(), "every option listed");
    let other = items
        .iter()
        .find(|(_, row)| PushBehavior::ALL[*row] != before)
        .unwrap()
        .0;
    state.on_mouse(
        MouseEventKind::Down(MouseButton::Left),
        other.x + 1,
        other.y,
    );
    assert!(state.choice.is_none());
    assert_ne!(state.config.git.push_behavior, before);
}

#[test]
fn clicking_away_from_an_open_choice_only_closes_it() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = push_button_state(dir.path());
    let before = state.config.git.push_behavior;
    draw_screen(&mut state);
    state.activate();
    draw_screen(&mut state);
    assert!(state.on_mouse(MouseEventKind::Down(MouseButton::Left), 2, 28));
    assert!(state.choice.is_none());
    assert_eq!(state.config.git.push_behavior, before);
}
