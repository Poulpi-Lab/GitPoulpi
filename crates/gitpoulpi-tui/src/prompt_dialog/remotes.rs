//! The "Remotes of <branch>" shape of the prompt dialog: its keys, mouse and
//! paint around the table of `branch_remotes_dialog` (the rows, the Apply
//! request) and the nested "Branch name on <remote>" prompt.

use super::*;
use crate::branch_remotes_dialog::RowsKey;
use crate::dialog_kit::{button, button_width, put, ButtonState, Win};
use ratatui::style::Style;

/// The dialog moves; its height follows the number of remotes.
const WIN: Win = Win::movable("branch_remotes");
/// The small nested prompt moves on its own.
pub(super) const WIN_DEST: Win = Win::movable("branch_remote_dest");

impl PromptDialog {
    /// The nested prompt: the branch's name on `remote`, prefilled.
    pub(crate) fn branch_on_remote(remote: &str, current: &str) -> Self {
        let mut dialog = Self::base(
            PromptKind::BranchOnRemote {
                remote: remote.to_string(),
            },
            msg_br::rename_title(remote),
            msg_br::RENAME_PLACEHOLDER.get(),
            msg_br::RENAME_SET.get().to_string(),
            false,
            current,
        );
        dialog.nested = true;
        dialog
    }

    /// Takes the table's answer once the daemon has given it (every tick).
    pub(crate) fn poll(&mut self) {
        let Some(pane) = &mut self.remotes else {
            return;
        };
        if pane.poll() && !pane.has_rows() && self.focus == PromptPart::Rows {
            self.set_focus(PromptPart::Field);
        }
    }

    /// Opens the nested prompt on the first renamable row (layout tests).
    #[cfg(test)]
    pub(crate) fn open_rename_for_test(&mut self) {
        if let Some(pane) = &mut self.remotes {
            pane.select_row(1);
        }
        self.rename_selected();
    }

    pub(super) fn is_remotes(&self) -> bool {
        self.remotes.is_some()
    }

    /// `Rename on remote...`: opens the nested prompt for the selected row.
    pub(super) fn rename_selected(&mut self) -> PromptOutcome {
        let Some(pane) = &mut self.remotes else {
            return PromptOutcome::Stay;
        };
        if let Some(why) = pane.rename_error() {
            self.error = Some(why);
            return PromptOutcome::Stay;
        }
        if let Some((row, remote, dest)) = pane.rename_target() {
            let dialog = PromptDialog::branch_on_remote(&remote, &dest);
            pane.rename = Some((Box::new(dialog), row));
            self.error = None;
        }
        PromptOutcome::Stay
    }

    /// What the nested prompt answered.
    fn settle_nested(&mut self, outcome: PromptOutcome) -> PromptOutcome {
        let Some(pane) = &mut self.remotes else {
            return PromptOutcome::Stay;
        };
        match outcome {
            PromptOutcome::Entered(dest) => {
                if let Some((_, row)) = pane.rename.take() {
                    pane.set_dest(row, &dest);
                }
            }
            PromptOutcome::Close => pane.rename = None,
            _ => {}
        }
        PromptOutcome::Stay
    }

    /// A key while the nested prompt is up: it owns the keyboard.
    pub(super) fn nested_key(&mut self, key: KeyEvent) -> Option<PromptOutcome> {
        let pane = self.remotes.as_mut()?;
        let (dialog, _) = pane.rename.as_mut()?;
        let outcome = dialog.on_key(key);
        Some(self.settle_nested(outcome))
    }

    /// A mouse event while the nested prompt is up.
    pub(super) fn nested_mouse(
        &mut self,
        mouse: MouseEvent,
        hit: Option<PromptPart>,
    ) -> Option<PromptOutcome> {
        let pane = self.remotes.as_mut()?;
        let (dialog, _) = pane.rename.as_mut()?;
        let outcome = dialog.on_mouse(mouse, hit);
        Some(self.settle_nested(outcome))
    }

    /// The table's keys (the keyboard is in it).
    pub(super) fn rows_key(&mut self, key: &KeyEvent) -> Option<PromptOutcome> {
        let pane = self.remotes.as_mut()?;
        match pane.on_key(key) {
            RowsKey::Ignored => None,
            RowsKey::Handled => {
                self.error = None;
                Some(PromptOutcome::Stay)
            }
            RowsKey::LeaveUp => {
                self.set_focus(PromptPart::Field);
                Some(PromptOutcome::Stay)
            }
            RowsKey::Rename => Some(self.rename_selected()),
        }
    }

    /// A press on one of the table's own controls; `None` for any other.
    pub(super) fn rows_press(&mut self, part: PromptPart) -> Option<PromptOutcome> {
        let pane = self.remotes.as_mut()?;
        match part {
            PromptPart::Row(i) => pane.select_row(usize::from(i)),
            PromptPart::Push(i) => pane.toggle_push(usize::from(i)),
            PromptPart::Pull(i) => pane.toggle_pull(usize::from(i)),
            PromptPart::Remove(i) => pane.remove(usize::from(i)),
            _ => return None,
        }
        self.error = None;
        self.set_focus(PromptPart::Rows);
        Some(PromptOutcome::Stay)
    }

    /// Wheel and scrollbar of the table; `true` when the table took it.
    pub(super) fn rows_bar(&mut self, mouse: MouseEvent) -> bool {
        self.remotes.as_mut().is_some_and(|p| p.bar_mouse(mouse))
    }

    pub(super) fn rows_wheel(&mut self, delta: isize) -> bool {
        match &mut self.remotes {
            Some(pane) => {
                pane.wheel(delta);
                true
            }
            None => false,
        }
    }

    /// Draws the dialog (where the user put it, else centered in `area`) and
    /// returns its click targets, from the same walk that painted them.
    pub(super) fn render_remotes(
        &self,
        frame: &mut Frame,
        area: Rect,
        hovered: Option<PromptPart>,
    ) -> Vec<(Rect, PromptPart)> {
        let Some(pane) = &self.remotes else {
            return Vec::new();
        };
        let width = area.width.saturating_sub(4).min(84);
        let want = (pane.wanted_rows() as u16).saturating_add(8);
        let height = want.min(area.height);
        let popup = WIN.place(
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
        guard.framed(buf, popup, &self.title, &WIN);
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
        if inner.width < 30 || inner.height < 6 {
            guard.render(buf);
            return hits;
        }
        let panel = th().ui.panel;

        // The upstream list on the first row, behind its label.
        let label = msg_br::UPSTREAM_LABEL.get();
        let label_w = display_width(label) as u16;
        put(
            buf,
            inner.x,
            inner.y,
            label,
            label_w,
            Style::default().fg(th().ui.muted).bg(panel),
        );
        let field = Rect::new(
            inner.x + label_w + 1,
            inner.y,
            inner.width.saturating_sub(label_w + 1),
            1,
        );
        if let Some(combo) = &self.combo {
            combo.borrow_mut().render_field(
                buf,
                field,
                self.focus == PromptPart::Field,
                hovered == Some(PromptPart::Field),
            );
        }
        hits.push((field, PromptPart::Field));

        let by = inner.bottom() - 1;
        let status_y = by - 1;
        let warn_y = by - 2;
        let table_y = inner.y + 2;
        let table = Rect::new(
            inner.x,
            table_y,
            inner.width,
            warn_y.saturating_sub(table_y).max(2),
        );
        // Inner-first: the table's checkboxes and buttons before the field's
        // own rect is irrelevant (they do not overlap), the buttons come next.
        hits.extend(pane.paint(buf, table, self.focus == PromptPart::Rows, hovered));

        if let Some(warning) = pane.same_url_warning() {
            put(
                buf,
                inner.x,
                warn_y,
                &truncate_with_ellipsis(&warning, inner.width as usize),
                inner.width,
                Style::default().fg(th().status.warning).bg(panel),
            );
        }
        let (status, color) = match &self.error {
            Some(error) => (error.clone(), th().status.danger),
            None => (msg_br::KEYS_HINT.get().to_string(), th().ui.muted),
        };
        put(
            buf,
            inner.x,
            status_y,
            &truncate_with_ellipsis(&status, inner.width as usize),
            inner.width,
            Style::default().fg(color).bg(panel),
        );

        // Buttons: Rename and Unset at the left, Cancel and Apply at the right.
        let cancel_label = gitpoulpi_i18n::msg::dialog::CANCEL.get();
        let apply_label = msg_br::APPLY_BUTTON.get();
        let total = button_width(cancel_label) + 1 + button_width(apply_label);
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
        let apply_state = if pane.is_ready() {
            ButtonState::Primary {
                hovered: hovered == Some(PromptPart::Submit),
                focused: self.focus == PromptPart::Submit,
            }
        } else {
            ButtonState::Disabled
        };
        hits.push((button(buf, x, by, apply_label, apply_state), PromptPart::Submit));
        let mut x = inner.x;
        for part in self.extra_parts() {
            let label = match part {
                PromptPart::Extra(0) => msg_br::RENAME_BUTTON.get(),
                _ => msg_br::UNSET_BUTTON.get(),
            };
            let w = button_width(label);
            // Never over Cancel: a button that does not fit is not drawn.
            if x + w + 1 > inner.x + inner.width.saturating_sub(total) {
                continue;
            }
            let rect = button(
                buf,
                x,
                by,
                label,
                ButtonState::Normal {
                    hovered: hovered == Some(part),
                    focused: self.focus == part,
                },
            );
            hits.push((rect, part));
            x += rect.width + 1;
        }
        // The open list over everything else, then the nested prompt, then
        // the discard question.
        if let Some(combo) = &self.combo {
            let bounds = buf.area;
            combo.borrow_mut().render_popup(buf, bounds);
        }
        guard.render(buf);
        if let Some((dialog, _)) = &pane.rename {
            drop(guard);
            // While it is up only its own controls answer.
            return dialog.render(frame, area, hovered);
        }
        hits
    }
}
