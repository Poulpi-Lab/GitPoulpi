//! The "you can always go back" panel: three views over what the daemon keeps (Undo history, Reflog,
//! Lost work), shown in a dialog. This file is the state and what the actions do; `safety_render` paints
//! it and takes the keys.
//!
//! Every read and every change is a request on the link's connection answered into a future the view
//! awaits on the window's executor, so nothing here blocks the render thread. Each action keeps the
//! terminal UI's confirmations: "undo up to here" lists the entries first, a risky entry the daemon
//! flags asks again ("Undo anyway"), "restore only this" and a reflog restore show the plan first, a
//! name taken asks for another, a reflog restore that would overwrite local changes asks what happens to
//! them. One action is one undo entry.

use gitpoulpi_core::domain::{JournalState, Oid};
use gitpoulpi_core::Error as CoreError;
use gitpoulpi_i18n::msg;
use gitpoulpi_i18n::msg::undo_safety as m;
use gitpoulpi_link::error_text::error_text;
use gpui_kit::{
    AnyWindowHandle, App, Context, Entity, FocusHandle, Global, Task, UniformListScrollHandle,
    WeakEntity, Window,
};

use super::safety_model::{actions, Act, HistoryModel, LostModel, ReflogModel, Tab};
use crate::commands::status;
use crate::daemon::Shared;
use crate::dialogs::{self, Choice, Spec};
use crate::form_dialog::{self, Field, FormSpec};

/// How many reflog positions one request fetches.
mod actions;

const REFLOG_PAGE: usize = 200;
/// How many lost commits one request fetches.
const LOST_PAGE: usize = 200;

/// The open panel, for the commands and the palette's availability.
pub(super) struct OpenSafety(pub WeakEntity<SafetyView>);
impl Global for OpenSafety {}

/// What a typed name is for.
#[derive(Debug, Clone)]
enum NameFor {
    /// A restore whose name is taken: run it again under the new one.
    Restore {
        id: u64,
    },
    Branch {
        at: Oid,
    },
    Tag {
        at: Oid,
    },
}

pub struct SafetyView {
    pub backend: Shared,
    pub(super) window: AnyWindowHandle,
    pub tab: Tab,
    pub history: HistoryModel,
    pub reflog: ReflogModel,
    pub lost: LostModel,
    /// A line under the lists: what the last action did (or why it failed).
    pub note: Option<(String, bool)>,
    /// An action is running: the buttons are inert.
    pub busy: bool,
    pub(super) focus: FocusHandle,
    pub(super) scroll: UniformListScrollHandle,
    /// The journal as it was when the history was read: the list reloads when the daemon's moved.
    seen: JournalState,
    job: Option<Task<()>>,
    loads: Vec<Task<()>>,
    _watch: Option<Task<()>>,
}

impl SafetyView {
    pub fn new(
        backend: Shared,
        tab: Tab,
        refname: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let seen = backend.with_link(|l| l.journal_state());
        let mut this = Self {
            backend: backend.clone(),
            window: window.window_handle(),
            tab,
            history: HistoryModel::default(),
            reflog: ReflogModel {
                refname: refname.unwrap_or_else(|| "HEAD".to_string()),
                ..Default::default()
            },
            lost: LostModel::default(),
            note: None,
            busy: false,
            focus: cx.focus_handle(),
            scroll: UniformListScrollHandle::new(),
            seen,
            job: None,
            loads: Vec::new(),
            _watch: None,
        };
        this.load(tab, cx);
        // The undo journal moving (a toolbar Undo, another frontend) reloads the history.
        let notifier = backend.notifier().clone();
        let mut epoch = notifier.epoch();
        this._watch = Some(cx.spawn(async move |this, cx| loop {
            notifier.changed(&mut epoch).await;
            if this.update(cx, |this, cx| this.on_wake(cx)).is_err() {
                break;
            }
        }));
        this
    }

    fn on_wake(&mut self, cx: &mut Context<Self>) {
        let now = self.backend.with_link(|l| l.journal_state());
        if now != self.seen && !self.busy {
            self.seen = now;
            if self.history.loaded {
                self.load(Tab::History, cx);
            }
        }
        cx.notify();
    }

    /// The daemon's view of the two top entries.
    pub fn journal_state(&self) -> JournalState {
        self.backend.with_link(|l| l.journal_state())
    }

    /// The key that undoes, as bound.
    fn undo_key(&self) -> String {
        let key = gitpoulpi_app::keymap::current().shown(gitpoulpi_app::keymap::Action::GlobalUndo);
        if key.is_empty() {
            "Ctrl+Z".to_string()
        } else {
            key
        }
    }

    // ---- loading -------------------------------------------------------------------------------

    pub fn set_tab(&mut self, tab: Tab, cx: &mut Context<Self>) {
        if self.tab == tab {
            return;
        }
        self.tab = tab;
        let loaded = match tab {
            Tab::History => self.history.loaded,
            Tab::Reflog => self.reflog.loaded,
            Tab::Lost => self.lost.loaded || self.lost.scanning,
        };
        if !loaded {
            self.load(tab, cx);
        }
        cx.notify();
    }

    /// Shows the Reflog of `refname` (`HEAD` or a branch).
    pub fn show_reflog_of(&mut self, refname: &str, cx: &mut Context<Self>) {
        self.reflog = ReflogModel {
            refname: refname.to_string(),
            ..Default::default()
        };
        self.tab = Tab::Reflog;
        self.load(Tab::Reflog, cx);
        cx.notify();
    }

    pub fn load(&mut self, tab: Tab, cx: &mut Context<Self>) {
        let backend = self.backend.clone();
        match tab {
            Tab::History => {
                self.history.error = None;
                let fut = backend.journal_listing();
                self.loads.push(cx.spawn(async move |this, cx| {
                    let result = fut.await;
                    let _ = this.update(cx, |this, cx| {
                        match result {
                            Ok(listing) => {
                                this.seen = this.journal_state();
                                this.history.set(listing)
                            }
                            Err(e) => this.history.error = Some(error_text(&e)),
                        }
                        cx.notify();
                    });
                }));
            }
            Tab::Reflog => {
                self.reflog.error = None;
                self.reflog.loaded = false;
                let fut = backend.reflog_page(&self.reflog.refname, 0, REFLOG_PAGE);
                self.loads.push(cx.spawn(async move |this, cx| {
                    let result = fut.await;
                    let _ = this.update(cx, |this, cx| {
                        match result {
                            Ok(page) => {
                                this.reflog.entries = page.entries;
                                this.reflog.more = page.more;
                                this.reflog.loaded = true;
                                let rows = this.reflog.rows();
                                this.reflog.cursor.clamp(rows);
                            }
                            Err(e) => this.reflog.error = Some(error_text(&e)),
                        }
                        cx.notify();
                    });
                }));
            }
            Tab::Lost => {
                self.lost.error = None;
                self.lost.scanning = true;
                // The cheap pass first (reflogs and undo pins) so something shows at once; the scan of
                // dangling objects follows and replaces it.
                let (quick, deep) = (
                    backend.lost_commits(false, LOST_PAGE),
                    backend.lost_commits(true, LOST_PAGE),
                );
                self.loads.push(cx.spawn(async move |this, cx| {
                    let first = quick.await;
                    let _ = this.update(cx, |this, cx| {
                        if this.lost.scanning {
                            this.lost_accept(first);
                        }
                        cx.notify();
                    });
                    let second = deep.await;
                    let _ = this.update(cx, |this, cx| {
                        this.lost.scanning = false;
                        this.lost_accept(second);
                        cx.notify();
                    });
                }));
            }
        }
        cx.notify();
    }

    fn lost_accept(&mut self, result: Result<gitpoulpi_core::domain::LostCommitsPage, CoreError>) {
        match result {
            Ok(page) => {
                let keep = self.lost.selected().map(|c| c.oid);
                self.lost.commits = page.commits;
                self.lost.partial = page.partial;
                if let Some(at) =
                    keep.and_then(|oid| self.lost.commits.iter().position(|c| c.oid == oid))
                {
                    self.lost.cursor.selected = at;
                }
                let len = self.lost.commits.len();
                self.lost.cursor.clamp(len);
                self.lost.loaded = true;
            }
            Err(e) => self.lost.error = Some(error_text(&e)),
        }
    }

    /// Loads the next page of the reflog.
    fn load_more(&mut self, cx: &mut Context<Self>) {
        if !self.reflog.more || !self.reflog.loaded || self.reflog.fetching_more {
            return;
        }
        self.reflog.fetching_more = true;
        let fut =
            self.backend
                .reflog_page(&self.reflog.refname, self.reflog.entries.len(), REFLOG_PAGE);
        self.loads.push(cx.spawn(async move |this, cx| {
            let result = fut.await;
            let _ = this.update(cx, |this, cx| {
                this.reflog.fetching_more = false;
                match result {
                    Ok(page) => {
                        this.reflog.entries.extend(page.entries);
                        this.reflog.more = page.more;
                        let held = this.reflog.entries.len() as u64;
                        this.say(m::reflog_loaded(held), cx);
                    }
                    Err(e) => this.complain(error_text(&e), cx),
                }
                cx.notify();
            });
        }));
        cx.notify();
    }

    // ---- notes ---------------------------------------------------------------------------------

    pub(super) fn say(&mut self, text: String, cx: &mut Context<Self>) {
        status::set(cx, text.clone());
        self.note = Some((text, false));
        cx.notify();
    }

    pub(super) fn complain(&mut self, text: String, cx: &mut Context<Self>) {
        status::set_error(cx, text.clone());
        self.note = Some((text, true));
        cx.notify();
    }

    // ---- selection -----------------------------------------------------------------------------

    pub fn move_selection(&mut self, delta: isize, cx: &mut Context<Self>) {
        match self.tab {
            Tab::History => {
                self.history.move_by(delta);
                self.scroll
                    .scroll_to_item(self.history.selected, gpui_kit::ScrollStrategy::Nearest);
            }
            Tab::Reflog => {
                let rows = self.reflog.rows();
                self.reflog.cursor.move_by(delta, rows);
                self.scroll.scroll_to_item(
                    self.reflog.cursor.selected,
                    gpui_kit::ScrollStrategy::Nearest,
                );
            }
            Tab::Lost => {
                let rows = self.lost.commits.len();
                self.lost.cursor.move_by(delta, rows);
                self.scroll
                    .scroll_to_item(self.lost.cursor.selected, gpui_kit::ScrollStrategy::Nearest);
            }
        }
        cx.notify();
    }

    /// Selects row `ix` of the open view (a click).
    pub fn select(&mut self, ix: usize, cx: &mut Context<Self>) {
        match self.tab {
            Tab::History => {
                if self.history.row_at(ix).is_some() {
                    self.history.selected = ix;
                }
            }
            Tab::Reflog => {
                if self.reflog.is_more_row(ix) {
                    self.load_more(cx);
                } else if ix < self.reflog.entries.len() {
                    self.reflog.cursor.selected = ix;
                }
            }
            Tab::Lost => {
                if ix < self.lost.commits.len() {
                    self.lost.cursor.selected = ix;
                }
            }
        }
        cx.notify();
    }

    /// The actions that apply to the selected row of the open view.
    pub fn actions(&self) -> Vec<Act> {
        actions(
            self.tab,
            &self.history,
            &self.reflog,
            &self.lost,
            &self.journal_state(),
        )
    }

    // ---- dialogs -------------------------------------------------------------------------------

    /// Runs `f` on the window once the current event is over (the dialogs belong to the window).
    fn with_window(&self, cx: &mut Context<Self>, f: impl FnOnce(&mut Window, &mut App) + 'static) {
        let window = self.window;
        cx.defer(move |cx| {
            let _ = window.update(cx, move |_, window, cx| f(window, cx));
        });
    }

    /// A question: choice 0 runs `then`, any other answer (or closing it) does nothing.
    fn ask(
        &mut self,
        spec: Spec,
        then: impl FnOnce(&mut SafetyView, &mut Context<SafetyView>) + 'static,
        cx: &mut Context<Self>,
    ) {
        let me = cx.weak_entity();
        self.with_window(cx, move |window, cx| {
            dialogs::open(
                window,
                cx,
                spec,
                move |ix, cx| {
                    if ix == 0 {
                        let _ = me.update(cx, then);
                    }
                },
                || {},
            );
        });
    }

    fn confirm_spec(title: String, lines: Vec<String>, button: String, red: bool) -> Spec {
        Spec {
            title,
            lines,
            choices: vec![
                if red {
                    Choice::destructive(button)
                } else {
                    Choice::new(button)
                },
                Choice::new(msg::dialog::CANCEL.get()),
            ],
            hint: msg::diff_actions::CONFIRM_HINT.get().to_string(),
            highlight_first: false,
            yes_no_keys: false,
        }
    }

    /// A one-line text prompt (a name); `submit` runs with the trimmed value.
    fn ask_name(
        &mut self,
        prompt: String,
        initial: String,
        purpose: NameFor,
        cx: &mut Context<Self>,
    ) {
        let me = cx.weak_entity();
        self.with_window(cx, move |window, cx| {
            form_dialog::open(
                window,
                cx,
                FormSpec {
                    title: m::TITLE_NAME.get().to_string(),
                    intro: vec![prompt],
                    fields: vec![Field::text(m::TITLE_NAME.get(), "", &initial)],
                    submit: m::BTN_CREATE_OK.get().to_string(),
                    cancel: msg::dialog::CANCEL.get().to_string(),
                },
                |values| {
                    if values.text(0).is_empty() {
                        Err(m::TITLE_NAME.get().to_string())
                    } else {
                        Ok(())
                    }
                },
                move |values, _, cx| {
                    let value = values.text(0).to_string();
                    let _ = me.update(cx, |this, cx| this.submit_name(purpose, value, cx));
                },
            );
        });
    }
}

/// The open panel, if any.
pub fn current(cx: &App) -> Option<Entity<SafetyView>> {
    cx.try_global::<OpenSafety>().and_then(|o| o.0.upgrade())
}
