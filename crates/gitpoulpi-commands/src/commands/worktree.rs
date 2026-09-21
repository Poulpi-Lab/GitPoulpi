//! `gitpoulpi worktree` handlers.

use gitpoulpi_core::domain::{OperationKind, Worktree, WorktreeAddOptions, WorktreeStart};
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
        WorktreeAction::Add {
            path,
            start,
            branch,
        } => {
            // The daemon's working directory is not ours: a relative path
            // is made absolute here, against where the command was typed.
            let path = std::path::Path::new(&path);
            let path = if path.is_absolute() {
                path.to_path_buf()
            } else {
                std::env::current_dir()
                    .map_err(|e| format!("cannot read the current directory: {e}"))?
                    .join(path)
            };
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
