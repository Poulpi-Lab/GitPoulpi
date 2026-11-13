//! The prompt dialog: one small framed dialog for every question that used to
//! be typed into the status line (stash message, tag message, upstream,
//! Git LFS patterns).
//!
//! Nothing is typed in the footer any more: a prompt is either an inline box
//! on the row it concerns (a ref name) or this dialog. The dialog is hosted
//! like the reword popup: an `InputMode::Prompt` that owns the keyboard and the
//! mouse, drawn in the overlay pass with its hit rects spliced to the front.
//!
//! One component, a few shapes (`PromptKind`): a single-line field, or a
//! multi-line area; an optional AI button (only where there is an AI twin of
//! the question); Cancel and a primary button. What a press of the primary
//! button does is `PromptKind`'s business and comes back as a
//! [`PromptOutcome`]; the host submits the actions.

mod kinds;
mod render;
#[cfg(test)]
mod tests;

use std::cell::{Cell, RefCell};

use crate::combo::{Combo, ComboOutcome};

use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};

use super::*;
use crate::dialog_kit::{self, CloseHit, DiscardGuard, Dismiss, DismissAction, DismissKind, Guard};
use crate::keymap::Action as Bind;
use gitpoulpi_i18n::msg::input as msg_input;
use gitpoulpi_i18n::msg::prompt_dialog as msg;
use gitpoulpi_i18n::msg::remotes as msg_remotes;

/// The dialog's focusable parts, in Tab order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PromptPart {
    /// The text field (or area).
    Field,
    /// The ✦ AI button (only on a prompt with an AI twin, AI on).
    Ai,
    /// An extra button, by index (`Unset upstream`).
    Extra(u8),
    Cancel,
    Submit,
    /// The close cross on the frame (not in the Tab order).
    Close,
}

impl PromptPart {
    pub(crate) fn is_field(self) -> bool {
        self == PromptPart::Field
    }
}

/// Which question the dialog asks, and what it needs to answer it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PromptKind {
    /// `Ctrl+G m`: stash with an (optional) message.
    Stash,
    /// `Create annotated tag here...`, step two: the tag's message.
    TagMessage { at: Oid, name: String },
    /// `Annotate <tag>...` on an existing tag.
    AnnotateTag { tag: String },
    /// `Set upstream`: pick a remote branch (or type `remote/branch`) for `branch`.
    /// `remotes` are the configured remote names a typed value is checked against.
    SetUpstream {
        branch: String,
        remotes: Vec<String>,
    },
    /// `LFS: Track Pattern...`: space-separated patterns.
    LfsTrack,
    /// `LFS: Untrack Pattern...`: pick one of the tracked patterns (or type one).
    LfsUntrack,
}

/// What a key or click in the dialog asks of the host.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum PromptOutcome {
    Stay,
    /// Closed without doing anything (the host also stops a running AI request).
    Close,
    /// The actions to send and the status line to show.
    Submit(Vec<Action>, String),
    /// The ✦ button, or its key: the host starts the request (an explicit
    /// user action) and the dialog shows "writing..." until it lands.
    StartAi(AiRequest),
}

pub(crate) struct PromptDialog {
    pub(crate) kind: PromptKind,
    title: String,
    placeholder: String,
    submit_label: String,
    multiline: bool,
    text: String,
    /// The field is a searchable list (with custom text allowed) instead of
    /// a text box: Set upstream and Untrack. It paints and routes through
    /// `&self` like the rest of the dialog, hence the `RefCell`.
    combo: Option<RefCell<Combo>>,
    /// A second button next to Cancel (`Unset upstream`).
    unset: Option<String>,
    /// What the field held when the dialog opened: only a change to it is
    /// "typed" work.
    baseline: String,
    editor: crate::text_edit::FieldEditor<PromptPart>,
    pub(crate) focus: PromptPart,
    error: Option<String>,
    /// The ✦ button exists (AI on, and this question has an AI twin).
    ai: bool,
    /// A request is running: the field shows "writing..." and is locked.
    pub(crate) ai_running: bool,
    /// The external agent whose proposal opened the dialog.
    proposed_by: Option<String>,
    guard: RefCell<DiscardGuard>,
    /// The multi-line field's scrollbar as last drawn.
    bar: Cell<Option<crate::scrollbar::Track>>,
    bar_drag: crate::scrollbar::BarDrag,
    /// `Ctrl+N` was pressed: the controls show a badge and the next key is
    /// one (a press is a click on the control).
    pub(crate) badges: bool,
}

/// The one close decision of `dialog_kit`, asked of the dialog's state alone
/// (its cross lives in the painted guard, behind a `RefCell`).
struct Rule {
    dirty: bool,
    guard: Guard,
    close: CloseHit,
}

impl Dismiss for Rule {
    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn guard(&self) -> Guard {
        self.guard
    }

    fn close_hit(&self) -> &CloseHit {
        &self.close
    }

    fn close_hit_mut(&mut self) -> &mut CloseHit {
        &mut self.close
    }
}

impl PromptDialog {
    /// The shared constructor: the caret starts at the end of `text`.
    fn base(
        kind: PromptKind,
        title: String,
        placeholder: &str,
        submit_label: String,
        multiline: bool,
        text: &str,
    ) -> Self {
        let mut dialog = Self {
            kind,
            title,
            placeholder: placeholder.to_string(),
            submit_label,
            multiline,
            text: text.to_string(),
            combo: None,
            unset: None,
            baseline: text.to_string(),
            editor: Default::default(),
            focus: PromptPart::Field,
            error: None,
            ai: false,
            ai_running: false,
            proposed_by: None,
            guard: RefCell::new(DiscardGuard::default()),
            bar: Cell::new(None),
            bar_drag: Default::default(),
            badges: false,
        };
        dialog.set_focus(PromptPart::Field);
        dialog
    }

    /// The text field's content.
    #[cfg(test)]
    pub(crate) fn text(&self) -> &str {
        &self.text
    }

    /// AI went on or off, or a request finished: the button and the "writing"
    /// state follow (called by the host every tick).
    pub(crate) fn sync_ai(&mut self, enabled: bool, running: bool) {
        let has_twin = matches!(self.kind, PromptKind::Stash);
        self.ai = has_twin && enabled;
        if !self.ai {
            self.ai_running = false;
            if self.focus == PromptPart::Ai {
                self.set_focus(PromptPart::Field);
            }
        } else if self.ai_running && !running {
            self.ai_running = false;
        }
    }

    /// The AI's answer lands in the field (one line, like a typed message).
    /// Returns whether it was waiting for one (an answer for a dialog that
    /// asked for nothing is dropped).
    pub(crate) fn fill_from_ai(&mut self, answer: &str) -> bool {
        if !self.ai_running {
            return false;
        }
        self.ai_running = false;
        let line = answer.replace(['\r', '\n'], " ");
        self.set_text(line.trim());
        true
    }

    /// Replaces the field's text from outside, the caret at its end.
    fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
        self.editor.reset();
        let len = self.text.len();
        self.editor
            .caret_mut(PromptPart::Field)
            .set_cursor(&self.text, len, false);
        self.error = None;
    }

    fn short(at: Oid) -> String {
        at.to_hex()[..SHORT_SHA_LEN].to_string()
    }

    fn edited(&mut self) {
        self.error = None;
    }

    /// Moves the keyboard to `part`; the field starts with its caret at the
    /// end and nothing selected.
    fn set_focus(&mut self, part: PromptPart) {
        self.focus = part;
        self.editor.sync(part.is_field().then_some(part));
    }

    fn extras(&self) -> u8 {
        u8::from(self.unset.is_some())
    }

    /// The list is open and owns the keyboard and the pointer.
    fn combo_open(&self) -> bool {
        self.combo.as_ref().is_some_and(|c| c.borrow().is_open())
    }

    /// What the field holds: the chosen list entry, or the typed text.
    fn value(&self) -> Option<String> {
        match &self.combo {
            Some(combo) => combo.borrow().selected().map(str::to_string),
            None => Some(self.text.trim().to_string()).filter(|t| !t.is_empty()),
        }
    }

    /// Tab order: the field, the AI button, the extras, Cancel, the primary.
    fn order(&self) -> Vec<PromptPart> {
        let mut order = vec![PromptPart::Field];
        if self.ai {
            order.push(PromptPart::Ai);
        }
        order.extend((0..self.extras()).map(PromptPart::Extra));
        order.push(PromptPart::Cancel);
        order.push(PromptPart::Submit);
        order
    }

    fn step_focus(&mut self, delta: isize) {
        let order = self.order();
        let at = order.iter().position(|p| *p == self.focus).unwrap_or(0) as isize;
        let next = (at + delta).rem_euclid(order.len() as isize) as usize;
        self.set_focus(order[next]);
    }

    /// Typed work that closing would lose.
    fn dirty(&self) -> bool {
        self.combo.is_none() && self.text.trim() != self.baseline.trim()
    }

    fn rule(&self) -> Rule {
        Rule {
            dirty: self.dirty(),
            // A multi-line message asks "discard?"; one line is cheap to retype
            // (an outside click on it is ignored, the cross and Esc close).
            guard: if self.multiline {
                Guard::ConfirmWhenDirty
            } else {
                Guard::IgnoreOutsideWhenDirty
            },
            close: CloseHit::default(),
        }
    }

    /// Esc, Cancel, the cross and a click outside: one rule for all of them.
    pub(crate) fn request_close(&mut self, kind: DismissKind) -> PromptOutcome {
        match dialog_kit::dismiss(&self.rule(), kind) {
            DismissAction::Close => PromptOutcome::Close,
            DismissAction::Swallow => {
                self.guard.get_mut().close.request_flash();
                PromptOutcome::Stay
            }
            DismissAction::Confirm => {
                self.guard.get_mut().ask();
                PromptOutcome::Stay
            }
        }
    }

    fn verdict(&self, v: dialog_kit::Verdict) -> PromptOutcome {
        match v {
            dialog_kit::Verdict::Discard => PromptOutcome::Close,
            dialog_kit::Verdict::Stay => PromptOutcome::Stay,
        }
    }

    /// The primary button, Enter on a single line, or the apply chord.
    pub(crate) fn try_submit(&mut self) -> PromptOutcome {
        if self.ai_running {
            return PromptOutcome::Stay;
        }
        let message = self.text.replace("\r\n", "\n").trim().to_string();
        match self.kind.clone() {
            PromptKind::Stash => {
                // The one prompt whose empty answer is fine.
                let action = if message.is_empty() {
                    Action::StashPush
                } else {
                    Action::StashPushWith { message }
                };
                PromptOutcome::Submit(vec![action], msg_input::STASHING.get().into())
            }
            PromptKind::AnnotateTag { tag } => {
                if message.is_empty() {
                    self.error = Some(msg_input::ANNOTATED_TAG_NEEDS_MESSAGE.get().into());
                    return PromptOutcome::Stay;
                }
                let said = msg_input::annotating_tag(&tag);
                PromptOutcome::Submit(vec![Action::TagAnnotate { name: tag, message }], said)
            }
            PromptKind::TagMessage { at, name } => {
                if message.is_empty() {
                    self.error = Some(msg_input::ANNOTATED_TAG_NEEDS_MESSAGE.get().into());
                    return PromptOutcome::Stay;
                }
                // An agent's proposal compares what was confirmed with its own.
                crate::ui_intent::proposal::confirmed(
                    crate::ui_intent::proposal::Dialog::Tag(at),
                    &format!("{name}\n\n{message}"),
                );
                let said = msg_input::creating_annotated_tag_at(&name, Self::short(at));
                PromptOutcome::Submit(vec![Action::TagCreateAnnotated { name, at, message }], said)
            }
            PromptKind::SetUpstream { branch, remotes } => {
                let Some(upstream) = self.value() else {
                    self.error = Some(msg::UPSTREAM_NONE.get().into());
                    return PromptOutcome::Stay;
                };
                if let Some(why) = self.upstream_problem(&upstream, &remotes) {
                    self.error = Some(why);
                    return PromptOutcome::Stay;
                }
                let said = msg_remotes::setting_upstream(&branch, &upstream);
                let upstream = Some(upstream);
                PromptOutcome::Submit(vec![Action::BranchSetUpstream { branch, upstream }], said)
            }
            PromptKind::LfsTrack => {
                let patterns = patterns_of(&message);
                if patterns.is_empty() {
                    self.error = Some(msg::PATTERN_EMPTY.get().into());
                    return PromptOutcome::Stay;
                }
                let said = msg_input::lfs_tracking(patterns.join(" "));
                PromptOutcome::Submit(
                    vec![Action::LfsTrack {
                        patterns,
                        opts: gitpoulpi_core::domain::LfsTrackOptions::default(),
                    }],
                    said,
                )
            }
            PromptKind::LfsUntrack => {
                let patterns = self.value().map(|v| patterns_of(&v)).unwrap_or_default();
                if patterns.is_empty() {
                    self.error = Some(msg::PATTERN_EMPTY.get().into());
                    return PromptOutcome::Stay;
                }
                let said = msg_input::lfs_untracking(patterns.join(" "));
                PromptOutcome::Submit(vec![Action::LfsUntrack(patterns)], said)
            }
        }
    }

    /// Why a typed upstream cannot be used: it must be `known-remote/branch`
    /// (a listed entry always is).
    fn upstream_problem(&self, upstream: &str, remotes: &[String]) -> Option<String> {
        let known = remotes.iter().any(|r| {
            upstream
                .strip_prefix(r.as_str())
                .and_then(|rest| rest.strip_prefix('/'))
                .is_some_and(|branch| !branch.is_empty())
        });
        (!known).then(|| msg::unknown_remote(upstream.split('/').next().unwrap_or(upstream)))
    }

    /// The `Unset upstream` button: drops the branch's upstream.
    fn unset_upstream(&mut self) -> PromptOutcome {
        let PromptKind::SetUpstream { branch, .. } = self.kind.clone() else {
            return PromptOutcome::Stay;
        };
        let said = msg_remotes::unsetting_upstream(&branch);
        PromptOutcome::Submit(
            vec![Action::BranchSetUpstream {
                branch,
                upstream: None,
            }],
            said,
        )
    }

    /// What the list answered. A pick moves the keyboard to the primary
    /// button (a second Enter confirms); a typed value is checked first.
    fn combo_outcome(&mut self, outcome: ComboOutcome) -> PromptOutcome {
        match outcome {
            ComboOutcome::Picked(_) => {}
            ComboOutcome::Custom(value) => {
                let problem = match &self.kind {
                    PromptKind::SetUpstream { remotes, .. } => {
                        self.upstream_problem(&value, remotes)
                    }
                    _ => None,
                };
                if let Some(why) = problem {
                    if let Some(combo) = &self.combo {
                        combo.borrow_mut().select(None);
                    }
                    self.error = Some(why);
                    return PromptOutcome::Stay;
                }
            }
            ComboOutcome::Handled | ComboOutcome::Ignored => return PromptOutcome::Stay,
        }
        self.error = None;
        self.set_focus(PromptPart::Submit);
        PromptOutcome::Stay
    }

    fn start_ai(&mut self) -> PromptOutcome {
        if !self.ai || self.ai_running {
            return PromptOutcome::Stay;
        }
        self.ai_running = true;
        PromptOutcome::StartAi(AiRequest::StashMessage)
    }

    /// A press on a button, or Enter on a focused one.
    fn activate(&mut self, part: PromptPart) -> PromptOutcome {
        match part {
            PromptPart::Field => {
                self.set_focus(part);
                PromptOutcome::Stay
            }
            PromptPart::Ai => {
                self.set_focus(part);
                self.start_ai()
            }
            PromptPart::Cancel | PromptPart::Close => self.request_close(DismissKind::Cancel),
            PromptPart::Submit => {
                self.set_focus(part);
                self.try_submit()
            }
            PromptPart::Extra(_) => self.unset_upstream(),
        }
    }

    /// The dialog's whole keyboard model.
    ///
    /// Enter submits a one-line prompt and breaks the line in a message area,
    /// where `Ctrl+Enter` / `Ctrl+S` (the reword popup's apply key) submits.
    /// Esc closes (asking first for a typed message). Tab / Shift+Tab walk
    /// the controls. Every other key is the shared text editor's.
    pub(crate) fn on_key(&mut self, key: KeyEvent) -> PromptOutcome {
        if let Some(v) = self.guard.get_mut().key(key) {
            return self.verdict(v);
        }
        // An open list is modal: it takes every key, Esc closing it first.
        if self.combo_open() {
            let outcome = self.combo.as_ref().map(|c| c.borrow_mut().on_key(key));
            return outcome.map_or(PromptOutcome::Stay, |o| self.combo_outcome(o));
        }
        let km = crate::keymap::current();
        if key.code == KeyCode::Esc {
            return self.request_close(DismissKind::Esc);
        }
        if self.ai_running {
            // The field is locked while the model writes; only closing works.
            return PromptOutcome::Stay;
        }
        if km.is(Bind::PromptAiStashMessage, &key) && self.ai {
            return self.start_ai();
        }
        if km.is(Bind::RewordApply, &key) {
            return self.try_submit();
        }
        match key.code {
            KeyCode::Tab => {
                self.step_focus(1);
                return PromptOutcome::Stay;
            }
            KeyCode::BackTab => {
                self.step_focus(-1);
                return PromptOutcome::Stay;
            }
            // On a list field Enter opens the list (below); elsewhere it presses.
            KeyCode::Enter if !(self.focus.is_field() && self.combo.is_some()) => {
                return match self.focus {
                    PromptPart::Field if self.multiline => {
                        let mut text = std::mem::take(&mut self.text);
                        self.editor
                            .caret_mut(PromptPart::Field)
                            .insert(&mut text, "\n");
                        self.text = text;
                        self.edited();
                        PromptOutcome::Stay
                    }
                    PromptPart::Field => self.try_submit(),
                    other => self.activate(other),
                };
            }
            _ => {}
        }
        if let Some(combo) = &self.combo {
            // The list field: Enter, Space and Down open it; typing opens it
            // and starts the search.
            if self.focus.is_field() {
                let typed = matches!(key.code, KeyCode::Char(c) if c != ' ')
                    && !key.modifiers.contains(KeyModifiers::CONTROL);
                if typed {
                    combo.borrow_mut().open();
                }
                let outcome = combo.borrow_mut().on_key(key);
                if outcome != ComboOutcome::Ignored {
                    return self.combo_outcome(outcome);
                }
            }
            if !self.focus.is_field() {
                match key.code {
                    KeyCode::Left | KeyCode::Up => self.step_focus(-1),
                    KeyCode::Right | KeyCode::Down => self.step_focus(1),
                    _ => {}
                }
            }
            return PromptOutcome::Stay;
        }
        if !self.focus.is_field() {
            match key.code {
                KeyCode::Left | KeyCode::Up => {
                    self.step_focus(-1);
                    return PromptOutcome::Stay;
                }
                KeyCode::Right | KeyCode::Down => {
                    self.step_focus(1);
                    return PromptOutcome::Stay;
                }
                // Typing with a button focused goes back to the field.
                _ if self
                    .editor
                    .claims(PromptPart::Field, &self.text, &key, self.multiline) =>
                {
                    self.set_focus(PromptPart::Field);
                }
                _ => return PromptOutcome::Stay,
            }
        }
        let mut text = std::mem::take(&mut self.text);
        let result = self
            .editor
            .on_key(PromptPart::Field, &mut text, &key, self.multiline);
        self.text = text;
        match result {
            crate::text_edit::KeyEdit::Edited => self.edited(),
            crate::text_edit::KeyEdit::Copy(s) | crate::text_edit::KeyEdit::Cut(s) => {
                let _ = crate::clipboard::copy_shared(msg_input::WHAT_SELECTION.get(), &s);
                self.edited();
            }
            crate::text_edit::KeyEdit::Moved | crate::text_edit::KeyEdit::Ignored => {}
        }
        PromptOutcome::Stay
    }

    /// Pasted text goes in at the caret. A one-line prompt turns every line
    /// break into a space, so a paste can never submit it; the message area
    /// keeps them.
    pub(crate) fn on_paste(&mut self, text: &str) {
        if self.ai_running || !self.badge_ready() {
            return;
        }
        if let Some(combo) = &self.combo {
            let mut combo = combo.borrow_mut();
            if !combo.is_open() {
                combo.open();
            }
            combo.paste(text);
            return;
        }
        let unix = text.replace("\r\n", "\n").replace('\r', "\n");
        let clean: String = if self.multiline {
            unix.chars()
                .filter(|c| *c == '\n' || *c == '\t' || !c.is_control())
                .collect()
        } else {
            unix.chars()
                .map(|c| if c == '\n' { ' ' } else { c })
                .filter(|c| !c.is_control())
                .collect()
        };
        self.set_focus(PromptPart::Field);
        let mut text = std::mem::take(&mut self.text);
        self.editor
            .caret_mut(PromptPart::Field)
            .insert(&mut text, &clean);
        self.text = text;
        self.edited();
    }

    /// Whether `Ctrl+N` may badge the controls: not while the discard question
    /// owns the keyboard.
    pub(crate) fn badge_ready(&self) -> bool {
        !self.guard.borrow().is_open() && !self.combo_open()
    }

    /// What `Ctrl+N` badges: every control the last paint registered.
    pub(crate) fn nav_targets(hits: &[(Rect, PromptPart)]) -> Vec<(Rect, (u16, u16))> {
        dialog_kit::nav_cells_in(hits, |_| true)
    }

    /// The wheel over the message area scrolls it `delta` lines.
    pub(crate) fn on_wheel(&mut self, delta: isize) {
        let Some(track) = self.bar.get() else {
            return;
        };
        let rows = track.rect.height as usize;
        let top = self
            .editor
            .caret(PromptPart::Field)
            .map_or(0, |c| c.top.get());
        let top = (top as isize + delta).max(0) as usize;
        self.editor
            .caret_mut(PromptPart::Field)
            .scroll_lines_to(&self.text, top, rows);
    }

    /// A mouse event for the dialog: the discard question first, then the
    /// cross and the click outside, the area's scrollbar, the field, and the
    /// buttons (`hit` is the control under the pointer, from the paint walk).
    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent, hit: Option<PromptPart>) -> PromptOutcome {
        if let Some(v) = self.guard.get_mut().mouse(mouse) {
            return self.verdict(v);
        }
        // An open list is modal: it picks, scrolls, or a click away closes it.
        if self.combo_open() {
            let outcome = self.combo.as_ref().map(|c| c.borrow_mut().on_mouse(mouse));
            return outcome.map_or(PromptOutcome::Stay, |o| self.combo_outcome(o));
        }
        match mouse.kind {
            MouseEventKind::ScrollDown => {
                self.on_wheel(3);
                return PromptOutcome::Stay;
            }
            MouseEventKind::ScrollUp => {
                self.on_wheel(-3);
                return PromptOutcome::Stay;
            }
            _ => {}
        }
        if let Some(kind) = self.guard.get_mut().pointer(mouse) {
            return self.request_close(kind);
        }
        let top = self
            .editor
            .caret(PromptPart::Field)
            .map_or(0, |c| c.top.get());
        let lines = self.text.split('\n').count();
        if let Some(top) = self.bar_drag.on_mouse(self.bar.get(), mouse, lines, top) {
            let rows = self.bar.get().map_or(1, |t| t.rect.height as usize);
            self.editor
                .caret_mut(PromptPart::Field)
                .scroll_lines_to(&self.text, top, rows);
            return PromptOutcome::Stay;
        }
        // A selection dragged in the field follows the pointer until released.
        if matches!(
            mouse.kind,
            MouseEventKind::Drag(MouseButton::Left) | MouseEventKind::Up(MouseButton::Left)
        ) {
            self.editor.dialog_mouse(
                Some(PromptPart::Field),
                &self.text,
                mouse.kind,
                mouse.column,
                mouse.row,
            );
            return PromptOutcome::Stay;
        }
        if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
            match hit {
                Some(PromptPart::Field) if self.combo.is_some() => {
                    self.set_focus(PromptPart::Field);
                    let outcome = self.combo.as_ref().map(|c| c.borrow_mut().on_mouse(mouse));
                    return outcome.map_or(PromptOutcome::Stay, |o| self.combo_outcome(o));
                }
                Some(PromptPart::Field) if !self.ai_running => {
                    self.set_focus(PromptPart::Field);
                    self.editor.mouse_down(
                        PromptPart::Field,
                        &self.text,
                        mouse.column,
                        mouse.row,
                        std::time::Instant::now(),
                    );
                }
                Some(part) if part != PromptPart::Field => return self.activate(part),
                _ => {}
            }
        }
        PromptOutcome::Stay
    }
}

/// The dialog's own target under the pointer, among *its* targets only.
pub(crate) fn prompt_hit_at(
    hits: &[(Rect, crate::chrome::HitTarget)],
    col: u16,
    row: u16,
) -> Option<PromptPart> {
    hits.iter().find_map(|(rect, target)| match target {
        crate::chrome::HitTarget::Prompt(part) if rect.contains((col, row).into()) => Some(*part),
        _ => None,
    })
}

/// The patterns of a typed or chosen value, split on whitespace as `git lfs track` takes them.
fn patterns_of(value: &str) -> Vec<String> {
    value.split_whitespace().map(str::to_string).collect()
}
