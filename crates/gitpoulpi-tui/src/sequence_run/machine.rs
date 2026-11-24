//! The sequence runner's state machine: pure data in, effects out.
//!
//! No ratatui, no daemon link, no window: the window's per-frame glue
//! (`event_loop/window/sequence.rs`) turns what it sees into [`RunEvent`]s,
//! hands them to [`Run::on`] and carries out the [`RunEffect`]s it gets back.
//! That keeps every transition testable without a terminal.
//!
//! One run is a list of resolved steps and a cursor. A step is `Ready`, gets
//! executed (`Submitted` while the daemon works, `AwaitingAi` while the AI
//! does, `Paused` when it waits for a person, `WaitingForUser` when a dialog
//! of the action itself does), and then the cursor moves on. A run that ends
//! well or is canceled leaves the machine; one that fails stays visible
//! (`Failed`) until dismissed.

use crate::daemon_link::{StepOutcome, StepPayload, Ticket, UserWait};
use crate::palette::PaletteCommand;
use gitpoulpi_config::sequences::{PauseMode, SequenceDef};
use gitpoulpi_i18n::msg::sequence_run as m;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// How long a step waits for the window of its tab to come up (a worktree
/// tab opens once the daemon lists it).
const WINDOW_WAIT: Duration = Duration::from_secs(30);
/// How long a step waits for the status refresh that follows a mutation
/// before it goes on anyway (a mutation with nothing to refresh).
const REFRESH_WAIT: Duration = Duration::from_secs(3);

/// A step's options: the TOML table of the step, read by key.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct StepOptions(pub(crate) toml::Table);

impl StepOptions {
    pub(crate) fn text(&self, key: &str) -> Option<&str> {
        self.0
            .get(key)
            .and_then(toml::Value::as_str)
            .filter(|s| !s.trim().is_empty())
    }

    pub(crate) fn flag(&self, key: &str) -> bool {
        self.0
            .get(key)
            .and_then(toml::Value::as_bool)
            .unwrap_or(false)
    }

    /// `key = value, ...` for the Activity Log.
    pub(crate) fn describe(&self) -> String {
        if self.0.is_empty() {
            return "-".to_string();
        }
        self.0
            .iter()
            .map(|(k, v)| format!("{k} = {v}"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// One step as the runner sees it.
#[derive(Debug, Clone)]
pub(crate) struct Step {
    /// The palette command id (`stage_all`).
    pub(crate) id: String,
    /// What the step is called to the user (the palette's label).
    pub(crate) label: String,
    /// `None` when this build has no such command.
    pub(crate) command: Option<PaletteCommand>,
    pub(crate) pause: PauseMode,
    pub(crate) options: StepOptions,
    /// The vocabulary says the step always asks before it runs.
    pub(crate) pause_forced: bool,
}

/// What the run tracks across steps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RunContext {
    /// The repository the run started in.
    pub(crate) origin_root: PathBuf,
    /// The repository the next step acts on (a worktree step with
    /// `open_tab` moves it to the new worktree).
    pub(crate) target_root: PathBuf,
    pub(crate) last_worktree: Option<PathBuf>,
    pub(crate) last_commit: Option<String>,
    pub(crate) started_at: Instant,
    /// The window of `target_root` has been seen: from then on a missing
    /// tab means it was closed.
    pub(crate) target_seen: bool,
}

/// Why a step stopped to ask.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PauseKind {
    /// `pause = always` (or a forced one): "Next: <step>" before it runs.
    BeforeStep,
    /// The AI wrote the message into the composer: look at it first.
    ReviewMessage,
    /// The commit has no message: the field has the keyboard, and the commit
    /// (the button, the key, or Continue) goes on from there.
    EmptyCommit,
    /// The composer's review of the proposed commits is up.
    ComposeReview,
    /// A dialog of the step's own is up.
    Dialog(DialogKind),
}

/// Which dialog a paused step waits on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DialogKind {
    Worktree,
    PushChooser,
    /// A confirmation (the force push's).
    Confirm,
    /// Any other form or prompt the command opened.
    Other,
}

/// Where the run is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum StepState {
    /// The current step has not been executed.
    Ready,
    /// Executed; the daemon works on it under this ticket.
    Submitted(Ticket),
    /// The AI request of the step runs.
    AwaitingAi,
    Paused(PauseKind),
    /// A dialog of the action itself (auto-stash, sensitive files, a
    /// rejected push) waits for a person; the ticket's final outcome follows.
    WaitingForUser(UserWait),
    /// The step's tab is not the one on screen.
    WaitingForWindow,
    /// Every step ran.
    Done,
    Failed {
        step: usize,
        text: String,
    },
    Canceled,
}

/// What the glue saw when it executed a step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ExecResult {
    /// The command submitted an action under the step's ticket.
    Submitted,
    /// The AI request started.
    AiStarted,
    /// Nothing to wait for (opening a terminal).
    Instant,
    /// The command opened a dialog the step waits on.
    Dialog(DialogKind),
    /// The commit has no message.
    EmptyCommit,
    /// The step could not start; this is why.
    Refused(String),
}

/// What the window tells the machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RunEvent {
    /// Once per frame, from the window that is on screen.
    Tick {
        now: Instant,
        /// The repository root of that window.
        active_root: PathBuf,
        /// No dialog or form owns the keyboard.
        input_idle: bool,
        /// A counter of the status refreshes the windows applied.
        refreshes: u64,
        /// The target's tab is still open.
        target_open: bool,
    },
    /// The ticket of the step ended.
    StepDone {
        ticket: Ticket,
        outcome: StepOutcome,
    },
    /// The AI request of the step ended.
    AiDone(crate::ai_ui::AiEnd),
    /// The step was executed: what came of it.
    Executed(ExecResult),
    /// The Continue button, key or palette row.
    Continue,
    Cancel,
    /// A dialog the step waits on closed without an answer.
    DialogClosed,
    /// The tab the run acts on is gone.
    WindowGone,
}

/// What the glue has to do.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum RunEffect {
    /// Run the step's command under `ticket`.
    Execute {
        step: usize,
        ticket: Ticket,
        command: Option<PaletteCommand>,
        options: StepOptions,
    },
    /// The strip, the job label and the status line follow the state.
    ShowStrip,
    /// Stop what the step has in flight (the daemon job, the AI request).
    CancelJob,
    /// Put this on the status line.
    Say(String),
    /// An Activity Log failure entry.
    LogFailure(String),
    /// Apply the composer's review as proposed (`pause = never`).
    ApplyCompose,
    /// The run is over (done, canceled or failed): close its job.
    Finish,
}

/// One run.
#[derive(Debug, Clone)]
pub(crate) struct Run {
    pub(crate) name: String,
    pub(crate) steps: Vec<Step>,
    pub(crate) step: usize,
    pub(crate) state: StepState,
    pub(crate) ctx: RunContext,
    /// The jobs-panel entry of the run.
    pub(crate) job_id: Option<u64>,
    /// What the current step waits on.
    pub(crate) ticket: Option<Ticket>,
    /// The "Next: <step>" pause was answered for the current step.
    before_acked: bool,
    /// The status-refresh counter when the last step finished; the next one
    /// waits for a later one.
    refresh_floor: Option<u64>,
    last_refreshes: u64,
    waiting_since: Option<Instant>,
    /// When the step in flight began (the daemon jobs it started are those
    /// first seen after it).
    pub(crate) step_started: Option<Instant>,
    /// Where the run first saw a refresh floor.
    floor_set_at: Option<Instant>,
}

impl Run {
    /// A run of `def`; `resolve` turns a step id into its command and label.
    pub(crate) fn new(
        def: &SequenceDef,
        origin_root: &Path,
        now: Instant,
        resolve: impl Fn(&str) -> (Option<PaletteCommand>, String),
    ) -> Run {
        let steps = def
            .steps
            .iter()
            .map(|spec| {
                let (command, label) = resolve(&spec.run);
                Step {
                    id: spec.run.clone(),
                    label,
                    command,
                    pause: spec.pause,
                    options: StepOptions(spec.options.clone()),
                    pause_forced: gitpoulpi_config::sequences::step_kind(&spec.run)
                        .is_some_and(|k| k.pause_forced),
                }
            })
            .collect();
        Run {
            name: def.name.clone(),
            steps,
            step: 0,
            state: StepState::Ready,
            ctx: RunContext {
                origin_root: origin_root.to_path_buf(),
                target_root: origin_root.to_path_buf(),
                last_worktree: None,
                last_commit: None,
                started_at: now,
                target_seen: true,
            },
            job_id: None,
            ticket: None,
            before_acked: false,
            refresh_floor: None,
            last_refreshes: 0,
            waiting_since: None,
            step_started: None,
            floor_set_at: None,
        }
    }

    pub(crate) fn current(&self) -> Option<&Step> {
        self.steps.get(self.step)
    }

    /// The current step's label, or `-` once past the end.
    pub(crate) fn step_label(&self) -> String {
        self.current()
            .map_or_else(|| "-".to_string(), |s| s.label.clone())
    }

    /// 1-based position of the current step, and the total.
    pub(crate) fn position(&self) -> (usize, usize) {
        ((self.step + 1).min(self.steps.len()), self.steps.len())
    }

    /// The run is over: nothing more will happen (a failed run is shown
    /// until dismissed, but it does not block another).
    pub(crate) fn is_over(&self) -> bool {
        matches!(
            self.state,
            StepState::Done | StepState::Canceled | StepState::Failed { .. }
        )
    }

    /// The jobs-panel label.
    pub(crate) fn job_label(&self) -> String {
        let (k, n) = self.position();
        m::job_label(&self.name, k, n, self.step_label())
    }

    /// The step wants an answer from a person the button can give.
    pub(crate) fn can_continue(&self) -> bool {
        matches!(
            self.state,
            StepState::Paused(
                PauseKind::BeforeStep | PauseKind::ReviewMessage | PauseKind::EmptyCommit
            )
        )
    }

    /// The effects that bring the visible parts (job label, strip) up to date.
    fn show(&self) -> RunEffect {
        RunEffect::ShowStrip
    }

    fn fail(&mut self, text: String) -> Vec<RunEffect> {
        let (k, n) = self.position();
        let step = self.step_label();
        let options = self
            .current()
            .map_or_else(|| "-".to_string(), |s| s.options.describe());
        self.state = StepState::Failed {
            step: self.step,
            text: text.clone(),
        };
        self.ticket = None;
        vec![
            RunEffect::LogFailure(m::log_failure(&self.name, k, n, &step, options, &text)),
            RunEffect::Say(m::say_stopped(&self.name, k, n, &step, &text)),
            RunEffect::Finish,
            self.show(),
        ]
    }

    fn cancel(&mut self, reason: Option<String>, stop_job: bool) -> Vec<RunEffect> {
        let (k, n) = self.position();
        let step = self.step_label();
        let mut out = Vec::new();
        if stop_job
            && matches!(
                self.state,
                StepState::Submitted(_) | StepState::AwaitingAi | StepState::WaitingForUser(_)
            )
        {
            out.push(RunEffect::CancelJob);
        }
        self.state = StepState::Canceled;
        self.ticket = None;
        let mut said = m::say_canceled(&self.name, k, n, &step);
        if let Some(reason) = reason {
            said = format!("{said} ({reason})");
        }
        out.push(RunEffect::Say(said));
        out.push(RunEffect::Finish);
        out
    }

    /// The current step is done: on to the next, or the end.
    fn advance(&mut self) -> Vec<RunEffect> {
        self.ticket = None;
        self.before_acked = false;
        self.refresh_floor = Some(self.last_refreshes);
        self.floor_set_at = None;
        self.step += 1;
        self.step_started = None;
        if self.step >= self.steps.len() {
            self.step = self.steps.len().saturating_sub(1);
            self.state = StepState::Done;
            return vec![
                RunEffect::Say(m::say_finished(&self.name)),
                RunEffect::Finish,
            ];
        }
        self.state = StepState::Ready;
        vec![self.show()]
    }

    /// What a successful step leaves behind.
    fn remember(&mut self, payload: Option<StepPayload>) {
        // Opening the worktree the run made as a tab: the next steps act there.
        if self
            .current()
            .is_some_and(|s| s.id == "open_worktree_in_tab")
        {
            if let Some(path) = self.ctx.last_worktree.clone() {
                self.ctx.target_root = path;
                self.ctx.target_seen = false;
            }
        }
        if let Some(StepPayload::Worktree(path)) = payload {
            self.ctx.last_worktree = Some(path.clone());
            let opens = self.current().is_some_and(|s| s.options.flag("open_tab"));
            if opens {
                self.ctx.target_root = path;
                self.ctx.target_seen = false;
            }
        }
    }

    /// Feeds one event, returns what the glue has to do.
    pub(crate) fn on(&mut self, event: RunEvent) -> Vec<RunEffect> {
        if self.is_over() {
            // A failed run is dismissed with Cancel; anything else is moot.
            if matches!(self.state, StepState::Failed { .. }) && event == RunEvent::Cancel {
                self.state = StepState::Canceled;
            }
            return Vec::new();
        }
        match event {
            RunEvent::Cancel => self.cancel(None, true),
            RunEvent::WindowGone => {
                let tab = self.ctx.target_root.display().to_string();
                self.cancel(Some(m::reason_tab_closed(tab)), true)
            }
            RunEvent::DialogClosed => match self.state {
                StepState::Paused(
                    PauseKind::ComposeReview
                    | PauseKind::Dialog(DialogKind::Other | DialogKind::Confirm),
                ) => self.cancel(Some(m::REASON_DIALOG_CLOSED.get().to_string()), false),
                _ => Vec::new(),
            },
            RunEvent::Continue => self.on_continue(),
            RunEvent::StepDone { ticket, outcome } => self.on_step_done(ticket, outcome),
            RunEvent::AiDone(end) => self.on_ai_done(end),
            RunEvent::Executed(result) => self.on_executed(result),
            RunEvent::Tick {
                now,
                active_root,
                input_idle,
                refreshes,
                target_open,
            } => self.on_tick(now, &active_root, input_idle, refreshes, target_open),
        }
    }

    fn on_continue(&mut self) -> Vec<RunEffect> {
        match self.state {
            StepState::Paused(PauseKind::BeforeStep) => {
                self.before_acked = true;
                self.state = StepState::Ready;
                vec![self.show()]
            }
            StepState::Paused(PauseKind::ReviewMessage) => self.advance(),
            // The commit is tried again: the message may be there now.
            StepState::Paused(PauseKind::EmptyCommit) => {
                self.before_acked = true;
                self.state = StepState::Ready;
                vec![self.show()]
            }
            _ => Vec::new(),
        }
    }

    fn on_step_done(&mut self, ticket: Ticket, outcome: StepOutcome) -> Vec<RunEffect> {
        if self.ticket != Some(ticket) {
            return Vec::new();
        }
        match outcome {
            StepOutcome::Ok(payload) => {
                self.remember(payload);
                self.advance()
            }
            StepOutcome::Failed { text } => self.fail(text),
            StepOutcome::NeedsUser(wait) => {
                self.state = StepState::WaitingForUser(wait);
                vec![self.show()]
            }
            StepOutcome::Canceled => {
                self.cancel(Some(m::REASON_DIALOG_CLOSED.get().to_string()), false)
            }
        }
    }

    fn on_ai_done(&mut self, end: crate::ai_ui::AiEnd) -> Vec<RunEffect> {
        if self.state != StepState::AwaitingAi {
            return Vec::new();
        }
        use crate::ai_ui::AiEnd;
        let Some(step) = self.current().cloned() else {
            return Vec::new();
        };
        match end {
            AiEnd::Failed(text) => self.fail(text),
            AiEnd::Canceled => self.cancel(None, false),
            AiEnd::Ok if step.id == "ai_compose_commits" => {
                self.state = StepState::Paused(PauseKind::ComposeReview);
                if step.pause == PauseMode::Never {
                    vec![RunEffect::ApplyCompose, self.show()]
                } else {
                    vec![self.show()]
                }
            }
            AiEnd::Ok if step.pause == PauseMode::Never => self.advance(),
            AiEnd::Ok => {
                self.state = StepState::Paused(PauseKind::ReviewMessage);
                vec![self.show()]
            }
        }
    }

    fn on_executed(&mut self, result: ExecResult) -> Vec<RunEffect> {
        if !matches!(self.state, StepState::Submitted(_)) {
            return Vec::new();
        }
        match result {
            ExecResult::Submitted => Vec::new(),
            ExecResult::AiStarted => {
                self.state = StepState::AwaitingAi;
                vec![self.show()]
            }
            ExecResult::Instant => {
                self.remember(None);
                self.advance()
            }
            ExecResult::Dialog(kind) => {
                self.state = StepState::Paused(PauseKind::Dialog(kind));
                vec![self.show()]
            }
            ExecResult::EmptyCommit => {
                self.state = StepState::Paused(PauseKind::EmptyCommit);
                vec![self.show()]
            }
            ExecResult::Refused(text) => self.fail(text),
        }
    }

    fn on_tick(
        &mut self,
        now: Instant,
        active_root: &Path,
        input_idle: bool,
        refreshes: u64,
        target_open: bool,
    ) -> Vec<RunEffect> {
        self.last_refreshes = refreshes;
        let at_target = same_dir(active_root, &self.ctx.target_root);
        if at_target {
            self.ctx.target_seen = true;
        } else if self.ctx.target_seen && !target_open {
            return self.on(RunEvent::WindowGone);
        }
        match self.state {
            StepState::WaitingForWindow => {
                if at_target {
                    self.state = StepState::Ready;
                    self.waiting_since = None;
                    return vec![self.show()];
                }
                let since = *self.waiting_since.get_or_insert(now);
                if !self.ctx.target_seen && now.duration_since(since) > WINDOW_WAIT {
                    let tab = self.ctx.target_root.display().to_string();
                    return self.fail(m::reason_tab_never(tab));
                }
                Vec::new()
            }
            StepState::Ready => {
                if !at_target {
                    self.state = StepState::WaitingForWindow;
                    self.waiting_since = Some(now);
                    return vec![self.show()];
                }
                if !input_idle {
                    return Vec::new();
                }
                // The status the step reads must be one read after the
                // mutation before it (the stage is shown optimistically).
                if let Some(floor) = self.refresh_floor {
                    let since = *self.floor_set_at.get_or_insert(now);
                    if refreshes <= floor && now.duration_since(since) < REFRESH_WAIT {
                        return Vec::new();
                    }
                    self.refresh_floor = None;
                    self.floor_set_at = None;
                }
                let Some(step) = self.current().cloned() else {
                    return self.advance();
                };
                if (step.pause == PauseMode::Always || step.pause_forced) && !self.before_acked {
                    self.state = StepState::Paused(PauseKind::BeforeStep);
                    return vec![self.show()];
                }
                let ticket = crate::daemon_link::issue();
                self.ticket = Some(ticket);
                self.state = StepState::Submitted(ticket);
                self.step_started = Some(now);
                self.before_acked = false;
                vec![
                    RunEffect::Execute {
                        step: self.step,
                        ticket,
                        command: step.command,
                        options: step.options,
                    },
                    self.show(),
                ]
            }
            _ => Vec::new(),
        }
    }
}

/// Same folder, whatever the spelling (symlinks, trailing separators).
pub(crate) fn same_dir(a: &Path, b: &Path) -> bool {
    a == b
        || match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
}

#[cfg(test)]
mod tests;
