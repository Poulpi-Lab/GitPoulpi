//! The TUI's half of the session files: which repositories are open in
//! this session, and which have been opened recently.
//!
//! ADR-0006 puts this state beside the daemon rather than inside it: the
//! daemon owns everything a `GitBackend` could tell you, and a tab list is
//! a property of a *window*. So these are plain file reads and writes
//! through `gitpoulpi-session`, with no IPC.
//!
//! **Every function here is best-effort.** A config directory that cannot be
//! written is a lost tab list, not a TUI that refuses to start. Failures are
//! swallowed deliberately: a `Result` at a dozen call sites inside a render
//! loop would have nothing useful to do with one.

use std::path::{Path, PathBuf};

use gitpoulpi_session::{now_seconds, FavoriteRepos, GlobalSession, PaneLayout, RecentRepos};

use crate::repo_manager::KnownRepo;
use crate::repo_switcher::RepoTab;

/// The config directory every function here works relative to.
///
/// Each public function resolves this once and hands it to a `*_in` twin,
/// which the tests drive. Going through the process environment
/// (`GITPOULPI_CONFIG_DIR`) is wrong: it is global and the daemon's own
/// port file lives in the same directory, so a test that repointed it raced
/// `ensure_daemon` in an unrelated test and deadlocked it.
pub(crate) fn config_dir() -> Option<PathBuf> {
    gitpoulpi_config::ConfigPaths::discover().map(|p| p.config_dir().to_path_buf())
}

pub(crate) fn session_file_in(config_dir: &Path, session: &str) -> PathBuf {
    config_dir.join("sessions").join(format!("{session}.toml"))
}

fn recent_file_in(config_dir: &Path) -> PathBuf {
    RecentRepos::file_in(config_dir)
}

pub(crate) fn load_session_in(config_dir: &Path, session: &str) -> (PathBuf, GlobalSession) {
    let path = session_file_in(config_dir, session);
    let loaded = GlobalSession::load(&path).unwrap_or_default();
    (path, loaded)
}

/// The tabs open in `session`, labeled from the recent list where it knows
/// the name and branch.
///
/// The labels come from the recent file rather than a `discover` per tab:
/// drawing the bar must not cost N round trips to the daemon, and a tab's
/// name is the last path component anyway (what `RepoTab::name` falls back
/// to for a tab the recent list has forgotten).
pub fn tabs(session: &str) -> Vec<RepoTab> {
    match config_dir() {
        Some(dir) => tabs_in(&dir, session),
        None => Vec::new(),
    }
}

fn tabs_in(config_dir: &Path, session: &str) -> Vec<RepoTab> {
    let (_, loaded) = load_session_in(config_dir, session);
    let recents = RecentRepos::load(&recent_file_in(config_dir));
    loaded
        .tabs
        .iter()
        .map(|tab| {
            let known = recents.iter().find(|r| r.path == tab.repo_path);
            RepoTab {
                name: known
                    .map(|r| r.name.clone())
                    .unwrap_or_else(|| name_for(&tab.repo_path)),
                branch: known.and_then(|r| r.branch.clone()),
                path: tab.repo_path.clone(),
            }
        })
        .collect()
}

pub(crate) fn name_for(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}

/// Whether `path` *is* a repository: the root of one, not merely somewhere
/// inside one.
///
/// This guard keeps the recent list honest. The TUI falls back to the
/// working directory when no `.git` is found above it (`repo_ref_for`), so
/// recording that directory unconditionally would put a home directory in
/// the picker forever, as a repository that opens to a backend error. The
/// one place every recording goes through enforces it.
///
/// Both layouts count: an ordinary repository (`.git`, a directory, or a
/// *file* for a linked worktree) and a bare one (`HEAD` + `objects` +
/// `refs` in the directory itself), which is what `GitBackend::discover`
/// reports as `is_bare`.
pub fn is_repository(path: &Path) -> bool {
    if path.join(".git").exists() {
        return true;
    }
    path.join("HEAD").is_file() && path.join("objects").is_dir() && path.join("refs").is_dir()
}

/// Opens (or focuses) `path` as a tab, and records it in the recent list.
/// `branch` is what the tab's subtitle shows; `None` leaves whatever the
/// recent list already knew.
///
/// Returns whether anything was recorded: a path that is not a repository
/// is refused here rather than at each call site.
pub fn open_tab(session: &str, path: &Path, branch: Option<String>) -> bool {
    match config_dir() {
        Some(dir) => open_tab_in(&dir, session, path, branch),
        None => false,
    }
}

fn open_tab_in(config_dir: &Path, session: &str, path: &Path, branch: Option<String>) -> bool {
    if !is_repository(path) {
        return false;
    }
    let (file, mut loaded) = load_session_in(config_dir, session);
    loaded.open(path);
    if crate::live_settings::group_worktree_tabs() {
        loaded.group_worktrees(gitpoulpi_session::linked_worktree_main);
    }
    let _ = loaded.save(&file);

    let file = recent_file_in(config_dir);
    let mut recents = RecentRepos::load(&file);
    // An `open_tab` that knows no branch must not *erase* the one the
    // list already had: the tab bar would lose its subtitle every time
    // a session was restored from its file.
    let branch = branch.or_else(|| {
        recents
            .iter()
            .find(|r| r.path == path)
            .and_then(|r| r.branch.clone())
    });
    recents.record(path, &name_for(path), branch, now_seconds());
    let _ = recents.save(&file);
    true
}

/// Whether this session's Radar tab was closed by the user.
pub fn radar_closed(session: &str) -> bool {
    config_dir().is_some_and(|dir| radar_closed_in(&dir, session))
}

fn radar_closed_in(config_dir: &Path, session: &str) -> bool {
    load_session_in(config_dir, session).1.radar_closed
}

/// Closes (or reopens) this session's Radar tab, persistently.
pub fn set_radar_closed(session: &str, closed: bool) {
    if let Some(dir) = config_dir() {
        set_radar_closed_in(&dir, session, closed);
    }
}

fn set_radar_closed_in(config_dir: &Path, session: &str, closed: bool) {
    let (file, mut loaded) = load_session_in(config_dir, session);
    if loaded.radar_closed != closed {
        loaded.radar_closed = closed;
        let _ = loaded.save(&file);
    }
}

/// Whether this session's Workspaces tab is open (it persists).
pub fn workspaces_open(session: &str) -> bool {
    config_dir().is_some_and(|dir| workspaces_open_in(&dir, session))
}

fn workspaces_open_in(config_dir: &Path, session: &str) -> bool {
    load_session_in(config_dir, session).1.workspaces_open
}

/// Opens (or closes) this session's Workspaces tab, persistently.
pub fn set_workspaces_open(session: &str, open: bool) {
    if let Some(dir) = config_dir() {
        set_workspaces_open_in(&dir, session, open);
    }
}

fn set_workspaces_open_in(config_dir: &Path, session: &str, open: bool) {
    let (file, mut loaded) = load_session_in(config_dir, session);
    if loaded.workspaces_open != open {
        loaded.workspaces_open = open;
        let _ = loaded.save(&file);
    }
}

/// Closes the tab at `index`. The repository itself is untouched.
pub fn close_tab(session: &str, index: usize) {
    if let Some(dir) = config_dir() {
        close_tab_in(&dir, session, index);
    }
}

fn close_tab_in(config_dir: &Path, session: &str, index: usize) {
    let (file, mut loaded) = load_session_in(config_dir, session);
    loaded.close(index);
    let _ = loaded.save(&file);
}

/// Moves the tab at `from` to index `to`, so the new order survives a
/// restart (the tab list is the session file's own order).
pub fn move_tab(session: &str, from: usize, to: usize) {
    if let Some(dir) = config_dir() {
        move_tab_in(&dir, session, from, to);
    }
}

fn move_tab_in(config_dir: &Path, session: &str, from: usize, to: usize) {
    let (file, mut loaded) = load_session_in(config_dir, session);
    // With grouping on, a worktree tab dropped away from its repository
    // snaps back next to it (and a repository tab brings its worktrees).
    let moved = loaded.move_tab(from, to);
    let regrouped = crate::live_settings::group_worktree_tabs()
        && loaded.group_worktrees(gitpoulpi_session::linked_worktree_main);
    if moved || regrouped {
        let _ = loaded.save(&file);
    }
}

/// Regroups `session`'s worktree tabs behind their repositories' tabs: the
/// moment the setting is turned on.
pub fn regroup_worktree_tabs(session: &str) {
    if let Some(dir) = config_dir() {
        let (file, mut loaded) = load_session_in(&dir, session);
        if loaded.group_worktrees(gitpoulpi_session::linked_worktree_main) {
            let _ = loaded.save(&file);
        }
    }
}

/// A cheap fingerprint of `session`'s file (modification time, length), to
/// notice that something else - `gitpoulpi repo open|close` - rewrote it.
pub fn session_stamp(session: &str) -> Option<(std::time::SystemTime, u64)> {
    let meta = std::fs::metadata(session_file_in(&config_dir()?, session)).ok()?;
    Some((meta.modified().ok()?, meta.len()))
}

/// The index of the tab last focused in `session`, if any.
pub fn active_tab_index(session: &str) -> Option<usize> {
    let dir = config_dir()?;
    let (_, loaded) = load_session_in(&dir, session);
    loaded.active_index()
}

/// Focuses the tab at `index`, so the next start of this session opens
/// on the repository the user was last looking at.
pub fn activate_tab(session: &str, index: usize) {
    if let Some(dir) = config_dir() {
        let (file, mut loaded) = load_session_in(&dir, session);
        loaded.activate(index);
        let _ = loaded.save(&file);
    }
}

/// What a *launch* does to the tab list (`general.remember_tabs`).
///
/// Sessions are derived from the launch folder's repository root, so the
/// tabs a user left open sit in the session file of whichever repository
/// they launched from last, not the one they launch from now.
/// `last-session` (a one-line file beside `sessions/`) names that session.
///
/// * `remember` on and `implicit_session` (no `-s`): `session` is seeded
///   with the last session's tabs (its own extra tabs are kept after them,
///   deduplicated on path), then the launch repository is opened: added if
///   absent, and active either way. Outside a repository
///   (`launch == None`) the remembered active tab stays active.
/// * `remember` off: one tab, the launch repository; outside a repository
///   (`launch == None`) no tab at all, the session starts on Home.
///
/// * `activate_launch` off (a start that opens another tab, see
///   `startup_tab`): the launch repository is still added as a tab, but the
///   tab that was active stays active.
///
/// Finally `session` becomes the last session, so tabs opened during this
/// run are what the next launch, from anywhere, restores.
pub fn restore_launch_tabs(
    session: &str,
    launch: Option<&Path>,
    remember: bool,
    implicit_session: bool,
    activate_launch: bool,
) {
    // The launch opens a tab, which groups worktrees: the setting is read
    // here, before any window installs it.
    if let Ok(settings) = gitpoulpi_config::Settings::resolve_ambient() {
        crate::live_settings::install_group_worktree_tabs(
            settings.config().general.group_worktree_tabs,
        );
    }
    if let Some(dir) = config_dir() {
        restore_launch_tabs_with(
            &dir,
            session,
            launch,
            remember,
            implicit_session,
            activate_launch,
        );
    }
}

/// The repository path of the tab last focused in `session`.
pub fn active_tab_path(session: &str) -> Option<PathBuf> {
    let dir = config_dir()?;
    let (_, loaded) = load_session_in(&dir, session);
    loaded
        .tabs
        .get(loaded.active_index()?)
        .map(|t| t.repo_path.clone())
}

fn last_session_file_in(config_dir: &Path) -> PathBuf {
    config_dir.join("last-session")
}

#[cfg(test)]
fn restore_launch_tabs_in(
    config_dir: &Path,
    session: &str,
    launch: Option<&Path>,
    remember: bool,
    implicit_session: bool,
) {
    restore_launch_tabs_with(
        config_dir,
        session,
        launch,
        remember,
        implicit_session,
        true,
    );
}

fn restore_launch_tabs_with(
    config_dir: &Path,
    session: &str,
    launch: Option<&Path>,
    remember: bool,
    implicit_session: bool,
    activate_launch: bool,
) {
    let last_file = last_session_file_in(config_dir);
    if remember && implicit_session {
        let last = std::fs::read_to_string(&last_file)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        if !last.is_empty() && last != session {
            let (_, source) = load_session_in(config_dir, &last);
            let (file, mut own) = load_session_in(config_dir, session);
            let mut merged = source.clone();
            for tab in &own.tabs {
                if merged.index_of(&tab.repo_path).is_none() {
                    merged.tabs.push(gitpoulpi_session::TabEntry {
                        repo_path: tab.repo_path.clone(),
                        active: false,
                    });
                }
            }
            if !merged.tabs.is_empty() {
                own.tabs = merged.tabs;
                let _ = own.save(&file);
            }
        }
    }
    if let Some(repo) = launch {
        let keep_active = if activate_launch {
            None
        } else {
            let (_, loaded) = load_session_in(config_dir, session);
            loaded
                .active_index()
                .and_then(|i| loaded.tabs.get(i))
                .map(|t| t.repo_path.clone())
        };
        // Records the recent entry too, and activates an existing tab.
        open_tab_in(config_dir, session, repo, None);
        if let Some(previous) = keep_active {
            let (file, mut loaded) = load_session_in(config_dir, session);
            if let Some(index) = loaded.index_of(&previous) {
                loaded.activate(index);
                let _ = loaded.save(&file);
            }
        }
        if !remember {
            forget_other_tabs_in(config_dir, session, repo);
        }
    } else if !remember {
        // Outside a repository there is nothing to keep: the session
        // starts on Home alone, whatever it had open when it last quit.
        forget_all_tabs_in(config_dir, session);
    }
    if let Some(parent) = last_file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&last_file, session);
}

#[cfg(test)]
fn active_tab_index_in(config_dir: &Path, session: &str) -> Option<usize> {
    load_session_in(config_dir, session).1.active_index()
}

/// `general.remember_tabs = false`'s effect: collapses `session`'s
/// persisted tab list down to `keep` alone, so the *next* launch does not
/// restore whatever was open when this one quit. Called only by
/// `restore_launch_tabs` at launch - never from `run_in_repo`'s
/// per-switch tab reload, which must show every tab opened *during* a run.
fn forget_other_tabs_in(config_dir: &Path, session: &str, keep: &Path) {
    let (file, mut loaded) = load_session_in(config_dir, session);
    loaded
        .tabs
        .retain(|tab| gitpoulpi_session::same_repo_path(&tab.repo_path, keep));
    if !loaded.tabs.is_empty() {
        loaded.activate(0);
    }
    let _ = loaded.save(&file);
}

/// `forget_other_tabs_in` for a launch outside any repository: no tab is
/// kept.
fn forget_all_tabs_in(config_dir: &Path, session: &str) {
    let (file, mut loaded) = load_session_in(config_dir, session);
    if !loaded.tabs.is_empty() {
        loaded.tabs.clear();
        let _ = loaded.save(&file);
    }
}

/// The three-pane split's persisted shape for `session` - the two
/// divider percentages and which side panes are collapsed. `default()`
/// (20/65, nothing collapsed) when the session has never saved one.
pub fn layout(session: &str) -> PaneLayout {
    match config_dir() {
        Some(dir) => layout_in(&dir, session),
        None => PaneLayout::default(),
    }
}

fn layout_in(config_dir: &Path, session: &str) -> PaneLayout {
    let (_, loaded) = load_session_in(config_dir, session);
    loaded.layout.sanitized()
}

/// Persists `layout` for `session`, the same way a tab-bar change is
/// persisted: read the file, change the one field, write it back. Called
/// whenever a divider drag ends or a pane is collapsed/restored - not on
/// every intermediate drag frame, which would be a write per pixel of
/// mouse motion.
pub fn save_layout(session: &str, layout: PaneLayout) {
    if let Some(dir) = config_dir() {
        save_layout_in(&dir, session, layout);
    }
}

/// Persists just the Refs sidebar's section heights, leaving the rest of
/// the saved layout as it is.
pub fn save_sidebar_heights(session: &str, heights: &[u16]) {
    if let Some(dir) = config_dir() {
        let mut layout = layout_in(&dir, session);
        layout.sidebar_heights = heights.to_vec();
        save_layout_in(&dir, session, layout);
    }
}

/// Persists the height the user dragged the commit message block to
/// (`0` = automatic), leaving the rest of the saved layout as it is.
pub fn save_commit_message_rows(session: &str, rows: u16) {
    if let Some(dir) = config_dir() {
        let mut layout = layout_in(&dir, session);
        layout.commit_message_rows = rows;
        save_layout_in(&dir, session, layout);
    }
}

/// Persists the History view's list width (percent, `0` = default),
/// leaving the rest of the saved layout as it is.
pub fn save_history_split(session: &str, pct: u16) {
    if let Some(dir) = config_dir() {
        let mut layout = layout_in(&dir, session);
        layout.history_split_pct = pct;
        save_layout_in(&dir, session, layout);
    }
}

/// What the commit column remembers: shown or not (`0` automatic, `1` on,
/// `2` off), its width (cells, `0` = default), which list it shows when shown
/// (`0` all commits, `1` the file's). The widths of its parts and which parts
/// are shown are `config.toml`'s (`ui.commit_column_columns_*`), not here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitColumnSave {
    pub pref: u8,
    pub cols: u16,
    pub mode: u8,
}

/// Persists the commit column's choices, leaving the rest of the saved layout
/// as it is.
pub fn save_commit_column(session: &str, save: CommitColumnSave) {
    if let Some(dir) = config_dir() {
        let mut layout = layout_in(&dir, session);
        layout.commit_column = save.pref;
        layout.commit_column_cols = save.cols;
        layout.commit_column_mode = save.mode;
        save_layout_in(&dir, session, layout);
    }
}

/// Forgets the part widths and shown parts an older session file held, once
/// they are in `config.toml` (`ui.commit_column_columns_*`).
pub fn forget_legacy_commit_column(session: &str) {
    if let Some(dir) = config_dir() {
        let mut layout = layout_in(&dir, session);
        layout.commit_column_widths = Vec::new();
        layout.commit_column_visible = Vec::new();
        save_layout_in(&dir, session, layout);
    }
}

/// Persists the merge editor's column weights (`None` = the equal share)
/// and Output height (`0` = default), leaving the rest as it is.
pub fn save_merge_layout(session: &str, cols: Option<[u16; 3]>, output_pct: u16) {
    if let Some(dir) = config_dir() {
        let mut layout = layout_in(&dir, session);
        layout.merge_cols = cols.map(|c| c.to_vec()).unwrap_or_default();
        layout.merge_output_pct = output_pct;
        save_layout_in(&dir, session, layout);
    }
}

/// Persists the folder picker's left column (per mille, `0` = default).
pub fn save_picker_split(session: &str, permille: u16) {
    if let Some(dir) = config_dir() {
        let mut layout = layout_in(&dir, session);
        layout.picker_split_permille = permille;
        save_layout_in(&dir, session, layout);
    }
}

fn save_layout_in(config_dir: &Path, session: &str, layout: PaneLayout) {
    let (file, mut loaded) = load_session_in(config_dir, session);
    loaded.layout = layout;
    let _ = loaded.save(&file);
}

pub fn forget_recent(path: &Path) {
    if let Some(dir) = config_dir() {
        forget_recent_in(&dir, path);
    }
}

fn forget_recent_in(config_dir: &Path, path: &Path) {
    let file = recent_file_in(config_dir);
    let mut recents = RecentRepos::load(&file);
    if recents.remove(path) {
        let _ = recents.save(&file);
    }
}

/// Empties the recent list - the Recent section's *Remove all*. The
/// repositories themselves, their tabs and their stars are untouched.
pub fn clear_recent() {
    if let Some(dir) = config_dir() {
        clear_recent_in(&dir);
    }
}

fn clear_recent_in(config_dir: &Path) {
    let file = recent_file_in(config_dir);
    let mut recents = RecentRepos::load(&file);
    recents.repos.clear();
    let _ = recents.save(&file);
}

/// Closes every tab whose repository is one of `paths`, in one write.
/// By path rather than by index: closing tab 1 renumbers every tab after
/// it, so a list of indices is wrong from its second entry on.
pub fn close_tabs(session: &str, paths: &[PathBuf]) {
    if let Some(dir) = config_dir() {
        close_tabs_in(&dir, session, paths);
    }
}

fn close_tabs_in(config_dir: &Path, session: &str, paths: &[PathBuf]) {
    let (file, mut loaded) = load_session_in(config_dir, session);
    for path in paths {
        loaded.close_path(path);
    }
    let _ = loaded.save(&file);
}

fn favorites_file_in(config_dir: &Path) -> PathBuf {
    FavoriteRepos::file_in(config_dir)
}

/// Stars or un-stars `path`, persisting at once. Returns whether it is a
/// favorite afterwards.
pub fn toggle_favorite(path: &Path) -> bool {
    match config_dir() {
        Some(dir) => toggle_favorite_in(&dir, path),
        None => false,
    }
}

fn toggle_favorite_in(config_dir: &Path, path: &Path) -> bool {
    let file = favorites_file_in(config_dir);
    let mut favorites = FavoriteRepos::load(&file);
    // The recent list's label when it has one, so a repository shows the
    // same name in every section.
    let name = RecentRepos::load(&recent_file_in(config_dir))
        .iter()
        .find(|r| r.path == path)
        .map(|r| r.name.clone())
        .unwrap_or_else(|| name_for(path));
    let now = favorites.toggle(path, &name);
    let _ = favorites.save(&file);
    now
}

/// Every repository the Repository Management tab knows about - the
/// union of this session's tabs, the favorites and the recent list,
/// joined by path - with what each list says about it. The probe's
/// cached answers come along, so a revisit starts filled in.
pub fn known_repos(session: &str) -> Vec<KnownRepo> {
    match config_dir() {
        Some(dir) => known_repos_in(&dir, session),
        None => Vec::new(),
    }
}

/// Loads the favorites and the recent list, and writes back what a stored
/// file got wrong: spellings of one directory collapsed in both, and recent
/// entries whose folder is gone from disk dropped. A gone *favorite* is
/// kept (starring was deliberate), so it shows as gone with a remove action.
fn tidy_lists_in(config_dir: &Path) -> (FavoriteRepos, RecentRepos) {
    let favorites_file = favorites_file_in(config_dir);
    let mut favorites = FavoriteRepos::load(&favorites_file);
    if favorites.dedupe() > 0 {
        let _ = favorites.save(&favorites_file);
    }
    let recent_file = recent_file_in(config_dir);
    let mut recents = RecentRepos::load(&recent_file);
    let changed = recents.dedupe() + recents.prune_missing();
    if changed > 0 {
        let _ = recents.save(&recent_file);
    }
    (favorites, recents)
}

/// *Remove from list* on an *All repositories* row: forgets the path in
/// the recent list and the favorites. Open tabs are not touched, and
/// nothing on disk is.
pub fn forget_everywhere(path: &Path) {
    if let Some(dir) = config_dir() {
        forget_everywhere_in(&dir, path);
    }
}

fn forget_everywhere_in(config_dir: &Path, path: &Path) {
    forget_recent_in(config_dir, path);
    let file = favorites_file_in(config_dir);
    let mut favorites = FavoriteRepos::load(&file);
    if favorites.remove(path) {
        let _ = favorites.save(&file);
    }
}

fn known_repos_in(config_dir: &Path, session: &str) -> Vec<KnownRepo> {
    let tabs = tabs_in(config_dir, session);
    let (favorites, recents) = tidy_lists_in(config_dir);

    let mut out: Vec<KnownRepo> = Vec::new();
    let mut keys: Vec<String> = Vec::new();
    fn entry<'a>(
        out: &'a mut Vec<KnownRepo>,
        keys: &mut Vec<String>,
        path: &Path,
        name: &str,
    ) -> &'a mut KnownRepo {
        let key = gitpoulpi_session::repo_path_key(path);
        let index = match keys.iter().position(|k| *k == key) {
            Some(index) => index,
            None => {
                keys.push(key);
                out.push(KnownRepo {
                    path: path.to_path_buf(),
                    name: name.to_string(),
                    branch_hint: None,
                    exists: path.exists(),
                    is_repo: is_repository(path),
                    tab: None,
                    favorite: None,
                    recent: None,
                    facts: crate::repo_probe::cached(path),
                });
                out.len() - 1
            }
        };
        &mut out[index]
    }
    for (index, tab) in tabs.iter().enumerate() {
        let known = entry(&mut out, &mut keys, &tab.path, &tab.name);
        known.tab = Some(index);
        known.branch_hint = tab.branch.clone();
    }
    for (index, fav) in favorites.iter().enumerate() {
        entry(&mut out, &mut keys, &fav.path, &fav.name).favorite = Some(index);
    }
    for (index, recent) in recents.iter().enumerate() {
        let known = entry(&mut out, &mut keys, &recent.path, &recent.name);
        known.recent = Some(index);
        if known.branch_hint.is_none() {
            known.branch_hint = recent.branch.clone();
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // These drive the `*_in` functions with an explicit temp config
    // directory, never `GITPOULPI_CONFIG_DIR`: that variable is
    // process-wide and the daemon's port file lives in the same directory,
    // so repointing it races `ensure_daemon` in an unrelated test.

    /// A directory that *is* a repository, as far as the guard is
    /// concerned. `.git` is all `is_repository` looks at, and running a
    /// real `git init` here would make every test in this module depend
    /// on git being installed to assert something about file handling.
    fn repo_dir(base: &Path, name: &str) -> PathBuf {
        let path = base.join(name);
        std::fs::create_dir_all(path.join(".git")).unwrap();
        path
    }

    #[test]
    fn opening_repositories_builds_a_tab_bar_and_a_recent_list() {
        let config = tempfile::tempdir().unwrap();
        let config = config.path();
        let work = tempfile::tempdir().unwrap();
        let alpha = repo_dir(work.path(), "alpha");
        let beta = repo_dir(work.path(), "beta");

        open_tab_in(config, "s", &alpha, Some("main".into()));
        open_tab_in(config, "s", &beta, Some("side".into()));

        let tabs = tabs_in(config, "s");
        assert_eq!(tabs.len(), 2);
        assert_eq!(tabs[0].name, "alpha");
        assert_eq!(tabs[1].branch.as_deref(), Some("side"));

        // Newest first in the recent list, both open as tabs.
        let rows = known_repos_in(config, "s");
        let newest = rows.iter().find(|r| r.recent == Some(0)).unwrap();
        assert_eq!(newest.name, "beta");
        assert!(rows.iter().all(|r| r.tab.is_some() && r.exists));
    }

    /// Restoring a session must not erase the branch subtitles: the
    /// tab bar is rebuilt from the file on every switch, and
    /// `open_tab(.., None)` is what that rebuild calls.
    #[test]
    fn reopening_without_a_branch_keeps_the_one_already_known() {
        let config = tempfile::tempdir().unwrap();
        let config = config.path();
        let work = tempfile::tempdir().unwrap();
        let alpha = repo_dir(work.path(), "alpha");

        open_tab_in(config, "s", &alpha, Some("main".into()));
        open_tab_in(config, "s", &alpha, None);

        assert_eq!(tabs_in(config, "s")[0].branch.as_deref(), Some("main"));
    }

    #[test]
    fn a_moved_tab_stays_moved_when_the_session_is_read_back() {
        let config = tempfile::tempdir().unwrap();
        let config = config.path();
        let work = tempfile::tempdir().unwrap();
        let names = ["alpha", "beta", "gamma"];
        for n in names {
            open_tab_in(config, "s", &repo_dir(work.path(), n), None);
        }
        move_tab_in(config, "s", 0, 2);
        let order: Vec<_> = tabs_in(config, "s").into_iter().map(|t| t.name).collect();
        assert_eq!(order, vec!["beta", "gamma", "alpha"]);
        // The focused tab (gamma, opened last) is still the focused one.
        assert_eq!(active_tab_index_in(config, "s"), Some(1));
    }

    /// The Radar tab's closed mark is per session and survives a
    /// restart (it is in the session file), and reopening clears it.
    #[test]
    fn the_radar_tab_closed_mark_persists_per_session() {
        let config = tempfile::tempdir().unwrap();
        let config = config.path();
        assert!(!radar_closed_in(config, "s"));
        set_radar_closed_in(config, "s", true);
        assert!(radar_closed_in(config, "s"), "read back from the file");
        assert!(!radar_closed_in(config, "other"), "per session");
        set_radar_closed_in(config, "s", false);
        assert!(!radar_closed_in(config, "s"));
    }

    /// The Workspaces tab's open mark is per session and survives a restart.
    #[test]
    fn the_workspaces_tab_open_mark_persists_per_session() {
        let config = tempfile::tempdir().unwrap();
        let config = config.path();
        assert!(!workspaces_open_in(config, "s"), "closed by default");
        set_workspaces_open_in(config, "s", true);
        assert!(workspaces_open_in(config, "s"), "read back from the file");
        assert!(!workspaces_open_in(config, "other"), "per session");
        set_workspaces_open_in(config, "s", false);
        assert!(!workspaces_open_in(config, "s"));
    }

    #[test]
    fn closing_a_tab_leaves_the_repository_in_the_recent_list() {
        let config = tempfile::tempdir().unwrap();
        let config = config.path();
        let work = tempfile::tempdir().unwrap();
        let alpha = repo_dir(work.path(), "alpha");
        let beta = repo_dir(work.path(), "beta");
        open_tab_in(config, "s", &alpha, None);
        open_tab_in(config, "s", &beta, None);

        close_tab_in(config, "s", 1);
        assert_eq!(tabs_in(config, "s").len(), 1);

        let rows = known_repos_in(config, "s");
        assert_eq!(rows.len(), 2, "a closed tab is still reopenable");
        let beta_row = rows.iter().find(|r| r.path == beta).unwrap();
        assert!(beta_row.tab.is_none() && beta_row.recent.is_some());
    }

    /// `general.remember_tabs = false`: only the repository the launch was
    /// pointed at survives, every other tab is dropped from
    /// `sessions/<name>.toml` (a second launch reads that same file). The
    /// recent list is untouched: "don't restore my tabs" is not "forget
    /// these repositories exist".
    #[test]
    fn forgetting_other_tabs_keeps_only_the_one_named() {
        let config = tempfile::tempdir().unwrap();
        let config = config.path();
        let work = tempfile::tempdir().unwrap();
        let alpha = repo_dir(work.path(), "alpha");
        let beta = repo_dir(work.path(), "beta");
        let gamma = repo_dir(work.path(), "gamma");
        open_tab_in(config, "s", &alpha, None);
        open_tab_in(config, "s", &beta, None);
        open_tab_in(config, "s", &gamma, None);
        assert_eq!(tabs_in(config, "s").len(), 3);

        forget_other_tabs_in(config, "s", &beta);

        let tabs = tabs_in(config, "s");
        assert_eq!(tabs.len(), 1);
        assert_eq!(tabs[0].path, beta);
        assert_eq!(
            known_repos_in(config, "s")
                .iter()
                .filter(|r| r.recent.is_some())
                .count(),
            3,
            "the recent list must still remember alpha and gamma"
        );
    }

    /// Sessions are per launch folder, so the tabs a user left open live in
    /// the session of whichever folder they launched from last. A launch from
    /// another repository must pick those up and make its own repository's
    /// tab the active one.
    #[test]
    fn a_launch_restores_the_last_sessions_tabs_and_activates_the_launch_repo() {
        let config = tempfile::tempdir().unwrap();
        let config = config.path();
        let work = tempfile::tempdir().unwrap();
        let alpha = repo_dir(work.path(), "alpha");
        let beta = repo_dir(work.path(), "beta");
        // First run, from alpha: beta is opened as a tab, then quit.
        restore_launch_tabs_in(config, "alpha-s", Some(&alpha), true, true);
        open_tab_in(config, "alpha-s", &beta, None);
        // Second run, from alpha's folder but its own session name is
        // what a *different* folder would derive.
        restore_launch_tabs_in(config, "beta-s", Some(&alpha), true, true);

        let tabs = tabs_in(config, "beta-s");
        let paths: Vec<_> = tabs.iter().map(|t| t.path.clone()).collect();
        assert_eq!(paths, vec![alpha.clone(), beta.clone()]);
        let active = active_tab_index_in(config, "beta-s").unwrap();
        assert_eq!(tabs[active].path, alpha);
    }

    #[test]
    fn a_launch_repo_missing_from_the_remembered_tabs_is_added_and_active() {
        let config = tempfile::tempdir().unwrap();
        let config = config.path();
        let work = tempfile::tempdir().unwrap();
        let alpha = repo_dir(work.path(), "alpha");
        let beta = repo_dir(work.path(), "beta");
        let gamma = repo_dir(work.path(), "gamma");
        restore_launch_tabs_in(config, "a", Some(&alpha), true, true);
        open_tab_in(config, "a", &beta, None);

        restore_launch_tabs_in(config, "g", Some(&gamma), true, true);

        let tabs = tabs_in(config, "g");
        assert_eq!(tabs.len(), 3);
        assert_eq!(tabs[active_tab_index_in(config, "g").unwrap()].path, gamma);
    }

    #[test]
    fn restoring_twice_does_not_duplicate_tabs() {
        let config = tempfile::tempdir().unwrap();
        let config = config.path();
        let work = tempfile::tempdir().unwrap();
        let alpha = repo_dir(work.path(), "alpha");
        let beta = repo_dir(work.path(), "beta");
        restore_launch_tabs_in(config, "a", Some(&alpha), true, true);
        open_tab_in(config, "a", &beta, None);
        restore_launch_tabs_in(config, "b", Some(&beta), true, true);
        restore_launch_tabs_in(config, "a", Some(&alpha), true, true);
        restore_launch_tabs_in(config, "b", Some(&beta), true, true);
        assert_eq!(tabs_in(config, "a").len(), 2);
        assert_eq!(tabs_in(config, "b").len(), 2);
    }

    #[test]
    fn a_launch_outside_a_repository_still_restores_the_last_sessions_tabs() {
        let config = tempfile::tempdir().unwrap();
        let config = config.path();
        let work = tempfile::tempdir().unwrap();
        let alpha = repo_dir(work.path(), "alpha");
        let beta = repo_dir(work.path(), "beta");
        restore_launch_tabs_in(config, "a", Some(&alpha), true, true);
        open_tab_in(config, "a", &beta, None);
        restore_launch_tabs_in(config, "plain-folder", None, true, true);
        assert_eq!(tabs_in(config, "plain-folder").len(), 2);
    }

    /// A taskbar launch has a folder like `C:\Windows\System32` (no repository,
    /// its own implicit session): "last" must still find the tab that was
    /// active in the previous session, through `last-session`.
    #[test]
    fn a_launch_from_a_plain_folder_reads_the_previous_sessions_active_tab_as_last() {
        let config = tempfile::tempdir().unwrap();
        let config = config.path();
        let work = tempfile::tempdir().unwrap();
        let alpha = repo_dir(work.path(), "alpha");
        let beta = repo_dir(work.path(), "beta");
        restore_launch_tabs_in(config, "a", Some(&alpha), true, true);
        open_tab_in(config, "a", &beta, None);
        // The next run starts from System32: another implicit session, no
        // launch repository, nothing activated.
        restore_launch_tabs_with(config, "system32", None, true, true, false);
        let (_, loaded) = load_session_in(config, "system32");
        let active = loaded.tabs[loaded.active_index().unwrap()]
            .repo_path
            .clone();
        assert_eq!(active, beta);
        // And again from a folder in yet another session.
        restore_launch_tabs_with(config, "profile", None, true, true, false);
        let (_, loaded) = load_session_in(config, "profile");
        assert_eq!(loaded.tabs[loaded.active_index().unwrap()].repo_path, beta);
    }

    /// A start that opens on another tab (`startup_tab`) still records the
    /// launch repository as a tab, but the remembered active tab stays active.
    #[test]
    fn a_launch_that_does_not_activate_keeps_the_remembered_active_tab() {
        let config = tempfile::tempdir().unwrap();
        let config = config.path();
        let work = tempfile::tempdir().unwrap();
        let alpha = repo_dir(work.path(), "alpha");
        let beta = repo_dir(work.path(), "beta");
        restore_launch_tabs_in(config, "s", Some(&alpha), true, false);
        open_tab_in(config, "s", &beta, None);
        assert_eq!(
            tabs_in(config, "s")[active_tab_index_in(config, "s").unwrap()].path,
            beta
        );

        let gamma = repo_dir(work.path(), "gamma");
        restore_launch_tabs_with(config, "s", Some(&gamma), true, false, false);

        let tabs = tabs_in(config, "s");
        assert_eq!(tabs.len(), 3, "the launch repository is still a tab");
        assert_eq!(tabs[active_tab_index_in(config, "s").unwrap()].path, beta);
    }

    #[test]
    fn with_remember_off_a_launch_is_one_tab() {
        let config = tempfile::tempdir().unwrap();
        let config = config.path();
        let work = tempfile::tempdir().unwrap();
        let alpha = repo_dir(work.path(), "alpha");
        let beta = repo_dir(work.path(), "beta");
        restore_launch_tabs_in(config, "a", Some(&alpha), true, true);
        open_tab_in(config, "a", &beta, None);

        restore_launch_tabs_in(config, "b", Some(&beta), false, true);
        let tabs = tabs_in(config, "b");
        assert_eq!(tabs.len(), 1);
        assert_eq!(tabs[0].path, beta);
    }

    /// Outside a repository, with the setting off, nothing comes back.
    #[test]
    fn with_remember_off_a_launch_outside_a_repository_has_no_tab() {
        let config = tempfile::tempdir().unwrap();
        let config = config.path();
        let work = tempfile::tempdir().unwrap();
        let alpha = repo_dir(work.path(), "alpha");
        let beta = repo_dir(work.path(), "beta");
        restore_launch_tabs_in(config, "home", Some(&alpha), true, true);
        open_tab_in(config, "home", &beta, None);
        assert_eq!(tabs_in(config, "home").len(), 2);

        restore_launch_tabs_in(config, "home", None, false, true);
        assert!(tabs_in(config, "home").is_empty());
        // On, the same launch keeps them.
        restore_launch_tabs_in(config, "other", Some(&alpha), true, true);
        restore_launch_tabs_in(config, "other", None, true, true);
        assert_eq!(tabs_in(config, "other").len(), 1);
    }

    /// An explicit `-s` session is its own world: nothing is borrowed
    /// from another session.
    #[test]
    fn an_explicit_session_is_not_seeded_from_the_last_one() {
        let config = tempfile::tempdir().unwrap();
        let config = config.path();
        let work = tempfile::tempdir().unwrap();
        let alpha = repo_dir(work.path(), "alpha");
        let beta = repo_dir(work.path(), "beta");
        restore_launch_tabs_in(config, "a", Some(&alpha), true, true);
        open_tab_in(config, "a", &beta, None);
        restore_launch_tabs_in(config, "named", Some(&beta), true, false);
        assert_eq!(tabs_in(config, "named").len(), 1);
    }

    /// A repository deleted from disk must still be listed, flagged --
    /// the user needs to see why it is unusable, and to be able to
    /// forget it.
    #[test]
    fn a_missing_repository_is_flagged_and_forgettable() {
        let config = tempfile::tempdir().unwrap();
        let config = config.path();
        let work = tempfile::tempdir().unwrap();
        let gone = repo_dir(work.path(), "gone");
        open_tab_in(config, "s", &gone, Some("main".into()));
        std::fs::remove_dir_all(&gone).unwrap();

        let rows = known_repos_in(config, "s");
        assert_eq!(rows.len(), 1);
        assert!(!rows[0].exists);

        forget_recent_in(config, &gone);
        // Still an open tab, so it stays listed -- but the recent entry
        // is gone.
        let rows = known_repos_in(config, "s");
        assert_eq!(rows.len(), 1);
        assert!(rows[0].tab.is_some());
        assert_eq!(rows[0].recent, None);
    }

    /// **The home-directory bug.** `gitpoulpi-tui` started outside any
    /// repository falls back to the working directory, and the startup
    /// path recorded it as a tab unconditionally - which is how
    /// `C:\Users\<name>` ended up in the picker as a repository. A path
    /// that is not one must never reach either file.
    #[test]
    fn a_directory_that_is_not_a_repository_is_never_recorded() {
        let config = tempfile::tempdir().unwrap();
        let config = config.path();
        let home = tempfile::tempdir().unwrap();
        let home = home.path();
        // A plausible home directory: plenty of children, no `.git`.
        std::fs::create_dir_all(home.join("Documents")).unwrap();

        assert!(
            !open_tab_in(config, "s", home, None),
            "a non-repository must be refused"
        );
        assert!(tabs_in(config, "s").is_empty(), "no tab");
        assert!(
            known_repos_in(config, "s").is_empty(),
            "and nothing in the recent list"
        );
    }

    /// A bare repository is a repository. The guard must not be so
    /// strict that it refuses one.
    #[test]
    fn a_bare_repository_is_still_recorded() {
        let config = tempfile::tempdir().unwrap();
        let config = config.path();
        let work = tempfile::tempdir().unwrap();
        let bare = work.path().join("thing.git");
        std::fs::create_dir_all(bare.join("objects")).unwrap();
        std::fs::create_dir_all(bare.join("refs")).unwrap();
        std::fs::write(bare.join("HEAD"), "ref: refs/heads/main\n").unwrap();

        assert!(open_tab_in(config, "s", &bare, None));
        assert_eq!(tabs_in(config, "s").len(), 1);
    }

    /// An entry recorded before the guard existed, or whose `.git` has
    /// since been deleted, is *shown* - flagged unavailable - rather
    /// than silently offered or silently dropped.
    #[test]
    fn a_recent_entry_that_stopped_being_a_repository_is_flagged() {
        let config = tempfile::tempdir().unwrap();
        let config = config.path();
        let work = tempfile::tempdir().unwrap();
        let alpha = repo_dir(work.path(), "alpha");
        open_tab_in(config, "s", &alpha, None);

        std::fs::remove_dir_all(alpha.join(".git")).unwrap();

        let rows = known_repos_in(config, "s");
        assert_eq!(rows.len(), 1);
        assert!(rows[0].exists, "the directory is still there");
        assert!(!rows[0].is_repo, "but it is not a repository any more");
        assert!(!rows[0].available());
    }

    /// The overlay shows how long ago each repository was opened, and
    /// gets it for free from the file it already reads.
    #[test]
    fn a_row_carries_the_time_it_was_last_opened() {
        let config = tempfile::tempdir().unwrap();
        let config = config.path();
        let work = tempfile::tempdir().unwrap();
        let alpha = repo_dir(work.path(), "alpha");
        open_tab_in(config, "s", &alpha, None);

        let opened = RecentRepos::load(&recent_file_in(config))
            .iter()
            .next()
            .expect("a recent entry")
            .last_opened;
        assert!(
            (now_seconds() - opened).abs() < 60,
            "recorded {opened}, now {}",
            now_seconds()
        );
    }

    /// The pane layout round-trips, and a session that never saved one
    /// gets the same defaults `divider1_pct`/`divider2_pct` start at in
    /// `lib.rs`.
    #[test]
    fn pane_layout_persists_across_a_reload_and_defaults_when_never_saved() {
        let config = tempfile::tempdir().unwrap();
        let config = config.path();

        let fresh = layout_in(config, "s");
        assert_eq!(fresh, PaneLayout::default());
        assert_eq!(fresh.divider1_pct, 20);
        assert_eq!(fresh.divider2_pct, 65);
        assert!(!fresh.left_collapsed && !fresh.right_collapsed);

        save_layout_in(
            config,
            "s",
            PaneLayout {
                divider1_pct: 12,
                divider2_pct: 40,
                left_collapsed: true,
                right_collapsed: false,
                sidebar_heights: vec![0, 9, 0, 0, 4],
                commit_message_rows: 6,
                ..PaneLayout::default()
            },
        );

        let reloaded = layout_in(config, "s");
        assert_eq!(reloaded.divider1_pct, 12);
        assert_eq!(reloaded.divider2_pct, 40);
        assert!(reloaded.left_collapsed);
        assert!(!reloaded.right_collapsed);
        assert_eq!(reloaded.sidebar_heights, vec![0, 9, 0, 0, 4]);
        assert_eq!(reloaded.commit_message_rows, 6);

        // A second, unrelated session is unaffected -- the same
        // independence `sessions_have_their_own_tabs_but_share_the_recent_list`
        // pins for tabs.
        assert_eq!(layout_in(config, "other"), PaneLayout::default());
    }

    /// The merge editor's and the folder picker's sizes sit in the same
    /// record; saving one leaves the others alone, and a hand-edited wild
    /// value comes back clamped.
    #[test]
    fn merge_and_picker_sizes_round_trip_and_clamp_on_load() {
        let config = tempfile::tempdir().unwrap();
        let config = config.path();
        let mut layout = layout_in(config, "s");
        layout.sidebar_heights = vec![0, 7];
        layout.merge_cols = vec![2500, 5000, 2500];
        layout.merge_output_pct = 65;
        layout.picker_split_permille = 300;
        save_layout_in(config, "s", layout.clone());
        assert_eq!(layout_in(config, "s"), layout);

        let mut wild = layout;
        wild.merge_output_pct = 4000;
        wild.divider1_pct = 70;
        wild.divider2_pct = 20;
        save_layout_in(config, "s", wild);
        let back = layout_in(config, "s");
        assert_eq!(back.merge_output_pct, 85);
        assert!(back.divider2_pct >= back.divider1_pct + 10);
        assert_eq!(back.sidebar_heights, vec![0, 7]);
    }

    /// Two sessions are independent tab lists over one shared recent
    /// list, which is the shape ADR-0006 describes.
    #[test]
    fn sessions_have_their_own_tabs_but_share_the_recent_list() {
        let config = tempfile::tempdir().unwrap();
        let config = config.path();
        let work = tempfile::tempdir().unwrap();
        let alpha = repo_dir(work.path(), "alpha");
        let beta = repo_dir(work.path(), "beta");

        open_tab_in(config, "one", &alpha, None);
        open_tab_in(config, "two", &beta, None);

        assert_eq!(tabs_in(config, "one").len(), 1);
        assert_eq!(tabs_in(config, "two").len(), 1);
        assert_eq!(tabs_in(config, "one")[0].path, alpha);

        // But each session's management tab offers both, because both
        // were opened on this machine.
        assert_eq!(known_repos_in(config, "one").len(), 2);
        let alpha_row = known_repos_in(config, "two")
            .into_iter()
            .find(|r| r.path == alpha)
            .unwrap();
        assert!(
            alpha_row.tab.is_none(),
            "alpha is not a tab of session `two`"
        );
    }

    /// A star survives a restart -- it is written to `favorites.toml` at
    /// once - and shows up in the management tab's union with what the
    /// other two lists say about the same repository.
    #[test]
    fn a_favorite_persists_and_joins_the_known_repositories_by_path() {
        let config = tempfile::tempdir().unwrap();
        let config = config.path();
        let work = tempfile::tempdir().unwrap();
        let alpha = repo_dir(work.path(), "alpha");
        let beta = repo_dir(work.path(), "beta");
        let starred_only = repo_dir(work.path(), "starred");

        open_tab_in(config, "s", &alpha, Some("main".into()));
        open_tab_in(config, "s", &beta, None);
        close_tabs_in(config, "s", std::slice::from_ref(&beta));
        assert!(toggle_favorite_in(config, &alpha));
        assert!(toggle_favorite_in(config, &starred_only));

        // Read back from disk, as a new process would.
        let favorites = FavoriteRepos::load(&favorites_file_in(config));
        assert!(favorites.contains(&alpha) && favorites.contains(&starred_only));

        let known = known_repos_in(config, "s");
        assert_eq!(known.len(), 3, "joined by path: {known:?}");
        let get = |p: &Path| known.iter().find(|k| k.path == p).unwrap();
        assert_eq!(get(&alpha).tab, Some(0));
        assert_eq!(get(&alpha).favorite, Some(0));
        assert!(get(&alpha).recent.is_some());
        assert_eq!(get(&alpha).branch_hint.as_deref(), Some("main"));
        assert_eq!(get(&beta).tab, None, "closed");
        assert!(get(&beta).recent.is_some());
        assert_eq!(get(&starred_only).favorite, Some(1));
        assert_eq!(get(&starred_only).recent, None);

        // Un-starring persists too.
        assert!(!toggle_favorite_in(config, &alpha));
        assert_eq!(
            known_repos_in(config, "s")
                .iter()
                .find(|k| k.path == alpha)
                .unwrap()
                .favorite,
            None
        );
    }

    #[test]
    fn remove_all_empties_the_recent_list_and_leaves_tabs_and_stars() {
        let config = tempfile::tempdir().unwrap();
        let config = config.path();
        let work = tempfile::tempdir().unwrap();
        let alpha = repo_dir(work.path(), "alpha");
        open_tab_in(config, "s", &alpha, None);
        toggle_favorite_in(config, &alpha);
        clear_recent_in(config);
        let known = known_repos_in(config, "s");
        assert_eq!(known.len(), 1);
        assert_eq!(known[0].recent, None);
        assert_eq!(known[0].tab, Some(0));
        assert_eq!(known[0].favorite, Some(0));
    }

    /// Closing several tabs goes by path: by index, the second close
    /// would hit the tab that slid into the first one's slot.
    #[test]
    fn closing_several_tabs_closes_exactly_those() {
        let config = tempfile::tempdir().unwrap();
        let config = config.path();
        let work = tempfile::tempdir().unwrap();
        let names = ["a", "b", "c", "d"];
        let paths: Vec<PathBuf> = names.iter().map(|n| repo_dir(work.path(), n)).collect();
        for p in &paths {
            open_tab_in(config, "s", p, None);
        }
        close_tabs_in(config, "s", &[paths[1].clone(), paths[2].clone()]);
        let left: Vec<PathBuf> = tabs_in(config, "s").into_iter().map(|t| t.path).collect();
        assert_eq!(left, [paths[0].clone(), paths[3].clone()]);
    }

    #[test]
    fn known_repos_drops_gone_recents_and_duplicates_but_keeps_gone_favorites() {
        let config = tempfile::tempdir().unwrap();
        let config = config.path();
        let work = tempfile::tempdir().unwrap();
        let alpha = repo_dir(work.path(), "alpha");
        let gone = work.path().join("gone");
        let gone_fav = work.path().join("gonefav");

        let mut recents = RecentRepos::default();
        recents.repos.push(gitpoulpi_session::RecentRepo {
            path: alpha.join("."),
            name: "alpha".into(),
            last_opened: 3,
            branch: None,
        });
        recents.record(&gone, "gone", None, 2);
        recents.repos.push(gitpoulpi_session::RecentRepo {
            path: PathBuf::from(format!("{}/", alpha.display())),
            name: "alpha".into(),
            last_opened: 1,
            branch: None,
        });
        recents.save(&recent_file_in(config)).unwrap();
        let mut favs = FavoriteRepos::default();
        favs.toggle(&gone_fav, "gonefav");
        favs.toggle(&alpha, "alpha");
        favs.save(&favorites_file_in(config)).unwrap();

        let rows = known_repos_in(config, "s");
        let names: Vec<_> = rows.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, ["gonefav", "alpha"], "{names:?}");
        assert!(!rows[0].exists && rows[0].favorite.is_some());
        assert_eq!(rows[1].recent, Some(0));
        // Migrated on disk too.
        assert_eq!(RecentRepos::load(&recent_file_in(config)).repos.len(), 1);
    }

    #[test]
    fn forgetting_everywhere_removes_recent_and_favorite_only() {
        let config = tempfile::tempdir().unwrap();
        let config = config.path();
        let work = tempfile::tempdir().unwrap();
        let alpha = repo_dir(work.path(), "alpha");
        open_tab_in(config, "s", &alpha, None);
        toggle_favorite_in(config, &alpha);

        forget_everywhere_in(config, &alpha);
        assert!(FavoriteRepos::load(&favorites_file_in(config))
            .repos
            .is_empty());
        assert!(RecentRepos::load(&recent_file_in(config)).is_empty());
        assert!(alpha.join(".git").exists());
        assert_eq!(tabs_in(config, "s").len(), 1);
    }
}
