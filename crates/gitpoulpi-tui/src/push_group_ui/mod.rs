//! The grouped push dialog: "Push <branch> to several remotes".
//!
//! Three steps in one movable frame:
//!
//! 1. **Selection** (only for a one-off "Push to selected remotes..."): every
//!    remote of the repository with a checkbox, pre-ticked with the branch's
//!    upstream and push targets. "Remember for this branch" writes the ticked
//!    remotes (`BranchSetRemotes`); nothing is written otherwise.
//! 2. **Plan**: one row per target with where the branch stands (`up n down n`,
//!    diverged, rewritten, ...), an Action combobox of the allowed actions
//!    (default: the daemon's suggestion), the conflict prediction of a merge,
//!    the "Then:" summary and the buttons. A force push goes through the
//!    ordinary destructive confirmation (`PendingDestructive::GroupForcePush`)
//!    naming the remotes and the commits it replaces.
//! 3. **Progress**: the same dialog fed by `Event::RemoteGroupProgress` (and
//!    the run's outcome). Closing it never stops the run (the state lives
//!    outside the dialog; `PushGroupUi::open_progress` brings it back).
//!
//! A run that stopped on conflicts closes the dialog and leaves a strip on the
//! Log's bottom border (`strip`): Continue appears once the daemon says the
//! resolution is concluded (`PendingGroup::ready_to_continue`), Abort group
//! throws the merge or rebase away. **Nothing continues on its own.**
//!
//! Process-wide state, like the other small modals: the window restarts on
//! every tab switch and the run's answers come from the link's own thread.

mod input;
mod model;
mod render;
mod strip;
#[cfg(test)]
pub(crate) mod tests;

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crossterm::event::{KeyEvent, MouseEvent};
use gitpoulpi_core::domain::{
    GroupMode, GroupRunStatus, OperationKind, RemoteGroupOutcome, RemoteGroupPlanRequest,
    RemoteGroupRunRequest, RemoteGroupStatus, TargetRef, TargetStatus,
};
use ratatui::layout::Rect;
use ratatui::Frame;

use crate::daemon_link::{Action, DaemonLink, Requester};
use crate::dialog_kit;
use crate::modes::{ConfirmAction, InputMode, PendingDestructive};

use model::{Part, Phase, PlanView, RunRow, RunState, Select};

pub(crate) use strip::{
    click as strip_click, render as render_strip, visible as strip_visible, StripButton,
};

const WIN: dialog_kit::Win = dialog_kit::Win::movable("push_group");

/// How often a paused group's readiness is asked again.
const POLL_EVERY: Duration = Duration::from_millis(1000);

// One dialog at a time, never in a collection: the size gap is harmless.
#[allow(clippy::large_enum_variant)]
pub(super) enum Step {
    Select(Select),
    Plan(PlanView),
    /// Reads `Core::run`.
    Run,
}

pub(super) struct Dlg {
    pub root: PathBuf,
    pub branch: String,
    pub step: Step,
    pub hover: Option<Part>,
    pub close: dialog_kit::CloseHit,
    pub frame: Rect,
    /// Inner-first hit rects of the last paint.
    pub hits: Vec<(Rect, Part)>,
    /// The button a Tab landed on.
    pub btn_focus: Option<usize>,
}

/// A grouped run waiting on conflicts.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Paused {
    pub root: PathBuf,
    pub branch: String,
    pub run_id: String,
    pub remote: String,
    pub ready: bool,
    pub prefer_rebase: bool,
}

#[derive(Default)]
pub(super) struct Core {
    pub dlg: Option<Dlg>,
    pub run: Option<RunState>,
    pub paused: Option<Paused>,
    pub requester: Option<Requester>,
    status: Option<crate::daemon_link::Reply<RemoteGroupStatus>>,
    last_poll: Option<Instant>,
    /// The (repository, branch) whose saved run was looked up already.
    probed: Option<(PathBuf, String)>,
}

static CORE: Mutex<Option<Core>> = Mutex::new(None);

fn with<R>(f: impl FnOnce(&mut Core) -> R) -> R {
    let mut slot = CORE.lock().unwrap_or_else(|e| e.into_inner());
    f(slot.get_or_insert_with(Core::default))
}

/// What a key or click asks the window to do (the dialog has no handle on
/// the link or on the confirmation modal).
pub(crate) enum Effect {
    None,
    Submit(Vec<Action>),
    /// The force confirmation: the ordinary destructive modal.
    Confirm {
        prompt: String,
        pending: PendingDestructive,
    },
}

/// `git.pull_mode` is rebase: the plan's default and the engine's pull-side
/// fallback follow it.
fn prefer_rebase() -> bool {
    crate::remotes_ui::sync_settings().0 == gitpoulpi_config::PullButtonMode::Rebase
}

/// The entry points for the menus.
pub(crate) struct PushGroupUi;

impl PushGroupUi {
    /// The one-off "Push to selected remotes..." dialog: the selection step,
    /// then the plan.
    pub(crate) fn open_selection(root: &Path, branch: &str) {
        with(|c| {
            let Some(requester) = c.requester.clone() else {
                return;
            };
            let select = Select::new(
                requester.call(OperationKind::Remotes, serde_json::json!({})),
                requester.call(
                    OperationKind::RemoteGroupStatus,
                    serde_json::json!({ "branch": branch }),
                ),
            );
            c.open(root, branch, Step::Select(select));
        });
    }

    /// The plan step for `branch` (`None` targets: the configured primary
    /// plus push targets).
    // Entered by the Push / Pull menus (a later slice); tested here.
    #[allow(dead_code)]
    pub(crate) fn open_plan(root: &Path, branch: &str, targets: Option<Vec<TargetRef>>) {
        with(|c| {
            let Some(requester) = c.requester.clone() else {
                return;
            };
            let plan = c.ask_plan(&requester, branch, targets.clone());
            c.open(root, branch, Step::Plan(plan));
        });
    }

    /// Reopens the progress view of the last run (closed or still going).
    /// `false` when there is none.
    // Entered by the jobs indicator and a palette command (a later slice).
    #[allow(dead_code)]
    pub(crate) fn open_progress() -> bool {
        with(|c| c.open_progress())
    }
}

impl Core {
    fn open(&mut self, root: &Path, branch: &str, step: Step) {
        self.dlg = Some(Dlg {
            root: root.to_path_buf(),
            branch: branch.to_string(),
            step,
            hover: None,
            close: Default::default(),
            frame: Rect::default(),
            hits: Vec::new(),
            btn_focus: None,
        });
        crate::redraw::poke();
    }

    fn open_progress(&mut self) -> bool {
        let Some(run) = &self.run else {
            return false;
        };
        let branch = run.branch.clone();
        let root = self
            .paused
            .as_ref()
            .map(|p| p.root.clone())
            .or_else(|| self.dlg.as_ref().map(|d| d.root.clone()))
            .unwrap_or_default();
        self.open(&root, &branch, Step::Run);
        true
    }

    /// Asks the daemon for the plan.
    fn ask_plan(
        &self,
        requester: &Requester,
        branch: &str,
        targets: Option<Vec<TargetRef>>,
    ) -> PlanView {
        let prefer = prefer_rebase();
        let request = RemoteGroupPlanRequest {
            branch: branch.to_string(),
            mode: GroupMode::Push,
            targets: targets.clone(),
            preflight: true,
            prefer_rebase: prefer,
            refused: Vec::new(),
        };
        let reply = requester.call(
            OperationKind::RemoteGroupPlan,
            serde_json::to_value(&request).unwrap_or_default(),
        );
        PlanView::loading(reply, targets, prefer)
    }

    /// Switches to the progress view of a freshly built run.
    fn begin_run(&mut self, request: &RemoteGroupRunRequest) {
        self.run = Some(RunState::from_request(request));
        if let Some(d) = self.dlg.as_mut() {
            d.step = Step::Run;
            d.btn_focus = None;
        }
        crate::redraw::poke();
    }

    /// Polls what the open step waits for.
    fn poll(&mut self) {
        let Some(dlg) = self.dlg.as_mut() else {
            return;
        };
        let branch = dlg.branch.clone();
        match &mut dlg.step {
            Step::Select(s) => {
                s.poll(&branch);
            }
            Step::Plan(p) => {
                p.poll();
            }
            Step::Run => {}
        }
    }
}

// ---------------------------------------------------------------------
// Fed by the link
// ---------------------------------------------------------------------

/// The link's requester, set once at connect (and by tests).
pub(crate) fn set_requester(requester: Requester) {
    with(|c| c.requester = Some(requester));
}

/// A grouped action begins on the link thread. Idempotent for the request
/// the dialog already started itself; the one place a confirmed force push
/// (which went through the destructive modal) flips the dialog to progress.
pub(crate) fn on_started(root: &Path, request: &RemoteGroupRunRequest) {
    with(|c| {
        let fresh = request.resume.is_none() && !request.retry;
        if fresh {
            if c.run
                .as_ref()
                .is_none_or(|r| &r.request != request || !r.going())
            {
                c.begin_run(request);
            }
            return;
        }
        // Continue / retry / abort of the saved run: its rows stay.
        let Some(run) = c.run.as_mut() else {
            return;
        };
        if request.resume == Some(gitpoulpi_core::domain::GroupResume::Abort) {
            return;
        }
        run.phase = Phase::Going;
        if request.retry {
            for row in run.rows.iter_mut().filter(|r| {
                matches!(
                    r.status,
                    Some(
                        TargetStatus::Failed { .. }
                            | TargetStatus::Rejected { .. }
                            | TargetStatus::Unknown
                            | TargetStatus::NotAttempted
                    )
                )
            }) {
                row.status = None;
                row.detail = None;
            }
        }
        // A continue brings the progress view back.
        let _ = root;
        c.paused = None;
        c.open_progress();
    });
}

/// One target moved (`Event::RemoteGroupProgress`).
pub(crate) fn on_progress(run_id: &str, remote: &str, status: TargetStatus) {
    with(|c| {
        let Some(run) = c.run.as_mut().filter(|r| r.going()) else {
            return;
        };
        match &run.run_id {
            None => run.run_id = Some(run_id.to_string()),
            Some(id) if id != run_id => return,
            Some(_) => {}
        }
        if let Some(row) = run.rows.iter_mut().find(|r| r.remote == remote) {
            row.status = Some(status);
        }
        crate::redraw::poke();
    });
}

/// The run request itself failed (not a target: the whole call).
pub(crate) fn on_failed(_root: &Path, why: &str) {
    with(|c| {
        if let Some(run) = c.run.as_mut() {
            run.phase = Phase::Failed(why.to_string());
        }
        crate::redraw::poke();
    });
}

/// The run's answer: final rows, and where it stopped.
pub(crate) fn on_outcome(root: &Path, outcome: &RemoteGroupOutcome) {
    with(|c| {
        if let Some(run) = c.run.as_mut() {
            run.run_id = Some(outcome.run_id.clone());
            run.message = outcome.message.clone();
            for t in &outcome.targets {
                let row = match run
                    .rows
                    .iter_mut()
                    .find(|r| r.remote == t.remote && r.branch == t.branch)
                {
                    Some(row) => row,
                    None => {
                        run.rows.push(RunRow {
                            remote: t.remote.clone(),
                            branch: t.branch.clone(),
                            status: None,
                            detail: None,
                            integrated: None,
                        });
                        run.rows.last_mut().expect("just pushed")
                    }
                };
                row.status = Some(t.status.clone());
                row.detail = t.detail.clone();
                row.integrated = t.integrated;
            }
            run.phase = Phase::Ended(outcome.status);
        }
        match outcome.status {
            GroupRunStatus::Conflicted => {
                let remote = outcome
                    .stopped
                    .as_ref()
                    .map(|s| s.remote.clone())
                    .unwrap_or_default();
                c.paused = Some(Paused {
                    root: root.to_path_buf(),
                    branch: outcome.branch.clone(),
                    run_id: outcome.run_id.clone(),
                    remote,
                    ready: false,
                    prefer_rebase: prefer_rebase(),
                });
                // The ordinary merge / rebase banner and conflict pages take
                // over; the strip carries the group.
                c.dlg = None;
            }
            GroupRunStatus::NeedsResolution => {
                c.paused = None;
                if let Some(plan) = outcome.plan.clone() {
                    let view = PlanView::from_plan(plan, prefer_rebase());
                    let branch = outcome.branch.clone();
                    if let Some(d) = c.dlg.as_mut() {
                        d.step = Step::Plan(view);
                    } else {
                        c.open(root, &branch, Step::Plan(view));
                    }
                }
            }
            GroupRunStatus::Aborted => {
                c.paused = None;
                c.dlg = None;
                c.run = None;
            }
            GroupRunStatus::Done | GroupRunStatus::Problems | GroupRunStatus::Canceled => {
                c.paused = None;
            }
        }
        crate::redraw::poke();
    });
}

// ---------------------------------------------------------------------
// The window's hooks
// ---------------------------------------------------------------------

pub(crate) fn is_open() -> bool {
    CORE.lock()
        .ok()
        .is_some_and(|c| c.as_ref().is_some_and(|c| c.dlg.is_some()))
}

/// A key while the dialog is open (it takes them all).
pub(crate) fn on_key(key: KeyEvent) -> Effect {
    with(|c| c.on_key(key))
}

/// A mouse event while the dialog is open (it takes them all).
pub(crate) fn on_mouse(mouse: MouseEvent) -> Effect {
    with(|c| c.on_mouse(mouse))
}

/// Paints the dialog, last, over the panes.
pub(crate) fn render(frame: &mut Frame) {
    with(|c| {
        c.poll();
        c.render(frame);
    });
}

/// Turns what a key or click asked into the window's doing.
pub(crate) fn apply(effect: Effect, link: &DaemonLink, input_mode: &mut InputMode) {
    match effect {
        Effect::None => {}
        Effect::Submit(actions) => {
            for action in actions {
                link.submit_action(action);
            }
        }
        Effect::Confirm { prompt, pending } => {
            *input_mode = InputMode::Confirm {
                prompt,
                action: ConfirmAction::Destructive(pending),
            };
        }
    }
}

/// Per-frame upkeep of the paused strip: asks (never blocking) whether the
/// saved run can continue now, and finds one left by an earlier session.
pub(crate) fn tick(root: &Path, head_branch: Option<&str>) {
    with(|c| {
        if let Some(answer) = c.status.as_ref().and_then(crate::daemon_link::Reply::take) {
            c.status = None;
            if let Ok(status) = answer {
                match status.pending {
                    Some(p) if p.stopped.is_some() => {
                        let paused = Paused {
                            root: root.to_path_buf(),
                            branch: p.branch.clone(),
                            run_id: p.run_id.clone(),
                            remote: p
                                .stopped
                                .as_ref()
                                .map(|s| s.remote.clone())
                                .unwrap_or_default(),
                            ready: p.ready_to_continue,
                            prefer_rebase: prefer_rebase(),
                        };
                        if c.paused.as_ref() != Some(&paused) {
                            c.paused = Some(paused);
                            crate::redraw::poke();
                        }
                    }
                    _ => {
                        if c.paused.take().is_some() {
                            crate::redraw::poke();
                        }
                    }
                }
            }
        }
        let Some(requester) = c.requester.clone() else {
            return;
        };
        let branch = match (&c.paused, head_branch) {
            (Some(p), _) if p.root == root => Some(p.branch.clone()),
            (None, Some(head)) => {
                // Once per repository and branch: a saved run left by a
                // restart (or another frontend).
                let key = (root.to_path_buf(), head.to_string());
                if c.probed.as_ref() == Some(&key) {
                    None
                } else {
                    c.probed = Some(key);
                    Some(head.to_string())
                }
            }
            _ => None,
        };
        let due = c.paused.is_none() || c.last_poll.is_none_or(|t| t.elapsed() >= POLL_EVERY);
        if let (Some(branch), true, true) = (branch, due, c.status.is_none()) {
            c.last_poll = Some(Instant::now());
            c.status = Some(requester.call(
                OperationKind::RemoteGroupStatus,
                serde_json::json!({ "branch": branch }),
            ));
        }
    });
}

/// The dialog is process-wide: tests that open it take turns.
#[cfg(test)]
pub(crate) static TEST_SERIAL: Mutex<()> = Mutex::new(());

#[cfg(test)]
pub(crate) fn reset_for_tests() {
    *CORE.lock().unwrap_or_else(|e| e.into_inner()) = None;
}
