//! Host-driven flows in the TUI (plan `branching-workflows.md`, slice W4):
//! "Flow: New Issue <type>..." (the issue made from the workflow's template,
//! then the branch the host or a bot makes), the waiting card (spinner,
//! elapsed, stop), "Flow: Pick Up..." and the "a branch appeared" card the
//! next fetch raises for an issue still being watched.
//!
//! Every host call runs on its own thread over the shared engine
//! (`gitpoulpi_hosting::flow_engine`), answers come back on a channel polled
//! every frame, so the render thread never waits; a call or wait that takes
//! longer than 150 ms shows a spinner. The git half (fetch, create the
//! tracking branch, record the parent, check out) is one `FlowRun` request of
//! action `pick_up`, journaled like every flow action, behind [`PickUp`] so
//! tests need no daemon.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use gitpoulpi_core::workflow::{IssueFlow, IssueFlowMode, WaitFor};
use gitpoulpi_hosting::flow_engine::{self, Found, HostMade};
use gitpoulpi_hosting::{CreatedIssue, HostingClient, Integration};
use gitpoulpi_i18n::msg::flow as msg;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use super::view::FlowView;
use crate::combo::{Combo, ComboOption, ComboOutcome};
use crate::dialog_kit::{self, ButtonState, CheckState};
use crate::hosting_task::ClientFactory;
use crate::text_edit::FieldEditor;
use crate::th;

/// A request in flight for longer than this shows a spinner.
const SPIN_AFTER: Duration = Duration::from_millis(150);

/// The integration and project one half of the flow talks to.
#[derive(Clone)]
pub(crate) struct Target {
    pub(crate) integration: Integration,
    pub(crate) project: String,
}

/// What takes a branch over: fetch, track, record the parent, check out.
/// Production sends daemon requests; tests answer by hand. The answer is the
/// status line, or why it failed.
pub(crate) trait PickUp: Send + Sync {
    fn pick_up(
        &self,
        remote: &str,
        branch: &str,
        parent: Option<&str>,
    ) -> Receiver<Result<String, String>>;
}

/// What the window knows when it opens the dialog.
#[derive(Clone)]
pub(crate) struct IssueFlowSeed {
    pub(crate) view: Arc<FlowView>,
    pub(crate) kind: IssueFlowKind,
    pub(crate) issue_target: Option<Target>,
    pub(crate) code_target: Option<Target>,
    pub(crate) integrations: Vec<Integration>,
    pub(crate) factory: Arc<dyn ClientFactory>,
    pub(crate) picker: Arc<dyn PickUp>,
    /// The default remote's name, and the branches it has (without it).
    pub(crate) remote: String,
    pub(crate) remote_branches: Vec<String>,
    pub(crate) repo_root: PathBuf,
    /// The repository's `repo-state.toml`, where watched issues live.
    pub(crate) state_path: PathBuf,
    pub(crate) user: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum IssueFlowKind {
    /// New issue; `Some(i)` fixes the type (the i-th of the workflow).
    New(Option<usize>),
    /// Take over an issue's, a request's or a branch's work.
    PickUp,
    /// A fetch brought the branch of a watched issue.
    Appeared { number: u64, branches: Vec<String> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    TypeCombo,
    Title,
    Reference,
    PickAfter,
    Primary,
    Secondary,
}

/// What the dialog asks its owner for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum IssueFlowOutcome {
    Stay,
    Close,
}

enum Work {
    Creating,
    Making,
    Looking,
}

enum Phase {
    Form,
    Working(Work, Instant),
    Waiting {
        since: Instant,
        number: u64,
        wait: WaitFor,
        max: Duration,
    },
    Found {
        number: Option<u64>,
        found: Found,
    },
    Picking(Instant),
    Done(String),
    Failed(String),
}

struct Created {
    issue: CreatedIssue,
}

enum Msg {
    Created(Result<Created, String>),
    Made(Result<HostMade, String>),
    Waited(Result<(Found, bool), String>),
    Looked(Result<(Option<u64>, Found), String>),
}

pub(crate) struct IssueFlowDialog {
    seed: IssueFlowSeed,
    kind: IssueFlowKind,
    phase: Phase,
    focus: Focus,
    hover: Option<Focus>,
    message: Option<String>,
    topic: usize,
    type_combo: Combo,
    title: String,
    reference: String,
    pick_after: bool,
    rx: Option<Receiver<Msg>>,
    pick_rx: Option<Receiver<Result<String, String>>>,
    /// The watched issue a running pick-up settles.
    pick_number: Option<u64>,
    cancel: Arc<AtomicBool>,
    /// The issue made, when it was.
    issue: Option<Created>,
    hits: Vec<(Rect, Focus)>,
    edit: FieldEditor<Focus>,
    /// The close cross on the frame.
    close: dialog_kit::CloseHit,
}

fn flow_of(seed: &IssueFlowSeed) -> IssueFlow {
    seed.view
        .report
        .workflow
        .issue_flow
        .clone()
        .unwrap_or_default()
}

fn runtime() -> Result<tokio::runtime::Runtime, String> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())
}

fn client_of(seed: &IssueFlowSeed, target: &Target) -> Result<Box<dyn HostingClient>, String> {
    seed.factory
        .client(&target.integration, &seed.integrations)
        .map_err(|e| crate::hosting_task::error_text(&e))
}

fn type_ids(seed: &IssueFlowSeed) -> Vec<String> {
    seed.view.topics().iter().map(|t| t.id.clone()).collect()
}

/// `#12`, `12`, `!5` or a branch name.
enum Reference {
    Issue(u64),
    Request(u64),
    Branch(String),
}

fn parse_reference(text: &str) -> Option<Reference> {
    let t = text.trim();
    if t.is_empty() {
        return None;
    }
    if let Some(n) = t
        .strip_prefix('#')
        .or(Some(t).filter(|s| s.bytes().all(|b| b.is_ascii_digit())))
    {
        if let Ok(n) = n.trim().parse::<u64>() {
            return Some(Reference::Issue(n));
        }
    }
    if let Some(n) = t.strip_prefix('!') {
        if let Ok(n) = n.trim().parse::<u64>() {
            return Some(Reference::Request(n));
        }
    }
    Some(Reference::Branch(t.to_string()))
}

/// Records / forgets a watched issue in the repository state file.
fn watch(
    path: &std::path::Path,
    issue: Option<gitpoulpi_session::WatchedIssue>,
    forget: Option<u64>,
) {
    if let Ok(mut state) = gitpoulpi_session::RepoState::load(path) {
        if let Some(i) = issue {
            state.watch_issue(i);
        }
        if let Some(n) = forget {
            state.unwatch_issue(n);
        }
        let _ = state.save(path);
    }
}

impl IssueFlowDialog {
    /// What `Ctrl+N` badges: every field, combobox and button; nothing while the list is open.
    pub(crate) fn nav_targets(&self) -> Vec<(Rect, (u16, u16))> {
        if self.type_combo.is_open() {
            return Vec::new();
        }
        dialog_kit::nav_cells(self.hits.iter().map(|(r, _)| *r))
    }

    pub(crate) fn new(seed: IssueFlowSeed) -> Self {
        let kind = seed.kind.clone();
        let topic = match &kind {
            IssueFlowKind::New(Some(i)) => (*i).min(seed.view.topics().len().saturating_sub(1)),
            _ => 0,
        };
        let mut dialog = Self {
            kind: kind.clone(),
            phase: Phase::Form,
            focus: Focus::Primary,
            hover: None,
            message: None,
            topic,
            type_combo: Combo::new(""),
            title: String::new(),
            reference: String::new(),
            pick_after: true,
            rx: None,
            pick_rx: None,
            pick_number: None,
            cancel: Arc::new(AtomicBool::new(false)),
            issue: None,
            hits: Vec::new(),
            close: dialog_kit::CloseHit::default(),
            edit: Default::default(),
            seed,
        };
        dialog.type_combo.set_options(
            dialog
                .seed
                .view
                .topics()
                .iter()
                .enumerate()
                .map(|(i, t)| ComboOption::new(t.display_label(), i.to_string()))
                .collect(),
        );
        dialog.type_combo.select(Some(dialog.topic.to_string()));
        if let IssueFlowKind::Appeared { number, branches } = &kind {
            dialog.phase = Phase::Found {
                number: Some(*number),
                found: Found {
                    branches: branches.clone(),
                    ..Found::default()
                },
            };
        }
        dialog.focus = dialog.first_focus();
        dialog
    }

    fn flow(&self) -> IssueFlow {
        flow_of(&self.seed)
    }

    fn mode(&self) -> IssueFlowMode {
        self.flow().mode
    }

    // -- the work ------------------------------------------------------------

    fn spawn(&mut self, work: impl FnOnce(Sender<Msg>) + Send + 'static) {
        let (tx, rx) = channel();
        self.rx = Some(rx);
        std::thread::spawn(move || work(tx));
    }

    fn create_issue(&mut self) {
        let title = self.title.trim().to_string();
        if title.is_empty() {
            self.message = Some(msg::ISSUE_NEED_TITLE.get().into());
            return;
        }
        let Some(target) = self.seed.issue_target.clone() else {
            self.message = Some(msg::ISSUE_WHY_NO_TRACKER.get().into());
            return;
        };
        let Some(topic) = self.seed.view.topics().get(self.topic).cloned() else {
            return;
        };
        let seed = self.seed.clone();
        let flow = self.flow();
        let user = seed.user.clone().unwrap_or_default();
        self.message = None;
        self.phase = Phase::Working(Work::Creating, Instant::now());
        self.spawn(move |tx| {
            let result = (|| {
                let template = flow.description_template.as_deref().map(|rel| {
                    std::fs::read_to_string(seed.repo_root.join(rel)).unwrap_or_default()
                });
                let draft = gitpoulpi_core::workflow::issue_draft(
                    &flow,
                    &topic.id,
                    &title,
                    template.as_deref(),
                    &user,
                    &gitpoulpi_core::time::today(),
                );
                let client = client_of(&seed, &target)?;
                let issue = runtime()?
                    .block_on(flow_engine::create_issue(
                        &*client,
                        &target.project,
                        &draft,
                        None,
                    ))
                    .map_err(|e| crate::hosting_task::error_text(&e))?;
                watch(
                    &seed.state_path,
                    Some(gitpoulpi_session::WatchedIssue {
                        number: issue.number,
                        type_id: topic.id.clone(),
                        title: draft.title.clone(),
                        created: gitpoulpi_core::time::unix_now(),
                    }),
                    None,
                );
                Ok(Created { issue })
            })();
            let _ = tx.send(Msg::Created(result));
        });
    }

    /// The issue exists: what the mode does next.
    fn after_created(&mut self, created: Created) {
        self.message = Some(msg::issue_created(&created.issue.reference));
        let number = created.issue.number;
        let flow = self.flow();
        let topic = self
            .seed
            .view
            .topics()
            .get(self.topic)
            .cloned()
            .unwrap_or_default();
        let title = self.title.trim().to_string();
        self.issue = Some(created);
        match flow.mode {
            IssueFlowMode::Local => {
                self.phase = Phase::Done(self.message.take().unwrap_or_default());
            }
            IssueFlowMode::HostCreatesBranch => {
                let Some(code) = self
                    .seed
                    .code_target
                    .clone()
                    .or(self.seed.issue_target.clone())
                else {
                    return;
                };
                let compiled = self.seed.view.compiled();
                let remote_branches = self.seed.remote_branches.clone();
                let from = topic.parent.as_deref().and_then(|p| {
                    compiled.concrete_base(p, &|n| remote_branches.iter().any(|b| b == n))
                });
                let Some(from) = from else {
                    self.phase = Phase::Failed(msg::ISSUE_PARENT_MISSING.get().into());
                    return;
                };
                let seed = self.seed.clone();
                self.phase = Phase::Working(Work::Making, Instant::now());
                let topic_id = topic.id.clone();
                let open_request = flow.wants_merge_request();
                self.spawn(move |tx| {
                    let result = (|| {
                        let name =
                            flow_engine::issue_branch_name(&flow, &topic_id, number, &title)?;
                        let client = client_of(&seed, &code)?;
                        runtime()?
                            .block_on(flow_engine::host_creates_branch(
                                &*client,
                                &code.project,
                                number,
                                &name,
                                &from,
                                &from,
                                &title,
                                open_request,
                            ))
                            .map_err(|e| crate::hosting_task::error_text(&e))
                    })();
                    let _ = tx.send(Msg::Made(result));
                });
            }
            IssueFlowMode::Bot => self.begin_wait(number, flow),
        }
    }

    fn begin_wait(&mut self, number: u64, flow: IssueFlow) {
        let Some(code) = self
            .seed
            .code_target
            .clone()
            .or(self.seed.issue_target.clone())
        else {
            return;
        };
        let wait = flow.wait_kind();
        let max = flow.timeout();
        let seed = self.seed.clone();
        let types = type_ids(&seed);
        self.cancel = Arc::new(AtomicBool::new(false));
        let cancel = Arc::clone(&self.cancel);
        self.phase = Phase::Waiting {
            since: Instant::now(),
            number,
            wait,
            max,
        };
        self.spawn(move |tx| {
            let result = (|| {
                let client = client_of(&seed, &code)?;
                let rt = runtime()?;
                let started = Instant::now();
                let mut attempt = 0u32;
                let mut last = Found::default();
                loop {
                    if cancel.load(Ordering::SeqCst) {
                        return Err(String::new());
                    }
                    match rt.block_on(flow_engine::find_for_issue(
                        &*client,
                        &code.project,
                        &flow,
                        &types,
                        number,
                    )) {
                        Ok(found) => {
                            if found.satisfies(wait) {
                                return Ok((found, false));
                            }
                            last = found;
                        }
                        Err(e) if attempt == 0 => {
                            return Err(crate::hosting_task::error_text(&e));
                        }
                        Err(_) => {}
                    }
                    let pause = flow_engine::backoff(attempt);
                    if started.elapsed() + pause > max {
                        return Ok((last, true));
                    }
                    // Sleep in slices so a stop is heard at once.
                    let until = Instant::now() + pause;
                    while Instant::now() < until {
                        if cancel.load(Ordering::SeqCst) {
                            return Err(String::new());
                        }
                        std::thread::sleep(Duration::from_millis(25));
                    }
                    attempt += 1;
                }
            })();
            // A stop sends nothing: the dialog is gone.
            if matches!(&result, Err(e) if e.is_empty()) {
                return;
            }
            let _ = tx.send(Msg::Waited(result));
        });
    }

    fn look_up(&mut self) {
        let Some(reference) = parse_reference(&self.reference) else {
            self.message = Some(msg::ISSUE_NEED_REFERENCE.get().into());
            return;
        };
        self.message = None;
        match reference {
            Reference::Branch(name) => {
                self.phase = Phase::Found {
                    number: None,
                    found: Found {
                        branches: vec![name],
                        ..Found::default()
                    },
                };
                self.focus = Focus::Primary;
            }
            Reference::Issue(_) | Reference::Request(_) => {
                let Some(code) = self
                    .seed
                    .code_target
                    .clone()
                    .or(self.seed.issue_target.clone())
                else {
                    self.message = Some(msg::ISSUE_WHY_NO_TRACKER.get().into());
                    return;
                };
                let seed = self.seed.clone();
                let flow = self.flow();
                let types = type_ids(&seed);
                self.phase = Phase::Working(Work::Looking, Instant::now());
                self.spawn(move |tx| {
                    let result = (|| {
                        let client = client_of(&seed, &code)?;
                        let rt = runtime()?;
                        match reference {
                            Reference::Issue(n) => rt
                                .block_on(flow_engine::find_for_issue(
                                    &*client,
                                    &code.project,
                                    &flow,
                                    &types,
                                    n,
                                ))
                                .map(|f| (Some(n), f))
                                .map_err(|e| crate::hosting_task::error_text(&e)),
                            Reference::Request(n) => {
                                let detail = rt
                                    .block_on(client.pull_request_detail(&code.project, n))
                                    .map_err(|e| crate::hosting_task::error_text(&e))?;
                                let mut found = Found::default();
                                found.branches.push(detail.summary.source_branch.clone());
                                found.requests.push(detail.summary);
                                Ok((None, found))
                            }
                            Reference::Branch(_) => unreachable!("handled above"),
                        }
                    })();
                    let _ = tx.send(Msg::Looked(result));
                });
            }
        }
    }

    fn begin_pick(&mut self) {
        let Phase::Found { found, number } = &self.phase else {
            return;
        };
        self.pick_number = number
            .or_else(|| self.issue.as_ref().map(|c| c.issue.number))
            .or(match &self.kind {
                IssueFlowKind::Appeared { number, .. } => Some(*number),
                _ => None,
            });
        let Some(branch) = found.best_branch().map(str::to_string) else {
            self.message = Some(msg::ISSUE_FOUND_NONE.get().into());
            return;
        };
        let parent = found.request_of(&branch).map(|r| r.target_branch.clone());
        self.pick_rx = Some(self.seed.picker.pick_up(
            &self.seed.remote,
            &branch,
            parent.as_deref(),
        ));
        self.phase = Phase::Picking(Instant::now());
        self.message = None;
    }

    // -- polling ---------------------------------------------------------------

    /// A request or a wait is on its way (the spinner needs frames).
    pub(crate) fn animating(&self) -> bool {
        matches!(
            self.phase,
            Phase::Working(..) | Phase::Waiting { .. } | Phase::Picking(_)
        )
    }

    /// Drains the channels. Returns a status line when something ended.
    pub(crate) fn poll(&mut self) -> Option<String> {
        let mut said = None;
        let message = self.rx.as_ref().map(|rx| rx.try_recv());
        match message {
            Some(Ok(Msg::Created(Ok(created)))) => {
                self.rx = None;
                said = Some(msg::issue_created(&created.issue.reference));
                self.after_created(created);
            }
            Some(Ok(Msg::Created(Err(why)))) => {
                self.rx = None;
                self.phase = Phase::Form;
                self.message = Some(why);
            }
            Some(Ok(Msg::Made(Ok(made)))) => {
                self.rx = None;
                let mut found = Found {
                    branches: vec![made.branch.name.clone()],
                    ..Found::default()
                };
                let mut line = msg::issue_made(&made.branch.name);
                if let Some(request) = &made.request {
                    line = format!("{line}. {}", msg::issue_request_opened(request.number));
                    found.requests.push(gitpoulpi_hosting::PullRequestSummary {
                        number: request.number,
                        iid_label: format!("#{}", request.number),
                        source_branch: made.branch.name.clone(),
                        target_branch: String::new(),
                        web_url: request.web_url.clone(),
                        ..Default::default()
                    });
                }
                if let Some(w) = &made.warning {
                    line = format!("{line} ({w})");
                }
                said = Some(line);
                self.finish_found(found);
            }
            Some(Ok(Msg::Waited(Ok((found, timed_out))))) => {
                self.rx = None;
                if timed_out {
                    let secs = match &self.phase {
                        Phase::Waiting { since, .. } => since.elapsed().as_secs(),
                        _ => 0,
                    };
                    let line = msg::issue_timed_out(secs);
                    said = Some(line.clone());
                    self.phase = Phase::Failed(line);
                } else {
                    self.finish_found(found);
                }
            }
            Some(Ok(Msg::Waited(Err(why)))) => {
                self.rx = None;
                self.phase = Phase::Failed(why);
            }
            Some(Ok(Msg::Looked(Ok((number, found))))) => {
                self.rx = None;
                self.phase = Phase::Found { number, found };
                self.focus = Focus::Primary;
            }
            Some(Ok(Msg::Looked(Err(why)))) => {
                self.rx = None;
                self.phase = Phase::Form;
                self.message = Some(why);
            }
            Some(Err(TryRecvError::Disconnected)) => {
                self.rx = None;
            }
            _ => {}
        }
        if let Some(rx) = &self.pick_rx {
            match rx.try_recv() {
                Ok(Ok(line)) => {
                    self.pick_rx = None;
                    if let Some(n) = self.pick_number.take() {
                        let path = self.seed.state_path.clone();
                        std::thread::spawn(move || watch(&path, None, Some(n)));
                    }
                    said = Some(line.clone());
                    self.phase = Phase::Done(line);
                    self.focus = Focus::Primary;
                }
                Ok(Err(why)) => {
                    self.pick_rx = None;
                    said = Some(why.clone());
                    self.phase = Phase::Failed(why);
                }
                Err(TryRecvError::Disconnected) => {
                    self.pick_rx = None;
                    self.phase = Phase::Failed(msg::LOST.get().into());
                }
                Err(TryRecvError::Empty) => {}
            }
        }
        said
    }

    /// What the host made: shown, and picked up at once when the form asked.
    fn finish_found(&mut self, found: Found) {
        let number = self.issue.as_ref().map(|c| c.issue.number);
        self.phase = Phase::Found { number, found };
        self.focus = Focus::Primary;
        if self.pick_after {
            self.begin_pick();
        }
    }

    // -- focus -----------------------------------------------------------------

    fn order(&self) -> Vec<Focus> {
        let mut order = Vec::new();
        match &self.phase {
            Phase::Form => {
                match self.kind {
                    IssueFlowKind::New(_) => {
                        order.push(Focus::TypeCombo);
                        order.push(Focus::Title);
                        if self.mode() != IssueFlowMode::Local {
                            order.push(Focus::PickAfter);
                        }
                    }
                    _ => order.push(Focus::Reference),
                }
                order.push(Focus::Primary);
                order.push(Focus::Secondary);
            }
            Phase::Working(..) | Phase::Picking(_) => {}
            Phase::Waiting { .. } => order.push(Focus::Secondary),
            Phase::Found { .. } => {
                order.push(Focus::Primary);
                order.push(Focus::Secondary);
            }
            Phase::Done(_) | Phase::Failed(_) => order.push(Focus::Secondary),
        }
        order
    }

    fn first_focus(&self) -> Focus {
        let order = self.order();
        order
            .iter()
            .copied()
            .find(|f| matches!(f, Focus::Title | Focus::Reference))
            .or_else(|| order.first().copied())
            .unwrap_or(Focus::Primary)
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

    fn field_parts(&mut self) -> (&mut FieldEditor<Focus>, Option<(Focus, &mut String)>) {
        let focus = self.focus;
        let field = match (focus, &self.phase) {
            (Focus::Title, Phase::Form) => Some(&mut self.title),
            (Focus::Reference, Phase::Form) => Some(&mut self.reference),
            _ => None,
        };
        self.edit.sync(field.is_some().then_some(focus));
        (&mut self.edit, field.map(|f| (focus, f)))
    }

    // -- activation ------------------------------------------------------------

    fn primary(&mut self) -> IssueFlowOutcome {
        match &self.phase {
            Phase::Form => {
                match self.kind {
                    IssueFlowKind::New(_) => self.create_issue(),
                    _ => self.look_up(),
                }
                IssueFlowOutcome::Stay
            }
            Phase::Found { .. } => {
                self.begin_pick();
                IssueFlowOutcome::Stay
            }
            _ => IssueFlowOutcome::Stay,
        }
    }

    /// The secondary button: cancel, stop waiting, later, close.
    fn secondary(&mut self) -> IssueFlowOutcome {
        match &self.phase {
            Phase::Working(..) | Phase::Picking(_) => IssueFlowOutcome::Stay,
            Phase::Waiting { .. } => {
                // The issue stays watched: the next fetch tells.
                self.cancel.store(true, Ordering::SeqCst);
                IssueFlowOutcome::Close
            }
            _ => IssueFlowOutcome::Close,
        }
    }

    fn activate(&mut self, focus: Focus) -> IssueFlowOutcome {
        match focus {
            Focus::Primary => self.primary(),
            Focus::Secondary => self.secondary(),
            Focus::PickAfter => {
                self.pick_after = !self.pick_after;
                IssueFlowOutcome::Stay
            }
            Focus::TypeCombo => {
                self.type_combo.open();
                IssueFlowOutcome::Stay
            }
            Focus::Title | Focus::Reference => self.primary(),
        }
    }

    fn combo_outcome(&mut self, outcome: ComboOutcome) {
        if let ComboOutcome::Picked(value) | ComboOutcome::Custom(value) = outcome {
            if let Ok(i) = value.parse::<usize>() {
                self.topic = i;
            }
        }
    }

    // -- input -----------------------------------------------------------------

    pub(crate) fn paste(&mut self, text: &str) {
        if self.type_combo.is_open() {
            self.type_combo.paste(text);
            return;
        }
        let clean: String = text.chars().filter(|c| !c.is_control()).collect();
        if let (edit, Some((id, field))) = self.field_parts() {
            edit.caret_mut(id).insert(field, &clean);
        }
    }

    pub(crate) fn on_key(&mut self, key: KeyEvent) -> IssueFlowOutcome {
        if self.type_combo.is_open() {
            let outcome = self.type_combo.on_key(key);
            self.combo_outcome(outcome);
            return IssueFlowOutcome::Stay;
        }
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        if let (edit, Some((id, field))) = self.field_parts() {
            if let Some((result, copied)) = edit.dialog_key(id, field, &key) {
                if result == crate::text_edit::KeyEdit::Edited {
                    self.message = None;
                }
                if copied.is_some() {
                    self.message = copied;
                }
                return IssueFlowOutcome::Stay;
            }
        }
        match key.code {
            KeyCode::Esc => return self.secondary(),
            KeyCode::Tab | KeyCode::BackTab | KeyCode::Down | KeyCode::Up => {
                let forward =
                    matches!(key.code, KeyCode::Down) || (key.code == KeyCode::Tab && !shift);
                self.step_focus(forward);
            }
            KeyCode::Enter => return self.activate(self.focus),
            KeyCode::Char(' ') if !matches!(self.focus, Focus::Title | Focus::Reference) => {
                return self.activate(self.focus);
            }
            _ => {}
        }
        IssueFlowOutcome::Stay
    }

    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent) -> IssueFlowOutcome {
        if self.type_combo.is_open() {
            let outcome = self.type_combo.on_mouse(mouse);
            self.combo_outcome(outcome);
            return IssueFlowOutcome::Stay;
        }
        let hit = dialog_kit::hit_at(&self.hits, mouse.column, mouse.row);
        if matches!(mouse.kind, MouseEventKind::Moved | MouseEventKind::Drag(_)) {
            self.close.set_hover(mouse.column, mouse.row);
        }
        if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left))
            && self.close.hit(mouse.column, mouse.row)
        {
            return self.secondary();
        }
        if !matches!(mouse.kind, MouseEventKind::Down(_)) {
            let handled = match self.field_parts() {
                (edit, Some((id, field))) => {
                    edit.dialog_mouse(Some(id), field, mouse.kind, mouse.column, mouse.row)
                }
                (edit, None) => edit.dialog_mouse(None, "", mouse.kind, mouse.column, mouse.row),
            };
            if handled {
                return IssueFlowOutcome::Stay;
            }
        }
        match mouse.kind {
            MouseEventKind::Moved | MouseEventKind::Drag(_) => self.hover = hit,
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(part) = hit {
                    self.focus = part;
                    if matches!(part, Focus::Title | Focus::Reference) {
                        if let (edit, Some((id, field))) = self.field_parts() {
                            edit.dialog_mouse(Some(id), field, mouse.kind, mouse.column, mouse.row);
                        }
                    } else {
                        return self.activate(part);
                    }
                }
            }
            _ => {}
        }
        IssueFlowOutcome::Stay
    }
}

mod render;
pub(crate) use render::issue_flow_rect;

/// Production [`PickUp`]: a fetch, then the `pick_up` flow action, as daemon
/// requests. A thread polls the replies (they are polled, not awaited), so
/// the render thread never waits.
pub(crate) struct DaemonPickUp {
    pub(crate) requester: crate::daemon_link::Requester,
}

fn wait_reply<T>(reply: &crate::daemon_link::Reply<T>) -> Result<T, String> {
    let give_up = Instant::now() + Duration::from_secs(600);
    loop {
        if let Some(answer) = reply.take() {
            return answer.map_err(|e| e.to_string());
        }
        if Instant::now() > give_up {
            return Err(gitpoulpi_i18n::msg::flow::LOST.get().to_string());
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

impl PickUp for DaemonPickUp {
    fn pick_up(
        &self,
        remote: &str,
        branch: &str,
        parent: Option<&str>,
    ) -> Receiver<Result<String, String>> {
        let (tx, rx) = channel();
        let requester = self.requester.clone();
        let (remote, branch, parent) = (
            remote.to_string(),
            branch.to_string(),
            parent.map(str::to_string),
        );
        std::thread::spawn(move || {
            use gitpoulpi_core::domain::{FetchOptions, FetchOutcome, OperationKind};
            use gitpoulpi_core::workflow::{FlowOutcome, FlowStatus};
            let result = (|| {
                let fetch = requester.call::<FetchOutcome>(
                    OperationKind::Fetch,
                    serde_json::json!({ "remote": remote, "opts": FetchOptions::default() }),
                );
                wait_reply(&fetch)?;
                let run = requester.call::<FlowOutcome>(
                    OperationKind::FlowRun,
                    serde_json::json!({
                        "action": "pick_up",
                        "branch": branch,
                        "parent": parent,
                        "remote": remote,
                    }),
                );
                let outcome = wait_reply(&run)?;
                match outcome.status {
                    FlowStatus::Done | FlowStatus::NothingToDo => Ok(msg::issue_picked(&branch)),
                    _ => Err(outcome.message),
                }
            })();
            let _ = tx.send(result);
            crate::redraw::poke();
        });
        rx
    }
}

#[cfg(test)]
mod tests;
