//! The Flow dialog: Start (a form built from the topic type's pattern),
//! Finish, Sync, Promote and Set parent. Every kind goes through the same
//! phases:
//!
//! ```text
//! form  ->  preview (the step list, the hooks, "run hooks")  ->  running
//!       ->  done | stopped (a failure, or a conflict with Continue / Abort)
//! ```
//!
//! The dialog never touches the repository: previews and runs go through
//! [`FlowActions`] and come back on channels it polls every frame, so the
//! render thread never waits. A preview or step that takes longer than
//! 150 ms shows a spinner (the maintainer's in-flight rule).

use std::sync::Arc;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use gitpoulpi_core::workflow::{FinishMode, Token};
use gitpoulpi_i18n::msg::flow as msg;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use super::actions::{
    EventRx, FlowActions, FlowEvent, FlowOp, FlowPreview, FlowRequest, FlowStep, PreviewRx, Resume,
    StepKind, StoppedFlow, TrustRx,
};
use super::form::{next_version, today_iso, Bump, StartForm, Suggestions};
use super::hook_trust::{self, HookTrust, Pick};
use super::view::FlowView;
use crate::combo::{Combo, ComboOption, ComboOutcome};
use crate::dialog_kit::{self, ButtonState, CheckState};
use crate::text_edit::FieldEditor;
use crate::th;

/// A request in flight for longer than this shows a spinner.
const SPIN_AFTER: Duration = Duration::from_millis(150);

/// What the dialog is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FlowKind {
    /// Start a topic branch; `Some(i)` fixes the type (the i-th of the
    /// workflow), `None` lets the form's first row choose.
    Start(Option<usize>),
    Finish,
    Sync,
    Promote,
    SetParent,
    /// Continue or abort an action a conflict stopped (reopened after the
    /// conflicts were resolved in the usual tools).
    Resume,
}

/// What the window knows when it opens the dialog.
#[derive(Clone)]
pub(crate) struct FlowSeed {
    pub(crate) view: Arc<FlowView>,
    pub(crate) kind: FlowKind,
    /// The checked-out local branch.
    pub(crate) current: Option<String>,
    /// The branch selected in the sidebar, when it is a local branch.
    pub(crate) selected: Option<String>,
    pub(crate) branches: Vec<String>,
    pub(crate) tags: Vec<String>,
    /// `(reference, title)` of the linked tracker's issues.
    pub(crate) issues: Vec<(String, String)>,
    /// The active profile's name (`{user}`).
    pub(crate) user: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    TypeCombo,
    IssueCombo,
    Field(usize),
    StartCombo,
    TargetCombo,
    Publish,
    Children,
    Hooks,
    Primary,
    Secondary,
    /// Continue after a conflict (Primary is Continue, this is Abort).
    Abort,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StepState {
    Pending,
    Running(Instant),
    Done,
    Failed,
}

struct Run {
    steps: Vec<FlowStep>,
    state: Vec<StepState>,
    rx: EventRx,
}

/// How a polled run ended.
enum End {
    Finished,
    Failed(String, bool),
}

enum Phase {
    Form,
    Previewing {
        rx: PreviewRx,
        since: Instant,
    },
    Preview(FlowPreview),
    Running(Run),
    /// A step failed. With `conflict` Continue and Abort are offered.
    Stopped {
        run: Run,
        message: String,
        conflict: bool,
    },
    Done(Run),
}

/// The hook trust question over a preview, and the request that remembers
/// the answer while it is in flight.
struct TrustUi {
    pane: HookTrust,
    pending: Option<(TrustRx, Instant)>,
}

/// Whether the preview's hooks must be trusted before they may run.
fn untrusted(preview: &FlowPreview) -> bool {
    matches!(
        preview.trust,
        gitpoulpi_core::workflow::HooksTrust::Untrusted { .. }
    ) && !preview.hook_lines.is_empty()
}

/// What the dialog asks its owner for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FlowOutcome {
    Stay,
    Close,
}

pub(crate) struct FlowDialog {
    seed: FlowSeed,
    actions: Arc<dyn FlowActions>,
    kind: FlowKind,
    phase: Phase,
    focus: Focus,
    hover: Option<Focus>,
    message: Option<String>,
    /// The topic type of a Start (index into the workflow's topics).
    topic: usize,
    type_combo: Combo,
    issue_combo: Combo,
    start_combo: Combo,
    target_combo: Combo,
    form: StartForm,
    publish: bool,
    with_children: bool,
    run_hooks: bool,
    /// The question about the committed hooks, while it is open over the
    /// preview.
    trust: Option<TrustUi>,
    /// The run now going (or done) is an Abort.
    aborted: bool,
    /// Finish asks how (the type says `ask`): the form has the mode row.
    finish_asks: bool,
    /// The merge request the daemon's finish left to open (it pushed).
    merge_request: Option<gitpoulpi_core::workflow::MergeRequestSpec>,
    /// The branch Finish / Sync / Set parent act on.
    branch: String,
    hits: Vec<(Rect, Focus)>,
    close: dialog_kit::CloseHit,
    edit: FieldEditor<Focus>,
}

fn label_of(token: Token) -> &'static str {
    match token {
        Token::Slug => msg::TOKEN_SLUG.get(),
        Token::Issue => msg::TOKEN_ISSUE.get(),
        Token::IssueKey => msg::TOKEN_ISSUE_KEY.get(),
        Token::User => msg::TOKEN_USER.get(),
        Token::Date => msg::TOKEN_DATE.get(),
        Token::Version => msg::TOKEN_VERSION.get(),
        Token::N => msg::TOKEN_N.get(),
    }
}

fn hint_of(token: Token) -> &'static str {
    match token {
        Token::Slug => msg::HINT_SLUG.get(),
        Token::Issue => msg::HINT_ISSUE.get(),
        Token::IssueKey => msg::HINT_ISSUE_KEY.get(),
        Token::User => msg::HINT_USER.get(),
        Token::Date => msg::HINT_DATE.get(),
        Token::Version => msg::HINT_VERSION.get(),
        Token::N => msg::HINT_N.get(),
    }
}

impl dialog_kit::Dismiss for FlowDialog {
    fn is_dirty(&self) -> bool {
        // Typing into the topic's free fields is the work worth keeping;
        // the prefilled ones (user, date, version) are not.
        matches!(self.phase, Phase::Form)
            && self.form.fields.iter().any(|f| {
                matches!(
                    f.token,
                    Token::Slug | Token::Issue | Token::IssueKey | Token::N
                ) && !f.value.is_empty()
            })
    }

    fn close_hit(&self) -> &dialog_kit::CloseHit {
        &self.close
    }

    fn close_hit_mut(&mut self) -> &mut dialog_kit::CloseHit {
        &mut self.close
    }
}

impl FlowDialog {
    /// What `Ctrl+N` badges: every field, combobox and button; nothing while a list is open.
    pub(crate) fn nav_targets(&mut self) -> Vec<(Rect, (u16, u16))> {
        if self.open_combo().is_some() {
            return Vec::new();
        }
        dialog_kit::nav_cells(self.hits.iter().map(|(r, _)| *r))
    }

    pub(crate) fn new(seed: FlowSeed, actions: Arc<dyn FlowActions>) -> Self {
        let kind = seed.kind;
        // Finish and Sync act on the checked-out branch (their labels say
        // "current"); Set parent on the one selected in the sidebar.
        let branch = match kind {
            FlowKind::SetParent => seed.selected.clone().or_else(|| seed.current.clone()),
            _ => seed.current.clone().or_else(|| seed.selected.clone()),
        }
        .unwrap_or_default();
        let topic = match kind {
            FlowKind::Start(Some(i)) => i.min(seed.view.topics().len().saturating_sub(1)),
            _ => 0,
        };
        let mut dialog = Self {
            actions,
            kind,
            phase: Phase::Form,
            focus: Focus::Primary,
            hover: None,
            message: None,
            topic,
            type_combo: Combo::new(""),
            issue_combo: Combo::new(msg::PICK_ISSUE.get()).with_search(),
            start_combo: Combo::new("").with_custom().with_search(),
            target_combo: Combo::new("").with_custom().with_search(),
            form: StartForm::new("", &Suggestions::default()),
            publish: false,
            with_children: false,
            run_hooks: false,
            trust: None,
            aborted: false,
            finish_asks: false,
            merge_request: None,
            branch,
            hits: Vec::new(),
            close: Default::default(),
            edit: Default::default(),
            seed,
        };
        match kind {
            FlowKind::Start(_) => dialog.rebuild_start(),
            FlowKind::Finish => {
                dialog.finish_asks = dialog.finish_asks();
                if dialog.finish_asks {
                    dialog.build_finish_mode();
                } else {
                    dialog.begin_preview();
                }
            }
            FlowKind::Resume => dialog.build_resume(),
            FlowKind::Sync => {}
            FlowKind::Promote => dialog.build_promote(),
            FlowKind::SetParent => dialog.build_set_parent(),
        }
        if matches!(dialog.phase, Phase::Form) {
            dialog.focus = dialog.first_focus();
        }
        dialog
    }

    // -- building the forms --------------------------------------------

    fn exists(&self, name: &str) -> bool {
        self.seed.branches.iter().any(|b| b == name)
    }

    /// The concrete branch standing for a base reference (`main` or
    /// `master`), when one exists.
    fn concrete(&self, reference: &str) -> Option<String> {
        self.seed
            .view
            .compiled()
            .concrete_base(reference, &|n| self.exists(n))
    }

    /// Fills the Start form for the chosen type.
    fn rebuild_start(&mut self) {
        let topics = self.seed.view.topics();
        let Some(topic) = topics.get(self.topic).cloned() else {
            return;
        };
        self.type_combo.set_options(
            topics
                .iter()
                .enumerate()
                .map(|(i, t)| ComboOption::new(t.display_label(), i.to_string()))
                .collect(),
        );
        self.type_combo.select(Some(self.topic.to_string()));
        let bump = if topic.id.contains("hotfix") || topic.id.contains("patch") {
            Bump::Patch
        } else {
            Bump::Minor
        };
        let suggest = Suggestions {
            user: self.seed.user.clone(),
            today: today_iso(),
            version: next_version(&self.seed.tags, bump),
        };
        let pattern = self
            .seed
            .view
            .compiled()
            .render_pattern(&topic.id)
            .map(|p| p.as_str().to_string())
            .unwrap_or_else(|| topic.pattern.clone());
        self.form = StartForm::new(&pattern, &suggest);
        // The start point: the type's parent (its concrete spelling), then
        // every local branch.
        let parent = topic.parent.as_deref().and_then(|p| self.concrete(p));
        let mut options: Vec<ComboOption> = Vec::new();
        for name in parent.iter().chain(self.seed.branches.iter()) {
            if !options.iter().any(|o| o.value == *name) {
                options.push(ComboOption::plain(name.clone()));
            }
        }
        self.start_combo.set_options(options);
        self.start_combo
            .select(parent.or_else(|| self.seed.current.clone()));
        self.issue_combo.set_options(
            self.seed
                .issues
                .iter()
                .map(|(reference, title)| {
                    ComboOption::new(format!("{reference}  {title}"), reference.clone())
                })
                .collect(),
        );
        self.publish = false;
        self.focus = self.first_focus();
    }

    /// Whether the branch's type leaves the finish mode to the user.
    fn finish_asks(&self) -> bool {
        let Some(type_id) = self
            .seed
            .view
            .branch(&self.branch)
            .and_then(|b| b.type_id.clone())
        else {
            return false;
        };
        self.seed
            .view
            .topics()
            .iter()
            .any(|t| t.id == type_id && t.finish_mode == Some(FinishMode::Ask))
    }

    fn build_finish_mode(&mut self) {
        self.target_combo = Combo::new(msg::PICK_TARGET.get());
        self.target_combo.set_options(vec![
            ComboOption::new(msg::FINISH_LOCAL.get(), "local-merge"),
            ComboOption::new(msg::FINISH_REQUEST.get(), "merge-request"),
        ]);
        self.target_combo.select(Some("local-merge".to_string()));
    }

    /// The Resume dialog: the stopped action's steps, Continue / Abort.
    fn build_resume(&mut self) {
        let Some(StoppedFlow {
            steps,
            failed,
            message,
            ..
        }) = self.actions.stopped()
        else {
            self.message = Some(msg::WHY_NOTHING_STOPPED.get().to_string());
            return;
        };
        let state = (0..steps.len())
            .map(|i| match i.cmp(&failed) {
                std::cmp::Ordering::Less => StepState::Done,
                std::cmp::Ordering::Equal => StepState::Failed,
                std::cmp::Ordering::Greater => StepState::Pending,
            })
            .collect();
        let (_, rx) = std::sync::mpsc::channel();
        self.phase = Phase::Stopped {
            run: Run { steps, state, rx },
            message,
            conflict: true,
        };
        self.focus = Focus::Primary;
    }

    fn has_form(&self) -> bool {
        match self.kind {
            FlowKind::Finish => self.finish_asks,
            FlowKind::Resume => false,
            _ => true,
        }
    }

    /// Handed over once: the merge request to open after the run.
    pub(crate) fn take_merge_request(
        &mut self,
    ) -> Option<gitpoulpi_core::workflow::MergeRequestSpec> {
        if matches!(self.phase, Phase::Done(_)) {
            self.merge_request.take()
        } else {
            None
        }
    }

    fn build_promote(&mut self) {
        let wf = &self.seed.view.report.workflow;
        let options: Vec<ComboOption> = wf
            .bases
            .iter()
            .filter(|b| b.environment && b.parent.is_some())
            .map(|b| {
                let parent = b.parent.clone().unwrap_or_default();
                ComboOption::new(msg::promote_option(&b.name, &parent), b.name.clone())
            })
            .collect();
        let first = options.first().map(|o| o.value.clone());
        self.target_combo = Combo::new(msg::PICK_TARGET.get());
        self.target_combo.set_options(options);
        self.target_combo.select(first);
    }

    fn build_set_parent(&mut self) {
        let mut options: Vec<ComboOption> = Vec::new();
        for name in self
            .seed
            .view
            .base_names()
            .into_iter()
            .chain(self.seed.branches.iter().cloned())
        {
            if name != self.branch && !options.iter().any(|o| o.value == name) {
                options.push(ComboOption::plain(name));
            }
        }
        let current = self
            .seed
            .view
            .branch(&self.branch)
            .and_then(|b| b.parent.as_ref())
            .map(|p| p.name.clone());
        self.target_combo = Combo::new(msg::PICK_TARGET.get())
            .with_custom()
            .with_search();
        self.target_combo.set_options(options);
        self.target_combo.select(current);
    }

    // -- the request -----------------------------------------------------

    /// The action the form describes, or why it cannot run yet.
    fn request(&self) -> Result<FlowRequest, String> {
        let op = match self.kind {
            FlowKind::Start(_) => {
                let topic = self
                    .seed
                    .view
                    .topics()
                    .get(self.topic)
                    .ok_or_else(|| msg::WHY_NO_SUCH_TYPE.get().to_string())?;
                let name = self.form.name(|n| self.exists(n))?;
                let start_point = self
                    .start_combo
                    .selected()
                    .map(str::to_string)
                    .or_else(|| self.seed.current.clone())
                    .ok_or_else(|| msg::WHY_NO_START_POINT.get().to_string())?;
                FlowOp::Start {
                    type_id: topic.id.clone(),
                    fields: self.form.values(),
                    name,
                    start_point,
                    publish: self.publish,
                }
            }
            FlowKind::Finish => FlowOp::Finish {
                branch: self.branch.clone(),
                mode: match self.target_combo.selected().filter(|_| self.finish_asks) {
                    Some("merge-request") => Some(FinishMode::MergeRequest),
                    Some(_) => Some(FinishMode::LocalMerge),
                    None => None,
                },
            },
            FlowKind::Resume => return Err(String::new()),
            FlowKind::Sync => FlowOp::Sync {
                branch: self.branch.clone(),
                with_children: self.with_children,
            },
            FlowKind::Promote => {
                let to = self
                    .target_combo
                    .selected()
                    .ok_or_else(|| msg::WHY_NO_TARGET.get().to_string())?
                    .to_string();
                let parent = self
                    .seed
                    .view
                    .report
                    .workflow
                    .bases
                    .iter()
                    .find(|b| b.name == to)
                    .and_then(|b| b.parent.clone())
                    .ok_or_else(|| msg::WHY_NO_TARGET.get().to_string())?;
                let from = self.concrete(&parent).unwrap_or(parent);
                let to = self.concrete(&to).unwrap_or(to);
                FlowOp::Promote { from, to }
            }
            FlowKind::SetParent => FlowOp::SetParent {
                branch: self.branch.clone(),
                parent: self
                    .target_combo
                    .selected()
                    .ok_or_else(|| msg::WHY_NO_TARGET.get().to_string())?
                    .to_string(),
            },
        };
        Ok(FlowRequest {
            op,
            run_hooks: self.run_hooks,
        })
    }

    fn begin_preview(&mut self) {
        match self.request() {
            Ok(mut request) => {
                // The preview lists the hooks that *could* run; whether they
                // do is the checkbox's say at execute time.
                request.run_hooks = true;
                let rx = self.actions.preview(&request);
                self.phase = Phase::Previewing {
                    rx,
                    since: Instant::now(),
                };
                self.message = None;
            }
            Err(why) => self.message = Some(why),
        }
    }

    fn begin_run(&mut self, preview: &FlowPreview) {
        let Ok(request) = self.request() else {
            return;
        };
        let rx = self.actions.execute(&request);
        let steps: Vec<FlowStep> = preview
            .steps
            .iter()
            .filter(|s| s.kind != StepKind::Hook || self.run_hooks)
            .cloned()
            .collect();
        let state = vec![StepState::Pending; steps.len()];
        self.phase = Phase::Running(Run { steps, state, rx });
        self.message = None;
    }

    /// Opens the question about the committed hooks over the preview.
    fn ask_hook_trust(&mut self, preview: &FlowPreview) {
        let gitpoulpi_core::workflow::HooksTrust::Untrusted { hash, previous } = &preview.trust
        else {
            return;
        };
        self.trust = Some(TrustUi {
            pane: HookTrust::new(hash.clone(), &preview.hook_lines, previous.as_deref()),
            pending: None,
        });
        self.message = None;
    }

    /// An answer of the hook trust question.
    fn hook_trust_pick(&mut self, pick: Pick) {
        let Phase::Preview(preview) = &self.phase else {
            self.trust = None;
            return;
        };
        let preview = preview.clone();
        match pick {
            Pick::Skip => {
                self.trust = None;
                self.run_hooks = false;
                self.begin_run(&preview);
            }
            Pick::Trust => {
                if let Some(ui) = &mut self.trust {
                    let rx = self.actions.trust_hooks(&ui.pane.hash);
                    ui.pending = Some((rx, Instant::now()));
                }
            }
        }
    }

    /// The daemon's answer to "Trust and run": run with the hooks, or show
    /// why it was refused (the file changed: the preview is read again).
    fn poll_hook_trust(&mut self) {
        let Some(ui) = &mut self.trust else {
            return;
        };
        let Some((rx, _)) = &ui.pending else {
            return;
        };
        let answer = match rx.try_recv() {
            Ok(answer) => answer,
            Err(std::sync::mpsc::TryRecvError::Empty) => return,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => Err(msg::LOST.get().into()),
        };
        self.trust = None;
        match answer {
            Ok(()) => {
                self.run_hooks = true;
                if let Phase::Preview(preview) = &self.phase {
                    let preview = preview.clone();
                    self.begin_run(&preview);
                }
            }
            Err(why) => {
                self.begin_preview();
                self.message = Some(why);
            }
        }
    }

    // -- polling -------------------------------------------------------------

    /// A request or a run is on its way (the spinner needs frames).
    pub(crate) fn animating(&self) -> bool {
        matches!(self.phase, Phase::Previewing { .. } | Phase::Running(_))
            || self.trust.as_ref().is_some_and(|t| t.pending.is_some())
    }

    /// Drains the channels. Returns a status line when an action ended.
    pub(crate) fn poll(&mut self) -> Option<String> {
        let mut said = None;
        self.poll_hook_trust();
        let phase = std::mem::replace(&mut self.phase, Phase::Form);
        self.phase = match phase {
            Phase::Previewing { rx, since } => match rx.try_recv() {
                Ok(Ok(preview)) => {
                    self.focus = Focus::Primary;
                    Phase::Preview(preview)
                }
                Ok(Err(why)) => {
                    self.message = Some(why);
                    self.focus = self.first_focus();
                    // Finish has no form to go back to: stay on a closed
                    // preview with the reason shown.
                    if !self.has_form() {
                        Phase::Preview(FlowPreview {
                            blockers: vec![self.message.take().unwrap_or_default()],
                            ..Default::default()
                        })
                    } else {
                        Phase::Form
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => Phase::Previewing { rx, since },
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.message = Some(msg::LOST.get().into());
                    Phase::Form
                }
            },
            Phase::Running(mut run) => {
                let mut next: Option<End> = None;
                while let Ok(event) = run.rx.try_recv() {
                    match event {
                        FlowEvent::StepStarted(i) => {
                            if let Some(s) = run.state.get_mut(i) {
                                *s = StepState::Running(Instant::now());
                            }
                        }
                        FlowEvent::StepDone(i) => {
                            if let Some(s) = run.state.get_mut(i) {
                                *s = StepState::Done;
                            }
                        }
                        FlowEvent::StepFailed {
                            index,
                            message,
                            conflict,
                        } => {
                            if let Some(s) = run.state.get_mut(index) {
                                *s = StepState::Failed;
                            }
                            self.focus = Focus::Primary;
                            next = Some(End::Failed(message, conflict));
                            break;
                        }
                        FlowEvent::Steps(steps) => {
                            run.state = vec![StepState::Pending; steps.len()];
                            run.steps = steps;
                        }
                        FlowEvent::MergeRequest(spec) => self.merge_request = Some(spec),
                        FlowEvent::Finished => {
                            for s in &mut run.state {
                                if matches!(s, StepState::Pending | StepState::Running(_)) {
                                    *s = StepState::Done;
                                }
                            }
                            said = Some(if self.aborted {
                                msg::ABORTED.get().to_string()
                            } else {
                                msg::finished(self.title_plain())
                            });
                            self.focus = Focus::Primary;
                            next = Some(End::Finished);
                            break;
                        }
                    }
                }
                match next {
                    Some(End::Finished) => Phase::Done(run),
                    Some(End::Failed(message, conflict)) => {
                        said = Some(message.clone());
                        Phase::Stopped {
                            run,
                            message,
                            conflict,
                        }
                    }
                    None => Phase::Running(run),
                }
            }
            other => other,
        };
        said
    }

    fn title_plain(&self) -> String {
        match self.kind {
            FlowKind::Start(_) => self
                .seed
                .view
                .topics()
                .get(self.topic)
                .map(|t| msg::start_type(t.display_label()))
                .unwrap_or_default(),
            FlowKind::Finish => msg::title_finish(&self.branch),
            FlowKind::Sync => msg::title_sync(&self.branch),
            FlowKind::Promote => msg::TITLE_PROMOTE.get().to_string(),
            FlowKind::SetParent => msg::title_set_parent(&self.branch),
            FlowKind::Resume => msg::TITLE_RESUME.get().to_string(),
        }
    }

    // -- focus ---------------------------------------------------------------

    /// The focusable parts of the current phase, in `Tab` order.
    fn order(&self) -> Vec<Focus> {
        let mut order = Vec::new();
        match &self.phase {
            Phase::Form => {
                match self.kind {
                    FlowKind::Start(_) => {
                        order.push(Focus::TypeCombo);
                        if self.has_issue_picker() {
                            order.push(Focus::IssueCombo);
                        }
                        order.extend((0..self.form.fields.len()).map(Focus::Field));
                        order.push(Focus::StartCombo);
                        order.push(Focus::Publish);
                    }
                    FlowKind::Sync => order.push(Focus::Children),
                    FlowKind::Promote | FlowKind::SetParent => order.push(Focus::TargetCombo),
                    FlowKind::Finish if self.finish_asks => order.push(Focus::TargetCombo),
                    FlowKind::Finish | FlowKind::Resume => {}
                }
                order.push(Focus::Primary);
                order.push(Focus::Secondary);
            }
            Phase::Previewing { .. } => order.push(Focus::Secondary),
            // The question owns the body: only its Cancel button is a stop.
            Phase::Preview(_) if self.trust.is_some() => order.push(Focus::Secondary),
            Phase::Preview(p) => {
                // Untrusted hooks have no checkbox: the question asks.
                if !p.hooks.is_empty() && !untrusted(p) {
                    order.push(Focus::Hooks);
                }
                if p.blockers.is_empty() {
                    order.push(Focus::Primary);
                }
                order.push(Focus::Secondary);
            }
            Phase::Running(_) => {}
            Phase::Stopped { conflict, .. } => {
                if *conflict {
                    order.push(Focus::Primary);
                    order.push(Focus::Abort);
                }
                order.push(Focus::Secondary);
            }
            Phase::Done(_) => order.push(Focus::Secondary),
        }
        order
    }

    fn first_focus(&self) -> Focus {
        let order = self.order();
        // A Start form begins in the first text field; the others on the
        // first control.
        order
            .iter()
            .copied()
            .find(|f| matches!(f, Focus::Field(_)))
            .or_else(|| order.first().copied())
            .unwrap_or(Focus::Primary)
    }

    fn has_issue_picker(&self) -> bool {
        (self.form.has(Token::Issue) || self.form.has(Token::IssueKey))
            && !self.seed.issues.is_empty()
    }

    fn step_focus(&mut self, forward: bool) {
        let order = self.order();
        if order.is_empty() {
            return;
        }
        let at = order.iter().position(|f| *f == self.focus);
        let next = match (at, forward) {
            (Some(i), true) => (i + 1) % order.len(),
            (Some(i), false) => (i + order.len() - 1) % order.len(),
            (None, _) => 0,
        };
        self.focus = order[next];
    }

    fn combo_of(&mut self, focus: Focus) -> Option<&mut Combo> {
        match focus {
            Focus::TypeCombo => Some(&mut self.type_combo),
            Focus::IssueCombo => Some(&mut self.issue_combo),
            Focus::StartCombo => Some(&mut self.start_combo),
            Focus::TargetCombo => Some(&mut self.target_combo),
            _ => None,
        }
    }

    fn open_combo(&mut self) -> Option<Focus> {
        [
            Focus::TypeCombo,
            Focus::IssueCombo,
            Focus::StartCombo,
            Focus::TargetCombo,
        ]
        .into_iter()
        .find(|f| self.combo_of(*f).is_some_and(|c| c.is_open()))
    }

    fn field_parts(&mut self) -> (&mut FieldEditor<Focus>, Option<(Focus, &mut String)>) {
        let focus = self.focus;
        let field = match (focus, &self.phase) {
            (Focus::Field(i), Phase::Form) => self.form.fields.get_mut(i).map(|f| &mut f.value),
            _ => None,
        };
        self.edit.sync(field.is_some().then_some(focus));
        (&mut self.edit, field.map(|f| (focus, f)))
    }

    // -- activation ----------------------------------------------------------

    fn picked(&mut self, focus: Focus, value: &str) {
        match focus {
            Focus::TypeCombo => {
                if let Ok(i) = value.parse::<usize>() {
                    if i != self.topic {
                        self.topic = i;
                        self.rebuild_start();
                        self.focus = Focus::TypeCombo;
                    }
                }
            }
            Focus::IssueCombo => {
                let title = self
                    .seed
                    .issues
                    .iter()
                    .find(|(r, _)| r == value)
                    .map(|(_, t)| t.clone())
                    .unwrap_or_default();
                self.form.pick_issue(value, &title);
            }
            _ => {}
        }
        self.message = None;
    }

    fn primary(&mut self) -> FlowOutcome {
        let phase = std::mem::replace(&mut self.phase, Phase::Form);
        match phase {
            Phase::Form => {
                self.phase = Phase::Form;
                self.begin_preview();
                FlowOutcome::Stay
            }
            Phase::Preview(preview) => {
                if preview.blockers.is_empty() && untrusted(&preview) && self.trust.is_none() {
                    // Committed hooks nobody trusted: ask before anything runs.
                    self.ask_hook_trust(&preview);
                    self.phase = Phase::Preview(preview);
                } else if preview.blockers.is_empty() {
                    self.begin_run(&preview);
                    if !matches!(self.phase, Phase::Running(_)) {
                        self.phase = Phase::Preview(preview);
                    }
                } else {
                    self.phase = Phase::Preview(preview);
                }
                FlowOutcome::Stay
            }
            Phase::Stopped {
                run,
                conflict: true,
                ..
            } => {
                let rx = self.actions.resume(Resume::Continue);
                self.phase = Phase::Running(Run {
                    steps: run.steps,
                    state: run
                        .state
                        .into_iter()
                        .map(|s| {
                            if s == StepState::Failed {
                                StepState::Pending
                            } else {
                                s
                            }
                        })
                        .collect(),
                    rx,
                });
                FlowOutcome::Stay
            }
            Phase::Stopped {
                run,
                message,
                conflict: false,
            } => {
                self.phase = Phase::Stopped {
                    run,
                    message,
                    conflict: false,
                };
                FlowOutcome::Close
            }
            Phase::Done(_) => FlowOutcome::Close,
            other => {
                self.phase = other;
                FlowOutcome::Stay
            }
        }
    }

    fn secondary(&mut self) -> FlowOutcome {
        // Cancel of the hook trust question goes back to the preview.
        if let Some(ui) = &self.trust {
            if ui.pending.is_none() {
                self.trust = None;
                self.focus = Focus::Primary;
            }
            return FlowOutcome::Stay;
        }
        match &self.phase {
            // Back from a preview to the form (Finish has none).
            Phase::Preview(_) if self.has_form() => {
                self.phase = Phase::Form;
                self.focus = self.first_focus();
                self.message = None;
                FlowOutcome::Stay
            }
            Phase::Running(_) => FlowOutcome::Stay,
            _ => FlowOutcome::Close,
        }
    }

    fn abort(&mut self) -> FlowOutcome {
        if let Phase::Stopped { run, .. } = std::mem::replace(&mut self.phase, Phase::Form) {
            let rx = self.actions.resume(Resume::Abort);
            // The abort's own answer: Finished closes the story.
            let mut run = run;
            run.rx = rx;
            self.aborted = true;
            self.phase = Phase::Running(run);
        }
        FlowOutcome::Stay
    }

    fn activate(&mut self, focus: Focus) -> FlowOutcome {
        match focus {
            Focus::Primary => self.primary(),
            Focus::Secondary => self.secondary(),
            Focus::Abort => self.abort(),
            Focus::Publish => {
                self.publish = !self.publish;
                FlowOutcome::Stay
            }
            Focus::Children => {
                self.with_children = !self.with_children;
                FlowOutcome::Stay
            }
            Focus::Hooks => {
                self.run_hooks = !self.run_hooks;
                FlowOutcome::Stay
            }
            Focus::TypeCombo | Focus::IssueCombo | Focus::StartCombo | Focus::TargetCombo => {
                if let Some(combo) = self.combo_of(focus) {
                    combo.open();
                }
                FlowOutcome::Stay
            }
            Focus::Field(_) => self.primary(),
        }
    }

    // -- input ---------------------------------------------------------------

    pub(crate) fn paste(&mut self, text: &str) {
        if let Some(open) = self.open_combo() {
            if let Some(c) = self.combo_of(open) {
                c.paste(text);
            }
            return;
        }
        let clean: String = text.chars().filter(|c| !c.is_control()).collect();
        let focus = self.focus;
        if let (edit, Some((id, field))) = self.field_parts() {
            edit.caret_mut(id).insert(field, &clean);
            if let Focus::Field(i) = focus {
                self.form.edited(i);
            }
        }
    }

    fn combo_outcome(&mut self, focus: Focus, outcome: ComboOutcome) -> bool {
        match outcome {
            ComboOutcome::Ignored => false,
            ComboOutcome::Handled => true,
            ComboOutcome::Picked(value) | ComboOutcome::Custom(value) => {
                self.picked(focus, &value);
                true
            }
        }
    }

    pub(crate) fn on_key(&mut self, key: KeyEvent) -> FlowOutcome {
        if let Some(ui) = &mut self.trust {
            // While the daemon remembers the answer nothing else happens.
            if ui.pending.is_some() {
                return FlowOutcome::Stay;
            }
            match ui.pane.on_key(key) {
                hook_trust::Event::Cancel => {
                    self.trust = None;
                    self.focus = Focus::Primary;
                    return FlowOutcome::Stay;
                }
                hook_trust::Event::Pick(pick) => {
                    self.hook_trust_pick(pick);
                    return FlowOutcome::Stay;
                }
                hook_trust::Event::Handled => return FlowOutcome::Stay,
                hook_trust::Event::Ignored => {}
            }
        }
        // An open dropdown is modal: it swallows every key.
        if let Some(open) = self.open_combo() {
            if let Some(combo) = self.combo_of(open) {
                let outcome = combo.on_key(key);
                self.combo_outcome(open, outcome);
            }
            return FlowOutcome::Stay;
        }
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        let focus = self.focus;
        if let (edit, Some((id, field))) = self.field_parts() {
            if let Some((result, copied)) = edit.dialog_key(id, field, &key) {
                if result == crate::text_edit::KeyEdit::Edited {
                    if let Focus::Field(i) = focus {
                        self.form.edited(i);
                    }
                    self.message = None;
                }
                if copied.is_some() {
                    self.message = copied;
                }
                return FlowOutcome::Stay;
            }
        }
        match key.code {
            KeyCode::Esc => return self.secondary_or_close(),
            KeyCode::Tab | KeyCode::BackTab | KeyCode::Down | KeyCode::Up => {
                let forward =
                    matches!(key.code, KeyCode::Down) || (key.code == KeyCode::Tab && !shift);
                self.step_focus(forward);
            }
            KeyCode::Enter => {
                return match self.focus {
                    Focus::Field(_) => self.primary(),
                    f => self.activate(f),
                }
            }
            KeyCode::Char(' ') => {
                if let Focus::Field(_) = self.focus {
                } else {
                    return self.activate(self.focus);
                }
            }
            _ => {}
        }
        FlowOutcome::Stay
    }

    /// Esc: leave the dialog, except while a run is going.
    fn secondary_or_close(&mut self) -> FlowOutcome {
        match self.phase {
            Phase::Running(_) => FlowOutcome::Stay,
            Phase::Preview(_) if self.has_form() => self.secondary(),
            _ => FlowOutcome::Close,
        }
    }

    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent) -> FlowOutcome {
        if let Some(ui) = &mut self.trust {
            if ui.pending.is_some() {
                return FlowOutcome::Stay;
            }
            match ui.pane.on_mouse(mouse) {
                hook_trust::Event::Pick(pick) => {
                    self.hook_trust_pick(pick);
                    return FlowOutcome::Stay;
                }
                hook_trust::Event::Handled => return FlowOutcome::Stay,
                hook_trust::Event::Cancel | hook_trust::Event::Ignored => {}
            }
        }
        // An open dropdown owns the pointer.
        if let Some(open) = self.open_combo() {
            if let Some(combo) = self.combo_of(open) {
                let outcome = combo.on_mouse(mouse);
                self.combo_outcome(open, outcome);
            }
            return FlowOutcome::Stay;
        }
        let hit = dialog_kit::hit_at(&self.hits, mouse.column, mouse.row);
        let focus = self.focus;
        if !matches!(mouse.kind, MouseEventKind::Down(_)) {
            let handled = match self.field_parts() {
                (edit, Some((id, field))) => {
                    edit.dialog_mouse(Some(id), field, mouse.kind, mouse.column, mouse.row)
                }
                (edit, None) => edit.dialog_mouse(None, "", mouse.kind, mouse.column, mouse.row),
            };
            if handled {
                return FlowOutcome::Stay;
            }
        }
        match mouse.kind {
            MouseEventKind::Moved | MouseEventKind::Drag(_) => self.hover = hit,
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(part) = hit {
                    self.focus = part;
                    if let Focus::Field(_) = part {
                        if let (edit, Some((id, field))) = self.field_parts() {
                            edit.dialog_mouse(Some(id), field, mouse.kind, mouse.column, mouse.row);
                        }
                    } else {
                        return self.activate(part);
                    }
                }
                let _ = focus;
            }
            _ => {}
        }
        FlowOutcome::Stay
    }

    // -- drawing -------------------------------------------------------------

    fn title(&self) -> String {
        self.title_plain()
    }

    fn primary_label(&self) -> Option<&'static str> {
        match &self.phase {
            Phase::Form => Some(msg::BTN_PREVIEW.get()),
            Phase::Previewing { .. } | Phase::Running(_) => None,
            Phase::Preview(_) if self.trust.is_some() => None,
            Phase::Preview(p) => p.blockers.is_empty().then(|| match self.kind {
                FlowKind::Start(_) => msg::BTN_START.get(),
                FlowKind::Finish => msg::BTN_FINISH.get(),
                FlowKind::Sync => msg::BTN_SYNC.get(),
                FlowKind::Promote => msg::BTN_PROMOTE.get(),
                FlowKind::SetParent => msg::BTN_SET_PARENT.get(),
                FlowKind::Resume => msg::BTN_CONTINUE.get(),
            }),
            Phase::Stopped { conflict, .. } => conflict.then(|| msg::BTN_CONTINUE.get()),
            Phase::Done(_) => None,
        }
    }

    fn secondary_label(&self) -> Option<&'static str> {
        match &self.phase {
            Phase::Form => Some(gitpoulpi_i18n::msg::dialog::CANCEL.get()),
            Phase::Previewing { .. } => Some(gitpoulpi_i18n::msg::dialog::CANCEL.get()),
            Phase::Preview(_) if self.trust.is_some() => {
                Some(gitpoulpi_i18n::msg::dialog::CANCEL.get())
            }
            Phase::Preview(_) if self.has_form() => Some(msg::BTN_BACK.get()),
            Phase::Preview(_) => Some(gitpoulpi_i18n::msg::dialog::CANCEL.get()),
            Phase::Running(_) => None,
            Phase::Stopped { .. } | Phase::Done(_) => Some(msg::BTN_CLOSE.get()),
        }
    }

    pub(crate) fn render(&mut self, rect: Rect, buf: &mut Buffer) {
        self.field_parts();
        self.edit.begin_frame();
        let title = self.title();
        self.close.panel(buf, rect, &title);
        let mut hits: Vec<(Rect, Focus)> = Vec::new();
        let inner = Rect::new(
            rect.x + 2,
            rect.y + 1,
            rect.width.saturating_sub(4),
            rect.height.saturating_sub(2),
        );
        if inner.width < 30 || inner.height < 8 {
            self.hits = hits;
            return;
        }
        let panel_bg = th().ui.panel;
        let muted = Style::default().fg(th().ui.muted).bg(panel_bg);
        let normal = Style::default().fg(th().ui.text).bg(panel_bg);
        // The footer: message, then buttons.
        let buttons_y = inner.bottom().saturating_sub(1);
        let message_y = buttons_y.saturating_sub(1);
        let body_bottom = message_y;

        let mut popups: Vec<Focus> = Vec::new();
        match &self.phase {
            Phase::Form => self.render_form(buf, inner, body_bottom, &mut hits, &mut popups),
            Phase::Previewing { since, .. } => {
                if since.elapsed() >= SPIN_AFTER {
                    let spin = crate::activity::spinner_frame(since.elapsed());
                    dialog_kit::put(
                        buf,
                        inner.x,
                        inner.y,
                        &format!("{spin} {}", msg::READING_PLAN.get()),
                        inner.width,
                        muted,
                    );
                }
            }
            Phase::Preview(_) if self.trust.is_some() => {
                let area = Rect::new(
                    inner.x,
                    inner.y,
                    inner.width,
                    body_bottom.saturating_sub(inner.y),
                );
                let mut waiting = None;
                if let Some(ui) = &mut self.trust {
                    ui.pane.render(buf, area);
                    waiting = ui.pending.as_ref().map(|(_, since)| *since);
                }
                if let Some(since) = waiting.filter(|s| s.elapsed() >= SPIN_AFTER) {
                    let spin = crate::activity::spinner_frame(since.elapsed());
                    dialog_kit::put(
                        buf,
                        inner.x,
                        message_y,
                        &format!("{spin} {}", gitpoulpi_i18n::msg::flow_hook_trust::TRUSTING.get()),
                        inner.width,
                        muted,
                    );
                }
            }
            Phase::Preview(p) => self.render_preview(buf, inner, body_bottom, p, &mut hits),
            Phase::Running(run) | Phase::Done(run) => {
                self.render_steps(buf, inner, body_bottom, run, None)
            }
            Phase::Stopped {
                run,
                message,
                conflict,
            } => {
                let help = if *conflict {
                    msg::CONFLICT_HELP.get()
                } else {
                    ""
                };
                self.render_steps(buf, inner, body_bottom, run, Some((message, help)));
            }
        }
        // The footer line.
        let shown = match &self.message {
            Some(m) => Some((m.clone(), th().status.danger)),
            None => match &self.phase {
                Phase::Done(_) => Some((msg::DONE.get().to_string(), th().status.success)),
                _ => None,
            },
        };
        if let Some((text, color)) = shown {
            dialog_kit::put(
                buf,
                inner.x,
                message_y,
                &text,
                inner.width,
                Style::default().fg(color).bg(panel_bg),
            );
        }
        // Buttons, right-aligned: primary (accent), secondary.
        let mut right = inner.right();
        let mut place = |label: &str, focus: Focus, primary: bool, hits: &mut Vec<_>| {
            let w = dialog_kit::button_width(label);
            right = right.saturating_sub(w);
            let focused = self.focus == focus;
            let hovered = self.hover == Some(focus);
            let state = if primary {
                ButtonState::Primary { hovered, focused }
            } else {
                ButtonState::Normal { hovered, focused }
            };
            let r = dialog_kit::button(buf, right, buttons_y, label, state);
            hits.push((r, focus));
            right = right.saturating_sub(1);
        };
        if let Some(label) = self.secondary_label() {
            place(label, Focus::Secondary, false, &mut hits);
        }
        if matches!(&self.phase, Phase::Stopped { conflict: true, .. }) {
            place(msg::BTN_ABORT.get(), Focus::Abort, false, &mut hits);
        }
        if let Some(label) = self.primary_label() {
            place(label, Focus::Primary, true, &mut hits);
        }
        let _ = (normal, muted);
        // Nothing here overlaps, so the order of the hit list is free.
        self.hits = hits;
        // Dropdown popups last, over everything; while one is open its rows
        // are the only hits (popups own the pointer).
        for focus in popups {
            if let Some(combo) = self.combo_of(focus) {
                combo.render_popup(buf, rect);
            }
        }
    }

    fn label(buf: &mut Buffer, x: u16, y: u16, w: u16, text: &str) {
        dialog_kit::put(
            buf,
            x,
            y,
            text,
            w,
            Style::default().fg(th().ui.muted).bg(th().ui.panel),
        );
    }

    fn render_form(
        &mut self,
        buf: &mut Buffer,
        inner: Rect,
        bottom: u16,
        hits: &mut Vec<(Rect, Focus)>,
        popups: &mut Vec<Focus>,
    ) {
        let label_w: u16 = 16;
        let field_x = inner.x + label_w;
        let field_w = inner.width.saturating_sub(label_w);
        let mut y = inner.y;
        let hover = self.hover;
        let focus = self.focus;
        let mut combo_row = |dialog: &mut Self,
                             buf: &mut Buffer,
                             y: u16,
                             label: &str,
                             which: Focus,
                             hits: &mut Vec<(Rect, Focus)>| {
            Self::label(buf, inner.x, y, label_w, label);
            let rect = Rect::new(field_x, y, field_w, 1);
            if let Some(combo) = dialog.combo_of(which) {
                combo.render_field(buf, rect, focus == which, hover == Some(which));
            }
            hits.push((rect, which));
            popups.push(which);
        };
        match self.kind {
            FlowKind::Start(_) => {
                combo_row(self, buf, y, msg::FIELD_TYPE.get(), Focus::TypeCombo, hits);
                y += 1;
                if self.has_issue_picker() && y < bottom {
                    combo_row(
                        self,
                        buf,
                        y,
                        msg::FIELD_ISSUE.get(),
                        Focus::IssueCombo,
                        hits,
                    );
                    y += 1;
                }
                for i in 0..self.form.fields.len() {
                    if y >= bottom {
                        break;
                    }
                    let token = self.form.fields[i].token;
                    Self::label(buf, inner.x, y, label_w, label_of(token));
                    let rect = Rect::new(field_x, y, field_w, 1);
                    let value = self.form.fields[i].value.clone();
                    dialog_kit::text_input(
                        buf,
                        rect,
                        &value,
                        hint_of(token),
                        focus == Focus::Field(i),
                        hover == Some(Focus::Field(i)),
                        (&self.edit, Focus::Field(i)),
                    );
                    hits.push((rect, Focus::Field(i)));
                    y += 1;
                }
                if y < bottom {
                    combo_row(
                        self,
                        buf,
                        y,
                        msg::FIELD_START_FROM.get(),
                        Focus::StartCombo,
                        hits,
                    );
                    y += 1;
                }
                if y < bottom {
                    let r = dialog_kit::checkbox(
                        buf,
                        field_x,
                        y,
                        field_w,
                        msg::PUBLISH.get(),
                        self.publish,
                        CheckState::Enabled {
                            hovered: hover == Some(Focus::Publish),
                            focused: focus == Focus::Publish,
                        },
                    );
                    hits.push((r, Focus::Publish));
                    y += 2;
                }
                if y < bottom {
                    Self::label(buf, inner.x, y, label_w, msg::FIELD_NAME.get());
                    let (text, color) = match self.form.name(|n| self.exists(n)) {
                        Ok(name) => (name, th().status.success),
                        Err(why) => (why, th().status.warning),
                    };
                    dialog_kit::put(
                        buf,
                        field_x,
                        y,
                        &text,
                        field_w,
                        Style::default()
                            .fg(color)
                            .bg(th().ui.panel)
                            .add_modifier(Modifier::BOLD),
                    );
                    y += 2;
                }
                if y < bottom {
                    dialog_kit::put(
                        buf,
                        inner.x,
                        y,
                        msg::TOKEN_HELP.get(),
                        inner.width,
                        Style::default().fg(th().ui.muted).bg(th().ui.panel),
                    );
                }
            }
            FlowKind::Sync => {
                Self::label(buf, inner.x, y, label_w, msg::FIELD_BRANCH.get());
                dialog_kit::put(
                    buf,
                    field_x,
                    y,
                    &self.branch,
                    field_w,
                    Style::default().fg(th().ui.text).bg(th().ui.panel),
                );
                y += 2;
                let r = dialog_kit::checkbox(
                    buf,
                    field_x,
                    y,
                    field_w,
                    msg::WITH_CHILDREN.get(),
                    self.with_children,
                    CheckState::Enabled {
                        hovered: hover == Some(Focus::Children),
                        focused: focus == Focus::Children,
                    },
                );
                hits.push((r, Focus::Children));
            }
            FlowKind::Promote => {
                combo_row(
                    self,
                    buf,
                    y,
                    msg::FIELD_PROMOTE.get(),
                    Focus::TargetCombo,
                    hits,
                );
            }
            FlowKind::SetParent => {
                Self::label(buf, inner.x, y, label_w, msg::FIELD_BRANCH.get());
                dialog_kit::put(
                    buf,
                    field_x,
                    y,
                    &self.branch,
                    field_w,
                    Style::default().fg(th().ui.text).bg(th().ui.panel),
                );
                y += 2;
                combo_row(
                    self,
                    buf,
                    y,
                    msg::FIELD_PARENT.get(),
                    Focus::TargetCombo,
                    hits,
                );
            }
            FlowKind::Finish if self.finish_asks => {
                Self::label(buf, inner.x, y, label_w, msg::FIELD_BRANCH.get());
                dialog_kit::put(
                    buf,
                    field_x,
                    y,
                    &self.branch,
                    field_w,
                    Style::default().fg(th().ui.text).bg(th().ui.panel),
                );
                y += 2;
                combo_row(
                    self,
                    buf,
                    y,
                    msg::FIELD_FINISH_BY.get(),
                    Focus::TargetCombo,
                    hits,
                );
            }
            FlowKind::Finish | FlowKind::Resume => {}
        }
    }

    fn render_preview(
        &self,
        buf: &mut Buffer,
        inner: Rect,
        bottom: u16,
        preview: &FlowPreview,
        hits: &mut Vec<(Rect, Focus)>,
    ) {
        let mut y = inner.y;
        let heading = Style::default()
            .fg(th().ui.accent)
            .bg(th().ui.panel)
            .add_modifier(Modifier::BOLD);
        for why in &preview.blockers {
            if y >= bottom {
                return;
            }
            dialog_kit::put(
                buf,
                inner.x,
                y,
                why,
                inner.width,
                Style::default().fg(th().status.danger).bg(th().ui.panel),
            );
            y += 1;
        }
        if !preview.steps.is_empty() && y < bottom {
            dialog_kit::put(
                buf,
                inner.x,
                y,
                msg::STEPS_HEADING.get(),
                inner.width,
                heading,
            );
            y += 1;
        }
        for (i, step) in preview.steps.iter().enumerate() {
            if y >= bottom {
                break;
            }
            let (text, style) = step_line(i, step);
            dialog_kit::put(
                buf,
                inner.x + 1,
                y,
                &text,
                inner.width.saturating_sub(1),
                style,
            );
            y += 1;
        }
        let note = |y: &mut u16, buf: &mut Buffer, text: &str, style: Style| {
            if *y < bottom {
                dialog_kit::put(
                    buf,
                    inner.x + 1,
                    *y,
                    text,
                    inner.width.saturating_sub(1),
                    style,
                );
                *y += 1;
            }
        };
        if !preview.irreversible.is_empty() {
            y += 1;
            note(
                &mut y,
                buf,
                msg::IRREVERSIBLE_HEADING.get(),
                Style::default()
                    .fg(th().status.warning)
                    .bg(th().ui.panel)
                    .add_modifier(Modifier::BOLD),
            );
            for line in &preview.irreversible {
                note(
                    &mut y,
                    buf,
                    &format!("! {line}"),
                    Style::default().fg(th().status.warning).bg(th().ui.panel),
                );
            }
        }
        for line in &preview.warnings {
            note(
                &mut y,
                buf,
                line,
                Style::default().fg(th().ui.muted).bg(th().ui.panel),
            );
        }
        if !preview.hooks.is_empty() && y + 2 < bottom {
            y += 1;
            dialog_kit::put(
                buf,
                inner.x,
                y,
                msg::HOOKS_WARNING.get(),
                inner.width,
                Style::default()
                    .fg(th().status.warning)
                    .bg(th().ui.panel)
                    .add_modifier(Modifier::BOLD),
            );
            y += 1;
            for hook in &preview.hooks {
                if y + 1 >= bottom {
                    break;
                }
                dialog_kit::put(
                    buf,
                    inner.x + 3,
                    y,
                    &format!("$ {hook}"),
                    inner.width.saturating_sub(3),
                    Style::default().fg(th().ui.text).bg(th().ui.panel),
                );
                y += 1;
            }
            if untrusted(preview) {
                // No checkbox: the question comes when the action runs.
                dialog_kit::put(
                    buf,
                    inner.x + 1,
                    y,
                    gitpoulpi_i18n::msg::flow_hook_trust::UNTRUSTED_NOTE.get(),
                    inner.width.saturating_sub(1),
                    Style::default().fg(th().ui.muted).bg(th().ui.panel),
                );
                return;
            }
            let r = dialog_kit::checkbox(
                buf,
                inner.x + 1,
                y,
                inner.width.saturating_sub(1),
                msg::RUN_HOOKS.get(),
                self.run_hooks,
                CheckState::Enabled {
                    hovered: self.hover == Some(Focus::Hooks),
                    focused: self.focus == Focus::Hooks,
                },
            );
            hits.push((r, Focus::Hooks));
        }
    }

    fn render_steps(
        &self,
        buf: &mut Buffer,
        inner: Rect,
        bottom: u16,
        run: &Run,
        failure: Option<(&String, &str)>,
    ) {
        let mut y = inner.y;
        for (i, step) in run.steps.iter().enumerate() {
            if y >= bottom {
                break;
            }
            let (mark, color) = match run.state.get(i).copied().unwrap_or(StepState::Pending) {
                StepState::Pending => ("[ ]".to_string(), th().ui.muted),
                StepState::Running(since) => {
                    // In-flight: the spinner only after 150 ms.
                    if since.elapsed() >= SPIN_AFTER {
                        (
                            format!("[{}]", crate::activity::spinner_frame(since.elapsed())),
                            th().ui.accent,
                        )
                    } else {
                        ("[>]".to_string(), th().ui.accent)
                    }
                }
                StepState::Done => ("[x]".to_string(), th().status.success),
                StepState::Failed => ("[!]".to_string(), th().status.danger),
            };
            let style = match step.kind {
                StepKind::Destructive => Style::default().fg(th().status.warning),
                _ => Style::default().fg(th().ui.text),
            };
            dialog_kit::put(
                buf,
                inner.x + 1,
                y,
                &mark,
                3,
                Style::default().fg(color).bg(th().ui.panel),
            );
            dialog_kit::put(
                buf,
                inner.x + 5,
                y,
                &step.text,
                inner.width.saturating_sub(5),
                style.bg(th().ui.panel),
            );
            y += 1;
        }
        if let Some((message, help)) = failure {
            y += 1;
            if y < bottom {
                dialog_kit::put(
                    buf,
                    inner.x,
                    y,
                    message,
                    inner.width,
                    Style::default().fg(th().status.danger).bg(th().ui.panel),
                );
            }
            if !help.is_empty() && y + 1 < bottom {
                dialog_kit::put(
                    buf,
                    inner.x,
                    y + 1,
                    help,
                    inner.width,
                    Style::default().fg(th().ui.muted).bg(th().ui.panel),
                );
            }
        }
    }
}

/// `1. text`, destructive steps in the warning colour with a marker.
fn step_line(index: usize, step: &FlowStep) -> (String, Style) {
    let base = Style::default().bg(th().ui.panel);
    match step.kind {
        StepKind::Normal => (
            format!("{}. {}", index + 1, step.text),
            base.fg(th().ui.text),
        ),
        StepKind::Hook => (
            format!("{}. $ {}", index + 1, step.text),
            base.fg(th().ui.muted),
        ),
        StepKind::Destructive => (
            format!("{}. ! {}", index + 1, step.text),
            base.fg(th().status.warning).add_modifier(Modifier::BOLD),
        ),
    }
}

/// The dialog's rect: wide enough for a branch name and a step list.
pub(crate) fn flow_rect(frame: Rect) -> Rect {
    dialog_kit::centered(frame, frame.width.saturating_sub(4).min(96), 26)
}
