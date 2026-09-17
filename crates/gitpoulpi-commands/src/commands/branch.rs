//! `gitpoulpi branch` handlers.

use gitpoulpi_core::domain::{
    short, FastForwardOutcome, OperationKind, PatchApplied, PatchCreated, PatchSource, Ref, RefKind,
};
use serde_json::json;

use crate::cli::{BranchAction, PatchAction};
use crate::client::Session;

use super::*;

pub(super) async fn branch(session: &mut Session, action: BranchAction) -> Result<Emitted, String> {
    match action {
        BranchAction::Create { name, at } => {
            let at = resolve(session, at.as_deref().unwrap_or("HEAD")).await?;
            session
                .call::<()>(
                    OperationKind::BranchCreate,
                    json!({ "name": name, "at": at }),
                )
                .await?;
            Ok(emitted(
                json!({ "branch": name, "at": at }),
                format!("created {name} at {}", short(at)),
            ))
        }
        BranchAction::Track { upstream, name } => {
            // The default local name is the *branch* part of the upstream,
            // so the configured remote is stripped (not whatever sits
            // before the first `/`: a remote may be named `grp/sub`), which
            // needs the daemon's remote list.
            let local_name = match name {
                Some(n) => n,
                None => {
                    let remotes: Vec<gitpoulpi_core::domain::Remote> =
                        session.call(OperationKind::Remotes, json!({})).await?;
                    let names: Vec<String> = remotes.into_iter().map(|r| r.name).collect();
                    match gitpoulpi_core::domain::split_remote_ref(&upstream, &names) {
                        Some((_, branch)) => branch.to_string(),
                        None => {
                            return Err(format!(
                                "`{upstream}` does not name a remote branch - pass --name to \
                                 choose the local branch name explicitly"
                            ))
                        }
                    }
                }
            };
            session
                .call::<()>(
                    OperationKind::BranchCreateTracking,
                    json!({ "local_name": local_name, "upstream": upstream }),
                )
                .await?;
            Ok(emitted(
                json!({ "branch": local_name, "upstream": upstream }),
                format!("created {local_name} tracking {upstream}"),
            ))
        }
        BranchAction::Delete { name } => {
            session
                .call::<()>(OperationKind::BranchDelete, json!({ "name": name }))
                .await?;
            Ok(emitted(
                json!({ "deleted": name }),
                format!("deleted {name}"),
            ))
        }
        BranchAction::DeleteRemote { name, remote } => {
            session
                .call::<()>(
                    OperationKind::BranchDeleteRemote,
                    json!({ "remote": remote, "name": name }),
                )
                .await?;
            Ok(emitted(
                json!({ "deleted": name, "remote": remote }),
                format!("deleted branch {name} from {remote}"),
            ))
        }
        BranchAction::Rename { old_name, new_name } => {
            session
                .call::<()>(
                    OperationKind::BranchRename,
                    json!({ "old_name": old_name, "new_name": new_name }),
                )
                .await?;
            Ok(emitted(
                json!({ "renamed": old_name, "to": new_name }),
                format!("renamed {old_name} to {new_name}"),
            ))
        }
        BranchAction::SetUpstream { branch, upstream } => {
            session
                .call::<()>(
                    OperationKind::BranchSetUpstream,
                    json!({ "branch": branch, "upstream": upstream }),
                )
                .await?;
            Ok(emitted(
                json!({ "branch": branch, "upstream": upstream }),
                format!("{branch} now tracks {upstream}"),
            ))
        }
        BranchAction::UnsetUpstream { branch } => {
            session
                .call::<()>(
                    OperationKind::BranchSetUpstream,
                    json!({ "branch": branch, "upstream": null }),
                )
                .await?;
            Ok(emitted(
                json!({ "branch": branch, "upstream": null }),
                format!("{branch} tracks nothing"),
            ))
        }
        BranchAction::FastForward { branch, target } => {
            let target = resolve(session, target.as_deref().unwrap_or("HEAD")).await?;
            let outcome: FastForwardOutcome = session
                .call(
                    OperationKind::FastForward,
                    json!({ "branch": branch, "target": target }),
                )
                .await?;
            let human = match outcome {
                FastForwardOutcome::FastForwarded { from, to } => {
                    format!(
                        "fast-forwarded {branch} from {} to {}",
                        short(from),
                        short(to)
                    )
                }
                FastForwardOutcome::UpToDate => format!("{branch} is already up to date"),
            };
            Ok(emitted(to_value(&outcome)?, human))
        }
        BranchAction::List => {
            let refs = all_refs(session).await?;
            let branches: Vec<&Ref> = refs
                .iter()
                .filter(|r| matches!(r.kind, RefKind::LocalBranch { .. }))
                .collect();
            let human = branches
                .iter()
                .map(|r| {
                    format!(
                        "{} {} {}",
                        if r.is_head { "*" } else { " " },
                        ref_name(&r.kind),
                        short(r.target)
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            Ok(emitted(to_value(&branches)?, human))
        }
    }
}

pub(super) async fn patch(session: &mut Session, action: PatchAction) -> Result<Emitted, String> {
    match action {
        PatchAction::Create {
            revs,
            range,
            wip,
            output,
        } => {
            let dest = absolute(output)?;
            let source = if wip {
                PatchSource::WorkingTree { paths: revs }
            } else if let Some(range) = range {
                let (base, tip) = range
                    .split_once("..")
                    .ok_or_else(|| format!("`{range}` is not `base..tip`"))?;
                PatchSource::Range {
                    base: resolve(session, base).await?,
                    tip: resolve(session, tip).await?,
                }
            } else {
                if revs.is_empty() {
                    return Err(
                        "`patch create` needs at least one commit, `--range`, or `--wip`".into(),
                    );
                }
                let mut oids = Vec::with_capacity(revs.len());
                for rev in &revs {
                    oids.push(resolve(session, rev).await?);
                }
                PatchSource::Commits(oids)
            };
            let created: PatchCreated = session
                .call(
                    OperationKind::PatchCreate,
                    json!({ "source": source, "dest": dest }),
                )
                .await?;
            let human = format!(
                "wrote {} commit(s) to {}",
                created.commits,
                created.path.display()
            );
            Ok(emitted(to_value(&created)?, human))
        }
        PatchAction::Apply { path } => {
            let path = absolute(path)?;
            let applied: PatchApplied = session
                .call(OperationKind::PatchApply, json!({ "patch": path }))
                .await?;
            let human = format!("applied patch to {} file(s)", applied.files.len());
            Ok(emitted(to_value(&applied)?, human))
        }
    }
}
