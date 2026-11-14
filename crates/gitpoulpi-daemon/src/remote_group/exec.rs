//! Running a grouped push or pull, and resuming or aborting a saved run.
//!
//! The order is fixed: every integration the decisions ask for (a
//! fast-forward, merge or rebase, in target order) first, so local reaches
//! its final tip once; then, in push mode, the pushes one remote at a time,
//! the primary first. Every target ends with its own status and the run goes
//! on past a failure. A merge or rebase that stops on conflicts saves the
//! run and answers `conflicted`; nothing ever continues by itself.

// `Failure` is the dispatcher's own (large) error tuple.
#![allow(clippy::result_large_err)]

use gitpoulpi_core::domain::{
    normalize_url, GroupAction, GroupMode, GroupResume, GroupRunStatus, Integration, MergeOptions,
    MergeOutcome, Oid, PullMode, PullOptions, PullOutcome, PushLease, PushOptions, RebaseOptions,
    RebaseOutcome, Ref, RefKind, RemoteGroupOutcome, RemoteGroupPlanRequest, RemoteGroupRunRequest,
    RemoteRelation, RepoId, RepoRef, RunTarget, StateTarget, StoppedAt, StoppedOp, TagSelection,
    TargetRef, TargetResult, TargetStatus,
};
use gitpoulpi_core::{Error, FailureKind, GitBackend};
use gitpoulpi_ipc::Event;
use tokio::sync::broadcast;

use super::state::{self, GroupState};
use crate::dispatch::Failure;

fn failure_of(e: Error) -> Failure {
    (e.to_string(), e.typed(), e.detail().cloned())
}

fn plain(message: impl Into<String>) -> Failure {
    (message.into(), None, None)
}

/// Why an integration or a push did not finish.
enum Stop {
    Conflict {
        op: StoppedOp,
        remote_tip: Oid,
        pre_tip: Oid,
    },
    Fail {
        failure: Box<Failure>,
        kind: FailureKind,
    },
}

fn stop_of(e: Error) -> Stop {
    let kind = match &e {
        Error::NotFastForward => FailureKind::NonFastForward,
        other => other.detail().map_or(FailureKind::Other, |d| d.kind()),
    };
    Stop::Fail {
        failure: Box::new(failure_of(e)),
        kind,
    }
}

fn stop_plain(message: impl Into<String>) -> Stop {
    Stop::Fail {
        failure: Box::new(plain(message)),
        kind: FailureKind::Other,
    }
}

fn is_integration(action: GroupAction) -> bool {
    matches!(
        action,
        GroupAction::FastForwardFrom | GroupAction::MergeFrom | GroupAction::RebaseOnto
    )
}

fn in_progress(repo: &RepoRef) -> Option<String> {
    crate::journal::git_dir(&repo.root).and_then(|d| crate::journal::in_progress(&d))
}

fn local_branch(backend: &dyn GitBackend, repo: &RepoRef, branch: &str) -> Option<Ref> {
    backend
        .refs(repo)
        .ok()?
        .into_iter()
        .find(|r| matches!(&r.kind, RefKind::LocalBranch { name } if name == branch))
}

/// Whether the saved run could go on right now (the readiness flag): nothing
/// in progress, nothing unmerged, and, for a stopped run, the branch checked
/// out with its tip containing the stopped remote's tip.
pub(super) fn ready_to_continue(backend: &dyn GitBackend, repo: &RepoRef, st: &GroupState) -> bool {
    if in_progress(repo).is_some() {
        return false;
    }
    if backend.conflict_entries(repo).is_ok_and(|c| !c.is_empty()) {
        return false;
    }
    let Some(stopped) = &st.stopped else {
        return true;
    };
    let Some(local) = local_branch(backend, repo, &st.branch) else {
        return false;
    };
    local.is_head
        && backend
            .ahead_behind(repo, local.target, stopped.remote_tip)
            .is_ok_and(|(_, behind)| behind == 0)
}

struct Exec<'a> {
    backend: &'a dyn GitBackend,
    repo: &'a RepoRef,
    events: &'a broadcast::Sender<Event>,
    repo_id: RepoId,
    st: GroupState,
    /// Something changed (a branch moved, a push went out): a failure
    /// from here on is a per-target status, not a plain error.
    changed: bool,
    /// After resuming a stopped run: ask the remotes again before pushing,
    /// and stop for a decision where a push now needs more than a push.
    replan: bool,
}

pub(super) async fn run(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    events: &broadcast::Sender<Event>,
    args: &serde_json::Value,
) -> Result<serde_json::Value, Failure> {
    let req: RemoteGroupRunRequest = serde_json::from_value(args.clone())
        .map_err(|e| plain(format!("invalid remote group run request: {e}")))?;
    let outcome = match (req.resume, req.retry) {
        (Some(GroupResume::Abort), _) => abort(backend, repo, &req)?,
        (Some(GroupResume::Continue), _) | (None, true) => {
            resume(backend, repo, events, &req).await?
        }
        (None, false) => start(backend, repo, events, &req).await?,
    };
    serde_json::to_value(&outcome).map_err(|e| plain(format!("failed to encode result: {e}")))
}

fn result_of(t: &RunTarget) -> TargetResult {
    TargetResult {
        remote: t.remote.clone(),
        branch: t.branch.clone(),
        action: t.action,
        status: TargetStatus::NotAttempted,
        integrated: None,
        detail: None,
        replaced: None,
        recovery: None,
        tags: None,
    }
}

async fn start(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    events: &broadcast::Sender<Event>,
    req: &RemoteGroupRunRequest,
) -> Result<RemoteGroupOutcome, Failure> {
    if let Some(saved) = state::load(repo) {
        return Err(failure_of(Error::RemoteGroupPending {
            branch: saved.branch,
            run_id: saved.run_id,
        }));
    }
    if req.branch.trim().is_empty() {
        return Err(plain("a grouped run needs a branch"));
    }
    if req.targets.is_empty() {
        return Err(plain("a grouped run needs at least one target"));
    }
    let local = local_branch(backend, repo, &req.branch)
        .ok_or_else(|| plain(format!("no such branch: {}", req.branch)))?;
    if let Some(op) = in_progress(repo) {
        return Err(plain(format!(
            "a {op} is in progress: finish or abort it first"
        )));
    }
    let mut seen: Vec<(&str, &str)> = Vec::new();
    let mut targets: Vec<StateTarget> = Vec::new();
    for t in &req.targets {
        if seen.contains(&(t.remote.as_str(), t.branch.as_str())) {
            continue;
        }
        seen.push((&t.remote, &t.branch));
        let merging = matches!(t.action, GroupAction::MergeFrom | GroupAction::RebaseOnto);
        // A pull integrates into HEAD; a merge or rebase needs the branch
        // checked out. Refused before anything changed.
        if !local.is_head
            && ((req.mode == GroupMode::Pull && t.action != GroupAction::Skip)
                || (req.mode == GroupMode::Push && merging))
        {
            return Err(plain(format!(
                "{} must be checked out to integrate {}/{}",
                req.branch, t.remote, t.branch
            )));
        }
        targets.push(StateTarget {
            decision: t.clone(),
            integrated: false,
            result: result_of(t),
        });
    }
    let st = GroupState::new(req.mode, &req.branch, local.target, req.tags, targets);
    let mut exec = Exec {
        backend,
        repo,
        events,
        repo_id: repo.id.clone(),
        st,
        changed: false,
        replan: false,
    };
    exec.go().await
}

fn check_run_id(st: &GroupState, req: &RemoteGroupRunRequest) -> Result<(), Failure> {
    match &req.expect_run_id {
        Some(id) if *id != st.run_id => Err(plain(format!(
            "the saved grouped run is {}, not {id}",
            st.run_id
        ))),
        _ => Ok(()),
    }
}

async fn resume(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    events: &broadcast::Sender<Event>,
    req: &RemoteGroupRunRequest,
) -> Result<RemoteGroupOutcome, Failure> {
    let mut st = state::load(repo)
        .ok_or_else(|| plain("no grouped push or pull is waiting to be continued"))?;
    check_run_id(&st, req)?;
    let local = local_branch(backend, repo, &st.branch).ok_or_else(|| {
        plain(format!(
            "{} no longer exists: abort the grouped run",
            st.branch
        ))
    })?;
    let mut changed = false;
    let mut replan = false;
    if let Some(stopped) = st.stopped.clone() {
        if let Some(op) = in_progress(repo) {
            return Err(plain(format!(
                "the {op} is still in progress: resolve and conclude it first"
            )));
        }
        if backend.conflict_entries(repo).is_ok_and(|c| !c.is_empty()) {
            return Err(plain("conflicts are still unresolved"));
        }
        if !local.is_head {
            return Err(plain(format!("{} must be checked out again", st.branch)));
        }
        let contains = backend
            .ahead_behind(repo, local.target, stopped.remote_tip)
            .is_ok_and(|(_, behind)| behind == 0);
        if !contains {
            return Err(plain(format!(
                "{} does not contain {}'s tip yet: finish the {} or abort the grouped run",
                st.branch,
                stopped.remote,
                match stopped.op {
                    StoppedOp::Merge => "merge",
                    StoppedOp::Rebase => "rebase",
                }
            )));
        }
        for t in &mut st.targets {
            if t.decision.remote == stopped.remote && t.decision.branch == stopped.branch {
                t.integrated = true;
                t.result.integrated = Some(match stopped.op {
                    StoppedOp::Merge => Integration::Merged,
                    StoppedOp::Rebase => Integration::Rebased,
                });
                if st.mode == GroupMode::Pull {
                    t.result.status = TargetStatus::Integrated;
                }
            }
        }
        st.stopped = None;
        changed = true;
        replan = st.mode == GroupMode::Push;
    } else if let Some(op) = in_progress(repo) {
        return Err(plain(format!(
            "a {op} is in progress: finish or abort it first"
        )));
    }
    // The caller's answers to a `needs_resolution`: new decisions.
    for answer in &req.targets {
        for t in &mut st.targets {
            if t.decision.remote == answer.remote && t.decision.branch == answer.branch {
                let integration_changed = answer.action != t.decision.action;
                t.decision = answer.clone();
                t.result.action = answer.action;
                if integration_changed {
                    t.integrated = false;
                    t.result.integrated = None;
                    t.result.status = TargetStatus::NotAttempted;
                    t.result.detail = None;
                }
            }
        }
    }
    for t in &mut st.targets {
        if t.result.status.is_retryable(req.retry) {
            t.result.status = TargetStatus::NotAttempted;
            t.result.detail = None;
        }
    }
    let mut exec = Exec {
        backend,
        repo,
        events,
        repo_id: repo.id.clone(),
        st,
        changed,
        replan,
    };
    exec.go().await
}

fn abort(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    req: &RemoteGroupRunRequest,
) -> Result<RemoteGroupOutcome, Failure> {
    let st = state::load(repo)
        .ok_or_else(|| plain("no grouped push or pull is waiting to be aborted"))?;
    check_run_id(&st, req)?;
    let mut note = String::new();
    // Only the merge or rebase this run started is aborted.
    if st.stopped.is_some() && in_progress(repo).is_some() {
        backend.operation_abort(repo).map_err(failure_of)?;
        note = "the merge or rebase was aborted; ".to_string();
    }
    state::clear(repo);
    let done = st
        .targets
        .iter()
        .filter(|t| t.result.status.is_ok())
        .count();
    let local_tip = local_branch(backend, repo, &st.branch).map(|r| r.target);
    Ok(RemoteGroupOutcome {
        run_id: st.run_id,
        branch: st.branch,
        mode: st.mode,
        status: GroupRunStatus::Aborted,
        local_tip,
        targets: st.targets.into_iter().map(|t| t.result).collect(),
        pending: false,
        stopped: None,
        conflicts: Vec::new(),
        plan: None,
        message: format!(
            "{note}the grouped run was dropped; {done} target{} already done stay as they are",
            if done == 1 { "" } else { "s" }
        ),
    })
}

impl Exec<'_> {
    fn save(&self) {
        state::save(self.repo, &self.st);
    }

    fn emit(&self, i: usize, status: TargetStatus) {
        let _ = self.events.send(Event::RemoteGroupProgress {
            repo: self.repo_id.clone(),
            run_id: self.st.run_id.clone(),
            remote: self.st.targets[i].decision.remote.clone(),
            status,
        });
    }

    fn set_status(&mut self, i: usize, status: TargetStatus) {
        self.st.targets[i].result.status = status.clone();
        self.emit(i, status);
    }

    fn local_tip(&self) -> Option<Oid> {
        local_branch(self.backend, self.repo, &self.st.branch).map(|r| r.target)
    }

    fn head_is_branch(&self) -> bool {
        local_branch(self.backend, self.repo, &self.st.branch).is_some_and(|r| r.is_head)
    }

    fn conflicted_paths(&self) -> Vec<String> {
        self.backend
            .conflict_entries(self.repo)
            .map(|entries| {
                entries
                    .iter()
                    .map(|e| e.path.to_string_lossy().replace('\\', "/"))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn pending(&self, i: usize) -> bool {
        self.st.targets[i].result.status == TargetStatus::NotAttempted
    }

    fn outcome(
        &self,
        status: GroupRunStatus,
        message: String,
        conflicts: Vec<String>,
        plan: Option<gitpoulpi_core::domain::RemoteGroupPlan>,
    ) -> RemoteGroupOutcome {
        RemoteGroupOutcome {
            run_id: self.st.run_id.clone(),
            branch: self.st.branch.clone(),
            mode: self.st.mode,
            status,
            local_tip: self.local_tip(),
            targets: self.st.targets.iter().map(|t| t.result.clone()).collect(),
            pending: state::load(self.repo).is_some_and(|s| s.run_id == self.st.run_id),
            stopped: self.st.stopped.clone(),
            conflicts,
            plan,
            message,
        }
    }

    async fn go(&mut self) -> Result<RemoteGroupOutcome, Failure> {
        // Skip decisions are final at once.
        for i in 0..self.st.targets.len() {
            if self.pending(i) && self.st.targets[i].decision.action == GroupAction::Skip {
                self.set_status(i, TargetStatus::Skipped);
            }
        }
        if let Some(out) = self.integrate_all().await? {
            return Ok(out);
        }
        if self.st.mode == GroupMode::Push {
            if self.replan {
                if let Some(out) = self.check_replan().await? {
                    return Ok(out);
                }
            }
            if let Some(out) = self.push_all().await? {
                return Ok(out);
            }
        }
        Ok(self.finish())
    }

    /// Canceled: the target in flight (if any) is unknown, the rest stay
    /// not attempted, and the run is kept for a retry once anything ran.
    fn canceled(&mut self, in_flight: Option<usize>) -> RemoteGroupOutcome {
        if let Some(i) = in_flight {
            self.set_status(i, TargetStatus::Unknown);
            self.changed = true;
        }
        let progressed = self.st.targets.iter().any(|t| {
            !matches!(
                t.result.status,
                TargetStatus::NotAttempted | TargetStatus::Skipped
            )
        });
        if self.changed || progressed {
            self.save();
        }
        let message = match in_flight {
            Some(i) => format!(
                "canceled while talking to {}: it may or may not have received the push; the \
                 rest was not attempted (continue to retry, or abort)",
                self.st.targets[i].decision.remote
            ),
            None => "canceled between targets: the rest was not attempted (continue to retry, \
                     or abort)"
                .to_string(),
        };
        self.outcome(GroupRunStatus::Canceled, message, Vec::new(), None)
    }

    async fn integrate_all(&mut self) -> Result<Option<RemoteGroupOutcome>, Failure> {
        let total = self.st.targets.len();
        for i in 0..total {
            let action = self.st.targets[i].decision.action;
            if !self.pending(i) || !is_integration(action) || self.st.targets[i].integrated {
                continue;
            }
            crate::test_hook::step_delay().await;
            if crate::jobs::cancel_requested_now() {
                return Ok(Some(self.canceled(None)));
            }
            let remote = self.st.targets[i].decision.remote.clone();
            if !self.remote_known(&remote) {
                self.set_status(i, TargetStatus::RemoteMissing);
                continue;
            }
            crate::jobs::set_phase(&format!("integrate {remote} ({}/{total})", i + 1));
            self.emit(i, TargetStatus::Integrating);
            let mut attempt = self.integrate_one(i).await;
            // A fast-forward that an earlier integration of this run made
            // impossible (each source was ahead alone; after the first, the
            // second has diverged) is the merge the user's pull mode means.
            if matches!(
                &attempt,
                Err(Stop::Fail {
                    kind: FailureKind::NonFastForward,
                    ..
                })
            ) && action == GroupAction::FastForwardFrom
                && self.changed
            {
                self.st.targets[i].decision.action = GroupAction::MergeFrom;
                self.st.targets[i].result.action = GroupAction::MergeFrom;
                attempt = self.integrate_one(i).await;
            }
            match attempt {
                Ok(how) => {
                    if how != Integration::UpToDate {
                        self.changed = true;
                    }
                    let t = &mut self.st.targets[i];
                    t.integrated = true;
                    t.result.integrated = Some(how);
                    if self.st.mode == GroupMode::Pull {
                        let status = if how == Integration::UpToDate {
                            TargetStatus::UpToDate
                        } else {
                            TargetStatus::Integrated
                        };
                        self.set_status(i, status);
                    } else {
                        // The push (or its skip, when local equals the
                        // remote) comes later; the row is not final.
                        self.emit(i, TargetStatus::NotAttempted);
                    }
                }
                Err(Stop::Conflict {
                    op,
                    remote_tip,
                    pre_tip,
                }) => {
                    self.changed = true;
                    let paths = self.conflicted_paths();
                    let d = self.st.targets[i].decision.clone();
                    self.st.stopped = Some(StoppedAt {
                        remote: d.remote.clone(),
                        branch: d.branch.clone(),
                        op,
                        pre_tip,
                        remote_tip,
                    });
                    self.save();
                    let message = format!(
                        "stopped on conflicts in {} file{} integrating {}/{}: resolve them and \
                         conclude the {}, then continue the grouped run (or abort it)",
                        paths.len(),
                        if paths.len() == 1 { "" } else { "s" },
                        d.remote,
                        d.branch,
                        match op {
                            StoppedOp::Merge => "merge",
                            StoppedOp::Rebase => "rebase",
                        }
                    );
                    return Ok(Some(self.outcome(
                        GroupRunStatus::Conflicted,
                        message,
                        paths,
                        None,
                    )));
                }
                Err(Stop::Fail { failure, kind }) => {
                    // Refused before anything changed: a plain error, so the
                    // auto-stash layer can see local changes in the way.
                    if !self.changed {
                        return Err(*failure);
                    }
                    let t = &mut self.st.targets[i];
                    t.result.detail = Some(failure.0.lines().next().unwrap_or_default().into());
                    self.set_status(i, TargetStatus::Failed { kind });
                }
            }
        }
        Ok(None)
    }

    fn remote_known(&self, remote: &str) -> bool {
        self.backend
            .remotes(self.repo)
            .map(|rs| rs.iter().any(|r| r.name == remote))
            .unwrap_or(true)
    }

    /// The tracking ref when it is at `tip`, else a name for the oid itself:
    /// what a merge or rebase is told to take in.
    fn source_ref(&self, d: &RunTarget, tip: Oid) -> Ref {
        let tracking = self.backend.refs(self.repo).ok().and_then(|refs| {
            refs.into_iter().find(|r| {
                matches!(&r.kind, RefKind::RemoteBranch { remote, name }
                    if *remote == d.remote && *name == d.branch)
                    && r.target == tip
            })
        });
        tracking.unwrap_or(Ref {
            kind: RefKind::Tag { name: tip.to_hex() },
            target: tip,
            visibility: Default::default(),
            is_head: false,
            upstream: None,
            tag_sync: None,
            multi_remote: None,
        })
    }

    async fn integrate_one(&mut self, i: usize) -> Result<Integration, Stop> {
        let d = self.st.targets[i].decision.clone();
        let pre_tip = self
            .local_tip()
            .ok_or_else(|| stop_plain(format!("no such branch: {}", self.st.branch)))?;
        match self.st.mode {
            GroupMode::Pull => self.pull_one(&d, pre_tip).await,
            GroupMode::Push => {
                let tip = d.lease.ok_or_else(|| {
                    stop_plain(format!(
                        "{}/{} was not seen by a plan: plan again before integrating it",
                        d.remote, d.branch
                    ))
                })?;
                self.backend
                    .ahead_behind(self.repo, pre_tip, tip)
                    .map_err(|_| {
                        stop_plain(format!(
                            "{}'s tip {} is not in this repository: plan again",
                            d.remote,
                            gitpoulpi_core::domain::short(tip)
                        ))
                    })?;
                let from = self.source_ref(&d, tip);
                self.integrate_oid(&d, &from, tip, pre_tip)
            }
        }
    }

    fn integrate_oid(
        &self,
        d: &RunTarget,
        from: &Ref,
        tip: Oid,
        pre_tip: Oid,
    ) -> Result<Integration, Stop> {
        let backend = self.backend;
        let repo = self.repo;
        match d.action {
            GroupAction::FastForwardFrom => {
                if self.head_is_branch() {
                    let opts = MergeOptions {
                        fast_forward_only: true,
                        ..MergeOptions::default()
                    };
                    match backend.merge(repo, from, &opts).map_err(stop_of)? {
                        MergeOutcome::UpToDate => Ok(Integration::UpToDate),
                        MergeOutcome::FastForwarded(_) | MergeOutcome::Merged(_) => {
                            Ok(Integration::FastForwarded)
                        }
                        MergeOutcome::Conflicted(_) => Err(Stop::Conflict {
                            op: StoppedOp::Merge,
                            remote_tip: tip,
                            pre_tip,
                        }),
                    }
                } else {
                    match backend
                        .fast_forward(repo, &self.st.branch, tip)
                        .map_err(stop_of)?
                    {
                        gitpoulpi_core::domain::FastForwardOutcome::UpToDate => {
                            Ok(Integration::UpToDate)
                        }
                        gitpoulpi_core::domain::FastForwardOutcome::FastForwarded { .. } => {
                            Ok(Integration::FastForwarded)
                        }
                    }
                }
            }
            GroupAction::MergeFrom => {
                match backend
                    .merge(repo, from, &MergeOptions::default())
                    .map_err(stop_of)?
                {
                    MergeOutcome::UpToDate => Ok(Integration::UpToDate),
                    MergeOutcome::FastForwarded(_) => Ok(Integration::FastForwarded),
                    MergeOutcome::Merged(_) => Ok(Integration::Merged),
                    MergeOutcome::Conflicted(_) => Err(Stop::Conflict {
                        op: StoppedOp::Merge,
                        remote_tip: tip,
                        pre_tip,
                    }),
                }
            }
            GroupAction::RebaseOnto => {
                match backend
                    .rebase(repo, from, &RebaseOptions::default())
                    .map_err(stop_of)?
                {
                    RebaseOutcome::Completed => {
                        if self.local_tip() == Some(pre_tip) {
                            Ok(Integration::UpToDate)
                        } else {
                            Ok(Integration::Rebased)
                        }
                    }
                    RebaseOutcome::Conflicted { .. } | RebaseOutcome::Stopped { .. } => {
                        Err(Stop::Conflict {
                            op: StoppedOp::Rebase,
                            remote_tip: tip,
                            pre_tip,
                        })
                    }
                }
            }
            _ => Err(stop_plain("not an integration")),
        }
    }

    async fn pull_one(&mut self, d: &RunTarget, pre_tip: Oid) -> Result<Integration, Stop> {
        let mode = match d.action {
            GroupAction::FastForwardFrom => PullMode::FastForwardOnly,
            GroupAction::RebaseOnto => PullMode::Rebase,
            _ => PullMode::Merge,
        };
        let opts = PullOptions {
            mode,
            remote: Some(d.remote.clone()),
            remote_branch: Some(d.branch.clone()),
        };
        let remote_tip = |this: &Self| {
            this.backend.refs(this.repo).ok().and_then(|refs| {
                refs.into_iter().find_map(|r| match &r.kind {
                    RefKind::RemoteBranch { remote, name }
                        if *remote == d.remote && *name == d.branch =>
                    {
                        Some(r.target)
                    }
                    _ => None,
                })
            })
        };
        self.backend
            .pull_fetch(self.repo, &opts)
            .await
            .map_err(stop_of)?;
        match self
            .backend
            .pull_integrate(self.repo, &opts)
            .await
            .map_err(stop_of)?
        {
            PullOutcome::UpToDate => Ok(Integration::UpToDate),
            PullOutcome::FastForwarded(_) => Ok(Integration::FastForwarded),
            PullOutcome::Merged(_) => Ok(if mode == PullMode::Rebase {
                Integration::Rebased
            } else {
                Integration::Merged
            }),
            PullOutcome::Conflicted => Err(Stop::Conflict {
                op: if mode == PullMode::Rebase {
                    StoppedOp::Rebase
                } else {
                    StoppedOp::Merge
                },
                remote_tip: remote_tip(self).or(d.lease).unwrap_or(pre_tip),
                pre_tip,
            }),
        }
    }

    /// After resuming from conflicts: ask the pending push targets again.
    /// A target whose plain push no longer works (it holds the old commits a
    /// rebase replaced, or moved on) is not forced silently: the run stops
    /// with a fresh plan and nothing pushed.
    async fn check_replan(&mut self) -> Result<Option<RemoteGroupOutcome>, Failure> {
        let wanted: Vec<usize> = (0..self.st.targets.len())
            .filter(|&i| {
                self.pending(i)
                    && matches!(
                        self.st.targets[i].decision.action,
                        GroupAction::Push
                            | GroupAction::FastForwardFrom
                            | GroupAction::MergeFrom
                            | GroupAction::RebaseOnto
                    )
            })
            .collect();
        if wanted.is_empty() {
            return Ok(None);
        }
        let req = RemoteGroupPlanRequest {
            branch: self.st.branch.clone(),
            mode: GroupMode::Push,
            targets: Some(
                wanted
                    .iter()
                    .map(|&i| {
                        let d = &self.st.targets[i].decision;
                        TargetRef {
                            remote: d.remote.clone(),
                            branch: d.branch.clone(),
                        }
                    })
                    .collect(),
            ),
            preflight: true,
            prefer_rebase: false,
            refused: Vec::new(),
        };
        let plan = match super::plan::build(self.backend, self.repo, &req).await {
            Ok(plan) => plan,
            // Cannot ask: the pushes themselves will say.
            Err(_) => return Ok(None),
        };
        let blocked: Vec<String> = plan
            .targets
            .iter()
            .filter(|p| {
                p.error.is_none()
                    && !matches!(
                        p.relation,
                        RemoteRelation::UpToDate
                            | RemoteRelation::LocalAhead(_)
                            | RemoteRelation::Absent
                    )
            })
            .map(|p| p.remote.clone())
            .collect();
        if blocked.is_empty() {
            return Ok(None);
        }
        self.save();
        let message = format!(
            "a plain push to {} no longer works from the resolved branch; nothing was pushed: \
             decide for them (continue with new decisions, or abort)",
            blocked.join(", ")
        );
        Ok(Some(self.outcome(
            GroupRunStatus::NeedsResolution,
            message,
            Vec::new(),
            Some(plan),
        )))
    }

    async fn push_all(&mut self) -> Result<Option<RemoteGroupOutcome>, Failure> {
        let total = self.st.targets.len();
        let remotes = self.backend.remotes(self.repo).unwrap_or_default();
        let primary_exists = self
            .backend
            .config_section(self.repo, &format!("branch.{}", self.st.branch))
            .map(|e| gitpoulpi_core::domain::parse_branch_remotes(&e, &self.st.branch).primary)
            .map(|p| p.is_some())
            .unwrap_or(true);
        let mut twin_keys: Vec<(String, String, usize)> = Vec::new();
        for i in 0..total {
            if !self.pending(i) {
                continue;
            }
            let d = self.st.targets[i].decision.clone();
            // Same URL, same branch as a target already pushed: once.
            let remote_cfg = remotes.iter().find(|r| r.name == d.remote);
            let Some(remote_cfg) = remote_cfg else {
                self.set_status(i, TargetStatus::RemoteMissing);
                continue;
            };
            crate::test_hook::step_delay().await;
            if crate::jobs::cancel_requested_now() {
                return Ok(Some(self.canceled(None)));
            }
            let key = normalize_url(
                remote_cfg
                    .push_url
                    .as_deref()
                    .unwrap_or(&remote_cfg.fetch_url),
            );
            let twin_of = twin_keys
                .iter()
                .find(|(u, b, j)| {
                    *u == key
                        && *b == d.branch
                        && matches!(
                            self.st.targets[*j].result.status,
                            TargetStatus::Pushed { .. } | TargetStatus::UpToDate
                        )
                })
                .map(|(_, _, j)| self.st.targets[*j].decision.remote.clone());
            if let Some(first) = twin_of {
                self.st.targets[i].result.detail = Some(format!("same URL as {first}"));
                self.set_status(i, TargetStatus::UpToDate);
                continue;
            }
            let local_tip = self
                .local_tip()
                .ok_or_else(|| plain(format!("no such branch: {}", self.st.branch)))?;
            // A fast-forward that left local exactly at the remote's tip
            // has nothing to send.
            if d.action == GroupAction::FastForwardFrom
                && self.st.targets[i].integrated
                && d.lease == Some(local_tip)
            {
                self.set_status(i, TargetStatus::UpToDate);
                twin_keys.push((key, d.branch.clone(), i));
                continue;
            }
            crate::jobs::set_phase(&format!("push {} ({}/{total})", d.remote, i + 1));
            self.emit(i, TargetStatus::Pushing);
            let first_new_upstream = i == 0 && !primary_exists;
            let opts = match self.push_options(&d, first_new_upstream) {
                Ok(opts) => opts,
                Err(message) => {
                    self.st.targets[i].result.detail = Some(message);
                    self.set_status(
                        i,
                        TargetStatus::Failed {
                            kind: FailureKind::Other,
                        },
                    );
                    continue;
                }
            };
            let pushed = {
                let push = self.backend.push(self.repo, &d.remote, &opts);
                tokio::select! {
                    r = push => Some(r),
                    () = crate::jobs::cancel_requested_in_job() => None,
                }
            };
            let Some(pushed) = pushed else {
                return Ok(Some(self.canceled(Some(i))));
            };
            match pushed {
                Ok(_) => {
                    self.changed = true;
                    let created = d.absent;
                    let t = &mut self.st.targets[i];
                    if d.action == GroupAction::ForceWithLease {
                        if let Some(old) = d.lease {
                            t.result.replaced = Some(old);
                            t.result.recovery = Some(format!(
                                "git push --force-with-lease=refs/heads/{dst}:{new} {remote} \
                                 {old}:refs/heads/{dst}",
                                dst = d.branch,
                                new = local_tip,
                                remote = d.remote,
                                old = old,
                            ));
                        }
                    }
                    self.set_status(i, TargetStatus::Pushed { created });
                    twin_keys.push((key, d.branch.clone(), i));
                    if self.st.tags {
                        let summary = match self
                            .backend
                            .tag_push(self.repo, &d.remote, &TagSelection::AllUnpushed)
                            .await
                        {
                            Ok(out) => format!("pushed {} tag(s)", out.pushed_refs.len()),
                            Err(e) => format!("tags failed: {e}"),
                        };
                        self.st.targets[i].result.tags = Some(summary);
                    }
                }
                Err(Error::PushRejected(r)) => {
                    let detail = r
                        .detail
                        .as_ref()
                        .map(|d| d.stderr.trim().to_string())
                        .filter(|s| !s.is_empty());
                    self.st.targets[i].result.detail = detail;
                    self.set_status(i, TargetStatus::Rejected { reason: r.reason });
                }
                Err(e) => {
                    let kind = e.detail().map_or(FailureKind::Other, |d| d.kind());
                    self.st.targets[i].result.detail =
                        Some(e.to_string().lines().next().unwrap_or_default().to_string());
                    self.set_status(i, TargetStatus::Failed { kind });
                }
            }
        }
        Ok(None)
    }

    /// What the push of `d` is told: the branch, the destination name, the
    /// explicit lease for a force or a new branch, never the bare one.
    fn push_options(&self, d: &RunTarget, set_upstream: bool) -> Result<PushOptions, String> {
        let lease = match d.action {
            GroupAction::ForceWithLease => Some(match (d.lease, d.absent) {
                (Some(oid), _) => PushLease::Expect(oid),
                (None, true) => PushLease::Absent,
                (None, false) => {
                    return Err(format!(
                        "no lease recorded for {}: plan again before forcing",
                        d.remote
                    ))
                }
            }),
            _ if d.absent => Some(PushLease::Absent),
            _ => None,
        };
        Ok(PushOptions {
            force_with_lease: lease.is_some(),
            set_upstream,
            branch: Some(self.st.branch.clone()),
            lease,
            remote_branch: (d.branch != self.st.branch).then(|| d.branch.clone()),
        })
    }

    /// Every target has a status: summarise, and keep the run only when
    /// something is left to retry.
    fn finish(&mut self) -> RemoteGroupOutcome {
        let retry_left = self
            .st
            .targets
            .iter()
            .any(|t| t.result.status.is_retryable(true));
        if retry_left {
            self.save();
        } else {
            state::clear(self.repo);
        }
        let all_ok = self.st.targets.iter().all(|t| t.result.status.is_ok());
        let total = self.st.targets.len();
        let ok = self
            .st
            .targets
            .iter()
            .filter(|t| t.result.status.is_ok())
            .count();
        let verb = if self.st.mode == GroupMode::Push {
            "pushed to"
        } else {
            "took in"
        };
        let mut message = if all_ok {
            format!("{verb} {ok} of {total}")
        } else {
            let problems: Vec<String> = self
                .st
                .targets
                .iter()
                .filter(|t| !t.result.status.is_ok())
                .map(|t| {
                    format!(
                        "{}: {}",
                        t.result.remote,
                        t.result
                            .detail
                            .clone()
                            .unwrap_or_else(|| describe(&t.result.status))
                    )
                })
                .collect();
            format!("{verb} {ok} of {total}; {}", problems.join("; "))
        };
        if self.st.targets.iter().any(|t| t.result.replaced.is_some()) {
            message.push_str(" (a forced push replaced remote commits: see recovery)");
        }
        let status = if all_ok {
            GroupRunStatus::Done
        } else {
            GroupRunStatus::Problems
        };
        let mut out = self.outcome(status, message, Vec::new(), None);
        out.pending = retry_left;
        out
    }
}

fn describe(status: &TargetStatus) -> String {
    match status {
        TargetStatus::Rejected { reason } => format!("rejected ({reason:?})"),
        TargetStatus::Failed { kind } => format!("failed ({kind:?})"),
        TargetStatus::RemoteMissing => "the remote is not configured".into(),
        TargetStatus::Unknown => "unknown".into(),
        TargetStatus::NotAttempted => "not attempted".into(),
        other => format!("{other:?}"),
    }
}
