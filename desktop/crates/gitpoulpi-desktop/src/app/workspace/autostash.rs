//! The auto-stash question, shared by every operation the daemon refuses because local changes are in the
//! way (a ref's checkout, merge or rebase, a commit's detached checkout or cherry-pick, a pull): the link
//! raises one event, the window asks once here.

use gitpoulpi_i18n::msg;
use gitpoulpi_link::Action;
use gpui_kit::Context;

use super::Workspace;
use crate::app::actions::with_workspace;
use crate::dialogs::{self, Choice, Spec};

impl Workspace {
    /// Asks the auto-stash question for each checkout, merge, rebase or pull the daemon refused because
    /// local changes are in the way: stash them and continue (the stash is popped again afterwards), or
    /// cancel. Nothing is highlighted until a key is pressed.
    pub(in crate::app) fn ask_autostash(
        &mut self,
        needed: Vec<gitpoulpi_link::AutoStashNeeded>,
        cx: &mut Context<Self>,
    ) {
        for n in needed {
            with_workspace(cx, move |ws, window, cx| {
                use msg::remotes as m;
                let gitpoulpi_link::AutoStashNeeded {
                    operation,
                    action,
                    ticket,
                } = n;
                ws.say(m::autostash_message(&operation), cx);
                let spec = Spec {
                    title: m::autostash_title(&operation),
                    lines: vec![m::autostash_message(&operation)],
                    choices: vec![
                        Choice::new(m::AUTOSTASH_CONTINUE.get()),
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
                            with_workspace(cx, move |ws, _, cx| {
                                let again = Action::AutoStashed(Box::new(action));
                                ws.backend.with_link(|link| link.submit_as(ticket, again));
                                ws.say(m::STASHING_CONTINUING.get(), cx);
                            });
                        } else {
                            gitpoulpi_link::report_ticket(
                                ticket,
                                gitpoulpi_link::StepOutcome::Canceled,
                            );
                        }
                    },
                    move || {
                        gitpoulpi_link::report_ticket(ticket, gitpoulpi_link::StepOutcome::Canceled)
                    },
                );
            });
        }
    }
}
