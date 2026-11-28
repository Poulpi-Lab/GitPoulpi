//! The commit form under the working changes: summary and description, the length guide, Amend and Skip
//! hooks, the author, and the commit button.
//!
//! The rules are the terminal UI's (see `daemon::staging`): the button says why it cannot commit yet,
//! the summary counts down from 72, the repository's template pre-fills an empty form and its comment
//! lines are dropped from the message. The form clears only once the commit succeeded; a refusal (a hook,
//! nothing staged) keeps what was typed and shows the daemon's message in the status line.

use gitpoulpi_app::commit_form::{self, CommitPlan};
use gitpoulpi_app::palette::vocab::status_pane::StatusSection;
use gitpoulpi_i18n::msg;
use gpui_kit::component::{
    h_flex,
    input::{Input, Textarea},
    v_flex, Sizable as _,
};
use gpui_kit::{
    div, px, Context, Focusable as _, FontWeight, InteractiveElement as _, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, Window,
};

use super::RightPanel;
use crate::daemon::staging::{self, summary_remaining, SUMMARY_WARN_REMAINING};
use crate::theme::Skin;

impl RightPanel {
    // ---- state -------------------------------------------------------------------------------------

    /// Whether a text field of the pane (summary, description, filter) has the keyboard.
    pub fn field_focused(&self, window: &Window, cx: &gpui_kit::App) -> bool {
        [
            self.summary.read(cx).focus_handle(cx),
            self.description.read(cx).focus_handle(cx),
            self.filter.read(cx).focus_handle(cx),
        ]
        .iter()
        .any(|h| h.is_focused(window))
    }

    /// Whether the summary or the description has the keyboard (the key context `Composer`).
    pub fn composer_focused(&self, window: &Window, cx: &gpui_kit::App) -> bool {
        [
            self.summary.read(cx).focus_handle(cx),
            self.description.read(cx).focus_handle(cx),
        ]
        .iter()
        .any(|h| h.is_focused(window))
    }

    pub fn summary_text(&self, cx: &gpui_kit::App) -> String {
        self.summary.read(cx).value().to_string()
    }

    pub fn description_text(&self, cx: &gpui_kit::App) -> String {
        self.description.read(cx).value().to_string()
    }

    /// Types into the form (tests, and a template).
    pub fn set_message(
        &mut self,
        summary: &str,
        description: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.summary
            .update(cx, |s, cx| s.set_value(summary.to_string(), window, cx));
        self.description
            .update(cx, |s, cx| s.set_value(description.to_string(), window, cx));
        self.summary_len = summary.chars().count();
        cx.notify();
    }

    pub fn is_amending(&self) -> bool {
        self.amend
    }

    pub fn skips_hooks(&self) -> bool {
        self.skip_hooks
    }

    /// The identity the commit is made as, when the repository configures one.
    pub fn author(&self) -> Option<&str> {
        self.author.as_deref()
    }

    pub fn toggle_amend(&mut self, cx: &mut Context<Self>) {
        self.amend = !self.amend;
        cx.notify();
    }

    pub fn toggle_skip_hooks(&mut self, cx: &mut Context<Self>) {
        self.skip_hooks = !self.skip_hooks;
        cx.notify();
    }

    /// `Path` or `Tree` for the lists (`m`).
    pub fn toggle_tree(&mut self, cx: &mut Context<Self>) {
        self.tree = !self.tree;
        self.rebuild_rows(cx);
    }

    pub fn is_tree(&self) -> bool {
        self.tree
    }

    /// `c`: the keyboard goes to the summary.
    pub fn focus_summary(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.summary.read(cx).focus_handle(cx).focus(window, cx);
    }

    /// Whether a stopped merge is being committed (it needs nothing staged).
    pub fn set_merging(&mut self, merging: bool, cx: &mut Context<Self>) {
        if self.merging != merging {
            self.merging = merging;
            cx.notify();
        }
    }

    /// Whether the form is ready to commit, the shared rule (`gitpoulpi_app::commit_form`).
    fn readiness(&self, cx: &gpui_kit::App) -> Result<(), gitpoulpi_i18n::M> {
        commit_form::commit_readiness(
            self.summary.read(cx).value().as_ref(),
            self.amend,
            self.section_len(StatusSection::Staged),
            self.merging,
            self.conflict_count(),
        )
    }

    /// Why the button cannot commit yet (`None`: it can), the terminal UI's order.
    pub fn commit_blocker(&self, cx: &gpui_kit::App) -> Option<&'static str> {
        self.readiness(cx).err().map(|m| m.get())
    }

    // ---- the repository's settings -----------------------------------------------------------------

    /// Reads the settings the form depends on (template, comment character, identity) and pre-fills an
    /// empty form with the template when the repository asks for it.
    pub(super) fn load_commit_context(&mut self, cx: &mut Context<Self>) {
        let fut = self.backend.repo_config();
        let root = self.backend.repo_root();
        cx.spawn(async move |this, cx| {
            let config = fut.await.unwrap_or_default();
            let _ = this.update(cx, |this, cx| {
                this.settings = staging::commit_settings(&root, &config);
                this.author = staging::author_line(&config);
                if !this.template_applied {
                    this.skip_hooks = this.settings.skip_hooks_default;
                }
                this.apply_template(cx);
                cx.notify();
            });
        })
        .detach();
    }

    fn apply_template(&mut self, cx: &mut Context<Self>) {
        let empty = self.summary.read(cx).value().is_empty() && !self.has_description(cx);
        if !self.settings.template_apply || self.template_applied || !empty {
            return;
        }
        let (summary, description) = self.settings.template_fields();
        if summary.is_empty() && description.is_empty() {
            return;
        }
        self.template_applied = true;
        self.with_window(cx, move |this, window, cx| {
            this.set_message(&summary, &description, window, cx)
        });
    }

    fn has_description(&self, cx: &gpui_kit::App) -> bool {
        !self.description.read(cx).value().is_empty()
    }

    /// Runs `f` with the window once the current update is over (a link wake has no window of its own).
    fn with_window(
        &mut self,
        cx: &mut Context<Self>,
        f: impl FnOnce(&mut Self, &mut Window, &mut Context<Self>) + 'static,
    ) {
        let me = cx.weak_entity();
        cx.defer(move |cx| {
            let Some(window) = cx.windows().first().copied() else {
                return;
            };
            let _ = window.update(cx, move |_, window, cx| {
                let _ = me.update(cx, |this, cx| f(this, window, cx));
            });
        });
    }

    // ---- committing --------------------------------------------------------------------------------

    /// The commit button, `Ctrl+Enter` in a field: commits the staged changes with the form's message.
    /// When it cannot, the status line (and the button) says why; nothing is sent.
    pub fn commit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.commit_ticket.is_some() {
            return;
        }
        if let Some(why) = self.commit_blocker(cx) {
            if self.summary.read(cx).value().trim().is_empty() {
                self.focus_summary(window, cx);
            }
            return self.say(why, cx);
        }
        let summary = self.summary_text(cx);
        let description = self.description_text(cx);
        let strip = self
            .settings
            .strip_comments
            .then_some(self.settings.comment_char);
        let (message, sent) =
            match commit_form::plan_commit(&summary, &description, self.amend, strip) {
                CommitPlan::Rejected(error) => return self.say(error, cx),
                CommitPlan::Commit { message, status } => (message, status),
            };
        let action = staging::commit_action(message, self.amend, self.skip_hooks, &self.settings);
        self.commit_ticket = Some(self.backend.submit_tracked(action));
        self.say(sent, cx);
        cx.notify();
    }

    /// `Ctrl+G Enter` and `Ctrl+G c`: commit when the form is ready, else take the keyboard to the
    /// summary and say what is missing (the terminal UI's behaviour).
    pub fn commit_now(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.shows_working_changes() {
            return self.say(msg::input::SELECT_FILE_STATUS_OR_COMMIT.get(), cx);
        }
        match self.commit_blocker(cx) {
            None => self.commit(window, cx),
            Some(why) => {
                self.focus_summary(window, cx);
                self.say(why, cx);
            }
        }
    }

    /// Reads what became of the commit that was sent: on success the form clears (and the Log, which the
    /// daemon's refresh reloads, shows the new commit); on a failure the text stays for another try.
    pub fn settle(&mut self, cx: &mut Context<Self>) {
        use gitpoulpi_link::StepOutcome;
        let Some(ticket) = self.commit_ticket else {
            return;
        };
        for outcome in gitpoulpi_link::take_for(ticket) {
            match outcome {
                StepOutcome::Ok(_) => {
                    self.commit_ticket = None;
                    self.template_applied = false;
                    self.with_window(cx, |this, window, cx| {
                        this.set_message("", "", window, cx);
                        this.amend = false;
                        this.skip_hooks = this.settings.skip_hooks_default;
                        cx.notify();
                    });
                }
                StepOutcome::Failed { .. } | StepOutcome::Canceled => self.commit_ticket = None,
                StepOutcome::NeedsUser(_) => {}
            }
        }
        cx.notify();
    }

    /// Whether a commit was sent and has no answer yet.
    pub fn commit_pending(&self) -> bool {
        self.commit_ticket.is_some()
    }

    // ---- the view ----------------------------------------------------------------------------------

    pub(super) fn composer_view(&self, skin: &Skin, cx: &mut Context<Self>) -> gpui_kit::Div {
        let staged = self.section_len(StatusSection::Staged);
        let blocker = self.commit_blocker(cx);
        let ready = blocker.is_none() && self.commit_ticket.is_none();
        let label =
            commit_form::commit_button_label(self.amend, staged, self.merging, self.readiness(cx));
        let remaining = summary_remaining(self.summary_len);
        let counter_color = if remaining < 0 {
            skin.danger
        } else if remaining <= SUMMARY_WARN_REMAINING {
            skin.warning
        } else {
            skin.muted
        };
        let check = |id: &'static str, on: bool, text: &str| {
            h_flex()
                .id(id)
                .gap(px(6.0))
                .text_sm()
                .cursor_pointer()
                .child(if on { "[x]" } else { "[ ]" })
                .child(text.to_string())
        };
        use gpui_kit::prelude::FluentBuilder as _;
        v_flex()
            .flex_none()
            .gap(px(6.0))
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(
                        v_flex()
                            .gap(px(2.0))
                            .child(
                                check("amend", self.amend, msg::composer::AMEND.get())
                                    .on_click(cx.listener(|this, _, _, cx| this.toggle_amend(cx))),
                            )
                            .child(
                                check("hooks", self.skip_hooks, msg::composer::SKIP_HOOKS.get())
                                    .on_click(
                                        cx.listener(|this, _, _, cx| this.toggle_skip_hooks(cx)),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .id("summary-counter")
                            .text_xs()
                            .text_color(counter_color)
                            .child(format!("#\u{25BE} {remaining}")),
                    ),
            )
            .child(
                v_flex()
                    .border_1()
                    .border_color(skin.border)
                    .rounded(px(3.0))
                    .p(px(4.0))
                    .gap(px(4.0))
                    .child(Input::new(&self.summary).small())
                    .child(Textarea::new(&self.description)),
            )
            .when_some(self.author.clone(), |d, author| {
                d.child(
                    div()
                        .id("author")
                        .text_xs()
                        .text_color(skin.muted)
                        .truncate()
                        .child(author),
                )
            })
            .child(
                div()
                    .id("commit")
                    .h(px(28.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(3.0))
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .bg(if ready { skin.accent } else { skin.input_bg })
                    .text_color(if ready { skin.on_button } else { skin.muted })
                    .when(ready, |d| d.cursor_pointer())
                    .child(label)
                    .on_click(cx.listener(|this, _, window, cx| this.commit(window, cx))),
            )
    }
}
