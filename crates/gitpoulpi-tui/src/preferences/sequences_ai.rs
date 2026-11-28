//! "Create with AI" / "Edit with AI" of the Sequences page: the user writes
//! what the sequence must do (or how to change the selected one), the AI
//! proposes, the user reads and validates.
//!
//! **The AI only proposes.** The dialog sets a [`SeqAiWish`] the window
//! collects and runs through the ordinary `AiUi` (same profile, key, jobs
//! indicator and Esc as every other ✦ action); the answer comes back as a
//! [`SeqDraftAnswer`], plain data. Nothing is written before **Accept** (or
//! **Edit before saving**), which is one `seq_commit` like any other edit of
//! the page, and nothing is ever run: there is no call to the runner here.
//!
//! Two steps in one framed dialog:
//!
//! 1. the description (a multi-line field), the "send context" box (off by
//!    default) and **Send**;
//! 2. the proposal: name, id, description and steps with their pause and
//!    options, a diff against the current sequence in Edit mode (`+` added,
//!    `-` removed, `~` changed), what was left out and why, the AI's notes
//!    and a suggested shortcut checked against the live keymap (never
//!    bound). Buttons: **Accept**, **Edit before saving** (accept, then open
//!    its steps), **Reject** (writes nothing) and **Back** (to the
//!    description, to send again).
//!
//! The dialog is a [`dialog_kit::Dismiss`] with the "Discard changes?" guard:
//! a typed description or an open proposal asks before it is dropped, and
//! the Settings close guard knows about it too.

use crate::text_edit::CaretKeys as _;
use ratatui::buffer::Buffer;
use ratatui::style::Style;

use super::sequences_page::{option_label, pause_label};
use super::sequences_picker::step_label;
use super::*;
use crate::dialog_kit::{self, button, ButtonState, CheckState, DismissKind};
use crate::text_edit::{Caret, KeyEdit};
use crate::text_field::{edit_lines, offset_at, EditLook};
use crate::th;
use gitpoulpi_config::sequences::{self as seqs, PauseMode, SequenceDef, StepSpec};
use gitpoulpi_i18n::msg::sequence_ai as ai;
use gitpoulpi_i18n::msg::settings_sequences as t;

const WIN: dialog_kit::Win = dialog_kit::Win::movable("sequence_ai");

/// What the user asked for, as the window sends it to the AI. The lists are
/// empty unless the box was ticked; the window adds the remote names.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct SeqDraftRequest {
    pub description: String,
    /// Edit mode: the sequence to change.
    pub edit: Option<SequenceDef>,
    /// Ids in use (a new proposal gets another one).
    pub taken_ids: Vec<String>,
    /// The "send context" box was ticked.
    pub context: bool,
    pub remotes: Vec<String>,
    pub ai_profiles: Vec<String>,
    pub open_in_labels: Vec<String>,
}

// The only non-`Eq` part is a TOML value inside an option of a step; no
// request is ever compared for ordering or hashed.
impl Eq for SeqDraftRequest {}

/// An entry of the answer that was left out, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SeqProblem {
    /// 1-based position in the answer's steps; 0 for a field.
    pub position: usize,
    pub what: String,
    pub reason: String,
}

/// What the model proposed: data, nothing else.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SeqDraftAnswer {
    pub def: SequenceDef,
    pub problems: Vec<SeqProblem>,
    pub notes: Vec<String>,
    pub suggested_shortcut: Option<String>,
}

impl Eq for SeqDraftAnswer {}

/// What the dialog asks the window to do.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SeqAiWish {
    Send(Box<SeqDraftRequest>),
    /// Stop the running request.
    Cancel,
}

// ---------------------------------------------------------------------
// The diff model
// ---------------------------------------------------------------------

/// What happened to one step of the sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StepDiff {
    /// Kept as it is: (old index, new index).
    Same(usize, usize),
    /// The same command with another pause or other options.
    Changed(usize, usize),
    Added(usize),
    Removed(usize),
}

/// A text field of the sequence that changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FieldKind {
    Name,
    Description,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FieldDiff {
    pub kind: FieldKind,
    pub old: String,
    pub new: String,
}

/// The difference between a sequence and its proposed replacement.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(super) struct SeqDiff {
    pub fields: Vec<FieldDiff>,
    /// Every step of old and new, in the order to show them.
    pub steps: Vec<StepDiff>,
}

impl SeqDiff {
    /// Nothing differs.
    pub(super) fn is_empty(&self) -> bool {
        self.fields.is_empty() && self.steps.iter().all(|s| matches!(s, StepDiff::Same(_, _)))
    }
}

/// Steps are matched by their command (a longest common subsequence), so an
/// inserted step does not make every later one look changed; a matched pair
/// is `Same` or `Changed` by its pause and options.
pub(super) fn diff(old: &SequenceDef, new: &SequenceDef) -> SeqDiff {
    let mut out = SeqDiff::default();
    if old.name != new.name {
        out.fields.push(FieldDiff {
            kind: FieldKind::Name,
            old: old.name.clone(),
            new: new.name.clone(),
        });
    }
    let text = |d: &SequenceDef| d.description.clone().unwrap_or_default();
    if text(old) != text(new) {
        out.fields.push(FieldDiff {
            kind: FieldKind::Description,
            old: text(old),
            new: text(new),
        });
    }
    let (a, b) = (&old.steps, &new.steps);
    let (n, m) = (a.len(), b.len());
    // lcs[i][j]: the longest common subsequence of a[i..] and b[j..].
    let mut lcs = vec![vec![0usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            lcs[i][j] = if a[i].run == b[j].run {
                lcs[i + 1][j + 1] + 1
            } else {
                lcs[i + 1][j].max(lcs[i][j + 1])
            };
        }
    }
    let (mut i, mut j) = (0, 0);
    while i < n || j < m {
        if i < n && j < m && a[i].run == b[j].run && lcs[i][j] == lcs[i + 1][j + 1] + 1 {
            out.steps.push(if a[i] == b[j] {
                StepDiff::Same(i, j)
            } else {
                StepDiff::Changed(i, j)
            });
            i += 1;
            j += 1;
        } else if i < n && (j >= m || lcs[i + 1][j] >= lcs[i][j + 1]) {
            out.steps.push(StepDiff::Removed(i));
            i += 1;
        } else {
            out.steps.push(StepDiff::Added(j));
            j += 1;
        }
    }
    out
}

/// A step's pause and options as one line, without its name; empty when
/// they are all at their defaults.
pub(super) fn step_details(step: &StepSpec) -> String {
    let mut parts = Vec::new();
    if step.pause != PauseMode::Default {
        parts.push(format!("{}: {}", t::PAUSE.get(), pause_label(step.pause)));
    }
    for (key, value) in &step.options {
        let shown = match value {
            toml::Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        parts.push(format!("{}: {shown}", option_label(key)));
    }
    parts.join(", ")
}

// ---------------------------------------------------------------------
// State
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Text,
    Context,
    Send,
    Cancel,
    Accept,
    EditFirst,
    Reject,
    Back,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Hit {
    Text,
    Context,
    Send,
    Cancel,
    Accept,
    EditFirst,
    Reject,
    Back,
    Body,
}

impl Hit {
    fn focus(self) -> Option<Focus> {
        Some(match self {
            Hit::Text => Focus::Text,
            Hit::Context => Focus::Context,
            Hit::Send => Focus::Send,
            Hit::Cancel => Focus::Cancel,
            Hit::Accept => Focus::Accept,
            Hit::EditFirst => Focus::EditFirst,
            Hit::Reject => Focus::Reject,
            Hit::Back => Focus::Back,
            Hit::Body => return None,
        })
    }
}

/// How a line of the proposal is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Heading,
    Plain,
    Dim,
    Added,
    Removed,
    Changed,
    Problem,
}

/// The proposal as the user reads it.
struct Proposal {
    def: SequenceDef,
    lines: Vec<(Kind, String)>,
    /// Why Accept is not possible, when it is not.
    blocker: Option<String>,
}

enum Stage {
    Describe,
    Result(Box<Proposal>),
}

/// The open dialog.
pub(super) struct SeqAi {
    /// Edit mode: the sequence being changed (as it was when the dialog opened).
    edit: Option<SequenceDef>,
    stage: Stage,
    text: String,
    caret: Caret,
    context: bool,
    focus: Focus,
    working: bool,
    note: Option<(String, bool)>,
    guard: dialog_kit::DiscardGuard,
    /// First visible line of the proposal.
    top: usize,
    /// How many lines the body shows, as last painted.
    rows: usize,
    total: usize,
    text_area: Rect,
    /// The proposal's scrollbar as last painted, and its drag.
    bar: Option<crate::scrollbar::Track>,
    bar_drag: crate::scrollbar::BarDrag,
    hits: Vec<(Rect, Hit)>,
    hover: Option<Hit>,
}

impl SeqAi {
    fn new(edit: Option<SequenceDef>) -> Self {
        SeqAi {
            edit,
            stage: Stage::Describe,
            text: String::new(),
            caret: Caret::default(),
            context: false,
            focus: Focus::Text,
            working: false,
            note: None,
            guard: dialog_kit::DiscardGuard::default(),
            top: 0,
            rows: 1,
            total: 0,
            text_area: Rect::default(),
            bar: None,
            bar_drag: Default::default(),
            hits: Vec::new(),
            hover: None,
        }
    }

    fn result(&self) -> Option<&Proposal> {
        match &self.stage {
            Stage::Result(p) => Some(p),
            Stage::Describe => None,
        }
    }

    fn foci(&self) -> &'static [Focus] {
        match self.stage {
            Stage::Describe => &[Focus::Text, Focus::Context, Focus::Send, Focus::Cancel],
            Stage::Result(_) => &[Focus::Accept, Focus::EditFirst, Focus::Reject, Focus::Back],
        }
    }

    fn step_focus(&mut self, delta: isize) {
        let order = self.foci();
        let at = order.iter().position(|f| *f == self.focus).unwrap_or(0) as isize;
        self.focus = order[(at + delta).rem_euclid(order.len() as isize) as usize];
    }

    fn scroll_by(&mut self, delta: isize) {
        let max = self.total.saturating_sub(self.rows);
        self.top = (self.top as isize + delta).clamp(0, max as isize) as usize;
    }
}

impl dialog_kit::Dismiss for SeqAi {
    fn is_dirty(&self) -> bool {
        !self.text.trim().is_empty() || self.result().is_some()
    }

    fn guard(&self) -> dialog_kit::Guard {
        dialog_kit::Guard::ConfirmWhenDirty
    }

    fn close_hit(&self) -> &dialog_kit::CloseHit {
        &self.guard.close
    }

    fn close_hit_mut(&mut self) -> &mut dialog_kit::CloseHit {
        &mut self.guard.close
    }
}

/// What the page keeps about the dialog besides the dialog itself.
#[derive(Default)]
pub(super) struct SeqAiLink {
    pub(super) dialog: Option<SeqAi>,
    wish: Option<SeqAiWish>,
    /// Another AI request runs (set by the window every tick).
    busy: bool,
}

// ---------------------------------------------------------------------
// The page's side
// ---------------------------------------------------------------------

impl PreferencesState {
    /// The entries exist: AI is built and turned on.
    pub(super) fn seq_ai_offered(&self) -> bool {
        crate::build_features::AI && self.config.ai.enabled
    }

    /// A typed description or an open proposal: Settings asks before it
    /// closes.
    pub(super) fn seq_ai_dirty(&self) -> bool {
        use dialog_kit::Dismiss as _;
        self.seq.ai.dialog.as_ref().is_some_and(|d| d.is_dirty())
    }

    /// The window's every-tick sync: AI on / off and whether a request runs.
    pub(crate) fn seq_ai_sync(&mut self, enabled: bool, running: bool) {
        self.seq.ai.busy = running;
        if !enabled && !self.config.ai.enabled {
            self.seq.ai.dialog = None;
        }
    }

    /// The wish of the dialog, once.
    pub(crate) fn take_seq_ai_wish(&mut self) -> Option<SeqAiWish> {
        self.seq.ai.wish.take()
    }

    /// The request failed or was canceled: back to the dialog, with why.
    pub(crate) fn seq_ai_stopped(&mut self, why: String) {
        if let Some(d) = self.seq.ai.dialog.as_mut() {
            d.working = false;
            d.note = Some((why, true));
        }
    }

    /// "Create with AI...": the dialog. Refused with a message when AI is
    /// off or another request runs.
    pub(crate) fn seq_ai_open_create(&mut self) {
        self.seq_ai_open(None);
    }

    /// "Edit with AI..." on the selected sequence.
    pub(crate) fn seq_ai_open_edit(&mut self) {
        let Some(cur) = self.seq_cur() else { return };
        self.seq_ai_open(Some(cur));
    }

    /// The palette's `Sequence: Create with AI...`: Settings on the page,
    /// the dialog on top.
    pub(crate) fn open_sequence_ai_create(&mut self) {
        let _ = self.show_page(Some(gitpoulpi_ipc::ui::SettingsPage::Sequences), None);
        self.seq_normalise();
        self.seq_ai_open_create();
    }

    fn seq_ai_open(&mut self, edit: Option<SequenceDef>) {
        if !self.seq_ai_offered() {
            self.message = Some(ai::DLG_OFF.get().to_string());
            return;
        }
        if self.seq.ai.busy {
            self.message = Some(ai::DLG_BUSY.get().to_string());
            return;
        }
        self.seq.picker = None;
        self.seq.ai.wish = None;
        self.seq.ai.dialog = Some(SeqAi::new(edit));
    }

    fn seq_ai_close(&mut self) {
        if self.seq.ai.dialog.as_ref().is_some_and(|d| d.working) {
            self.seq.ai.wish = Some(SeqAiWish::Cancel);
        }
        self.seq.ai.dialog = None;
    }

    /// The cross, Esc, Cancel and a click outside all ask the one rule.
    fn seq_ai_request_close(&mut self, kind: DismissKind) {
        use dialog_kit::{dismiss, DismissAction};
        let Some(d) = self.seq.ai.dialog.as_mut() else {
            return;
        };
        match dismiss(&*d, kind) {
            DismissAction::Close => self.seq_ai_close(),
            DismissAction::Swallow => d.guard.close.request_flash(),
            DismissAction::Confirm => d.guard.ask(),
        }
    }

    fn seq_ai_verdict(&mut self, verdict: dialog_kit::Verdict) {
        if verdict == dialog_kit::Verdict::Discard {
            self.seq_ai_close();
        }
    }

    // ------------------------------------------------------------ send

    /// Why Send is not possible now (empty: only a wait).
    fn seq_ai_send_blocker(&self) -> Option<String> {
        let d = self.seq.ai.dialog.as_ref()?;
        if d.working {
            return Some(String::new());
        }
        if d.text.trim().is_empty() {
            return Some(ai::DLG_NEED_TEXT.get().to_string());
        }
        if self.seq.ai.busy {
            return Some(ai::DLG_BUSY.get().to_string());
        }
        None
    }

    fn seq_ai_send(&mut self) {
        if let Some(why) = self.seq_ai_send_blocker() {
            if !why.is_empty() {
                if let Some(d) = self.seq.ai.dialog.as_mut() {
                    d.note = Some((why, true));
                }
            }
            return;
        }
        let taken_ids: Vec<String> = self.seq_entries().iter().map(|d| d.id.clone()).collect();
        let (ai_profiles, open_in_labels) =
            if self.seq.ai.dialog.as_ref().is_some_and(|d| d.context) {
                (
                    self.config
                        .ai
                        .profiles
                        .iter()
                        .map(|p| p.id.clone())
                        .collect(),
                    self.config
                        .ui
                        .open_in
                        .iter()
                        .flatten()
                        .map(|e| e.label.clone())
                        .collect(),
                )
            } else {
                (Vec::new(), Vec::new())
            };
        let Some(d) = self.seq.ai.dialog.as_mut() else {
            return;
        };
        d.working = true;
        d.note = None;
        let request = SeqDraftRequest {
            description: d.text.trim().to_string(),
            edit: d.edit.clone(),
            taken_ids,
            context: d.context,
            remotes: Vec::new(),
            ai_profiles,
            open_in_labels,
        };
        self.seq.ai.wish = Some(SeqAiWish::Send(Box::new(request)));
    }

    // ---------------------------------------------------------- answer

    /// The model's proposal, or why there is none.
    pub(crate) fn seq_ai_answer(&mut self, answer: Result<SeqDraftAnswer, String>) {
        if self.seq.ai.dialog.is_none() {
            self.message = Some(ai::DLG_CLOSED.get().to_string());
            return;
        }
        let proposal = match answer {
            Ok(a) => self.seq_ai_proposal(a),
            Err(why) => return self.seq_ai_stopped(why),
        };
        if let Some(d) = self.seq.ai.dialog.as_mut() {
            d.working = false;
            d.note = None;
            d.top = 0;
            d.stage = Stage::Result(Box::new(proposal));
            d.focus = Focus::Accept;
        }
    }

    fn seq_ai_proposal(&self, answer: SeqDraftAnswer) -> Proposal {
        let edit = self.seq.ai.dialog.as_ref().and_then(|d| d.edit.clone());
        let mut def = answer.def;
        if def.name.trim().is_empty() {
            def.name = def.id.clone();
        }
        let mut lines: Vec<(Kind, String)> = Vec::new();
        let delta = edit.as_ref().map(|old| diff(old, &def));
        match (&edit, &delta) {
            (Some(old), Some(delta)) => self.seq_ai_diff_lines(old, &def, delta, &mut lines),
            _ => self.seq_ai_new_lines(&def, &mut lines),
        }
        if !answer.problems.is_empty() {
            lines.push((Kind::Plain, String::new()));
            lines.push((Kind::Heading, ai::RES_LEFT_OUT.get().to_string()));
            for p in &answer.problems {
                let text = if p.position == 0 {
                    ai::res_left_item(&p.what, &p.reason)
                } else {
                    ai::res_left_step(p.position, &p.what, &p.reason)
                };
                lines.push((Kind::Problem, text));
            }
        }
        if !answer.notes.is_empty() {
            lines.push((Kind::Plain, String::new()));
            lines.push((Kind::Heading, ai::RES_NOTES.get().to_string()));
            for n in &answer.notes {
                lines.push((Kind::Plain, n.clone()));
            }
        }
        if let Some(key) = &answer.suggested_shortcut {
            lines.push((Kind::Plain, String::new()));
            lines.push((Kind::Heading, ai::res_shortcut(key)));
            let (kind, text) = self.seq_ai_shortcut_note(key);
            lines.push((kind, text));
        }
        let blocker = if def.steps.is_empty() {
            Some(ai::RES_NO_STEPS.get().to_string())
        } else if let Some(p) = seqs::validate(&def).into_iter().next() {
            Some(p.reason)
        } else if delta.as_ref().is_some_and(|d| d.is_empty()) {
            Some(ai::RES_NO_CHANGE.get().to_string())
        } else {
            None
        };
        Proposal {
            def,
            lines,
            blocker,
        }
    }

    /// What the live keymap says about a suggested key. It is never bound.
    fn seq_ai_shortcut_note(&self, key: &str) -> (Kind, String) {
        use crate::keymap::chord::KeyChord;
        let Ok(chord) = KeyChord::parse(key) else {
            return (Kind::Dim, ai::RES_SHORTCUT_UNUSABLE.get().to_string());
        };
        match self.keys.km.global_holders(chord).first() {
            Some(label) => (Kind::Problem, ai::res_shortcut_taken(label)),
            None => (Kind::Dim, ai::RES_SHORTCUT_FREE.get().to_string()),
        }
    }

    fn seq_ai_step_line(step: &StepSpec) -> String {
        let details = step_details(step);
        if details.is_empty() {
            step_label(&step.run)
        } else {
            format!("{}  ({details})", step_label(&step.run))
        }
    }

    fn seq_ai_new_lines(&self, def: &SequenceDef, lines: &mut Vec<(Kind, String)>) {
        let field = |label: &str, value: &str| format!("{label}: {value}");
        lines.push((Kind::Plain, field(t::NAME.get(), &def.name)));
        lines.push((Kind::Plain, field(t::ID.get(), &def.id)));
        if let Some(d) = def.description.as_ref().filter(|d| !d.is_empty()) {
            lines.push((Kind::Plain, field(t::DESCRIPTION.get(), d)));
        }
        if def.scope != seqs::Scope::Installation {
            lines.push((
                Kind::Plain,
                field(
                    t::SCOPE.get(),
                    super::sequences_page::scope_label(def.scope),
                ),
            ));
        }
        lines.push((Kind::Plain, String::new()));
        lines.push((Kind::Heading, ai::res_steps(def.steps.len())));
        for (i, step) in def.steps.iter().enumerate() {
            lines.push((
                Kind::Plain,
                format!("{}. {}", i + 1, Self::seq_ai_step_line(step)),
            ));
        }
    }

    fn seq_ai_diff_lines(
        &self,
        old: &SequenceDef,
        new: &SequenceDef,
        delta: &SeqDiff,
        lines: &mut Vec<(Kind, String)>,
    ) {
        lines.push((Kind::Dim, ai::RES_LEGEND.get().to_string()));
        if delta.is_empty() {
            lines.push((Kind::Plain, ai::RES_NO_CHANGE.get().to_string()));
        }
        let shown = |s: &str| {
            if s.is_empty() {
                ai::RES_NONE.get().to_string()
            } else {
                s.to_string()
            }
        };
        for f in &delta.fields {
            let label = match f.kind {
                FieldKind::Name => t::NAME.get(),
                FieldKind::Description => t::DESCRIPTION.get(),
            };
            lines.push((
                Kind::Changed,
                format!(
                    "~ {}",
                    ai::res_field_changed(label, shown(&f.old), shown(&f.new))
                ),
            ));
        }
        lines.push((Kind::Plain, String::new()));
        lines.push((Kind::Heading, ai::res_steps(new.steps.len())));
        let defaults = || ai::RES_DEFAULTS.get().to_string();
        for s in &delta.steps {
            match *s {
                StepDiff::Same(_, j) => lines.push((
                    Kind::Plain,
                    format!("  {}. {}", j + 1, Self::seq_ai_step_line(&new.steps[j])),
                )),
                StepDiff::Added(j) => lines.push((
                    Kind::Added,
                    format!("+ {}. {}", j + 1, Self::seq_ai_step_line(&new.steps[j])),
                )),
                StepDiff::Removed(i) => lines.push((
                    Kind::Removed,
                    format!("- {}", Self::seq_ai_step_line(&old.steps[i])),
                )),
                StepDiff::Changed(i, j) => {
                    let (a, b) = (step_details(&old.steps[i]), step_details(&new.steps[j]));
                    let a = if a.is_empty() { defaults() } else { a };
                    let b = if b.is_empty() { defaults() } else { b };
                    lines.push((
                        Kind::Changed,
                        format!("~ {}. {}: {a} -> {b}", j + 1, step_label(&new.steps[j].run)),
                    ));
                }
            }
        }
    }

    // ---------------------------------------------------------- accept

    /// Accept (and, with `tweak`, open the steps to tweak): one write, then
    /// the list selects the sequence.
    fn seq_ai_accept(&mut self, tweak: bool) {
        let Some(d) = self.seq.ai.dialog.as_mut() else {
            return;
        };
        let Some(proposal) = d.result() else { return };
        if let Some(why) = proposal.blocker.clone() {
            d.note = Some((ai::res_blocked(&why), true));
            return;
        }
        let def = proposal.def.clone();
        let old = d.edit.clone();
        if !self.seq_commit(old.as_ref(), def.clone()) {
            let why = self.message.clone().unwrap_or_default();
            if let Some(d) = self.seq.ai.dialog.as_mut() {
                d.note = Some((why, true));
            }
            return;
        }
        self.seq.ai.dialog = None;
        self.seq_select_id(&def.id);
        self.message = Some(if old.is_some() {
            ai::accepted_edit(&def.name)
        } else {
            ai::accepted_create(&def.name)
        });
        self.focus = PrefFocus::List;
        self.strip = false;
        if tweak {
            self.seq.in_steps = true;
            self.seq.step = 0;
        }
    }

    /// Reject: nothing was written, nothing is.
    fn seq_ai_reject(&mut self) {
        self.seq.ai.dialog = None;
    }

    fn seq_ai_back(&mut self) {
        if let Some(d) = self.seq.ai.dialog.as_mut() {
            d.stage = Stage::Describe;
            d.focus = Focus::Text;
            d.note = None;
        }
    }

    fn seq_ai_press(&mut self, focus: Focus) {
        match focus {
            Focus::Text => {}
            Focus::Context => {
                if let Some(d) = self.seq.ai.dialog.as_mut() {
                    d.context = !d.context;
                }
            }
            Focus::Send => self.seq_ai_send(),
            Focus::Cancel => self.seq_ai_request_close(DismissKind::Cancel),
            Focus::Accept => self.seq_ai_accept(false),
            Focus::EditFirst => self.seq_ai_accept(true),
            Focus::Reject => self.seq_ai_reject(),
            Focus::Back => self.seq_ai_back(),
        }
    }

    // ------------------------------------------------------------ keys

    pub(super) fn seq_ai_key(&mut self, key: KeyEvent) {
        let Some(d) = self.seq.ai.dialog.as_mut() else {
            return;
        };
        if d.guard.is_open() {
            if let Some(v) = d.guard.key(key) {
                self.seq_ai_verdict(v);
            }
            return;
        }
        if d.working {
            if key.code == KeyCode::Esc {
                self.seq.ai.wish = Some(SeqAiWish::Cancel);
            }
            return;
        }
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let plain = !key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
        match key.code {
            KeyCode::Esc => return self.seq_ai_request_close(DismissKind::Esc),
            KeyCode::Tab => return self.seq_ai_step(1),
            KeyCode::BackTab => return self.seq_ai_step(-1),
            _ => {}
        }
        let describing = matches!(d.stage, Stage::Describe);
        let focus = d.focus;
        if describing
            && ctrl
            && !key.modifiers.contains(KeyModifiers::ALT)
            && matches!(
                key.code,
                KeyCode::Enter | KeyCode::Char('s') | KeyCode::Char('S')
            )
        {
            return self.seq_ai_send();
        }
        if focus == Focus::Text {
            d.note = None;
            // Enter is a new line in a multi-line field.
            if key.code == KeyCode::Enter && !ctrl {
                d.caret.insert(&mut d.text, "\n");
                return;
            }
            if let KeyEdit::Copy(s) | KeyEdit::Cut(s) = d.caret.on_key(&mut d.text, &key, true) {
                let msg = crate::clipboard::copy_shared(ai::DLG_COPY_WHAT.get(), &s);
                d.note = Some((msg, false));
            }
            return;
        }
        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') if plain => self.seq_ai_press(focus),
            KeyCode::Right | KeyCode::Down if plain && describing => self.seq_ai_step(1),
            KeyCode::Left | KeyCode::Up if plain && describing => self.seq_ai_step(-1),
            KeyCode::Right if plain => self.seq_ai_step(1),
            KeyCode::Left if plain => self.seq_ai_step(-1),
            KeyCode::Down if plain => d.scroll_by(1),
            KeyCode::Up if plain => d.scroll_by(-1),
            KeyCode::PageDown => {
                let page = d.rows.max(1) as isize;
                d.scroll_by(page)
            }
            KeyCode::PageUp => {
                let page = d.rows.max(1) as isize;
                d.scroll_by(-page)
            }
            KeyCode::Home => d.top = 0,
            KeyCode::End => d.scroll_by(isize::MAX / 2),
            _ => {}
        }
    }

    fn seq_ai_step(&mut self, delta: isize) {
        if let Some(d) = self.seq.ai.dialog.as_mut() {
            d.step_focus(delta);
        }
    }

    pub(super) fn seq_ai_paste(&mut self, pasted: &str) {
        if let Some(d) = self.seq.ai.dialog.as_mut() {
            if !d.working && d.result().is_none() && d.focus == Focus::Text {
                let text = pasted.replace("\r\n", "\n").replace('\r', "\n");
                d.caret.insert(&mut d.text, &text);
                d.note = None;
            }
        }
    }

    // ----------------------------------------------------------- mouse

    pub(super) fn seq_ai_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) {
        let mouse = crossterm::event::MouseEvent {
            kind,
            column: col,
            row,
            modifiers: KeyModifiers::NONE,
        };
        let Some(d) = self.seq.ai.dialog.as_mut() else {
            return;
        };
        if d.guard.is_open() {
            if let Some(v) = d.guard.mouse(mouse) {
                self.seq_ai_verdict(v);
            }
            return;
        }
        if d.result().is_some() {
            if let Some(top) = d.bar_drag.on_mouse(d.bar, mouse, d.total, d.top) {
                d.top = top.min(d.total.saturating_sub(d.rows));
                return;
            }
        }
        match kind {
            MouseEventKind::ScrollDown | MouseEventKind::ScrollUp => {
                let delta = if kind == MouseEventKind::ScrollDown {
                    3
                } else {
                    -3
                };
                if d.result().is_some() {
                    d.scroll_by(delta);
                } else if d.text_area.contains((col, row).into()) {
                    let rows = d.text_area.height as usize;
                    let top = (d.caret.top.get() as isize + delta).max(0) as usize;
                    d.caret.scroll_lines_to(&d.text, top, rows);
                }
            }
            MouseEventKind::Moved | MouseEventKind::Drag(_) => {
                d.hover = dialog_kit::hit_at(&d.hits, col, row);
                let _ = d.guard.pointer(mouse);
            }
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(kind) = d.guard.pointer(mouse) {
                    return self.seq_ai_request_close(kind);
                }
                if d.working {
                    return;
                }
                let Some(hit) = dialog_kit::hit_at(&d.hits, col, row) else {
                    return;
                };
                if let Some(focus) = hit.focus() {
                    d.focus = focus;
                }
                if hit == Hit::Text {
                    let area = d.text_area;
                    let at = offset_at(
                        &d.text,
                        &d.caret,
                        area.width as usize,
                        true,
                        col.saturating_sub(area.x) as usize,
                        row.saturating_sub(area.y) as usize,
                    );
                    let text = d.text.clone();
                    d.caret.set_cursor(&text, at, false);
                } else if let Some(focus) = hit.focus() {
                    self.seq_ai_press(focus);
                }
            }
            _ => {}
        }
    }

    // ----------------------------------------------------------- paint

    pub(super) fn seq_ai_render(
        &mut self,
        buf: &mut Buffer,
        screen: Rect,
        pointer: Option<(u16, u16)>,
    ) {
        let blocker = self.seq_ai_send_blocker();
        let Some(d) = self.seq.ai.dialog.as_mut() else {
            return;
        };
        d.hits.clear();
        let editing = d.edit.as_ref().map(|e| {
            if e.name.trim().is_empty() {
                e.id.clone()
            } else {
                e.name.clone()
            }
        });
        let title = format!(
            "{} {}",
            crate::theme::AI_SPARK,
            match (&d.stage, editing.is_some()) {
                (Stage::Describe, false) => ai::DLG_TITLE_CREATE.get(),
                (Stage::Describe, true) => ai::DLG_TITLE_EDIT.get(),
                (Stage::Result(_), false) => ai::RES_TITLE_CREATE.get(),
                (Stage::Result(_), true) => ai::RES_TITLE_EDIT.get(),
            }
        );
        let width = screen.width.saturating_sub(4).min(92);
        let height = match d.stage {
            Stage::Describe => 16,
            // As tall as the proposal needs (a long one scrolls).
            Stage::Result(ref p) => (p.lines.len() as u16 + 6)
                .max(14)
                .min(screen.height.saturating_sub(4).min(32)),
        };
        let rect = WIN.place(screen, dialog_kit::centered(screen, width, height));
        if let Some((c, r)) = pointer {
            d.guard.close.set_hover(c, r);
        }
        d.guard.framed(buf, rect, &title, &WIN);
        let inner = Rect::new(
            rect.x + 1,
            rect.y + 1,
            rect.width.saturating_sub(2),
            rect.height.saturating_sub(2),
        );
        let dim = Style::default().fg(th().ui.muted).bg(th().ui.panel);
        let text = Style::default().fg(th().ui.text).bg(th().ui.panel);
        let danger = Style::default().fg(th().status.danger).bg(th().ui.panel);
        let wide = inner.width.saturating_sub(2);
        if inner.width < 36 || inner.height < 9 {
            dialog_kit::put(
                buf,
                inner.x + 1,
                inner.y,
                ai::DLG_TOO_SMALL.get(),
                wide,
                dim,
            );
            d.guard.render(buf);
            return;
        }
        let x = inner.x + 1;
        let buttons_y = inner.bottom().saturating_sub(1);
        let note_y = buttons_y.saturating_sub(1);
        let hover = |h: Hit, d: &SeqAi| d.hover == Some(h) && pointer.is_some();
        let stage_is_result = d.result().is_some();
        if !stage_is_result {
            let label = match &editing {
                Some(name) => ai::dlg_label_edit(name),
                None => ai::DLG_LABEL_CREATE.get().to_string(),
            };
            dialog_kit::put(
                buf,
                x,
                inner.y,
                &label,
                wide,
                text.add_modifier(ratatui::style::Modifier::BOLD),
            );
            // Rows under the field: hint, context, its note, footer, note,
            // buttons.
            let field_h = inner.height.saturating_sub(7).max(3);
            let field = Rect::new(x, inner.y + 1, wide, field_h);
            let focused = d.focus == Focus::Text && !d.working;
            let bg = crate::input_bg(focused, hover(Hit::Text, d));
            dialog_kit::fill(buf, field, Style::default().bg(bg));
            let body = Rect::new(field.x + 1, field.y, field.width.saturating_sub(2), field_h);
            d.text_area = body;
            if d.text.is_empty() && !focused {
                let example = if editing.is_some() {
                    ai::DLG_EXAMPLE_EDIT.get()
                } else {
                    ai::DLG_EXAMPLE_CREATE.get()
                };
                dialog_kit::put(
                    buf,
                    body.x,
                    body.y,
                    example,
                    body.width,
                    Style::default().fg(th().ui.muted).bg(bg),
                );
            } else {
                let look = EditLook::new(Style::default().fg(th().ui.text).bg(bg), "\u{2588}");
                let lines = edit_lines(
                    &d.text,
                    Some(&d.caret),
                    body.width.saturating_sub(1) as usize,
                    field_h as usize,
                    focused,
                    look,
                );
                for (i, line) in lines.iter().enumerate() {
                    buf.set_line(body.x, body.y + i as u16, line, body.width);
                }
            }
            d.hits.push((field, Hit::Text));
            let mut y = field.bottom();
            dialog_kit::put(buf, x, y, ai::DLG_HINT.get(), wide, dim);
            y += 1;
            let state = if d.working {
                CheckState::Disabled
            } else {
                CheckState::Enabled {
                    hovered: hover(Hit::Context, d),
                    focused: d.focus == Focus::Context,
                }
            };
            let cr = dialog_kit::checkbox(buf, x, y, wide, ai::DLG_CONTEXT.get(), d.context, state);
            d.hits
                .push((Rect::new(cr.x, cr.y, cr.width.max(3), 1), Hit::Context));
            y += 1;
            if !d.context {
                dialog_kit::put(
                    buf,
                    x + 3,
                    y,
                    ai::DLG_CONTEXT_OFF.get(),
                    wide.saturating_sub(3),
                    dim,
                );
            }
            y += 1;
            dialog_kit::put(
                buf,
                x,
                y.min(note_y.saturating_sub(1)),
                ai::DLG_FOOTER.get(),
                wide,
                dim,
            );
        } else if d.result().is_some() {
            // The body: the proposal's lines, wrapped, scrolled.
            let rows = note_y.saturating_sub(inner.y) as usize;
            let shown = wrap_proposal(
                d.result().map(|p| p.lines.as_slice()).unwrap_or(&[]),
                wide.saturating_sub(2) as usize,
            );
            d.rows = rows.max(1);
            d.total = shown.len();
            let max = d.total.saturating_sub(d.rows);
            d.top = d.top.min(max);
            let top = d.top;
            let body = Rect::new(x, inner.y, wide, rows as u16);
            for (n, (kind, line)) in shown.iter().skip(top).take(rows).enumerate() {
                let style = match kind {
                    Kind::Heading => text.add_modifier(ratatui::style::Modifier::BOLD),
                    Kind::Plain => text,
                    Kind::Dim => dim,
                    Kind::Added => Style::default().fg(th().status.success).bg(th().ui.panel),
                    Kind::Removed | Kind::Problem => danger,
                    Kind::Changed => Style::default().fg(th().status.warning).bg(th().ui.panel),
                };
                dialog_kit::put(buf, x, inner.y + n as u16, line, wide, style);
            }
            d.hits.push((body, Hit::Body));
            // The shared scrollbar, once the proposal outgrows the dialog.
            let bar_rect = Rect::new(inner.right().saturating_sub(1), inner.y, 1, rows as u16);
            d.bar = crate::scrollbar::Track::column(bar_rect, d.total, d.rows, top);
            if let Some(track) = d.bar {
                crate::scrollbar::render_buf(buf, track, true, false);
            }
        }

        // The note: the wait, what went wrong, or why Send cannot go.
        if d.working {
            let line = format!(
                "{} {}",
                crate::theme::AI_SPARK,
                gitpoulpi_i18n::msg::ai_ui::working_start(ai::ACTION.get())
            );
            let r = dialog_kit::put(buf, x, note_y, &line, wide, text);
            tint_spark(buf, r);
        } else if let Some((note, bad)) = &d.note {
            dialog_kit::put(
                buf,
                x,
                note_y,
                note,
                wide.saturating_sub(14),
                if *bad { danger } else { text },
            );
        } else if !stage_is_result {
            if let Some(why) = blocker.as_ref().filter(|w| !w.is_empty()) {
                dialog_kit::put(buf, x, note_y, why, wide, dim);
            }
        }

        // The buttons.
        let mut bx = x;
        let mut place = |buf: &mut Buffer,
                         hits: &mut Vec<(Rect, Hit)>,
                         label: &str,
                         hit: Hit,
                         primary: bool,
                         disabled: bool,
                         d_focus: Focus,
                         hov: bool| {
            let state = if disabled {
                ButtonState::Disabled
            } else if primary {
                ButtonState::Primary {
                    hovered: hov,
                    focused: d_focus == hit.focus().unwrap_or(Focus::Text),
                }
            } else {
                ButtonState::Normal {
                    hovered: hov,
                    focused: d_focus == hit.focus().unwrap_or(Focus::Text),
                }
            };
            let r = button(buf, bx, buttons_y, label, state);
            hits.push((r, hit));
            bx = r.right() + 1;
            r
        };
        let focus = d.focus;
        if stage_is_result {
            let blocked = d.result().is_some_and(|p| p.blocker.is_some());
            let hv = |h: Hit| d.hover == Some(h) && pointer.is_some();
            let (a, e, r, b) = (
                hv(Hit::Accept),
                hv(Hit::EditFirst),
                hv(Hit::Reject),
                hv(Hit::Back),
            );
            let mut hits = std::mem::take(&mut d.hits);
            place(
                buf,
                &mut hits,
                ai::BTN_ACCEPT.get(),
                Hit::Accept,
                true,
                blocked,
                focus,
                a,
            );
            place(
                buf,
                &mut hits,
                ai::BTN_EDIT_FIRST.get(),
                Hit::EditFirst,
                false,
                blocked,
                focus,
                e,
            );
            place(
                buf,
                &mut hits,
                ai::BTN_REJECT.get(),
                Hit::Reject,
                false,
                false,
                focus,
                r,
            );
            place(
                buf,
                &mut hits,
                ai::BTN_BACK.get(),
                Hit::Back,
                false,
                false,
                focus,
                b,
            );
            d.hits = hits;
        } else {
            let send_label = format!("{} {}", crate::theme::AI_SPARK, ai::DLG_SEND.get());
            let blocked = blocker.is_some();
            let hv = |h: Hit| d.hover == Some(h) && pointer.is_some();
            let (s, c) = (hv(Hit::Send), hv(Hit::Cancel));
            let mut hits = std::mem::take(&mut d.hits);
            let sr = place(
                buf,
                &mut hits,
                &send_label,
                Hit::Send,
                true,
                blocked,
                focus,
                s,
            );
            if !blocked {
                tint_spark(buf, sr);
            }
            place(
                buf,
                &mut hits,
                ai::DLG_CANCEL.get(),
                Hit::Cancel,
                false,
                false,
                focus,
                c,
            );
            d.hits = hits;
        }
        // Inner-first: the buttons and fields were pushed before the panel.
        d.guard.render(buf);
    }
}

/// The proposal's lines wrapped to `width`, continuation lines indented.
fn wrap_proposal(lines: &[(Kind, String)], width: usize) -> Vec<(Kind, String)> {
    let mut shown = Vec::new();
    for (kind, line) in lines {
        if line.is_empty() {
            shown.push((*kind, String::new()));
            continue;
        }
        // A line that fits keeps its spacing (the diff's marks and indent).
        if crate::display_width(line) <= width {
            shown.push((*kind, line.clone()));
            continue;
        }
        for (n, part) in crate::wrap_words(line, width).into_iter().enumerate() {
            shown.push((*kind, if n == 0 { part } else { format!("    {part}") }));
        }
    }
    shown
}

fn tint_spark(buf: &mut Buffer, rect: Rect) {
    for x in rect.x..rect.right() {
        if let Some(cell) = buf.cell_mut((x, rect.y)) {
            if cell.symbol() == crate::theme::AI_SPARK {
                cell.set_fg(th().status.ai);
            }
        }
    }
}
