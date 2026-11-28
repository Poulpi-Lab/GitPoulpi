//! What the window does about the Log's commit operations: running one (the commands of
//! `commands::commit_ops`), the comparison the right pane shows for several selected rows, and the
//! banner of a merge, cherry-pick or revert stopped on conflicts (Resolve / Continue / Skip / Abort).
//!
//! The daemon's operation state arrives with every refresh; the banner and the status line are a
//! function of it and of the conflicted files, as in the terminal UI's `operation_banner.rs`.

use gitpoulpi_core::domain::OperationState;
use gitpoulpi_i18n::msg;
use gitpoulpi_i18n::msg::merge as tr;
use gpui_kit::component::h_flex;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    div, px, App, Context, Focusable as _, InteractiveElement as _, IntoElement as _,
    ParentElement as _, StatefulInteractiveElement as _, Styled as _, Window,
};

use super::Workspace;
use crate::commands::commit_ops::CommitOp;
use crate::daemon::commit_ops::{abort_action, short, skip_action};
use crate::dialogs::{self, Choice, Spec};
use crate::strings;
use crate::theme::Skin;

/// The banner's buttons, in the order they are drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BannerButton {
    /// The merge editor: not available yet.
    Resolve,
    /// Commit (or continue) once nothing is left to resolve: the commit form takes it from here.
    Continue,
    Skip,
    Abort,
}

/// Whether a banner is shown for `state` (a `git am` keeps its own strip).
pub fn has_banner(state: &OperationState) -> bool {
    !matches!(state, OperationState::Am { .. })
}

/// The rebase strip's words, as the terminal UI draws them.
fn rebase_text(state: &gitpoulpi_core::domain::RebaseState) -> String {
    use msg::rebase_banner as rb;
    let head = state.head_name.as_deref().unwrap_or("HEAD");
    let onto = short(state.onto);
    match state.stopped_at {
        Some(at) => rb::stopped(
            head,
            onto,
            state.step,
            state.total,
            short(at),
            if state.conflicted {
                rb::KIND_CONFLICT
            } else {
                rb::KIND_EDIT
            },
        ),
        None => rb::running(head, onto, state.step, state.total),
    }
}

/// The banner's words: the operation and what is left to resolve.
pub fn banner_text(state: &OperationState, conflicts: usize) -> String {
    let head = match state {
        OperationState::Merge {
            head_name: Some(head),
            their_label,
        } => tr::banner_merge(their_label, head),
        OperationState::Merge { their_label, .. } => tr::banner_merge_detached(their_label),
        OperationState::CherryPick { commit } => tr::banner_pick(short(*commit)),
        OperationState::Revert { commit } => tr::banner_revert(short(*commit)),
        OperationState::Rebase(rebase) => rebase_text(rebase),
        OperationState::Am { .. } => String::new(),
    };
    let tail = if conflicts == 0 {
        tr::BANNER_ALL_RESOLVED.get().to_string()
    } else {
        tr::banner_files(conflicts as u64)
    };
    format!("{head} \u{b7} {tail}")
}

/// The buttons that make sense now: Resolve while files are conflicted, Continue once none is, Skip for
/// a cherry-pick or revert, and always Abort.
pub fn banner_buttons(state: &OperationState, conflicts: usize) -> Vec<(BannerButton, String)> {
    let mut out = Vec::new();
    if conflicts > 0 {
        out.push((BannerButton::Resolve, tr::BANNER_RESOLVE.get().to_string()));
    } else {
        out.push((
            BannerButton::Continue,
            match state {
                OperationState::Merge { .. } => tr::BANNER_COMMIT_MERGE.get(),
                _ => tr::BANNER_CONTINUE.get(),
            }
            .to_string(),
        ));
    }
    if matches!(
        state,
        OperationState::CherryPick { .. }
            | OperationState::Revert { .. }
            | OperationState::Rebase(_)
    ) {
        out.push((BannerButton::Skip, tr::BANNER_SKIP.get().to_string()));
    }
    out.push((
        BannerButton::Abort,
        match state {
            OperationState::Merge { .. } => tr::BANNER_ABORT_MERGE.get(),
            _ => tr::BANNER_ABORT.get(),
        }
        .to_string(),
    ));
    out
}

impl Workspace {
    /// The merge, cherry-pick or revert stopped in the repository, as the last refresh saw it.
    pub fn operation(&self) -> Option<&OperationState> {
        self.operation.as_ref()
    }

    /// The status line's words for it.
    pub fn operation_status(&self, cx: &App) -> Option<String> {
        let state = self.operation.as_ref().filter(|s| has_banner(s))?;
        Some(banner_text(state, self.right.read(cx).conflict_count()))
    }

    /// The labels of the banner's buttons, in order (none while no operation is stopped).
    pub fn banner_button_labels(&self, cx: &App) -> Vec<String> {
        match self.operation.as_ref().filter(|s| has_banner(s)) {
            Some(state) => banner_buttons(state, self.right.read(cx).conflict_count())
                .into_iter()
                .map(|(_, label)| label)
                .collect(),
            None => Vec::new(),
        }
    }

    /// Does what a Log command says.
    pub fn run_commit_op(&mut self, op: CommitOp, window: &mut Window, cx: &mut Context<Self>) {
        match op {
            CommitOp::CherryPick => self.log.update(cx, |l, cx| l.cherry_pick(cx)),
            CommitOp::Revert => self.log.update(cx, |l, cx| l.revert(window, cx)),
            CommitOp::Reset(mode) => self.log.update(cx, |l, cx| l.reset(mode, window, cx)),
            CommitOp::Checkout => self.log.update(cx, |l, cx| l.checkout_commit(cx)),
            CommitOp::Copy(field) => self.log.update(cx, |l, cx| l.copy_field(field, cx)),
            CommitOp::OpenMenu => self.open_context_menu(window, cx),
            CommitOp::CompareSwap => self.log.update(cx, |l, cx| l.swap_compare(cx)),
            CommitOp::CompareNext => self.log.update(cx, |l, cx| l.next_compared(cx)),
            CommitOp::CompareSelection => {
                if self.log.read(cx).marked_count() < 2 {
                    return self.say(msg::compare::SELECT_MORE.get(), cx);
                }
                self.right.read(cx).focus_handle(cx).focus(window, cx);
                self.say(msg::compare::COMPARING.get(), cx);
            }
            CommitOp::Abort => self.ask_operation(true, window, cx),
            CommitOp::Skip => self.ask_operation(false, window, cx),
        }
    }

    /// The menu key: the context menu of the pane that has the keyboard. In the Refs it is the ref menu
    /// of the selected row; in the Log, the ref menu when a ref pill of the selected commit was picked
    /// (a click on it), else the commit menu. This is the one place the key is dispatched.
    fn open_context_menu(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self
            .refs
            .read(cx)
            .focus_handle(cx)
            .contains_focused(window, cx)
        {
            return self.open_menu_for_selection(window, cx);
        }
        if self
            .log
            .read(cx)
            .focus_handle(cx)
            .contains_focused(window, cx)
        {
            if let Some(target) = self.log.read(cx).picked_pill() {
                let at = self.log.read(cx).selected_row_origin();
                return self.open_ref_menu(target, at, window, cx);
            }
            let palette = self.command_state(window, cx).palette_context();
            self.log
                .update(cx, |l, cx| l.open_menu(palette, window, cx));
        } else {
            let label = crate::commands::exec::label_of(crate::commands::Command::Action(
                gitpoulpi_app::keymap::Action::GlobalContextMenu,
            ));
            self.say(strings::not_available_yet(&label), cx);
        }
    }

    /// The banner's Abort or Skip: asks first, then runs as one action.
    fn ask_operation(&mut self, abort: bool, window: &mut Window, cx: &mut Context<Self>) {
        let Some(state) = self.operation.clone() else {
            return self.say(tr::MSG_NO_OPERATION.get(), cx);
        };
        let prompt = match (&state, abort) {
            (OperationState::Merge { .. }, true) => tr::ABORT_CONFIRM_MERGE.get(),
            (OperationState::CherryPick { .. }, true) => tr::ABORT_CONFIRM_PICK.get(),
            (OperationState::Revert { .. }, true) => tr::ABORT_CONFIRM_REVERT.get(),
            (OperationState::Rebase(_), true) => msg::input::ABORT_REBASE_CONFIRM.get(),
            (OperationState::CherryPick { .. }, false) => tr::SKIP_CONFIRM_PICK.get(),
            (OperationState::Revert { .. }, false) => tr::SKIP_CONFIRM_REVERT.get(),
            (OperationState::Rebase(_), false) => msg::rebase_banner::SKIP.get(),
            (_, false) => return self.say(tr::REASON_NOT_SKIPPABLE.get(), cx),
            // `git am` has its own abort.
            _ => return self.say(strings::not_available_yet(tr::BANNER_ABORT.get()), cx),
        };
        let rebase = matches!(state, OperationState::Rebase(_));
        let backend = self.backend.clone();
        let label = if abort {
            tr::BANNER_ABORT.get()
        } else {
            tr::BANNER_SKIP.get()
        };
        dialogs::open(
            window,
            cx,
            Spec {
                title: prompt.to_string(),
                lines: Vec::new(),
                choices: vec![
                    Choice::destructive(label.to_string()),
                    Choice::new(msg::dialog::CANCEL.get()),
                ],
                hint: msg::footer::CONFIRM_HINT.get().trim().to_string(),
                highlight_first: true,
                yes_no_keys: true,
            },
            move |ix, _| {
                if ix == 0 {
                    backend.submit(match (rebase, abort) {
                        (true, true) => gitpoulpi_link::Action::RebaseAbort,
                        (true, false) => gitpoulpi_link::Action::RebaseSkip,
                        (false, true) => abort_action(),
                        (false, false) => skip_action(),
                    });
                }
            },
            || {},
        );
    }

    /// A click on a banner button.
    pub fn banner_click(
        &mut self,
        button: BannerButton,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match button {
            BannerButton::Resolve => {
                self.say(strings::not_available_yet(tr::BANNER_RESOLVE.get()), cx)
            }
            BannerButton::Continue if matches!(self.operation, Some(OperationState::Rebase(_))) => {
                self.backend.submit(gitpoulpi_link::Action::RebaseContinue);
                self.say(msg::input::CONTINUING_REBASE.get(), cx);
            }
            BannerButton::Continue => {
                // Committing concludes a merge, a cherry-pick and a revert alike: the WIP row's commit
                // form takes it from here (git refuses an empty pick: say so, and point to Skip).
                let has_wip = self.summary.as_ref().is_some_and(|s| !s.status.is_empty());
                if !has_wip && !matches!(self.operation, Some(OperationState::Merge { .. })) {
                    return self.say(tr::MSG_EMPTY_PICK.get(), cx);
                }
                self.log.update(cx, |l, cx| l.select(0, cx));
                self.right.update(cx, |r, cx| r.focus_summary(window, cx));
            }
            BannerButton::Skip => self.ask_operation(false, window, cx),
            BannerButton::Abort => self.ask_operation(true, window, cx),
        }
    }

    /// The strip above the panes while an operation is stopped.
    pub(in crate::app) fn operation_banner(
        &self,
        cx: &mut Context<Self>,
    ) -> Option<gpui_kit::AnyElement> {
        let state = self.operation.as_ref().filter(|s| has_banner(s))?;
        let conflicts = self.right.read(cx).conflict_count();
        let skin = cx.global::<Skin>().clone();
        let buttons = banner_buttons(state, conflicts);
        Some(
            h_flex()
                .id("operation-banner")
                .flex_none()
                .h(px(30.0))
                .px(px(10.0))
                .gap(px(8.0))
                .items_center()
                .border_b_1()
                .border_color(skin.border)
                .text_sm()
                .text_color(skin.warning)
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .child(banner_text(state, conflicts)),
                )
                .children(buttons.into_iter().map(|(button, label)| {
                    let abort = button == BannerButton::Abort;
                    div()
                        .id(match button {
                            BannerButton::Resolve => "op-resolve",
                            BannerButton::Continue => "op-continue",
                            BannerButton::Skip => "op-skip",
                            BannerButton::Abort => "op-abort",
                        })
                        .flex_none()
                        .px(px(10.0))
                        .h(px(22.0))
                        .flex()
                        .items_center()
                        .rounded(px(3.0))
                        .cursor_pointer()
                        .bg(if abort { skin.danger } else { skin.accent })
                        .text_color(skin.on_button)
                        .text_xs()
                        .when(abort, |d| d.font_weight(gpui_kit::FontWeight::SEMIBOLD))
                        .child(label)
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.banner_click(button, window, cx)
                        }))
                }))
                .into_any_element(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::Oid;

    fn pick() -> OperationState {
        OperationState::CherryPick {
            commit: Oid([0xab; 20]),
        }
    }

    #[test]
    fn the_banner_names_the_operation_and_what_is_left() {
        let text = banner_text(&pick(), 2);
        assert!(text.contains("abababa") && text.contains('2'), "{text}");
        assert!(banner_text(&pick(), 0).contains(tr::BANNER_ALL_RESOLVED.get()));
    }

    #[test]
    fn the_buttons_follow_the_conflicts() {
        let kinds = |s: &OperationState, n| {
            banner_buttons(s, n)
                .into_iter()
                .map(|(b, _)| b)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            kinds(&pick(), 1),
            [
                BannerButton::Resolve,
                BannerButton::Skip,
                BannerButton::Abort
            ]
        );
        assert_eq!(
            kinds(&pick(), 0),
            [
                BannerButton::Continue,
                BannerButton::Skip,
                BannerButton::Abort
            ]
        );
        let merge = OperationState::Merge {
            head_name: None,
            their_label: "x".into(),
        };
        assert!(!kinds(&merge, 0).contains(&BannerButton::Skip));
    }
}
