//! Inline buttons (`Control::Buttons`): a row's actions drawn as buttons in
//! the value column, the state text before them. Left/Right move between a
//! row's buttons, Enter presses the focused one, a click presses the one
//! under it. The hit rects come out of the paint walk (`btn_cells` ->
//! `btn_rects`), like the chips'.
//!
//! A destructive action (forget the credentials, delete the archive, remove
//! a shortcut, an alias or the PATH entry, disable sparse checkout) asks
//! first through `dialog_kit::Confirm`, with Cancel focused.

use super::*;
use crate::dialog_kit::{self, ButtonState, Confirm, ConfirmEvent};
use crate::th;
use crossterm::event::{KeyEvent, MouseEvent};
use gitpoulpi_i18n::msg::settings_screen as screen;

/// Whether pressing `action` throws something away and so asks first.
pub(super) fn is_destructive(action: PrefAction) -> bool {
    use super::launcher_alias::ToolsButton as T;
    use super::launcher_page::LauncherButton as L;
    use super::repo_pages::RpButton;
    match action {
        PrefAction::ForgetCredentials => true,
        PrefAction::Launcher(L::Remove(_)) => true,
        PrefAction::Tools(T::AliasRemove | T::PathRemove) => true,
        PrefAction::RepoPage(RpButton::Disable) => true,
        #[cfg(feature = "ai")]
        PrefAction::Ai(ai_page::AiButton::DeleteLegacy) => true,
        _ => false,
    }
}

impl PreferencesState {
    /// A `Buttons` row: the label (unless the button is its own label), the
    /// state text, then the buttons, wrapped to the page width, then the
    /// help. Records each enabled button's cell for the hit rects.
    pub(super) fn button_lines(
        &self,
        field: &Field,
        buttons: &[Btn],
        index: usize,
        selected: bool,
        width: usize,
    ) -> Vec<Line<'static>> {
        let self_labelled = field.is_self_labelled();
        let label = crate::terminology::tr(&field.label).into_owned();
        let (label_col, stacked) = match self.label_col.get() {
            (0, _) => Self::layout_for(display_width(&label), width),
            set => set,
        };
        // A button that is its own label starts in the value column unless it
        // would not fit there, then it takes the left margin.
        let widest = buttons
            .iter()
            .map(|b| dialog_kit::button_width(&b.label) as usize)
            .max()
            .unwrap_or(0);
        let value_col = if stacked || (self_labelled && label_col + widest > width) {
            2
        } else {
            label_col
        };
        let room = width.saturating_sub(value_col).max(1);
        let label_style = if field.coming.is_some() {
            Style::default().fg(th().ui.muted)
        } else if selected {
            Style::default().fg(th().ui.text).bg(th().ui.selection)
        } else {
            Style::default().fg(th().ui.text)
        };

        let mut lines: Vec<Line<'static>> = Vec::new();
        // The first control line's prefix.
        let mut first: Vec<Span<'static>> = Vec::new();
        if self_labelled {
            first.push(Span::raw(" ".repeat(value_col)));
        } else if stacked {
            lines.push(Line::from(Span::styled(
                crate::ui_kit::fit(&label, width),
                label_style,
            )));
            first.push(Span::raw("  "));
        } else {
            let text = crate::ui_kit::fit(&label, label_col.saturating_sub(2));
            let pad = label_col.saturating_sub(display_width(&text));
            first.push(Span::styled(text, label_style));
            first.push(Span::raw(" ".repeat(pad)));
        }
        let mut row = first;
        let mut used = 0usize;
        if let Some(reason) = &field.coming {
            let text = crate::ui_kit::fit(reason, room);
            used += display_width(&text) + 2;
            row.push(Span::styled(
                text,
                Style::default()
                    .fg(th().ui.muted)
                    .add_modifier(Modifier::ITALIC),
            ));
            row.push(Span::raw("  "));
        }
        let focus = self.ai.chip_focus(buttons.len());
        let pointer = self.mouse_pos;
        // A filling button (a long text value) takes what the others leave.
        let others: usize = buttons
            .iter()
            .filter(|b| !b.fill)
            .map(|b| dialog_kit::button_width(&b.label) as usize + 2)
            .sum();
        let fill_w = room.saturating_sub(others).max(3);
        for (i, button) in buttons.iter().enumerate() {
            let mut text = crate::ui_kit::fit(&button.label, room.saturating_sub(2).max(1));
            let mut w = dialog_kit::button_width(&text) as usize;
            if button.fill {
                w = fill_w;
                text = crate::ui_kit::fit(&button.label, w - 2);
                let pad = (w - 2).saturating_sub(display_width(&text));
                text.push_str(&" ".repeat(pad));
            }
            if used > 0 && used + w > room {
                lines.push(Line::from(std::mem::take(&mut row)));
                row = vec![Span::raw(" ".repeat(value_col))];
                used = 0;
            }
            let live = button.enabled && field.coming.is_none();
            let state = if live {
                let hovered = pointer.is_some_and(|(c, r)| {
                    self.btn_rects
                        .iter()
                        .any(|(rect, f, b)| *f == index && *b == i && rect.contains((c, r).into()))
                });
                ButtonState::Normal {
                    hovered,
                    focused: selected && i == focus,
                }
            } else {
                ButtonState::Disabled
            };
            if live {
                self.btn_cells.borrow_mut().push((
                    index,
                    i,
                    lines.len(),
                    (value_col + used) as u16,
                    w as u16,
                ));
            }
            row.push(Span::styled(
                format!(" {text} "),
                dialog_kit::button_style(state),
            ));
            row.push(Span::raw("  "));
            used += w + 2;
        }
        if !buttons.is_empty() {
            row.pop(); // the last button's trailing gap
        }
        lines.push(Line::from(row));
        self.tail_lines(field, value_col, width, &mut lines);
        lines.push(Line::from(""));
        lines
    }

    /// Left/Right/Tab walk the buttons of a selected row with several.
    /// `true` when the key was used; Enter and Space fall through to
    /// `activate`.
    pub(super) fn buttons_key(&mut self, key: &KeyEvent) -> bool {
        let fields = self.fields();
        let Some(Control::Buttons(buttons)) = fields.get(self.field).map(|f| &f.control) else {
            return false;
        };
        let at = self.ai.chip_focus(buttons.len());
        let last = buttons.len().saturating_sub(1);
        match key.code {
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Tab if at < last => {
                self.ai.chip = at + 1;
                true
            }
            KeyCode::Left | KeyCode::Char('h') | KeyCode::BackTab if at > 0 => {
                self.ai.chip = at - 1;
                true
            }
            _ => false,
        }
    }

    /// Presses a button: a destructive one asks first.
    pub(super) fn press_button(&mut self, button: &Btn) {
        if !button.enabled {
            return;
        }
        if is_destructive(button.action) {
            let confirm = Confirm::new(
                button.label.clone(),
                screen::confirm_action(&button.label),
                vec![
                    (screen::CANCEL.get().to_string(), false),
                    (button.label.clone(), true),
                ],
                false,
            )
            .primary(true);
            self.confirm_action = Some((confirm, button.action));
        } else {
            self.run_action(button.action);
        }
    }

    pub(super) fn confirm_action_key(&mut self, key: KeyEvent) {
        let Some((confirm, _)) = self.confirm_action.as_mut() else {
            return;
        };
        let event = confirm.on_key(key);
        self.finish_confirm_action(event);
    }

    pub(super) fn confirm_action_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) {
        let Some((confirm, _)) = self.confirm_action.as_mut() else {
            return;
        };
        let event = confirm.on_mouse(MouseEvent {
            kind,
            column: col,
            row,
            modifiers: KeyModifiers::NONE,
        });
        self.finish_confirm_action(event);
    }

    fn finish_confirm_action(&mut self, event: ConfirmEvent<bool>) {
        let ConfirmEvent::Chose(yes) = event else {
            return;
        };
        if let Some((_, action)) = self.confirm_action.take() {
            if yes {
                self.run_action(action);
            }
        }
    }

    pub(super) fn render_confirm_action(&mut self, frame: &mut Frame, area: Rect) {
        if let Some((confirm, _)) = self.confirm_action.as_mut() {
            confirm.render(frame.buffer_mut(), area);
        }
    }
}
