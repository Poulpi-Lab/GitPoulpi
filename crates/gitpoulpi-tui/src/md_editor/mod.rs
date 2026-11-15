//! The Markdown composer: one widget for every text that goes to a host
//! (the review comment box and Finish summary, the new-issue description,
//! the pull request body; the conversation reply to come).
//!
//! Three modes, picked with three real buttons or with `Alt+P` (Write and
//! Preview) and `Alt+L` (Split): **Write** is the text field, **Preview**
//! shows the text as `md_render` will draw it once posted (the host's
//! references, `#N`, `!N`, mentions and commit ids, already links), **Split**
//! shows both side by side when the editor is [`SPLIT_MIN_WIDTH`] cells wide
//! or more. The Preview of a long text is laid out on a worker (`preview`),
//! never on the render thread. Drafts are kept per target (`draft`).
//!
//! The widget owns its text, its caret and its scrolling; the form around it
//! owns focus, the close question (`dialog_kit::Dismiss`) and the buttons.
//! The form routes keys to [`MdEditor::on_key`] while the editor has focus
//! and every mouse event to [`MdEditor::on_mouse`]. Nothing is ever typed in
//! the status line: the field lives in the dialog.
//!
//! The top row is the mode bar. Its right end holds the "Write with AI"
//! button (`ai_slot`) once the owner bound the helper (`bind_ai`, only while
//! AI is on): `Alt+W` or the button opens its panel under the bar (`ai`), the
//! draft it writes fills the text in one undo step and switches to Preview.

// Not every build shows a form with a Markdown field.
#![allow(dead_code)]

pub(crate) mod ai;
#[cfg_attr(not(feature = "ai"), path = "ai_worker_stub.rs")]
mod ai_worker;
pub(crate) mod draft;
pub(crate) mod preview;
#[cfg(test)]
mod ai_tests;
#[cfg(test)]
mod tests;

use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use gitpoulpi_i18n::msg::md_editor as msg;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use crate::dialog_kit::{self, ButtonState};
use crate::keymap::Action as Bind;
use crate::text_edit::{FieldEditor, KeyEdit};
use crate::{md_render, th};

/// The editor is wide enough for two panes at this many cells.
pub(crate) const SPLIT_MIN_WIDTH: u16 = 100;
/// Cells reserved at the right of the mode bar for the AI button.
const AI_SLOT_WIDTH: u16 = 22;
/// The mode bar, then at least this many rows of body.
pub(crate) const MIN_HEIGHT: u16 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Mode {
    #[default]
    Write,
    Preview,
    Split,
}

/// What a key meant to the editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KeyUse {
    /// The editor took it (an edit, a caret move, a mode switch, a scroll).
    Handled,
    /// The send key: the form decides whether the text may go.
    Submit,
    /// Not the editor's: Tab, Esc, the form's own keys.
    Pass,
}

/// Where a mouse event landed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MouseUse {
    /// Not on the editor (and no drag of its own in progress).
    Outside,
    /// On the editor: the form gives it the keyboard on a press.
    Inside,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Part {
    Write,
    Preview,
    Split,
    Text,
    Pane,
    Ai,
}

/// A draft kept for the editor's target.
struct Draft {
    key: String,
    dir: Option<std::path::PathBuf>,
    /// The last text written (or restored), to skip writing the same twice.
    saved: String,
    /// When the text last changed and is not written yet.
    edited: Option<Instant>,
    /// Sent or thrown away: nothing is left behind on drop.
    settled: bool,
}

pub(crate) struct MdEditor {
    pub(crate) text: String,
    edit: FieldEditor<()>,
    bar: Option<crate::scrollbar::Track>,
    drag: crate::scrollbar::BarDrag,
    mode: Mode,
    preview: preview::Preview,
    ctx: md_render::Context,
    /// "Sent as Jira wiki markup": the host does not read Markdown.
    sent_as: Option<String>,
    /// A short line on the mode bar (why Split did nothing).
    note: Option<String>,
    hits: Vec<(Rect, Part)>,
    hover: Option<Part>,
    area: Rect,
    text_rect: Rect,
    pane_rect: Rect,
    ai_slot: Rect,
    /// Rows of the preview pane as last painted, and lines of its text.
    pane_rows: usize,
    pv_len: usize,
    draft: Option<Draft>,
    /// The "Write with AI" helper, bound by the owner while AI is on.
    ai: Option<ai::AiHelp>,
}

impl MdEditor {
    pub(crate) fn new(text: String, ctx: md_render::Context) -> Self {
        let mut me = Self {
            text,
            edit: FieldEditor::default(),
            bar: None,
            drag: Default::default(),
            mode: Mode::Write,
            preview: Default::default(),
            ctx,
            sent_as: None,
            note: None,
            hits: Vec::new(),
            hover: None,
            area: Rect::default(),
            text_rect: Rect::default(),
            pane_rect: Rect::default(),
            ai_slot: Rect::default(),
            pane_rows: 1,
            pv_len: 0,
            draft: None,
            ai: None,
        };
        // The caret starts at the end of what is being edited.
        me.edit.sync(Some(()));
        me
    }

    /// Keeps a draft of this editor's text under `key` (see `draft`): an
    /// empty editor takes the saved one back.
    pub(crate) fn with_draft(self, key: String) -> Self {
        self.with_draft_in(key, draft::config_dir())
    }

    pub(crate) fn with_draft_in(mut self, key: String, dir: Option<std::path::PathBuf>) -> Self {
        let mut saved = String::new();
        if let Some(dir) = &dir {
            if self.text.trim().is_empty() {
                if let Some(text) = draft::load(dir, &key) {
                    self.text = text.clone();
                    saved = text;
                    self.edit.sync(Some(()));
                }
            }
        }
        self.draft = Some(Draft {
            key,
            dir,
            saved,
            edited: None,
            settled: false,
        });
        self
    }

    /// Gives the editor its "Write with AI" help: `source` says what the text
    /// is about. Called only while AI is on; without it there is no button and
    /// `Alt+W` does nothing.
    pub(crate) fn bind_ai(&mut self, scope: ai::AiScope, source: Box<dyn ai::AiContextSource>) {
        self.ai = Some(ai::AiHelp::new(scope, source));
    }

    #[cfg(test)]
    pub(crate) fn bind_ai_with(&mut self, help: ai::AiHelp) {
        self.ai = Some(help);
    }

    #[cfg(test)]
    pub(crate) fn ai_mut(&mut self) -> Option<&mut ai::AiHelp> {
        self.ai.as_mut()
    }

    /// Whether the helper is bound (the button is shown).
    pub(crate) fn ai_bound(&self) -> bool {
        self.ai.is_some()
    }

    /// Whether `key` is the helper's own (`Alt+W`): a form whose own `Alt+W`
    /// means something else gives it to the editor while the text has focus.
    pub(crate) fn takes_ai_key(&self, key: &KeyEvent) -> bool {
        self.ai.is_some() && crate::keymap::current().is(Bind::MdEditorAi, key)
    }

    /// Opens (or closes) the "Write with AI" panel.
    pub(crate) fn toggle_ai(&mut self) {
        let has_text = !self.is_blank();
        if let Some(ai) = self.ai.as_mut() {
            ai.toggle(has_text);
        }
    }

    /// Whether the panel is open, and whether a request is running.
    pub(crate) fn ai_open(&self) -> bool {
        self.ai.as_ref().is_some_and(|a| a.is_open())
    }

    pub(crate) fn ai_running(&self) -> bool {
        self.ai.as_ref().is_some_and(|a| a.running())
    }

    /// The rects of the bar's buttons and the panel's controls, for the
    /// form's `Ctrl+N` badges.
    pub(crate) fn nav_rects(&self) -> Vec<Rect> {
        let mut rects: Vec<Rect> = self
            .hits
            .iter()
            .filter(|(_, p)| matches!(p, Part::Write | Part::Preview | Part::Split | Part::Ai))
            .map(|(r, _)| *r)
            .collect();
        if let Some(ai) = &self.ai {
            rects.extend(ai.rects());
        }
        rects
    }

    /// Runs the helper over the current text.
    fn start_ai(&mut self) {
        let text = self.text.clone();
        if let Some(ai) = self.ai.as_mut() {
            ai.start(&text);
        }
    }

    /// The answer of the helper: a draft replaces the text (one undo step)
    /// and the Preview shows it; a failure stays in the panel.
    fn take_ai_answer(&mut self) {
        let Some(answer) = self.ai.as_mut().and_then(|a| a.poll()) else {
            return;
        };
        if let Some(ai) = self.ai.as_mut() {
            ai.finished(&answer);
        }
        if let Ok(draft) = answer {
            self.fill_with(&draft);
        }
    }

    /// Replaces the whole text by `draft` as one edit of the field's history,
    /// so `Ctrl+Z` gives the previous text back, and shows it in Preview.
    pub(crate) fn fill_with(&mut self, draft: &str) {
        self.edit.sync(Some(()));
        let mut value = std::mem::take(&mut self.text);
        let caret = self.edit.caret_mut(());
        caret.select_all(&value);
        caret.insert(&mut value, draft);
        self.text = value;
        self.touched();
        self.preview.top = 0;
        self.set_mode(Mode::Preview);
    }

    /// `Ctrl+Z` / `Ctrl+Y` while the Preview shows: the text goes back (or
    /// forward) and the Write mode shows it.
    fn undo_in_preview(&mut self, redo: bool) {
        self.edit.sync(Some(()));
        let mut value = std::mem::take(&mut self.text);
        let caret = self.edit.caret_mut(());
        let done = if redo {
            caret.redo(&mut value)
        } else {
            caret.undo(&mut value)
        };
        self.text = value;
        if done {
            self.touched();
            self.mode = Mode::Write;
        }
    }

    /// The text goes to the host as `format` (Jira wiki markup, Textile):
    /// the Preview says so.
    pub(crate) fn set_sent_as(&mut self, format: Option<String>) {
        self.sent_as = format;
    }

    pub(crate) fn set_context(&mut self, ctx: md_render::Context) {
        self.ctx = ctx;
    }

    pub(crate) fn context(&self) -> &md_render::Context {
        &self.ctx
    }

    pub(crate) fn mode(&self) -> Mode {
        self.mode
    }

    pub(crate) fn text(&self) -> &str {
        &self.text
    }

    pub(crate) fn is_blank(&self) -> bool {
        self.text.trim().is_empty()
    }

    /// Replaces the whole text (a generated body, a restored comment); the
    /// caret goes to the end.
    pub(crate) fn set_text(&mut self, text: String) {
        self.text = text;
        self.edit.reset();
        self.edit.sync(Some(()));
        self.touched();
    }

    /// Adds `text` at the end, after a blank line when something is there.
    pub(crate) fn append_block(&mut self, text: &str) {
        if !self.text.trim().is_empty() {
            if !self.text.ends_with('\n') {
                self.text.push('\n');
            }
            self.text.push('\n');
        }
        self.text.push_str(text);
        self.edit.sync(Some(()));
        self.touched();
    }

    /// The area the editor was last painted in (tests, hit routing).
    pub(crate) fn area(&self) -> Rect {
        self.area
    }

    /// Where the "Write with AI" button goes; empty until it is built.
    pub(crate) fn ai_slot(&self) -> Rect {
        self.ai_slot
    }

    /// The two footer hints of the editor: `(key, what)`.
    pub(crate) fn hints() -> [(String, &'static str); 2] {
        [
            (
                dialog_kit::key_label(Bind::MdEditorPreview),
                msg::HINT_PREVIEW.get(),
            ),
            (
                dialog_kit::key_label(Bind::MdEditorSplit),
                msg::HINT_SPLIT.get(),
            ),
        ]
    }

    // -- modes -------------------------------------------------------------

    fn split_fits(&self) -> bool {
        self.area.width >= SPLIT_MIN_WIDTH
    }

    pub(crate) fn set_mode(&mut self, mode: Mode) {
        self.note = None;
        if mode == Mode::Split && !self.split_fits() {
            self.note = Some(msg::SPLIT_NEEDS_WIDTH.get().to_string());
            return;
        }
        self.mode = mode;
    }

    /// `Alt+P`: Write and Preview swap; from Split it goes to Preview.
    pub(crate) fn toggle_preview(&mut self) {
        self.set_mode(if self.mode == Mode::Preview {
            Mode::Write
        } else {
            Mode::Preview
        });
    }

    /// `Alt+L`: Split on, or back to Write.
    pub(crate) fn toggle_split(&mut self) {
        self.set_mode(if self.mode == Mode::Split {
            Mode::Write
        } else {
            Mode::Split
        });
    }

    // -- the text ------------------------------------------------------------

    fn touched(&mut self) {
        self.note = None;
        if let Some(d) = self.draft.as_mut() {
            d.edited = Some(Instant::now());
        }
    }

    pub(crate) fn paste(&mut self, text: &str) {
        if self.mode == Mode::Preview {
            return;
        }
        let clean = text.replace("\r\n", "\n").replace('\r', "\n");
        self.edit.sync(Some(()));
        let mut value = std::mem::take(&mut self.text);
        self.edit.caret_mut(()).insert(&mut value, &clean);
        self.text = value;
        self.touched();
    }

    /// Inserts `text` at the caret (an issue reference).
    pub(crate) fn insert(&mut self, text: &str) {
        self.paste(text);
    }

    fn scroll_preview(&mut self, delta: isize) {
        self.preview.top = (self.preview.top as isize + delta).max(0) as usize;
    }

    /// One key, while the editor has the keyboard.
    pub(crate) fn on_key(&mut self, key: KeyEvent) -> KeyUse {
        let km = crate::keymap::current();
        if km.is(Bind::MdEditorSubmit, &key) {
            return KeyUse::Submit;
        }
        if km.is(Bind::MdEditorPreview, &key) {
            self.toggle_preview();
            return KeyUse::Handled;
        }
        if km.is(Bind::MdEditorSplit, &key) {
            self.toggle_split();
            return KeyUse::Handled;
        }
        if self.ai.is_some() && km.is(Bind::MdEditorAi, &key) {
            self.toggle_ai();
            return KeyUse::Handled;
        }
        if self.ai.as_ref().is_some_and(|a| a.has_focus()) {
            let used = self.ai.as_mut().map_or(ai::AiKey::Pass, |a| a.on_key(key));
            match used {
                ai::AiKey::Handled | ai::AiKey::Closed => return KeyUse::Handled,
                ai::AiKey::Start => {
                    self.start_ai();
                    return KeyUse::Handled;
                }
                ai::AiKey::Pass => {}
            }
        }
        if self.mode == Mode::Preview {
            let page = self.pane_rows.max(1) as isize;
            let ctrl = key
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL);
            match key.code {
                KeyCode::Char('z' | 'Z') if ctrl => self.undo_in_preview(false),
                KeyCode::Char('y' | 'Y') if ctrl => self.undo_in_preview(true),
                KeyCode::Up => self.scroll_preview(-1),
                KeyCode::Down => self.scroll_preview(1),
                KeyCode::PageUp => self.scroll_preview(-page),
                KeyCode::PageDown => self.scroll_preview(page),
                KeyCode::Home => self.preview.top = 0,
                KeyCode::End => self.preview.top = usize::MAX / 2,
                _ => return KeyUse::Pass,
            }
            return KeyUse::Handled;
        }
        self.edit.sync(Some(()));
        if key.code == KeyCode::Enter {
            let mut value = std::mem::take(&mut self.text);
            self.edit.caret_mut(()).insert(&mut value, "\n");
            self.text = value;
            self.touched();
            return KeyUse::Handled;
        }
        if self.edit.claims((), &self.text, &key, true) {
            let mut value = std::mem::take(&mut self.text);
            let edit = self.edit.on_key((), &mut value, &key, true);
            self.text = value;
            match edit {
                KeyEdit::Copy(s) => {
                    crate::clipboard::copy_shared(msg_selection(), &s);
                }
                KeyEdit::Cut(s) => {
                    crate::clipboard::copy_shared(msg_selection(), &s);
                    self.touched();
                }
                KeyEdit::Edited => self.touched(),
                KeyEdit::Moved | KeyEdit::Ignored => {}
            }
            return KeyUse::Handled;
        }
        KeyUse::Pass
    }

    /// One mouse event. The form calls this for every event while it is
    /// open; `Inside` means it landed on the editor.
    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent) -> MouseUse {
        let pos = (mouse.column, mouse.row);
        let at = || pos.into();
        // The two scrollbars.
        let top = self.edit.caret(()).map_or(0, |c| c.top.get());
        let lines = self.text.split('\n').count();
        if let Some(top) = self.drag.on_mouse(self.bar, mouse, lines, top) {
            let rows = self.bar.map_or(1, |t| t.rect.height as usize);
            self.edit
                .caret_mut(())
                .scroll_lines_to(&self.text, top, rows);
            return MouseUse::Inside;
        }
        let len = self.pv_len;
        if let Some(top) =
            self.preview
                .drag
                .on_mouse(self.preview.bar, mouse, len, self.preview.top)
        {
            self.preview.top = top;
            return MouseUse::Inside;
        }
        // A selection drag goes on wherever the pointer is.
        if !matches!(mouse.kind, MouseEventKind::Down(_)) && self.edit.is_dragging() {
            self.edit.area_mouse((), &self.text, mouse.kind, pos, false);
            return MouseUse::Inside;
        }
        if !self.area.contains(at()) {
            if matches!(mouse.kind, MouseEventKind::Moved) {
                self.hover = None;
            }
            return MouseUse::Outside;
        }
        match self.ai.as_mut().map(|a| a.on_mouse(mouse)) {
            Some(ai::AiMouse::Handled | ai::AiMouse::Closed) => return MouseUse::Inside,
            Some(ai::AiMouse::Start) => {
                self.start_ai();
                return MouseUse::Inside;
            }
            Some(ai::AiMouse::Outside) | None => {}
        }
        let hit = dialog_kit::hit_at(&self.hits, mouse.column, mouse.row);
        match mouse.kind {
            MouseEventKind::Moved => self.hover = hit,
            MouseEventKind::Down(MouseButton::Left) => match hit {
                Some(Part::Ai) => self.toggle_ai(),
                Some(Part::Write) => self.set_mode(Mode::Write),
                Some(Part::Preview) => self.set_mode(Mode::Preview),
                Some(Part::Split) => self.set_mode(Mode::Split),
                Some(Part::Text) => {
                    if let Some(ai) = self.ai.as_mut() {
                        ai.focus_text();
                    }
                    self.edit.sync(Some(()));
                    let shift = mouse
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::SHIFT);
                    self.edit.area_mouse((), &self.text, mouse.kind, pos, shift);
                }
                _ => {}
            },
            MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
                let step: isize = if mouse.kind == MouseEventKind::ScrollUp {
                    -3
                } else {
                    3
                };
                if self.pane_rect.contains(at()) {
                    self.scroll_preview(step);
                } else {
                    self.edit.area_mouse((), &self.text, mouse.kind, pos, false);
                }
            }
            MouseEventKind::Drag(MouseButton::Left) | MouseEventKind::Up(MouseButton::Left) => {
                self.edit.area_mouse((), &self.text, mouse.kind, pos, false);
            }
            _ => {}
        }
        MouseUse::Inside
    }

    // -- drafts ----------------------------------------------------------------

    /// Writes the draft once the typing has paused (called from the paint).
    fn save_when_idle(&mut self) {
        let Some(d) = self.draft.as_mut() else {
            return;
        };
        if d.edited.is_some_and(|t| t.elapsed() >= draft::IDLE) {
            Self::write_draft(d, &self.text);
        }
    }

    fn write_draft(d: &mut Draft, text: &str) {
        d.edited = None;
        if d.saved == text {
            return;
        }
        if let Some(dir) = &d.dir {
            draft::save(dir, &d.key, text);
        }
        d.saved = text.to_string();
    }

    /// The text was sent: its draft is gone.
    pub(crate) fn sent(&mut self) {
        self.forget();
    }

    /// The text was thrown away: its draft is gone.
    pub(crate) fn discard(&mut self) {
        self.forget();
    }

    fn forget(&mut self) {
        if let Some(d) = self.draft.as_mut() {
            d.settled = true;
            if let Some(dir) = &d.dir {
                draft::remove(dir, &d.key);
            }
        }
    }

    // -- painting ----------------------------------------------------------------

    /// Paints the editor into `rect`. `focused` gives the text its caret.
    pub(crate) fn render(
        &mut self,
        buf: &mut Buffer,
        rect: Rect,
        focused: bool,
        placeholder: &str,
    ) {
        self.save_when_idle();
        self.take_ai_answer();
        self.edit.begin_frame();
        self.area = rect;
        self.hits.clear();
        self.bar = None;
        self.preview.bar = None;
        self.text_rect = Rect::default();
        self.pane_rect = Rect::default();
        self.ai_slot = Rect::default();
        if rect.width < 12 || rect.height == 0 {
            return;
        }
        if self.mode == Mode::Split && !self.split_fits() {
            self.mode = Mode::Write;
        }
        let tight = rect.height < MIN_HEIGHT;
        let mut body = if tight {
            rect
        } else {
            Rect::new(rect.x, rect.y + 1, rect.width, rect.height - 1)
        };
        // The AI panel sits under the bar and keeps the text its rows.
        let panel_rows = if tight {
            0
        } else {
            self.ai.as_ref().map_or(0, |a| a.height(body.height))
        };
        if panel_rows > 0 {
            let panel = Rect::new(body.x, body.y, body.width, panel_rows);
            if let Some(ai) = self.ai.as_mut() {
                ai.render(buf, panel);
            }
            body = Rect::new(
                body.x,
                body.y + panel_rows,
                body.width,
                body.height - panel_rows,
            );
        } else if let Some(ai) = self.ai.as_mut() {
            ai.render(buf, Rect::default());
        }
        let focused = focused && !self.ai.as_ref().is_some_and(|a| a.has_focus());
        match self.mode {
            Mode::Write => self.paint_text(buf, body, focused, placeholder),
            Mode::Preview => self.paint_preview(buf, body),
            Mode::Split => {
                let left = (body.width.saturating_sub(1)) / 2;
                let text = Rect::new(body.x, body.y, left, body.height);
                let pane = Rect::new(
                    body.x + left + 1,
                    body.y,
                    body.width.saturating_sub(left + 1),
                    body.height,
                );
                self.paint_text(buf, text, focused, placeholder);
                self.paint_preview(buf, pane);
            }
        }
        // After the body: the bar says whether the Preview is still being
        // laid out, which the body's paint just found out.
        if !tight {
            self.paint_bar(buf, rect);
        }
    }

    fn paint_bar(&mut self, buf: &mut Buffer, rect: Rect) {
        let panel = Style::default().bg(th().ui.panel);
        dialog_kit::fill(buf, Rect::new(rect.x, rect.y, rect.width, 1), panel);
        let slot = AI_SLOT_WIDTH.min(rect.width / 3);
        self.ai_slot = Rect::new(rect.right().saturating_sub(slot), rect.y, slot, 1);
        let mut x = rect.x;
        let split_ok = self.split_fits();
        for (part, mode, label) in [
            (Part::Write, Mode::Write, msg::MODE_WRITE.get()),
            (Part::Preview, Mode::Preview, msg::MODE_PREVIEW.get()),
            (Part::Split, Mode::Split, msg::MODE_SPLIT.get()),
        ] {
            let hovered = self.hover == Some(part);
            let state = if mode == Mode::Split && !split_ok {
                ButtonState::Disabled
            } else if self.mode == mode {
                ButtonState::Primary {
                    hovered,
                    focused: false,
                }
            } else {
                ButtonState::Normal {
                    hovered,
                    focused: false,
                }
            };
            if x + dialog_kit::button_width(label) > self.ai_slot.x {
                break;
            }
            let r = dialog_kit::button(buf, x, rect.y, label, state);
            self.hits.push((r, part));
            x = r.right() + 1;
        }
        let muted = Style::default().fg(th().ui.muted).bg(th().ui.panel);
        if let Some(ai) = &self.ai {
            let state = if ai.is_open() {
                ButtonState::Primary {
                    hovered: self.hover == Some(Part::Ai),
                    focused: false,
                }
            } else {
                ButtonState::Normal {
                    hovered: self.hover == Some(Part::Ai),
                    focused: false,
                }
            };
            let full = format!("{} {}", crate::theme::AI_SPARK, msg::AI_BUTTON.get());
            let short = format!("{} {}", crate::theme::AI_SPARK, msg::AI_BUTTON_SHORT.get());
            let label = [full, short]
                .into_iter()
                .find(|l| dialog_kit::button_width(l) <= self.ai_slot.width);
            if let Some(label) = label {
                let w = dialog_kit::button_width(&label);
                let bx = rect.right().saturating_sub(w);
                let r = dialog_kit::button(buf, bx, rect.y, &label, state);
                self.hits.push((r, Part::Ai));
                self.ai_slot = r;
            } else {
                self.ai_slot = Rect::new(rect.right(), rect.y, 0, 1);
            }
        }
        // The note stops where the AI button starts.
        let room = if self.ai.is_some() {
            self.ai_slot.x.saturating_sub(x + 2)
        } else {
            rect.right().saturating_sub(x + 1)
        };
        if let Some(note) = self.note.as_deref() {
            dialog_kit::put(
                buf,
                x + 1,
                rect.y,
                note,
                room,
                Style::default().fg(th().status.warning).bg(th().ui.panel),
            );
        } else if self.preview.updating() && self.mode != Mode::Write {
            dialog_kit::put(buf, x + 1, rect.y, msg::PREVIEW_UPDATING.get(), room, muted);
        }
    }

    fn paint_text(&mut self, buf: &mut Buffer, rect: Rect, focused: bool, placeholder: &str) {
        if rect.width < 4 || rect.height == 0 {
            return;
        }
        self.text_rect = rect;
        self.bar = dialog_kit::text_area(
            buf,
            rect,
            &self.text,
            placeholder,
            focused && self.mode != Mode::Preview,
            self.hover == Some(Part::Text),
            (&self.edit, ()),
        );
        self.hits.push((rect, Part::Text));
    }

    fn paint_preview(&mut self, buf: &mut Buffer, rect: Rect) {
        if rect.width < 4 || rect.height == 0 {
            return;
        }
        self.pane_rect = rect;
        self.hits.push((rect, Part::Pane));
        let base = Style::default().fg(th().ui.text).bg(th().ui.panel);
        dialog_kit::fill(buf, rect, base);
        let note_rows = u16::from(self.sent_as.is_some() && rect.height > 2);
        let rows = rect.height.saturating_sub(note_rows);
        self.pane_rows = usize::from(rows.max(1));
        let width = usize::from(rect.width.saturating_sub(2));
        if self.text.trim().is_empty() {
            dialog_kit::put(
                buf,
                rect.x + 1,
                rect.y,
                msg::PREVIEW_EMPTY.get(),
                rect.width.saturating_sub(1),
                Style::default().fg(th().ui.muted).bg(th().ui.panel),
            );
        } else {
            let (doc, _) = self.preview.doc(&self.text, width, &self.ctx);
            if let Some(doc) = doc {
                let len = doc.lines.len();
                self.pv_len = len;
                let top = self.preview.top.min(len.saturating_sub(usize::from(rows)));
                self.preview.top = top;
                for i in 0..usize::from(rows) {
                    if let Some(line) = doc.lines.get(top + i) {
                        md_render::paint(
                            buf,
                            rect.x + 1,
                            rect.y + i as u16,
                            rect.width.saturating_sub(2),
                            line,
                            base,
                            false,
                        );
                    }
                }
                let strip = Rect::new(rect.right() - 1, rect.y, 1, rows);
                self.preview.bar =
                    crate::scrollbar::Track::column(strip, len, usize::from(rows), top);
                if let Some(track) = self.preview.bar {
                    crate::scrollbar::render_buf(buf, track, true, false);
                }
            }
        }
        if let Some(format) = &self.sent_as {
            dialog_kit::put(
                buf,
                rect.x + 1,
                rect.bottom() - 1,
                &msg::sent_as(format),
                rect.width.saturating_sub(1),
                Style::default().fg(th().ui.muted).bg(th().ui.panel),
            );
        }
    }
}

fn msg_selection() -> &'static str {
    gitpoulpi_i18n::msg::review_view::NOUN_SELECTION.get()
}

impl Drop for MdEditor {
    fn drop(&mut self) {
        if let Some(d) = self.draft.as_mut() {
            if !d.settled {
                Self::write_draft(d, &self.text);
            }
        }
    }
}
