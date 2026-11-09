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

/// One row per cap, in the order of `AiLimits::KEYS`, each with today's value.
#[test]
fn every_cap_has_a_row_with_todays_default() {
    let dir = tempfile::tempdir().unwrap();
    let state = global_state(dir.path());
    let fields = state.fields();
    let labels = [
        ("Largest conflict to send (bytes)", 40_000),
        ("Largest diff to send (bytes)", 60_000),
        ("Largest file diff in a review (bytes)", 12_000),
        ("Largest review description (bytes)", 4_000),
        (
            "Largest list of existing comments in a review (bytes)",
            6_000,
        ),
        ("Largest context file in a review (bytes)", 8_192),
        ("Largest context in a review, all files (bytes)", 16_384),
    ];
    let heading = fields
        .iter()
        .position(|f| f.label == "Limits")
        .expect("the heading");
    for (i, (label, value)) in labels.iter().enumerate() {
        let row = &fields[heading + 1 + i];
        assert_eq!(row.label, *label);
        assert!(
            matches!(row.control, Control::Number { value: v, .. } if v == *value),
            "{label}: {:?}",
            row.control
        );
        assert!(row.warning.is_none(), "{label}");
    }
}

/// "Open setting" of the truncation dialog names the key of the first limit
/// listed: every key lands on its own row.
#[test]
fn open_setting_lands_on_every_limit_row() {
    for key in gitpoulpi_config::AiLimits::KEYS {
        let dir = tempfile::tempdir().unwrap();
        let mut state = global_state(dir.path());
        state.focus = PrefFocus::Sections;
        state.field = 0;
        state
            .show_page(None, Some(key))
            .unwrap_or_else(|_| panic!("{key} has a row"));
        let row = &state.fields()[state.field];
        let index = gitpoulpi_config::AiLimits::KEYS
            .iter()
            .position(|k| *k == key)
            .unwrap();
        // The row after the heading, `index` further down.
        let fields = state.fields();
        let heading = fields.iter().position(|f| f.label == "Limits").unwrap();
        assert_eq!(
            row.label,
            fields[heading + 1 + index].label,
            "{key} lands on its own row"
        );
    }
}

/// Editing a cap writes `config.toml`; a very large one is flagged red.
#[test]
fn a_cap_is_written_to_config_toml_and_a_huge_one_is_flagged() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = global_state(dir.path());
    select_in(&mut state, "Limits", "Largest diff to send (bytes)");
    state.activate();
    for _ in 0..6 {
        state.on_key(key(KeyCode::Backspace));
    }
    type_text(&mut state, "90000");
    state.on_key(key(KeyCode::Enter));
    assert!(
        file(dir.path()).contains("diff_bytes = 90000"),
        "{}",
        file(dir.path())
    );
    assert_eq!(state.config.ai.limits.diff_bytes, 90_000);
    state.config.ai.limits.review_context_bytes = 5_000_000;
    let fields = state.fields();
    let row = fields
        .iter()
        .find(|f| f.label == "Largest context in a review, all files (bytes)")
        .unwrap();
    assert!(row.warning.is_some());
}
