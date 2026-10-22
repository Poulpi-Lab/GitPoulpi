use super::*;

#[test]
fn the_repo_page_has_a_help_line_and_a_block_per_action() {
    let dir = tempfile::tempdir().unwrap();
    let state = repo_state_on_page(dir.path());
    let fields = state.fields();
    assert!(
        fields[0].label.starts_with("Overrides for "),
        "{}",
        fields[0].label
    );
    assert!(fields[0]
        .label
        .ends_with("Empty fields use the global AI settings."));
    for title in [
        "Commit message generation",
        "Explain changes",
        "Stash message generation",
        "Pull Request",
        "Commit composer",
    ] {
        assert!(
            fields
                .iter()
                .any(|f| matches!(f.control, Control::Heading) && f.label == title),
            "{title}"
        );
    }
    // No override yet: no Reset button anywhere.
    assert!(!fields.iter().any(|f| f.label == "Reset to global"));
}

#[test]
fn the_repo_profile_combo_leads_with_use_global_and_saves_to_repo_state() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = repo_state_on_page(dir.path());
    select_in(&mut state, "Commit message generation", "Profile");
    match &state.fields()[state.field].control {
        Control::Choice { options, index } => {
            assert_eq!(options[0], "Use global (Default)");
            assert_eq!(options.len(), 3);
            assert_eq!(*index, 0);
        }
        other => panic!("{other:?}"),
    }
    state.choose_next(); // Default
    state.choose_next(); // Second
    assert_eq!(
        repo_state(dir.path()).preferences.ai_overrides["commit_message"].profile,
        "second"
    );
    // Nothing in config.toml.
    assert!(!file(dir.path()).contains("commit_message"));
    // The Reset button appears and clears the override.
    select_in(&mut state, "Commit message generation", "Reset to global");
    state.activate();
    assert!(repo_state(dir.path()).preferences.ai_overrides.is_empty());
    assert!(!state.fields().iter().any(|f| f.label == "Reset to global"));
}

#[test]
fn a_repo_chip_and_the_editor_write_the_override_not_the_global() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = repo_state_on_page(dir.path());
    let action = AiAction::CommitMessage;
    select_in(
        &mut state,
        "Commit message generation",
        "Custom instructions",
    );
    state.activate();
    state.on_key(key(KeyCode::Tab));
    state.on_key(key(KeyCode::Right));
    state.on_key(key(KeyCode::Enter));
    state.on_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));
    let sentence = AiSection::Action(action).chips()[1].1.english();
    assert_eq!(
        repo_state(dir.path()).preferences.ai_overrides["commit_message"].instructions,
        sentence
    );
    assert_eq!(state.config.ai.actions.commit_message.instructions, "");
    // The editor saves through the same path, more text after the chip.
    select_in(
        &mut state,
        "Commit message generation",
        "Custom instructions",
    );
    state.activate();
    type_text(&mut state, " In French.");
    state.on_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));
    let saved = repo_state(dir.path()).preferences.ai_overrides["commit_message"]
        .instructions
        .clone();
    assert!(saved.ends_with("In French."), "{saved}");
}

#[test]
fn the_repo_instructions_and_their_combo_save_to_repo_state() {
    use gitpoulpi_config::RepoInstructionsMode;
    let dir = tempfile::tempdir().unwrap();
    let mut state = repo_state_on_page(dir.path());
    select_in(&mut state, "Repository instructions", "Custom instructions");
    state.activate();
    type_text(&mut state, "Repo rule.");
    state.on_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));
    let saved = repo_state(dir.path()).preferences.ai_instructions;
    assert_eq!(saved.text, "Repo rule.");
    assert_eq!(saved.mode, RepoInstructionsMode::Add);
    assert_eq!(state.config.ai.global_instructions, "");
    state.ai_commit_choice(AiTarget::RepoMode, 1);
    let saved = repo_state(dir.path()).preferences.ai_instructions;
    assert_eq!(saved.mode, RepoInstructionsMode::Replace);
    assert_eq!(saved.text, "Repo rule.");
    let fields = state.fields();
    let combo = fields
        .iter()
        .find(|f| f.label == "Combine with global")
        .unwrap();
    assert!(matches!(combo.control, Control::Choice { index: 1, .. }));
}

#[test]
fn an_empty_repo_textarea_shows_the_global_instructions_grayed() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = repo_state_on_page(dir.path());
    state.config.ai.actions.commit_message.instructions = "Global rule.".into();
    select_in(
        &mut state,
        "Commit message generation",
        "Custom instructions",
    );
    let fields = state.fields();
    assert_eq!(
        fields[state.field].placeholder.as_deref(),
        Some("Global rule.")
    );
    assert!(draw(&mut state, 160, 60).contains("Global rule."));
}

#[test]
fn a_repo_override_changes_what_the_resolver_returns() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = repo_state_on_page(dir.path());
    state.config.ai.global_instructions = "Global.".into();
    let ai = state.config.ai.clone();
    let before = ai.resolve(AiAction::CommitMessage, None).unwrap();
    assert_eq!(before.profile.id, "default");
    state.edit_repo_override(AiAction::CommitMessage, |o| {
        o.profile = "second".into();
        o.instructions = "Repo rule.".into();
    });
    let reloaded = repo_state(dir.path());
    let o = reloaded.preferences.ai_override("commit_message", &ai);
    let after = ai.resolve(AiAction::CommitMessage, o.as_ref()).unwrap();
    assert_eq!(after.profile.id, "second");
    assert_eq!(after.action_instructions, "Repo rule.");
    assert_eq!(after.global_instructions, "Global.");
    // Other actions still inherit.
    assert!(reloaded
        .preferences
        .ai_override("explain_changes", &ai)
        .is_none());
}
