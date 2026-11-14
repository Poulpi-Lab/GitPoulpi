//! `gitpoulpi branch upstream ...` and `gitpoulpi remote-group status`: a
//! branch's several remotes (extra push targets and pull sources, see
//! `gitpoulpi_core::domain::remote_group`). Every write is one
//! `BranchSetRemotes` request (journaled, so `undo` restores the config);
//! the primary upstream is the existing `BranchSetUpstream`.

use gitpoulpi_core::domain::{
    value_remote, OperationKind, RefKind, RemoteGroupStatus, RemoteRelation, RemoteTargetState,
    TargetRef, TargetRole,
};
use serde_json::json;

use crate::cli::{RemoteGroupAction, UpstreamAction};
use crate::client::Session;

use super::*;

/// A refusal a caller can tell from a failure: `{"refused": {reason,
/// message}}`, exit code 2.
fn refused(reason: &str, message: impl Into<String>) -> Emitted {
    let message = message.into();
    let mut out = emitted(
        json!({ "refused": { "reason": reason, "message": message } }),
        format!("refused: {message}"),
    );
    out.exit_code = 2;
    out
}

/// `--branch`, else the checked-out branch; `None` when HEAD is detached.
async fn branch_or_current(
    session: &mut Session,
    branch: Option<String>,
) -> Result<Option<String>, String> {
    if branch.is_some() {
        return Ok(branch);
    }
    Ok(all_refs(session)
        .await?
        .into_iter()
        .find_map(|r| match r.kind {
            RefKind::LocalBranch { name } if r.is_head => Some(name),
            _ => None,
        }))
}

fn detached() -> Emitted {
    refused(
        "detached_head",
        "HEAD is detached: name the branch with --branch",
    )
}

async fn status(session: &mut Session, branch: &str) -> Result<RemoteGroupStatus, String> {
    session
        .call(
            OperationKind::RemoteGroupStatus,
            json!({ "branch": branch }),
        )
        .await
}

async fn set_remotes(
    session: &mut Session,
    branch: &str,
    push_to: &[String],
    pull_from: &[String],
) -> Result<(), String> {
    session
        .call::<()>(
            OperationKind::BranchSetRemotes,
            json!({ "branch": branch, "push_to": push_to, "pull_from": pull_from }),
        )
        .await
}

fn relation(r: &RemoteRelation) -> String {
    match r {
        RemoteRelation::UpToDate => "up to date".into(),
        RemoteRelation::LocalAhead(n) => format!("ahead {n}"),
        RemoteRelation::RemoteAhead(n) => format!("behind {n}"),
        RemoteRelation::Diverged { ahead, behind } => {
            format!("diverged (ahead {ahead}, behind {behind})")
        }
        RemoteRelation::Absent => "not on the remote".into(),
        RemoteRelation::Unrelated => "unrelated history".into(),
    }
}

fn describe(t: &RemoteTargetState) -> String {
    let role = match t.role {
        TargetRole::Primary => "primary",
        TargetRole::PushTo if t.pull => "push + pull",
        TargetRole::PushTo => "push",
        TargetRole::PullFrom => "pull",
    };
    let gone = if t.missing { " [remote gone]" } else { "" };
    format!(
        "{}:{} ({role}) {}{gone}",
        t.remote,
        t.branch,
        relation(&t.relation)
    )
}

fn status_emitted(branch: &str, st: &RemoteGroupStatus) -> Result<Emitted, String> {
    let mut human = format!("{branch}:");
    if st.targets.is_empty() {
        human.push_str(" no remotes");
    }
    for t in &st.targets {
        human.push_str(&format!("\n  {}", describe(t)));
    }
    Ok(emitted(to_value(st)?, human))
}

/// `<remote>` or `<remote>:<dst>` as a target of `branch`.
fn parse_target(text: &str, branch: &str) -> Result<TargetRef, Emitted> {
    TargetRef::parse(text, branch)
        .ok_or_else(|| refused("bad_target", format!("{text:?} names no remote")))
}

fn values(targets: &[TargetRef], branch: &str) -> Vec<String> {
    targets.iter().map(|t| t.to_value(branch)).collect()
}

pub(super) async fn upstream(
    session: &mut Session,
    action: UpstreamAction,
) -> Result<Emitted, String> {
    match action {
        UpstreamAction::List { branch } => {
            let Some(branch) = branch_or_current(session, branch).await? else {
                return Ok(detached());
            };
            let st = status(session, &branch).await?;
            status_emitted(&branch, &st)
        }
        UpstreamAction::Primary { upstream, branch } => {
            let Some(branch) = branch_or_current(session, branch).await? else {
                return Ok(detached());
            };
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
        UpstreamAction::Add {
            target,
            branch,
            pull,
        } => {
            let Some(branch) = branch_or_current(session, branch).await? else {
                return Ok(detached());
            };
            let t = match parse_target(&target, &branch) {
                Ok(t) => t,
                Err(e) => return Ok(e),
            };
            let before = status(session, &branch).await?.remotes;
            if before.primary.as_ref() == Some(&t) {
                return Ok(refused(
                    "already_upstream",
                    format!(
                        "{}:{} is already the upstream of {branch}",
                        t.remote, t.branch
                    ),
                ));
            }
            let mut push_to = before.push_to.clone();
            let mut pull_from = before.pull_from.clone();
            if !push_to.contains(&t) {
                push_to.push(t.clone());
            }
            if pull && !pull_from.contains(&t) {
                pull_from.push(t);
            }
            set_remotes(
                session,
                &branch,
                &values(&push_to, &branch),
                &values(&pull_from, &branch),
            )
            .await?;
            status_emitted(&branch, &status(session, &branch).await?)
        }
        UpstreamAction::Remove { remote, branch } => {
            let Some(branch) = branch_or_current(session, branch).await? else {
                return Ok(detached());
            };
            let before = status(session, &branch).await?.remotes;
            let keep = |l: &[TargetRef]| -> Vec<TargetRef> {
                l.iter().filter(|t| t.remote != remote).cloned().collect()
            };
            let (push_to, pull_from) = (keep(&before.push_to), keep(&before.pull_from));
            if push_to.len() == before.push_to.len() && pull_from.len() == before.pull_from.len() {
                let why = if before.primary.as_ref().is_some_and(|p| p.remote == remote) {
                    format!(
                        "{remote} is the primary upstream of {branch}: use `branch unset-upstream`"
                    )
                } else {
                    format!("{remote} is not a push target or pull source of {branch}")
                };
                return Ok(refused("not_a_target", why));
            }
            set_remotes(
                session,
                &branch,
                &values(&push_to, &branch),
                &values(&pull_from, &branch),
            )
            .await?;
            status_emitted(&branch, &status(session, &branch).await?)
        }
        UpstreamAction::Pull {
            target,
            on,
            off,
            branch,
        } => {
            let Some(branch) = branch_or_current(session, branch).await? else {
                return Ok(detached());
            };
            let t = match parse_target(&target, &branch) {
                Ok(t) => t,
                Err(e) => return Ok(e),
            };
            let before = status(session, &branch).await?.remotes;
            let mut pull_from = before.pull_from.clone();
            if on && !off {
                if before.primary.as_ref() == Some(&t) {
                    return Ok(refused(
                        "already_upstream",
                        format!(
                            "{}:{} is the upstream: a pull always integrates it",
                            t.remote, t.branch
                        ),
                    ));
                }
                if !pull_from.contains(&t) {
                    pull_from.push(t);
                }
            } else {
                // `--off`: the exact target, or every source of that remote
                // when only its name was given.
                let by_name = !target.contains(':');
                let before_len = pull_from.len();
                pull_from.retain(|p| {
                    if by_name {
                        p.remote != value_remote(&target)
                    } else {
                        *p != t
                    }
                });
                if pull_from.len() == before_len {
                    return Ok(refused(
                        "not_a_target",
                        format!("{target} is not a pull source of {branch}"),
                    ));
                }
            }
            set_remotes(
                session,
                &branch,
                &values(&before.push_to, &branch),
                &values(&pull_from, &branch),
            )
            .await?;
            status_emitted(&branch, &status(session, &branch).await?)
        }
    }
}

pub(super) async fn remote_group(
    session: &mut Session,
    action: RemoteGroupAction,
) -> Result<Emitted, String> {
    match action {
        RemoteGroupAction::Status { branch } => {
            let Some(branch) = branch_or_current(session, branch).await? else {
                return Ok(detached());
            };
            let st = status(session, &branch).await?;
            status_emitted(&branch, &st)
        }
    }
}
