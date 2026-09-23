//! `gitpoulpi lfs` handlers.

use gitpoulpi_core::domain::OperationKind;
use serde_json::json;

use crate::cli::LfsAction;
use crate::client::Session;

use super::*;

pub(super) async fn lfs(session: &mut Session, action: LfsAction) -> Result<Emitted, String> {
    use gitpoulpi_core::domain::{
        human_size, LfsBinary, LfsFetchOptions, LfsFile, LfsInstallScope, LfsLock, LfsPattern,
        LfsPushOptions, LfsRepoStatus, LfsTrackOptions,
    };
    match action {
        LfsAction::Status => {
            let status: LfsRepoStatus = session.call(OperationKind::LfsStatus, json!({})).await?;
            let mut lines = vec![match &status.binary {
                LfsBinary::Installed { version } => format!("git-lfs: installed ({version})"),
                LfsBinary::TooOld { version } => {
                    format!("git-lfs: {version} is too old (3.0 or newer needed)")
                }
                LfsBinary::Missing => "git-lfs: not installed".to_string(),
            }];
            if let Some(reason) = status.binary.unavailable() {
                lines.push(reason.message());
            }
            lines.push(format!(
                "this repository uses LFS: {}",
                if status.uses_lfs { "yes" } else { "no" }
            ));
            lines.push(format!(
                "filters: {}, pre-push hook: {}",
                if status.filters_configured {
                    "configured"
                } else {
                    "not configured"
                },
                if status.hooks_installed {
                    "installed"
                } else {
                    "not installed"
                }
            ));
            for p in &status.patterns {
                lines.push(format!(
                    "tracked: {}{}",
                    p.pattern,
                    if p.lockable { " (lockable)" } else { "" }
                ));
            }
            Ok(emitted(to_value(&status)?, lines.join("\n")))
        }
        LfsAction::Files { at } => {
            let at = match at {
                Some(rev) => Some(resolve(session, &rev).await?),
                None => None,
            };
            let files: Vec<LfsFile> = session
                .call(OperationKind::LfsFiles, json!({ "at": at }))
                .await?;
            let human = if files.is_empty() {
                "no LFS files".to_string()
            } else {
                files
                    .iter()
                    .map(|f| {
                        format!(
                            "{}  {}  {}",
                            f.path.display(),
                            human_size(f.pointer.size),
                            if f.local { "local" } else { "not downloaded" }
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            Ok(emitted(to_value(&files)?, human))
        }
        LfsAction::Install { user } => {
            let scope = if user {
                LfsInstallScope::User
            } else {
                LfsInstallScope::Repository
            };
            session
                .call::<()>(OperationKind::LfsInstall, json!({ "scope": scope }))
                .await?;
            let human = match scope {
                LfsInstallScope::Repository => "Git LFS installed for this repository",
                LfsInstallScope::User => "Git LFS installed for this user",
            };
            Ok(emitted(json!({ "installed": scope }), human))
        }
        LfsAction::Track {
            patterns,
            lockable,
            renormalize,
        } => {
            let opts = LfsTrackOptions {
                lockable,
                renormalize,
            };
            let all: Vec<LfsPattern> = session
                .call(
                    OperationKind::LfsTrack,
                    json!({ "patterns": patterns, "opts": opts }),
                )
                .await?;
            let human = format!(
                "tracking {} with LFS (.gitattributes is changed, not staged)",
                patterns.join(", ")
            );
            Ok(emitted(to_value(&all)?, human))
        }
        LfsAction::Untrack { patterns } => {
            let all: Vec<LfsPattern> = session
                .call(OperationKind::LfsUntrack, json!({ "patterns": patterns }))
                .await?;
            let human = format!("no longer tracking {} with LFS", patterns.join(", "));
            Ok(emitted(to_value(&all)?, human))
        }
        LfsAction::Prune => {
            session
                .call::<()>(OperationKind::LfsPrune, json!({}))
                .await?;
            Ok(emitted(
                json!({ "pruned": true }),
                "pruned the local LFS store",
            ))
        }
        // The daemon streams `Event::LfsProgress` to subscribers (the TUI
        // shows it live); this one-shot process just waits for the reply,
        // like `clone`.
        LfsAction::Fetch { remote, all } => {
            let remote = default_remote(session, remote).await?;
            let opts = LfsFetchOptions { all_refs: all };
            session
                .call::<()>(
                    OperationKind::LfsFetch,
                    json!({ "remote": remote, "opts": opts }),
                )
                .await?;
            Ok(emitted(
                json!({ "fetched_lfs_from": remote, "all_refs": all }),
                format!("fetched LFS objects from {remote}"),
            ))
        }
        LfsAction::Pull { remote } => {
            let remote = default_remote(session, remote).await?;
            session
                .call::<()>(OperationKind::LfsPull, json!({ "remote": remote }))
                .await?;
            Ok(emitted(
                json!({ "pulled_lfs_from": remote }),
                format!("pulled LFS objects from {remote}"),
            ))
        }
        LfsAction::Checkout { paths } => {
            session
                .call::<()>(OperationKind::LfsCheckout, json!({ "paths": paths }))
                .await?;
            Ok(emitted(
                json!({ "checked_out": true, "paths": paths }),
                "checked out the LFS files",
            ))
        }
        LfsAction::Locks { remote } => {
            let remote = default_remote(session, remote).await?;
            let locks: Vec<LfsLock> = session
                .call(OperationKind::LfsLocks, json!({ "remote": remote }))
                .await?;
            let human = if locks.is_empty() {
                "no locks".to_string()
            } else {
                locks
                    .iter()
                    .map(|l| {
                        format!(
                            "{}  locked by {}{}",
                            l.path.display(),
                            l.owner.as_deref().unwrap_or("someone"),
                            if l.ours { " (you)" } else { "" }
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            Ok(emitted(to_value(&locks)?, human))
        }
        LfsAction::Lock { path, remote } => {
            let remote = default_remote(session, remote).await?;
            let lock: LfsLock = session
                .call(
                    OperationKind::LfsLock,
                    json!({ "remote": remote, "path": path }),
                )
                .await?;
            Ok(emitted(to_value(&lock)?, format!("locked {path}")))
        }
        LfsAction::Unlock {
            path,
            remote,
            force,
        } => {
            let remote = default_remote(session, remote).await?;
            session
                .call::<()>(
                    OperationKind::LfsUnlock,
                    json!({ "remote": remote, "path": path, "force": force }),
                )
                .await?;
            Ok(emitted(
                json!({ "unlocked": path, "force": force }),
                format!("unlocked {path}"),
            ))
        }
        LfsAction::Push { remote, all } => {
            let remote = default_remote(session, remote).await?;
            let opts = LfsPushOptions { all_refs: all };
            session
                .call::<()>(
                    OperationKind::LfsPush,
                    json!({ "remote": remote, "opts": opts }),
                )
                .await?;
            Ok(emitted(
                json!({ "pushed_lfs_to": remote, "all_refs": all }),
                format!("pushed LFS objects to {remote}"),
            ))
        }
    }
}
