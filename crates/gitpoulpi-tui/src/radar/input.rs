//! The Radar's keyboard and mouse: one decision per event, the same
//! [`Act`]s as the menu and the palette.

use super::*;

impl RadarUi {
    pub(crate) fn typing(&self) -> bool {
        self.filter_focused
    }

    pub(crate) fn animating(&self) -> bool {
        self.in_flight || self.api_busy()
    }

    /// When the dwell on the checks glyphs ends, if it is still running.
    pub(crate) fn tooltip_due(&self) -> Option<Instant> {
        if self.popup_open() || !matches!(self.hover, Some(Hit::Status(_) | Hit::Cut(..))) {
            return None;
        }
        let due = self.hover_since? + crate::tooltip::DWELL;
        (due > Instant::now()).then_some(due)
    }

    /// The palette's twin of every key and click.
    pub(crate) fn command(&mut self, command: RadarCommand) -> Outcome {
        // The hosted review owns the window: the Radar's own commands wait
        // for it to close (Back closes it).
        if self.api_review.is_some() {
            return match command {
                RadarCommand::Open | RadarCommand::CloseTab => Outcome::None,
                RadarCommand::Back => {
                    self.api_review = None;
                    Outcome::None
                }
                _ => self.say(msg::radar::MSG_CLOSE_REVIEW_FIRST.get().to_string()),
            };
        }
        match command {
            RadarCommand::Open => Outcome::None,
            RadarCommand::Refresh => {
                self.reload();
                self.say(msg::hosting_ui::REFRESHING.get().to_string())
            }
            RadarCommand::Search => {
                self.close_popups();
                self.filter_focused = true;
                Outcome::None
            }
            RadarCommand::Act(act) => self.run(act),
            RadarCommand::Scope => {
                self.open_combo(Hit::Scope);
                Outcome::None
            }
            RadarCommand::View => {
                self.open_combo(Hit::View);
                Outcome::None
            }
            RadarCommand::Filter => {
                self.open_combo(Hit::Who);
                Outcome::None
            }
            RadarCommand::NotCloned => self.toggle_not_cloned(),
            RadarCommand::Drafts => self.toggle_drafts(),
            RadarCommand::Menu => {
                let (col, row) = self.selected_anchor();
                self.open_menu(col, row);
                Outcome::None
            }
            RadarCommand::Back => Outcome::Close,
            RadarCommand::Column(cmd) => self.column_command(cmd),
            // Handled by the event loop (it owns the session); a no-op here.
            RadarCommand::CloseTab => Outcome::None,
        }
    }

    /// What `Ctrl+N` badges, in the order the badges are handed out: the
    /// filter, the three comboboxes, the two buttons, then the section
    /// headings on screen (the one part whose count changes). The rects are
    /// the last paint's own hit rects.
    pub(crate) fn nav_targets(&self) -> Vec<(Rect, Spot)> {
        // The hosted review is badged by nobody (like the other review
        // pages); the choice dialog badges its own controls only.
        if self.api_review.is_some() {
            return Vec::new();
        }
        if let Some(asking) = &self.asking {
            return asking
                .dialog
                .nav_targets()
                .into_iter()
                .map(|(rect, i)| (rect, Spot::Choice(i)))
                .collect();
        }
        let spot_of = |hit: Hit| match hit {
            Hit::Filter => Some(Spot::Filter),
            Hit::Scope => Some(Spot::Scope),
            Hit::View => Some(Spot::View),
            Hit::Who => Some(Spot::Who),
            Hit::NotCloned => Some(Spot::NotCloned),
            Hit::Drafts => Some(Spot::Drafts),
            Hit::TurnOn(_) => Some(Spot::TurnOn),
            Hit::Refresh => Some(Spot::Refresh),
            Hit::Back => Some(Spot::Back),
            Hit::Heading(i) => Some(Spot::Heading(i)),
            _ => None,
        };
        let mut out: Vec<(Rect, Spot)> = Vec::new();
        for want in [
            Spot::Filter,
            Spot::Scope,
            Spot::View,
            Spot::Who,
            Spot::Refresh,
            Spot::Back,
            Spot::NotCloned,
            Spot::Drafts,
        ] {
            if let Some((rect, _)) = self
                .hits
                .iter()
                .find(|(_, hit)| spot_of(*hit) == Some(want))
            {
                // A button's badge starts on the gap before it, so it hides
                // fewer letters of the label.
                let rect = if matches!(want, Spot::Refresh | Spot::Back) {
                    Rect::new(rect.x.saturating_sub(1), rect.y, rect.width + 1, 1)
                } else {
                    *rect
                };
                out.push((rect, want));
            }
        }
        for (rect, hit) in &self.hits {
            if let Hit::Heading(i) = hit {
                out.push((*rect, Spot::Heading(*i)));
            }
        }
        // After the headings, so that the hint row (which comes and goes)
        // never renumbers the badges before it. A button's badge starts on
        // the gap before it, like Refresh and Back.
        if let Some((rect, _)) = self
            .hits
            .iter()
            .find(|(_, hit)| matches!(hit, Hit::TurnOn(_)))
        {
            out.push((
                Rect::new(rect.x.saturating_sub(1), rect.y, rect.width + 1, 1),
                Spot::TurnOn,
            ));
        }
        out
    }

    /// A pressed badge. Text and list parts take the focus; a combobox
    /// opens; a button runs (the same as its key and its click).
    pub(crate) fn nav_focus(&mut self, spot: Spot) -> Outcome {
        self.message = None;
        match spot {
            Spot::Choice(i) => self.choice_press(i),
            Spot::Filter => self.command(RadarCommand::Search),
            Spot::Scope => self.command(RadarCommand::Scope),
            Spot::View => self.command(RadarCommand::View),
            Spot::Who => self.command(RadarCommand::Filter),
            Spot::NotCloned => self.command(RadarCommand::NotCloned),
            Spot::Drafts => self.command(RadarCommand::Drafts),
            Spot::TurnOn => self.run(Act::TurnOn),
            Spot::Refresh => self.command(RadarCommand::Refresh),
            Spot::Back => self.command(RadarCommand::Back),
            // Lands the selection on the heading, like the Home tab's section
            // badges; folding stays on `Space`/`Enter`/a click.
            Spot::Heading(i) => {
                self.close_popups();
                self.filter_focused = false;
                self.selected = i;
                self.follow = true;
                Outcome::None
            }
        }
    }

    /// Where a keyboard-opened menu hangs: under the selected row.
    fn selected_anchor(&self) -> (u16, u16) {
        self.hits
            .iter()
            .find(|(_, h)| *h == Hit::Row(self.selected))
            .map(|(r, _)| (r.x + 6, r.y + 1))
            .unwrap_or((6, 6))
    }

    pub(crate) fn on_key(&mut self, key: KeyEvent) -> Outcome {
        self.message = None;
        self.hover_since = None;
        if self.api_review.is_some() {
            return self.api_key(key);
        }
        if self.asking.is_some() {
            return self.choice_key(key);
        }
        // A combobox popup is modal.
        for which in [Hit::Scope, Hit::View, Hit::Who, Hit::Snooze] {
            if self.combo_of(which).is_open() {
                return match self.combo_of(which).on_key(key) {
                    ComboOutcome::Picked(value) => self.picked(which, &value),
                    _ => {
                        if !self.combo_of(which).is_open() {
                            self.snooze_for = None;
                        }
                        Outcome::None
                    }
                };
            }
        }
        if let Some(menu) = self.menu.as_mut() {
            return match menu.on_key(key.code) {
                MenuOutcome::Run(act) => {
                    self.menu = None;
                    self.run(act)
                }
                MenuOutcome::Close => {
                    self.menu = None;
                    Outcome::None
                }
                MenuOutcome::Stay | MenuOutcome::NotHandled => Outcome::None,
            };
        }
        if self.filter_focused {
            return self.filter_key(key);
        }
        // A picked column takes its own keys first.
        if self.col_pick.is_some() {
            if let Some(outcome) = self.column_key(key) {
                return outcome;
            }
        }
        if crate::context_menu::is_menu_key(key.code, key.modifiers) {
            return self.command(RadarCommand::Menu);
        }
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        if ctrl {
            return match key.code {
                KeyCode::Char('f') | KeyCode::Char('F') => self.command(RadarCommand::Search),
                _ => Outcome::None,
            };
        }
        match key.code {
            KeyCode::Esc => Outcome::Close,
            KeyCode::Up | KeyCode::Char('k') => self.step(-1),
            KeyCode::Down | KeyCode::Char('j') => self.step(1),
            KeyCode::PageUp => self.step(-(self.list_height as isize)),
            KeyCode::PageDown => self.step(self.list_height as isize),
            KeyCode::Home => self.step(isize::MIN / 2),
            KeyCode::End => self.step(isize::MAX / 2),
            KeyCode::Enter => self.activate(),
            KeyCode::Char(' ') => self.toggle_fold(),
            KeyCode::Char('o') => self.run(Act::OpenBrowser),
            KeyCode::Char('a') => self.command(RadarCommand::NotCloned),
            KeyCode::Char('d') => self.command(RadarCommand::Drafts),
            KeyCode::Char('T') => self.run(Act::TurnOn),
            KeyCode::Char('C') => self.command(RadarCommand::Column(ColumnCmd::Pick)),
            KeyCode::Char('c') => self.run(Act::Checkout),
            KeyCode::Char('e') if crate::build_features::REVIEW => self.run(Act::Review),
            KeyCode::Char('t') => self.run(Act::Worktree),
            KeyCode::Char('s') => self.run(Act::Show),
            KeyCode::Char('n') => self.run(Act::StartWork),
            KeyCode::Char('p') => self.run(Act::Pin),
            KeyCode::Char('z') => self.run(Act::Snooze),
            KeyCode::Char('u') => self.run(Act::Unsnooze),
            KeyCode::Char('r') => self.command(RadarCommand::Refresh),
            KeyCode::Char('w') => self.command(RadarCommand::Scope),
            KeyCode::Char('v') => self.command(RadarCommand::View),
            KeyCode::Char('f') => self.command(RadarCommand::Filter),
            KeyCode::Char('/') => self.command(RadarCommand::Search),
            KeyCode::Char('q') => Outcome::Quit,
            _ => Outcome::None,
        }
    }

    fn filter_key(&mut self, key: KeyEvent) -> Outcome {
        self.edit.sync(Some(()));
        if self.edit.claims((), &self.query, &key, false) {
            let edit = self.edit.on_key((), &mut self.query, &key, false);
            return match edit {
                crate::text_edit::KeyEdit::Edited => {
                    self.selected = 0;
                    self.relayout();
                    Outcome::None
                }
                crate::text_edit::KeyEdit::Copy(text) | crate::text_edit::KeyEdit::Cut(text) => {
                    Outcome::CopyText(text)
                }
                _ => Outcome::None,
            };
        }
        match key.code {
            // Enter keeps the filter and gives the keyboard back to the list.
            KeyCode::Enter | KeyCode::Down => self.filter_focused = false,
            KeyCode::Esc => {
                self.filter_focused = false;
                if !self.query.is_empty() {
                    self.query.clear();
                    self.selected = 0;
                    self.relayout();
                }
            }
            _ => {}
        }
        self.edit.sync(self.filter_focused.then_some(()));
        Outcome::None
    }

    fn step(&mut self, delta: isize) -> Outcome {
        let last = self.rows.len().saturating_sub(1) as isize;
        self.selected = (self.selected as isize)
            .saturating_add(delta)
            .clamp(0, last) as usize;
        // The blank spacer line is never a stop: slide past it in the
        // direction of travel (back when it is the end of the list).
        if matches!(self.rows.get(self.selected), Some(Row::Blank)) {
            let forward = if delta >= 0 { 1 } else { -1 };
            let next = self.selected as isize + forward;
            self.selected = if (0..=last).contains(&next) {
                next as usize
            } else {
                (self.selected as isize - forward) as usize
            };
        }
        self.follow = true;
        Outcome::None
    }

    /// Enter: the repository for an item, the fold for a heading, the
    /// note's own action for a note.
    fn activate(&mut self) -> Outcome {
        match self.rows.get(self.selected) {
            Some(Row::Heading(..)) => self.toggle_fold(),
            Some(Row::Note(_)) => self.run(Act::Link),
            Some(Row::Hint(_)) => self.run(Act::TurnOn),
            Some(Row::Item(_)) => self.run(Act::OpenRepo),
            _ => Outcome::None,
        }
    }

    fn toggle_fold(&mut self) -> Outcome {
        if let Some(Row::Heading(heading, _)) = self.rows.get(self.selected).cloned() {
            if !self.folded.remove(&heading) {
                self.folded.insert(heading);
            }
            self.relayout();
            if let Some(i) = self
                .rows
                .iter()
                .position(|r| matches!(r, Row::Heading(h, _) if *h == heading))
            {
                self.selected = i;
            }
        }
        Outcome::None
    }

    // -- mouse --------------------------------------------------------------

    /// The hit under a point, when nothing popped up covers it.
    fn hit_at(&self, col: u16, row: u16) -> Option<Hit> {
        self.hits
            .iter()
            .find(|(r, _)| r.contains((col, row).into()))
            .map(|(_, h)| *h)
    }

    /// Whether the pointer is on a column separator or sizing one.
    pub(crate) fn resize_pointer(&self) -> bool {
        self.col_drag.is_some() || matches!(self.hover, Some(Hit::Divider(_)))
    }

    /// Whether a click at the point does something: the pointing hand. A
    /// popup answers for itself (its rows), otherwise the hovered control or
    /// row. The filter box is a text field, the separators have their own
    /// resize shape and the item rows (and their cut cells and status glyphs)
    /// keep the arrow, lit as a whole: none of them takes it.
    pub(crate) fn pointer_hand_at(&self, col: u16, row: u16) -> bool {
        let at = ratatui::layout::Position::new(col, row);
        if let Some(asking) = &self.asking {
            return asking
                .dialog
                .nav_targets()
                .iter()
                .any(|(r, _)| r.contains(at));
        }
        if self.any_combo_open() {
            return [
                &self.scope_combo,
                &self.view_combo,
                &self.who_combo,
                &self.snooze_combo,
            ]
            .iter()
            .any(|c| c.pointer_over(col, row));
        }
        if self.menu.is_some() {
            return self.menu_rects.iter().any(|r| r.contains(at));
        }
        !matches!(
            self.hover,
            None | Some(
                Hit::Filter | Hit::Divider(_) | Hit::Cut(..) | Hit::Row(_) | Hit::Status(_)
            )
        )
    }

    /// The pointer left the body (the tab bar, the status bar): nothing is
    /// hovered any more. The Radar only hears moves inside its body, so
    /// without this the last hover (a lit separator) stayed.
    pub(crate) fn pointer_left(&mut self) {
        self.hover = None;
        self.hover_since = None;
    }

    /// Whether a separator is being dragged (the button is down).
    pub(crate) fn sizing_column(&self) -> bool {
        self.col_drag.is_some()
    }

    /// Whether the pointer is on the filter box (a text field: the I-beam).
    pub(crate) fn pointer_text_at(&self) -> bool {
        !self.popup_open() && matches!(self.hover, Some(Hit::Filter))
    }

    /// Whether a popup (a combobox list, the row menu) owns the pointer.
    pub(crate) fn popup_open(&self) -> bool {
        self.any_combo_open() || self.menu.is_some() || self.asking.is_some()
    }

    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent, double: bool) -> Outcome {
        let (col, row) = (mouse.column, mouse.row);
        if self.api_review.is_some() {
            return self.api_mouse(mouse);
        }
        if self.asking.is_some() {
            self.hover = None;
            return self.choice_mouse(mouse);
        }
        // A separator drag owns the pointer until the button comes up.
        if self.col_drag.is_some() {
            match mouse.kind {
                MouseEventKind::Drag(MouseButton::Left) => {
                    self.divider_drag(col);
                    return Outcome::None;
                }
                MouseEventKind::Up(MouseButton::Left) => {
                    self.divider_release();
                    // A release brings no `Moved`: the hover (the line, the
                    // pointer) is re-read for where the pointer ended.
                    self.hover = self.hit_at(col, row);
                    self.hover_pos = (col, row);
                    return Outcome::None;
                }
                _ => {}
            }
        }
        // Popups own the pointer: first, and nothing behind one hovers.
        for which in [Hit::Scope, Hit::View, Hit::Who, Hit::Snooze] {
            if self.combo_of(which).is_open() {
                self.hover = None;
                return match self.combo_of(which).on_mouse(mouse) {
                    ComboOutcome::Picked(value) => self.picked(which, &value),
                    _ => {
                        if !self.combo_of(which).is_open() {
                            self.snooze_for = None;
                        }
                        Outcome::None
                    }
                };
            }
        }
        if self.menu.is_some() {
            return self.menu_mouse(mouse);
        }
        // Moved: the hover. Clicking a closed combobox's field opens it.
        for which in [Hit::Scope, Hit::View, Hit::Who] {
            if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left))
                && self.hit_at(col, row) == Some(which)
            {
                self.open_combo(which);
                return Outcome::None;
            }
        }
        let hit = self.hit_at(col, row);
        // Anything but a move hides a tooltip; a click on the glyphs is a click
        // on the row.
        let hit = if matches!(mouse.kind, MouseEventKind::Moved) {
            hit
        } else {
            self.hover_since = None;
            match hit {
                Some(Hit::Status(i) | Hit::Cut(i, _)) => Some(Hit::Row(i)),
                other => other,
            }
        };
        match mouse.kind {
            MouseEventKind::Moved => {
                if hit != self.hover {
                    self.hover_since =
                        matches!(hit, Some(Hit::Status(_) | Hit::Cut(..))).then(Instant::now);
                }
                self.hover = hit;
                self.hover_pos = (col, row);
                Outcome::None
            }
            MouseEventKind::ScrollDown => self.scroll_by(3),
            MouseEventKind::ScrollUp => self.scroll_by(-3),
            MouseEventKind::Down(MouseButton::Left) => self.click(hit, double, mouse),
            MouseEventKind::Down(MouseButton::Right) => {
                if let Some(
                    Hit::Row(i) | Hit::Star(i) | Hit::Menu(i) | Hit::NoteAction(i) | Hit::TurnOn(i),
                ) = hit
                {
                    self.selected = i;
                    self.open_menu(col, row + 1);
                }
                Outcome::None
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                self.edit.mouse_drag(&self.query, col, row);
                Outcome::None
            }
            MouseEventKind::Up(MouseButton::Left) => {
                self.edit.mouse_up();
                Outcome::None
            }
            _ => Outcome::None,
        }
    }

    fn scroll_by(&mut self, delta: isize) -> Outcome {
        let max = self.rows.len().saturating_sub(self.list_height.max(1));
        self.scroll = (self.scroll as isize + delta).clamp(0, max as isize) as usize;
        self.follow = false;
        Outcome::None
    }

    fn click(&mut self, hit: Option<Hit>, double: bool, mouse: MouseEvent) -> Outcome {
        self.message = None;
        if hit != Some(Hit::Filter) {
            self.filter_focused = false;
        }
        match hit {
            Some(Hit::Back) => Outcome::Close,
            Some(Hit::Refresh) => self.command(RadarCommand::Refresh),
            Some(Hit::Filter) => {
                self.filter_focused = true;
                self.edit.sync(Some(()));
                self.edit.mouse_down(
                    (),
                    &self.query,
                    mouse.column,
                    mouse.row,
                    std::time::Instant::now(),
                );
                Outcome::None
            }
            Some(Hit::Heading(i)) => {
                self.selected = i;
                self.toggle_fold()
            }
            Some(Hit::Star(i)) => {
                self.selected = i;
                self.run(Act::Pin)
            }
            Some(Hit::Menu(i)) => {
                self.selected = i;
                self.open_menu(mouse.column, mouse.row + 1);
                Outcome::None
            }
            Some(Hit::NoteAction(i)) => {
                self.selected = i;
                self.run(Act::Link)
            }
            Some(Hit::TurnOn(i)) => {
                self.selected = i;
                self.run(Act::TurnOn)
            }
            Some(Hit::Divider(cell)) => self.divider_press(cell, mouse.column, double),
            Some(Hit::NotCloned) => self.command(RadarCommand::NotCloned),
            Some(Hit::Drafts) => self.command(RadarCommand::Drafts),
            Some(Hit::Row(i)) => {
                self.selected = i;
                self.follow = false;
                if double {
                    self.activate()
                } else {
                    Outcome::None
                }
            }
            _ => Outcome::None,
        }
    }

    fn menu_mouse(&mut self, mouse: MouseEvent) -> Outcome {
        let at = (mouse.column, mouse.row).into();
        let row = self.menu_rects.iter().position(|r| r.contains(at));
        match mouse.kind {
            MouseEventKind::Moved => {
                self.menu_hover = row;
                Outcome::None
            }
            MouseEventKind::Down(MouseButton::Left) => {
                match (row, self.menu.as_mut()) {
                    (Some(r), Some(menu)) => match menu.activate(r) {
                        MenuOutcome::Run(act) => {
                            self.menu = None;
                            return self.run(act);
                        }
                        MenuOutcome::Close => self.menu = None,
                        _ => {}
                    },
                    // Outside the menu: close it, and the click goes no
                    // further.
                    _ => self.menu = None,
                }
                Outcome::None
            }
            MouseEventKind::Down(MouseButton::Right) => {
                self.menu = None;
                Outcome::None
            }
            _ => Outcome::None,
        }
    }
}
