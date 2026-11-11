//! "How do you want to review this request?": the question a Radar Review
//! asks for a request of an account-wide row, unless the profile's
//! `review_mode` already answered it.
//!
//! Three ways, as numbered-by-letter rows (a letter picks, Enter picks the
//! highlighted one, a click picks):
//!
//! - `l` **Open the local clone**, only when a clone of the repository is
//!   known;
//! - `c` **Clone it, then review**, only when none is;
//! - `a` **Review through the API**, the host's diff, no clone.
//!
//! plus **Cancel** and a **Remember for `<profile>`** checkbox (`r`) that
//! writes the profile's `review_mode` for the clone and API picks. Closable
//! like every dialog: the cross, `Esc`, a click outside.

use crossterm::event::{
    KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use crate::dialog_kit::{self, CheckState};
use crate::th;
use gitpoulpi_i18n::msg::radar as msg;

/// The way chosen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Pick {
    Local,
    Clone,
    Api,
}

impl Pick {
    fn key(self) -> char {
        match self {
            Pick::Local => 'l',
            Pick::Clone => 'c',
            Pick::Api => 'a',
        }
    }

    fn label(self) -> &'static str {
        match self {
            Pick::Local => msg::CHOICE_LOCAL.get(),
            Pick::Clone => msg::CHOICE_CLONE.get(),
            Pick::Api => msg::CHOICE_API.get(),
        }
    }
}

/// What the dialog decided after an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChoiceOutcome {
    Stay,
    Cancel,
    Pick { pick: Pick, remember: bool },
}

/// A control of the dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Slot {
    Row(usize),
    Cancel,
    Remember,
}

pub(crate) struct ReviewChoice {
    /// The request, as the dialog names it (`!58 Round invoice totals`).
    caption: String,
    /// `owner/name`, for the line under it.
    project: String,
    /// The profile the choice can be remembered for.
    profile: String,
    options: Vec<Pick>,
    focus: Slot,
    remember: bool,
    /// The paint walk's rects, in click order (inner first).
    hits: Vec<(Rect, Slot)>,
    frame: Rect,
    close: dialog_kit::CloseHit,
}

/// The choice moves (its size follows its options).
const WIN: crate::dialog_kit::Win = crate::dialog_kit::Win::movable("review_choice");

impl ReviewChoice {
    /// `cloned`: a known clone exists (offers `l`, else `c`).
    pub(crate) fn new(caption: String, project: String, profile: String, cloned: bool) -> Self {
        let options = if cloned {
            vec![Pick::Local, Pick::Api]
        } else {
            vec![Pick::Clone, Pick::Api]
        };
        Self {
            caption,
            project,
            profile,
            options,
            focus: Slot::Row(0),
            remember: false,
            hits: Vec::new(),
            frame: Rect::default(),
            close: Default::default(),
        }
    }

    /// Every control in Tab order.
    fn slots(&self) -> Vec<Slot> {
        let mut out: Vec<Slot> = (0..self.options.len()).map(Slot::Row).collect();
        out.push(Slot::Cancel);
        out.push(Slot::Remember);
        out
    }

    fn step(&mut self, forward: bool) {
        let slots = self.slots();
        let at = slots.iter().position(|s| *s == self.focus).unwrap_or(0);
        let next = if forward {
            (at + 1) % slots.len()
        } else {
            (at + slots.len() - 1) % slots.len()
        };
        self.focus = slots[next];
    }

    fn pick(&self, index: usize) -> ChoiceOutcome {
        match self.options.get(index) {
            Some(pick) => ChoiceOutcome::Pick {
                pick: *pick,
                remember: self.remember,
            },
            None => ChoiceOutcome::Stay,
        }
    }

    fn activate(&mut self, slot: Slot) -> ChoiceOutcome {
        match slot {
            Slot::Row(i) => self.pick(i),
            Slot::Cancel => ChoiceOutcome::Cancel,
            Slot::Remember => {
                self.remember = !self.remember;
                ChoiceOutcome::Stay
            }
        }
    }

    pub(crate) fn on_key(&mut self, key: KeyEvent) -> ChoiceOutcome {
        if key.kind != KeyEventKind::Press {
            return ChoiceOutcome::Stay;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return ChoiceOutcome::Stay;
        }
        match key.code {
            KeyCode::Esc => ChoiceOutcome::Cancel,
            KeyCode::Up | KeyCode::BackTab | KeyCode::Char('k') => {
                self.step(false);
                ChoiceOutcome::Stay
            }
            KeyCode::Down | KeyCode::Tab | KeyCode::Char('j') => {
                self.step(true);
                ChoiceOutcome::Stay
            }
            KeyCode::Enter | KeyCode::Char(' ') => self.activate(self.focus),
            KeyCode::Char('r') => {
                self.remember = !self.remember;
                ChoiceOutcome::Stay
            }
            KeyCode::Char(c) => match self.options.iter().position(|p| p.key() == c) {
                Some(i) => self.pick(i),
                None => ChoiceOutcome::Stay,
            },
            _ => ChoiceOutcome::Stay,
        }
    }

    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent) -> ChoiceOutcome {
        let (col, row) = (mouse.column, mouse.row);
        let over = dialog_kit::hit_at(&self.hits, col, row);
        match mouse.kind {
            MouseEventKind::Moved => {
                self.close.set_hover(col, row);
                if let Some(slot) = over {
                    self.focus = slot;
                }
                ChoiceOutcome::Stay
            }
            MouseEventKind::Down(MouseButton::Left) => {
                if self.close.hit(col, row) {
                    return ChoiceOutcome::Cancel;
                }
                match over {
                    Some(slot) => {
                        self.focus = slot;
                        self.activate(slot)
                    }
                    // A click away closes the dialog; one inside it on
                    // nothing in particular does nothing.
                    None if !self.frame.contains((col, row).into()) => ChoiceOutcome::Cancel,
                    None => ChoiceOutcome::Stay,
                }
            }
            _ => ChoiceOutcome::Stay,
        }
    }

    /// What `Ctrl+N` badges: every control, from this paint's own rects.
    pub(crate) fn nav_targets(&self) -> Vec<(Rect, u8)> {
        let slots = self.slots();
        let mut out = Vec::new();
        for (rect, slot) in &self.hits {
            if let Some(i) = slots.iter().position(|s| s == slot) {
                // A row's badge sits on the gap before its label, so it does
                // not hide the letter that picks it.
                let rect = match slot {
                    Slot::Row(_) => Rect::new(rect.x + 2, rect.y, rect.width.saturating_sub(2), 1),
                    _ => Rect::new(rect.x.saturating_sub(1), rect.y, rect.width + 1, 1),
                };
                out.push((rect, i as u8));
            }
        }
        out.sort_by_key(|(r, i)| (r.y, r.x, *i));
        out
    }

    /// A badge pressed: the same as a click on its control.
    pub(crate) fn nav_press(&mut self, index: u8) -> ChoiceOutcome {
        match self.slots().get(index as usize).copied() {
            Some(slot) => {
                self.focus = slot;
                self.activate(slot)
            }
            None => ChoiceOutcome::Stay,
        }
    }

    pub(crate) fn render(&mut self, buf: &mut Buffer, body: Rect) {
        let width = 64u16.min(body.width.saturating_sub(2));
        let height = (self.options.len() as u16 + 10).min(body.height);
        if width < 30 || height < 9 {
            self.hits.clear();
            return;
        }
        let outer = WIN.place(body, crate::dialog_kit::centered(body, width, height));
        self.frame = outer;
        self.hits.clear();
        self.close
            .framed(buf, outer, msg::CHOICE_TITLE.get(), &WIN, true);
        let inner_w = outer.width.saturating_sub(4);
        let text = Style::default().fg(th().ui.text).bg(th().ui.panel);
        let muted = Style::default().fg(th().ui.muted).bg(th().ui.panel);
        dialog_kit::put(
            buf,
            outer.x + 2,
            outer.y + 1,
            &self.caption,
            inner_w,
            text.add_modifier(Modifier::BOLD),
        );
        dialog_kit::put(
            buf,
            outer.x + 2,
            outer.y + 2,
            &msg::choice_project(&self.project),
            inner_w,
            muted,
        );
        let mut y = outer.y + 4;
        let mut rows = Vec::new();
        for (i, pick) in self.options.iter().enumerate() {
            let selected = self.focus == Slot::Row(i);
            let style = if selected {
                Style::default()
                    .fg(th().ui.text)
                    .bg(th().ui.selection)
                    .add_modifier(Modifier::BOLD)
            } else {
                text
            };
            let row = Rect::new(outer.x + 1, y, outer.width.saturating_sub(2), 1);
            dialog_kit::fill(buf, row, style);
            dialog_kit::put(
                buf,
                outer.x + 2,
                y,
                &format!("{}  {}", pick.key(), pick.label()),
                inner_w,
                style,
            );
            rows.push((row, Slot::Row(i)));
            y += 1;
        }
        y += 1;
        let cancel = dialog_kit::button(
            buf,
            outer.x + 2,
            y,
            msg::CHOICE_CANCEL.get(),
            dialog_kit::ButtonState::Normal {
                hovered: false,
                focused: self.focus == Slot::Cancel,
            },
        );
        self.hits.push((cancel, Slot::Cancel));
        let check = dialog_kit::checkbox(
            buf,
            cancel.right() + 2,
            y,
            outer.right().saturating_sub(cancel.right() + 4),
            &msg::choice_remember(&self.profile),
            self.remember,
            CheckState::Enabled {
                hovered: false,
                focused: self.focus == Slot::Remember,
            },
        );
        self.hits.push((check, Slot::Remember));
        self.hits.extend(rows);
        dialog_kit::put(
            buf,
            outer.x + 2,
            outer.bottom().saturating_sub(2),
            msg::CHOICE_HINT.get(),
            inner_w,
            muted,
        );
    }
}

#[cfg(test)]
mod tests;
