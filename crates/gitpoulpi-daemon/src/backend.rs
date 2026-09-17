//! The one place a `GitBackend` implementation is chosen.
//!
//! Every frontend that can start a daemon (`gitpoulpi daemon run`, the
//! CLI's and the TUI's embedded daemon) calls in here, so which backend
//! serves a repo never depends on which frontend started its daemon first.
//! It lives here rather than in `gitpoulpi-config` because choosing a
//! backend means naming the backend crates, and `gitpoulpi-config` is a
//! leaf that must not depend on them.

use std::sync::Arc;

use gitpoulpi_config::{DefaultBackend, Overrides, Settings};
use gitpoulpi_core::GitBackend;

/// Constructs the backend a daemon will serve every request from.
///
/// `Git2Backend` decorates `SystemGitBackend` (it dispatches what libgit2
/// does not do well back to the git binary), so choosing it does not
/// remove the dependency on an installed `git`. The contract suite in
/// `gitpoulpi-testkit` holds the two behaviorally identical.
pub fn backend_for(selection: DefaultBackend) -> Arc<dyn GitBackend> {
    match selection {
        DefaultBackend::Git2 => Arc::new(gitpoulpi_git2_backend::Git2Backend::new()),
        DefaultBackend::System => Arc::new(gitpoulpi_cli_backend::SystemGitBackend::new()),
    }
}

/// `backend_for` with the selection resolved from `config.toml` plus the
/// `GITPOULPI_*` environment, for callers with no command-line layer (the
/// TUI). A config that cannot be parsed is reported rather than silently
/// replaced with the defaults.
pub fn backend_from_ambient_config() -> gitpoulpi_config::Result<Arc<dyn GitBackend>> {
    let settings = Settings::resolve(
        gitpoulpi_config::ConfigPaths::discover().as_ref(),
        &Overrides::default(),
    )?;
    Ok(backend_for(settings.config().general.default_backend))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The selection decides which implementation is built; `name()` is
    /// what lets the CLI test prove which backend a daemon actually built.
    #[test]
    fn the_selection_actually_changes_which_implementation_is_built() {
        let git2 = backend_for(DefaultBackend::Git2);
        let system = backend_for(DefaultBackend::System);
        assert_eq!(git2.name(), "git2");
        assert_eq!(system.name(), "system");
    }
}
