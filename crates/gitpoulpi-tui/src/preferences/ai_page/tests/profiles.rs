use super::*;
use crate::preferences::list_editor::ListOp;

#[test]
fn args_round_trip_with_quotes() {
    assert_eq!(split_args("-p --model \"a b\""), ["-p", "--model", "a b"]);
    assert_eq!(join_args(&split_args("-p \"a b\"")), "-p \"a b\"");
    assert!(split_args("   ").is_empty());
}

#[test]
fn the_page_starts_off_with_nothing_configured() {
    let dir = tempfile::tempdir().unwrap();
    let state = state(dir.path());
    let fields = state.fields();
    assert!(matches!(fields[0].control, Control::Toggle(false)));
    assert!(state.ai_list_spec().duplicate);
    assert!(fields.iter().any(|f| f.label == "Custom instructions"));
}

#[test]
fn enabling_ai_and_adding_a_profile_reach_config_toml() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    state.field = 0;
    state.activate();
    assert!(
        file(dir.path()).contains("enabled = true"),
        "{}",
        file(dir.path())
    );

    state.apply_list_op(ListOp::Add(None));
    let raw = file(dir.path());
    assert!(raw.contains("[[ai.profiles]]"), "{raw}");
    assert!(raw.contains("token_ref = \"ai:profile\""), "{raw}");

    // Rename it, switch it to a local agent.
    select(&mut state, "Name");
    state.activate();
    for _ in 0..20 {
        state.on_key(key(KeyCode::Backspace));
    }
    type_text(&mut state, "Claude Code");
    state.on_key(key(KeyCode::Enter));
    select(&mut state, "Provider");
    for _ in 0..4 {
        state.choose_next();
    }
    let raw = file(dir.path());
    assert!(raw.contains("name = \"Claude Code\""), "{raw}");
    assert!(raw.contains("kind = \"local_agent\""), "{raw}");
    assert!(raw.contains("command = \"claude\""), "{raw}");
}

#[test]
fn a_typed_key_goes_to_the_token_store_and_never_to_config_toml() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    state.apply_list_op(ListOp::Add(None));
    select(&mut state, "API key");
    state.activate();
    type_text(&mut state, "sk-very-secret");
    state.on_key(key(KeyCode::Enter));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while state.tokens.load("ai:profile").unwrap().is_none() {
        assert!(
            std::time::Instant::now() < deadline,
            "the key was never stored"
        );
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert_eq!(
        state.tokens.load("ai:profile").unwrap().as_deref(),
        Some("sk-very-secret")
    );
    assert!(!file(dir.path()).contains("sk-very-secret"));
    state.poll_ai();
    let row = state
        .fields()
        .into_iter()
        .find(|f| f.label == "API key")
        .unwrap();
    assert_eq!(row.value_text(), "(stored)");
}

#[test]
fn removing_a_profile_clears_what_pointed_at_it() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    state.apply_list_op(ListOp::Add(None));
    state.config.ai.actions.stash_message.profile = "profile".into();
    state.apply_list_op(ListOp::Remove(1));
    assert!(state.config.ai.profiles.iter().all(|p| p.id != "profile"));
    assert_eq!(state.config.ai.actions.stash_message.profile, "");
    assert!(!file(dir.path()).contains("id = \"profile\""));
}

#[test]
fn an_action_profile_combo_starts_on_default_and_persists_a_pick() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    state.config.ai.profiles.push(AiProfile::new(
        "second",
        "Second",
        AiProviderKind::Anthropic,
    ));
    select_in(&mut state, "Commit message generation", "Profile");
    match &state.fields()[state.field].control {
        Control::Choice { options, index } => {
            assert_eq!(options[0], "Default (Default)");
            assert_eq!(options.len(), 3);
            assert_eq!(*index, 0);
        }
        other => panic!("{other:?}"),
    }
    state.write_ai_profiles();
    state.choose_next(); // -> the first real profile
    state.choose_next(); // -> "Second"
    let raw = file(dir.path());
    assert!(raw.contains("[ai.actions.commit_message]"), "{raw}");
    assert_eq!(state.config.ai.actions.commit_message.profile, "second");
    // Back to "Default (...)": the follow-the-default empty value.
    select_in(&mut state, "Commit message generation", "Profile");
    state.choose_next(); // wraps to the first entry
    assert_eq!(state.config.ai.actions.commit_message.profile, "");
}

#[test]
fn the_last_profile_cannot_be_removed() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    assert_eq!(state.config.ai.profiles.len(), 1);
    // The list disables Remove at one item, and the op refuses anyway.
    assert_eq!(state.ai_list_spec().min_len, 1);
    state.apply_list_op(ListOp::Remove(0));
    assert_eq!(state.config.ai.profiles.len(), 1);
    state.apply_list_op(ListOp::Add(None));
    assert_eq!(state.config.ai.profiles.len(), 2);
    state.apply_list_op(ListOp::Remove(1));
    assert_eq!(state.config.ai.profiles.len(), 1);
}

#[test]
fn a_profile_is_duplicated_without_its_key() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    state.config.ai.profiles[0].token_ref = "ref".into();
    state.config.ai.profiles[0].model = "m".into();
    state.apply_list_op(ListOp::Duplicate(0));
    let profiles = &state.config.ai.profiles;
    assert_eq!(profiles.len(), 2);
    assert_ne!(profiles[0].id, profiles[1].id);
    assert_eq!(profiles[1].model, "m");
    assert!(profiles[1].token_ref.is_empty());
    assert_eq!(state.ai.selected_index(), 1);
    assert!(file(dir.path()).contains(&profiles[1].id));
}

#[test]
fn the_default_profile_combo_lists_every_profile() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    state.apply_list_op(ListOp::Add(None));
    select(&mut state, "Default profile");
    match &state.fields()[state.field].control {
        Control::Choice { options, index } => {
            assert_eq!(options.len(), 2);
            assert_eq!(*index, 0);
        }
        other => panic!("{other:?}"),
    }
    state.choose_next();
    assert_eq!(state.config.ai.default_profile, "profile");
}

#[test]
fn the_legacy_block_is_hidden_when_empty_and_deletable_otherwise() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    assert!(!state
        .fields()
        .iter()
        .any(|f| f.label.contains("Old presets")));
    state
        .config
        .ai
        .legacy_presets
        .push(gitpoulpi_config::AiPreset {
            id: "terse".into(),
            name: "Terse".into(),
            system_prompt: "Be terse.".into(),
            ..Default::default()
        });
    state.write_ai_list("ai.legacy_presets", &state.config.ai.legacy_presets.clone());
    assert!(state
        .fields()
        .iter()
        .any(|f| f.label == "Old presets (from an earlier version)"));
    // Collapsed: the preset itself is not listed yet.
    assert!(!state.fields().iter().any(|f| f.label.starts_with("Terse")));
    state.ai_run_button(AiButton::ToggleLegacy);
    let fields = state.fields();
    let row = fields
        .iter()
        .find(|f| f.label.starts_with("Terse"))
        .expect("expanded");
    assert!(!row.selectable(), "read-only");
    assert_eq!(row.value_text(), "Be terse.");
    state.ai_run_button(AiButton::DeleteLegacy);
    assert!(state.config.ai.legacy_presets.is_empty());
    assert!(!file(dir.path()).contains("legacy_presets"));
    assert!(!state
        .fields()
        .iter()
        .any(|f| f.label.contains("Old presets")));
}

pub(super) fn local_agent_state(dir: &Path) -> PreferencesState {
    let mut state = state(dir);
    state
        .config
        .ai
        .profiles
        .push(AiProfile::new("cc", "CC", AiProviderKind::LocalAgent));
    state.config.ai.profiles.remove(0);
    state.config.ai.enabled = true;
    state.remember_sub_tab(PrefSection::Ai, SubTab::List);
    state
}

#[test]
fn the_agent_type_and_the_command_are_separate_rows() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = local_agent_state(dir.path());
    let labels: Vec<_> = state.fields().into_iter().map(|f| f.label).collect();
    assert!(labels.contains(&"Agent type".to_string()));
    let command = state
        .fields()
        .into_iter()
        .find(|f| f.label == "Command")
        .unwrap();
    // The command is free text, not a pick list.
    assert!(matches!(command.control, Control::Text(_)));

    // Picking a type writes its command...
    state.ai_commit_combo(AiTarget::ProfileHarness, "codex");
    let p = &state.config.ai.profiles[0];
    assert_eq!((p.command.as_str(), p.harness.as_str()), ("codex", "codex"));
    assert_eq!(p.args, ["exec", "-"]);
    // ...the user then edits it into an alias, which keeps the type...
    state.ai_commit_text(AiTarget::ProfileCommand, "/opt/bin/work-codex");
    let p = &state.config.ai.profiles[0];
    assert_eq!(p.command, "/opt/bin/work-codex");
    assert_eq!(p.harness, "codex");
    // ...and picking another type writes that type's command again.
    state.ai_commit_combo(AiTarget::ProfileHarness, "claude");
    let p = &state.config.ai.profiles[0];
    assert_eq!(p.command, "claude");
    assert_eq!(p.harness, "claude");
    assert_eq!(p.args, ["-p"]);
    let raw = file(dir.path());
    assert!(raw.contains("harness = \"claude\""), "{raw}");
    // "Other" turns the suggestions off.
    state.ai_commit_combo(AiTarget::ProfileHarness, "other");
    assert!(!state.fields().iter().any(|f| f.label == "Argument options"));
}

#[test]
fn a_profile_from_before_the_harness_key_keeps_its_type_through_an_alias() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = local_agent_state(dir.path());
    state.config.ai.profiles[0].harness.clear();
    state.ai_commit_text(AiTarget::ProfileCommand, "my-claude");
    assert_eq!(state.config.ai.profiles[0].harness, "claude");
    assert_eq!(state.config.ai.profiles[0].args, ["-p"]);
}

#[test]
fn env_vars_are_added_from_suggestions_edited_masked_and_removed() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = local_agent_state(dir.path());
    // A suggested name takes its suggested value; NAME=value is split.
    state.ai_commit_combo(AiTarget::ProfileEnvAdd, "DISABLE_TELEMETRY");
    state.ai_commit_combo(AiTarget::ProfileEnvAdd, "MY_FLAG = on");
    state.ai_commit_combo(AiTarget::ProfileEnvAdd, "ANTHROPIC_API_KEY=sk-ant-secret");
    state.ai_commit_combo(AiTarget::ProfileEnvAdd, "bad name");
    let env = &state.config.ai.profiles[0].env;
    assert_eq!(env["DISABLE_TELEMETRY"], "1");
    assert_eq!(env["MY_FLAG"], "on");
    assert!(!env.contains_key("bad name"));
    let raw = file(dir.path());
    assert!(raw.contains("[ai.profiles.env]"), "{raw}");
    assert!(raw.contains("DISABLE_TELEMETRY = \"1\""), "{raw}");

    // The secret value is never shown in a row; the others are.
    let fields = state.fields();
    let value_of = |name: &str| {
        fields
            .iter()
            .find(|f| f.label == format!("Env {name}"))
            .unwrap_or_else(|| panic!("no row for {name}"))
            .value_text()
    };
    assert!(value_of("MY_FLAG").starts_with("on"));
    assert!(!value_of("ANTHROPIC_API_KEY").contains("sk-ant"));
    assert!(!draw(&mut state, 160, 60).contains("sk-ant-secret"));
    // A suggestion already set is no longer offered.
    let add = fields
        .iter()
        .find(|f| f.label == "Add an environment variable")
        .unwrap();
    match &add.control {
        Control::Combo { options, .. } => {
            assert!(!options.iter().any(|(_, v)| v == "DISABLE_TELEMETRY"));
            assert!(options.iter().any(|(_, v)| v == "ANTHROPIC_MODEL"));
        }
        other => panic!("{other:?}"),
    }

    // Edit a plain value as a text field; a secret is typed from scratch
    // and an empty draft leaves it alone.
    let n = state.config.ai.profiles[0]
        .env
        .keys()
        .position(|k| k == "MY_FLAG")
        .unwrap();
    state.ai_commit_text(AiTarget::ProfileEnv(n), "off");
    assert_eq!(state.config.ai.profiles[0].env["MY_FLAG"], "off");
    let k = state.config.ai.profiles[0]
        .env
        .keys()
        .position(|k| k == "ANTHROPIC_API_KEY")
        .unwrap();
    state.ai_commit_text(AiTarget::ProfileEnv(k), "");
    assert_eq!(
        state.config.ai.profiles[0].env["ANTHROPIC_API_KEY"],
        "sk-ant-secret"
    );

    state.ai_run_button(AiButton::EnvRemove(n));
    assert!(!state.config.ai.profiles[0].env.contains_key("MY_FLAG"));
    assert!(!file(dir.path()).contains("MY_FLAG"));
}
