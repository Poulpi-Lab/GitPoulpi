//! Drawing the prompt dialog, and handing back its hit rects from the same walk.

use super::*;
use crate::dialog_kit::{button, button_width, put, text_area, text_input, ButtonState, Win};
use gitpoulpi_i18n::msg::prompt_dialog as msg;
use ratatui::style::Style;

/// A one-line prompt moves (its size follows its single row).
const WIN_LINE: Win = Win::movable("prompt");
/// A message area moves and resizes (the field grows with the height).
const WIN_AREA: Win = Win::resizable("tag_message", 50, 10);

impl PromptDialog {
    fn win(&self) -> &'static Win {
        if matches!(self.kind, PromptKind::BranchOnRemote { .. }) {
            &super::remotes::WIN_DEST
        } else if self.multiline {
            &WIN_AREA
        } else {
            &WIN_LINE
        }
    }

    /// Rows the dialog wants: borders, the field, the status row, the buttons.
    fn wanted_height(&self) -> u16 {
        if self.multiline {
            14
        } else {
            7
        }
    }

    /// Draws the dialog (where the user put it, else centered in `area`) and
    /// returns its click targets, from the same walk that painted them.
    pub(crate) fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        hovered: Option<PromptPart>,
    ) -> Vec<(Rect, PromptPart)> {
        if self.is_remotes() {
            return self.render_remotes(frame, area, hovered);
        }
        let width = area
            .width
            .saturating_sub(4)
            .min(if self.multiline { 72 } else { 60 });
        let height = self.wanted_height().min(area.height);
        let win = self.win();
        let popup = win.place(
            area,
            Rect::new(
                area.x + area.width.saturating_sub(width) / 2,
                area.y + area.height.saturating_sub(height) / 3,
                width,
                height,
            ),
        );
        let buf = frame.buffer_mut();
        let mut guard = self.guard.borrow_mut();
        let suffix = self
            .proposed_by
            .as_ref()
            .map(|client| format!(" {} {}", crate::theme::AI_SPARK, proposed_by_text(client)));
        let title = match &suffix {
            Some(suffix) => format!("{}{suffix}", self.title),
            None => self.title.clone(),
        };
        guard.framed(buf, popup, &title, win);
        if let Some(suffix) = &suffix {
            paint_proposed(buf, popup, suffix);
        }
        let mut hits = Vec::new();
        if let Some(cross) = guard.close.rect() {
            hits.push((cross, PromptPart::Close));
        }
        let inner = Rect::new(
            popup.x + 2,
            popup.y + 1,
            popup.width.saturating_sub(4),
            popup.height.saturating_sub(2),
        );
        if inner.width < 16 || inner.height < 4 {
            guard.render(buf);
            return hits;
        }
        self.editor.begin_frame();

        let focused = self.focus == PromptPart::Field;
        let top = inner.y + u16::from(inner.height >= 5);
        let by = inner.bottom() - 1;
        let status_y = inner.bottom() - 2;
        let field_h = if self.multiline {
            status_y.saturating_sub(top).max(1)
        } else {
            1
        };
        let field = Rect::new(inner.x, top, inner.width, field_h);
        self.bar.set(None);
        if self.ai_running {
            // The model is writing: the field says so and is locked.
            let bg = crate::input_bg(false, false);
            dialog_kit::fill(buf, field, Style::default().bg(bg));
            put(
                buf,
                field.x + 1,
                field.y,
                &format!("{} {}", crate::theme::AI_SPARK, msg::AI_WRITING.get()),
                field.width.saturating_sub(1),
                Style::default().fg(th().status.ai).bg(bg),
            );
        } else if let Some(combo) = &self.combo {
            combo.borrow_mut().render_field(
                buf,
                field,
                focused,
                hovered == Some(PromptPart::Field),
            );
        } else if self.multiline {
            let bar = text_area(
                buf,
                field,
                &self.text,
                &self.placeholder,
                focused,
                false,
                (&self.editor, PromptPart::Field),
            );
            self.bar.set(bar);
            self.hint_when_empty(buf, field);
        } else {
            text_input(
                buf,
                field,
                &self.text,
                &self.placeholder,
                focused,
                false,
                (&self.editor, PromptPart::Field),
            );
            self.hint_when_empty(buf, field);
        }
        hits.push((field, PromptPart::Field));

        // The status row: the reason a press was refused, else the keys of an area.
        let (status, color) = match &self.error {
            Some(error) => (error.clone(), th().status.danger),
            None if self.multiline => (
                msg::keys_hint(crate::keymap::current().shown(Bind::RewordApply)),
                th().ui.muted,
            ),
            None => (String::new(), th().ui.muted),
        };
        put(
            buf,
            inner.x,
            status_y,
            &truncate_with_ellipsis(&status, inner.width as usize),
            inner.width,
            Style::default().fg(color).bg(th().ui.panel),
        );

        // Buttons: Cancel and the primary one right-aligned, the AI one at the left.
        let cancel_label = gitpoulpi_i18n::msg::dialog::CANCEL.get();
        let total = button_width(cancel_label) + 1 + button_width(&self.submit_label);
        let mut x = inner.x + inner.width.saturating_sub(total);
        let cancel = button(
            buf,
            x,
            by,
            cancel_label,
            ButtonState::Normal {
                hovered: hovered == Some(PromptPart::Cancel),
                focused: self.focus == PromptPart::Cancel,
            },
        );
        hits.push((cancel, PromptPart::Cancel));
        x += cancel.width + 1;
        // A list prompt's primary button is flat until something is chosen.
        let submit_state = if self.combo.is_some() && self.value().is_none() {
            ButtonState::Disabled
        } else {
            ButtonState::Primary {
                hovered: hovered == Some(PromptPart::Submit),
                focused: self.focus == PromptPart::Submit,
            }
        };
        let submit = button(buf, x, by, &self.submit_label, submit_state);
        hits.push((submit, PromptPart::Submit));
        if self.ai {
            let label = format!(
                "{} {} {}",
                crate::theme::AI_SPARK,
                msg::AI_MESSAGE_BUTTON.get(),
                crate::keymap::current().shown(Bind::PromptAiStashMessage)
            );
            if inner.width >= total + button_width(&label) + 2 {
                let rect = button(
                    buf,
                    inner.x,
                    by,
                    &label,
                    ButtonState::Normal {
                        hovered: hovered == Some(PromptPart::Ai),
                        focused: self.focus == PromptPart::Ai,
                    },
                );
                // The sparkle keeps the AI purple, as on every AI control.
                for col in rect.x..rect.right() {
                    if let Some(cell) = buf.cell_mut((col, rect.y)) {
                        if cell.symbol() == crate::theme::AI_SPARK {
                            cell.set_fg(th().status.ai);
                        }
                    }
                }
                hits.push((rect, PromptPart::Ai));
            }
        }
        // The open list over everything else, then the discard question.
        if let Some(combo) = &self.combo {
            let area = buf.area;
            combo.borrow_mut().render_popup(buf, area);
        }
        guard.render(buf);
        hits
    }

    /// An empty, focused field would show nothing but its caret: the
    /// placeholder stays visible, dim, after it.
    fn hint_when_empty(&self, buf: &mut Buffer, field: Rect) {
        if self.text.is_empty() && self.focus == PromptPart::Field {
            put(
                buf,
                field.x + 2,
                field.y,
                &self.placeholder,
                field.width.saturating_sub(2),
                Style::default()
                    .fg(th().ui.muted)
                    .bg(crate::input_bg(true, false)),
            );
        }
    }
}

fn proposed_by_text(client: &str) -> String {
    gitpoulpi_i18n::msg::ui_intent::proposed_by(client)
}

/// The "✦ Proposed by ..." tail of the title keeps the AI purple.
fn paint_proposed(buf: &mut Buffer, popup: Rect, suffix: &str) {
    let spark = crate::theme::AI_SPARK;
    let Some(start) = (popup.x..popup.right()).find(|col| {
        buf.cell((*col, popup.y))
            .is_some_and(|c| c.symbol() == spark)
    }) else {
        return;
    };
    let width = display_width(suffix) as u16;
    let end = (start + width).min(popup.right().saturating_sub(1));
    for col in start..end {
        if let Some(cell) = buf.cell_mut((col, popup.y)) {
            cell.set_fg(th().status.ai);
        }
    }
}
