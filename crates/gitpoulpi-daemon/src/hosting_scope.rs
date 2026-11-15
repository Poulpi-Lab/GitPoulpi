//! A repository's hosting settings, as the hosting clients ask for them
//! (`gitpoulpi_hosting::RepoScopes`):
//!
//! - **The token**: the repository's authentication profile, when its token
//!   command is marked "Use for the hosting API", gives the token of that
//!   repository's host calls (a GitHub App installation token, say), in place
//!   of the personal one. The same trust (a command a person confirmed, its
//!   files unchanged), the same 30 s limit, the same in-memory cache and the
//!   same redaction as for git's own credential requests: it is the same
//!   `token_command` module. A command that cannot run **refuses the call**;
//!   the personal token is never the fallback.
//! - **Assignment**: `RepoState::assignment_blocked`, read from the file on
//!   every ask so a change applies at once, and a file that cannot be read
//!   counts as blocked.
//!
//! Hosting calls run in the frontends' own processes (the TUI, the CLI, the
//! MCP server), not in the daemon, so this runs there. A surface registers
//! the repository it resolved a project for ([`register`]); an unregistered
//! project gets the plain behaviour.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use gitpoulpi_core::domain::{HttpsAuth, RepoAuthProfile};
use gitpoulpi_hosting::{Error, RepoScopes};

use crate::token_command::{TokenCommand, DEFAULT_CACHE_MINUTES, TIMEOUT};

type Key = (String, String);

#[derive(Default)]
struct Registry {
    roots: Mutex<HashMap<Key, PathBuf>>,
}

fn key(integration_id: &str, project: &str) -> Key {
    (integration_id.to_string(), project.to_ascii_lowercase())
}

fn registry() -> &'static Arc<Registry> {
    static REGISTRY: OnceLock<Arc<Registry>> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let registry = Arc::new(Registry::default());
        gitpoulpi_hosting::install_repo_scopes(registry.clone());
        registry
    })
}

/// Tells the hosting clients that `project` of `integration_id` is the
/// repository at `root`, so its stored settings apply to the calls about it.
/// Cheap; call it wherever a surface resolves a repository's project.
pub fn register(root: &Path, integration_id: &str, project: &str) {
    if project.is_empty() {
        return;
    }
    registry()
        .roots
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .insert(key(integration_id, project), root.to_path_buf());
}

impl Registry {
    fn root_of(&self, integration_id: &str, project: &str) -> Option<PathBuf> {
        self.roots
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&key(integration_id, project))
            .cloned()
    }
}

fn state_path(root: &Path) -> PathBuf {
    let state_dir = gitpoulpi_config::Settings::resolve_ambient()
        .map(|s| s.config().session.per_repo_state_dir.clone())
        .unwrap_or_else(|_| gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION.to_string());
    gitpoulpi_session::RepoState::path_for(root, &state_dir)
}

/// Whether assignment is blocked for the repository at `root`. The file is
/// parsed again only when its modification time changed, so a form can ask on
/// every key press.
pub fn assignment_blocked(root: &Path) -> bool {
    type Seen = (Option<std::time::SystemTime>, bool);
    static SEEN: OnceLock<Mutex<HashMap<PathBuf, Seen>>> = OnceLock::new();
    let path = state_path(root);
    let modified = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
    let mut seen = SEEN
        .get_or_init(Mutex::default)
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if let Some((at, blocked)) = seen.get(&path) {
        if *at == modified && modified.is_some() {
            return *blocked;
        }
    }
    let blocked = gitpoulpi_session::RepoState::load(&path)
        .map(|s| s.assignment_blocked)
        .unwrap_or(true);
    seen.insert(path, (modified, blocked));
    blocked
}

/// The "Post as" name of the repository at `root`: its own override, else the
/// global `general.hosting_post_as`; `None` when neither is set.
pub fn post_as(root: &Path) -> Option<String> {
    let own = gitpoulpi_session::RepoState::load(&state_path(root))
        .ok()
        .and_then(|s| s.hosting_post_as);
    let name = own
        .filter(|n| !n.trim().is_empty())
        .or_else(|| {
            gitpoulpi_config::Settings::resolve_ambient()
                .ok()
                .map(|s| s.config().general.hosting_post_as.clone())
        })?;
    let name = name.trim();
    (!name.is_empty()).then(|| name.to_string())
}

/// Keeps `login` as who the repository's bot token posts as, in its state.
pub fn learn_login(root: &Path, login: &str) {
    let path = state_path(root);
    let Ok(mut state) = gitpoulpi_session::RepoState::load(&path) else {
        return;
    };
    if state.hosting_identity.as_deref() != Some(login) {
        state.hosting_identity = Some(login.to_string());
        let _ = state.save(&path);
    }
}

/// The token command of the repository's profile when it is marked for the
/// hosting API: `Ok(None)` when it is not.
fn hosting_command(root: &Path) -> Result<Option<TokenCommand>, Error> {
    let profile = match crate::repo_auth::stored_profile(root) {
        Ok(p) => p,
        Err(e) => return Err(Error::Local(e.to_string())),
    };
    command_of(root, profile.as_ref())
}

fn command_of(
    root: &Path,
    profile: Option<&RepoAuthProfile>,
) -> Result<Option<TokenCommand>, Error> {
    let Some(
        https @ HttpsAuth::TokenCommand {
            argv,
            username,
            cache_minutes,
            use_for_hosting: true,
            ..
        },
    ) = profile.and_then(|p| p.https.as_ref())
    else {
        return Ok(None);
    };
    crate::command_trust::verify(root, https).map_err(|e| Error::Local(e.to_string()))?;
    Ok(Some(TokenCommand {
        argv: argv.clone(),
        cwd: root.to_path_buf(),
        username: username.clone(),
        cache: Duration::from_secs(u64::from(cache_minutes.unwrap_or(DEFAULT_CACHE_MINUTES)) * 60),
        timeout: TIMEOUT,
    }))
}

/// The token the repository at `root` gives its host calls: `None` when its
/// profile does not use the token command for hosting. `refresh`: the host
/// refused the last one, so the command is asked again.
pub async fn hosting_token(root: &Path, refresh: bool) -> Option<Result<String, Error>> {
    let command = match hosting_command(root) {
        Ok(Some(command)) => command,
        Ok(None) => return None,
        Err(e) => return Some(Err(e)),
    };
    let _one_at_a_time = crate::token_command::turn().await;
    if refresh {
        crate::token_command::invalidate(&command);
    }
    Some(
        crate::token_command::credential_in_turn(&command)
            .await
            .map(|c| c.password)
            .map_err(|e| Error::Local(e.to_string())),
    )
}

#[async_trait::async_trait]
impl RepoScopes for Registry {
    fn assignment_blocked(&self, integration_id: &str, project: &str) -> bool {
        self.root_of(integration_id, project)
            .is_some_and(|root| assignment_blocked(&root))
    }

    async fn token(
        &self,
        integration_id: &str,
        project: &str,
        refresh: bool,
    ) -> Option<Result<String, Error>> {
        let root = self.root_of(integration_id, project)?;
        hosting_token(&root, refresh).await
    }

    fn acts_as_bot(&self, integration_id: &str, project: &str) -> bool {
        self.root_of(integration_id, project)
            .is_some_and(|root| matches!(hosting_command(&root), Ok(Some(_))))
    }

    fn post_as(&self, integration_id: &str, project: &str) -> Option<String> {
        let root = self.root_of(integration_id, project)?;
        post_as(&root)
    }

    fn learned_login(&self, integration_id: &str, project: &str) -> Option<String> {
        let root = self.root_of(integration_id, project)?;
        gitpoulpi_session::RepoState::load(&state_path(&root))
            .ok()?
            .hosting_identity
            .filter(|l| !l.trim().is_empty())
    }

    fn learn_login(&self, integration_id: &str, project: &str, login: &str) {
        if let Some(root) = self.root_of(integration_id, project) {
            learn_login(&root, login);
        }
    }
}
