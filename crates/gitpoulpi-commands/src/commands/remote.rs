//! `gitpoulpi remote` handlers.

use gitpoulpi_core::domain::{OperationKind, Ref, Remote, SparseState};
use serde_json::json;

use crate::cli::{RemoteAction, RepoConfigAction, SparseAction, SshCommandAction};
use crate::client::Session;

use super::*;

pub(super) async fn remote(session: &mut Session, action: RemoteAction) -> Result<Emitted, String> {
    match action {
        RemoteAction::Add {
            name,
            url,
            push_url,
        } => {
            session
                .call::<()>(
                    OperationKind::RemoteAdd,
                    json!({ "name": name, "url": url }),
                )
                .await?;
            if let Some(push_url) = &push_url {
                session
                    .call::<()>(
                        OperationKind::RemoteSetPushUrl,
                        json!({ "name": name, "url": push_url }),
                    )
                    .await?;
            }
            Ok(emitted(
                json!({ "remote": name, "url": url, "push_url": push_url }),
                format!("added remote {name}"),
            ))
        }
        RemoteAction::Rename { old_name, new_name } => {
            session
                .call::<()>(
                    OperationKind::RemoteRename,
                    json!({ "old_name": old_name, "new_name": new_name }),
                )
                .await?;
            Ok(emitted(
                json!({ "renamed": old_name, "to": new_name }),
                format!("renamed remote {old_name} to {new_name}"),
            ))
        }
        RemoteAction::SetUrl { name, url, push } => {
            let (command, which) = if push {
                (OperationKind::RemoteSetPushUrl, "push URL")
            } else {
                (OperationKind::RemoteSetUrl, "URL")
            };
            session
                .call::<()>(command, json!({ "name": name, "url": url }))
                .await?;
            Ok(emitted(
                json!({ "remote": name, "url": url, "push": push }),
                format!("{name} {which} set to {url}"),
            ))
        }
        RemoteAction::ClearPushUrl { name } => {
            session
                .call::<()>(
                    OperationKind::RemoteSetPushUrl,
                    json!({ "name": name, "url": null }),
                )
                .await?;
            Ok(emitted(
                json!({ "remote": name, "push_url": null }),
                format!("{name} pushes to its fetch URL again"),
            ))
        }
        RemoteAction::Default { name } => {
            let root = session
                .repo_root()
                .ok_or_else(|| "not inside a git repository (pass --repo)".to_string())?
                .to_path_buf();
            let state_path = repo_state_path(&root);
            if let Some(name) = name {
                let remotes: Vec<Remote> = session.call(OperationKind::Remotes, json!({})).await?;
                if !remotes.iter().any(|r| r.name == name) {
                    return Err(format!("no such remote: {name}"));
                }
                let mut state =
                    gitpoulpi_session::RepoState::load(&state_path).map_err(|e| e.to_string())?;
                state.default_remote = Some(name.clone());
                state.save(&state_path).map_err(|e| e.to_string())?;
            }
            let remote = default_remote(session, None).await?;
            Ok(emitted(
                json!({ "default_remote": remote }),
                format!("default remote: {remote}"),
            ))
        }
        RemoteAction::Remove { name } => {
            session
                .call::<()>(OperationKind::RemoteRemove, json!({ "name": name }))
                .await?;
            Ok(emitted(
                json!({ "removed": name }),
                format!("removed remote {name}"),
            ))
        }
        RemoteAction::List => {
            let remotes: Vec<Remote> = session.call(OperationKind::Remotes, json!({})).await?;
            let human = remotes
                .iter()
                .map(|r| format!("{}\t{}", r.name, r.fetch_url))
                .collect::<Vec<_>>()
                .join("\n");
            Ok(emitted(to_value(&remotes)?, human))
        }
    }
}

pub(super) async fn repo_config(
    session: &mut Session,
    action: RepoConfigAction,
) -> Result<Emitted, String> {
    match action {
        RepoConfigAction::Show { key } => {
            let args = match &key {
                Some(key) => json!({ "key": key }),
                None => json!({}),
            };
            let value: serde_json::Value = session.call(OperationKind::RepoConfig, args).await?;
            let human = match (&key, &value) {
                (Some(key), v) => format!("{key} = {}", v.as_str().unwrap_or("(unset)")),
                (None, serde_json::Value::Object(map)) => map
                    .iter()
                    .map(|(k, v)| format!("{k} = {}", v.as_str().unwrap_or("(unset)")))
                    .collect::<Vec<_>>()
                    .join("\n"),
                (None, v) => v.to_string(),
            };
            Ok(emitted(json!({ "repo_config": value }), human))
        }
        RepoConfigAction::Set { key, value } => {
            session
                .call::<()>(
                    OperationKind::SetRepoConfig,
                    json!({ "key": key, "value": value }),
                )
                .await?;
            Ok(emitted(
                json!({ "key": key, "value": value }),
                format!("set {key} to {value}"),
            ))
        }
        RepoConfigAction::Unset { key } => {
            session
                .call::<()>(
                    OperationKind::SetRepoConfig,
                    json!({ "key": key, "value": null }),
                )
                .await?;
            Ok(emitted(
                json!({ "key": key, "value": null }),
                format!("removed the repository-local {key}"),
            ))
        }
    }
}

pub(super) async fn ssh_command(
    session: &mut Session,
    action: SshCommandAction,
) -> Result<Emitted, String> {
    match action {
        SshCommandAction::Show => {
            let current: Option<String> =
                session.call(OperationKind::SshCommand, json!({})).await?;
            let human = current
                .clone()
                .unwrap_or_else(|| "(not pinned)".to_string());
            Ok(emitted(json!({ "ssh_command": current }), human))
        }
        SshCommandAction::Set { key, command } => {
            let value = match (command, key) {
                (Some(command), _) => command,
                (None, Some(key)) => gitpoulpi_hosting::ssh_key::ssh_command_for_key(&key),
                (None, None) => return Err("pass --key <file> or --command <text>".to_string()),
            };
            session
                .call::<()>(OperationKind::SetSshCommand, json!({ "command": value }))
                .await?;
            Ok(emitted(
                json!({ "ssh_command": value }),
                format!("pinned core.sshCommand to {value}"),
            ))
        }
        SshCommandAction::Clear => {
            session
                .call::<()>(OperationKind::SetSshCommand, json!({ "command": null }))
                .await?;
            Ok(emitted(
                json!({ "ssh_command": null }),
                "cleared the core.sshCommand pin",
            ))
        }
    }
}

/// The remote a command without `--remote` targets: the one the user
/// chose for this repository (TUI "Set as default remote", or `remote
/// default <name>`), else the checked-out branch's upstream remote, else
/// `origin`, else the only remote - the ladder
/// `gitpoulpi_session::resolve_default_remote` defines for every frontend.
pub(super) async fn default_remote(
    session: &mut Session,
    explicit: Option<String>,
) -> Result<String, String> {
    if let Some(remote) = explicit {
        return Ok(remote);
    }
    let remotes: Vec<String> = session
        .call::<Vec<Remote>>(OperationKind::Remotes, json!({}))
        .await?
        .into_iter()
        .map(|r| r.name)
        .collect();
    let chosen = session
        .repo_root()
        .map(repo_state_path)
        .and_then(|p| gitpoulpi_session::RepoState::load(&p).ok())
        .and_then(|s| s.default_remote);
    let refs: Vec<Ref> = session.call(OperationKind::Refs, json!({})).await?;
    let upstream_remote = refs
        .iter()
        .find(|r| r.is_head)
        .and_then(|r| r.upstream.as_deref())
        .and_then(|u| gitpoulpi_core::domain::split_remote_ref(u, &remotes))
        .map(|(remote, _)| remote);
    gitpoulpi_session::resolve_default_remote(
        chosen.as_deref(),
        upstream_remote.as_deref(),
        &remotes,
    )
    .ok_or_else(|| {
        if remotes.is_empty() {
            "this repository has no remote".to_string()
        } else {
            format!(
                "several remotes ({}) and no default - pass --remote, or set one with \
                 `gitpoulpi remote default <name>`",
                remotes.join(", ")
            )
        }
    })
}

pub(super) async fn sparse_checkout(
    session: &mut Session,
    action: SparseAction,
) -> Result<Emitted, String> {
    match action {
        SparseAction::Status => {
            let state = sparse_state(session).await?;
            let human = if state.enabled {
                format!("sparse checkout: {}", state.dirs.join(" "))
            } else {
                "full checkout (sparse checkout is off)".to_string()
            };
            Ok(emitted(to_value(&state)?, human))
        }
        SparseAction::Set { dirs } => set_sparse(session, dirs).await,
        SparseAction::Add { dirs } => {
            let mut all = sparse_state(session).await?.dirs;
            all.extend(dirs);
            set_sparse(session, all).await
        }
        SparseAction::Disable => {
            session
                .call::<()>(OperationKind::SparseDisable, json!({}))
                .await?;
            Ok(emitted(
                to_value(&SparseState::default())?,
                "sparse checkout disabled: every file is checked out",
            ))
        }
    }
}

async fn sparse_state(session: &mut Session) -> Result<SparseState, String> {
    session
        .call::<SparseState>(OperationKind::SparseStatus, json!({}))
        .await
}

async fn set_sparse(session: &mut Session, dirs: Vec<String>) -> Result<Emitted, String> {
    let dirs = gitpoulpi_core::domain::normalize_sparse_dirs(&dirs).map_err(|e| e.to_string())?;
    session
        .call::<()>(OperationKind::SparseSet, json!({ "dirs": dirs }))
        .await?;
    let state = sparse_state(session).await?;
    let human = format!("sparse checkout: {}", state.dirs.join(" "));
    Ok(emitted(to_value(&state)?, human))
}
