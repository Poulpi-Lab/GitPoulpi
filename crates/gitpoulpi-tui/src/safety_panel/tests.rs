//! The panel's state machine, driven without a terminal or a daemon (the
//! PTY suites `pty_undo_history`, `pty_reflog` and `pty_lost_commits` cover
//! the same flows end to end).

use std::sync::Arc;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use gitpoulpi_core::domain::{
    EntryState, EntrySummary, JournalListing, ListedEntry, OperationKind, RestoreOffer,
};

use super::input::act_for_key;
use super::*;

pub(super) fn key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

pub(super) fn entry(id: u64, label: &str) -> ListedEntry {
    ListedEntry {
        source: Default::default(),
        outside: None,
        summary: EntrySummary {
            id,
            kind: OperationKind::Commit,
            label: label.to_string(),
            at: 0,
            state: EntryState::Done,
        },
        changes: Vec::new(),
        restore: RestoreOffer::default(),
    }
}

pub(super) fn panel_with(undo: &[(u64, &str)], redo: &[(u64, &str)]) -> SafetyPanel {
    let activity = Arc::new(crate::activity::Activity::default());
    let mut panel = SafetyPanel::open(activity, Tab::History, None, 100);
    panel.history.set(JournalListing {
        undo: undo.iter().map(|(id, l)| entry(*id, l)).collect(),
        redo: redo.iter().map(|(id, l)| entry(*id, l)).collect(),
        retired: Vec::new(),
    });
    panel
}

#[test]
fn rows_are_newest_first_with_the_undone_ones_after_them() {
    let panel = panel_with(
        &[(1, "old"), (2, "new")],
        &[(3, "undone-older"), (4, "undone-top")],
    );
    let ids: Vec<u64> = panel
        .history
        .rows
        .iter()
        .map(|r| r.entry.summary.id)
        .collect();
    assert_eq!(ids, vec![2, 1, 4, 3]);
    assert_eq!(panel.history.undo_count, 2);
    assert!(panel.history.rows[2].redo);
}

#[test]
fn the_arrow_keys_move_the_selection() {
    let mut panel = panel_with(&[(1, "a"), (2, "b"), (3, "c")], &[]);
    panel.on_key(key(KeyCode::Down));
    panel.on_key(key(KeyCode::Down));
    assert_eq!(panel.history.list.selected, 2);
    panel.on_key(key(KeyCode::Up));
    assert_eq!(panel.history.list.selected, 1);
}

#[test]
fn undo_up_to_the_third_row_asks_about_all_three_newest_first() {
    let mut panel = panel_with(&[(1, "a"), (2, "b"), (3, "c")], &[]);
    panel.on_key(key(KeyCode::Down));
    panel.on_key(key(KeyCode::Down));
    panel.on_key(key(KeyCode::Char('u')));
    let Some(Overlay::Confirm(confirm)) = panel.overlay.as_ref() else {
        panic!("a confirmation was expected");
    };
    assert_eq!(confirm.lines, vec!["c", "b", "a"]);
    let Next::Steps { dir, ids, .. } = &confirm.next else {
        panic!("steps expected");
    };
    assert_eq!(*dir, Direction::Undo);
    assert_eq!(ids, &vec![3, 2, 1]);
    assert!(confirm.red, "undoing several actions is a red confirmation");
}

#[test]
fn redo_up_to_a_deeper_undone_row_redoes_the_ones_before_it() {
    let mut panel = panel_with(&[(1, "a")], &[(2, "older"), (3, "top")]);
    // Rows: a, top, older. The list paints older, top, the divider, then a:
    // select "older" (the first line).
    panel.history.list.selected = 0;
    panel.on_key(key(KeyCode::Char('y')));
    let Some(Overlay::Confirm(confirm)) = panel.overlay.as_ref() else {
        panic!("a confirmation was expected");
    };
    let Next::Steps { dir, ids, .. } = &confirm.next else {
        panic!("steps expected");
    };
    assert_eq!(*dir, Direction::Redo);
    assert_eq!(ids, &vec![3, 2]);
}

#[test]
fn restore_only_this_is_offered_only_when_the_entry_can_be_restored() {
    let mut panel = panel_with(&[(1, "a")], &[]);
    let state = panel.journal_state();
    assert!(!panel.actions(&state).contains(&Act::RestoreOnly));
    panel.history.rows[0].entry.restore = RestoreOffer {
        items: vec![gitpoulpi_core::domain::RestoreItem {
            kind: gitpoulpi_core::domain::RestoreItemKind::Branch,
            label: "branch x (abc1234)".into(),
        }],
        not_restorable: None,
    };
    assert!(panel.actions(&state).contains(&Act::RestoreOnly));
}

#[test]
fn escape_closes_the_panel_and_a_confirmation_first() {
    let mut panel = panel_with(&[(1, "a")], &[]);
    panel.on_key(key(KeyCode::Char('u')));
    assert!(panel.overlay.is_some());
    assert!(matches!(
        panel.on_key(key(KeyCode::Esc)),
        PanelOutcome::Stay
    ));
    assert!(panel.overlay.is_none());
    assert!(matches!(
        panel.on_key(key(KeyCode::Esc)),
        PanelOutcome::Close
    ));
}

#[test]
fn a_blocked_top_entry_with_a_reflog_position_offers_the_restore() {
    let mut state = gitpoulpi_core::domain::JournalState {
        undo: Some(entry(9, "Commit \"x\" (abc1234)").summary),
        ..Default::default()
    };
    let oid = gitpoulpi_core::domain::Oid::from_hex(&"a".repeat(40)).unwrap();
    state.blocked = Some(UndoRefusal::RefMoved {
        name: "refs/heads/main".into(),
        expected: Some(oid),
        actual: None,
        restore: Some(Box::new(gitpoulpi_core::domain::ReflogPosition {
            refname: "refs/heads/main".into(),
            oid,
            index: 1,
            time: 0,
            message: "commit: x".into(),
        })),
    });
    let (name, got, op) = offer_of(&state).expect("an offer");
    assert_eq!((name.as_str(), got), ("main", oid));
    assert!(op.starts_with("Commit"));
    state.blocked = Some(UndoRefusal::IndexChanged);
    assert!(offer_of(&state).is_none());
}

#[test]
fn the_reflog_tab_restores_the_selected_position_of_a_named_branch() {
    let activity = Arc::new(crate::activity::Activity::default());
    let mut panel = SafetyPanel::open(activity, Tab::Reflog, Some("topic".into()), 100);
    let oid = |c: char| gitpoulpi_core::domain::Oid::from_hex(&c.to_string().repeat(40)).unwrap();
    panel.reflog.entries = vec![
        ReflogEntry {
            old: oid('a'),
            new: oid('b'),
            committer: "A <a@x>".into(),
            time: 0,
            message: "branch: Reset".into(),
        },
        ReflogEntry {
            old: oid('0'),
            new: oid('a'),
            committer: "A <a@x>".into(),
            time: 0,
            message: "branch: Created".into(),
        },
    ];
    panel.on_key(key(KeyCode::Down));
    assert_eq!(panel.reflog.list.selected, 1);
    let state = panel.journal_state();
    assert!(panel.actions(&state).contains(&Act::RestoreRef));
    panel.on_key(key(KeyCode::Char('r')));
    assert!(matches!(panel.job, Some(Job::ReflogRestore { .. })));
}

/// The x of the first cell of `needle` on buffer row `y`, in cells.
pub(super) fn column_of(buf: &ratatui::buffer::Buffer, y: u16, needle: &str) -> Option<u16> {
    let (mut text, mut starts) = (String::new(), Vec::new());
    let mut skip = 0;
    for x in 0..buf.area.width {
        // The cell after a wide glyph is its blank continuation.
        if skip > 0 {
            skip -= 1;
            continue;
        }
        let sym = buf[(x, y)].symbol();
        skip = crate::display_width(sym).saturating_sub(1);
        for _ in sym.chars() {
            starts.push(x);
        }
        text.push_str(sym);
    }
    let byte = text.find(needle)?;
    Some(starts[text[..byte].chars().count()])
}

fn age_column_xs(lang: gitpoulpi_i18n::Lang) -> (u16, u16) {
    let _guard = gitpoulpi_i18n::override_lang(lang);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let mut panel = panel_with(&[(1, "Stage all changes"), (2, "Commit")], &[]);
    for row in &mut panel.history.rows {
        row.entry.summary.at = now;
    }
    // Row 0 (id 2) carries a tag, row 1 does not.
    panel.history.rows[0].entry.restore = RestoreOffer {
        items: vec![gitpoulpi_core::domain::RestoreItem {
            kind: gitpoulpi_core::domain::RestoreItemKind::Branch,
            label: "branch x".into(),
        }],
        not_restorable: None,
    };
    let area = Rect::new(0, 0, 100, 30);
    let mut buf = ratatui::buffer::Buffer::empty(area);
    panel.render(render::safety_rect(area), &mut buf);
    let age = rows::age(now);
    let ys: Vec<u16> = (0..area.height)
        .filter(|y| column_of(&buf, *y, &age).is_some())
        .collect();
    assert_eq!(ys.len(), 2, "two rows show an age");
    (
        column_of(&buf, ys[0], &age).unwrap(),
        column_of(&buf, ys[1], &age).unwrap(),
    )
}

#[test]
fn the_age_column_lines_up_on_rows_with_and_without_a_tag() {
    for lang in [gitpoulpi_i18n::Lang::En, gitpoulpi_i18n::Lang::Ja] {
        let (tagged, untagged) = age_column_xs(lang);
        assert_eq!(tagged, untagged, "{lang:?}");
    }
}

// -- palette twins ---------------------------------------------------------

fn oid_of(c: char) -> gitpoulpi_core::domain::Oid {
    gitpoulpi_core::domain::Oid::from_hex(&c.to_string().repeat(40)).unwrap()
}

#[test]
fn a_palette_row_acts_on_the_selected_history_row() {
    let mut panel = panel_with(&[(1, "a"), (2, "b"), (3, "c")], &[]);
    let state = panel.journal_state();
    panel.on_key(key(KeyCode::Down));
    let (outcome, why) = panel.run_command(SafetyCmd::UndoTo, &state);
    assert!(matches!(outcome, PanelOutcome::Stay));
    assert!(why.is_none());
    let Some(Overlay::Confirm(c)) = &panel.overlay else {
        panic!("a confirmation listing the rows");
    };
    // Newest first: the selected row is the second one, so rows 3 and 2.
    let Next::Steps { ids, .. } = &c.next else {
        panic!("steps");
    };
    assert_eq!(ids, &vec![3, 2]);
}

#[test]
fn a_palette_row_that_cannot_run_says_why() {
    let mut panel = panel_with(&[(1, "a")], &[(2, "undone")]);
    let state = panel.journal_state();
    // Restore only this: the entry has nothing to restore.
    panel.history.rows[0].entry.restore = RestoreOffer {
        items: Vec::new(),
        not_restorable: Some("it only changed the index".into()),
    };
    let (_, why) = panel.run_command(SafetyCmd::RestoreOnly, &state);
    assert!(why
        .as_deref()
        .unwrap()
        .contains("it only changed the index"));
    assert!(panel.note.as_ref().is_some_and(|n| n.error));
    assert!(panel.overlay.is_none());
    // Redo on an entry that is not undone.
    let (_, why) = panel.run_command(SafetyCmd::RedoTo, &state);
    assert_eq!(why.as_deref(), Some(m::WHY_NOT_UNDONE.get()));
    // Undo on an undone one (the undone entries are painted above).
    panel.on_key(key(KeyCode::Up));
    let (_, why) = panel.run_command(SafetyCmd::UndoTo, &state);
    assert_eq!(why.as_deref(), Some(m::WHY_ALREADY_UNDONE.get()));
    // The blocked-undo offer: nothing blocked.
    let (_, why) = panel.run_command(SafetyCmd::Offer, &state);
    assert!(why.is_some());
}

#[test]
fn palette_rows_of_another_view_are_refused_and_close_closes() {
    let mut panel = panel_with(&[(1, "a")], &[]);
    let state = panel.journal_state();
    let (outcome, why) = panel.run_command(SafetyCmd::ReflogRestore, &state);
    assert!(matches!(outcome, PanelOutcome::Stay));
    assert_eq!(why.as_deref(), Some(m::WHY_OPEN_REFLOG.get()));
    assert!(matches!(
        panel.run_command(SafetyCmd::Close, &state).0,
        PanelOutcome::Close
    ));
}

#[test]
fn the_reflog_and_lost_work_rows_act_on_their_selected_row() {
    let activity = Arc::new(crate::activity::Activity::default());
    let mut panel = SafetyPanel::open(activity, Tab::Reflog, Some("topic".into()), 100);
    panel.reflog.entries = vec![
        ReflogEntry {
            old: oid_of('a'),
            new: oid_of('b'),
            committer: "A <a@x>".into(),
            time: 0,
            message: "branch: Reset".into(),
        },
        ReflogEntry {
            old: oid_of('0'),
            new: oid_of('a'),
            committer: "A <a@x>".into(),
            time: 0,
            message: "branch: Created".into(),
        },
    ];
    panel.on_key(key(KeyCode::Down));
    let state = panel.journal_state();
    let (outcome, _) = panel.run_command(SafetyCmd::ReflogShow, &state);
    assert!(matches!(
        outcome,
        PanelOutcome::Effect(PanelEffect::ShowCommit(o)) if o == oid_of('a')
    ));
    let (_, why) = panel.run_command(SafetyCmd::ReflogBranch, &state);
    assert!(why.is_none());
    let Some(Overlay::Name(n)) = &panel.overlay else {
        panic!("a name prompt");
    };
    assert!(matches!(n.purpose, NameFor::Branch { at } if at == oid_of('a')));
    // A tag-like ref cannot be restored: says so.
    panel.overlay = None;
    panel.reflog.refname = "refs/tags/v1".into();
    let (_, why) = panel.run_command(SafetyCmd::ReflogRestore, &state);
    assert_eq!(why.as_deref(), Some(m::WHY_NO_REF.get()));
    // The Lost work view, with no row yet.
    panel.set_tab(Tab::Lost);
    let (_, why) = panel.run_command(SafetyCmd::LostPick, &state);
    assert_eq!(why.as_deref(), Some(m::WHY_NO_ROW.get()));
}

#[test]
fn a_reflog_with_more_pages_ends_in_a_load_more_row_that_only_loads() {
    let activity = Arc::new(crate::activity::Activity::default());
    let mut panel = SafetyPanel::open(activity, Tab::Reflog, Some("topic".into()), 100);
    panel.reflog.entries = vec![ReflogEntry {
        old: oid_of('a'),
        new: oid_of('b'),
        committer: "A <a@b>".into(),
        time: 1_700_000_000,
        message: "commit: x".into(),
    }];
    let state = panel.journal_state();
    // Nothing more: no extra row, no action.
    assert_eq!(panel.reflog.rows(), 1);
    assert!(!panel.actions(&state).contains(&Act::LoadMore));
    let (_, why) = panel.run_command(SafetyCmd::ReflogMore, &state);
    assert_eq!(why.as_deref(), Some(m::WHY_NO_MORE.get()));

    panel.reflog.more = true;
    assert_eq!(panel.reflog.rows(), 2);
    assert!(panel.reflog.is_more_row(1) && !panel.reflog.is_more_row(0));
    // On a position the button comes after the position's own actions.
    assert_eq!(panel.actions(&state).last(), Some(&Act::LoadMore));
    assert!(panel.actions(&state).contains(&Act::ShowInLog));
    // On the row itself it is the only action (Enter runs it).
    panel.on_key(key(KeyCode::Down));
    assert_eq!(panel.reflog.list.selected, 1);
    assert_eq!(panel.actions(&state), vec![Act::LoadMore]);
    assert!(panel.selected_reflog().is_none());
}

fn press(col: u16, row: u16) -> crossterm::event::MouseEvent {
    crossterm::event::MouseEvent {
        kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
        column: col,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

fn painted(panel: &mut SafetyPanel) -> String {
    let area = ratatui::layout::Rect::new(0, 0, 130, 40);
    let mut buf = ratatui::buffer::Buffer::empty(area);
    panel.render(safety_rect(area), &mut buf);
    crate::dialog_kit::buffer_text(&buf)
}

#[test]
fn the_cross_and_a_click_outside_close_the_panel_and_a_click_inside_does_not() {
    let mut panel = panel_with(&[(1, "old")], &[]);
    let text = painted(&mut panel);
    assert!(text.contains('\u{2715}'), "{text}");
    assert!(!text.contains("Esc"), "no Esc hint\n{text}");
    let frame = panel.frame;
    // Inside the frame, on nothing: stays open.
    assert!(matches!(
        panel.on_mouse(press(frame.x + 2, frame.y + 2)),
        PanelOutcome::Stay
    ));
    assert!(matches!(panel.on_mouse(press(0, 0)), PanelOutcome::Close));
    let cross = panel.close.rect().expect("a cross");
    assert!(matches!(
        panel.on_mouse(press(cross.x + 1, cross.y)),
        PanelOutcome::Close
    ));
}

#[test]
fn an_outside_click_cancels_an_open_confirmation_before_closing() {
    let mut panel = panel_with(&[(1, "old"), (2, "new")], &[]);
    painted(&mut panel);
    panel.overlay = Some(Overlay::Confirm(Confirm {
        title: "t".into(),
        lines: vec!["b".into()],
        warning: None,
        button: "Go".into(),
        red: false,
        next: Next::Restore { id: 1, name: None },
    }));
    painted(&mut panel);
    assert!(matches!(panel.on_mouse(press(0, 0)), PanelOutcome::Stay));
    assert!(panel.overlay.is_none(), "the confirmation got Cancel");
    painted(&mut panel);
    assert!(matches!(panel.on_mouse(press(0, 0)), PanelOutcome::Close));
}

fn lost_panel() -> (SafetyPanel, Arc<crate::activity::Activity>) {
    let activity = Arc::new(crate::activity::Activity::default());
    let mut panel = SafetyPanel::open(activity.clone(), Tab::Lost, None, 100);
    panel.lost.commits = vec![gitpoulpi_core::domain::LostCommit {
        oid: oid_of('c'),
        summary: "lost work".into(),
        author: "A".into(),
        time: 0,
        parents: vec![oid_of('d')],
        source: gitpoulpi_core::domain::LostSource::Reflog,
    }];
    panel.lost.loaded = true;
    // The open view asked the daemon for its lists; only the actions count.
    activity.raw.queued();
    (panel, activity)
}

/// Lost work has no "Show in Log" (a lost commit is not in the Log) but a
/// tag, "Apply to working tree" and "Save as stash"; Reflog keeps its own.
#[test]
fn lost_work_offers_tag_apply_and_stash_but_no_show_in_log() {
    let (panel, _) = lost_panel();
    let state = panel.journal_state();
    let acts = panel.actions(&state);
    assert!(!acts.contains(&Act::ShowInLog), "{acts:?}");
    for act in [
        Act::CreateBranch,
        Act::CreateTag,
        Act::CherryPick,
        Act::ApplyToWorktree,
        Act::SaveAsStash,
    ] {
        assert!(acts.contains(&act), "{act:?} in {acts:?}");
    }
    // The letter `s` does nothing in Lost work; it is Show in Log in the Reflog.
    assert_eq!(act_for_key('s', Tab::Lost), None);
    assert_eq!(act_for_key('s', Tab::Reflog), Some(Act::ShowInLog));
    for tab in [Tab::Lost, Tab::Reflog] {
        assert_eq!(act_for_key('t', tab), Some(Act::CreateTag));
        assert_eq!(act_for_key('w', tab), Some(Act::ApplyToWorktree));
        assert_eq!(act_for_key('z', tab), Some(Act::SaveAsStash));
    }
}

#[test]
fn create_tag_asks_for_a_name_that_defaults_to_restored_short_id() {
    let (mut panel, activity) = lost_panel();
    panel.on_key(key(KeyCode::Char('t')));
    let Some(Overlay::Name(n)) = &panel.overlay else {
        panic!("a name prompt");
    };
    assert!(matches!(n.purpose, NameFor::Tag { at } if at == oid_of('c')));
    assert_eq!(n.value, format!("restored-{}", short(oid_of('c'))));
    panel.on_key(key(KeyCode::Enter));
    assert!(matches!(panel.job, Some(Job::Tag { .. })));
    let sent = activity.raw.queued();
    assert_eq!(sent.len(), 1, "{sent:?}");
    assert_eq!(sent[0].0, OperationKind::TagCreate);
    assert_eq!(sent[0].1["at"], serde_json::json!(oid_of('c')));
    assert_eq!(
        sent[0].1["name"],
        format!("restored-{}", short(oid_of('c')))
    );
}

#[test]
fn apply_runs_the_apply_operation_and_save_as_stash_the_stash_one() {
    let (mut panel, activity) = lost_panel();
    panel.on_key(key(KeyCode::Char('w')));
    assert!(matches!(panel.job, Some(Job::Apply { .. })));
    let sent = activity.raw.queued();
    assert_eq!(sent[0].0, OperationKind::ApplyCommit);
    assert_eq!(sent[0].1["commit"], serde_json::json!(oid_of('c')));

    panel.job = None;
    panel.on_key(key(KeyCode::Char('z')));
    assert!(matches!(panel.job, Some(Job::Stash { .. })));
    let sent = activity.raw.queued();
    assert_eq!(sent[0].0, OperationKind::StashFromCommit);
    assert_eq!(sent[0].1["commit"], serde_json::json!(oid_of('c')));
}

/// A failed apply changes nothing and offers the stash; accepting it runs
/// the stash operation, declining leaves the panel as it was.
#[test]
fn a_failed_apply_offers_save_as_stash() {
    let (mut panel, activity) = lost_panel();
    panel.job = Some(Job::Apply {
        commit: oid_of('c'),
        pending: Reply::ready(Err(CoreError::Backend("hunk 1 does not apply".into()))),
    });
    panel.poll();
    let Some(Overlay::Confirm(c)) = &panel.overlay else {
        panic!("a question");
    };
    assert!(
        c.lines[0].contains("hunk 1 does not apply"),
        "{:?}",
        c.lines
    );
    assert_eq!(c.button, m::BTN_STASH.get());
    assert!(matches!(c.next, Next::Stash { commit } if commit == oid_of('c')));
    assert!(panel.job.is_none());

    panel.on_key(key(KeyCode::Enter));
    assert!(matches!(panel.job, Some(Job::Stash { .. })));
    assert!(activity
        .raw
        .queued()
        .iter()
        .any(|(op, _)| *op == OperationKind::StashFromCommit));

    // Esc on the question declines it.
    panel.job = Some(Job::Apply {
        commit: oid_of('c'),
        pending: Reply::ready(Err(CoreError::ApplyRefused(
            gitpoulpi_core::domain::ApplyRefusal::MergeCommit,
        ))),
    });
    panel.poll();
    assert!(panel.overlay.is_some());
    panel.on_key(key(KeyCode::Esc));
    assert!(panel.overlay.is_none() && panel.job.is_none());
}

#[test]
fn the_palette_rows_of_the_new_actions_say_why_they_are_greyed() {
    let activity = Arc::new(crate::activity::Activity::default());
    let mut panel = SafetyPanel::open(activity, Tab::Lost, None, 100);
    let state = panel.journal_state();
    // No row yet.
    for cmd in [
        SafetyCmd::LostTag,
        SafetyCmd::LostApply,
        SafetyCmd::LostStash,
    ] {
        let (_, why) = panel.run_command(cmd, &state);
        assert_eq!(why.as_deref(), Some(m::WHY_NO_ROW.get()), "{cmd:?}");
    }
    // The Reflog rows need the Reflog view.
    for cmd in [
        SafetyCmd::ReflogTag,
        SafetyCmd::ReflogApply,
        SafetyCmd::ReflogStash,
    ] {
        let (_, why) = panel.run_command(cmd, &state);
        assert_eq!(why.as_deref(), Some(m::WHY_OPEN_REFLOG.get()), "{cmd:?}");
    }
}
