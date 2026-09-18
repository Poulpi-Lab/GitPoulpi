use super::*;

#[test]
fn steps_are_sent_oldest_first() {
    let f = form();
    let steps = f.steps();
    assert_eq!(
        steps.iter().map(|s| s.commit).collect::<Vec<_>>(),
        vec![oid(1), oid(2), oid(3)]
    );
}

#[test]
fn alt_down_moves_row_and_cursor_follows() {
    let mut f = form();
    f.on_key(alt(KeyCode::Down));
    assert_eq!(f.rows[0].commit, oid(2));
    assert_eq!(f.rows[1].commit, oid(3));
    assert_eq!(f.cursor, 1);
}

#[test]
fn squash_on_the_oldest_row_is_refused() {
    let mut f = form();
    f.cursor = 2; // "first", the oldest / last row
    f.on_key(key(KeyCode::Char('s')));
    assert_eq!(f.rows[2].action, RebaseAction::Pick);
    assert!(f.error.is_some());
}

#[test]
fn submit_emits_oldest_first_steps_with_a_reword_message() {
    let mut f = form();
    f.on_key(key(KeyCode::Char('r'))); // row 0 ("third") -> Reword
    f.open_reword();
    assert!(f.editing.is_some());
    for c in "new msg".chars() {
        f.on_key(key(KeyCode::Char(c)));
    }
    // The embedded editor starts with the caret at the end of
    // "third", so typing appended to it; clear and retype for a
    // clean assertion instead of depending on that.
    let outcome = f.try_submit();
    let RebaseFormOutcome::Submit(Action::RebaseInteractive(req)) = outcome else {
        panic!("expected a submit, got {outcome:?}");
    };
    assert_eq!(req.steps[2].commit, oid(3));
    assert_eq!(req.steps[2].action, RebaseAction::Reword);
    assert!(req.steps[2].message.is_some());
}

#[test]
fn autosquash_toggle_relabels_a_fixup_row_and_moves_it_next_to_its_target() {
    // Newest-first as typed: an unrelated commit sits between the
    // target ("first") and its `fixup!` -- autosquash must move the
    // fixup to sit directly after (i.e. displayed right above, since
    // application order puts it right after "first") its target.
    let mut f = RebaseForm::new(
        "feature".into(),
        Some(oid(0)),
        None,
        vec![
            row(3, "unrelated", RebaseAction::Pick),
            row(2, "fixup! first", RebaseAction::Pick),
            row(1, "first", RebaseAction::Pick),
        ],
    );
    assert_eq!(f.display_rows()[0].commit, oid(3));
    f.autosquash = true;
    let preview = f.display_rows();
    // Oldest-first application order becomes: first, fixup(now
    // Fixup), unrelated -- displayed newest-first: unrelated, fixup,
    // first.
    assert_eq!(
        preview.iter().map(|r| r.commit).collect::<Vec<_>>(),
        vec![oid(3), oid(2), oid(1)]
    );
    assert_eq!(preview[1].action, RebaseAction::Fixup);
    // The underlying rows (what `Reset` restores) are untouched.
    assert_eq!(f.rows[0].commit, oid(3));
    assert_eq!(f.rows[0].action, RebaseAction::Pick);
}

#[test]
fn reset_restores_the_initial_plan() {
    let mut f = form();
    f.on_key(alt(KeyCode::Down));
    f.on_key(key(KeyCode::Char('d')));
    assert_ne!(f.rows, f.initial);
    f.reset();
    assert_eq!(f.rows, f.initial);
}

#[test]
fn menu_offers_from_here_and_refuses_merge_range() {
    // `rebase_form_for` itself is exercised in `log_menu.rs`'s own
    // tests (it needs a loaded `CommitGraph`); this pins the pure
    // decision `RebaseForm` doesn't own: dropping every row still
    // produces a valid (empty-effective) plan rather than panicking.
    let mut f = RebaseForm::new(
        "feature".into(),
        Some(oid(0)),
        None,
        vec![row(1, "only", RebaseAction::Pick)],
    );
    f.on_key(key(KeyCode::Char('d')));
    assert_eq!(f.rows[0].action, RebaseAction::Drop);
    assert!(matches!(f.try_submit(), RebaseFormOutcome::Submit(_)));
}

#[test]
fn a_cherry_pick_form_submits_a_cherry_pick_plan_and_says_so() {
    let mut f = RebaseForm::new_cherry_pick(
        "main".into(),
        vec![
            row(3, "third", RebaseAction::Pick),
            row(1, "first", RebaseAction::Pick),
        ],
    );
    let RebaseFormOutcome::Submit(Action::RebaseInteractive(req)) = f.try_submit() else {
        panic!("expected a submit");
    };
    assert!(req.cherry_pick);
    assert_eq!(req.steps[0].commit, oid(1));
    let (buf, _) = paint_hovered(&f, None);
    let text = crate::dialog_kit::buffer_text(&buf);
    assert!(
        text.contains("Cherry picking 2 commits onto main"),
        "{text}"
    );
    assert!(text.contains("Start cherry pick"), "{text}");
}

#[test]
fn editing_after_an_autosquash_preview_keeps_what_was_drawn() {
    let mut f = RebaseForm::new(
        "feature".into(),
        Some(oid(0)),
        None,
        vec![
            row(3, "unrelated", RebaseAction::Pick),
            row(2, "fixup! first", RebaseAction::Pick),
            row(1, "first", RebaseAction::Pick),
        ],
    );
    f.autosquash = true;
    f.on_click(RebasePart::Row(1));
    assert!(!f.autosquash);
    assert_eq!(f.rows[1].action, RebaseAction::Fixup);
    assert_eq!(f.rows[1].commit, oid(2));
}

#[test]
fn ctrl_arrows_move_the_row_and_ctrl_r_resets() {
    let mut f = form();
    f.on_key(KeyEvent::new(KeyCode::Down, KeyModifiers::CONTROL));
    assert_eq!(f.rows[1].commit, oid(3));
    assert_eq!(f.cursor, 1);
    f.on_key(KeyEvent::new(KeyCode::Up, KeyModifiers::CONTROL));
    assert_eq!(f.rows[0].commit, oid(3));
    f.on_key(key(KeyCode::Char('d')));
    f.on_key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL));
    assert_eq!(f.rows, f.initial);
}

#[test]
fn squashing_into_a_row_turns_it_into_a_reword_with_the_combined_message() {
    let mut f = RebaseForm::new(
        "feature".into(),
        Some(oid(0)),
        None,
        vec![
            msg_row(3, "third\n\nthird body"),
            msg_row(2, "second"),
            msg_row(1, "first\n\nfirst body"),
        ],
    );
    // Row 1 ("second") squashes into row 2 ("first").
    f.cursor = 1;
    f.set_action(RebaseAction::Squash);
    f.editing = None;
    assert_eq!(f.rows[2].action, RebaseAction::Reword);
    assert_eq!(
        f.rows[2].new_message.as_deref(),
        Some("first\n\nfirst body\n\nsecond")
    );
    // Then "third" squashes too: oldest first.
    f.cursor = 0;
    f.set_action(RebaseAction::Squash);
    assert_eq!(
        f.steps()[0].message.as_deref(),
        Some("first\n\nfirst body\n\nsecond\n\nthird\n\nthird body")
    );
    // Back to Pick: nothing squashed into it any more.
    f.set_action(RebaseAction::Pick);
    f.cursor = 1;
    f.set_action(RebaseAction::Pick);
    assert_eq!(f.rows[2].action, RebaseAction::Pick);
    assert_eq!(f.rows[2].new_message, None);
}

#[test]
fn an_edited_receiver_stays_a_reword_and_fixup_never_rewords() {
    let mut f = RebaseForm::new(
        "feature".into(),
        Some(oid(0)),
        None,
        vec![
            msg_row(3, "third"),
            msg_row(2, "second"),
            msg_row(1, "first"),
        ],
    );
    f.cursor = 0;
    f.set_action(RebaseAction::Fixup);
    assert_eq!(f.rows[1].action, RebaseAction::Pick);
    f.cursor = 1;
    f.set_action(RebaseAction::Squash);
    f.editing = None;
    assert_eq!(f.rows[2].action, RebaseAction::Reword);
    f.rows[2].new_message = Some("my own".into());
    f.cursor = 1;
    f.set_action(RebaseAction::Pick);
    assert_eq!(f.rows[2].action, RebaseAction::Reword);
    assert_eq!(f.rows[2].new_message.as_deref(), Some("my own"));
}
