//! "Edit commit message..." and "Amend...": the popup that edits a
//! commit's full message (summary and body, several lines) and, in the
//! amend scopes, its author name, email and date; and the two questions
//! the Log has to answer before opening it -- is this commit on the
//! current branch at all, and has it already been pushed.
//!
//! One form: `AmendScope::Message` is the original Edit commit message
//! popup (summary + description, applied with `GitBackend::reword`); every
//! other scope is the complete Amend dialog (message and author fields,
//! applied with `GitBackend::amend_commit`, sending just the fields the user
//! changed) and only picks which field starts focused.
//!
//! The rewrite itself (`Action::Reword` / `Action::AmendCommit`)
//! never touches the index or the working tree, so the popup needs no
//! "your changes will be stashed" step. What it does need is a warning
//! when the commit is already on the branch's upstream, because then the
//! branch and its upstream diverge and only a force push (with lease)
//! publishes the new message. That warning is a second press of Apply,
//! inside the popup, rather than a separate modal first: the user sees
//! the message they are about to rewrite while deciding.

use crate::keymap::Action as Bind;
use std::collections::{HashMap, HashSet};

use gitpoulpi_core::domain::{
    normalize_reword_message, CommitAmend, CommitGraph, Ref, RewordRefusal,
};

use super::*;
use gitpoulpi_i18n::msg::reword_ui as msg;

/// The commit HEAD's branch points at, when a branch is checked out.
fn head_tip(refs: &[Ref]) -> Option<Oid> {
    refs.iter()
        .find(|r| r.is_head && matches!(r.kind, RefKind::LocalBranch { .. }))
        .map(|r| r.target)
}

/// Whether `commit` is `from` or one of its ancestors, walking only the
/// commits the Log has loaded. `None` when the walk falls off the loaded
/// page before it can tell.
fn reaches(graph: &CommitGraph, from: Oid, commit: Oid) -> Option<bool> {
    let parents: HashMap<Oid, &[Oid]> = graph
        .nodes
        .iter()
        .map(|n| (n.id, n.parents.as_slice()))
        .collect();
    let mut seen = HashSet::new();
    let mut stack = vec![from];
    let mut incomplete = false;
    while let Some(oid) = stack.pop() {
        if oid == commit {
            return Some(true);
        }
        if !seen.insert(oid) {
            continue;
        }
        match parents.get(&oid) {
            Some(ps) => stack.extend(ps.iter().copied()),
            None => incomplete = true,
        }
    }
    if incomplete {
        None
    } else {
        Some(false)
    }
}

/// `Some(false)` when the loaded graph shows `commit` is *not* on the
/// checked-out branch - the backend would refuse, so the Log says so
/// without opening a popup. `Some(true)` or `None` (detached HEAD, or a
/// history deeper than what is loaded) let the popup open; the backend
/// has the last word either way.
pub(crate) fn on_current_branch(graph: &CommitGraph, refs: &[Ref], commit: Oid) -> Option<bool> {
    reaches(graph, head_tip(refs)?, commit)
}

/// The local branches (other than the checked-out one) whose history, as far
/// as the loaded graph shows, holds `commit`: where it could be edited after
/// a checkout. Sorted by name.
pub(crate) fn other_branches_holding(
    graph: &CommitGraph,
    refs: &[Ref],
    commit: Oid,
) -> Vec<String> {
    let mut names: Vec<String> = refs
        .iter()
        .filter(|r| !r.is_head)
        .filter_map(|r| match &r.kind {
            RefKind::LocalBranch { name } => {
                (reaches(graph, r.target, commit) == Some(true)).then(|| name.clone())
            }
            _ => None,
        })
        .collect();
    names.sort();
    names
}

/// The upstream (`origin/main`) of the checked-out branch when `commit`
/// is already on it - i.e. rewording it will need a force push. `None`
/// when the branch has no upstream, the upstream is not a known remote
/// branch, or the commit is not on it.
///
/// When the loaded graph cannot tell (the upstream's history leaves the
/// loaded page before reaching the commit), the answer is the upstream:
/// warning about a force push that turns out unnecessary costs a click,
/// not warning about one that is costs a rejected push.
pub(crate) fn pushed_upstream(graph: &CommitGraph, refs: &[Ref], commit: Oid) -> Option<String> {
    let upstream = refs
        .iter()
        .find(|r| r.is_head && matches!(r.kind, RefKind::LocalBranch { .. }))?
        .upstream
        .clone()?;
    let tip = refs.iter().find_map(|r| match &r.kind {
        RefKind::RemoteBranch { remote, name } if format!("{remote}/{name}") == upstream => {
            Some(r.target)
        }
        _ => None,
    })?;
    match reaches(graph, tip, commit) {
        Some(false) => None,
        Some(true) | None => Some(upstream),
    }
}

/// The popup's focusable parts, in Tab order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RewordPart {
    Summary,
    Description,
    AuthorName,
    AuthorEmail,
    AuthorDate,
    /// "Modify History Deeper...": leaves for the rewrite workbench.
    Deeper,
    Cancel,
    Apply,
    /// The close cross on the frame (not in the Tab order).
    Close,
}

impl RewordPart {
    pub(crate) fn is_field(self) -> bool {
        matches!(
            self,
            RewordPart::Summary
                | RewordPart::Description
                | RewordPart::AuthorName
                | RewordPart::AuthorEmail
                | RewordPart::AuthorDate
        )
    }
}

mod vocab;
pub(crate) use vocab::AmendScope;

impl AmendScope {
    /// The fields shown, in Tab order.
    fn fields(self) -> &'static [RewordPart] {
        use RewordPart as P;
        match self {
            AmendScope::Message => &[P::Summary, P::Description],
            _ => &[
                P::Summary,
                P::Description,
                P::AuthorName,
                P::AuthorEmail,
                P::AuthorDate,
            ],
        }
    }

    /// The field the caret starts in.
    fn focus(self) -> RewordPart {
        match self {
            AmendScope::Description => RewordPart::Description,
            AmendScope::Author => RewordPart::AuthorName,
            AmendScope::Date => RewordPart::AuthorDate,
            _ => RewordPart::Summary,
        }
    }

    fn shows(self, part: RewordPart) -> bool {
        self.fields().contains(&part)
    }

    fn has_message(self) -> bool {
        self.shows(RewordPart::Summary) || self.shows(RewordPart::Description)
    }

    fn has_author(self) -> bool {
        self.shows(RewordPart::AuthorName)
    }

    fn title(self) -> &'static str {
        match self {
            AmendScope::Message => msg::TITLE.get(),
            _ => msg::TITLE_AMEND.get(),
        }
    }
}

/// The author's date as the Amend popup shows and parses it: UTC, because
/// the Log only knows the instant (a date left as shown is never sent, so
/// the commit keeps its own zone).
pub(crate) fn author_date_text(timestamp: i64) -> String {
    format!(
        "{} +00:00",
        crate::live_settings::format_date("YYYY-MM-DD HH:mm:ss", timestamp)
    )
}

/// What a key or click in the popup asks of the caller.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum RewordFormOutcome {
    Stay,
    Close,
    Submit(Action),
    /// Close the popup (nothing applied) and open the history rewrite
    /// workbench scoped from this commit.
    ModifyDeeper,
}

/// Splits a commit message into its one-line summary and its
/// description (everything after the blank line that follows the
/// summary), line endings normalized to `\n`.
pub(crate) fn split_message(message: &str) -> (String, String) {
    let unix = message.replace("\r\n", "\n");
    let trimmed = unix.trim_end();
    match trimmed.split_once('\n') {
        Some((summary, rest)) => (
            summary.trim_end().to_string(),
            rest.trim_start_matches('\n').to_string(),
        ),
        None => (trimmed.to_string(), String::new()),
    }
}

/// The Reword commit message popup: a one-line summary (with a live
/// character counter), a multi-line description, Cancel and Update
/// Message. Both fields edit through the shared
/// [`crate::text_edit`] core.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RewordForm {
    pub(crate) commit: Oid,
    original: String,
    summary: String,
    description: String,
    scope: AmendScope,
    author_name: String,
    author_email: String,
    author_date: String,
    /// The author fields as the popup opened: only a change is sent.
    original_author: [String; 3],
    editor: crate::text_edit::FieldEditor<RewordPart>,
    pub(crate) focus: RewordPart,
    /// The upstream the commit is already on, when it is.
    pushed_to: Option<String>,
    /// Apply was pressed once on a pushed commit; the next press
    /// rewrites.
    confirming: bool,
    error: Option<String>,
    /// Opened from inside the interactive rebase / cherry-pick view: the
    /// popup then edits the plan row's message, and confirming the text
    /// it was opened with is fine (an automatic combined squash message
    /// is a perfectly good answer).
    embedded: bool,
    /// The Description's scrollbar as last drawn (it only shows while
    /// that field has focus and overflows), and where on the thumb a
    /// drag holds it.
    bar: std::cell::Cell<Option<crate::scrollbar::Track>>,
    bar_grab: Option<i32>,
    /// The external agent whose proposal filled the fields in; the
    /// popup's title says so.
    proposed_by: Option<String>,
    /// Esc was pressed on an edited message: the status line asks "discard?"
    /// and a second Esc closes.
    discarding: bool,
    /// The discard question was asked by "Modify History Deeper...": the
    /// confirming Esc then leaves for the workbench instead of closing.
    deeper_pending: bool,
    /// What the fields held when the popup opened (an agent's proposal
    /// counts as unedited): only a change to it asks before closing.
    baseline: String,
    /// The popup's frame as last drawn (a click outside closes).
    frame: std::cell::Cell<Rect>,
    /// `Ctrl+N` was pressed: the controls show a badge and the next key is
    /// one (a press is a click on the control).
    pub(crate) badges: bool,
}

impl RewordForm {
    /// Opens on `message`, the caret at the end of the summary.
    pub(crate) fn new(commit: Oid, message: &str, pushed_to: Option<String>) -> Self {
        let (summary, description) = split_message(message);
        let mut form = Self {
            commit,
            original: message.to_string(),
            summary,
            description,
            scope: AmendScope::Message,
            author_name: String::new(),
            author_email: String::new(),
            author_date: String::new(),
            original_author: Default::default(),
            editor: Default::default(),
            focus: RewordPart::Summary,
            pushed_to,
            confirming: false,
            error: None,
            embedded: false,
            bar: std::cell::Cell::new(None),
            bar_grab: None,
            proposed_by: None,
            discarding: false,
            deeper_pending: false,
            baseline: message.trim().to_string(),
            frame: std::cell::Cell::new(Rect::default()),
            badges: false,
        };
        form.set_focus(RewordPart::Summary);
        form
    }

    /// The Amend dialog over `scope`'s fields, the author ones prefilled
    /// with the commit's own; the first field shown has the keyboard.
    pub(crate) fn amend(
        mut self,
        scope: AmendScope,
        author_name: &str,
        author_email: &str,
        timestamp: i64,
    ) -> Self {
        // Every entry point shows the whole dialog; the scope only picks
        // the focused field.
        self.scope = if scope == AmendScope::Message {
            AmendScope::Message
        } else {
            AmendScope::All
        };
        self.author_name = author_name.to_string();
        self.author_email = author_email.to_string();
        self.author_date = author_date_text(timestamp);
        self.original_author = [
            self.author_name.clone(),
            self.author_email.clone(),
            self.author_date.clone(),
        ];
        self.set_focus(scope.focus());
        self
    }

    /// The fields set to an agent's proposed `message`, marked "Proposed by
    /// `client`" (the commit's own message stays the "unchanged" reference).
    pub(crate) fn proposed(mut self, message: &str, client: &str) -> Self {
        let (summary, description) = split_message(message);
        self.summary = summary;
        self.description = description;
        self.proposed_by = Some(client.to_string());
        self.baseline = self.text().trim().to_string();
        let first = self.scope.fields()[0];
        self.set_focus(first);
        self
    }

    /// The same popup as the plan view's Reword editor.
    pub(crate) fn embedded(mut self) -> Self {
        self.embedded = true;
        self
    }

    /// The edited message: summary, then (when there is one) a blank
    /// line and the description.
    pub(crate) fn text(&self) -> String {
        let summary = self.summary.trim_end();
        let description = self.description.trim_matches('\n').trim_end();
        if description.trim().is_empty() {
            summary.to_string()
        } else {
            format!("{summary}\n\n{description}")
        }
    }

    /// The upstream the commit is already on, if any.
    #[cfg(test)]
    pub(crate) fn pushed_to(&self) -> Option<&str> {
        self.pushed_to.as_deref()
    }

    fn short(&self) -> String {
        self.commit.to_hex()[..SHORT_SHA_LEN].to_string()
    }

    fn edited(&mut self) {
        self.error = None;
        self.confirming = false;
        self.discarding = false;
        self.deeper_pending = false;
    }

    /// Moves the keyboard to `part`; a field starts with its caret at the
    /// end, nothing selected.
    fn set_focus(&mut self, part: RewordPart) {
        self.focus = part;
        self.editor.sync(part.is_field().then_some(part));
    }

    /// Tab order: the scope's fields, then Cancel and Apply.
    fn order(&self) -> Vec<RewordPart> {
        let mut order = self.scope.fields().to_vec();
        if self.deeper_available() {
            order.push(RewordPart::Deeper);
        }
        order.push(RewordPart::Cancel);
        order.push(RewordPart::Apply);
        order
    }

    /// Whether the "Modify History Deeper..." button exists: the complete
    /// Amend dialog over a real commit, in a build with the rewrite
    /// workbench.
    pub(crate) fn deeper_available(&self) -> bool {
        crate::build_features::HISTORY_REWRITE && self.scope == AmendScope::All && !self.embedded
    }

    /// The button, or its key: leave for the workbench, asking first when
    /// fields were edited.
    pub(crate) fn request_deeper(&mut self) -> RewordFormOutcome {
        if !self.deeper_available() {
            return RewordFormOutcome::Stay;
        }
        if self.changed() {
            self.confirming = false;
            self.discarding = true;
            self.deeper_pending = true;
            RewordFormOutcome::Stay
        } else {
            RewordFormOutcome::ModifyDeeper
        }
    }

    fn step_focus(&mut self, delta: isize) {
        let order = self.order();
        let at = order.iter().position(|p| *p == self.focus).unwrap_or(0) as isize;
        let next = (at + delta).rem_euclid(order.len() as isize) as usize;
        self.set_focus(order[next]);
    }

    /// Moves to the previous (`-1`) or next (`1`) *field*, if there is one.
    fn step_field(&mut self, delta: isize) -> bool {
        let fields = self.scope.fields();
        let Some(at) = fields.iter().position(|p| *p == self.focus) else {
            return false;
        };
        match at.checked_add_signed(delta).and_then(|i| fields.get(i)) {
            Some(next) => {
                self.set_focus(*next);
                true
            }
            None => false,
        }
    }

    /// The text of field `part` (`Summary` for any non-field).
    fn field(&self, part: RewordPart) -> &str {
        match part {
            RewordPart::Description => &self.description,
            RewordPart::AuthorName => &self.author_name,
            RewordPart::AuthorEmail => &self.author_email,
            RewordPart::AuthorDate => &self.author_date,
            _ => &self.summary,
        }
    }

    fn field_mut(&mut self, part: RewordPart) -> &mut String {
        match part {
            RewordPart::Description => &mut self.description,
            RewordPart::AuthorName => &mut self.author_name,
            RewordPart::AuthorEmail => &mut self.author_email,
            RewordPart::AuthorDate => &mut self.author_date,
            _ => &mut self.summary,
        }
    }

    /// Pasted text goes in at the caret. A paste into the summary keeps
    /// its first line there and moves the rest to the description - a
    /// whole commit message pasted in one go lands where it belongs.
    pub(crate) fn on_paste(&mut self, text: &str) {
        let unix: String = text
            .replace("\r\n", "\n")
            .replace('\r', "\n")
            .chars()
            .filter(|c| *c == '\n' || *c == '\t' || !c.is_control())
            .collect();
        if !self.focus.is_field() {
            self.set_focus(self.scope.fields()[0]);
        }
        if !matches!(self.focus, RewordPart::Summary | RewordPart::Description) {
            // A single-line field takes the first line only.
            let line = unix.lines().next().unwrap_or_default().to_string();
            let part = self.focus;
            let mut text = std::mem::take(self.field_mut(part));
            self.editor.caret_mut(part).insert(&mut text, &line);
            *self.field_mut(part) = text;
        } else if self.focus == RewordPart::Description {
            self.editor
                .caret_mut(RewordPart::Description)
                .insert(&mut self.description, &unix);
        } else {
            let (first, rest) = match unix.split_once('\n') {
                Some((f, r)) => (f, r.trim_start_matches('\n')),
                None => (unix.as_str(), ""),
            };
            self.editor
                .caret_mut(RewordPart::Summary)
                .insert(&mut self.summary, first);
            if !rest.is_empty() {
                let old = std::mem::take(&mut self.description);
                self.description = if old.is_empty() {
                    rest.to_string()
                } else {
                    format!("{rest}\n{old}")
                };
                if self.scope.shows(RewordPart::Description) {
                    self.set_focus(RewordPart::Description);
                }
            }
        }
        self.edited();
    }

    /// What the popup would change: the message when it differs from the
    /// commit's, each author field that differs from what it opened with.
    fn amend_request(&self) -> CommitAmend {
        let mut amend = CommitAmend::default();
        if self.scope.has_message() {
            let new = normalize_reword_message(&self.text());
            if new != normalize_reword_message(&self.original) {
                amend.message = Some(self.text());
            }
        }
        if self.scope.has_author() {
            let [name, email, date] = &self.original_author;
            let changed = |now: &String, then: &String| (now != then).then(|| now.clone());
            amend.author_name = changed(&self.author_name, name);
            amend.author_email = changed(&self.author_email, email);
            amend.author_date = changed(&self.author_date, date);
        }
        amend
    }

    /// Apply: refuses an empty (or, in the popup over a commit, an
    /// unchanged) message in place, asks once more for a pushed commit,
    /// and otherwise hands back the action.
    pub(crate) fn try_apply(&mut self) -> RewordFormOutcome {
        let text = self.text();
        if self.scope.has_message() && normalize_reword_message(&text).is_none() {
            self.error = Some(msg::ERR_EMPTY.get().into());
            return RewordFormOutcome::Stay;
        }
        let amend = self.amend_request();
        if !self.embedded && amend == CommitAmend::default() {
            self.error = Some(
                if self.scope == AmendScope::Message {
                    msg::ERR_UNCHANGED
                } else {
                    msg::ERR_NOTHING_CHANGED
                }
                .get()
                .into(),
            );
            return RewordFormOutcome::Stay;
        }
        // The same checks the backend makes, answered in the popup.
        if let Err(refusal) = amend.resolve() {
            self.error = Some(
                match refusal {
                    RewordRefusal::InvalidIdentity => msg::ERR_IDENTITY,
                    RewordRefusal::InvalidDate => msg::ERR_DATE,
                    _ => msg::ERR_EMPTY,
                }
                .get()
                .into(),
            );
            return RewordFormOutcome::Stay;
        }
        if self.pushed_to.is_some() && !self.confirming {
            self.confirming = true;
            self.set_focus(RewordPart::Apply);
            return RewordFormOutcome::Stay;
        }
        // A message-only change keeps the original reword request (and its
        // journal label); anything else is an amend.
        let message_only = amend.author_name.is_none()
            && amend.author_email.is_none()
            && amend.author_date.is_none();
        if message_only && (self.scope == AmendScope::Message || self.embedded) {
            return RewordFormOutcome::Submit(Action::Reword {
                commit: self.commit,
                message: text,
            });
        }
        RewordFormOutcome::Submit(Action::AmendCommit {
            commit: self.commit,
            amend: Box::new(amend),
        })
    }

    /// The status line once the action is sent.
    pub(crate) fn submitted_message(&self) -> String {
        match (&self.pushed_to, self.scope == AmendScope::Message) {
            (Some(upstream), true) => msg::submitted_pushed(self.short(), upstream),
            (None, true) => msg::submitted(self.short()),
            (_, false) => msg::submitted_amend(self.short()),
        }
    }

    /// The status line when the popup is closed without applying.
    pub(crate) fn canceled_message(&self) -> &'static str {
        if self.scope == AmendScope::Message {
            gitpoulpi_i18n::msg::input::EDIT_MESSAGE_CANCELED.get()
        } else {
            gitpoulpi_i18n::msg::input::AMEND_CANCELED.get()
        }
    }

    /// The popup's whole keyboard model.
    ///
    /// Ctrl+Enter - or Ctrl+S / Ctrl+J, because many terminals send
    /// Ctrl+Enter as a plain Enter - confirms; Esc cancels (backing out
    /// of the force-push confirmation first). Tab / Shift+Tab walk
    /// Summary, Description, Cancel and Update message. In the summary,
    /// Enter and Down go to the description; in the description Enter
    /// breaks the line and Up on its first line goes back to the summary.
    /// Every other key is the shared text editor's.
    pub(crate) fn on_key(&mut self, key: KeyEvent) -> RewordFormOutcome {
        if self.discarding {
            // The inline "discard?" question: Esc again discards, anything
            // else keeps editing (and is not typed).
            return if key.code == KeyCode::Esc {
                if self.deeper_pending {
                    RewordFormOutcome::ModifyDeeper
                } else {
                    RewordFormOutcome::Close
                }
            } else {
                self.discarding = false;
                self.deeper_pending = false;
                RewordFormOutcome::Stay
            };
        }
        match key.code {
            KeyCode::Esc if self.confirming => {
                self.confirming = false;
                self.set_focus(RewordPart::Summary);
                return RewordFormOutcome::Stay;
            }
            KeyCode::Esc => return self.request_close(),
            _ if crate::keymap::current().is(Bind::RewordApply, &key) => return self.try_apply(),
            _ if self.deeper_available()
                && crate::keymap::current().is(Bind::RewordDeeper, &key) =>
            {
                return self.request_deeper()
            }
            KeyCode::Tab => {
                self.step_focus(1);
                return RewordFormOutcome::Stay;
            }
            KeyCode::BackTab => {
                self.step_focus(-1);
                return RewordFormOutcome::Stay;
            }
            KeyCode::Enter => {
                match self.focus {
                    RewordPart::Summary
                    | RewordPart::AuthorName
                    | RewordPart::AuthorEmail
                    | RewordPart::AuthorDate => {
                        // Next field, or Apply after the last one.
                        if !self.step_field(1) {
                            self.set_focus(RewordPart::Apply);
                        }
                    }
                    RewordPart::Description => {
                        self.editor
                            .caret_mut(RewordPart::Description)
                            .insert(&mut self.description, "\n");
                        self.edited();
                    }
                    RewordPart::Cancel | RewordPart::Close => return self.request_close(),
                    RewordPart::Deeper => return self.request_deeper(),
                    RewordPart::Apply => return self.try_apply(),
                }
                return RewordFormOutcome::Stay;
            }
            _ => {}
        }

        if !self.focus.is_field() {
            match key.code {
                KeyCode::Left | KeyCode::Up => {
                    self.step_focus(-1);
                    return RewordFormOutcome::Stay;
                }
                KeyCode::Right | KeyCode::Down => {
                    self.step_focus(1);
                    return RewordFormOutcome::Stay;
                }
                // Typing with a button focused goes back to the first field.
                _ if self
                    .editor
                    .claims(RewordPart::Summary, &self.summary, &key, false) =>
                {
                    self.set_focus(self.scope.fields()[0]);
                }
                _ => return RewordFormOutcome::Stay,
            }
        }

        let part = self.focus;
        match (part, key.code) {
            (p, KeyCode::Down) if p != RewordPart::Description => {
                if self.step_field(1) {
                    return RewordFormOutcome::Stay;
                }
            }
            (p, KeyCode::Up) if p != RewordPart::Description => {
                if self.step_field(-1) {
                    return RewordFormOutcome::Stay;
                }
            }
            (RewordPart::Description, KeyCode::Up) => {
                let caret = self.editor.caret_mut(part);
                if !self.description[..caret.cursor(&self.description)].contains('\n')
                    && self.step_field(-1)
                {
                    return RewordFormOutcome::Stay;
                }
            }
            (RewordPart::Description, KeyCode::Down) => {
                let caret = self.editor.caret_mut(part);
                if !self.description[caret.cursor(&self.description)..].contains('\n')
                    && self.step_field(1)
                {
                    return RewordFormOutcome::Stay;
                }
            }
            _ => {}
        }
        let multiline = part == RewordPart::Description;
        let mut buffer = std::mem::take(self.field_mut(part));
        let result = self.editor.on_key(part, &mut buffer, &key, multiline);
        *self.field_mut(part) = buffer;
        match result {
            crate::text_edit::KeyEdit::Edited => self.edited(),
            crate::text_edit::KeyEdit::Copy(s) | crate::text_edit::KeyEdit::Cut(s) => {
                let _ = crate::clipboard::copy_shared(
                    gitpoulpi_i18n::msg::input::WHAT_SELECTION.get(),
                    &s,
                );
                self.edited();
            }
            crate::text_edit::KeyEdit::Moved | crate::text_edit::KeyEdit::Ignored => {}
        }
        RewordFormOutcome::Stay
    }

    /// The message differs from the one the popup opened with.
    fn changed(&self) -> bool {
        self.text().trim() != self.baseline
            || (self.scope.has_author()
                && [&self.author_name, &self.author_email, &self.author_date]
                    .into_iter()
                    .zip(&self.original_author)
                    .any(|(now, then)| now != then))
    }

    /// Esc, Cancel, the cross and a click outside: close at once, or ask
    /// once (inline) when the message was edited.
    pub(crate) fn request_close(&mut self) -> RewordFormOutcome {
        if self.changed() {
            self.deeper_pending = false;
            self.confirming = false;
            self.discarding = true;
            RewordFormOutcome::Stay
        } else {
            RewordFormOutcome::Close
        }
    }

    /// Whether `Ctrl+N` may badge the controls: not while the inline
    /// "discard?" question owns the next key.
    pub(crate) fn badge_ready(&self) -> bool {
        !self.discarding
    }

    /// What `Ctrl+N` badges: every control the last paint registered.
    pub(crate) fn nav_targets(hits: &[(Rect, RewordPart)]) -> Vec<(Rect, (u16, u16))> {
        crate::dialog_kit::nav_cells_in(hits, |_| true)
    }

    /// Whether `(col, row)` is outside the popup as last drawn.
    pub(crate) fn is_outside(&self, col: u16, row: u16) -> bool {
        let frame = self.frame.get();
        frame.width > 0 && !frame.contains((col, row).into())
    }

    /// A press on a button (or, without a position, on a field).
    pub(crate) fn on_click(&mut self, hit: RewordPart) -> RewordFormOutcome {
        match hit {
            _ if hit.is_field() => {
                self.set_focus(hit);
                RewordFormOutcome::Stay
            }
            RewordPart::Cancel | RewordPart::Close => self.request_close(),
            RewordPart::Deeper => self.request_deeper(),
            RewordPart::Apply => {
                self.set_focus(RewordPart::Apply);
                self.try_apply()
            }
            _ => RewordFormOutcome::Stay,
        }
    }

    /// A press at (`col`, `row`) on `hit`: like [`Self::on_click`], and
    /// in a field it also places the caret (a double-click selects a
    /// word, a drag selects a range).
    pub(crate) fn on_press(&mut self, hit: RewordPart, col: u16, row: u16) -> RewordFormOutcome {
        if let Some(track) = self
            .bar
            .get()
            .filter(|t| t.rect.contains((col, row).into()))
        {
            // The Description's scrollbar: grab the thumb, or jump the
            // thumb under the pointer and keep dragging.
            self.set_focus(RewordPart::Description);
            let pos = (row - track.rect.y) as i32;
            if let Some(step) = track.arrow_step(pos as u16) {
                self.on_wheel(step);
            } else if track.on_thumb(pos as u16) {
                self.bar_grab = Some(pos - track.thumb_start as i32);
            } else {
                self.bar_grab = Some((track.thumb_len / 2) as i32);
                self.bar_drag_to(row);
            }
            return RewordFormOutcome::Stay;
        }
        if hit.is_field() {
            self.set_focus(hit);
            let text = self.field(hit).to_string();
            self.editor
                .mouse_down(hit, &text, col, row, std::time::Instant::now());
            return RewordFormOutcome::Stay;
        }
        self.on_click(hit)
    }

    /// The pointer moved with the button held: extends a selection begun
    /// in a field. Returns whether it was one.
    pub(crate) fn on_drag(&mut self, col: u16, row: u16) -> bool {
        if self.bar_grab.is_some() {
            self.bar_drag_to(row);
            return true;
        }
        let Some(part) = self.editor.dragging_field() else {
            return false;
        };
        let text = self.field(part).to_string();
        self.editor.mouse_drag(&text, col, row)
    }

    pub(crate) fn on_release(&mut self) {
        self.bar_grab = None;
        self.editor.mouse_up();
    }

    /// Puts the Description's thumb under screen row `row`.
    fn bar_drag_to(&mut self, row: u16) {
        let (Some(track), Some(grab)) = (self.bar.get(), self.bar_grab) else {
            return;
        };
        let rows = track.rect.height as usize;
        let len = self.description.split('\n').count();
        let pos = row as i32 - track.rect.y as i32;
        let top = track.drag_offset(len, rows, grab, pos);
        self.editor
            .caret_mut(RewordPart::Description)
            .scroll_lines_to(&self.description, top, rows);
    }

    /// The wheel over the Description scrolls it `delta` lines (the
    /// keyboard twin is the caret's own Up/Down/PageUp/PageDown).
    pub(crate) fn on_wheel(&mut self, delta: isize) {
        let Some(track) = self.bar.get() else {
            return;
        };
        let rows = track.rect.height as usize;
        let top = self
            .editor
            .caret(RewordPart::Description)
            .map_or(0, |c| c.top.get());
        let top = (top as isize + delta).max(0) as usize;
        self.editor
            .caret_mut(RewordPart::Description)
            .scroll_lines_to(&self.description, top, rows);
    }

    /// The Apply button's label: it says what the second press does.
    fn apply_label(&self) -> &'static str {
        if self.confirming {
            msg::APPLY_CONFIRM.get()
        } else if self.scope == AmendScope::Message {
            msg::APPLY.get()
        } else {
            msg::APPLY_AMEND.get()
        }
    }

    /// Rows the popup wants for its scope (borders, warning and buttons
    /// included).
    fn wanted_height(&self) -> u16 {
        let warning: u16 = if self.pushed_to.is_some() { 2 } else { 0 };
        warning
            + match self.scope {
                AmendScope::Message => 18,
                _ => 24,
            }
    }

    /// Draws the popup (where the user put it, else centered in `area`) and returns its click targets,
    /// from the same walk that painted them.
    pub(crate) fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        hovered: Option<RewordPart>,
    ) -> Vec<(Rect, RewordPart)> {
        use crate::dialog_kit::{button, button_width, put, text_area, text_input, ButtonState};
        let width = area.width.saturating_sub(4).min(80);
        let height = self.wanted_height().min(area.height);
        let popup = WIN.place(
            area,
            Rect::new(
                area.x + area.width.saturating_sub(width) / 2,
                area.y + area.height.saturating_sub(height) / 3,
                width,
                height,
            ),
        );
        frame.render_clipped(Clear, popup);
        self.frame.set(popup);
        let title = crate::ui_intent::proposal::dialog_title(self.scope.title(), &self.proposed_by);
        frame.render_clipped(
            Block::default()
                .title(title)
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(th().ui.accent))
                .style(Style::default().bg(th().ui.panel)),
            popup,
        );
        let inner = Rect::new(
            popup.x + 2,
            popup.y + 1,
            popup.width.saturating_sub(4),
            popup.height.saturating_sub(2),
        );
        let mut hits = Vec::new();
        if let Some(cross) = crate::chrome::close_cross::paint(
            frame.buffer_mut(),
            popup,
            hovered == Some(RewordPart::Close),
        ) {
            hits.push((cross, RewordPart::Close));
        }
        WIN.decorate(
            frame.buffer_mut(),
            popup,
            hits.first().map(|(r, _)| *r),
            true,
        );
        let warning_rows: u16 = if self.pushed_to.is_some() { 2 } else { 0 };
        if inner.width < 16 || inner.height < 4 + warning_rows {
            crate::dialog_kit::too_small::note(frame.buffer_mut(), popup);
            return hits;
        }
        let w = inner.width as usize;
        self.editor.begin_frame();

        let mut y = inner.y;
        if let Some(upstream) = &self.pushed_to {
            frame.render_clipped(
                Paragraph::new(msg::pushed_warning(upstream))
                    .style(Style::default().fg(th().status.warning))
                    .wrap(ratatui::widgets::Wrap { trim: true }),
                Rect::new(inner.x, y, inner.width, warning_rows),
            );
            y += warning_rows;
        }
        // Rows kept at the bottom: the status line and the buttons, and
        // under the Description the author block.
        let author_rows: u16 = if self.scope.has_author() { 3 } else { 0 };
        let author_gap: u16 = u16::from(self.scope.has_author() && self.scope.has_message());
        let bottom = inner.bottom().saturating_sub(2);

        if self.scope.shows(RewordPart::Summary) {
            // Summary, its live counter at the right.
            let count_w = 5u16;
            let summary_rect = Rect::new(inner.x, y, inner.width.saturating_sub(count_w + 1), 1);
            text_input(
                frame.buffer_mut(),
                summary_rect,
                &self.summary,
                gitpoulpi_i18n::msg::composer::SUMMARY.get(),
                self.focus == RewordPart::Summary,
                false,
                (&self.editor, RewordPart::Summary),
            );
            hits.push((summary_rect, RewordPart::Summary));
            let remaining = SUMMARY_CHAR_LIMIT as i32 - self.summary.chars().count() as i32;
            let count_color = if remaining < 0 {
                th().status.danger
            } else if remaining <= SUMMARY_WARN_REMAINING {
                th().status.warning
            } else {
                th().ui.muted
            };
            let count = format!("{remaining:>w$}", w = count_w as usize);
            put(
                frame.buffer_mut(),
                inner.right().saturating_sub(count_w),
                y,
                &count,
                count_w,
                Style::default().fg(count_color).bg(th().ui.panel),
            );
            y += 2;
        }

        if self.scope.shows(RewordPart::Description) {
            // Description fills what is left above the author block, the
            // status and the buttons.
            let desc_h = bottom
                .saturating_sub(author_rows + author_gap)
                .saturating_sub(y);
            let desc_rect = Rect::new(inner.x, y, inner.width, desc_h);
            let bar = text_area(
                frame.buffer_mut(),
                desc_rect,
                &self.description,
                gitpoulpi_i18n::msg::composer::DESCRIPTION.get(),
                self.focus == RewordPart::Description,
                false,
                (&self.editor, RewordPart::Description),
            );
            self.bar.set(bar);
            hits.push((desc_rect, RewordPart::Description));
            y += desc_h;
        } else {
            self.bar.set(None);
        }

        if self.scope.has_author() {
            y += author_gap;
            let labels = [
                (RewordPart::AuthorName, msg::LABEL_AUTHOR_NAME.get()),
                (RewordPart::AuthorEmail, msg::LABEL_AUTHOR_EMAIL.get()),
                (RewordPart::AuthorDate, msg::LABEL_AUTHOR_DATE.get()),
            ];
            let label_w = labels
                .iter()
                .map(|(_, l)| display_width(l) as u16)
                .max()
                .unwrap_or(0)
                + 1;
            let label_w = label_w.min(inner.width / 3);
            for (part, label) in labels {
                put(
                    frame.buffer_mut(),
                    inner.x,
                    y,
                    label,
                    label_w,
                    Style::default().fg(th().ui.muted).bg(th().ui.panel),
                );
                let rect = Rect::new(inner.x + label_w, y, inner.width.saturating_sub(label_w), 1);
                text_input(
                    frame.buffer_mut(),
                    rect,
                    self.field(part),
                    "",
                    self.focus == part,
                    false,
                    (&self.editor, part),
                );
                hits.push((rect, part));
                y += 1;
            }
        }

        // Status line: an error, the force-push confirmation, or the keys.
        let (status, color) = if self.discarding {
            (msg::DISCARD_HINT.get().to_string(), th().status.warning)
        } else if let Some(error) = &self.error {
            (error.clone(), th().status.danger)
        } else if self.confirming {
            (
                msg::confirm_hint(crate::keymap::current().shown(Bind::RewordApply)),
                th().status.warning,
            )
        } else {
            (
                msg::keys_hint(crate::keymap::current().shown(Bind::RewordApply)),
                th().ui.muted,
            )
        };
        frame.render_clipped(
            Paragraph::new(Span::styled(
                truncate_with_ellipsis(&status, w),
                Style::default().fg(color),
            )),
            Rect::new(inner.x, inner.bottom() - 2, inner.width, 1),
        );

        // Buttons, right-aligned.
        let apply_label = self.apply_label();
        let cancel_label = gitpoulpi_i18n::msg::dialog::CANCEL.get();
        let total = button_width(cancel_label) + 1 + button_width(apply_label);
        let by = inner.bottom() - 1;
        let mut x = inner.x + inner.width.saturating_sub(total);
        let cancel = button(
            frame.buffer_mut(),
            x,
            by,
            cancel_label,
            ButtonState::Normal {
                hovered: hovered == Some(RewordPart::Cancel),
                focused: self.focus == RewordPart::Cancel,
            },
        );
        hits.push((cancel, RewordPart::Cancel));
        x += cancel.width + 1;
        let apply = button(
            frame.buffer_mut(),
            x,
            by,
            apply_label,
            ButtonState::Primary {
                hovered: hovered == Some(RewordPart::Apply),
                focused: self.focus == RewordPart::Apply,
            },
        );
        hits.push((apply, RewordPart::Apply));
        if self.deeper_available() {
            let label = format!(
                "{} {}",
                msg::MODIFY_DEEPER.get(),
                crate::keymap::current().shown(Bind::RewordDeeper)
            );
            let width = button_width(&label);
            if inner.width >= total + width + 2 {
                let deeper = button(
                    frame.buffer_mut(),
                    inner.x,
                    by,
                    &label,
                    ButtonState::Normal {
                        hovered: hovered == Some(RewordPart::Deeper),
                        focused: self.focus == RewordPart::Deeper,
                    },
                );
                hits.push((deeper, RewordPart::Deeper));
            }
        }
        hits
    }
}

/// The popup's own target under the pointer, among *its* targets only --
/// the same reason as `remote_hit_at`: it is drawn over the panes but
/// registered after them.
pub(crate) fn reword_hit_at(hits: &[(Rect, HitTarget)], col: u16, row: u16) -> Option<RewordPart> {
    hits.iter().find_map(|(rect, target)| match target {
        HitTarget::RewordForm(part) if rect.contains((col, row).into()) => Some(*part),
        _ => None,
    })
}

#[cfg(test)]
#[path = "reword_ui_tests.rs"]
mod tests;

/// The commit message form moves (its size follows its scope).
const WIN: crate::dialog_kit::Win = crate::dialog_kit::Win::resizable("reword", 50, 12);
