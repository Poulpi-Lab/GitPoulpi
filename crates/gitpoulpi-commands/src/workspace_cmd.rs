//! `gitpoulpi workspace list|show|create|rename|delete|add|remove|set-profile`
//! and the `--workspace` / `GITPOULPI_WORKSPACE` selection.
//!
//! The workspaces are `<config>/workspaces.toml` (`gitpoulpi_session::Workspaces`),
//! user data shared by every session, so - like `session` and favorites -
//! these commands read and write the file directly (ADR-0006, "the
//! boundary"). Only the commands that record a repository's `origin` URL
//! (`create --add`, `add`) go through the daemon: that URL is a
//! `GitBackend` read, and a CLI that ran `git config` itself would put a
//! read outside the seam.

use std::path::{Path, PathBuf};

use gitpoulpi_config::ConfigPaths;
use gitpoulpi_core::domain::{CommitAuthor, OperationKind, Remote};
use gitpoulpi_session::{Workspace, Workspaces};
use serde_json::{json, Value};

use crate::cli::{InstructionModeArg, WorkspaceAction};
use crate::client::{discover_repo, Session};
use crate::commands::emitted;
use crate::repo_cmd::Emitted;

/// The environment variable that stands in for `--workspace`.
pub const WORKSPACE_ENV: &str = "GITPOULPI_WORKSPACE";

/// Whether this action needs the session's daemon (it reads a repository's
/// remotes).
pub fn needs_daemon(action: &WorkspaceAction) -> bool {
    match action {
        WorkspaceAction::Create { repos, .. } => !repos.is_empty(),
        WorkspaceAction::Add { .. } => true,
        _ => false,
    }
}

fn workspaces_file() -> Result<PathBuf, String> {
    let paths = ConfigPaths::discover()
        .ok_or_else(|| "could not determine the config directory for this OS".to_string())?;
    Ok(Workspaces::file_in(paths.config_dir()))
}

/// The id a command was told to act in: the flag, else
/// `GITPOULPI_WORKSPACE`; an empty value is "none".
pub fn selected_workspace(flag: Option<&str>) -> Option<String> {
    flag.map(str::to_string)
        .or_else(|| std::env::var(WORKSPACE_ENV).ok())
        .map(|id| id.trim().to_string())
        .filter(|id| !id.is_empty())
}

fn workspace_or_err<'a>(all: &'a Workspaces, id: &str) -> Result<&'a Workspace, String> {
    all.get(id).ok_or_else(|| unknown(all, id))
}

fn unknown(all: &Workspaces, id: &str) -> String {
    let known: Vec<&str> = all.workspaces.iter().map(|w| w.id.as_str()).collect();
    if known.is_empty() {
        format!("no workspace `{id}` (there are none: `gitpoulpi workspace create <name>`)")
    } else {
        format!("no workspace `{id}` (known: {})", known.join(", "))
    }
}

/// The workspace `id`, loaded, for callers outside this module (search).
pub fn load_workspace(id: &str) -> Result<Workspace, String> {
    let all = Workspaces::load(&workspaces_file()?);
    workspace_or_err(&all, id).cloned()
}

/// The commit author a workspace gives in `repo_root`: its default profile
/// when the repository is one of its own and the profile exists, else
/// `general.active_profile` (`Config::profile_for`). `None` when that names
/// no profile or half an identity (git would refuse an empty ident), so the
/// commit falls back to `git config user.*`.
pub fn commit_author(workspace_id: &str, repo_root: &Path) -> Result<Option<CommitAuthor>, String> {
    let workspace = load_workspace(workspace_id)?;
    let settings = gitpoulpi_config::Settings::resolve_ambient().map_err(|e| e.to_string())?;
    Ok(workspace
        .profile_for(settings.config(), repo_root)
        .filter(|p| !p.author_name.is_empty() && !p.author_email.is_empty())
        .map(|p| CommitAuthor {
            name: p.author_name.clone(),
            email: p.author_email.clone(),
        }))
}

fn absolute(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        return Ok(path);
    }
    let cwd = std::env::current_dir()
        .map_err(|e| format!("could not read the current directory: {e}"))?;
    Ok(cwd.join(path))
}

/// `https://user:token@host/x.git` -> `https://host/x.git`: a recorded URL
/// is shown and saved in a file, never carrying a credential.
fn without_userinfo(url: &str) -> String {
    let Some((scheme, rest)) = url.split_once("://") else {
        return url.to_string();
    };
    let end = rest.find('/').unwrap_or(rest.len());
    match rest[..end].rfind('@') {
        Some(at) => format!("{scheme}://{}", &rest[at + 1..]),
        None => url.to_string(),
    }
}

fn workspace_json(w: &Workspace, profile_exists: Option<bool>) -> Value {
    let repos: Vec<Value> = w
        .repos
        .iter()
        .map(|p| {
            json!({
                "path": p,
                "exists": p.exists(),
                "remote": w.remote_of(p),
            })
        })
        .collect();
    json!({
        "id": w.id,
        "name": w.name,
        "color": w.color,
        "description": w.description,
        "profile": w.profile,
        "profile_exists": profile_exists,
        "ai_instructions": ai_json(w),
        "repos": repos,
    })
}

fn mode_name(mode: gitpoulpi_config::RepoInstructionsMode) -> &'static str {
    match mode {
        gitpoulpi_config::RepoInstructionsMode::Add => "append",
        gitpoulpi_config::RepoInstructionsMode::Replace => "replace",
    }
}

/// The workspace's AI instructions as `{ text, mode, actions: { key: { text, mode } } }`.
fn ai_json(w: &Workspace) -> Value {
    let actions: serde_json::Map<String, Value> = w
        .ai_overrides
        .iter()
        .filter(|(_, v)| !v.is_default())
        .map(|(k, v)| {
            (
                k.clone(),
                json!({ "text": v.text, "mode": mode_name(v.mode) }),
            )
        })
        .collect();
    json!({
        "text": w.ai_instructions.text,
        "mode": mode_name(w.ai_instructions.mode),
        "actions": actions,
    })
}

/// The AI instructions of the workspace that applies to `repo`: the one in
/// `GITPOULPI_WORKSPACE` when it holds the repository, else the first that
/// does. Empty when there is none or the file cannot be read.
pub fn ai_for(repo: &Path) -> gitpoulpi_config::WorkspaceAi {
    let Ok(file) = workspaces_file() else {
        return Default::default();
    };
    Workspaces::load(&file).ai_for(repo, selected_workspace(None).as_deref())
}

fn profile_exists(w: &Workspace) -> Option<bool> {
    let id = w.profile.as_deref()?;
    let settings = gitpoulpi_config::Settings::resolve_ambient().ok()?;
    Some(settings.config().profiles.iter().any(|p| p.id == id))
}

fn describe(w: &Workspace) -> String {
    let mut line = format!("{}  {}  ({} repositories)", w.id, w.name, w.repos.len());
    if let Some(profile) = &w.profile {
        line.push_str(&format!("  profile: {profile}"));
    }
    line
}

fn save(all: &Workspaces, file: &Path) -> Result<(), String> {
    all.save(file).map_err(|e| e.to_string())
}

/// The actions that answer from the file alone.
pub fn run_local(action: WorkspaceAction) -> Result<Emitted, String> {
    let file = workspaces_file()?;
    let mut all = Workspaces::load(&file);
    match action {
        WorkspaceAction::List => {
            let rows: Vec<Value> = all
                .workspaces
                .iter()
                .map(|w| workspace_json(w, profile_exists(w)))
                .collect();
            let human = if all.workspaces.is_empty() {
                "no workspaces".to_string()
            } else {
                all.workspaces
                    .iter()
                    .map(describe)
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            Ok(emitted(json!({ "workspaces": rows }), human))
        }
        WorkspaceAction::Show { id } => {
            let w = workspace_or_err(&all, &id)?;
            let mut human = describe(w);
            for repo in &w.repos {
                human.push_str(&format!(
                    "\n  {}{}",
                    if repo.exists() { "" } else { "(missing) " },
                    repo.display()
                ));
            }
            Ok(emitted(
                json!({ "workspace": workspace_json(w, profile_exists(w)) }),
                human,
            ))
        }
        WorkspaceAction::Create {
            name,
            profile,
            color,
            description,
            ..
        } => {
            if name.trim().is_empty() {
                return Err("a workspace needs a name".into());
            }
            if let Some(same) = all.find_by_name(&name) {
                return Err(format!(
                    "a workspace named `{}` already exists (id `{}`)",
                    same.name, same.id
                ));
            }
            let w = all.create(&name);
            w.profile = profile.filter(|p| !p.trim().is_empty());
            w.color = color.filter(|c| !c.trim().is_empty());
            w.description = description.unwrap_or_default();
            let id = w.id.clone();
            save(&all, &file)?;
            let w = all.get(&id).expect("just created");
            Ok(emitted(
                json!({ "workspace": workspace_json(w, profile_exists(w)) }),
                format!("created workspace {id}"),
            ))
        }
        WorkspaceAction::Rename { id, name } => {
            if name.trim().is_empty() {
                return Err("a workspace needs a name".into());
            }
            if let Some(same) = all.find_by_name(&name).filter(|w| w.id != id) {
                return Err(format!(
                    "a workspace named `{}` already exists (id `{}`)",
                    same.name, same.id
                ));
            }
            let missing = unknown(&all, &id);
            let w = all.get_mut(&id).ok_or(missing)?;
            w.name = name.trim().to_string();
            save(&all, &file)?;
            let w = all.get(&id).expect("exists");
            Ok(emitted(
                json!({ "workspace": workspace_json(w, profile_exists(w)) }),
                format!("renamed {id} to {}", w.name),
            ))
        }
        WorkspaceAction::Delete { id } => {
            workspace_or_err(&all, &id)?;
            all.delete(&id);
            save(&all, &file)?;
            Ok(emitted(
                json!({ "deleted": id }),
                format!("deleted workspace {id} (its repositories are untouched)"),
            ))
        }
        WorkspaceAction::Remove { id, paths } => {
            let missing = unknown(&all, &id);
            let w = all.get_mut(&id).ok_or(missing)?;
            let mut removed = Vec::new();
            let mut absent = Vec::new();
            for path in &paths {
                // The repository's root when it is still one, else the path
                // as typed: a repository that is gone must be removable.
                let target = discover_repo(Some(path))
                    .map(|r| r.root)
                    .or_else(|_| absolute(path))?;
                if w.remove_repo(&target) {
                    removed.push(target);
                } else {
                    absent.push(target);
                }
            }
            save(&all, &file)?;
            let w = all.get(&id).expect("exists");
            Ok(emitted(
                json!({
                    "removed": removed,
                    "not_in_workspace": absent,
                    "workspace": workspace_json(w, profile_exists(w)),
                }),
                format!("removed {} from {id}", removed.len()),
            ))
        }
        WorkspaceAction::SetProfile { id, profile, none } => {
            let missing = unknown(&all, &id);
            let w = all.get_mut(&id).ok_or(missing)?;
            w.profile = if none {
                None
            } else {
                profile.filter(|p| !p.trim().is_empty())
            };
            save(&all, &file)?;
            let w = all.get(&id).expect("exists");
            let exists = profile_exists(w);
            let human = match (&w.profile, exists) {
                (None, _) => format!("{id} has no default profile"),
                (Some(p), Some(false)) => {
                    format!(
                        "{id} defaults to profile `{p}` (no such profile yet: it does not apply)"
                    )
                }
                (Some(p), _) => format!("{id} defaults to profile `{p}`"),
            };
            Ok(emitted(
                json!({ "workspace": workspace_json(w, exists) }),
                human,
            ))
        }
        WorkspaceAction::SetAiInstructions {
            id,
            action,
            mode,
            text,
            clear,
        } => {
            let missing = unknown(&all, &id);
            let w = all.get_mut(&id).ok_or(missing)?;
            let value = if clear {
                gitpoulpi_config::RepoAiInstructions::default()
            } else {
                gitpoulpi_config::RepoAiInstructions {
                    text: text.unwrap_or_default().trim().to_string(),
                    mode: match mode {
                        InstructionModeArg::Append => gitpoulpi_config::RepoInstructionsMode::Add,
                        InstructionModeArg::Replace => {
                            gitpoulpi_config::RepoInstructionsMode::Replace
                        }
                    },
                }
            };
            let cleared = value.is_default() || value.text.is_empty();
            let scope = match &action {
                Some(key) => {
                    if cleared {
                        w.ai_overrides.remove(key);
                    } else {
                        w.ai_overrides.insert(key.clone(), value);
                    }
                    format!("`{key}`")
                }
                None => {
                    w.ai_instructions = if cleared { Default::default() } else { value };
                    "every AI action".to_string()
                }
            };
            save(&all, &file)?;
            let w = all.get(&id).expect("exists");
            let human = if cleared {
                format!("{id} has no AI instructions for {scope} any more")
            } else {
                format!("{id} now has its own AI instructions for {scope}")
            };
            Ok(emitted(
                json!({ "workspace": workspace_json(w, profile_exists(w)) }),
                human,
            ))
        }
        WorkspaceAction::Add { .. } => unreachable!("handled by run_via_daemon"),
    }
}

/// The repository's `origin` URL (credentials stripped), `None` without one.
async fn origin_of(ipc: &mut Session) -> Result<Option<String>, String> {
    let remotes: Vec<Remote> = ipc.call(OperationKind::Remotes, json!({})).await?;
    Ok(remotes
        .iter()
        .find(|r| r.name == "origin")
        .map(|r| without_userinfo(&r.fetch_url))
        .filter(|u| !u.is_empty()))
}

/// Resolves `paths` to repository roots with their origin URLs. A path that
/// is not inside a repository is an error: nothing is saved for any path.
async fn resolve_repos(
    ipc: &mut Session,
    paths: &[String],
) -> Result<Vec<(PathBuf, Option<String>)>, String> {
    let mut out = Vec::new();
    for path in paths {
        let abs = absolute(path)?;
        // A folder that is not there has no root to resolve and no origin to
        // record: refused, and nothing is saved. (A repository that vanishes
        // *later* stays listed, marked missing.)
        if !abs.exists() {
            return Err(format!(
                "{} does not exist: only an existing repository can be added \
                 (one that disappears later stays listed as missing)",
                abs.display()
            ));
        }
        let repo = discover_repo(Some(&abs.to_string_lossy()))?;
        ipc.set_repo(Some(repo.clone()));
        let url = origin_of(ipc).await?;
        out.push((repo.root, url));
    }
    Ok(out)
}

/// The actions that read a repository's remotes.
pub async fn run_via_daemon(ipc: &mut Session, action: WorkspaceAction) -> Result<Emitted, String> {
    let file = workspaces_file()?;
    match action {
        WorkspaceAction::Add { id, paths } => {
            let all = Workspaces::load(&file);
            workspace_or_err(&all, &id)?;
            let repos = resolve_repos(ipc, &paths).await?;
            // Reload: the file may have changed while the daemon answered.
            let mut all = Workspaces::load(&file);
            let missing = unknown(&all, &id);
            let w = all.get_mut(&id).ok_or(missing)?;
            let mut added = Vec::new();
            for (root, url) in &repos {
                if w.add_repo(root, url.as_deref()) {
                    added.push(root.clone());
                }
            }
            save(&all, &file)?;
            let w = all.get(&id).expect("exists");
            Ok(emitted(
                json!({ "added": added, "workspace": workspace_json(w, profile_exists(w)) }),
                format!("added {} to {id}", added.len()),
            ))
        }
        WorkspaceAction::Create {
            name,
            repos,
            profile,
            color,
            description,
        } => {
            let repos = resolve_repos(ipc, &repos).await?;
            let made = run_local(WorkspaceAction::Create {
                name,
                repos: Vec::new(),
                profile,
                color,
                description,
            })?;
            let id = made.json["workspace"]["id"]
                .as_str()
                .expect("create answers the new id")
                .to_string();
            let mut all = Workspaces::load(&file);
            let w = all.get_mut(&id).expect("just created");
            for (root, url) in &repos {
                w.add_repo(root, url.as_deref());
            }
            save(&all, &file)?;
            let w = all.get(&id).expect("exists");
            Ok(emitted(
                json!({ "workspace": workspace_json(w, profile_exists(w)) }),
                made.human,
            ))
        }
        other => run_local(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_recorded_url_never_carries_a_credential() {
        assert_eq!(
            without_userinfo("https://me:secret@github.com/o/r.git"),
            "https://github.com/o/r.git"
        );
        assert_eq!(
            without_userinfo("https://token@host:8443/o/r.git"),
            "https://host:8443/o/r.git"
        );
        assert_eq!(
            without_userinfo("git@github.com:o/r.git"),
            "git@github.com:o/r.git"
        );
        assert_eq!(
            without_userinfo("https://github.com/o/r.git"),
            "https://github.com/o/r.git"
        );
        // An @ in the path is not userinfo.
        assert_eq!(
            without_userinfo("https://h/o/r@v1.git"),
            "https://h/o/r@v1.git"
        );
    }
}
