//! `gitpoulpi worktree` handlers.

use gitpoulpi_core::domain::{
    OperationKind, Worktree, WorktreeAddOptions, WorktreeCommit, WorktreeRemoveOptions,
    WorktreeRemoved, WorktreeStart,
};
use serde_json::json;

use crate::cli::WorktreeAction;
use crate::client::Session;

use super::*;

pub(super) async fn worktree(
    session: &mut Session,
    action: WorktreeAction,
) -> Result<Emitted, String> {
    match action {
        WorktreeAction::List => {
            let worktrees: Vec<Worktree> =
                session.call(OperationKind::Worktrees, json!({})).await?;
            let human = worktrees
                .iter()
                .map(|w| {
                    format!(
                        "{} {} {}",
                        if w.is_main { "*" } else { " " },
                        w.path.display(),
                        w.branch.as_deref().unwrap_or("(detached)")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            Ok(emitted(to_value(&worktrees)?, human))
        }
        WorktreeAction::Changes { path } => {
            let path = absolute(&path)?;
            let changes: Vec<gitpoulpi_core::domain::WorktreeChange> = session
                .call(OperationKind::WorktreeChanges, json!({ "path": path }))
                .await?;
            let human = if changes.is_empty() {
                "clean".to_string()
            } else {
                changes
                    .iter()
                    .map(|c| format!("{} {}", if c.untracked { "?" } else { "M" }, c.path))
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            Ok(emitted(to_value(&changes)?, human))
        }
        WorktreeAction::Remove {
            path,
            force,
            commit_message,
            commit_branch,
            stash_message,
            delete_branch,
        } => {
            let path = absolute(&path)?;
            let branch = if delete_branch {
                let listed: Vec<Worktree> =
                    session.call(OperationKind::Worktrees, json!({})).await?;
                let canonical = |p: &std::path::Path| {
                    std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf())
                };
                let found = listed
                    .iter()
                    .find(|w| canonical(&w.path) == canonical(&path))
                    .ok_or_else(|| {
                        format!("{} is not a worktree of this repository", path.display())
                    })?;
                Some(
                    found
                        .branch
                        .clone()
                        .ok_or("the worktree is detached: it has no branch to delete")?,
                )
            } else {
                None
            };
            let opts = WorktreeRemoveOptions {
                force,
                commit: commit_message.map(|message| WorktreeCommit {
                    message,
                    branch: commit_branch,
                }),
                stash: stash_message,
            };
            let removed: WorktreeRemoved = session
                .call(
                    OperationKind::WorktreeRemove,
                    json!({ "path": path, "opts": opts }),
                )
                .await?;
            // The journaled branch delete, after the folder is gone (git
            // refuses to delete a branch a worktree has checked out).
            let mut deleted = None;
            if let Some(name) = branch {
                let () = session
                    .call(OperationKind::BranchDelete, json!({ "name": name }))
                    .await?;
                deleted = Some(name);
            }
            let mut value = to_value(&removed)?;
            value["deleted_branch"] = json!(deleted);
            let human = format!(
                "removed worktree {}{}",
                removed.path.display(),
                deleted
                    .map(|b| format!(" and branch {b}"))
                    .unwrap_or_default()
            );
            Ok(emitted(value, human))
        }
        WorktreeAction::Add {
            path,
            start,
            branch,
        } => {
            let path = absolute(&path)?;
            let refs = all_refs(session).await?;
            let is_local_branch = refs
                .iter()
                .any(|r| matches!(&r.kind, RefKind::LocalBranch { name } if *name == start));
            let start = if is_local_branch {
                WorktreeStart::Branch(start)
            } else {
                WorktreeStart::Commit(resolve(session, &start).await?)
            };
            let opts = WorktreeAddOptions {
                start,
                new_branch: branch,
            };
            let added: Worktree = session
                .call(
                    OperationKind::WorktreeAdd,
                    json!({ "path": path, "opts": opts }),
                )
                .await?;
            let human = format!(
                "added worktree {} ({})",
                added.path.display(),
                added.branch.as_deref().unwrap_or("detached")
            );
            Ok(emitted(to_value(&added)?, human))
        }
    }
}

/// The daemon's working directory is not ours: a relative path is made
/// absolute here, against where the command was typed.
fn absolute(path: &str) -> Result<std::path::PathBuf, String> {
    let path = std::path::Path::new(path);
    Ok(if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| format!("cannot read the current directory: {e}"))?
            .join(path)
    })
}
