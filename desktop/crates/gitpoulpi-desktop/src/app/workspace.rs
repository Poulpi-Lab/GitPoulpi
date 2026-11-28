//! The main window's content: title bar, tab row, three panes (Refs sidebar | Log or file diff | right
//! pane), the toolbar row and the status line. Chrome lives in `chrome.rs`, the palette in `palette.rs`.

use std::sync::Arc;
use std::time::Duration;

use gitpoulpi_app::keymap::Ctx;
use gitpoulpi_app::palette::vocab::status_pane::StatusSection;
use gpui_kit::component::{
    resizable::{h_resizable, resizable_panel, ResizablePanelEvent, ResizableState},
    v_flex,
};
use gpui_kit::{
    div, px, App, AppContext as _, Context, Entity, Focusable as _, Image, ImageFormat,
    InteractiveElement as _, IntoElement, ParentElement as _, Render, Styled as _, Subscription,
    Task, Window,
};

mod ref_forms;
mod ref_menu_host;
mod refs;
mod stage;

use super::actions::WorkspaceHandle;
use super::live::{LiveState, Notice};
use crate::commands::context::WindowState;
use crate::daemon::Shared;
use crate::menu::MenuBar;
use crate::panels::diff::{DiffPane, DiffPaneEvent, DiffSource};
use crate::panels::log::{LogEvent, LogPanel, Selection, Summary};
use crate::panels::refs::{RefsEvent, RefsPanel};
use crate::panels::right::{RightEvent, RightPanel};
use crate::prefs::{LayoutPrefs, PrefsStore, WindowPrefs};
use crate::theme::Skin;
use gitpoulpi_app::palette::vocab::undo::Avail;
use gitpoulpi_core::domain::Direction;

/// How long the window size must stay unchanged before it is written to the preferences.
const SAVE_WINDOW_AFTER: Duration = Duration::from_millis(600);

const LOGO_PNG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../assets/logo/gitpoulpi.png"
));

#[derive(Clone, Copy, PartialEq)]
enum Center {
    Log,
    Diff,
}

pub struct Workspace {
    pub(super) backend: Shared,
    pub refs: Entity<RefsPanel>,
    pub log: Entity<LogPanel>,
    pub right: Entity<RightPanel>,
    pub diff: Entity<DiffPane>,
    center: Center,
    show_left: bool,
    show_right: bool,
    panes: Entity<ResizableState>,
    menu_bar: Entity<MenuBar>,
    selected_commit: bool,
    /// The ref menu, while one is open.
    ref_menu: Option<ref_menu_host::OpenMenu>,
    pub(super) summary: Option<Summary>,
    /// What the last action said, for the status line.
    pub(super) notice: Option<Notice>,
    pub(super) live: LiveState,
    pub(super) _pump: Option<Task<()>>,
    pub(super) logo: Arc<Image>,
    _save_window: Option<Task<()>>,
    _subs: Vec<Subscription>,
}

impl Workspace {
    pub fn new(backend: Shared, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let layout = PrefsStore::get(cx).layout.clone();
        let refs = cx.new(|cx| RefsPanel::new(backend.clone(), window, cx));
        let log = cx.new(|cx| LogPanel::new(backend.clone(), window, cx));
        let right = cx.new(|cx| RightPanel::new(backend.clone(), window, cx));
        let diff = cx.new(|cx| DiffPane::new(window, cx));
        let panes = cx.new(|_| ResizableState::default());
        let menu_bar = cx.new(MenuBar::new);
        crate::menu::register(&menu_bar, cx);
        let subs = vec![
            cx.subscribe(&log, |this, _, ev: &LogEvent, cx| this.on_log_event(ev, cx)),
            cx.subscribe_in(
                &refs,
                window,
                |this, _, ev: &RefsEvent, window, cx| match ev {
                    RefsEvent::Selected(oid) => {
                        this.show_log(cx);
                        this.log.update(cx, |l, cx| l.select_oid(*oid, cx));
                    }
                    RefsEvent::Menu { target, at } => {
                        this.open_ref_menu(target.clone(), *at, window, cx)
                    }
                    RefsEvent::Open(target) => this.open_ref_row(target.clone(), window, cx),
                    RefsEvent::Dropped { source, target, at } => {
                        this.open_drop_menu(source.clone(), target.clone(), *at, window, cx)
                    }
                    RefsEvent::VisibilityChanged { hidden, solo } => {
                        let (hidden, solo) = (hidden.clone(), solo.clone());
                        this.log
                            .update(cx, |l, cx| l.set_visibility(hidden, solo, cx));
                    }
                },
            ),
            cx.subscribe_in(
                &right,
                window,
                |this, _, ev: &RightEvent, window, cx| match ev {
                    RightEvent::Open { files, ix, source } => {
                        this.diff
                            .update(cx, |d, cx| d.show(files.clone(), *ix, *source, cx));
                        this.center = Center::Diff;
                        this.diff.read(cx).focus_handle(cx).focus(window, cx);
                        cx.notify();
                    }
                    RightEvent::Say(text) => {
                        this.notice = Some(Notice::info(text.clone()));
                        cx.notify();
                    }
                },
            ),
            cx.subscribe_in(
                &diff,
                window,
                |this, _, ev: &DiffPaneEvent, window, cx| match ev {
                    DiffPaneEvent::Close => this.close_diff(window, cx),
                    DiffPaneEvent::Part(req) => this.on_part(req.clone(), window, cx),
                    DiffPaneEvent::Say(text) => {
                        this.notice = Some(Notice::info(text.clone()));
                        cx.notify();
                    }
                },
            ),
            cx.subscribe(&panes, |this, _, _: &ResizablePanelEvent, cx| {
                this.save_pane_sizes(cx)
            }),
            cx.observe_window_bounds(window, |this, window, cx| this.window_resized(window, cx)),
        ];
        let weak = cx.weak_entity();
        cx.set_global(WorkspaceHandle(weak));
        cx.defer_in(window, |this, window, cx| {
            this.log.read(cx).focus_handle(cx).focus(window, cx)
        });
        let mut this = Self {
            backend,
            refs,
            log,
            right,
            diff,
            center: Center::Log,
            show_left: layout.show_left,
            show_right: layout.show_right,
            panes,
            menu_bar,
            selected_commit: false,
            ref_menu: None,
            summary: None,
            notice: None,
            live: LiveState::default(),
            _pump: None,
            logo: Arc::new(Image::from_bytes(ImageFormat::Png, LOGO_PNG.to_vec())),
            _save_window: None,
            _subs: subs,
        };
        this.start_pump(cx);
        this
    }

    fn on_log_event(&mut self, ev: &LogEvent, cx: &mut Context<Self>) {
        match ev {
            LogEvent::Selected(sel) => {
                self.selected_commit = matches!(sel, Selection::Commit(_));
                let status = self
                    .summary
                    .as_ref()
                    .map(|s| s.status.clone())
                    .unwrap_or_default();
                let sel = sel.clone();
                self.right.update(cx, |r, cx| r.show(sel, &status, cx));
            }
            LogEvent::Loaded(s) => {
                let status = s.status.clone();
                let branch = s.branch.clone();
                self.right.update(cx, |r, cx| {
                    r.set_branch(branch);
                    r.update_status(&status, cx);
                });
                self.summary = Some(Summary {
                    loaded: s.loaded,
                    total: s.total,
                    branch: s.branch.clone(),
                    ms: s.ms,
                    status: s.status.clone(),
                });
                cx.notify();
            }
        }
    }

    /// Closes the file diff and returns to the Log.
    pub(crate) fn close_diff(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.center == Center::Diff {
            self.center = Center::Log;
            self.log.read(cx).focus_handle(cx).focus(window, cx);
            cx.notify();
        }
    }

    fn show_log(&mut self, cx: &mut Context<Self>) {
        if self.center != Center::Log {
            self.center = Center::Log;
            cx.notify();
        }
    }

    pub(crate) fn focus_log(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.show_log(cx);
        self.log.read(cx).focus_handle(cx).focus(window, cx);
    }

    /// The open diff replaces the Log in the centre.
    pub fn diff_is_open(&self) -> bool {
        self.center == Center::Diff
    }

    /// Puts a line in the status line.
    pub(crate) fn say(&mut self, text: impl Into<String>, cx: &mut Context<Self>) {
        self.notice = Some(Notice::info(text));
        cx.notify();
    }

    /// The menu bar.
    pub fn menu_bar(&self) -> &Entity<MenuBar> {
        &self.menu_bar
    }

    /// What the command layer needs to know: the pane with the keyboard and a few facts for the palette.
    pub(crate) fn command_state(&self, window: &Window, cx: &App) -> WindowState {
        let has = |handle: gpui_kit::FocusHandle| handle.contains_focused(window, cx);
        let right = self.right.read(cx);
        let pane = if right.composer_focused(window, cx) {
            Ctx::Composer
        } else if has(self.refs.read(cx).focus_handle(cx)) {
            Ctx::Refs
        } else if has(self.log.read(cx).focus_handle(cx)) {
            Ctx::Log
        } else if has(self.right.read(cx).focus_handle(cx)) {
            Ctx::Status
        } else if has(self.diff.read(cx).focus_handle(cx)) {
            Ctx::Diff
        } else {
            Ctx::Global
        };
        let right = self.right.read(cx);
        let source = self.diff.read(cx).source();
        let diff_open = self.center == Center::Diff;
        WindowState {
            pane,
            unstaged_selected: right.selected_in(StatusSection::Unstaged),
            staged_selected: right.selected_in(StatusSection::Staged),
            has_conflicts: right.conflict_count() > 0,
            stageable_diff: diff_open && source.is_working(),
            diff_stages: diff_open && source == DiffSource::Unstaged,
            diff_open,
            selected_commit: self.selected_commit,
            has_wip: self.summary.as_ref().is_some_and(|s| !s.status.is_empty()),
            refs: self.refs.read(cx).selection_facts(),
            undo: self.undo_avail(Direction::Undo),
            redo: self.undo_avail(Direction::Redo),
        }
    }

    /// What the journal offers, for the commands' availability (toolbar, menus, palette).
    pub(crate) fn undo_avail(&self, d: Direction) -> Avail {
        if self.backend.can_undo_redo(d) {
            Avail::Ready
        } else {
            Avail::Nothing
        }
    }

    pub fn toggle_left(&mut self, cx: &mut Context<Self>) {
        self.show_left = !self.show_left;
        let on = self.show_left;
        PrefsStore::update(cx, |p| p.layout.show_left = on);
        cx.notify();
    }

    pub fn toggle_right(&mut self, cx: &mut Context<Self>) {
        self.show_right = !self.show_right;
        let on = self.show_right;
        PrefsStore::update(cx, |p| p.layout.show_right = on);
        cx.notify();
    }

    pub fn open_repo(&mut self, path: &std::path::Path, cx: &mut Context<Self>) {
        if self.backend.set_repo(path) {
            let root = self.backend.repo_root().to_string_lossy().into_owned();
            PrefsStore::update(cx, |p| p.remember_recent(&root));
            self.reset_live();
            self.refs.update(cx, |r, cx| r.reload(cx));
            self.log.update(cx, |l, cx| l.reload(cx));
            cx.notify();
        }
    }

    /// Writes the side panes' widths once a resize settles. The panel order is Refs (if shown), the
    /// centre, the right pane (if shown).
    fn save_pane_sizes(&mut self, cx: &mut Context<Self>) {
        let sizes: Vec<f32> = self
            .panes
            .read(cx)
            .sizes()
            .iter()
            .map(|s| s.as_f32())
            .collect();
        let (left_shown, right_shown) = (self.show_left, self.show_right);
        let left = left_shown.then(|| sizes.first().copied()).flatten();
        let right = right_shown
            .then(|| sizes.last().copied())
            .flatten()
            .filter(|_| sizes.len() >= 2);
        PrefsStore::update(cx, |p| {
            if let Some(w) = left {
                p.layout.left_width = w.clamp(LayoutPrefs::LEFT_RANGE.0, LayoutPrefs::LEFT_RANGE.1);
            }
            if let Some(w) = right {
                p.layout.right_width =
                    w.clamp(LayoutPrefs::RIGHT_RANGE.0, LayoutPrefs::RIGHT_RANGE.1);
            }
        });
    }

    /// Remembers the window size shortly after the user stops resizing.
    fn window_resized(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let size = window.bounds().size;
        let (width, height) = (size.width.as_f32(), size.height.as_f32());
        self._save_window = Some(cx.spawn(async move |_this, cx| {
            cx.background_executor().timer(SAVE_WINDOW_AFTER).await;
            cx.update(|cx| {
                PrefsStore::update(cx, |p| p.window = WindowPrefs { width, height }.clamped());
            });
        }));
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let skin = cx.global::<Skin>().clone();
        let layout = PrefsStore::get(cx).layout.clone();
        let center = match self.center {
            Center::Log => self.log.clone().into_any_element(),
            Center::Diff => self.diff.clone().into_any_element(),
        };
        let mut panes = h_resizable("gp-panes").with_state(&self.panes);
        if self.show_left {
            panes = panes.child(
                resizable_panel()
                    .size(px(layout.left_width))
                    .size_range(px(LayoutPrefs::LEFT_RANGE.0)..px(LayoutPrefs::LEFT_RANGE.1))
                    .child(self.refs.clone()),
            );
        }
        panes = panes.child(resizable_panel().child(div().size_full().p(px(4.0)).child(center)));
        if self.show_right {
            panes = panes.child(
                resizable_panel()
                    .size(px(layout.right_width))
                    .size_range(px(LayoutPrefs::RIGHT_RANGE.0)..px(LayoutPrefs::RIGHT_RANGE.1))
                    .child(self.right.clone()),
            );
        }
        v_flex()
            .id("workspace")
            .size_full()
            .bg(skin.bg)
            .text_color(skin.text)
            .child(self.title_bar(cx))
            .child(self.menu_bar.clone())
            .child(self.tab_row(cx))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .border_b_1()
                    .border_color(skin.border)
                    .child(panes),
            )
            .child(self.toolbar(cx))
            .child(self.status_line(cx))
            .children(self.menu_overlay(cx))
    }
}
