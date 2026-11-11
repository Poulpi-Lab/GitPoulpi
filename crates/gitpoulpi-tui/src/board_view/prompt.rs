//! "Set Column Labels": a one-line prompt for the ordered, comma-separated
//! labels that are the columns of a host board (`status::todo, status::doing`).
//! A list typed in order, not a choice among a few values, so a text field.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use gitpoulpi_i18n::msg::board as msg;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use crate::dialog_kit::{self, ButtonState};
use crate::text_edit::FieldEditor;
use crate::th;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum PromptOutcome {
    Stay,
    Cancel,
    /// The labels, in board order (empty: back to Open / Closed only).
    Submit(Vec<String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Part {
    Text,
    Save,
    Cancel,
}

pub(super) struct LabelsPrompt {
    text: String,
    edit: FieldEditor<()>,
    focus: Part,
    hover: Option<Part>,
    hits: Vec<(Rect, Part)>,
    /// Labels of the project that look like statuses, proposed in the hint.
    pub(super) proposal: String,
}

impl LabelsPrompt {
    pub(super) fn new(current: &[String], proposal: Vec<String>) -> Self {
        let mut me = Self {
            text: current.join(", "),
            edit: FieldEditor::default(),
            focus: Part::Text,
            hover: None,
            hits: Vec::new(),
            proposal: proposal.join(", "),
        };
        me.edit.sync(Some(()));
        me
    }

    fn submit(&self) -> PromptOutcome {
        PromptOutcome::Submit(gitpoulpi_hosting::board::parse_label_list(&self.text))
    }

    pub(super) fn paste(&mut self, text: &str) {
        if self.focus != Part::Text {
            return;
        }
        let clean = text.replace(['\r', '\n'], " ");
        self.edit.sync(Some(()));
        let mut value = std::mem::take(&mut self.text);
        self.edit.caret_mut(()).insert(&mut value, &clean);
        self.text = value;
    }

    pub(super) fn on_key(&mut self, key: KeyEvent) -> PromptOutcome {
        if self.focus == Part::Text {
            self.edit.sync(Some(()));
            let mut value = std::mem::take(&mut self.text);
            let done = self.edit.dialog_key((), &mut value, &key);
            self.text = value;
            if done.is_some() {
                return PromptOutcome::Stay;
            }
        }
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        match key.code {
            KeyCode::Esc => return PromptOutcome::Cancel,
            KeyCode::Tab | KeyCode::BackTab => {
                let order = [Part::Text, Part::Save, Part::Cancel];
                let at = order.iter().position(|p| *p == self.focus).unwrap_or(0);
                self.focus = if shift || key.code == KeyCode::BackTab {
                    order[(at + order.len() - 1) % order.len()]
                } else {
                    order[(at + 1) % order.len()]
                };
            }
            KeyCode::Enter | KeyCode::Char(' ') if self.focus == Part::Cancel => {
                return PromptOutcome::Cancel
            }
            KeyCode::Enter => return self.submit(),
            KeyCode::Char(' ') if self.focus == Part::Save => return self.submit(),
            _ => {}
        }
        PromptOutcome::Stay
    }

    pub(super) fn on_mouse(&mut self, mouse: MouseEvent) -> PromptOutcome {
        if !matches!(mouse.kind, MouseEventKind::Down(_)) && self.edit.is_dragging() {
            self.edit
                .dialog_mouse(Some(()), &self.text, mouse.kind, mouse.column, mouse.row);
            return PromptOutcome::Stay;
        }
        let hit = dialog_kit::hit_at(&self.hits, mouse.column, mouse.row);
        match mouse.kind {
            MouseEventKind::Moved => self.hover = hit,
            MouseEventKind::Down(MouseButton::Left) => match hit {
                Some(Part::Text) => {
                    self.focus = Part::Text;
                    self.edit.sync(Some(()));
                    self.edit.dialog_mouse(
                        Some(()),
                        &self.text,
                        mouse.kind,
                        mouse.column,
                        mouse.row,
                    );
                }
                Some(Part::Save) => return self.submit(),
                Some(Part::Cancel) => return PromptOutcome::Cancel,
                None => {}
            },
            MouseEventKind::Up(MouseButton::Left) => {
                self.edit
                    .dialog_mouse(Some(()), &self.text, mouse.kind, mouse.column, mouse.row);
            }
            _ => {}
        }
        PromptOutcome::Stay
    }

    pub(super) fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let rect = WIN.place(area, dialog_kit::centered(area, 76, 10));
        self.edit.begin_frame();
        dialog_kit::panel(buf, rect, msg::COLUMNS_TITLE.get());
        WIN.decorate(buf, rect, None, true);
        let inner = Rect::new(
            rect.x + 2,
            rect.y + 1,
            rect.width.saturating_sub(4),
            rect.height.saturating_sub(2),
        );
        let mut hits: Vec<(Rect, Part)> = Vec::new();
        if inner.width < 20 || inner.height < 6 {
            self.hits = hits;
            return;
        }
        let muted = Style::default().fg(th().ui.muted).bg(th().ui.panel);
        dialog_kit::put(
            buf,
            inner.x,
            inner.y,
            msg::COLUMNS_ABOUT.get(),
            inner.width,
            muted,
        );
        if !self.proposal.is_empty() {
            dialog_kit::put(
                buf,
                inner.x,
                inner.y + 1,
                &msg::columns_proposal(&self.proposal),
                inner.width,
                muted,
            );
        }
        let field = Rect::new(inner.x, inner.y + 3, inner.width, 1);
        dialog_kit::text_input(
            buf,
            field,
            &self.text,
            msg::COLUMNS_PLACEHOLDER.get(),
            self.focus == Part::Text,
            self.hover == Some(Part::Text),
            (&self.edit, ()),
        );
        hits.push((field, Part::Text));
        let button_y = inner.bottom().saturating_sub(1);
        let save_label = msg::COLUMNS_SAVE.get();
        let cancel_label = msg::CANCEL.get();
        let save_w = dialog_kit::button_width(save_label);
        let cancel_w = dialog_kit::button_width(cancel_label);
        let save_x = inner.right().saturating_sub(save_w);
        let cancel_x = save_x.saturating_sub(cancel_w + 1);
        let cancel = dialog_kit::button(
            buf,
            cancel_x,
            button_y,
            cancel_label,
            ButtonState::Normal {
                hovered: self.hover == Some(Part::Cancel),
                focused: self.focus == Part::Cancel,
            },
        );
        let save = dialog_kit::button(
            buf,
            save_x,
            button_y,
            save_label,
            ButtonState::Primary {
                hovered: self.hover == Some(Part::Save),
                focused: self.focus == Part::Save,
            },
        );
        hits.insert(0, (cancel, Part::Cancel));
        hits.insert(0, (save, Part::Save));
        dialog_kit::hints(
            buf,
            inner.x,
            button_y,
            cancel_x.saturating_sub(inner.x + 1),
            &[
                (
                    crate::dialog_kit::key_name(crate::keymap::chord::Key::Enter).as_str(),
                    msg::HINT_SAVE.get(),
                ),
                (msg::KEY_ESC.get(), msg::HINT_CANCEL.get()),
            ],
        );
        self.hits = hits;
    }
}

/// The dialog moves (its size follows its content).
const WIN: crate::dialog_kit::Win = crate::dialog_kit::Win::movable("board_columns");
