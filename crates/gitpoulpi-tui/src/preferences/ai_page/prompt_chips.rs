//! The "Quick insert" row of the instructions editor: the section's chips
//! above the text, a click or the keyboard inserting a sentence at the
//! caret.

use super::*;
use crate::dialog_kit;
use ratatui::buffer::Buffer;

/// The text keeps at least this many rows; the chip row gives way first.
const MIN_TEXT_ROWS: u16 = 3;

impl PromptEdit {
    /// Inserts chip `chip`'s sentence at the caret, with a space on either
    /// side where it would touch other text. A sentence that is already in
    /// the instructions is not inserted again.
    pub(super) fn insert_chip(&mut self, chip: usize) {
        let Some((_, sentence)) = self.section.chips().get(chip).copied() else {
            return;
        };
        // The sentence is sent to the model: always the English text.
        let sentence = sentence.english();
        self.chips_focused = false;
        if self.text.contains(sentence) {
            self.note = Some(msg::ALREADY_INSERTED.get().to_string());
            return;
        }
        let at = self.cursor();
        let lead = !self.text[..at].is_empty() && !self.text[..at].ends_with(char::is_whitespace);
        let trail =
            !self.text[at..].is_empty() && !self.text[at..].starts_with(char::is_whitespace);
        let piece = format!(
            "{}{sentence}{}",
            if lead { " " } else { "" },
            if trail { " " } else { "" }
        );
        self.field.caret_mut(()).insert(&mut self.text, &piece);
        if trail {
            // The caret lands after the sentence, before the added space.
            let back = self.cursor() - 1;
            self.field.caret_mut(()).set_cursor(&self.text, back, false);
        }
        self.after_edit();
    }

    /// A key while the chip row has the keyboard: Left/Right (and Home/End)
    /// choose, Enter or Space insert and return to the text, Tab, Down or
    /// Esc return without inserting. `true` when it was the row's.
    pub(super) fn chips_key(&mut self, key: &KeyEvent) -> bool {
        if !self.chips_focused {
            return false;
        }
        let last = self.section.chips().len().saturating_sub(1);
        let at = self.chip.min(last);
        match key.code {
            KeyCode::Right | KeyCode::Char('l') => self.chip = (at + 1).min(last),
            KeyCode::Left | KeyCode::Char('h') => self.chip = at.saturating_sub(1),
            KeyCode::Home => self.chip = 0,
            KeyCode::End => self.chip = last,
            KeyCode::Enter | KeyCode::Char(' ') => self.insert_chip(at),
            KeyCode::Tab | KeyCode::BackTab | KeyCode::Down | KeyCode::Esc => {
                self.chips_focused = false
            }
            // Ctrl+S and the like keep working.
            _ if key.modifiers.contains(KeyModifiers::CONTROL) => return false,
            _ => {
                self.chips_focused = false;
                return false;
            }
        }
        true
    }

    /// Paints the row at the top of `area`: how many rows it took, and the
    /// chips' hit rects. Wraps to the width; nothing when the editor is too
    /// short to leave the text a few rows.
    pub(super) fn draw_chips(
        &self,
        buf: &mut Buffer,
        area: Rect,
        hovered: impl Fn(Rect) -> bool,
    ) -> (u16, Vec<(Rect, usize)>) {
        let chips = self.section.chips();
        let room = area.height.saturating_sub(MIN_TEXT_ROWS);
        if chips.is_empty() || room == 0 || area.width < 12 {
            return (0, Vec::new());
        }
        let label = format!("{}: ", msg::QUICK_INSERT.get());
        let indent = display_width(&label) as u16;
        let mut hits = Vec::new();
        let (mut x, mut y) = (area.x + indent, area.y);
        dialog_kit::put(
            buf,
            area.x,
            y,
            &label,
            area.width,
            Style::default().fg(th().ui.muted),
        );
        for (i, (text, sentence)) in chips.iter().enumerate() {
            let shown = format!("[+ {}]", text.get());
            let w = display_width(&shown) as u16;
            if x > area.x + indent && x + w > area.right() {
                if y + 1 >= area.y + room {
                    break;
                }
                y += 1;
                x = area.x + indent;
            }
            let rect = Rect::new(x, y, w.min(area.right().saturating_sub(x)), 1);
            let style = if self.chips_focused && i == self.chip.min(chips.len() - 1) {
                Style::default().fg(th().ui.text).bg(th().ui.selection)
            } else if self.text.contains(sentence.english()) {
                Style::default().fg(th().ui.muted)
            } else if hovered(rect) {
                crate::theme::plain_button_style(true)
            } else {
                Style::default().fg(th().ui.accent)
            };
            dialog_kit::put(buf, x, y, &shown, rect.width, style);
            hits.push((rect, i));
            x += w + 1;
        }
        (y - area.y + 1, hits)
    }
}
