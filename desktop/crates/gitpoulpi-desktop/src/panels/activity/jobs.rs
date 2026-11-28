//! Background jobs: how a job is worded, how one is cancelled, and the jobs panel (running, queued and
//! finished jobs, a Cancel where one can be had and the reason where not).
//!
//! The daemon sends facts (`kind`, `repo`, `dest`); the window words them in its language. The list is the
//! link host's job map, fed by the link's job events, so a push started from the terminal UI or the CLI
//! shows here too. Cancelling is not an undoable mutation: a cancelled push, fetch or clone leaves the
//! repository as it was.

use std::path::Path;

use gitpoulpi_core::domain::OperationKind;
use gitpoulpi_core::NotCancellableReason;
use gitpoulpi_i18n::msg::jobs as m;
use gitpoulpi_ipc::{JobInfo, JobState};
use gitpoulpi_link::error_text::error_text;
use gpui_kit::component::{h_flex, v_flex, WindowExt as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    div, px, AnyElement, AnyWindowHandle, App, AppContext as _, Context, Entity, FocusHandle,
    Focusable, Global, InteractiveElement as _, IntoElement, KeyDownEvent, ParentElement as _,
    Render, SharedString, StatefulInteractiveElement as _, Styled as _, Task, WeakEntity, Window,
};

use super::widgets::chip;
use crate::commands::status;
use crate::daemon::Shared;
use crate::theme::Skin;

/// The short name of what a job does.
pub fn kind(kind: OperationKind) -> &'static str {
    use OperationKind as K;
    match kind {
        K::Fetch | K::FetchAll | K::FetchRefspec | K::FetchRefAsBranch => m::KIND_FETCH.get(),
        K::Push | K::PushForReview => m::KIND_PUSH.get(),
        K::TagPush => m::KIND_PUSH_TAGS.get(),
        K::Pull => m::KIND_PULL.get(),
        K::Clone => m::KIND_CLONE.get(),
        K::LfsFetch
        | K::LfsPull
        | K::LfsPush
        | K::LfsCheckout
        | K::LfsLocks
        | K::LfsLock
        | K::LfsUnlock => m::KIND_LFS.get(),
        _ => m::KIND_REMOTE.get(),
    }
}

fn folder_name(path: &str) -> Option<String> {
    Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .filter(|n| !n.is_empty())
}

/// `Push: tool`: the kind and the repository (or clone destination) folder.
pub fn label(job: &JobInfo) -> String {
    let name = job
        .repo
        .as_ref()
        .and_then(|r| folder_name(&r.0))
        .or_else(|| {
            job.dest
                .as_ref()
                .and_then(|d| folder_name(&d.to_string_lossy()))
        });
    match name {
        Some(name) => m::label(kind(job.kind), name),
        None => kind(job.kind).to_string(),
    }
}

pub fn state_word(state: JobState) -> &'static str {
    match state {
        JobState::Queued => m::STATE_QUEUED.get(),
        JobState::Running => m::STATE_RUNNING.get(),
        JobState::Cancelling => m::STATE_CANCELLING.get(),
        JobState::Done => m::STATE_DONE.get(),
        JobState::Failed => m::STATE_FAILED.get(),
        JobState::Canceled => m::STATE_CANCELED.get(),
    }
}

pub fn reason(reason: NotCancellableReason) -> &'static str {
    match reason {
        NotCancellableReason::Integrating => m::WHY_INTEGRATING.get(),
        NotCancellableReason::RewritingFiles => m::WHY_REWRITING.get(),
        NotCancellableReason::InSteps => m::WHY_STEPS.get(),
        NotCancellableReason::Sent => m::WHY_SENT.get(),
        NotCancellableReason::Purging => m::WHY_PURGING.get(),
        NotCancellableReason::Finished => m::WHY_FINISHED.get(),
        NotCancellableReason::UnknownJob => m::WHY_UNKNOWN.get(),
    }
}

/// Why a Cancel on this job would do nothing right now (`None`: it can be cancelled, or it has ended).
pub fn why(job: &JobInfo) -> Option<&'static str> {
    if job.state.is_finished() {
        return None;
    }
    if job.state == JobState::Cancelling {
        return Some(m::WHY_CANCELLING.get());
    }
    job.not_cancellable.map(reason)
}

/// The sentence for a Cancel the window will not even send.
pub fn refusal(job: &JobInfo) -> Option<String> {
    if job.state.is_finished() {
        return Some(m::not_cancellable(m::WHY_FINISHED.get()));
    }
    why(job).map(m::not_cancellable)
}

/// Asks the daemon to cancel job `id`. A job that cannot be stopped says why in the status line instead
/// and nothing is sent for it.
pub fn cancel(backend: &Shared, id: u64, cx: &mut App) {
    let Some(job) = backend.host().job(id) else {
        status::set(cx, m::not_cancellable(m::WHY_UNKNOWN.get()));
        return;
    };
    if let Some(why) = refusal(&job) {
        status::set(cx, why);
        return;
    }
    let name = label(&job);
    let fut = backend.cancel_job(id);
    cx.spawn(async move |cx| {
        let result = fut.await;
        cx.update(|cx| match result {
            Ok(_) => status::set(cx, m::canceled(name)),
            Err(e) => status::set_error(cx, format!("{name}: {}", error_text(&e))),
        });
    })
    .detach();
}

/// Cancels every job that can be; how many were asked.
pub fn cancel_all(backend: &Shared, cx: &mut App) -> usize {
    let jobs: Vec<JobInfo> = backend
        .host()
        .all_jobs()
        .into_iter()
        .filter(JobInfo::is_cancellable)
        .collect();
    for job in &jobs {
        cancel(backend, job.id, cx);
    }
    jobs.len()
}

/// The indicator's `x` and its keyboard twin: the oldest job that can be cancelled, else the reason the
/// first job gives.
pub fn cancel_oldest(backend: &Shared, cx: &mut App) {
    let jobs = backend.host().live_jobs();
    if let Some(job) = jobs.iter().find(|j| j.is_cancellable()).or(jobs.first()) {
        cancel(backend, job.id, cx);
    }
}

/// The open panel, for the commands.
struct OpenJobs(WeakEntity<JobsView>);
impl Global for OpenJobs {}

pub struct JobsView {
    backend: Shared,
    selected: Option<u64>,
    pub note: Option<String>,
    focus: FocusHandle,
    _watch: Option<Task<()>>,
}

impl JobsView {
    pub fn new(backend: Shared, cx: &mut Context<Self>) -> Self {
        let notifier = backend.notifier().clone();
        let mut epoch = notifier.epoch();
        let watch = cx.spawn(async move |this, cx| loop {
            notifier.changed(&mut epoch).await;
            if this.update(cx, |_, cx| cx.notify()).is_err() {
                break;
            }
        });
        let mut this = Self {
            backend,
            selected: None,
            note: None,
            focus: cx.focus_handle(),
            _watch: Some(watch),
        };
        this.selected = this.jobs().first().map(|j| j.id);
        this
    }

    /// Every job listed, the active ones first.
    pub fn jobs(&self) -> Vec<JobInfo> {
        self.backend.host().all_jobs()
    }

    pub fn selected(&self) -> Option<u64> {
        self.selected
    }

    pub fn select(&mut self, id: u64, cx: &mut Context<Self>) {
        self.selected = Some(id);
        cx.notify();
    }

    pub fn move_selection(&mut self, delta: isize, cx: &mut Context<Self>) {
        let jobs = self.jobs();
        if jobs.is_empty() {
            return;
        }
        let at = jobs
            .iter()
            .position(|j| Some(j.id) == self.selected)
            .unwrap_or(0);
        let to = at.saturating_add_signed(delta).min(jobs.len() - 1);
        self.selected = Some(jobs[to].id);
        cx.notify();
    }

    /// Cancels the selected job (its reason in the panel's note when it cannot be).
    pub fn cancel_selected(&mut self, cx: &mut Context<Self>) {
        let Some(job) = self.selected.and_then(|id| self.backend.host().job(id)) else {
            return;
        };
        self.note = refusal(&job);
        let backend = self.backend.clone();
        cancel(&backend, job.id, cx);
        cx.notify();
    }

    pub fn cancel_all(&mut self, cx: &mut Context<Self>) {
        let backend = self.backend.clone();
        self.note = (cancel_all(&backend, cx) == 0).then(|| m::PALETTE_NONE.get().to_string());
        cx.notify();
    }

    fn key(&mut self, ev: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let ks = &ev.keystroke;
        if ks.modifiers.control {
            if ks.key == "p" {
                window.defer(cx, crate::commands::palette::open);
            } else if ks.key == "j" {
                self.move_selection(1, cx);
            } else if ks.key == "k" {
                self.move_selection(-1, cx);
            }
            return;
        }
        match ks.key.as_str() {
            "up" => self.move_selection(-1, cx),
            "down" => self.move_selection(1, cx),
            "c" | "delete" => self.cancel_selected(cx),
            "escape" => window.close_dialog(cx),
            _ => {}
        }
    }

    fn row(&self, job: &JobInfo, skin: &Skin, cx: &mut Context<Self>) -> AnyElement {
        let id = job.id;
        let selected = self.selected == Some(id);
        let color = match job.state {
            JobState::Failed => skin.danger,
            JobState::Done | JobState::Canceled => skin.muted,
            _ => skin.text,
        };
        let progress = job
            .progress
            .map(|p| format!("{p}%"))
            .or_else(|| job.phase.clone())
            .unwrap_or_default();
        let blocked = why(job);
        let can = job.is_cancellable();
        h_flex()
            .id(("job", id as usize))
            .h(px(30.0))
            .px_3()
            .gap_3()
            .items_center()
            .text_sm()
            .text_color(color)
            .when(selected, |d| d.bg(skin.selection))
            .cursor_pointer()
            .hover(|d| d.bg(skin.hover))
            .child(div().flex_none().w(px(96.0)).child(state_word(job.state)))
            .child(div().flex_1().truncate().child(label(job)))
            .child(
                div()
                    .flex_none()
                    .text_xs()
                    .text_color(skin.muted)
                    .child(progress),
            )
            .child(
                div()
                    .flex_none()
                    .max_w(px(260.0))
                    .truncate()
                    .text_xs()
                    .text_color(skin.muted)
                    .child(blocked.unwrap_or("")),
            )
            .when(!job.state.is_finished(), |d| {
                d.child(
                    chip(
                        ("job-cancel", id as usize),
                        m::BUTTON_CANCEL.get(),
                        "",
                        can,
                        false,
                        skin,
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.selected = Some(id);
                        this.cancel_selected(cx);
                    })),
                )
            })
            .on_click(cx.listener(move |this, _, _, cx| this.select(id, cx)))
            .into_any_element()
    }
}

impl Focusable for JobsView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for JobsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let skin = cx.global::<Skin>().clone();
        let jobs = self.jobs();
        let any_live = jobs.iter().any(|j| j.is_cancellable());
        let rows: Vec<AnyElement> = jobs.iter().map(|j| self.row(j, &skin, cx)).collect();
        v_flex()
            .key_context("GpJobs")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, ev, window, cx| this.key(ev, window, cx)))
            .w_full()
            .h(px(420.0))
            .child(
                div()
                    .px_3()
                    .pt_2()
                    .pr(px(40.0))
                    .text_sm()
                    .text_color(skin.muted)
                    .child(m::TITLE.get()),
            )
            .child(
                v_flex()
                    .id("jobs-list")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .mt_1()
                    .border_t_1()
                    .border_b_1()
                    .border_color(skin.border)
                    .when(rows.is_empty(), |d| {
                        d.child(
                            div()
                                .p_3()
                                .text_sm()
                                .text_color(skin.muted)
                                .child(m::EMPTY.get()),
                        )
                    })
                    .children(rows),
            )
            .child(
                div()
                    .px_3()
                    .py_1()
                    .h(px(24.0))
                    .text_xs()
                    .text_color(skin.text)
                    .child(SharedString::from(self.note.clone().unwrap_or_default())),
            )
            .child(
                h_flex()
                    .px_2()
                    .pb_2()
                    .gap_2()
                    .child(
                        chip(
                            "jobs-cancel-all",
                            m::BUTTON_CANCEL_ALL.get(),
                            "",
                            any_live,
                            false,
                            &skin,
                        )
                        .on_click(cx.listener(|this, _, _, cx| this.cancel_all(cx))),
                    )
                    .child(div().flex_1())
                    .child(
                        chip(
                            "jobs-close",
                            gitpoulpi_i18n::msg::undo_safety::BTN_CLOSE.get(),
                            "",
                            true,
                            false,
                            &skin,
                        )
                        .on_click(|_, window, cx| window.close_dialog(cx)),
                    ),
            )
            .child(
                div()
                    .px_3()
                    .pb_2()
                    .text_xs()
                    .text_color(skin.muted)
                    .child(m::HINT.get()),
            )
    }
}

/// Shows the jobs panel in a dialog; the open one is reused.
pub fn open(window: &mut Window, cx: &mut App) -> Option<Entity<JobsView>> {
    if let Some(existing) = cx.try_global::<OpenJobs>().and_then(|o| o.0.upgrade()) {
        return Some(existing);
    }
    let backend = crate::app::workspace_of(cx)?.read(cx).backend().clone();
    let view = cx.new(|cx| JobsView::new(backend, cx));
    cx.set_global(OpenJobs(view.downgrade()));
    window.open_dialog(cx, {
        let view = view.clone();
        move |dialog, _, _| {
            let view = view.clone();
            dialog
                .close_button(true)
                .overlay_closable(true)
                .w(px(720.0))
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

/// The open jobs panel, if any.
pub fn current(cx: &App) -> Option<Entity<JobsView>> {
    cx.try_global::<OpenJobs>().and_then(|o| o.0.upgrade())
}

/// Needs a window: kept so the type is used where the dialog is built.
#[allow(dead_code)]
type _Handle = AnyWindowHandle;
