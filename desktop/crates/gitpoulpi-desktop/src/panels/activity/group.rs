//! The grouped push: the progress of a run to several remotes (one row per remote with its status), fed by
//! the link's push-group events, and the per-run Retry. Closing the view never stops the run: the state
//! lives in the link host, so a run started from the terminal UI or the CLI shows up here too.
//!
//! The rejected-push dialog (pull first, force push with lease, cancel) is `resolve`.

use gitpoulpi_core::domain::{GroupRunStatus, Integration, TargetStatus};
use gitpoulpi_i18n::msg::push_group as m;
use gitpoulpi_link::Action;
use gpui_kit::component::{h_flex, v_flex, WindowExt as _};
use gpui_kit::{
    div, px, AnyElement, App, AppContext as _, Context, Entity, FocusHandle, Focusable, Global,
    InteractiveElement as _, IntoElement, KeyDownEvent, ParentElement as _, Render, SharedString,
    StatefulInteractiveElement as _, Styled as _, Task, WeakEntity, Window,
};

use super::widgets::chip;
use crate::commands::status;
use crate::daemon::activity::{GroupPhase, GroupRow, GroupRun};
use crate::daemon::Shared;
use crate::theme::Skin;

/// How a row's state reads: good, still going, bad or quiet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Good,
    Warn,
    Bad,
    Muted,
}

pub fn integrated_text(i: Integration) -> String {
    match i {
        Integration::FastForwarded => m::RUN_FAST_FORWARDED.get().to_string(),
        Integration::Merged => m::RUN_MERGED.get().to_string(),
        Integration::Rebased => m::RUN_REBASED.get().to_string(),
        Integration::UpToDate => m::RUN_UP_TO_DATE.get().to_string(),
    }
}

/// One row's words: (glyph, text, tone).
pub fn row_text(row: &GroupRow) -> (&'static str, String, Tone) {
    let (glyph, text, tone) = row_base(row);
    match row.integrated {
        Some(i) if tone == Tone::Good && row.status != Some(TargetStatus::UpToDate) => {
            (glyph, format!("{}, {text}", integrated_text(i)), tone)
        }
        _ => (glyph, text, tone),
    }
}

fn row_base(row: &GroupRow) -> (&'static str, String, Tone) {
    let why = row.detail.clone().unwrap_or_default();
    match &row.status {
        None | Some(TargetStatus::NotAttempted) => {
            ("\u{00B7}", m::RUN_WAITING.get().to_string(), Tone::Muted)
        }
        Some(TargetStatus::Integrating) => {
            ("\u{2026}", m::RUN_MERGING.get().to_string(), Tone::Warn)
        }
        Some(TargetStatus::Pushing) => ("\u{2026}", m::RUN_PUSHING.get().to_string(), Tone::Warn),
        Some(TargetStatus::Integrated) => {
            ("\u{2713}", m::RUN_INTEGRATED.get().to_string(), Tone::Good)
        }
        Some(TargetStatus::Pushed { created: true }) => {
            ("\u{2713}", m::RUN_CREATED.get().to_string(), Tone::Good)
        }
        Some(TargetStatus::Pushed { created: false }) => {
            ("\u{2713}", m::RUN_PUSHED.get().to_string(), Tone::Good)
        }
        Some(TargetStatus::UpToDate) => {
            ("\u{2713}", m::RUN_UP_TO_DATE.get().to_string(), Tone::Good)
        }
        Some(TargetStatus::Skipped) => ("\u{00B7}", m::RUN_SKIPPED.get().to_string(), Tone::Muted),
        Some(TargetStatus::Rejected { reason }) => (
            "\u{2717}",
            m::run_rejected(if why.is_empty() {
                format!("{reason:?}")
            } else {
                why
            }),
            Tone::Bad,
        ),
        Some(TargetStatus::Failed { kind }) => (
            "\u{2717}",
            m::run_failed(if why.is_empty() {
                format!("{kind:?}")
            } else {
                why
            }),
            Tone::Bad,
        ),
        Some(TargetStatus::RemoteMissing) => {
            ("\u{2717}", m::RUN_MISSING.get().to_string(), Tone::Bad)
        }
        Some(TargetStatus::Unknown) => (
            "?",
            if why.is_empty() {
                m::RUN_UNKNOWN.get().to_string()
            } else {
                format!("{} - {why}", m::RUN_UNKNOWN.get())
            },
            Tone::Warn,
        ),
    }
}

/// The line under the rows: going, done, or what is left to retry.
pub fn footer_text(run: &GroupRun) -> String {
    match &run.phase {
        GroupPhase::Going => m::RUN_GOING.get().to_string(),
        GroupPhase::Failed(why) => why.clone(),
        GroupPhase::Ended(GroupRunStatus::Done) => m::RUN_DONE.get().to_string(),
        GroupPhase::Ended(_) if run.has_retryable() => m::RUN_PROBLEMS.get().to_string(),
        GroupPhase::Ended(_) if !run.message.is_empty() => run.message.clone(),
        GroupPhase::Ended(_) => m::RUN_DONE.get().to_string(),
    }
}

/// Retries the targets of the last run that failed, were refused or never ran. `false` when there is
/// nothing to retry (still going, or everything went through).
pub fn retry(backend: &Shared) -> bool {
    let Some(run) = backend
        .host()
        .group_run()
        .filter(|r| !r.going() && r.has_retryable())
    else {
        return false;
    };
    backend.submit(Action::RemoteGroupRetry {
        expect_run_id: run.run_id.clone(),
        prefer_rebase: false,
    });
    true
}

struct OpenGroup(WeakEntity<GroupView>);
impl Global for OpenGroup {}

pub struct GroupView {
    backend: Shared,
    focus: FocusHandle,
    _watch: Option<Task<()>>,
}

impl GroupView {
    fn new(backend: Shared, cx: &mut Context<Self>) -> Self {
        let notifier = backend.notifier().clone();
        let mut epoch = notifier.epoch();
        let watch = cx.spawn(async move |this, cx| loop {
            notifier.changed(&mut epoch).await;
            if this.update(cx, |_, cx| cx.notify()).is_err() {
                break;
            }
        });
        Self {
            backend,
            focus: cx.focus_handle(),
            _watch: Some(watch),
        }
    }

    pub fn run(&self) -> Option<GroupRun> {
        self.backend.host().group_run()
    }

    /// Retry button and key: says in the status line when there is nothing to retry.
    pub fn retry(&mut self, cx: &mut Context<Self>) {
        if !retry(&self.backend) {
            status::set(cx, m::RUN_DONE.get());
        }
        cx.notify();
    }

    fn key(&mut self, ev: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let ks = &ev.keystroke;
        if ks.modifiers.control {
            if ks.key == "p" {
                window.defer(cx, crate::commands::palette::open);
            }
            return;
        }
        match ks.key.as_str() {
            "r" => self.retry(cx),
            "escape" => window.close_dialog(cx),
            _ => {}
        }
    }
}

impl Focusable for GroupView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for GroupView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let skin = cx.global::<Skin>().clone();
        let run = self.run();
        let title = run
            .as_ref()
            .map(|r| m::title_run(&r.branch))
            .unwrap_or_default();
        let can_retry = run
            .as_ref()
            .is_some_and(|r| !r.going() && r.has_retryable());
        let rows: Vec<AnyElement> = run
            .as_ref()
            .map(|r| {
                r.rows
                    .iter()
                    .enumerate()
                    .map(|(i, row)| {
                        let (glyph, text, tone) = row_text(row);
                        let color = match tone {
                            Tone::Good => skin.success,
                            Tone::Warn => skin.warning,
                            Tone::Bad => skin.danger,
                            Tone::Muted => skin.muted,
                        };
                        h_flex()
                            .id(("group-row", i))
                            .h(px(26.0))
                            .px_3()
                            .gap_3()
                            .items_center()
                            .text_sm()
                            .child(div().flex_none().w(px(16.0)).text_color(color).child(glyph))
                            .child(
                                div()
                                    .flex_none()
                                    .w(px(200.0))
                                    .truncate()
                                    .child(SharedString::from(row.remote.clone())),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .truncate()
                                    .text_color(color)
                                    .child(SharedString::from(text)),
                            )
                            .into_any_element()
                    })
                    .collect()
            })
            .unwrap_or_default();
        v_flex()
            .key_context("GpPushGroup")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, ev, window, cx| this.key(ev, window, cx)))
            .w_full()
            .child(
                div()
                    .px_3()
                    .pt_2()
                    .pr(px(40.0))
                    .text_sm()
                    .text_color(skin.muted)
                    .child(title),
            )
            .child(
                v_flex()
                    .mt_1()
                    .py_1()
                    .border_t_1()
                    .border_b_1()
                    .border_color(skin.border)
                    .min_h(px(60.0))
                    .children(rows),
            )
            .child(
                div()
                    .px_3()
                    .py_1()
                    .h(px(26.0))
                    .text_xs()
                    .text_color(skin.text)
                    .child(SharedString::from(
                        run.as_ref().map(footer_text).unwrap_or_default(),
                    )),
            )
            .child(
                h_flex()
                    .px_2()
                    .pb_2()
                    .gap_2()
                    .child(
                        chip(
                            "group-retry",
                            m::BTN_RETRY.get(),
                            "r",
                            can_retry,
                            false,
                            &skin,
                        )
                        .on_click(cx.listener(|this, _, _, cx| this.retry(cx))),
                    )
                    .child(div().flex_1())
                    .child(
                        chip("group-close", m::BTN_CLOSE.get(), "", true, false, &skin)
                            .on_click(|_, window, cx| window.close_dialog(cx)),
                    ),
            )
            .child(
                div()
                    .px_3()
                    .pb_2()
                    .text_xs()
                    .text_color(skin.muted)
                    .child(m::HINT_RUN.get()),
            )
    }
}

/// Shows the progress of the last grouped run in a dialog; `None` when there is none.
pub fn open(window: &mut Window, cx: &mut App) -> Option<Entity<GroupView>> {
    if let Some(existing) = cx.try_global::<OpenGroup>().and_then(|o| o.0.upgrade()) {
        return Some(existing);
    }
    let backend = crate::app::workspace_of(cx)?.read(cx).backend().clone();
    backend.host().group_run()?;
    let view = cx.new(|cx| GroupView::new(backend, cx));
    cx.set_global(OpenGroup(view.downgrade()));
    window.open_dialog(cx, {
        let view = view.clone();
        move |dialog, _, _| {
            let view = view.clone();
            dialog
                .close_button(true)
                .overlay_closable(true)
                .w(px(560.0))
                .p_0()
                .content(move |content, _, _| content.child(view.clone()))
        }
    });
    let focus = view.clone();
    window.defer(cx, move |window, cx| {
        let handle = focus.read(cx).focus.clone();
        handle.focus(window, cx);
    });
    Some(view)
}

/// The open progress view, if any.
pub fn current(cx: &App) -> Option<Entity<GroupView>> {
    cx.try_global::<OpenGroup>().and_then(|o| o.0.upgrade())
}
