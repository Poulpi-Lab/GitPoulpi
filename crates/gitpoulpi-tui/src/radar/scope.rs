//! Which repositories the Radar asks about: the active workspace's, else
//! the open tabs. The scope combobox picks another workspace for this view
//! only; the window's own active workspace stays the tab bar's business.

use std::path::{Path, PathBuf};

use gitpoulpi_session::{GlobalSession, Workspaces};

/// The scope value that means "the tabs open in this session".
pub(crate) const OPEN_TABS: &str = "";

/// The scope value for "every repository in the default clone folder".
pub(crate) const CLONE_FOLDER: &str = "@clone";
/// The scope value for "every repository in the default browse folder".
pub(crate) const BROWSE_FOLDER: &str = "@browse";

/// Whether `scope` is one of the two folder scopes.
pub(crate) fn is_folder_scope(scope: &str) -> bool {
    scope == CLONE_FOLDER || scope == BROWSE_FOLDER
}

/// The folder a folder scope covers, read from `config.toml` each time (so a
/// change in Settings applies at once). `None` when it is not set, or is not
/// a folder on this machine. The clone folder counts as set through its
/// per-OS default (the folder the Clone dialog starts on); the browse folder
/// has no default of its own.
pub(crate) fn folder_of(config_dir: &Path, scope: &str) -> Option<PathBuf> {
    let config = gitpoulpi_config::Config::load(&config_dir.join("config.toml")).ok()?;
    let folder = match scope {
        CLONE_FOLDER => config.repos.clone_root_or_default(),
        BROWSE_FOLDER => config.repos.browse_root_override(),
        _ => None,
    }?;
    folder.is_dir().then_some(folder)
}

/// The most repositories one folder scope lists: a home folder must not turn
/// into thousands of probes.
const MAX_FOLDER_REPOS: usize = 200;

/// Every repository in `root`, two levels deep: `root` itself when it is one,
/// its children, and the children's children of a folder that is not itself a
/// repository (`~/src/work/app`). Hidden folders are skipped, and the inside
/// of a repository is never entered. Does file system work: never call it on
/// the thread that draws.
pub(crate) fn discover_repos(root: &Path) -> Vec<PathBuf> {
    fn subfolders(dir: &Path) -> Vec<PathBuf> {
        let mut out: Vec<PathBuf> = std::fs::read_dir(dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        out.sort_by_key(|p| p.to_string_lossy().to_lowercase());
        out
    }
    let is_repo = |p: &Path| crate::session_store::is_repository(p);
    if is_repo(root) {
        return vec![root.to_path_buf()];
    }
    let mut found = Vec::new();
    'top: for child in subfolders(root) {
        let level: Vec<PathBuf> = if is_repo(&child) {
            vec![child]
        } else {
            subfolders(&child)
                .into_iter()
                .filter(|p| is_repo(p))
                .collect()
        };
        for repo in level {
            found.push(repo);
            if found.len() >= MAX_FOLDER_REPOS {
                break 'top;
            }
        }
    }
    found
}

type Cache = std::sync::Mutex<std::collections::HashMap<PathBuf, Vec<PathBuf>>>;

fn cache() -> &'static Cache {
    static CACHE: std::sync::OnceLock<Cache> = std::sync::OnceLock::new();
    CACHE.get_or_init(Default::default)
}

/// What the last scan of `root` found, if one ran in this process.
pub(crate) fn cached_repos(root: &Path) -> Option<Vec<PathBuf>> {
    cache().lock().ok()?.get(root).cloned()
}

/// Scans `root` and remembers the answer until the next scan (the Refresh
/// button, `r`, the palette's `Radar: Refresh`).
pub(crate) fn scan_and_cache(root: &Path) -> Vec<PathBuf> {
    let found = discover_repos(root);
    if let Ok(mut c) = cache().lock() {
        c.insert(root.to_path_buf(), found.clone());
    }
    found
}

/// `paths` as the scope's repositories (no workspace color).
pub(crate) fn folder_repos(paths: Vec<PathBuf>) -> Vec<ScopeRepo> {
    disambiguate(
        paths
            .into_iter()
            .map(|p| ScopeRepo {
                name: name_of(&p),
                path: p,
                color: None,
            })
            .collect(),
    )
}

/// One repository of the scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScopeRepo {
    pub(crate) path: PathBuf,
    pub(crate) name: String,
    /// The workspace's color name or `#rrggbb`, for the repository chip.
    pub(crate) color: Option<String>,
}

/// A workspace the scope combobox offers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScopeChoice {
    pub(crate) id: String,
    pub(crate) name: String,
}

pub(crate) fn name_of(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}

fn session_file(config_dir: &Path, session: &str) -> PathBuf {
    crate::session_store::session_file_in(config_dir, session)
}

/// The workspaces, for the combobox.
pub(crate) fn choices_in(config_dir: &Path) -> Vec<ScopeChoice> {
    Workspaces::load(&Workspaces::file_in(config_dir))
        .workspaces
        .into_iter()
        .map(|w| ScopeChoice {
            id: w.id,
            name: w.name,
        })
        .collect()
}

/// The scope the Radar opens on: the session's active workspace when it
/// is set and still exists, else the open tabs.
pub(crate) fn default_scope_in(config_dir: &Path, session: &str) -> String {
    let active = GlobalSession::load(&session_file(config_dir, session))
        .ok()
        .and_then(|s| s.active_workspace);
    match active {
        Some(id)
            if Workspaces::load(&Workspaces::file_in(config_dir))
                .get(&id)
                .is_some() =>
        {
            id
        }
        _ => OPEN_TABS.to_string(),
    }
}

/// Two repositories with one name (two clones of `app`) get their parent
/// folder appended, so their chips can be told apart.
fn disambiguate(mut repos: Vec<ScopeRepo>) -> Vec<ScopeRepo> {
    let mut counts: std::collections::HashMap<String, usize> = Default::default();
    for r in &repos {
        *counts.entry(r.name.to_lowercase()).or_default() += 1;
    }
    for r in repos
        .iter_mut()
        .filter(|r| counts[&r.name.to_lowercase()] > 1)
    {
        if let Some(parent) = r.path.parent().and_then(Path::file_name) {
            r.name = format!("{} ({})", r.name, parent.to_string_lossy());
        }
    }
    repos
}

/// The repositories of `scope` (a workspace id, or [`OPEN_TABS`]).
pub(crate) fn repos_in(config_dir: &Path, session: &str, scope: &str) -> Vec<ScopeRepo> {
    disambiguate(repos_of(config_dir, session, scope))
}

fn repos_of(config_dir: &Path, session: &str, scope: &str) -> Vec<ScopeRepo> {
    if is_folder_scope(scope) {
        // Only what a scan already found: the scan itself runs on a thread.
        return folder_of(config_dir, scope)
            .and_then(|root| cached_repos(&root))
            .map(folder_repos)
            .unwrap_or_default();
    }
    if scope != OPEN_TABS {
        let all = Workspaces::load(&Workspaces::file_in(config_dir));
        if let Some(w) = all.get(scope) {
            return w
                .repos
                .iter()
                .map(|p| ScopeRepo {
                    path: p.clone(),
                    name: name_of(p),
                    color: w.color.clone(),
                })
                .collect();
        }
    }
    let recents =
        gitpoulpi_session::RecentRepos::load(&gitpoulpi_session::RecentRepos::file_in(config_dir));
    GlobalSession::load(&session_file(config_dir, session))
        .map(|s| s.tabs)
        .unwrap_or_default()
        .into_iter()
        .map(|tab| ScopeRepo {
            name: recents
                .iter()
                .find(|r| r.path == tab.repo_path)
                .map(|r| r.name.clone())
                .unwrap_or_else(|| name_of(&tab.repo_path)),
            path: tab.repo_path,
            color: None,
        })
        .collect()
}

/// `radar_scope()`: the scope the Radar opens on and its repositories.
#[cfg(test)]
pub(crate) fn radar_scope_in(config_dir: &Path, session: &str) -> (String, Vec<ScopeRepo>) {
    radar_scope_saved(config_dir, session, None)
}

/// [`radar_scope_in`] preferring the scope the user last picked
/// (`radar.toml`), when it is still valid: the open tabs, or a workspace
/// that still exists.
pub(crate) fn radar_scope_saved(
    config_dir: &Path,
    session: &str,
    saved: Option<&str>,
) -> (String, Vec<ScopeRepo>) {
    let still_valid = |id: &str| {
        id == OPEN_TABS
            || (is_folder_scope(id) && folder_of(config_dir, id).is_some())
            || Workspaces::load(&Workspaces::file_in(config_dir))
                .get(id)
                .is_some()
    };
    let scope = match saved {
        Some(id) if still_valid(id) => id.to_string(),
        _ => default_scope_in(config_dir, session),
    };
    let repos = repos_in(config_dir, session, &scope);
    (scope, repos)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session_with(dir: &Path, tabs: &[&str], active: Option<&str>) {
        let mut session = GlobalSession {
            active_workspace: active.map(str::to_string),
            ..GlobalSession::default()
        };
        for p in tabs {
            session.tabs.push(gitpoulpi_session::TabEntry {
                repo_path: PathBuf::from(p),
                active: false,
            });
        }
        std::fs::create_dir_all(dir.join("sessions")).unwrap();
        session.save(&session_file(dir, "s")).unwrap();
    }

    fn workspace(dir: &Path, name: &str, repos: &[&str]) -> String {
        let file = Workspaces::file_in(dir);
        let mut all = Workspaces::load(&file);
        let w = all.create(name);
        w.color = Some("teal".into());
        for r in repos {
            w.add_repo(Path::new(r), None);
        }
        let id = w.id.clone();
        all.save(&file).unwrap();
        id
    }

    #[test]
    fn without_a_workspace_the_scope_is_the_open_tabs() {
        let dir = tempfile::tempdir().unwrap();
        session_with(dir.path(), &["/r/a", "/r/b"], None);
        let (_, repos) = radar_scope_in(dir.path(), "s");
        let names: Vec<&str> = repos.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, vec!["a", "b"]);
        assert!(repos.iter().all(|r| r.color.is_none()));
    }

    #[test]
    fn the_active_workspace_replaces_the_tabs() {
        let dir = tempfile::tempdir().unwrap();
        let id = workspace(dir.path(), "Acme", &["/w/x", "/w/y", "/w/z"]);
        session_with(dir.path(), &["/r/a"], Some(&id));
        let (scope, repos) = radar_scope_in(dir.path(), "s");
        assert_eq!(scope, id);
        assert_eq!(repos.len(), 3);
        assert_eq!(repos[0].color.as_deref(), Some("teal"));
        // Another scope for this view only.
        assert_eq!(repos_in(dir.path(), "s", OPEN_TABS).len(), 1);
    }

    #[test]
    fn the_saved_scope_wins_while_it_is_valid() {
        let dir = tempfile::tempdir().unwrap();
        let id = workspace(dir.path(), "Acme", &["/w/x"]);
        let other = workspace(dir.path(), "Side", &["/w/y", "/w/z"]);
        session_with(dir.path(), &["/r/a"], Some(&id));
        let (scope, repos) = radar_scope_saved(dir.path(), "s", Some(&other));
        assert_eq!((scope.as_str(), repos.len()), (other.as_str(), 2));
        // The open tabs are a valid saved choice even with an active workspace.
        let (scope, repos) = radar_scope_saved(dir.path(), "s", Some(OPEN_TABS));
        assert_eq!((scope.as_str(), repos.len()), (OPEN_TABS, 1));
        // A deleted workspace falls back to the default.
        let (scope, _) = radar_scope_saved(dir.path(), "s", Some("gone"));
        assert_eq!(scope, id);
    }

    #[test]
    fn a_workspace_that_no_longer_exists_falls_back_to_the_tabs() {
        let dir = tempfile::tempdir().unwrap();
        session_with(dir.path(), &["/r/a"], Some("gone"));
        assert_eq!(default_scope_in(dir.path(), "s"), OPEN_TABS);
        assert_eq!(radar_scope_in(dir.path(), "s").1.len(), 1);
    }

    #[test]
    fn two_clones_with_one_name_are_told_apart_by_their_folder() {
        let dir = tempfile::tempdir().unwrap();
        session_with(dir.path(), &["/a/app", "/b/app", "/b/lib"], None);
        let (_, repos) = radar_scope_in(dir.path(), "s");
        let names: Vec<&str> = repos.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, vec!["app (a)", "app (b)", "lib"]);
    }

    fn mkrepo(root: &Path, rel: &str) {
        std::fs::create_dir_all(root.join(rel).join(".git")).unwrap();
    }

    #[test]
    fn discovery_goes_two_levels_deep_and_never_into_a_repository_or_a_hidden_folder() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        mkrepo(root, "a");
        mkrepo(root, "work/b");
        mkrepo(root, "work/deep/c");
        mkrepo(root, "a/inner");
        mkrepo(root, ".hidden/d");
        std::fs::create_dir_all(root.join("plain")).unwrap();
        let names: Vec<String> = discover_repos(root)
            .iter()
            .map(|p| {
                p.strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect();
        assert_eq!(names, vec!["a", "work/b"]);
        // A folder that is itself a repository is the only one.
        assert_eq!(discover_repos(&root.join("a")), vec![root.join("a")]);
    }

    #[test]
    fn a_folder_scope_needs_a_set_existing_folder_and_reads_the_cache() {
        let cfg = tempfile::tempdir().unwrap();
        let repos = tempfile::tempdir().unwrap();
        mkrepo(repos.path(), "x");
        std::fs::write(
            cfg.path().join("config.toml"),
            format!("[repos]\nclone_root = '{}'\n", repos.path().display()),
        )
        .unwrap();
        // The browse folder has no default: unset.
        assert_eq!(folder_of(cfg.path(), BROWSE_FOLDER), None);
        assert_eq!(
            folder_of(cfg.path(), CLONE_FOLDER).as_deref(),
            Some(repos.path())
        );
        // Nothing scanned yet: no repositories (the scan is the thread's).
        assert!(repos_in(cfg.path(), "s", CLONE_FOLDER).is_empty());
        scan_and_cache(repos.path());
        let found = repos_in(cfg.path(), "s", CLONE_FOLDER);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "x");
        // A saved folder scope whose folder went away falls back.
        let (scope, _) = radar_scope_saved(cfg.path(), "s", Some(BROWSE_FOLDER));
        assert_eq!(scope, OPEN_TABS);
    }

    #[test]
    fn the_combobox_lists_every_workspace_by_name() {
        let dir = tempfile::tempdir().unwrap();
        workspace(dir.path(), "Acme Corp", &[]);
        workspace(dir.path(), "Side", &[]);
        let names: Vec<String> = choices_in(dir.path()).into_iter().map(|c| c.name).collect();
        assert_eq!(names, vec!["Acme Corp", "Side"]);
    }
}
