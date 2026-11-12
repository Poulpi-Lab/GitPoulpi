use super::*;

#[test]
fn the_instructions_editor_takes_several_lines_and_saves_on_ctrl_s() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    select(&mut state, "Custom instructions");
    state.activate();
    assert!(state.ai.prompt_open());
    type_text(&mut state, "Be brief.");
    state.on_key(key(KeyCode::Enter));
    type_text(&mut state, "In English.");
    state.on_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(!state.ai.prompt_open());
    assert_eq!(
        state.config.ai.global_instructions,
        "Be brief.\nIn English."
    );
    let loaded = Config::load(&dir.path().join("config.toml")).unwrap();
    assert_eq!(loaded.ai.global_instructions, "Be brief.\nIn English.");
}

#[test]
fn escape_discards_an_instructions_edit() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    select(&mut state, "Custom instructions");
    state.activate();
    type_text(&mut state, "junk");
    state.on_key(key(KeyCode::Esc));
    assert!(
        state.ai.prompt_open(),
        "edited text asks before it is thrown away"
    );
    // Keep editing is the first button, Discard the second.
    state.on_key(key(KeyCode::Right));
    state.on_key(key(KeyCode::Enter));
    assert!(!state.ai.prompt_open());
    assert_eq!(state.config.ai.global_instructions, "");

    // An untouched editor closes on Esc without asking.
    state.activate();
    assert!(state.ai.prompt_open());
    state.on_key(key(KeyCode::Esc));
    assert!(!state.ai.prompt_open());
}

#[test]
fn the_page_has_a_section_per_action_with_chips_and_instructions() {
    let dir = tempfile::tempdir().unwrap();
    let state = state(dir.path());
    let fields = state.fields();
    for title in [
        "Global instructions",
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
    let chips = |s: AiSection| s.chips().len();
    assert_eq!(chips(AiSection::Global), 5);
    assert_eq!(chips(AiSection::Action(AiAction::CommitMessage)), 6);
    assert_eq!(chips(AiSection::Action(AiAction::ExplainChanges)), 4);
    assert_eq!(chips(AiSection::Action(AiAction::StashMessage)), 3);
    assert_eq!(
        chips(AiSection::Action(AiAction::PullRequestDescription)),
        3
    );
    assert!(chips(AiSection::Action(AiAction::CommitComposer)) >= 3);
    // The first profile entry follows the default.
    select_in(&mut { state }, "Explain changes", "Profile");
}

#[test]
fn a_chip_inserts_at_the_caret_once_and_keeps_what_was_typed() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    select_in(
        &mut state,
        "Commit message generation",
        "Custom instructions",
    );
    state.activate();
    let expected = "Use the conventional commits format: type: description.";
    // Tab moves to the row, Enter inserts the first chip and returns.
    state.on_key(key(KeyCode::Tab));
    state.on_key(key(KeyCode::Enter));
    assert_eq!(state.ai.prompt.as_ref().unwrap().text, expected);
    assert!(!state.ai.prompt.as_ref().unwrap().chips_focused);
    // Inserting it again changes nothing and says why.
    state.on_key(key(KeyCode::Tab));
    state.on_key(key(KeyCode::Enter));
    let edit = state.ai.prompt.as_ref().unwrap();
    assert_eq!(edit.text, expected);
    assert!(edit.note.is_some());
    // Arrows choose the next chip; Space inserts it after a space.
    state.on_key(key(KeyCode::Tab));
    state.on_key(key(KeyCode::Right));
    state.on_key(key(KeyCode::Char(' ')));
    assert_eq!(
        state.ai.prompt.as_ref().unwrap().text,
        format!("{expected} Keep the subject line to 50 characters or fewer.")
    );
    // Typing goes on at the caret, after the sentence.
    type_text(&mut state, "!");
    assert!(state.ai.prompt.as_ref().unwrap().text.ends_with("fewer.!"));
    // Nothing is saved until Save; then only this action holds it.
    assert_eq!(state.config.ai.actions.commit_message.instructions, "");
    state.on_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(state
        .config
        .ai
        .actions
        .commit_message
        .instructions
        .starts_with(expected));
    assert_eq!(state.config.ai.actions.explain_changes.instructions, "");
}

#[test]
fn a_chip_lands_in_the_middle_of_the_text_at_the_caret() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    select(&mut state, "Custom instructions");
    state.activate();
    type_text(&mut state, "AB");
    state.on_key(key(KeyCode::Left));
    state.on_key(key(KeyCode::Tab));
    state.on_key(key(KeyCode::Right));
    state.on_key(key(KeyCode::Enter));
    assert_eq!(
        state.ai.prompt.as_ref().unwrap().text,
        "A Keep responses concise. B"
    );
}

#[test]
fn the_page_has_no_chip_rows_and_the_editor_has_them_above_the_text() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    state.focus = PrefFocus::Fields;
    select(&mut state, "Custom instructions");
    let page = draw(&mut state, 120, 40);
    assert!(!page.contains("Quick insert"), "{page}");
    assert!(!page.contains("+ Respond in"), "{page}");
    assert!(state.fields().iter().all(|f| f.label != "Quick insert"));
    state.activate();
    let screen = draw(&mut state, 120, 40);
    assert!(
        screen.contains("Quick insert:  + Respond in <language> "),
        "{screen}"
    );
    let chip_row = screen
        .lines()
        .position(|l| l.contains("Quick insert"))
        .unwrap();
    let hint = state.ai.prompt.as_ref().unwrap();
    assert!(hint.text.is_empty());
    // The first text row sits below the chips.
    let (rect, chip) = *state
        .ai
        .chip_hits
        .iter()
        .find(|(_, c)| *c == 1)
        .expect("the second chip has a rect");
    assert_eq!(rect.y as usize, chip_row);
    let painted: String = screen
        .lines()
        .nth(rect.y as usize)
        .unwrap()
        .chars()
        .skip(rect.x as usize)
        .collect();
    assert!(
        painted.starts_with(" + Keep responses concise "),
        "{painted}"
    );
    state.on_mouse(MouseEventKind::Down(MouseButton::Left), rect.x + 2, rect.y);
    assert_eq!(chip, 1);
    assert_eq!(
        state.ai.prompt.as_ref().unwrap().text,
        "Keep responses concise."
    );
    // Already inserted: shown dimmed (the hit stays) and a second click does nothing.
    state.on_mouse(MouseEventKind::Down(MouseButton::Left), rect.x + 2, rect.y);
    assert_eq!(
        state.ai.prompt.as_ref().unwrap().text,
        "Keep responses concise."
    );
    let again = draw(&mut state, 120, 40);
    assert!(again.contains("already in the instructions"), "{again}");
}

#[test]
fn an_actions_textarea_saves_to_its_own_key() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    select_in(
        &mut state,
        "Stash message generation",
        "Custom instructions",
    );
    state.activate();
    type_text(&mut state, "Mention the ticket.");
    state.on_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert_eq!(
        state.config.ai.actions.stash_message.instructions,
        "Mention the ticket."
    );
    assert_eq!(state.config.ai.global_instructions, "");
}
