//! The popup of the predicted conflicts: the files a merge with the base
//! branch would conflict on, and two ways forward, "Open merge preview" (the
//! branch menu's Merge of the base into this branch) and "Rebase onto base"
//! (its Rebase). Opened by a click on the badge or the palette's `Conflicts:
//! Show predicted conflicts`.
//!
//! Modal like `revert_mainline`: it owns the keyboard and the mouse while
//! open, a click away closes it, nothing behind it hovers. Every mouse
//! gesture has a key: a button = `Left` / `Right` / `Tab` to move and `Enter`
//! to press, the wheel = `Up` / `Down` / `PageUp` / `PageDown` on the file
//! list, a click away = `Esc`. Hit rects are kept from the paint walk.

use std::sync::Mutex;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use gitpoulpi_i18n::msg::conflict_prevention as msg;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::Frame;

use crate::dialog_kit::{self, ButtonState};
use crate::th;

/// What the popup asks the window to do (it closes itself first).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PopupAction {
    /// Merge this base branch into the current one.
    Merge(String),
    /// Rebase the current branch onto this base branch.
    Rebase(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Button {
    Merge,
    Rebase,
    Close,
}

const BUTTONS: [Button; 3] = [Button::Merge, Button::Rebase, Button::Close];

/// The most file rows shown at once.
const MAX_ROWS: usize = 10;

struct Open {
    base_ref: String,
    files: Vec<String>,
    /// The first file row shown.
    top: usize,
    focus: Button,
    hovered: Option<Button>,
    /// Rects from the paint walk.
    outer: Rect,
    list: Rect,
    buttons: Vec<(Rect, Button)>,
    close: dialog_kit::CloseHit,
}

static POPUP: Mutex<Option<Open>> = Mutex::new(None);

pub(crate) fn open(base_ref: &str, files: &[String]) {
    if let Ok(mut p) = POPUP.lock() {
        *p = Some(Open {
            base_ref: base_ref.to_string(),
            files: files.to_vec(),
            top: 0,
            focus: Button::Merge,
            hovered: None,
            outer: Rect::default(),
            list: Rect::default(),
            buttons: Vec::new(),
            close: Default::default(),
        });
    }
}

pub(crate) fn is_open() -> bool {
    POPUP.lock().is_ok_and(|p| p.is_some())
}

#[cfg(test)]
pub(crate) fn close() {
    if let Ok(mut p) = POPUP.lock() {
        *p = None;
    }
}

impl Open {
    fn rows(&self) -> usize {
        self.files.len().min(MAX_ROWS)
    }

    fn scroll_by(&mut self, delta: isize) {
        let max_top = self.files.len().saturating_sub(self.rows());
        self.top = self.top.saturating_add_signed(delta).min(max_top);
    }

    fn press(&self, button: Button) -> Option<PopupAction> {
        match button {
            Button::Merge => Some(PopupAction::Merge(self.base_ref.clone())),
            Button::Rebase => Some(PopupAction::Rebase(self.base_ref.clone())),
            Button::Close => None,
        }
    }

    fn move_focus(&mut self, step: isize) {
        let at = BUTTONS.iter().position(|b| *b == self.focus).unwrap_or(0) as isize;
        let next = (at + step).rem_euclid(BUTTONS.len() as isize) as usize;
        self.focus = BUTTONS[next];
    }
}

/// A key while the popup is open (it swallows them all). A pressed button
/// closes the popup and returns its action.
pub(crate) fn on_key(key: KeyEvent) -> Option<PopupAction> {
    let mut slot = POPUP.lock().ok()?;
    let open = slot.as_mut()?;
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);
    match key.code {
        KeyCode::Esc => *slot = None,
        KeyCode::Left | KeyCode::BackTab => open.move_focus(-1),
        KeyCode::Right => open.move_focus(1),
        KeyCode::Tab if shift => open.move_focus(-1),
        KeyCode::Tab => open.move_focus(1),
        KeyCode::Up => open.scroll_by(-1),
        KeyCode::Down => open.scroll_by(1),
        KeyCode::PageUp => open.scroll_by(-(MAX_ROWS as isize)),
        KeyCode::PageDown => open.scroll_by(MAX_ROWS as isize),
        KeyCode::Home => open.top = 0,
        KeyCode::End => open.scroll_by(isize::MAX / 2),
        KeyCode::Enter => {
            let action = open.press(open.focus);
            *slot = None;
            return action;
        }
        _ => {}
    }
    None
}

/// A mouse event while the popup is open (it swallows them all).
pub(crate) fn on_mouse(mouse: MouseEvent) -> Option<PopupAction> {
    let mut slot = POPUP.lock().ok()?;
    let open = slot.as_mut()?;
    let at: ratatui::layout::Position = (mouse.column, mouse.row).into();
    let over = open
        .buttons
        .iter()
        .find(|(rect, _)| rect.contains(at))
        .map(|(_, b)| *b);
    match mouse.kind {
        MouseEventKind::Moved | MouseEventKind::Drag(_) => {
            open.hovered = over;
            open.close.set_hover(mouse.column, mouse.row);
        }
        MouseEventKind::ScrollUp => open.scroll_by(-3),
        MouseEventKind::ScrollDown => open.scroll_by(3),
        MouseEventKind::Down(MouseButton::Left) => {
            if let Some(button) = over {
                let action = open.press(button);
                *slot = None;
                return action;
            }
            // The cross, or a click away, closes it and is consumed.
            if open.close.hit(mouse.column, mouse.row) || !open.outer.contains(at) {
                *slot = None;
            }
        }
        _ => {}
    }
    None
}

/// Paints the box, last, over the panes.
pub(crate) fn render(frame: &mut Frame) {
    let Ok(mut slot) = POPUP.lock() else {
        return;
    };
    let Some(open) = slot.as_mut() else {
        return;
    };
    let area = frame.area();
    let width = 72u16.min(area.width.saturating_sub(4));
    let rows = open.rows() as u16;
    let height = rows + 5;
    if width < 36 || area.height < height + 2 {
        return;
    }
    let outer = Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 3,
        width,
        height,
    );
    let panel_style = Style::default().fg(th().ui.text).bg(th().ui.panel);
    let buf = frame.buffer_mut();
    open.close.panel(buf, outer, msg::POPUP_TITLE.get());
    dialog_kit::put(
        buf,
        outer.x + 2,
        outer.y + 1,
        &msg::popup_intro(&open.base_ref),
        outer.width.saturating_sub(4),
        panel_style,
    );
    let list = Rect::new(
        outer.x + 2,
        outer.y + 2,
        outer.width.saturating_sub(4),
        rows,
    );
    let more_below = open.files.len() > open.top + open.rows();
    for (i, file) in open
        .files
        .iter()
        .skip(open.top)
        .take(open.rows())
        .enumerate()
    {
        let last = i + 1 == open.rows();
        // The last visible row says more follow, so a long list is not mistaken
        // for a short one.
        let text = if last && more_below {
            format!("{file}  ...")
        } else {
            file.clone()
        };
        dialog_kit::put(
            buf,
            list.x,
            list.y + i as u16,
            &text,
            list.width,
            Style::default().fg(th().status.warning).bg(th().ui.panel),
        );
    }
    let y = outer.y + outer.height - 2;
    let mut x = outer.x + 2;
    let mut buttons = Vec::new();
    for button in BUTTONS {
        let label = match button {
            Button::Merge => msg::POPUP_MERGE.get(),
            Button::Rebase => msg::POPUP_REBASE.get(),
            Button::Close => msg::POPUP_CLOSE.get(),
        };
        let state = ButtonState::Normal {
            hovered: open.hovered == Some(button),
            focused: open.focus == button,
        };
        let rect = dialog_kit::button(buf, x, y, label, state);
        x = rect.right() + 1;
        buttons.push((rect, button));
    }
    open.outer = outer;
    open.list = list;
    open.buttons = buttons;
}

#[cfg(test)]
mod tests;
