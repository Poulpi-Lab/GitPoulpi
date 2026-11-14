use super::*;
use crate::test_support::*;
use gitpoulpi_core::domain::{
    BranchRemotes, Ref, Remote, RemoteGroupStatus, RemoteRelation, RemoteTargetState, TargetRef,
    TargetRole,
};

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
    assert_eq!(
        empty.request_close(DismissKind::Outside),
        PromptOutcome::Close
    );
    let mut typed = PromptDialog::stash(false);
    type_str(&mut typed, "x");
    assert_eq!(
        typed.request_close(DismissKind::Outside),
        PromptOutcome::Stay
    );
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
    let PromptOutcome::Submit(actions, said) = press(&mut dialog, PromptPart::Submit) else {
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

fn upstream_refs() -> Vec<Ref> {
    vec![
        remote_branch_at("origin", "alpha", oid(1)),
        remote_branch_at("origin", "feature", oid(2)),
        remote_branch_at("origin", "HEAD", oid(2)),
        remote_branch_at("fork", "feature", oid(3)),
    ]
}

fn remotes() -> Vec<Remote> {
    let remote = |name: &str, url: &str| Remote {
        name: name.to_string(),
        fetch_url: url.to_string(),
        push_url: None,
    };
    vec![
        remote("origin", "https://example.test/a.git"),
        remote("fork", "https://example.test/b.git"),
        remote("mirror", "https://example.test/b"),
    ]
}

/// The daemon's answer for `feature`: `origin` is the upstream, `fork` a push
/// target that is also pulled; `oldbox` is a name the branch still mentions
/// but the repository lost when `gone` is set.
fn status(gone: bool) -> RemoteGroupStatus {
    let target = |remote: &str, branch: &str| TargetRef {
        remote: remote.into(),
        branch: branch.into(),
    };
    let state = |remote: &str, role, relation| RemoteTargetState {
        remote: remote.into(),
        branch: "feature".into(),
        role,
        pull: false,
        tracking: None,
        relation,
        missing: false,
    };
    let mut remotes = BranchRemotes {
        branch: "feature".into(),
        primary: Some(target("origin", "feature")),
        push_to: vec![target("fork", "feature")],
        pull_from: vec![target("fork", "feature")],
        missing: Vec::new(),
    };
    let mut targets = vec![
        state("origin", TargetRole::Primary, RemoteRelation::LocalAhead(2)),
        state(
            "fork",
            TargetRole::PushTo,
            RemoteRelation::Diverged {
                ahead: 2,
                behind: 1,
            },
        ),
    ];
    if gone {
        remotes.push_to.push(target("oldbox", "feature"));
        remotes.missing.push("oldbox".into());
        targets.push(RemoteTargetState {
            missing: true,
            ..state("oldbox", TargetRole::PushTo, RemoteRelation::Absent)
        });
    }
    RemoteGroupStatus { remotes, targets }
}

/// The dialog for `feature` with the daemon's answer already in.
fn remotes_dialog(current: Option<&str>, gone: bool) -> PromptDialog {
    let reply = crate::daemon_link::Reply::ready(Ok(status(gone)));
    let mut dialog =
        PromptDialog::branch_remotes("feature", current, &upstream_refs(), &remotes(), reply);
    dialog.poll();
    dialog
}

#[test]
fn remotes_dialog_lists_same_named_branches_first_with_the_current_one_chosen() {
    let dialog = remotes_dialog(Some("origin/feature"), false);
    assert!(
        !dialog.combo_open(),
        "with an upstream the keyboard starts in the table"
    );
    assert_eq!(dialog.focus, PromptPart::Rows);
    let combo = dialog.combo.as_ref().unwrap().borrow();
    let values: Vec<&str> = combo.options.iter().map(|o| o.value.as_str()).collect();
    assert_eq!(
        values,
        ["fork/feature", "origin/feature", "origin/alpha"],
        "same-named first, HEAD never listed"
    );
    assert_eq!(combo.selected(), Some("origin/feature"));
    drop(combo);
    assert_eq!(
        dialog.extra_parts(),
        [PromptPart::Extra(0), PromptPart::Extra(1)],
        "Rename, and Unset because there is an upstream"
    );
    let bare = remotes_dialog(None, false);
    assert!(bare.combo_open(), "no upstream: the list opens");
    assert_eq!(bare.extra_parts(), [PromptPart::Extra(0)]);
}

#[test]
fn esc_closes_the_open_list_first_then_the_dialog() {
    let mut dialog = remotes_dialog(None, false);
    assert_eq!(dialog.on_key(key(KeyCode::Esc)), PromptOutcome::Stay);
    assert!(!dialog.combo_open());
    assert_eq!(dialog.on_key(key(KeyCode::Esc)), PromptOutcome::Close);
}

#[test]
fn picking_a_branch_then_enter_applies_the_upstream_in_one_request() {
    let mut dialog = remotes_dialog(None, false);
    // Pick the highlighted row (fork/feature, the first), then confirm.
    dialog.on_key(key(KeyCode::Enter));
    assert!(!dialog.combo_open());
    assert_eq!(dialog.focus, PromptPart::Rows, "a pick lands in the table");
    let PromptOutcome::Submit(actions, _) = dialog.on_key(key(KeyCode::Enter)) else {
        panic!("Enter applies");
    };
    // `fork` became the upstream, so it carries no extra flags; `origin` was
    // the upstream and is now an unchecked row.
    assert_eq!(
        actions,
        vec![Action::BranchSetRemotes {
            branch: "feature".into(),
            upstream: Some("fork/feature".into()),
            push_to: vec![],
            pull_from: vec![],
        }]
    );
}

#[test]
fn nothing_changed_closes_instead_of_sending() {
    let mut dialog = remotes_dialog(Some("origin/feature"), false);
    assert_eq!(dialog.on_key(key(KeyCode::Enter)), PromptOutcome::Close);
}

#[test]
fn a_typed_upstream_must_name_a_known_remote() {
    let mut dialog = remotes_dialog(None, false);
    type_str(&mut dialog, "nowhere/x");
    dialog.on_key(key(KeyCode::Enter));
    assert!(!dialog.combo_open());
    let error = dialog.error.clone().expect("refused inside");
    assert!(error.contains("nowhere"), "{error}");
    assert_eq!(dialog.value(), None, "the refused value is not kept");
    // A value on a known remote is accepted.
    dialog.set_focus(PromptPart::Field);
    dialog.on_key(key(KeyCode::Enter));
    type_str(&mut dialog, "origin/brand-new");
    dialog.on_key(key(KeyCode::Enter));
    assert_eq!(dialog.value().as_deref(), Some("origin/brand-new"));
    assert!(dialog.error.is_none());
}

#[test]
fn unset_upstream_submits_none() {
    let mut dialog = remotes_dialog(Some("origin/feature"), false);
    let PromptOutcome::Submit(actions, said) = press(&mut dialog, PromptPart::Extra(1)) else {
        panic!("the Unset button submits");
    };
    assert_eq!(
        actions,
        vec![Action::BranchSetUpstream {
            branch: "feature".into(),
            upstream: None
        }]
    );
    assert!(said.contains("feature"), "{said}");
}

#[test]
fn the_table_toggles_push_and_pull_by_key_and_by_mouse_and_applies_both() {
    let mut dialog = remotes_dialog(Some("origin/feature"), false);
    // Rows: origin (the upstream), fork, mirror. Down to `mirror`.
    dialog.on_key(key(KeyCode::Down));
    dialog.on_key(key(KeyCode::Down));
    dialog.on_key(key(KeyCode::Char(' ')));
    dialog.on_key(key(KeyCode::Char('p')));
    // Ctrl+J is a row-down key in the table, not the apply chord.
    dialog.on_key(ctrl('k'));
    // `fork` loses its Push by a click on its checkbox, keeps Pull too.
    let (_, hits) = paint(&dialog);
    let (rect, _) = hits
        .iter()
        .find(|(_, p)| *p == PromptPart::Push(1))
        .expect("fork's Push box");
    dialog.on_mouse(click(rect.x, rect.y), Some(PromptPart::Push(1)));
    let PromptOutcome::Submit(actions, _) = dialog.on_key(key(KeyCode::Enter)) else {
        panic!("Enter applies");
    };
    assert_eq!(
        actions,
        vec![Action::BranchSetRemotes {
            branch: "feature".into(),
            upstream: None,
            push_to: vec!["mirror".into()],
            pull_from: vec!["fork".into(), "mirror".into()],
        }]
    );
}

#[test]
fn the_upstream_row_has_no_push_or_pull_box_to_toggle() {
    let mut dialog = remotes_dialog(Some("origin/feature"), false);
    let (all, hits) = paint(&dialog);
    assert!(all.contains("upstream"), "{all}");
    assert!(!hits.iter().any(|(_, p)| *p == PromptPart::Push(0)));
    dialog.on_key(key(KeyCode::Char(' ')));
    assert_eq!(dialog.on_key(key(KeyCode::Enter)), PromptOutcome::Close);
}

#[test]
fn rename_on_remote_opens_a_nested_prompt_and_applies_the_new_name() {
    let mut dialog = remotes_dialog(Some("origin/feature"), false);
    dialog.on_key(key(KeyCode::Down));
    dialog.on_key(key(KeyCode::Char('r')));
    assert!(
        dialog.remotes.as_ref().unwrap().rename.is_some(),
        "r opens the prompt"
    );
    let (all, _) = paint(&dialog);
    assert!(all.contains("Branch name on fork"), "{all}");
    // Esc closes only the nested prompt.
    assert_eq!(dialog.on_key(key(KeyCode::Esc)), PromptOutcome::Stay);
    assert!(dialog.remotes.as_ref().unwrap().rename.is_none());
    dialog.on_key(key(KeyCode::Char('r')));
    for _ in 0..7 {
        dialog.on_key(key(KeyCode::Backspace));
    }
    type_str(&mut dialog, "feat-x");
    assert_eq!(dialog.on_key(key(KeyCode::Enter)), PromptOutcome::Stay);
    assert!(dialog.remotes.as_ref().unwrap().rename.is_none());
    let PromptOutcome::Submit(actions, _) = dialog.on_key(key(KeyCode::Enter)) else {
        panic!("Enter applies");
    };
    assert_eq!(
        actions,
        vec![Action::BranchSetRemotes {
            branch: "feature".into(),
            upstream: None,
            push_to: vec!["fork:feat-x".into()],
            pull_from: vec!["fork:feat-x".into()],
        }]
    );
}

#[test]
fn renaming_the_upstream_row_is_refused_inside_the_dialog() {
    let mut dialog = remotes_dialog(Some("origin/feature"), false);
    dialog.on_key(key(KeyCode::Char('r')));
    assert!(dialog.remotes.as_ref().unwrap().rename.is_none());
    assert!(dialog.error.is_some());
}

#[test]
fn a_gone_remote_blocks_apply_until_its_row_is_removed() {
    let mut dialog = remotes_dialog(Some("origin/feature"), true);
    let (all, hits) = paint(&dialog);
    assert!(
        all.contains("remote gone") && all.contains("oldbox"),
        "{all}"
    );
    assert!(hits.iter().any(|(_, p)| matches!(p, PromptPart::Remove(_))));
    assert_eq!(dialog.on_key(key(KeyCode::Enter)), PromptOutcome::Stay);
    assert!(dialog
        .error
        .as_deref()
        .is_some_and(|e| e.contains("oldbox")));
    // Down to the gone row (the last), Del removes it.
    for _ in 0..4 {
        dialog.on_key(key(KeyCode::Down));
    }
    dialog.on_key(key(KeyCode::Delete));
    let (all, _) = paint(&dialog);
    assert!(!all.contains("oldbox"), "{all}");
    let PromptOutcome::Submit(actions, _) = dialog.on_key(key(KeyCode::Enter)) else {
        panic!("removing the stale entry is a change");
    };
    assert_eq!(
        actions,
        vec![Action::BranchSetRemotes {
            branch: "feature".into(),
            upstream: None,
            push_to: vec!["fork".into()],
            pull_from: vec!["fork".into()],
        }]
    );
}

#[test]
fn two_remotes_with_one_address_get_a_warning() {
    let mut dialog = remotes_dialog(Some("origin/feature"), false);
    let (all, _) = paint(&dialog);
    assert!(!all.contains("same URL"), "{all}");
    // `fork` and `mirror` are one address once both are pushed to.
    dialog.on_key(key(KeyCode::Down));
    dialog.on_key(key(KeyCode::Down));
    dialog.on_key(key(KeyCode::Char(' ')));
    let (all, _) = paint(&dialog);
    assert!(
        all.contains("fork and mirror push to the same URL"),
        "{all}"
    );
}

#[test]
fn lfs_track_takes_several_patterns_and_refuses_none() {
    let mut dialog = PromptDialog::lfs_track();
    assert_eq!(dialog.on_key(key(KeyCode::Enter)), PromptOutcome::Stay);
    assert!(dialog.error.is_some());
    type_str(&mut dialog, "*.psd  assets/big.bin");
    let PromptOutcome::Submit(actions, _) = dialog.on_key(key(KeyCode::Enter)) else {
        panic!("Enter submits");
    };
    assert_eq!(
        actions,
        vec![Action::LfsTrack {
            patterns: vec!["*.psd".into(), "assets/big.bin".into()],
            opts: Default::default()
        }]
    );
}

#[test]
fn lfs_untrack_offers_the_tracked_patterns_and_takes_a_typed_one() {
    let tracked = vec!["*.psd".to_string(), "*.zip".to_string()];
    let mut dialog = PromptDialog::lfs_untrack(&tracked);
    let (all, _) = paint(&dialog);
    assert!(all.contains("*.psd") && all.contains("*.zip"), "{all}");
    dialog.on_key(key(KeyCode::Enter));
    assert_eq!(dialog.focus, PromptPart::Submit);
    let PromptOutcome::Submit(actions, _) = dialog.on_key(key(KeyCode::Enter)) else {
        panic!("a second Enter confirms");
    };
    assert_eq!(actions, vec![Action::LfsUntrack(vec!["*.psd".into()])]);
    // Nothing tracked: a typed pattern still works.
    let mut empty = PromptDialog::lfs_untrack(&[]);
    type_str(&mut empty, "*.bin");
    empty.on_key(key(KeyCode::Enter));
    let PromptOutcome::Submit(actions, _) = empty.on_key(key(KeyCode::Enter)) else {
        panic!("a typed pattern is accepted");
    };
    assert_eq!(actions, vec![Action::LfsUntrack(vec!["*.bin".into()])]);
}
