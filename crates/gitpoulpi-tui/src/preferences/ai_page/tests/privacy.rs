//! The Privacy rows of the AI page: the two protections with their red
//! warnings, one "show what is sent" switch per action, and "Show all again".

use super::*;
use gitpoulpi_config::AiAction;

fn global_state(dir: &Path) -> PreferencesState {
    let mut state = state(dir);
    state.config.ai.enabled = true;
    state
}

fn row<'a>(fields: &'a [Field], label: &str) -> &'a Field {
    fields
        .iter()
        .find(|f| f.label == label)
        .unwrap_or_else(|| panic!("no row {label:?}"))
}

#[test]
fn the_protections_write_config_toml_and_warn_in_red_while_off() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = global_state(dir.path());
    let fields = state.fields();
    assert!(matches!(
        row(&fields, "Redact secrets before sending").control,
        Control::Toggle(true)
    ));
    assert!(row(&fields, "Redact secrets before sending")
        .warning
        .is_none());

    select_in(&mut state, "Privacy", "Redact secrets before sending");
    state.activate();
    assert!(
        file(dir.path()).contains("redact_secrets = false"),
        "{}",
        file(dir.path())
    );
    assert!(!state.config.ai.privacy.redact_secrets);
    let fields = state.fields();
    assert!(
        row(&fields, "Redact secrets before sending")
            .warning
            .as_deref()
            .is_some_and(|w| w.contains("sent to the AI service as they are")),
        "a red row while redaction is off"
    );
    assert!(
        row(&fields, "Leave out sensitive files").warning.is_none(),
        "the other protection is untouched"
    );

    select_in(&mut state, "Privacy", "Leave out sensitive files");
    state.activate();
    assert!(file(dir.path()).contains("exclude_sensitive_files = false"));
    assert!(row(&state.fields(), "Leave out sensitive files")
        .warning
        .is_some());
    // And back on: the warning goes.
    state.activate();
    assert!(state.config.ai.privacy.exclude_sensitive_files);
    assert!(row(&state.fields(), "Leave out sensitive files")
        .warning
        .is_none());
}

#[test]
fn one_switch_per_action_and_show_all_again() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = global_state(dir.path());
    // The history-rewrite actions show their own dialog: no switch here.
    let labels: Vec<String> = state.fields().iter().map(|f| f.label.clone()).collect();
    assert!(
        !state
            .fields()
            .iter()
            .any(|f| f.label.starts_with("Show all again")),
        "the button only shows while something is off: {labels:?}"
    );

    select_in(&mut state, "Show what is sent before...", "Commit message");
    state.activate();
    assert!(
        file(dir.path()).contains("commit_message = false"),
        "{}",
        file(dir.path())
    );
    assert!(!state.config.ai.sent_preview.shows(AiAction::CommitMessage));
    assert!(state.config.ai.sent_preview.shows(AiAction::ExplainChanges));

    select_in(&mut state, "Show what is sent before...", "Explain changes");
    state.activate();
    assert!(!state.config.ai.sent_preview.all_shown());

    // The button turns every one back on.
    let fields = state.fields();
    let at = fields
        .iter()
        .position(|f| f.label == "Show all again")
        .expect("the button is there while one is off");
    state.field = at;
    state.activate();
    assert!(state.config.ai.sent_preview.all_shown());
    assert!(file(dir.path()).contains("commit_message = true"));
    assert!(!state.fields().iter().any(|f| f.label == "Show all again"));
}
