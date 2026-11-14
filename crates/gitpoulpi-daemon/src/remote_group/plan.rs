//! `RemoteGroupPlan`: where a branch stands against each of its remotes.
//!
//! Per target, in run order:
//! 1. `ls-remote` for the exact branch (`remote_ref_value`): cheap, and the
//!    truth even when auto-fetch has not run;
//! 2. a fetch of **that remote only**, and only when the tip it holds is an
//!    object this repository lacks;
//! 3. classification of local against that tip (ahead/behind, merge base,
//!    cherry counts, the branch's reflog), then the pure `suggest`;
//! 4. for diverged rows, the paths a merge would conflict in, without
//!    touching the work tree.
//!
//! A remote that cannot be asked or fetched does not fail the plan: its row
//! falls back to the local tracking ref, carries the error and defaults to
//! Skip. With `preflight: false` nothing leaves the machine.

use std::collections::HashMap;

use gitpoulpi_core::domain::{
    classify, normalize_url, parse_branch_remotes, reflog_refname, suggest, BranchRemotes,
    FetchOptions, GroupAction, GroupMode, Oid, PlannedTarget, Ref, RefKind, RelationFacts, Remote,
    RemoteGroupPlan, RemoteGroupPlanRequest, RemoteRelation, RepoRef, SuggestContext, TargetRef,
    TargetRole,
};
use gitpoulpi_core::{Error, GitBackend, Result};

/// Reflog entries looked through for "you rebased what you pushed".
const REFLOG_WINDOW: usize = 200;

/// The targets in run order: the explicit list, else the primary then the
/// push targets (push) or pull sources (pull). Repeats collapse.
fn ordered_targets(req: &RemoteGroupPlanRequest, remotes: &BranchRemotes) -> Vec<TargetRef> {
    let mut list: Vec<TargetRef> = match &req.targets {
        Some(t) => t.clone(),
        None => {
            let mut v: Vec<TargetRef> = remotes.primary.iter().cloned().collect();
            let extra = match req.mode {
                GroupMode::Push => &remotes.push_to,
                GroupMode::Pull => &remotes.pull_from,
            };
            v.extend(extra.iter().cloned());
            v
        }
    };
    let mut seen: Vec<TargetRef> = Vec::new();
    list.retain(|t| {
        if seen.contains(t) {
            false
        } else {
            seen.push(t.clone());
            true
        }
    });
    list
}

fn role_of(req: &RemoteGroupPlanRequest, remotes: &BranchRemotes, t: &TargetRef) -> TargetRole {
    if remotes.primary.as_ref() == Some(t) {
        TargetRole::Primary
    } else if req.mode == GroupMode::Pull {
        TargetRole::PullFrom
    } else {
        TargetRole::PushTo
    }
}

/// What was learned about one target's tip.
struct Seen {
    tip: Option<Oid>,
    checked: bool,
    fetched: bool,
    error: Option<String>,
}

async fn look(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    refs: &[Ref],
    local: Oid,
    target: &TargetRef,
    known: bool,
    preflight: bool,
) -> Seen {
    let tracking = || {
        refs.iter().find_map(|r| match &r.kind {
            RefKind::RemoteBranch { remote, name }
                if *remote == target.remote && *name == target.branch =>
            {
                Some(r.target)
            }
            _ => None,
        })
    };
    let mut seen = Seen {
        tip: None,
        checked: false,
        fetched: false,
        error: None,
    };
    if !known {
        seen.error = Some(format!("no such remote: {}", target.remote));
        return seen;
    }
    if !preflight {
        seen.tip = tracking();
        return seen;
    }
    let full_ref = format!("refs/heads/{}", target.branch);
    match backend
        .remote_ref_value(repo, &target.remote, &full_ref)
        .await
    {
        Ok(None) => {
            seen.checked = true;
        }
        Ok(Some(oid)) => {
            seen.checked = true;
            seen.tip = Some(oid);
            // The server's tip is an object we may lack: fetch that remote
            // (only) when so.
            if backend.ahead_behind(repo, local, oid).is_err() {
                match backend
                    .fetch(repo, &target.remote, &FetchOptions::default())
                    .await
                {
                    Ok(_) => {
                        seen.fetched = true;
                        if backend.ahead_behind(repo, local, oid).is_err() {
                            seen.error = Some(format!(
                                "fetched {} but its tip {} is still unknown here",
                                target.remote, oid
                            ));
                            seen.tip = tracking();
                        }
                    }
                    Err(e) => {
                        seen.error = Some(e.to_string());
                        seen.tip = tracking();
                    }
                }
            }
        }
        Err(e) => {
            seen.error = Some(e.to_string());
            seen.tip = tracking();
        }
    }
    seen
}

fn tip_in_reflog(backend: &dyn GitBackend, repo: &RepoRef, branch: &str, tip: Oid) -> bool {
    backend
        .reflog(repo, &reflog_refname(branch), 0, REFLOG_WINDOW)
        .map(|p| p.entries.iter().any(|e| e.new == tip || e.old == tip))
        .unwrap_or(false)
}

/// One target before the cross-target pass.
struct Row {
    target: PlannedTarget,
    class: gitpoulpi_core::domain::RemoteClass,
}

pub(super) async fn build(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    req: &RemoteGroupPlanRequest,
) -> Result<RemoteGroupPlan> {
    let refs = backend.refs(repo)?;
    let local = refs
        .iter()
        .find_map(|r| match &r.kind {
            RefKind::LocalBranch { name } if *name == req.branch => Some(r.target),
            _ => None,
        })
        .ok_or_else(|| Error::Backend(format!("no such branch: {}", req.branch)))?;
    let entries = backend.config_section(repo, &format!("branch.{}", req.branch))?;
    let remotes = parse_branch_remotes(&entries, &req.branch);
    let configured: Vec<Remote> = backend.remotes(repo)?;
    let order = ordered_targets(req, &remotes);

    let mut rows: Vec<Row> = Vec::new();
    let mut urls: HashMap<(String, String), String> = HashMap::new();
    for t in &order {
        let cfg = configured.iter().find(|r| r.name == t.remote);
        let seen = look(backend, repo, &refs, local, t, cfg.is_some(), req.preflight).await;
        let (mut ahead_behind, mut related, mut remote_unmatched, mut local_unmatched) =
            (None, true, None, None);
        let mut reflog_hit = false;
        let mut error = seen.error.clone();
        if let Some(tip) = seen.tip {
            match backend.ahead_behind(repo, local, tip) {
                Ok(ab) => {
                    ahead_behind = Some(ab);
                    if ab.0 > 0 && ab.1 > 0 {
                        related = backend.merge_base(repo, local, tip)?.is_some();
                        if related {
                            if let Ok((l, r)) = backend.cherry_counts(repo, local, tip) {
                                local_unmatched = Some(l);
                                remote_unmatched = Some(r);
                            }
                            reflog_hit = tip_in_reflog(backend, repo, &req.branch, tip);
                        }
                    }
                }
                Err(e) => error = error.or(Some(e.to_string())),
            }
        }
        let class = classify(&RelationFacts {
            ahead_behind,
            related,
            remote_unmatched,
            remote_tip_in_reflog: reflog_hit,
        });
        let conflicts = match (class.relation, seen.tip) {
            (RemoteRelation::Diverged { .. }, Some(tip)) => backend
                .predict_merge_conflicts(repo, local, tip)
                .unwrap_or_default(),
            _ => Vec::new(),
        };
        let same_url_as = cfg.and_then(|r| {
            let url = match req.mode {
                GroupMode::Push => r.push_url.as_deref().unwrap_or(&r.fetch_url),
                GroupMode::Pull => &r.fetch_url,
            };
            let key = (normalize_url(url), t.branch.clone());
            match urls.get(&key) {
                Some(first) => Some(first.clone()),
                None => {
                    urls.insert(key, t.remote.clone());
                    None
                }
            }
        });
        let (ahead, behind) = ahead_behind.unwrap_or((0, 0));
        rows.push(Row {
            class,
            target: PlannedTarget {
                remote: t.remote.clone(),
                branch: t.branch.clone(),
                role: role_of(req, &remotes, t),
                relation: class.relation,
                rewritten: class.rewritten,
                ahead,
                behind,
                local_unmatched,
                remote_unmatched,
                suggested: GroupAction::Skip,
                allowed: Vec::new(),
                lease: seen.tip,
                conflicts,
                same_url_as,
                checked: seen.checked,
                fetched: seen.fetched,
                error,
                missing: cfg.is_none(),
            },
        });
    }

    // Cross-target pass: does another target already hold the local
    // commits (a rebase would then need a force push there)?
    let holds: Vec<bool> = rows
        .iter()
        .map(|r| r.class.holds_local(r.target.local_unmatched))
        .collect();
    for (i, row) in rows.iter_mut().enumerate() {
        let others = holds.iter().enumerate().any(|(j, h)| j != i && *h);
        let refused = req
            .refused
            .iter()
            .any(|t| t.remote == row.target.remote && t.branch == row.target.branch);
        let s = suggest(
            &row.class,
            &SuggestContext {
                mode: req.mode,
                prefer_rebase: req.prefer_rebase,
                others_already_have_local: others,
                refused,
            },
        );
        let t = &mut row.target;
        t.suggested = s.action;
        t.allowed = s.allowed;
        // A row that could not be checked, or that repeats an earlier
        // remote, defaults to Skip; the user can still pick the rest.
        if (t.error.is_some() || t.same_url_as.is_some()) && t.suggested != GroupAction::Skip {
            t.suggested = GroupAction::Skip;
            t.allowed.retain(|a| *a != GroupAction::Skip);
            t.allowed.insert(0, GroupAction::Skip);
        }
    }

    Ok(RemoteGroupPlan {
        branch: req.branch.clone(),
        mode: req.mode,
        local,
        preflight: req.preflight,
        targets: rows.into_iter().map(|r| r.target).collect(),
    })
}
