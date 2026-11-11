//! Keys and mouse for the panel. Every click has its key: rows move with
//! the arrows, a button is its letter, the right-click menu is `.`.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use super::panes::{Divider, Pane};
use super::render::Hit;
use super::{Act, Overlay, PanelOutcome, RowMenu, SafetyPanel, Tab};
use crate::combo::ComboOutcome;
use crate::dialog_kit::{self, FrameMouse};

/// The three scrollable things of the panel, each with its own bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Bar {
    List,
    Files,
    Diff,
}

/// The letter that does `act` (the buttons show it in the hint line).
pub(super) fn act_for_key(c: char, tab: Tab) -> Option<Act> {
    match (c, tab) {
        ('u', Tab::History) => Some(Act::UndoTo),
        ('y', Tab::History) => Some(Act::RedoTo),
        ('r', Tab::History) => Some(Act::RestoreOnly),
        ('o', Tab::History) => Some(Act::Offer),
        ('r', Tab::Reflog) => Some(Act::RestoreRef),
        ('l', Tab::Reflog) => Some(Act::ShowInLog),
        ('b', Tab::Reflog | Tab::Lost) => Some(Act::CreateBranch),
        ('t', Tab::Reflog | Tab::Lost) => Some(Act::CreateTag),
        ('w', Tab::Reflog | Tab::Lost) => Some(Act::ApplyToWorktree),
        ('s', Tab::Reflog | Tab::Lost) => Some(Act::SaveAsStash),
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
        // The arrange mode (`Ctrl+N m`) takes every key until it is left.
        if self.window.on_key(key) {
            return PanelOutcome::Stay;
        }
        let state = self.journal_state();
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        if alt && self.tab != Tab::History && self.alt_key(key.code) {
            return PanelOutcome::Stay;
        }
        let page = self.focus_rows().max(1) as isize;
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => return PanelOutcome::Close,
            // `Ctrl+Tab` switches the view (its twin is `Ctrl+N` and the
            // badge of a view's tab); plain `Tab` walks the panes.
            KeyCode::Tab if ctrl => self.cycle_tab(true),
            KeyCode::BackTab if ctrl => self.cycle_tab(false),
            KeyCode::Tab => self.cycle_focus(true),
            KeyCode::BackTab => self.cycle_focus(false),
            KeyCode::Char('1') => self.set_tab(Tab::History),
            KeyCode::Char('2') => self.set_tab(Tab::Reflog),
            KeyCode::Char('3') => self.set_tab(Tab::Lost),
            KeyCode::Char('v') if self.tab != Tab::History => self.panes.toggle_mode(),
            KeyCode::Down | KeyCode::Char('j') => self.move_focused(1),
            KeyCode::Up | KeyCode::Char('k') => self.move_focused(-1),
            KeyCode::PageDown => self.move_focused(page),
            KeyCode::PageUp => self.move_focused(-page),
            KeyCode::Home => self.move_focused(isize::MIN / 2),
            KeyCode::End | KeyCode::Char('G') => self.move_focused(isize::MAX / 2),
            KeyCode::Right | KeyCode::Left
                if self.tab != Tab::History && self.panes.focus == Pane::Files =>
            {
                self.fold_at_cursor(key.code == KeyCode::Right);
            }
            // Enter never runs an action (each button has its own letter):
            // on the file list it only navigates, a folder folds and a file
            // sends the keyboard to its hunks.
            KeyCode::Enter if self.tab != Tab::History && self.panes.focus == Pane::Files => {
                let row = self.panes.files.selected;
                if !self.panes.toggle_folder(row) {
                    self.panes.focus = Pane::Diff;
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

    /// Rows the focused pane shows (a page).
    fn focus_rows(&mut self) -> usize {
        match (self.tab, self.panes.focus) {
            (Tab::History, _) | (_, Pane::List) => self.list_mut().view_rows,
            (_, Pane::Files) => self.panes.files.view_rows,
            (_, Pane::Diff) => self.panes.scroll.view_rows,
        }
    }

    /// Moves what the keyboard points at: the commit list, the file list or
    /// the diff's scroll.
    fn move_focused(&mut self, delta: isize) {
        if self.tab == Tab::History {
            return self.move_selection(delta);
        }
        match self.panes.focus {
            Pane::List => self.move_selection(delta),
            Pane::Files => self.panes.move_file(delta),
            Pane::Diff => {
                let s = &mut self.panes.scroll;
                s.offset = s.offset.saturating_add_signed(delta);
            }
        }
    }

    /// `Tab` / `Shift+Tab`: the next pane (the history view has only its list).
    pub(super) fn cycle_focus_pub(&mut self, forward: bool) {
        self.cycle_focus(forward);
    }

    fn cycle_focus(&mut self, forward: bool) {
        if self.tab == Tab::History {
            return;
        }
        let focus = self.panes.focus;
        self.panes.focus = if forward { focus.next() } else { focus.prev() };
    }

    /// `Right` / `Left` on the file list: unfold / fold the folder row.
    fn fold_at_cursor(&mut self, open: bool) {
        let row = self.panes.files.selected;
        if let Some(crate::status_pane::TreeRow::Folder { collapsed, .. }) =
            self.panes.tree.get(row)
        {
            if *collapsed == open {
                self.panes.toggle_folder(row);
            }
        }
    }

    /// `Alt+arrows` move the dividers, `Alt+0` puts them back. `true` when
    /// the key was one of them.
    fn alt_key(&mut self, code: KeyCode) -> bool {
        let body = self.panes_body();
        match code {
            KeyCode::Left => self.panes.nudge_divider(Divider::Vertical, -1, body),
            KeyCode::Right => self.panes.nudge_divider(Divider::Vertical, 1, body),
            KeyCode::Up => self.panes.nudge_divider(Divider::Horizontal, -1, body),
            KeyCode::Down => self.panes.nudge_divider(Divider::Horizontal, 1, body),
            KeyCode::Char('0') => self.panes.reset_dividers(),
            _ => return false,
        }
        true
    }

    /// The area the list and the right pane share, as last painted.
    pub(super) fn panes_body(&self) -> ratatui::layout::Rect {
        self.panes.layout.body()
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
        // The frame: a title drag moves, the borders and bottom corners
        // resize, and a gesture that started continues wherever the pointer
        // goes (its release is the frame's, so letting go outside never
        // closes). The close cross keeps its click.
        let on_cross = self.close.hit(col, row);
        if self
            .window
            .on_mouse(mouse, on_cross, self.overlay.is_none())
            == FrameMouse::Consumed
        {
            self.overlay = None;
            return PanelOutcome::Stay;
        }
        // A held divider follows the pointer, in or out of the dialog.
        if self.panes.drag.is_some() {
            match mouse.kind {
                MouseEventKind::Drag(MouseButton::Left) => {
                    let body = self.panes_body();
                    self.panes.drag_to(col, row, body);
                    return PanelOutcome::Stay;
                }
                MouseEventKind::Up(MouseButton::Left) => {
                    self.panes.drag = None;
                    return PanelOutcome::Stay;
                }
                _ => {}
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
                self.wheel(col, row, 3);
            }
            MouseEventKind::ScrollUp if self.overlay.is_none() => {
                self.wheel(col, row, -3);
            }
            MouseEventKind::Drag(MouseButton::Left) => self.drag_bar(row),
            MouseEventKind::Up(MouseButton::Left) => self.release_bars(),
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
                if let (None, Some(Hit::Divider(which))) = (&self.overlay, hit) {
                    self.panes.press_divider(which);
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
                self.panes.focus = Pane::List;
                let len = self.list_len();
                let list = self.list_mut();
                list.selected = index.min(len.saturating_sub(1));
                list.keep_visible();
            }
            (Hit::Pane(pane), false) => self.panes.focus = pane,
            (Hit::File(index), false) => {
                self.panes.focus = Pane::Files;
                self.panes.select_file(index);
                // A folder row folds on a click; a file row shows its hunks.
                self.panes.toggle_folder(index);
            }
            (Hit::Mode(mode), false) => self.panes.set_mode(mode),
            (Hit::Divider(_), false) => {}
            (Hit::Act(act), false) => return self.run(act, state),
            (Hit::Close, false) => return PanelOutcome::Close,
            _ => {}
        }
        PanelOutcome::Stay
    }

    /// The wheel over `(col, row)`: the pane under the pointer scrolls (the
    /// commit list moves its cursor, as it always did).
    fn wheel(&mut self, col: u16, row: u16, delta: isize) {
        let at = ratatui::layout::Position::new(col, row);
        let lay = self.panes.layout;
        if self.tab != Tab::History && lay.files.contains(at) {
            self.panes.move_file(delta);
        } else if self.tab != Tab::History && lay.diff.contains(at) {
            let s = &mut self.panes.scroll;
            s.offset = s.offset.saturating_add_signed(delta);
        } else {
            self.move_selection(delta);
        }
    }

    fn bar_region(&mut self, bar: Bar) -> crate::ui_kit::ListBarRegion {
        let (area, len, visible) = match bar {
            Bar::List => (
                self.list_body,
                self.list_len(),
                self.list_mut().view_rows.max(1),
            ),
            Bar::Files => (
                self.panes.layout.files,
                self.panes.tree.len(),
                self.panes.files.view_rows.max(1),
            ),
            Bar::Diff => (
                self.panes.layout.diff,
                self.panes.diff_len,
                self.panes.scroll.view_rows.max(1),
            ),
        };
        crate::ui_kit::ListBarRegion {
            bar: crate::ui_kit::ListBar::SafetyPanel,
            area,
            len,
            visible,
        }
    }

    fn bar_state(&mut self, bar: Bar) -> &mut super::ListState {
        match bar {
            Bar::List => self.list_mut(),
            Bar::Files => &mut self.panes.files,
            Bar::Diff => &mut self.panes.scroll,
        }
    }

    fn bars(&self) -> &'static [Bar] {
        if self.tab == Tab::History {
            &[Bar::List]
        } else {
            &[Bar::List, Bar::Files, Bar::Diff]
        }
    }

    fn press_bar(&mut self, col: u16, row: u16) -> bool {
        for bar in self.bars() {
            let region = self.bar_region(*bar);
            let offset = self.bar_state(*bar).offset;
            let Some(press) = crate::ui_kit::list_bar_press(region, offset, col, row) else {
                continue;
            };
            self.panes.focus = match bar {
                Bar::List => Pane::List,
                Bar::Files => Pane::Files,
                Bar::Diff => Pane::Diff,
            };
            let state = self.bar_state(*bar);
            if let Some(to) = press.jump_to {
                state.offset = to;
            }
            state.grab = (!press.arrow).then_some(press.grab);
            return true;
        }
        false
    }

    fn drag_bar(&mut self, row: u16) {
        for bar in self.bars() {
            let region = self.bar_region(*bar);
            let state = self.bar_state(*bar);
            if let Some(grab) = state.grab {
                state.offset = crate::ui_kit::list_bar_drag(region, state.offset, grab, row);
            }
        }
    }

    fn release_bars(&mut self) {
        for bar in [Bar::List, Bar::Files, Bar::Diff] {
            self.bar_state(bar).grab = None;
        }
    }
}
