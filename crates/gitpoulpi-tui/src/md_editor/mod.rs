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
//! The top row is the mode bar. Its right end is reserved for the "Write
//! with AI" button (`ai_slot`), hidden until that feature lands.

// Not every build shows a form with a Markdown field.
#![allow(dead_code)]

pub(crate) mod draft;
pub(crate) mod preview;
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
        if self.mode == Mode::Preview {
            let page = self.pane_rows.max(1) as isize;
            match key.code {
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
        if let Some(top) = self
            .preview
            .drag
            .on_mouse(self.preview.bar, mouse, len, self.preview.top)
        {
            self.preview.top = top;
            return MouseUse::Inside;
        }
        // A selection drag goes on wherever the pointer is.
        if !matches!(mouse.kind, MouseEventKind::Down(_)) && self.edit.is_dragging() {
            self.edit
                .area_mouse((), &self.text, mouse.kind, pos, false);
            return MouseUse::Inside;
        }
        if !self.area.contains(at()) {
            if matches!(mouse.kind, MouseEventKind::Moved) {
                self.hover = None;
            }
            return MouseUse::Outside;
        }
        let hit = dialog_kit::hit_at(&self.hits, mouse.column, mouse.row);
        match mouse.kind {
            MouseEventKind::Moved => self.hover = hit,
            MouseEventKind::Down(MouseButton::Left) => match hit {
                Some(Part::Write) => self.set_mode(Mode::Write),
                Some(Part::Preview) => self.set_mode(Mode::Preview),
                Some(Part::Split) => self.set_mode(Mode::Split),
                Some(Part::Text) => {
                    self.edit.sync(Some(()));
                    let shift = mouse
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::SHIFT);
                    self.edit
                        .area_mouse((), &self.text, mouse.kind, pos, shift);
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
                    self.edit
                        .area_mouse((), &self.text, mouse.kind, pos, false);
                }
            }
            MouseEventKind::Drag(MouseButton::Left) | MouseEventKind::Up(MouseButton::Left) => {
                self.edit
                    .area_mouse((), &self.text, mouse.kind, pos, false);
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
    pub(crate) fn render(&mut self, buf: &mut Buffer, rect: Rect, focused: bool, placeholder: &str) {
        self.save_when_idle();
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
        let body = if tight {
            rect
        } else {
            self.paint_bar(buf, rect);
            Rect::new(rect.x, rect.y + 1, rect.width, rect.height - 1)
        };
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
        let room = self.ai_slot.x.saturating_sub(x + 1);
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
