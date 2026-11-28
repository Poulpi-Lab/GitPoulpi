//! The questions the link raises about undo and pushes, answered in dialogs:
//!
//! - a push the remote refused because it moved on (`LinkEvent::PushRejected`): pull first then push,
//!   force push with lease (always confirmed, naming the branch and the remote), or cancel;
//! - an undo or redo the daemon found risky (`LinkEvent::UndoConfirm`): "Undo anyway" or cancel;
//! - forgetting the blocked newest undo entry (`Edit: Forget Blocked Undo Entry`).
//!
//! No answer is highlighted until a key is pressed, so a stray Enter answers nothing. A ticket held for a
//! tracked push ends as cancelled when the question is dismissed.

use gitpoulpi_core::domain::{Direction, PushRejectReason};
use gitpoulpi_i18n::msg;
use gitpoulpi_link::undo_ui::{top_id, PendingConfirm};
use gitpoulpi_link::{cancel_ticket, Action, PushRejectedNotice, Ticket};
use gpui_kit::{App, Window};

use crate::daemon::Shared;
use crate::dialogs::{self, Choice, Spec};

fn spec(title: String, lines: Vec<String>, choices: Vec<Choice>) -> Spec {
    Spec {
        title,
        lines,
        choices,
        hint: msg::diff_actions::CONFIRM_HINT.get().to_string(),
        highlight_first: false,
        yes_no_keys: false,
    }
}

/// Asks what to do about a push the remote rejected. Only a diverged branch has a question (the lease
/// refusal and the hosts' own refusals are the status line's words); returns whether one was asked.
pub fn ask_rejected(
    notice: PushRejectedNotice,
    backend: Shared,
    window: &mut Window,
    cx: &mut App,
) -> bool {
    use msg::remotes as m;
    let PushRejectedNotice { rejection, ticket } = notice;
    if rejection.reason != PushRejectReason::NonFastForward {
        if let Some(ticket) = ticket {
            cancel_ticket(ticket);
        }
        return false;
    }
    let (remote, branch) = (rejection.remote.clone(), rejection.branch.clone());
    let question = spec(
        m::diverged_title(&remote, &branch),
        vec![m::diverged_message(&remote, &branch)],
        vec![
            Choice::new(m::PULL_THEN_PUSH.get()),
            Choice::destructive(m::FORCE_PUSH_LEASE.get()),
            Choice::new(msg::dialog::CANCEL.get()),
        ],
    );
    let (b1, r1, b2, r2) = (backend.clone(), remote.clone(), backend, remote);
    let (branch1, branch2) = (branch.clone(), branch);
    dialogs::open(
        window,
        cx,
        question,
        move |ix, cx| match ix {
            0 => submit_as(
                &b1,
                ticket,
                Action::PullThenPush {
                    remote: r1,
                    branch: branch1,
                },
            ),
            1 => confirm_force(b2, r2, branch2, ticket, cx),
            _ => release(ticket),
        },
        move || release(ticket),
    );
    true
}

fn submit_as(backend: &Shared, ticket: Option<Ticket>, action: Action) {
    backend.with_link(|link| link.submit_as(ticket, action));
}

fn release(ticket: Option<Ticket>) {
    if let Some(ticket) = ticket {
        cancel_ticket(ticket);
    }
}

/// The force push's own confirmation: it replaces the remote's branch for everyone who uses it.
fn confirm_force(
    backend: Shared,
    remote: String,
    branch: String,
    ticket: Option<Ticket>,
    cx: &mut App,
) {
    let Some(win) = cx.windows().first().copied() else {
        release(ticket);
        return;
    };
    let question = spec(
        msg::remotes::force_confirm_fetch(&branch, &remote),
        Vec::new(),
        vec![
            Choice::destructive(msg::remotes::FORCE_PUSH_LEASE.get()),
            Choice::new(msg::dialog::CANCEL.get()),
        ],
    );
    // After the question that raised this one is gone: the window is not updatable inside its answer.
    cx.defer(move |cx| {
        let _ = win.update(cx, move |_, window, cx| {
            dialogs::open(
                window,
                cx,
                question,
                move |ix, _| {
                    if ix == 0 {
                        submit_as(
                            &backend,
                            ticket,
                            Action::FetchThenForcePush { remote, branch },
                        );
                    } else {
                        release(ticket);
                    }
                },
                move || release(ticket),
            );
        });
    });
}

/// Asks to go ahead with an undo or redo the daemon flagged as risky.
pub fn ask_undo_confirm(
    confirm: PendingConfirm,
    backend: Shared,
    window: &mut Window,
    cx: &mut App,
) {
    let button = match confirm.direction {
        Direction::Undo => msg::confirm_button::UNDO_ANYWAY,
        Direction::Redo => msg::confirm_button::REDO_ANYWAY,
    };
    let question = spec(
        confirm.prompt.clone(),
        Vec::new(),
        vec![
            Choice::destructive(button.get()),
            Choice::new(msg::dialog::CANCEL.get()),
        ],
    );
    dialogs::open(
        window,
        cx,
        question,
        move |ix, _| {
            if ix != 0 {
                return;
            }
            backend.with_link(|link| match confirm.sequence {
                Some(group) => link.send_undo_sequence(group, true),
                None => link.send_undo_redo(confirm.direction, confirm.expect, true),
            });
        },
        || {},
    );
}

/// Asks before dropping the blocked newest undo entry (the repository is not touched).
/// Returns the sentence of the refusal when there is nothing to forget.
pub fn ask_forget(backend: Shared, window: &mut Window, cx: &mut App) -> Result<(), String> {
    let state = backend.with_link(|l| l.journal_state());
    let Some(entry) = state.undo.as_ref() else {
        return Err(gitpoulpi_link::undo_ui::nothing_to_undo().to_string());
    };
    let prompt = msg::undo_journal::forget_confirm(gitpoulpi_link::undo_label::summary_text(entry));
    let expect = top_id(&state, Direction::Undo);
    let question = spec(
        prompt,
        Vec::new(),
        vec![
            Choice::destructive(msg::undo_journal::FORGET_BUTTON.get()),
            Choice::new(msg::dialog::CANCEL.get()),
        ],
    );
    dialogs::open(
        window,
        cx,
        question,
        move |ix, _| {
            if ix == 0 {
                backend.submit(Action::UndoForget { expect });
            }
        },
        || {},
    );
    Ok(())
}
