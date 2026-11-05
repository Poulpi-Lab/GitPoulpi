//! Painting the board, and the hit rects of that same walk (maintainer rule 6):
//! `render` clears the hit list, every part pushes the rects it draws (a
//! card before the column body it sits in), and the list is what the mouse
//! handler reads next.

use gitpoulpi_hosting::IssueSummary;
use gitpoulpi_i18n::msg::board as msg;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use super::{BoardView, FilterKind, SPIN_AFTER};
use crate::commit_detail::wrap_text;
use crate::dialog_kit::{self, ButtonState};
use crate::th;
use crate::ui_kit::{display_width, truncate_with_ellipsis};

/// Rows one card takes, its gap included.
pub(super) const CARD_H: u16 = 5;
/// The narrowest and widest a column is drawn.
const MIN_COL: u16 = 26;
const MAX_COL: u16 = 44;
const GAP: u16 = 1;

/// A click target of the view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Hit {
    Close,
    /// `(column, row in the column)`.
    Card(usize, usize),
    /// The empty part of a column (a drop target).
    Column(usize),
    Header(usize),
    Filter(FilterKind),
    Search,
    ClearFilters,
    New,
    Columns,
    Refresh,
}

/// How many columns fit in `avail` cells and how wide each is drawn.
pub(super) fn layout(columns: usize, avail: u16) -> (usize, u16) {
    if columns == 0 {
        return (1, MIN_COL.min(avail.max(1)));
    }
    let fit = (((avail + GAP) / (MIN_COL + GAP)) as usize).max(1);
    let visible = fit.min(columns);
    let each =
        (avail.saturating_sub(GAP * (visible as u16 - 1)) / visible as u16).clamp(1, MAX_COL);
    (visible, each)
}

fn panel() -> Style {
    Style::default().fg(th().ui.text).bg(th().ui.panel)
}

fn muted() -> Style {
    Style::default().fg(th().ui.muted).bg(th().ui.panel)
}

/// Up to two letters standing for a person.
pub(super) fn initials(name: &str) -> String {
    let words: Vec<&str> = name
        .split(|c: char| c.is_whitespace() || matches!(c, '.' | '_' | '-'))
        .filter(|w| !w.is_empty())
        .collect();
    let letters: Vec<char> = match words.as_slice() {
        [] => Vec::new(),
        [one] => one.chars().take(2).collect(),
        [first, .., last] => first.chars().take(1).chain(last.chars().take(1)).collect(),
    };
    letters.iter().flat_map(|c| c.to_uppercase()).collect()
}

impl BoardView {
    fn hovered(&self, hit: Hit) -> bool {
        self.hover == Some(hit)
    }

    /// Paints the whole view into `area`.
    pub(crate) fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let mut hits: Vec<(Rect, Hit)> = Vec::new();
        let r = dialog_kit::centered(
            area,
            area.width.saturating_sub(2),
            area.height.saturating_sub(2),
        );
        let title = msg::title(&self.target.seed.provider);
        dialog_kit::panel(buf, r, &title);
        if r.width < 50 || r.height < 14 {
            dialog_kit::put(
                buf,
                r.x + 2,
                r.y + 1,
                msg::TOO_SMALL.get(),
                r.width.saturating_sub(4),
                muted(),
            );
            self.hits = hits;
            return;
        }
        // The close mark in the top border.
        let close = " \u{2715} ";
        let cx = r.right().saturating_sub(5);
        let style = if self.hovered(Hit::Close) {
            Style::default().fg(th().ui.on_accent).bg(th().ui.accent)
        } else {
            muted()
        };
        let rect = dialog_kit::put(buf, cx, r.y, close, 3, style);
        hits.push((rect, Hit::Close));

        let inner = Rect::new(r.x + 2, r.y + 1, r.width - 4, r.height - 2);
        self.info_line(buf, inner);
        self.toolbar(
            buf,
            Rect::new(inner.x, inner.y + 1, inner.width, 1),
            &mut hits,
        );
        let status_y = inner.bottom().saturating_sub(2);
        let hints_y = inner.bottom().saturating_sub(1);
        let body = Rect::new(
            inner.x,
            inner.y + 2,
            inner.width,
            status_y.saturating_sub(inner.y + 2),
        );
        self.sel_rect = None;
        if let Some(why) = &self.load_err {
            if !self.loaded {
                dialog_kit::put(
                    buf,
                    body.x,
                    body.y,
                    &msg::load_failed(why),
                    body.width,
                    Style::default().fg(th().status.danger).bg(th().ui.panel),
                );
                dialog_kit::put(
                    buf,
                    body.x,
                    body.y + 1,
                    msg::RETRY_HINT.get(),
                    body.width,
                    muted(),
                );
            }
        }
        if !self.loaded {
            if self.load_err.is_none() && self.started.elapsed() >= SPIN_AFTER {
                dialog_kit::put(
                    buf,
                    body.x,
                    body.y,
                    &format!("{} {}", self.spinner(), msg::LOADING.get()),
                    body.width,
                    Style::default().fg(th().ui.accent).bg(th().ui.panel),
                );
            }
        } else {
            self.columns_page(buf, body, &mut hits);
        }
        self.status_line(buf, inner.x, status_y, inner.width);
        self.footer(buf, inner.x, hints_y, inner.width);
        crate::pointer_map::from_hits(&hits, |h| match h {
            Hit::Search => Some(crate::pointer_map::Kind::Field),
            Hit::Column(_) => None,
            // A card lights on hover and keeps the arrow.
            Hit::Card(..) => Some(crate::pointer_map::Kind::Plain),
            _ => Some(crate::pointer_map::Kind::Hand),
        });
        self.hits = hits;

        // Popups last, over everything.
        for kind in FilterKind::ALL {
            self.combos[kind.index()].render_popup(buf, area);
        }
        if let Some((_, combo)) = self.menu.as_mut() {
            combo.render_popup(buf, area);
        }
        if let Some(combo) = self.move_combo.as_mut() {
            combo.render_popup(buf, area);
        }
        if let Some(p) = self.prompt.as_mut() {
            p.render(area, buf);
        }
        if let Some(m) = self.meta.as_mut() {
            m.render(area, buf);
        }
    }

    fn info_line(&self, buf: &mut Buffer, inner: Rect) {
        let shown: usize = self.cards.iter().map(Vec::len).sum();
        let mut text = msg::info(
            self.target.project_label(),
            self.issues.len() as u64,
            shown as u64,
        );
        if self.loaded && !self.can_move() {
            text.push_str("  -  ");
            text.push_str(msg::READ_ONLY_NOTE.get());
        }
        dialog_kit::put(buf, inner.x, inner.y, &text, inner.width, muted());
    }

    /// The filter dropdowns, the text filter and the buttons.
    fn toolbar(&mut self, buf: &mut Buffer, row: Rect, hits: &mut Vec<(Rect, Hit)>) {
        // Buttons from the right: Refresh, Columns (a label board), New.
        let mut right = row.right();
        let mut button = |buf: &mut Buffer, label: &str, hit: Hit, hot: bool, right: &mut u16| {
            let w = dialog_kit::button_width(label);
            if right.saturating_sub(row.x) < w + 40 {
                return;
            }
            *right -= w;
            let rect = dialog_kit::button(
                buf,
                *right,
                row.y,
                label,
                ButtonState::Normal {
                    hovered: hot,
                    focused: false,
                },
            );
            hits.push((rect, hit));
            *right -= 1;
        };
        button(
            buf,
            msg::REFRESH.get(),
            Hit::Refresh,
            self.hovered(Hit::Refresh),
            &mut right,
        );
        if self.label_columns() {
            button(
                buf,
                msg::COLUMNS.get(),
                Hit::Columns,
                self.hovered(Hit::Columns),
                &mut right,
            );
        }
        button(
            buf,
            msg::NEW.get(),
            Hit::New,
            self.hovered(Hit::New),
            &mut right,
        );

        let room = right.saturating_sub(row.x);
        let combo_w = ((room.saturating_sub(24)) / 3).clamp(14, 24);
        let mut x = row.x;
        for kind in FilterKind::ALL {
            if x + combo_w > right {
                break;
            }
            let rect = Rect::new(x, row.y, combo_w, 1);
            let hot = self.hovered(Hit::Filter(kind));
            let combo = &mut self.combos[kind.index()];
            combo.render_field(buf, rect, false, hot);
            hits.push((rect, Hit::Filter(kind)));
            x += combo_w + 1;
        }
        // The text filter takes what is left (up to 26 cells).
        let search_w = right.saturating_sub(x).min(26);
        if search_w >= 10 {
            let rect = Rect::new(x, row.y, search_w, 1);
            dialog_kit::text_input(
                buf,
                rect,
                &self.search,
                msg::SEARCH_PLACEHOLDER.get(),
                self.searching,
                self.hovered(Hit::Search),
                (&self.search_edit, ()),
            );
            hits.push((rect, Hit::Search));
            x += search_w + 1;
            if !self.filter.is_empty()
                && right.saturating_sub(x) >= dialog_kit::button_width(msg::CLEAR.get())
            {
                let rect = dialog_kit::button(
                    buf,
                    x,
                    row.y,
                    msg::CLEAR.get(),
                    ButtonState::Normal {
                        hovered: self.hovered(Hit::ClearFilters),
                        focused: false,
                    },
                );
                hits.push((rect, Hit::ClearFilters));
            }
        }
    }

    /// The headers, the cards and the horizontal bar.
    fn columns_page(&mut self, buf: &mut Buffer, body: Rect, hits: &mut Vec<(Rect, Hit)>) {
        let n = self.columns.len();
        self.body_rect = body;
        let (visible, each) = layout(n, body.width);
        let need_bar = visible < n;
        self.visible_cols = visible;
        let bar_h = u16::from(need_bar);
        let header_y = body.y;
        let top = body.y + 1;
        let cards_h = body.height.saturating_sub(1 + bar_h);
        self.rows_per_col = ((cards_h / CARD_H) as usize).max(1);
        self.first_col = self.first_col.min(n.saturating_sub(visible));
        // The selection is scrolled into view when it (or the room) changes,
        // not on every paint: that would undo the wheel and the bars.
        let key = (self.sel_col, self.sel_row(), self.rows_per_col, visible);
        if key != self.followed {
            self.followed = key;
            self.ensure_visible();
        }
        self.col_bars.clear();
        let first = self.first_col;

        for (slot, col) in (first..n).take(visible).enumerate() {
            let x = body.x + slot as u16 * (each + GAP);
            let drop = self.drag.is_some_and(|d| d.moved && d.over == Some(col));
            let header = Rect::new(x, header_y, each, 1);
            self.header(buf, header, col, drop);
            let column = Rect::new(x, top, each, cards_h);
            if drop {
                dialog_kit::fill(buf, column, Style::default().bg(th().ui.hover));
            }
            let list = self.cards.get(col).cloned().unwrap_or_default();
            let offset = self
                .offsets
                .get(col)
                .copied()
                .unwrap_or(0)
                .min(list.len().saturating_sub(self.rows_per_col));
            if let Some(off) = self.offsets.get_mut(col) {
                *off = offset;
            }
            if list.is_empty() {
                dialog_kit::put(
                    buf,
                    x + 1,
                    top,
                    msg::NO_CARDS.get(),
                    each.saturating_sub(1),
                    muted().bg(if drop { th().ui.hover } else { th().ui.panel }),
                );
            }
            for (k, row) in (offset..list.len()).take(self.rows_per_col).enumerate() {
                let rect = Rect::new(x, top + k as u16 * CARD_H, each, CARD_H - 1);
                if rect.bottom() > top + cards_h {
                    break;
                }
                let issue = self.issues[list[row]].clone();
                let selected = self.sel_number == Some(issue.number);
                if selected {
                    self.sel_rect = Some(rect);
                }
                self.card(
                    buf,
                    rect,
                    &issue,
                    selected,
                    self.hovered(Hit::Card(col, row)),
                );
                hits.push((rect, Hit::Card(col, row)));
            }
            // The column's bar rides its last cell when its cards overflow.
            if let Some(track) = crate::scrollbar::Track::column(
                Rect::new(x + each - 1, top, 1, cards_h),
                list.len(),
                self.rows_per_col,
                offset,
            ) {
                crate::scrollbar::render_buf(buf, track, true, false);
                self.col_bars.push((col, track));
            }
            // A column's empty part is a drop target and a click that
            // selects the column.
            hits.push((column, Hit::Column(col)));
            hits.push((header, Hit::Header(col)));
        }
        if need_bar {
            let bar_y = body.bottom().saturating_sub(1);
            let track_body = Rect::new(
                body.x.saturating_sub(1),
                bar_y.saturating_sub(1),
                body.width + 2,
                2,
            );
            // The bar speaks in cells: the columns' whole width, the part
            // shown, how far in.
            let stride = each + GAP;
            self.col_stride = stride;
            let whole = n * stride as usize - GAP as usize;
            self.hbar =
                crate::scrollbar::Track::horizontal(track_body, whole, first * stride as usize);
            if let Some(track) = self.hbar {
                crate::scrollbar::render_buf(buf, track, false, false);
            }
        } else {
            self.hbar = None;
        }
    }

    fn header(&self, buf: &mut Buffer, rect: Rect, col: usize, drop: bool) {
        let hot = self.hovered(Hit::Header(col));
        let style = if drop {
            Style::default()
                .fg(th().ui.on_accent)
                .bg(th().ui.accent)
                .add_modifier(Modifier::BOLD)
        } else if hot || self.sel_col == col {
            Style::default()
                .fg(th().ui.on_button)
                .bg(th().ui.button_hover)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(th().ui.on_button)
                .bg(th().ui.button)
                .add_modifier(Modifier::BOLD)
        };
        dialog_kit::fill(buf, rect, style);
        let count = self.cards.get(col).map_or(0, Vec::len);
        let offset = self.offsets.get(col).copied().unwrap_or(0);
        let more_below = offset + self.rows_per_col < count;
        let mut right = format!(" {count} ");
        if more_below {
            right.push('\u{25BC}');
        }
        if offset > 0 {
            right.insert(0, '\u{25B2}');
        }
        let right_w = display_width(&right) as u16;
        let title_w = rect.width.saturating_sub(right_w + 1) as usize;
        let title = truncate_with_ellipsis(&self.column_title(col), title_w);
        dialog_kit::put(buf, rect.x + 1, rect.y, &title, title_w as u16, style);
        dialog_kit::put(
            buf,
            rect.right().saturating_sub(right_w),
            rect.y,
            &right,
            right_w,
            style,
        );
    }

    fn card(&self, buf: &mut Buffer, rect: Rect, issue: &IssueSummary, selected: bool, hot: bool) {
        let dragged = self
            .drag
            .is_some_and(|d| d.moved && d.number == issue.number);
        let bg = if selected {
            th().ui.selection
        } else if hot {
            th().ui.input_hover
        } else {
            crate::input_bg(false, false)
        };
        let fg = if dragged { th().ui.muted } else { th().ui.text };
        dialog_kit::fill(buf, rect, Style::default().bg(bg));
        let inner_w = rect.width.saturating_sub(2);
        let text = Style::default().fg(fg).bg(bg);
        // Row 0: the reference on the left, the assignees' initials on the right.
        let mut right = rect.right().saturating_sub(1);
        let mut shown = 0usize;
        for name in &issue.assignees {
            let badge = format!(" {} ", initials(name));
            let w = display_width(&badge) as u16;
            if right < rect.x + 1 + 10 + w {
                break;
            }
            right -= w;
            dialog_kit::put(
                buf,
                right,
                rect.y,
                &badge,
                w,
                Style::default().fg(th().ui.on_accent).bg(th().ui.accent),
            );
            right = right.saturating_sub(1);
            shown += 1;
        }
        if shown < issue.assignees.len() {
            let more = format!("+{}", issue.assignees.len() - shown);
            let w = display_width(&more) as u16;
            if right >= rect.x + 1 + 8 + w {
                right -= w;
                dialog_kit::put(buf, right, rect.y, &more, w, text);
            }
        }
        let reference = if issue.iid_label.is_empty() {
            &issue.reference
        } else {
            &issue.iid_label
        };
        let ref_room = right.saturating_sub(rect.x + 1);
        let used = dialog_kit::put(
            buf,
            rect.x + 1,
            rect.y,
            reference,
            ref_room,
            Style::default()
                .fg(th().ui.accent)
                .bg(bg)
                .add_modifier(Modifier::BOLD),
        );
        if let Some(m) = self.pending(issue.number) {
            if m.since.elapsed() >= SPIN_AFTER && used.right() + 2 <= right {
                dialog_kit::put(
                    buf,
                    used.right() + 1,
                    rect.y,
                    &self.spinner().to_string(),
                    1,
                    Style::default().fg(th().status.warning).bg(bg),
                );
            }
        }
        // Rows 1-2: the title, two lines.
        let lines = wrap_text(&issue.title, inner_w.max(1) as usize);
        for (i, line) in lines.iter().take(2).enumerate() {
            let last = i == 1 && lines.len() > 2;
            let shown = if last {
                truncate_with_ellipsis(&format!("{line} \u{2026}"), inner_w as usize)
            } else {
                line.clone()
            };
            dialog_kit::put(
                buf,
                rect.x + 1,
                rect.y + 1 + i as u16,
                &shown,
                inner_w,
                text,
            );
        }
        // Row 3: label chips, then the milestone.
        let mut x = rect.x + 1;
        let limit = rect.right().saturating_sub(1);
        let mut chips = issue.labels.iter().peekable();
        while let Some(label) = chips.next() {
            let chip = format!(" {label} ");
            let w = display_width(&chip) as u16;
            let remaining = chips.clone().count();
            // Keep room for a "+n" when more would not fit.
            if x + w > limit || (remaining > 0 && x + w + 4 > limit) {
                let more = format!("+{}", remaining + 1);
                dialog_kit::put(
                    buf,
                    x,
                    rect.y + 3,
                    &more,
                    limit.saturating_sub(x),
                    muted().bg(bg),
                );
                x = limit;
                break;
            }
            dialog_kit::put(
                buf,
                x,
                rect.y + 3,
                &chip,
                w,
                Style::default().fg(th().ui.on_button).bg(th().ui.button),
            );
            x += w + 1;
        }
        if let Some(ms) = issue.milestone.as_deref().filter(|m| !m.is_empty()) {
            if x < limit {
                let text = format!("\u{25C6} {ms}");
                dialog_kit::put(buf, x, rect.y + 3, &text, limit - x, muted().bg(bg));
            }
        }
    }

    fn status_line(&self, buf: &mut Buffer, x: u16, y: u16, width: u16) {
        let slow = self.moves.iter().find(|m| m.since.elapsed() >= SPIN_AFTER);
        let (text, style) = if let Some(m) = slow {
            (
                format!(
                    "{} {}",
                    self.spinner(),
                    msg::moving(&m.caption, &m.column_title)
                ),
                Style::default().fg(th().ui.accent).bg(th().ui.panel),
            )
        } else if let Some(text) = &self.message {
            (
                text.clone(),
                Style::default().fg(th().status.warning).bg(th().ui.panel),
            )
        } else if self.loaded && self.load_err.is_some() {
            (
                msg::load_failed(self.load_err.as_deref().unwrap_or("")),
                Style::default().fg(th().status.danger).bg(th().ui.panel),
            )
        } else if self.loaded && self.load_rx.is_some() && self.started.elapsed() >= SPIN_AFTER {
            (
                format!("{} {}", self.spinner(), msg::LOADING.get()),
                Style::default().fg(th().ui.accent).bg(th().ui.panel),
            )
        } else {
            (String::new(), panel())
        };
        dialog_kit::fill(buf, Rect::new(x, y, width, 1), panel());
        dialog_kit::put(buf, x, y, &text, width, style);
    }

    fn footer(&self, buf: &mut Buffer, x: u16, y: u16, width: u16) {
        use crossterm::event::KeyCode;
        let enter = dialog_kit::key_name(crate::keymap::chord::Key::Enter);
        let shift_arrows = msg::KEY_SHIFT_ARROWS.get();
        let _ = KeyCode::Enter;
        dialog_kit::hints(
            buf,
            x,
            y,
            width,
            &[
                (shift_arrows, msg::HINT_MOVE.get()),
                (enter.as_str(), msg::HINT_OPEN.get()),
                ("e", msg::HINT_EDIT.get()),
                ("n", msg::HINT_NEW.get()),
                ("/", msg::HINT_FILTER.get()),
                (msg::KEY_ESC.get(), msg::HINT_CLOSE.get()),
            ],
        );
    }
}
