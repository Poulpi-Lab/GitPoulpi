//! The Limits row of the AI page: the conflict size limit, its warning while
//! very large, and where "Open setting" of the "too large" dialog lands.

use super::*;

fn global_state(dir: &Path) -> PreferencesState {
    let mut state = state(dir);
    state.config.ai.enabled = true;
    state
}

const LABEL: &str = "Largest conflict to send (bytes)";

#[test]
fn the_limit_row_shows_todays_value_and_writes_config_toml() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = global_state(dir.path());
    let fields = state.fields();
    let row = fields.iter().find(|f| f.label == LABEL).expect("the row");
    assert!(matches!(row.control, Control::Number { value: 40_000, .. }));
    assert!(row.warning.is_none());

    select_in(&mut state, "Limits", LABEL);
    state.activate();
    for _ in 0..6 {
        state.on_key(key(KeyCode::Backspace));
    }
    type_text(&mut state, "90000");
    state.on_key(key(KeyCode::Enter));
    assert!(
        file(dir.path()).contains("conflict_bytes = 90000"),
        "{}",
        file(dir.path())
    );
    assert_eq!(state.config.ai.limits.conflict_bytes, 90_000);
}

#[test]
fn a_very_large_limit_is_flagged_in_red() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = global_state(dir.path());
    state.config.ai.limits.conflict_bytes = 5_000_000;
    let fields = state.fields();
    let row = fields.iter().find(|f| f.label == LABEL).expect("the row");
    assert!(
        row.warning
            .as_deref()
            .is_some_and(|w| w.contains("fails at the provider")),
        "{:?}",
        row.warning
    );
}

#[test]
fn open_setting_lands_on_the_limit_row() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = global_state(dir.path());
    state.focus = PrefFocus::Sections;
    state.field = 0;
    state
        .show_page(None, Some(crate::guide_dialog::CONFLICT_LIMIT_KEY))
        .expect("the key has a row");
    assert_eq!(
        PrefSection::ALL[state.section],
        PrefSection::Ai,
        "the AI page"
    );
    assert_eq!(state.fields()[state.field].label, LABEL);
}
