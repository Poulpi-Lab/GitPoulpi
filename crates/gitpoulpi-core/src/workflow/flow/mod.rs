//! Workflow **actions**: start, finish, sync, promote, set-parent. This
//! module is the wire vocabulary (what `OperationKind::FlowPlan` and
//! `OperationKind::FlowRun` carry) and the pure planner (`plan.rs`); the
//! daemon gathers the facts, runs the steps and journals them.
//!
//! One request type serves both operations: `FlowPlan` answers a
//! [`FlowPlan`] (every git step the action would do, plus blockers and
//! warnings, nothing changes), `FlowRun` plans again and executes, answering
//! a [`FlowOutcome`]. A run that stops on a conflict is resumed with the
//! same action and `resume: continue`, or given up with `resume: abort`.

mod plan;

use serde::{Deserialize, Serialize};

use super::{Fields, FinishMode, MergeStrategy};

pub use plan::{branch_title, plan_flow, FlowFacts};

fn is_false(b: &bool) -> bool {
    !*b
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowAction {
    #[default]
    Start,
    Finish,
    Sync,
    Promote,
    SetParent,
    /// Take over a branch a host or bot made (host-driven flows): create the
    /// local tracking branch of `remote/branch`, record its parent, check it out.
    PickUp,
}

impl FlowAction {
    pub fn word(self) -> &'static str {
        match self {
            FlowAction::Start => "start",
            FlowAction::Finish => "finish",
            FlowAction::Sync => "sync",
            FlowAction::Promote => "promote",
            FlowAction::SetParent => "set-parent",
            FlowAction::PickUp => "pick-up",
        }
    }
}

/// What to do with a flow a conflict stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowResume {
    /// Resolve the conflict first; this finishes the step it stopped at
    /// (commits the merge) and runs the remaining ones.
    Continue,
    /// Abort the git operation in progress, go back to the branch the flow
    /// started on and forget the remaining steps. Steps already completed
    /// stay (take them back with Undo).
    Abort,
}

/// How `sync` brings the parent's commits in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncMethod {
    Merge,
    Rebase,
}

/// The arguments of `FlowPlan` and `FlowRun`. Every field but `action` is
/// optional; which ones matter depends on the action.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct FlowRequest {
    pub action: FlowAction,
    /// start: the topic type id (`feature`, `hotfix`, ...).
    pub topic: Option<String>,
    /// start: the values of the pattern's tokens (`slug`, `version`, ...).
    /// `user` and `date` default to the user name and today.
    pub fields: Fields,
    /// start: the start point (a branch, a remote branch, a tag); default is
    /// the type's parent.
    pub from: Option<String>,
    /// start: push the new branch with `-u` afterwards.
    pub publish: bool,
    /// pick-up: the branch to take over, as named on the remote (without the
    /// remote). finish, sync, promote, set-parent: the branch; finish and sync
    /// default to the checked-out one.
    pub branch: Option<String>,
    /// set-parent: the new parent. pick-up: the branch the merge request
    /// targets (recorded as the parent when it exists locally).
    pub parent: Option<String>,
    /// set-parent: forget the recorded parent instead.
    pub clear: bool,
    /// sync: also update the branches stacked on this one, in order.
    pub stack: bool,
    /// sync: merge or rebase; default follows the branch's configured
    /// strategy (a `rebase` strategy rebases, anything else merges).
    pub method: Option<SyncMethod>,
    /// finish: `local-merge` or `merge-request`; needed when the type says
    /// `ask`, overrides it otherwise.
    pub finish_mode: Option<FinishMode>,
    /// finish: the merge request / squash commit title (default from the
    /// branch name).
    pub title: Option<String>,
    /// finish: delete the local branch afterwards (default: the type's
    /// `delete_after_finish`).
    pub delete: Option<bool>,
    /// finish: also delete the branch on its remote (needs `delete`).
    pub delete_remote: bool,
    /// The remote to publish to / open the merge request from (default
    /// `origin`, else the only one).
    pub remote: Option<String>,
    /// Run the configured hooks. They are shell commands from a file the
    /// team shares, so they never run unless asked for.
    pub run_hooks: bool,
    /// finish, promote: go on although the target is behind its remote.
    pub allow_behind: bool,
    /// Resume or abort a flow a conflict stopped (`action` must be the one
    /// that stopped).
    pub resume: Option<FlowResume>,
}

/// One thing a flow does. Serialized with an `op` tag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum FlowStep {
    Checkout {
        branch: String,
    },
    CreateBranch {
        name: String,
        /// A branch, a remote branch (`origin/x`), a tag or any revision.
        from: String,
    },
    /// `git branch --set-upstream-to`: the local `branch` tracks `upstream`
    /// (`origin/x`).
    SetUpstream {
        branch: String,
        upstream: String,
    },
    /// `branch.<name>.gitpoulpi-parent`; `None` removes it.
    RecordParent {
        branch: String,
        parent: Option<String>,
    },
    Push {
        remote: String,
        branch: String,
        set_upstream: bool,
    },
    /// Merge `branch` into the checked-out `into`.
    Merge {
        branch: String,
        into: String,
        strategy: MergeStrategy,
        /// Always make a merge commit (git-flow's `--no-ff`); without it a
        /// fast-forward is taken when possible. Only for `merge`.
        #[serde(default, skip_serializing_if = "is_false")]
        no_ff: bool,
        /// The commit message of a squash.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    /// Check `branch` out and rebase it onto `onto`.
    Rebase {
        branch: String,
        onto: String,
    },
    /// An annotated tag at the tip of `at`.
    Tag {
        name: String,
        at: String,
        message: String,
    },
    DeleteBranch {
        name: String,
    },
    DeleteRemoteBranch {
        remote: String,
        name: String,
    },
    /// Done by the client (it holds the hosting token), after the daemon's
    /// steps: the outcome carries the [`MergeRequestSpec`].
    OpenMergeRequest {
        source: String,
        target: String,
        title: String,
    },
    Hook {
        /// `after_start` or `before_finish`.
        phase: String,
        command: String,
    },
}

impl FlowStep {
    /// One line for a preview.
    pub fn describe(&self) -> String {
        match self {
            FlowStep::Checkout { branch } => format!("Check out {branch}"),
            FlowStep::CreateBranch { name, from } => format!("Create branch {name} from {from}"),
            FlowStep::SetUpstream { branch, upstream } => {
                format!("Make {branch} track {upstream}")
            }
            FlowStep::RecordParent { branch, parent } => match parent {
                Some(p) => format!("Record {p} as the parent of {branch}"),
                None => format!("Forget the recorded parent of {branch}"),
            },
            FlowStep::Push {
                remote,
                branch,
                set_upstream,
            } => format!(
                "Push {branch} to {remote}{}",
                if *set_upstream { " and track it" } else { "" }
            ),
            FlowStep::Merge {
                branch,
                into,
                strategy,
                no_ff,
                ..
            } => match strategy {
                MergeStrategy::Merge if *no_ff => {
                    format!("Merge {branch} into {into} with a merge commit")
                }
                MergeStrategy::Merge => format!("Merge {branch} into {into}"),
                MergeStrategy::Squash => format!("Squash {branch} into {into} as one commit"),
                MergeStrategy::Rebase | MergeStrategy::FastForward => {
                    format!("Fast-forward {into} to {branch}")
                }
            },
            FlowStep::Rebase { branch, onto } => format!("Rebase {branch} onto {onto}"),
            FlowStep::Tag { name, at, .. } => format!("Tag {at} as {name}"),
            FlowStep::DeleteBranch { name } => format!("Delete the local branch {name}"),
            FlowStep::DeleteRemoteBranch { remote, name } => {
                format!("Delete {name} on {remote}")
            }
            FlowStep::OpenMergeRequest { source, target, .. } => {
                format!("Open a merge request from {source} into {target} on the host")
            }
            FlowStep::Hook { phase, command } => format!("Run the {phase} hook: {command}"),
        }
    }

    /// The git commands this step amounts to, for a preview that wants to
    /// show exactly what happens.
    pub fn git(&self) -> Vec<String> {
        match self {
            FlowStep::Checkout { branch } => vec![format!("git checkout {branch}")],
            FlowStep::CreateBranch { name, from } => vec![format!("git branch {name} {from}")],
            FlowStep::SetUpstream { branch, upstream } => {
                vec![format!("git branch --set-upstream-to={upstream} {branch}")]
            }
            FlowStep::RecordParent { branch, parent } => match parent {
                Some(p) => vec![format!(
                    "git config branch.{branch}.{} {p}",
                    super::RECORDED_PARENT_VARIABLE
                )],
                None => vec![format!(
                    "git config --unset branch.{branch}.{}",
                    super::RECORDED_PARENT_VARIABLE
                )],
            },
            FlowStep::Push {
                remote,
                branch,
                set_upstream,
            } => vec![format!(
                "git push{} {remote} {branch}",
                if *set_upstream { " -u" } else { "" }
            )],
            FlowStep::Merge {
                branch,
                strategy,
                no_ff,
                message,
                ..
            } => match strategy {
                MergeStrategy::Merge if *no_ff => vec![format!("git merge --no-ff {branch}")],
                MergeStrategy::Merge => vec![format!("git merge {branch}")],
                MergeStrategy::Squash => vec![
                    format!("git merge --squash {branch}"),
                    format!(
                        "git commit -m \"{}\"",
                        message.as_deref().unwrap_or("Squashed commit")
                    ),
                ],
                MergeStrategy::Rebase | MergeStrategy::FastForward => {
                    vec![format!("git merge --ff-only {branch}")]
                }
            },
            FlowStep::Rebase { branch, onto } => {
                vec![
                    format!("git checkout {branch}"),
                    format!("git rebase {onto}"),
                ]
            }
            FlowStep::Tag { name, at, message } => {
                vec![format!("git tag -a {name} {at} -m \"{message}\"")]
            }
            FlowStep::DeleteBranch { name } => vec![format!("git branch -d {name}")],
            FlowStep::DeleteRemoteBranch { remote, name } => {
                vec![format!("git push {remote} --delete {name}")]
            }
            FlowStep::OpenMergeRequest { .. } => Vec::new(),
            FlowStep::Hook { command, .. } => vec![command.clone()],
        }
    }

    /// Whether Undo cannot take the step back. A push, a remote deletion
    /// and a merge request live on the host.
    pub fn irreversible(&self) -> bool {
        matches!(
            self,
            FlowStep::Push { .. }
                | FlowStep::DeleteRemoteBranch { .. }
                | FlowStep::OpenMergeRequest { .. }
                | FlowStep::Hook { .. }
        )
    }
}

/// A merge request the client has to open (the daemon has no hosting
/// credentials): its source branch is already pushed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeRequestSpec {
    pub source: String,
    pub target: String,
    pub title: String,
    pub remote: String,
}

/// A configured hook, listed in a preview whether it will run or not.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookInfo {
    pub phase: String,
    pub command: String,
}

/// What an action would do. Answer of `FlowPlan`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FlowPlan {
    pub action: FlowAction,
    /// One line: "Finish feature/x".
    pub title: String,
    /// The branch the action is about (the new one for a start).
    pub branch: Option<String>,
    /// Every step, in order. Hooks are in here only when `run_hooks` was
    /// asked for.
    pub steps: Vec<FlowStep>,
    /// The hooks the workflow configures for this action, run or not.
    pub hooks: Vec<HookInfo>,
    /// Whether the hooks are among the steps.
    pub hooks_run: bool,
    /// Things worth knowing; the action still runs.
    pub warnings: Vec<String>,
    /// Reasons the action cannot run; a run with blockers is refused.
    pub blockers: Vec<String>,
    /// Whether the working tree has changes (checkouts may need the
    /// auto-stash setting).
    pub dirty: bool,
    /// Whether Undo takes the whole action back (false when a step is
    /// irreversible; the rest is still undone).
    pub undoable: bool,
    /// What Undo will not take back, in words.
    pub irreversible: Vec<String>,
    /// The merge request the client opens at the end, if any.
    pub merge_request: Option<MergeRequestSpec>,
    /// This plan is the rest of a flow a conflict stopped.
    #[serde(default, skip_serializing_if = "is_false")]
    pub resumed: bool,
}

impl FlowPlan {
    pub fn can_run(&self) -> bool {
        self.blockers.is_empty()
    }

    /// Every step as `describe()` lines, numbered, for a terminal.
    pub fn lines(&self) -> Vec<String> {
        self.steps
            .iter()
            .enumerate()
            .map(|(i, s)| format!("{}. {}", i + 1, s.describe()))
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowStatus {
    /// Every step ran.
    Done,
    /// A merge or rebase stopped on conflicts: resolve them, then
    /// `resume: continue` (or `abort`).
    Conflicted,
    /// A step failed (a hook exited non-zero, git refused); the flow
    /// stopped there. What ran before stays.
    Failed,
    /// `resume: abort` ran.
    Aborted,
    /// There was nothing to do.
    NothingToDo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Done,
    /// Nothing to do (already up to date, already checked out).
    Skipped,
    Failed,
    Conflicted,
    /// Left for the client (opening the merge request).
    Pending,
    /// Not reached.
    NotRun,
}

/// One step of a run and how it went.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepReport {
    pub step: FlowStep,
    pub status: StepStatus,
    /// A hook's output (stdout and stderr), a git error, a note.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    /// A hook's exit code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
}

/// Answer of `FlowRun`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlowOutcome {
    pub action: FlowAction,
    pub status: FlowStatus,
    pub branch: Option<String>,
    pub steps: Vec<StepReport>,
    /// The paths in conflict when `status` is `conflicted`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conflicts: Vec<String>,
    /// A line for a human: what happened, or what to do next.
    pub message: String,
    /// The merge request to open (the daemon pushed the branch).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merge_request: Option<MergeRequestSpec>,
    /// A conflict or a failure left a flow that `resume` can go on with.
    #[serde(default, skip_serializing_if = "is_false")]
    pub resumable: bool,
}

/// What the daemon persists to resume a stopped flow
/// (`<git-dir>/gitpoulpi/flow-state.json`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlowState {
    pub request: FlowRequest,
    pub steps: Vec<FlowStep>,
    /// The step the flow stopped at (it is finished first when resumed).
    pub next: usize,
    /// It stopped on a conflict (the step is completed by committing or
    /// continuing); otherwise it failed there and the step is run again.
    #[serde(default)]
    pub conflict: bool,
    /// The branch that was checked out when the flow started.
    pub original_head: Option<String>,
    /// A squash that stopped on conflicts: the commit message to use once
    /// they are resolved.
    #[serde(default)]
    pub squash_message: Option<String>,
    #[serde(default)]
    pub merge_request: Option<MergeRequestSpec>,
}
