//! What sits over the merge editor: the popup menu, the unsaved-changes
//! and confirm boxes, and the strip that replaces the key hints when the
//! file moved under the editor. Every hit rect comes out of this paint
//! (maintainer rule 6) and goes in front of the editor's own (rule 5).

use crate::th;
use crossterm::event::KeyCode;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use gitpoulpi_i18n::msg::merge as tr;

use crate::dialog_kit::{self, ButtonState};
use crate::keymap::{self, Action, KeyChord};
use crate::{display_width, fit};

use super::input::{Prompt, Strip};
use super::view::MergeView;
use super::MergeHit;

/// The effective key of `action`, upper-cased, `-` when unbound.
fn key_of(action: Action) -> String {
    match keymap::current().shown(action) {
        k if k.is_empty() => "-".to_string(),
        k => k.to_uppercase(),
    }
}

fn esc_label() -> String {
    KeyChord::from_code(KeyCode::Esc)
        .map(|c| c.short())
        .unwrap_or_default()
}

impl MergeView {
    /// Paints the strip, the menu and the boxes over `area` and returns
    /// their hit rects, the topmost first.
    pub(super) fn paint_overlay(
        &self,
        buf: &mut Buffer,
        area: Rect,
        hovered: Option<MergeHit>,
    ) -> Vec<(Rect, MergeHit)> {
        let mut hits: Vec<(Rect, MergeHit)> = Vec::new();
        if let Some(prompt) = &self.ui.prompt {
            self.paint_prompt(buf, area, prompt, hovered, &mut hits);
        }
        if let Some(menu) = &self.ui.menu {
            self.paint_menu(buf, area, menu, hovered, &mut hits);
        }
        if let Some(strip) = &self.ui.strip {
            let row = Rect::new(area.x, area.bottom().saturating_sub(1), area.width, 1);
            self.paint_strip(buf, row, strip, hovered, &mut hits);
        }
        hits
    }

    // -- the strip -------------------------------------------------------------

    fn paint_strip(
        &self,
        buf: &mut Buffer,
        row: Rect,
        strip: &Strip,
        hovered: Option<MergeHit>,
        hits: &mut Vec<(Rect, MergeHit)>,
    ) {
        dialog_kit::fill(buf, row, Style::default().bg(th().merge.strip_bg));
        let (text, primary, secondary) = match strip {
            Strip::EditedOutside { disk } => (
                tr::STRIP_EDITED_OUTSIDE.get(),
                // With no text read from the disk there is nothing to
                // continue from: the strip only offers the merge.
                disk.is_some().then(|| tr::STRIP_CONTINUE.get()),
                tr::STRIP_START_OVER.get(),
            ),
            Strip::Resolved => (
                tr::STRIP_RESOLVED.get(),
                Some(tr::STRIP_CLOSE.get()),
                tr::STRIP_UNRESOLVE.get(),
            ),
            Strip::Changed => (
                tr::STRIP_CHANGED.get(),
                Some(tr::STRIP_CLOSE.get()),
                tr::STRIP_RELOAD.get(),
            ),
        };
        let state = |hit: MergeHit| ButtonState::Normal {
            hovered: hovered == Some(hit),
            focused: false,
        };
        let sw = dialog_kit::button_width(secondary);
        let mut x = row.right().saturating_sub(sw + 1);
        let secondary_rect = dialog_kit::button(
            buf,
            x,
            row.y,
            secondary,
            ButtonState::Primary {
                hovered: hovered == Some(MergeHit::StripSecondary),
                focused: false,
            },
        );
        hits.push((secondary_rect, MergeHit::StripSecondary));
        let mut left_end = x;
        if let Some(label) = primary {
            let pw = dialog_kit::button_width(label);
            x = x.saturating_sub(pw + 1);
            let rect = dialog_kit::button(buf, x, row.y, label, state(MergeHit::StripPrimary));
            hits.push((rect, MergeHit::StripPrimary));
            left_end = x;
        }
        dialog_kit::put(
            buf,
            row.x + 1,
            row.y,
            text,
            left_end.saturating_sub(row.x + 2),
            Style::default()
                .fg(th().merge.strip_fg)
                .bg(th().merge.strip_bg),
        );
    }

    // -- the boxes ---------------------------------------------------------------

    fn paint_prompt(
        &self,
        buf: &mut Buffer,
        area: Rect,
        prompt: &Prompt,
        hovered: Option<MergeHit>,
        hits: &mut Vec<(Rect, MergeHit)>,
    ) {
        let path = self.path().to_string_lossy().into_owned();
        let (title, lines, primary, secondary, primary_key) = match prompt {
            Prompt::ResetAll => (
                tr::RESET_CONFIRM_TITLE.get().to_string(),
                vec![tr::RESET_CONFIRM_TEXT.get().to_string()],
                format!(
                    "{} ({})",
                    tr::RESET_CONFIRM_BUTTON.get(),
                    key_of(Action::EditorPromptSave)
                ),
                None,
                true,
            ),
            Prompt::Unsaved { .. } => (
                tr::PROMPT_TITLE.get().to_string(),
                vec![tr::prompt_question(&path)],
                format!(
                    "{} ({})",
                    tr::PROMPT_SAVE.get(),
                    key_of(Action::EditorPromptSave)
                ),
                Some(format!(
                    "{} ({})",
                    tr::PROMPT_DISCARD.get(),
                    key_of(Action::EditorPromptDiscard)
                )),
                true,
            ),
            Prompt::Confirm { checks, .. } => {
                let mut lines = Vec::new();
                if !checks.unpicked.is_empty() {
                    lines.push(tr::confirm_unpicked(checks.unpicked.len() as u64));
                }
                if let Some(&n) = checks.marker_lines.first() {
                    lines.push(tr::confirm_markers(n));
                }
                let go_to = checks.unpicked.first().map_or(1, |c| c + 1);
                (
                    tr::CONFIRM_TITLE.get().to_string(),
                    lines,
                    format!(
                        "{} ({})",
                        tr::CONFIRM_SAVE_ANYWAY.get(),
                        key_of(Action::EditorPromptSave)
                    ),
                    Some(tr::confirm_go_to(go_to)),
                    true,
                )
            }
        };
        let _ = primary_key;
        let cancel = format!("{} ({})", tr::PROMPT_CANCEL.get(), esc_label());
        let buttons_w = [Some(&primary), secondary.as_ref(), Some(&cancel)]
            .iter()
            .flatten()
            .map(|l| dialog_kit::button_width(l) + 2)
            .sum::<u16>();
        let text_w = lines.iter().map(|l| display_width(l)).max().unwrap_or(0) as u16;
        let w = (text_w.max(buttons_w) + 4).min(area.width);
        let h = (lines.len() as u16 + 4).min(area.height);
        if w < 12 || h < 5 {
            return;
        }
        let rect = WIN.place(area, dialog_kit::centered(area, w, h));
        dialog_kit::panel(buf, rect, &title);
        WIN.decorate(buf, rect, None, true);
        let inner_w = rect.width.saturating_sub(4);
        for (i, line) in lines.iter().enumerate() {
            dialog_kit::put(
                buf,
                rect.x + 2,
                rect.y + 1 + i as u16,
                &fit(line, inner_w as usize),
                inner_w,
                Style::default().fg(th().ui.text).bg(th().ui.panel),
            );
        }
        let y = rect.bottom().saturating_sub(2);
        let mut x = rect.x + 2;
        for (label, hit, primary) in [
            (Some(&primary), MergeHit::PromptSave, true),
            (secondary.as_ref(), MergeHit::PromptDiscard, false),
            (Some(&cancel), MergeHit::PromptCancel, false),
        ] {
            let Some(label) = label else { continue };
            let lit = hovered == Some(hit);
            let state = if primary {
                ButtonState::Primary {
                    hovered: lit,
                    focused: false,
                }
            } else {
                ButtonState::Normal {
                    hovered: lit,
                    focused: false,
                }
            };
            let r = dialog_kit::button(buf, x, y, label, state);
            hits.push((r, hit));
            x += r.width + 2;
        }
    }

    fn paint_menu(
        &self,
        buf: &mut Buffer,
        area: Rect,
        menu: &super::input::Menu,
        hovered: Option<MergeHit>,
        hits: &mut Vec<(Rect, MergeHit)>,
    ) {
        let label_w = menu
            .rows
            .iter()
            .map(|r| display_width(&r.label))
            .max()
            .unwrap_or(0) as u16;
        let w = (label_w + 4).min(area.width);
        let h = (menu.rows.len() as u16 + 2).min(area.height);
        if w < 6 || h < 3 {
            return;
        }
        let x = menu.at.0.min(area.right().saturating_sub(w)).max(area.x);
        let y = menu.at.1.min(area.bottom().saturating_sub(h)).max(area.y);
        let rect = Rect::new(x, y, w, h);
        dialog_kit::panel(buf, rect, "");
        for (i, row) in menu.rows.iter().enumerate() {
            let ry = rect.y + 1 + i as u16;
            if ry >= rect.bottom() - 1 {
                break;
            }
            let hit = MergeHit::MenuRow(i);
            let bg = if hovered == Some(hit) {
                th().ui.hover
            } else if i == menu.selected {
                th().ui.selection
            } else {
                th().ui.panel
            };
            let text = format!(" {}", row.label);
            dialog_kit::fill(
                buf,
                Rect::new(rect.x + 1, ry, rect.width - 2, 1),
                Style::default().bg(bg),
            );
            dialog_kit::put(
                buf,
                rect.x + 1,
                ry,
                &text,
                rect.width - 2,
                Style::default()
                    .fg(if row.cmd == super::MergeCmd::AiSuggest {
                        th().status.ai
                    } else if i == menu.selected {
                        th().ui.accent
                    } else {
                        th().ui.text
                    })
                    .bg(bg)
                    .add_modifier(if i == menu.selected {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
            );
            hits.push((Rect::new(rect.x + 1, ry, rect.width - 2, 1), hit));
        }
        let _ = th().ui.muted;
    }
}

/// The dialog moves (its size follows its content).
const WIN: crate::dialog_kit::Win = crate::dialog_kit::Win::movable("merge_notice");
