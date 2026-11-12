//! The Radar's paint walk. Hit rects come out of the same walk that
//! writes the glyphs (the `log_table` rule), registered inner-first: a row's
//! star and menu button before the band that covers them.

use crate::wide_edge::Clear;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;
use ratatui::Frame;

use gitpoulpi_i18n::msg;

use super::labels::status_words;
use super::model::{iso_to_unix, Group, Heading, Item, ItemKind, Row};
use super::scope;
use super::{heading_title, Cell, Hit, RadarUi};
use crate::dialog_kit::{fill, put};
use crate::hosting_task::age_label;
use crate::{button_bg, display_width, th};
use gitpoulpi_hosting::ChecksState;

/// Rows above the list: title, blank, combos, filter, status, and the
/// column header row.
const CHROME_TOP: u16 = 6;
/// Rows below it: the footer.
const CHROME_BOTTOM: u16 = 1;

const STAR: &str = "\u{2605}";
const MORE: &str = "\u{22EE}";
const PR_ICON: &str = "\u{21C4}";
const ISSUE_ICON: &str = "\u{25CF}";
const WIP_ICON: &str = "\u{270E}";
const CHECK_PASS: &str = "\u{2713}";
const CHECK_FAIL: &str = "\u{2717}";
const CHECK_PENDING: &str = "\u{25CF}";
/// A request that conflicts with its target.
const CONFLICT: &str = "!";

/// Where each column of an item row starts, the same for every row: the
/// header, the separators and the rows all read it, so they cannot drift.
#[derive(Debug, Clone, Copy)]
struct Geo {
    repo_x: u16,
    chip_w: usize,
    label_x: Option<u16>,
    /// The `#` column's width (label and check marks), 0 = no such column.
    num_w: usize,
    /// Cells at its right end kept for the checks / conflict marks.
    status_w: usize,
    title_x: u16,
    title_w: usize,
    /// Where the right-hand columns end (the row menu button sits here).
    edge: u16,
    /// The state chip, the author and the age, each when there is room.
    state: Option<(u16, usize)>,
    author: Option<(u16, usize)>,
    age: Option<(u16, usize)>,
}

/// The widths one frame measures over every item.
#[derive(Debug, Clone, Copy)]
struct Widths {
    chip: usize,
    label: usize,
    /// The `#` column's content width (label and check marks, see `Geo`).
    num: usize,
    status: usize,
    author: usize,
    state: usize,
    age: usize,
}

impl Geo {
    fn new(body: Rect, w: Widths) -> Self {
        let repo_x = body.x + 5;
        let mut x = repo_x + w.chip as u16 + 1;
        let label_x = (w.label > 0).then_some(x);
        let mut num_w = 0;
        let mut status_w = 0;
        if w.label > 0 {
            num_w = w.num;
            // The marks need their cells and a label of at least two.
            if w.status > 0 && num_w >= w.status + 2 {
                status_w = w.status;
            }
            x += num_w as u16 + 1;
        }
        // The right edge, from the right: the menu button, the age, the
        // author, the state chip; dropped in that order when narrow.
        let edge = body.right().saturating_sub(1).saturating_sub(2);
        let room = edge.saturating_sub(x) as usize;
        let mut tail: Vec<(u8, usize)> = Vec::new();
        if room >= 48 && w.state > 0 {
            tail.push((0, w.state));
        }
        if room >= 60 && w.author > 0 {
            tail.push((1, w.author));
        }
        if room >= 40 && w.age > 0 {
            tail.push((2, w.age));
        }
        let tail_w: usize = tail.iter().map(|(_, t)| t + 2).sum();
        let mut geo = Geo {
            repo_x,
            chip_w: w.chip,
            label_x,
            num_w,
            status_w,
            title_x: x,
            title_w: room.saturating_sub(tail_w + 1),
            edge,
            state: None,
            author: None,
            age: None,
        };
        let mut tx = edge.saturating_sub(tail_w as u16);
        for (kind, width) in tail {
            let slot = Some((tx, width));
            match kind {
                0 => geo.state = slot,
                1 => geo.author = slot,
                _ => geo.age = slot,
            }
            tx += width as u16 + 2;
        }
        geo
    }

    /// The sizable columns' boundaries: the repository's, then the number's, the author's and the age's.
    fn edges(&self) -> Vec<(Cell, crate::column_resize::Edge)> {
        let mut out = vec![(
            Cell::Repo,
            crate::column_resize::Edge::before(self.repo_x + self.chip_w as u16 + 1),
        )];
        if let Some(x) = self.label_x {
            out.push((
                Cell::Num,
                crate::column_resize::Edge::before(x + self.num_w as u16 + 1),
            ));
        }
        if let Some((x, _)) = self.author {
            out.push((Cell::Author, crate::column_resize::Edge::before(x)));
        }
        if let Some((x, _)) = self.age {
            out.push((Cell::Age, crate::column_resize::Edge::before(x)));
        }
        out
    }
}

/// Whether the host said anything about a row's checks or conflicts.
fn has_status(item: &Item) -> bool {
    item.checks.is_some() || item.has_conflicts.is_some()
}

/// A workspace's color name or `#rrggbb` as a terminal color; the accent when
/// it names nothing known.
pub(super) fn chip_color(name: Option<&str>) -> Color {
    let Some(name) = name else {
        return th().ui.accent;
    };
    if let Some((r, g, b)) = crate::theme::parse_hex_color(name) {
        return Color::Rgb(r, g, b);
    }
    match name.trim().to_ascii_lowercase().as_str() {
        "red" => Color::Rgb(0xE0, 0x6C, 0x75),    // color-data
        "orange" => Color::Rgb(0xD1, 0x9A, 0x66), // color-data
        "yellow" => Color::Rgb(0xE5, 0xC0, 0x7B), // color-data
        "green" => Color::Rgb(0x98, 0xC3, 0x79),  // color-data
        "teal" => Color::Rgb(0x56, 0xB6, 0xC2),   // color-data
        "blue" => Color::Rgb(0x61, 0xAF, 0xEF),   // color-data
        "purple" => Color::Rgb(0xC6, 0x78, 0xDD), // color-data
        "pink" => Color::Rgb(0xE0, 0x8F, 0xC0),   // color-data
        _ => th().ui.accent,
    }
}

fn button(hovered: bool) -> Style {
    Style::default()
        .fg(th().ui.on_button)
        .bg(button_bg(hovered))
}

impl RadarUi {
    fn repo_color(&self, path: &std::path::Path) -> Color {
        chip_color(
            self.repos
                .iter()
                .find(|r| r.path == path)
                .and_then(|r| r.color.as_deref()),
        )
    }

    /// Paints the view into `body` (the whole Home window under the tab
    /// bar) and records every hit rect.
    pub(crate) fn render(&mut self, frame: &mut Frame, body: Rect) {
        self.hits.clear();
        self.menu_rects.clear();
        let hover_blocked = self.popup_open();
        let hover = if hover_blocked { None } else { self.hover };
        {
            let buf = frame.buffer_mut();
            Clear.render(body, buf);
            self.paint_chrome(buf, body, hover);
            self.paint_list(buf, body, hover);
            self.paint_footer(buf, body);
            // Popups last: they cover whatever is under them.
            for which in [Hit::Scope, Hit::View, Hit::Who, Hit::Snooze] {
                self.combo_of(which).render_popup(buf, body);
            }
        }
        if let Some(menu) = &self.menu {
            self.menu_rects = menu.render(frame, self.menu_hover);
        }
        self.paint_tooltip(frame, body, hover_blocked);
        // The question, then the hosted review: last, over everything.
        self.paint_choice(frame.buffer_mut(), body);
        if let Some(view) = self.api_review.as_mut() {
            view.render_all(body, frame.buffer_mut());
        }
    }

    /// The checks / conflicts tooltip, after the dwell. Never under a popup
    /// (nothing behind one hovers).
    fn paint_tooltip(&self, frame: &mut Frame, body: Rect, blocked: bool) {
        let (Some(hit), Some(since)) = (self.hover, self.hover_since) else {
            return;
        };
        let (Hit::Status(offset) | Hit::Cut(offset, _)) = hit else {
            return;
        };
        if blocked || since.elapsed() < crate::tooltip::DWELL {
            return;
        }
        let Some(Row::Item(i)) = self.rows.get(offset) else {
            return;
        };
        let Some(item) = self.board.items.get(*i) else {
            return;
        };
        let words = match hit {
            // A cut cell: its full text, wrapped to a readable width.
            Hit::Cut(_, cell) => {
                let full = Self::cut_text(item, cell);
                let width = (body.width as usize).saturating_sub(8).clamp(20, 72);
                super::columns::wrap(&full, width)
            }
            _ => status_words(item),
        };
        if words.is_empty() {
            return;
        }
        let lines = words
            .into_iter()
            .map(|w| ratatui::text::Line::styled(w, Style::default().fg(th().ui.text)))
            .collect();
        let (col, row) = self.hover_pos;
        crate::tooltip::paint_box(frame, body, col, row + 1, lines);
    }

    fn paint_chrome(&mut self, buf: &mut Buffer, body: Rect, hover: Option<Hit>) {
        let text = Style::default().fg(th().ui.text);
        let muted = Style::default().fg(th().ui.muted);
        let right = body.right();
        // Title and its two buttons, right-aligned.
        let y = body.y;
        put(
            buf,
            body.x + 1,
            y,
            msg::radar::TITLE.get(),
            body.width.saturating_sub(2),
            text.add_modifier(Modifier::BOLD),
        );
        let back = format!(" {} ", msg::radar::BTN_BACK.get());
        let refresh = format!(" {} ", msg::radar::BTN_REFRESH.get());
        let back_w = display_width(&back) as u16;
        let refresh_w = display_width(&refresh) as u16;
        if body.width > back_w + refresh_w + 24 {
            let back_x = right - back_w - 1;
            let refresh_x = back_x - refresh_w - 1;
            let r = put(
                buf,
                back_x,
                y,
                &back,
                back_w,
                button(hover == Some(Hit::Back)),
            );
            self.hits.push((r, Hit::Back));
            let r = put(
                buf,
                refresh_x,
                y,
                &refresh,
                refresh_w,
                button(hover == Some(Hit::Refresh)),
            );
            self.hits.push((r, Hit::Refresh));
        }

        // The three comboboxes.
        let y = body.y + 2;
        let mut x = body.x + 1;
        let widths = {
            let free = body.width.saturating_sub(2 + 3 * 7) as usize;
            [free * 38 / 100, free * 26 / 100, free * 30 / 100].map(|w| w.clamp(10, 30) as u16)
        };
        let labels = [
            msg::radar::LABEL_SCOPE.get(),
            msg::radar::LABEL_VIEW.get(),
            msg::radar::LABEL_SHOW.get(),
        ];
        for (i, which) in [Hit::Scope, Hit::View, Hit::Who].into_iter().enumerate() {
            let label = format!("{} ", labels[i]);
            let lw = display_width(&label) as u16;
            if x + lw + widths[i] > right {
                break;
            }
            put(buf, x, y, &label, lw, muted);
            x += lw;
            let field = Rect::new(x, y, widths[i], 1);
            let hovered = hover == Some(which);
            let combo = self.combo_of(which);
            let open = combo.is_open();
            combo.render_field(buf, field, open, hovered);
            self.hits.push((field, which));
            x += widths[i] + 2;
        }

        // The text filter.
        let y = body.y + 3;
        let focused = self.filter_focused;
        put(
            buf,
            body.x + 1,
            y,
            "/ ",
            2,
            Style::default().fg(if focused {
                th().ui.accent
            } else {
                th().ui.muted
            }),
        );
        // The two account checkboxes sit after the field, when an account-wide
        // list is being asked; the field gives them the room.
        let boxes = self.account_checkboxes();
        let boxes_w: u16 = boxes
            .iter()
            .map(|(text, _)| display_width(text) as u16 + 2)
            .sum();
        let fw = body
            .width
            .saturating_sub(4 + boxes_w)
            .min(64)
            .max(10.min(body.width.saturating_sub(4)));
        let field = Rect::new(body.x + 3, y, fw, 1);
        let bg = crate::input_bg(focused, hover == Some(Hit::Filter));
        fill(buf, field, Style::default().bg(bg));
        self.edit.begin_frame();
        self.edit.drawn((), field, false);
        if self.query.is_empty() && !focused {
            put(
                buf,
                field.x + 1,
                y,
                msg::radar::FILTER_PROMPT.get(),
                fw.saturating_sub(1),
                Style::default().fg(th().ui.muted).bg(bg),
            );
        } else {
            let style = Style::default().fg(th().ui.text).bg(bg);
            let look =
                crate::text_field::EditLook::new(style, "_").caret_style(style.fg(th().ui.accent));
            let line = crate::text_field::edit_line(
                &self.query,
                self.edit.caret(()),
                fw as usize,
                focused,
                look,
            );
            buf.set_line(field.x, y, &line, fw);
        }
        self.hits.push((field, Hit::Filter));
        let mut bx = field.right() + 2;
        for (text, hit) in boxes {
            let w = display_width(&text) as u16;
            if bx + w > right {
                break;
            }
            let style = if hover == Some(hit) {
                Style::default().fg(th().ui.text).bg(th().ui.hover)
            } else {
                Style::default().fg(th().ui.text)
            };
            let r = put(buf, bx, y, &text, w, style);
            self.hits.push((r, hit));
            bx += w + 2;
        }

        // The status line: the spinner after 150 ms, else the counts.
        let y = body.y + 4;
        let slow = self
            .started
            .map(|t| t.elapsed())
            .filter(|e| self.in_flight && *e >= std::time::Duration::from_millis(150));
        let status = match slow {
            Some(elapsed) => msg::radar::status_loading(crate::activity::spinner_frame(elapsed)),
            None => {
                let items = self.board.items.len() as u64;
                let repos = self.repos.len() as u64;
                let mut line = format!(
                    "{}  \u{00B7}  {}",
                    msg::radar::status_items(items),
                    msg::radar::status_repos(repos)
                );
                if scope::is_folder_scope(&self.scope) {
                    let unlinked = self.unlinked_count();
                    if unlinked > 0 {
                        line.push_str(&format!(
                            "  \u{00B7}  {}",
                            msg::radar::status_unlinked(unlinked)
                        ));
                    }
                }
                // The selected request's checks and conflicts: the keyboard's
                // twin of the glyphs' tooltip.
                if let Some(item) = match self.rows.get(self.selected) {
                    Some(Row::Item(i)) => self.board.items.get(*i),
                    _ => None,
                } {
                    let mut words = status_words(item);
                    if item.repo.is_none() {
                        words.insert(0, msg::radar::NOT_CLONED.get().to_string());
                    }
                    // The full project name and title, whatever the columns cut.
                    let name = Self::cut_text(item, Cell::Repo);
                    line.push_str(&format!(
                        "  \u{00B7}  {name} {} {}",
                        item.label,
                        row_title(item)
                    ));
                    if !words.is_empty() {
                        line.push_str(&format!(": {}", words.join(", ")));
                    }
                }
                line
            }
        };
        put(
            buf,
            body.x + 1,
            y,
            &status,
            body.width.saturating_sub(2),
            muted,
        );
    }

    fn paint_list(&mut self, buf: &mut Buffer, body: Rect, hover: Option<Hit>) {
        let top = body.y + CHROME_TOP;
        let height = body.height.saturating_sub(CHROME_TOP + CHROME_BOTTOM) as usize;
        self.list_height = height.max(1);
        if self.follow && height > 0 {
            if self.selected < self.scroll {
                self.scroll = self.selected;
            } else if self.selected >= self.scroll + height {
                self.scroll = self.selected + 1 - height;
            }
        }
        self.scroll = self
            .scroll
            .min(self.rows.len().saturating_sub(height.max(1)));
        // A list longer than its room keeps the last column for the bar.
        self.bar = crate::scrollbar::Track::column(
            Rect::new(body.right().saturating_sub(1), top, 1, height as u16),
            self.rows.len(),
            height,
            self.scroll,
        );
        let body = if self.bar.is_some() {
            Rect::new(body.x, body.y, body.width - 1, body.height)
        } else {
            body
        };
        // Column widths measured over every item, so they do not jump while
        // scrolling.
        let content = self
            .board
            .items
            .iter()
            .map(|i| display_width(&i.repo_name))
            .max()
            .unwrap_or(4)
            // The header's title is never cut by an automatic width.
            .max(display_width(msg::radar::COLUMN_REPOSITORY.get()));
        let (chip_w, repo_max) =
            super::columns::repo_width(content, self.state.columns.repo, body.width);
        let author_content = self
            .board
            .items
            .iter()
            .map(|i| display_width(&i.author))
            .max()
            .filter(|w| *w > 0)
            .map_or(0, |w| w.max(display_width(msg::radar::COLUMN_AUTHOR.get())));
        let author_w = super::columns::author_width(author_content, self.state.columns.author);
        self.repo_max = repo_max;
        let label_w = self
            .board
            .items
            .iter()
            .map(|i| display_width(&i.label))
            .max()
            .unwrap_or(0)
            .clamp(0, 9);
        // Room for the checks / conflict glyphs when any row has them.
        let status_w = if self.board.items.iter().any(has_status) {
            3
        } else {
            0
        };
        // The number column: the widest label and the check marks, unless sized.
        let num_w = super::columns::num_width(label_w, status_w, self.state.columns.number);
        let now = (self.now)();
        // The state chip and the age get one width for every row, so the
        // author column between them starts at the same cell on each.
        let state_w = self
            .board
            .items
            .iter()
            .map(|i| display_width(&state_chip(i)))
            .max()
            .unwrap_or(0);
        let age_content = self
            .board
            .items
            .iter()
            .filter_map(|i| iso_to_unix(&i.updated_at).map(|t| age_label(now, t)))
            .map(|a| display_width(&a))
            .max()
            .filter(|w| *w > 0)
            .map_or(0, |w| w.max(display_width(msg::radar::COLUMN_AGE.get())));
        let age_w = super::columns::age_width(age_content, self.state.columns.age);
        self.col_eff = [chip_w, author_w, if label_w > 0 { num_w } else { 0 }, age_w];
        self.col_auto = [
            super::columns::repo_width(content, None, body.width).0,
            super::columns::author_width(author_content, None),
            super::columns::num_width(label_w, status_w, None),
            super::columns::age_width(age_content, None),
        ];
        self.col_content = [
            content,
            author_content,
            super::columns::num_width(label_w, status_w, None),
            age_content,
        ];
        let widths = Widths {
            chip: chip_w,
            label: label_w,
            num: num_w,
            status: status_w,
            author: author_w,
            state: state_w,
            age: age_w,
        };
        let geo = Geo::new(body, widths);
        let header_y = top - 1;
        let mode = crate::column_resize::mode();
        // The separators first: a hit rect registered before the rows wins
        // over the row band that crosses it. `line` mode: the gap cell down
        // the header and the list; `header` mode: the first cell of the next
        // column on the header row.
        let edges = geo.edges();
        let rects: Vec<(Cell, Rect)> = edges
            .iter()
            .map(|(cell, edge)| {
                (
                    *cell,
                    crate::column_resize::grab_rect(mode, *edge, header_y, top + height as u16),
                )
            })
            .collect();
        for (cell, rect) in &rects {
            self.hits.push((*rect, Hit::Divider(*cell)));
        }
        self.paint_header(buf, header_y, &geo);
        // The titles of the sizable columns: a right-click opens the fit
        // menu. After the separators, which keep their cells.
        let titles = [
            Some((Cell::Repo, geo.repo_x, geo.chip_w)),
            geo.label_x.map(|x| (Cell::Num, x, geo.num_w)),
            geo.author.map(|(x, w)| (Cell::Author, x, w)),
            geo.age.map(|(x, w)| (Cell::Age, x, w)),
        ];
        for (cell, x, w) in titles.into_iter().flatten() {
            if w > 0 {
                self.hits
                    .push((Rect::new(x, header_y, w as u16, 1), Hit::Header(cell)));
            }
        }
        let rows = self.rows.clone();
        for (offset, row) in rows.iter().enumerate().skip(self.scroll).take(height) {
            let y = top + (offset - self.scroll) as u16;
            let band = Rect::new(body.x, y, body.width, 1);
            let selected = offset == self.selected && !self.filter_focused;
            let hovered = matches!(
                (hover, row),
                (Some(Hit::Row(i)), _) if i == offset
            ) || matches!(
                (hover, row),
                (Some(Hit::Cut(i, _)), _) if i == offset
            ) || matches!(
                (hover, row),
                (Some(Hit::Heading(i)), Row::Heading(..)) if i == offset
            );
            if selected {
                fill(buf, band, Style::default().bg(th().ui.selection));
            } else if hovered {
                fill(buf, band, Style::default().bg(th().ui.hover));
            }
            let bg = if selected {
                Some(th().ui.selection)
            } else if hovered {
                Some(th().ui.hover)
            } else {
                None
            };
            match row {
                Row::Heading(heading, count) => {
                    self.paint_heading(buf, band, offset, *heading, *count, bg);
                }
                Row::Item(i) => {
                    let Some(item) = self.board.items.get(*i).cloned() else {
                        continue;
                    };
                    self.paint_item(buf, band, offset, &item, &geo, widths, now, bg, hover);
                }
                Row::Note(i) => self.paint_note(buf, band, offset, *i, bg, hover),
                Row::Hint(i) => self.paint_hint(buf, band, offset, *i, bg, hover),
                // Spacer: no fill (never selected), no hit rect, no hover.
                Row::Blank => {}
                Row::Empty => {
                    let text = if self.repos.is_empty() {
                        msg::radar::EMPTY_SCOPE.get()
                    } else {
                        msg::radar::EMPTY.get()
                    };
                    put(
                        buf,
                        band.x + 3,
                        y,
                        text,
                        band.width.saturating_sub(4),
                        Style::default()
                            .fg(th().ui.muted)
                            .add_modifier(Modifier::ITALIC),
                    );
                }
            }
        }
        if let Some(track) = self.bar {
            crate::scrollbar::render_buf(buf, track, true, false);
        }
        // The separators last, over the rows: a thin line (or the header
        // block) while the pointer is on one or it is being sized, else the
        // resting tick in the header.
        for (cell, rect) in rects {
            let lit = hover == Some(Hit::Divider(cell))
                || self.col_pick == Some(cell)
                || self.col_drag.is_some_and(|d| d.is_for(cell));
            if lit {
                crate::column_resize::paint_grab(buf, mode, rect);
            } else if let Some((_, edge)) = edges.iter().find(|(c, _)| *c == cell) {
                crate::column_resize::paint_tick(buf, mode, edge.gap_x, header_y);
            }
        }
    }

    /// The column titles over the list: the columns the rows show, at the
    /// cells the rows use.
    fn paint_header(&self, buf: &mut Buffer, y: u16, geo: &Geo) {
        let style = Style::default()
            .fg(th().ui.muted)
            .add_modifier(Modifier::BOLD);
        let title = |buf: &mut Buffer, x: u16, width: usize, text: &str| {
            if width > 0 {
                put(buf, x, y, text, width as u16, style);
            }
        };
        title(
            buf,
            geo.repo_x,
            geo.chip_w,
            msg::radar::COLUMN_REPOSITORY.get(),
        );
        if let Some(x) = geo.label_x {
            title(buf, x, geo.num_w, "#");
        }
        title(
            buf,
            geo.title_x,
            geo.title_w,
            msg::radar::COLUMN_TITLE.get(),
        );
        if let Some((x, width)) = geo.author {
            title(buf, x, width, msg::radar::COLUMN_AUTHOR.get());
        }
        if let Some((x, width)) = geo.age {
            title(buf, x, width, msg::radar::COLUMN_AGE.get());
        }
    }

    fn paint_heading(
        &mut self,
        buf: &mut Buffer,
        band: Rect,
        offset: usize,
        heading: Heading,
        count: usize,
        bg: Option<Color>,
    ) {
        let style = |s: Style| match bg {
            Some(bg) => s.bg(bg),
            None => s,
        };
        let folded = self.folded.contains(&heading);
        let chevron = &format!(" {} ", crate::fold_mark::glyph(folded));
        let r = put(
            buf,
            band.x,
            band.y,
            chevron,
            3,
            style(Style::default().fg(th().ui.accent)),
        );
        let title = heading_title(heading).to_uppercase();
        let r2 = put(
            buf,
            r.right(),
            band.y,
            &title,
            band.width.saturating_sub(4),
            style(
                Style::default()
                    .fg(th().ui.text)
                    .add_modifier(Modifier::BOLD),
            ),
        );
        put(
            buf,
            r2.right(),
            band.y,
            &format!("  {count}"),
            8,
            style(Style::default().fg(th().ui.muted)),
        );
        self.hits.push((band, Hit::Heading(offset)));
    }

    #[allow(clippy::too_many_arguments)]
    fn paint_item(
        &mut self,
        buf: &mut Buffer,
        band: Rect,
        offset: usize,
        item: &Item,
        geo: &Geo,
        w: Widths,
        now: i64,
        bg: Option<Color>,
        hover: Option<Hit>,
    ) {
        let chip_w = w.chip;
        let st = |s: Style| match bg {
            Some(bg) => s.bg(bg),
            None => s,
        };
        let text = st(Style::default().fg(th().ui.text));
        let muted = st(Style::default().fg(th().ui.muted));
        let y = band.y;
        let mut x = band.x + 1;

        // Inner controls first: the star and the menu button.
        let pinned = self.state.is_pinned(&item.key);
        let star_hover = hover == Some(Hit::Star(offset));
        let r = put(
            buf,
            x,
            y,
            if pinned { STAR } else { " " },
            1,
            st(Style::default().fg(if star_hover {
                th().ui.accent
            } else {
                th().status.warning
            })),
        );
        self.hits.push((Rect::new(r.x, y, 1, 1), Hit::Star(offset)));
        x += 2;
        let (icon, icon_color) = match item.kind {
            ItemKind::Pr => (
                PR_ICON,
                if item.draft {
                    th().ui.muted
                } else {
                    th().refs.local
                },
            ),
            ItemKind::Issue => (ISSUE_ICON, th().refs.local),
            ItemKind::Wip => (WIP_ICON, th().status.warning),
        };
        put(buf, x, y, icon, 1, st(Style::default().fg(icon_color)));
        x += 2;
        // A repository with no known clone has a dimmed chip.
        let color = match item.repo.as_deref() {
            Some(path) => self.repo_color(path),
            None => th().ui.muted,
        };
        put(
            buf,
            x,
            y,
            &item.repo_name,
            chip_w as u16,
            st(Style::default().fg(color)),
        );
        if display_width(&item.repo_name) > chip_w {
            self.hits.push((
                Rect::new(x, y, chip_w as u16, 1),
                Hit::Cut(offset, Cell::Repo),
            ));
        }
        // The separator after the chip is the Radar's own (`paint_list`).
        x += chip_w as u16 + 1;
        if let Some(lx) = geo.label_x {
            let label_area = geo.num_w - geo.status_w;
            let r = put(buf, lx, y, &item.label, label_area as u16, muted);
            if display_width(&item.label) > label_area {
                self.hits.push((r, Hit::Cut(offset, Cell::Num)));
            }
            x = lx + geo.num_w as u16 + 1;
        }
        if let Some(lx) = geo.label_x.filter(|_| geo.status_w > 0) {
            // The marks sit at the column's right end, a cell after the label.
            let x = lx + (geo.num_w - geo.status_w) as u16 + 1;
            if has_status(item) {
                let (glyph, color) = match item.checks {
                    Some(ChecksState::Passing) => (CHECK_PASS, th().status.success),
                    Some(ChecksState::Failing) => (CHECK_FAIL, th().status.danger),
                    Some(ChecksState::Pending) => (CHECK_PENDING, th().status.warning),
                    None => (" ", th().ui.muted),
                };
                put(buf, x, y, glyph, 1, st(Style::default().fg(color)));
                if item.has_conflicts == Some(true) {
                    put(
                        buf,
                        x + 1,
                        y,
                        CONFLICT,
                        1,
                        st(Style::default()
                            .fg(th().status.danger)
                            .add_modifier(Modifier::BOLD)),
                    );
                }
                // Inner before the band that covers it.
                self.hits.push((Rect::new(x, y, 2, 1), Hit::Status(offset)));
            }
        }

        // The right edge, from the right: the menu button, the age, the
        // author, the state chip; dropped in that order when narrow.
        let edge = geo.edge;
        let more_hover = hover == Some(Hit::Menu(offset));
        let r = put(
            buf,
            edge,
            y,
            MORE,
            1,
            st(Style::default().fg(if more_hover {
                th().ui.accent
            } else {
                th().ui.muted
            })),
        );
        self.hits.push((Rect::new(r.x, y, 1, 1), Hit::Menu(offset)));
        let age = iso_to_unix(&item.updated_at)
            .map(|t| age_label(now, t))
            .unwrap_or_default();
        let state = state_chip(item);
        let title_w = geo.title_w;
        let title = row_title(item);
        let shown = put(buf, x, y, &title, title_w as u16, text);
        if display_width(&title) > shown.width as usize {
            self.hits.push((
                Rect::new(x, y, shown.width, 1),
                Hit::Cut(offset, Cell::Title),
            ));
        }
        // Labels after the title, when there is room.
        if !item.labels.is_empty() {
            let used = shown.width as usize;
            let left = title_w.saturating_sub(used + 2);
            if left > 6 {
                let labels = item.labels.join(" \u{00B7} ");
                let r = put(buf, shown.right() + 2, y, &labels, left as u16, muted);
                if display_width(&labels) > r.width as usize {
                    self.hits.push((r, Hit::Cut(offset, Cell::Labels)));
                }
            }
        }
        if let Some((tx, width)) = geo.state {
            put(
                buf,
                tx,
                y,
                &super::columns::fit_pad(&state, width),
                width as u16,
                st(Style::default().fg(th().status.warning)),
            );
        }
        if let Some((tx, width)) = geo.author {
            let r = put(
                buf,
                tx,
                y,
                &super::columns::fit_pad(&item.author, width),
                width as u16,
                muted,
            );
            if display_width(&item.author) > width {
                self.hits.push((r, Hit::Cut(offset, Cell::Author)));
            }
        }
        if let Some((tx, width)) = geo.age {
            let r = put(
                buf,
                tx,
                y,
                &super::columns::fit_pad(&age, width),
                width as u16,
                muted,
            );
            if display_width(&age) > width {
                self.hits.push((r, Hit::Cut(offset, Cell::Age)));
            }
        }
        self.hits.push((band, Hit::Row(offset)));
    }

    fn paint_note(
        &mut self,
        buf: &mut Buffer,
        band: Rect,
        offset: usize,
        index: usize,
        bg: Option<Color>,
        hover: Option<Hit>,
    ) {
        let Some(note) = self.notes.get(index).cloned() else {
            return;
        };
        let st = |s: Style| match bg {
            Some(bg) => s.bg(bg),
            None => s,
        };
        let y = band.y;
        let color = self.repo_color(&note.path);
        let r = put(
            buf,
            band.x + 3,
            y,
            &note.name,
            18,
            st(Style::default().fg(color)),
        );
        let mut right = band.right().saturating_sub(1);
        if let Some((act, _)) = &note.act {
            let label = format!(" {} ", super::act_label(*act));
            let w = display_width(&label) as u16;
            if right > r.right() + w + 4 {
                right -= w;
                let rect = put(
                    buf,
                    right,
                    y,
                    &label,
                    w,
                    button(hover == Some(Hit::NoteAction(offset))),
                );
                self.hits.push((rect, Hit::NoteAction(offset)));
            }
        }
        put(
            buf,
            r.right() + 2,
            y,
            &note.text,
            right.saturating_sub(r.right() + 3),
            st(Style::default().fg(th().status.warning)),
        );
        self.hits.push((band, Hit::Row(offset)));
    }

    /// The code host offering its account-wide review requests, with the
    /// `[Turn on]` button.
    fn paint_hint(
        &mut self,
        buf: &mut Buffer,
        band: Rect,
        offset: usize,
        index: usize,
        bg: Option<Color>,
        hover: Option<Hit>,
    ) {
        let Some(offer) = self.offers.get(index).cloned() else {
            return;
        };
        let st = |s: Style| match bg {
            Some(bg) => s.bg(bg),
            None => s,
        };
        let y = band.y;
        let mut right = band.right().saturating_sub(1);
        let label = format!(" {} ", msg::radar::ACT_TURN_ON.get());
        let w = display_width(&label) as u16;
        if right > band.x + w + 8 {
            right -= w;
            let rect = put(
                buf,
                right,
                y,
                &label,
                w,
                button(hover == Some(Hit::TurnOn(offset))),
            );
            self.hits.push((rect, Hit::TurnOn(offset)));
        }
        put(
            buf,
            band.x + 3,
            y,
            &msg::radar::offer(offer.label),
            right.saturating_sub(band.x + 4),
            st(Style::default().fg(th().ui.muted)),
        );
        self.hits.push((band, Hit::Row(offset)));
    }

    /// `[x] Not cloned` and `[ ] Drafts`, with what a click on each does.
    /// Nothing while no account-wide list is being asked.
    fn account_checkboxes(&self) -> Vec<(String, Hit)> {
        if self.active.is_empty() {
            return Vec::new();
        }
        let mark = |on: bool| if on { "[x]" } else { "[ ]" };
        vec![
            (
                format!(
                    "{} {}",
                    mark(self.state.show_not_cloned),
                    msg::radar::NOT_CLONED.get()
                ),
                Hit::NotCloned,
            ),
            (
                format!(
                    "{} {}",
                    mark(self.state.show_drafts),
                    msg::radar::CHECK_DRAFTS.get()
                ),
                Hit::Drafts,
            ),
        ]
    }

    fn paint_footer(&mut self, buf: &mut Buffer, body: Rect) {
        let y = body.bottom().saturating_sub(1);
        let (text, style) = match self.mode_hint.as_deref().or(self.message.as_deref()) {
            Some(message) => (
                format!(" {message}"),
                Style::default().fg(th().status.warning),
            ),
            None => (
                format!(" {}", msg::radar::HINT.get()),
                Style::default().fg(th().ui.muted),
            ),
        };
        put(buf, body.x, y, &text, body.width, style);
    }
}

/// The title cell's full text: a work-in-progress row says its branch and
/// what is waiting in it.
pub(super) fn row_title(item: &Item) -> String {
    match &item.wip {
        Some(wip) => wip_text(wip),
        None => item.title.clone(),
    }
}

/// `draft`, `review`: what the row's chip says about a request.
fn state_chip(item: &Item) -> String {
    if item.kind != ItemKind::Pr {
        return String::new();
    }
    if item.group == Group::NeedsReview {
        msg::radar::STATE_REVIEW.get().to_string()
    } else if item.draft {
        msg::radar::STATE_DRAFT.get().to_string()
    } else {
        String::new()
    }
}

/// A WIP row's words: the branch and what is waiting in it.
fn wip_text(wip: &super::model::WipFacts) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(branch) = &wip.branch {
        parts.push(branch.clone());
    }
    if wip.modified > 0 {
        parts.push(format!("{WIP_ICON} {}", wip.modified));
    }
    if wip.added > 0 {
        parts.push(format!("+ {}", wip.added));
    }
    if wip.ahead > 0 {
        parts.push(format!("\u{2191} {}", wip.ahead));
    }
    if wip.behind > 0 {
        parts.push(format!("\u{2193} {}", wip.behind));
    }
    parts.join("  ")
}
