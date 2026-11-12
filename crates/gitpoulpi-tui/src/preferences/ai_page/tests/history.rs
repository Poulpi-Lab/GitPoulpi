//! The instructions editor's snippet buttons are real buttons, and its text
//! has an edit history (Ctrl+Z / Ctrl+Y and the Undo / Redo buttons).

use super::*;

fn ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

fn text(state: &PreferencesState) -> String {
    state.ai.prompt.as_ref().unwrap().text.clone()
}

fn button(state: &PreferencesState, which: PromptButton) -> Rect {
    state
        .ai
        .prompt_hits
        .iter()
        .find(|(_, b)| *b == which)
        .map(|(r, _)| *r)
        .expect("the button")
}

fn can(state: &PreferencesState) -> (bool, bool) {
    let edit = state.ai.prompt.as_ref().unwrap();
    edit.field.caret(()).map_or((false, false), |c| {
        (c.can_undo(&edit.text), c.can_redo(&edit.text))
    })
}

/// The snippet buttons: no bracket, the hand pointer over the text area, and
/// the footer buttons the same.
#[test]
fn the_snippet_buttons_are_real_buttons_with_the_hand_pointer() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    select(&mut state, "Custom instructions");
    state.activate();
    crate::pointer_map::begin_frame();
    let screen = draw(&mut state, 120, 40);
    let chips_row = screen
        .lines()
        .find(|l| l.contains("Quick insert"))
        .expect("the chips row");
    assert!(!chips_row.contains('['), "{chips_row}");
    assert!(!chips_row.contains(']'), "{chips_row}");
    assert!(!state.ai.chip_hits.is_empty());
    for (rect, _) in &state.ai.chip_hits {
        assert_eq!(
            crate::pointer_map::at(rect.x + 1, rect.y),
            Some(crate::pointer_map::Kind::Hand),
            "a snippet button keeps the hand, never the text cursor"
        );
    }
    assert_eq!(state.ai.prompt_hits.len(), 4, "Save, Cancel, Undo, Redo");
    for (rect, _) in &state.ai.prompt_hits {
        assert_eq!(
            crate::pointer_map::at(rect.x + 1, rect.y),
            Some(crate::pointer_map::Kind::Hand)
        );
    }
    let footer = screen
        .lines()
        .find(|l| l.contains("Save") && l.contains("Cancel"))
        .expect("the button row");
    assert!(!footer.contains('['), "{footer}");
    assert!(
        footer.contains("Undo") && footer.contains("Redo"),
        "{footer}"
    );
}

/// Ctrl+Z / Ctrl+Y and the two buttons walk the edits; the buttons are
/// disabled (flat) when there is nothing to step to.
#[test]
fn the_editor_undoes_and_redoes_with_keys_and_buttons() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    select(&mut state, "Custom instructions");
    state.activate();
    draw(&mut state, 120, 40);
    assert_eq!(can(&state), (false, false));
    type_text(&mut state, "one two");
    state.on_key(key(KeyCode::Enter));
    type_text(&mut state, "three");
    assert_eq!(text(&state), "one two\nthree");
    assert_eq!(can(&state), (true, false));
    // Ctrl+Z: the last word, then the newline.
    state.on_key(ctrl('z'));
    assert_eq!(text(&state), "one two\n");
    state.on_key(ctrl('z'));
    assert_eq!(text(&state), "one two");
    assert_eq!(can(&state), (true, true));
    // Ctrl+Y brings it back.
    state.on_key(ctrl('y'));
    assert_eq!(text(&state), "one two\n");
    // The buttons do the same.
    draw(&mut state, 120, 40);
    let undo = button(&state, PromptButton::Undo);
    state.on_mouse(MouseEventKind::Down(MouseButton::Left), undo.x + 1, undo.y);
    assert_eq!(text(&state), "one two");
    draw(&mut state, 120, 40);
    let redo = button(&state, PromptButton::Redo);
    state.on_mouse(MouseEventKind::Down(MouseButton::Left), redo.x + 1, redo.y);
    assert_eq!(text(&state), "one two\n");
    // A snippet insert is one entry and is undone whole.
    draw(&mut state, 120, 40);
    let (chip, _) = *state.ai.chip_hits.iter().find(|(_, c)| *c == 1).unwrap();
    state.on_mouse(MouseEventKind::Down(MouseButton::Left), chip.x + 2, chip.y);
    assert!(text(&state).contains("Keep responses concise."));
    state.on_key(ctrl('z'));
    assert_eq!(text(&state), "one two\n");
    // Nothing to undo: the button does nothing (and the text is untouched).
    while can(&state).0 {
        state.on_key(ctrl('z'));
    }
    assert_eq!(text(&state), "");
    draw(&mut state, 120, 40);
    let undo = button(&state, PromptButton::Undo);
    state.on_mouse(MouseEventKind::Down(MouseButton::Left), undo.x + 1, undo.y);
    assert_eq!(text(&state), "");
}
