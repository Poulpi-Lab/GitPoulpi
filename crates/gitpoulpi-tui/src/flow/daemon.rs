//! [`FlowActions`] over the daemon's `FlowPlan` (a read: the preview) and
//! `FlowRun` (the run, journaled as ONE undo entry) requests.
//!
//! Nothing here runs on the render thread: a request goes out through the
//! window's [`Requester`] and a small bridge thread turns its slot into the
//! channel the dialog polls every frame (and pokes the redraw when an answer
//! lands). `FlowRun` answers once, when the whole run ended, so the per-step
//! progress is replayed from the outcome's step reports; the first step shows
//! as running (with the 150 ms spinner) until then.
//!
//! A run a conflict stopped is remembered process-wide ([`STOPPED`]): the
//! daemon keeps the flow's state, and the client needs the request that
//! started it (`resume` must name the same action) and the step list to show.
//! That is what lets the dialog be closed, the conflicts resolved in the
//! existing tools, and `Flow: Continue` reopen it.

use std::sync::mpsc::{channel, Sender};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use gitpoulpi_core::domain::OperationKind;
use gitpoulpi_core::workflow::{
    self as wf, FlowAction, FlowOutcome, FlowPlan, FlowResume, FlowStatus, StepStatus,
};

use super::actions::{
    EventRx, FlowActions, FlowEvent, FlowOp, FlowPreview, FlowRequest, FlowStep, PreviewRx, Resume,
    StepKind, StoppedFlow, TrustRx,
};
use crate::daemon_link::error_text::error_text;
use crate::daemon_link::{Reply, Requester};

/// A reply that has not come after this long is given up on (the bridge
/// thread must not live forever when the link died silently).
const GIVE_UP: Duration = Duration::from_secs(30 * 60);

#[derive(Clone)]
struct Stopped {
    request: wf::FlowRequest,
    steps: Vec<FlowStep>,
    failed: usize,
    message: String,
    title: String,
}

/// The run a conflict stopped (one per process: the window has one repo
/// at a time doing a flow, and the daemon refuses a second one).
static STOPPED: Mutex<Option<Stopped>> = Mutex::new(None);

fn stopped_slot() -> std::sync::MutexGuard<'static, Option<Stopped>> {
    STOPPED.lock().unwrap_or_else(|e| e.into_inner())
}

/// Whether an action is waiting to be continued or aborted.
pub(crate) fn has_stopped() -> bool {
    stopped_slot().is_some()
}

pub(crate) struct DaemonFlowActions {
    requester: Requester,
}

impl DaemonFlowActions {
    pub(crate) fn new(requester: Requester) -> Self {
        Self { requester }
    }
}

/// The wire request for what the dialog built.
pub(crate) fn to_wire(request: &FlowRequest) -> wf::FlowRequest {
    let mut wire = wf::FlowRequest {
        run_hooks: request.run_hooks,
        ..Default::default()
    };
    match &request.op {
        FlowOp::Start {
            type_id,
            fields,
            start_point,
            publish,
            ..
        } => {
            wire.action = FlowAction::Start;
            wire.topic = Some(type_id.clone());
            wire.fields = fields.clone();
            wire.from = Some(start_point.clone());
            wire.publish = *publish;
        }
        FlowOp::Finish { branch, mode } => {
            wire.action = FlowAction::Finish;
            wire.branch = Some(branch.clone());
            wire.finish_mode = *mode;
        }
        FlowOp::Sync {
            branch,
            with_children,
        } => {
            wire.action = FlowAction::Sync;
            wire.branch = Some(branch.clone());
            wire.stack = *with_children;
        }
        // The plan names the environment branch to update; its parent is
        // the one the workflow gives it.
        FlowOp::Promote { to, .. } => {
            wire.action = FlowAction::Promote;
            wire.branch = Some(to.clone());
        }
        FlowOp::SetParent { branch, parent } => {
            wire.action = FlowAction::SetParent;
            wire.branch = Some(branch.clone());
            wire.parent = Some(parent.clone());
        }
    }
    wire
}

fn args(wire: &wf::FlowRequest) -> serde_json::Value {
    serde_json::to_value(wire).unwrap_or_default()
}

pub(crate) fn step_of(step: &wf::FlowStep) -> FlowStep {
    let kind = match step {
        wf::FlowStep::Hook { .. } => StepKind::Hook,
        s if s.irreversible() => StepKind::Destructive,
        _ => StepKind::Normal,
    };
    FlowStep::new(step.describe(), kind)
}

pub(crate) fn preview_of(plan: &FlowPlan) -> FlowPreview {
    FlowPreview {
        steps: plan.steps.iter().map(step_of).collect(),
        hooks: plan.hooks.iter().map(|h| h.command.clone()).collect(),
        blockers: plan.blockers.clone(),
        warnings: plan.warnings.clone(),
        irreversible: plan.irreversible.clone(),
        trust: plan.hooks_trust.clone(),
        hook_lines: plan
            .hooks
            .iter()
            .map(|h| (h.phase.clone(), h.command.clone()))
            .collect(),
    }
}

/// Turns a reply slot into a channel message, off the render thread.
fn bridge<T, U>(
    reply: Reply<T>,
    tx: Sender<U>,
    convert: impl FnOnce(gitpoulpi_core::Result<T>) -> Vec<U> + Send + 'static,
) where
    T: Send + 'static,
    U: Send + 'static,
{
    std::thread::spawn(move || {
        let started = Instant::now();
        let answer = loop {
            if let Some(answer) = reply.take() {
                break Some(answer);
            }
            if started.elapsed() > GIVE_UP {
                break None;
            }
            std::thread::sleep(Duration::from_millis(5));
        };
        if let Some(answer) = answer {
            for message in convert(answer) {
                if tx.send(message).is_err() {
                    break;
                }
            }
        }
        // A dropped sender without a message reads as "lost" in the dialog.
        crate::redraw::poke();
    });
}

/// The events one outcome stands for. `offset` is the index of the first
/// report in the dialog's step list (a continuation reports from the step
/// it stopped at).
pub(crate) fn outcome_events(outcome: &FlowOutcome, offset: usize) -> Vec<FlowEvent> {
    let mut events = Vec::new();
    if offset == 0 {
        events.push(FlowEvent::Steps(
            outcome.steps.iter().map(|r| step_of(&r.step)).collect(),
        ));
    }
    for (i, report) in outcome.steps.iter().enumerate() {
        let index = offset + i;
        match report.status {
            StepStatus::Done | StepStatus::Skipped | StepStatus::Pending => {
                events.push(FlowEvent::StepStarted(index));
                events.push(FlowEvent::StepDone(index));
            }
            StepStatus::Failed => {
                events.push(FlowEvent::StepFailed {
                    index,
                    message: failure_text(outcome, report.output.as_deref()),
                    conflict: false,
                });
                return events;
            }
            StepStatus::Conflicted => {
                events.push(FlowEvent::StepFailed {
                    index,
                    message: conflict_text(outcome),
                    conflict: true,
                });
                return events;
            }
            StepStatus::NotRun => break,
        }
    }
    match outcome.status {
        FlowStatus::Done | FlowStatus::NothingToDo | FlowStatus::Aborted => {
            if let Some(spec) = &outcome.merge_request {
                events.push(FlowEvent::MergeRequest(spec.clone()));
            }
            events.push(FlowEvent::Finished);
        }
        FlowStatus::Conflicted | FlowStatus::Failed => events.push(FlowEvent::StepFailed {
            index: offset,
            message: outcome.message.clone(),
            conflict: outcome.status == FlowStatus::Conflicted,
        }),
    }
    events
}

fn failure_text(outcome: &FlowOutcome, output: Option<&str>) -> String {
    // A hook's tail says more than "exit 1".
    match output.map(str::trim).filter(|o| !o.is_empty()) {
        Some(out) => {
            let last = out.lines().last().unwrap_or(out);
            format!("{}: {last}", outcome.message)
        }
        None => outcome.message.clone(),
    }
}

fn conflict_text(outcome: &FlowOutcome) -> String {
    if outcome.conflicts.is_empty() {
        outcome.message.clone()
    } else {
        format!("{} ({})", outcome.message, outcome.conflicts.join(", "))
    }
}

/// Remembers (or forgets) the stopped run after `outcome`.
fn remember(wire: &wf::FlowRequest, outcome: &FlowOutcome, events: &[FlowEvent]) {
    let failed = events.iter().find_map(|e| match e {
        FlowEvent::StepFailed {
            index,
            message,
            conflict: true,
        } => Some((*index, message.clone())),
        _ => None,
    });
    let mut slot = stopped_slot();
    match failed {
        Some((failed, message)) if outcome.status == FlowStatus::Conflicted => {
            let steps = events
                .iter()
                .find_map(|e| match e {
                    FlowEvent::Steps(steps) => Some(steps.clone()),
                    _ => None,
                })
                .or_else(|| slot.as_ref().map(|s| s.steps.clone()))
                .unwrap_or_default();
            let mut request = wire.clone();
            request.resume = None;
            *slot = Some(Stopped {
                request,
                steps,
                failed,
                message,
                title: wire.action.word().to_string(),
            });
        }
        _ => *slot = None,
    }
}

fn run_events(requester: &Requester, wire: wf::FlowRequest, offset: usize, tx: Sender<FlowEvent>) {
    let reply = requester.call::<FlowOutcome>(OperationKind::FlowRun, args(&wire));
    bridge(reply, tx, move |answer| match answer {
        Ok(outcome) => {
            let events = outcome_events(&outcome, offset);
            remember(&wire, &outcome, &events);
            events
        }
        Err(e) => {
            // A refused continuation (conflicts not resolved yet) leaves
            // the flow stopped: the dialog keeps Continue and Abort.
            let conflict = offset > 0 || wire.resume == Some(FlowResume::Continue);
            vec![FlowEvent::StepFailed {
                index: offset,
                message: error_text(&e),
                conflict: conflict && has_stopped(),
            }]
        }
    });
}

impl FlowActions for DaemonFlowActions {
    fn preview(&self, request: &FlowRequest) -> PreviewRx {
        let (tx, rx) = channel();
        let reply = self
            .requester
            .call::<FlowPlan>(OperationKind::FlowPlan, args(&to_wire(request)));
        bridge(reply, tx, |answer| {
            vec![match answer {
                Ok(plan) => Ok(preview_of(&plan)),
                Err(e) => Err(error_text(&e)),
            }]
        });
        rx
    }

    fn execute(&self, request: &FlowRequest) -> EventRx {
        let (tx, rx) = channel();
        // The first step is running until the daemon answers.
        let _ = tx.send(FlowEvent::StepStarted(0));
        crate::redraw::poke();
        *stopped_slot() = None;
        run_events(&self.requester, to_wire(request), 0, tx);
        rx
    }

    fn resume(&self, how: Resume) -> EventRx {
        let (tx, rx) = channel();
        let Some(stopped) = stopped_slot().clone() else {
            let _ = tx.send(FlowEvent::Finished);
            return rx;
        };
        let mut wire = stopped.request.clone();
        wire.resume = Some(match how {
            Resume::Continue => FlowResume::Continue,
            Resume::Abort => FlowResume::Abort,
        });
        let offset = match how {
            Resume::Continue => stopped.failed,
            Resume::Abort => 0,
        };
        if how == Resume::Continue {
            let _ = tx.send(FlowEvent::StepStarted(offset));
            crate::redraw::poke();
        }
        let requester = self.requester.clone();
        if how == Resume::Abort {
            // An abort answers with no steps: just the end of the story.
            let reply = requester.call::<FlowOutcome>(OperationKind::FlowRun, args(&wire));
            bridge(reply, tx, move |answer| {
                match answer {
                    Ok(_) => *stopped_slot() = None,
                    Err(e) => {
                        return vec![FlowEvent::StepFailed {
                            index: stopped.failed,
                            message: error_text(&e),
                            conflict: true,
                        }]
                    }
                }
                vec![FlowEvent::Finished]
            });
        } else {
            run_events(&requester, wire, offset, tx);
        }
        rx
    }

    fn trust_hooks(&self, hash: &str) -> TrustRx {
        let (tx, rx) = channel();
        let args = serde_json::to_value(wf::TrustHooksArgs {
            hash: Some(hash.to_string()),
        })
        .unwrap_or_default();
        let reply = self
            .requester
            .call::<wf::HooksReport>(OperationKind::FlowTrustHooks, args);
        bridge(reply, tx, |answer| {
            vec![match answer {
                Ok(_) => Ok(()),
                Err(e) => Err(error_text(&e)),
            }]
        });
        rx
    }

    fn stopped(&self) -> Option<StoppedFlow> {
        stopped_slot().as_ref().map(|s| StoppedFlow {
            title: s.title.clone(),
            steps: s.steps.clone(),
            failed: s.failed,
            message: s.message.clone(),
        })
    }
}
