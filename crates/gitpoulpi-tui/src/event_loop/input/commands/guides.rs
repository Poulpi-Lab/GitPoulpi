//! The guards that explain instead of failing (`guide_dialog.rs`): an action
//! that cannot work yet because of an expected state asks the user about it
//! before anything is sent or run, so no error report is ever produced for
//! it.
//!
//! * Conflicts: the AI commit composer and every way of committing open
//!   "N files still have conflicts" with a button into the merge editor.
//! * AI input limits: a conflict over `ai.limits.conflict_bytes` opens the
//!   "too large" warning (Send anyway / Open setting / Cancel).
//!
//! The guards sit in the entry points *and* the daemon / worker keep their
//! own checks, so a path that skips the dialog still ends with a sentence.

use crossterm::event::{KeyEvent, MouseEvent};

use crate::guide_dialog::{Blocked, Choice, GuideDialog, Kind, Outcome, CONFLICT_LIMIT_KEY};

use super::CommandCtx;
use crate::event_loop::*;

/// The files that still have conflicts, `/`-separated, in the order the
/// merge editor lists them.
pub(in crate::event_loop) fn conflicted_files(cx: &CommandCtx<'_>) -> Vec<String> {
    cx.conflicts
        .iter()
        .map(|e| e.path.to_string_lossy().replace('\\', "/"))
        .collect()
}

/// Opens "Conflicts first" when files still have conflicts. `true`: the
/// action must stop here (the dialog is up).
pub(in crate::event_loop) fn block_on_conflicts(cx: &mut CommandCtx<'_>, what: Blocked) -> bool {
    let files = conflicted_files(cx);
    if files.is_empty() {
        return false;
    }
    *cx.input_mode = InputMode::Guide(Box::new(GuideDialog::conflicts_first(what, &files)));
    true
}

/// Starts an AI request the way every entry point does, after the guards:
/// the commit composer waits for conflicts to be resolved, a conflict over
/// the size limit asks first. Anything the window cannot start anyway (AI
/// off, a request already running) is left to `start_into`, which says so on
/// the status line.
pub(in crate::event_loop) fn start_ai(cx: &mut CommandCtx<'_>, request: AiRequest) {
    if cx.ai_ui.enabled() && !cx.ai_ui.running() {
        match &request {
            AiRequest::Compose if block_on_conflicts(cx, Blocked::Compose) => return,
            AiRequest::ResolveConflict {
                path,
                context,
                oversize_ok: false,
                ..
            } => {
                let (bytes, limit) = (context.size(), cx.ai_ui.conflict_limit());
                if bytes > limit {
                    let dialog = GuideDialog::too_large(request.clone(), path, bytes, limit);
                    *cx.input_mode = InputMode::Guide(Box::new(dialog));
                    return;
                }
            }
            _ => {}
        }
    }
    cx.ai_ui
        .start_into(cx.session, cx.repo, request, &mut *cx.last_message);
}

/// Opens Settings on the limit "Open setting" of the truncation dialog chose
/// (`ai_limit_dialog`), once.
pub(in crate::event_loop) fn open_limit_setting(cx: &mut CommandCtx<'_>) {
    if let Some(key) = crate::ai_limit_dialog::take_open_setting() {
        let mut state = PreferencesState::open(cx.repo_root);
        let _ = state.show_page(None, Some(key));
        *cx.preferences = Some(state);
    }
}

/// A key while the dialog is open.
pub(in crate::event_loop) fn guide_key(cx: &mut CommandCtx<'_>, key: KeyEvent) {
    drive(cx, |dialog| dialog.on_key(key));
}

/// A mouse event while the dialog is open.
pub(in crate::event_loop) fn guide_mouse(cx: &mut CommandCtx<'_>, mouse: MouseEvent) {
    drive(cx, |dialog| dialog.on_mouse(mouse));
}

fn drive(cx: &mut CommandCtx<'_>, f: impl FnOnce(&mut GuideDialog) -> Outcome) {
    let InputMode::Guide(dialog) = &mut *cx.input_mode else {
        return;
    };
    let Outcome::Chose(choice) = f(dialog) else {
        return;
    };
    let InputMode::Guide(dialog) = std::mem::replace(&mut *cx.input_mode, InputMode::Normal) else {
        return;
    };
    apply(cx, *dialog, choice);
}

/// What a button does.
fn apply(cx: &mut CommandCtx<'_>, dialog: GuideDialog, choice: Choice) {
    match (dialog.kind().clone(), choice) {
        (_, Choice::Cancel) => {
            *cx.last_message = Some(gitpoulpi_i18n::msg::input::CANCELED.get().into());
        }
        // The merge editor, on the selected conflicted file or the first.
        (Kind::ConflictsFirst(_), _) => super::open_selected(cx),
        // This request only: the flag skips the size check, and the worker
        // runs the same privacy pipeline as for any request.
        (Kind::TooLarge(request), Choice::Primary) => {
            let mut request = *request;
            if let AiRequest::ResolveConflict { oversize_ok, .. } = &mut request {
                *oversize_ok = true;
            }
            cx.ai_ui
                .start_into(cx.session, cx.repo, request, &mut *cx.last_message);
        }
        // Never in the window's input mode: a worker asks it (`ai_limit_dialog`).
        (Kind::Truncation, _) => {}
        (Kind::OffBranch(off), choice) => apply_off_branch(cx, off, choice),
        (Kind::TooLarge(_), Choice::Truncated) => {}
        (Kind::TooLarge(_), Choice::Pick(_) | Choice::Rewrite | Choice::CreateBranch) => {}
        // Settings, on the limit's row.
        (Kind::TooLarge(_), Choice::Secondary) => {
            let mut state = PreferencesState::open(cx.repo_root);
            let _ = state.show_page(None, Some(CONFLICT_LIMIT_KEY));
            *cx.preferences = Some(state);
        }
    }
}

/// An Edit commit message or Amend on a commit that is not on the checked-out
/// branch asks what to do instead of refusing (`guide_dialog`): check out a
/// branch that holds it, rewrite the history, or make a branch. `true`: the
/// dialog is up and the action stops here.
pub(in crate::event_loop) fn guard_off_branch(
    cx: &mut CommandCtx<'_>,
    action: &LogContextAction,
    commit: Oid,
) -> bool {
    let scope = match action {
        LogContextAction::EditMessage => None,
        LogContextAction::Amend(scope) => Some(*scope),
        _ => return false,
    };
    let refs = cx.refs_result.as_deref().unwrap_or(&[]);
    let Ok(graph) = cx.log_result.as_ref() else {
        return false;
    };
    if crate::log_menu::loaded_commit(&*cx.log_result, commit).is_none()
        || crate::reword_ui::on_current_branch(graph, refs, commit) != Some(false)
    {
        return false;
    }
    let branches = crate::reword_ui::other_branches_holding(graph, refs, commit);
    let dialog = GuideDialog::off_branch(
        commit,
        scope,
        &branches,
        crate::build_features::HISTORY_REWRITE,
    );
    *cx.input_mode = InputMode::Guide(Box::new(dialog));
    true
}

/// What the off-branch edit dialog's buttons do.
fn apply_off_branch(cx: &mut CommandCtx<'_>, off: crate::guide_dialog::OffBranch, choice: Choice) {
    match choice {
        // The usual checkout (the daemon's own dirty-worktree handling); the
        // edit opens once the branch is the checked-out one.
        Choice::Pick(i) => {
            let Some(branch) = off.branches.get(i as usize).cloned() else {
                return;
            };
            let message = crate::git_actions::checkout_ref_now(
                &branch,
                &*cx.refs_result,
                &*cx.remote_names,
                cx.link,
                &mut *cx.input_mode,
            );
            crate::off_branch_edit::wait_for(off.commit, off.scope, branch);
            if !message.is_empty() {
                *cx.last_message = Some(message);
            }
        }
        Choice::Rewrite => {
            super::super::keys_prefix::open_rewrite_from_menu(
                cx,
                crate::log_menu::RewriteFrom::Selected,
                off.commit,
            );
        }
        Choice::CreateBranch => {
            *cx.input_mode = InputMode::InlineName {
                op: RefNameOp::NewBranchAt(off.commit),
                value: String::new(),
            };
        }
        _ => {}
    }
}
