//! Keys, mouse and the palette's commands for the board. Every mouse
//! gesture has its keyboard twin and its palette command: a drag is
//! `Shift+Left/Right` and "Board: Move Card To...", a click is the arrows, a
//! double click is `Enter`, the right click is `.`.

use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use gitpoulpi_i18n::msg::board as msg;
use ratatui::layout::Rect;

use super::draw::Hit;
use super::prompt::PromptOutcome;
use super::{BoardEffect, BoardView, Drag, FilterKind, MenuKind};
use crate::board_cmd::BoardCmd;
use crate::combo::{Combo, ComboOption, ComboOutcome};
use crate::meta_picker::MetaField;
use crate::text_edit::KeyEdit;

const DOUBLE_CLICK: Duration = Duration::from_millis(400);
/// The drag scrolls the columns this often while the pointer rests at an edge.
const EDGE_SCROLL: Duration = Duration::from_millis(180);

impl BoardView {
    // -- commands ---------------------------------------------------------------

    pub(crate) fn run(&mut self, cmd: BoardCmd) -> BoardEffect {
        if let Some(why) = cmd.unavailable(Some(self.avail())) {
            self.message = Some(why.to_string());
            return BoardEffect::None;
        }
        match cmd {
            BoardCmd::MoveTo => self.open_move_combo(),
            BoardCmd::MoveLeft => self.step_move(-1),
            BoardCmd::MoveRight => self.step_move(1),
            BoardCmd::OpenCard => return self.open_card(),
            BoardCmd::EditAssignees => self.open_meta(MetaField::Assignees),
            BoardCmd::EditLabels => self.open_meta(MetaField::Labels),
            BoardCmd::SetMilestone => self.open_meta(MetaField::Milestone),
            BoardCmd::NewCard => {
                return BoardEffect::NewIssue(Box::new(self.target.seed.clone()), self.new_in())
            }
            BoardCmd::FilterAssignee => self.open_filter(FilterKind::Assignee),
            BoardCmd::FilterLabel => self.open_filter(FilterKind::Label),
            BoardCmd::FilterMilestone => self.open_filter(FilterKind::Milestone),
            BoardCmd::FilterText => self.start_search(),
            BoardCmd::ClearFilters => self.clear_filters(),
            BoardCmd::SetColumnLabels => self.open_columns_prompt(),
            BoardCmd::Refresh => {
                self.refresh();
                self.message = Some(msg::REFRESHING.get().to_string());
            }
            BoardCmd::Close => return BoardEffect::Close,
        }
        BoardEffect::None
    }

    fn open_card(&mut self) -> BoardEffect {
        match self.selected_issue().map(|i| i.web_url.clone()) {
            Some(url) if !url.is_empty() => BoardEffect::OpenUrl(url),
            _ => {
                self.message = Some(msg::WHY_NO_CARD.get().to_string());
                BoardEffect::None
            }
        }
    }

    /// `Shift+Left/Right`: the selected card to the next column.
    fn step_move(&mut self, delta: isize) {
        let Some(number) = self.sel_number else {
            self.message = Some(msg::WHY_NO_CARD.get().to_string());
            return;
        };
        let to = self.sel_col as isize + delta;
        if to < 0 || to as usize >= self.columns.len() {
            self.message = Some(msg::WHY_NO_COLUMN.get().to_string());
            return;
        }
        self.move_card(number, to as usize);
    }

    fn anchor(&self) -> Rect {
        let r = self.sel_rect.unwrap_or(Rect::new(4, 4, 24, 1));
        Rect::new(r.x + 1, r.y, r.width.saturating_sub(2).max(16), 1)
    }

    fn open_move_combo(&mut self) {
        let mut combo = Combo::new("").with_click_picks();
        combo.set_options(self.column_options());
        combo.select(Some(self.sel_col.to_string()));
        combo.set_anchor(self.anchor());
        combo.open();
        self.move_combo = Some(combo);
    }

    fn open_filter(&mut self, kind: FilterKind) {
        for other in FilterKind::ALL {
            self.combos[other.index()].close();
        }
        self.combos[kind.index()].open();
    }

    fn start_search(&mut self) {
        self.searching = true;
        self.search_edit.sync(Some(()));
    }

    fn open_menu(&mut self, kind: MenuKind, at: Option<Rect>) {
        if self.selected_issue().is_none() {
            self.message = Some(msg::WHY_NO_CARD.get().to_string());
            return;
        }
        let mut options: Vec<ComboOption> = Vec::new();
        if kind == MenuKind::Card {
            options.push(ComboOption::new(msg::MENU_OPEN.get(), "open"));
        }
        if self.can_edit_meta() {
            options.push(ComboOption::new(
                gitpoulpi_i18n::msg::hosting_ui::EDIT_ASSIGNEES.get(),
                "assignees",
            ));
            options.push(ComboOption::new(
                gitpoulpi_i18n::msg::hosting_ui::EDIT_LABELS.get(),
                "labels",
            ));
            options.push(ComboOption::new(
                gitpoulpi_i18n::msg::hosting_ui::SET_MILESTONE.get(),
                "milestone",
            ));
        } else if kind == MenuKind::Edit {
            self.message = Some(msg::WHY_NO_META.get().to_string());
            return;
        }
        if kind == MenuKind::Card {
            if self.can_move() {
                options.push(ComboOption::new(msg::MENU_MOVE.get(), "move"));
            }
            options.push(ComboOption::new(msg::MENU_COPY.get(), "copy"));
        }
        let mut combo = Combo::new("").with_click_picks();
        combo.set_options(options);
        combo.set_anchor(at.unwrap_or_else(|| self.anchor()));
        combo.open();
        self.menu = Some((kind, combo));
    }

    fn menu_pick(&mut self, value: &str) -> BoardEffect {
        match value {
            "open" => return self.open_card(),
            "assignees" => self.open_meta(MetaField::Assignees),
            "labels" => self.open_meta(MetaField::Labels),
            "milestone" => self.open_meta(MetaField::Milestone),
            "move" => self.open_move_combo(),
            "copy" => {
                if let Some(text) = self.selected_issue().map(|i| i.iid_label.clone()) {
                    let said = crate::clipboard::copy_shared(msg::NOUN_REFERENCE.get(), &text);
                    return BoardEffect::Message(said);
                }
            }
            _ => {}
        }
        BoardEffect::None
    }

    // -- keys --------------------------------------------------------------------

    pub(crate) fn paste(&mut self, text: &str) {
        if let Some(p) = self.prompt.as_mut() {
            p.paste(text);
        } else if self.searching {
            let clean = text.replace(['\r', '\n'], " ");
            self.search_edit.sync(Some(()));
            let mut value = std::mem::take(&mut self.search);
            self.search_edit.caret_mut(()).insert(&mut value, &clean);
            self.search = value;
            self.filter.text = self.search.clone();
            self.rebuild();
        }
    }

    pub(crate) fn on_key(&mut self, key: KeyEvent) -> BoardEffect {
        // Popups own the keyboard, innermost first.
        if let Some(meta) = self.meta.as_mut() {
            let outcome = meta.on_key(key);
            return self.on_meta(outcome).unwrap_or(BoardEffect::None);
        }
        if let Some(p) = self.prompt.as_mut() {
            match p.on_key(key) {
                PromptOutcome::Stay => {}
                PromptOutcome::Cancel => self.prompt = None,
                PromptOutcome::Submit(labels) => {
                    self.prompt = None;
                    self.save_column_labels(labels);
                }
            }
            return BoardEffect::None;
        }
        if let Some(combo) = self.move_combo.as_mut() {
            let outcome = combo.on_key(key);
            if !combo.is_open() {
                self.move_combo = None;
            }
            if let ComboOutcome::Picked(value) = outcome {
                self.picked_column(&value);
            }
            return BoardEffect::None;
        }
        if let Some((_, combo)) = self.menu.as_mut() {
            let outcome = combo.on_key(key);
            if !combo.is_open() {
                self.menu = None;
            }
            if let ComboOutcome::Picked(value) = outcome {
                return self.menu_pick(&value);
            }
            return BoardEffect::None;
        }
        for kind in FilterKind::ALL {
            if self.combos[kind.index()].is_open() {
                let outcome = self.combos[kind.index()].on_key(key);
                if let ComboOutcome::Picked(value) = outcome {
                    self.set_filter(kind, (!value.is_empty()).then_some(value));
                }
                return BoardEffect::None;
            }
        }
        if self.searching {
            return self.search_key(key);
        }
        self.message = None;
        self.navigate(key)
    }

    fn picked_column(&mut self, value: &str) {
        if let (Some(number), Ok(to)) = (self.sel_number, value.parse::<usize>()) {
            self.move_card(number, to);
        }
    }

    fn search_key(&mut self, key: KeyEvent) -> BoardEffect {
        self.search_edit.sync(Some(()));
        let mut value = std::mem::take(&mut self.search);
        let edited = self.search_edit.dialog_key((), &mut value, &key);
        self.search = value;
        if let Some((edit, said)) = edited {
            if matches!(edit, KeyEdit::Edited | KeyEdit::Cut(_)) {
                self.filter.text = self.search.clone();
                self.rebuild();
            }
            if let Some(said) = said {
                self.message = Some(said);
            }
            return BoardEffect::None;
        }
        match key.code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Down | KeyCode::Tab => self.searching = false,
            _ => {}
        }
        BoardEffect::None
    }

    fn navigate(&mut self, key: KeyEvent) -> BoardEffect {
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        let plain = !key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
        match key.code {
            KeyCode::Esc => return BoardEffect::Close,
            KeyCode::Left if shift => return self.run(BoardCmd::MoveLeft),
            KeyCode::Right if shift => return self.run(BoardCmd::MoveRight),
            KeyCode::Left => self.step_column(-1),
            KeyCode::Right => self.step_column(1),
            KeyCode::Up => self.step_card(-1),
            KeyCode::Down => self.step_card(1),
            KeyCode::PageUp => self.step_card(-(self.rows_per_col as isize)),
            KeyCode::PageDown => self.step_card(self.rows_per_col as isize),
            KeyCode::Home => self.step_card(isize::MIN / 2),
            KeyCode::End => self.step_card(isize::MAX / 2),
            KeyCode::Enter => return self.run(BoardCmd::OpenCard),
            KeyCode::Char('e') if plain => self.open_menu(MenuKind::Edit, None),
            KeyCode::Char('.') if plain => self.open_menu(MenuKind::Card, None),
            KeyCode::Char('n') if plain => return self.run(BoardCmd::NewCard),
            KeyCode::Char('m') if plain => return self.run(BoardCmd::MoveTo),
            KeyCode::Char('/') if plain => self.start_search(),
            KeyCode::Char('a') if plain => return self.run(BoardCmd::FilterAssignee),
            KeyCode::Char('l') if plain => return self.run(BoardCmd::FilterLabel),
            KeyCode::Char('M') if plain => return self.run(BoardCmd::FilterMilestone),
            KeyCode::Char('x') if plain => return self.run(BoardCmd::ClearFilters),
            KeyCode::Char('c') if plain => return self.run(BoardCmd::SetColumnLabels),
            KeyCode::Char('r') if plain => return self.run(BoardCmd::Refresh),
            _ => {}
        }
        BoardEffect::None
    }

    /// Up / Down: the card above / below in the column.
    fn step_card(&mut self, delta: isize) {
        let Some(list) = self.cards.get(self.sel_col) else {
            return;
        };
        if list.is_empty() {
            return;
        }
        let at = self.sel_row().unwrap_or(0) as isize;
        let to = at.saturating_add(delta).clamp(0, list.len() as isize - 1);
        self.select(self.sel_col, Some(to as usize));
    }

    /// Left / Right: the neighbouring column, on the card nearest the same
    /// row (an empty column holds no selection).
    fn step_column(&mut self, delta: isize) {
        let to = (self.sel_col as isize + delta).clamp(0, self.columns.len() as isize - 1) as usize;
        if to == self.sel_col {
            return;
        }
        let row = self.sel_row().unwrap_or(0);
        let len = self.cards.get(to).map_or(0, Vec::len);
        self.select(to, (len > 0).then(|| row.min(len - 1)));
    }

    // -- mouse -------------------------------------------------------------------

    fn any_popup(&self) -> bool {
        self.meta.is_some()
            || self.prompt.is_some()
            || self.menu.is_some()
            || self.move_combo.is_some()
            || self.combos.iter().any(Combo::is_open)
    }

    fn hit_at(&self, mouse: MouseEvent) -> Option<Hit> {
        self.hits
            .iter()
            .find(|(r, _)| r.contains((mouse.column, mouse.row).into()))
            .map(|(_, h)| *h)
    }

    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent) -> BoardEffect {
        // Over a popup nothing behind it lights up.
        if self.any_popup() {
            self.hover = None;
        }
        if let Some(meta) = self.meta.as_mut() {
            let outcome = meta.on_mouse(mouse);
            return self.on_meta(outcome).unwrap_or(BoardEffect::None);
        }
        if let Some(p) = self.prompt.as_mut() {
            match p.on_mouse(mouse) {
                PromptOutcome::Stay => {}
                PromptOutcome::Cancel => self.prompt = None,
                PromptOutcome::Submit(labels) => {
                    self.prompt = None;
                    self.save_column_labels(labels);
                }
            }
            return BoardEffect::None;
        }
        if let Some(combo) = self.move_combo.as_mut() {
            let outcome = combo.on_mouse(mouse);
            if !combo.is_open() {
                self.move_combo = None;
            }
            if let ComboOutcome::Picked(value) = outcome {
                self.picked_column(&value);
            }
            return BoardEffect::None;
        }
        if let Some((_, combo)) = self.menu.as_mut() {
            let outcome = combo.on_mouse(mouse);
            if !combo.is_open() {
                self.menu = None;
            }
            if let ComboOutcome::Picked(value) = outcome {
                return self.menu_pick(&value);
            }
            return BoardEffect::None;
        }
        for kind in FilterKind::ALL {
            if self.combos[kind.index()].is_open() {
                let outcome = self.combos[kind.index()].on_mouse(mouse);
                if let ComboOutcome::Picked(value) = outcome {
                    self.set_filter(kind, (!value.is_empty()).then_some(value));
                }
                return BoardEffect::None;
            }
        }
        if self.bar_mouse(mouse) {
            return BoardEffect::None;
        }
        // A press elsewhere ends the text filter's editing.
        let hit = self.hit_at(mouse);
        if self.searching
            && matches!(mouse.kind, MouseEventKind::Down(_))
            && hit != Some(Hit::Search)
        {
            self.searching = false;
        }
        match mouse.kind {
            MouseEventKind::Moved => self.hover = hit,
            MouseEventKind::ScrollUp => self.wheel(hit, mouse, -1),
            MouseEventKind::ScrollDown => self.wheel(hit, mouse, 1),
            MouseEventKind::ScrollLeft => self.scroll_columns(-1),
            MouseEventKind::ScrollRight => self.scroll_columns(1),
            MouseEventKind::Down(MouseButton::Left) => {
                self.message = None;
                if let Some(hit) = hit {
                    return self.click(hit, mouse);
                }
            }
            MouseEventKind::Down(MouseButton::Right) => {
                if let Some(Hit::Card(c, r)) = hit {
                    self.select(c, Some(r));
                    let at = Rect::new(mouse.column, mouse.row, 22, 1);
                    self.open_menu(MenuKind::Card, Some(at));
                }
            }
            MouseEventKind::Drag(MouseButton::Left) => self.drag_to(hit, mouse),
            MouseEventKind::Up(MouseButton::Left) => return self.release(),
            _ => {}
        }
        BoardEffect::None
    }

    fn click(&mut self, hit: Hit, mouse: MouseEvent) -> BoardEffect {
        match hit {
            Hit::Close => return BoardEffect::Close,
            Hit::Refresh => return self.run(BoardCmd::Refresh),
            Hit::New => return self.run(BoardCmd::NewCard),
            Hit::Columns => return self.run(BoardCmd::SetColumnLabels),
            Hit::ClearFilters => self.clear_filters(),
            Hit::Filter(kind) => self.open_filter(kind),
            Hit::Search => {
                self.start_search();
                self.search_edit.dialog_mouse(
                    Some(()),
                    &self.search,
                    mouse.kind,
                    mouse.column,
                    mouse.row,
                );
            }
            Hit::Header(c) | Hit::Column(c) => {
                let len = self.cards.get(c).map_or(0, Vec::len);
                let row = self.offsets.get(c).copied().unwrap_or(0);
                self.select(c, (len > 0).then(|| row.min(len - 1)));
            }
            Hit::Card(c, r) => {
                let now = Instant::now();
                let double = self.last_click.is_some_and(|(t, x, y)| {
                    now.duration_since(t) < DOUBLE_CLICK && x == mouse.column && y == mouse.row
                });
                self.last_click = (!double).then_some((now, mouse.column, mouse.row));
                self.select(c, Some(r));
                if double {
                    self.drag = None;
                    return self.run(BoardCmd::OpenCard);
                }
                if let Some(number) = self.sel_number {
                    self.drag = Some(Drag {
                        number,
                        from: c,
                        start: (mouse.column, mouse.row),
                        moved: false,
                        over: Some(c),
                    });
                }
            }
        }
        BoardEffect::None
    }

    /// The column under `hit`, for the drop highlight.
    fn column_of_hit(hit: Option<Hit>) -> Option<usize> {
        match hit? {
            Hit::Card(c, _) | Hit::Column(c) | Hit::Header(c) => Some(c),
            _ => None,
        }
    }

    fn drag_to(&mut self, hit: Option<Hit>, mouse: MouseEvent) {
        if self.search_edit.is_dragging() {
            self.search_edit.dialog_mouse(
                Some(()),
                &self.search,
                mouse.kind,
                mouse.column,
                mouse.row,
            );
            return;
        }
        let Some(mut drag) = self.drag else {
            return;
        };
        if (mouse.column, mouse.row) != drag.start {
            drag.moved = true;
        }
        if drag.moved {
            drag.over = Self::column_of_hit(hit).or(drag.over);
            // At the left / right edge the columns scroll under the pointer.
            let (left, right) = (self.body_rect.x, self.body_rect.right());
            let due = self.edge_scroll.is_none_or(|t| t.elapsed() >= EDGE_SCROLL);
            if due && right > left {
                if mouse.column <= left && self.first_col > 0 {
                    self.first_col -= 1;
                    self.edge_scroll = Some(Instant::now());
                } else if mouse.column + 1 >= right {
                    let max = self.columns.len().saturating_sub(self.visible_cols);
                    if self.first_col < max {
                        self.first_col += 1;
                        self.edge_scroll = Some(Instant::now());
                    }
                }
            }
        }
        self.drag = Some(drag);
    }

    fn release(&mut self) -> BoardEffect {
        self.hgrab = None;
        self.vgrab = None;
        self.edge_scroll = None;
        self.search_edit.dialog_mouse(
            Some(()),
            &self.search,
            MouseEventKind::Up(MouseButton::Left),
            0,
            0,
        );
        if let Some(drag) = self.drag.take() {
            if drag.moved {
                if let Some(to) = drag.over.filter(|to| *to != drag.from) {
                    self.move_card(drag.number, to);
                }
            }
        }
        BoardEffect::None
    }

    fn wheel(&mut self, hit: Option<Hit>, mouse: MouseEvent, dir: isize) {
        if mouse.modifiers.contains(KeyModifiers::SHIFT) {
            self.scroll_columns(dir);
            return;
        }
        if let Some(c) = Self::column_of_hit(hit) {
            self.scroll_column(c, dir);
        }
    }

    fn scroll_columns(&mut self, dir: isize) {
        let max = self.columns.len().saturating_sub(self.visible_cols);
        self.first_col = (self.first_col as isize + dir).clamp(0, max as isize) as usize;
    }

    /// The horizontal bar's arrows, thumb and track (the shared rules).
    fn bar_mouse(&mut self, mouse: MouseEvent) -> bool {
        if self.col_bar_mouse(mouse) {
            return true;
        }
        let Some(track) = self.hbar else {
            self.hgrab = None;
            return false;
        };
        let n = self.columns.len();
        let at = (mouse.column, mouse.row).into();
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) if track.rect.contains(at) => {
                let pos = mouse.column - track.rect.x;
                match track.press(pos) {
                    crate::scrollbar::TrackPress::Arrow(step) => self.scroll_columns(step),
                    crate::scrollbar::TrackPress::Grab { grab, jump } => {
                        self.hgrab = Some(grab);
                        if jump {
                            self.bar_drag_to(track, n, mouse.column);
                        }
                    }
                }
                true
            }
            MouseEventKind::Drag(MouseButton::Left) if self.hgrab.is_some() => {
                self.bar_drag_to(track, n, mouse.column);
                true
            }
            _ => false,
        }
    }

    /// A column's vertical bar: arrows step one card, the thumb drags, the
    /// bare track jumps the thumb under the pointer. The selection stays.
    fn col_bar_mouse(&mut self, mouse: MouseEvent) -> bool {
        let at = (mouse.column, mouse.row).into();
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let Some((c, track)) = self
                    .col_bars
                    .iter()
                    .find(|(_, t)| t.rect.contains(at))
                    .copied()
                else {
                    return false;
                };
                match track.press(mouse.row - track.rect.y) {
                    crate::scrollbar::TrackPress::Arrow(step) => self.scroll_column(c, step),
                    crate::scrollbar::TrackPress::Grab { grab, jump } => {
                        self.vgrab = Some((c, grab));
                        if jump {
                            self.col_bar_drag_to(c, mouse.row);
                        }
                    }
                }
                true
            }
            MouseEventKind::Drag(MouseButton::Left) if self.vgrab.is_some() => {
                if let Some((c, _)) = self.vgrab {
                    self.col_bar_drag_to(c, mouse.row);
                }
                true
            }
            _ => false,
        }
    }

    fn scroll_column(&mut self, c: usize, dir: isize) {
        let len = self.cards.get(c).map_or(0, Vec::len);
        let max = len.saturating_sub(self.rows_per_col);
        if let Some(off) = self.offsets.get_mut(c) {
            *off = (*off as isize + dir).clamp(0, max as isize) as usize;
        }
    }

    fn col_bar_drag_to(&mut self, c: usize, row: u16) {
        let (Some((_, grab)), Some((_, track))) = (
            self.vgrab,
            self.col_bars.iter().find(|(col, _)| *col == c).copied(),
        ) else {
            return;
        };
        let len = self.cards.get(c).map_or(0, Vec::len);
        let pos = i32::from(row) - i32::from(track.rect.y);
        let off = track.drag_offset(len, self.rows_per_col, grab, pos);
        if let Some(slot) = self.offsets.get_mut(c) {
            *slot = off;
        }
    }

    fn bar_drag_to(&mut self, track: crate::scrollbar::Track, n: usize, column: u16) {
        if let Some(grab) = self.hgrab {
            let pos = i32::from(column) - i32::from(track.rect.x);
            let stride = self.col_stride.max(1) as usize;
            let whole = n * stride;
            let cells = track.drag_offset(whole, self.body_rect.width as usize, grab, pos);
            self.first_col =
                ((cells + stride / 2) / stride).min(n.saturating_sub(self.visible_cols));
        }
    }
}
