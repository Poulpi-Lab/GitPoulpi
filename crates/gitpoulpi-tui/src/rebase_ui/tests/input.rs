use super::*;

#[test]
fn double_click_and_enter_reword_any_row() {
    let mut f = form();
    f.on_click(RebasePart::Row(1));
    assert!(f.editing.is_none());
    f.on_click(RebasePart::Row(1));
    assert!(f.editing.is_some());
    assert_eq!(f.rows[1].action, RebaseAction::Reword);
    f.editing = None;
    f.on_key(key(KeyCode::Up));
    f.on_key(key(KeyCode::Enter));
    assert!(f.editing.is_some());
    assert_eq!(f.rows[0].action, RebaseAction::Reword);
}

#[test]
fn clicking_an_action_pill_sets_that_action_and_reword_opens_the_editor() {
    let mut f = form();
    f.on_click(RebasePart::Action(1, RebaseAction::Drop));
    assert_eq!(f.rows[1].action, RebaseAction::Drop);
    assert_eq!(f.cursor, 1);
    f.on_click(RebasePart::Action(0, RebaseAction::Reword));
    assert_eq!(f.rows[0].action, RebaseAction::Reword);
    assert!(f.editing.is_some());
}

#[test]
fn dragging_a_row_over_another_reorders_the_plan() {
    let mut f = form();
    f.on_click(RebasePart::Row(0)); // press on "third"
    f.on_drag(Some(RebasePart::Row(1)));
    f.on_drag(Some(RebasePart::Row(2)));
    f.on_release();
    assert_eq!(
        f.rows.iter().map(|r| r.commit).collect::<Vec<_>>(),
        vec![oid(2), oid(1), oid(3)]
    );
    assert_eq!(f.cursor, 2);
    // Released: further pointer motion does nothing.
    f.on_drag(Some(RebasePart::Row(0)));
    assert_eq!(f.rows[2].commit, oid(3));
}

#[test]
fn space_opens_the_action_list_and_enter_picks_from_it() {
    let mut f = form();
    f.on_key(key(KeyCode::Down)); // cursor on "second"
    f.on_key(key(KeyCode::Char(' ')));
    assert_eq!(f.dropdown, Some(1));
    // Highlight starts on the row's current action (Pick).
    assert_eq!(f.dropdown_cursor, 0);
    f.on_key(key(KeyCode::Down));
    f.on_key(key(KeyCode::Down)); // Pick -> Reword -> Squash
    f.on_key(key(KeyCode::Enter));
    assert_eq!(f.dropdown, None);
    assert_eq!(f.rows[1].action, RebaseAction::Squash);
    // Esc closes the list only; a second Esc closes the view.
    f.on_key(key(KeyCode::Char(' ')));
    assert_eq!(f.on_key(key(KeyCode::Esc)), RebaseFormOutcome::Stay);
    assert_eq!(f.dropdown, None);
    assert_eq!(f.on_key(key(KeyCode::Esc)), RebaseFormOutcome::Close);
}

#[test]
fn a_letter_key_with_the_list_open_closes_it_and_still_acts() {
    let mut f = form();
    f.on_key(key(KeyCode::Char(' ')));
    f.on_key(key(KeyCode::Char('d')));
    assert_eq!(f.dropdown, None);
    assert_eq!(f.rows[0].action, RebaseAction::Drop);
}

#[test]
fn clicking_the_button_toggles_the_list_and_a_choice_applies() {
    let mut f = form();
    f.on_click(RebasePart::Dropdown(1));
    assert_eq!(f.dropdown, Some(1));
    f.on_click(RebasePart::Dropdown(1));
    assert_eq!(f.dropdown, None);
    f.on_click(RebasePart::Dropdown(1));
    f.on_click(RebasePart::Action(1, RebaseAction::Drop));
    assert_eq!(f.dropdown, None);
    assert_eq!(f.rows[1].action, RebaseAction::Drop);
    // Any other click closes an open list.
    f.on_click(RebasePart::Dropdown(0));
    f.on_click(RebasePart::Row(2));
    assert_eq!(f.dropdown, None);
}

#[test]
fn a_cherry_pick_offers_four_actions_and_a_rebase_six() {
    let cp = RebaseForm::new_cherry_pick("main".into(), vec![row(1, "a", RebaseAction::Pick)]);
    assert_eq!(cp.choices().len(), 4);
    assert!(!cp.choices().contains(&RebaseAction::Fixup));
    assert_eq!(form().choices().len(), 6);
}

/// `rebase.reset = "Alt+r"`, `rebase.drop = "x"`: the new keys work,
/// the old ones (Ctrl+R, `d`) are inert, and the footer shows them.
#[test]
fn rebound_rebase_keys_work_and_show_in_the_footer() {
    let km =
        crate::keymap::test_keymap(&[("rebase", "reset", &["Alt+r"]), ("rebase", "drop", &["x"])]);
    crate::keymap::with_test_keymap(km, || {
        let mut f = form();
        f.on_key(key(KeyCode::Char('d')));
        assert_eq!(f.rows, f.initial, "the old Drop key is inert");
        f.on_key(key(KeyCode::Char('x')));
        assert_eq!(f.rows[0].action, RebaseAction::Drop);
        f.on_key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL));
        assert_ne!(f.rows, f.initial, "Ctrl+R no longer resets");
        f.on_key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::ALT));
        assert_eq!(f.rows, f.initial, "Alt+R resets");
        let (buf, _) = paint_hovered(&f, None);
        let text = crate::dialog_kit::buffer_text(&buf);
        assert!(
            text.contains("Alt+R reset") && text.contains("x Drop"),
            "{text}"
        );
    });
}
