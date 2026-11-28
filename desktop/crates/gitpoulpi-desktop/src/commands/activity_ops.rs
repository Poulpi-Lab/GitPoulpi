//! The undo history, Activity Log, background jobs and grouped push commands: which registry ids the
//! desktop implements as an [`ActivityOp`] and how one is run. Following `refs_ops.rs`: the ids map to an
//! op here, `ops.rs` carries one dispatch line per direction and `exec.rs` one line to run it. The panels
//! are `panels::activity`; this file only opens them and forwards what the person asked.

use gitpoulpi_app::keymap::Action;
use gitpoulpi_app::palette::vocab::activity_log::LevelCmd;
use gitpoulpi_app::palette::vocab::jobs_ui::JobsCmd;
use gitpoulpi_app::palette::vocab::safety_cmd::SafetyCmd;
use gitpoulpi_app::palette::{palette_entry, PaletteCommand};
use gitpoulpi_i18n::msg;
use gitpoulpi_link::undo_label::offer_of;
use gitpoulpi_log::Level;
use gpui_kit::{App, Window};

use super::status;
use crate::app::workspace_of;
use crate::panels::activity::log_model::{Levels, LogTab, Scope};
use crate::panels::activity::safety_model::{Act, Tab};
use crate::panels::activity::{self, activity_log, group, jobs, resolve, safety};

/// What an undo-history, Activity Log, jobs or grouped-push command does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityOp {
    /// Opens the undo-safety panel on a view (the Reflog of HEAD).
    Safety(Tab),
    /// The Reflog of the branch selected in the Refs sidebar.
    ReflogSelected,
    /// A row action or a control of the open undo-safety panel.
    SafetyCmd(SafetyCmd),
    /// Moves the branch back to where the blocked action left it (confirmed).
    UndoOffer,
    /// Drops the blocked newest undo entry (confirmed).
    UndoForget,
    /// Undoes the newest entries when one sequence run made them.
    UndoSequence,
    /// Opens the Activity Log; `global` shows every repository.
    OpenLog {
        global: bool,
    },
    /// A level, tab, scope or scroll command of the Activity Log (opens it when closed).
    Level(LevelCmd),
    CopyLastError,
    OpenLastError,
    ShowJobs,
    CancelJob(u64),
    CancelAllJobs,
    ShowGroupedPush,
}

/// The op of a key action, if the desktop implements it here.
pub fn for_action(action: Action) -> Option<ActivityOp> {
    Some(match action {
        Action::GlobalUndoHistory => ActivityOp::Safety(Tab::History),
        Action::GlobalUndoOffer => ActivityOp::UndoOffer,
        Action::NavOpenActivityLog => ActivityOp::OpenLog { global: false },
        Action::NavOpenGlobalLog => ActivityOp::OpenLog { global: true },
        Action::NavShowJobs => ActivityOp::ShowJobs,
        Action::ActivityLogScrollLeft => ActivityOp::Level(LevelCmd::ScrollLeft),
        Action::ActivityLogScrollRight => ActivityOp::Level(LevelCmd::ScrollRight),
        Action::Palette(_) => return palette_entry(action).and_then(|e| for_palette(e.command)),
        _ => return None,
    })
}

/// The op of a palette command, if the desktop implements it here.
pub fn for_palette(command: PaletteCommand) -> Option<ActivityOp> {
    use PaletteCommand as C;
    Some(match command {
        C::UndoHistory => ActivityOp::Safety(Tab::History),
        C::ReflogHead => ActivityOp::Safety(Tab::Reflog),
        C::LostCommits => ActivityOp::Safety(Tab::Lost),
        C::ReflogSelectedBranch => ActivityOp::ReflogSelected,
        C::UndoOffer => ActivityOp::UndoOffer,
        C::UndoForget => ActivityOp::UndoForget,
        C::UndoSequence => ActivityOp::UndoSequence,
        // The panes of the Reflog and Lost work views, and the dialog frame, are the terminal's.
        C::Safety(
            SafetyCmd::NextPane
            | SafetyCmd::PathTree
            | SafetyCmd::ResetPanes
            | SafetyCmd::ArrangeDialog
            | SafetyCmd::ResetDialog,
        ) => return None,
        C::Safety(cmd) => ActivityOp::SafetyCmd(cmd),
        C::OpenActivityLog => ActivityOp::OpenLog { global: false },
        C::OpenGlobalLog => ActivityOp::OpenLog { global: true },
        C::ActivityLevel(cmd) => ActivityOp::Level(cmd),
        C::CopyLastError => ActivityOp::CopyLastError,
        C::OpenLastError => ActivityOp::OpenLastError,
        C::Jobs(JobsCmd::Show) => ActivityOp::ShowJobs,
        C::Jobs(JobsCmd::Cancel(id)) => ActivityOp::CancelJob(id),
        C::Jobs(JobsCmd::CancelAll) => ActivityOp::CancelAllJobs,
        C::ShowGroupedPush => ActivityOp::ShowGroupedPush,
        _ => return None,
    })
}

/// The panel action a `Safety: ...` row stands for.
fn act_of(cmd: SafetyCmd) -> Option<Act> {
    Some(match cmd {
        SafetyCmd::UndoTo => Act::UndoTo,
        SafetyCmd::RedoTo => Act::RedoTo,
        SafetyCmd::UndoSequence => Act::UndoSequence,
        SafetyCmd::RestoreOnly => Act::RestoreOnly,
        SafetyCmd::Offer => Act::Offer,
        SafetyCmd::ReflogShow => Act::ShowInLog,
        SafetyCmd::ReflogBranch | SafetyCmd::LostBranch => Act::CreateBranch,
        SafetyCmd::ReflogTag | SafetyCmd::LostTag => Act::CreateTag,
        SafetyCmd::ReflogApply | SafetyCmd::LostApply => Act::ApplyToWorktree,
        SafetyCmd::ReflogStash | SafetyCmd::LostStash => Act::SaveAsStash,
        SafetyCmd::ReflogRestore => Act::RestoreRef,
        SafetyCmd::ReflogMore => Act::LoadMore,
        SafetyCmd::LostPick => Act::CherryPick,
        _ => return None,
    })
}

/// Runs `op`.
pub fn run(op: ActivityOp, window: &mut Window, cx: &mut App) {
    let Some(ws) = workspace_of(cx) else {
        return;
    };
    let backend = ws.read(cx).backend().clone();
    match op {
        ActivityOp::Safety(tab) => {
            let name = (tab == Tab::Reflog).then(|| "HEAD".to_string());
            activity::open_safety(tab, name, window, cx);
        }
        ActivityOp::ReflogSelected => match ws.read(cx).refs.read(cx).selected_target() {
            Some(crate::ref_target::RefTarget::Local(name)) => {
                activity::open_safety(Tab::Reflog, Some(name), window, cx);
            }
            _ => status::set(cx, msg::input::SELECT_BRANCH_REFS.get()),
        },
        ActivityOp::SafetyCmd(SafetyCmd::Close) => {
            use gpui_kit::component::WindowExt as _;
            if safety::current(cx).is_some() {
                window.close_dialog(cx);
            }
        }
        ActivityOp::SafetyCmd(cmd) => {
            let Some(view) = safety::current(cx) else {
                return status::set(cx, msg::undo_safety::WHY_OPEN_HISTORY.get());
            };
            view.update(cx, |v, cx| match (cmd, act_of(cmd)) {
                (_, Some(act)) => v.run(act, cx),
                (SafetyCmd::HistoryRefresh, _) => reload(v, Tab::History, cx),
                (SafetyCmd::ReflogRefresh, _) => reload(v, Tab::Reflog, cx),
                (SafetyCmd::LostRefresh, _) => reload(v, Tab::Lost, cx),
                _ => {}
            });
        }
        ActivityOp::UndoOffer => {
            let state = backend.with_link(|l| l.journal_state());
            match offer_of(&state) {
                Some((name, oid, _)) => confirm_restore(name, oid, backend, window, cx),
                None => status::set(cx, msg::undo_safety::WHY_NO_OFFER.get()),
            }
        }
        ActivityOp::UndoForget => {
            if let Err(why) = resolve::ask_forget(backend, window, cx) {
                status::set(cx, why);
            }
        }
        ActivityOp::UndoSequence => {
            let said = backend.with_link(|l| l.start_undo_sequence());
            status::set(cx, said);
        }
        ActivityOp::OpenLog { global } => {
            let scope = if global { Scope::All } else { Scope::ThisRepo };
            activity::open_activity(LogTab::Git, scope, window, cx);
        }
        ActivityOp::Level(cmd) => run_level(cmd, window, cx),
        ActivityOp::CopyLastError => match backend.host().last_error() {
            Some(entry) => {
                let text = entry.to_text();
                let preview: String = text.chars().filter(|c| !c.is_control()).take(40).collect();
                cx.write_to_clipboard(gpui_kit::ClipboardItem::new_string(text));
                status::set(
                    cx,
                    msg::input::copied(msg::activity_log::WHAT_ERROR.get(), preview),
                );
            }
            None => status::set(cx, msg::activity_log::WHY_NO_LAST_ERROR.get()),
        },
        ActivityOp::OpenLastError => {
            if backend.host().last_error().is_none() {
                return status::set(cx, msg::activity_log::WHY_NO_LAST_ERROR.get());
            }
            if let Some(view) = activity::open_activity(LogTab::Git, Scope::ThisRepo, window, cx) {
                view.update(cx, |v, cx| {
                    v.set_levels(Levels::default(), cx);
                    v.set_tab(LogTab::Git, cx);
                    if let Some(i) = (0..v.len()).find(|&i| v.level_of(i) == Level::Error) {
                        v.select(i, cx);
                    }
                });
            }
        }
        ActivityOp::ShowJobs => {
            jobs::open(window, cx);
        }
        ActivityOp::CancelJob(id) => jobs::cancel(&backend, id, cx),
        ActivityOp::CancelAllJobs => {
            if jobs::cancel_all(&backend, cx) == 0 {
                status::set(cx, msg::jobs::PALETTE_NONE.get());
            }
        }
        ActivityOp::ShowGroupedPush => {
            if group::open(window, cx).is_none() {
                status::set(cx, msg::palette::WHY_NO_GROUPED_PUSH.get());
            }
        }
    }
}

fn reload(v: &mut safety::SafetyView, tab: Tab, cx: &mut gpui_kit::Context<safety::SafetyView>) {
    v.set_tab(tab, cx);
    v.reload_current(cx);
}

/// "Restore `name` to where the blocked action left it?", then the reflog restore (one undo entry).
fn confirm_restore(
    name: String,
    oid: gitpoulpi_core::domain::Oid,
    backend: crate::daemon::Shared,
    window: &mut Window,
    cx: &mut App,
) {
    use crate::dialogs::{self, Choice, Spec};
    let spec = Spec {
        title: msg::undo_safety::restore_confirm_ref(&name),
        lines: Vec::new(),
        choices: vec![
            Choice::new(msg::undo_safety::MENU_OFFER.get()),
            Choice::new(msg::dialog::CANCEL.get()),
        ],
        hint: msg::diff_actions::CONFIRM_HINT.get().to_string(),
        highlight_first: false,
        yes_no_keys: false,
    };
    dialogs::open(
        window,
        cx,
        spec,
        move |ix, cx| {
            if ix == 0 {
                backend.submit(gitpoulpi_link::Action::ReflogRestore {
                    refname: name.clone(),
                    oid,
                });
                status::set(cx, msg::undo_safety::restored_ref(&name, undo_key()));
            }
        },
        || {},
    );
}

fn undo_key() -> String {
    let key = gitpoulpi_app::keymap::current().shown(Action::GlobalUndo);
    if key.is_empty() {
        "Ctrl+Z".to_string()
    } else {
        key
    }
}

fn run_level(cmd: LevelCmd, window: &mut Window, cx: &mut App) {
    let tab = match cmd {
        LevelCmd::AppTab | LevelCmd::ClearLog => LogTab::App,
        _ => LogTab::Git,
    };
    let Some(view) = activity_log::current(cx)
        .or_else(|| activity::open_activity(tab, Scope::ThisRepo, window, cx))
    else {
        return;
    };
    view.update(cx, |v, cx| match cmd {
        LevelCmd::Toggle(level) => v.toggle_level(level, cx),
        LevelCmd::All => v.set_levels(Levels::all(), cx),
        LevelCmd::Clear => v.set_levels(Levels::none(), cx),
        LevelCmd::ClearLog => {
            v.set_tab(LogTab::App, cx);
            v.ask_clear(cx);
        }
        LevelCmd::ShowOperation => v.show_operation(cx),
        LevelCmd::NextProblem => v.step_problem(true, cx),
        LevelCmd::PrevProblem => v.step_problem(false, cx),
        LevelCmd::Scope => v.cycle_scope(cx),
        LevelCmd::GitTab => v.set_tab(LogTab::Git, cx),
        LevelCmd::AppTab => v.set_tab(LogTab::App, cx),
        LevelCmd::ScrollLeft => v.scroll_by(-64.0, cx),
        LevelCmd::ScrollRight => v.scroll_by(64.0, cx),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_known_ids_map_to_ops() {
        assert_eq!(
            for_action(Action::GlobalUndoHistory),
            Some(ActivityOp::Safety(Tab::History))
        );
        assert_eq!(
            for_palette(PaletteCommand::Jobs(JobsCmd::Cancel(7))),
            Some(ActivityOp::CancelJob(7))
        );
        assert_eq!(for_palette(PaletteCommand::Refresh), None);
    }

    #[test]
    fn the_terminal_panes_have_no_op() {
        assert_eq!(
            for_palette(PaletteCommand::Safety(SafetyCmd::NextPane)),
            None
        );
        assert!(for_palette(PaletteCommand::Safety(SafetyCmd::UndoTo)).is_some());
    }
}
