//! The Settings > AI rows of the writing help on a host: the action blocks,
//! the editable base prompt, and the "Repository access" group whose
//! reading switch is **off by default**.

use super::*;

const HEADING: &str = "Replies on a host";

#[test]
fn the_page_has_the_two_action_blocks_with_profile_and_instructions() {
    let dir = tempfile::tempdir().unwrap();
    let state = state(dir.path());
    let fields = state.fields();
    for title in [HEADING, "New issue descriptions"] {
        let at = fields
            .iter()
            .position(|f| matches!(f.control, Control::Heading) && f.label == title)
            .unwrap_or_else(|| panic!("no block {title:?}"));
        assert_eq!(fields[at + 1].label, "Profile", "{title}");
        assert_eq!(fields[at + 2].label, "Custom instructions", "{title}");
    }
}

#[test]
fn reading_the_repository_is_off_by_default_and_the_switch_writes_config_toml() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    let fields = state.fields();
    let row = fields
        .iter()
        .find(|f| f.label == "Let the AI read the repository by default")
        .expect("the row");
    assert!(
        matches!(row.control, Control::Toggle(false)),
        "off by default: {:?}",
        row.control
    );
    let ttl = fields
        .iter()
        .find(|f| f.label == "Keep a repository context for (minutes)")
        .expect("the row");
    assert!(matches!(ttl.control, Control::Number { value: 30, .. }));
    assert!(fields
        .iter()
        .any(|f| f.label == "Keep the agent's session between replies"));

    select_in(
        &mut state,
        "Repository access",
        "Let the AI read the repository by default",
    );
    state.activate();
    assert!(state.config.ai.hosting_reply.read_repository);
    assert!(
        file(dir.path()).contains("read_repository = true"),
        "{}",
        file(dir.path())
    );
}

#[test]
fn the_base_prompt_is_edited_in_the_overlay_and_saved_globally() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    let fields = state.fields();
    let row = fields
        .iter()
        .find(|f| f.label == "Base prompt")
        .expect("the row");
    assert!(
        row.placeholder
            .as_deref()
            .is_some_and(|p| p.starts_with("You help the user write text")),
        "an empty prompt shows the built-in one: {:?}",
        row.placeholder
    );
    select_in(&mut state, HEADING, "Base prompt");
    state.activate();
    assert!(state.ai.prompt_open());
    type_text(&mut state, "Reply like a pirate.");
    state.on_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(!state.ai.prompt_open());
    assert_eq!(
        state.config.ai.hosting_reply.base_prompt,
        "Reply like a pirate."
    );
    let loaded = Config::load(&dir.path().join("config.toml")).unwrap();
    assert_eq!(loaded.ai.hosting_reply.base_prompt, "Reply like a pirate.");
    // Another action's instructions are untouched.
    assert_eq!(state.config.ai.actions.hosting_reply.instructions, "");
}

#[test]
fn the_reply_caps_are_rows_in_the_limits_group_and_write_config_toml() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    let fields = state.fields();
    let heading = fields.iter().position(|f| f.label == "Limits").unwrap();
    // The five new caps follow the seven existing ones, in AiLimits::KEYS order.
    let rows = [
        ("Largest conversation to send for a reply (bytes)", 49_152),
        ("Largest repository file for a reply (bytes)", 8_192),
        ("Largest repository map for a reply (bytes)", 6_144),
        ("Most search hits per term for a reply", 20),
        ("Most search terms for a reply", 8),
    ];
    for (i, (label, value)) in rows.iter().enumerate() {
        let row = &fields[heading + 1 + 7 + i];
        assert_eq!(row.label, *label);
        assert!(
            matches!(row.control, Control::Number { value: v, .. } if v == *value),
            "{label}: {:?}",
            row.control
        );
    }
    select_in(&mut state, "Limits", rows[3].0);
    state.activate();
    for _ in 0..3 {
        state.on_key(key(KeyCode::Backspace));
    }
    type_text(&mut state, "5");
    state.on_key(key(KeyCode::Enter));
    assert_eq!(state.config.ai.limits.hosting_reply_hits_per_term, 5);
    assert!(
        file(dir.path()).contains("hosting_reply_hits_per_term = 5"),
        "{}",
        file(dir.path())
    );
}

#[test]
fn the_two_actions_have_a_show_what_is_sent_switch() {
    let dir = tempfile::tempdir().unwrap();
    let state = state(dir.path());
    let fields = state.fields();
    for label in ["Reply on a host", "Issue description"] {
        assert!(
            fields
                .iter()
                .any(|f| f.label == label && matches!(f.control, Control::Toggle(true))),
            "{label}"
        );
    }
}
