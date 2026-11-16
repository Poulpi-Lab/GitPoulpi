//! "Write with AI" inside the Markdown composer.
//!
//! **The AI only drafts.** The helper turns what the user typed at the moment
//! of use ("what I want in this answer") into a request, runs it off the
//! render thread (`ai_worker`) and hands the text back to the editor, which
//! fills itself in one undo step (`Ctrl+Z` gives the previous text back) and
//! switches to Preview so the draft is read before it is sent. Nothing in
//! here posts, edits or deletes on a host: the text goes out only through the
//! form's own Send.
//!
//! The panel sits under the mode bar while open: the instruction field (with
//! a Recent list: click one, or `Up` / `Down` in the field), the "Improve my
//! draft" box, the "Let the AI read the repository" box (unticked, and
//! disabled with its reason until the repository-context slice lands), the
//! Draft / Stop / Close buttons and the line that says what is going on.
//! Every mouse action has a key: `Alt+E` opens and closes the panel, `Enter`
//! in the field drafts, `Esc` stops a run, then closes, `Tab` walks the parts.
//!
//! What a host's text is a draft *of* comes from an [`AiContextSource`]: the
//! form that owns the editor says which conversation, thread or branch pair
//! the text is about. The conversation view implements it for its replies.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use gitpoulpi_i18n::msg::md_editor as msg;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use super::ai_context;
use crate::dialog_kit::{self, ButtonState, CheckState};
use crate::text_edit::FieldEditor;
use crate::th;

#[cfg(feature = "ai")]
pub(crate) use gitpoulpi_ai::hosting_reply::{Exchange, ReplyRequest};

/// A build without AI never builds a request: the type only has to exist.
#[cfg(not(feature = "ai"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReplyRequest;

/// Likewise: no exchange is ever kept without AI.
#[cfg(not(feature = "ai"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Exchange;

/// State a form shares with the source it handed the editor (the issue
/// form's title, the pull request form's branches).
pub(crate) type Shared<T> = Arc<Mutex<T>>;

/// Rows the open panel takes at most.
const PANEL_ROWS: u16 = 5;
/// Rows of text the panel always leaves to the editor.
const KEEP_TEXT_ROWS: u16 = 3;
/// Instructions remembered.
const RECENT_MAX: usize = 8;
/// Cells of one Recent entry.
const RECENT_CELLS: usize = 22;

/// What to draft.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AiTask {
    /// A comment, a thread answer, a review comment or summary, an issue
    /// description (`AiAction::HostingReply` / `AiAction::IssueBody`).
    Reply(Box<ReplyRequest>),
    /// The description of a pull request from `source` into `target`
    /// (`AiAction::PullRequestDescription`): the daemon reads the commits and
    /// the diff.
    PullRequest {
        source: String,
        target: String,
        hint: String,
        draft: Option<String>,
    },
}

/// What a text is about, asked when the user runs the helper. The owner of
/// the editor implements it (the review's comment box and Finish summary, the
/// issue form and the pull request form here; the conversation view's reply).
pub(crate) trait AiContextSource {
    /// The request for `hint` (what the user wants in this text) and `draft`
    /// (their own text to improve; `None` writes a new one). `Err` is a
    /// sentence for the panel ("Pick the branches first").
    fn task(&self, hint: &str, draft: Option<&str>) -> Result<AiTask, String>;

    /// Whether "Let the AI read the repository" makes sense for this text
    /// (a pull request description is written from its diff instead).
    fn reads_repository(&self) -> bool {
        true
    }
}

/// Where a run reads its settings and instructions from.
#[derive(Debug, Clone, Default)]
pub(crate) struct AiScope {
    pub(crate) session: String,
    /// The repository the text is about, when there is a clone: its
    /// preferences and workspace instructions layer over the global ones, and
    /// a pull request description is read from it.
    pub(crate) root: Option<std::path::PathBuf>,
}

/// A fixed request template: the review's comment box and Finish summary
/// know the thread and the item when they open, so the request is built then.
#[cfg(feature = "ai")]
pub(crate) struct ReplyTemplate(pub(crate) ReplyRequest);

#[cfg(feature = "ai")]
impl AiContextSource for ReplyTemplate {
    fn task(&self, hint: &str, draft: Option<&str>) -> Result<AiTask, String> {
        let mut request = self.0.clone();
        request.hint = hint.to_string();
        request.draft = draft.map(str::to_string);
        Ok(AiTask::Reply(Box::new(request)))
    }
}

/// The new-issue form's description: the title the form holds right now.
pub(crate) struct IssueBodySource {
    pub(crate) title: Shared<String>,
}

impl AiContextSource for IssueBodySource {
    fn task(&self, hint: &str, draft: Option<&str>) -> Result<AiTask, String> {
        #[cfg(feature = "ai")]
        {
            use gitpoulpi_ai::hosting_reply::ReplyTarget;
            let title = self.title.lock().map(|t| t.clone()).unwrap_or_default();
            let mut request = ReplyRequest::new(ReplyTarget::IssueBody { title });
            request.hint = hint.to_string();
            request.draft = draft.map(str::to_string);
            Ok(AiTask::Reply(Box::new(request)))
        }
        #[cfg(not(feature = "ai"))]
        {
            let _ = (hint, draft, &self.title);
            Err(gitpoulpi_core::Error::NotBuilt { feature: "ai" }.to_string())
        }
    }
}

/// The pull request form's description: the branch pair the form holds now.
pub(crate) struct PullRequestSource {
    /// `(source, target)` branch names; empty until chosen.
    pub(crate) branches: Shared<(String, String)>,
}

impl AiContextSource for PullRequestSource {
    fn reads_repository(&self) -> bool {
        false
    }

    fn task(&self, hint: &str, draft: Option<&str>) -> Result<AiTask, String> {
        let (source, target) = self.branches.lock().map(|b| b.clone()).unwrap_or_default();
        if source.trim().is_empty() || target.trim().is_empty() {
            return Err(msg::AI_PICK_BRANCHES.get().to_string());
        }
        Ok(AiTask::PullRequest {
            source,
            target,
            hint: hint.to_string(),
            draft: draft.map(str::to_string),
        })
    }
}

/// What the panel's focus is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AiPart {
    /// The editor's own text: the panel keeps no focus.
    Text,
    Hint,
    Improve,
    Repo,
    Recent(usize),
    Run,
    Stop,
    Close,
    /// "Refresh context": forget what was kept and read the repository again.
    Refresh,
}

/// The parts `Tab` walks, in order, once the panel has the keyboard.
const WALK: [AiPart; 6] = [
    AiPart::Hint,
    AiPart::Improve,
    AiPart::Repo,
    AiPart::Run,
    AiPart::Close,
    AiPart::Refresh,
];

/// What a key meant to the panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AiKey {
    Handled,
    /// Not the panel's.
    Pass,
    /// Run the helper (the editor knows the text).
    Start,
    /// The panel closed.
    Closed,
}

/// What a mouse event meant to the panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AiMouse {
    Outside,
    Handled,
    Start,
    Closed,
}

enum Note {
    None,
    Info(String),
    Error(String),
}

pub(crate) type Answer = Result<String, String>;

/// A running request.
struct Run {
    rx: mpsc::Receiver<Answer>,
    cancel: Arc<AtomicBool>,
    started: Instant,
    /// What the worker says it read of the repository (see `ai_context`).
    context: Arc<Mutex<Option<String>>>,
    /// The instruction and the text to improve this run was asked with; its
    /// answer joins the exchange. `None` for a task that is not a reply.
    asked: Option<(String, Option<String>)>,
}

/// A test's stand-in for the model: the task in, the text out.
#[cfg(test)]
pub(crate) type Fake = Arc<dyn Fn(&AiTask) -> Answer + Send + Sync>;

pub(crate) struct AiHelp {
    scope: AiScope,
    source: Box<dyn AiContextSource>,
    open: bool,
    hint: String,
    edit: FieldEditor<()>,
    improve: bool,
    /// "Let the AI read the repository" is ticked (never without a clone).
    repo: bool,
    /// Whether that box can be ticked, and if not why.
    repo_choice: ai_context::Choice,
    /// What the last draft read of the repository.
    context_line: Option<String>,
    /// The finished requests of this sitting, oldest first. Given back to the
    /// model with the next one when it would not remember them (see
    /// `gitpoulpi_ai::hosting_reply::exchange`); lives as long as the
    /// composer, dropped by "Refresh context".
    exchange: Vec<Exchange>,
    focus: AiPart,
    hover: Option<AiPart>,
    /// Newest first.
    recent: VecDeque<String>,
    /// Where `Up` / `Down` in the field stand in the Recent list.
    recall: Option<usize>,
    run: Option<Run>,
    note: Note,
    hits: Vec<(Rect, AiPart)>,
    rect: Rect,
    /// Where the Recent file is kept (`None` under test: memory only).
    dir: Option<std::path::PathBuf>,
    #[cfg(test)]
    pub(crate) fake: Option<Fake>,
    /// Whether the last run was asked to read the repository (tests).
    #[cfg(test)]
    pub(crate) last_read_repo: Option<bool>,
}

impl Drop for AiHelp {
    fn drop(&mut self) {
        self.stop();
    }
}

// -- the Recent list -----------------------------------------------------------

const RECENT_FILE: &str = "ai-hints.toml";

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct RecentFile {
    #[serde(default)]
    recent: Vec<String>,
}

fn load_recent(dir: &std::path::Path) -> VecDeque<String> {
    std::fs::read_to_string(dir.join(RECENT_FILE))
        .ok()
        .and_then(|s| toml::from_str::<RecentFile>(&s).ok())
        .map(|f| f.recent.into_iter().take(RECENT_MAX).collect())
        .unwrap_or_default()
}

fn save_recent(dir: &std::path::Path, recent: &VecDeque<String>) {
    let file = RecentFile {
        recent: recent.iter().cloned().collect(),
    };
    let Ok(text) = toml::to_string(&file) else {
        return;
    };
    if std::fs::create_dir_all(dir).is_ok() {
        let _ = std::fs::write(dir.join(RECENT_FILE), text);
    }
}

impl AiHelp {
    pub(crate) fn new(scope: AiScope, source: Box<dyn AiContextSource>) -> Self {
        Self::new_in(scope, source, super::draft::config_dir())
    }

    pub(crate) fn new_in(
        scope: AiScope,
        source: Box<dyn AiContextSource>,
        dir: Option<std::path::PathBuf>,
    ) -> Self {
        let recent = dir.as_deref().map(load_recent).unwrap_or_default();
        Self {
            scope,
            source,
            open: false,
            hint: String::new(),
            edit: FieldEditor::default(),
            improve: false,
            repo: false,
            repo_choice: ai_context::Choice::Disabled(ai_context::Why::NoClone),
            context_line: None,
            exchange: Vec::new(),
            focus: AiPart::Text,
            hover: None,
            recent,
            recall: None,
            run: None,
            note: Note::None,
            hits: Vec::new(),
            rect: Rect::default(),
            dir,
            #[cfg(test)]
            fake: None,
            #[cfg(test)]
            last_read_repo: None,
        }
    }

    pub(crate) fn is_open(&self) -> bool {
        self.open
    }

    pub(crate) fn running(&self) -> bool {
        self.run.is_some()
    }

    /// Whether the panel has the keyboard (not the editor's text).
    pub(crate) fn has_focus(&self) -> bool {
        self.open && self.focus != AiPart::Text
    }

    /// The keyboard goes back to the editor's text (a click on it).
    pub(crate) fn focus_text(&mut self) {
        self.focus = AiPart::Text;
    }

    pub(crate) fn hint(&self) -> &str {
        &self.hint
    }

    pub(crate) fn improve(&self) -> bool {
        self.improve
    }

    pub(crate) fn recent(&self) -> Vec<String> {
        self.recent.iter().cloned().collect()
    }

    /// Opens the panel with the keyboard in the instruction field. "Improve
    /// my draft" starts ticked when there is a text to improve.
    pub(crate) fn open(&mut self, has_text: bool) {
        if !self.open {
            self.improve = has_text;
            self.note = Note::None;
            self.context_line = None;
            self.repo_choice = ai_context::offered(&self.scope, self.source.reads_repository());
            // Off unless the user turned the default on in Settings.
            self.repo = self.repo_choice == ai_context::Choice::Offered
                && ai_context::default_on(&self.scope);
        }
        self.open = true;
        self.focus = AiPart::Hint;
        self.edit.sync(Some(()));
    }

    /// Closes the panel; a run in progress is stopped.
    pub(crate) fn close(&mut self) {
        self.stop();
        self.open = false;
        self.focus = AiPart::Text;
        self.hits.clear();
        self.rect = Rect::default();
    }

    pub(crate) fn toggle(&mut self, has_text: bool) {
        if self.open {
            self.close();
        } else {
            self.open(has_text);
        }
    }

    /// Stops the run, if any.
    pub(crate) fn stop(&mut self) {
        if let Some(run) = self.run.take() {
            run.cancel.store(true, Ordering::Relaxed);
            self.note = Note::Info(msg::AI_STOPPED.get().to_string());
        }
    }

    /// Rows the panel takes in `avail` rows of editor body (0: none fits).
    pub(crate) fn height(&self, avail: u16) -> u16 {
        if !self.open {
            return 0;
        }
        let room = avail.saturating_sub(KEEP_TEXT_ROWS);
        if room < 2 {
            0
        } else {
            room.min(PANEL_ROWS + u16::from(self.shows_context_line()))
        }
    }

    /// The panel has a row of its own for what the last draft read of the
    /// repository.
    fn shows_context_line(&self) -> bool {
        self.repo_on() && self.context_line.is_some()
    }

    /// Makes the box offerable, as a repository with a clone and a trusted
    /// agent does, and says what the last draft read (tests).
    #[cfg(test)]
    pub(crate) fn offer_repo(&mut self, ticked: bool) {
        self.repo_choice = ai_context::Choice::Offered;
        self.repo = ticked;
    }

    #[cfg(test)]
    pub(crate) fn set_context_line(&mut self, line: &str) {
        self.context_line = Some(line.to_string());
    }

    /// The box is ticked and can be.
    pub(crate) fn repo_on(&self) -> bool {
        self.repo && self.repo_choice == ai_context::Choice::Offered
    }

    /// What the last draft read of the repository, when it did.
    #[cfg(test)]
    pub(crate) fn context_line(&self) -> Option<&str> {
        self.context_line.as_deref()
    }

    /// Where `part` was painted (tests).
    #[cfg(test)]
    pub(crate) fn rect_of(&self, part: AiPart) -> Option<Rect> {
        self.hits.iter().find(|(_, p)| *p == part).map(|(r, _)| *r)
    }

    /// Where the rects of the panel's controls are (for `Ctrl+N` badges).
    pub(crate) fn rects(&self) -> Vec<Rect> {
        self.hits.iter().map(|(r, _)| *r).collect()
    }

    // -- running ------------------------------------------------------------------

    /// Asks for a draft. `text` is the editor's text (improved when the box
    /// is ticked and there is some).
    pub(crate) fn start(&mut self, text: &str) {
        if self.run.is_some() {
            return;
        }
        let hint = self.hint.trim().to_string();
        let draft = (self.improve && !text.trim().is_empty()).then_some(text);
        #[allow(unused_mut)]
        let mut task = match self.source.task(&hint, draft) {
            Ok(task) => task,
            Err(why) => {
                self.note = Note::Error(msg::ai_error(&why));
                return;
            }
        };
        let asked = self.attach_exchange(&mut task, &hint, draft);
        if !hint.is_empty() {
            self.remember(&hint);
        }
        self.note = Note::None;
        let cancel = Arc::new(AtomicBool::new(false));
        #[cfg(test)]
        let fake = self.fake.clone();
        #[cfg(test)]
        {
            self.last_read_repo = Some(self.repo_on());
        }
        let context = Arc::new(Mutex::new(None));
        let rx = super::ai_worker::start(super::ai_worker::Input {
            scope: self.scope.clone(),
            task,
            cancel: cancel.clone(),
            read_repo: self.repo_on(),
            context: context.clone(),
            #[cfg(test)]
            fake,
        });
        self.run = Some(Run {
            rx,
            cancel,
            started: Instant::now(),
            context,
            asked,
        });
    }

    /// Gives a reply task what came before it in this sitting, and returns
    /// what the run is asked with, to record its answer under.
    #[cfg(feature = "ai")]
    fn attach_exchange(
        &self,
        task: &mut AiTask,
        hint: &str,
        draft: Option<&str>,
    ) -> Option<(String, Option<String>)> {
        match task {
            AiTask::Reply(request) => {
                request.exchange = self.exchange.clone();
                Some((hint.to_string(), draft.map(str::to_string)))
            }
            AiTask::PullRequest { .. } => None,
        }
    }

    #[cfg(not(feature = "ai"))]
    fn attach_exchange(
        &self,
        _: &mut AiTask,
        _: &str,
        _: Option<&str>,
    ) -> Option<(String, Option<String>)> {
        None
    }

    /// The answer of a finished run joins the exchange.
    #[cfg(feature = "ai")]
    fn record(&mut self, asked: Option<(String, Option<String>)>, answer: &Answer) {
        if let (Some((hint, draft)), Ok(text)) = (asked, answer) {
            gitpoulpi_ai::hosting_reply::exchange::push_capped(
                &mut self.exchange,
                Exchange {
                    hint,
                    draft,
                    answer: text.clone(),
                },
            );
        }
    }

    #[cfg(not(feature = "ai"))]
    fn record(&mut self, _: Option<(String, Option<String>)>, _: &Answer) {}

    /// The requests of this sitting that go back to the model (tests).
    #[cfg(test)]
    pub(crate) fn exchange_len(&self) -> usize {
        self.exchange.len()
    }

    fn remember(&mut self, hint: &str) {
        self.recent.retain(|h| h != hint);
        self.recent.push_front(hint.to_string());
        self.recent.truncate(RECENT_MAX);
        self.recall = None;
        if let Some(dir) = &self.dir {
            save_recent(dir, &self.recent);
        }
    }

    /// The finished run's answer, once it is there.
    pub(crate) fn poll(&mut self) -> Option<Answer> {
        let run = self.run.as_ref()?;
        match run.rx.try_recv() {
            Ok(answer) => {
                self.context_line = run.context.lock().ok().and_then(|mut c| c.take());
                let asked = self.run.take().and_then(|r| r.asked);
                self.record(asked, &answer);
                Some(answer)
            }
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => {
                self.run = None;
                Some(Err(gitpoulpi_i18n::msg::ai_ui::WORKER_STOPPED.get().into()))
            }
        }
    }

    /// The draft landed in the editor.
    pub(crate) fn finished(&mut self, answer: &Answer) {
        self.note = match answer {
            Ok(_) => Note::Info(msg::AI_DONE.get().to_string()),
            Err(why) if why.is_empty() => Note::None,
            Err(why) => Note::Error(msg::ai_error(why)),
        };
        self.focus = AiPart::Text;
    }

    // -- keys ------------------------------------------------------------------------

    /// The parts `Tab` visits now: Refresh only exists while the box is ticked.
    fn walk(&self) -> Vec<AiPart> {
        WALK.iter()
            .copied()
            .filter(|p| *p != AiPart::Refresh || self.repo_on())
            // A box that cannot be ticked is not a stop: its reason is on screen.
            .filter(|p| *p != AiPart::Repo || self.repo_choice == ai_context::Choice::Offered)
            .collect()
    }

    fn step(&mut self, back: bool) {
        let walk = self.walk();
        let at = walk.iter().position(|p| *p == self.focus);
        let next = match (at, back) {
            (Some(i), false) if i + 1 < walk.len() => walk[i + 1],
            (Some(_), false) => AiPart::Text,
            (Some(0), true) | (None, true) => AiPart::Text,
            (Some(i), true) => walk[i - 1],
            (None, false) => walk[0],
        };
        // The Run button is the Stop button while a request runs.
        self.focus = if next == AiPart::Run && self.run.is_some() {
            AiPart::Stop
        } else {
            next
        };
        if self.focus == AiPart::Text {
            return;
        }
        self.edit.sync((self.focus == AiPart::Hint).then_some(()));
    }

    /// One key, while the panel is open and `focus` is on it.
    pub(crate) fn on_key(&mut self, key: KeyEvent) -> AiKey {
        if !self.open || self.focus == AiPart::Text {
            return AiKey::Pass;
        }
        match key.code {
            KeyCode::Esc => {
                if self.run.is_some() {
                    self.stop();
                    return AiKey::Handled;
                }
                self.close();
                return AiKey::Closed;
            }
            KeyCode::Tab => {
                self.step(false);
                return AiKey::Handled;
            }
            KeyCode::BackTab => {
                self.step(true);
                return AiKey::Handled;
            }
            _ => {}
        }
        match self.focus {
            AiPart::Hint => self.hint_key(key),
            AiPart::Improve | AiPart::Repo => match key.code {
                KeyCode::Enter | KeyCode::Char(' ') => {
                    self.activate(self.focus);
                    AiKey::Handled
                }
                _ => AiKey::Pass,
            },
            AiPart::Run | AiPart::Stop | AiPart::Close | AiPart::Refresh | AiPart::Recent(_) => {
                match key.code {
                    KeyCode::Enter | KeyCode::Char(' ') => self.activate(self.focus),
                    _ => AiKey::Pass,
                }
            }
            AiPart::Text => AiKey::Pass,
        }
    }

    fn hint_key(&mut self, key: KeyEvent) -> AiKey {
        let plain = !key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
        match key.code {
            KeyCode::Enter if plain => return self.activate(AiPart::Run),
            KeyCode::Up if plain => {
                self.walk_recent(true);
                return AiKey::Handled;
            }
            KeyCode::Down if plain => {
                self.walk_recent(false);
                return AiKey::Handled;
            }
            _ => {}
        }
        self.edit.sync(Some(()));
        let mut value = std::mem::take(&mut self.hint);
        let handled = self.edit.dialog_key((), &mut value, &key);
        self.hint = value;
        match handled {
            Some((edit, _)) => {
                if edit == crate::text_edit::KeyEdit::Edited {
                    self.recall = None;
                    self.note = Note::None;
                }
                AiKey::Handled
            }
            None => AiKey::Pass,
        }
    }

    /// `Up` / `Down` in the field: the instruction before / after this one.
    fn walk_recent(&mut self, older: bool) {
        if self.recent.is_empty() {
            return;
        }
        let next = match (self.recall, older) {
            (None, true) => Some(0),
            (None, false) => None,
            (Some(i), true) => Some((i + 1).min(self.recent.len() - 1)),
            (Some(0), false) => None,
            (Some(i), false) => Some(i - 1),
        };
        self.recall = next;
        self.hint = next
            .and_then(|i| self.recent.get(i).cloned())
            .unwrap_or_default();
        self.edit.reset();
    }

    /// "Refresh context": the button and the palette row.
    pub(crate) fn refresh_context(&mut self) {
        if let Some(root) = self.scope.root.as_deref() {
            ai_context::forget(root);
        }
        self.context_line = None;
        self.exchange.clear();
        self.note = Note::Info(msg::AI_REFRESHED.get().to_string());
    }

    /// Why Refresh context has nothing to do (`None`: the repository is
    /// read, so there is a context to forget).
    pub(crate) fn refresh_reason(&self) -> Option<&'static str> {
        (!self.repo_on()).then_some(msg::WHY_NO_REPO_READ.get())
    }

    fn activate(&mut self, part: AiPart) -> AiKey {
        match part {
            AiPart::Improve => {
                self.improve = !self.improve;
                AiKey::Handled
            }
            AiPart::Repo => {
                match self.repo_choice {
                    ai_context::Choice::Offered => {
                        self.repo = !self.repo;
                        self.note = Note::None;
                        if !self.repo {
                            self.context_line = None;
                            if self.focus == AiPart::Refresh {
                                self.focus = AiPart::Repo;
                            }
                        }
                    }
                    ai_context::Choice::Disabled(why) => {
                        self.note = Note::Info(why.full().to_string());
                    }
                }
                AiKey::Handled
            }
            AiPart::Refresh => {
                self.refresh_context();
                AiKey::Handled
            }
            AiPart::Recent(i) => {
                if let Some(h) = self.recent.get(i).cloned() {
                    self.hint = h;
                    self.recall = Some(i);
                    self.focus = AiPart::Hint;
                    self.edit.reset();
                }
                AiKey::Handled
            }
            AiPart::Run => {
                if self.run.is_some() {
                    AiKey::Handled
                } else {
                    AiKey::Start
                }
            }
            AiPart::Stop => {
                self.stop();
                self.focus = AiPart::Run;
                AiKey::Handled
            }
            AiPart::Close => {
                self.close();
                AiKey::Closed
            }
            AiPart::Hint | AiPart::Text => AiKey::Handled,
        }
    }

    // -- mouse -------------------------------------------------------------------------

    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent) -> AiMouse {
        if !self.open {
            return AiMouse::Outside;
        }
        let at = (mouse.column, mouse.row);
        // A selection drag in the field goes on wherever the pointer is.
        if !matches!(mouse.kind, MouseEventKind::Down(_)) && self.edit.is_dragging() {
            self.edit
                .dialog_mouse(Some(()), &self.hint, mouse.kind, mouse.column, mouse.row);
            return AiMouse::Handled;
        }
        if !self.rect.contains(at.into()) {
            if matches!(mouse.kind, MouseEventKind::Moved) {
                self.hover = None;
            }
            return AiMouse::Outside;
        }
        let hit = dialog_kit::hit_at(&self.hits, mouse.column, mouse.row);
        match mouse.kind {
            MouseEventKind::Moved => self.hover = hit,
            MouseEventKind::Down(MouseButton::Left) => match hit {
                Some(AiPart::Hint) => {
                    self.focus = AiPart::Hint;
                    self.edit.sync(Some(()));
                    self.edit.dialog_mouse(
                        Some(()),
                        &self.hint,
                        mouse.kind,
                        mouse.column,
                        mouse.row,
                    );
                }
                Some(part) => {
                    self.focus = match part {
                        AiPart::Run if self.run.is_some() => AiPart::Stop,
                        other => other,
                    };
                    match self.activate(part) {
                        AiKey::Start => return AiMouse::Start,
                        AiKey::Closed => return AiMouse::Closed,
                        _ => {}
                    }
                }
                None => {}
            },
            MouseEventKind::Drag(MouseButton::Left) | MouseEventKind::Up(MouseButton::Left) => {
                self.edit
                    .dialog_mouse(Some(()), &self.hint, mouse.kind, mouse.column, mouse.row);
            }
            _ => {}
        }
        AiMouse::Handled
    }

    // -- painting ------------------------------------------------------------------------

    /// Paints the panel into `rect` (its height was asked from [`Self::height`]).
    pub(crate) fn render(&mut self, buf: &mut Buffer, rect: Rect) {
        self.hits.clear();
        self.edit.begin_frame();
        self.rect = rect;
        if rect.height == 0 || rect.width < 12 {
            return;
        }
        let panel = Style::default().fg(th().ui.text).bg(th().ui.panel);
        let muted = Style::default().fg(th().ui.muted).bg(th().ui.panel);
        dialog_kit::fill(buf, rect, panel);
        // Rows, from the top; the label, then the Recent list, then the boxes
        // go first when the panel is short.
        let h = rect.height;
        let want_label = h >= 5;
        let want_boxes = h >= 3;
        let want_recent = h >= 4;
        let mut y = rect.y;
        if want_label {
            let spark = Style::default()
                .fg(th().status.ai)
                .bg(th().ui.panel)
                .add_modifier(Modifier::BOLD);
            let used = dialog_kit::put(buf, rect.x + 1, y, crate::theme::AI_SPARK, 1, spark);
            let label = dialog_kit::put(
                buf,
                used.right() + 1,
                y,
                msg::AI_LABEL.get(),
                rect.right().saturating_sub(used.right() + 1),
                panel.add_modifier(Modifier::BOLD),
            );
            dialog_kit::put(
                buf,
                label.right() + 2,
                y,
                msg::AI_ONLY_DRAFTS.get(),
                rect.right().saturating_sub(label.right() + 2),
                muted,
            );
            y += 1;
        }
        // The instruction field.
        let field = Rect::new(rect.x + 1, y, rect.width.saturating_sub(2), 1);
        dialog_kit::text_input(
            buf,
            field,
            &self.hint,
            msg::AI_PLACEHOLDER.get(),
            self.focus == AiPart::Hint,
            self.hover == Some(AiPart::Hint),
            (&self.edit, ()),
        );
        // The example stays visible next to the caret while the field is empty.
        if self.hint.is_empty() && self.focus == AiPart::Hint {
            let hint_style = Style::default()
                .fg(th().ui.muted)
                .bg(crate::input_bg(true, self.hover == Some(AiPart::Hint)));
            dialog_kit::put(
                buf,
                field.x + 2,
                y,
                msg::AI_PLACEHOLDER.get(),
                field.width.saturating_sub(2),
                hint_style,
            );
        }
        self.hits.push((field, AiPart::Hint));
        y += 1;
        if want_boxes {
            let mut x = rect.x + 1;
            let improve = dialog_kit::checkbox(
                buf,
                x,
                y,
                rect.right().saturating_sub(x),
                msg::AI_IMPROVE.get(),
                self.improve,
                CheckState::Enabled {
                    hovered: self.hover == Some(AiPart::Improve),
                    focused: self.focus == AiPart::Improve,
                },
            );
            self.hits.push((improve, AiPart::Improve));
            x = improve.right() + 2;
            if x < rect.right() {
                let (label, checked, state) = match self.repo_choice {
                    ai_context::Choice::Offered => (
                        msg::AI_REPO.get().to_string(),
                        self.repo,
                        CheckState::Enabled {
                            hovered: self.hover == Some(AiPart::Repo),
                            focused: self.focus == AiPart::Repo,
                        },
                    ),
                    ai_context::Choice::Disabled(why) => {
                        // The reason in parentheses only when the row has room
                        // for it (activating the box says it in full anyway).
                        let with = format!("{} ({})", msg::AI_REPO.get(), why.short());
                        let room = usize::from(rect.right().saturating_sub(x));
                        let label = if crate::ui_kit::display_width(&with) + 5 <= room {
                            with
                        } else {
                            msg::AI_REPO.get().to_string()
                        };
                        (label, false, CheckState::Disabled)
                    }
                };
                let repo = dialog_kit::checkbox(
                    buf,
                    x,
                    y,
                    rect.right().saturating_sub(x),
                    &label,
                    checked,
                    state,
                );
                self.hits.push((repo, AiPart::Repo));
            }
            y += 1;
        }
        // What the last draft read of the repository has a row of its own when
        // the panel can have six; in a form that gives it five, the line takes
        // the Recent row's place.
        let ctx_line = self.context_line.clone().filter(|_| self.repo_on());
        let ctx_takes_recent = h < PANEL_ROWS + 1 && ctx_line.is_some();
        if want_recent && ctx_takes_recent {
            if let Some(line) = &ctx_line {
                dialog_kit::put(
                    buf,
                    rect.x + 1,
                    y,
                    line,
                    rect.right().saturating_sub(rect.x + 2),
                    muted,
                );
            }
            y += 1;
        } else if want_recent {
            let mut x = rect.x + 1;
            let head = dialog_kit::put(
                buf,
                x,
                y,
                &format!("{}:", msg::AI_RECENT.get()),
                rect.right().saturating_sub(x),
                muted,
            );
            x = head.right() + 1;
            if self.recent.is_empty() {
                dialog_kit::put(
                    buf,
                    x,
                    y,
                    msg::AI_RECENT_NONE.get(),
                    rect.right().saturating_sub(x),
                    muted,
                );
            }
            for (i, entry) in self.recent.iter().enumerate() {
                let label = crate::ui_kit::truncate_with_ellipsis(entry, RECENT_CELLS);
                let w = crate::ui_kit::display_width(&label) as u16;
                if x + w + 1 > rect.right() {
                    break;
                }
                let hot = self.hover == Some(AiPart::Recent(i));
                let style = Style::default()
                    .fg(th().ui.accent)
                    .bg(if hot { th().ui.hover } else { th().ui.panel })
                    .add_modifier(Modifier::UNDERLINED);
                let r = dialog_kit::put(buf, x, y, &label, w, style);
                crate::pointer_map::hand(r);
                self.hits.push((r, AiPart::Recent(i)));
                x = r.right() + 2;
            }
            y += 1;
        }
        // The buttons and what is going on.
        let mut x = rect.x + 1;
        let running = self.run.is_some();
        let (run_part, run_label) = if running {
            (AiPart::Stop, msg::AI_STOP.get())
        } else {
            (AiPart::Run, msg::AI_DRAFT.get())
        };
        let focused = |p: AiPart| self.focus == p;
        let run = dialog_kit::button(
            buf,
            x,
            y,
            run_label,
            ButtonState::Primary {
                hovered: self.hover == Some(run_part),
                focused: focused(run_part),
            },
        );
        self.hits
            .push((run, if running { AiPart::Stop } else { AiPart::Run }));
        x = run.right() + 1;
        let close = dialog_kit::button(
            buf,
            x,
            y,
            msg::AI_CLOSE.get(),
            ButtonState::Normal {
                hovered: self.hover == Some(AiPart::Close),
                focused: focused(AiPart::Close),
            },
        );
        self.hits.push((close, AiPart::Close));
        x = close.right() + 1;
        if self.repo_on() {
            let refresh = dialog_kit::button(
                buf,
                x,
                y,
                msg::AI_REFRESH.get(),
                ButtonState::Normal {
                    hovered: self.hover == Some(AiPart::Refresh),
                    focused: focused(AiPart::Refresh),
                },
            );
            self.hits.push((refresh, AiPart::Refresh));
            x = refresh.right() + 1;
        }
        x += 1;
        let room = rect.right().saturating_sub(x);
        match (&self.run, &self.note) {
            (Some(run), _) => {
                let line = format!(
                    "{} {}",
                    crate::theme::AI_SPARK,
                    msg::ai_working(run.started.elapsed().as_secs())
                );
                dialog_kit::put(
                    buf,
                    x,
                    y,
                    &line,
                    room,
                    Style::default().fg(th().status.warning).bg(th().ui.panel),
                );
            }
            (None, Note::Error(why)) => {
                dialog_kit::put(
                    buf,
                    x,
                    y,
                    why,
                    room,
                    Style::default().fg(th().status.danger).bg(th().ui.panel),
                );
            }
            (None, Note::Info(line)) => {
                dialog_kit::put(buf, x, y, line, room, muted);
            }
            (None, Note::None) => {
                dialog_kit::put(buf, x, y, msg::AI_KEYS.get(), room, muted);
            }
        }
        // What the last draft read of the repository, on a row of its own.
        if let Some(line) = ctx_line.as_deref().filter(|_| !ctx_takes_recent) {
            if y + 1 < rect.bottom() {
                dialog_kit::put(
                    buf,
                    rect.x + 1,
                    y + 1,
                    line,
                    rect.right().saturating_sub(rect.x + 2),
                    muted,
                );
            }
        }
    }
}
