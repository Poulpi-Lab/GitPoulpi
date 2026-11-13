//! The `general` part of the config schema.

use serde::{Deserialize, Serialize};

use super::quantity::{read_quantity, DisplayUnit};

/// Which `GitBackend` a daemon constructs. Read by
/// `gitpoulpi_daemon::backend_for`; the two implementations are held
/// interchangeable by the contract suite in `gitpoulpi-testkit`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DefaultBackend {
    #[default]
    Git2,
    System,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutoStashMode {
    #[default]
    Ask,
    Always,
    Never,
}

/// Which tab a start opens on: `general.startup_tab_in_repo`,
/// `general.startup_tab_outside_repo` and `general.startup_tab_launcher`.
/// Read once at startup.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum StartupTab {
    /// The tab of the git repository the start folder is in (opened if
    /// needed). Outside a repository there is none, so it means `Last`.
    #[default]
    #[serde(rename = "current-repo")]
    CurrentRepo,
    /// The tab that was active when the session last quit. Home when there
    /// is none, and always Home with `remember_tabs` off.
    #[serde(rename = "last")]
    Last,
    /// The Home tab (the repository list).
    #[serde(rename = "home")]
    Home,
    /// The Radar tab; Home when the Radar is off or not built.
    #[serde(rename = "radar")]
    Radar,
}

impl StartupTab {
    pub const ALL: &'static [StartupTab] = &[
        StartupTab::CurrentRepo,
        StartupTab::Last,
        StartupTab::Home,
        StartupTab::Radar,
    ];

    /// The choices of `startup_tab_outside_repo` / `startup_tab_launcher`:
    /// no repository to be "current" there.
    pub const WITHOUT_REPO: &'static [StartupTab] =
        &[StartupTab::Last, StartupTab::Home, StartupTab::Radar];

    pub fn as_str(self) -> &'static str {
        match self {
            StartupTab::CurrentRepo => "current-repo",
            StartupTab::Last => "last",
            StartupTab::Home => "home",
            StartupTab::Radar => "radar",
        }
    }
}

/// How many commits the TUI asks the daemon for when it first opens a
/// repository's graph.
///
/// The default and the floor are 5000 and 500; the floor
/// keeps a mistyped `1` from producing a one-row Log. The TUI clamps
/// rather than rejects - see
/// [`GeneralConfig::initial_commits_in_graph_clamped`].
pub const DEFAULT_INITIAL_COMMITS_IN_GRAPH: u32 = 5000;

pub const MIN_INITIAL_COMMITS_IN_GRAPH: u32 = 500;

/// How many commits one lazily-loaded page fetches once the scroll
/// approaches the earliest commit held.
///
/// A fifth of the initial load: big enough that a fetch is a rare event,
/// small enough to come back much faster than the initial load. The floor
/// exists because below it the daemon round trip plus a fresh revwalk
/// dominate and the user out-scrolls the loader.
pub const DEFAULT_COMMITS_PER_LAZY_PAGE: u32 = 1000;

pub const MIN_COMMITS_PER_LAZY_PAGE: u32 = 50;

/// The ceiling on how many commits the client will ever hold for one
/// repository's graph - the point at which lazy loading stops asking.
///
/// **100 000.** A `CommitNode` measures a little under 300 bytes with its
/// heap strings and parent list, so that is roughly 30-45 MB per open
/// repository, while covering git.git (~75 000 commits) and nearly every
/// repository in full.
///
/// A *ceiling*, not a budget: nothing is allocated up front. It exists so
/// that pointing GitPoulpi at the Linux kernel (~1.3 million commits) and
/// holding `PageDown` cannot walk the process into swap. Raise it in
/// `config.toml` for the whole of such a history; `0` means no ceiling.
pub const DEFAULT_MAX_COMMITS_IN_GRAPH: u32 = 100_000;

/// Default for `general.auto_fetch_interval_minutes`.
pub const DEFAULT_AUTO_FETCH_INTERVAL_MINUTES: u16 = 5;

/// Upper bound for `general.auto_fetch_interval_minutes`, matching
/// The 0-60 field. `0` disables auto-fetch entirely.
pub const MAX_AUTO_FETCH_INTERVAL_MINUTES: u16 = 60;

/// Default and upper bound for `general.hosting_refresh_minutes`.
pub const DEFAULT_HOSTING_REFRESH_MINUTES: u16 = 5;

pub const MAX_HOSTING_REFRESH_MINUTES: u16 = 120;

/// The units the two minute settings are offered in.
pub const MINUTES_OR_HOURS: &[DisplayUnit] = &[DisplayUnit::Minutes, DisplayUnit::Hours];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    pub default_backend: DefaultBackend,
    pub auto_stash: AutoStashMode,
    pub include_untracked_in_autostash: bool,
    /// Live: the `limit` the TUI's first `Log` request carries.
    pub initial_commits_in_graph: u32,
    /// Live: lazy loading of commits in the graph - *additional commits are loaded
    /// when you reach the earliest commit in the graph*. Off, you get
    /// `initial_commits_in_graph` and nothing more.
    pub lazy_load_commits_in_graph: bool,
    /// Live: how many commits one lazily-loaded page fetches. Only
    /// consulted when `lazy_load_commits_in_graph` is on. Clamped by
    /// [`GeneralConfig::commits_per_lazy_page_clamped`].
    pub commits_per_lazy_page: u32,
    /// Live: the ceiling on how many commits the graph will ever hold.
    /// `0` means no ceiling. See [`DEFAULT_MAX_COMMITS_IN_GRAPH`].
    pub max_commits_in_graph: u32,
    /// Live: minutes between background fetches, `0` to disable.
    /// Defaults to 5.
    pub auto_fetch_interval_minutes: u16,
    /// Live: the unit Settings shows `auto_fetch_interval_minutes` in (`auto`,
    /// `minutes`, `hours`). Display only. See [`DisplayUnit`].
    pub auto_fetch_interval_unit: DisplayUnit,
    /// Live: run Fetch All in the background when a repository tab
    /// opens (app start for the active tab, opening or switching to a
    /// tab), unless that repository was fetched within the last minute.
    /// On by default.
    pub fetch_on_tab_open: bool,
    /// Live: minutes between background refreshes of the pull/merge
    /// requests and issues the sidebar lists, `0` to disable (they then
    /// refresh only on tab open, HEAD change, push/fetch, or on demand).
    pub hosting_refresh_minutes: u16,
    /// Live: the unit Settings shows `hosting_refresh_minutes` in (`auto`,
    /// `minutes`, `hours`). Display only. See [`DisplayUnit`].
    pub hosting_refresh_unit: DisplayUnit,
    pub auto_prune: bool,
    /// Live: on a fetch, also delete the local tags the remote has deleted
    /// (only tags that were on it: an unpushed tag is never touched). Off
    /// by default, like git's `fetch.pruneTags`.
    pub auto_prune_tags: bool,
    pub keep_submodules_up_to_date: bool,
    pub default_branch_name: String,
    pub delete_orig_files_after_merge: bool,
    pub remember_tabs: bool,
    /// Live: whether the Radar exists at all (its tab, its palette
    /// commands, `Alt+R`, and its background hosting fetches). On by
    /// default; a user who never uses it turns it off. A build without the
    /// `radar` cargo feature has no Radar whatever this says.
    pub radar_enabled: bool,
    /// Read once at startup: the tab a start opens on when the start folder
    /// is inside a git work tree. `current-repo` (default: that repository's
    /// tab, opened if needed), `last`, `home` or `radar`. A repository
    /// path given on the command line (`--repo`) always opens its own tab.
    pub startup_tab_in_repo: StartupTab,
    /// Read once at startup: the tab a start opens on when the start folder
    /// is not a git repository. `home` (default), `last` or `radar`.
    pub startup_tab_outside_repo: StartupTab,
    /// Read once at startup: the tab a start opens on when GitPoulpi was
    /// started by its desktop / Start menu shortcut (`--launcher`). `last`
    /// (default, owner 2026-10-21), `home` or `radar`. Wins over the two settings above.
    pub startup_tab_launcher: StartupTab,
    /// Live: tabs of a repository's linked worktrees are kept right after
    /// that repository's tab ("Automatically group worktree tabs
    /// together"), when opened, when dragged away and when this is turned
    /// on. On by default.
    pub group_worktree_tabs: bool,
    /// The `id` of the entry in `[[profiles]]` that is currently in
    /// force. An id naming no profile falls back to the built-in
    /// identity, which is whatever `git config user.*` says.
    pub active_profile: String,
    /// Live: the editor "Open in <editor>" launches, as a command plus
    /// arguments with a `{file}` placeholder (appended when absent).
    /// Empty = VS Code (`code`) when it is on `PATH`, else the system
    /// default program.
    pub external_editor: String,
    /// Live: the diff tool "Open in external diff tool" launches, a
    /// command template with `{local}` (the HEAD/index version of the
    /// file) and `{remote}` (the working-tree file). Empty = `git
    /// difftool`.
    pub external_diff_tool: String,
    /// Live: the merge tool "Open in external merge tool" (conflict cards)
    /// launches, a command template with `{base}`, `{local}` (A, the
    /// current side), `{remote}` (B, the incoming side) and `{merged}` (the
    /// file to write the result to). Empty = `git mergetool`.
    pub external_merge_tool: String,
    /// Live: what "Show in folder" launches, a command template with
    /// `{path}` (the file's absolute path) and `{dir}` (its folder).
    /// Defaults per platform, see [`default_file_explorer`].
    pub file_explorer: String,
    /// Live: how many operations Undo can step back through (the
    /// undo journal, the undo-redo design). Default
    /// [`DEFAULT_UNDO_DEPTH`]; `0` means unlimited.
    pub undo_depth: u32,
    /// Live: undo/redo on or off. Off, the daemon does no undo work at
    /// all (no journal, no snapshots, no pins), and the existing history
    /// (journal file and pins) is erased once (`journal/housekeeping.rs`).
    pub undo_enabled: bool,
    /// Live: record what changes in the repository outside GitPoulpi (a
    /// shell, an AI agent) as entries of the undo history
    /// (the outside-operations design). Needs `undo_enabled`.
    pub journal_outside: bool,
    /// Live: proactive conflict detection (a badge when merging the current
    /// branch into its base branch would conflict). A repository can turn it
    /// off for itself (`RepoPreferences::conflict_detection`); this is the
    /// global switch. Default on.
    pub conflict_detection: bool,
    /// Developer mode, never shown in Settings. Absent = automatic: on when
    /// the running exe sits in `<root>/target/<profile>/` of a GitPoulpi
    /// checkout. `true` / `false` wins. See [`crate::dev_mode`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dev_mode: Option<bool>,
}

/// Default for `general.undo_depth`.
pub const DEFAULT_UNDO_DEPTH: u32 = 100;

/// The built-in `general.file_explorer`: Explorer with the file selected on
/// Windows, `open -R` on macOS, `xdg-open` on the parent folder elsewhere.
pub fn default_file_explorer() -> String {
    if cfg!(windows) {
        "explorer.exe /select,\"{path}\"".into()
    } else if cfg!(target_os = "macos") {
        "open -R \"{path}\"".into()
    } else {
        "xdg-open \"{dir}\"".into()
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            default_backend: DefaultBackend::default(),
            auto_stash: AutoStashMode::default(),
            include_untracked_in_autostash: false,
            initial_commits_in_graph: DEFAULT_INITIAL_COMMITS_IN_GRAPH,
            lazy_load_commits_in_graph: true,
            commits_per_lazy_page: DEFAULT_COMMITS_PER_LAZY_PAGE,
            max_commits_in_graph: DEFAULT_MAX_COMMITS_IN_GRAPH,
            auto_fetch_interval_minutes: DEFAULT_AUTO_FETCH_INTERVAL_MINUTES,
            auto_fetch_interval_unit: DisplayUnit::default(),
            fetch_on_tab_open: true,
            hosting_refresh_minutes: DEFAULT_HOSTING_REFRESH_MINUTES,
            hosting_refresh_unit: DisplayUnit::default(),
            auto_prune: true,
            auto_prune_tags: false,
            keep_submodules_up_to_date: false,
            default_branch_name: "main".into(),
            delete_orig_files_after_merge: false,
            remember_tabs: true,
            radar_enabled: true,
            startup_tab_in_repo: StartupTab::CurrentRepo,
            startup_tab_outside_repo: StartupTab::Home,
            startup_tab_launcher: StartupTab::Last,
            group_worktree_tabs: true,
            active_profile: String::new(),
            external_editor: String::new(),
            external_diff_tool: String::new(),
            external_merge_tool: String::new(),
            file_explorer: default_file_explorer(),
            undo_depth: DEFAULT_UNDO_DEPTH,
            undo_enabled: true,
            journal_outside: true,
            conflict_detection: true,
            dev_mode: None,
        }
    }
}

impl GeneralConfig {
    /// The journal's bound as an entry count: `0` (unlimited) becomes
    /// `usize::MAX`.
    pub fn undo_depth_limit(&self) -> usize {
        match self.undo_depth {
            0 => usize::MAX,
            n => n as usize,
        }
    }

    /// The value a component should actually act on: a configured `0`
    /// (or anything under the floor) would otherwise produce an empty
    /// graph, which reads exactly like a broken repository.
    pub fn initial_commits_in_graph_clamped(&self) -> u32 {
        self.initial_commits_in_graph
            .max(MIN_INITIAL_COMMITS_IN_GRAPH)
    }

    /// The page size a lazy fetch should actually ask for. Clamped for
    /// the same reason `initial_commits_in_graph` is: a configured `0`
    /// would request an empty page forever, which reads exactly like a
    /// graph that has stopped loading.
    pub fn commits_per_lazy_page_clamped(&self) -> u32 {
        self.commits_per_lazy_page.max(MIN_COMMITS_PER_LAZY_PAGE)
    }

    /// The ceiling to act on, as `None` when there is none.
    ///
    /// A ceiling below the initial load would be self-contradictory --
    /// the first request already exceeds it - so it is raised to the
    /// initial load rather than silently truncating the first page.
    pub fn max_commits_in_graph_effective(&self) -> Option<u32> {
        match self.max_commits_in_graph {
            0 => None,
            n => Some(n.max(self.initial_commits_in_graph_clamped())),
        }
    }

    /// `None` when auto-fetch is off. Clamped at the top for the same
    /// reason the graph limit is clamped at the bottom.
    pub fn auto_fetch_interval(&self) -> Option<std::time::Duration> {
        match self
            .auto_fetch_interval_minutes
            .min(MAX_AUTO_FETCH_INTERVAL_MINUTES)
        {
            0 => None,
            minutes => Some(std::time::Duration::from_secs(u64::from(minutes) * 60)),
        }
    }

    /// The auto-fetch interval as Settings reads it back: a count and a
    /// concrete unit, always exactly `auto_fetch_interval_minutes` (clamped
    /// to 0..=[`MAX_AUTO_FETCH_INTERVAL_MINUTES`]) minutes.
    pub fn auto_fetch_quantity(&self) -> (u64, DisplayUnit) {
        read_quantity(
            u64::from(
                self.auto_fetch_interval_minutes
                    .min(MAX_AUTO_FETCH_INTERVAL_MINUTES),
            ),
            DisplayUnit::Minutes,
            self.auto_fetch_interval_unit,
            MINUTES_OR_HOURS,
        )
    }

    /// The hosting refresh interval as Settings reads it back, like
    /// [`Self::auto_fetch_quantity`].
    pub fn hosting_refresh_quantity(&self) -> (u64, DisplayUnit) {
        read_quantity(
            u64::from(
                self.hosting_refresh_minutes
                    .min(MAX_HOSTING_REFRESH_MINUTES),
            ),
            DisplayUnit::Minutes,
            self.hosting_refresh_unit,
            MINUTES_OR_HOURS,
        )
    }

    /// `None` when the periodic hosting refresh is off.
    pub fn hosting_refresh_interval(&self) -> Option<std::time::Duration> {
        match self
            .hosting_refresh_minutes
            .min(MAX_HOSTING_REFRESH_MINUTES)
        {
            0 => None,
            minutes => Some(std::time::Duration::from_secs(u64::from(minutes) * 60)),
        }
    }
}

/// A named commit identity. `id` is the stable slug
/// `general.active_profile` points at; `name` is what the user reads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub author_name: String,
    pub author_email: String,
}

impl Profile {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            author_name: String::new(),
            author_email: String::new(),
        }
    }
}
