//! The daemon adapter's pure parts: the wire request, the preview and the
//! events one outcome stands for.

use gitpoulpi_core::workflow::{
    self as wf, FlowAction, FlowOutcome, FlowPlan, FlowStatus, StepReport, StepStatus,
};

use super::actions::{FlowEvent, FlowOp, FlowRequest, StepKind};
use super::daemon::{outcome_events, preview_of, to_wire};

fn report(step: wf::FlowStep, status: StepStatus) -> StepReport {
    StepReport {
        step,
        status,
        output: None,
        exit_code: None,
    }
}

fn checkout(b: &str) -> wf::FlowStep {
    wf::FlowStep::Checkout { branch: b.into() }
}

fn outcome(status: FlowStatus, steps: Vec<StepReport>) -> FlowOutcome {
    FlowOutcome {
        action: FlowAction::Finish,
        status,
        branch: None,
        steps,
        conflicts: vec!["a.txt".into()],
        message: "stopped".into(),
        merge_request: None,
        resumable: true,
        remote_deleted: Vec::new(),
    }
}

#[test]
fn the_dialogs_requests_become_the_wire_request() {
    let wire = to_wire(&FlowRequest {
        op: FlowOp::Promote {
            from: "develop".into(),
            to: "production".into(),
        },
        run_hooks: true,
    });
    assert_eq!(wire.action, FlowAction::Promote);
    assert_eq!(wire.branch.as_deref(), Some("production"));
    assert!(wire.run_hooks);
    let wire = to_wire(&FlowRequest {
        op: FlowOp::Sync {
            branch: "feature/x".into(),
            with_children: true,
        },
        run_hooks: false,
    });
    assert!(wire.stack && wire.branch.as_deref() == Some("feature/x"));
    let wire = to_wire(&FlowRequest {
        op: FlowOp::Finish {
            branch: "feature/x".into(),
            mode: Some(wf::FinishMode::MergeRequest),
        },
        run_hooks: false,
    });
    assert_eq!(wire.finish_mode, Some(wf::FinishMode::MergeRequest));
}

#[test]
fn a_plan_marks_irreversible_steps_and_hooks() {
    let plan = FlowPlan {
        steps: vec![
            checkout("main"),
            wf::FlowStep::Push {
                remote: "origin".into(),
                branch: "x".into(),
                set_upstream: true,
            },
            wf::FlowStep::Hook {
                phase: "before_finish".into(),
                command: "cargo fmt".into(),
                personal: false,
            },
        ],
        blockers: vec!["dirty".into()],
        warnings: vec!["careful".into()],
        irreversible: vec!["push".into()],
        ..Default::default()
    };
    let p = preview_of(&plan);
    let kinds: Vec<StepKind> = p.steps.iter().map(|s| s.kind).collect();
    assert_eq!(
        kinds,
        [StepKind::Normal, StepKind::Destructive, StepKind::Hook]
    );
    assert_eq!(p.blockers, ["dirty"]);
    assert_eq!(p.warnings, ["careful"]);
    assert_eq!(p.irreversible, ["push"]);
}

#[test]
fn a_done_outcome_replays_every_step_then_hands_over_the_merge_request() {
    let mut out = outcome(
        FlowStatus::Done,
        vec![
            report(checkout("a"), StepStatus::Done),
            report(checkout("b"), StepStatus::Skipped),
        ],
    );
    out.merge_request = Some(wf::MergeRequestSpec {
        source: "feature/x".into(),
        target: "main".into(),
        title: "X".into(),
        remote: "origin".into(),
    });
    let events = outcome_events(&out, 0);
    assert!(matches!(&events[0], FlowEvent::Steps(s) if s.len() == 2));
    assert_eq!(events[1], FlowEvent::StepStarted(0));
    assert_eq!(events[4], FlowEvent::StepDone(1));
    assert!(matches!(events[5], FlowEvent::MergeRequest(_)));
    assert_eq!(events[6], FlowEvent::Finished);
}

#[test]
fn a_conflict_stops_at_its_step_and_a_continuation_is_offset() {
    let out = outcome(
        FlowStatus::Conflicted,
        vec![
            report(checkout("a"), StepStatus::Done),
            report(checkout("b"), StepStatus::Conflicted),
            report(checkout("c"), StepStatus::NotRun),
        ],
    );
    let events = outcome_events(&out, 0);
    assert_eq!(
        events.last(),
        Some(&FlowEvent::StepFailed {
            index: 1,
            message: "stopped (a.txt)".into(),
            conflict: true,
        })
    );
    // The continuation reports from the step it stopped at: no `Steps`,
    // indices shifted.
    let out = outcome(
        FlowStatus::Done,
        vec![report(checkout("b"), StepStatus::Done)],
    );
    let events = outcome_events(&out, 1);
    assert_eq!(events[0], FlowEvent::StepStarted(1));
    assert_eq!(events.last(), Some(&FlowEvent::Finished));
}

#[test]
fn a_failed_hook_says_its_last_output_line() {
    let mut failed = report(
        wf::FlowStep::Hook {
            phase: "before_finish".into(),
            command: "make".into(),
            personal: false,
        },
        StepStatus::Failed,
    );
    failed.output = Some("compiling\nerror: no rule\n".into());
    let events = outcome_events(&outcome(FlowStatus::Failed, vec![failed]), 0);
    assert!(matches!(
        events.last(),
        Some(FlowEvent::StepFailed { message, conflict: false, .. })
            if message == "stopped: error: no rule"
    ));
}
