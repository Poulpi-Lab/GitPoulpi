use super::*;
use crate::test_support::*;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

fn type_str(dialog: &mut PromptDialog, s: &str) {
    for c in s.chars() {
        dialog.on_key(key(KeyCode::Char(c)));
    }
}

fn click(col: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: col,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

fn paint(dialog: &PromptDialog) -> (String, Vec<(Rect, PromptPart)>) {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    let mut hits = Vec::new();
    terminal
        .draw(|frame| hits = dialog.render(frame, frame.area(), None))
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let rows: Vec<String> = (0..24)
        .map(|y| (0..80).map(|x| buf[(x, y)].symbol().to_string()).collect())
        .collect();
    (rows.join("\n"), hits)
}

/// Paints the dialog (so it knows where it is), then clicks the middle of `part`.
fn press(dialog: &mut PromptDialog, part: PromptPart) -> PromptOutcome {
    let (_, hits) = paint(dialog);
    let (rect, _) = hits.iter().find(|(_, p)| *p == part).expect("a control");
    let (col, row) = (rect.x + rect.width / 2, rect.y);
    dialog.on_mouse(click(col, row), Some(part))
}

#[test]
fn enter_stashes_with_the_typed_message_and_an_empty_one_is_fine() {
    let mut dialog = PromptDialog::stash(false);
    type_str(&mut dialog, "my message");
    let PromptOutcome::Submit(actions, _) = dialog.on_key(key(KeyCode::Enter)) else {
        panic!("Enter submits a one-line prompt");
    };
    assert_eq!(
        actions,
        vec![Action::StashPushWith {
            message: "my message".into()
        }]
    );

    let mut empty = PromptDialog::stash(false);
    let PromptOutcome::Submit(actions, _) = empty.on_key(key(KeyCode::Enter)) else {
        panic!("an empty message is allowed");
    };
    assert_eq!(actions, vec![Action::StashPush]);
}

#[test]
fn a_paste_into_a_one_line_prompt_never_submits() {
    let mut dialog = PromptDialog::stash(false);
    dialog.on_paste("line one\r\nline two\n");
    assert_eq!(dialog.text(), "line one line two ");
}

#[test]
fn the_ai_button_exists_only_with_ai_on_and_asks_on_a_press() {
    let off = PromptDialog::stash(false);
    assert!(!off.order().contains(&PromptPart::Ai));
    let mut on = PromptDialog::stash(true);
    assert!(on.order().contains(&PromptPart::Ai));
    assert_eq!(
        press(&mut on, PromptPart::Ai),
        PromptOutcome::StartAi(AiRequest::StashMessage)
    );
    assert!(on.ai_running);
    // Locked while the model writes, then filled with its answer on one line.
    type_str(&mut on, "ignored");
    assert_eq!(on.text(), "");
    assert!(on.fill_from_ai("WIP: a\nsecond line"));
    assert_eq!(on.text(), "WIP: a second line");
    assert!(!on.ai_running);
    // An answer nobody waits for is dropped.
    assert!(!on.fill_from_ai("late"));
}

#[test]
fn a_click_outside_a_typed_one_line_prompt_is_ignored_and_an_empty_one_closes() {
    let mut empty = PromptDialog::stash(false);
    assert_eq!(empty.request_close(DismissKind::Outside), PromptOutcome::Close);
    let mut typed = PromptDialog::stash(false);
    type_str(&mut typed, "x");
    assert_eq!(typed.request_close(DismissKind::Outside), PromptOutcome::Stay);
    // The cross and Esc still close without a question for one line.
    assert_eq!(typed.request_close(DismissKind::Esc), PromptOutcome::Close);
}

#[test]
fn the_tag_message_takes_lines_and_applies_with_the_apply_chord() {
    let mut dialog = PromptDialog::annotate_tag("v1");
    assert_eq!(dialog.on_key(ctrl('s')), PromptOutcome::Stay);
    assert!(dialog.error.is_some(), "an empty message is refused inside");
    type_str(&mut dialog, "first");
    dialog.on_key(key(KeyCode::Enter));
    type_str(&mut dialog, "second");
    assert_eq!(dialog.text(), "first\nsecond");
    assert!(dialog.error.is_none(), "typing clears the reason");
    let PromptOutcome::Submit(actions, _) = dialog.on_key(ctrl('s')) else {
        panic!("the apply chord submits");
    };
    assert_eq!(
        actions,
        vec![Action::TagAnnotate {
            name: "v1".into(),
            message: "first\nsecond".into()
        }]
    );
}

#[test]
fn a_new_annotated_tag_creates_at_its_commit_with_a_prefilled_message() {
    let mut dialog = PromptDialog::tag_message(oid(7), "v2", "proposed text");
    assert_eq!(dialog.text(), "proposed text");
    // Unchanged from what it opened with: closing asks nothing.
    assert_eq!(dialog.request_close(DismissKind::Esc), PromptOutcome::Close);
    let PromptOutcome::Submit(actions, said) = press(&mut dialog, PromptPart::Submit)
    else {
        panic!("the primary button submits");
    };
    assert_eq!(
        actions,
        vec![Action::TagCreateAnnotated {
            name: "v2".into(),
            at: oid(7),
            message: "proposed text".into()
        }]
    );
    assert!(said.contains("v2"), "{said}");
}

#[test]
fn closing_an_edited_message_asks_to_discard_first() {
    let mut dialog = PromptDialog::annotate_tag("v1");
    type_str(&mut dialog, "typed");
    assert_eq!(dialog.request_close(DismissKind::Esc), PromptOutcome::Stay);
    assert!(!dialog.badge_ready(), "the question owns the keyboard");
    // Esc answers Keep editing; the dialog stays.
    assert_eq!(dialog.on_key(key(KeyCode::Esc)), PromptOutcome::Stay);
    assert!(dialog.badge_ready());
    assert_eq!(dialog.text(), "typed");
    // Asked again, Discard (the second button) closes.
    dialog.request_close(DismissKind::Esc);
    dialog.on_key(key(KeyCode::Tab));
    assert_eq!(dialog.on_key(key(KeyCode::Enter)), PromptOutcome::Close);
}

#[test]
fn the_dialog_paints_its_field_and_buttons_and_registers_them() {
    let dialog = PromptDialog::stash(true);
    let (all, hits) = paint(&dialog);
    assert!(all.contains("Stash changes"), "{all}");
    assert!(all.contains("AI message"), "{all}");
    assert!(all.contains("Cancel"), "{all}");
    for part in [
        PromptPart::Field,
        PromptPart::Ai,
        PromptPart::Cancel,
        PromptPart::Submit,
        PromptPart::Close,
    ] {
        assert!(hits.iter().any(|(_, p)| *p == part), "no hit for {part:?}");
    }
}

#[test]
fn the_message_area_hints_its_keys_and_shows_its_error() {
    let mut dialog = PromptDialog::annotate_tag("v1");
    let (all, _) = paint(&dialog);
    assert!(all.contains("Annotate tag v1"), "{all}");
    assert!(all.contains("Enter: new line"), "{all}");
    dialog.on_key(ctrl('s'));
    let (all, _) = paint(&dialog);
    assert!(all.contains("needs a message"), "{all}");
}
