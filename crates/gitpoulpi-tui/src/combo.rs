//! The combobox: a field showing the chosen value (or a `Select...`
//! placeholder) and a `v` at its right, that opens a popup list below it
//! (above when there is no room). Long lists are searchable: typing while
//! the popup is open narrows it, `Up`/`Down` move, `Enter` picks, `Esc`
//! closes. With `allow_custom`, a typed value that is not in the list can
//! be used as it is (a branch the remote listing does not know yet).
//!
//! Used by the pull-request form (repositories and branches), the sidebar's
//! PULL REQUESTS / ISSUES filter and issue-source dropdowns, and anywhere
//! else a fixed set of choices is picked from. It draws straight into a
//! `Buffer` and keeps the rects it painted, so the hit rects come out of
//! the same walk that paints (the dialogs' rule).
//!
//! Every mouse gesture has its keyboard twin: click = `Enter`/`Space`/
//! `Down` on the focused field, click an item = `Up`/`Down` then `Enter`,
//! click away = `Esc`, wheel = `Up`/`Down`.

use crate::th;
use crate::wide_edge::Clear;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders, Widget};

use crate::dialog_kit::{self};
use crate::display_width;
use crate::scrollbar::{self, Track};

/// Rows of choices the popup assumes before its first paint (afterwards it
/// uses whatever height is free above or below the field).
const DEFAULT_ROWS: usize = 8;
/// Rows one wheel notch scrolls.
const WHEEL_STEP: usize = 3;
/// A list longer than this gets a search line.
const SEARCH_ABOVE: usize = 7;
/// A list this short whose search line only appeared because it is longer than
/// `SEARCH_ABOVE` (the log retention presets: 8 rows) still picks on the first
/// click. The select-then-confirm click pair is for long lists, where a click
/// may only be steering the highlight; on a short one it read as a click that
/// does nothing.
const CLICK_PICKS_UP_TO: usize = 12;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ComboOption {
    pub label: String,
    pub value: String,
    /// Optional color chips drawn at the row's right edge (the lane
    /// palette combo shows each palette's first colors).
    pub swatch: Vec<ratatui::style::Color>,
    /// Drawn greyed. Still pickable: the owner decides what a pick of an
    /// unavailable choice does (the Radar opens the Settings row that
    /// makes it available).
    pub greyed: bool,
}

impl ComboOption {
    pub(crate) fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            swatch: Vec::new(),
            greyed: false,
        }
    }

    /// The same option with color chips on its row.
    pub(crate) fn with_swatch(mut self, colors: Vec<ratatui::style::Color>) -> Self {
        self.swatch = colors;
        self
    }

    /// The same option drawn greyed.
    #[cfg(all(feature = "radar", feature = "hosting"))]
    pub(crate) fn greyed(mut self) -> Self {
        self.greyed = true;
        self
    }

    /// An option whose label is its value.
    pub(crate) fn plain(text: impl Into<String>) -> Self {
        let text = text.into();
        Self {
            label: text.clone(),
            value: text,
            swatch: Vec::new(),
            greyed: false,
        }
    }
}

/// What a key or click did to the combobox.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ComboOutcome {
    /// Not the combobox's: the owner handles it.
    Ignored,
    /// Consumed (moved, typed, opened, closed).
    Handled,
    /// A listed option was chosen: its `value`.
    Picked(String),
    /// The typed text was chosen (`allow_custom`).
    Custom(String),
}

#[derive(Debug, Clone)]
pub(crate) struct Combo {
    pub(crate) options: Vec<ComboOption>,
    /// The chosen option's `value`, or the custom text.
    selected: Option<String>,
    /// More options drawn with the check mark, besides `selected` (a menu
    /// of independent settings, each showing which value is on).
    marks: Vec<String>,
    placeholder: String,
    allow_custom: bool,
    /// Always show the search line, however short the list.
    force_search: bool,
    /// A click on a row picks it at once, as in a menu, even when the
    /// list is searchable.
    click_picks: bool,
    /// The list is still being fetched.
    pub(crate) loading: bool,
    open: bool,
    query: String,
    /// Highlighted row, in the filtered list (plus the custom row).
    cursor: usize,
    offset: usize,
    field: Rect,
    popup: Rect,
    items: Vec<(Rect, usize)>,
    hover: Option<usize>,
    /// Rows the popup showed last frame (its height minus the frame and
    /// the search line).
    view_rows: usize,
    /// The scrollbar painted last frame, when the rows overflow.
    track: Option<Track>,
    /// While the thumb is dragged: where in the thumb it was grabbed.
    drag_grab: Option<i32>,
    /// The last row click, for the "second click picks" of a searchable list.
    /// Kept here, not in `popup_list::View`: a combobox does not paint through
    /// it, so its frame reset would drop the first click of every pair.
    clicks: crate::popup_list::ClickPair,
}

impl Combo {
    pub(crate) fn new(placeholder: &str) -> Self {
        Self {
            options: Vec::new(),
            selected: None,
            marks: Vec::new(),
            placeholder: placeholder.to_string(),
            allow_custom: false,
            force_search: false,
            click_picks: false,
            loading: false,
            open: false,
            query: String::new(),
            cursor: 0,
            offset: 0,
            field: Rect::default(),
            popup: Rect::default(),
            items: Vec::new(),
            hover: None,
            view_rows: DEFAULT_ROWS,
            track: None,
            drag_grab: None,
            clicks: Default::default(),
        }
    }

    /// Replaces the text shown while nothing is chosen (a loading note).
    #[cfg_attr(not(feature = "hosting"), allow(dead_code))]
    pub(crate) fn set_placeholder(&mut self, placeholder: &str) {
        if self.placeholder != placeholder {
            self.placeholder = placeholder.to_string();
        }
    }

    /// Always offers the search line (a short list stays filterable).
    #[cfg_attr(not(feature = "hosting"), allow(dead_code))]
    pub(crate) fn with_search(mut self) -> Self {
        self.force_search = true;
        self
    }

    /// A click on a row picks it (a menu), instead of selecting it first.
    pub(crate) fn with_click_picks(mut self) -> Self {
        self.click_picks = true;
        self
    }

    pub(crate) fn with_custom(mut self) -> Self {
        self.allow_custom = true;
        self
    }

    pub(crate) fn set_options(&mut self, options: Vec<ComboOption>) {
        self.options = options;
        self.cursor = 0;
        self.offset = 0;
    }

    pub(crate) fn select(&mut self, value: Option<String>) {
        self.selected = value;
    }

    /// Options, besides the selected one, that carry the check mark.
    pub(crate) fn set_marks(&mut self, marks: Vec<String>) {
        self.marks = marks;
    }

    pub(crate) fn selected(&self) -> Option<&str> {
        self.selected.as_deref()
    }

    /// What the field shows for the choice: the option's label, or the
    /// custom text.
    pub(crate) fn selected_label(&self) -> Option<&str> {
        let value = self.selected.as_deref()?;
        Some(
            self.options
                .iter()
                .find(|o| o.value == value)
                .map_or(value, |o| o.label.as_str()),
        )
    }

    pub(crate) fn is_open(&self) -> bool {
        self.open
    }

    pub(crate) fn open(&mut self) {
        self.open = true;
        self.query.clear();
        self.cursor = self
            .selected
            .as_deref()
            .and_then(|v| self.options.iter().position(|o| o.value == v))
            .unwrap_or(0);
        self.offset = 0;
        self.clicks.reset();
        self.keep_visible();
    }

    pub(crate) fn close(&mut self) {
        self.open = false;
        self.drag_grab = None;
        self.track = None;
        self.query.clear();
        self.hover = None;
    }

    fn searchable(&self) -> bool {
        self.allow_custom || self.force_search || self.options.len() > SEARCH_ABOVE
    }

    /// Indexes into `options` matching the query (substring, any case).
    fn filtered(&self) -> Vec<usize> {
        let q = self.query.trim().to_lowercase();
        self.options
            .iter()
            .enumerate()
            .filter(|(_, o)| q.is_empty() || o.label.to_lowercase().contains(&q))
            .map(|(i, _)| i)
            .collect()
    }

    /// The `Use "text"` row: offered when custom text is allowed, typed,
    /// and not already an option.
    fn custom_row(&self) -> Option<String> {
        let q = self.query.trim();
        (self.allow_custom
            && !q.is_empty()
            && !self
                .options
                .iter()
                .any(|o| o.label.eq_ignore_ascii_case(q) || o.value.eq_ignore_ascii_case(q)))
        .then(|| q.to_string())
    }

    fn row_count(&self) -> usize {
        self.filtered().len() + usize::from(self.custom_row().is_some())
    }

    /// Scrolls the window over the rows by `delta` (the wheel, the
    /// scrollbar), leaving the highlight where it is.
    fn scroll_by(&mut self, delta: isize) {
        let max = self.row_count().saturating_sub(self.view_rows);
        self.offset = (self.offset as isize + delta).clamp(0, max as isize) as usize;
    }

    /// Whether the thumb is being dragged.
    #[cfg_attr(not(feature = "hosting"), allow(dead_code))]
    pub(crate) fn dragging(&self) -> bool {
        self.drag_grab.is_some()
    }

    /// Whether `(col, row)` is on the scrollbar (open popup only).
    #[cfg_attr(not(feature = "hosting"), allow(dead_code))]
    pub(crate) fn on_scrollbar(&self, col: u16, row: u16) -> bool {
        self.open
            && self
                .track
                .is_some_and(|t| t.rect.contains((col, row).into()))
    }

    fn keep_visible(&mut self) {
        let rows = self.view_rows.max(1);
        if self.cursor < self.offset {
            self.offset = self.cursor;
        } else if self.cursor >= self.offset + rows {
            self.offset = self.cursor + 1 - rows;
        }
    }

    fn pick_row(&mut self, row: usize) -> ComboOutcome {
        let filtered = self.filtered();
        if let Some(&i) = filtered.get(row) {
            let value = self.options[i].value.clone();
            self.selected = Some(value.clone());
            self.close();
            return ComboOutcome::Picked(value);
        }
        if row == filtered.len() {
            if let Some(text) = self.custom_row() {
                self.selected = Some(text.clone());
                self.close();
                return ComboOutcome::Custom(text);
            }
        }
        ComboOutcome::Handled
    }

    /// A key. Closed: `Enter`, `Space` and `Down` open it (the owner
    /// calls this only while the field has the focus). Open: it is modal
    /// and swallows everything.
    pub(crate) fn on_key(&mut self, key: KeyEvent) -> ComboOutcome {
        if !self.open {
            return match key.code {
                KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Down => {
                    self.open();
                    ComboOutcome::Handled
                }
                _ => ComboOutcome::Ignored,
            };
        }
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let nav_to = crate::popup_list::nav(&key, self.cursor, self.row_count(), self.view_rows);
        match key.code {
            KeyCode::Esc => self.close(),
            // The shared popup-list keys: arrows, Ctrl+N/P/J/K, Home/End, PgUp/PgDn.
            _ if nav_to.is_some() => {
                if let (Some(to), true) = (nav_to, self.row_count() > 0) {
                    self.cursor = to;
                    self.keep_visible();
                }
            }
            KeyCode::Tab | KeyCode::BackTab => self.close(),
            KeyCode::Enter => return self.pick_row(self.cursor),
            KeyCode::Backspace => {
                self.query.pop();
                self.cursor = 0;
                self.offset = 0;
            }
            KeyCode::Char('u') if ctrl => {
                self.query.clear();
                self.cursor = 0;
                self.offset = 0;
            }
            KeyCode::Char(c) if !ctrl && self.searchable() => {
                self.query.push(c);
                self.cursor = 0;
                self.offset = 0;
            }
            _ => {}
        }
        ComboOutcome::Handled
    }

    /// Pasted text goes into the search line.
    pub(crate) fn paste(&mut self, text: &str) {
        if self.open && self.searchable() {
            self.query.extend(text.chars().filter(|c| !c.is_control()));
            self.cursor = 0;
            self.offset = 0;
        }
    }

    /// A mouse event. Open, the popup is modal: a click on an item picks
    /// it, a click anywhere else closes it (and is consumed).
    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent) -> ComboOutcome {
        let at = (mouse.column, mouse.row).into();
        if self.open {
            match mouse.kind {
                MouseEventKind::Moved => {
                    self.hover = self
                        .items
                        .iter()
                        .find(|(r, _)| r.contains(at))
                        .map(|(_, i)| *i);
                    return ComboOutcome::Handled;
                }
                MouseEventKind::ScrollUp => {
                    self.scroll_by(-(WHEEL_STEP as isize));
                    return ComboOutcome::Handled;
                }
                MouseEventKind::ScrollDown => {
                    self.scroll_by(WHEEL_STEP as isize);
                    return ComboOutcome::Handled;
                }
                MouseEventKind::Drag(MouseButton::Left) => {
                    if let (Some(grab), Some(track)) = (self.drag_grab, self.track) {
                        self.offset = track.drag_offset(
                            self.row_count(),
                            self.view_rows,
                            grab,
                            mouse.row as i32 - track.rect.y as i32,
                        );
                    }
                    return ComboOutcome::Handled;
                }
                MouseEventKind::Up(MouseButton::Left) => {
                    self.drag_grab = None;
                    return ComboOutcome::Handled;
                }
                MouseEventKind::Down(MouseButton::Left) => {
                    if let Some(track) = self.track.filter(|t| t.rect.contains(at)) {
                        let pos = mouse.row - track.rect.y;
                        match track.press(pos) {
                            crate::scrollbar::TrackPress::Arrow(step) => self.scroll_by(step),
                            crate::scrollbar::TrackPress::Grab { grab, jump } => {
                                self.drag_grab = Some(grab);
                                if jump {
                                    self.offset = track.drag_offset(
                                        self.row_count(),
                                        self.view_rows,
                                        grab,
                                        i32::from(pos),
                                    );
                                }
                            }
                        }
                        return ComboOutcome::Handled;
                    }
                    if let Some(row) = self
                        .items
                        .iter()
                        .find(|(r, _)| r.contains(at))
                        .map(|(_, i)| *i)
                    {
                        return self.click_row(row);
                    }
                    let on_popup = self.popup.contains(at);
                    if !on_popup {
                        self.close();
                    }
                    return ComboOutcome::Handled;
                }
                _ => return ComboOutcome::Handled,
            }
        }
        if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) && self.field.contains(at)
        {
            self.open();
            return ComboOutcome::Handled;
        }
        ComboOutcome::Ignored
    }

    /// A click on row `row` of the filtered list (also when the owner routes
    /// the mouse itself). A searchable popup is a filterable list: the click
    /// selects, a second click on the same row picks (= `Enter`). A short
    /// dropdown without a search line picks on the first click.
    pub(crate) fn click_row(&mut self, row: usize) -> ComboOutcome {
        let short_auto_search =
            !self.allow_custom && !self.force_search && self.options.len() <= CLICK_PICKS_UP_TO;
        if self.click_picks || !self.searchable() || short_auto_search {
            return self.pick_row(row);
        }
        self.cursor = row.min(self.row_count().saturating_sub(1));
        if self.clicks.register(row) {
            self.pick_row(row)
        } else {
            ComboOutcome::Handled
        }
    }

    /// Where the field was last painted.
    #[cfg(test)]
    #[cfg_attr(not(feature = "ai"), allow(dead_code))] // only the AI Settings page uses it
    pub(crate) fn field_rect(&self) -> Rect {
        self.field
    }

    /// Whether a click at the point does something: the field (open or
    /// close the list) or, while open, a row of the list. The pointing hand.
    #[cfg_attr(not(all(feature = "radar", feature = "hosting")), allow(dead_code))] // only the Radar uses it
    pub(crate) fn pointer_over(&self, col: u16, row: u16) -> bool {
        let at = ratatui::layout::Position::new(col, row);
        self.field.contains(at) || (self.open && self.items.iter().any(|(r, _)| r.contains(at)))
    }

    /// The `value` of the option at filtered row `row`.
    #[cfg_attr(not(feature = "ai"), allow(dead_code))] // only the AI Settings page uses it
    pub(crate) fn value_at(&self, row: usize) -> Option<&str> {
        let i = *self.filtered().get(row)?;
        Some(self.options[i].value.as_str())
    }

    /// The `value` of the highlighted row (open popup only).
    pub(crate) fn highlighted_value(&self) -> Option<&str> {
        if self.open {
            self.value_at(self.cursor)
        } else {
            None
        }
    }

    /// The `value` of the row under the pointer, when open.
    #[cfg_attr(not(feature = "ai"), allow(dead_code))] // only the AI Settings page uses it
    pub(crate) fn hovered_value(&self) -> Option<&str> {
        self.hover.and_then(|r| self.value_at(r))
    }

    /// The hovered row, when the owner tracks the pointer itself.
    pub(crate) fn set_hover(&mut self, row: Option<usize>) {
        self.hover = row;
    }

    /// The rows painted last frame: `(rect, row in the filtered list)`.
    pub(crate) fn item_rects(&self) -> Vec<(Rect, usize)> {
        self.items.clone()
    }

    /// The rectangle the popup covered when it was last painted (empty
    /// once a frame has been painted with it closed). An owner uses it to
    /// shield what lies underneath for the rest of a click that just
    /// closed the popup.
    pub(crate) fn popup_rect(&self) -> Rect {
        self.popup
    }

    /// Where the popup hangs from, for an owner that paints the field
    /// itself (the sidebar's rows).
    pub(crate) fn set_anchor(&mut self, field: Rect) {
        self.field = field;
    }

    /// Paints the field: the choice (or the placeholder) and a `v`.
    pub(crate) fn render_field(
        &mut self,
        buf: &mut Buffer,
        rect: Rect,
        focused: bool,
        hovered: bool,
    ) {
        self.field = rect;
        crate::pointer_map::hand(rect);
        let bg = crate::input_bg(focused || self.open, hovered);
        dialog_kit::fill(buf, rect, Style::default().bg(bg));
        let arrow_w = 2u16;
        let text_w = rect.width.saturating_sub(arrow_w + 1);
        let (text, style) = match self.selected_label() {
            Some(label) => (label.to_string(), Style::default().fg(th().ui.text).bg(bg)),
            None => (
                self.placeholder.clone(),
                Style::default().fg(th().ui.muted).bg(bg),
            ),
        };
        dialog_kit::put(buf, rect.x + 1, rect.y, &text, text_w, style);
        dialog_kit::put(
            buf,
            rect.right().saturating_sub(arrow_w),
            rect.y,
            "\u{25BE}",
            1,
            Style::default()
                .fg(if focused || self.open {
                    th().ui.accent
                } else {
                    th().ui.muted
                })
                .bg(bg),
        );
    }

    /// Paints the open popup over everything else, inside `bounds`.
    /// Call it last.
    pub(crate) fn render_popup(&mut self, buf: &mut Buffer, bounds: Rect) {
        self.items.clear();
        self.track = None;
        self.popup = Rect::default();
        if !self.open || self.field.width == 0 {
            return;
        }
        let filtered = self.filtered();
        let custom = self.custom_row();
        let search = self.searchable();
        let total = filtered.len() + usize::from(custom.is_some());
        let overhead = 2 + u16::from(search);
        let widest = filtered
            .iter()
            .map(|&i| {
                let o = &self.options[i];
                let chips = if o.swatch.is_empty() {
                    0
                } else {
                    o.swatch.len() * 2 + 1
                };
                display_width(&o.label) + chips
            })
            .chain(custom.iter().map(|c| display_width(c) + 6))
            .max()
            .unwrap_or(0) as u16
            + 4;
        let width = self
            .field
            .width
            .max(widest.min(60))
            .max(16)
            .min(bounds.width);
        let x = self
            .field
            .x
            .min(bounds.right().saturating_sub(width))
            .max(bounds.x);
        // As tall as the list wants, capped by the room below the field
        // or, when that is short, by the larger of the two sides.
        let want = (total.max(1) as u16).saturating_add(overhead);
        let below = self.field.bottom();
        let room_below = bounds.bottom().saturating_sub(below);
        let room_above = self.field.y.saturating_sub(bounds.y);
        let (height, y) = if want <= room_below {
            (want, below)
        } else if want <= room_above {
            (want, self.field.y - want)
        } else if room_below >= room_above {
            (room_below.max(overhead + 1).min(bounds.height), below)
        } else {
            let h = room_above.max(overhead + 1).min(bounds.height);
            (h, self.field.y.saturating_sub(h).max(bounds.y))
        };
        let height = height.min(bounds.height);
        let y = y.min(bounds.bottom().saturating_sub(height)).max(bounds.y);
        let rows = (height.saturating_sub(overhead) as usize).max(1);
        // The window follows the highlight only when the popup's height
        // changed under it; a wheel/scrollbar scroll must not be undone.
        let resized = self.view_rows != rows;
        self.view_rows = rows;
        self.offset = self.offset.min(total.saturating_sub(rows));
        if resized {
            self.keep_visible();
        }
        let popup = Rect::new(x, y, width, height.min(bounds.height));
        self.popup = popup;
        Clear.render(popup, buf);
        Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(th().ui.accent))
            .style(Style::default().bg(th().ui.panel))
            .render(popup, buf);
        let inner_x = popup.x + 1;
        let inner_w = popup.width.saturating_sub(2);
        let mut row_y = popup.y + 1;
        if search {
            let line = if self.query.is_empty() {
                (
                    gitpoulpi_i18n::msg::combo::TYPE_TO_SEARCH.get().to_string(),
                    th().ui.muted,
                )
            } else {
                (format!("{}\u{2588}", self.query), th().ui.text)
            };
            dialog_kit::fill(
                buf,
                Rect::new(inner_x, row_y, inner_w, 1),
                Style::default().bg(crate::input_focus_bg()),
            );
            dialog_kit::put(
                buf,
                inner_x,
                row_y,
                &format!("\u{2315} {}", line.0),
                inner_w,
                Style::default().fg(line.1).bg(crate::input_focus_bg()),
            );
            row_y += 1;
        }
        if self.loading && total == 0 {
            dialog_kit::put(
                buf,
                inner_x,
                row_y,
                gitpoulpi_i18n::msg::combo::LOADING.get(),
                inner_w,
                Style::default().fg(th().ui.muted).bg(th().ui.panel),
            );
            return;
        }
        if total == 0 {
            dialog_kit::put(
                buf,
                inner_x,
                row_y,
                gitpoulpi_i18n::msg::combo::NO_MATCH.get(),
                inner_w,
                Style::default().fg(th().ui.disabled).bg(th().ui.panel),
            );
            return;
        }
        for (slot, row) in (self.offset..total).take(rows).enumerate() {
            let y = row_y + slot as u16;
            if y >= popup.bottom().saturating_sub(1) {
                break;
            }
            let band = Rect::new(inner_x, y, inner_w, 1);
            let highlighted = row == self.cursor;
            let hovered = self.hover == Some(row);
            let bg = if highlighted {
                th().ui.selection
            } else if hovered {
                th().ui.hover
            } else {
                th().ui.panel
            };
            dialog_kit::fill(buf, band, Style::default().bg(bg));
            let greyed = filtered.get(row).is_some_and(|&i| self.options[i].greyed);
            let (label, current, swatch) = match filtered.get(row) {
                Some(&i) => (
                    self.options[i].label.clone(),
                    self.selected.as_deref() == Some(self.options[i].value.as_str())
                        || self.marks.contains(&self.options[i].value),
                    self.options[i].swatch.clone(),
                ),
                None => (
                    gitpoulpi_i18n::msg::combo::use_value(custom.clone().unwrap_or_default()),
                    false,
                    Vec::new(),
                ),
            };
            // Chips first, so the label is cut short instead of over them.
            let chips_w = (swatch.len() as u16) * 2;
            let label_w = if chips_w > 0 {
                inner_w.saturating_sub(chips_w + 1)
            } else {
                inner_w
            };
            let mark = if current { "\u{2713} " } else { "  " };
            dialog_kit::put(
                buf,
                inner_x,
                y,
                &format!("{mark}{label}"),
                label_w,
                Style::default()
                    .fg(if greyed {
                        th().ui.disabled
                    } else {
                        th().ui.text
                    })
                    .bg(bg)
                    .add_modifier(if highlighted {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
            );
            for (n, color) in swatch.iter().enumerate() {
                let x = inner_x + inner_w.saturating_sub(chips_w) + (n as u16) * 2;
                dialog_kit::fill(buf, Rect::new(x, y, 2, 1), Style::default().bg(*color));
            }
            // A row of the list: the arrow, lit on hover.
            crate::pointer_map::plain(band);
            self.items.push((band, row));
        }
        // Overflowing rows get a scrollbar over the popup's right border.
        let body = Rect::new(popup.x, row_y - 1, popup.width, rows as u16 + 2);
        self.track = Track::vertical(body, total, self.offset);
        if let Some(track) = self.track {
            scrollbar::render_buf(buf, track, true, self.drag_grab.is_some());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn click(col: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: col,
            row,
            modifiers: KeyModifiers::NONE,
        }
    }

    fn combo(n: usize) -> Combo {
        let mut c = Combo::new("Select...");
        c.set_options(
            (0..n)
                .map(|i| ComboOption::plain(format!("item{i}")))
                .collect(),
        );
        c
    }

    fn screen(buf: &Buffer) -> String {
        dialog_kit::buffer_text(buf)
    }

    #[test]
    fn the_keyboard_opens_moves_and_picks() {
        let mut c = combo(3);
        assert_eq!(c.on_key(key(KeyCode::Esc)), ComboOutcome::Ignored);
        assert_eq!(c.on_key(key(KeyCode::Enter)), ComboOutcome::Handled);
        assert!(c.is_open());
        c.on_key(key(KeyCode::Down));
        c.on_key(key(KeyCode::Down));
        c.on_key(key(KeyCode::Down));
        assert_eq!(
            c.on_key(key(KeyCode::Enter)),
            ComboOutcome::Picked("item2".into())
        );
        assert!(!c.is_open());
        assert_eq!(c.selected_label(), Some("item2"));
        // Reopened, the highlight starts on the current choice; Esc keeps it.
        c.on_key(key(KeyCode::Down));
        c.on_key(key(KeyCode::Up));
        c.on_key(key(KeyCode::Esc));
        assert_eq!(c.selected(), Some("item2"));
    }

    #[test]
    fn a_long_list_is_searchable_and_a_short_one_is_not() {
        let mut long = combo(20);
        long.open();
        for ch in "m1".chars() {
            long.on_key(key(KeyCode::Char(ch)));
        }
        assert_eq!(long.filtered().len(), 11, "item1, item10..item19");
        assert_eq!(
            long.on_key(key(KeyCode::Enter)),
            ComboOutcome::Picked("item1".into())
        );

        let mut short = combo(3);
        short.open();
        short.on_key(key(KeyCode::Char('x')));
        assert_eq!(short.filtered().len(), 3, "no search line, keys ignored");
    }

    #[test]
    fn custom_text_is_offered_only_when_allowed_and_unknown() {
        let mut c = combo(2).with_custom();
        c.open();
        for ch in "topic".chars() {
            c.on_key(key(KeyCode::Char(ch)));
        }
        assert_eq!(c.filtered().len(), 0);
        assert_eq!(
            c.on_key(key(KeyCode::Enter)),
            ComboOutcome::Custom("topic".into())
        );
        assert_eq!(c.selected_label(), Some("topic"));
        c.open();
        for ch in "ITEM0".chars() {
            c.on_key(key(KeyCode::Char(ch)));
        }
        assert!(c.custom_row().is_none(), "a known option is not custom");
    }

    #[test]
    fn the_popup_paints_inside_its_bounds_and_the_mouse_reaches_each_row() {
        let bounds = Rect::new(0, 0, 40, 20);
        for field_y in [1u16, 17] {
            let mut c = combo(4);
            let mut buf = Buffer::empty(bounds);
            c.render_field(&mut buf, Rect::new(2, field_y, 20, 1), false, false);
            assert!(screen(&buf).contains("Select..."));
            assert_eq!(c.on_mouse(click(3, field_y)), ComboOutcome::Handled);
            assert!(c.is_open());
            c.render_popup(&mut buf, bounds);
            assert!(c.popup.right() <= bounds.right() && c.popup.bottom() <= bounds.bottom());
            assert_eq!(c.items.len(), 4);
            let (rect, _) = c.items[2];
            assert_eq!(
                c.on_mouse(click(rect.x + 1, rect.y)),
                ComboOutcome::Picked("item2".into())
            );
        }
    }

    fn wheel(down: bool, col: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind: if down {
                MouseEventKind::ScrollDown
            } else {
                MouseEventKind::ScrollUp
            },
            column: col,
            row,
            modifiers: KeyModifiers::NONE,
        }
    }

    fn open_long(n: usize, bounds: Rect) -> (Combo, Buffer) {
        let mut c = combo(n);
        let mut buf = Buffer::empty(bounds);
        c.render_field(&mut buf, Rect::new(2, 1, 20, 1), false, false);
        c.open();
        c.render_popup(&mut buf, bounds);
        (c, buf)
    }

    #[test]
    fn a_long_list_fills_the_available_height_and_scrolls_over_every_item() {
        let bounds = Rect::new(0, 0, 40, 30);
        let (mut c, mut buf) = open_long(100, bounds);
        assert!(c.items.len() > 8, "uses the room, not a fixed 8 rows");
        assert!(c.track.is_some(), "overflow gets a scrollbar");
        // End reaches the last item, Home the first, PageDown a page.
        c.on_key(key(KeyCode::End));
        c.render_popup(&mut buf, bounds);
        assert!(screen(&buf).contains("item99"));
        c.on_key(key(KeyCode::Home));
        c.render_popup(&mut buf, bounds);
        assert!(screen(&buf).contains("item0 "));
        c.on_key(key(KeyCode::PageDown));
        assert_eq!(c.cursor, c.view_rows - 1);
        // Down keeps the highlight in view all the way.
        for _ in 0..60 {
            c.on_key(key(KeyCode::Down));
            c.render_popup(&mut buf, bounds);
            assert!(c.items.iter().any(|(_, r)| *r == c.cursor));
        }
    }

    #[test]
    fn the_wheel_scrolls_the_window_and_the_thumb_drags() {
        let bounds = Rect::new(0, 0, 40, 30);
        let (mut c, mut buf) = open_long(100, bounds);
        let inside = (c.popup.x + 2, c.popup.y + 3);
        c.on_mouse(wheel(true, inside.0, inside.1));
        assert_eq!(c.offset, WHEEL_STEP);
        assert_eq!(c.cursor, 0, "the wheel leaves the highlight alone");
        c.render_popup(&mut buf, bounds);
        assert_eq!(c.offset, WHEEL_STEP, "a repaint keeps the scrolled window");
        for _ in 0..100 {
            c.on_mouse(wheel(true, inside.0, inside.1));
        }
        assert_eq!(c.offset, 100 - c.view_rows, "clamped at the end");
        c.on_mouse(wheel(false, inside.0, inside.1));
        assert_eq!(c.offset, 100 - c.view_rows - WHEEL_STEP);

        // Drag the thumb from the top to the bottom of the track.
        c.offset = 0;
        c.render_popup(&mut buf, bounds);
        let track = c.track.unwrap();
        let thumb_row = track.rect.y + track.thumb_start;
        assert!(c.on_scrollbar(track.rect.x, thumb_row));
        // The arrows scroll one row and never grab the thumb.
        c.on_mouse(click(track.rect.x, track.rect.bottom() - 1));
        assert_eq!((c.offset, c.dragging()), (1, false));
        c.on_mouse(click(track.rect.x, track.rect.y));
        assert_eq!((c.offset, c.dragging()), (0, false));
        c.on_mouse(click(track.rect.x, thumb_row));
        assert!(c.dragging());
        c.on_mouse(MouseEvent {
            kind: MouseEventKind::Drag(MouseButton::Left),
            column: track.rect.x,
            row: track.rect.bottom() + 3,
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(c.offset, 100 - c.view_rows);
        c.on_mouse(MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: track.rect.x,
            row: track.rect.y,
            modifiers: KeyModifiers::NONE,
        });
        assert!(!c.dragging());
        assert!(c.is_open(), "the scrollbar never closes or picks");
    }

    #[test]
    fn the_list_keys_move_the_highlight_in_a_long_list() {
        let bounds = Rect::new(0, 0, 40, 30);
        let (mut c, _buf) = open_long(100, bounds);
        let ctrl = |ch: char| KeyEvent::new(KeyCode::Char(ch), KeyModifiers::CONTROL);
        c.on_key(key(KeyCode::Down));
        c.on_key(key(KeyCode::Down));
        assert_eq!(c.cursor, 2);
        // No `Ctrl+letter` pair moves a list.
        for ch in ['n', 'p', 'j'] {
            c.on_key(ctrl(ch));
        }
        assert_eq!(c.cursor, 2);
        c.on_key(key(KeyCode::Up));
        c.on_key(key(KeyCode::Up));
        assert_eq!(c.cursor, 0);
        c.on_key(key(KeyCode::End));
        assert_eq!(c.cursor, 99);
        c.on_key(key(KeyCode::PageUp));
        assert_eq!(c.cursor, 99 - (c.view_rows - 1));
        c.on_key(key(KeyCode::Home));
        assert_eq!(c.cursor, 0);
        // A plain `j` is typed into the search, not a move.
        c.on_key(key(KeyCode::Char('j')));
        assert_eq!(c.query, "j");
    }

    #[test]
    fn in_a_searchable_popup_a_click_selects_and_a_double_click_picks() {
        let bounds = Rect::new(0, 0, 40, 30);
        let (mut c, _buf) = open_long(20, bounds);
        let (rect, row) = c.items[3];
        assert_eq!(c.on_mouse(click(rect.x + 1, rect.y)), ComboOutcome::Handled);
        assert!(c.is_open(), "one click does not validate");
        assert_eq!(c.cursor, row);
        assert_eq!(
            c.on_mouse(click(rect.x + 1, rect.y)),
            ComboOutcome::Picked("item3".into())
        );
    }

    #[test]
    fn a_short_list_that_only_just_got_a_search_line_picks_on_the_first_click() {
        let bounds = Rect::new(0, 0, 40, 30);
        let (mut c, _buf) = open_long(8, bounds);
        assert!(c.searchable());
        let (rect, _) = c.items[5];
        assert_eq!(
            c.on_mouse(click(rect.x + 1, rect.y)),
            ComboOutcome::Picked("item5".into())
        );
    }

    #[test]
    fn filtering_narrows_a_long_list_and_drops_the_scrollbar() {
        let bounds = Rect::new(0, 0, 40, 30);
        let (mut c, mut buf) = open_long(100, bounds);
        for ch in "item9".chars() {
            c.on_key(key(KeyCode::Char(ch)));
        }
        c.render_popup(&mut buf, bounds);
        assert_eq!(c.filtered().len(), 11);
        assert!(c.track.is_none(), "11 rows fit");
    }

    #[test]
    fn clicking_away_closes_without_choosing() {
        let bounds = Rect::new(0, 0, 40, 12);
        let mut c = combo(3);
        let mut buf = Buffer::empty(bounds);
        c.render_field(&mut buf, Rect::new(2, 1, 20, 1), false, false);
        c.open();
        c.render_popup(&mut buf, bounds);
        assert_eq!(c.on_mouse(click(35, 10)), ComboOutcome::Handled);
        assert!(!c.is_open());
        assert_eq!(c.selected(), None);
    }

    #[test]
    fn an_empty_loading_list_says_so() {
        let bounds = Rect::new(0, 0, 40, 12);
        let mut c = Combo::new("Select...");
        c.loading = true;
        let mut buf = Buffer::empty(bounds);
        c.render_field(&mut buf, Rect::new(2, 1, 20, 1), false, false);
        c.open();
        c.render_popup(&mut buf, bounds);
        assert!(screen(&buf).contains("loading"));
    }
}
