//! The small modals over the review view: the comment box (a line
//! comment, a reply, a conversation comment, an edit of a pending one), the
//! Finish review dialog and the discard confirmation. Each owns its keys
//! and its mouse while open, and registers its hit rects in the walk that
//! paints it.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use gitpoulpi_hosting::{DiffSide, ReviewCapabilities, ReviewEvent, VoteLabel};
use gitpoulpi_i18n::msg::review_ai as ai_msg;
use gitpoulpi_i18n::msg::review_view as msg;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use crate::combo::{Combo, ComboOption};
use crate::dialog_kit::{self, ButtonState};
use crate::keymap::Action as Bind;
use crate::text_edit::{FieldEditor, KeyEdit};
use crate::th;

// -- the comment box -----------------------------------------------------------

/// What a comment box writes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ComposeFor {
    /// A new pending comment on code.
    Line {
        path: String,
        previous_path: Option<String>,
        side: DiffSide,
        line: u32,
        start_line: Option<u32>,
        other_line: Option<u32>,
    },
    /// The text of pending comment `index`.
    EditPending(usize),
    /// A reply, posted at once.
    Reply { thread_id: String },
    /// A comment in the conversation, posted at once.
    Conversation,
    /// The words of ✦ suggestion `id`, edited before it is accepted.
    Suggestion(u64),
    /// The text of the ✦ review draft, before it is used.
    DraftText,
}

impl ComposeFor {
    /// Whether the text reaches the host the moment it is submitted.
    pub(super) fn posts_at_once(&self) -> bool {
        matches!(self, Self::Reply { .. } | Self::Conversation)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ComposeOutcome {
    Stay,
    Cancel,
    Submit(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ComposePart {
    Text,
    Submit,
    Cancel,
}

pub(super) struct ComposeBox {
    pub(super) purpose: ComposeFor,
    title: String,
    /// What the text is about: `src/a.rs:12`, `#9`, a thread's author.
    about: String,
    pub(super) text: String,
    edit: FieldEditor<()>,
    bar: Option<crate::scrollbar::Track>,
    drag: crate::scrollbar::BarDrag,
    focus: ComposePart,
    hover: Option<ComposePart>,
    close: dialog_kit::CloseHit,
    hits: Vec<(Rect, ComposePart)>,
}

impl ComposeBox {
    pub(super) fn new(purpose: ComposeFor, title: String, about: String, text: String) -> Self {
        let mut me = Self {
            purpose,
            title,
            about,
            text,
            edit: FieldEditor::default(),
            bar: None,
            drag: Default::default(),
            focus: ComposePart::Text,
            hover: None,
            close: dialog_kit::CloseHit::default(),
            hits: Vec::new(),
        };
        // The caret starts at the end of what is being edited.
        me.edit.sync(Some(()));
        me
    }

    fn submit(&self) -> ComposeOutcome {
        if self.text.trim().is_empty() {
            ComposeOutcome::Stay
        } else {
            ComposeOutcome::Submit(self.text.trim().to_string())
        }
    }

    pub(super) fn paste(&mut self, text: &str) {
        if self.focus != ComposePart::Text {
            return;
        }
        let clean = text.replace("\r\n", "\n").replace('\r', "\n");
        self.edit.sync(Some(()));
        let mut value = std::mem::take(&mut self.text);
        self.edit.caret_mut(()).insert(&mut value, &clean);
        self.text = value;
    }

    pub(super) fn on_key(&mut self, key: KeyEvent) -> ComposeOutcome {
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        if self.focus == ComposePart::Text {
            self.edit.sync(Some(()));
            if key.code == KeyCode::Enter {
                let mut value = std::mem::take(&mut self.text);
                self.edit.caret_mut(()).insert(&mut value, "\n");
                self.text = value;
                return ComposeOutcome::Stay;
            }
            if self.edit.claims((), &self.text, &key, true) {
                let mut value = std::mem::take(&mut self.text);
                let edit = self.edit.on_key((), &mut value, &key, true);
                self.text = value;
                if let KeyEdit::Copy(s) | KeyEdit::Cut(s) = edit {
                    crate::clipboard::copy_shared(msg::NOUN_SELECTION.get(), &s);
                }
                return ComposeOutcome::Stay;
            }
        }
        match key.code {
            KeyCode::Esc => return ComposeOutcome::Cancel,
            KeyCode::Tab | KeyCode::BackTab => {
                let order = [ComposePart::Text, ComposePart::Submit, ComposePart::Cancel];
                let at = order.iter().position(|p| *p == self.focus).unwrap_or(0);
                self.focus = if shift || key.code == KeyCode::BackTab {
                    order[(at + order.len() - 1) % order.len()]
                } else {
                    order[(at + 1) % order.len()]
                };
            }
            _ if crate::keymap::current().is(Bind::DialogSubmit, &key) => return self.submit(),
            KeyCode::Enter | KeyCode::Char(' ') => {
                return match self.focus {
                    ComposePart::Submit => self.submit(),
                    ComposePart::Cancel => ComposeOutcome::Cancel,
                    ComposePart::Text => ComposeOutcome::Stay,
                }
            }
            _ => {}
        }
        ComposeOutcome::Stay
    }

    pub(super) fn on_mouse(&mut self, mouse: MouseEvent) -> ComposeOutcome {
        let top = self.edit.caret(()).map_or(0, |c| c.top.get());
        let lines = self.text.split('\n').count();
        if let Some(top) = self.drag.on_mouse(self.bar, mouse, lines, top) {
            let rows = self.bar.map_or(1, |t| t.rect.height as usize);
            self.edit
                .caret_mut(())
                .scroll_lines_to(&self.text, top, rows);
            return ComposeOutcome::Stay;
        }
        let hit = dialog_kit::hit_at(&self.hits, mouse.column, mouse.row);
        if !matches!(mouse.kind, MouseEventKind::Down(_)) && self.edit.is_dragging() {
            self.edit
                .dialog_mouse(Some(()), &self.text, mouse.kind, mouse.column, mouse.row);
            return ComposeOutcome::Stay;
        }
        match mouse.kind {
            MouseEventKind::Moved => {
                self.hover = hit;
                self.close.set_hover(mouse.column, mouse.row);
            }
            MouseEventKind::Down(MouseButton::Left) if self.close.hit(mouse.column, mouse.row) => {
                return ComposeOutcome::Cancel;
            }
            MouseEventKind::Down(MouseButton::Left) => match hit {
                Some(ComposePart::Text) => {
                    self.focus = ComposePart::Text;
                    self.edit.sync(Some(()));
                    self.edit.dialog_mouse(
                        Some(()),
                        &self.text,
                        mouse.kind,
                        mouse.column,
                        mouse.row,
                    );
                }
                Some(ComposePart::Submit) => return self.submit(),
                Some(ComposePart::Cancel) => return ComposeOutcome::Cancel,
                None => {}
            },
            MouseEventKind::Up(MouseButton::Left) => {
                self.edit
                    .dialog_mouse(Some(()), &self.text, mouse.kind, mouse.column, mouse.row);
            }
            _ => {}
        }
        ComposeOutcome::Stay
    }

    pub(super) fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let rect = WIN_REVIEW_TEXT.place(area, dialog_kit::centered(area, 84, 16));
        self.edit.begin_frame();
        self.close
            .framed(buf, rect, &self.title, &WIN_REVIEW_TEXT, true);
        let inner = Rect::new(
            rect.x + 2,
            rect.y + 1,
            rect.width.saturating_sub(4),
            rect.height.saturating_sub(2),
        );
        let mut hits: Vec<(Rect, ComposePart)> = Vec::new();
        if inner.width < 20 || inner.height < 6 {
            self.hits = hits;
            return;
        }
        let muted = Style::default().fg(th().ui.muted).bg(th().ui.panel);
        dialog_kit::put(buf, inner.x, inner.y, &self.about, inner.width, muted);
        let note = if self.purpose.posts_at_once() {
            msg::POSTS_AT_ONCE.get()
        } else {
            msg::GOES_TO_PENDING.get()
        };
        let warn = Style::default().fg(th().status.warning).bg(th().ui.panel);
        dialog_kit::put(buf, inner.x, inner.y + 1, note, inner.width, warn);
        let button_y = inner.bottom().saturating_sub(1);
        let text = Rect::new(
            inner.x,
            inner.y + 3,
            inner.width,
            button_y.saturating_sub(inner.y + 4).max(2),
        );
        self.bar = dialog_kit::text_area(
            buf,
            text,
            &self.text,
            msg::COMPOSE_PLACEHOLDER.get(),
            self.focus == ComposePart::Text,
            self.hover == Some(ComposePart::Text),
            (&self.edit, ()),
        );
        let cancel_label = msg::CANCEL.get();
        let submit_label = if self.purpose.posts_at_once() {
            msg::POST.get()
        } else {
            msg::SAVE_COMMENT.get()
        };
        let submit_w = dialog_kit::button_width(submit_label);
        let cancel_w = dialog_kit::button_width(cancel_label);
        let submit_x = inner.right().saturating_sub(submit_w);
        let cancel_x = submit_x.saturating_sub(cancel_w + 1);
        let cancel = dialog_kit::button(
            buf,
            cancel_x,
            button_y,
            cancel_label,
            ButtonState::Normal {
                hovered: self.hover == Some(ComposePart::Cancel),
                focused: self.focus == ComposePart::Cancel,
            },
        );
        let submit = dialog_kit::button(
            buf,
            submit_x,
            button_y,
            submit_label,
            if self.text.trim().is_empty() {
                ButtonState::Disabled
            } else {
                ButtonState::Primary {
                    hovered: self.hover == Some(ComposePart::Submit),
                    focused: self.focus == ComposePart::Submit,
                }
            },
        );
        hits.push((submit, ComposePart::Submit));
        hits.push((cancel, ComposePart::Cancel));
        hits.push((text, ComposePart::Text));
        let key = dialog_kit::key_label(Bind::DialogSubmit);
        dialog_kit::hints(
            buf,
            inner.x,
            button_y,
            cancel_x.saturating_sub(inner.x + 1),
            &[
                (key.as_str(), msg::HINT_SUBMIT.get()),
                (msg::KEY_ESC.get(), msg::HINT_CANCEL.get()),
            ],
        );
        self.hits = hits;
    }
}

// -- Finish review -------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum FinishOutcome {
    Stay,
    Cancel,
    Submit {
        event: ReviewEvent,
        body: String,
        /// The votes that changed (Gerrit): `(label, value)`.
        votes: Vec<(String, i32)>,
    },
    /// Throw the pending comments away (the owner confirms first).
    Discard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FinishPart {
    Body,
    /// "✦ Use AI summary" (only when the AI gave one).
    UseSummary,
    Event,
    /// The combobox of vote label `n` (Gerrit).
    Vote(usize),
    Submit,
    Discard,
    Cancel,
}

/// One label the user may vote on, with the values the server permits.
struct VoteRow {
    name: String,
    combo: Combo,
    /// The vote the user has now: a row left on it sends nothing.
    current: i32,
}

/// `+1  Looks good to me`, `0  No score`, `-2  Do not submit`.
fn vote_text(value: i32, text: &str) -> String {
    let number = if value == 0 {
        "0".to_string()
    } else {
        format!("{value:+}")
    };
    if text.trim().is_empty() {
        number
    } else {
        format!("{number}  {}", text.trim())
    }
}

pub(super) struct FinishDialog {
    body: String,
    edit: FieldEditor<()>,
    bar: Option<crate::scrollbar::Track>,
    drag: crate::scrollbar::BarDrag,
    combo: Combo,
    /// The vote comboboxes (Gerrit): they replace the verdict list.
    votes: Vec<VoteRow>,
    pending: usize,
    /// The AI's overall summary, offered by a button.
    ai_summary: Option<String>,
    focus: FinishPart,
    hover: Option<FinishPart>,
    close: dialog_kit::CloseHit,
    hits: Vec<(Rect, FinishPart)>,
    pub(super) message: Option<String>,
    pub(super) busy: bool,
}

fn event_value(event: ReviewEvent) -> &'static str {
    event.as_str()
}

impl FinishDialog {
    pub(super) fn new(
        caps: ReviewCapabilities,
        pending: usize,
        ai_summary: Option<String>,
        labels: &[VoteLabel],
    ) -> Self {
        let votes: Vec<VoteRow> = labels
            .iter()
            .map(|l| {
                let mut combo = Combo::new(&l.name);
                combo.set_options(
                    l.values
                        .iter()
                        .map(|v| ComboOption::new(vote_text(v.value, &v.text), v.value.to_string()))
                        .collect(),
                );
                combo.select(Some(l.current.to_string()));
                VoteRow {
                    name: l.name.clone(),
                    combo,
                    current: l.current,
                }
            })
            .collect();
        let mut combo = Combo::new(msg::EVENT_PLACEHOLDER.get());
        let mut options = Vec::new();
        for (event, label) in [
            (ReviewEvent::Comment, msg::EVENT_COMMENT.get()),
            (ReviewEvent::Approve, msg::EVENT_APPROVE.get()),
            (
                ReviewEvent::RequestChanges,
                msg::EVENT_REQUEST_CHANGES.get(),
            ),
        ] {
            // Only what the host offers: GitLab has no "request changes".
            if caps.allows(event) {
                options.push(ComboOption::new(label, event_value(event)));
            }
        }
        combo.set_options(options);
        combo.select(Some(event_value(ReviewEvent::Comment).to_string()));
        Self {
            body: String::new(),
            edit: FieldEditor::default(),
            bar: None,
            drag: Default::default(),
            combo,
            votes,
            pending,
            ai_summary,
            focus: FinishPart::Body,
            hover: None,
            close: dialog_kit::CloseHit::default(),
            hits: Vec::new(),
            message: None,
            busy: false,
        }
    }

    /// The verdicts the dropdown offers (tests).
    #[cfg(test)]
    pub(super) fn offered(&self) -> Vec<ReviewEvent> {
        self.combo
            .options
            .iter()
            .filter_map(|o| ReviewEvent::parse(&o.value))
            .collect()
    }

    /// The votes the user changed.
    fn changed_votes(&self) -> Vec<(String, i32)> {
        self.votes
            .iter()
            .filter_map(|r| {
                let value: i32 = r.combo.selected()?.parse().ok()?;
                (value != r.current).then(|| (r.name.clone(), value))
            })
            .collect()
    }

    fn voting(&self) -> bool {
        !self.votes.is_empty()
    }

    fn event(&self) -> ReviewEvent {
        self.combo
            .selected()
            .and_then(ReviewEvent::parse)
            .unwrap_or(ReviewEvent::Comment)
    }

    /// Why Submit does nothing yet.
    fn blocked(&self) -> Option<&'static str> {
        let empty =
            self.body.trim().is_empty() && self.pending == 0 && self.changed_votes().is_empty();
        (empty && self.event() != ReviewEvent::Approve).then(|| msg::NOTHING_TO_SUBMIT.get())
    }

    fn submit(&mut self) -> FinishOutcome {
        if self.busy {
            return FinishOutcome::Stay;
        }
        if let Some(why) = self.blocked() {
            self.message = Some(why.to_string());
            return FinishOutcome::Stay;
        }
        FinishOutcome::Submit {
            event: self.event(),
            body: self.body.trim().to_string(),
            votes: self.changed_votes(),
        }
    }

    pub(super) fn paste(&mut self, text: &str) {
        if self.focus != FinishPart::Body {
            return;
        }
        let clean = text.replace("\r\n", "\n").replace('\r', "\n");
        self.edit.sync(Some(()));
        let mut value = std::mem::take(&mut self.body);
        self.edit.caret_mut(()).insert(&mut value, &clean);
        self.body = value;
    }

    /// Fills the form from the ✦ draft: its text, and its verdict when the
    /// host offers it. Returns what had to change (the host has no such
    /// verdict, or no vote of that size), for the dialog to say.
    pub(super) fn prefill(&mut self, body: &str, event: ReviewEvent) -> Option<String> {
        self.body = body.trim().to_string();
        self.edit.sync(Some(()));
        self.message = None;
        if self.voting() {
            let want = match event {
                ReviewEvent::Approve => 1,
                ReviewEvent::RequestChanges => -1,
                ReviewEvent::Comment => return None,
            };
            let at = self
                .votes
                .iter()
                .position(|r| r.name.eq_ignore_ascii_case("Code-Review"))
                .unwrap_or(0);
            let row = self.votes.get_mut(at)?;
            if row
                .combo
                .options
                .iter()
                .any(|o| o.value == want.to_string())
            {
                row.combo.select(Some(want.to_string()));
                return None;
            }
            return Some(ai_msg::NOTE_NO_VOTE.get().to_string());
        }
        if self
            .combo
            .options
            .iter()
            .any(|o| ReviewEvent::parse(&o.value) == Some(event))
        {
            self.combo.select(Some(event_value(event).to_string()));
            return None;
        }
        Some(
            match event {
                ReviewEvent::RequestChanges => ai_msg::NOTE_NO_REQUEST_CHANGES,
                _ => ai_msg::NOTE_NO_APPROVE,
            }
            .get()
            .to_string(),
        )
    }

    /// The verdict and text the form would send (tests).
    #[cfg(test)]
    pub(super) fn filled(&self) -> (ReviewEvent, String) {
        (self.event(), self.body.clone())
    }

    /// The votes the form would change (tests).
    #[cfg(test)]
    pub(super) fn vote_changes(&self) -> Vec<(String, i32)> {
        self.changed_votes()
    }

    /// Where the "Use AI summary" button was painted (tests).
    #[cfg(test)]
    pub(super) fn summary_button(&self) -> Option<Rect> {
        self.hits
            .iter()
            .find(|(_, p)| *p == FinishPart::UseSummary)
            .map(|(r, _)| *r)
    }

    /// Puts the AI's summary in the review text (after what is there).
    pub(super) fn use_summary(&mut self) {
        let Some(summary) = self.ai_summary.clone() else {
            return;
        };
        if self.body.contains(summary.trim()) {
            return;
        }
        if !self.body.trim().is_empty() {
            if !self.body.ends_with('\n') {
                self.body.push('\n');
            }
            self.body.push('\n');
        }
        self.body.push_str(summary.trim());
        self.edit.sync(Some(()));
        self.message = None;
    }

    fn order(&self) -> Vec<FinishPart> {
        let mut order = vec![FinishPart::Body];
        if self.ai_summary.is_some() {
            order.push(FinishPart::UseSummary);
        }
        if self.voting() {
            order.extend((0..self.votes.len()).map(FinishPart::Vote));
        } else {
            order.push(FinishPart::Event);
        }
        order.push(FinishPart::Submit);
        if self.pending > 0 {
            order.push(FinishPart::Discard);
        }
        order.push(FinishPart::Cancel);
        order
    }

    pub(super) fn on_key(&mut self, key: KeyEvent) -> FinishOutcome {
        if self.combo.is_open() {
            self.combo.on_key(key);
            return FinishOutcome::Stay;
        }
        if let Some(open) = self.votes.iter_mut().find(|r| r.combo.is_open()) {
            open.combo.on_key(key);
            return FinishOutcome::Stay;
        }
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        if self.focus == FinishPart::Body {
            self.edit.sync(Some(()));
            if key.code == KeyCode::Enter {
                let mut value = std::mem::take(&mut self.body);
                self.edit.caret_mut(()).insert(&mut value, "\n");
                self.body = value;
                self.message = None;
                return FinishOutcome::Stay;
            }
            if self.edit.claims((), &self.body, &key, true) {
                let mut value = std::mem::take(&mut self.body);
                let edit = self.edit.on_key((), &mut value, &key, true);
                self.body = value;
                if let KeyEdit::Copy(s) | KeyEdit::Cut(s) = edit {
                    crate::clipboard::copy_shared(msg::NOUN_SELECTION.get(), &s);
                }
                self.message = None;
                return FinishOutcome::Stay;
            }
        }
        if self.focus == FinishPart::Event
            && !matches!(key.code, KeyCode::Tab | KeyCode::BackTab | KeyCode::Esc)
            && self.combo.on_key(key) == crate::combo::ComboOutcome::Handled
        {
            return FinishOutcome::Stay;
        }
        if let FinishPart::Vote(i) = self.focus {
            if !matches!(key.code, KeyCode::Tab | KeyCode::BackTab | KeyCode::Esc) {
                if let Some(row) = self.votes.get_mut(i) {
                    if row.combo.on_key(key) == crate::combo::ComboOutcome::Handled {
                        self.message = None;
                        return FinishOutcome::Stay;
                    }
                }
            }
        }
        match key.code {
            KeyCode::Esc => return FinishOutcome::Cancel,
            KeyCode::Tab | KeyCode::BackTab | KeyCode::Down | KeyCode::Up => {
                let forward =
                    matches!(key.code, KeyCode::Down) || (key.code == KeyCode::Tab && !shift);
                let order = self.order();
                let at = order.iter().position(|p| *p == self.focus).unwrap_or(0);
                let n = order.len();
                self.focus = order[if forward {
                    (at + 1) % n
                } else {
                    (at + n - 1) % n
                }];
            }
            _ if crate::keymap::current().is(Bind::DialogSubmit, &key) => return self.submit(),
            KeyCode::Enter | KeyCode::Char(' ') => {
                if self.focus == FinishPart::UseSummary {
                    self.use_summary();
                    return FinishOutcome::Stay;
                }
                return match self.focus {
                    FinishPart::Submit => self.submit(),
                    FinishPart::Discard => FinishOutcome::Discard,
                    FinishPart::Cancel => FinishOutcome::Cancel,
                    _ => FinishOutcome::Stay,
                };
            }
            _ => {}
        }
        FinishOutcome::Stay
    }

    pub(super) fn on_mouse(&mut self, mouse: MouseEvent) -> FinishOutcome {
        if self.combo.is_open() {
            self.combo.on_mouse(mouse);
            return FinishOutcome::Stay;
        }
        if let Some(open) = self.votes.iter_mut().find(|r| r.combo.is_open()) {
            open.combo.on_mouse(mouse);
            return FinishOutcome::Stay;
        }
        let top = self.edit.caret(()).map_or(0, |c| c.top.get());
        let lines = self.body.split('\n').count();
        if let Some(top) = self.drag.on_mouse(self.bar, mouse, lines, top) {
            let rows = self.bar.map_or(1, |t| t.rect.height as usize);
            self.edit
                .caret_mut(())
                .scroll_lines_to(&self.body, top, rows);
            return FinishOutcome::Stay;
        }
        let hit = dialog_kit::hit_at(&self.hits, mouse.column, mouse.row);
        if !matches!(mouse.kind, MouseEventKind::Down(_)) && self.edit.is_dragging() {
            self.edit
                .dialog_mouse(Some(()), &self.body, mouse.kind, mouse.column, mouse.row);
            return FinishOutcome::Stay;
        }
        match mouse.kind {
            MouseEventKind::Moved => {
                self.hover = hit;
                self.close.set_hover(mouse.column, mouse.row);
            }
            MouseEventKind::Down(MouseButton::Left) if self.close.hit(mouse.column, mouse.row) => {
                return FinishOutcome::Cancel;
            }
            MouseEventKind::Down(MouseButton::Left) => match hit {
                Some(FinishPart::Body) => {
                    self.focus = FinishPart::Body;
                    self.edit.sync(Some(()));
                    self.edit.dialog_mouse(
                        Some(()),
                        &self.body,
                        mouse.kind,
                        mouse.column,
                        mouse.row,
                    );
                }
                Some(FinishPart::UseSummary) => {
                    self.focus = FinishPart::UseSummary;
                    self.use_summary();
                }
                Some(FinishPart::Event) => {
                    self.focus = FinishPart::Event;
                    self.combo.on_mouse(mouse);
                }
                Some(FinishPart::Vote(i)) => {
                    self.focus = FinishPart::Vote(i);
                    if let Some(row) = self.votes.get_mut(i) {
                        row.combo.on_mouse(mouse);
                    }
                }
                Some(FinishPart::Submit) => {
                    self.focus = FinishPart::Submit;
                    return self.submit();
                }
                Some(FinishPart::Discard) => return FinishOutcome::Discard,
                Some(FinishPart::Cancel) => return FinishOutcome::Cancel,
                None => {}
            },
            MouseEventKind::Up(MouseButton::Left) => {
                self.edit
                    .dialog_mouse(Some(()), &self.body, mouse.kind, mouse.column, mouse.row);
            }
            _ => {}
        }
        FinishOutcome::Stay
    }

    pub(super) fn render(&mut self, area: Rect, buf: &mut Buffer, spinner: char) {
        let rect = WIN_REVIEW_FINISH.place(area, dialog_kit::centered(area, 84, 20));
        self.edit.begin_frame();
        self.close
            .framed(buf, rect, msg::FINISH_TITLE.get(), &WIN_REVIEW_FINISH, true);
        let inner = Rect::new(
            rect.x + 2,
            rect.y + 1,
            rect.width.saturating_sub(4),
            rect.height.saturating_sub(2),
        );
        let mut hits: Vec<(Rect, FinishPart)> = Vec::new();
        if inner.width < 30 || inner.height < 10 {
            self.hits = hits;
            return;
        }
        let muted = Style::default().fg(th().ui.muted).bg(th().ui.panel);
        let mut y = inner.y;
        dialog_kit::put(
            buf,
            inner.x,
            y,
            &msg::pending_summary(self.pending as u64),
            inner.width,
            Style::default().fg(th().ui.accent).bg(th().ui.panel),
        );
        y += 2;
        dialog_kit::put(
            buf,
            inner.x,
            y,
            msg::FIELD_SUMMARY.get(),
            inner.width,
            muted,
        );
        if self.ai_summary.is_some() {
            let label = format!("{} {}", crate::theme::AI_SPARK, ai_msg::USE_SUMMARY.get());
            let w = dialog_kit::button_width(&label);
            let rect = dialog_kit::button(
                buf,
                inner.right().saturating_sub(w),
                y,
                &label,
                ButtonState::Normal {
                    hovered: self.hover == Some(FinishPart::UseSummary),
                    focused: self.focus == FinishPart::UseSummary,
                },
            );
            hits.push((rect, FinishPart::UseSummary));
        }
        y += 1;
        let button_y = inner.bottom().saturating_sub(1);
        // One row per vote label instead of the single verdict row.
        let extra = self.votes.len().saturating_sub(1) as u16;
        let body_h = button_y.saturating_sub(y + 5 + extra).clamp(2, 9);
        let body = Rect::new(inner.x, y, inner.width, body_h);
        self.bar = dialog_kit::text_area(
            buf,
            body,
            &self.body,
            msg::SUMMARY_PLACEHOLDER.get(),
            self.focus == FinishPart::Body,
            self.hover == Some(FinishPart::Body),
            (&self.edit, ()),
        );
        hits.push((body, FinishPart::Body));
        y += body_h + 1;
        if self.voting() {
            let (focus, hover) = (self.focus, self.hover);
            for (i, row) in self.votes.iter_mut().enumerate() {
                dialog_kit::put(buf, inner.x, y, &row.name, 14, muted);
                let field = Rect::new(inner.x + 14, y, inner.width.saturating_sub(14).min(56), 1);
                row.combo.render_field(
                    buf,
                    field,
                    focus == FinishPart::Vote(i),
                    hover == Some(FinishPart::Vote(i)),
                );
                hits.push((field, FinishPart::Vote(i)));
                y += 1;
            }
            y += 1;
        } else {
            dialog_kit::put(buf, inner.x, y, msg::FIELD_VERDICT.get(), 14, muted);
            let field = Rect::new(inner.x + 14, y, inner.width.saturating_sub(14).min(34), 1);
            self.combo.render_field(
                buf,
                field,
                self.focus == FinishPart::Event,
                self.hover == Some(FinishPart::Event),
            );
            hits.push((field, FinishPart::Event));
            y += 2;
        }
        let status = if self.busy {
            Some((
                format!("{spinner} {}", msg::SUBMITTING.get()),
                Style::default().fg(th().ui.accent).bg(th().ui.panel),
            ))
        } else if let Some(m) = &self.message {
            Some((
                m.clone(),
                Style::default().fg(th().status.danger).bg(th().ui.panel),
            ))
        } else {
            self.blocked().map(|why| {
                (
                    why.to_string(),
                    Style::default().fg(th().status.warning).bg(th().ui.panel),
                )
            })
        };
        if let Some((text, style)) = status {
            // A host's refusal can be a sentence: wrap it over the rows left
            // above the buttons instead of cutting it with an ellipsis.
            let first = y.min(button_y.saturating_sub(1));
            let rows = usize::from(button_y.saturating_sub(first)).max(1);
            let mut lines = crate::ui_kit::wrap_words(&text, usize::from(inner.width));
            if lines.len() > rows {
                lines.truncate(rows);
            }
            for (i, line) in lines.iter().enumerate() {
                dialog_kit::put(buf, inner.x, first + i as u16, line, inner.width, style);
            }
        }
        // Buttons, right to left: Submit, Cancel, then Discard on the left.
        let submit_label = msg::SUBMIT_REVIEW.get();
        let cancel_label = msg::CANCEL.get();
        let submit_w = dialog_kit::button_width(submit_label);
        let cancel_w = dialog_kit::button_width(cancel_label);
        let submit_x = inner.right().saturating_sub(submit_w);
        let cancel_x = submit_x.saturating_sub(cancel_w + 1);
        let submit = dialog_kit::button(
            buf,
            submit_x,
            button_y,
            submit_label,
            if self.busy || self.blocked().is_some() {
                ButtonState::Disabled
            } else {
                ButtonState::Primary {
                    hovered: self.hover == Some(FinishPart::Submit),
                    focused: self.focus == FinishPart::Submit,
                }
            },
        );
        let cancel = dialog_kit::button(
            buf,
            cancel_x,
            button_y,
            cancel_label,
            ButtonState::Normal {
                hovered: self.hover == Some(FinishPart::Cancel),
                focused: self.focus == FinishPart::Cancel,
            },
        );
        hits.insert(0, (cancel, FinishPart::Cancel));
        hits.insert(0, (submit, FinishPart::Submit));
        let mut left = inner.x;
        if self.pending > 0 {
            let discard = dialog_kit::button(
                buf,
                inner.x,
                button_y,
                msg::DISCARD_REVIEW.get(),
                ButtonState::Normal {
                    hovered: self.hover == Some(FinishPart::Discard),
                    focused: self.focus == FinishPart::Discard,
                },
            );
            left = discard.right() + 1;
            hits.insert(0, (discard, FinishPart::Discard));
        }
        let key = dialog_kit::key_label(Bind::DialogSubmit);
        dialog_kit::hints(
            buf,
            left,
            button_y,
            cancel_x.saturating_sub(left + 1),
            &[(key.as_str(), msg::HINT_SUBMIT.get())],
        );
        self.hits = hits;
        self.combo.render_popup(buf, buf.area);
        for row in &mut self.votes {
            row.combo.render_popup(buf, buf.area);
        }
    }
}

// -- a yes / no question -------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ConfirmOutcome {
    Stay,
    Yes,
    No,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfirmPart {
    Yes,
    No,
}

pub(super) struct Confirm {
    title: &'static str,
    prompt: String,
    yes: &'static str,
    focus: ConfirmPart,
    hover: Option<ConfirmPart>,
    close: dialog_kit::CloseHit,
    hits: Vec<(Rect, ConfirmPart)>,
}

impl Confirm {
    pub(super) fn new(title: &'static str, prompt: String, yes: &'static str) -> Self {
        Self {
            title,
            prompt,
            yes,
            focus: ConfirmPart::No,
            hover: None,
            close: dialog_kit::CloseHit::default(),
            hits: Vec::new(),
        }
    }

    pub(super) fn on_key(&mut self, key: KeyEvent) -> ConfirmOutcome {
        match key.code {
            KeyCode::Esc => ConfirmOutcome::No,
            KeyCode::Tab | KeyCode::BackTab | KeyCode::Left | KeyCode::Right => {
                self.focus = match self.focus {
                    ConfirmPart::Yes => ConfirmPart::No,
                    ConfirmPart::No => ConfirmPart::Yes,
                };
                ConfirmOutcome::Stay
            }
            KeyCode::Enter | KeyCode::Char(' ') => match self.focus {
                ConfirmPart::Yes => ConfirmOutcome::Yes,
                ConfirmPart::No => ConfirmOutcome::No,
            },
            _ => ConfirmOutcome::Stay,
        }
    }

    pub(super) fn on_mouse(&mut self, mouse: MouseEvent) -> ConfirmOutcome {
        let hit = dialog_kit::hit_at(&self.hits, mouse.column, mouse.row);
        match mouse.kind {
            MouseEventKind::Moved => {
                self.hover = hit;
                self.close.set_hover(mouse.column, mouse.row);
            }
            MouseEventKind::Down(MouseButton::Left) if self.close.hit(mouse.column, mouse.row) => {
                return ConfirmOutcome::No;
            }
            MouseEventKind::Down(MouseButton::Left) => match hit {
                Some(ConfirmPart::Yes) => return ConfirmOutcome::Yes,
                Some(ConfirmPart::No) => return ConfirmOutcome::No,
                None => {}
            },
            _ => {}
        }
        ConfirmOutcome::Stay
    }

    pub(super) fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let rect = WIN_REVIEW_PICK.place(area, dialog_kit::centered(area, 64, 8));
        self.close
            .framed(buf, rect, self.title, &WIN_REVIEW_PICK, true);
        let inner = Rect::new(
            rect.x + 2,
            rect.y + 1,
            rect.width.saturating_sub(4),
            rect.height.saturating_sub(2),
        );
        // The question wraps over up to three lines (a long one is not cut).
        let mut lines: Vec<String> = vec![String::new()];
        for word in self.prompt.split_whitespace() {
            let last = lines.last_mut().expect("one line");
            if !last.is_empty()
                && crate::display_width(last) + 1 + crate::display_width(word)
                    > inner.width as usize
            {
                lines.push(word.to_string());
            } else {
                if !last.is_empty() {
                    last.push(' ');
                }
                last.push_str(word);
            }
        }
        for (i, line) in lines.iter().take(3).enumerate() {
            dialog_kit::put(
                buf,
                inner.x,
                inner.y + 1 + i as u16,
                line,
                inner.width,
                Style::default().fg(th().ui.text).bg(th().ui.panel),
            );
        }
        let y = inner.bottom().saturating_sub(1);
        let no_label = msg::KEEP.get();
        let no_w = dialog_kit::button_width(no_label);
        let yes_w = dialog_kit::button_width(self.yes);
        let yes_x = inner.right().saturating_sub(yes_w);
        let no_x = yes_x.saturating_sub(no_w + 1);
        let no = dialog_kit::button(
            buf,
            no_x,
            y,
            no_label,
            ButtonState::Normal {
                hovered: self.hover == Some(ConfirmPart::No),
                focused: self.focus == ConfirmPart::No,
            },
        );
        let yes = dialog_kit::button(
            buf,
            yes_x,
            y,
            self.yes,
            ButtonState::Normal {
                hovered: self.hover == Some(ConfirmPart::Yes),
                focused: self.focus == ConfirmPart::Yes,
            },
        );
        self.hits = vec![(yes, ConfirmPart::Yes), (no, ConfirmPart::No)];
    }
}

/// The dialog moves (its size follows its content).
const WIN_REVIEW_TEXT: crate::dialog_kit::Win = crate::dialog_kit::Win::movable("review_text");

/// The dialog moves (its size follows its content).
const WIN_REVIEW_FINISH: crate::dialog_kit::Win = crate::dialog_kit::Win::movable("review_finish");

/// The dialog moves (its size follows its content).
const WIN_REVIEW_PICK: crate::dialog_kit::Win = crate::dialog_kit::Win::movable("review_pick");
