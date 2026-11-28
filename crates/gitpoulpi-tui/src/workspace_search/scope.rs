//! Which repositories a workspace search covers. The model lives in
//! `gitpoulpi_link::workspace_search::scope`; this adds the open tabs and the
//! session file location of the terminal UI.

use std::path::Path;

use crate::repo_switcher::RepoTab;

pub(crate) use gitpoulpi_link::workspace_search::scope::Scope;
#[cfg(test)]
pub(crate) use gitpoulpi_link::workspace_search::scope::ScopeRepo;

/// The scope of `session` in `config_dir`: the repositories of the active
/// workspace (a stale id that names no workspace counts as none), else the
/// open tabs `open_tabs`.
pub(crate) fn search_scope_in(config_dir: &Path, session: &str, open_tabs: &[RepoTab]) -> Scope {
    let file = crate::session_store::session_file_in(config_dir, session);
    let paths: Vec<_> = open_tabs.iter().map(|t| t.path.clone()).collect();
    gitpoulpi_link::workspace_search::scope::search_scope_in(config_dir, &file, &paths)
}

/// [`search_scope_in`] for the ambient config directory.
pub(crate) fn search_scope(session: &str, open_tabs: &[RepoTab]) -> Scope {
    match gitpoulpi_config::ConfigPaths::discover() {
        Some(paths) => search_scope_in(paths.config_dir(), session, open_tabs),
        None => search_scope_in(Path::new(""), session, open_tabs),
    }
}
