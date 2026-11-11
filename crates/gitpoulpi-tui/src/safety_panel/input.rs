//! Keys and mouse for the panel. Every click has its key: rows move with
//! the arrows, a button is its letter, the right-click menu is `.`.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use super::render::Hit;
use super::{Act, Overlay, PanelOutcome, RowMenu, SafetyPanel, Tab};
use crate::combo::ComboOutcome;
use crate::dialog_kit;

/// The letter that does `act` (the buttons show it in the hint line).
pub(super) fn act_for_key(c: char, tab: Tab) -> Option<Act> {
    match (c, tab) {
        ('u', Tab::History) => Some(Act::UndoTo),
        ('y', Tab::History) => Some(Act::RedoTo),
        ('r', Tab::History) => Some(Act::RestoreOnly),
        ('o', Tab::History) => Some(Act::Offer),
        ('r', Tab::Reflog) => Some(Act::RestoreRef),
        ('s', Tab::Reflog) => Some(Act::ShowInLog),
        ('b', Tab::Reflog | Tab::Lost) => Some(Act::CreateBranch),
        ('t', Tab::Reflog | Tab::Lost) => Some(Act::CreateTag),
        ('w', Tab::Reflog | Tab::Lost) => Some(Act::ApplyToWorktree),
        ('z', Tab::Reflog | Tab::Lost) => Some(Act::SaveAsStash),
        ('m', Tab::Reflog) => Some(Act::LoadMore),
        ('c', Tab::Lost) => Some(Act::CherryPick),
        _ => None,
    }
}

impl SafetyPanel {
    pub(crate) fn on_key(&mut self, key: KeyEvent) -> PanelOutcome {
        if self.overlay.is_some() {
            return self.overlay_key(key);
        }
        let state = self.journal_state();
        let page = self.list_mut().view_rows.max(1) as isize;
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => return PanelOutcome::Close,
            KeyCode::Tab => self.cycle_tab(true),
            KeyCode::BackTab => self.cycle_tab(false),
            KeyCode::Char('1') => self.set_tab(Tab::History),
            KeyCode::Char('2') => self.set_tab(Tab::Reflog),
            KeyCode::Char('3') => self.set_tab(Tab::Lost),
            KeyCode::Down | KeyCode::Char('j') => self.move_selection(1),
            KeyCode::Up | KeyCode::Char('k') => self.move_selection(-1),
            KeyCode::PageDown => self.move_selection(page),
            KeyCode::PageUp => self.move_selection(-page),
            KeyCode::Home => self.move_selection(isize::MIN / 2),
            KeyCode::End | KeyCode::Char('G') => self.move_selection(isize::MAX / 2),
            KeyCode::Enter => {
                if let Some(act) = self.actions(&state).first().copied() {
                    return self.run(act, &state);
                }
            }
            KeyCode::Menu | KeyCode::Char('.') => self.open_menu(None, &state),
            KeyCode::F(10) if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.open_menu(None, &state)
            }
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(act) = act_for_key(c, self.tab) {
                    if self.actions(&state).contains(&act) {
                        return self.run(act, &state);
                    }
                }
            }
            _ => {}
        }
        PanelOutcome::Stay
    }

    /// Moves the selection `delta` lines; on the history list it never
    /// rests on a separator.
    fn move_selection(&mut self, delta: isize) {
        let len = self.list_len();
        self.list_mut().move_by(delta, len);
        if self.tab == Tab::History {
            self.history.snap(delta);
        }
    }

    fn cycle_tab(&mut self, forward: bool) {
        let order = [Tab::History, Tab::Reflog, Tab::Lost];
        let at = order.iter().position(|t| *t == self.tab).unwrap_or(0);
        let next = if forward {
            (at + 1) % order.len()
        } else {
            (at + order.len() - 1) % order.len()
        };
        self.set_tab(order[next]);
    }

    fn open_menu(&mut self, at: Option<(u16, u16)>, state: &gitpoulpi_core::domain::JournalState) {
        let items = self.actions(state);
        if items.is_empty() {
            return;
        }
        let list = *self.list_mut();
        let row_y = self.list_body.y + 1 + list.selected.saturating_sub(list.offset) as u16;
        self.overlay = Some(Overlay::Menu(RowMenu {
            items,
            selected: 0,
            at: at.unwrap_or((self.list_body.x + 4, row_y)),
        }));
    }

    fn overlay_key(&mut self, key: KeyEvent) -> PanelOutcome {
        let state = self.journal_state();
        let Some(overlay) = self.overlay.as_mut() else {
            return PanelOutcome::Stay;
        };
        match overlay {
            Overlay::Confirm(c) => match key.code {
                KeyCode::Enter | KeyCode::Char('y') => {
                    let next = c.next.clone();
                    self.confirm(next);
                }
                KeyCode::Esc | KeyCode::Char('n') => self.overlay = None,
                _ => {}
            },
            Overlay::Name(n) => {
                let purpose = n.purpose.clone();
                let before = n.value.clone();
                if let Some((_, copied)) =
                    n.edit
                        .dialog_key(super::NameField::Name, &mut n.value, &key)
                {
                    if copied.is_some() || before != n.value {
                        n.error = None;
                    }
                    return PanelOutcome::Stay;
                }
                match key.code {
                    KeyCode::Enter => {
                        let value = n.value.clone();
                        self.submit_name(purpose, value);
                    }
                    KeyCode::Esc => self.overlay = None,
                    _ => {}
                }
            }
            Overlay::Mode(p) => {
                if p.combo.is_open() {
                    if let ComboOutcome::Picked(_) = p.combo.on_key(key) {}
                    return PanelOutcome::Stay;
                }
                match key.code {
                    KeyCode::Esc => self.overlay = None,
                    KeyCode::Enter => {
                        let mode = p.combo.selected().unwrap_or("keep").to_string();
                        let (refname, oid) = (p.refname.clone(), p.oid);
                        self.submit_mode(refname, oid, mode);
                    }
                    _ => {
                        let _ = p.combo.on_key(key);
                    }
                }
            }
            Overlay::Menu(menu) => match key.code {
                KeyCode::Esc => self.overlay = None,
                KeyCode::Down | KeyCode::Char('j') => {
                    menu.selected = (menu.selected + 1) % menu.items.len();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    menu.selected = (menu.selected + menu.items.len() - 1) % menu.items.len();
                }
                KeyCode::Enter => {
                    let act = menu.items[menu.selected];
                    self.overlay = None;
                    return self.run(act, &state);
                }
                _ => {}
            },
        }
        PanelOutcome::Stay
    }

    pub(crate) fn on_paste(&mut self, text: &str) {
        let clean: String = text.chars().filter(|c| !c.is_control()).collect();
        if let Some(Overlay::Name(n)) = self.overlay.as_mut() {
            n.edit
                .caret_mut(super::NameField::Name)
                .insert(&mut n.value, &clean);
            n.error = None;
        }
    }

    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent) -> PanelOutcome {
        let (col, row) = (mouse.column, mouse.row);
        let hit = dialog_kit::hit_at(&self.hits, col, row);
        let state = self.journal_state();
        // The open mode list is modal: it answers first.
        if let Some(Overlay::Mode(p)) = self.overlay.as_mut() {
            if p.combo.is_open() {
                let _ = p.combo.on_mouse(mouse);
                return PanelOutcome::Stay;
            }
        }
        match mouse.kind {
            MouseEventKind::Moved => {
                self.close.set_hover(col, row);
                self.hover = hit.filter(|h| !matches!(h, Hit::Body));
                if let Some(Overlay::Menu(menu)) = self.overlay.as_mut() {
                    if let Some(Hit::MenuItem(k)) = hit {
                        menu.selected = k;
                    }
                }
            }
            MouseEventKind::ScrollDown if self.overlay.is_none() => {
                self.move_selection(3);
            }
            MouseEventKind::ScrollUp if self.overlay.is_none() => {
                self.move_selection(-3);
            }
            MouseEventKind::Drag(MouseButton::Left) => self.drag_bar(row),
            MouseEventKind::Up(MouseButton::Left) => self.list_mut().grab = None,
            MouseEventKind::Down(MouseButton::Left) => {
                // The cross and a click outside the panel are Esc: the
                // innermost open thing (menu, confirmation) cancels first.
                if self.close.hit(col, row) || !self.frame.contains((col, row).into()) {
                    if self.overlay.take().is_some() {
                        return PanelOutcome::Stay;
                    }
                    return PanelOutcome::Close;
                }
                if self.overlay.is_none() && self.press_bar(col, row) {
                    return PanelOutcome::Stay;
                }
                return self.click(hit, &state);
            }
            MouseEventKind::Down(MouseButton::Right) if self.overlay.is_none() => {
                if let Some(Hit::Row(index)) = hit {
                    self.list_mut().selected = index;
                    self.open_menu(Some((col, row)), &state);
                }
            }
            _ => {}
        }
        PanelOutcome::Stay
    }

    fn click(
        &mut self,
        hit: Option<Hit>,
        state: &gitpoulpi_core::domain::JournalState,
    ) -> PanelOutcome {
        let Some(hit) = hit else {
            // A click on nothing closes a menu, never a confirmation.
            if matches!(self.overlay, Some(Overlay::Menu(_))) {
                self.overlay = None;
            }
            return PanelOutcome::Stay;
        };
        match (hit, self.overlay.is_some()) {
            (Hit::Confirm, true) => match self.overlay.take() {
                Some(Overlay::Confirm(c)) => self.confirm(c.next),
                Some(Overlay::Name(n)) => self.submit_name(n.purpose, n.value),
                Some(Overlay::Mode(p)) => {
                    let mode = p.combo.selected().unwrap_or("keep").to_string();
                    self.submit_mode(p.refname, p.oid, mode);
                }
                other => self.overlay = other,
            },
            (Hit::Cancel, true) => self.overlay = None,
            (Hit::ModeField, true) => {
                if let Some(Overlay::Mode(p)) = self.overlay.as_mut() {
                    p.combo.open();
                }
            }
            (Hit::MenuItem(k), true) => {
                if let Some(Overlay::Menu(menu)) = self.overlay.take() {
                    if let Some(act) = menu.items.get(k).copied() {
                        return self.run(act, state);
                    }
                }
            }
            (_, true) => {}
            (Hit::Tab(tab), false) => self.set_tab(tab),
            (Hit::Row(index), false) => {
                let len = self.list_len();
                let list = self.list_mut();
                list.selected = index.min(len.saturating_sub(1));
                list.keep_visible();
            }
            (Hit::Act(act), false) => return self.run(act, state),
            (Hit::Close, false) => return PanelOutcome::Close,
            _ => {}
        }
        PanelOutcome::Stay
    }

    fn bar_region(&mut self) -> crate::ui_kit::ListBarRegion {
        let len = self.list_len();
        let area = self.list_body;
        let visible = self.list_mut().view_rows.max(1);
        crate::ui_kit::ListBarRegion {
            bar: crate::ui_kit::ListBar::SafetyPanel,
            area,
            len,
            visible,
        }
    }

    fn press_bar(&mut self, col: u16, row: u16) -> bool {
        let region = self.bar_region();
        let offset = self.list_mut().offset;
        let Some(press) = crate::ui_kit::list_bar_press(region, offset, col, row) else {
            return false;
        };
        let list = self.list_mut();
        if let Some(to) = press.jump_to {
            list.offset = to;
        }
        list.grab = (!press.arrow).then_some(press.grab);
        true
    }

    fn drag_bar(&mut self, row: u16) {
        let region = self.bar_region();
        let list = self.list_mut();
        if let Some(grab) = list.grab {
            list.offset = crate::ui_kit::list_bar_drag(region, list.offset, grab, row);
        }
    }
}
