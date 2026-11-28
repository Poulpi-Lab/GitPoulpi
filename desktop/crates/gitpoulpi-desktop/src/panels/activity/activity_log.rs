//! The Activity Log: a dialog with two tabs, **Git operations** (what the link reported: operations with
//! their duration, failures, messages) and **Application** (the lines of `gitpoulpi-log`, the same data
//! the terminal UI reads), filtered by level and scope, with copy, the log folder and clear.
//!
//! The Git tab is live: the link host appends to its feed and wakes the window, the view re-reads it.
//! The Application tab is read from the log files off the main thread, when it is opened, refreshed or
//! changed scope, and while it stays open at most once a second.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use gitpoulpi_i18n::msg;
use gitpoulpi_i18n::msg::activity_log as m;
use gitpoulpi_log::{Level, Record};
use gpui_kit::{
    AnyWindowHandle, App, AppContext as _, ClipboardItem, Context, Entity, FocusHandle, Global,
    Task, UniformListScrollHandle, WeakEntity, Window,
};

use super::log_model::{
    app_records, entry_line, git_entries, next_problem, record_line, Levels, LogTab, Scope,
};
use crate::commands::status;
use crate::daemon::activity::{clear_app_log, FeedEntry, APP_RECORDS};
use crate::daemon::Shared;
use crate::dialogs::{self, Choice, Spec};

/// The open Activity Log, for the commands.
pub(super) struct OpenLog(pub WeakEntity<ActivityView>);
impl Global for OpenLog {}

/// How often the Application tab re-reads the files while it is open and the link keeps waking.
const REREAD: Duration = Duration::from_secs(1);

pub struct ActivityView {
    pub(super) backend: Shared,
    window: AnyWindowHandle,
    pub tab: LogTab,
    pub levels: Levels,
    pub scope: Scope,
    /// The Git operations tab's entries under the filters, newest first.
    pub git: Vec<FeedEntry>,
    /// Every application line read, oldest first; `app` is the filtered list, newest first.
    app_all: Vec<Record>,
    pub app: Vec<Record>,
    /// The log files' folder (`None`: the log is kept in memory only).
    pub dir: Option<PathBuf>,
    pub selected: usize,
    /// How far the lines are scrolled sideways, in pixels.
    pub hscroll: f32,
    pub note: Option<String>,
    pub(super) focus: FocusHandle,
    pub(super) scroll: UniformListScrollHandle,
    last_read: Option<Instant>,
    reading: bool,
    read_task: Option<Task<()>>,
    clear_task: Option<Task<()>>,
    _watch: Option<Task<()>>,
}

impl ActivityView {
    pub fn new(
        backend: Shared,
        tab: LogTab,
        scope: Scope,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut this = Self {
            backend: backend.clone(),
            window: window.window_handle(),
            tab,
            levels: Levels::default(),
            scope,
            git: Vec::new(),
            app_all: Vec::new(),
            app: Vec::new(),
            dir: None,
            selected: 0,
            hscroll: 0.0,
            note: None,
            focus: cx.focus_handle(),
            scroll: UniformListScrollHandle::new(),
            last_read: None,
            reading: false,
            read_task: None,
            clear_task: None,
            _watch: None,
        };
        this.refilter();
        this.read_app(cx);
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

    fn root(&self) -> PathBuf {
        self.backend.repo_root()
    }

    fn on_wake(&mut self, cx: &mut Context<Self>) {
        self.refilter();
        let due = self.last_read.is_none_or(|t| t.elapsed() >= REREAD);
        if self.tab == LogTab::App && due {
            self.read_app(cx);
        }
        cx.notify();
    }

    /// Re-reads the Git operations feed and applies the levels and the scope to both tabs.
    pub fn refilter(&mut self) {
        let root = self.root();
        let feed = self.backend.host().feed();
        self.git = git_entries(&feed, &self.levels, self.scope, &root);
        self.app = app_records(&self.app_all, &self.levels, self.scope, &root);
        let len = self.len();
        self.selected = self.selected.min(len.saturating_sub(1));
    }

    /// Reads the application log's files on the background executor.
    pub fn read_app(&mut self, cx: &mut Context<Self>) {
        if self.reading {
            return;
        }
        self.reading = true;
        self.last_read = Some(Instant::now());
        self.read_task = Some(cx.spawn(async move |this, cx| {
            let (records, dir) = cx
                .background_spawn(async move { crate::daemon::activity::app_records(APP_RECORDS) })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.reading = false;
                this.app_all = records;
                this.dir = dir;
                this.refilter();
                cx.notify();
            });
        }));
    }

    /// How many lines the open tab lists.
    pub fn len(&self) -> usize {
        match self.tab {
            LogTab::Git => self.git.len(),
            LogTab::App => self.app.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The text of line `ix` of the open tab.
    pub fn line(&self, ix: usize) -> String {
        match self.tab {
            LogTab::Git => self.git.get(ix).map(entry_line).unwrap_or_default(),
            LogTab::App => self.app.get(ix).map(record_line).unwrap_or_default(),
        }
    }

    /// The level of line `ix`.
    pub fn level_of(&self, ix: usize) -> Level {
        match self.tab {
            LogTab::Git => self.git.get(ix).map_or(Level::Info, |e| e.level),
            LogTab::App => self.app.get(ix).map_or(Level::Info, |r| r.level),
        }
    }

    /// The details under the list: the selected entry's lines.
    pub fn detail_lines(&self) -> Vec<String> {
        match self.tab {
            LogTab::Git => self
                .git
                .get(self.selected)
                .map(|e| {
                    let mut lines = e.details.clone();
                    if let Some(repo) = &e.repo {
                        lines.push(m::ctx_repository(repo.display().to_string()));
                    }
                    lines
                })
                .unwrap_or_default(),
            LogTab::App => self
                .app
                .get(self.selected)
                .map(|r| r.message.lines().map(str::to_string).collect())
                .unwrap_or_default(),
        }
    }

    /// The text a copy of the selected line puts on the clipboard.
    pub fn selected_text(&self) -> Option<String> {
        match self.tab {
            LogTab::Git => self.git.get(self.selected).map(FeedEntry::to_text),
            LogTab::App => self.app.get(self.selected).map(Record::to_line),
        }
    }

    /// Every line shown, as text.
    pub fn all_text(&self) -> String {
        match self.tab {
            LogTab::Git => self
                .git
                .iter()
                .map(FeedEntry::to_text)
                .collect::<Vec<_>>()
                .join("\n\n"),
            LogTab::App => self
                .app
                .iter()
                .rev()
                .map(Record::to_line)
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }

    // ---- what the controls do ------------------------------------------------------------------

    pub fn set_tab(&mut self, tab: LogTab, cx: &mut Context<Self>) {
        if self.tab != tab {
            self.tab = tab;
            self.selected = 0;
            if tab == LogTab::App {
                self.read_app(cx);
            }
            self.refilter();
            cx.notify();
        }
    }

    pub fn toggle_level(&mut self, level: Level, cx: &mut Context<Self>) {
        self.levels.toggle(level);
        self.refilter();
        cx.notify();
    }

    pub fn set_levels(&mut self, levels: Levels, cx: &mut Context<Self>) {
        self.levels = levels;
        self.refilter();
        cx.notify();
    }

    pub fn cycle_scope(&mut self, cx: &mut Context<Self>) {
        self.scope = match self.scope {
            Scope::ThisRepo => Scope::All,
            Scope::All => Scope::ThisRepo,
        };
        self.refilter();
        cx.notify();
    }

    pub fn select(&mut self, ix: usize, cx: &mut Context<Self>) {
        self.selected = ix.min(self.len().saturating_sub(1));
        self.scroll
            .scroll_to_item(self.selected, gpui_kit::ScrollStrategy::Nearest);
        cx.notify();
    }

    pub fn move_selection(&mut self, delta: isize, cx: &mut Context<Self>) {
        let at = self.selected.saturating_add_signed(delta);
        self.select(at, cx);
    }

    pub fn scroll_by(&mut self, dx: f32, cx: &mut Context<Self>) {
        self.hscroll = (self.hscroll + dx).max(0.0);
        cx.notify();
    }

    /// Selects the next (or previous) error or warning of the open tab.
    pub fn step_problem(&mut self, forward: bool, cx: &mut Context<Self>) {
        let levels: Vec<Level> = (0..self.len()).map(|i| self.level_of(i)).collect();
        // The lists run newest first, so "next" (further down) goes to older lines.
        match next_problem(&levels, Some(self.selected), forward) {
            Some(i) => self.select(i, cx),
            None => {
                self.note = Some(m::GIT_EMPTY.get().to_string());
                cx.notify();
            }
        }
    }

    /// From a line of the Application tab: the Git operations entry of the operation it belongs to (the
    /// failed entry nearest in time).
    pub fn show_operation(&mut self, cx: &mut Context<Self>) {
        if self.tab != LogTab::App {
            return;
        }
        let Some(ts) = self.app.get(self.selected).map(|r| r.ts_ms) else {
            self.note = Some(m::APP_SELECT_FIRST.get().to_string());
            cx.notify();
            return;
        };
        let best = self
            .git
            .iter()
            .enumerate()
            .filter(|(_, e)| e.is_problem())
            .min_by_key(|(_, e)| (e.ts_ms - ts).abs())
            .map(|(i, _)| i);
        match best {
            Some(i) => {
                self.set_tab(LogTab::Git, cx);
                self.select(i, cx);
            }
            None => {
                self.note = Some(m::APP_NO_OPERATION.get().to_string());
                cx.notify();
            }
        }
    }

    fn copy(&mut self, what: &str, text: String, cx: &mut Context<Self>) {
        let preview: String = text
            .chars()
            .map(|c| if c.is_control() { ' ' } else { c })
            .take(40)
            .collect();
        cx.write_to_clipboard(ClipboardItem::new_string(text));
        let said = msg::input::copied(what, preview.trim().to_string());
        status::set(cx, said.clone());
        self.note = Some(said);
        cx.notify();
    }

    /// Copies the selected entry.
    pub fn copy_selected(&mut self, cx: &mut Context<Self>) {
        if let Some(text) = self.selected_text() {
            let what = match self.tab {
                LogTab::Git => m::WHAT_ERROR.get(),
                LogTab::App => m::WHAT_LOG.get(),
            };
            self.copy(what, text, cx);
        }
    }

    /// Copies every line shown (what the filters let through).
    pub fn copy_all(&mut self, cx: &mut Context<Self>) {
        let text = self.all_text();
        if !text.is_empty() {
            self.copy(m::WHAT_LOG.get(), text, cx);
        }
    }

    /// Shows the log files' folder in the system's file manager.
    pub fn open_folder(&mut self, cx: &mut Context<Self>) {
        match self.dir.clone() {
            Some(dir) => cx.reveal_path(&dir),
            None => {
                self.note = Some(m::MEMORY_ONLY.get().to_string());
                cx.notify();
            }
        }
    }

    /// Asks before emptying the application log; the answer is never "clear" by default.
    pub fn ask_clear(&mut self, cx: &mut Context<Self>) {
        let me = cx.weak_entity();
        let window = self.window;
        let spec = Spec {
            title: m::CLEAR_TITLE.get().to_string(),
            lines: vec![m::CLEAR_QUESTION.get().to_string()],
            choices: vec![
                Choice::destructive(m::CLEAR_CONFIRM.get()),
                Choice::new(m::CLEAR_CANCEL.get()),
            ],
            hint: m::CLEAR_HINT.get().to_string(),
            highlight_first: false,
            yes_no_keys: true,
        };
        cx.defer(move |cx| {
            let _ = window.update(cx, move |_, window, cx| {
                dialogs::open(
                    window,
                    cx,
                    spec,
                    move |ix, cx| {
                        if ix == 0 {
                            let _ = me.update(cx, |this, cx| this.clear_now(cx));
                        }
                    },
                    || {},
                );
            });
        });
    }

    fn clear_now(&mut self, cx: &mut Context<Self>) {
        self.clear_task = Some(cx.spawn(async move |this, cx| {
            cx.background_spawn(async { clear_app_log() }).await;
            let _ = this.update(cx, |this, cx| {
                this.app_all.clear();
                this.refilter();
                let said = m::CLEAR_DONE.get().to_string();
                status::set(cx, said.clone());
                this.note = Some(said);
                this.read_app(cx);
                cx.notify();
            });
        }));
    }
}

/// The open Activity Log, if any.
pub fn current(cx: &App) -> Option<Entity<ActivityView>> {
    cx.try_global::<OpenLog>().and_then(|o| o.0.upgrade())
}
