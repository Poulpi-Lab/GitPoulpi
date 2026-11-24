//! AI in the TUI: the worker that runs one AI request off the input loop,
//! and the state the render loop keeps about it.
//!
//! **Off by default and invisible while off.** [`AiUi::enabled`] is
//! `ai.enabled` from `config.toml`; every entry point (the composer's
//! Generate button, the palette entries, the `Ctrl+G` chords, the menu
//! rows) checks it, so with AI off there is nothing on screen that
//! mentions AI.
//!
//! **Same shape as `clone_task`.** One `std::thread` per request owns a
//! small `current_thread` tokio runtime and reports on an
//! `std::sync::mpsc` channel the render loop drains with `try_recv`. The
//! worker reads what it needs (the diff) over its *own* connection to
//! the session's daemon, resolves the action and loads the API key (a
//! credential-helper subprocess -- never on the input thread), then calls
//! the model. Esc sends on a `oneshot`; the worker's `select!` then drops
//! the request future, which kills a local agent's process.

use crate::redraw::NotifySend as _;
use crate::th;
use std::cell::Cell;
use std::sync::mpsc as std_mpsc;
use std::time::Instant;

use gitpoulpi_ai::assist::{render_diff, Job};
use gitpoulpi_ai::{parse, prompts};

use crate::wide_edge::Clear;
/// What the merge editor hands the AI about one conflict (the stub's twin
/// is a unit-like type that is never built).
pub(crate) use gitpoulpi_ai::prompts::ConflictContext;
use gitpoulpi_config::{AiAction, AiConfig};
use gitpoulpi_core::domain::{
    CommitGraph, DiffTarget, FileDiff, FileStatus, LogOptions, Oid, OperationKind, RepoRef,
    SplitPart, SplitSource, WorkingTree,
};
use gitpoulpi_ipc::{Client, Request};
use gitpoulpi_secrets::CredentialHelperTokens;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};
use ratatui::Frame;

use gitpoulpi_i18n::msg::ai_page as action_msg;
use gitpoulpi_i18n::msg::ai_ui as msg;
use gitpoulpi_i18n::msg::rewrite_ai as rewrite_msg;

/// What a history-rewrite script request got back (script, check problems).
pub(crate) use gitpoulpi_ai::assist::RewriteScriptOutcome;

use crate::rewrite_view::{RulesCtx, RulesProposal, SampleRow};

/// An action's name in the UI language.
fn action_label(action: AiAction) -> &'static str {
    match action {
        AiAction::CommitMessage => action_msg::ACTION_COMMIT_MESSAGE,
        AiAction::ExplainChanges => action_msg::ACTION_EXPLAIN,
        AiAction::StashMessage => action_msg::ACTION_STASH,
        AiAction::PullRequestDescription => action_msg::ACTION_PULL_REQUEST,
        AiAction::CommitComposer => action_msg::ACTION_COMPOSER,
        AiAction::ConflictResolve => action_msg::ACTION_CONFLICT_RESOLVE,
        AiAction::CodeReview => gitpoulpi_i18n::msg::review_ai::ACTION_CODE_REVIEW,
        AiAction::SplitCommit => action_msg::ACTION_SPLIT_COMMIT,
        AiAction::RewriteScript => rewrite_msg::ACTION,
        AiAction::RewriteRules => gitpoulpi_i18n::msg::rewrite_rules_ai::ACTION,
        AiAction::WorktreeName => action_msg::ACTION_WORKTREE_NAME,
        AiAction::HostingReply => action_msg::ACTION_HOSTING_REPLY,
        AiAction::IssueBody => action_msg::ACTION_ISSUE_BODY,
    }
    .get()
}

/// The English name of `lang` for the Explain prompt's "Answer in ..."
/// line: `None` for English (the base prompt already is) and for the
/// pseudo-locale.
fn answer_language_name(lang: gitpoulpi_i18n::Lang) -> Option<&'static str> {
    use gitpoulpi_i18n::Lang;
    match lang {
        Lang::En | Lang::Qps => None,
        Lang::Fr => Some("French"),                 // i18n-english
        Lang::Es => Some("Spanish"),                // i18n-english
        Lang::PtBr => Some("Brazilian Portuguese"), // i18n-english
        Lang::PtPt => Some("European Portuguese"),  // i18n-english
        Lang::It => Some("Italian"),                // i18n-english
        Lang::De => Some("German"),                 // i18n-english
        Lang::ZhCn => Some("Simplified Chinese"),   // i18n-english
        Lang::Ja => Some("Japanese"),               // i18n-english
        Lang::Ru => Some("Russian"),                // i18n-english
        Lang::Ko => Some("Korean"),                 // i18n-english
        Lang::Nl => Some("Dutch"),                  // i18n-english
        Lang::Pl => Some("Polish"),                 // i18n-english
        Lang::Uk => Some("Ukrainian"),              // i18n-english
        Lang::Cs => Some("Czech"),                  // i18n-english
        Lang::Sv => Some("Swedish"),                // i18n-english
        Lang::Tr => Some("Turkish"),                // i18n-english
    }
}

/// The UI language's English name, for the `${uilang}` prompt variable.
fn ui_language_english() -> &'static str {
    answer_language_name(gitpoulpi_i18n::current()).unwrap_or("English") // i18n-english
}

/// `job` told to write a ✦ Explain answer in the UI language (the other
/// actions ignore it: what lands in the repository follows the user's
/// instructions, not the UI).
fn with_ui_language(job: Job) -> Job {
    job.with_answer_language(answer_language_name(gitpoulpi_i18n::current()).map(str::to_string))
}

/// What an "explain" is about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ExplainTarget {
    /// A commit: its diff against its first parent.
    Commit {
        oid: Oid,
        /// Its full message, sent along as context.
        message: String,
        /// "abc1234", for titles.
        short: String,
    },
    /// Everything uncommitted, staged and unstaged.
    Wip,
}

/// What the user asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AiRequest {
    /// A commit message for what is staged (or, when nothing is, for
    /// everything that changed).
    CommitMessage,
    /// A plain-language explanation of a commit or of the WIP.
    Explain(ExplainTarget),
    /// A one-line name for a stash of the working changes.
    StashMessage,
    /// A title and description for a pull request from `source` into
    /// `target` (branch names in this repository).
    PullRequest { source: String, target: String },
    /// A plan splitting the working changes into logical commits.
    Compose,
    /// A split of one commit into several: `oid`, `short` for titles.
    SplitCommit { oid: Oid, short: String },
    /// A resolution of one merge conflict (0-based `conflict` of `path`);
    /// the context was read from the open editor.
    ResolveConflict {
        path: String,
        conflict: usize,
        context: Box<prompts::ConflictContext>,
        /// The user saw the "too large" warning and chose "Send anyway":
        /// this one request ignores `ai.limits.conflict_bytes`.
        oversize_ok: bool,
    },
    /// A history rewrite script for `instruction`; `sample` is what the user
    /// agreed to send along (empty unless the box was ticked).
    RewriteScript {
        instruction: String,
        sample: Vec<SampleRow>,
    },
    /// Rules and settings for a history rewrite, from a description;
    /// `context` is what the user agreed to send along (empty unless the box
    /// was ticked). The answer only fills the workbench.
    RewriteRules {
        description: String,
        context: RulesCtx,
    },
    /// A worktree name and a branch name for a described task; `branches`
    /// are the repository's local branch names, so the answer follows
    /// their conventions. Only fills the Create worktree dialog's fields.
    WorktreeName {
        description: String,
        branches: Vec<String>,
    },
}

impl AiRequest {
    fn action(&self) -> AiAction {
        match self {
            AiRequest::CommitMessage => AiAction::CommitMessage,
            AiRequest::Explain(_) => AiAction::ExplainChanges,
            AiRequest::StashMessage => AiAction::StashMessage,
            AiRequest::PullRequest { .. } => AiAction::PullRequestDescription,
            AiRequest::Compose => AiAction::CommitComposer,
            AiRequest::ResolveConflict { .. } => AiAction::ConflictResolve,
            AiRequest::SplitCommit { .. } => AiAction::SplitCommit,
            AiRequest::RewriteScript { .. } => AiAction::RewriteScript,
            AiRequest::RewriteRules { .. } => AiAction::RewriteRules,
            AiRequest::WorktreeName { .. } => AiAction::WorktreeName,
        }
    }
}

/// What came back, by kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AiAnswer {
    CommitMessage(parse::CommitMessage),
    Explanation(String),
    StashMessage(String),
    PullRequest(parse::PullRequestText),
    /// (message, files) per commit, validated against the change set.
    Plan(Vec<(String, Vec<String>)>),
    /// The resolved lines of one conflict.
    Resolution(Vec<String>),
    /// The commit's numbered changes (read either way, so the editor can
    /// open) and the AI's validated plan for them, or why it has none.
    Split {
        source: Box<SplitSource>,
        plan: Result<Vec<SplitPart>, String>,
    },
    /// The script the model wrote and what the check said about it.
    RewriteScript(Box<RewriteScriptOutcome>),
    /// The rules, scope and settings the model proposed (data only).
    RewriteRules(Box<RulesProposal>),
    /// The worktree name (a slug) and the branch name, both valid.
    WorktreeName(AiNames),
}

/// What the worktree naming answered (see [`gitpoulpi_ai::worktree_name`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AiNames {
    pub(crate) name: String,
    pub(crate) branch: String,
}

/// How a request ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AiOutcome {
    Done(AiRequest, Result<AiAnswer, String>),
    /// The user canceled; nothing to apply.
    Canceled(AiRequest),
}

/// How a finished request ended, for whoever waits on it (a sequence step).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AiEnd {
    Ok,
    Failed(String),
    Canceled,
}

/// The ends nobody has taken yet, by kind of request. Process-wide: the
/// window that polled the answer may not be the one that asked, and a tab
/// switch rebuilds it.
static FINISHED: std::sync::Mutex<Vec<(AiAction, AiEnd)>> = std::sync::Mutex::new(Vec::new());

/// What `outcome` means to a waiter; read from the outcome the window
/// applies (`event_loop::window::tick`), never from a second poll.
pub(crate) fn finished_of(outcome: &AiOutcome) -> (AiAction, AiEnd) {
    match outcome {
        AiOutcome::Done(request, Ok(_)) => (request.action(), AiEnd::Ok),
        AiOutcome::Done(request, Err(why)) => (request.action(), AiEnd::Failed(why.clone())),
        AiOutcome::Canceled(request) => (request.action(), AiEnd::Canceled),
    }
}

/// Makes the end of a request visible to [`take_finished`]; called once the
/// window has applied the answer. Only the newest end of a kind is kept.
pub(crate) fn record_finished((kind, end): (AiAction, AiEnd)) {
    let mut ends = FINISHED.lock().unwrap_or_else(|e| e.into_inner());
    ends.retain(|(k, _)| *k != kind);
    ends.push((kind, end));
}

/// The end of the last finished request of `kind`, once. `None` while it
/// runs (or nothing of that kind finished since the last call).
#[allow(dead_code)] // the sequence runner (a later slice) is the reader
pub(crate) fn take_finished(kind: AiAction) -> Option<AiEnd> {
    let mut ends = FINISHED.lock().unwrap_or_else(|e| e.into_inner());
    let at = ends.iter().position(|(k, _)| *k == kind)?;
    Some(ends.remove(at).1)
}

/// Shared by the run and the jobs list's Cancel, so a request that outlives
/// its window (the tab was left) still stops from the indicator.
type CancelSlot = std::sync::Arc<std::sync::Mutex<Option<tokio::sync::oneshot::Sender<()>>>>;

/// Requests whose tab was left, by repository root: they keep running and
/// their answer waits here until that repository's tab is shown again
/// (see the background-ops design, B11, maintainer answer 4).
static PARKED: std::sync::Mutex<Vec<(std::path::PathBuf, Run)>> = std::sync::Mutex::new(Vec::new());

struct Run {
    request: AiRequest,
    rx: std_mpsc::Receiver<AiOutcome>,
    cancel: CancelSlot,
    /// The entry in the jobs list (indicator, panel, palette).
    job: u64,
    started: Instant,
    /// "Commit message via Claude Code", for the status line.
    label: String,
    /// The profile's name alone.
    via: String,
}

pub(crate) struct AiUi {
    config: AiConfig,
    run: Option<Run>,
    popup: Option<AiPopup>,
    /// The repository this window shows: where a request is parked when the
    /// window goes away.
    owner: Option<std::path::PathBuf>,
}

impl Drop for AiUi {
    fn drop(&mut self) {
        let Some(run) = self.run.take() else {
            return;
        };
        match self.owner.take() {
            Some(owner) => PARKED
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .push((owner, run)),
            None => stop(&run.cancel),
        }
    }
}

fn stop(slot: &CancelSlot) {
    if let Some(cancel) = slot.lock().unwrap_or_else(|e| e.into_inner()).take() {
        let _ = cancel.send(());
    }
}

impl AiUi {
    pub(crate) fn new() -> Self {
        let mut ui = Self {
            config: AiConfig::default(),
            run: None,
            popup: None,
            owner: None,
        };
        ui.reload();
        ui
    }

    /// Binds the window to its repository and takes back the request that
    /// was running when its tab was left (its answer, if it already came, is
    /// delivered by the next [`AiUi::poll`]).
    pub(crate) fn adopt(&mut self, root: &std::path::Path) {
        self.owner = Some(root.to_path_buf());
        let mut parked = PARKED.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(at) = parked.iter().position(|(owner, _)| owner == root) {
            if self.run.is_none() {
                self.run = Some(parked.remove(at).1);
            }
        }
    }

    /// Re-reads `[ai]` from `config.toml` - at start, and when the
    /// Settings screen closes.
    pub(crate) fn reload(&mut self) {
        self.config = gitpoulpi_config::Settings::resolve_ambient()
            .map(|s| s.config().ai.clone())
            .unwrap_or_default();
    }

    /// `ai.limits.conflict_bytes`: the size over which a conflict is not sent
    /// without asking.
    pub(crate) fn conflict_limit(&self) -> usize {
        self.config.limits.conflict_bytes_clamped()
    }

    /// Whether AI entry points are shown at all.
    pub(crate) fn enabled(&self) -> bool {
        crate::build_features::AI && self.config.enabled
    }

    pub(crate) fn running(&self) -> bool {
        self.run.is_some()
    }

    /// A pull-request generation is the running request.
    pub(crate) fn running_pull_request(&self) -> bool {
        self.run
            .as_ref()
            .is_some_and(|r| matches!(r.request, AiRequest::PullRequest { .. }))
    }

    /// A worktree naming is the running request.
    pub(crate) fn running_worktree_name(&self) -> bool {
        self.run
            .as_ref()
            .is_some_and(|r| matches!(r.request, AiRequest::WorktreeName { .. }))
    }

    /// A conflict-resolution suggestion is the running request.
    pub(crate) fn running_conflict(&self) -> bool {
        self.run
            .as_ref()
            .is_some_and(|r| matches!(r.request, AiRequest::ResolveConflict { .. }))
    }

    /// The running suggestion's conflict (0-based), profile and seconds so
    /// far, for the merge editor's own progress line.
    pub(crate) fn running_conflict_info(&self) -> Option<(usize, String, u64)> {
        let run = self.run.as_ref()?;
        let AiRequest::ResolveConflict { conflict, .. } = &run.request else {
            return None;
        };
        Some((*conflict, run.via.clone(), run.started.elapsed().as_secs()))
    }

    /// "Generating: Commit message via X... 3s (Esc cancels)".
    pub(crate) fn progress_line(&self) -> Option<String> {
        self.run.as_ref().map(|r| {
            format!(
                "{} {}",
                crate::theme::AI_SPARK,
                msg::working(&r.label, r.started.elapsed().as_secs())
            )
        })
    }

    /// Starts `request` for the repository. `Err` is a sentence for the
    /// status line (AI off, no profile, a request already running).
    pub(crate) fn start(
        &mut self,
        session: &str,
        repo: &RepoRef,
        request: AiRequest,
    ) -> Result<String, String> {
        if !self.enabled() {
            return Err(msg::OFF.get().into());
        }
        if self.run.is_some() {
            return Err(msg::BUSY.get().into());
        }
        // Resolved here only to fail fast and name the profile in the
        // status line; the worker resolves again with the repository's
        // own override, which needs a file read.
        let profile = self
            .config
            .resolve(request.action(), None)
            .map_err(|e| e.to_string())?
            .profile;
        let profile_name = profile.display_name().to_string();
        let label = msg::label_via(action_label(request.action()), profile.display_name());

        let (tx, rx) = std_mpsc::channel();
        let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel::<()>();
        let config = self.config.clone();
        let session = session.to_string();
        let repo = repo.clone();
        let job_request = request.clone();
        let slot: CancelSlot = std::sync::Arc::new(std::sync::Mutex::new(Some(cancel_tx)));
        let stopper = std::sync::Arc::clone(&slot);
        let job = crate::jobs_ui::local_start(
            format!("{} {label}", crate::theme::AI_SPARK),
            std::sync::Arc::new(move || stop(&stopper)),
        );
        std::thread::spawn(move || {
            let outcome = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(runtime) => {
                    let outcome = runtime.block_on(async {
                        tokio::select! {
                            result = execute(config, session, repo, job_request.clone()) => {
                                AiOutcome::Done(job_request.clone(), result)
                            }
                            _ = cancel_rx => AiOutcome::Canceled(job_request.clone()),
                        }
                    });
                    // Dropping a runtime waits for its blocking threads, and
                    // on Windows a child's pipe is read on one: a canceled
                    // agent whose pipe something else still holds would keep
                    // the job "being canceled" until that ends.
                    runtime.shutdown_background();
                    outcome
                }
                Err(e) => AiOutcome::Done(job_request, Err(msg::runtime_failed(e))),
            };
            // The user read the request and chose Cancel: a cancel, not a failure.
            let outcome = match outcome {
                AiOutcome::Done(request, Err(why))
                    if why == gitpoulpi_ai::Error::Declined.to_string() =>
                {
                    AiOutcome::Canceled(request)
                }
                other => other,
            };
            let state = match &outcome {
                AiOutcome::Done(_, Ok(_)) => gitpoulpi_ipc::JobState::Done,
                AiOutcome::Done(_, Err(_)) => gitpoulpi_ipc::JobState::Failed,
                AiOutcome::Canceled(_) => gitpoulpi_ipc::JobState::Canceled,
            };
            crate::jobs_ui::local_finish(job, state);
            let _ = tx.send_notify(outcome);
        });
        self.run = Some(Run {
            request,
            rx,
            cancel: slot,
            job,
            started: Instant::now(),
            label: label.clone(),
            via: profile_name,
        });
        Ok(format!(
            "{} {}",
            crate::theme::AI_SPARK,
            msg::working_start(&label)
        ))
    }

    /// [`AiUi::start`], reporting through the status line the way every
    /// entry point (button, palette, chord) does.
    pub(crate) fn start_into(
        &mut self,
        session: &str,
        repo: &RepoRef,
        request: AiRequest,
        last_message: &mut Option<String>,
    ) {
        // A running request is named by the jobs indicator (with its Cancel
        // and elapsed time), not a second time by the status line.
        *last_message = self.start(session, repo, request).err();
    }

    /// Starts an explanation and opens the read-only popup that will
    /// show it (with a "working" body until the answer lands).
    pub(crate) fn explain_into(
        &mut self,
        session: &str,
        repo: &RepoRef,
        target: ExplainTarget,
        last_message: &mut Option<String>,
    ) {
        let title = match &target {
            ExplainTarget::Commit { short, .. } => msg::explain_commit_title(short),
            ExplainTarget::Wip => msg::EXPLAIN_WIP_TITLE.get().to_string(),
        };
        match self.start(session, repo, AiRequest::Explain(target)) {
            Ok(line) => {
                *last_message = None;
                self.popup = Some(AiPopup::working(title, line));
            }
            Err(line) => *last_message = Some(line),
        }
    }

    /// Esc: stop the running request. The worker reports `Canceled`.
    pub(crate) fn cancel(&mut self) {
        if let Some(run) = self.run.as_ref() {
            stop(&run.cancel);
        }
    }

    /// The finished request's outcome, if it finished.
    pub(crate) fn poll(&mut self) -> Option<AiOutcome> {
        let run = self.run.as_ref()?;
        match run.rx.try_recv() {
            Ok(outcome) => {
                self.run = None;
                Some(outcome)
            }
            Err(std_mpsc::TryRecvError::Empty) => None,
            Err(std_mpsc::TryRecvError::Disconnected) => {
                crate::jobs_ui::local_finish(run.job, gitpoulpi_ipc::JobState::Failed);
                let request = run.request.clone();
                self.run = None;
                Some(AiOutcome::Done(
                    request,
                    Err(msg::WORKER_STOPPED.get().into()),
                ))
            }
        }
    }

    // -- the read-only popup ---------------------------------------------

    pub(crate) fn popup_open(&self) -> bool {
        self.popup.is_some()
    }

    pub(crate) fn popup_mut(&mut self) -> Option<&mut AiPopup> {
        self.popup.as_mut()
    }

    pub(crate) fn close_popup(&mut self) {
        self.popup = None;
    }

    /// Shows an explanation (or the failure) in the popup, when it is
    /// still open.
    pub(crate) fn fill_popup(&mut self, result: Result<String, String>) {
        if let Some(popup) = self.popup.as_mut() {
            popup.set_result(result);
        }
    }

    pub(crate) fn render_popup(&self, frame: &mut Frame, area: Rect) {
        if let Some(popup) = &self.popup {
            popup.render(frame, area);
        }
    }
}

// ---------------------------------------------------------------------
// The popup
// ---------------------------------------------------------------------

enum PopupBody {
    Working(String),
    Text(String),
    Error(String),
}

/// A centered, scrollable, read-only text box: an explanation, or why
/// there is none.
pub(crate) struct AiPopup {
    title: String,
    body: PopupBody,
    scroll: u16,
    /// Recorded by the last render, so PageUp/PageDown page by what is
    /// on screen and the scroll can be clamped.
    inner: Cell<(u16, u16)>,
    /// The popup frame and its close cross as the last render painted them
    /// (hit rects come from the paint walk).
    frame: Cell<Rect>,
    cross: Cell<Option<Rect>>,
    cross_hover: Cell<bool>,
    /// The scrollbar in the right border as last painted, and the thumb held.
    bar: Cell<Option<crate::scrollbar::Track>>,
    bar_drag: Cell<crate::scrollbar::BarDrag>,
}

/// What a mouse event did to the popup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PopupMouse {
    Nothing,
    /// Close it (and cancel the request when it still runs).
    Close,
}

/// What a key did to the popup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PopupKey {
    Nothing,
    /// Close it (and cancel the request when it still runs).
    Close,
    /// Copy this text to the clipboard.
    Copy(String),
}

impl AiPopup {
    fn working(title: String, line: String) -> Self {
        Self {
            title,
            body: PopupBody::Working(line),
            scroll: 0,
            inner: Cell::new((0, 0)),
            frame: Cell::new(Rect::default()),
            cross: Cell::new(None),
            cross_hover: Cell::new(false),
            bar: Cell::new(None),
            bar_drag: Cell::new(Default::default()),
        }
    }

    /// The cross and a click outside the frame close the popup; a click
    /// inside never does (so text can be selected and scrolled by the bar).
    pub(crate) fn on_mouse(&mut self, mouse: crossterm::event::MouseEvent) -> PopupMouse {
        use crossterm::event::{MouseButton, MouseEventKind};
        let at: ratatui::layout::Position = (mouse.column, mouse.row).into();
        let on_cross = self.cross.get().is_some_and(|c| c.contains(at));
        // The scrollbar takes its presses and the drag of its thumb.
        let mut drag = self.bar_drag.get();
        let moved = drag.on_mouse(
            self.bar.get(),
            mouse,
            usize::from(self.line_count()),
            usize::from(self.scroll),
        );
        self.bar_drag.set(drag);
        if let Some(top) = moved {
            let max = self.line_count().saturating_sub(self.inner.get().1);
            self.scroll = top.min(usize::from(max)) as u16;
            return PopupMouse::Nothing;
        }
        match mouse.kind {
            MouseEventKind::ScrollUp => self.scroll_by(-3),
            MouseEventKind::ScrollDown => self.scroll_by(3),
            MouseEventKind::Moved | MouseEventKind::Drag(_) => self.cross_hover.set(on_cross),
            MouseEventKind::Down(MouseButton::Left)
                if on_cross || !self.frame.get().contains(at) =>
            {
                return PopupMouse::Close;
            }
            _ => {}
        }
        PopupMouse::Nothing
    }

    fn set_result(&mut self, result: Result<String, String>) {
        self.scroll = 0;
        self.body = match result {
            Ok(text) => PopupBody::Text(text.trim().to_string()),
            Err(why) => PopupBody::Error(why),
        };
    }

    pub(crate) fn is_working(&self) -> bool {
        matches!(self.body, PopupBody::Working(_))
    }

    fn text(&self) -> &str {
        match &self.body {
            PopupBody::Working(t) | PopupBody::Text(t) | PopupBody::Error(t) => t,
        }
    }

    /// The wrapped line count at the last drawn width (a lower bound on
    /// what `Paragraph` will wrap to; used only to clamp the scroll).
    fn line_count(&self) -> u16 {
        let width = self.inner.get().0.max(1) as usize;
        self.text()
            .lines()
            .map(|l| l.chars().count().div_ceil(width).max(1))
            .sum::<usize>()
            .min(u16::MAX as usize) as u16
    }

    pub(crate) fn scroll_by(&mut self, delta: i32) {
        let max = self.line_count().saturating_sub(self.inner.get().1);
        self.scroll = (self.scroll as i32 + delta).clamp(0, max as i32) as u16;
    }

    pub(crate) fn on_key(&mut self, key: &crossterm::event::KeyEvent) -> PopupKey {
        use crossterm::event::KeyCode;
        let page = self.inner.get().1.max(1) as i32;
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter => PopupKey::Close,
            KeyCode::Up | KeyCode::Char('k') => {
                self.scroll_by(-1);
                PopupKey::Nothing
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.scroll_by(1);
                PopupKey::Nothing
            }
            KeyCode::PageUp => {
                self.scroll_by(-page);
                PopupKey::Nothing
            }
            KeyCode::PageDown | KeyCode::Char(' ') => {
                self.scroll_by(page);
                PopupKey::Nothing
            }
            KeyCode::Home => {
                self.scroll = 0;
                PopupKey::Nothing
            }
            KeyCode::End => {
                self.scroll_by(i32::MAX / 2);
                PopupKey::Nothing
            }
            _ if crate::keymap::current().is(crate::keymap::Action::AiPopupCopy, key) => {
                match &self.body {
                    PopupBody::Text(text) => PopupKey::Copy(text.clone()),
                    _ => PopupKey::Nothing,
                }
            }
            _ => PopupKey::Nothing,
        }
    }

    pub(crate) fn render(&self, frame: &mut Frame, area: Rect) {
        let rect = WIN.place(area, crate::centered_rect(70, 80, area));
        frame.render_widget(Clear, rect);
        // Only a finished answer has a hint (copy); Esc is not advertised.
        let hint = match &self.body {
            PopupBody::Text(_) => Some(format!(
                " {} ",
                msg::hint_text(crate::dialog_kit::key_label(
                    crate::keymap::Action::AiPopupCopy
                ))
            )),
            PopupBody::Working(_) | PopupBody::Error(_) => None,
        };
        let mut block = Block::default()
            .title(format!(" \u{2726} {} ", self.title))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(th().ui.accent));
        if let Some(hint) = hint {
            block = block.title_bottom(Line::from(hint).right_aligned());
        }
        let inner = block.inner(rect);
        self.frame.set(rect);
        self.inner.set((inner.width, inner.height));
        let (text, style) = match &self.body {
            PopupBody::Working(t) => (t.clone(), Style::default().fg(th().status.warning)),
            PopupBody::Text(t) => (t.clone(), Style::default().fg(th().ui.text)),
            PopupBody::Error(t) => (
                msg::error_prefix(t),
                Style::default().fg(th().status.danger),
            ),
        };
        let lines: Vec<Line> = text
            .lines()
            .map(|l| Line::from(Span::styled(l.to_string(), style)))
            .collect();
        frame.render_widget(
            Paragraph::new(lines)
                .wrap(Wrap { trim: false })
                .scroll((self.scroll, 0))
                .block(block),
            rect,
        );
        self.cross.set(crate::chrome::close_cross::paint(
            frame.buffer_mut(),
            rect,
            self.cross_hover.get(),
        ));
        WIN.decorate(frame.buffer_mut(), rect, self.cross.get(), true);
        // The bar, in the right border, when the text outgrows the box.
        self.bar.set(crate::scrollbar::Track::vertical(
            rect,
            usize::from(self.line_count()),
            usize::from(self.scroll),
        ));
        if let Some(track) = self.bar.get() {
            crate::scrollbar::render(frame, track, true, false);
        }
    }
}

/// The answer popup moves like any dialog.
const WIN: crate::dialog_kit::Win = crate::dialog_kit::Win::movable("ai_popup");

// ---------------------------------------------------------------------
// The worker
// ---------------------------------------------------------------------

/// The worker body: everything that may block.
async fn execute(
    config: AiConfig,
    session: String,
    repo: RepoRef,
    request: AiRequest,
) -> Result<AiAnswer, String> {
    match request {
        AiRequest::CommitMessage => commit_message(&config, &session, &repo)
            .await
            .map(AiAnswer::CommitMessage),
        AiRequest::Explain(target) => explain(&config, &session, &repo, &target)
            .await
            .map(AiAnswer::Explanation),
        AiRequest::StashMessage => stash_message(&config, &session, &repo)
            .await
            .map(AiAnswer::StashMessage),
        AiRequest::Compose => compose(&config, &session, &repo).await.map(AiAnswer::Plan),
        AiRequest::SplitCommit { oid, .. } => split_commit(&config, &session, &repo, oid).await,
        AiRequest::ResolveConflict {
            context,
            oversize_ok,
            ..
        } => resolve_conflict(&config, &session, &repo, &context, oversize_ok)
            .await
            .map(AiAnswer::Resolution),
        AiRequest::RewriteScript {
            instruction,
            sample,
        } => rewrite_script(&config, &session, &repo, &instruction, &sample)
            .await
            .map(|o| AiAnswer::RewriteScript(Box::new(o))),
        AiRequest::RewriteRules {
            description,
            context,
        } => rewrite_rules(&config, &session, &repo, &description, &context)
            .await
            .map(|p| AiAnswer::RewriteRules(Box::new(p))),
        AiRequest::WorktreeName {
            description,
            branches,
        } => worktree_name(&config, &session, &repo, &description, &branches)
            .await
            .map(AiAnswer::WorktreeName),
        AiRequest::PullRequest { source, target } => {
            pull_request(&config, &session, &repo, &source, &target)
                .await
                .map(AiAnswer::PullRequest)
        }
    }
}

/// The repository's AI state, from `.gitpoulpi/repo-state.toml`.
fn repo_preferences(state_dir: &str, repo: &RepoRef) -> gitpoulpi_session::RepoPreferences {
    let path = gitpoulpi_session::RepoState::path_for(&repo.root, state_dir);
    gitpoulpi_session::RepoState::load(&path)
        .map(|s| s.preferences)
        .unwrap_or_default()
}

fn prepare(
    config: &AiConfig,
    session: &str,
    repo: &RepoRef,
    action: AiAction,
) -> Result<Job, String> {
    prepare_with(
        config,
        session,
        repo,
        action,
        &CredentialHelperTokens::new(),
    )
}

/// [`prepare`] with the store the key is read from (the credential helper
/// is a subprocess: a caller on the input thread passes an empty one).
fn prepare_with(
    config: &AiConfig,
    session: &str,
    repo: &RepoRef,
    action: AiAction,
    tokens: &dyn gitpoulpi_secrets::TokenStore,
) -> Result<Job, String> {
    let state_dir = gitpoulpi_config::Settings::resolve_ambient()
        .map(|s| s.config().session.per_repo_state_dir.clone())
        .unwrap_or_else(|_| gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION.to_string());
    let prefs = repo_preferences(&state_dir, repo);
    let over = prefs.ai_override(action.key(), config);
    Job::prepare(config, action, over.as_ref(), tokens).map(|job| {
        let mut context = gitpoulpi_ai::vars::PromptContext::from_repo(&repo.root);
        context.uilang = ui_language_english().to_string();
        with_privacy(
            with_ui_language(
                job.with_repo_instructions(prefs.ai_instructions)
                    .with_workspace(crate::workspace_ui::store::ai_for(session, &repo.root))
                    .with_context(context),
            ),
            action,
        )
    })
}

/// Actions whose request the generic "What is sent" dialog shows. The two
/// history-rewrite dialogs show their own before they ask.
fn previewed(action: AiAction) -> bool {
    // The worktree naming sends what the user typed in its own dialog and
    // branch names: nothing from the changes.
    !matches!(
        action,
        AiAction::RewriteScript | AiAction::RewriteRules | AiAction::WorktreeName
    )
}

/// The byte budget for a diff of `bytes`: `ai.limits.diff_bytes`, or no limit
/// when the user chose "Send all". Over the limit the worker waits here for
/// the answer of "Over the AI limit" (`ai_limit_dialog`); Cancel ends the
/// request as a cancel, not as a failure.
async fn diff_budget(job: &Job, bytes: usize) -> Result<usize, String> {
    job.diff_budget(gitpoulpi_ai::Part::Diff, bytes)
        .await
        .map_err(|e| e.to_string())
}

/// The job with the caps of `[ai.limits]` as the file says now, and the
/// dialog that asks before an input over one is cut.
fn with_limits(job: Job) -> Job {
    let mut job = job;
    if let Ok(settings) = gitpoulpi_config::Settings::resolve_ambient() {
        job.limit.limits = gitpoulpi_ai::Limits::from_config(&settings.config().ai.limits);
    }
    job.with_limit_gate(crate::ai_limit_dialog::Gate::shared())
}

/// The job with the privacy settings as the files say now (a window's copy of
/// the config can be older), and the "What is sent" dialog as its gate when
/// the user wants to see this action's request first.
fn with_privacy(job: Job, action: AiAction) -> Job {
    let job = with_limits(job);
    let Ok(settings) = gitpoulpi_config::Settings::resolve_ambient() else {
        return job;
    };
    let config = settings.config();
    let job = job.with_privacy(&config.ai.privacy, &config.privacy.sensitive_patterns);
    if previewed(action) && config.ai.sent_preview.shows(action) {
        let gate = crate::ai_sent_dialog::Gate::shared(
            action,
            job.profile.display_name().to_string(),
            action_label(action).to_string(),
        );
        job.with_send_gate(gate)
    } else {
        job
    }
}

/// One request on the worker's own daemon connection.
async fn call_daemon<T: serde::de::DeserializeOwned>(
    client: &mut Client,
    session: &str,
    repo: &RepoRef,
    id: u64,
    command: OperationKind,
    args: serde_json::Value,
) -> Result<T, String> {
    let request = Request {
        id,
        session: session.to_string(),
        repo: Some(repo.id.clone()),
        command,
        args,
    };
    let response = client.call(&request).await.map_err(msg::daemon_lost)?;
    match response.result {
        Ok(value) => serde_json::from_value(value)
            .map_err(|e| msg::daemon_malformed(format!("{command:?}"), e)),
        Err(e) => Err(e.message),
    }
}

async fn connect(session: &str) -> Result<Client, String> {
    let endpoint = crate::daemon_link::ensure_daemon(session).await?;
    Client::connect(&endpoint)
        .await
        .map_err(msg::daemon_unreachable)
}

/// The commits and the diff a pull request from `source` into `target`
/// would carry, and the model's title and description for them.
async fn pull_request(
    config: &AiConfig,
    session: &str,
    repo: &RepoRef,
    source: &str,
    target: &str,
) -> Result<parse::PullRequestText, String> {
    pull_request_hinted(config, session, repo, source, target, "", None).await
}

/// [`pull_request`] with what the author typed in the composer's "Write with
/// AI" field and, when improving, their own draft of the description.
pub(crate) async fn pull_request_hinted(
    config: &AiConfig,
    session: &str,
    repo: &RepoRef,
    source: &str,
    target: &str,
    hint: &str,
    draft: Option<&str>,
) -> Result<parse::PullRequestText, String> {
    let job = prepare(config, session, repo, AiAction::PullRequestDescription)?;
    let mut client = connect(session).await?;
    let mut id = 0u64;
    macro_rules! ask {
        ($t:ty, $kind:expr, $args:expr) => {{
            id += 1;
            call_daemon::<$t>(&mut client, session, repo, id, $kind, $args).await
        }};
    }
    // Full ref names: the log's `start_ref` is a ref, not a revision.
    let source_ref = format!("refs/heads/{source}");
    let source_tip: Oid = ask!(
        Oid,
        OperationKind::Resolve,
        serde_json::json!({ "rev": source_ref })
    )
    .map_err(|_| msg::branch_missing(source))?;
    // The remote-tracking branch is what the pull request merges into;
    // the local one is the fallback.
    let remote_ref = format!("refs/remotes/origin/{target}");
    let target_start = match ask!(
        Oid,
        OperationKind::Resolve,
        serde_json::json!({ "rev": remote_ref })
    ) {
        Ok(_) => remote_ref,
        Err(_) => format!("refs/heads/{target}"),
    };
    let log_of = |start: &str, limit: u32| LogOptions {
        limit: Some(limit),
        start_ref: Some(start.to_string()),
        ..LogOptions::default()
    };
    let source_log: CommitGraph = ask!(
        CommitGraph,
        OperationKind::Log,
        serde_json::json!({ "opts": log_of(&source_ref, 60) })
    )?;
    let target_log: CommitGraph = ask!(
        CommitGraph,
        OperationKind::Log,
        serde_json::json!({ "opts": log_of(&target_start, 3000) })
    )?;
    let target_ids = target_log.nodes.iter().map(|n| n.id).collect();
    let (subjects, base) = gitpoulpi_ai::assist::branch_range(&source_log.nodes, &target_ids);
    let base = match base {
        Some(base) => base,
        // The walk never met the target: fall back to the target's tip.
        None => ask!(
            Oid,
            OperationKind::Resolve,
            serde_json::json!({ "rev": target_start })
        )?,
    };
    if subjects.is_empty() {
        return Err(msg::no_new_commits(source, target));
    }
    let diffs: Vec<FileDiff> = ask!(
        Vec<FileDiff>,
        OperationKind::Diff,
        serde_json::json!({ "target": DiffTarget::CommitRange(base, source_tip) })
    )?;
    let diff = render_diff(&diffs);
    let budget = diff_budget(&job, diff.len()).await?;
    let prompt = prompts::pull_request_with_hint(&subjects, &diff, budget, hint, draft);
    let answer = job.run(&prompt).await.map_err(|e| e.to_string())?;
    parse::pull_request(&answer).map_err(|e| e.to_string())
}

/// One diff per target, over the worker's own daemon connection.
async fn fetch_diffs(
    session: &str,
    repo: &RepoRef,
    targets: Vec<DiffTarget>,
) -> Result<Vec<Vec<FileDiff>>, String> {
    let endpoint = crate::daemon_link::ensure_daemon(session).await?;
    let mut client = Client::connect(&endpoint)
        .await
        .map_err(msg::daemon_unreachable)?;
    let mut out = Vec::new();
    for (i, target) in targets.into_iter().enumerate() {
        let request = Request {
            id: i as u64 + 1,
            session: session.to_string(),
            repo: Some(repo.id.clone()),
            command: OperationKind::Diff,
            args: serde_json::json!({ "target": target }),
        };
        let response = client.call(&request).await.map_err(msg::daemon_lost)?;
        out.push(match response.result {
            Ok(value) => serde_json::from_value(value).map_err(msg::malformed_diff)?,
            Err(e) => return Err(e.message),
        });
    }
    Ok(out)
}

async fn commit_message(
    config: &AiConfig,
    session: &str,
    repo: &RepoRef,
) -> Result<parse::CommitMessage, String> {
    let job = prepare(config, session, repo, AiAction::CommitMessage)?;
    let mut diffs = fetch_diffs(
        session,
        repo,
        vec![DiffTarget::IndexVsHead, DiffTarget::WorkingTreeVsIndex],
    )
    .await?;
    let unstaged = diffs.pop().unwrap_or_default();
    let staged = diffs.pop().unwrap_or_default();
    // What is staged, or, when nothing is, everything that changed.
    let diffs = if staged.is_empty() { unstaged } else { staged };
    if diffs.is_empty() {
        return Err(msg::NO_CHANGES_DESCRIBE.get().into());
    }
    let diff = render_diff(&diffs);
    let budget = diff_budget(&job, diff.len()).await?;
    let prompt = prompts::commit_message(&diff, budget);
    let answer = job.run(&prompt).await.map_err(|e| e.to_string())?;
    parse::commit_message(&answer).map_err(|e| e.to_string())
}

/// Asks for a commit plan and validates it against the real change set;
/// an invalid answer is sent back once with what was wrong with it.
async fn compose(
    config: &AiConfig,
    session: &str,
    repo: &RepoRef,
) -> Result<Vec<(String, Vec<String>)>, String> {
    let job = prepare(config, session, repo, AiAction::CommitComposer)?;
    let mut client = connect(session).await?;
    let tree: WorkingTree = call_daemon(
        &mut client,
        session,
        repo,
        1,
        OperationKind::Status,
        serde_json::json!({}),
    )
    .await?;
    let changed: Vec<String> = tree
        .files
        .iter()
        .filter(|f| f.index.is_some() || f.worktree.is_some())
        .map(|f| f.path.to_string_lossy().replace('\\', "/"))
        .collect();
    if changed.is_empty() {
        return Err(msg::NO_CHANGES_SPLIT.get().into());
    }
    if tree.files.iter().any(|f| {
        matches!(f.index, Some(FileStatus::Conflicted))
            || matches!(f.worktree, Some(FileStatus::Conflicted))
    }) {
        return Err(msg::RESOLVE_FIRST.get().into());
    }
    drop(client);
    let mut diff = wip_diff_text(session, repo).await?;
    // Paths with no textual diff (untracked, binary, mode-only) still
    // have to be assigned to a commit.
    for path in &changed {
        if !diff_names(&diff, path) {
            diff.push_str(&no_textual_diff(path));
        }
    }
    let budget = diff_budget(&job, diff.len()).await?;
    let prompt = prompts::commit_composer(&changed, &diff, budget);
    let answer = job.run(&prompt).await.map_err(|e| e.to_string())?;
    let plan = match parse::composer_plan(&answer, &changed) {
        Ok(plan) => plan,
        Err(first) => {
            let retry = retry_prompt(&prompt, &first.to_string());
            let answer = job.run(&retry).await.map_err(|e| e.to_string())?;
            parse::composer_plan(&answer, &changed).map_err(msg::plan_invalid_twice)?
        }
    };
    Ok(plan.into_iter().map(|c| (c.message, c.files)).collect())
}

/// Reads the commit's numbered changes and asks for a split plan. The
/// changes are returned even when the plan is not (`Done(Ok)` with an
/// `Err` plan), so the editor can still open for a manual split; only a
/// failure to read them is an `Err`.
async fn split_commit(
    config: &AiConfig,
    session: &str,
    repo: &RepoRef,
    commit: Oid,
) -> Result<AiAnswer, String> {
    let mut client = connect(session).await?;
    let source: SplitSource = call_daemon(
        &mut client,
        session,
        repo,
        1,
        OperationKind::SplitSource,
        serde_json::json!({ "commit": commit }),
    )
    .await?;
    drop(client);
    let plan = match prepare(config, session, repo, AiAction::SplitCommit) {
        Ok(job) => job.plan_split(&source).await.map_err(|e| e.to_string()),
        Err(why) => Err(why),
    };
    Ok(AiAnswer::Split {
        source: Box::new(source),
        plan,
    })
}

/// Compiles a script through the worker's own daemon connection: the check
/// the retry of a ✦ script request goes by.
struct DaemonChecker {
    client: tokio::sync::Mutex<(Client, u64)>,
    session: String,
    repo: RepoRef,
}

#[async_trait::async_trait]
impl gitpoulpi_ai::assist::ScriptChecker for DaemonChecker {
    async fn check(&self, script: &str) -> Result<Vec<String>, String> {
        use gitpoulpi_core::domain::rewrite::script::ScriptCheck;
        use gitpoulpi_core::domain::rewrite::{ScriptLanguage, ScriptRef, ScriptSource};
        let args = gitpoulpi_ipc::rewrite::ScriptCheckArgs {
            script: ScriptRef {
                language: ScriptLanguage::Lua,
                source: ScriptSource::Inline {
                    text: script.to_string(),
                },
                params: Default::default(),
            },
            vouched: Vec::new(),
        };
        let mut guard = self.client.lock().await;
        guard.1 += 1;
        let id = guard.1;
        let check: ScriptCheck = call_daemon(
            &mut guard.0,
            &self.session,
            &self.repo,
            id,
            OperationKind::RewriteScriptCheck,
            serde_json::to_value(args).map_err(|e| e.to_string())?,
        )
        .await?;
        Ok(check.problems)
    }
}

/// The ✦ history rewrite script: the model writes it, the daemon checks it,
/// a compile error goes back once. Nothing is applied.
async fn rewrite_script(
    config: &AiConfig,
    session: &str,
    repo: &RepoRef,
    instruction: &str,
    sample: &[SampleRow],
) -> Result<RewriteScriptOutcome, String> {
    let job = prepare(config, session, repo, AiAction::RewriteScript)?;
    let checker = DaemonChecker {
        client: tokio::sync::Mutex::new((connect(session).await?, 0)),
        session: session.to_string(),
        repo: repo.clone(),
    };
    let request = rewrite_request(instruction, sample);
    job.write_rewrite_script(&request, &checker)
        .await
        .map_err(|e| e.to_string())
}

fn rewrite_request(
    instruction: &str,
    sample: &[SampleRow],
) -> gitpoulpi_ai::assist::RewriteScriptRequest {
    gitpoulpi_ai::assist::RewriteScriptRequest {
        instruction: instruction.to_string(),
        sample: sample
            .iter()
            .map(|r| prompts::SampleCommit {
                author_name: r.author_name.clone(),
                author_email: r.author_email.clone(),
                timestamp: r.timestamp,
                subject: r.subject.clone(),
            })
            .collect(),
    }
}

/// What a ✦ script request will send, for the dialog that shows it before
/// anything leaves the machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PromptPreview {
    /// The system prompt: the fixed contract, then the user's instructions.
    pub system: String,
    /// The profile's name.
    pub via: String,
}

/// The system prompt a script request would send and the profile it would go
/// to (no key is read: nothing is sent).
pub(crate) fn rewrite_prompt_preview(
    session: &str,
    repo: &RepoRef,
) -> Result<PromptPreview, String> {
    let config = gitpoulpi_config::Settings::resolve_ambient()
        .map(|s| s.config().ai.clone())
        .unwrap_or_default();
    let job = prepare_with(
        &config,
        session,
        repo,
        AiAction::RewriteScript,
        &gitpoulpi_secrets::InMemoryTokens::default(),
    )?;
    Ok(PromptPreview {
        system: job.system_prompt(),
        via: job.profile.display_name().to_string(),
    })
}

/// The user message of a script request: exactly what the model is sent
/// besides [`rewrite_prompt_preview`]'s system prompt.
pub(crate) fn rewrite_user_message(instruction: &str, sample: &[SampleRow]) -> String {
    let request = rewrite_request(instruction, sample);
    prompts::rewrite_script(&request.instruction, &request.sample)
}

/// The ✦ "Create Rules with AI": the model proposes rules, a scope and
/// settings; the answer is parsed into data and nothing else happens (no
/// daemon call, no preview, no apply).
async fn rewrite_rules(
    config: &AiConfig,
    session: &str,
    repo: &RepoRef,
    description: &str,
    context: &RulesCtx,
) -> Result<RulesProposal, String> {
    let job = prepare(config, session, repo, AiAction::RewriteRules)?;
    let request = rules_request(description, context);
    let outcome = job
        .create_rewrite_rules(&request)
        .await
        .map_err(|e| e.to_string())?;
    let a = outcome.answer;
    Ok(RulesProposal {
        scope: a.scope,
        settings: crate::rewrite_view::ProposalSettings {
            include_tags: a.settings.include_tags,
            committer: a.settings.committer,
            prune_empty: a.settings.prune_empty,
            update_message_hashes: a.settings.update_message_hashes,
            keep_backup_refs: a.settings.keep_backup_refs,
        },
        rules: a.rules,
        problems: a
            .problems
            .into_iter()
            .map(|p| crate::rewrite_view::ProposalProblem {
                position: p.position,
                what: p.what,
                reason: p.reason,
            })
            .collect(),
        notes: a.notes,
    })
}

fn rules_request(
    description: &str,
    context: &RulesCtx,
) -> gitpoulpi_ai::assist::RewriteRulesRequest {
    gitpoulpi_ai::assist::RewriteRulesRequest {
        description: description.to_string(),
        context: prompts::RulesContext {
            scope: context.scope.clone(),
            current_branch: context.current_branch.clone(),
            refs: context.refs.clone(),
            authors: context.authors.clone(),
        },
    }
}

/// The system prompt a rules request would send and the profile it would go
/// to (no key is read: nothing is sent).
pub(crate) fn rewrite_rules_prompt_preview(
    session: &str,
    repo: &RepoRef,
) -> Result<PromptPreview, String> {
    let config = gitpoulpi_config::Settings::resolve_ambient()
        .map(|s| s.config().ai.clone())
        .unwrap_or_default();
    let job = prepare_with(
        &config,
        session,
        repo,
        AiAction::RewriteRules,
        &gitpoulpi_secrets::InMemoryTokens::default(),
    )?;
    Ok(PromptPreview {
        system: job.system_prompt(),
        via: job.profile.display_name().to_string(),
    })
}

/// The user message of a rules request: exactly what the model is sent
/// besides [`rewrite_rules_prompt_preview`]'s system prompt.
pub(crate) fn rewrite_rules_user_message(description: &str, context: &RulesCtx) -> String {
    let request = rules_request(description, context);
    prompts::rewrite_rules(&request.description, &request.context)
}

/// Whether the rendered diff has a section for `path`.
fn diff_names(diff: &str, path: &str) -> bool {
    diff.contains(&format!("diff -- {path} ")) // i18n-english
}

/// A line of the model's input for a path with no textual diff.
fn no_textual_diff(path: &str) -> String {
    format!("(no textual diff available for {path})\n") // i18n-english
}

/// The composer's second attempt: the prompt plus what was wrong. Sent to
/// the model, so English.
fn retry_prompt(prompt: &str, first: &str) -> String {
    format!(
        "{prompt}\n\nYour previous answer was rejected: {first}. Answer again with a valid JSON array only." // i18n-english
    )
}

/// Staged and unstaged changes, rendered one after the other.
async fn wip_diff_text(session: &str, repo: &RepoRef) -> Result<String, String> {
    let mut diffs = fetch_diffs(
        session,
        repo,
        vec![DiffTarget::IndexVsHead, DiffTarget::WorkingTreeVsIndex],
    )
    .await?;
    let unstaged = render_diff(&diffs.pop().unwrap_or_default());
    let staged = render_diff(&diffs.pop().unwrap_or_default());
    let mut text = String::new();
    if !staged.is_empty() {
        text.push_str("== Staged changes ==\n"); // i18n-english: sent to the model
        text.push_str(&staged);
    }
    if !unstaged.is_empty() {
        text.push_str("== Unstaged changes ==\n"); // i18n-english: sent to the model
        text.push_str(&unstaged);
    }
    Ok(text)
}

/// One conflict's resolution: the context came from the open editor, so
/// no daemon is needed.
async fn resolve_conflict(
    config: &AiConfig,
    session: &str,
    repo: &RepoRef,
    context: &prompts::ConflictContext,
    oversize_ok: bool,
) -> Result<Vec<String>, String> {
    let job = prepare(config, session, repo, AiAction::ConflictResolve)?;
    // Past the warning: the privacy pipeline (`prepare` hooked it in) still
    // cleans the request exactly like any other.
    let job = if oversize_ok {
        job.without_conflict_limit()
    } else {
        job
    };
    job.resolve_conflict(context)
        .await
        .map_err(|e| e.to_string())
}

async fn stash_message(config: &AiConfig, session: &str, repo: &RepoRef) -> Result<String, String> {
    let job = prepare(config, session, repo, AiAction::StashMessage)?;
    let diff = wip_diff_text(session, repo).await?;
    if diff.trim().is_empty() {
        return Err(msg::NO_CHANGES_STASH.get().into());
    }
    let budget = diff_budget(&job, diff.len()).await?;
    let prompt = prompts::stash_message(&diff, budget);
    let answer = job.run(&prompt).await.map_err(|e| e.to_string())?;
    parse::stash_message(&answer).map_err(|e| e.to_string())
}

async fn worktree_name(
    config: &AiConfig,
    session: &str,
    repo: &RepoRef,
    description: &str,
    branches: &[String],
) -> Result<AiNames, String> {
    let job = prepare(config, session, repo, AiAction::WorktreeName)?;
    let names = gitpoulpi_ai::worktree_name::suggest(&job, description, branches)
        .await
        .map_err(|e| e.to_string())?;
    Ok(AiNames {
        name: names.name,
        branch: names.branch,
    })
}

async fn explain(
    config: &AiConfig,
    session: &str,
    repo: &RepoRef,
    target: &ExplainTarget,
) -> Result<String, String> {
    let mut job = prepare(config, session, repo, AiAction::ExplainChanges)?;
    if let ExplainTarget::Commit { message, short, .. } = target {
        // `${ref}` / `${commit}` / `${commit_subject}` name the commit asked about.
        job.context.commit = short.clone();
        job.context.commit_subject = message.lines().next().unwrap_or("").to_string();
    }
    let (subject, message, diff) = match target {
        ExplainTarget::Commit {
            oid,
            message,
            short,
        } => {
            let mut diffs =
                fetch_diffs(session, repo, vec![DiffTarget::CommitVsParent(*oid)]).await?;
            (
                format!("the commit {short}"), // i18n-english: sent to the model
                Some(message.clone()),
                render_diff(&diffs.pop().unwrap_or_default()),
            )
        }
        ExplainTarget::Wip => (
            "the uncommitted working changes".to_string(), // i18n-english: sent to the model
            None,
            wip_diff_text(session, repo).await?,
        ),
    };
    if diff.trim().is_empty() {
        return Err(msg::NOTHING_TO_EXPLAIN.get().into());
    }
    let budget = diff_budget(&job, diff.len()).await?;
    let prompt = prompts::explain_changes(&subject, message.as_deref(), &diff, budget);
    job.run(&prompt).await.map_err(|e| e.to_string())
}

/// [`prepare`] for the repository rooted at `root`: the review view's
/// suggestions resolve their job the way every other action does, with the
/// repository's and the workspace's instructions.
pub(crate) fn prepare_for_root(
    config: &AiConfig,
    session: &str,
    root: &std::path::Path,
    action: AiAction,
) -> Result<Job, String> {
    prepare(config, session, &RepoRef::from_root(root), action)
}

/// [`prepare_for_root`] for a request of a repository that is not cloned:
/// the global profile and instructions only (there is no repository to read
/// a preference or a work tree from).
pub(crate) fn prepare_remote(config: &AiConfig, action: AiAction) -> Result<Job, String> {
    let job = Job::prepare(config, action, None, &CredentialHelperTokens::new())?;
    let context = gitpoulpi_ai::vars::PromptContext {
        uilang: ui_language_english().to_string(),
        ..Default::default()
    };
    Ok(with_privacy(
        with_ui_language(job.with_context(context)),
        action,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ui_off() -> AiUi {
        AiUi {
            config: AiConfig::default(),
            run: None,
            popup: None,
            owner: None,
        }
    }

    fn repo() -> RepoRef {
        RepoRef {
            id: gitpoulpi_core::domain::RepoId("r".into()),
            root: std::path::PathBuf::from("."),
        }
    }

    #[test]
    fn with_ai_off_a_request_is_refused_and_nothing_runs() {
        let mut ui = ui_off();
        assert!(!ui.enabled());
        let err = ui
            .start("s", &repo(), AiRequest::CommitMessage)
            .unwrap_err();
        assert!(err.contains("turned off"), "{err}");
        assert!(!ui.running());
        // ...and no popup opens for an explain either.
        let mut message = None;
        ui.explain_into("s", &repo(), ExplainTarget::Wip, &mut message);
        assert!(!ui.popup_open());
        assert!(message.unwrap().contains("turned off"));
    }

    fn rendered(popup: &AiPopup, width: u16, height: u16) -> String {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|f| popup.render(f, f.area())).unwrap();
        let buffer = terminal.backend().buffer().clone();
        (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| buffer[(x, y)].symbol().to_string())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn the_french_popup_shows_its_title_hint_and_error_inside_the_box() {
        let _fr = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::Fr);
        let mut popup =
            AiPopup::working(msg::explain_commit_title("abc1234"), "En cours".to_string());
        let text = rendered(&popup, 80, 20);
        assert!(text.contains("Expliquer le commit abc1234"), "{text}");
        assert!(!text.contains("Esc"), "no Esc hint\n{text}");
        assert!(text.contains('\u{2715}'), "a close cross\n{text}");
        popup.set_result(Ok("Un texte.".into()));
        let text = rendered(&popup, 80, 20);
        assert!(text.contains("défiler"), "{text}");
        assert!(text.contains("copier"), "{text}");
        assert!(!text.contains("Esc"), "{text}");
        popup.set_result(Err(msg::NOTHING_TO_EXPLAIN.get().to_string()));
        let text = rendered(&popup, 80, 20);
        assert!(
            text.contains("IA\u{a0}: il n'y a rien à expliquer"),
            "{text}"
        );
        assert!(!text.contains("Esc"), "{text}");
    }

    #[test]
    fn the_french_progress_line_and_refusals_are_translated() {
        let _fr = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::Fr);
        let mut ui = ui_off();
        let err = ui
            .start("s", &repo(), AiRequest::CommitMessage)
            .unwrap_err();
        assert_eq!(err, "L'IA est désactivée (Paramètres > IA)");
        assert!(msg::working("Message de commit via X", 3).starts_with("en cours"));
    }

    fn job_for(action: AiAction) -> Job {
        let config = AiConfig {
            enabled: true,
            ..AiConfig::default()
        };
        with_ui_language(
            Job::prepare(
                &config,
                action,
                None,
                &gitpoulpi_secrets::InMemoryTokens::default(),
            )
            .unwrap(),
        )
    }

    #[test]
    fn explain_answers_in_the_ui_language_and_other_prompts_ignore_it() {
        let english = job_for(AiAction::ExplainChanges).system_prompt();
        assert!(!english.contains("Answer in"), "{english}");
        let _fr = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::Fr);
        let french = job_for(AiAction::ExplainChanges).system_prompt();
        assert!(french.ends_with("\n\nAnswer in French."), "{french}");
        // What lands in the repository follows the user's instructions.
        for action in [
            AiAction::CommitMessage,
            AiAction::StashMessage,
            AiAction::PullRequestDescription,
            AiAction::CommitComposer,
        ] {
            let prompt = job_for(action).system_prompt();
            assert!(!prompt.contains("Answer in"), "{action:?}: {prompt}");
            assert_eq!(prompt, prompts::system_prompt(action, "", ""));
        }
    }

    #[test]
    fn the_popup_survives_an_inside_click_and_closes_on_the_cross_or_outside() {
        use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
        let click = |x: u16, y: u16| MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: x,
            row: y,
            modifiers: KeyModifiers::NONE,
        };
        let mut popup = AiPopup::working("t".into(), "working".into());
        let text = rendered(&popup, 100, 30);
        assert!(text.contains('\u{2715}'), "{text}");
        assert!(!text.contains("Esc"), "{text}");
        let frame = popup.frame.get();
        assert_eq!(
            popup.on_mouse(click(frame.x + 3, frame.y + 3)),
            PopupMouse::Nothing,
            "an inside click does not close"
        );
        assert_eq!(popup.on_mouse(click(0, 0)), PopupMouse::Close);
        let cross = popup.cross.get().expect("a cross");
        assert_eq!(
            popup.on_mouse(click(cross.x + 1, cross.y)),
            PopupMouse::Close
        );
    }
    #[test]
    fn the_popup_scrolls_within_its_text_and_copies_only_an_answer() {
        let mut popup = AiPopup::working("t".into(), "working".into());
        assert!(popup.is_working());
        let esc = crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Esc);
        assert_eq!(popup.on_key(&esc), PopupKey::Close);
        let copy = crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char('c'));
        assert_eq!(popup.on_key(&copy), PopupKey::Nothing);
        popup.set_result(Ok("line\n".repeat(50)));
        popup.inner.set((40, 10));
        popup.scroll_by(1000);
        assert_eq!(popup.scroll, 40, "clamped to the last page");
        popup.scroll_by(-1000);
        assert_eq!(popup.scroll, 0);
        assert!(matches!(popup.on_key(&copy), PopupKey::Copy(_)));
    }

    #[test]
    fn the_popup_draws_a_bar_when_the_text_outgrows_it_and_the_thumb_drags() {
        use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
        let mouse = |kind, x: u16, y: u16| MouseEvent {
            kind,
            column: x,
            row: y,
            modifiers: KeyModifiers::NONE,
        };
        let mut popup = AiPopup::working("t".into(), "short".into());
        rendered(&popup, 100, 30);
        assert!(popup.bar.get().is_none(), "short text: no bar");
        popup.set_result(Ok("line\n".repeat(80)));
        rendered(&popup, 100, 30);
        let track = popup.bar.get().expect("long text: a bar");
        let x = track.rect.x;
        assert_eq!(
            popup.on_mouse(mouse(
                MouseEventKind::Down(MouseButton::Left),
                x,
                track.rect.y + 1
            )),
            PopupMouse::Nothing
        );
        popup.on_mouse(mouse(
            MouseEventKind::Drag(MouseButton::Left),
            x,
            track.rect.bottom() + 4,
        ));
        popup.on_mouse(mouse(
            MouseEventKind::Up(MouseButton::Left),
            x,
            track.rect.bottom() + 4,
        ));
        let max = popup.line_count() - popup.inner.get().1;
        assert_eq!(popup.scroll, max, "dragged to the end, popup still open");
        assert!(!popup.is_working());
        // The wheel brings it back up; a key too.
        popup.on_mouse(mouse(MouseEventKind::ScrollUp, x - 5, track.rect.y + 2));
        assert_eq!(popup.scroll, max - 3);
    }
}

#[cfg(test)]
mod finished_tests {
    use super::*;

    /// What a waiter reads comes from the outcome the window applied: `Ok`,
    /// the failure's text, or a cancel, once per kind of request.
    #[test]
    fn the_applied_outcome_is_what_a_waiter_takes_once() {
        assert_eq!(take_finished(AiAction::StashMessage), None);

        let done = AiOutcome::Done(
            AiRequest::StashMessage,
            Ok(AiAnswer::StashMessage("wip".into())),
        );
        record_finished(finished_of(&done));
        assert_eq!(take_finished(AiAction::StashMessage), Some(AiEnd::Ok));
        assert_eq!(take_finished(AiAction::StashMessage), None, "taken once");

        let failed = AiOutcome::Done(AiRequest::StashMessage, Err("no profile".into()));
        record_finished(finished_of(&failed));
        assert_eq!(
            take_finished(AiAction::StashMessage),
            Some(AiEnd::Failed("no profile".into()))
        );

        let canceled = AiOutcome::Canceled(AiRequest::StashMessage);
        record_finished(finished_of(&canceled));
        assert_eq!(take_finished(AiAction::StashMessage), Some(AiEnd::Canceled));
    }

    #[test]
    fn kinds_do_not_see_each_other() {
        let done = AiOutcome::Done(AiRequest::Compose, Ok(AiAnswer::Plan(Vec::new())));
        record_finished(finished_of(&done));
        assert_eq!(take_finished(AiAction::CommitMessage), None);
        assert_eq!(take_finished(AiAction::CommitComposer), Some(AiEnd::Ok));
    }
}
