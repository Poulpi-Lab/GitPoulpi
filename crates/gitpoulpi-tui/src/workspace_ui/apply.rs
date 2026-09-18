//! Carrying out what the Home page, the chip, the menus and the palette ask
//! of workspaces. One place, so the mouse path and the keyboard / palette
//! path cannot drift (the rule the tab menu already follows).
//!
//! Everything here is quick: files, no daemon. What needs the daemon (an
//! origin URL, Fetch all) is started as a `task` job and reported on by
//! `RepoDialogs::poll`.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use gitpoulpi_i18n::msg::workspaces as m;

use super::chip::{self, Pick};
use super::dialog::Seed;
use super::store;
use super::task::{self, Bulk, Missing};
use crate::modes::{ConfirmAction, InputMode};
use crate::palette::WorkspaceCommand;
use crate::repo_manager::{WsAction, WsEntry, WsRepo};
use crate::repo_menu::WsMenu;
use crate::tabs::{
    go_to_management, repo_ref_for, ManagementEntry, RepoMenuState, RepoTarget, TabCtx,
};
use crate::ContextMenu;
use crate::{repo_switcher, session_store};

/// Bumped by anything that changed `workspaces.toml` or the active
/// workspace *in this process*; a Home window reloads its page when it moved.
static EPOCH: AtomicU64 = AtomicU64::new(1);

pub(crate) fn changed() {
    EPOCH.fetch_add(1, Ordering::SeqCst);
    // The commit author can depend on it (the workspace's default profile).
    crate::profile_bar::refresh_workspace();
}

pub(crate) fn epoch() -> u64 {
    EPOCH.load(Ordering::SeqCst)
}

/// What the Home tab should open once it is showing. Process-wide because
/// the window restarts on the way to the Home tab (`ManagementEntry`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Pending {
    New,
    /// New workspace with the open tabs checked.
    FromTabs,
    Edit(String),
}

static PENDING: Mutex<Option<Pending>> = Mutex::new(None);

fn request(p: Pending) {
    *PENDING.lock().unwrap_or_else(|e| e.into_inner()) = Some(p);
}

pub(crate) fn take_pending() -> Option<Pending> {
    PENDING.lock().unwrap_or_else(|e| e.into_inner()).take()
}

/// `[[profiles]]` as (id, name).
pub(crate) fn profiles() -> Vec<(String, String)> {
    gitpoulpi_config::Settings::resolve_ambient()
        .map(|s| {
            s.config()
                .profiles
                .iter()
                .map(|p| (p.id.clone(), p.name.clone()))
                .collect()
        })
        .unwrap_or_default()
}

/// The page's rows from `workspaces.toml`; the profile names resolved.
pub(crate) fn entries(
    all: &gitpoulpi_session::Workspaces,
    profiles: &[(String, String)],
) -> Vec<WsEntry> {
    all.workspaces
        .iter()
        .map(|w| WsEntry {
            id: w.id.clone(),
            name: w.name.clone(),
            color: w.color.clone(),
            description: w.description.clone(),
            profile: w.profile.as_deref().and_then(|id| {
                profiles
                    .iter()
                    .find(|(p, _)| p == id)
                    .map(|(_, name)| name.clone())
            }),
            repos: w
                .repos
                .iter()
                .map(|p| WsRepo {
                    path: p.clone(),
                    name: p
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| p.display().to_string()),
                    present: session_store::is_repository(p),
                    remote: w.remote_of(p).map(str::to_string),
                })
                .collect(),
        })
        .collect()
}

/// Loads the Home page's data for `session`.
pub(crate) fn load_page(session: &str) -> (Vec<WsEntry>, Option<String>) {
    let all = store::load();
    let active = store::active(session).map(|w| w.id);
    (entries(&all, &profiles()), active)
}

/// Reloads a Home window's page when something in this process changed it.
pub(crate) fn sync_management(
    session: &str,
    manager: Option<&mut crate::repo_manager::RepoManager>,
) {
    let Some(manager) = manager else { return };
    let now = epoch();
    if manager.ws.epoch == now {
        return;
    }
    manager.ws.epoch = now;
    let (entries, active) = load_page(session);
    manager.ws.set_entries(entries, active);
}

/// The form's seed for `pending`.
pub(crate) fn seed(pending: Pending, session: &str) -> Option<Seed> {
    let all = store::load();
    let known: Vec<(PathBuf, String)> = session_store::known_repos(session)
        .into_iter()
        .map(|k| (k.path, k.name))
        .collect();
    let profiles = profiles();
    // The AI instructions row exists only while AI is on.
    let ai_enabled = crate::build_features::AI
        && gitpoulpi_config::Settings::resolve_ambient().is_ok_and(|s| s.config().ai.enabled);
    let others = |keep: Option<&str>| -> Vec<String> {
        all.workspaces
            .iter()
            .filter(|w| Some(w.id.as_str()) != keep)
            .map(|w| w.name.clone())
            .collect()
    };
    Some(match pending {
        Pending::New => Seed {
            id: None,
            name: String::new(),
            color: None,
            description: String::new(),
            profile: None,
            members: Vec::new(),
            known,
            profiles,
            taken_names: others(None),
            ai_enabled,
            ai: Default::default(),
        },
        Pending::FromTabs => Seed {
            id: None,
            name: String::new(),
            color: None,
            description: String::new(),
            profile: None,
            members: session_store::tabs(session)
                .into_iter()
                .map(|t| t.path)
                .collect(),
            known,
            profiles,
            taken_names: others(None),
            ai_enabled,
            ai: Default::default(),
        },
        Pending::Edit(id) => {
            let w = all.get(&id)?;
            Seed {
                id: Some(w.id.clone()),
                name: w.name.clone(),
                color: w.color.clone(),
                description: w.description.clone(),
                profile: w.profile.clone(),
                members: w.repos.clone(),
                known,
                profiles,
                taken_names: others(Some(&id)),
                ai_enabled,
                ai: w.ai(),
            }
        }
    })
}

/// The folder picker's title for Locate...
pub(crate) fn locate_title(old: &Path) -> String {
    m::locate_title(
        old.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| old.display().to_string()),
    )
}

/// Locate...: the repository of workspace `id` that was at `old` is at `new`.
pub(crate) fn locate_now(id: &str, old: &Path, new: &Path) -> String {
    match store::replace_repo(id, old, new) {
        Some(true) => {
            changed();
            m::located(new.display())
        }
        _ => m::save_failed(old.display()),
    }
}

fn name_of(id: &str) -> String {
    store::load()
        .get(id)
        .map(|w| w.name.clone())
        .unwrap_or_else(|| id.to_string())
}

/// Moves the window to workspace `target` (`None` = no workspace): the tab
/// list swaps (`store::switch`) and the window follows it, like a tab switch.
/// `go_to_tabs`: leave the Home tab for the workspace's active tab (the Open
/// button); off, a window on Home stays there (the chip's choice).
pub(crate) fn switch_to(
    target: Option<&str>,
    go_to_tabs: bool,
    ctx: &mut TabCtx,
) -> Option<String> {
    let moved = store::switch(ctx.session, target)?;
    chip::refresh(ctx.session);
    changed();
    *ctx.tabs = session_store::tabs(ctx.session);
    let on_home = ctx.current.root == std::path::Path::new(crate::tabs::NO_REPOSITORY);
    let message = match &moved.now {
        Some(id) => m::switched(name_of(id)),
        None => m::LEFT.get().to_string(),
    };
    if ctx.tabs.is_empty() {
        // Closing every tab keeps the workspace and shows the Home tab.
        *ctx.active_tab = 0;
        if !on_home {
            *ctx.close_window = true;
        }
    } else if on_home && !go_to_tabs {
        *ctx.active_tab = session_store::active_tab_index(ctx.session)
            .unwrap_or(0)
            .min(ctx.tabs.len() - 1);
    } else {
        let index = session_store::active_tab_index(ctx.session)
            .unwrap_or(0)
            .min(ctx.tabs.len() - 1);
        *ctx.active_tab = index;
        ctx.dialogs.close();
        let next = repo_ref_for(ctx.tabs[index].path.clone());
        if next.id != ctx.current.id {
            *ctx.switch_to = Some(next);
        }
    }
    crate::tabs::refresh_management(ctx.session, ctx.management);
    Some(message)
}

/// A pick in the tab bar's workspace combo.
pub(crate) fn apply_pick(pick: Pick, ctx: &mut TabCtx) -> Option<String> {
    match pick {
        Pick::Switch(target) => switch_to(target.as_deref(), false, ctx),
        Pick::Manage => {
            go_to_management(ManagementEntry::Workspaces, ctx);
            None
        }
    }
}

/// A confirmed delete: leaves the workspace first when the window is in it,
/// forgets what it parked, removes it from the file. The repositories are
/// not touched.
#[allow(clippy::too_many_arguments)]
pub(crate) fn delete_now(
    id: &str,
    session: &str,
    current: &gitpoulpi_core::domain::RepoRef,
    tabs: &mut Vec<repo_switcher::RepoTab>,
    active_tab: &mut usize,
    switch_to: &mut Option<gitpoulpi_core::domain::RepoRef>,
    close_window: &mut bool,
) -> Option<String> {
    let name = name_of(id);
    let left = store::forget(session, id);
    let mut all = store::load();
    all.delete(id);
    store::save(&all);
    chip::refresh(session);
    changed();
    if left {
        *tabs = session_store::tabs(session);
        let on_home = current.root == std::path::Path::new(crate::tabs::NO_REPOSITORY);
        *active_tab = session_store::active_tab_index(session)
            .unwrap_or(0)
            .min(tabs.len().saturating_sub(1));
        match tabs.get(*active_tab) {
            // The tab on screen is still there: nothing to move.
            Some(_) if tabs.iter().any(|t| t.path == current.root) => {}
            Some(tab) if !on_home => *switch_to = Some(repo_ref_for(tab.path.clone())),
            None if !on_home => *close_window = true,
            _ => {}
        }
    }
    Some(m::deleted(name))
}

fn confirm_delete(id: &str, ctx: &mut TabCtx) -> Option<String> {
    let all = store::load();
    let ws = all.get(id)?;
    *ctx.input_mode = InputMode::Confirm {
        prompt: m::delete_prompt(&ws.name),
        action: ConfirmAction::DeleteWorkspace(id.to_string()),
    };
    None
}

fn existing_repos(id: &str) -> Vec<PathBuf> {
    store::load()
        .get(id)
        .map(|w| w.repos.clone())
        .unwrap_or_default()
}

/// Runs a walk over workspace `id`'s repositories.
fn start_bulk(id: &str, kind: Bulk, ctx: &mut TabCtx) -> Option<String> {
    let name = name_of(id);
    let repos = existing_repos(id);
    if repos.is_empty() {
        return Some(m::no_repositories(name));
    }
    if ctx.dialogs.ws_busy() {
        return Some(m::BUSY.get().into());
    }
    let line = match kind {
        Bulk::Fetch => m::fetching_progress(&name, 0, repos.len() as u64),
        Bulk::Pull => m::pulling_progress(&name, 0, repos.len() as u64),
    };
    ctx.dialogs.start_job(
        task::bulk(ctx.session.to_string(), kind, name.clone(), repos),
        &name,
    );
    Some(line)
}

fn start_clone(id: &str, only: Option<&Path>, ctx: &mut TabCtx) -> Option<String> {
    let all = store::load();
    let ws = all.get(id)?;
    let missing: Vec<Missing> = ws
        .repos
        .iter()
        .filter(|p| !session_store::is_repository(p))
        .filter(|p| match only {
            Some(o) => gitpoulpi_session::same_repo_path(o, p),
            None => true,
        })
        .map(|p| Missing {
            path: p.clone(),
            url: ws.remote_of(p).map(str::to_string),
        })
        .collect();
    if missing.is_empty() {
        return Some(m::NOTHING_TO_CLONE.get().into());
    }
    if ctx.dialogs.ws_busy() {
        return Some(m::BUSY.get().into());
    }
    let line = m::cloning_progress(&ws.name, 0, missing.len() as u64);
    ctx.dialogs.start_job(
        task::clone_missing(ctx.session.to_string(), ws.name.clone(), missing),
        &ws.name,
    );
    Some(line)
}

fn open_menu(id: &str, repo: Option<PathBuf>, anchor: (u16, u16), ctx: &mut TabCtx) {
    let all = store::load();
    let Some(ws) = all.get(id) else { return };
    let entries = super::menu::entries(
        ws.repos.iter().any(|p| !session_store::is_repository(p)),
        repo.as_ref()
            .map(|p| (session_store::is_repository(p), ws.remote_of(p).is_some())),
    );
    let name = match &repo {
        Some(p) => p
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| p.display().to_string()),
        None => ws.name.clone(),
    };
    *ctx.repo_menu = Some(RepoMenuState {
        target: RepoTarget {
            path: repo.unwrap_or_default(),
            name: name.clone(),
            tab: None,
            workspace: Some(id.to_string()),
        },
        menu: ContextMenu::new(name, entries, anchor.0, anchor.1),
    });
}

/// One request of the Home page.
pub(crate) fn apply_action(
    action: WsAction,
    anchor: (u16, u16),
    ctx: &mut TabCtx,
) -> Option<String> {
    match action {
        WsAction::Open(id) => switch_to(Some(&id), true, ctx),
        WsAction::New => {
            request(Pending::New);
            go_to_management(ManagementEntry::Workspaces, ctx);
            None
        }
        WsAction::FromTabs => {
            if ctx.tabs.is_empty() {
                return Some(m::NO_TABS.get().into());
            }
            request(Pending::FromTabs);
            go_to_management(ManagementEntry::Workspaces, ctx);
            None
        }
        WsAction::Edit(id) => {
            request(Pending::Edit(id));
            go_to_management(ManagementEntry::Workspaces, ctx);
            None
        }
        WsAction::Delete(id) => confirm_delete(&id, ctx),
        WsAction::FetchAll(id) => start_bulk(&id, Bulk::Fetch, ctx),
        WsAction::PullAll(id) => start_bulk(&id, Bulk::Pull, ctx),
        WsAction::CloneMissing(id) => start_clone(&id, None, ctx),
        WsAction::CloneRepo { id, path } => start_clone(&id, Some(&path), ctx),
        WsAction::OpenRepo(path) => ctx.switcher_action(repo_switcher::SwitcherAction::Open(path)),
        WsAction::Locate { id, path } => {
            ctx.dialogs.open_locate(id, path);
            None
        }
        WsAction::RemoveRepo { id, path } => {
            let name = name_of(&id);
            let shown = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            match store::remove_repo(&id, &path) {
                Some(true) => {
                    changed();
                    Some(m::removed_from(shown, name))
                }
                _ => Some(m::save_failed(path.display())),
            }
        }
        WsAction::Menu { id, repo } => {
            open_menu(&id, repo, anchor, ctx);
            None
        }
    }
}

/// A row of a workspace's menu.
pub(crate) fn apply_menu(action: WsMenu, target: RepoTarget, ctx: &mut TabCtx) -> Option<String> {
    let id = target.workspace?;
    let path = target.path;
    let to = match action {
        WsMenu::Open => WsAction::Open(id),
        WsMenu::Edit => WsAction::Edit(id),
        WsMenu::Delete => WsAction::Delete(id),
        WsMenu::FetchAll => WsAction::FetchAll(id),
        WsMenu::PullAll => WsAction::PullAll(id),
        WsMenu::CloneMissing => WsAction::CloneMissing(id),
        WsMenu::OpenRepo => WsAction::OpenRepo(path),
        WsMenu::Locate => WsAction::Locate { id, path },
        WsMenu::CloneRepo => WsAction::CloneRepo { id, path },
        WsMenu::RemoveRepo => WsAction::RemoveRepo { id, path },
    };
    apply_action(to, (4, 6), ctx)
}

/// `Add to workspace` / `Remove from workspace` on a tab or a row: the active
/// workspace gets (or loses) that repository.
pub(crate) fn membership(add: bool, target: &RepoTarget, ctx: &mut TabCtx) -> Option<String> {
    let Some(id) = chip::active_id() else {
        return Some(m::NO_WORKSPACE_ACTIVE.get().into());
    };
    let name = name_of(&id);
    if add {
        if ctx.dialogs.ws_busy() {
            return Some(m::BUSY.get().into());
        }
        ctx.dialogs.start_job(
            task::add_with_origin(ctx.session.to_string(), id, target.path.clone()),
            &name,
        );
        None
    } else {
        match store::remove_repo(&id, &target.path) {
            Some(true) => {
                changed();
                Some(m::removed_from(&target.name, name))
            }
            Some(false) => Some(m::not_in(&target.name, name)),
            None => Some(m::save_failed(target.path.display())),
        }
    }
}

/// A `Workspace: ...` palette command.
pub(crate) fn run_command(cmd: WorkspaceCommand, ctx: &mut TabCtx) -> Option<String> {
    use WorkspaceCommand as C;
    // The workspace a command acts on: the Home page's selection, else the
    // window's active one.
    let target = |ctx: &TabCtx| -> Option<String> {
        ctx.management
            .as_ref()
            .filter(|m| m.page == crate::repo_manager::HomePage::Workspaces)
            .and_then(|m| m.ws.selected_workspace().map(|e| e.id.clone()))
            .or_else(chip::active_id)
    };
    let on_target = |ctx: &mut TabCtx, make: fn(String) -> WsAction| match target(ctx) {
        Some(id) => apply_action(make(id), (4, 6), ctx),
        None => Some(m::SELECT_FIRST.get().into()),
    };
    match cmd {
        C::New => apply_action(WsAction::New, (4, 6), ctx),
        C::FromTabs => apply_action(WsAction::FromTabs, (4, 6), ctx),
        C::Switch => {
            chip::open();
            None
        }
        C::Close => switch_to(None, false, ctx),
        C::Manage => {
            go_to_management(ManagementEntry::Workspaces, ctx);
            None
        }
        C::Locate => match ctx.management.as_mut() {
            Some(m) => {
                let action = m.ws_locate_selected();
                match action {
                    crate::repo_manager::ManagerAction::Workspace(a) => {
                        apply_action(a, (4, 6), ctx)
                    }
                    _ => m.message.clone(),
                }
            }
            None => Some(m::SELECT_REPOSITORY_FIRST.get().into()),
        },
        C::CloseTab => {
            crate::tabs::close_workspaces_tab(ctx);
            None
        }
        // On the Workspaces page a selected missing repository is cloned
        // alone (its Clone button); otherwise every missing one is.
        C::CloneMissing => {
            let one = ctx.management.as_ref().and_then(|m| {
                m.ws.selected_repo()
                    .filter(|(_, r)| !r.present)
                    .map(|(e, r)| (e.id.clone(), r.path.clone()))
            });
            match one {
                Some((id, path)) => apply_action(WsAction::CloneRepo { id, path }, (4, 6), ctx),
                None => on_target(ctx, WsAction::CloneMissing),
            }
        }
        C::Edit => on_target(ctx, WsAction::Edit),
        C::Delete => on_target(ctx, WsAction::Delete),
        C::FetchAll => on_target(ctx, WsAction::FetchAll),
        C::PullAll => on_target(ctx, WsAction::PullAll),
        C::AddTab | C::RemoveTab => match crate::tabs::current_target(ctx) {
            Some(tab) => membership(cmd == C::AddTab, &tab, ctx),
            None => Some(gitpoulpi_i18n::msg::tabs::NO_REPO_SELECTED.get().into()),
        },
    }
}
