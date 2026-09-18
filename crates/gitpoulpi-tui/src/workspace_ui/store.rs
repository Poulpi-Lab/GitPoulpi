//! Workspaces on disk, from the TUI's side: `workspaces.toml` (the user's
//! groups of repositories) and the per-session bookkeeping of which one is
//! active and what each remembers of its tabs.
//!
//! Plain file reads and writes through `gitpoulpi-session`, no IPC (the
//! boundary of `session-and-state.md`), and **best effort** like
//! `session_store`: an unwritable config directory is a lost edit, not a
//! refusal to start. Every public function has a `*_in(config_dir, ..)` twin
//! the tests drive.
//!
//! **Without the `workspaces` feature** the readers below see nothing: no
//! workspace exists and none is active, so every surface that follows them
//! (the chip, the profile bar, the AI layers, the Radar and search scopes)
//! behaves as in "No workspace", and `workspaces.toml` is neither read nor
//! written by the window.
//!
//! **Tab lists.** `GlobalSession::tabs` is always the *live* list: what the
//! window shows, what `gitpoulpi repo open` appends to, what every existing
//! `session_store` function edits. A workspace swap therefore *moves* lists:
//! the live one is parked in `workspace_tabs[<leaving>]` and the target's is
//! taken out of `workspace_tabs[<target>]` (its repositories, the first time).
//! "No workspace" has no id, so its parked list lives under [`NONE_KEY`]; no
//! slug can be that (`slugify` never starts with a dash).

use std::path::Path;

use gitpoulpi_session::{same_repo_path, TabEntry, Workspace, WorkspaceTabs, Workspaces};

use crate::session_store::{config_dir, is_repository, load_session_in};

/// The `workspace_tabs` key the "no workspace" tab list is parked under.
pub(crate) const NONE_KEY: &str = "-";

/// Every workspace, in file order.
pub(crate) fn load() -> Workspaces {
    if !crate::build_features::WORKSPACES {
        return Workspaces::default();
    }
    config_dir().map_or_else(Workspaces::default, |d| load_in(&d))
}

/// The AI instructions of the workspace that applies to `repo` in `session`:
/// the session's active workspace when it holds the repository, else the
/// first workspace that does (empty when none does).
#[cfg(feature = "ai")]
pub(crate) fn ai_for(session: &str, repo: &Path) -> gitpoulpi_config::WorkspaceAi {
    if !crate::build_features::WORKSPACES {
        return Default::default();
    }
    let Some(dir) = config_dir() else {
        return Default::default();
    };
    let active = active_id_in(&dir, session);
    load_in(&dir).ai_for(repo, active.as_deref())
}

pub(crate) fn load_in(config_dir: &Path) -> Workspaces {
    Workspaces::load(&Workspaces::file_in(config_dir))
}

/// Writes `all`. `false` when it could not be written.
pub(crate) fn save(all: &Workspaces) -> bool {
    config_dir().is_some_and(|d| save_in(&d, all))
}

pub(crate) fn save_in(config_dir: &Path, all: &Workspaces) -> bool {
    all.save(&Workspaces::file_in(config_dir)).is_ok()
}

/// The workspace `session` is in, when it still exists in
/// `workspaces.toml` (a deleted one no longer applies).
pub(crate) fn active(session: &str) -> Option<Workspace> {
    if !crate::build_features::WORKSPACES {
        return None;
    }
    config_dir().and_then(|d| active_in(&d, session))
}

pub(crate) fn active_in(config_dir: &Path, session: &str) -> Option<Workspace> {
    let id = active_id_in(config_dir, session)?;
    load_in(config_dir).get(&id).cloned()
}

/// The id stored in the session file, whether or not it still exists.
pub(crate) fn active_id_in(config_dir: &Path, session: &str) -> Option<String> {
    load_session_in(config_dir, session).1.active_workspace
}

/// What a swap did to the tab list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Switched {
    /// The workspace the window is in now (`None` = no workspace).
    pub(crate) now: Option<String>,
    /// How many tabs the target's list has.
    pub(crate) tabs: usize,
}

/// Moves the window to `target` (`None` = no workspace): parks the live tab
/// list, takes the target's (its repositories that exist, the first time),
/// records the choice. A swap to where the window already is changes nothing.
pub(crate) fn switch(session: &str, target: Option<&str>) -> Option<Switched> {
    if !crate::build_features::WORKSPACES {
        return None;
    }
    let dir = config_dir()?;
    Some(switch_in(&dir, session, target))
}

pub(crate) fn switch_in(config_dir: &Path, session: &str, target: Option<&str>) -> Switched {
    let all = load_in(config_dir);
    // A target that names nothing is "no workspace": a deleted or mistyped
    // id must not strand the window in a list nobody can see.
    let target = target.filter(|id| all.get(id).is_some());
    let (file, mut s) = load_session_in(config_dir, session);
    let leaving = s.active_workspace.clone();
    if leaving.as_deref() == target {
        return Switched {
            now: leaving,
            tabs: s.tabs.len(),
        };
    }
    let park = leaving.as_deref().unwrap_or(NONE_KEY).to_string();
    let live = std::mem::take(&mut s.tabs);
    s.workspace_tabs.insert(park, WorkspaceTabs { tabs: live });

    let key = target.unwrap_or(NONE_KEY);
    s.tabs = match s.workspace_tabs.remove(key) {
        Some(parked) => parked.tabs,
        None => target
            .and_then(|id| all.get(id))
            .map(first_tabs)
            .unwrap_or_default(),
    };
    s.active_workspace = target.map(str::to_string);
    let _ = s.save(&file);
    Switched {
        now: s.active_workspace.clone(),
        tabs: s.tabs.len(),
    }
}

/// A workspace's repositories as a first tab list: the ones that exist and
/// are repositories, the first one focused.
fn first_tabs(ws: &Workspace) -> Vec<TabEntry> {
    let mut tabs: Vec<TabEntry> = ws
        .repos
        .iter()
        .filter(|p| is_repository(p))
        .map(|p| TabEntry {
            repo_path: p.clone(),
            active: false,
        })
        .collect();
    if let Some(first) = tabs.first_mut() {
        first.active = true;
    }
    tabs
}

/// Leaves workspace `id` if the window is in it, and forgets what it had
/// parked. Called just before the workspace is deleted. Returns whether the
/// window moved to "no workspace".
pub(crate) fn forget(session: &str, id: &str) -> bool {
    config_dir().is_some_and(|d| forget_in(&d, session, id))
}

pub(crate) fn forget_in(config_dir: &Path, session: &str, id: &str) -> bool {
    let in_it = active_id_in(config_dir, session).as_deref() == Some(id);
    if in_it {
        // The workspace is still in `workspaces.toml` here, so the swap
        // parks its tabs, which are dropped just below.
        switch_in(config_dir, session, None);
    }
    let (file, mut s) = load_session_in(config_dir, session);
    if s.workspace_tabs.remove(id).is_some() {
        let _ = s.save(&file);
    }
    in_it
}

/// Adds `path` to workspace `id`, recording `origin` (read off the render
/// thread by the caller). `Some(true)` when it was new.
pub(crate) fn add_repo(id: &str, path: &Path, origin: Option<&str>) -> Option<bool> {
    add_repo_in(&config_dir()?, id, path, origin)
}

pub(crate) fn add_repo_in(
    config_dir: &Path,
    id: &str,
    path: &Path,
    origin: Option<&str>,
) -> Option<bool> {
    let mut all = load_in(config_dir);
    let new = all.get_mut(id)?.add_repo(path, origin);
    save_in(config_dir, &all).then_some(new)
}

/// Whether `path` is in workspace `id`.
pub(crate) fn is_member(id: &str, path: &Path) -> bool {
    load().get(id).is_some_and(|w| w.contains(path))
}

/// Points workspace `id`'s entry for `old` at `new` (Locate...), keeping its
/// place and its recorded remote. `Some(false)` when `old` was not there.
pub(crate) fn replace_repo(id: &str, old: &Path, new: &Path) -> Option<bool> {
    let dir = config_dir()?;
    replace_repo_in(&dir, id, old, new)
}

pub(crate) fn replace_repo_in(config_dir: &Path, id: &str, old: &Path, new: &Path) -> Option<bool> {
    let mut all = load_in(config_dir);
    let ws = all.get_mut(id)?;
    let Some(at) = ws.repos.iter().position(|r| same_repo_path(r, old)) else {
        return Some(false);
    };
    let remote = ws.remote_of(old).map(str::to_string);
    ws.remove_repo(old);
    // Back where it was, unless the new folder is already a member.
    if ws.add_repo(new, remote.as_deref()) {
        let moved = ws.repos.pop().expect("just added");
        ws.repos.insert(at.min(ws.repos.len()), moved);
    }
    save_in(config_dir, &all).then_some(true)
}

/// Removes `path` from workspace `id`. `Some(true)` when it was there.
pub(crate) fn remove_repo(id: &str, path: &Path) -> Option<bool> {
    let dir = config_dir()?;
    let mut all = load_in(&dir);
    let was = all.get_mut(id)?.remove_repo(path);
    save_in(&dir, &all).then_some(was)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn repo(dir: &Path, name: &str) -> PathBuf {
        let p = dir.join(name);
        std::fs::create_dir_all(p.join(".git")).unwrap();
        p
    }

    fn setup() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let (a, b) = (repo(dir.path(), "a"), repo(dir.path(), "b"));
        let mut all = Workspaces::default();
        let acme = all.create("Acme");
        acme.add_repo(&a, None);
        acme.add_repo(&b, None);
        save_in(dir.path(), &all);
        (dir, a, b)
    }

    fn paths(dir: &Path, session: &str) -> Vec<PathBuf> {
        load_session_in(dir, session)
            .1
            .tabs
            .into_iter()
            .map(|t| t.repo_path)
            .collect()
    }

    #[test]
    fn switching_parks_the_live_list_and_brings_back_the_targets() {
        let (dir, a, b) = setup();
        let other = repo(dir.path(), "other");
        let (file, mut s) = load_session_in(dir.path(), "s");
        s.open(&other);
        s.save(&file).unwrap();

        // First visit: the workspace's repositories, the first focused.
        let to = switch_in(dir.path(), "s", Some("acme"));
        assert_eq!(to.now.as_deref(), Some("acme"));
        assert_eq!(paths(dir.path(), "s"), vec![a.clone(), b.clone()]);
        assert_eq!(load_session_in(dir.path(), "s").1.active_index(), Some(0));

        // The user closes one and opens another while in the workspace.
        let (file, mut s) = load_session_in(dir.path(), "s");
        s.close_path(&a);
        s.open(&other);
        s.save(&file).unwrap();

        // Back to no workspace: the plain list is what it was.
        switch_in(dir.path(), "s", None);
        assert_eq!(paths(dir.path(), "s"), vec![other.clone()]);
        assert_eq!(active_id_in(dir.path(), "s"), None);

        // And the workspace remembers its own list, not its repositories.
        switch_in(dir.path(), "s", Some("acme"));
        assert_eq!(paths(dir.path(), "s"), vec![b, other]);
    }

    #[test]
    fn a_swap_to_where_the_window_is_changes_nothing_and_a_gone_target_means_none() {
        let (dir, a, _b) = setup();
        let one = switch_in(dir.path(), "s", Some("acme"));
        let same = switch_in(dir.path(), "s", Some("acme"));
        assert_eq!(one, same);
        assert!(paths(dir.path(), "s").contains(&a));
        let gone = switch_in(dir.path(), "s", Some("nope"));
        assert_eq!(gone.now, None);
    }

    #[test]
    fn closing_every_tab_keeps_the_workspace_and_an_empty_list() {
        let (dir, a, b) = setup();
        switch_in(dir.path(), "s", Some("acme"));
        let (file, mut s) = load_session_in(dir.path(), "s");
        s.close_path(&a);
        s.close_path(&b);
        s.save(&file).unwrap();
        assert_eq!(active_id_in(dir.path(), "s").as_deref(), Some("acme"));
        // Away and back: still empty, not refilled from the repositories.
        switch_in(dir.path(), "s", None);
        switch_in(dir.path(), "s", Some("acme"));
        assert!(paths(dir.path(), "s").is_empty());
    }

    #[test]
    fn a_missing_repository_is_not_opened_as_a_tab() {
        let (dir, a, _b) = setup();
        std::fs::remove_dir_all(&a).unwrap();
        switch_in(dir.path(), "s", Some("acme"));
        assert_eq!(paths(dir.path(), "s").len(), 1);
    }

    #[test]
    fn forgetting_a_workspace_leaves_it_and_drops_its_parked_tabs() {
        let (dir, _a, _b) = setup();
        switch_in(dir.path(), "s", Some("acme"));
        assert!(forget_in(dir.path(), "s", "acme"));
        assert_eq!(active_id_in(dir.path(), "s"), None);
        let (_, s) = load_session_in(dir.path(), "s");
        assert!(!s.workspace_tabs.contains_key("acme"));
    }

    #[test]
    fn locating_a_repository_keeps_its_place_and_its_remote() {
        let (dir, a, b) = setup();
        add_repo_in(dir.path(), "acme", &a, Some("ssh://h/o/a.repo"));
        let moved = repo(dir.path(), "moved");
        assert_eq!(replace_repo_in(dir.path(), "acme", &a, &moved), Some(true));
        let all = load_in(dir.path());
        let ws = all.get("acme").unwrap();
        assert_eq!(ws.repos, vec![moved.clone(), b]);
        assert_eq!(ws.remote_of(&moved), Some("ssh://h/o/a.repo"));
        assert_eq!(replace_repo_in(dir.path(), "acme", &a, &moved), Some(false));
    }

    #[test]
    fn adding_a_repository_goes_through_the_file_and_keeps_its_origin() {
        let (dir, _a, _b) = setup();
        let c = repo(dir.path(), "c");
        assert_eq!(
            add_repo_in(dir.path(), "acme", &c, Some("git@h:o/c.git")),
            Some(true)
        );
        assert_eq!(add_repo_in(dir.path(), "acme", &c, None), Some(false));
        let all = load_in(dir.path());
        assert_eq!(
            all.get("acme").unwrap().remote_of(&c),
            Some("git@h:o/c.git")
        );
        assert_eq!(add_repo_in(dir.path(), "nope", &c, None), None);
    }

    /// The workspace search and the Radar read the session file the swap
    /// writes: with a workspace active their scope is its repositories, and
    /// after leaving it, the open tabs.
    #[cfg(feature = "global-search")]
    #[test]
    fn the_search_and_radar_scopes_follow_the_active_workspace() {
        let (dir, a, b) = setup();
        let other = repo(dir.path(), "other");
        let (file, mut s) = load_session_in(dir.path(), "s");
        s.open(&other);
        s.save(&file).unwrap();
        let tab = crate::repo_switcher::RepoTab {
            path: other.clone(),
            name: "other".into(),
            branch: None,
        };
        switch_in(dir.path(), "s", Some("acme"));
        let scope =
            crate::workspace_search::search_scope_in(dir.path(), "s", std::slice::from_ref(&tab));
        assert_eq!(scope.workspace.as_deref(), Some("Acme"));
        let roots: Vec<_> = scope.repos.iter().map(|r| r.path.clone()).collect();
        assert_eq!(roots.len(), 2);
        assert!(roots
            .iter()
            .any(|p| gitpoulpi_session::same_repo_path(p, &a)));
        assert!(roots
            .iter()
            .any(|p| gitpoulpi_session::same_repo_path(p, &b)));
        #[cfg(all(feature = "hosting", feature = "radar"))]
        {
            let (name, repos) = crate::radar::scope::radar_scope_in(dir.path(), "s");
            assert_eq!(repos.len(), 2, "{name}");
        }
        switch_in(dir.path(), "s", None);
        let scope =
            crate::workspace_search::search_scope_in(dir.path(), "s", std::slice::from_ref(&tab));
        assert_eq!(scope.workspace, None);
        assert_eq!(scope.repos.len(), 1);
    }
}
