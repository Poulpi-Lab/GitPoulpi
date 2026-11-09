//! Opening and driving the merge conflict editor from the window: the
//! palette's entries, Enter / double-click / the menu on a `!` row, the
//! operation banner, and what the editor's own keys and clicks ask of the
//! daemon link (`merge_flow.rs` holds the logic, `merge_view` the editor).

use crate::event_loop::*;
use crate::merge_flow::Env;
use crate::merge_view::{MergeCmd, Outcome};

use super::CommandCtx;

pub(super) fn run(cx: &mut CommandCtx<'_>, cmd: PaletteCommand) -> bool {
    match cmd {
        PaletteCommand::OpenMergeEditor => open_selected(cx),
        PaletteCommand::Merge(c) => run_merge(cx, c),
        PaletteCommand::AbortOperation => abort_operation(cx),
        PaletteCommand::SkipOperation => skip_operation(cx),
        _ => return false,
    }
    true
}

macro_rules! env {
    ($cx:expr) => {
        Env {
            link: $cx.link,
            conflicts: &*$cx.conflicts,
            merge: &mut *$cx.merge,
            root: &$cx.repo.root,
            last_message: &mut *$cx.last_message,
        }
    };
}

/// Opens the editor on `path`.
pub(in crate::event_loop) fn open_path(cx: &mut CommandCtx<'_>, path: PathBuf) {
    let mut env = env!(cx);
    cx.merge_flow.open_path(&mut env, path);
}

/// Opens the editor on the Status pane's selected conflicted file, else on
/// the Diff view's file when it is conflicted, else on the first one.
pub(in crate::event_loop) fn open_selected(cx: &mut CommandCtx<'_>) {
    let selected = selected_conflicted_path(
        &*cx.status_result,
        &*cx.status_states,
        *cx.status_list_mode,
        &*cx.collapsed_status_folders,
        &*cx.conflicts,
    );
    let in_diff = cx
        .diff_view_state
        .as_ref()
        .filter(|_| *cx.central_view == CentralView::Diff)
        .map(|d| d.path.clone())
        .filter(|p| cx.conflicts.iter().any(|e| &e.path == p));
    match selected.or(in_diff) {
        Some(path) => open_path(cx, path),
        None => {
            let mut env = env!(cx);
            cx.merge_flow.open_first(&mut env);
        }
    }
}

fn run_merge(cx: &mut CommandCtx<'_>, cmd: MergeCmd) {
    let Some(view) = cx.merge.as_mut() else {
        *cx.last_message = Some(
            gitpoulpi_i18n::msg::merge::REASON_NO_EDITOR
                .get()
                .to_string(),
        );
        return;
    };
    let out = view.run(cmd);
    apply_merge_outcome(cx, out);
}

/// Shows or hides the Refs and Status panels around the open editor (the
/// editor hides both on open, for the full width). Showing them is the
/// user's own state, so it is saved like any other pane toggle.
fn toggle_side_panels(cx: &mut CommandCtx<'_>) {
    let hide = !*cx.left_pane_collapsed && !*cx.right_pane_collapsed;
    cx.left_auto_collapse.peek(!hide, cx.left_pane_collapsed);
    cx.right_auto_collapse.peek(!hide, cx.right_pane_collapsed);
    if hide && matches!(*cx.focus, Focus::Refs | Focus::Status) {
        *cx.focus = Focus::Log;
    }
    persist_pane_layout(
        cx.session,
        *cx.divider1_pct,
        *cx.divider2_pct,
        cx.left_auto_collapse.persisted(*cx.left_pane_collapsed),
        cx.right_auto_collapse.persisted(*cx.right_pane_collapsed),
    );
}

/// Runs what a key, a click or a command asked of the window.
pub(in crate::event_loop) fn apply_merge_outcome(cx: &mut CommandCtx<'_>, out: Outcome) {
    if out.effect == Some(crate::merge_view::Effect::TogglePanes) {
        toggle_side_panels(cx);
        return;
    }
    if let Some(crate::merge_view::Effect::AiSuggest { conflict }) = out.effect {
        start_ai_suggest(cx, conflict);
        return;
    }
    let mut env = env!(cx);
    cx.merge_flow.apply(&mut env, out);
}

/// ✦ Starts the AI on conflict `conflict` (0-based) of the open file. The
/// answer is put into the Output by the window's tick, never saved.
fn start_ai_suggest(cx: &mut CommandCtx<'_>, conflict: usize) {
    use gitpoulpi_i18n::msg::merge as tr;
    let Some(view) = cx.merge.as_ref() else {
        return;
    };
    let Some(context) = crate::ai_ui::ConflictContext::from_document(view.model.merge(), conflict)
    else {
        *cx.last_message = Some(tr::MSG_AI_NO_TEXT.get().to_string());
        return;
    };
    let request = crate::ai_ui::AiRequest::ResolveConflict {
        path: context.path.clone(),
        conflict,
        context: Box::new(context),
        oversize_ok: false,
    };
    super::start_ai(cx, request);
}

/// The merge / cherry-pick / revert Abort: asks first (it throws the
/// operation's work away).
fn abort_operation(cx: &mut CommandCtx<'_>) {
    use gitpoulpi_core::domain::OperationState as O;
    use gitpoulpi_i18n::msg::merge as tr;
    let prompt = match cx.operation_state {
        Some(O::Merge { .. }) => tr::ABORT_CONFIRM_MERGE.get(),
        Some(O::CherryPick { .. }) => tr::ABORT_CONFIRM_PICK.get(),
        Some(O::Revert { .. }) => tr::ABORT_CONFIRM_REVERT.get(),
        // An `am` has its own, unjournaled abort.
        Some(O::Am { .. }) => {
            *cx.input_mode = InputMode::Confirm {
                prompt: gitpoulpi_i18n::msg::stuck_states::AM_ABORT_CONFIRM
                    .get()
                    .to_string(),
                action: ConfirmAction::Destructive(PendingDestructive::AbortAm),
            };
            return;
        }
        _ => {
            *cx.last_message = Some(tr::MSG_NO_OPERATION.get().to_string());
            return;
        }
    };
    *cx.input_mode = InputMode::Confirm {
        prompt: prompt.to_string(),
        action: ConfirmAction::Destructive(PendingDestructive::AbortOperation),
    };
}

/// The cherry-pick / revert Skip: asks first (the pick's resolutions are
/// dropped and nothing is recorded for it). A merge has nothing to skip.
fn skip_operation(cx: &mut CommandCtx<'_>) {
    use gitpoulpi_core::domain::OperationState as O;
    use gitpoulpi_i18n::msg::merge as tr;
    let prompt = match cx.operation_state {
        Some(O::CherryPick { .. }) => tr::SKIP_CONFIRM_PICK.get(),
        Some(O::Revert { .. }) => tr::SKIP_CONFIRM_REVERT.get(),
        Some(_) => {
            *cx.last_message = Some(tr::REASON_NOT_SKIPPABLE.get().to_string());
            return;
        }
        None => {
            *cx.last_message = Some(tr::MSG_NO_OPERATION.get().to_string());
            return;
        }
    };
    *cx.input_mode = InputMode::Confirm {
        prompt: prompt.to_string(),
        action: ConfirmAction::Destructive(PendingDestructive::SkipOperation),
    };
}
