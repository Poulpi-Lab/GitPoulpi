//! `gitpoulpi repo` -- the multi-repository surface.
//!
//! Two halves, on purpose:
//!
//! - **`discover`** goes through the daemon, because it is a
//!   `GitBackend` operation and architecture principle 2 says every read
//!   does; a CLI `.git` walk of its own would derive `RepoId` from the cwd.
//! - **`open`/`close`/`list`/`recent`** are session and window state,
//!   which ADR-0006 puts in `gitpoulpi-session`'s files rather than in the
//!   daemon. They answer without starting one, so `gitpoulpi repo recent`
//!   works from anywhere, including outside a repository.

use gitpoulpi_core::domain::{DiscoveredRepo, InitOptions, OperationKind, RepoRef};
use gitpoulpi_session::{now_seconds, GlobalSession, RecentRepos};
use serde_json::{json, Value};

use crate::cli::RepoAction;
use crate::client::Session;

#[derive(Debug)]
pub struct Emitted {
    pub json: Value,
    pub human: String,
    /// 0 normally; 2 when an undo/redo was refused (the typed refusal is
    /// in `json`), so a script can tell "refused" from "failed" (1).
    pub exit_code: i32,
}

/// Whether this action needs the session's daemon. `main` uses it to
/// avoid starting one for a question the filesystem can answer.
pub fn needs_daemon(action: &RepoAction) -> bool {
    matches!(
        action,
        RepoAction::Discover { .. } | RepoAction::Open { .. } | RepoAction::Init { .. }
    )
}

/// The path arguments are made absolute against *this* process'
/// directory before they leave it: the daemon that answers may be a
/// long-running one started elsewhere (the same reasoning as `clone`'s
/// destination).
fn absolute(path: &str) -> Result<std::path::PathBuf, String> {
    let path = std::path::PathBuf::from(path);
    if path.is_absolute() {
        return Ok(path);
    }
    let cwd = std::env::current_dir()
        .map_err(|e| format!("could not read the current directory: {e}"))?;
    Ok(cwd.join(path))
}

fn session_file(session: &str) -> Result<std::path::PathBuf, String> {
    let paths = gitpoulpi_config::ConfigPaths::discover()
        .ok_or_else(|| "could not determine the config directory for this OS".to_string())?;
    paths.seed_env_from_legacy();
    Ok(paths.session_file(session))
}

fn recent_file() -> Result<std::path::PathBuf, String> {
    let paths = gitpoulpi_config::ConfigPaths::discover()
        .ok_or_else(|| "could not determine the config directory for this OS".to_string())?;
    Ok(RecentRepos::file_in(paths.config_dir()))
}

/// The actions that need no daemon.
pub fn run_local(session_name: &str, action: RepoAction) -> Result<Emitted, String> {
    match action {
        RepoAction::Close { path } => {
            let path = absolute(&path)?;
            let file = session_file(session_name)?;
            let mut session = GlobalSession::load(&file).map_err(|e| e.to_string())?;
            let closed = session.close_path(&path);
            session.save(&file).map_err(|e| e.to_string())?;
            Ok(Emitted {
                exit_code: 0,
                json: json!({ "closed": closed, "path": path }),
                human: if closed {
                    format!("closed {}", path.display())
                } else {
                    // Not an error: closing something already closed is
                    // the state the caller asked for.
                    format!("{} was not open in session {session_name}", path.display())
                },
            })
        }

        RepoAction::List => {
            let session =
                GlobalSession::load(&session_file(session_name)?).map_err(|e| e.to_string())?;
            let rows: Vec<Value> = session
                .tabs
                .iter()
                .map(|t| json!({ "path": t.repo_path, "active": t.active }))
                .collect();
            let human = if rows.is_empty() {
                format!("no repositories open in session {session_name}")
            } else {
                session
                    .tabs
                    .iter()
                    .map(|t| {
                        format!(
                            "{} {}",
                            if t.active { "*" } else { " " },
                            t.repo_path.display()
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            Ok(Emitted {
                exit_code: 0,
                json: json!({ "session": session_name, "tabs": rows }),
                human,
            })
        }

        RepoAction::Recent { prune, forget } => {
            let file = recent_file()?;
            let mut recents = RecentRepos::load(&file);
            let mut changed = false;
            let mut pruned = 0;
            if let Some(path) = &forget {
                changed |= recents.remove(&absolute(path)?);
            }
            if prune {
                pruned = recents.prune_missing();
                changed |= pruned > 0;
            }
            if changed {
                recents.save(&file).map_err(|e| e.to_string())?;
            }

            // `exists` is reported rather than filtered on, so a UI can
            // gray an entry out instead of silently forgetting a
            // repository on an unmounted drive.
            let rows: Vec<Value> = recents
                .iter()
                .map(|r| {
                    json!({
                        "path": r.path,
                        "name": r.name,
                        "branch": r.branch,
                        "last_opened": r.last_opened,
                        "exists": r.path.exists(),
                    })
                })
                .collect();
            let human = if rows.is_empty() {
                "no recent repositories".to_string()
            } else {
                recents
                    .iter()
                    .map(|r| {
                        format!(
                            "{}{}  {}",
                            if r.path.exists() { "" } else { "(missing) " },
                            r.name,
                            r.path.display()
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            Ok(Emitted {
                exit_code: 0,
                json: json!({ "pruned": pruned, "repos": rows }),
                human,
            })
        }

        RepoAction::Discover { .. } | RepoAction::Open { .. } | RepoAction::Init { .. } => {
            unreachable!("handled by run_via_daemon")
        }
    }
}

/// The actions that go through the daemon.
pub async fn run_via_daemon(
    session_name: &str,
    ipc: &mut Session,
    action: RepoAction,
) -> Result<Emitted, String> {
    match action {
        RepoAction::Discover { path } => {
            let path = absolute(path.as_deref().unwrap_or("."))?;
            let found: DiscoveredRepo = ipc
                .call_without_repo(OperationKind::Discover, json!({ "path": path }))
                .await?;
            let human = describe(&found);
            Ok(Emitted {
                exit_code: 0,
                json: serde_json::to_value(&found)
                    .map_err(|e| format!("failed to encode output: {e}"))?,
                human,
            })
        }

        RepoAction::Init {
            path,
            initial_branch,
            bare,
        } => {
            let path = absolute(&path)?;
            // The configured name unless the flag says otherwise, and
            // an empty string means *say nothing* -- git then applies
            // its own `init.defaultBranch`.
            let initial_branch = initial_branch
                .or_else(|| {
                    gitpoulpi_config::Settings::resolve_ambient()
                        .ok()
                        .map(|s| s.config().general.default_branch_name.clone())
                })
                .filter(|name| !name.trim().is_empty());
            let opts = InitOptions {
                bare,
                initial_branch,
            };
            let made: RepoRef = ipc
                .call_without_repo(OperationKind::Init, json!({ "path": path, "opts": opts }))
                .await?;

            // Opened as a tab in the same breath, like the TUI does:
            // creating a repository and then having to go and find it
            // is the gap the Create form exists to close.
            let file = session_file(session_name)?;
            let mut session = GlobalSession::load(&file).map_err(|e| e.to_string())?;
            let index = session.open(&made.root);
            session.save(&file).map_err(|e| e.to_string())?;

            let recent_file = recent_file()?;
            let mut recents = RecentRepos::load(&recent_file);
            let name = gitpoulpi_core::domain::DiscoveredRepo::name_for_root(&made.root);
            recents.record(&made.root, &name, opts_branch(&opts), now_seconds());
            recents.save(&recent_file).map_err(|e| e.to_string())?;

            let human = format!(
                "created {} as tab {} of session {session_name}",
                made.root.display(),
                index + 1
            );
            Ok(Emitted {
                exit_code: 0,
                json: json!({
                    "session": session_name,
                    "tab_index": index,
                    "repo": made,
                }),
                human,
            })
        }

        RepoAction::Open { path } => {
            let path = absolute(&path)?;
            // Discovered *first*, so a path that is not a repository is
            // rejected before anything is written to the session file.
            let found: DiscoveredRepo = ipc
                .call_without_repo(OperationKind::Discover, json!({ "path": path }))
                .await?;

            let file = session_file(session_name)?;
            let mut session = GlobalSession::load(&file).map_err(|e| e.to_string())?;
            let index = session.open(&found.repo.root);
            session.save(&file).map_err(|e| e.to_string())?;

            let recent_file = recent_file()?;
            let mut recents = RecentRepos::load(&recent_file);
            recents.record(
                &found.repo.root,
                &found.name,
                found.head_branch.clone(),
                now_seconds(),
            );
            recents.save(&recent_file).map_err(|e| e.to_string())?;

            let human = format!(
                "opened {} as tab {} of session {session_name}",
                found.repo.root.display(),
                index + 1
            );
            Ok(Emitted {
                exit_code: 0,
                json: json!({
                    "session": session_name,
                    "tab_index": index,
                    "repo": found,
                }),
                human,
            })
        }

        other => run_local(session_name, other),
    }
}

/// The branch a just-initialized repository is on, for the recent
/// list's subtitle. `None` when git chose, because guessing here would
/// put a name in the list that the repository may not have.
fn opts_branch(opts: &InitOptions) -> Option<String> {
    opts.initial_branch.clone()
}

fn describe(found: &DiscoveredRepo) -> String {
    let mut parts = vec![found.repo.root.display().to_string()];
    if let Some(branch) = &found.head_branch {
        parts.push(format!("on {branch}"));
    } else {
        parts.push("detached HEAD".to_string());
    }
    if found.is_bare {
        parts.push("(bare)".to_string());
    }
    if found.is_worktree {
        parts.push("(linked worktree)".to_string());
    }
    parts.join(" ")
}
