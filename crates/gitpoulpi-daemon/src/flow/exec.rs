//! Running the steps of a plan, and resuming or aborting a stopped flow.

// `Failure` is the dispatcher's own (large) error tuple.
#![allow(clippy::result_large_err)]

use gitpoulpi_core::domain::{
    short, CheckoutTarget, CommitOptions, FileStatus, HeadState, MergeOptions, MergeOutcome,
    PushOptions, RebaseOptions, RebaseOutcome, Ref, RefKind, RefScope, RepoRef, ResetMode,
};
use gitpoulpi_core::workflow::{
    FlowOutcome, FlowRequest, FlowState, FlowStatus, FlowStep, MergeRequestSpec, MergeStrategy,
    StepReport, StepStatus, RECORDED_PARENT_VARIABLE,
};
use gitpoulpi_core::GitBackend;

use super::facts::{backend_failure, plain_failure};
use super::{hooks, plan_of, state};
use crate::dispatch::Failure;

/// Why a step did not finish.
enum Stop {
    Conflict {
        paths: Vec<String>,
        squash_message: Option<String>,
    },
    Fail {
        failure: Box<Failure>,
        exit_code: Option<i32>,
    },
}

fn stop(e: gitpoulpi_core::Error) -> Stop {
    Stop::Fail {
        failure: Box::new(backend_failure(e)),
        exit_code: None,
    }
}

fn fail(message: impl Into<String>) -> Stop {
    Stop::Fail {
        failure: Box::new(plain_failure(message)),
        exit_code: None,
    }
}

/// How a step that finished went.
struct Ran {
    status: StepStatus,
    output: Option<String>,
    exit_code: Option<i32>,
}

impl Ran {
    fn done(output: impl Into<Option<String>>) -> Self {
        Self {
            status: StepStatus::Done,
            output: output.into(),
            exit_code: None,
        }
    }

    fn skipped(note: &str) -> Self {
        Self {
            status: StepStatus::Skipped,
            output: Some(note.to_string()),
            exit_code: None,
        }
    }
}

struct Exec<'a> {
    backend: &'a dyn GitBackend,
    repo: &'a RepoRef,
    req: FlowRequest,
    steps: Vec<FlowStep>,
    reports: Vec<StepReport>,
    original_head: Option<String>,
    branch: Option<String>,
    mr: Option<MergeRequestSpec>,
    /// Something was changed: a failure from here on is reported with the
    /// flow saved, not as a plain error.
    changed: bool,
}

pub(super) fn in_progress(repo: &RepoRef) -> Option<String> {
    crate::journal::git_dir(&repo.root).and_then(|d| crate::journal::in_progress(&d))
}

impl<'a> Exec<'a> {
    fn head_branch(&self) -> Option<String> {
        let snap =
            crate::journal_snap::ref_snapshot(self.backend, self.repo, &RefScope::heads()).ok()?;
        match snap.head {
            HeadState::Branch(b) => Some(b),
            HeadState::Detached(_) => None,
        }
    }

    fn local_ref(&self, name: &str) -> Result<Ref, Stop> {
        self.backend
            .refs(self.repo)
            .map_err(stop)?
            .into_iter()
            .find(|r| matches!(&r.kind, RefKind::LocalBranch { name: n } if n == name))
            .ok_or_else(|| fail(format!("there is no local branch {name}")))
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

    fn staged_changes(&self) -> bool {
        self.backend.status(self.repo).is_ok_and(|tree| {
            tree.files
                .iter()
                .any(|f| f.index.is_some_and(|s| s != FileStatus::Untracked))
        })
    }

    fn outcome(
        &self,
        status: FlowStatus,
        message: String,
        conflicts: Vec<String>,
        resumable: bool,
    ) -> FlowOutcome {
        FlowOutcome {
            action: self.req.action,
            status,
            branch: self.branch.clone(),
            steps: self.reports.clone(),
            conflicts,
            message,
            merge_request: self.mr.clone().filter(|_| status == FlowStatus::Done),
            resumable,
        }
    }

    fn report(
        &mut self,
        step: &FlowStep,
        status: StepStatus,
        output: Option<String>,
        code: Option<i32>,
    ) {
        self.reports.push(StepReport {
            step: step.clone(),
            status,
            output,
            exit_code: code,
        });
    }

    fn not_run(&mut self, from: usize) {
        let rest: Vec<FlowStep> = self.steps[from.min(self.steps.len())..].to_vec();
        for step in &rest {
            self.report(step, StepStatus::NotRun, None, None);
        }
    }

    fn save(&self, next: usize, conflict: bool, squash_message: Option<String>) {
        state::save(
            self.repo,
            &FlowState {
                request: self.req.clone(),
                steps: self.steps.clone(),
                next,
                conflict,
                original_head: self.original_head.clone(),
                squash_message,
                merge_request: self.mr.clone(),
            },
        );
    }

    /// The flow stopped at step `i` on conflicts.
    fn stopped_on_conflict(
        &mut self,
        i: usize,
        paths: Vec<String>,
        squash_message: Option<String>,
    ) -> FlowOutcome {
        self.changed = true;
        let step = self.steps[i].clone();
        self.report(
            &step,
            StepStatus::Conflicted,
            Some(format!("conflicts in {}", paths.join(", "))),
            None,
        );
        self.not_run(i + 1);
        self.save(i, true, squash_message);
        let word = self.req.action.word();
        self.outcome(
            FlowStatus::Conflicted,
            format!(
                "stopped on conflicts in {} file{}: resolve them, then `flow {word} --continue` \
                 (or `--abort`)",
                paths.len(),
                if paths.len() == 1 { "" } else { "s" }
            ),
            paths,
            true,
        )
    }

    async fn run_from(&mut self, from: usize) -> Result<FlowOutcome, Failure> {
        for i in from..self.steps.len() {
            crate::test_hook::step_delay().await;
            // A cancel request is honoured between steps only: the step that
            // ran finished, the rest do not start, and the flow stops where
            // `--continue` or `--abort` can take it from.
            if crate::jobs::cancel_requested_now() {
                if !self.changed {
                    return Err(plain_failure("canceled before any step ran"));
                }
                self.not_run(i);
                self.save(i, false, None);
                let word = self.req.action.word();
                return Ok(self.outcome(
                    FlowStatus::Failed,
                    format!(
                        "canceled before step {}: the steps before it were done (`flow {word} \
                         --continue` runs the rest, or `--abort`)",
                        i + 1
                    ),
                    Vec::new(),
                    true,
                ));
            }
            let step = self.steps[i].clone();
            match self.run_step(&step).await {
                Ok(ran) => {
                    if ran.status == StepStatus::Done {
                        self.changed = true;
                    }
                    self.report(&step, ran.status, ran.output, ran.exit_code);
                }
                Err(Stop::Conflict {
                    paths,
                    squash_message,
                }) => return Ok(self.stopped_on_conflict(i, paths, squash_message)),
                Err(Stop::Fail { failure, exit_code }) => {
                    let failure = *failure;
                    if !self.changed {
                        return Err(failure);
                    }
                    self.report(
                        &step,
                        StepStatus::Failed,
                        Some(failure.0.clone()),
                        exit_code,
                    );
                    self.not_run(i + 1);
                    self.save(i, false, None);
                    let word = self.req.action.word();
                    return Ok(self.outcome(
                        FlowStatus::Failed,
                        format!(
                            "stopped at step {}: {} (fix it, then `flow {word} --continue` to retry \
                             the step, or `--abort`)",
                            i + 1,
                            failure.0.lines().next().unwrap_or_default()
                        ),
                        Vec::new(),
                        true,
                    ));
                }
            }
        }
        state::clear(self.repo);
        let message = self.done_message();
        Ok(self.outcome(FlowStatus::Done, message, Vec::new(), false))
    }

    fn done_message(&self) -> String {
        let branch = self.branch.clone().unwrap_or_default();
        let mut text = match self.req.action {
            gitpoulpi_core::workflow::FlowAction::Start => format!("started {branch}"),
            gitpoulpi_core::workflow::FlowAction::Finish => format!("finished {branch}"),
            gitpoulpi_core::workflow::FlowAction::Sync => format!("updated {branch}"),
            gitpoulpi_core::workflow::FlowAction::Promote => format!("promoted into {branch}"),
            gitpoulpi_core::workflow::FlowAction::SetParent => {
                format!("recorded the parent of {branch}")
            }
            gitpoulpi_core::workflow::FlowAction::PickUp => format!("picked up {branch}"),
        };
        if let Some(mr) = &self.mr {
            text.push_str(&format!(
                "; {} is pushed, the merge request into {} is still to be opened",
                mr.source, mr.target
            ));
        }
        text
    }

    async fn run_step(&mut self, step: &FlowStep) -> Result<Ran, Stop> {
        let backend = self.backend;
        let repo = self.repo;
        match step {
            FlowStep::Checkout { branch } => {
                if self.head_branch().as_deref() == Some(branch.as_str()) {
                    return Ok(Ran::skipped("already checked out"));
                }
                backend
                    .checkout(repo, &CheckoutTarget::Branch(branch.clone()))
                    .map_err(stop)?;
                Ok(Ran::done(None))
            }
            FlowStep::CreateBranch { name, from } => {
                let at = backend.resolve(repo, from).map_err(stop)?;
                backend.branch_create(repo, name, at).map_err(stop)?;
                Ok(Ran::done(format!("at {}", short(at))))
            }
            FlowStep::SetUpstream { branch, upstream } => {
                backend
                    .branch_set_upstream(repo, branch, Some(upstream))
                    .map_err(stop)?;
                Ok(Ran::done(None))
            }
            FlowStep::RecordParent { branch, parent } => {
                let section = format!("branch.{branch}");
                let key = format!("{section}.{RECORDED_PARENT_VARIABLE}");
                let mut entries = backend.config_section(repo, &section).map_err(stop)?;
                entries.retain(|(k, _)| !k.eq_ignore_ascii_case(&key));
                if let Some(parent) = parent {
                    entries.push((key, parent.clone()));
                }
                backend
                    .set_config_section(repo, &section, &entries)
                    .map_err(stop)?;
                Ok(Ran::done(None))
            }
            FlowStep::Push {
                remote,
                branch,
                set_upstream,
            } => {
                backend
                    .push(
                        repo,
                        remote,
                        &PushOptions {
                            set_upstream: *set_upstream,
                            branch: Some(branch.clone()),
                            ..PushOptions::default()
                        },
                    )
                    .await
                    .map_err(stop)?;
                Ok(Ran::done(None))
            }
            FlowStep::Merge {
                branch,
                into,
                strategy,
                no_ff,
                message,
            } => {
                if self.head_branch().as_deref() != Some(into.as_str()) {
                    return Err(fail(format!(
                        "{into} should be checked out to merge {branch} into it"
                    )));
                }
                let from = self.local_ref(branch)?;
                let squash_message = message
                    .clone()
                    .unwrap_or_else(|| format!("Squashed commit of '{branch}'"));
                let opts = match strategy {
                    MergeStrategy::Merge => MergeOptions {
                        no_ff: *no_ff,
                        ..MergeOptions::default()
                    },
                    MergeStrategy::Squash => MergeOptions {
                        squash: true,
                        message: Some(squash_message.clone()),
                        ..MergeOptions::default()
                    },
                    MergeStrategy::Rebase | MergeStrategy::FastForward => MergeOptions {
                        fast_forward_only: true,
                        ..MergeOptions::default()
                    },
                };
                match backend.merge(repo, &from, &opts).map_err(stop)? {
                    MergeOutcome::Conflicted(paths) => Err(Stop::Conflict {
                        paths,
                        squash_message: (*strategy == MergeStrategy::Squash)
                            .then_some(squash_message),
                    }),
                    MergeOutcome::UpToDate => Ok(Ran::skipped("already up to date")),
                    MergeOutcome::FastForwarded(oid) => {
                        Ok(Ran::done(format!("fast-forwarded to {}", short(oid))))
                    }
                    MergeOutcome::Merged(oid) => Ok(Ran::done(format!("now at {}", short(oid)))),
                }
            }
            FlowStep::Rebase { branch, onto } => {
                if self.head_branch().as_deref() != Some(branch.as_str()) {
                    backend
                        .checkout(repo, &CheckoutTarget::Branch(branch.clone()))
                        .map_err(stop)?;
                }
                let onto = self.local_ref(onto)?;
                match backend
                    .rebase(repo, &onto, &RebaseOptions::default())
                    .map_err(stop)?
                {
                    RebaseOutcome::Completed => Ok(Ran::done(None)),
                    RebaseOutcome::Conflicted { .. } | RebaseOutcome::Stopped { .. } => {
                        Err(Stop::Conflict {
                            paths: self.conflicted_paths(),
                            squash_message: None,
                        })
                    }
                }
            }
            FlowStep::Tag { name, at, message } => {
                let oid = backend.resolve(repo, at).map_err(stop)?;
                backend
                    .tag_create(repo, name, oid, Some(message))
                    .map_err(stop)?;
                Ok(Ran::done(None))
            }
            FlowStep::DeleteBranch { name } => {
                if self.head_branch().as_deref() == Some(name.as_str()) {
                    return Err(fail(format!("{name} is checked out and cannot be deleted")));
                }
                backend.branch_delete(repo, name).map_err(stop)?;
                Ok(Ran::done(None))
            }
            FlowStep::DeleteRemoteBranch { remote, name } => {
                backend
                    .branch_delete_remote(repo, remote, name)
                    .await
                    .map_err(stop)?;
                Ok(Ran::done(None))
            }
            FlowStep::OpenMergeRequest { .. } => Ok(Ran {
                status: StepStatus::Pending,
                output: Some("left to the client (it holds the hosting token)".to_string()),
                exit_code: None,
            }),
            FlowStep::Hook { phase, command } => {
                let ran = hooks::run(
                    repo.root.clone(),
                    command.clone(),
                    self.req.action.word(),
                    self.branch.clone(),
                )
                .await
                .map_err(fail)?;
                if ran.exit_code != 0 {
                    return Err(Stop::Fail {
                        failure: Box::new(plain_failure(format!(
                            "the {phase} hook `{command}` exited with {}:\n{}",
                            ran.exit_code, ran.output
                        ))),
                        exit_code: Some(ran.exit_code),
                    });
                }
                Ok(Ran {
                    status: StepStatus::Done,
                    output: Some(ran.output),
                    exit_code: Some(ran.exit_code),
                })
            }
        }
    }

    /// Completes the step a conflict stopped at, once the user resolved it.
    fn complete_interrupted(&mut self, state: &FlowState) -> Result<Ran, Stop> {
        let step = self.steps.get(state.next).cloned();
        match in_progress(self.repo).as_deref() {
            Some("merge") => {
                let message = match &step {
                    Some(FlowStep::Merge { branch, into, .. }) => {
                        format!("Merge branch '{branch}' into {into}")
                    }
                    _ => "Merge".to_string(),
                };
                self.backend
                    .commit(self.repo, &message, &CommitOptions::default())
                    .map_err(stop)?;
                Ok(Ran::done("committed the merge".to_string()))
            }
            Some("rebase") => match self.backend.rebase_continue(self.repo).map_err(stop)? {
                RebaseOutcome::Completed => Ok(Ran::done("rebase continued".to_string())),
                _ => Err(Stop::Conflict {
                    paths: self.conflicted_paths(),
                    squash_message: None,
                }),
            },
            Some(other) => Err(fail(format!(
                "a {other} is in progress: finish or abort it first"
            ))),
            None => match &state.squash_message {
                Some(message) if self.staged_changes() => {
                    self.backend
                        .commit(self.repo, message, &CommitOptions::default())
                        .map_err(stop)?;
                    Ok(Ran::done("committed the squash".to_string()))
                }
                _ => Ok(Ran::skipped("already completed")),
            },
        }
    }
}

fn exec_for<'a>(backend: &'a dyn GitBackend, repo: &'a RepoRef, state: &FlowState) -> Exec<'a> {
    Exec {
        backend,
        repo,
        req: state.request.clone(),
        steps: state.steps.clone(),
        reports: Vec::new(),
        original_head: state.original_head.clone(),
        branch: state.request.branch.clone().or_else(|| {
            // A start's branch is the one its create step names.
            state.steps.iter().find_map(|s| match s {
                FlowStep::CreateBranch { name, .. } => Some(name.clone()),
                _ => None,
            })
        }),
        mr: state.merge_request.clone(),
        changed: false,
    }
}

/// A fresh run of `req`.
#[allow(clippy::result_large_err)]
pub(super) async fn start_run(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    req: &FlowRequest,
) -> Result<FlowOutcome, Failure> {
    if let Some(old) = state::load(repo) {
        let busy = in_progress(repo).is_some()
            || backend
                .conflict_entries(repo)
                .map(|c| !c.is_empty())
                .unwrap_or(false);
        if old.conflict && busy {
            return Err(plain_failure(format!(
                "a flow `{0}` is stopped on a conflict: `flow {0} --continue` or `--abort` it first",
                old.request.action.word()
            )));
        }
        state::clear(repo);
    }
    let plan = plan_of(backend, repo, req)?;
    if !plan.can_run() {
        return Err(plain_failure(plan.blockers.join("; ")));
    }
    let mut ex = Exec {
        backend,
        repo,
        req: req.clone(),
        steps: plan.steps.clone(),
        reports: Vec::new(),
        original_head: None,
        branch: plan.branch.clone(),
        mr: plan.merge_request.clone(),
        changed: false,
    };
    ex.original_head = ex.head_branch();
    if plan.steps.is_empty() {
        let message = if plan.warnings.is_empty() {
            "nothing to do".to_string()
        } else {
            plan.warnings.join("; ")
        };
        return Ok(ex.outcome(FlowStatus::NothingToDo, message, Vec::new(), false));
    }
    ex.run_from(0).await
}

/// `resume: continue`.
#[allow(clippy::result_large_err)]
pub(super) async fn resume_continue(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    req: &FlowRequest,
) -> Result<FlowOutcome, Failure> {
    let saved =
        state::load(repo).ok_or_else(|| plain_failure("no flow is waiting to be continued"))?;
    if saved.request.action != req.action {
        return Err(plain_failure(format!(
            "the stopped flow is `{}`, not `{}`",
            saved.request.action.word(),
            req.action.word()
        )));
    }
    let mut ex = exec_for(backend, repo, &saved);
    let mut start = saved.next;
    if saved.conflict {
        let still = ex.conflicted_paths();
        if !still.is_empty() {
            return Err(plain_failure(format!(
                "{} file{} still in conflict ({}): resolve and mark them resolved first",
                still.len(),
                if still.len() == 1 { " is" } else { "s are" },
                still.join(", ")
            )));
        }
        match ex.complete_interrupted(&saved) {
            Ok(ran) => {
                if let Some(step) = ex.steps.get(saved.next).cloned() {
                    ex.report(&step, ran.status, ran.output, ran.exit_code);
                }
                ex.changed = true;
                start = saved.next + 1;
            }
            Err(Stop::Conflict {
                paths,
                squash_message,
            }) => {
                return Ok(ex.stopped_on_conflict(
                    saved.next,
                    paths,
                    squash_message.or(saved.squash_message.clone()),
                ))
            }
            Err(Stop::Fail { failure, .. }) => return Err(*failure),
        }
    }
    ex.run_from(start).await
}

/// `resume: abort`.
#[allow(clippy::result_large_err)]
pub(super) fn resume_abort(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    req: &FlowRequest,
) -> Result<FlowOutcome, Failure> {
    let saved =
        state::load(repo).ok_or_else(|| plain_failure("no flow is waiting to be aborted"))?;
    if saved.request.action != req.action {
        return Err(plain_failure(format!(
            "the stopped flow is `{}`, not `{}`",
            saved.request.action.word(),
            req.action.word()
        )));
    }
    let ex = exec_for(backend, repo, &saved);
    if in_progress(repo).is_some() {
        backend.operation_abort(repo).map_err(backend_failure)?;
    } else if saved.conflict && saved.squash_message.is_some() {
        // A squash leaves no operation behind, only a conflicted index.
        let head = backend.resolve(repo, "HEAD").map_err(backend_failure)?;
        backend
            .reset(repo, head, ResetMode::Hard)
            .map_err(backend_failure)?;
    }
    if let Some(original) = &saved.original_head {
        if ex.head_branch().as_deref() != Some(original.as_str())
            && backend
                .checkout(repo, &CheckoutTarget::Branch(original.clone()))
                .is_err()
        {
            tracing::warn!(target: "daemon", "could not return to {original} after aborting the flow");
        }
    }
    state::clear(repo);
    let message = match &saved.original_head {
        Some(b) => format!("aborted; back on {b}"),
        None => "aborted".to_string(),
    };
    Ok(ex.outcome(FlowStatus::Aborted, message, Vec::new(), false))
}
