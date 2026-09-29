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
fn the_mouse_reaches_tabs_combos_the_checkbox_and_the_buttons() {
    let mut d = dialog();
    let (_, _) = paint(&mut d, 110, 34);
    // A tab.
    let tab = d.hit_rect(PrHit::Tab(0));
    d.on_mouse(click(tab.x + 1, tab.y));
    assert_eq!(d.tab, 0);
    d.on_mouse(click(d.hit_rect(PrHit::Tab(1)).x + 1, tab.y));
    assert_eq!(d.tab, 1);
    let _ = paint(&mut d, 110, 34);
    // A combobox opens on a click and a click on a row picks it.
    let field = d.hit_rect(PrHit::Part(PrFocus::ToBranch));
    d.on_mouse(click(field.x + 1, field.y));
    assert!(d.to_branch.is_open());
    let _ = paint(&mut d, 110, 34);
    let (item, _) = d.to_branch.item_rects()[0];
    // A searchable popup: one click selects, the second (double click) picks.
    d.on_mouse(click(item.x + 2, item.y));
    assert!(d.to_branch.is_open(), "a single click only selects");
    d.on_mouse(click(item.x + 2, item.y));
    assert!(!d.to_branch.is_open());
    assert_eq!(d.to_branch.selected(), Some("develop"));
    let _ = paint(&mut d, 110, 34);
    // The checkbox toggles, Create submits, Cancel cancels.
    let draft = d.hit_rect(PrHit::Part(PrFocus::Draft));
    d.on_mouse(click(draft.x + 1, draft.y));
    assert!(d.draft);
    let create = d.hit_rect(PrHit::Part(PrFocus::Create));
    assert!(matches!(
        d.on_mouse(click(create.x + 2, create.y)),
        PullRequestOutcome::Submit { .. }
    ));
    let cancel = d.hit_rect(PrHit::Part(PrFocus::Cancel));
    assert_eq!(
        d.on_mouse(click(cancel.x + 1, cancel.y)),
        PullRequestOutcome::Cancel
    );
}

#[test]
fn a_press_right_after_a_popup_pick_never_reaches_the_generate_button_under_it() {
    let mut d = dialog();
    d.ai_generate = true;
    let _ = paint(&mut d, 110, 34);
    let generate = d.hit_rect(PrHit::Part(PrFocus::Generate));
    let field = d.hit_rect(PrHit::Part(PrFocus::ToBranch));
    d.on_mouse(click(field.x + 1, field.y));
    let _ = paint(&mut d, 110, 34);
    let popup = d.to_branch.popup_rect();
    assert!(
        popup.contains((generate.x + 1, generate.y).into()),
        "{popup:?}"
    );
    let (item, _) = d.to_branch.item_rects()[0];
    // A searchable popup: one click selects, the second (double click) picks.
    d.on_mouse(click(item.x + 2, item.y));
    assert!(d.to_branch.is_open(), "a single click only selects");
    d.on_mouse(click(item.x + 2, item.y));
    assert!(!d.to_branch.is_open());
    let _ = paint(&mut d, 110, 34);
    // The second press of a double click: swallowed, nothing generates.
    assert_eq!(
        d.on_mouse(click(generate.x + 1, generate.y)),
        PullRequestOutcome::Stay
    );
    assert!(!d.generating);
    // A press elsewhere lifts the shield; the button works again.
    d.on_mouse(click(1, 1));
    assert!(matches!(
        d.on_mouse(click(generate.x + 1, generate.y)),
        PullRequestOutcome::Generate(_)
    ));
}
