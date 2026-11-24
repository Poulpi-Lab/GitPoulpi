//! The Sequences page's "Create with AI" / "Edit with AI" dialog: the diff
//! model, the request the dialog asks the window for, the proposal it shows
//! and what Accept / Reject write. No model is involved: the answers are
//! built by hand, as the window would hand them over.

use super::super::list_editor::{AddKind, ListOp};
use super::super::sequences_ai::{
    diff, FieldKind, SeqAiWish, SeqDraftAnswer, SeqProblem, StepDiff,
};
use super::super::sequences_page::SeqButton;
use super::*;
use crossterm::event::{KeyEvent, KeyModifiers};
use gitpoulpi_config::sequences::{PauseMode, Scope, SequenceDef, StepSpec};

const INSTALL: &str = "inst-test";

fn press(st: &mut PreferencesState, code: KeyCode, modifiers: KeyModifiers) {
    st.on_key(KeyEvent::new(code, modifiers));
}

fn type_text(st: &mut PreferencesState, text: &str) {
    for c in text.chars() {
        press(st, KeyCode::Char(c), KeyModifiers::NONE);
    }
}

/// The Sequences page over a temp config folder, AI on.
fn page(dir: &Path) -> PreferencesState {
    std::fs::create_dir_all(dir.join(".git")).unwrap();
    let mut st = state_at(dir, "Sequences");
    st.seq.install = Some(INSTALL.into());
    st.focus = PrefFocus::List;
    st.config.ai.enabled = true;
    st
}

fn saved(dir: &Path) -> Config {
    Config::load(&dir.join("config.toml")).unwrap()
}

fn def(id: &str, name: &str, runs: &[&str]) -> SequenceDef {
    SequenceDef {
        id: id.into(),
        name: name.into(),
        steps: runs.iter().map(|r| StepSpec::new(r)).collect(),
        ..SequenceDef::default()
    }
}

fn answer(def: SequenceDef) -> SeqDraftAnswer {
    SeqDraftAnswer {
        def,
        problems: Vec::new(),
        notes: Vec::new(),
        suggested_shortcut: None,
    }
}

/// Adds `def` to the page's config file and selects it.
fn seed(st: &mut PreferencesState, def: SequenceDef) {
    assert!(st.seq_commit(None, def));
}

/// Types `description` into the open dialog and sends it; returns the wish.
fn send(st: &mut PreferencesState, description: &str) -> SeqAiWish {
    type_text(st, description);
    press(st, KeyCode::Char('s'), KeyModifiers::CONTROL);
    st.take_seq_ai_wish().expect("a wish")
}

fn lines(st: &mut PreferencesState) -> String {
    draw_screen(st)
}

// -- the diff model -------------------------------------------------------

#[test]
fn an_added_step_is_marked_added_and_the_others_stay_the_same() {
    let old = def("a", "A", &["stage_all", "commit"]);
    let new = def("a", "A", &["stage_all", "fetch", "commit"]);
    let d = diff(&old, &new);
    assert_eq!(
        d.steps,
        [
            StepDiff::Same(0, 0),
            StepDiff::Added(1),
            StepDiff::Same(1, 2)
        ]
    );
    assert!(d.fields.is_empty());
    assert!(!d.is_empty());
}

#[test]
fn a_removed_step_is_marked_removed() {
    let old = def("a", "A", &["stage_all", "commit", "push"]);
    let new = def("a", "A", &["stage_all", "push"]);
    assert_eq!(
        diff(&old, &new).steps,
        [
            StepDiff::Same(0, 0),
            StepDiff::Removed(1),
            StepDiff::Same(2, 1)
        ]
    );
}

#[test]
fn a_step_with_another_pause_or_option_is_changed_not_replaced() {
    let old = def("a", "A", &["stage_all", "commit"]);
    let mut new = old.clone();
    new.steps[1] = StepSpec::new("commit").with_pause(PauseMode::Always);
    let d = diff(&old, &new);
    assert_eq!(d.steps, [StepDiff::Same(0, 0), StepDiff::Changed(1, 1)]);
    let mut other = old.clone();
    other.steps[0] = StepSpec::new("stage_all").with_option("message", "x");
    assert_eq!(
        diff(&old, &other).steps,
        [StepDiff::Changed(0, 0), StepDiff::Same(1, 1)]
    );
}

#[test]
fn a_replaced_command_is_one_removal_and_one_addition() {
    let old = def("a", "A", &["stage_all", "commit"]);
    let new = def("a", "A", &["stage_all", "stash_push"]);
    let d = diff(&old, &new);
    assert_eq!(
        d.steps
            .iter()
            .filter(|s| matches!(s, StepDiff::Removed(1)))
            .count(),
        1
    );
    assert_eq!(
        d.steps
            .iter()
            .filter(|s| matches!(s, StepDiff::Added(1)))
            .count(),
        1
    );
}

#[test]
fn changed_name_and_description_are_field_changes() {
    let old = def("a", "A", &["commit"]);
    let mut new = old.clone();
    new.name = "B".into();
    new.description = Some("why".into());
    let d = diff(&old, &new);
    assert_eq!(d.fields.len(), 2);
    assert_eq!(d.fields[0].kind, FieldKind::Name);
    assert_eq!(
        (d.fields[0].old.as_str(), d.fields[0].new.as_str()),
        ("A", "B")
    );
    assert_eq!(d.fields[1].kind, FieldKind::Description);
    assert_eq!(d.fields[1].old, "");
}

#[test]
fn identical_sequences_have_an_empty_diff() {
    let old = def("a", "A", &["stage_all", "commit"]);
    assert!(diff(&old, &old.clone()).is_empty());
}

// -- the entries ----------------------------------------------------------

#[test]
fn with_ai_off_the_add_menu_and_the_buttons_do_not_mention_it() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    st.config.ai.enabled = false;
    seed(&mut st, def("a", "Alpha", &["commit"]));
    let AddKind::Menu(menu) = st.seq_list_spec().add else {
        panic!("the Add menu");
    };
    assert!(menu.iter().all(|m| !m.contains("with AI")), "{menu:?}");
    assert!(!lines(&mut st).contains("Edit with AI"));
    // The button and the menu row do nothing either.
    st.seq_button(SeqButton::EditWithAi);
    assert!(!st.seq_ai_dirty());
    st.apply_list_op(ListOp::Add(Some(menu.len())));
    assert_eq!(saved(dir.path()).sequences.len(), 1, "nothing was added");
}

#[test]
fn with_ai_on_the_add_menu_ends_with_create_with_ai_and_the_button_shows() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    seed(&mut st, def("a", "Alpha", &["commit"]));
    let AddKind::Menu(menu) = st.seq_list_spec().add else {
        panic!("the Add menu");
    };
    assert_eq!(
        menu.last().map(String::as_str),
        Some("\u{2726} Create with AI...")
    );
    assert!(lines(&mut st).contains("Edit with AI"));
}

#[test]
fn the_add_menu_row_opens_the_dialog_and_asks_no_model() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    let AddKind::Menu(menu) = st.seq_list_spec().add else {
        panic!("the Add menu");
    };
    st.apply_list_op(ListOp::Add(Some(menu.len() - 1)));
    assert!(st.seq_ai_dirty() || st.seq_overlay_open());
    assert!(st.take_seq_ai_wish().is_none(), "opening sends nothing");
    let screen = lines(&mut st);
    assert!(screen.contains("Create a sequence with AI"), "{screen}");
    assert!(screen.contains("Send context"), "{screen}");
    assert!(
        saved(dir.path()).sequences.is_empty(),
        "nothing is written by opening"
    );
}

// -- Create ---------------------------------------------------------------

fn open_create(st: &mut PreferencesState) {
    st.seq_ai_open_create();
    assert!(st.seq_overlay_open());
}

#[test]
fn send_asks_for_the_description_with_no_context_by_default() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    seed(&mut st, def("taken", "Taken", &["commit"]));
    open_create(&mut st);
    // Enter in the description is a new line, not a send.
    type_text(&mut st, "stage all");
    press(&mut st, KeyCode::Enter, KeyModifiers::NONE);
    assert!(st.take_seq_ai_wish().is_none());
    let SeqAiWish::Send(request) = send(&mut st, "then push") else {
        panic!("a send");
    };
    assert_eq!(request.description, "stage all\nthen push");
    assert!(request.edit.is_none());
    assert_eq!(request.taken_ids, ["taken"]);
    assert!(!request.context);
    assert!(request.ai_profiles.is_empty() && request.open_in_labels.is_empty());
}

#[test]
fn the_context_box_adds_the_names_it_names_and_nothing_else() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    st.config.ai.profiles = vec![gitpoulpi_config::AiProfile {
        id: "fast".into(),
        ..Default::default()
    }];
    open_create(&mut st);
    type_text(&mut st, "x");
    // Tab to the box, Space ticks it.
    press(&mut st, KeyCode::Tab, KeyModifiers::NONE);
    press(&mut st, KeyCode::Char(' '), KeyModifiers::NONE);
    press(&mut st, KeyCode::Char('s'), KeyModifiers::CONTROL);
    let Some(SeqAiWish::Send(request)) = st.take_seq_ai_wish() else {
        panic!("a send");
    };
    assert!(request.context);
    assert_eq!(request.ai_profiles, ["fast"]);
}

#[test]
fn an_empty_description_is_not_sent() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    open_create(&mut st);
    press(&mut st, KeyCode::Char('s'), KeyModifiers::CONTROL);
    assert!(st.take_seq_ai_wish().is_none());
    assert!(lines(&mut st).contains("Describe what you want first."));
}

#[test]
fn the_proposal_lists_its_steps_what_was_left_out_and_the_notes_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    open_create(&mut st);
    send(&mut st, "backup then wipe");
    let mut a = answer(def("backup-2", "Backup 2", &["stage_all", "commit"]));
    a.def.steps[1] = StepSpec::new("commit").with_pause(PauseMode::Always);
    a.problems = vec![SeqProblem {
        position: 4,
        what: "reset_hard".into(),
        reason: "destructive steps are not allowed".into(),
    }];
    a.notes = vec!["No tag step exists.".into()];
    st.seq_ai_answer(Ok(a));
    let screen = lines(&mut st);
    for needle in [
        "Proposed sequence",
        "Name: Backup 2",
        "Id: backup-2",
        "Steps (2)",
        "Pause: Always",
        "Left out",
        "step 4 `reset_hard`: destructive steps are not allowed",
        "Notes from the AI",
        "No tag step exists.",
        "Accept",
        "Edit before saving",
        "Reject",
        "Back",
    ] {
        assert!(screen.contains(needle), "{needle}\n{screen}");
    }
    assert!(
        saved(dir.path()).sequences.is_empty(),
        "nothing is written before Accept"
    );
}

#[test]
fn reject_writes_nothing_and_closes() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    open_create(&mut st);
    send(&mut st, "x");
    st.seq_ai_answer(Ok(answer(def("n", "N", &["commit"]))));
    // Accept, Edit before saving, Reject.
    press(&mut st, KeyCode::Right, KeyModifiers::NONE);
    press(&mut st, KeyCode::Right, KeyModifiers::NONE);
    press(&mut st, KeyCode::Enter, KeyModifiers::NONE);
    assert!(!st.seq_overlay_open());
    assert!(saved(dir.path()).sequences.is_empty());
}

#[test]
fn accept_writes_it_once_and_the_list_selects_it() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    seed(&mut st, def("first", "First", &["commit"]));
    open_create(&mut st);
    send(&mut st, "x");
    st.seq_ai_answer(Ok(answer(def(
        "second",
        "Second",
        &["stage_all", "commit"],
    ))));
    press(&mut st, KeyCode::Enter, KeyModifiers::NONE);
    assert!(!st.seq_overlay_open());
    let file = saved(dir.path());
    let ids: Vec<&str> = file.sequences.iter().map(|d| d.id.as_str()).collect();
    assert_eq!(ids, ["first", "second"]);
    assert_eq!(file.sequences[1].scope, Scope::Installation);
    assert_eq!(st.seq_cur().map(|d| d.id), Some("second".into()));
    assert!(!st.seq.in_steps);
}

#[test]
fn edit_before_saving_opens_the_steps_of_the_new_sequence() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    open_create(&mut st);
    send(&mut st, "x");
    st.seq_ai_answer(Ok(answer(def("n", "N", &["stage_all", "commit"]))));
    press(&mut st, KeyCode::Right, KeyModifiers::NONE);
    press(&mut st, KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(saved(dir.path()).sequences.len(), 1);
    assert!(st.seq.in_steps, "the steps are in front");
    assert_eq!(st.seq_cur().map(|d| d.id), Some("n".into()));
}

#[test]
fn a_proposal_with_no_step_cannot_be_accepted() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    open_create(&mut st);
    send(&mut st, "x");
    st.seq_ai_answer(Ok(answer(def("n", "N", &[]))));
    press(&mut st, KeyCode::Enter, KeyModifiers::NONE);
    assert!(st.seq_overlay_open(), "still open");
    assert!(saved(dir.path()).sequences.is_empty());
    assert!(lines(&mut st).contains("Cannot accept"));
}

#[test]
fn back_returns_to_the_description_to_send_again() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    open_create(&mut st);
    send(&mut st, "first try");
    st.seq_ai_answer(Ok(answer(def("n", "N", &["commit"]))));
    for _ in 0..3 {
        press(&mut st, KeyCode::Right, KeyModifiers::NONE);
    }
    press(&mut st, KeyCode::Enter, KeyModifiers::NONE);
    assert!(lines(&mut st).contains("first try"), "{}", lines(&mut st));
    let SeqAiWish::Send(request) = send(&mut st, " again") else {
        panic!("a send");
    };
    assert_eq!(request.description, "first try again");
}

#[test]
fn a_failed_request_shows_why_and_keeps_the_description() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    open_create(&mut st);
    send(&mut st, "keep me");
    st.seq_ai_answer(Err("AI: no answer".into()));
    let screen = lines(&mut st);
    assert!(screen.contains("AI: no answer"), "{screen}");
    assert!(screen.contains("keep me"), "{screen}");
}

#[test]
fn escape_while_the_request_runs_cancels_it() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    open_create(&mut st);
    send(&mut st, "x");
    press(&mut st, KeyCode::Esc, KeyModifiers::NONE);
    assert_eq!(st.take_seq_ai_wish(), Some(SeqAiWish::Cancel));
    assert!(st.seq_overlay_open(), "still open, waiting for the answer");
}

// -- the dirty guard ------------------------------------------------------

#[test]
fn a_typed_description_asks_before_it_is_dropped_and_blocks_closing_settings() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    open_create(&mut st);
    type_text(&mut st, "half a thought");
    assert!(st.seq_ai_dirty());
    assert!(st.request_close(), "Settings asks first");
    st.close_ask = None;
    press(&mut st, KeyCode::Esc, KeyModifiers::NONE);
    assert!(st.seq_overlay_open(), "a question, not a close");
    assert!(lines(&mut st).contains("Discard"), "{}", lines(&mut st));
    // The question's Esc keeps editing.
    press(&mut st, KeyCode::Esc, KeyModifiers::NONE);
    assert!(st.seq_overlay_open());
    assert!(!lines(&mut st).contains("Discard"));
}

#[test]
fn an_untouched_dialog_closes_at_once_and_is_not_dirty() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    open_create(&mut st);
    assert!(!st.seq_ai_dirty());
    press(&mut st, KeyCode::Esc, KeyModifiers::NONE);
    assert!(!st.seq_overlay_open());
}

// -- Edit -----------------------------------------------------------------

#[test]
fn edit_shows_a_diff_against_the_current_sequence_and_accept_replaces_it() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    seed(
        &mut st,
        def("daily", "Daily", &["stage_all", "commit", "push"]),
    );
    seed(&mut st, def("other", "Other", &["fetch"]));
    st.seq.index = 0;
    st.seq_button(SeqButton::EditWithAi);
    assert!(st.seq_overlay_open());
    let SeqAiWish::Send(request) = send(&mut st, "fetch first, ask before pushing") else {
        panic!("a send");
    };
    assert_eq!(request.edit.as_ref().map(|d| d.id.as_str()), Some("daily"));
    // The model drops `commit`, adds a fetch up front, renames, pauses push.
    let mut new = def("daily", "Daily 2", &["fetch", "stage_all", "push"]);
    new.steps[2] = StepSpec::new("push").with_pause(PauseMode::Always);
    st.seq_ai_answer(Ok(answer(new)));
    let screen = lines(&mut st);
    for needle in [
        "Proposed change",
        "~ Name: \"Daily\" -> \"Daily 2\"",
        "+ 1.",
        "- ",
        "~ 3.",
        "Pause: Always",
    ] {
        assert!(screen.contains(needle), "{needle}\n{screen}");
    }
    assert_eq!(
        saved(dir.path()).sequences[0].name,
        "Daily",
        "nothing changed yet"
    );
    press(&mut st, KeyCode::Enter, KeyModifiers::NONE);
    let file = saved(dir.path());
    assert_eq!(file.sequences.len(), 2, "replaced, not added");
    assert_eq!(file.sequences[0].id, "daily");
    assert_eq!(file.sequences[0].name, "Daily 2");
    let runs: Vec<&str> = file.sequences[0]
        .steps
        .iter()
        .map(|s| s.run.as_str())
        .collect();
    assert_eq!(runs, ["fetch", "stage_all", "push"]);
    assert_eq!(file.sequences[1].name, "Other", "the others are untouched");
}

#[test]
fn an_edit_that_changes_nothing_cannot_be_accepted() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    seed(&mut st, def("daily", "Daily", &["stage_all", "commit"]));
    st.seq_button(SeqButton::EditWithAi);
    send(&mut st, "keep it");
    st.seq_ai_answer(Ok(answer(def("daily", "Daily", &["stage_all", "commit"]))));
    press(&mut st, KeyCode::Enter, KeyModifiers::NONE);
    assert!(st.seq_overlay_open());
    assert!(lines(&mut st).contains("No change"));
}

// -- the suggested shortcut -----------------------------------------------

#[test]
fn a_suggested_key_is_checked_against_the_live_keymap_and_never_bound() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    open_create(&mut st);
    send(&mut st, "x");
    let mut a = answer(def("n", "N", &["commit"]));
    a.suggested_shortcut = Some("Ctrl+,".into());
    st.seq_ai_answer(Ok(a));
    let screen = lines(&mut st);
    assert!(screen.contains("Suggested shortcut: Ctrl+,"), "{screen}");
    assert!(screen.contains("Already used by"), "{screen}");
    // Accepting binds nothing.
    press(&mut st, KeyCode::Enter, KeyModifiers::NONE);
    let text = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert!(!text.contains("keybindings"), "{text}");
}

#[test]
fn a_free_suggested_key_is_said_to_be_free_and_a_chord_of_two_keys_to_be_unusable() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    open_create(&mut st);
    send(&mut st, "x");
    let mut a = answer(def("n", "N", &["commit"]));
    a.suggested_shortcut = Some("Ctrl+Alt+F9".into());
    st.seq_ai_answer(Ok(a));
    assert!(lines(&mut st).contains("It is free."), "{}", lines(&mut st));
    let mut b = answer(def("n", "N", &["commit"]));
    b.suggested_shortcut = Some("Ctrl+G b".into());
    st.seq_ai_answer(Ok(b));
    assert!(
        lines(&mut st).contains("Not a single key."),
        "{}",
        lines(&mut st)
    );
}
