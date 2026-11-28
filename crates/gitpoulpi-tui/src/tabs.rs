//! Repository tabs and switching: cycling and closing tabs, applying a
//! tab action, the Repository Management tab's effects, and finding a
//! repository root. The management tab's Browse / Init / Clone dialogs
//! live in `repo_dialogs`.

use super::*;
use gitpoulpi_i18n::msg;

/// The `RepoRef` a window with no repository subscribes with.
///
/// Every pane in this TUI is written around *having* a repository, and a
/// window with none is a narrow, transient state (the first run outside a
/// repository, the moment after the last tab is closed). Rather than make
/// every piece of per-repo state `Option`-aware, the window subscribes to a
/// path that cannot be a repository, so every request fails identically and
/// every pane draws empty; the overlay on top is then the only thing to
/// interact with.
///
/// Deliberately *not* the current directory: it may be the repository the
/// user just closed, and half-populating the panes from it is the bug this
/// exists to prevent.
pub(crate) fn no_repository_ref() -> RepoRef {
    let root = PathBuf::from(NO_REPOSITORY);
    RepoRef {
        id: RepoId::from_path(&root),
        root,
    }
}

pub(crate) use crate::no_repository::NO_REPOSITORY;

/// What the Repository Management tab should do as it opens: just show
/// itself, or also open the Browse / Clone / Init dialog over itself
/// (`RepoDialogs::open`).
///
/// It exists because the management tab is its own window (the
/// no-repository window), so "clone a repository" from a repository window
/// is two steps with a window restart in between that has to carry the
/// second step across.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ManagementEntry {
    Show,
    Browse,
    Clone,
    Init,
    /// The `+` on the tab bar: the same view as a transient "New tab" with
    /// the search field focused.
    NewTab,
    /// The Home tab on its Workspaces page, with the dialog a pending
    /// request asks for (`workspace_ui::apply::take_pending`).
    Workspaces,
    /// The Radar over the Home tab's list.
    Radar,
    /// The Release notes tab over the Home tab's list.
    ReleaseNotes,
}

/// Builds the Repository Management view and starts the background
/// probe over every repository it lists, in display order so the rows on
/// screen fill first. Rows the probe has answered before come up filled
/// in from its cache (`repo_probe::cached`), and are refreshed.
pub(crate) fn open_management(session: &str) -> (repo_manager::RepoManager, repo_probe::Probe) {
    let mut manager = repo_manager::RepoManager::new(session_store::known_repos(session));
    let (entries, active) = crate::workspace_ui::apply::load_page(session);
    manager.ws.set_entries(entries, active);
    manager.ws.epoch = crate::workspace_ui::apply::epoch();
    let mut order: Vec<PathBuf> = Vec::new();
    for row in manager.rows() {
        if let repo_manager::ManagerRow::Repo { repo, .. } = row {
            let known = &manager.repos[repo];
            if known.available() && !order.contains(&known.path) {
                order.push(known.path.clone());
            }
        }
    }
    (manager, repo_probe::start(order))
}

/// Re-reads the three lists after something changed one of them (a star,
/// a closed tab, a forgotten entry). The probe's answers are kept.
pub(crate) fn refresh_management(session: &str, manager: &mut Option<repo_manager::RepoManager>) {
    if let Some(manager) = manager.as_mut() {
        manager.set_repos(session_store::known_repos(session));
        // Anything a workspace action changed is picked up here too.
        crate::workspace_ui::apply::sync_management(session, Some(manager));
    }
}

/// The question asked before a tab is closed.
///
/// It names the repository and says plainly that nothing on disk is
/// touched, which is what the question really asks. The guarantee is real
/// and tested (`closing_a_tab_leaves_the_repository_on_disk` asserts the
/// `.git` directory survives).
pub(crate) fn close_tab_prompt(tabs: &[repo_switcher::RepoTab], index: usize) -> Option<String> {
    let tab = tabs.get(index)?;
    // Four lines, because that is what the shared confirm modal's message
    // area holds at `centered_rect(44, 18)`; a fifth is drawn half-off the
    // bottom border.
    let last = if tabs.len() == 1 {
        msg::tabs::CLOSE_LAST.get()
    } else {
        ""
    };
    Some(msg::tabs::close_prompt(&tab.name, last))
}

/// Whether `key` is the `Ctrl+PageDown`/`Ctrl+PageUp` chord
/// `cycle_repository_tab` is bound to: `Some(true)` for PageDown (forward),
/// `Some(false)` for PageUp (backward), `None` for anything else, including
/// the plain (no-Ctrl) keys the Log's own paging uses. A plain function so a
/// test can call it without a live terminal.
pub(crate) fn repo_tab_cycle_chord(key: &KeyEvent) -> Option<bool> {
    let km = keymap::current();
    // Previous first: `Ctrl+Tab` is lenient about extra modifiers, so it
    // would also swallow `Ctrl+Shift+Tab`.
    if km.is(keymap::Action::GlobalPrevTab, key) {
        Some(false)
    } else if km.is(keymap::Action::GlobalNextTab, key) {
        Some(true)
    } else {
        None
    }
}

/// The tab picker's filtered, ranked rows for `query`: indices into `tabs`,
/// most relevant first. Reuses the command palette's scorer
/// (`palette::score`) so the two "type to narrow a list" popups agree on
/// what "matches" means. An empty query matches every tab, in tab order
/// (`score` returns `Some(0)` for everything and the sort is stable).
pub(crate) fn filter_tab_indices(tabs: &[repo_switcher::RepoTab], query: &str) -> Vec<usize> {
    let query = query.trim().to_lowercase();
    let mut scored: Vec<(i32, usize)> = tabs
        .iter()
        .enumerate()
        .filter_map(|(i, t)| palette::score(&t.name, &query).map(|s| (s, i)))
        .collect();
    scored.sort_by_key(|(s, i)| (-*s, *i));
    scored.into_iter().map(|(_, i)| i).collect()
}

/// The tab picker as it opens: unfiltered, with the row of the tab on screen
/// highlighted (an empty draft lists tabs in tab order, so that row is the
/// active index itself) so Up/Down move from where the owner is. Typing a
/// filter still resets to the best match (`main_field_edited`): whoever types
/// is looking for another tab. The popup derives its scroll window from
/// `selected`, so the row is in view however many tabs there are.
pub(crate) fn open_tab_picker(
    tabs: &[repo_switcher::RepoTab],
    active: usize,
) -> crate::modes::InputMode {
    crate::modes::InputMode::TabPicker {
        draft: String::new(),
        selected: filter_tab_indices(tabs, "")
            .iter()
            .position(|i| *i == active)
            .unwrap_or(0),
    }
}

/// The tab index `Ctrl+PageDown`/`Ctrl+PageUp` (and the palette command)
/// land on, wrapping in both directions: unlike `move_index`, which clamps,
/// a tab bar is a ring like a browser's.
pub(crate) fn next_tab_index(active: usize, len: usize, forward: bool) -> usize {
    let len = len as isize;
    let delta = if forward { 1 } else { -1 };
    (active as isize + delta).rem_euclid(len) as usize
}

/// Moves to the next/previous repository tab. `Ctrl+PageDown`/`Ctrl+PageUp`
/// and the palette's `Repository: Next/Previous Tab`
/// (`PaletteCommand::CycleRepositoryTab`) both go through this one function.
/// A no-op with no tabs open.
#[allow(clippy::too_many_arguments)]
pub(crate) fn cycle_repository_tab(
    forward: bool,
    session: &str,
    link: &DaemonLink,
    current: &RepoRef,
    dialogs: &mut RepoDialogs,
    tabs: &mut Vec<repo_switcher::RepoTab>,
    active_tab: &mut usize,
    switch_to: &mut Option<RepoRef>,
    close_window: &mut bool,
) -> Option<String> {
    if tabs.is_empty() {
        return None;
    }
    let next = next_tab_index(*active_tab, tabs.len(), forward);
    apply_switcher_action(
        repo_switcher::SwitcherAction::Focus(next),
        session,
        link,
        current,
        dialogs,
        tabs,
        active_tab,
        switch_to,
        close_window,
    )
}

/// Moves the repository tab at `from` to index `to` and saves the order in
/// the session file. The one function the bar's drag, the tab picker (keys and
/// drag) and the palette's `Tab: Move Left/Right` all end in.
///
/// Only the repository tabs are in `tabs`: Home, the workspace chip and the
/// Radar are drawn by the bar itself and can neither move nor be passed,
/// so "nothing can land before them" holds by construction. The window keeps
/// showing the same repository, so `active_tab` follows it by path; no
/// repository switch is armed. Returns the status line, `None` for a no-op.
pub(crate) fn reorder_tab(
    session: &str,
    tabs: &mut Vec<repo_switcher::RepoTab>,
    active_tab: &mut usize,
    from: usize,
    to: usize,
) -> Option<String> {
    if from >= tabs.len() || to >= tabs.len() || from == to {
        return None;
    }
    let active = tabs.get(*active_tab).map(|t| t.path.clone());
    let name = tabs[from].name.clone();
    session_store::move_tab(session, from, to);
    *tabs = session_store::tabs(session);
    if let Some(index) = active.and_then(|a| tabs.iter().position(|t| t.path == a)) {
        *active_tab = index;
    }
    Some(msg::tabs::moved(&name, to + 1, tabs.len()))
}

/// The index of the neighbour a tab at `active` swaps with when moved
/// `forward` (right) or back; `None` at the end of the row.
pub(crate) fn tab_move_neighbour(active: usize, len: usize, forward: bool) -> Option<usize> {
    if forward {
        (active + 1 < len).then_some(active + 1)
    } else {
        active.checked_sub(1)
    }
}

/// Carries out one [`repo_switcher::SwitcherAction`], and returns the line
/// to show the user about it, if any.
///
/// Every decision this acts on was made in `repo_switcher`, which is pure
/// and tested; everything here is the effect (writing a session file,
/// starting a clone, arming the repository switch).
///
/// A repository switch is *armed*, not performed: `switch_to` is set and the
/// caller ends the inner loop, which returns to `run` and rebuilds
/// everything for the new repository. Doing it in place would mean
/// rebuilding the `DaemonLink` and all per-repo state from inside a key
/// handler.
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_switcher_action(
    action: repo_switcher::SwitcherAction,
    session: &str,
    link: &DaemonLink,
    current: &RepoRef,
    dialogs: &mut RepoDialogs,
    tabs: &mut Vec<repo_switcher::RepoTab>,
    active_tab: &mut usize,
    switch_to: &mut Option<RepoRef>,
    // Set when the window should end up with no repository at all --
    // the last tab was closed. Distinct from `switch_to`, which names
    // a repository to move to, and from neither being set, which is a
    // quit.
    close_window: &mut bool,
) -> Option<String> {
    use repo_switcher::SwitcherAction as A;
    let _ = link;

    match action {
        A::Open(path) => {
            // Discovery happens on the *new* repository's terms: the root a
            // `.git` walk finds here is the one the daemon keys on, because
            // both go through `repo_ref_for`. A path that is not a repository
            // is reported rather than opened as a tab that can never load.
            if !path.join(".git").exists() && find_repo_root(&path).is_none() {
                return Some(msg::tabs::not_in_repo(path.display()));
            }
            let next = repo_ref_for(path);
            // `ui.tab_dir_follow = "auto"`: a tab the user opens now gets a
            // run of the default "Open in" entry. Never for a tab that
            // already exists (that is a focus) nor for the ones restored at
            // start-up, which do not come through here.
            let brand_new = !tabs.iter().any(|t| t.path == next.root);
            session_store::open_tab(session, &next.root, None);
            if brand_new && tab_dir::auto() {
                // The window restarts on the new tab; a launcher that fails
                // later reports through the late-error queue.
                let _ = tab_dir::auto_open(&next.root);
            }
            *tabs = session_store::tabs(session);
            *active_tab = tabs.iter().position(|t| t.path == next.root).unwrap_or(0);
            if next.id == current.id {
                dialogs.close();
                // Already showing it. Focusing the tab is the whole
                // effect, and restarting the loop for the repository we
                // are already on would flash the screen for nothing.
                session_store::activate_tab(session, *active_tab);
                return Some(msg::tabs::already_on(next.root.display()));
            }
            session_store::activate_tab(session, *active_tab);
            // The window ends here, with a review left open: it is written
            // down on the way out and reopens with the tab.
            *switch_to = Some(next);
            None
        }
        A::Focus(index) => {
            // A stale index is a no-op rather than a panic: it can come
            // from a click on a bar that has since been redrawn.
            let tab = tabs.get(index).cloned()?;
            *active_tab = index;
            session_store::activate_tab(session, index);
            let next = repo_ref_for(tab.path);
            if next.id != current.id {
                // An open review stays: `finish` writes it down.
                *switch_to = Some(next);
            } else {
                dialogs.close();
            }
            None
        }
        A::Close(index) => {
            let tab = tabs.get(index).cloned()?;
            session_store::close_tab(session, index);
            *tabs = session_store::tabs(session);
            *active_tab = (*active_tab).min(tabs.len().saturating_sub(1));

            // Closing the tab that is on screen has to move the window off
            // it, or the tab bar says `no repository open` over a
            // fully-drawn repository and a broken Log.
            if tab.path == current.root {
                match tabs.get(*active_tab).cloned() {
                    Some(next) => *switch_to = Some(repo_ref_for(next.path)),
                    // Nothing left to show. The window stays open, with
                    // no repository, offering the overlay -- the same
                    // shape as starting outside a repository.
                    None => *close_window = true,
                }
            }
            Some(msg::tabs::closed(&tab.name))
        }
    }
}

// -- the Repository Management tab and the repository menu ------------

/// An open repository menu: the tab menu (right-click a tab, `Ctrl+T`)
/// or a management row's `⋮` menu - one state, since only one can be
/// open at a time and both speak [`repo_menu::RepoMenuAction`].
pub(crate) struct RepoMenuState {
    pub target: RepoTarget,
    pub menu: ContextMenu<repo_menu::RepoMenuAction>,
}

/// The repository a menu row (or its palette twin) acts on.
#[derive(Debug, Clone)]
pub(crate) struct RepoTarget {
    pub path: PathBuf,
    pub name: String,
    /// Its tab index, when it is open as a tab.
    pub tab: Option<usize>,
    /// Set on the Workspaces page's menus: the workspace the row is in
    /// (`path` is then the repository's, or empty for the workspace row).
    pub workspace: Option<String>,
}

/// Everything the tab-level actions touch, borrowed from the event
/// loop's locals for the length of one action. A struct rather than the
/// fifteen-argument functions `apply_switcher_action` grew into.
pub(crate) struct TabCtx<'a> {
    pub session: &'a str,
    pub link: &'a DaemonLink,
    pub current: &'a RepoRef,
    pub cwd: &'a Path,
    /// The management tab's Browse / Init / Clone dialogs.
    pub dialogs: &'a mut RepoDialogs,
    pub tabs: &'a mut Vec<repo_switcher::RepoTab>,
    pub active_tab: &'a mut usize,
    pub switch_to: &'a mut Option<RepoRef>,
    pub close_window: &'a mut bool,
    /// Set to leave this repository window for the management tab.
    pub show_management: &'a mut Option<ManagementEntry>,
    pub management: &'a mut Option<repo_manager::RepoManager>,
    pub repo_menu: &'a mut Option<RepoMenuState>,
    pub input_mode: &'a mut InputMode,
    pub clipboard: &'a mut clipboard::Clipboard,
    /// A worktree Create worktree asked to open as a tab, waiting for
    /// the refresh that lists it (`open_pending_worktree_tab`).
    pub pending_worktree_tab: &'a mut Option<PathBuf>,
    /// The active repository's remote UI: the SSH-key pin menu opens on it.
    pub remote_ui: &'a mut RemoteUi,
}

impl TabCtx<'_> {
    pub(crate) fn switcher_action(
        &mut self,
        action: repo_switcher::SwitcherAction,
    ) -> Option<String> {
        apply_switcher_action(
            action,
            self.session,
            self.link,
            self.current,
            self.dialogs,
            self.tabs,
            self.active_tab,
            self.switch_to,
            self.close_window,
        )
    }
}

/// The `⌂`, `Ctrl+O` and the palette's repository entries: go to the
/// management tab, opening `entry`'s dialog there. Already on it, the
/// dialog opens in place; from a repository window, the window ends and
/// the management window starts with the dialog open (see
/// [`ManagementEntry`]).
pub(crate) fn go_to_management(entry: ManagementEntry, ctx: &mut TabCtx) {
    // `general.radar_enabled` off: every way in (tab, `Alt+R`, `3`, the
    // palette, a launch request) ends here, and goes nowhere.
    if entry == ManagementEntry::Radar
        && !crate::build_features::offered(Some(crate::build_features::Feature::Radar))
    {
        return;
    }
    // Without the `workspaces` feature there is no Workspaces tab to go to.
    if entry == ManagementEntry::Workspaces && !crate::build_features::WORKSPACES {
        return;
    }
    // Opening it brings the tab back when it was closed, also from a
    // repository window (which ends and starts Home on it).
    if entry == ManagementEntry::Radar {
        reopen_radar_tab(ctx.session);
    }
    // Same for the Workspaces tab: opening it adds it to the bar.
    if entry == ManagementEntry::Workspaces {
        open_workspaces_tab(ctx.session);
    }
    let Some(manager) = ctx.management.as_mut() else {
        *ctx.show_management = Some(entry);
        return;
    };
    match entry {
        // `+` / `Tab: New Tab` on the Home view: it becomes the New tab.
        ManagementEntry::NewTab => {
            manager.show_page(repo_manager::HomePage::Repositories);
            manager.radar.close();
            manager.notes.close();
            manager.new_tab = true;
            manager.search_focused = true;
        }
        // The `⌂` while a New tab is showing: the New tab is dropped and
        // the Home tab stays.
        ManagementEntry::Show => {
            manager.show_page(repo_manager::HomePage::Repositories);
            manager.new_tab = false;
            // The `⌂` while the Radar shows: back to the repository list.
            manager.radar.close();
            manager.notes.close();
        }
        ManagementEntry::ReleaseNotes => {
            manager.show_page(repo_manager::HomePage::Repositories);
            manager.radar.close();
            manager.new_tab = false;
            manager.notes.open();
            return;
        }
        ManagementEntry::Radar => {
            manager.show_page(repo_manager::HomePage::Repositories);
            manager.new_tab = false;
            manager.notes.close();
            manager.radar.open(ctx.session);
            return;
        }
        ManagementEntry::Workspaces => {
            manager.new_tab = false;
            manager.radar.close();
            manager.notes.close();
            manager.show_page(repo_manager::HomePage::Workspaces);
        }
        _ => {}
    }
    ctx.dialogs.open(entry, ctx.cwd);
}

/// Adds the Workspaces tab to the bar (persisted for the session).
pub(crate) fn open_workspaces_tab(session: &str) {
    session_store::set_workspaces_open(session, true);
    crate::build_features::install_workspaces_open(true);
}

/// Closes the Workspaces tab (its `x`, middle-click, `Ctrl+W`,
/// `Workspaces: Close Tab`) for this session; the view, if showing, goes
/// back to the Home list.
pub(crate) fn close_workspaces_tab(ctx: &mut TabCtx) {
    session_store::set_workspaces_open(ctx.session, false);
    crate::build_features::install_workspaces_open(false);
    if let Some(manager) = ctx.management.as_mut() {
        manager.show_page(repo_manager::HomePage::Repositories);
    }
}

/// Brings the Radar tab back (it stays where it belongs, right after
/// Home): clears the session's "closed" mark.
pub(crate) fn reopen_radar_tab(session: &str) {
    session_store::set_radar_closed(session, false);
    crate::build_features::install_radar_closed(false);
}

/// Closes the Radar tab for this session (its `x`, middle-click,
/// `Ctrl+W`, `Radar: Close tab`). Closing is per session and says
/// nothing about `general.radar_enabled`. The view, if showing, goes
/// back to the Home list.
pub(crate) fn close_radar_tab(ctx: &mut TabCtx) {
    session_store::set_radar_closed(ctx.session, true);
    crate::build_features::install_radar_closed(true);
    if let Some(manager) = ctx.management.as_mut() {
        manager.radar.close();
    }
}

/// Closes the Release notes tab (its `x`, middle-click, `Ctrl+W`, `Esc`,
/// `Release notes: Close tab`): back to the Home list.
pub(crate) fn close_notes_tab(ctx: &mut TabCtx) {
    if let Some(manager) = ctx.management.as_mut() {
        manager.notes.close();
    }
}

/// Closes the transient New tab (its `x`, `Ctrl+W`, middle-click, `Esc`):
/// back to the tab that was active before it (`ManagerAction::Leave`), or,
/// with no repository tab to go back to, to the plain Home tab.
pub(crate) fn close_new_tab(ctx: &mut TabCtx) -> Option<String> {
    if let Some(manager) = ctx.management.as_mut() {
        manager.new_tab = false;
        manager.search_focused = false;
    }
    if ctx.tabs.is_empty() {
        return None;
    }
    apply_manager_action(repo_manager::ManagerAction::Leave, (4, 6), ctx)
}

/// Opens the repository menu for `path`, anchored at `(col, row)`.
pub(crate) fn open_repo_menu(
    path: &Path,
    from_tab_bar: bool,
    (col, row): (u16, u16),
    ctx: &mut TabCtx,
) {
    let known = session_store::known_repos(ctx.session)
        .into_iter()
        .find(|k| k.path == path);
    let tab = ctx.tabs.iter().position(|t| t.path == path);
    let name = known
        .as_ref()
        .map(|k| k.name.clone())
        .or_else(|| tab.map(|i| ctx.tabs[i].name.clone()))
        .unwrap_or_else(|| path.display().to_string());
    let entries = repo_menu::repo_menu_entries(repo_menu::RepoMenuContext {
        tab,
        tab_count: ctx.tabs.len(),
        favorite: known.as_ref().is_some_and(|k| k.favorite.is_some()),
        recent: known.as_ref().is_some_and(|k| k.recent.is_some()),
        from_tab_bar,
        workspace: crate::workspace_ui::chip::active_id()
            .map(|id| crate::workspace_ui::store::is_member(&id, path)),
    });
    *ctx.repo_menu = Some(RepoMenuState {
        target: RepoTarget {
            path: path.to_path_buf(),
            name: name.clone(),
            tab,
            workspace: None,
        },
        menu: ContextMenu::new(name, entries, col, row),
    });
}

/// The tab bar's `+` (and the palette's `Tab: New...`): the small menu of what
/// can be created, hanging at `anchor`. Its rows are the `RepoMenuAction`s
/// that need no repository, so it rides the tab menu's slot, keys and mouse.
pub(crate) fn open_new_menu(anchor: (u16, u16), ctx: &mut TabCtx) {
    *ctx.repo_menu = Some(RepoMenuState {
        target: RepoTarget {
            path: PathBuf::new(),
            name: String::new(),
            tab: None,
            workspace: None,
        },
        menu: ContextMenu::new(
            msg::tab_bar::PLUS_MENU_TITLE.get(),
            repo_menu::new_menu_entries(
                crate::build_features::plus_menu_offers_radar(),
                crate::build_features::plus_menu_offers_workspaces(),
            ),
            anchor.0,
            anchor.1,
        ),
    });
}

/// The repository the palette's `Tab: ...` commands act on: the active
/// tab in a repository window, the selected row on the management tab.
pub(crate) fn current_target(ctx: &TabCtx) -> Option<RepoTarget> {
    match ctx.management.as_ref() {
        Some(manager) => manager.selected_repo().map(|repo| RepoTarget {
            path: repo.path.clone(),
            name: repo.name.clone(),
            tab: repo.tab,
            workspace: None,
        }),
        None => ctx.tabs.get(*ctx.active_tab).map(|tab| RepoTarget {
            path: tab.path.clone(),
            name: tab.name.clone(),
            tab: Some(*ctx.active_tab),
            workspace: None,
        }),
    }
}

/// `Tab: Actions Menu...` (and `Ctrl+T`): the current tab's menu, under
/// its tab.
pub(crate) fn open_current_tab_menu(anchor: (u16, u16), ctx: &mut TabCtx) -> Option<String> {
    match ctx.tabs.get(*ctx.active_tab).map(|t| t.path.clone()) {
        Some(path) if ctx.management.is_none() => {
            open_repo_menu(&path, true, anchor, ctx);
            None
        }
        _ => Some(msg::tabs::NO_TAB.get().into()),
    }
}

/// A `Tab: ...` palette command: one row of the tab menu, run on the
/// current repository without opening the menu.
pub(crate) fn run_tab_menu_command(
    action: repo_menu::RepoMenuAction,
    ctx: &mut TabCtx,
) -> Option<String> {
    match current_target(ctx) {
        Some(target) => apply_repo_menu_action(action, target, ctx),
        None => Some(msg::tabs::NO_REPO_SELECTED.get().into()),
    }
}

/// A `Repositories: ...` palette command, carried out by calling the
/// manager's own operations - the same ones its keys reach through the
/// keymap - so rebinding a Home key never breaks the palette command, and
/// a focused search field never receives the command as typed text.
pub(crate) fn run_management_command(
    command: palette::ManagementCommand,
    ctx: &mut TabCtx,
) -> Option<String> {
    use palette::ManagementCommand as C;
    let manager = ctx.management.as_mut()?;
    let action = match command {
        C::OpenSelected => manager.activate_selected(),
        C::ToggleSelectedFavorite => manager.favorite_selected(),
        C::CloseOrRemoveSelected => manager.close_selected(),
        C::SelectedMenu => manager.menu_selected(),
        C::CopySelectedPath => manager.copy_path_selected(),
        C::OpenSelectedIn => manager.open_in_selected(),
        C::Search => manager.focus_search(),
        C::CollapseAll => {
            manager.set_collapsed_all(true);
            repo_manager::ManagerAction::Handled
        }
        C::ExpandAll => {
            manager.set_collapsed_all(false);
            repo_manager::ManagerAction::Handled
        }
        C::CloseAllTabs => repo_manager::ManagerAction::CloseAllTabs,
        C::RemoveAllRecent => repo_manager::ManagerAction::RemoveAllRecent,
        C::RemoveSelectedFromList => manager.remove_selected_from_list(),
    };
    apply_manager_action(action, (4, 6), ctx)
}

/// Asks before closing `indices`: the shared single-tab question for
/// one, one question naming them all for several.
pub(crate) fn request_close_tabs(indices: Vec<usize>, ctx: &mut TabCtx) -> Option<String> {
    match indices.as_slice() {
        [] => Some(msg::tabs::NO_TAB_TO_CLOSE.get().into()),
        [index] => {
            let prompt = close_tab_prompt(ctx.tabs, *index)?;
            *ctx.input_mode = InputMode::Confirm {
                prompt,
                action: ConfirmAction::CloseTab(*index),
            };
            None
        }
        many => {
            let picked: Vec<&repo_switcher::RepoTab> =
                many.iter().filter_map(|&i| ctx.tabs.get(i)).collect();
            let names: Vec<String> = picked.iter().map(|t| t.name.clone()).collect();
            *ctx.input_mode = InputMode::Confirm {
                prompt: repo_menu::close_tabs_prompt(&names),
                action: ConfirmAction::CloseTabs(picked.iter().map(|t| t.path.clone()).collect()),
            };
            None
        }
    }
}

/// Closes the tabs for `paths` (confirmed), and moves the window off the
/// repository on screen if it was one of them: to the first tab left,
/// or - none left - to the management tab.
#[allow(clippy::too_many_arguments)]
pub(crate) fn close_tabs_now(
    paths: &[PathBuf],
    session: &str,
    current: &RepoRef,
    tabs: &mut Vec<repo_switcher::RepoTab>,
    active_tab: &mut usize,
    switch_to: &mut Option<RepoRef>,
    close_window: &mut bool,
) -> Option<String> {
    session_store::close_tabs(session, paths);
    *tabs = session_store::tabs(session);
    let showing_closed = paths.contains(&current.root);
    match tabs.iter().position(|t| t.path == current.root) {
        Some(index) => *active_tab = index,
        None => *active_tab = (*active_tab).min(tabs.len().saturating_sub(1)),
    }
    if showing_closed {
        match tabs.first() {
            Some(first) => {
                *active_tab = 0;
                session_store::activate_tab(session, 0);
                *switch_to = Some(repo_ref_for(first.path.clone()));
            }
            None => *close_window = true,
        }
    }
    Some(msg::tabs::closed_tabs(paths.len() as u64))
}

fn toggle_favorite_message(path: &Path, name: &str) -> String {
    if session_store::toggle_favorite(path) {
        msg::tabs::favorite_added(name)
    } else {
        msg::tabs::favorite_removed(name)
    }
}

/// Carries out one row of the repository menu.
pub(crate) fn apply_repo_menu_action(
    action: repo_menu::RepoMenuAction,
    target: RepoTarget,
    ctx: &mut TabCtx,
) -> Option<String> {
    use repo_menu::RepoMenuAction as A;
    let message = match action {
        A::Open => match target.tab {
            Some(tab) => ctx.switcher_action(repo_switcher::SwitcherAction::Focus(tab)),
            None => ctx.switcher_action(repo_switcher::SwitcherAction::Open(target.path.clone())),
        },
        A::Close | A::CloseOthers | A::CloseToTheRight => match target.tab {
            Some(tab) => {
                request_close_tabs(repo_menu::tabs_to_close(action, tab, ctx.tabs.len()), ctx)
            }
            None => Some(msg::tabs::not_open_as_tab(&target.name)),
        },
        A::ToggleFavorite => Some(toggle_favorite_message(&target.path, &target.name)),
        A::RemoveFromRecent => {
            session_store::forget_recent(&target.path);
            Some(msg::tabs::removed_from_recent(&target.name))
        }
        A::RemoveFromList => {
            session_store::forget_everywhere(&target.path);
            Some(msg::tabs::removed_from_lists(&target.name))
        }
        A::CopyPath => Some(ctx.clipboard.copy(
            msg::tabs::REPOSITORY_PATH.get(),
            &repo_menu::path_text(&target.path),
        )),
        A::OpenInFileManager => {
            Some(repo_menu::open_in_file_manager(&target.path).unwrap_or_else(|e| e))
        }
        A::OpenInTerminal => Some(repo_menu::open_in_terminal(&target.path).unwrap_or_else(|e| e)),
        A::OpenIn(index) => Some(crate::open_in_bar::run_entry(index, &target.path)),
        A::NewTab => {
            go_to_management(ManagementEntry::NewTab, ctx);
            None
        }
        A::OpenRadar => {
            go_to_management(ManagementEntry::Radar, ctx);
            None
        }
        A::OpenWorkspaces => {
            go_to_management(ManagementEntry::Workspaces, ctx);
            None
        }
        A::AddToWorkspace => crate::workspace_ui::apply::membership(true, &target, ctx),
        A::RemoveFromWorkspace => crate::workspace_ui::apply::membership(false, &target, ctx),
        A::Ws(action) => crate::workspace_ui::apply::apply_menu(action, target, ctx),
        // The pin is a daemon call against the window's own repository,
        // so a row for another repository has to be switched to first.
        A::SshKey => {
            if target.path == ctx.current.root {
                ctx.remote_ui.want_repo_auth = true;
                None
            } else {
                Some(msg::tabs::switch_first(&target.name))
            }
        }
        // Like the pin: the dialog works on the window's own repository.
        A::Sparse => {
            if target.path == ctx.current.root {
                ctx.dialogs.open_sparse();
                None
            } else {
                Some(msg::tabs::switch_first(&target.name))
            }
        }
    };
    refresh_management(ctx.session, ctx.management);
    message
}

/// Carries out one [`repo_manager::ManagerAction`]. `anchor` is where a
/// menu it opens should hang from. `Quit` is the caller's (it ends the
/// loop).
pub(crate) fn apply_manager_action(
    action: repo_manager::ManagerAction,
    anchor: (u16, u16),
    ctx: &mut TabCtx,
) -> Option<String> {
    use repo_manager::ManagerAction as M;
    let message = match action {
        M::Handled | M::NotHandled | M::Quit => return None,
        M::Open(path) => ctx.switcher_action(repo_switcher::SwitcherAction::Open(path)),
        M::FocusTab(index) => ctx.switcher_action(repo_switcher::SwitcherAction::Focus(index)),
        M::CloseTab(index) => request_close_tabs(vec![index], ctx),
        M::CloseAllTabs => request_close_tabs((0..ctx.tabs.len()).collect(), ctx),
        M::ToggleFavorite(path) => {
            let name = ctx
                .management
                .as_ref()
                .and_then(|m| m.repos.iter().find(|r| r.path == path))
                .map(|r| r.name.clone())
                .unwrap_or_else(|| path.display().to_string());
            Some(toggle_favorite_message(&path, &name))
        }
        M::RemoveRecent(path) => {
            session_store::forget_recent(&path);
            Some(msg::tabs::removed_from_recent(path.display()))
        }
        M::RemoveFromList(path) => {
            session_store::forget_everywhere(&path);
            Some(msg::tabs::removed_from_lists_kept(path.display()))
        }
        M::RemoveAllRecent => {
            session_store::clear_recent();
            Some(msg::tabs::CLEARED_RECENT.get().into())
        }
        M::CopyPath(path) => Some(ctx.clipboard.copy(
            msg::tabs::REPOSITORY_PATH.get(),
            &repo_menu::path_text(&path),
        )),
        M::OpenIn(path) => Some(crate::open_in_bar::run_default(&path)),
        M::Browse => {
            go_to_management(ManagementEntry::Browse, ctx);
            None
        }
        M::Clone => {
            go_to_management(ManagementEntry::Clone, ctx);
            None
        }
        M::Init => {
            go_to_management(ManagementEntry::Init, ctx);
            None
        }
        M::Workspace(action) => crate::workspace_ui::apply::apply_action(action, anchor, ctx),
        M::Menu { repo } => {
            let path = ctx
                .management
                .as_ref()
                .and_then(|m| m.repos.get(repo))
                .map(|r| r.path.clone());
            if let Some(path) = path {
                open_repo_menu(&path, false, anchor, ctx);
            }
            None
        }
        M::Leave => {
            if let Some(manager) = ctx.management.as_mut() {
                manager.new_tab = false;
            }
            if ctx.tabs.is_empty() {
                Some(msg::tabs::NO_TAB_OPEN_FROM_LIST.get().into())
            } else {
                let index = (*ctx.active_tab).min(ctx.tabs.len() - 1);
                ctx.switcher_action(repo_switcher::SwitcherAction::Focus(index))
            }
        }
    };
    refresh_management(ctx.session, ctx.management);
    message
}

/// The `RepoRef` this TUI subscribes with, given the directory it was
/// started in.
///
/// Both fields come from the repository *root*, never from the cwd: a
/// `RepoId` is what every frontend keys its daemon subscription on, and the
/// `gitpoulpi` CLI derives it from the root, so a TUI launched in a
/// subdirectory would otherwise never see a CLI's live updates. The one
/// daemon owning all state needs the two to agree by construction.
pub(crate) fn repo_ref_for(cwd: PathBuf) -> RepoRef {
    let root = find_repo_root(&cwd).unwrap_or(cwd);
    RepoRef::from_root(&root)
}

pub(crate) fn find_repo_root(start: &std::path::Path) -> Option<PathBuf> {
    let mut dir = start;
    loop {
        if dir.join(".git").exists() {
            return Some(dir.to_path_buf());
        }
        dir = dir.parent()?;
    }
}

/// "Open worktree in tab": double-click on a Worktrees row, `Enter` on
/// it, and the palette command all land here. Opens the selected
/// worktree's path as its own tab, or focuses the tab it already has --
/// `SwitcherAction::Open` de-duplicates by path, so this is the same flow
/// as opening any other repository.
pub(crate) fn open_worktree_in_tab(
    worktrees: &[gitpoulpi_core::domain::Worktree],
    selected: Option<usize>,
    ctx: &mut TabCtx,
) -> Option<String> {
    let Some(wt) = selected.and_then(|i| worktrees.get(i)) else {
        return Some(msg::tabs::SELECT_WORKTREE.get().into());
    };
    ctx.switcher_action(repo_switcher::SwitcherAction::Open(wt.path.clone()))
}

/// Linked worktree paths any window of this process has seen listed.
/// Kept across window restarts (a tab switch rebuilds the event loop) so a
/// worktree tab can be recognized as one after its worktree is gone.
fn known_linked_worktrees() -> &'static std::sync::Mutex<std::collections::HashSet<PathBuf>> {
    static KNOWN: std::sync::OnceLock<std::sync::Mutex<std::collections::HashSet<PathBuf>>> =
        std::sync::OnceLock::new();
    KNOWN.get_or_init(Default::default)
}

fn normalized(p: &Path) -> PathBuf {
    std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf())
}

/// Indices (descending, so closing them in order keeps the rest valid) of
/// tabs that are worktrees which no longer exist: known linked worktrees
/// that the fresh `listed` no longer contains - trusted only when
/// `listed` still contains `current_root` (an empty or failed list proves
/// nothing) - or whose directory is gone from disk.
pub(crate) fn stale_worktree_tabs(
    tabs: &[repo_switcher::RepoTab],
    listed: &[gitpoulpi_core::domain::Worktree],
    current_root: &Path,
    known: &mut std::collections::HashSet<PathBuf>,
    exists: impl Fn(&Path) -> bool,
) -> Vec<usize> {
    let listed_paths: std::collections::HashSet<PathBuf> =
        listed.iter().map(|w| normalized(&w.path)).collect();
    let list_is_trusted = listed_paths.contains(&normalized(current_root));
    if list_is_trusted {
        known.extend(
            listed
                .iter()
                .filter(|w| !w.is_main)
                .map(|w| normalized(&w.path)),
        );
    }
    let mut stale: Vec<usize> = tabs
        .iter()
        .enumerate()
        .filter(|(_, t)| {
            let path = normalized(&t.path);
            known.contains(&path)
                && (!exists(&t.path) || (list_is_trusted && !listed_paths.contains(&path)))
        })
        .map(|(i, _)| i)
        .collect();
    stale.reverse();
    stale
}

/// Closes every tab whose worktree was removed (by GitPoulpi, by `git
/// worktree remove`, or by deleting the folder). Runs on each refresh.
pub(crate) fn close_stale_worktree_tabs(
    listed: &[gitpoulpi_core::domain::Worktree],
    ctx: &mut TabCtx,
) -> Option<String> {
    let stale = {
        let mut known = known_linked_worktrees()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        stale_worktree_tabs(ctx.tabs, listed, &ctx.current.root, &mut known, |p| {
            p.exists()
        })
    };
    let mut message = None;
    for index in stale {
        let msg = ctx.switcher_action(repo_switcher::SwitcherAction::Close(index));
        if msg.is_some() {
            message = msg;
        }
    }
    message
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wt(path: &str, is_main: bool) -> gitpoulpi_core::domain::Worktree {
        gitpoulpi_core::domain::Worktree {
            path: PathBuf::from(path),
            branch: None,
            is_main,
            is_locked: false,
        }
    }

    #[test]
    fn a_tab_for_a_removed_worktree_is_stale_and_others_are_not() {
        let tabs = vec![tab("main"), tab("wt-a"), tab("wt-b"), tab("other")];
        let tabs: Vec<_> = tabs
            .into_iter()
            .map(|mut t| {
                t.path = PathBuf::from(format!("/nonexistent-gp/{}", t.name));
                t
            })
            .collect();
        let mut known = std::collections::HashSet::new();
        let before = [
            wt("/nonexistent-gp/main", true),
            wt("/nonexistent-gp/wt-a", false),
            wt("/nonexistent-gp/wt-b", false),
        ];
        let root = Path::new("/nonexistent-gp/main");
        // Everything listed and on disk: nothing is stale, and the linked
        // ones are remembered.
        assert!(stale_worktree_tabs(&tabs, &before, root, &mut known, |_| true).is_empty());
        assert_eq!(known.len(), 2);
        // wt-a leaves the list: only its tab goes (indices descending).
        let after = [
            wt("/nonexistent-gp/main", true),
            wt("/nonexistent-gp/wt-b", false),
        ];
        assert_eq!(
            stale_worktree_tabs(&tabs, &after, root, &mut known, |_| true),
            vec![1]
        );
    }

    #[test]
    fn a_worktree_gone_from_disk_is_stale_even_while_still_listed() {
        let mut tabs = vec![tab("main"), tab("wt-a")];
        tabs[0].path = PathBuf::from("/nonexistent-gp/main");
        tabs[1].path = PathBuf::from("/nonexistent-gp/wt-a");
        let mut known = std::collections::HashSet::new();
        let listed = [
            wt("/nonexistent-gp/main", true),
            wt("/nonexistent-gp/wt-a", false),
        ];
        let root = Path::new("/nonexistent-gp/main");
        let exists = |p: &Path| p != Path::new("/nonexistent-gp/wt-a");
        assert_eq!(
            stale_worktree_tabs(&tabs, &listed, root, &mut known, exists),
            vec![1]
        );
    }

    #[test]
    fn an_untrusted_list_never_closes_a_tab_and_unknown_tabs_are_never_touched() {
        let mut tabs = vec![tab("main"), tab("wt-a"), tab("plain")];
        for t in &mut tabs {
            t.path = PathBuf::from(format!("/nonexistent-gp/{}", t.name));
        }
        let mut known = std::collections::HashSet::new();
        known.insert(PathBuf::from("/nonexistent-gp/wt-a"));
        let root = Path::new("/nonexistent-gp/main");
        // A failed refresh gives an empty list: it proves nothing.
        assert!(stale_worktree_tabs(&tabs, &[], root, &mut known, |_| true).is_empty());
        // A trusted list that omits wt-a closes it, but never `plain`,
        // which was never a listed worktree.
        let listed = [wt("/nonexistent-gp/main", true)];
        assert_eq!(
            stale_worktree_tabs(&tabs, &listed, root, &mut known, |_| true),
            vec![1]
        );
    }

    /// A TUI started deep inside a repo must subscribe with the same
    /// `RepoId` as a CLI started at the root, or the daemon treats them as
    /// two unrelated repositories and neither sees the other's mutations.
    /// Derived from the cwd instead of the root, this test reports two
    /// different ids for the same repository.
    #[test]
    fn the_repo_id_comes_from_the_root_so_the_cli_and_the_tui_agree() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path().join("repo");
        let nested = root.join("crates").join("deep");
        std::fs::create_dir_all(root.join(".git")).expect("mkdir .git");
        std::fs::create_dir_all(&nested).expect("mkdir nested");

        let from_root = repo_ref_for(root.clone());
        let from_nested = repo_ref_for(nested);

        assert_eq!(from_nested.id, from_root.id);
        assert_eq!(from_nested.root, from_root.root);
        assert_eq!(
            from_root.id,
            gitpoulpi_core::domain::RepoId::from_path(&root)
        );
    }

    /// A tab named `name` under `/repos`.
    fn tab(name: &str) -> repo_switcher::RepoTab {
        repo_switcher::RepoTab {
            path: PathBuf::from(format!("/repos/{name}")),
            name: name.to_string(),
            branch: None,
        }
    }

    /// The question a tab close asks. It has to say plainly that
    /// nothing on disk changes - that is what the question is really
    /// asking, and the guarantee is real (the repository survives a
    /// close; `gitpoulpi-cli`'s `multi_repo` suite asserts its `.git`
    /// is still there afterwards).
    #[test]
    fn the_close_tab_prompt_names_the_tab_and_promises_the_disk_is_untouched() {
        let tabs = [tab("alpha"), tab("beta")];
        let prompt = close_tab_prompt(&tabs, 1).expect("a prompt");
        assert!(prompt.contains("beta"), "{prompt}");
        assert!(prompt.contains("not touched on disk"), "{prompt}");
        assert!(
            !prompt.contains("last tab"),
            "there is another tab: {prompt}"
        );
    }

    /// Closing the last tab leaves the window with no repository, and
    /// the prompt says so before it happens rather than after.
    #[test]
    fn the_close_tab_prompt_warns_when_it_is_the_last_one() {
        let prompt = close_tab_prompt(&[tab("only")], 0).expect("a prompt");
        assert!(prompt.contains("last tab"), "{prompt}");
    }

    /// A stale index - from a click on a bar that has since been
    /// redrawn - asks nothing rather than asking about nothing.
    #[test]
    fn a_stale_index_produces_no_prompt() {
        assert!(close_tab_prompt(&[tab("only")], 7).is_none());
    }

    /// The prompt has to fit the shared confirm modal's message area,
    /// which is four lines at `centered_rect(44, 18)`. A fifth line is
    /// drawn half-off the bottom border.
    #[test]
    fn the_close_tab_prompt_fits_the_confirm_modal() {
        let frame = Rect::new(0, 0, 120, 40);
        let popup = centered_rect(44, 18, frame);
        let message_height = popup.height.saturating_sub(2).saturating_sub(1) as usize;
        let width = popup.width.saturating_sub(4) as usize;

        for tabs in [vec![tab("made-repo")], vec![tab("a"), tab("b")]] {
            let prompt = close_tab_prompt(&tabs, 0).unwrap();
            // The modal wraps, so count wrapped lines, not `\n`s.
            let lines: usize = prompt
                .split('\n')
                .map(|line| {
                    if line.is_empty() {
                        1
                    } else {
                        display_width(line).div_ceil(width.max(1))
                    }
                })
                .sum();
            assert!(
                lines <= message_height,
                "{lines} lines in {message_height}: {prompt}"
            );
        }
    }

    /// A window with no repository must not subscribe to the directory
    /// it happens to be in: that may well be the repository just
    /// closed, and half-populating the panes from it is the bug the
    /// state exists to end.
    #[test]
    fn the_no_repository_ref_is_not_a_real_path() {
        let placeholder = no_repository_ref();
        assert_eq!(placeholder.root, PathBuf::from(NO_REPOSITORY));
        assert!(!placeholder.root.exists());
        assert_ne!(
            placeholder.root,
            std::env::current_dir().unwrap(),
            "never the working directory"
        );
    }

    /// `Ctrl+PageDown`/`Ctrl+PageUp` is the chord `cycle_repository_tab` is
    /// bound to. Red without the `CONTROL` check: the Log's own plain
    /// PageUp/PageDown paging would report `Some` too and the two features
    /// would fight over the key.
    #[test]
    fn the_repo_tab_cycle_chord_requires_control_and_only_page_keys() {
        let ctrl_page_down = KeyEvent::new(KeyCode::PageDown, KeyModifiers::CONTROL);
        let ctrl_page_up = KeyEvent::new(KeyCode::PageUp, KeyModifiers::CONTROL);
        assert_eq!(repo_tab_cycle_chord(&ctrl_page_down), Some(true));
        assert_eq!(repo_tab_cycle_chord(&ctrl_page_up), Some(false));

        // Ctrl+Tab / Ctrl+Shift+Tab, in every spelling a terminal uses.
        let m = KeyModifiers::CONTROL;
        assert_eq!(
            repo_tab_cycle_chord(&KeyEvent::new(KeyCode::Tab, m)),
            Some(true)
        );
        assert_eq!(
            repo_tab_cycle_chord(&KeyEvent::new(KeyCode::BackTab, m)),
            Some(false)
        );
        assert_eq!(
            repo_tab_cycle_chord(&KeyEvent::new(KeyCode::BackTab, m | KeyModifiers::SHIFT)),
            Some(false)
        );
        assert_eq!(
            repo_tab_cycle_chord(&KeyEvent::new(KeyCode::Tab, m | KeyModifiers::SHIFT)),
            Some(false)
        );
        // Plain Tab keeps cycling focus.
        assert_eq!(
            repo_tab_cycle_chord(&KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)),
            None
        );

        // The Log's own paging: no Ctrl, must not be claimed by this chord.
        let plain_page_down = KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE);
        assert_eq!(repo_tab_cycle_chord(&plain_page_down), None);

        // Any other Ctrl+key is not this chord.
        let ctrl_w = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL);
        assert_eq!(repo_tab_cycle_chord(&ctrl_w), None);

        // A key-up event must not double-fire the cycle.
        let mut release = KeyEvent::new(KeyCode::PageDown, KeyModifiers::CONTROL);
        release.kind = KeyEventKind::Release;
        assert_eq!(repo_tab_cycle_chord(&release), None);
    }

    /// An empty query keeps every tab, in tab order - the picker opens
    /// showing the full list, not an empty one.
    #[test]
    fn filter_tab_indices_with_an_empty_query_keeps_every_tab_in_order() {
        let tabs = [tab("alpha"), tab("beta"), tab("gamma")];
        assert_eq!(filter_tab_indices(&tabs, ""), vec![0, 1, 2]);
    }

    /// A query narrows the list to tabs whose name matches, dropping the
    /// rest - the whole point of typing to filter.
    #[test]
    fn filter_tab_indices_narrows_to_matching_tabs() {
        let tabs = [tab("gitpoulpi"), tab("herdr"), tab("gitpoulpi-testfixture")];
        assert_eq!(filter_tab_indices(&tabs, "git"), vec![0, 2]);
    }

    /// The match is case-insensitive, like the command palette's.
    #[test]
    fn filter_tab_indices_is_case_insensitive() {
        let tabs = [tab("GitPoulpi")];
        assert_eq!(filter_tab_indices(&tabs, "GITPOULPI"), vec![0]);
    }

    /// A query nothing matches leaves an empty list rather than falling
    /// back to "show everything" - the picker's own render must then say
    /// "no matching tab", not silently ignore the filter.
    #[test]
    fn filter_tab_indices_with_no_match_is_empty() {
        let tabs = [tab("alpha"), tab("beta")];
        assert!(filter_tab_indices(&tabs, "zzz").is_empty());
    }

    /// The picker opens on the active tab, and with many tabs that row is
    /// inside the drawn window (`window_start` keeps `selected` visible).
    #[test]
    fn the_tab_picker_opens_on_the_active_tab_in_view() {
        let tabs: Vec<_> = (0..40).map(|i| tab(&format!("repo{i:02}"))).collect();
        let crate::modes::InputMode::TabPicker { draft, selected } = open_tab_picker(&tabs, 33)
        else {
            panic!("not a tab picker");
        };
        assert!(draft.is_empty());
        assert_eq!(selected, 33);
        let rows = 10;
        let start = crate::popup_list::View::window_start(selected, rows, tabs.len());
        assert!((start..start + rows).contains(&selected), "start {start}");
        let crate::modes::InputMode::TabPicker { selected, .. } = open_tab_picker(&tabs, 99) else {
            panic!()
        };
        assert_eq!(selected, 0, "a stale active index falls back to the top");
    }

    /// `Ctrl+PageDown`/`Ctrl+PageUp` cycle the tab bar as a ring: past the
    /// last tab wraps to the first and back past the first wraps to the last.
    /// Red if `rem_euclid` becomes `%`: a negative `isize` remainder would
    /// panic the `as usize` cast instead of landing on the last tab.
    #[test]
    fn next_tab_index_wraps_in_both_directions() {
        assert_eq!(next_tab_index(0, 3, true), 1);
        assert_eq!(
            next_tab_index(2, 3, true),
            0,
            "forward past the end wraps to the start"
        );
        assert_eq!(
            next_tab_index(0, 3, false),
            2,
            "backward past the start wraps to the end"
        );
        assert_eq!(next_tab_index(1, 3, false), 0);
        assert_eq!(
            next_tab_index(0, 1, true),
            0,
            "a single tab cycles to itself"
        );
    }
}
