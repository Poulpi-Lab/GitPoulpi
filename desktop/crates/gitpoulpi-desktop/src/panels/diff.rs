//! The centre pane when a file is open: GitPoulpi swaps the Log for the diff of the clicked file
//! (Esc goes back). Kit `Diff`: side-by-side, word-level intra-line highlights, tree-sitter syntax.
//!
//! `DiffState` learns its viewport width while painting and calls `cx.notify()` on *itself*; the owner
//! of the `Diff` element must observe the state (`cx.observe`) or the second layout pass never happens
//! and the columns stay zero-width.

use std::sync::Arc;

use gitpoulpi_core::domain::FileDiff;
use gpui_kit::assets::IconName as AssetIcon;
use gpui_kit::component::{
    diff::{Diff, DiffFile, DiffMode, DiffState},
    h_flex,
    spinner::Spinner,
    v_flex, Icon,
};
use gpui_kit::{
    actions, div, px, App, AppContext as _, Context, Entity, EventEmitter, FocusHandle, Focusable,
    FontWeight, InteractiveElement as _, IntoElement, KeyBinding, ParentElement as _, Render,
    StatefulInteractiveElement as _, Styled as _, Subscription, Task, Window,
};

use crate::theme::Skin;
use crate::util::patch;

actions!(gp_diff, [CloseDiff, NextDiffFile, PrevDiffFile]);

pub fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("escape", CloseDiff, Some("GpDiffPane")),
        KeyBinding::new("]", NextDiffFile, Some("GpDiffPane")),
        KeyBinding::new("[", PrevDiffFile, Some("GpDiffPane")),
    ]);
}

pub enum DiffPaneEvent {
    Close,
}

pub struct DiffPane {
    parsed: Arc<Vec<DiffFile>>,
    selected: usize,
    diff: Entity<DiffState>,
    split: bool,
    loading: bool,
    revision: u64,
    focus: FocusHandle,
    _task: Option<Task<()>>,
    _subs: Vec<Subscription>,
}

impl EventEmitter<DiffPaneEvent> for DiffPane {}

impl DiffPane {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let diff = cx.new(|cx| DiffState::new([], cx).with_mode(DiffMode::Split));
        // The fix for the zero-width race: re-render when the state learns its viewport.
        let subs = vec![cx.observe(&diff, |_, _, cx| cx.notify())];
        Self {
            parsed: Arc::default(),
            selected: 0,
            diff,
            split: true,
            loading: false,
            revision: 0,
            focus: cx.focus_handle(),
            _task: None,
            _subs: subs,
        }
    }

    pub fn show(&mut self, files: Arc<Vec<FileDiff>>, ix: usize, cx: &mut Context<Self>) {
        self.revision += 1;
        let rev = self.revision;
        self.loading = true;
        self._task = Some(cx.spawn(async move |this, cx| {
            let parsed = cx
                .background_spawn(async move {
                    let patch = patch::to_patch(&files);
                    DiffFile::parse(&patch).unwrap_or_default()
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                if this.revision != rev {
                    return;
                }
                this.loading = false;
                this.parsed = Arc::new(parsed);
                this.select(ix, cx);
            });
        }));
        cx.notify();
    }

    fn select(&mut self, ix: usize, cx: &mut Context<Self>) {
        let Some(file) = self.parsed.get(ix).cloned() else {
            self.diff.update(cx, |d, cx| d.set_files([], cx));
            return;
        };
        self.selected = ix;
        self.diff.update(cx, |d, cx| d.set_files([file], cx));
        cx.notify();
    }

    fn step(&mut self, d: isize, cx: &mut Context<Self>) {
        if self.parsed.is_empty() {
            return;
        }
        let ix = self
            .selected
            .saturating_add_signed(d)
            .min(self.parsed.len() - 1);
        self.select(ix, cx);
    }

    pub fn focus_diff(&self, window: &mut Window, cx: &mut App) {
        self.diff.read(cx).focus_handle(cx).focus(window, cx);
    }
}

impl Focusable for DiffPane {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for DiffPane {
    fn render(&mut self, _w: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let skin = cx.global::<Skin>();
        let (border, text, muted, accent, success, danger, hover) = (
            skin.border,
            skin.text,
            skin.muted,
            skin.accent,
            skin.success,
            skin.danger,
            skin.hover,
        );
        let file = self.parsed.get(self.selected);
        let (path, adds, dels) = file
            .map(|f| (f.path().to_string(), f.additions(), f.deletions()))
            .unwrap_or_default();
        let n = self.parsed.len();
        let btn = |id: &'static str, icon: AssetIcon| {
            div()
                .id(id)
                .p(px(4.0))
                .rounded(px(3.0))
                .hover(move |d| d.bg(hover))
                .child(Icon::new(icon).size(px(14.0)).text_color(text))
        };
        let body = if self.loading {
            v_flex()
                .size_full()
                .items_center()
                .justify_center()
                .child(Spinner::new())
                .into_any_element()
        } else {
            Diff::new(&self.diff)
                .line_number(true)
                .soft_wrap(false)
                .into_any_element()
        };
        v_flex()
            .size_full()
            .key_context("GpDiffPane")
            .track_focus(&self.focus)
            .on_action(cx.listener(|_, _: &CloseDiff, _, cx| cx.emit(DiffPaneEvent::Close)))
            .on_action(cx.listener(|this, _: &NextDiffFile, _, cx| this.step(1, cx)))
            .on_action(cx.listener(|this, _: &PrevDiffFile, _, cx| this.step(-1, cx)))
            .border_1()
            .border_color(border)
            .rounded(px(3.0))
            .child(
                h_flex()
                    .flex_none()
                    .h(px(30.0))
                    .px(px(8.0))
                    .gap(px(8.0))
                    .items_center()
                    .border_b_1()
                    .border_color(border)
                    .child(
                        btn("back", AssetIcon::ArrowLeft)
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(DiffPaneEvent::Close))),
                    )
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .truncate()
                            .min_w_0()
                            .child(path),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(success)
                            .child(format!("+{adds}")),
                    )
                    .child(div().text_xs().text_color(danger).child(format!("-{dels}")))
                    .child(div().flex_1())
                    .child(div().text_xs().text_color(muted).child(format!(
                        "{} / {}",
                        if n == 0 { 0 } else { self.selected + 1 },
                        n
                    )))
                    .child(
                        btn("prev", AssetIcon::ChevronLeft)
                            .on_click(cx.listener(|this, _, _, cx| this.step(-1, cx))),
                    )
                    .child(
                        btn("next", AssetIcon::ChevronRight)
                            .on_click(cx.listener(|this, _, _, cx| this.step(1, cx))),
                    )
                    .child(
                        div()
                            .id("mode")
                            .px(px(8.0))
                            .h(px(20.0))
                            .flex()
                            .items_center()
                            .text_xs()
                            .rounded(px(3.0))
                            .text_color(accent)
                            .hover(move |d| d.bg(hover))
                            .child(if self.split {
                                gitpoulpi_i18n::msg::settings_value::SPLIT.get()
                            } else {
                                gitpoulpi_i18n::msg::settings_value::INLINE.get()
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.split = !this.split;
                                let mode = if this.split {
                                    DiffMode::Split
                                } else {
                                    DiffMode::Unified
                                };
                                this.diff.update(cx, |d, cx| d.set_mode(mode, cx));
                                cx.notify();
                            })),
                    ),
            )
            .child(div().flex_1().min_h_0().p(px(4.0)).child(body))
    }
}
