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
        // Settings, on the limit's row.
        (Kind::TooLarge(_), Choice::Secondary) => {
            let mut state = PreferencesState::open(cx.repo_root);
            let _ = state.show_page(None, Some(CONFLICT_LIMIT_KEY));
            *cx.preferences = Some(state);
        }
    }
}
