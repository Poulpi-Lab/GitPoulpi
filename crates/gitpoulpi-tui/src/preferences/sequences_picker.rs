//! The "Add a step" popup of the Sequences page: every command a sequence
//! may run (`STEP_VOCABULARY`), under the palette's own labels, filtered with
//! the palette's matcher. A step this build cannot run is listed grayed with
//! the reason and cannot be picked.
//!
//! Hosted by Settings itself (its own query field, hit rects and close
//! cross), so it never touches the main window's palette state. Keyboard:
//! type to filter, `Up`/`Down` move, `Enter` adds, `Esc` closes; the mouse
//! twins are a click on a row and the cross.

use ratatui::buffer::Buffer;

use super::*;
use crate::dialog_kit::{self, CloseHit};
use crate::display_width;
use crate::palette::{command_id, score, PALETTE_COMMANDS};
use crate::text_edit::FieldEditor;
use crate::th;
use gitpoulpi_config::sequences::STEP_VOCABULARY;
use gitpoulpi_i18n::msg::settings_sequences as t;

const WIN: dialog_kit::Win = dialog_kit::Win::movable("seq_step_picker");

/// One offered step.
pub(super) struct StepEntry {
    pub(super) id: &'static str,
    /// The palette's label (the sparkle included for an AI command).
    pub(super) label: String,
    /// Why it cannot be added, when it cannot.
    pub(super) why: Option<String>,
}

/// What a key or click on the picker asks of the page.
pub(super) enum PickOutcome {
    Stay,
    Close,
    Pick(&'static str),
}

/// The popup's state.
pub(super) struct StepPicker {
    query: String,
    /// Index into `visible()`.
    selected: usize,
    entries: Vec<StepEntry>,
    edit: FieldEditor<u8>,
    /// Painted this frame: each row's rect and its index in `entries`.
    rows: Vec<(Rect, usize)>,
    close: CloseHit,
    popup: Rect,
    /// How many rows fit, for the paging keys.
    page: usize,
}

/// The palette's name for the step `run`, else the id itself.
pub(super) fn step_label(run: &str) -> String {
    PALETTE_COMMANDS
        .iter()
        .find(|e| command_id(&e.command) == run)
        .map(|e| e.display_label().into_owned())
        .unwrap_or_else(|| run.to_string())
}

/// The vocabulary as the popup lists it, in the vocabulary's order.
pub(super) fn step_entries(ai_on: bool) -> Vec<StepEntry> {
    STEP_VOCABULARY
        .iter()
        .map(|kind| {
            let entry = PALETTE_COMMANDS
                .iter()
                .find(|e| command_id(&e.command) == kind.id);
            let why = if kind.pending {
                Some(t::PICKER_UNAVAILABLE_PENDING.get().to_string())
            } else {
                match entry {
                    None => Some(t::PICKER_UNKNOWN.get().to_string()),
                    Some(e) if !crate::build_features::offered(e.requires) => {
                        Some(t::PICKER_UNAVAILABLE_BUILD.get().to_string())
                    }
                    Some(e) if e.command.is_ai() && !ai_on => {
                        Some(t::PICKER_UNAVAILABLE_AI.get().to_string())
                    }
                    Some(_) => None,
                }
            };
            StepEntry {
                id: kind.id,
                label: entry
                    .map_or_else(|| kind.id.to_string(), |e| e.display_label().into_owned()),
                why,
            }
        })
        .collect()
}

impl StepPicker {
    pub(super) fn new(ai_on: bool) -> Self {
        Self::with_entries(step_entries(ai_on))
    }

    pub(super) fn with_entries(entries: Vec<StepEntry>) -> Self {
        StepPicker {
            query: String::new(),
            selected: 0,
            entries,
            edit: FieldEditor::default(),
            rows: Vec::new(),
            close: CloseHit::default(),
            popup: Rect::default(),
            page: 8,
        }
    }

    /// Indices into the entries that match the query, best first (the
    /// vocabulary's order while the query is empty).
    pub(super) fn visible(&self) -> Vec<usize> {
        let query = self.query.trim();
        if query.is_empty() {
            return (0..self.entries.len()).collect();
        }
        let mut scored: Vec<(i32, usize)> = self
            .entries
            .iter()
            .enumerate()
            .filter_map(|(i, e)| {
                let words = e.id.replace('_', " ");
                score(&e.label, query)
                    .into_iter()
                    .chain(score(&words, query))
                    .max()
                    .map(|s| (s, i))
            })
            .collect();
        // Stable: equal scores keep the vocabulary's order.
        scored.sort_by_key(|(s, _)| std::cmp::Reverse(*s));
        scored.into_iter().map(|(_, i)| i).collect()
    }

    /// The ids listed right now, in order (tests).
    #[cfg(test)]
    pub(super) fn visible_ids(&self) -> Vec<&'static str> {
        self.visible().iter().map(|&i| self.entries[i].id).collect()
    }

    pub(super) fn type_text(&mut self, text: &str) {
        let clean: String = text.chars().filter(|c| !c.is_control()).collect();
        self.edit.caret_mut(0).insert(&mut self.query, &clean);
        self.selected = 0;
    }

    fn pick_selected(&self) -> PickOutcome {
        let visible = self.visible();
        match visible.get(self.selected).map(|&i| &self.entries[i]) {
            Some(e) if e.why.is_none() => PickOutcome::Pick(e.id),
            _ => PickOutcome::Stay,
        }
    }

    pub(super) fn on_key(&mut self, key: &KeyEvent) -> PickOutcome {
        let len = self.visible().len();
        let last = len.saturating_sub(1);
        match key.code {
            KeyCode::Esc => return PickOutcome::Close,
            KeyCode::Enter => return self.pick_selected(),
            KeyCode::Down => self.selected = (self.selected + 1).min(last),
            KeyCode::Up => self.selected = self.selected.saturating_sub(1),
            KeyCode::PageDown => self.selected = (self.selected + self.page).min(last),
            KeyCode::PageUp => self.selected = self.selected.saturating_sub(self.page),
            _ => {
                let before = self.query.clone();
                if self.edit.dialog_key(0, &mut self.query, key).is_some() && self.query != before {
                    self.selected = 0;
                }
            }
        }
        PickOutcome::Stay
    }

    pub(super) fn on_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> PickOutcome {
        let last = self.visible().len().saturating_sub(1);
        match kind {
            MouseEventKind::ScrollDown => self.selected = (self.selected + 3).min(last),
            MouseEventKind::ScrollUp => self.selected = self.selected.saturating_sub(3),
            MouseEventKind::Down(MouseButton::Left) => {
                if self.close.hit(col, row) {
                    return PickOutcome::Close;
                }
                let hit = self
                    .rows
                    .iter()
                    .find(|(r, _)| r.contains((col, row).into()))
                    .copied();
                if let Some((_, entry)) = hit {
                    if let Some(at) = self.visible().iter().position(|&i| i == entry) {
                        self.selected = at;
                    }
                    return self.pick_selected();
                }
                if !self.popup.contains((col, row).into()) {
                    return PickOutcome::Close;
                }
            }
            _ => {}
        }
        PickOutcome::Stay
    }

    pub(super) fn render(&mut self, buf: &mut Buffer, screen: Rect, pointer: Option<(u16, u16)>) {
        let rect = WIN.place(screen, dialog_kit::centered(screen, 68, 22));
        self.popup = rect;
        if let Some((c, r)) = pointer {
            self.close.set_hover(c, r);
        }
        self.close
            .framed(buf, rect, t::PICKER_TITLE.get(), &WIN, true);
        let dim = Style::default().fg(th().ui.muted).bg(th().ui.panel);
        let x = rect.x + 2;
        let w = rect.width.saturating_sub(4);
        self.edit.begin_frame();
        self.edit.sync(Some(0));
        dialog_kit::text_input(
            buf,
            Rect::new(x, rect.y + 1, w, 1),
            &self.query,
            "",
            true,
            false,
            (&self.edit, 0),
        );
        let top = rect.y + 3;
        let bottom = rect.bottom().saturating_sub(2);
        let rows = bottom.saturating_sub(top) as usize;
        self.page = rows.max(1);
        self.rows.clear();
        let visible = self.visible();
        self.selected = self.selected.min(visible.len().saturating_sub(1));
        if visible.is_empty() {
            dialog_kit::put(buf, x, top, t::PICKER_EMPTY.get(), w, dim);
        }
        let start = crate::popup_list::View::window_start(self.selected, rows, visible.len());
        for (n, &index) in visible.iter().enumerate().skip(start).take(rows) {
            let y = top + (n - start) as u16;
            let entry = &self.entries[index];
            let selected = n == self.selected;
            let hovered = pointer.is_some_and(|(c, r)| r == y && c >= x && c < x + w);
            let row = Rect::new(x, y, w, 1);
            let bg = dialog_kit::row_bg(selected, hovered);
            dialog_kit::row_band(buf, row, bg);
            let (label_style, why_style) = if entry.why.is_some() {
                let s = Style::default().fg(th().ui.disabled).bg(bg);
                (s, s)
            } else {
                (
                    Style::default().fg(th().ui.text).bg(bg),
                    Style::default().fg(th().ui.muted).bg(bg),
                )
            };
            dialog_kit::put(
                buf,
                x + 1,
                y,
                &entry.label,
                w.saturating_sub(2),
                label_style,
            );
            if let Some(why) = &entry.why {
                let wd = display_width(why) as u16;
                let at = (x + w).saturating_sub(wd + 1);
                if at > x + 2 + display_width(&entry.label) as u16 {
                    dialog_kit::put(buf, at, y, why, wd, why_style);
                }
            }
            crate::pointer_map::hand(row);
            self.rows.push((row, index));
        }
        dialog_kit::put(
            buf,
            x,
            rect.bottom().saturating_sub(2),
            t::PICKER_HINT.get(),
            w,
            dim,
        );
    }
}
