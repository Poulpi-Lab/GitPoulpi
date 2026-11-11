//! The Activity Log window: every message and error of the session, and the
//! program's own log.
//!
//! Two tabs and a **Scope** box under them that applies to both:
//!
//! - **Git operations**: the entries of every window (operations with how long
//!   they took (`timing`), messages, failures, clones, workspace tasks), newest
//!   first, each one expandable (the command that ran, its exit code and
//!   stderr) and copyable. "Copy diagnostics" builds the block to paste to
//!   someone (or an AI) who has to work out what went wrong.
//! - **Application**: the log lines of the window and of the daemon (both write
//!   the same day file, `gitpoulpi-log`), newest first, with a level box, a
//!   filter, "Copy all (filtered)", "Open log folder" and a horizontal
//!   scrollbar (a long line is never cut).
//!
//! The Scope box (a combobox: a click opens the list, it never cycles) is
//! "This repository (name)" (not offered from Home, which has none), "All
//! repositories", each workspace and each repository the window knows. It
//! opens on "This repository" from a repository tab and on "All repositories"
//! from Home; the last explicit choice is remembered per entry point for the
//! session. Application lines belong to a repository when their event
//! carried a `repo` field (`Record::repo`); an entry or a line that belongs to
//! none (startup, config, a failed save) is **app-wide**: it shows under "All
//! repositories" and, marked `[app-wide]`, under every narrower scope, so a
//! failure is never hidden by a filter.
//!
//! Modal while open: `Esc` or a click outside closes it, nothing behind it
//! hovers. Process-wide state, like the other small popups, because the
//! window restarts on every tab switch. The status bar's Activity button (a
//! badge counts the errors not looked at) and `View: Activity Log` open it.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::Frame;

use gitpoulpi_i18n::msg::activity_log as msg;
use gitpoulpi_log::{Level, Record};

use super::feed::{self, Entry, Severity};
use super::report;
use crate::combo::{Combo, ComboOption, ComboOutcome};
use crate::dialog_kit::{self, ButtonState};
use crate::{display_width, th};

/// Records the Application log tab loads (the newest).
const APP_RECORDS: usize = 3000;
/// How often the open Application log tab re-reads the files.
const REFRESH: std::time::Duration = std::time::Duration::from_millis(1000);
/// Log lines "Copy diagnostics" quotes.
const DIAGNOSTIC_LINES: usize = 200;
/// Detail lines an expanded entry shows (Copy has all).
const ENTRY_LINES: usize = 40;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Tab {
    /// The entries: operations, messages, failures, clones, workspace tasks.
    Git,
    /// The application log.
    Application,
}

/// What the window shows both tabs for.
#[derive(Clone, PartialEq, Eq, Debug)]
enum Scope {
    /// The repository of the tab the window was opened from.
    This,
    All,
    /// A workspace, by name: its repositories and its own tasks.
    Workspace(String),
    /// One repository, by root path.
    Repo(std::path::PathBuf),
}

/// What a painted rect does when clicked.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Hit {
    Tab(Tab),
    /// An entry's header row: selects it and opens or closes its details.
    Header(usize),
    CopyEntry(usize),
    /// An entry's remedy button (Retry, Stash and retry, Pull first, Open
    /// Integrations, SSH & credentials): the entry, then which of its buttons.
    Remedy(usize, usize),
    CopyDiagnostics,
    CopyAll,
    OpenFolder,
    /// The "Clear log" button (Application tab): opens the confirmation.
    ClearLog,
    /// The confirmation's buttons.
    ClearCancel,
    ClearConfirm,
    /// The confirmation's close cross: like Esc, nothing is cleared.
    ClearClose,
    Filter,
    /// A level's display checkbox (Application tab).
    Toggle(Level),
    /// The "All" button: every level shown.
    ShowAll,
    /// The "None" button: no level shown.
    ShowNone,
}

/// What the display toggles can be asked to do (keys, mouse and palette).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum LevelCmd {
    Toggle(Level),
    All,
    Clear,
    /// Opens the confirmation that clears the application log files.
    ClearLog,
    /// Opens the Scope box's list.
    Scope,
    /// Shows the Git operations tab.
    GitTab,
    /// Shows the Application tab.
    AppTab,
    /// Scrolls the lines of the tab in front one step to the left.
    ScrollLeft,
    /// Scrolls the lines of the tab in front one step to the right.
    ScrollRight,
}

struct State {
    tab: Tab,
    /// The "clear the application log" confirmation is open: it owns the
    /// pointer and the keyboard. `clear_sel` is 0 Cancel / 1 Clear.
    clear_open: bool,
    clear_sel: usize,
    /// Index in the newest-first entry list.
    selected: usize,
    /// Entry ids whose details are open.
    expanded: Vec<u64>,
    /// First visual line shown, per tab (`tab_index`).
    offset: [usize; 2],
    /// First column shown (both tabs; a tab switch goes back to 0).
    hoffset: usize,
    /// Where the horizontal thumb is held while it is dragged.
    hgrab: Option<i32>,
    /// The Application list's outer rect (the horizontal bar lives on its
    /// bottom border row), and the longest line.
    hbody: Rect,
    longest: usize,
    /// The repository whose operations the Repository tab lists.
    active_repo: Option<std::path::PathBuf>,
    /// Keep the selection in view at the next paint (a key moved it).
    follow: bool,
    grab: Option<i32>,
    level: Option<Combo>,
    view_level: Level,
    /// Which levels the Application list shows (index = `Level as usize`).
    /// Independent of the capture level: it only hides lines.
    shown: [bool; 5],
    filter: String,
    filter_focus: bool,
    notice: Option<String>,
    pointer: Option<(u16, u16)>,
    window: Rect,
    close: Rect,
    list: Rect,
    body: Rect,
    lines: usize,
    hits: Vec<(Rect, Hit)>,
    status_button: Rect,
    /// The window in front is the Home tab (no repository): the Activity
    /// Log opens on "All repositories", and has no "This repository".
    home: bool,
    /// Where the level box was painted (a click on it opens the list).
    level_field: Rect,
    /// The Scope box, where it was painted, and what it is set to.
    scope_combo: Option<Combo>,
    scope_field: Rect,
    scope: Scope,
    /// The last explicit choice per entry point (`[repository tab, Home]`),
    /// for the session.
    remembered: [Option<Scope>; 2],
    /// The workspaces and their repositories, read when the window opens (a
    /// file read on the user's action, never on a frame).
    ws_cache: Vec<(String, Vec<std::path::PathBuf>)>,
}

static STATE: Mutex<State> = Mutex::new(State {
    tab: Tab::Git,
    clear_open: false,
    clear_sel: 0,
    selected: 0,
    expanded: Vec::new(),
    offset: [0, 0],
    hoffset: 0,
    hgrab: None,
    hbody: Rect::new(0, 0, 0, 0),
    longest: 0,
    active_repo: None,
    follow: false,
    grab: None,
    level: None,
    view_level: Level::Info,
    shown: [true; 5],
    filter: String::new(),
    filter_focus: false,
    notice: None,
    pointer: None,
    window: Rect::new(0, 0, 0, 0),
    close: Rect::new(0, 0, 0, 0),
    list: Rect::new(0, 0, 0, 0),
    body: Rect::new(0, 0, 0, 0),
    lines: 0,
    hits: Vec::new(),
    status_button: Rect::new(0, 0, 0, 0),
    home: false,
    level_field: Rect::new(0, 0, 0, 0),
    scope_combo: None,
    scope_field: Rect::new(0, 0, 0, 0),
    scope: Scope::All,
    remembered: [None, None],
    ws_cache: Vec::new(),
});

static OPEN: AtomicBool = AtomicBool::new(false);

/// The Application log tab's records, read off the render thread.
struct AppLog {
    records: Vec<Record>,
}

static APP: Mutex<AppLog> = Mutex::new(AppLog {
    records: Vec::new(),
});
static REFRESHING: AtomicBool = AtomicBool::new(false);

/// Tests: the refresher leaves `APP` alone, so a test can show exactly the
/// records it put there (every test of the process writes to the one log).
#[cfg(test)]
pub(super) static FROZEN: AtomicBool = AtomicBool::new(false);

#[cfg(test)]
pub(super) fn show_only(records: Vec<Record>) {
    FROZEN.store(true, Ordering::SeqCst);
    APP.lock().unwrap_or_else(|e| e.into_inner()).records = records;
}

#[cfg(test)]
pub(super) fn unfreeze() {
    FROZEN.store(false, Ordering::SeqCst);
}

fn frozen() -> bool {
    #[cfg(test)]
    return FROZEN.load(Ordering::SeqCst);
    #[cfg(not(test))]
    false
}

fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    f(&mut STATE.lock().unwrap_or_else(|e| e.into_inner()))
}

pub(crate) fn is_open() -> bool {
    OPEN.load(Ordering::Relaxed)
}

/// Opens the window on the Git operations tab, scoped to where it was
/// opened from: the repository in front, or every repository from Home (the
/// last explicit choice for that entry point wins).
pub(crate) fn open() {
    let scope = with(|s| {
        let slot = usize::from(s.home);
        s.remembered[slot]
            .clone()
            .filter(|r| !(s.home && *r == Scope::This))
            .unwrap_or(if s.home { Scope::All } else { Scope::This })
    });
    open_with(scope);
}

fn open_with(scope: Scope) {
    // Bars painted behind the window must not light under a stale pointer.
    crate::scrollbar::set_pointer(None);
    feed::mark_all_read();
    report::warm_up();
    refresh_workspaces();
    let recording = gitpoulpi_log::global().map_or(Level::Info, |l| l.level());
    with(|s| {
        s.tab = Tab::Git;
        s.scope = scope;
        s.selected = 0;
        s.offset = [0, 0];
        s.hoffset = 0;
        s.scope_combo = None;
        s.hgrab = None;
        s.view_level = recording;
        s.level = None;
        s.filter_focus = false;
        s.notice = None;
        s.grab = None;
    });
    OPEN.store(true, Ordering::Relaxed);
    crate::redraw::poke();
}

pub(crate) fn close() {
    OPEN.store(false, Ordering::Relaxed);
    with(|s| {
        s.level = None;
        s.scope_combo = None;
        s.clear_open = false;
        s.filter_focus = false;
        s.grab = None;
        s.hgrab = None;
        s.pointer = None;
    });
    crate::redraw::poke();
}

fn switch_tab(tab: Tab) {
    with(|s| {
        s.tab = tab;
        s.grab = None;
        s.hgrab = None;
        s.filter_focus = false;
        s.level = None;
        s.scope_combo = None;
        s.notice = None;
        s.selected = 0;
        s.hoffset = 0;
    });
    if tab == Tab::Application {
        start_refresher();
    }
}

/// Reads the workspaces for the Scope box and the workspace scope.
fn refresh_workspaces() {
    let cache = crate::workspace_ui::store::load()
        .workspaces
        .into_iter()
        .map(|w| (w.name, w.repos))
        .collect();
    with(|s| s.ws_cache = cache);
}

/// The palette's and the Home button's entry: the window on "All
/// repositories" (an explicit choice for this opening only).
pub(crate) fn open_global() {
    open_with(Scope::All);
}

/// A failure card's Details for something that is not in the tab in front:
/// "All repositories", with that entry selected and open.
pub(super) fn open_global_on(id: u64) {
    open_global();
    let index = entries().iter().position(|e| e.id == id);
    with(|s| {
        if let Some(i) = index {
            s.selected = i;
            s.follow = true;
        }
        if !s.expanded.contains(&id) {
            s.expanded.push(id);
        }
    });
}

/// Whether a card for `entry` should open the Global tab instead of
/// expanding in place: it happened somewhere the tab in front is not.
pub(super) fn belongs_elsewhere(entry: &Entry) -> bool {
    let active = with(|s| s.active_repo.clone());
    match (&entry.repo, active) {
        (Some(of), Some(active)) => !gitpoulpi_session::same_repo_path(of, &active),
        _ => false,
    }
}

fn level_value(level: Level) -> &'static str {
    match level {
        Level::Error => "error",
        Level::Warn => "warn",
        Level::Info => "info",
        Level::Debug => "debug",
        Level::Trace => "trace",
    }
}

fn level_label(level: Level) -> &'static str {
    match level {
        Level::Error => msg::LEVEL_ERROR,
        Level::Warn => msg::LEVEL_WARN,
        Level::Info => msg::LEVEL_INFO,
        Level::Debug => msg::LEVEL_DEBUG,
        Level::Trace => msg::LEVEL_TRACE,
    }
    .get()
}

fn level_combo(current: Level) -> Combo {
    let mut combo = Combo::new("").with_click_picks();
    combo.set_options(
        Level::ALL
            .iter()
            .map(|l| ComboOption::new(level_label(*l), level_value(*l)))
            .collect(),
    );
    combo.select(Some(level_value(current).to_string()));
    combo
}

/// Reads the log files (or the in-memory tail) on a thread of its own while
/// the Application log tab is open, so painting never waits on the disk.
fn start_refresher() {
    if REFRESHING.swap(true, Ordering::SeqCst) {
        return;
    }
    let spawned = std::thread::Builder::new()
        .name("log-view".into())
        .spawn(|| {
            while is_open() && with(|s| s.tab) == Tab::Application {
                let logger = gitpoulpi_log::global();
                let records = match &logger {
                    Some(l) => {
                        l.flush();
                        match l.dir() {
                            Some(dir) => gitpoulpi_log::read_records(dir, APP_RECORDS),
                            None => l.recent(),
                        }
                    }
                    None => Vec::new(),
                };
                if !frozen() {
                    APP.lock().unwrap_or_else(|e| e.into_inner()).records = records;
                }
                crate::redraw::poke();
                std::thread::sleep(REFRESH);
            }
            REFRESHING.store(false, Ordering::SeqCst);
        });
    if spawned.is_err() {
        REFRESHING.store(false, Ordering::SeqCst);
    }
}

/// The repository "This repository" means (the tab in front).
pub(super) fn set_active_repo(root: &std::path::Path) {
    with(|s| {
        if s.active_repo.as_deref() != Some(root) {
            s.active_repo = Some(root.to_path_buf());
        }
    });
}

/// How a scope sees something.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Belongs {
    No,
    Yes,
    /// It belongs to no repository: shown, and marked, under a narrow scope.
    AppWide,
}

/// The scope with what it needs to judge (read once per paint or query).
#[derive(Clone)]
struct Scoping {
    scope: Scope,
    active: Option<std::path::PathBuf>,
    cache: Vec<(String, Vec<std::path::PathBuf>)>,
}

fn scoping() -> Scoping {
    with(|s| Scoping {
        scope: s.scope.clone(),
        active: s.active_repo.clone(),
        cache: s.ws_cache.clone(),
    })
}

impl Scoping {
    /// Whether the scope shows something of `repo` / `workspace` (an entry's
    /// or a log line's owner; both `None` is app-wide).
    fn judge(&self, repo: Option<&std::path::Path>, workspace: Option<&str>) -> Belongs {
        if self.scope == Scope::All {
            return Belongs::Yes;
        }
        if repo.is_none() && workspace.is_none() {
            return Belongs::AppWide;
        }
        let same =
            |a: &std::path::Path, b: &std::path::Path| gitpoulpi_session::same_repo_path(a, b);
        let yes = match &self.scope {
            Scope::All => true,
            Scope::This => match (repo, &self.active) {
                (Some(r), Some(active)) => same(r, active),
                _ => false,
            },
            Scope::Repo(path) => repo.is_some_and(|r| same(r, path)),
            Scope::Workspace(name) => {
                workspace == Some(name.as_str())
                    || repo.is_some_and(|r| {
                        self.cache
                            .iter()
                            .any(|(n, repos)| n == name && repos.iter().any(|p| same(p, r)))
                    })
            }
        };
        if yes {
            Belongs::Yes
        } else {
            Belongs::No
        }
    }

    /// Whether rows of several repositories can show together (so each says
    /// whose it is).
    fn many(&self) -> bool {
        matches!(self.scope, Scope::All | Scope::Workspace(_))
    }
}

/// The Git operations tab's entries, newest first, for the scope.
fn entries() -> Vec<Entry> {
    let sc = scoping();
    feed::snapshot()
        .into_iter()
        .filter(|e| sc.judge(e.repo.as_deref(), e.workspace.as_deref()) != Belongs::No)
        .collect()
}

/// The last component of a path, for a repository's short name.
fn short_name(path: &std::path::Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    )
}

/// The Scope box's text for `scope`.
fn scope_label(scope: &Scope, active: Option<&std::path::Path>) -> String {
    match scope {
        Scope::This => msg::scope_this(active.map_or_else(|| "-".to_string(), short_name)),
        Scope::All => msg::SCOPE_ALL.get().to_string(),
        Scope::Workspace(name) => msg::scope_workspace(name.clone()),
        Scope::Repo(path) => short_name(path),
    }
}

fn scope_value(scope: &Scope) -> String {
    match scope {
        Scope::This => "this".into(),
        Scope::All => "all".into(),
        Scope::Workspace(name) => format!("ws:{name}"),
        Scope::Repo(path) => format!("repo:{}", path.display()),
    }
}

fn scope_of_value(value: &str) -> Option<Scope> {
    match value {
        "this" => Some(Scope::This),
        "all" => Some(Scope::All),
        _ => value
            .strip_prefix("ws:")
            .map(|n| Scope::Workspace(n.to_string()))
            .or_else(|| {
                value
                    .strip_prefix("repo:")
                    .map(|p| Scope::Repo(std::path::PathBuf::from(p)))
            }),
    }
}

/// The Scope box's rows: this repository (not from Home), all repositories,
/// each workspace, then each repository the feed, the log and the workspaces
/// know (newest first, the current one only as "This repository").
fn scope_options() -> Vec<ComboOption> {
    let (home, active, cache, current) = with(|s| {
        (
            s.home,
            s.active_repo.clone(),
            s.ws_cache.clone(),
            s.scope.clone(),
        )
    });
    let this = !home && active.is_some();
    let mut out = Vec::new();
    if this {
        out.push(ComboOption::new(
            scope_label(&Scope::This, active.as_deref()),
            scope_value(&Scope::This),
        ));
    }
    out.push(ComboOption::new(
        scope_label(&Scope::All, None),
        scope_value(&Scope::All),
    ));
    for (name, _) in &cache {
        let scope = Scope::Workspace(name.clone());
        out.push(ComboOption::new(
            scope_label(&scope, None),
            scope_value(&scope),
        ));
    }
    let mut repos: Vec<std::path::PathBuf> = Vec::new();
    let mut add = |p: &std::path::Path| {
        let own = this
            && active
                .as_deref()
                .is_some_and(|a| gitpoulpi_session::same_repo_path(a, p));
        if !own
            && !repos
                .iter()
                .any(|r| gitpoulpi_session::same_repo_path(r, p))
        {
            repos.push(p.to_path_buf());
        }
    };
    for e in feed::snapshot() {
        if let Some(r) = &e.repo {
            add(r);
        }
    }
    for (_, list) in &cache {
        for p in list {
            add(p);
        }
    }
    for r in APP
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .records
        .iter()
        .rev()
    {
        if let Some(repo) = &r.repo {
            add(std::path::Path::new(repo));
        }
    }
    if let Scope::Repo(p) = &current {
        add(p);
    }
    // Two repositories with one folder name show their full paths.
    let names: Vec<String> = repos.iter().map(|p| short_name(p)).collect();
    for (i, p) in repos.iter().enumerate() {
        let clash = names.iter().filter(|n| **n == names[i]).count() > 1;
        let label = if clash {
            p.display().to_string()
        } else {
            names[i].clone()
        };
        out.push(ComboOption::new(
            label,
            scope_value(&Scope::Repo(p.clone())),
        ));
    }
    out
}

/// The Scope box as painted when closed: one row, the current scope.
fn scope_combo_closed() -> Combo {
    let (scope, active) = with(|s| (s.scope.clone(), s.active_repo.clone()));
    let mut combo = Combo::new("").with_click_picks();
    combo.set_options(vec![ComboOption::new(
        scope_label(&scope, active.as_deref()),
        scope_value(&scope),
    )]);
    combo.select(Some(scope_value(&scope)));
    combo
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Box3 {
    Level,
    Scope,
}

fn box_slot(s: &mut State, which: Box3) -> &mut Option<Combo> {
    match which {
        Box3::Level => &mut s.level,
        Box3::Scope => &mut s.scope_combo,
    }
}

/// Whether a box is reachable on the tab.
fn box_on_tab(which: Box3, tab: Tab) -> bool {
    which == Box3::Scope || tab == Tab::Application
}

/// A box picked `value`.
fn box_picked(which: Box3, value: &str) {
    match which {
        Box3::Level => {
            if let Some(level) = Level::parse(value) {
                choose_level(level);
            }
        }
        Box3::Scope => {
            if let Some(scope) = scope_of_value(value) {
                choose_scope(scope);
            }
        }
    }
}

/// An explicit choice: it applies to both tabs and is what this entry point
/// opens on next time.
fn choose_scope(scope: Scope) {
    with(|s| {
        let slot = usize::from(s.home);
        s.remembered[slot] = Some(scope.clone());
        s.scope = scope;
        s.selected = 0;
        s.offset = [0, 0];
    });
}

/// Opens the Scope box's list, as `s` and the palette do.
fn open_scope_box() {
    refresh_workspaces();
    let current = with(|s| scope_value(&s.scope));
    let mut combo = Combo::new("").with_click_picks();
    let options = scope_options();
    // A scope that is not listed (a repository that left) still reads.
    let listed = options.iter().any(|o| o.value == current);
    combo.set_options(options);
    if listed {
        combo.select(Some(current));
    }
    combo.open();
    with(|s| s.scope_combo = Some(combo));
}

/// Whether the lowercase `needle` (empty matches all) is in the record.
fn text_matches(r: &Record, needle: &str) -> bool {
    needle.is_empty()
        || r.message.to_lowercase().contains(needle)
        || r.target.to_lowercase().contains(needle)
        || r.process.to_lowercase().contains(needle)
}

/// Whether the list shows `r`: at or above the capture `level`, its level
/// ticked in `shown`, and matching the filter.
fn is_shown(r: &Record, level: Level, shown: &[bool; 5], needle: &str) -> bool {
    r.level <= level && shown[r.level as usize] && text_matches(r, needle)
}

/// Whether the scope shows the log line.
fn in_scope(sc: &Scoping, r: &Record) -> bool {
    sc.judge(r.repo.as_deref().map(std::path::Path::new), None) != Belongs::No
}

/// The records the tab shows, newest first.
fn visible_records(sc: &Scoping, level: Level, shown: &[bool; 5], filter: &str) -> Vec<Record> {
    let needle = filter.trim().to_lowercase();
    let app = APP.lock().unwrap_or_else(|e| e.into_inner());
    app.records
        .iter()
        .rev()
        .filter(|r| is_shown(r, level, shown, &needle) && in_scope(sc, r))
        .cloned()
        .collect()
}

/// Records per level at or above `level` that match the filter and the scope
/// (the counts next to the checkboxes, whether ticked or not).
fn level_counts(sc: &Scoping, level: Level, filter: &str) -> [usize; 5] {
    let needle = filter.trim().to_lowercase();
    let app = APP.lock().unwrap_or_else(|e| e.into_inner());
    let mut counts = [0; 5];
    for r in app.records.iter().filter(|r| r.level <= level) {
        if text_matches(r, &needle) && in_scope(sc, r) {
            counts[r.level as usize] += 1;
        }
    }
    counts
}

/// A display toggle: a level above the capture level has nothing to show,
/// so it only explains itself.
pub(super) fn level_cmd(cmd: LevelCmd) {
    with(|s| match cmd {
        LevelCmd::Toggle(level) if level > s.view_level => {
            s.notice = Some(msg::why_level_off(level_label(s.view_level)));
        }
        LevelCmd::Toggle(level) => s.shown[level as usize] = !s.shown[level as usize],
        LevelCmd::All => s.shown = [true; 5],
        LevelCmd::Clear => s.shown = [false; 5],
        LevelCmd::ClearLog => ask_clear_log_in(s),
        LevelCmd::Scope
        | LevelCmd::GitTab
        | LevelCmd::AppTab
        | LevelCmd::ScrollLeft
        | LevelCmd::ScrollRight => {}
    });
    match cmd {
        LevelCmd::Scope => open_scope_box(),
        LevelCmd::GitTab => switch_tab(Tab::Git),
        LevelCmd::AppTab => switch_tab(Tab::Application),
        LevelCmd::ScrollLeft => hscroll(-H_STEP),
        LevelCmd::ScrollRight => hscroll(H_STEP),
        _ => {}
    }
}

/// The palette's entry: opens the window, on the Application tab for the
/// level commands (the Git operations tab or the Scope box for theirs), and
/// applies.
pub(crate) fn level_cmd_open(cmd: LevelCmd) {
    // Scrolling keeps the window as it is (reopening goes back to the left
    // edge and the first entry).
    if !(matches!(cmd, LevelCmd::ScrollLeft | LevelCmd::ScrollRight) && is_open()) {
        open();
    }
    if !matches!(
        cmd,
        LevelCmd::Scope | LevelCmd::GitTab | LevelCmd::ScrollLeft | LevelCmd::ScrollRight
    ) {
        with(|s| s.tab = Tab::Application);
        start_refresher();
    }
    level_cmd(cmd);
}

/// What "Copy diagnostics" quotes: the tail at the chosen level.
fn diagnostics_text() -> String {
    let level = with(|s| s.view_level);
    // What the Application log tab last read from the files (the daemon's
    // lines too), plus this process's own newer lines from memory.
    let mut records: Vec<Record> = APP
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .records
        .clone();
    let last = records.last().map_or(i64::MIN, |r| r.ts_ms);
    if let Some(logger) = gitpoulpi_log::global() {
        records.extend(logger.recent().into_iter().filter(|r| r.ts_ms > last));
    }
    records.retain(|r| r.level <= level);
    let skip = records.len().saturating_sub(DIAGNOSTIC_LINES);
    let records = records.split_off(skip);
    let latest = feed::snapshot()
        .into_iter()
        .find(|e| e.severity == Severity::Error);
    report::diagnostics(latest.as_ref(), &records, level)
}

/// Copies the diagnostics block; the sentence to show says whether it worked.
pub(crate) fn copy_diagnostics_now() -> String {
    let text = diagnostics_text();
    crate::clipboard::copy_shared(msg::WHAT_DIAGNOSTICS.get(), &text)
}

fn copy_diagnostics() {
    let note = copy_diagnostics_now();
    with(|s| s.notice = Some(note));
}

fn copy_entry(index: usize) {
    if let Some(entry) = entries().get(index) {
        let note =
            crate::clipboard::copy_shared(msg::WHAT_ERROR.get(), &report::entry_block(entry));
        with(|s| s.notice = Some(note));
    }
}

/// What the window is asked to do by a remedy button, until it takes it.
static COMMAND: Mutex<Option<super::Command>> = Mutex::new(None);

/// The remedy the window has not carried out yet (a button or its key).
pub(crate) fn take_command() -> Option<super::Command> {
    COMMAND.lock().unwrap_or_else(|e| e.into_inner()).take()
}

/// The button that does something about the failure `entry` records. An
/// action runs on the link in front, so an entry of another repository only
/// offers what does not depend on it (Open Integrations).
pub(super) fn remedies_of(entry: &Entry) -> Vec<super::friendly::Remedy> {
    use super::friendly::Remedy;
    let elsewhere = belongs_elsewhere(entry);
    super::friendly::remedies(entry.kind, entry.retry.as_ref(), entry.auth)
        .into_iter()
        .filter(|r| matches!(r, Remedy::OpenIntegrations) || !elsewhere)
        .collect()
}

/// The first (main) button of the entry.
pub(super) fn remedy_of(entry: &Entry) -> Option<super::friendly::Remedy> {
    remedies_of(entry).into_iter().next()
}

fn command_of(remedy: super::friendly::Remedy) -> super::Command {
    use super::friendly::Remedy;
    use super::Command;
    use crate::daemon_link::Action;
    match remedy {
        Remedy::PullFirst => {
            Command::Submit(Action::Pull(gitpoulpi_core::domain::PullOptions::default()))
        }
        Remedy::OpenIntegrations => Command::OpenIntegrations,
        Remedy::OpenRepoAuth => Command::OpenRepoAuth,
        Remedy::StashAndRetry(action) => Command::Submit(Action::AutoStashed(Box::new(action))),
        Remedy::Retry(action) => Command::Submit(action),
    }
}

/// The remedy of the failure the status line shows (the palette's twin).
pub(crate) fn last_error_command() -> Option<super::Command> {
    let entry = super::status_error::shown_entry()?;
    remedy_of(&entry).map(command_of)
}

pub(crate) fn last_error_has_remedy() -> bool {
    super::status_error::shown_entry().is_some_and(|e| remedy_of(&e).is_some())
}

/// Carries out entry `index`'s remedy: the window closes (Settings or the
/// running action show behind it) and is handed the command.
fn run_remedy(index: usize, nth: usize) {
    let Some(command) = entries()
        .get(index)
        .and_then(|e| remedies_of(e).into_iter().nth(nth))
        .map(command_of)
    else {
        return;
    };
    *COMMAND.lock().unwrap_or_else(|e| e.into_inner()) = Some(command);
    close();
}

fn copy_all() {
    let (level, shown, filter) = with(|s| (s.view_level, s.shown, s.filter.clone()));
    let mut records = visible_records(&scoping(), level, &shown, &filter);
    records.reverse();
    let note = crate::clipboard::copy_shared(
        msg::WHAT_LOG.get(),
        &gitpoulpi_core::redact(&gitpoulpi_log::to_text(&records)),
    );
    with(|s| s.notice = Some(note));
}

/// Opens the confirmation (never clears by itself: deleting the log is not
/// undoable, and it is not journaled in the undo history either).
fn ask_clear_log_in(s: &mut State) {
    s.clear_open = true;
    s.clear_sel = 0;
    s.filter_focus = false;
}

/// Empties the application log on a thread of its own (a day's file can be
/// large, and the writer has to answer first): the tail, every day's file
/// (see `gitpoulpi_log::clear_dir`), then one info line says so.
fn clear_log() {
    with(|s| s.clear_open = false);
    // What is on screen goes now, the files follow.
    APP.lock()
        .unwrap_or_else(|e| e.into_inner())
        .records
        .clear();
    let spawned = std::thread::Builder::new()
        .name("log-clear".into())
        .spawn(|| {
            if let Some(logger) = gitpoulpi_log::global() {
                logger.clear();
            }
            tracing::info!(target: "tui", "application log cleared");
            APP.lock()
                .unwrap_or_else(|e| e.into_inner())
                .records
                .clear();
            with(|s| s.notice = Some(msg::CLEAR_DONE.get().to_string()));
            crate::redraw::poke();
        });
    if spawned.is_err() {
        with(|s| s.notice = Some(msg::CLEAR_DONE.get().to_string()));
    }
}

/// A key while the confirmation is open.
fn clear_key(key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('q') => with(|s| s.clear_open = false),
        KeyCode::Char('y') => clear_log(),
        KeyCode::Left | KeyCode::Right | KeyCode::Tab | KeyCode::BackTab => {
            with(|s| s.clear_sel = 1 - s.clear_sel);
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            if with(|s| s.clear_sel) == 1 {
                clear_log();
            } else {
                with(|s| s.clear_open = false);
            }
        }
        _ => {}
    }
}

fn open_folder() {
    let dir = gitpoulpi_log::global().and_then(|l| l.dir().map(std::path::Path::to_path_buf));
    let note = match dir {
        None => msg::MEMORY_ONLY.get().to_string(),
        Some(dir) => {
            let _ = std::fs::create_dir_all(&dir);
            match crate::repo_menu::open_folder(&dir) {
                Ok(()) => String::new(),
                Err(why) => msg::folder_failed(why),
            }
        }
    };
    with(|s| s.notice = (!note.is_empty()).then_some(note));
}

/// The level box picked `level`: the view follows, and when the file only
/// records less than that, recording is raised (and saved) so lines appear.
fn choose_level(level: Level) {
    with(|s| s.view_level = level);
    let recording = gitpoulpi_log::global().map(|l| l.level());
    if let Some(recording) = recording.filter(|r| level > *r) {
        let _ = recording;
        gitpoulpi_log::set_level(level);
        tracing::info!(target: "log", "recording level raised to {}", level.as_str());
        persist_level(level);
        with(|s| s.notice = Some(msg::recording_raised(level_label(level))));
    }
}

fn persist_level(level: Level) {
    if let Some(path) = gitpoulpi_config::Settings::resolve_ambient()
        .ok()
        .and_then(|s| s.file().map(std::path::Path::to_path_buf))
    {
        let _ = gitpoulpi_config::set_in_file(&path, "log.level", level_value(level));
    }
}

fn toggle_expanded(id: u64) {
    with(|s| {
        if let Some(i) = s.expanded.iter().position(|x| *x == id) {
            s.expanded.remove(i);
        } else {
            s.expanded.push(id);
        }
    });
}

// ---- Ctrl+N badges ----

/// What `Ctrl+N` badges, from the rects the last paint registered for the
/// mouse: the tabs, the Scope and Level boxes, the filter, the toggles and
/// buttons, and the first entry of the list. Only the confirmation's two
/// buttons while it is up; nothing while a box's list is open (it owns the
/// keys).
pub(crate) fn nav_targets() -> Vec<(Rect, (u16, u16))> {
    with(|s| {
        if [&s.level, &s.scope_combo]
            .iter()
            .any(|c| c.as_ref().is_some_and(Combo::is_open))
        {
            return Vec::new();
        }
        let mut seen_entry = false;
        let mut rects: Vec<Rect> = s
            .hits
            .iter()
            .filter_map(|(r, hit)| match hit {
                Hit::ClearCancel | Hit::ClearConfirm => Some(*r),
                Hit::ClearClose => None,
                _ if s.clear_open => None,
                Hit::Header(_) if !std::mem::replace(&mut seen_entry, true) => Some(*r),
                Hit::Header(_) | Hit::CopyEntry(_) | Hit::Remedy(..) => None,
                _ => Some(*r),
            })
            .collect();
        if !s.clear_open {
            if s.scope_field.width > 0 {
                rects.push(s.scope_field);
            }
            if s.tab == Tab::Application && s.level_field.width > 0 {
                rects.push(s.level_field);
            }
        }
        dialog_kit::nav_cells(rects)
    })
}

/// Whether a click at the point does something: the pointing hand. Every
/// control the window painted (not only the ones `Ctrl+N` badges), the
/// Scope and Level boxes and the cross; the filter is a text field and keeps
/// the arrow. With the window closed: the status bar's button and the status
/// line's error.
pub(crate) fn pointer_hand(col: u16, row: u16) -> bool {
    let at = (col, row).into();
    if !is_open() {
        return status_button_hit(col, row) || super::status_error::hit(col, row);
    }
    with(|s| {
        if s.close.contains(at) {
            return true;
        }
        if s.clear_open {
            return s.hits.iter().any(|(r, h)| {
                matches!(h, Hit::ClearCancel | Hit::ClearConfirm | Hit::ClearClose)
                    && r.contains(at)
            });
        }
        if (s.tab == Tab::Application && s.level_field.contains(at)) || s.scope_field.contains(at) {
            return true;
        }
        s.hits
            .iter()
            .find(|(r, _)| r.contains(at))
            .is_some_and(|(_, h)| !matches!(h, Hit::Filter))
    })
}

/// Whether the point is on the window's filter box (a text field: the
/// I-beam).
pub(crate) fn pointer_text(col: u16, row: u16) -> bool {
    let at = (col, row).into();
    is_open()
        && with(|s| {
            !s.clear_open
                && s.hits
                    .iter()
                    .any(|(r, h)| matches!(h, Hit::Filter) && r.contains(at))
        })
}

/// A pressed badge: a left click at its cell.
pub(crate) fn nav_press((column, row): (u16, u16)) {
    for kind in [
        MouseEventKind::Down(MouseButton::Left),
        MouseEventKind::Up(MouseButton::Left),
    ] {
        on_mouse(MouseEvent {
            kind,
            column,
            row,
            modifiers: KeyModifiers::NONE,
        });
    }
}

// ---- keys ----

/// A key while the window is open. It takes every key.
pub(crate) fn on_key(key: KeyEvent) {
    // The confirmation owns the keyboard.
    if with(|s| s.clear_open) {
        clear_key(key);
        return;
    }
    // A box's list, open, owns the keyboard.
    for which in [Box3::Level, Box3::Scope] {
        if with(|s| box_slot(s, which).as_ref().is_some_and(Combo::is_open)) {
            let outcome = with(|s| box_slot(s, which).as_mut().map(|c| c.on_key(key)));
            if let Some(ComboOutcome::Picked(value)) = outcome {
                box_picked(which, &value);
            }
            return;
        }
    }
    // Typing in the filter.
    if with(|s| s.filter_focus) {
        match key.code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Tab => with(|s| s.filter_focus = false),
            KeyCode::Backspace => with(|s| {
                s.filter.pop();
                s.offset[1] = 0;
            }),
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => with(|s| {
                s.filter.clear();
                s.offset[1] = 0;
            }),
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => with(|s| {
                s.filter.push(c);
                s.offset[1] = 0;
            }),
            _ => {}
        }
        return;
    }
    let tab = with(|s| s.tab);
    let page = with(|s| s.list.height.max(1)) as isize;
    // Sideways scrolling of long lines, on both tabs (rebindable).
    match crate::keymap::current().action_in(crate::keymap::Ctx::ActivityLog, &key) {
        Some(crate::keymap::Action::ActivityLogScrollLeft) => return hscroll(-H_STEP),
        Some(crate::keymap::Action::ActivityLogScrollRight) => return hscroll(H_STEP),
        _ => {}
    }
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => close(),
        KeyCode::Tab | KeyCode::BackTab => switch_tab(other_tab(tab)),
        KeyCode::Char('d') => copy_diagnostics(),
        // The tabs by letter, and the Scope box.
        KeyCode::Char('g') if tab != Tab::Git => switch_tab(Tab::Git),
        KeyCode::Char('p') if tab != Tab::Application => switch_tab(Tab::Application),
        KeyCode::Char('s') => open_scope_box(),
        _ if tab == Tab::Git => activity_key(key, page),
        _ => app_key(key, page),
    }
}

fn other_tab(tab: Tab) -> Tab {
    match tab {
        Tab::Application => Tab::Git,
        Tab::Git => Tab::Application,
    }
}

/// Columns one Left/Right press or one wheel notch scrolls sideways.
const H_STEP: isize = 8;

fn hscroll(delta: isize) {
    with(|s| s.hoffset = s.hoffset.saturating_add_signed(delta));
}

fn activity_key(key: KeyEvent, page: isize) {
    let count = entries().len();
    let step = |delta: isize| {
        with(|s| {
            let last = count.saturating_sub(1) as isize;
            s.selected = (s.selected as isize + delta).clamp(0, last.max(0)) as usize;
            s.follow = true;
        })
    };
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => step(-1),
        KeyCode::Down | KeyCode::Char('j') => step(1),
        KeyCode::PageUp => step(-5),
        KeyCode::PageDown => step(5),
        // Home goes back to the first entry and the left edge.
        KeyCode::Home => {
            with(|s| s.hoffset = 0);
            step(isize::MIN / 2);
        }
        KeyCode::End => step(isize::MAX / 2),
        KeyCode::Enter | KeyCode::Char(' ') => {
            let selected = with(|s| s.selected);
            if let Some(entry) = entries().get(selected) {
                toggle_expanded(entry.id);
            }
        }
        KeyCode::Char('c') => copy_entry(with(|s| s.selected)),
        KeyCode::Char('r') => run_remedy(with(|s| s.selected), 0),
        KeyCode::Char('o') => run_remedy(with(|s| s.selected), 1),
        _ => {
            let _ = page;
        }
    }
}

fn app_key(key: KeyEvent, page: isize) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => scroll(-1),
        KeyCode::Down | KeyCode::Char('j') => scroll(1),
        KeyCode::PageUp => scroll(-page),
        KeyCode::PageDown => scroll(page),
        // Home goes back to the start of the lines: the top and the left edge.
        KeyCode::Home => {
            with(|s| s.hoffset = 0);
            scroll(isize::MIN / 2);
        }
        KeyCode::End => scroll(isize::MAX / 2),
        KeyCode::Char('l') => open_level_list(),
        KeyCode::Char('/') => with(|s| s.filter_focus = true),
        KeyCode::Char('c') => copy_all(),
        KeyCode::Char('o') => open_folder(),
        KeyCode::Char('x') => with(ask_clear_log_in),
        // 1-5 tick the levels Error..Trace, `a` all, `n` none.
        KeyCode::Char(c @ '1'..='5') => {
            level_cmd(LevelCmd::Toggle(Level::ALL[usize::from(c as u8 - b'1')]));
        }
        KeyCode::Char('a') => level_cmd(LevelCmd::All),
        KeyCode::Char('n') => level_cmd(LevelCmd::Clear),
        _ => {}
    }
}

fn open_level_list() {
    with(|s| {
        let mut combo = level_combo(s.view_level);
        combo.open();
        s.level = Some(combo);
    });
}

/// Scrolls the tab's list by `delta` lines (clamped at paint time).
fn scroll(delta: isize) {
    with(|s| {
        let i = tab_index(s.tab);
        s.offset[i] = s.offset[i].saturating_add_signed(delta);
    });
}

// ---- mouse ----

fn list_bar_region(s: &State) -> crate::ui_kit::ListBarRegion {
    crate::ui_kit::ListBarRegion {
        bar: crate::ui_kit::ListBar::ActivityLog,
        area: s.body,
        len: s.lines,
        visible: usize::from(s.list.height).max(1),
    }
}

fn tab_index(tab: Tab) -> usize {
    match tab {
        Tab::Git => 0,
        Tab::Application => 1,
    }
}

/// The horizontal bar as last painted (either tab).
fn hbar(s: &State) -> Option<crate::scrollbar::Track> {
    crate::scrollbar::Track::horizontal(s.hbody, s.longest.saturating_sub(1), s.hoffset)
}

/// The first column the view may start at (the longest line's end at the
/// right edge).
fn hmax(s: &State) -> usize {
    s.longest
        .saturating_sub(usize::from(s.list.width.saturating_sub(1)))
}

/// A press on the horizontal bar: the end arrows step, the thumb is grabbed
/// where it is held, the bare track jumps the thumb under the pointer and the
/// press keeps dragging (the rule every bar shares, `Track::press`).
fn hbar_press(col: u16, row: u16) -> bool {
    with(|s| {
        let Some(track) = hbar(s) else {
            return false;
        };
        if !track.rect.contains((col, row).into()) {
            return false;
        }
        let max = hmax(s);
        match track.press(col - track.rect.x) {
            crate::scrollbar::TrackPress::Arrow(step) => {
                s.hoffset = s.hoffset.saturating_add_signed(step * H_STEP).min(max);
            }
            crate::scrollbar::TrackPress::Grab { grab, jump } => {
                s.hgrab = Some(grab);
                if jump {
                    s.hoffset = hdrag(s, &track, grab, col);
                }
            }
        }
        true
    })
}

fn hdrag(s: &State, track: &crate::scrollbar::Track, grab: i32, col: u16) -> usize {
    let pos = i32::from(col) - i32::from(track.rect.x);
    track
        .drag_offset(
            s.longest.saturating_sub(1),
            usize::from(track.rect.width),
            grab,
            pos,
        )
        .min(hmax(s))
}

/// A mouse event while the window is open. It takes every event.
pub(crate) fn on_mouse(mouse: MouseEvent) {
    with(|s| s.pointer = Some((mouse.column, mouse.row)));
    let at = (mouse.column, mouse.row).into();

    // The confirmation owns the pointer: only its buttons answer.
    if with(|s| s.clear_open) {
        if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
            let hit = with(|s| s.hits.iter().find(|(r, _)| r.contains(at)).map(|(_, h)| *h));
            match hit {
                Some(Hit::ClearConfirm) => clear_log(),
                Some(Hit::ClearCancel | Hit::ClearClose) => with(|s| s.clear_open = false),
                _ => {}
            }
        }
        return;
    }

    // A box's list, open or about to open from a click on its field.
    // A click on the closed Scope box builds its rows and opens it.
    if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left))
        && with(|s| {
            !s.scope_combo.as_ref().is_some_and(Combo::is_open) && s.scope_field.contains(at)
        })
    {
        open_scope_box();
        return;
    }
    for which in [Box3::Level, Box3::Scope] {
        let takes = with(|s| {
            let field = match which {
                Box3::Level => s.level_field,
                Box3::Scope => s.scope_field,
            };
            let on_tab = box_on_tab(which, s.tab);
            box_slot(s, which).as_ref().is_some_and(Combo::is_open)
                || (matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left))
                    && on_tab
                    && box_slot(s, which).is_some()
                    && field.contains(at))
        });
        if takes {
            let outcome = with(|s| box_slot(s, which).as_mut().map(|c| c.on_mouse(mouse)));
            if let Some(ComboOutcome::Picked(value)) = outcome {
                box_picked(which, &value);
            }
            return;
        }
    }

    let tab = with(|s| s.tab);
    match mouse.kind {
        MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
            let delta = if mouse.kind == MouseEventKind::ScrollUp {
                -3
            } else {
                3
            };
            // Shift+wheel scrolls the lines sideways.
            if mouse.modifiers.contains(KeyModifiers::SHIFT) {
                hscroll(delta * 3);
            } else {
                scroll(delta);
            }
        }
        MouseEventKind::ScrollLeft | MouseEventKind::ScrollRight => {
            hscroll(if mouse.kind == MouseEventKind::ScrollLeft {
                -H_STEP
            } else {
                H_STEP
            });
        }
        MouseEventKind::Down(MouseButton::Left) => press(at, mouse),
        MouseEventKind::Drag(MouseButton::Left) => {
            with(|s| {
                if let (Some(grab), Some(track)) = (s.hgrab, hbar(s)) {
                    s.hoffset = hdrag(s, &track, grab, mouse.column);
                } else if let Some(grab) = s.grab {
                    let offset = s.offset[tab_index(tab)];
                    s.offset[tab_index(tab)] =
                        crate::ui_kit::list_bar_drag(list_bar_region(s), offset, grab, mouse.row);
                }
            });
        }
        MouseEventKind::Up(MouseButton::Left) => with(|s| {
            s.grab = None;
            s.hgrab = None;
        }),
        _ => {}
    }
}

fn press(at: ratatui::layout::Position, mouse: MouseEvent) {
    // The horizontal bar sits under the list's last line.
    if hbar_press(mouse.column, mouse.row) {
        return;
    }
    // The scrollbar first: it sits over the list's last column.
    let on_bar = with(|s| {
        let tab = tab_index(s.tab);
        let region = list_bar_region(s);
        let Some(press) =
            crate::ui_kit::list_bar_press(region, s.offset[tab], mouse.column, mouse.row)
        else {
            return false;
        };
        if let Some(offset) = press.jump_to {
            s.offset[tab] = offset;
        }
        s.grab = (!press.arrow).then_some(press.grab);
        true
    });
    if on_bar {
        return;
    }
    let (close_hit, inside, hit) = with(|s| {
        (
            s.close.contains(at),
            s.window.contains(at),
            s.hits.iter().find(|(r, _)| r.contains(at)).map(|(_, h)| *h),
        )
    });
    if close_hit {
        close();
        return;
    }
    if !inside {
        close();
        return;
    }
    with(|s| s.filter_focus = false);
    match hit {
        Some(Hit::Tab(tab)) => switch_tab(tab),
        Some(Hit::Header(i)) => {
            if let Some(entry) = entries().get(i) {
                with(|s| s.selected = i);
                toggle_expanded(entry.id);
            }
        }
        Some(Hit::CopyEntry(i)) => copy_entry(i),
        Some(Hit::Remedy(i, nth)) => run_remedy(i, nth),
        Some(Hit::CopyDiagnostics) => copy_diagnostics(),
        Some(Hit::CopyAll) => copy_all(),
        Some(Hit::OpenFolder) => open_folder(),
        Some(Hit::ClearLog) => with(ask_clear_log_in),
        Some(Hit::ClearCancel | Hit::ClearConfirm | Hit::ClearClose) => {}
        Some(Hit::Filter) => with(|s| s.filter_focus = true),
        Some(Hit::Toggle(level)) => level_cmd(LevelCmd::Toggle(level)),
        Some(Hit::ShowAll) => level_cmd(LevelCmd::All),
        Some(Hit::ShowNone) => level_cmd(LevelCmd::Clear),
        None => {}
    }
}

#[path = "panel_paint.rs"]
mod paint;
pub(crate) use paint::{
    clear_status_button, note_pointer, paint_status_button, render, set_home, status_button_hit,
    status_button_width,
};

#[cfg(test)]
#[path = "panel_tests.rs"]
mod tests;
