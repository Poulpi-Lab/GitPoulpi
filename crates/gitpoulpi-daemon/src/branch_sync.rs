//! `PullAllBranches`: fetch the upstream remotes of the local branches, then
//! bring every branch that tracks one up to date (see
//! `gitpoulpi_core::domain::branch_sync` for the rules and the planner).
//!
//! The order inside the run: download everything first (the remotes, plus the
//! checked-out branch's own pull download), then past the point of no return
//! (`jobs::enter_point`) the checked-out branch is integrated through the
//! ordinary pull path, and only then are the other branches fast-forwarded.
//! The checked-out branch goes first so that a refusal (local changes in the
//! way, which the auto-stash answers by stashing and running the whole request
//! again) happens before any ref moved; the other branches' moves are plain
//! ref updates (no file is touched) and never fail the run.
//!
//! Undo: the request is journaled as one entry (a snapshot of the heads
//! before and after, see `journal::capture` / `journal::after`). When the
//! checked-out branch's integration stops on conflicts, that entry covers the
//! other branches only; the conflicted pull gets the entry a plain `Pull`
//! would have (`PullRecord`), settled when the conflict is resolved or aborted.

// `Failure` is the dispatcher's own (large) error tuple.
#![allow(clippy::result_large_err)]

use std::collections::HashMap;
use std::path::Path;

use gitpoulpi_core::domain::{
    plan_branch_sync, BranchFacts, BranchSyncItem, BranchSyncStatus, FastForwardOutcome,
    MergeOptions, MergeOutcome, Oid, OperationKind, PlannedAction, PlannedBranch,
    PullAllBranchesOutcome, PullAllBranchesRequest, PullMode, PullOptions, PullOutcome,
    RebaseOptions, RebaseOutcome, Ref, RefKind, RemoteFetchFailure, RepoRef, UpstreamFacts,
    UpstreamKind,
};
use gitpoulpi_core::{Error, GitBackend};
use gitpoulpi_ipc::Request;
use serde_json::{json, Value};

use crate::dispatch::Failure;

fn fail(e: &Error) -> Failure {
    (e.to_string(), e.typed(), e.detail().cloned())
}

/// What the config says about one local branch's upstream.
struct Tracking {
    branch: String,
    remote: String,
    merge: String,
}

/// The branches that have a configured upstream (`branch.<n>.remote` and
/// `.merge`), in the order given.
fn trackings(backend: &dyn GitBackend, repo: &RepoRef, branches: &[String]) -> Vec<Tracking> {
    let sections: Vec<String> = branches.iter().map(|b| format!("branch.{b}")).collect();
    let Ok(read) = backend.config_sections(repo, &sections) else {
        return Vec::new();
    };
    branches
        .iter()
        .zip(read)
        .filter_map(|(branch, (_, entries))| {
            let get = |key: &str| {
                entries
                    .iter()
                    .rev()
                    .find(|(k, _)| k.eq_ignore_ascii_case(&format!("branch.{branch}.{key}")))
                    .map(|(_, v)| v.clone())
                    .filter(|v| !v.is_empty())
            };
            Some(Tracking {
                branch: branch.clone(),
                remote: get("remote")?,
                merge: get("merge")?,
            })
        })
        .collect()
}

fn same_dir(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

/// The pull mode the checked-out branch is integrated in when the request
/// names none: the one the Pull button uses.
fn configured_mode() -> PullMode {
    use gitpoulpi_config::PullButtonMode;
    let mode = gitpoulpi_config::Settings::resolve_ambient()
        .map(|s| s.config().git.pull_mode)
        .unwrap_or_default();
    match mode {
        PullButtonMode::Merge => PullMode::Merge,
        PullButtonMode::FfOnly => PullMode::FastForwardOnly,
        PullButtonMode::Rebase => PullMode::Rebase,
    }
}

/// The journal entry a plain `Pull` of the checked-out branch would have,
/// written ahead of the integration. Dropped without [`PullRecord::conflicted`]
/// it is removed again: when the pull completes, the run's own entry covers
/// the move.
struct PullRecord<'a> {
    backend: &'a dyn GitBackend,
    request: Request,
    recording: Option<crate::journal::Recording>,
}

impl<'a> PullRecord<'a> {
    fn begin(backend: &'a dyn GitBackend, request: &Request, opts: &PullOptions) -> Self {
        let request = Request {
            command: OperationKind::Pull,
            args: json!({ "opts": opts }),
            ..request.clone()
        };
        let recording = crate::journal::begin_record(backend, &request);
        PullRecord {
            backend,
            request,
            recording,
        }
    }

    /// The pull stopped on conflicts: the entry stays pending, like a
    /// conflicted pull's, until the conflict is resolved or aborted.
    fn conflicted(mut self) {
        if let Some(recording) = self.recording.take() {
            let mut dispatched = crate::dispatch::ok(self.request.id, json!("Conflicted"), true);
            crate::journal::finish_record(self.backend, &self.request, recording, &mut dispatched);
        }
    }
}

impl Drop for PullRecord<'_> {
    fn drop(&mut self) {
        if let Some(recording) = self.recording.take() {
            let mut undone = crate::dispatch::err(self.request.id, "not recorded".into(), false);
            crate::journal::finish_record(self.backend, &self.request, recording, &mut undone);
        }
    }
}

/// The checked-out branch's integration when nothing was downloaded for it
/// (`fetch: false`): the same three modes against the remote-tracking ref as
/// it is.
fn integrate_local(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    mode: PullMode,
    branch: &str,
    tracking: &Ref,
) -> Result<PullOutcome, Error> {
    match mode {
        PullMode::FastForwardOnly => match backend.fast_forward(repo, branch, tracking.target)? {
            FastForwardOutcome::FastForwarded { to, .. } => Ok(PullOutcome::FastForwarded(to)),
            FastForwardOutcome::UpToDate => Ok(PullOutcome::UpToDate),
        },
        PullMode::Merge => Ok(
            match backend.merge(repo, tracking, &MergeOptions::default())? {
                MergeOutcome::FastForwarded(oid) => PullOutcome::FastForwarded(oid),
                MergeOutcome::Merged(oid) => PullOutcome::Merged(oid),
                MergeOutcome::UpToDate => PullOutcome::UpToDate,
                MergeOutcome::Conflicted(_) => PullOutcome::Conflicted,
            },
        ),
        PullMode::Rebase => {
            let before = head_tip(backend, repo, branch);
            match backend.rebase(repo, tracking, &RebaseOptions::default())? {
                RebaseOutcome::Completed => {
                    let after = head_tip(backend, repo, branch);
                    Ok(match after {
                        Some(oid) if after != before => PullOutcome::Merged(oid),
                        _ => PullOutcome::UpToDate,
                    })
                }
                RebaseOutcome::Conflicted { .. } | RebaseOutcome::Stopped { .. } => {
                    Ok(PullOutcome::Conflicted)
                }
            }
        }
    }
}

fn head_tip(backend: &dyn GitBackend, repo: &RepoRef, branch: &str) -> Option<Oid> {
    backend
        .refs(repo)
        .ok()?
        .into_iter()
        .find_map(|r| match r.kind {
            RefKind::LocalBranch { name } if name == branch => Some(r.target),
            _ => None,
        })
}

/// `PullAllBranches`.
pub(crate) async fn run(
    backend: &dyn GitBackend,
    request: &Request,
    repo: &RepoRef,
) -> Result<Value, Failure> {
    let req: PullAllBranchesRequest = request
        .args
        .as_object()
        .map(|_| serde_json::from_value(request.args.clone()))
        .transpose()
        .map_err(|e| {
            (
                format!("invalid pull all branches request: {e}"),
                None,
                None,
            )
        })?
        .unwrap_or_default();
    let outcome = run_inner(backend, request, repo, &req).await?;
    serde_json::to_value(&outcome)
        .map_err(|e| (format!("failed to encode result: {e}"), None, None))
}

async fn run_inner(
    backend: &dyn GitBackend,
    request: &Request,
    repo: &RepoRef,
    req: &PullAllBranchesRequest,
) -> Result<PullAllBranchesOutcome, Failure> {
    let mut refs = backend.refs(repo).map_err(|e| fail(&e))?;
    let locals: Vec<(String, Oid, bool, Option<String>)> = refs
        .iter()
        .filter_map(|r| match &r.kind {
            RefKind::LocalBranch { name } => {
                Some((name.clone(), r.target, r.is_head, r.upstream.clone()))
            }
            _ => None,
        })
        .collect();
    let names: Vec<String> = locals.iter().map(|(n, ..)| n.clone()).collect();
    let tracked = trackings(backend, repo, &names);
    let remote_of: HashMap<&str, &Tracking> =
        tracked.iter().map(|t| (t.branch.as_str(), t)).collect();

    // ---- download
    let mut outcome = PullAllBranchesOutcome::default();
    crate::jobs::set_phase("fetch");
    crate::test_hook::pull_fetch_delay().await;
    let head_mode = req.head_mode.unwrap_or_else(configured_mode);
    let head_opts = PullOptions {
        mode: head_mode,
        ..PullOptions::default()
    };
    if req.fetch {
        let mut remotes: Vec<&str> = tracked
            .iter()
            .filter(|t| t.remote != ".")
            .map(|t| t.remote.as_str())
            .collect();
        remotes.sort_unstable();
        remotes.dedup();
        for remote in remotes {
            match backend.fetch(repo, remote, &req.fetch_opts).await {
                Ok(_) => outcome.fetched.push(remote.to_string()),
                Err(e) => outcome.fetch_failed.push(RemoteFetchFailure {
                    remote: remote.to_string(),
                    error: e.to_string(),
                }),
            }
        }
        refs = backend.refs(repo).map_err(|e| fail(&e))?;
    }
    let tracking_tip: HashMap<String, (Oid, &Ref)> = refs
        .iter()
        .filter_map(|r| match &r.kind {
            RefKind::RemoteBranch { remote, name } => {
                Some((format!("{remote}/{name}"), (r.target, r)))
            }
            _ => None,
        })
        .collect();

    // ---- facts and plan
    let worktrees = backend.worktrees(repo).unwrap_or_default();
    let mut facts: Vec<BranchFacts> = Vec::new();
    for (name, tip, is_head, ref_upstream) in &locals {
        let other_worktree = worktrees
            .iter()
            .find(|w| w.branch.as_deref() == Some(name) && !same_dir(&w.path, &repo.root))
            .map(|w| w.path.clone());
        let upstream = remote_of.get(name.as_str()).map(|t| {
            let merged = t.merge.strip_prefix("refs/heads/").unwrap_or(&t.merge);
            if t.remote == "." {
                return UpstreamFacts {
                    name: merged.to_string(),
                    kind: UpstreamKind::Local,
                    target: None,
                    ahead: 0,
                    behind: 0,
                    fetch_failed: false,
                };
            }
            let shorthand = ref_upstream
                .clone()
                .unwrap_or_else(|| format!("{}/{merged}", t.remote));
            let target = tracking_tip.get(&shorthand).map(|(oid, _)| *oid);
            let (ahead, behind) = target
                .and_then(|oid| backend.ahead_behind(repo, *tip, oid).ok())
                .unwrap_or((0, 0));
            UpstreamFacts {
                name: shorthand,
                kind: UpstreamKind::Remote,
                target,
                ahead,
                behind,
                fetch_failed: outcome.fetch_failed.iter().any(|f| f.remote == t.remote),
            }
        });
        facts.push(BranchFacts {
            branch: name.clone(),
            tip: *tip,
            is_head: *is_head,
            other_worktree,
            upstream,
        });
    }
    let planned = plan_branch_sync(&facts, req.worktrees);
    let head_step = planned
        .iter()
        .find(|p| p.is_head && p.action == PlannedAction::IntegrateHead);

    // The checked-out branch's own download (what `git pull` fetches first),
    // still inside the part of the run that can be canceled.
    if let (true, Some(_)) = (req.fetch, head_step) {
        backend
            .pull_fetch(repo, &head_opts)
            .await
            .map_err(|e| fail(&e))?;
    }

    // ---- the point of no return
    crate::jobs::enter_point("integrate").await;
    crate::test_hook::integrate_delay().await;
    crate::jobs::set_phase("integrate");

    let mut items: Vec<BranchSyncItem> = Vec::new();
    for p in &planned {
        let item = match &p.action {
            PlannedAction::Report(status) => {
                let detail = (*status == BranchSyncStatus::FetchFailed).then(|| {
                    let remote = remote_of.get(p.branch.as_str()).map_or("", |t| &t.remote);
                    outcome
                        .fetch_failed
                        .iter()
                        .find(|f| f.remote == remote)
                        .map(|f| f.error.clone())
                        .unwrap_or_default()
                });
                item_of(p, *status, Some(p.from), detail)
            }
            PlannedAction::IntegrateHead => {
                let tracking = tracking_tip.get(&p.upstream).map(|(_, r)| *r);
                match integrate_head(backend, request, repo, &head_opts, req.fetch, p, tracking)
                    .await
                {
                    Ok(item) => item,
                    Err(failure) => {
                        // Refused before anything changed (local changes in the
                        // way): the auto-stash's second download may be stopped.
                        crate::jobs::leave_point("fetch");
                        return Err(failure);
                    }
                }
            }
            // The other branches wait for the checked-out one (below).
            PlannedAction::FastForward { .. } | PlannedAction::FastForwardInWorktree { .. } => {
                continue;
            }
        };
        items.push(item);
    }
    // ---- the other branches: pure ref moves, in name order
    let mut moved: Vec<BranchSyncItem> = Vec::new();
    for p in &planned {
        match &p.action {
            PlannedAction::FastForward { to } => {
                moved.push(fast_forward(backend, repo, p, *to, false));
            }
            PlannedAction::FastForwardInWorktree { to, path } => {
                let wt = RepoRef::from_root(path);
                moved.push(fast_forward(backend, &wt, p, *to, true));
            }
            _ => {}
        }
    }
    items.extend(moved);
    // The planner's order: checked-out first, then by name.
    let order: HashMap<&str, usize> = planned
        .iter()
        .enumerate()
        .map(|(i, p)| (p.branch.as_str(), i))
        .collect();
    items.sort_by_key(|i| order.get(i.branch.as_str()).copied().unwrap_or(usize::MAX));
    outcome.head_mode = head_step.map(|_| head_mode);
    outcome.branches = items;
    Ok(outcome)
}

fn item_of(
    p: &PlannedBranch,
    status: BranchSyncStatus,
    to: Option<Oid>,
    detail: Option<String>,
) -> BranchSyncItem {
    BranchSyncItem {
        branch: p.branch.clone(),
        upstream: Some(p.upstream.clone()),
        status,
        from: Some(p.from),
        to,
        ahead: p.ahead,
        behind: p.behind,
        worktree: p.worktree.clone(),
        detail,
        is_head: p.is_head,
    }
}

/// Moves `p`'s branch to `to` (in `repo`, which is the other worktree for a
/// worktree update). A failure is the branch's own status, never the run's.
fn fast_forward(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    p: &PlannedBranch,
    to: Oid,
    in_worktree: bool,
) -> BranchSyncItem {
    match backend.fast_forward(repo, &p.branch, to) {
        Ok(FastForwardOutcome::FastForwarded { to, .. }) => item_of(
            p,
            if in_worktree {
                BranchSyncStatus::WorktreeUpdated
            } else {
                BranchSyncStatus::Updated
            },
            Some(to),
            None,
        ),
        Ok(FastForwardOutcome::UpToDate) => {
            item_of(p, BranchSyncStatus::UpToDate, Some(p.from), None)
        }
        Err(Error::NotFastForward) => item_of(p, BranchSyncStatus::Diverged, Some(p.from), None),
        Err(e) => item_of(
            p,
            BranchSyncStatus::Failed,
            Some(p.from),
            Some(e.to_string()),
        ),
    }
}

/// The checked-out branch: the pull's integrate step in `opts.mode`. A
/// refusal is the run's error (nothing has moved yet), except a fast-forward
/// only pull of a diverged branch, which is that branch's status.
async fn integrate_head(
    backend: &dyn GitBackend,
    request: &Request,
    repo: &RepoRef,
    opts: &PullOptions,
    fetched: bool,
    p: &PlannedBranch,
    tracking: Option<&Ref>,
) -> Result<BranchSyncItem, Failure> {
    let record = PullRecord::begin(backend, request, opts);
    let pulled = if fetched {
        backend.pull_integrate(repo, opts).await
    } else {
        match tracking {
            Some(tracking) => integrate_local(backend, repo, opts.mode, &p.branch, tracking),
            None => Err(Error::Backend(format!(
                "{} has no remote-tracking branch to integrate",
                p.upstream
            ))),
        }
    };
    match pulled {
        Ok(PullOutcome::Conflicted) => {
            record.conflicted();
            Ok(item_of(
                p,
                BranchSyncStatus::Conflicted,
                Some(p.from),
                Some("stopped on conflicts: resolve them or abort, like after a pull".into()),
            ))
        }
        Ok(PullOutcome::UpToDate) => Ok(item_of(p, BranchSyncStatus::UpToDate, Some(p.from), None)),
        Ok(PullOutcome::FastForwarded(oid) | PullOutcome::Merged(oid)) => {
            Ok(item_of(p, BranchSyncStatus::Updated, Some(oid), None))
        }
        Err(Error::NotFastForward) => Ok(item_of(
            p,
            BranchSyncStatus::Diverged,
            Some(p.from),
            Some("diverged: a fast-forward only pull cannot integrate it".into()),
        )),
        Err(e) => Err(fail(&e)),
    }
}
