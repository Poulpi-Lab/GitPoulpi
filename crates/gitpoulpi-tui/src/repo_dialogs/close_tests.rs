//! Closing the repository dialogs: the cross on every one, no "Esc" in the
//! footers, a click outside closes a pristine dialog and is swallowed (the
//! cross flashes) when the form holds typed work.

use super::*;
use crate::dialog_kit::buffer_text;
use crate::workspace_ui::dialog::Seed as WsSeed;

const FRAME: Rect = Rect {
    x: 0,
    y: 0,
    width: 130,
    height: 36,
};

fn left_click(col: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: col,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

fn draw(dialogs: &mut RepoDialogs) -> (Buffer, String) {
    let mut buf = Buffer::empty(FRAME);
    dialogs.render(FRAME, &mut buf);
    let text = buffer_text(&buf);
    (buf, text)
}

fn cross_of(dialogs: &mut RepoDialogs) -> Rect {
    dialogs
        .closable()
        .and_then(|(d, _)| d.close_hit().rect())
        .expect("the dialog painted a cross")
}

fn init(dialogs: &mut RepoDialogs) {
    dialogs.dialog = Some(RepoDialog::Init {
        form: InitDialog::new(Path::new("/tmp/gp-close"), "main"),
        picker: None,
    });
}

fn type_in(dialogs: &mut RepoDialogs, text: &str) {
    for c in text.chars() {
        dialogs.on_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
    }
}

fn ws_seed() -> WsSeed {
    WsSeed {
        id: None,
        name: String::new(),
        color: None,
        description: String::new(),
        profile: None,
        members: Vec::new(),
        known: Vec::new(),
        profiles: Vec::new(),
        taken_names: Vec::new(),
        ai_enabled: false,
        ai: gitpoulpi_config::WorkspaceAi::default(),
    }
}

#[test]
fn every_migrated_dialog_paints_a_cross_and_no_esc_hint() {
    let mut cases: Vec<(&str, RepoDialogs)> = Vec::new();
    let mut d = RepoDialogs::new("t");
    init(&mut d);
    cases.push(("init", d));
    let mut d = RepoDialogs::new("t");
    d.dialog = Some(RepoDialog::Clone {
        form: Box::new(CloneDialog::new(None)),
        picker: None,
    });
    cases.push(("clone", d));
    let mut d = RepoDialogs::new("t");
    d.open_sparse();
    cases.push(("sparse", d));
    #[cfg(feature = "worktree")]
    {
        let mut d = RepoDialogs::new("t");
        d.open_worktree(
            WorktreeSeed {
                start: gitpoulpi_core::domain::WorktreeStart::Branch("feature".into()),
                label: "feature".into(),
                new_branch: String::new(),
                branch_busy: false,
            },
            Path::new("/tmp/gp-close/repo"),
        );
        cases.push(("worktree", d));
    }
    let mut d = RepoDialogs::new("t");
    d.dialog = Some(RepoDialog::Workspace {
        form: Box::new(WorkspaceDialog::new(ws_seed())),
        picker: None,
    });
    cases.push(("workspace", d));
    let mut d = RepoDialogs::new("t");
    d.dialog = Some(RepoDialog::Browse(FolderPicker::new(
        "Open a repository",
        "Open",
        PathBuf::from("/tmp"),
    )));
    cases.push(("folder picker", d));
    for (name, mut dialogs) in cases {
        let (buf, text) = draw(&mut dialogs);
        let cross = cross_of(&mut dialogs);
        let glyph: String = (cross.x..cross.right())
            .map(|x| buf[(x, cross.y)].symbol())
            .collect();
        assert_eq!(
            glyph, " \u{2715} ",
            "{name}: the hit rect is the painted cross"
        );
        assert!(
            !text.contains("Esc"),
            "{name} still advertises Esc:\n{text}"
        );
    }
}

#[test]
fn the_cross_closes_a_dialog_even_with_typed_work() {
    let mut dialogs = RepoDialogs::new("t");
    init(&mut dialogs);
    type_in(&mut dialogs, "proj");
    draw(&mut dialogs);
    let cross = cross_of(&mut dialogs);
    dialogs.on_mouse(left_click(cross.x + 1, cross.y));
    assert!(!dialogs.is_open());
}

#[test]
fn a_click_outside_closes_a_pristine_dialog() {
    let mut dialogs = RepoDialogs::new("t");
    init(&mut dialogs);
    draw(&mut dialogs);
    dialogs.on_mouse(left_click(0, 0));
    assert!(!dialogs.is_open());
}

#[test]
fn a_click_outside_a_dirty_dialog_is_swallowed_and_the_cross_flashes() {
    let mut dialogs = RepoDialogs::new("t");
    init(&mut dialogs);
    type_in(&mut dialogs, "proj");
    let (buf, _) = draw(&mut dialogs);
    let cross = cross_of(&mut dialogs);
    let idle = buf[(cross.x + 1, cross.y)].style();
    dialogs.on_mouse(left_click(0, 0));
    assert!(
        dialogs.is_open(),
        "typed text is never lost to a stray click"
    );
    let (buf, _) = draw(&mut dialogs);
    assert_ne!(
        buf[(cross.x + 1, cross.y)].style(),
        idle,
        "the cross flashes"
    );
    // Esc still closes it.
    dialogs.on_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert!(!dialogs.is_open());
}

#[test]
fn a_click_inside_never_closes() {
    let mut dialogs = RepoDialogs::new("t");
    init(&mut dialogs);
    draw(&mut dialogs);
    let rect = init_rect(FRAME);
    dialogs.on_mouse(left_click(rect.x + 3, rect.y + 5));
    assert!(dialogs.is_open());
}

#[test]
fn a_picker_over_a_dialog_closes_on_its_cross_and_leaves_the_dialog() {
    let mut dialogs = RepoDialogs::new("t");
    dialogs.dialog = Some(RepoDialog::Init {
        form: InitDialog::new(Path::new("/tmp/gp-close"), "main"),
        picker: Some(FolderPicker::new("Pick", "Select", PathBuf::from("/tmp"))),
    });
    draw(&mut dialogs);
    let cross = cross_of(&mut dialogs);
    dialogs.on_mouse(left_click(cross.x + 1, cross.y));
    assert!(dialogs.is_open(), "the Init form is still there");
    assert!(dialogs.picker_mut().is_none(), "the picker is gone");
}

/// A Create worktree dialog opened for a sequence step: Create hands the
/// ticket to the effect (so the daemon's answer, with the new path, reports
/// under it); closing the dialog any other way cancels the ticket.
#[cfg(feature = "worktree")]
mod worktree_tickets {
    use super::*;
    use crate::daemon_link::{issue, take_for, StepOutcome};

    fn seed() -> WorktreeSeed {
        WorktreeSeed {
            start: gitpoulpi_core::domain::WorktreeStart::Branch("feature".into()),
            label: "feature".into(),
            new_branch: String::new(),
            branch_busy: false,
        }
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn create_hands_the_ticket_to_the_effect() {
        let tmp = tempfile::tempdir().unwrap();
        let ticket = issue();
        let mut dialogs = RepoDialogs::new("t");
        dialogs.open_worktree_for(
            Some(ticket),
            seed(),
            &tmp.path().join("repo"),
            Vec::new(),
            false,
        );
        if let Some(RepoDialog::Worktree { form, .. }) = dialogs.dialog.as_mut() {
            form.focus = crate::worktree_dialog::WtFocus::Create;
        }
        match dialogs.on_key(key(KeyCode::Enter)) {
            Some(DialogEffect::AddWorktree {
                ticket: carried, ..
            }) => {
                assert_eq!(carried, Some(ticket));
            }
            other => panic!("expected AddWorktree, got {other:?}"),
        }
        assert!(
            take_for(ticket).is_empty(),
            "submitting is not canceling: the ticket reports with the daemon's answer"
        );
    }

    #[test]
    fn closing_the_dialog_cancels_the_ticket() {
        let tmp = tempfile::tempdir().unwrap();
        let ticket = issue();
        let mut dialogs = RepoDialogs::new("t");
        dialogs.open_worktree_for(
            Some(ticket),
            seed(),
            &tmp.path().join("repo"),
            Vec::new(),
            false,
        );
        dialogs.on_key(key(KeyCode::Esc));
        assert!(!dialogs.is_open());
        assert_eq!(take_for(ticket), vec![StepOutcome::Canceled]);
    }

    #[test]
    fn a_window_rebuild_with_the_dialog_open_cancels_the_ticket() {
        let tmp = tempfile::tempdir().unwrap();
        let ticket = issue();
        let mut dialogs = RepoDialogs::new("t");
        dialogs.open_worktree_for(
            Some(ticket),
            seed(),
            &tmp.path().join("repo"),
            Vec::new(),
            false,
        );
        drop(dialogs);
        assert_eq!(take_for(ticket), vec![StepOutcome::Canceled]);
    }

    #[test]
    fn an_untracked_dialog_reports_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let mut dialogs = RepoDialogs::new("t");
        dialogs.open_worktree_for(None, seed(), &tmp.path().join("repo"), Vec::new(), false);
        dialogs.on_key(key(KeyCode::Esc));
        assert!(!dialogs.is_open());
    }
}
