//! What the window does about staging: carrying out a hunk or line request from the diff pane (asking
//! first when it discards), keeping the open working-tree diff in step with the index, and asking the
//! sensitive-files question when the daemon refused a stage.

use std::sync::Arc;

use gitpoulpi_i18n::msg;
use gpui_kit::{Context, Window};

use super::{Center, Workspace};
use crate::daemon::host::SensitivePrompt;
use crate::daemon::staging::{self, PartOp};
use crate::dialogs::{self, Choice, Spec};
use crate::panels::diff::{DiffSource, PartRequest};

impl Workspace {
    /// Stages or unstages a hunk or lines at once; a discard asks first.
    pub(in crate::app) fn on_part(
        &mut self,
        req: PartRequest,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let action = staging::part_action(req.path.clone(), req.hunk, req.lines.clone(), req.op);
        if req.op != PartOp::Discard {
            self.backend.submit(action);
            return;
        }
        use msg::diff_actions as m;
        let shown = req.path.display();
        let prompt = match &req.lines {
            None => m::discard_hunk_prompt(req.hunk.0, &shown),
            Some(lines) => {
                let spill = if req.spilled { m::SPILL.get() } else { "" };
                m::discard_lines_prompt(lines.len() as u64, req.hunk.0, &shown, spill)
            }
        };
        let backend = self.backend.clone();
        dialogs::open(
            window,
            cx,
            Spec {
                title: prompt,
                lines: Vec::new(),
                choices: vec![
                    Choice::destructive(msg::discard_changes::DISCARD.get()),
                    Choice::new(msg::dialog::CANCEL.get()),
                ],
                hint: m::CONFIRM_HINT.get().to_string(),
                highlight_first: true,
                yes_no_keys: true,
            },
            move |ix, _| {
                if ix == 0 {
                    backend.submit(action);
                }
            },
            || {},
        );
    }

    /// The working-tree diff in the centre follows the index: a staged hunk leaves the unstaged half and
    /// appears in the staged one. A file with nothing left closes the diff.
    pub(in crate::app) fn refresh_working_diff(&mut self, cx: &mut Context<Self>) {
        let source = self.diff.read(cx).source();
        if self.center != Center::Diff || !source.is_working() {
            return;
        }
        let fut = self.backend.working_diff(source == DiffSource::Staged);
        cx.spawn(async move |this, cx| {
            let files = fut.await.unwrap_or_default();
            let _ = this.update(cx, |this, cx| {
                this.diff
                    .update(cx, |d, cx| d.refresh_files(Arc::new(files), cx))
            });
        })
        .detach();
    }

    /// Asks the sensitive-files question for each stage the daemon refused.
    pub(in crate::app) fn ask_sensitive(
        &mut self,
        prompts: Vec<SensitivePrompt>,
        cx: &mut Context<Self>,
    ) {
        cx.defer(move |cx| {
            let Some(window) = cx.windows().first().copied() else {
                return;
            };
            let _ = window.update(cx, move |_, window, cx| {
                for prompt in prompts {
                    crate::sensitive::ask(prompt, window, cx);
                }
            });
        });
    }
}
