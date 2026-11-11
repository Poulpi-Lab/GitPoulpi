//! ✦ Create Rules with AI: the dialog of the workbench that asks, in plain
//! words, how the history must be rewritten, shows exactly what will be
//! sent, and fills the workbench with the rules and settings the model
//! proposes.
//!
//! **The AI only prepares. It never applies, runs or previews.** Structurally:
//! the dialog sets an [`AiWish`] the window collects and runs through the
//! usual `AiUi` (same profile, key and jobs indicator as every ✦ action); the
//! answer comes back as a [`RulesProposal`], plain data, which
//! [`RewriteView::put_ai_rules`] copies into `recipe` (rules, scope,
//! settings). Nothing on that path calls `preview`, `apply` or the daemon,
//! and the plan on screen is only marked out of date, like any other edit:
//! the user reads, edits and runs Preview and Apply (with their questions and
//! confirmations) themselves.
//!
//! **What is sent** is on screen before the button: the system prompt, the
//! request and, only when the box is ticked (off by default), the scope, the
//! branch and tag names and the authors of the newest commits shown. Never
//! file contents.
//!
//! While the request runs the dialog stays open and modal (Esc cancels it).
//! When the answer holds entries that were left out (with the reason) or
//! remarks, the dialog turns into a short result list; the rules are already
//! in the workbench behind it.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use gitpoulpi_core::domain::rewrite::{CommitterPolicy, PruneEmpty, Rule, Scope};
use gitpoulpi_i18n::msg::rewrite_ai as t;
use gitpoulpi_i18n::msg::rewrite_rules_ai as r;

use super::render::{bad, bold, dim, plain, wrap, RewriteHit};
use super::script_ai::{AiSystem, AiWish};
use super::{Focus, Notice, RewriteView};
use crate::dialog_kit::{self, button_width, ButtonState, CheckState};
use crate::sent_pane::{centered, sent_bg, wrap_width, SentPane, MAX_WIDTH};
use crate::text_edit::{Caret, KeyEdit};
use crate::text_field::{self, edit_lines, offset_at, EditLook};
use crate::th;

#[cfg(test)]
pub(super) fn sent_bg_for_test() -> ratatui::style::Color {
    sent_bg()
}

/// What the user agreed to send about the repository. All empty unless the
/// box was ticked.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RulesCtx {
    /// The scope on screen, in English (it goes to the model).
    pub scope: String,
    pub current_branch: Option<String>,
    /// Local branch and tag names.
    pub refs: Vec<String>,
    /// Distinct authors of the newest commits shown: (name, email).
    pub authors: Vec<(String, String)>,
}

/// The run options the answer stated; `None` leaves the workbench's value.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ProposalSettings {
    pub include_tags: Option<bool>,
    pub committer: Option<CommitterPolicy>,
    pub prune_empty: Option<PruneEmpty>,
    pub update_message_hashes: Option<bool>,
    pub keep_backup_refs: Option<bool>,
}

/// An entry of the answer that was left out, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProposalProblem {
    /// 1-based position in the answer's rules; 0 for the scope or a setting.
    pub position: usize,
    pub what: String,
    pub reason: String,
}

/// What the model proposed: data for the workbench, nothing else.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RulesProposal {
    pub scope: Option<Scope>,
    pub settings: ProposalSettings,
    pub rules: Vec<Rule>,
    pub problems: Vec<ProposalProblem>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RulesFocus {
    Text,
    Context,
    Replace,
    Sent,
    Send,
    Cancel,
}

/// The result list shown after an answer with entries left out or remarks.
struct RulesResult {
    /// (text, heading)
    lines: Vec<(String, bool)>,
}

/// The open dialog.
struct RulesDialog {
    text: String,
    caret: Caret,
    /// "Send the repository context": off unless the user ticks it.
    context: bool,
    /// Replace the current rules instead of adding after them.
    replace: bool,
    focus: RulesFocus,
    /// `None` until the window answered; `Err` when AI cannot run.
    system: Option<Result<AiSystem, String>>,
    working: bool,
    note: Option<Notice>,
    result: Option<RulesResult>,
}

/// All the AI state of the "Create Rules" dialog.
#[derive(Default)]
pub(crate) struct RulesAiState {
    /// `ai.enabled` and a build with AI (set by the window every tick).
    pub enabled: bool,
    /// A request is running (any AI request, not only ours).
    pub busy: bool,
    dialog: Option<RulesDialog>,
    pub(super) wish: Option<AiWish>,
    /// The shared "What is sent" pane (collapsed until opened), which also
    /// scrolls the result list.
    sent: SentPane,
    text_area: std::cell::Cell<Rect>,
    /// The description's scrollbar as last painted (`None` while not shown)
    /// and its drag.
    text_bar: std::cell::Cell<Option<crate::scrollbar::Track>>,
    text_drag: crate::scrollbar::BarDrag,
}

impl RulesAiState {
    pub(super) fn is_open(&self) -> bool {
        self.dialog.is_some()
    }
}

impl RulesDialog {
    fn new() -> Self {
        Self {
            text: String::new(),
            caret: Caret::default(),
            context: false,
            replace: false,
            focus: RulesFocus::Text,
            system: None,
            working: false,
            note: None,
            result: None,
        }
    }
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

impl RewriteView {
    // ------------------------------------------------------ the window's side

    /// Whether the workbench offers "Create Rules with AI": AI is built and
    /// turned on.
    pub(crate) fn rules_ai_offered(&self) -> bool {
        self.rules_ai.enabled
    }

    pub(crate) fn rules_ai_open(&self) -> bool {
        self.rules_ai.dialog.is_some()
    }

    /// The workbench opened by the palette's ✦ row: the dialog on top.
    pub(crate) fn with_rules_ai(mut self, enabled: bool) -> Self {
        self.rules_ai.enabled = enabled;
        self.open_rules_ai();
        self
    }

    /// The `Describe` or `Send` wish of the dialog, once.
    pub(super) fn take_rules_wish(&mut self) -> Option<AiWish> {
        self.rules_ai.wish.take()
    }

    pub(crate) fn set_rules_system(&mut self, system: Result<AiSystem, String>) {
        if let Some(d) = self.rules_ai.dialog.as_mut() {
            d.system = Some(system);
        }
    }

    /// The request failed or was canceled: back to the dialog, with why.
    pub(crate) fn rules_ai_stopped(&mut self, why: String) {
        if let Some(d) = self.rules_ai.dialog.as_mut() {
            d.working = false;
            d.note = Some(Notice {
                text: why,
                error: true,
            });
        }
    }

    // ----------------------------------------------------------------- open

    /// ✦ "Create Rules with AI...": the dialog. Refused with a notice when AI
    /// is off or another request runs.
    pub(crate) fn open_rules_ai(&mut self) {
        if !self.rules_ai.enabled {
            self.notice = Some(Notice {
                text: t::DLG_OFF.get().to_string(),
                error: true,
            });
            return;
        }
        if self.rules_ai.busy {
            self.notice = Some(Notice {
                text: t::BUSY.get().to_string(),
                error: true,
            });
            return;
        }
        if self.run.is_busy() || self.purge.is_some() {
            return;
        }
        self.editing = None;
        self.popup = None;
        self.rules_ai.dialog = Some(RulesDialog::new());
        self.rules_ai.sent = SentPane::default();
        self.rules_ai.wish = Some(AiWish::DescribeRules);
    }

    fn close_rules_ai(&mut self) {
        if self.rules_ai.dialog.as_ref().is_some_and(|d| d.working) {
            self.rules_ai.wish = Some(AiWish::Cancel);
        }
        self.rules_ai.dialog = None;
    }

    // ------------------------------------------------------------- queries

    /// What goes along when the box is ticked: the scope on screen, the
    /// branch and tag names and the authors of the commits the Log shows.
    fn rules_context(&self) -> RulesCtx {
        let ctx = &self.scope_ctx;
        let mut authors: Vec<(String, String)> = Vec::new();
        for row in &ctx.sample {
            let who = (row.author_name.clone(), row.author_email.clone());
            if !authors.contains(&who) {
                authors.push(who);
            }
        }
        RulesCtx {
            scope: match &self.recipe.scope {
                Scope::CurrentBranch { .. } => "the current branch", // i18n-english: sent to the model
                Scope::AllLocal => "every local branch and tag", // i18n-english: sent to the model
                Scope::Refs { .. } => "chosen branches and tags", // i18n-english: sent to the model
                Scope::Selection { .. } => "the selected commits", // i18n-english: sent to the model
                Scope::Range { .. } => "a range of commits", // i18n-english: sent to the model
            }
            .to_string(),
            current_branch: ctx.head_branch.clone(),
            refs: ctx
                .branches
                .iter()
                .chain(ctx.tags.iter())
                .cloned()
                .collect(),
            authors,
        }
    }

    /// The context as the dialog would send it: empty unless ticked.
    fn rules_sent_context(&self) -> RulesCtx {
        match &self.rules_ai.dialog {
            Some(d) if d.context => self.rules_context(),
            _ => RulesCtx::default(),
        }
    }

    fn rules_have_rules(&self) -> bool {
        !self.recipe.rules.is_empty()
    }

    /// Why Send is not possible now.
    fn rules_send_blocker(&self) -> Option<String> {
        let d = self.rules_ai.dialog.as_ref()?;
        if d.working {
            return Some(String::new());
        }
        if d.text.trim().is_empty() {
            return Some(r::DLG_NEED_TEXT.get().to_string());
        }
        match &d.system {
            Some(Err(why)) => Some(why.clone()),
            Some(Ok(_)) => None,
            None => Some(String::new()),
        }
    }

    fn rules_foci(&self) -> Vec<RulesFocus> {
        let mut order = vec![RulesFocus::Text, RulesFocus::Context];
        if self.rules_have_rules() {
            order.push(RulesFocus::Replace);
        }
        order.extend([RulesFocus::Sent, RulesFocus::Send, RulesFocus::Cancel]);
        order
    }

    fn rules_step_focus(&mut self, delta: isize) {
        let order = self.rules_foci();
        let Some(d) = self.rules_ai.dialog.as_mut() else {
            return;
        };
        let at = order.iter().position(|f| *f == d.focus).unwrap_or(0) as isize;
        d.focus = order[(at + delta).rem_euclid(order.len() as isize) as usize];
    }

    /// The "what is sent" pane as lines: (text, heading).
    fn rules_sent_lines(&self, width: usize) -> Vec<(String, bool)> {
        let Some(d) = &self.rules_ai.dialog else {
            return Vec::new();
        };
        let mut out: Vec<(String, bool)> = vec![(t::DLG_SENT_SYSTEM.get().to_string(), true)];
        let push = |text: &str, out: &mut Vec<(String, bool)>| {
            for line in text.lines() {
                if line.trim().is_empty() {
                    out.push((String::new(), false));
                } else {
                    out.extend(wrap(line, width).into_iter().map(|l| (l, false)));
                }
            }
        };
        if let Some(Ok(system)) = &d.system {
            push(&system.prompt, &mut out);
        }
        out.push((String::new(), false));
        out.push((t::DLG_SENT_USER.get().to_string(), true));
        let user =
            crate::ai_ui::rewrite_rules_user_message(d.text.trim(), &self.rules_sent_context());
        push(&user, &mut out);
        out
    }

    // ---------------------------------------------------------------- send

    fn rules_send(&mut self) {
        if let Some(why) = self.rules_send_blocker() {
            if !why.is_empty() {
                if let Some(d) = self.rules_ai.dialog.as_mut() {
                    d.note = Some(Notice {
                        text: why,
                        error: true,
                    });
                }
            }
            return;
        }
        let context = self.rules_sent_context();
        let Some(d) = self.rules_ai.dialog.as_mut() else {
            return;
        };
        d.working = true;
        d.note = None;
        self.rules_ai.wish = Some(AiWish::SendRules {
            description: d.text.trim().to_string(),
            context,
        });
    }

    // -------------------------------------------------------------- answer

    /// The model's proposal: into the recipe (rules, scope, settings), never
    /// applied. The plan on screen is out of date, like after any edit.
    pub(crate) fn put_ai_rules(&mut self, proposal: RulesProposal) {
        let replace = self
            .rules_ai
            .dialog
            .as_ref()
            .is_some_and(|d| d.replace && !self.recipe.rules.is_empty());
        let mut problems = proposal.problems.clone();
        let first_new = if replace { 0 } else { self.recipe.rules.len() };
        if replace {
            self.recipe.rules.clear();
        }
        let added = proposal.rules.len();
        self.recipe.rules.extend(proposal.rules);
        if let Some(scope) = proposal.scope {
            let scope = match scope {
                Scope::Refs { refs } => {
                    let known: Vec<String> = self
                        .scope_ctx
                        .branches
                        .iter()
                        .chain(self.scope_ctx.tags.iter())
                        .cloned()
                        .collect();
                    let (keep, drop): (Vec<String>, Vec<String>) =
                        refs.into_iter().partition(|r| known.contains(r));
                    for name in drop {
                        problems.push(ProposalProblem {
                            position: 0,
                            what: "scope".into(),
                            reason: format!("no branch or tag named {name}"), // i18n-english: a detail like the checks of the other AI answers
                        });
                    }
                    (!keep.is_empty()).then_some(Scope::Refs { refs: keep })
                }
                other => Some(other),
            };
            if let Some(scope) = scope {
                self.recipe.scope = scope;
            }
        }
        let s = proposal.settings;
        if let Some(v) = s.include_tags {
            self.recipe.scope_options.include_tags = v;
        }
        if let Some(v) = s.committer {
            self.recipe.options.committer = v;
        }
        if let Some(v) = s.prune_empty {
            self.recipe.options.prune_empty = v;
        }
        if let Some(v) = s.update_message_hashes {
            self.recipe.options.update_message_hashes = v;
        }
        if let Some(v) = s.keep_backup_refs {
            self.recipe.options.keep_backup_refs = v;
        }
        // The first new rule is selected, so the user starts reading there.
        self.rules.cursor = first_new.min(self.recipe.rules.len().saturating_sub(1));
        self.detail = super::DetailForm::default();
        self.sync_detail();
        self.touch();
        self.focus = Focus::Rules;
        self.notice = Some(Notice {
            text: if added == 0 {
                r::RES_NONE.get().to_string()
            } else {
                r::done(added)
            },
            error: added == 0,
        });
        if problems.is_empty() && proposal.notes.is_empty() && added > 0 {
            self.rules_ai.dialog = None;
            return;
        }
        let mut lines: Vec<(String, bool)> = vec![(
            if added == 0 {
                r::RES_NONE.get().to_string()
            } else {
                r::res_filled(self.recipe.rules.len())
            },
            true,
        )];
        if !problems.is_empty() {
            lines.push((String::new(), false));
            lines.push((r::RES_LEFT_OUT.get().to_string(), true));
            for p in &problems {
                lines.push((
                    if p.position == 0 {
                        r::res_setting(&p.what, &p.reason)
                    } else {
                        r::res_item(p.position, &p.what, &p.reason)
                    },
                    false,
                ));
            }
        }
        if !proposal.notes.is_empty() {
            lines.push((String::new(), false));
            lines.push((r::RES_NOTES.get().to_string(), true));
            for n in &proposal.notes {
                lines.push((n.clone(), false));
            }
        }
        lines.push((String::new(), false));
        lines.push((r::DLG_FOOTER.get().to_string(), false));
        if let Some(d) = self.rules_ai.dialog.as_mut() {
            d.working = false;
            d.note = None;
            d.focus = RulesFocus::Cancel;
            d.result = Some(RulesResult { lines });
            self.rules_ai.sent.top = 0;
        } else {
            self.notice = Some(Notice {
                text: t::CLOSED.get().to_string(),
                error: false,
            });
        }
    }

    // ---------------------------------------------------------------- keys

    pub(super) fn on_rules_ai_key(&mut self, key: KeyEvent) {
        let Some(d) = self.rules_ai.dialog.as_mut() else {
            return;
        };
        if d.working {
            if key.code == KeyCode::Esc {
                self.rules_ai.wish = Some(AiWish::Cancel);
            }
            return;
        }
        let plain_key = !key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
        if d.result.is_some() {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => return self.close_rules_ai(),
                KeyCode::Char(' ') if plain_key => return self.close_rules_ai(),
                code => {
                    if let Some(step) = self.rules_ai.sent.key_step(code) {
                        self.rules_ai.sent.scroll_by(step);
                    }
                    return;
                }
            }
        }
        let focus = d.focus;
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        // Ctrl+Enter and Ctrl+S send from anywhere in the dialog (the same
        // chords as Apply and the other prompts).
        if ctrl
            && !key.modifiers.contains(KeyModifiers::ALT)
            && matches!(
                key.code,
                KeyCode::Enter | KeyCode::Char('s') | KeyCode::Char('S')
            )
        {
            return self.rules_send();
        }
        // Alt+W shows or hides "What is sent" from anywhere in the dialog.
        if key.modifiers.contains(KeyModifiers::ALT)
            && !ctrl
            && matches!(key.code, KeyCode::Char('w') | KeyCode::Char('W'))
        {
            return self.rules_ai.sent.toggle();
        }
        match key.code {
            KeyCode::Esc => return self.close_rules_ai(),
            KeyCode::Tab => return self.rules_step_focus(1),
            KeyCode::BackTab => return self.rules_step_focus(-1),
            _ => {}
        }
        match focus {
            RulesFocus::Text => {
                if let Some(d) = self.rules_ai.dialog.as_mut() {
                    d.note = None;
                    // Enter is the caller's in a multi-line field: a new line.
                    if key.code == KeyCode::Enter && !ctrl {
                        d.caret.insert(&mut d.text, "\n");
                        return;
                    }
                    if let KeyEdit::Copy(s) | KeyEdit::Cut(s) =
                        d.caret.on_key(&mut d.text, &key, true)
                    {
                        let msg = crate::clipboard::copy_shared(r::DLG_PROMPT_LABEL.get(), &s);
                        d.note = Some(Notice {
                            text: msg,
                            error: false,
                        });
                    }
                    self.rules_ai.sent.top = 0;
                }
            }
            RulesFocus::Context | RulesFocus::Replace => match key.code {
                KeyCode::Enter | KeyCode::Char(' ') if plain_key => {
                    if let Some(d) = self.rules_ai.dialog.as_mut() {
                        if focus == RulesFocus::Context {
                            d.context = !d.context;
                        } else {
                            d.replace = !d.replace;
                        }
                        self.rules_ai.sent.top = 0;
                    }
                }
                KeyCode::Down | KeyCode::Right if plain_key => self.rules_step_focus(1),
                KeyCode::Up | KeyCode::Left if plain_key => self.rules_step_focus(-1),
                _ => {}
            },
            RulesFocus::Sent => {
                if !self.rules_ai.sent.open {
                    match key.code {
                        KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Right if plain_key => {
                            self.rules_ai.sent.toggle()
                        }
                        KeyCode::Down if plain_key => self.rules_step_focus(1),
                        KeyCode::Up if plain_key => self.rules_step_focus(-1),
                        _ => {}
                    }
                    return;
                }
                match key.code {
                    KeyCode::Enter | KeyCode::Left => self.rules_ai.sent.toggle(),
                    code => {
                        if let Some(step) = self.rules_ai.sent.key_step(code) {
                            self.rules_ai.sent.scroll_by(step);
                        }
                    }
                }
            }
            RulesFocus::Send | RulesFocus::Cancel => match key.code {
                KeyCode::Enter | KeyCode::Char(' ') if plain_key => {
                    if focus == RulesFocus::Send {
                        self.rules_send();
                    } else {
                        self.close_rules_ai();
                    }
                }
                KeyCode::Right | KeyCode::Down if plain_key => self.rules_step_focus(1),
                KeyCode::Left | KeyCode::Up if plain_key => self.rules_step_focus(-1),
                _ => {}
            },
        }
    }

    /// The wheel scrolls what is under the pointer: the description, the
    /// "What is sent" pane, or the result list.
    pub(crate) fn on_rules_ai_wheel(&mut self, hit: RewriteHit, delta: isize) {
        let Some(d) = self.rules_ai.dialog.as_ref() else {
            return;
        };
        if d.result.is_some() {
            return self.rules_ai.sent.scroll_by(delta);
        }
        match hit {
            RewriteHit::RaText | RewriteHit::RaTextBar => {
                let rows = self.rules_ai.text_area.get().height as usize;
                if let Some(d) = self.rules_ai.dialog.as_mut() {
                    let top = (d.caret.top.get() as isize + delta).max(0) as usize;
                    d.caret.scroll_lines_to(&d.text, top, rows);
                }
            }
            RewriteHit::RaSent | RewriteHit::RaSentBar if self.rules_ai.sent.open => {
                self.rules_ai.sent.scroll_by(delta)
            }
            _ => {}
        }
    }

    /// A drag or release with the left button: the thumb of either bar.
    pub(crate) fn on_rules_ai_bar_mouse(&mut self, mouse: crossterm::event::MouseEvent) {
        self.rules_bar_mouse(mouse);
    }

    fn rules_bar_mouse(&mut self, mouse: crossterm::event::MouseEvent) {
        let Some(d) = self.rules_ai.dialog.as_ref() else {
            return;
        };
        // The description's bar.
        let result = d.result.is_some();
        let content = d.text.split('\n').count();
        let top = d.caret.top.get();
        let rows = self.rules_ai.text_area.get().height as usize;
        let track = self.rules_ai.text_bar.get();
        if let Some(new) = self.rules_ai.text_drag.on_mouse(track, mouse, content, top) {
            if let Some(d) = self.rules_ai.dialog.as_mut() {
                d.focus = RulesFocus::Text;
                d.caret.scroll_lines_to(&d.text, new, rows);
            }
        }
        // The pane's (or the result list's) bar.
        if self.rules_ai.sent.bar_mouse(mouse) && !result {
            if let Some(d) = self.rules_ai.dialog.as_mut() {
                d.focus = RulesFocus::Sent;
            }
        }
    }

    /// Pasted text goes into the description (newlines kept).
    pub(super) fn paste_rules_ai(&mut self, pasted: &str) {
        if let Some(d) = self.rules_ai.dialog.as_mut() {
            if !d.working && d.result.is_none() && d.focus == RulesFocus::Text {
                let text = pasted.replace("\r\n", "\n").replace('\r', "\n");
                d.caret.insert(&mut d.text, &text);
                d.note = None;
            }
        }
    }

    // --------------------------------------------------------------- mouse

    pub(super) fn on_rules_ai_press(&mut self, hit: RewriteHit, col: u16, row: u16) {
        let working = self.rules_ai.dialog.as_ref().is_some_and(|d| d.working);
        let result = self
            .rules_ai
            .dialog
            .as_ref()
            .is_some_and(|d| d.result.is_some());
        match hit {
            RewriteHit::RaClose | RewriteHit::RaShield | RewriteHit::RaCancel => {
                self.close_rules_ai()
            }
            RewriteHit::RaPanel => {}
            RewriteHit::RaSentBar if !working => {
                self.rules_bar_mouse(crossterm::event::MouseEvent {
                    kind: crossterm::event::MouseEventKind::Down(
                        crossterm::event::MouseButton::Left,
                    ),
                    column: col,
                    row,
                    modifiers: KeyModifiers::NONE,
                });
            }
            _ if working || result => {}
            RewriteHit::RaText => {
                let area = self.rules_ai.text_area.get();
                if let Some(d) = self.rules_ai.dialog.as_mut() {
                    d.focus = RulesFocus::Text;
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
                }
            }
            RewriteHit::RaContext => {
                if let Some(d) = self.rules_ai.dialog.as_mut() {
                    d.focus = RulesFocus::Context;
                    d.context = !d.context;
                    self.rules_ai.sent.top = 0;
                }
            }
            RewriteHit::RaReplace => {
                if let Some(d) = self.rules_ai.dialog.as_mut() {
                    d.focus = RulesFocus::Replace;
                    d.replace = !d.replace;
                }
            }
            RewriteHit::RaSent => {
                if let Some(d) = self.rules_ai.dialog.as_mut() {
                    d.focus = RulesFocus::Sent;
                }
            }
            RewriteHit::RaSentHead => {
                if let Some(d) = self.rules_ai.dialog.as_mut() {
                    d.focus = RulesFocus::Sent;
                }
                self.rules_ai.sent.toggle();
            }
            RewriteHit::RaSentBar | RewriteHit::RaTextBar => {
                self.rules_bar_mouse(crossterm::event::MouseEvent {
                    kind: crossterm::event::MouseEventKind::Down(
                        crossterm::event::MouseButton::Left,
                    ),
                    column: col,
                    row,
                    modifiers: KeyModifiers::NONE,
                });
            }
            RewriteHit::RaSend => {
                if let Some(d) = self.rules_ai.dialog.as_mut() {
                    d.focus = RulesFocus::Send;
                }
                self.rules_send();
            }
            _ => {}
        }
    }

    // --------------------------------------------------------------- paint

    /// The workbench's "✦ Create Rules with AI..." button label.
    pub(super) fn rules_ai_button_label() -> String {
        format!("{} {}", crate::theme::AI_SPARK, r::BTN_CREATE.get())
    }

    /// Tints the sparkle of the button just painted at `rect` in AI purple.
    pub(super) fn tint_rules_ai_button(buf: &mut Buffer, rect: Rect) {
        tint_spark(buf, rect);
    }

    /// Paints the dialog and returns its click targets, inner-first.
    pub(super) fn paint_rules_ai(
        &self,
        buf: &mut Buffer,
        bounds: Rect,
        hovered: Option<RewriteHit>,
    ) -> Vec<(Rect, RewriteHit)> {
        let mut hits = Vec::new();
        let Some(d) = &self.rules_ai.dialog else {
            return hits;
        };
        // The room the dialog may take; it only takes what its content needs.
        let room = Rect::new(
            bounds.x + 4,
            bounds.y + 2,
            bounds.width.saturating_sub(8),
            bounds.height.saturating_sub(4),
        );
        let budget = room.height.saturating_sub(2);
        let wide_inner = room.width.min(MAX_WIDTH).saturating_sub(4);
        let title = format!("{} {}", crate::theme::AI_SPARK, r::DLG_TITLE.get());
        if wide_inner + 2 < 40 || budget < 14 {
            let rect = room;
            dialog_kit::panel(buf, rect, &title);
            if let Some(cross) =
                crate::chrome::close_cross::paint(buf, rect, hovered == Some(RewriteHit::RaClose))
            {
                WIN_RULES_AI.decorate(buf, rect, Some(cross), true);
            hits.push((cross, RewriteHit::RaClose));
            }
            dialog_kit::put(
                buf,
                rect.x + 2,
                rect.y + 1,
                t::DLG_TOO_SMALL.get(),
                rect.width.saturating_sub(3),
                dim(),
            );
            hits.push((rect, RewriteHit::RaPanel));
            hits.push((bounds, RewriteHit::RaShield));
            return hits;
        }
        let lines = if d.result.is_some() {
            Vec::new()
        } else {
            self.rules_sent_lines(wrap_width(wide_inner))
        };
        let rep = u16::from(self.rules_have_rules());
        let fixed = |fh: u16, g: u16| fh + 9 + 2 * g + rep;
        let extra = self.rules_ai.sent.least_rows();
        let (field_h, gap) = [(5u16, 1u16), (3, 1), (3, 0)]
            .into_iter()
            .find(|&(fh, g)| fixed(fh, g) + extra <= budget)
            .unwrap_or((3, 0));
        let pane_room = budget.saturating_sub(fixed(field_h, gap));
        let pane_h = self.rules_ai.sent.body_rows(lines.len(), pane_room);
        let inner_h = if d.result.is_some() {
            budget
        } else {
            fixed(field_h, gap) + pane_h
        };
        let rect = WIN_RULES_AI.place(room, centered(room, inner_h));
        dialog_kit::panel(buf, rect, &title);
        if let Some(cross) =
            crate::chrome::close_cross::paint(buf, rect, hovered == Some(RewriteHit::RaClose))
        {
            hits.push((cross, RewriteHit::RaClose));
        }
        let inner = Rect::new(
            rect.x + 1,
            rect.y + 1,
            rect.width.saturating_sub(2),
            rect.height.saturating_sub(2),
        );
        let wide = inner.width.saturating_sub(2);
        let buttons_y = inner.bottom().saturating_sub(1);
        if let Some(res) = &d.result {
            return self.paint_rules_result(buf, bounds, rect, inner, res, hovered, hits);
        }
        let mut y = inner.y;

        // The description: a multi-line field.
        dialog_kit::put(
            buf,
            inner.x + 1,
            y,
            r::DLG_PROMPT_LABEL.get(),
            wide,
            bold(d.focus == RulesFocus::Text),
        );
        y += 1;
        let field = Rect::new(inner.x + 1, y, wide, field_h);
        let focus = d.focus == RulesFocus::Text && !d.working;
        let hit = RewriteHit::RaText;
        let bg = crate::input_bg(focus, hovered == Some(hit));
        dialog_kit::fill(buf, field, Style::default().bg(bg));
        let body = Rect::new(field.x + 1, y, field.width.saturating_sub(2), field_h);
        self.rules_ai.text_area.set(body);
        if d.text.is_empty() && !focus {
            dialog_kit::put(
                buf,
                body.x,
                y,
                r::DLG_EXAMPLE.get(),
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
                focus,
                look,
            );
            for (i, line) in lines.iter().enumerate() {
                buf.set_line(body.x, y + i as u16, line, body.width);
            }
        }
        // The description's own scrollbar, once the text outgrows the field.
        let bar = text_field::field_track(field, &d.text, d.caret.top.get());
        self.rules_ai.text_bar.set(bar);
        if let Some(track) = bar {
            crate::scrollbar::render_buf(buf, track, true, false);
            hits.push((track.rect, RewriteHit::RaTextBar));
        }
        hits.push((field, hit));
        y += field_h;
        dialog_kit::put(buf, inner.x + 1, y, r::DLG_HINT.get(), wide, dim());
        y += 1 + gap;

        // What may go along.
        let state = |focused: bool, hot: bool| {
            if d.working {
                CheckState::Disabled
            } else {
                CheckState::Enabled {
                    hovered: hot,
                    focused,
                }
            }
        };
        let cr = dialog_kit::checkbox(
            buf,
            inner.x + 1,
            y,
            wide,
            r::DLG_CONTEXT.get(),
            d.context,
            state(
                d.focus == RulesFocus::Context,
                hovered == Some(RewriteHit::RaContext),
            ),
        );
        hits.push((
            Rect::new(cr.x, cr.y, cr.width.max(3), 1),
            RewriteHit::RaContext,
        ));
        y += 1;
        if !d.context {
            dialog_kit::put(
                buf,
                inner.x + 4,
                y,
                r::DLG_CONTEXT_OFF.get(),
                wide - 3,
                dim(),
            );
        }
        y += 1;
        if self.rules_have_rules() {
            let rr = dialog_kit::checkbox(
                buf,
                inner.x + 1,
                y,
                wide,
                r::DLG_REPLACE.get(),
                d.replace,
                state(
                    d.focus == RulesFocus::Replace,
                    hovered == Some(RewriteHit::RaReplace),
                ),
            );
            hits.push((
                Rect::new(rr.x, rr.y, rr.width.max(3), 1),
                RewriteHit::RaReplace,
            ));
            y += 1;
        }
        let mut facts = Vec::new();
        if let Some(Ok(system)) = &d.system {
            facts.push(t::dlg_profile(&system.via));
        }
        dialog_kit::put(buf, inner.x + 1, y, &facts.join("  "), wide, dim());
        y += 1;
        dialog_kit::put(buf, inner.x + 1, y, r::DLG_FOOTER.get(), wide, dim());
        y += 1 + gap;

        // What is sent: a pane of its own, tinted apart from the fields above
        // it. Its header row is always there (it shows or hides the body).
        let head = Rect::new(inner.x + 1, y, wide, 1);
        self.rules_ai.sent.paint_head(
            buf,
            head,
            d.focus == RulesFocus::Sent,
            hovered == Some(RewriteHit::RaSentHead),
        );
        hits.push((head, RewriteHit::RaSentHead));
        y += 1;
        let note_y = buttons_y.saturating_sub(1);
        self.rules_ai.sent.paint_none();
        if self.rules_ai.sent.open {
            let pane = Rect::new(inner.x + 1, y, wide, note_y.saturating_sub(y));
            self.rules_ai.sent.paint_body(
                buf,
                pane,
                &lines,
                true,
                &mut hits,
                (RewriteHit::RaSentBar, RewriteHit::RaSent),
            );
        }

        // The note: what went wrong, or the wait.
        if d.working {
            let line = format!(
                "{} {}",
                crate::theme::AI_SPARK,
                gitpoulpi_i18n::msg::ai_ui::working_start(r::DLG_TITLE.get())
            );
            let rr = dialog_kit::put(buf, inner.x + 1, note_y, &line, wide, plain());
            tint_spark(buf, rr);
        } else if let Some(note) = &d.note {
            let style = if note.error { bad() } else { plain() };
            dialog_kit::put(buf, inner.x + 1, note_y, &note.text, wide, style);
        } else if let Some(why) = self.rules_send_blocker().filter(|w| !w.is_empty()) {
            dialog_kit::put(buf, inner.x + 1, note_y, &why, wide, dim());
        }

        // The buttons.
        let send_label = format!("{} {}", crate::theme::AI_SPARK, t::DLG_SEND.get());
        let blocked = self.rules_send_blocker().is_some();
        let state = if blocked {
            ButtonState::Disabled
        } else {
            ButtonState::Primary {
                hovered: hovered == Some(RewriteHit::RaSend),
                focused: d.focus == RulesFocus::Send,
            }
        };
        let sr = dialog_kit::button(buf, inner.x + 1, buttons_y, &send_label, state);
        if !blocked {
            tint_spark(buf, sr);
        }
        hits.push((sr, RewriteHit::RaSend));
        let cancel = dialog_kit::button(
            buf,
            sr.right() + 1,
            buttons_y,
            t::DLG_CANCEL.get(),
            ButtonState::Normal {
                hovered: hovered == Some(RewriteHit::RaCancel),
                focused: d.focus == RulesFocus::Cancel,
            },
        );
        hits.push((cancel, RewriteHit::RaCancel));
        hits.push((rect, RewriteHit::RaPanel));
        hits.push((bounds, RewriteHit::RaShield));
        hits
    }

    /// The result list: what was filled in, what was left out and why.
    #[allow(clippy::too_many_arguments)]
    fn paint_rules_result(
        &self,
        buf: &mut Buffer,
        bounds: Rect,
        rect: Rect,
        inner: Rect,
        res: &RulesResult,
        hovered: Option<RewriteHit>,
        mut hits: Vec<(Rect, RewriteHit)>,
    ) -> Vec<(Rect, RewriteHit)> {
        let wide = inner.width.saturating_sub(2);
        let buttons_y = inner.bottom().saturating_sub(1);
        let pane_h = buttons_y.saturating_sub(inner.y + 1) as usize;
        let mut lines: Vec<(String, bool)> = Vec::new();
        for (text, heading) in &res.lines {
            if text.is_empty() {
                lines.push((String::new(), false));
            } else {
                lines.extend(
                    wrap(text, wrap_width(wide))
                        .into_iter()
                        .map(|l| (l, *heading)),
                );
            }
        }
        self.rules_ai.text_bar.set(None);
        let pane = Rect::new(inner.x + 1, inner.y, wide, pane_h as u16);
        self.rules_ai.sent.paint_body(
            buf,
            pane,
            &lines,
            false,
            &mut hits,
            (RewriteHit::RaSentBar, RewriteHit::RaSent),
        );
        let close = dialog_kit::button(
            buf,
            inner.x + 1,
            buttons_y,
            r::RES_CLOSE.get(),
            ButtonState::Primary {
                hovered: hovered == Some(RewriteHit::RaCancel),
                focused: true,
            },
        );
        hits.push((close, RewriteHit::RaCancel));
        hits.push((rect, RewriteHit::RaPanel));
        hits.push((bounds, RewriteHit::RaShield));
        hits
    }
}

/// The dialog moves (its size follows its content).
const WIN_RULES_AI: crate::dialog_kit::Win = crate::dialog_kit::Win::movable("rewrite_rules_ai");
