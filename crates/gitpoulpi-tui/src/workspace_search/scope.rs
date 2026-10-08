//! Which repositories a workspace search covers: the active workspace's when
//! the session has one (`GlobalSession.active_workspace`), else the open tabs.

use std::path::{Path, PathBuf};

use gitpoulpi_core::domain::RepoRef;
use gitpoulpi_session::{same_repo_path, GlobalSession, Workspaces};

use crate::repo_switcher::RepoTab;

/// One repository of the scope.
#[derive(Debug, Clone)]
pub(crate) struct ScopeRepo {
    pub(crate) path: PathBuf,
    /// The last path component, as a tab names it.
    pub(crate) name: String,
    /// What the daemon is asked about it.
    pub(crate) repo: RepoRef,
}

impl ScopeRepo {
    pub(crate) fn new(path: &Path) -> Self {
        let repo = RepoRef::from_root(path);
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());
        Self {
            path: repo.root.clone(),
            name,
            repo,
        }
    }
}

/// What is searched.
#[derive(Debug, Clone, Default)]
pub(crate) struct Scope {
    /// The active workspace's name; `None` = the open tabs.
    pub(crate) workspace: Option<String>,
    pub(crate) repos: Vec<ScopeRepo>,
}

fn push_unique(repos: &mut Vec<ScopeRepo>, path: &Path) {
    if !repos.iter().any(|r| same_repo_path(&r.path, path)) {
        repos.push(ScopeRepo::new(path));
    }
}

/// The scope of `session` in `config_dir`: the repositories of the active
/// workspace (a stale id that names no workspace counts as none), else the
/// open tabs `open_tabs`.
pub(crate) fn search_scope_in(config_dir: &Path, session: &str, open_tabs: &[RepoTab]) -> Scope {
    let file = config_dir.join("sessions").join(format!("{session}.toml"));
    let active = GlobalSession::load(&file)
        .ok()
        .and_then(|s| s.active_workspace);
    let mut repos = Vec::new();
    if let Some(id) = active {
        let all = Workspaces::load(&Workspaces::file_in(config_dir));
        if let Some(workspace) = all.get(&id) {
            for path in &workspace.repos {
                push_unique(&mut repos, path);
            }
            return Scope {
                workspace: Some(workspace.name.clone()),
                repos,
            };
        }
    }
    for tab in open_tabs {
        push_unique(&mut repos, &tab.path);
    }
    Scope {
        workspace: None,
        repos,
    }
}

/// [`search_scope_in`] for the ambient config directory.
pub(crate) fn search_scope(session: &str, open_tabs: &[RepoTab]) -> Scope {
    match gitpoulpi_config::ConfigPaths::discover() {
        Some(paths) => search_scope_in(paths.config_dir(), session, open_tabs),
        None => search_scope_in(Path::new(""), session, open_tabs),
    }
}
