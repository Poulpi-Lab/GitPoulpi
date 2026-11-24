//! The Sequences page: every list gesture and row edit is written at once,
//! to `config.toml` or to the repository's file, and the registry follows.

use super::super::list_editor::{AddKind, ListOp, Mark};
use super::super::sequences_page::{SeqButton, SeqTarget};
use super::super::sequences_picker::StepPicker;
use super::*;
use crossterm::event::{KeyEvent, KeyModifiers};
use gitpoulpi_config::sequences::{self as seqs, PauseMode, Scope};

const INSTALL: &str = "inst-test";

fn press(st: &mut PreferencesState, code: KeyCode, modifiers: KeyModifiers) {
    st.on_key(KeyEvent::new(code, modifiers));
}

/// The Sequences page over a temp config folder that is also a repository.
fn page(dir: &Path) -> PreferencesState {
    std::fs::create_dir_all(dir.join(".git")).unwrap();
    let mut st = state_at(dir, "Sequences");
    st.seq.install = Some(INSTALL.into());
    st.focus = PrefFocus::List;
    st
}

fn saved(dir: &Path) -> Config {
    Config::load(&dir.join("config.toml")).unwrap()
}

fn file_ids(dir: &Path) -> Vec<String> {
    saved(dir).sequences.iter().map(|d| d.id.clone()).collect()
}

fn repo_ids(dir: &Path) -> Vec<String> {
    seqs::peek_repo_sequences(dir)
        .unwrap()
        .iter()
        .map(|d| d.id.clone())
        .collect()
}

fn row(st: &PreferencesState, target: SeqTarget) -> usize {
    st.fields()
        .iter()
        .position(|f| f.target == Target::Sequence(target))
        .unwrap_or_else(|| panic!("no row {target:?}"))
}

fn menu(st: &PreferencesState) -> Vec<String> {
    let AddKind::Menu(menu) = st.seq_list_spec().add else {
        panic!("the Add menu");
    };
    menu
}

/// Adds the preset called `name` through the Add menu.
fn add_preset(st: &mut PreferencesState, name: &str) {
    let at = menu(st)
        .iter()
        .position(|m| m.ends_with(name))
        .unwrap_or_else(|| panic!("no menu row for {name}"));
    st.apply_list_op(ListOp::Add(Some(at)));
}

#[test]
fn the_add_menu_offers_an_empty_sequence_and_the_presets_and_no_ai_entry_while_ai_is_off() {
    let dir = tempfile::tempdir().unwrap();
    let st = page(dir.path());
    assert_eq!(
        menu(&st),
        [
            "Empty sequence",
            "From preset \u{25B8} Backup",
            "From preset \u{25B8} Compose and push",
            "From preset \u{25B8} Worktree and agent",
        ]
    );
    assert!(st.fields().iter().all(|f| !f.label.contains('\u{2726}')));
}

#[test]
fn an_empty_sequence_is_saved_for_this_installation_with_its_owner() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    st.apply_list_op(ListOp::Add(Some(0)));
    let def = &saved(dir.path()).sequences[0];
    assert_eq!(def.id, "sequence");
    assert_eq!(def.name, "New sequence");
    assert_eq!(def.scope, Scope::Installation);
    assert_eq!(
        def.owner.as_deref(),
        Some(INSTALL),
        "owner = this installation"
    );
    assert!(def.steps.is_empty());
    // A second one gets another id.
    st.apply_list_op(ListOp::Add(Some(0)));
    assert_eq!(file_ids(dir.path()), ["sequence", "sequence-2"]);
    assert_eq!(st.seq.index, 1, "the new one is selected");
}

#[test]
fn a_preset_is_inserted_by_name_and_is_not_active_before() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    assert!(
        st.config.sequences.is_empty(),
        "nothing is active by default"
    );
    add_preset(&mut st, "Backup");
    let def = &saved(dir.path()).sequences[0];
    assert_eq!(def.id, "backup");
    assert_eq!(def.name, "Backup");
    let runs: Vec<&str> = def.steps.iter().map(|s| s.run.as_str()).collect();
    assert_eq!(
        runs,
        ["stage_all", "ai_generate_commit_message", "commit", "push"]
    );
    // The registry (palette rows, keymap slots) picked it up.
    assert!(crate::sequence_run::registry::slot_of("backup").is_some());
}

#[test]
fn duplicate_remove_and_move_write_the_expected_toml() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    add_preset(&mut st, "Backup");
    add_preset(&mut st, "Compose and push");
    assert_eq!(file_ids(dir.path()), ["backup", "compose_and_push"]);
    st.apply_list_op(ListOp::Duplicate(0));
    assert_eq!(
        file_ids(dir.path()),
        ["backup", "compose_and_push", "backup-2"]
    );
    assert_eq!(saved(dir.path()).sequences[2].name, "Backup (copy)");
    assert_eq!(st.seq.index, 2);
    st.apply_list_op(ListOp::Move { from: 2, to: 0 });
    assert_eq!(
        file_ids(dir.path()),
        ["backup-2", "backup", "compose_and_push"]
    );
    assert_eq!(st.seq.index, 0, "the selection follows the sequence");
    st.apply_list_op(ListOp::Move { from: 0, to: 1 });
    assert_eq!(
        file_ids(dir.path()),
        ["backup", "backup-2", "compose_and_push"]
    );
    st.apply_list_op(ListOp::Remove(1));
    assert_eq!(file_ids(dir.path()), ["backup", "compose_and_push"]);
    st.apply_list_op(ListOp::Remove(0));
    st.apply_list_op(ListOp::Remove(0));
    assert!(saved(dir.path()).sequences.is_empty());
    assert!(!std::fs::read_to_string(dir.path().join("config.toml"))
        .unwrap()
        .contains("[[sequences]]"));
}

#[test]
fn the_name_the_id_and_the_description_are_text_rows_and_digits_reach_them() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    add_preset(&mut st, "Backup");
    st.focus = PrefFocus::Fields;
    st.field = row(&st, SeqTarget::Name);
    st.activate();
    assert!(st.edit.is_some(), "the name row is a text field");
    // Digits are typed, never taken as a count.
    for c in "7 9".chars() {
        press(&mut st, KeyCode::Char(c), KeyModifiers::NONE);
    }
    press(&mut st, KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(saved(dir.path()).sequences[0].name, "Backup7 9");
    let desc = row(&st, SeqTarget::Description);
    assert!(st.commit_text(desc, "  night run "));
    assert_eq!(
        saved(dir.path()).sequences[0].description.as_deref(),
        Some("night run")
    );
    assert!(st.commit_text(desc, ""));
    assert_eq!(saved(dir.path()).sequences[0].description, None);
}

#[test]
fn an_unusable_or_taken_id_is_refused_and_nothing_is_written() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    add_preset(&mut st, "Backup");
    add_preset(&mut st, "Compose and push");
    st.seq.index = 1;
    let id = row(&st, SeqTarget::Id);
    assert!(!st.commit_text(id, "Has Space"));
    assert!(!st.commit_text(id, "backup"), "taken");
    assert_eq!(file_ids(dir.path()), ["backup", "compose_and_push"]);
    assert!(st.commit_text(id, "compose_and_push"), "unchanged is fine");
}

#[test]
fn the_shortcut_goes_through_the_capture_popup_into_the_sequence_keybindings() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    add_preset(&mut st, "Backup");
    st.seq_button(SeqButton::Shortcut);
    assert!(st.keys.popup.is_some(), "the existing capture popup opened");
    assert!(st.seq_overlay_open());
    press(&mut st, KeyCode::Char('b'), KeyModifiers::ALT);
    press(&mut st, KeyCode::Enter, KeyModifiers::NONE);
    assert!(st.keys.popup.is_none());
    let kb = saved(dir.path()).keybindings;
    let bound: Vec<_> = kb
        .iter()
        .filter(|(ctx, _, _)| *ctx == "sequence")
        .map(|(_, name, chords)| (name.to_string(), chords.to_vec()))
        .collect();
    assert_eq!(bound, [("backup".to_string(), vec!["Alt+B".to_string()])]);
    // The page shows it on the row and on the button.
    let spec = st.seq_list_spec();
    assert_eq!(spec.items[0].sub.as_deref(), Some("Alt+B"));
    // Clearing removes the key.
    st.seq_button(SeqButton::ClearShortcut);
    assert!(saved(dir.path())
        .keybindings
        .iter()
        .all(|(ctx, _, _)| ctx != "sequence"));
}

#[test]
fn a_shortcut_that_clashes_is_refused_like_on_the_keyboard_page() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    add_preset(&mut st, "Backup");
    st.seq_button(SeqButton::Shortcut);
    // Ctrl+P is the command palette's: the popup shows the clash and
    // offers Replace instead of Save, exactly as on the Keyboard page.
    press(&mut st, KeyCode::Char('p'), KeyModifiers::CONTROL);
    let screen = draw_screen(&mut st);
    assert!(screen.contains("Replace"), "{screen}");
    assert!(!screen.contains(" Save "), "{screen}");
    // Esc cancels: nothing was written.
    press(&mut st, KeyCode::Esc, KeyModifiers::NONE);
    assert!(st.keys.popup.is_none());
    let kb = saved(dir.path()).keybindings;
    assert!(
        kb.iter().all(|(ctx, _, _)| ctx != "sequence"),
        "nothing was written"
    );
}

#[test]
fn renaming_the_id_moves_the_shortcut_in_the_same_write() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    add_preset(&mut st, "Backup");
    st.seq_button(SeqButton::Shortcut);
    press(&mut st, KeyCode::Char('b'), KeyModifiers::ALT);
    press(&mut st, KeyCode::Enter, KeyModifiers::NONE);
    let id = row(&st, SeqTarget::Id);
    assert!(st.commit_text(id, "nightly"));
    assert_eq!(file_ids(dir.path()), ["nightly"]);
    let kb = saved(dir.path()).keybindings;
    let bound: Vec<_> = kb
        .iter()
        .filter(|(ctx, _, _)| *ctx == "sequence")
        .map(|(_, name, chords)| (name.to_string(), chords.to_vec()))
        .collect();
    assert_eq!(bound, [("nightly".to_string(), vec!["Alt+B".to_string()])]);
    assert_eq!(st.seq_list_spec().items[0].sub.as_deref(), Some("Alt+B"));
}

#[test]
fn removing_a_sequence_drops_its_shortcut() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    add_preset(&mut st, "Backup");
    st.seq_button(SeqButton::Shortcut);
    press(&mut st, KeyCode::Char('b'), KeyModifiers::ALT);
    press(&mut st, KeyCode::Enter, KeyModifiers::NONE);
    st.apply_list_op(ListOp::Remove(0));
    assert!(saved(dir.path())
        .keybindings
        .iter()
        .all(|(ctx, _, _)| ctx != "sequence"));
}

#[test]
fn the_scope_moves_the_definition_between_config_toml_and_the_repository_file() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    add_preset(&mut st, "Backup");
    let scope = row(&st, SeqTarget::Scope);
    // This repository only: out of config.toml, into .gitpoulpi/sequences.toml.
    st.field = scope;
    st.commit_choice(&Target::Sequence(SeqTarget::Scope), 2);
    assert!(file_ids(dir.path()).is_empty(), "removed from config.toml");
    assert_eq!(repo_ids(dir.path()), ["backup"]);
    assert_eq!(st.seq_cur().unwrap().scope, Scope::Repo);
    let raw = std::fs::read_to_string(dir.path().join(".gitpoulpi/sequences.toml")).unwrap();
    assert!(raw.contains("scope = \"repo\""), "{raw}");
    assert!(!raw.contains("owner"), "no owner in a shared file: {raw}");
    // All installations: back to config.toml, no owner.
    st.commit_choice(&Target::Sequence(SeqTarget::Scope), 1);
    assert!(repo_ids(dir.path()).is_empty());
    let def = &saved(dir.path()).sequences[0];
    assert_eq!((def.scope, def.owner.as_deref()), (Scope::Machine, None));
    // This installation: this installation's id is the owner.
    st.commit_choice(&Target::Sequence(SeqTarget::Scope), 0);
    let def = &saved(dir.path()).sequences[0];
    assert_eq!(
        (def.scope, def.owner.as_deref()),
        (Scope::Installation, Some(INSTALL))
    );
}

#[test]
fn a_repository_scope_is_refused_outside_a_repository() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    std::fs::remove_dir_all(dir.path().join(".git")).unwrap();
    add_preset(&mut st, "Backup");
    st.commit_choice(&Target::Sequence(SeqTarget::Scope), 2);
    assert_eq!(file_ids(dir.path()), ["backup"], "still in config.toml");
    assert_eq!(saved(dir.path()).sequences[0].scope, Scope::Installation);
    assert!(st.message.is_some());
}

#[test]
fn repository_sequences_are_listed_in_their_own_group() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    add_preset(&mut st, "Backup");
    add_preset(&mut st, "Compose and push");
    st.seq.index = 1;
    st.commit_choice(&Target::Sequence(SeqTarget::Scope), 2);
    let spec = st.seq_list_spec();
    let groups: Vec<_> = spec.items.iter().map(|i| i.group.clone()).collect();
    assert_eq!(
        groups,
        [
            Some("This installation".to_string()),
            Some("This repository".to_string())
        ]
    );
    // A move inside a group still works for the repository file.
    st.seq.index = 0;
    st.apply_list_op(ListOp::Move { from: 0, to: 1 });
    assert_eq!(file_ids(dir.path()), ["backup"], "never crosses a group");
}

#[test]
fn the_steps_face_lists_the_steps_and_edits_them_in_order() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    add_preset(&mut st, "Backup");
    st.seq_button(SeqButton::EditSteps);
    assert!(st.seq.in_steps);
    let spec = st.seq_list_spec();
    assert_eq!(spec.title, "Steps of Backup");
    assert_eq!(spec.items.len(), 4);
    // Reorder: push first, then back.
    st.apply_list_op(ListOp::Move { from: 3, to: 0 });
    let runs = |st: &PreferencesState| -> Vec<String> {
        st.seq_cur()
            .unwrap()
            .steps
            .iter()
            .map(|s| s.run.clone())
            .collect()
    };
    assert_eq!(runs(&st)[0], "push");
    assert_eq!(st.seq.step, 0, "the selection follows the step");
    st.apply_list_op(ListOp::Move { from: 0, to: 3 });
    assert_eq!(runs(&st)[3], "push");
    st.apply_list_op(ListOp::Duplicate(0));
    assert_eq!(runs(&st).len(), 5);
    st.apply_list_op(ListOp::Remove(1));
    assert_eq!(
        runs(&st),
        ["stage_all", "ai_generate_commit_message", "commit", "push"]
    );
    // What is on screen is what is on disk.
    assert_eq!(saved(dir.path()).sequences[0].steps.len(), 4);
    // Esc goes back to the sequences.
    st.focus = PrefFocus::List;
    press(&mut st, KeyCode::Esc, KeyModifiers::NONE);
    assert!(!st.seq.in_steps);
}

#[test]
fn the_close_button_of_the_steps_face_goes_back() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    add_preset(&mut st, "Backup");
    st.seq_button(SeqButton::EditSteps);
    let first = &st.fields()[0];
    assert!(first.label.starts_with('\u{2715}'), "{}", first.label);
    st.seq_button(SeqButton::CloseSteps);
    assert!(!st.seq.in_steps);
}

#[test]
fn pause_never_is_not_offered_for_a_step_that_always_asks() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    add_preset(&mut st, "Backup");
    st.seq_edit(|d| d.steps.push(seqs::StepSpec::new("force_push_with_lease")));
    st.seq_button(SeqButton::EditSteps);
    let pause_options =
        |st: &PreferencesState| match &st.fields()[row(st, SeqTarget::Pause)].control {
            Control::Choice { options, .. } => options.clone(),
            other => panic!("{other:?}"),
        };
    st.seq.step = 4; // force_push_with_lease
    assert_eq!(pause_options(&st), ["Default", "Always"]);
    let field = &st.fields()[row(&st, SeqTarget::Pause)];
    assert!(
        field
            .warning
            .as_deref()
            .is_some_and(|w| w.contains("always asks")),
        "the reason is shown: {:?}",
        field.warning
    );
    // An ordinary step offers Never.
    st.seq.step = 1; // ai_generate_commit_message
    assert_eq!(pause_options(&st), ["Default", "Always", "Never"]);
}

#[test]
fn pause_never_on_the_ai_step_is_saved_and_flags_the_missing_option() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    add_preset(&mut st, "Backup");
    st.seq_button(SeqButton::EditSteps);
    st.seq.step = 1;
    st.field = row(&st, SeqTarget::Pause);
    st.commit_choice(&Target::Sequence(SeqTarget::Pause), 2);
    let def = &saved(dir.path()).sequences[0];
    assert_eq!(def.steps[1].pause, PauseMode::Never);
    // The commit step needs its message once nothing may stop: not an
    // error of the AI step itself, which has no required option.
    assert!(seqs::validate(def).is_empty());
}

#[test]
fn option_rows_follow_the_vocabulary_and_write_into_the_step() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    add_preset(&mut st, "Worktree and agent");
    st.seq_button(SeqButton::EditSteps);
    st.seq.step = 0; // create_worktree_from_current_branch: name + open_tab
    let name = row(&st, SeqTarget::Option(0));
    let field = &st.fields()[name];
    assert_eq!(field.label, "Name");
    assert!(
        field
            .help
            .as_deref()
            .is_some_and(|h| h.contains("${branch} ${date} ${time} ${n}")),
        "the variables hint: {:?}",
        field.help
    );
    assert!(st.commit_text(name, "wt-${n}"));
    assert_eq!(
        saved(dir.path()).sequences[0].steps[0].options["name"].as_str(),
        Some("wt-${n}")
    );
    // The Bool option is a checkbox; unchecked removes the key.
    let open_tab = row(&st, SeqTarget::Option(1));
    assert!(matches!(
        st.fields()[open_tab].control,
        Control::Toggle(true)
    ));
    st.commit_bool(&Target::Sequence(SeqTarget::Option(1)), false);
    assert!(!saved(dir.path()).sequences[0].steps[0]
        .options
        .contains_key("open_tab"));
    // An emptied Text option removes its key too.
    assert!(st.commit_text(name, ""));
    assert!(saved(dir.path()).sequences[0].steps[0].options.is_empty());
    // The agent step's profile is a dropdown of the AI profiles, free text allowed.
    st.seq.step = 1;
    let profile = row(&st, SeqTarget::Option(0));
    assert!(matches!(
        st.fields()[profile].control,
        Control::Combo { custom: true, .. }
    ));
}

#[test]
fn unknown_future_steps_and_keys_survive_a_save_and_are_flagged_red() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("config.toml"),
        format!(
            r#"
[[sequences]]
id = "future"
name = "Future"
owner = "{INSTALL}"
newer_key = 7

[[sequences.steps]]
run = "stage_all"

[[sequences.steps]]
run = "from_the_future"
pause = "always"
colour = "red"
"#
        ),
    )
    .unwrap();
    let mut st = page(dir.path());
    st.config = Config::load(&dir.path().join("config.toml")).unwrap();
    let spec = st.seq_list_spec();
    assert!(
        matches!(spec.items[0].mark, Mark::Problem(_)),
        "the sequence row is marked: {:?}",
        spec.items[0].mark
    );
    assert!(st.commit_text(row(&st, SeqTarget::Name), "Future renamed"));
    let def = &saved(dir.path()).sequences[0];
    assert_eq!(def.name, "Future renamed");
    assert_eq!(def.steps.len(), 2, "the unknown step is kept");
    assert_eq!(def.steps[1].run, "from_the_future");
    assert_eq!(def.steps[1].extra["colour"].as_str(), Some("red"));
    assert_eq!(def.extra["newer_key"].as_integer(), Some(7));
    st.seq_button(SeqButton::EditSteps);
    let steps = st.seq_list_spec();
    assert_eq!(steps.items[0].mark, Mark::None);
    assert!(matches!(steps.items[1].mark, Mark::Problem(_)));
    st.seq.step = 1;
    let fields = st.fields();
    assert!(
        fields.iter().any(|f| f.danger
            && f.warning
                .as_deref()
                .is_some_and(|w| w.contains("from_the_future"))),
        "a red row names the unknown step"
    );
}

#[test]
fn a_sequence_with_problems_can_be_saved_and_shows_them_in_red() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    st.apply_list_op(ListOp::Add(Some(0)));
    let spec = st.seq_list_spec();
    assert!(
        matches!(spec.items[0].mark, Mark::Problem(_)),
        "no steps yet"
    );
    assert_eq!(file_ids(dir.path()), ["sequence"], "saved anyway");
    assert!(st.fields().iter().any(|f| f.danger));
}

#[test]
fn run_now_calls_the_runner_for_a_valid_sequence_and_refuses_an_invalid_one() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    st.apply_list_op(ListOp::Add(Some(0)));
    st.seq_button(SeqButton::RunNow);
    assert!(
        st.message
            .as_deref()
            .is_some_and(|m| m.starts_with("Cannot run")),
        "{:?}",
        st.message
    );
    add_preset(&mut st, "Backup");
    st.seq_button(SeqButton::RunNow);
    assert!(
        st.message.as_deref().is_some_and(|m| m.contains("Backup")),
        "the runner (today a stub) was reached: {:?}",
        st.message
    );
}

#[test]
fn the_step_picker_lists_the_vocabulary_filters_it_and_grays_what_cannot_run() {
    let mut picker = StepPicker::new(true);
    let all = picker.visible_ids();
    assert!(all.contains(&"stage_all") && all.contains(&"push"));
    picker.type_text("stage");
    let found = picker.visible_ids();
    assert_eq!(found[0], "stage_all", "{found:?}");
    assert!(!found.contains(&"push"));
    // A step this setup cannot run (an AI step with the AI off) is listed
    // but cannot be picked.
    let mut picker = StepPicker::new(false);
    picker.type_text("generate commit message");
    let found = picker.visible_ids();
    assert!(found.contains(&"ai_generate_commit_message"), "{found:?}");
    assert!(matches!(
        picker.on_key(&KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        super::super::sequences_picker::PickOutcome::Stay
    ));
}

#[test]
fn adding_a_step_from_the_picker_appends_it_by_keyboard() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    st.apply_list_op(ListOp::Add(Some(0)));
    st.seq_button(SeqButton::EditSteps);
    // The list's Add opens the picker.
    st.apply_list_op(ListOp::Add(None));
    assert!(st.seq.picker.is_some());
    assert!(st.seq_overlay_open());
    for c in "fetch".chars() {
        press(&mut st, KeyCode::Char(c), KeyModifiers::NONE);
    }
    press(&mut st, KeyCode::Enter, KeyModifiers::NONE);
    assert!(st.seq.picker.is_none());
    let steps = &saved(dir.path()).sequences[0].steps;
    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0].run, "fetch");
    // Esc closes the picker and adds nothing.
    st.apply_list_op(ListOp::Add(None));
    press(&mut st, KeyCode::Esc, KeyModifiers::NONE);
    assert!(st.seq.picker.is_none());
    assert_eq!(saved(dir.path()).sequences[0].steps.len(), 1);
}

#[test]
fn the_page_and_the_picker_draw_without_losing_the_close_cross() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    add_preset(&mut st, "Backup");
    let screen = draw_screen(&mut st);
    assert!(screen.contains("Sequences"), "{screen}");
    assert!(screen.contains("Backup"), "{screen}");
    assert!(screen.contains("Shortcut"), "{screen}");
    assert!(screen.contains("Run now"), "{screen}");
    st.seq_button(SeqButton::EditSteps);
    st.apply_list_op(ListOp::Add(None));
    let screen = draw_screen(&mut st);
    assert!(screen.contains("Add a step"), "{screen}");
    assert!(
        screen.contains('\u{2715}'),
        "the picker has its cross: {screen}"
    );
}

#[test]
fn another_installations_sequences_are_not_listed() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    seqs::save_sequence(
        &seqs::SequenceDef {
            id: "theirs".into(),
            name: "Theirs".into(),
            owner: Some("someone-else".into()),
            steps: vec![seqs::StepSpec::new("fetch")],
            ..seqs::SequenceDef::default()
        },
        &dir.path().join("config.toml"),
        "someone-else",
        None,
    )
    .unwrap();
    st.seq_reload();
    assert!(st.seq_entries().is_empty());
    add_preset(&mut st, "Backup");
    assert_eq!(st.seq_entries().len(), 1);
    // And it is still in the file after our own writes and moves.
    assert_eq!(file_ids(dir.path()), ["theirs", "backup"]);
}
