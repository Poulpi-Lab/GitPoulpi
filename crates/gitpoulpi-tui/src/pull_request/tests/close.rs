use super::*;

fn click(col: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: col,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

#[test]
fn a_pristine_dialog_closes_on_esc_the_cross_and_outside_with_no_esc_hint() {
    let mut d = dialog();
    let (text, _) = paint(&mut d, 110, 34);
    assert!(text.contains('\u{2715}'), "{text}");
    assert!(!text.contains("Esc"), "no Esc hint\n{text}");
    assert_eq!(d.on_mouse(click(0, 0)), PullRequestOutcome::Cancel);
    let cross = d.guard.close.rect().expect("a cross");
    assert_eq!(
        d.on_mouse(click(cross.x + 1, cross.y)),
        PullRequestOutcome::Cancel
    );
    assert_eq!(d.on_key(key(KeyCode::Esc)), PullRequestOutcome::Cancel);
}

#[test]
fn a_typed_description_asks_before_closing_and_discard_closes() {
    let mut d = dialog();
    d.body = "some words".into();
    let (_, _) = paint(&mut d, 110, 34);
    let cross = d.guard.close.rect().expect("a cross");
    for step in 0..3 {
        let out = match step {
            0 => d.on_key(key(KeyCode::Esc)),
            1 => d.on_mouse(click(cross.x + 1, cross.y)),
            _ => d.on_mouse(click(0, 0)),
        };
        assert_eq!(out, PullRequestOutcome::Stay);
        let (text, _) = paint(&mut d, 110, 34);
        assert!(text.contains("Discard changes?"), "{text}");
        // Esc = Keep editing.
        assert_eq!(d.on_key(key(KeyCode::Esc)), PullRequestOutcome::Stay);
    }
    d.on_key(key(KeyCode::Esc));
    d.on_key(key(KeyCode::Right));
    assert_eq!(d.on_key(key(KeyCode::Enter)), PullRequestOutcome::Cancel);
}

#[test]
fn an_edited_title_counts_as_typed_work_but_the_prefill_does_not() {
    let mut d = dialog();
    assert_eq!(d.on_key(key(KeyCode::Esc)), PullRequestOutcome::Cancel);
    d.title.push('!');
    assert_eq!(d.on_key(key(KeyCode::Esc)), PullRequestOutcome::Stay);
}
