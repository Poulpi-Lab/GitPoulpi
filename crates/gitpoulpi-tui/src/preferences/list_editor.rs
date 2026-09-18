//! The master half of Settings' master-detail pages: a bordered list of items
//! with a toolbar under it (`+ Add`, `Duplicate`, `Remove`, up, down).
//!
//! Pure state plus render: it knows nothing of the config. The page hands it a
//! [`ListSpec`] (`set_spec`), feeds it keys and mouse events and applies the
//! [`ListOp`] it answers with to its own vector, then calls `set_spec` again
//! (and `select` when the selection must follow, after an add or a move).
//!
//! Every mouse gesture has a keyboard twin: click = `Up`/`Down`, double click
//! = `Enter`, the toolbar buttons = `Insert` / `Ctrl+D` / `Delete` /
//! `Alt+Up` / `Alt+Down` (bindable actions of `Ctx::Preferences`), drag to
//! reorder = `Alt+Up` / `Alt+Down`, wheel = the scrolling keys. The add
//! dropdown and the remove confirmation own the pointer while open. Hit rects
//! are recorded by the walk that paints them.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders, Widget};

use crate::combo::{Combo, ComboOption, ComboOutcome};
use crate::dialog_kit::{self, ButtonState};
use crate::keymap::Action;
use crate::popup_list::{self, ClickPair, WHEEL_STEP};
use crate::scrollbar::{self, Track, TrackPress};
use crate::{display_width, th};
use gitpoulpi_i18n::msg::list_editor as t;

/// What a row's gutter says about its item.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) enum Mark {
    #[default]
    None,
    /// The page's default entry.
    Default,
    /// The page's active entry.
    Active,
    /// The item is not usable; the reason replaces its `sub` text.
    Problem(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct ListItem {
    pub(crate) label: String,
    /// Dim text at the row's right edge.
    pub(crate) sub: Option<String>,
    pub(crate) mark: Mark,
    /// Added in this session and never edited: removing it asks nothing.
    pub(crate) pristine: bool,
    /// The group header this item sits under (a header row is painted where
    /// the group changes). Moves never cross groups. `None` everywhere = a
    /// plain list.
    pub(crate) group: Option<String>,
    /// Always there: it cannot be removed (the Workflow page's "General").
    pub(crate) fixed: bool,
}

/// One painted row of the list: a group header or an item.
#[derive(Debug, Clone, PartialEq, Eq)]
enum DisplayRow {
    Header(String),
    Item(usize),
}

impl ListItem {
    #[allow(dead_code)] // for the AI/Workflow lists (M4, M5)
    pub(crate) fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) enum AddKind {
    /// `+ Add` adds one blank item.
    #[default]
    Plain,
    /// `+ Add ▾` opens a dropdown of these presets; the pick's index goes in
    /// [`ListOp::Add`].
    Menu(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct ListSpec {
    pub(crate) title: String,
    pub(crate) items: Vec<ListItem>,
    pub(crate) add: AddKind,
    /// Offer the Duplicate button and key.
    pub(crate) duplicate: bool,
    /// Offer the up/down buttons, `Alt+Up`/`Alt+Down` and drag to reorder.
    pub(crate) reorder: bool,
    /// Remove is disabled at or below this many items.
    pub(crate) min_len: usize,
}

/// What the page must do to its vector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ListOp {
    /// The selection moved (already applied to the editor).
    Select(usize),
    /// Add an item: `None` for a plain add, else the menu index picked.
    Add(Option<usize>),
    Duplicate(usize),
    /// Remove the item (confirmed already, or pristine).
    Remove(usize),
    /// Move the item at `from` so it ends up at `to`; the page calls
    /// `select(to)` after applying it.
    Move {
        from: usize,
        to: usize,
    },
    /// Enter, Right or a double click: focus the detail's first field.
    FocusDetail,
    /// Left: back to the page list.
    Back,
}

/// What a key or mouse event did to the editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ListEvent {
    /// Not the editor's: the page handles it.
    Ignored,
    /// Consumed, nothing for the page to do.
    Handled,
    Op(ListOp),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ListHit {
    Row(usize),
    Add,
    Duplicate,
    Remove,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConfirmHit {
    Cancel,
    Remove,
}

/// A drag to reorder in progress.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Drag {
    pub(crate) from: usize,
    /// The index the item would end up at when released.
    pub(crate) over: usize,
    /// The pointer left the row it was pressed on.
    pub(crate) started: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct ListEditor {
    pub(crate) spec: ListSpec,
    pub(crate) selected: usize,
    pub(crate) scroll: usize,
    pub(crate) hover: Option<ListHit>,
    pub(crate) drag: Option<Drag>,
    add_combo: Option<Combo>,
    pub(crate) confirm_remove: Option<usize>,
    /// The remove confirmation (`dialog_kit::Confirm`), while `confirm_remove` is set.
    confirm: Option<dialog_kit::Confirm<ConfirmHit>>,
    confirm_hits: Vec<(Rect, ConfirmHit)>,
    /// Painted last frame: toolbar buttons then rows.
    hits: Vec<(Rect, ListHit)>,
    /// The whole editor (list and toolbar).
    area: Rect,
    /// The list's rows (inside its border).
    inner: Rect,
    add_rect: Rect,
    view_rows: usize,
    track: Option<Track>,
    bar_grab: Option<i32>,
    clicks: ClickPair,
}

/// A toolbar button before it is placed.
struct Btn {
    hit: ListHit,
    label: String,
    enabled: bool,
}

impl ListEditor {
    pub(crate) fn new(spec: ListSpec) -> Self {
        let mut editor = Self {
            spec: ListSpec::default(),
            selected: 0,
            scroll: 0,
            hover: None,
            drag: None,
            add_combo: None,
            confirm_remove: None,
            confirm: None,
            confirm_hits: Vec::new(),
            hits: Vec::new(),
            area: Rect::default(),
            inner: Rect::default(),
            add_rect: Rect::default(),
            view_rows: 8,
            track: None,
            bar_grab: None,
            clicks: ClickPair::default(),
        };
        editor.set_spec(spec);
        editor
    }

    /// The page's current items; keeps the selection in range and drops a
    /// confirmation or drag that no longer points at an item.
    pub(crate) fn set_spec(&mut self, spec: ListSpec) {
        self.spec = spec;
        let len = self.spec.items.len();
        self.selected = self.selected.min(len.saturating_sub(1));
        self.scroll = self
            .scroll
            .min(self.display_len().saturating_sub(self.view_rows));
        if self.confirm_remove.is_some_and(|i| i >= len) {
            self.confirm_remove = None;
        }
        if self.drag.is_some_and(|d| d.from >= len || d.over >= len) {
            self.drag = None;
        }
    }

    /// Selects `index` (clamped) and scrolls it into view, without an op.
    pub(crate) fn select(&mut self, index: usize) {
        self.selected = index.min(self.spec.items.len().saturating_sub(1));
        self.keep_visible();
    }

    /// Whether the add dropdown or the confirmation is up: the page then
    /// gives this editor the pointer first.
    pub(crate) fn owns_pointer(&self) -> bool {
        self.confirm_remove.is_some() || self.add_combo.as_ref().is_some_and(Combo::is_open)
    }

    /// Whether an item is being dragged (an `Esc` then cancels the drag only).
    #[cfg(test)]
    pub(crate) fn is_dragging(&self) -> bool {
        self.drag.is_some()
    }

    /// What `Ctrl+N` badges: the selected row (the list itself), then the
    /// enabled buttons, from the rects painted last frame.
    pub(crate) fn nav_hits(&self) -> Vec<(Rect, ListHit)> {
        let mut out: Vec<(Rect, ListHit)> = self
            .hits
            .iter()
            .find(|(_, h)| *h == ListHit::Row(self.selected))
            .or_else(|| self.hits.iter().find(|(_, h)| matches!(h, ListHit::Row(_))))
            .copied()
            .into_iter()
            .collect();
        out.extend(
            self.hits
                .iter()
                .filter(|(_, h)| !matches!(h, ListHit::Row(_)) && self.enabled(*h))
                .copied(),
        );
        out
    }

    /// The rects painted last frame, buttons then rows. The popups' rects come
    /// from [`Self::popup_hits`] and go first in the page's list.
    #[cfg(test)]
    pub(crate) fn hits(&self) -> &[(Rect, ListHit)] {
        &self.hits
    }

    /// The confirmation's buttons and the add dropdown's rows, painted last
    /// frame: they go before every other hit rect.
    #[cfg(test)]
    pub(crate) fn popup_hits(&self) -> Vec<Rect> {
        let mut rects: Vec<Rect> = self.confirm_hits.iter().map(|(r, _)| *r).collect();
        if let Some(combo) = self.add_combo.as_ref().filter(|c| c.is_open()) {
            rects.extend(combo.item_rects().into_iter().map(|(r, _)| r));
        }
        rects
    }

    // ---- what can be done ----

    fn len(&self) -> usize {
        self.spec.items.len()
    }

    /// The painted rows: the items with a header where the group changes.
    fn display_rows(&self) -> Vec<DisplayRow> {
        let mut rows = Vec::with_capacity(self.len() + 4);
        let mut previous: Option<&String> = None;
        for (i, item) in self.spec.items.iter().enumerate() {
            if let Some(group) = item.group.as_ref().filter(|g| Some(*g) != previous) {
                rows.push(DisplayRow::Header(group.clone()));
            }
            previous = item.group.as_ref();
            rows.push(DisplayRow::Item(i));
        }
        rows
    }

    fn display_len(&self) -> usize {
        self.display_rows().len()
    }

    /// The painted row of item `i`, and the first row of its block (its
    /// header when it opens a group).
    fn display_span(&self, i: usize) -> (usize, usize) {
        let rows = self.display_rows();
        let at = rows
            .iter()
            .position(|r| *r == DisplayRow::Item(i))
            .unwrap_or(0);
        let top = if at > 0 && matches!(rows[at - 1], DisplayRow::Header(_)) {
            at - 1
        } else {
            at
        };
        (at, top)
    }

    fn same_group(&self, a: usize, b: usize) -> bool {
        self.spec.items.get(a).map(|x| &x.group) == self.spec.items.get(b).map(|x| &x.group)
    }

    fn has_selection(&self) -> bool {
        self.selected < self.len()
    }

    pub(crate) fn can_duplicate(&self) -> bool {
        self.spec.duplicate && self.has_selection()
    }

    pub(crate) fn can_remove(&self) -> bool {
        self.has_selection()
            && self.len() > self.spec.min_len
            && !self.spec.items[self.selected].fixed
    }

    pub(crate) fn can_move_up(&self) -> bool {
        self.spec.reorder
            && self.has_selection()
            && self.selected > 0
            && self.same_group(self.selected, self.selected - 1)
    }

    pub(crate) fn can_move_down(&self) -> bool {
        self.spec.reorder
            && self.has_selection()
            && self.selected + 1 < self.len()
            && self.same_group(self.selected, self.selected + 1)
    }

    fn enabled(&self, hit: ListHit) -> bool {
        match hit {
            ListHit::Add | ListHit::Row(_) => true,
            ListHit::Duplicate => self.can_duplicate(),
            ListHit::Remove => self.can_remove(),
            ListHit::Up => self.can_move_up(),
            ListHit::Down => self.can_move_down(),
        }
    }

    fn keep_visible(&mut self) {
        let rows = self.view_rows.max(1);
        let (at, top) = self.display_span(self.selected);
        if top < self.scroll {
            self.scroll = top;
        } else if at >= self.scroll + rows {
            self.scroll = at + 1 - rows;
        }
    }

    fn scroll_by(&mut self, delta: isize) {
        let max = self.display_len().saturating_sub(self.view_rows);
        self.scroll = (self.scroll as isize + delta).clamp(0, max as isize) as usize;
    }

    /// Selects `index`; the op to report when it changed.
    fn select_op(&mut self, index: usize) -> Option<ListOp> {
        let index = index.min(self.len().saturating_sub(1));
        if self.len() == 0 || index == self.selected {
            self.keep_visible();
            return None;
        }
        self.selected = index;
        self.keep_visible();
        Some(ListOp::Select(index))
    }

    fn op(op: Option<ListOp>) -> ListEvent {
        op.map_or(ListEvent::Handled, ListEvent::Op)
    }

    // ---- actions shared by keys and buttons ----

    fn do_add(&mut self) -> ListEvent {
        match &self.spec.add {
            AddKind::Plain => ListEvent::Op(ListOp::Add(None)),
            AddKind::Menu(presets) => {
                let mut combo = Combo::new("").with_click_picks();
                combo.set_options(
                    presets
                        .iter()
                        .enumerate()
                        .map(|(i, label)| ComboOption::new(label.clone(), i.to_string()))
                        .collect(),
                );
                combo.set_anchor(self.add_rect);
                combo.open();
                self.add_combo = Some(combo);
                ListEvent::Handled
            }
        }
    }

    fn do_duplicate(&mut self) -> ListEvent {
        if self.can_duplicate() {
            ListEvent::Op(ListOp::Duplicate(self.selected))
        } else {
            ListEvent::Handled
        }
    }

    /// Asks first, unless the item is pristine.
    fn do_remove(&mut self) -> ListEvent {
        if !self.can_remove() {
            return ListEvent::Handled;
        }
        if self.spec.items[self.selected].pristine {
            return ListEvent::Op(ListOp::Remove(self.selected));
        }
        self.confirm_remove = Some(self.selected);
        self.confirm = Some(
            dialog_kit::Confirm::new(
                t::REMOVE_TITLE.get(),
                t::remove_prompt(self.spec.items[self.selected].label.clone()),
                vec![
                    (t::CANCEL.get().to_string(), ConfirmHit::Cancel),
                    (t::REMOVE.get().to_string(), ConfirmHit::Remove),
                ],
                ConfirmHit::Cancel,
            )
            .primary(ConfirmHit::Remove),
        );
        ListEvent::Handled
    }

    fn do_move(&mut self, up: bool) -> ListEvent {
        let ok = if up {
            self.can_move_up()
        } else {
            self.can_move_down()
        };
        if !ok {
            return ListEvent::Handled;
        }
        let from = self.selected;
        let to = if up { from - 1 } else { from + 1 };
        ListEvent::Op(ListOp::Move { from, to })
    }

    fn press_button(&mut self, hit: ListHit) -> ListEvent {
        if !self.enabled(hit) {
            return ListEvent::Handled;
        }
        match hit {
            ListHit::Add => self.do_add(),
            ListHit::Duplicate => self.do_duplicate(),
            ListHit::Remove => self.do_remove(),
            ListHit::Up => self.do_move(true),
            ListHit::Down => self.do_move(false),
            ListHit::Row(_) => ListEvent::Handled,
        }
    }

    // ---- keys ----

    pub(crate) fn on_key(&mut self, key: KeyEvent) -> ListEvent {
        if key.kind == KeyEventKind::Release {
            return ListEvent::Ignored;
        }
        if let Some(index) = self.confirm_remove {
            return self.confirm_key(index, key);
        }
        if self.add_combo.as_ref().is_some_and(Combo::is_open) {
            return self.combo_key(key);
        }
        if self.drag.is_some() && key.code == KeyCode::Esc {
            self.drag = None;
            return ListEvent::Handled;
        }
        let km = crate::keymap::current();
        // The bindable actions first.
        if km.is(Action::PreferencesListAdd, &key) {
            return self.do_add();
        }
        if km.is(Action::PreferencesListDuplicate, &key) {
            return self.do_duplicate();
        }
        if km.is(Action::PreferencesListRemove, &key) {
            return self.do_remove();
        }
        if km.is(Action::PreferencesListMoveUp, &key) {
            return self.do_move(true);
        }
        if km.is(Action::PreferencesListMoveDown, &key) {
            return self.do_move(false);
        }
        if let Some(to) = popup_list::nav(&key, self.selected, self.len(), self.view_rows) {
            return Self::op(self.select_op(to));
        }
        match key.code {
            KeyCode::Enter | KeyCode::Right if self.has_selection() => {
                ListEvent::Op(ListOp::FocusDetail)
            }
            KeyCode::Left => ListEvent::Op(ListOp::Back),
            _ => ListEvent::Ignored,
        }
    }

    fn combo_key(&mut self, key: KeyEvent) -> ListEvent {
        let Some(combo) = self.add_combo.as_mut() else {
            return ListEvent::Ignored;
        };
        Self::picked(combo.on_key(key))
    }

    fn picked(outcome: ComboOutcome) -> ListEvent {
        match outcome {
            ComboOutcome::Picked(value) => value
                .parse()
                .map_or(ListEvent::Handled, |i| ListEvent::Op(ListOp::Add(Some(i)))),
            _ => ListEvent::Handled,
        }
    }

    /// The confirmation is modal: it swallows every key.
    fn confirm_key(&mut self, index: usize, key: KeyEvent) -> ListEvent {
        let event = match self.confirm.as_mut() {
            Some(confirm) => confirm.on_key(key),
            None => dialog_kit::ConfirmEvent::Chose(ConfirmHit::Cancel),
        };
        self.confirm_answer(index, event)
    }

    fn confirm_answer(
        &mut self,
        index: usize,
        event: dialog_kit::ConfirmEvent<ConfirmHit>,
    ) -> ListEvent {
        match event {
            dialog_kit::ConfirmEvent::Stay => ListEvent::Handled,
            dialog_kit::ConfirmEvent::Chose(hit) => self.confirm_press(index, hit),
        }
    }

    fn confirm_press(&mut self, index: usize, hit: ConfirmHit) -> ListEvent {
        self.confirm_remove = None;
        self.confirm = None;
        match hit {
            ConfirmHit::Cancel => ListEvent::Handled,
            ConfirmHit::Remove => ListEvent::Op(ListOp::Remove(index)),
        }
    }

    // ---- mouse ----

    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent) -> ListEvent {
        let at = (mouse.column, mouse.row).into();
        if let Some(index) = self.confirm_remove {
            return self.confirm_mouse(index, mouse);
        }
        if let Some(combo) = self.add_combo.as_mut().filter(|c| c.is_open()) {
            return Self::picked(combo.on_mouse(mouse));
        }
        match mouse.kind {
            MouseEventKind::Drag(MouseButton::Left) => {
                self.drag_to(mouse.row);
                return if self.bar_grab.is_some() || self.drag.is_some() {
                    ListEvent::Handled
                } else {
                    ListEvent::Ignored
                };
            }
            MouseEventKind::Up(MouseButton::Left) => {
                let had_bar = self.bar_grab.take().is_some();
                return match self.drag.take() {
                    Some(d) if d.started && d.over != d.from => ListEvent::Op(ListOp::Move {
                        from: d.from,
                        to: d.over,
                    }),
                    Some(_) => ListEvent::Handled,
                    None if had_bar => ListEvent::Handled,
                    None => ListEvent::Ignored,
                };
            }
            _ => {}
        }
        if !self.area.contains(at) {
            if matches!(mouse.kind, MouseEventKind::Moved) {
                self.hover = None;
            }
            return ListEvent::Ignored;
        }
        match mouse.kind {
            MouseEventKind::Moved => {
                self.hover = dialog_kit::hit_at(&self.hits, mouse.column, mouse.row);
            }
            MouseEventKind::ScrollUp => self.scroll_by(-(WHEEL_STEP as isize)),
            MouseEventKind::ScrollDown => self.scroll_by(WHEEL_STEP as isize),
            MouseEventKind::Down(MouseButton::Left) => return self.press(mouse),
            _ => {}
        }
        ListEvent::Handled
    }

    fn press(&mut self, mouse: MouseEvent) -> ListEvent {
        let at = (mouse.column, mouse.row).into();
        if let Some(track) = self.track.filter(|t| t.rect.contains(at)) {
            let pos = mouse.row - track.rect.y;
            match track.press(pos) {
                TrackPress::Arrow(step) => self.scroll_by(step),
                TrackPress::Grab { grab, jump } => {
                    self.bar_grab = Some(grab);
                    if jump {
                        self.scroll = track.drag_offset(
                            self.display_len(),
                            self.view_rows,
                            grab,
                            i32::from(pos),
                        );
                    }
                }
            }
            return ListEvent::Handled;
        }
        match dialog_kit::hit_at(&self.hits, mouse.column, mouse.row) {
            Some(ListHit::Row(i)) => {
                let op = self.select_op(i);
                let double = self.clicks.register(i);
                if self.spec.reorder {
                    self.drag = Some(Drag {
                        from: i,
                        over: i,
                        started: false,
                    });
                }
                if double {
                    self.drag = None;
                    return ListEvent::Op(ListOp::FocusDetail);
                }
                Self::op(op)
            }
            Some(button) => self.press_button(button),
            None => ListEvent::Handled,
        }
    }

    /// A drag at screen row `row`: the thumb, or the dragged item's target.
    fn drag_to(&mut self, row: u16) {
        if let (Some(grab), Some(track)) = (self.bar_grab, self.track) {
            self.scroll = track.drag_offset(
                self.display_len(),
                self.view_rows,
                grab,
                i32::from(row) - i32::from(track.rect.y),
            );
            return;
        }
        let Some(mut drag) = self.drag else {
            return;
        };
        let rows = self.view_rows.max(1);
        if row < self.inner.y {
            self.scroll_by(-1);
        } else if row >= self.inner.bottom() {
            self.scroll_by(1);
        }
        let slot = (row.saturating_sub(self.inner.y) as usize).min(rows - 1);
        let rows = self.display_rows();
        let over = match rows.get((self.scroll + slot).min(rows.len().saturating_sub(1))) {
            Some(DisplayRow::Item(i)) => *i,
            // A header stands for the first item under it.
            Some(DisplayRow::Header(_)) => rows
                .iter()
                .skip(self.scroll + slot)
                .find_map(|r| match r {
                    DisplayRow::Item(i) => Some(*i),
                    DisplayRow::Header(_) => None,
                })
                .unwrap_or(drag.from),
            None => drag.from,
        };
        // Items never leave their group: the marker stays where it was.
        if self.same_group(over, drag.from) {
            drag.over = over.min(self.len().saturating_sub(1));
        }
        drag.started |= drag.over != drag.from;
        self.drag = Some(drag);
    }

    fn confirm_mouse(&mut self, index: usize, mouse: MouseEvent) -> ListEvent {
        let event = match self.confirm.as_mut() {
            Some(confirm) => confirm.on_mouse(mouse),
            None => dialog_kit::ConfirmEvent::Chose(ConfirmHit::Cancel),
        };
        self.confirm_answer(index, event)
    }

    // ---- paint ----

    fn buttons(&self) -> Vec<Btn> {
        let mut v = vec![Btn {
            hit: ListHit::Add,
            label: match self.spec.add {
                AddKind::Plain => t::ADD.get().to_string(),
                AddKind::Menu(_) => format!("{} \u{25BE}", t::ADD.get()),
            },
            enabled: true,
        }];
        if self.spec.duplicate {
            v.push(Btn {
                hit: ListHit::Duplicate,
                label: t::DUPLICATE.get().to_string(),
                enabled: self.can_duplicate(),
            });
        }
        v.push(Btn {
            hit: ListHit::Remove,
            label: t::REMOVE.get().to_string(),
            enabled: self.can_remove(),
        });
        if self.spec.reorder {
            v.push(Btn {
                hit: ListHit::Up,
                label: "\u{2191}".into(),
                enabled: self.can_move_up(),
            });
            v.push(Btn {
                hit: ListHit::Down,
                label: "\u{2193}".into(),
                enabled: self.can_move_down(),
            });
        }
        v
    }

    /// Where each button goes, wrapping to the next line when the row is
    /// full: `(button index, dx, line)` and the number of lines.
    fn flow(buttons: &[Btn], width: u16) -> (Vec<(usize, u16, u16)>, u16) {
        let mut placed = Vec::new();
        let (mut x, mut line) = (0u16, 0u16);
        for (i, b) in buttons.iter().enumerate() {
            let w = dialog_kit::button_width(&b.label);
            // The up/down pair wraps together.
            let need = match (b.hit, buttons.get(i + 1)) {
                (ListHit::Up, Some(down)) => w + 1 + dialog_kit::button_width(&down.label),
                _ => w,
            };
            if x > 0 && x + need > width {
                x = 0;
                line += 1;
            }
            placed.push((i, x, line));
            x += w + 1;
        }
        (placed, line + 1)
    }

    /// Paints the editor into `rect` and the popups (add dropdown,
    /// confirmation) within `bounds`, recording every hit rect on the way.
    pub(crate) fn render(&mut self, buf: &mut Buffer, rect: Rect, focused: bool, bounds: Rect) {
        self.hits.clear();
        self.confirm_hits.clear();
        self.track = None;
        self.area = rect;
        if rect.width < 4 || rect.height < 2 {
            self.inner = Rect::default();
            return;
        }
        let buttons = self.buttons();
        let (placed, lines) = Self::flow(&buttons, rect.width);
        // The list keeps at least a frame and one row.
        let toolbar_h = lines.min(rect.height.saturating_sub(3));
        let list = Rect::new(rect.x, rect.y, rect.width, rect.height - toolbar_h);
        self.paint_list(buf, list, focused);
        for (i, dx, line) in placed {
            if line >= toolbar_h {
                break;
            }
            let b = &buttons[i];
            let y = list.bottom() + line;
            let w = dialog_kit::button_width(&b.label);
            if dx + w > rect.width {
                continue;
            }
            let hovered = self.hover == Some(b.hit);
            let state = if b.enabled {
                ButtonState::Normal {
                    hovered,
                    focused: false,
                }
            } else {
                ButtonState::Disabled
            };
            let r = dialog_kit::button(buf, rect.x + dx, y, &b.label, state);
            if b.hit == ListHit::Add {
                self.add_rect = r;
            }
            self.hits.push((r, b.hit));
        }
        if let Some(combo) = self.add_combo.as_mut() {
            combo.set_anchor(self.add_rect);
            combo.render_popup(buf, bounds);
            if !combo.is_open() {
                self.add_combo = None;
            }
        }
        if self.confirm_remove.is_some() {
            if let Some(confirm) = self.confirm.as_mut() {
                self.confirm_hits = confirm.render(buf, bounds);
            }
        }
    }

    fn paint_list(&mut self, buf: &mut Buffer, list: Rect, focused: bool) {
        let border = if focused {
            th().ui.accent
        } else {
            th().ui.muted
        };
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border))
            .title(format!(" {} ", self.spec.title))
            .title_style(
                Style::default()
                    .fg(th().ui.text)
                    .add_modifier(Modifier::BOLD),
            )
            .render(list, buf);
        let inner = Rect::new(
            list.x + 1,
            list.y + 1,
            list.width.saturating_sub(2),
            list.height.saturating_sub(2),
        );
        self.inner = inner;
        self.view_rows = inner.height as usize;
        self.scroll = self
            .scroll
            .min(self.display_len().saturating_sub(self.view_rows));
        if inner.width == 0 || inner.height == 0 {
            return;
        }
        if self.spec.items.is_empty() {
            dialog_kit::put(
                buf,
                inner.x + 1,
                inner.y,
                t::EMPTY.get(),
                inner.width.saturating_sub(1),
                Style::default().fg(th().ui.muted),
            );
        }
        let rows = self.display_rows();
        for (slot, shown) in rows
            .iter()
            .skip(self.scroll)
            .take(self.view_rows)
            .enumerate()
        {
            let row = Rect::new(inner.x, inner.y + slot as u16, inner.width, 1);
            match shown {
                DisplayRow::Header(group) => Self::paint_header(buf, row, group),
                DisplayRow::Item(i) => {
                    self.paint_row(buf, row, *i, &self.spec.items[*i], focused);
                    self.hits.push((row, ListHit::Row(*i)));
                }
            }
        }
        if let Some(track) = Track::vertical(list, rows.len(), self.scroll) {
            scrollbar::render_buf(buf, track, true, self.bar_grab.is_some());
            self.track = Some(track);
        }
        if let Some(d) = self.drag.filter(|d| d.started && d.over != d.from) {
            self.paint_insertion(buf, inner, d);
        }
    }

    fn paint_header(buf: &mut Buffer, row: Rect, group: &str) {
        dialog_kit::fill(buf, row, Style::default().bg(th().ui.panel));
        dialog_kit::put(
            buf,
            row.x + 1,
            row.y,
            group,
            row.width.saturating_sub(1),
            Style::default()
                .fg(th().ui.muted)
                .bg(th().ui.panel)
                .add_modifier(Modifier::BOLD),
        );
    }

    fn paint_row(&self, buf: &mut Buffer, row: Rect, i: usize, item: &ListItem, focused: bool) {
        let selected = i == self.selected;
        let bg = if selected && focused {
            th().ui.selection
        } else if selected || self.hover == Some(ListHit::Row(i)) {
            th().ui.hover
        } else {
            th().ui.panel
        };
        let dragged = self.drag.is_some_and(|d| d.started && d.from == i);
        let fg = if dragged {
            th().ui.disabled
        } else {
            th().ui.text
        };
        dialog_kit::fill(buf, row, Style::default().bg(bg));
        let (glyph, glyph_color) = match &item.mark {
            Mark::None => (" ", th().ui.text),
            Mark::Default => ("\u{2605}", th().status.warning),
            Mark::Active => ("\u{25CF}", th().status.success),
            Mark::Problem(_) => ("\u{26A0}", th().status.warning),
        };
        dialog_kit::put(
            buf,
            row.x,
            row.y,
            glyph,
            1,
            Style::default().fg(glyph_color).bg(bg),
        );
        let avail = (row.width as usize).saturating_sub(2);
        let sub = match &item.mark {
            Mark::Problem(reason) => Some(reason.as_str()),
            _ => item.sub.as_deref(),
        };
        // The label has priority; the sub text gets at most half the row.
        let sub_w = sub.map_or(0, |s| display_width(s).min(avail / 2));
        let label_room = if sub_w > 0 { avail - sub_w - 1 } else { avail };
        dialog_kit::put(
            buf,
            row.x + 2,
            row.y,
            &item.label,
            label_room as u16,
            Style::default().fg(fg).bg(bg),
        );
        if let (Some(sub), true) = (sub, sub_w > 0) {
            let color = if matches!(item.mark, Mark::Problem(_)) {
                th().status.warning
            } else {
                th().ui.muted
            };
            let shown = crate::fit(sub, sub_w);
            let x = row.right().saturating_sub(display_width(&shown) as u16 + 1);
            dialog_kit::put(
                buf,
                x,
                row.y,
                &shown,
                sub_w as u16,
                Style::default().fg(color).bg(bg),
            );
        }
    }

    /// The accent bar at the slot a drag would drop into: on the upper edge
    /// of the target row when moving up, on its lower edge when moving down.
    fn paint_insertion(&self, buf: &mut Buffer, inner: Rect, drag: Drag) {
        let Some(slot) = self.display_span(drag.over).0.checked_sub(self.scroll) else {
            return;
        };
        if slot >= self.view_rows {
            return;
        }
        let y = inner.y + slot as u16;
        let glyph = if drag.over < drag.from {
            "\u{2594}"
        } else {
            "\u{2581}"
        };
        for x in inner.x..inner.right() {
            let cell = &mut buf[(x, y)];
            if cell.symbol() == " " {
                cell.set_symbol(glyph);
                cell.set_fg(th().ui.accent);
            } else {
                cell.set_style(
                    Style::default()
                        .add_modifier(Modifier::UNDERLINED)
                        .underline_color(th().ui.accent),
                );
            }
        }
    }
}
