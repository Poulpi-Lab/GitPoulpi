//! The seam between the Flow dialogs and the actions that change the
//! repository (slice W2: start / finish / sync / promote / set parent in the
//! backend, the daemon and the CLI).
//!
//! The dialogs only know [`FlowActions`]: ask for a *preview* (the step list
//! an action would run), then *execute* it and watch its progress. Both
//! return a channel the dialog polls every frame, so nothing here ever
//! blocks the render thread. The process uses [`super::daemon`]'s adapter
//! over the daemon's `FlowPlan` / `FlowRun` requests; unit tests use
//! [`FakeFlowActions`], which records what it was asked and plays back
//! canned steps (`GITPOULPI_FLOW_FAKE=<file>` selects it in the real binary
//! and appends one JSON line per call to that file).

use std::collections::BTreeMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

/// What an action does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FlowOp {
    /// Create (and check out) a topic branch of `type_id`.
    Start {
        type_id: String,
        /// The pattern's token values the form collected.
        fields: BTreeMap<String, String>,
        /// The branch name the pattern renders.
        name: String,
        /// Where it starts: a branch or ref name.
        start_point: String,
        publish: bool,
    },
    /// Merge `branch` into its finish targets, tag, delete, return, or push
    /// it and open a merge request (`mode`, when the type says `ask` or the
    /// user chose).
    Finish {
        branch: String,
        mode: Option<gitpoulpi_core::workflow::FinishMode>,
    },
    /// Update `branch` from its parent (children on request).
    Sync { branch: String, with_children: bool },
    /// GitLab Flow: merge `from` into the environment branch `to`.
    Promote { from: String, to: String },
    /// Record another parent for `branch`.
    SetParent { branch: String, parent: String },
}

impl FlowOp {
    /// The lower-case verb the CLI/daemon would call it.
    pub(crate) fn verb(&self) -> &'static str {
        match self {
            FlowOp::Start { .. } => "start",
            FlowOp::Finish { .. } => "finish",
            FlowOp::Sync { .. } => "sync",
            FlowOp::Promote { .. } => "promote",
            FlowOp::SetParent { .. } => "set-parent",
        }
    }
}

/// An action plus whether its hooks may run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FlowRequest {
    pub(crate) op: FlowOp,
    /// The user ticked "run hooks": without it hooks are skipped.
    pub(crate) run_hooks: bool,
}

/// How a step is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StepKind {
    Normal,
    /// A hook (a shell command from the workflow file).
    Hook,
    /// Cannot be taken back (a remote branch delete, a push, a merge
    /// request): built by the daemon adapter from `FlowStep::irreversible`.
    Destructive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FlowStep {
    pub(crate) text: String,
    pub(crate) kind: StepKind,
}

impl FlowStep {
    pub(crate) fn new(text: impl Into<String>, kind: StepKind) -> Self {
        Self {
            text: text.into(),
            kind,
        }
    }
}

/// What an action would do, before it does it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct FlowPreview {
    pub(crate) steps: Vec<FlowStep>,
    /// The shell commands that would run (a subset of `steps` of kind
    /// `Hook`, repeated so the dialog can list them under a warning).
    pub(crate) hooks: Vec<String>,
    /// Why the action cannot run at all (dirty tree, target behind, ...).
    pub(crate) blockers: Vec<String>,
    /// Things worth knowing; the action still runs.
    pub(crate) warnings: Vec<String>,
    /// What Undo will not take back, in words (highlighted).
    pub(crate) irreversible: Vec<String>,
}

/// Progress of a running action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FlowEvent {
    StepStarted(usize),
    StepDone(usize),
    /// A step stopped. With `conflict` the repository is mid-merge and the
    /// user may Continue (after resolving) or Abort.
    StepFailed {
        index: usize,
        message: String,
        conflict: bool,
    },
    /// The steps the daemon really planned (the dialog showed a guess made
    /// from the preview): replaces the list the following events index.
    Steps(Vec<FlowStep>),
    /// The daemon pushed the branch; the client opens this merge request.
    MergeRequest(gitpoulpi_core::workflow::MergeRequestSpec),
    Finished,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Resume {
    Continue,
    Abort,
}

/// The answer of a preview request.
pub(crate) type PreviewRx = Receiver<Result<FlowPreview, String>>;
/// The event stream of a running action.
pub(crate) type EventRx = Receiver<FlowEvent>;

/// What the Flow dialogs call. Implemented over W2's daemon requests; the
/// fake below stands in for tests.
pub(crate) trait FlowActions: Send + Sync {
    /// The steps `request` would run, without running them.
    fn preview(&self, request: &FlowRequest) -> PreviewRx;
    /// Runs `request`. Events arrive on the channel; `Finished` is last.
    fn execute(&self, request: &FlowRequest) -> EventRx;
    /// Continues or aborts an action stopped on a conflict.
    fn resume(&self, how: Resume) -> EventRx;
    /// The action a conflict stopped, if any (the Flow dialog can be
    /// reopened on it after the conflicts were resolved elsewhere).
    fn stopped(&self) -> Option<StoppedFlow> {
        None
    }
}

/// An action stopped on a conflict, as the dialog shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StoppedFlow {
    pub(crate) title: String,
    pub(crate) steps: Vec<FlowStep>,
    pub(crate) failed: usize,
    pub(crate) message: String,
}

/// One call the fake saw.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Call {
    Preview(FlowRequest),
    Execute(FlowRequest),
    Resume(Resume),
}

/// A scripted [`FlowActions`]: records every call, answers previews with
/// `preview`, and plays an execute as one event pair per previewed step
/// (with an optional delay so in-flight feedback can be seen).
pub(crate) struct FakeFlowActions {
    pub(crate) calls: Mutex<Vec<Call>>,
    pub(crate) preview: Mutex<Result<FlowPreview, String>>,
    /// Delay before each answer or event (milliseconds), 0 = at once.
    pub(crate) delay_ms: u64,
    /// The step that fails (`index`, message, conflict), if any.
    pub(crate) fail_at: Mutex<Option<(usize, String, bool)>>,
    /// Appended one JSON line per call (the PTY tests read it).
    pub(crate) log: Option<std::path::PathBuf>,
}

impl FakeFlowActions {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn new(preview: FlowPreview) -> Arc<Self> {
        Arc::new(Self {
            calls: Mutex::new(Vec::new()),
            preview: Mutex::new(Ok(preview)),
            delay_ms: 0,
            fail_at: Mutex::new(None),
            log: None,
        })
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn calls(&self) -> Vec<Call> {
        self.calls.lock().map(|c| c.clone()).unwrap_or_default()
    }

    fn record(&self, call: Call) {
        if let Some(path) = &self.log {
            use std::io::Write;
            let line = match &call {
                Call::Preview(r) => request_json("preview", r),
                Call::Execute(r) => request_json("execute", r),
                Call::Resume(how) => serde_json::json!({
                    "call": "resume",
                    "how": if *how == Resume::Continue { "continue" } else { "abort" },
                }),
            };
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
            {
                let _ = writeln!(f, "{line}");
            }
        }
        if let Ok(mut calls) = self.calls.lock() {
            calls.push(call);
        }
    }

    fn play(&self, steps: usize) -> EventRx {
        let (tx, rx) = channel();
        let fail = self.fail_at.lock().ok().and_then(|f| f.clone());
        let delay = self.delay_ms;
        std::thread::spawn(move || play_script(tx, steps, fail, delay));
        rx
    }
}

fn play_script(
    tx: Sender<FlowEvent>,
    steps: usize,
    fail: Option<(usize, String, bool)>,
    delay_ms: u64,
) {
    let wait = || {
        if delay_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        }
    };
    for index in 0..steps {
        if tx.send(FlowEvent::StepStarted(index)).is_err() {
            return;
        }
        crate::redraw::poke();
        wait();
        if let Some((at, message, conflict)) = &fail {
            if *at == index {
                let _ = tx.send(FlowEvent::StepFailed {
                    index,
                    message: message.clone(),
                    conflict: *conflict,
                });
                crate::redraw::poke();
                return;
            }
        }
        if tx.send(FlowEvent::StepDone(index)).is_err() {
            return;
        }
        crate::redraw::poke();
    }
    let _ = tx.send(FlowEvent::Finished);
    crate::redraw::poke();
}

fn request_json(call: &str, r: &FlowRequest) -> serde_json::Value {
    let mut v = match &r.op {
        FlowOp::Start {
            type_id,
            fields,
            name,
            start_point,
            publish,
        } => serde_json::json!({
            "type": type_id, "fields": fields, "name": name,
            "start_point": start_point, "publish": publish,
        }),
        FlowOp::Finish { branch, mode } => {
            serde_json::json!({ "branch": branch, "mode": mode.map(|m| format!("{m:?}")) })
        }
        FlowOp::Sync {
            branch,
            with_children,
        } => serde_json::json!({ "branch": branch, "with_children": with_children }),
        FlowOp::Promote { from, to } => serde_json::json!({ "from": from, "to": to }),
        FlowOp::SetParent { branch, parent } => {
            serde_json::json!({ "branch": branch, "parent": parent })
        }
    };
    v["call"] = call.into();
    v["op"] = r.op.verb().into();
    v["run_hooks"] = r.run_hooks.into();
    v
}

impl FlowActions for FakeFlowActions {
    fn preview(&self, request: &FlowRequest) -> PreviewRx {
        self.record(Call::Preview(request.clone()));
        let (tx, rx) = channel();
        let answer = self
            .preview
            .lock()
            .map(|p| p.clone())
            .unwrap_or_else(|_| Err(String::new()));
        let delay = self.delay_ms;
        std::thread::spawn(move || {
            if delay > 0 {
                std::thread::sleep(std::time::Duration::from_millis(delay));
            }
            let _ = tx.send(answer);
            crate::redraw::poke();
        });
        rx
    }

    fn execute(&self, request: &FlowRequest) -> EventRx {
        self.record(Call::Execute(request.clone()));
        let steps = self
            .preview
            .lock()
            .ok()
            .and_then(|p| p.as_ref().ok().map(|p| p.steps.len()))
            .unwrap_or(0);
        self.play(steps)
    }

    fn resume(&self, how: Resume) -> EventRx {
        self.record(Call::Resume(how));
        let (tx, rx) = channel();
        let _ = tx.send(FlowEvent::Finished);
        rx
    }
}

/// The actions the window's dialogs use: the fake named by
/// `GITPOULPI_FLOW_FAKE` (a test seam), else the daemon's flow requests.
pub(crate) fn default_actions(requester: crate::daemon_link::Requester) -> Arc<dyn FlowActions> {
    match std::env::var_os("GITPOULPI_FLOW_FAKE") {
        Some(path) if !path.is_empty() => Arc::new(FakeFlowActions {
            calls: Mutex::new(Vec::new()),
            preview: Mutex::new(Ok(canned_preview())),
            delay_ms: std::env::var("GITPOULPI_FLOW_FAKE_DELAY_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            fail_at: Mutex::new(None),
            log: Some(path.into()),
        }),
        _ => Arc::new(super::daemon::DaemonFlowActions::new(requester)),
    }
}

/// The steps the env-selected fake previews (English: it is a test double).
fn canned_preview() -> FlowPreview {
    // i18n-scan: off
    FlowPreview {
        steps: vec![
            FlowStep::new("Create the branch", StepKind::Normal),
            FlowStep::new("Check it out", StepKind::Normal),
        ],
        ..Default::default()
    }
    // i18n-scan: on
}
