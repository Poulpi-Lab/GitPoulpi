//! Hosting the prompt dialog (`prompt_dialog/`): the keys and the mouse while
//! `InputMode::Prompt` is open, and what to do with what the dialog answers.

use super::commands::{CommandCtx, Flow};
use super::KeyEnv;
use crate::event_loop::*;
use crate::prompt_dialog::{prompt_hit_at, PromptOutcome};
use crossterm::event::MouseEvent;

/// A key while the dialog is open. `Ctrl+N` badges its controls and the key
/// after it presses one, as in the reword popup.
pub(super) fn key(cx: &mut CommandCtx<'_>, env: &KeyEnv<'_>, key: KeyEvent) -> io::Result<()> {
    let InputMode::Prompt(dialog) = &mut *cx.input_mode else {
        return Ok(());
    };
    if key.kind != KeyEventKind::Press {
        return Ok(());
    }
    if dialog.badges {
        // The key after `Ctrl+N`: a badge presses its control, anything else
        // (Esc included) just takes the badges away.
        dialog.badges = false;
        if super::keys_modes::arrange_after_prefix(cx, key) {
            return Ok(());
        }
        if let Some((col, row)) = super::keys_modes::badge_cell(cx, key) {
            for kind in [
                MouseEventKind::Down(MouseButton::Left),
                MouseEventKind::Up(MouseButton::Left),
            ] {
                let click = MouseEvent {
                    kind,
                    column: col,
                    row,
                    modifiers: KeyModifiers::NONE,
                };
                mouse(cx, click)?;
            }
        }
        return Ok(());
    }
    if env.km.is(Bind::GlobalNavPrefix, &key) && dialog.badge_ready() {
        dialog.badges = true;
        return Ok(());
    }
    let outcome = dialog.on_key(key);
    settle(cx, outcome);
    Ok(())
}

/// The dialog is modal: a left press lands on it or is its own business (the
/// cross, a click outside), and every other mouse event is swallowed, so
/// nothing underneath moves while it is open.
pub(super) fn mouse(cx: &mut CommandCtx<'_>, mouse: MouseEvent) -> io::Result<Flow> {
    let hit = prompt_hit_at(&*cx.hit_targets, mouse.column, mouse.row);
    if matches!(mouse.kind, MouseEventKind::Moved) {
        *cx.hovered_target = hit.map(HitTarget::Prompt);
    }
    if let InputMode::Prompt(dialog) = &mut *cx.input_mode {
        let outcome = dialog.on_mouse(mouse, hit);
        settle(cx, outcome);
    }
    Ok(Flow::Handled)
}

/// Carries out what the dialog answered.
fn settle(cx: &mut CommandCtx<'_>, outcome: PromptOutcome) {
    match outcome {
        PromptOutcome::Stay => {}
        PromptOutcome::Close => {
            // Closing while the model still writes stops its request.
            if matches!(&*cx.input_mode, InputMode::Prompt(d) if d.ai_running) && cx.ai_ui.running()
            {
                cx.ai_ui.cancel();
            }
            *cx.input_mode = InputMode::Normal;
        }
        PromptOutcome::Submit(actions, said) => {
            for action in actions {
                cx.link.submit_action(action);
            }
            *cx.last_message = Some(said);
            *cx.input_mode = InputMode::Normal;
        }
        PromptOutcome::StartAi(request) => {
            // An explicit press of the ✦ button (or its key): the one place
            // the dialog starts the model.
            cx.ai_ui
                .start_into(cx.session, cx.repo, request, &mut *cx.last_message);
        }
    }
}
