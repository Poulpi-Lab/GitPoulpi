//! Painting the panel. Every hit rect comes out of the walk that paints
//! (`dialog_kit` returns the rect each piece landed in), and while an
//! overlay is up only the overlay's rects are registered: nothing behind a
//! popup hovers or answers a click.

use gitpoulpi_core::domain::{short, JournalState};
use gitpoulpi_i18n::msg::undo_safety as m;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders, Widget};

use super::{label, rows, Act, Confirm, ModePick, NamePrompt, Overlay, RowMenu, SafetyPanel, Tab};
use crate::dialog_kit::{self, ButtonState};
use crate::{display_width, th};

/// One list row: `(label, age, tag, tag style, dimmed)`. Age and tag are
/// separate so the list can give each a column of its own.
type RowCells = (String, String, String, Style, bool);

/// Cells of the icon column of a history row.
const ICON_W: usize = 2;

/// Pads `s` with spaces on the left to `width` display cells.
fn pad_left(s: &str, width: usize) -> String {
    format!("{}{s}", " ".repeat(width.saturating_sub(display_width(s))))
}

/// Pads `s` with spaces on the right to `width` display cells.
fn pad_right(s: &str, width: usize) -> String {
    format!("{s}{}", " ".repeat(width.saturating_sub(display_width(s))))
}

/// `-- text ----...` over `width` cells: a thin labelled rule.
fn rule(text: &str, width: usize) -> String {
    let head = format!("\u{2500}\u{2500} {text} ");
    let used = display_width(&head);
    format!("{head}{}", "\u{2500}".repeat(width.saturating_sub(used)))
}

/// Where a click or hover lands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Hit {
    Tab(Tab),
    Row(usize),
    Act(Act),
    Close,
    Cancel,
    Confirm,
    NameField,
    ModeField,
    /// A popup's body: swallows a click that misses its buttons.
    Body,
    MenuItem(usize),
}

pub(crate) fn safety_rect(frame: Rect) -> Rect {
    dialog_kit::centered(
        frame,
        frame.width.saturating_sub(4).min(124),
        frame.height.saturating_sub(2).min(34),
    )
}

fn act_label(act: Act, panel: &SafetyPanel, state: &JournalState) -> String {
    match act {
        Act::UndoTo => m::BTN_UNDO_TO.get().to_string(),
        Act::RedoTo => m::BTN_REDO_TO.get().to_string(),
        Act::RestoreOnly => m::BTN_RESTORE_ONLY.get().to_string(),
        Act::Offer => panel
            .offer(state)
            .map(|(name, _, op)| m::btn_offer(name, op))
            .unwrap_or_default(),
        Act::ShowInLog => m::BTN_SHOW_IN_LOG.get().to_string(),
        Act::CreateBranch => m::BTN_CREATE_BRANCH.get().to_string(),
        Act::RestoreRef => m::btn_restore_ref(
            panel
                .reflog_target()
                .unwrap_or_else(|| panel.reflog.refname.clone()),
        ),
        Act::CherryPick => m::BTN_CHERRY_PICK.get().to_string(),
        Act::CreateTag => m::BTN_CREATE_TAG.get().to_string(),
        Act::ApplyToWorktree => m::BTN_APPLY.get().to_string(),
        Act::SaveAsStash => m::BTN_STASH.get().to_string(),
        Act::LoadMore => m::BTN_LOAD_MORE.get().to_string(),
    }
}

fn style_text() -> Style {
    Style::default().fg(th().ui.text).bg(th().ui.panel)
}

fn style_muted() -> Style {
    Style::default().fg(th().ui.muted).bg(th().ui.panel)
}

fn style_row(selected: bool, hovered: bool) -> Style {
    let bg = if selected {
        th().ui.selection
    } else if hovered {
        th().ui.hover
    } else {
        th().ui.panel
    };
    Style::default().fg(th().ui.text).bg(bg)
}

impl SafetyPanel {
    pub(crate) fn render(&mut self, rect: Rect, buf: &mut Buffer) {
        let state = self.journal_state();
        let title = match self.tab {
            Tab::History => m::TITLE_HISTORY.get(),
            Tab::Reflog => m::TITLE_REFLOG.get(),
            Tab::Lost => m::TITLE_LOST.get(),
        };
        self.frame = rect;
        self.close.panel(buf, rect, title);
        let mut hits: Vec<(Rect, Hit)> = Vec::new();
        let inner = Rect::new(
            rect.x + 2,
            rect.y + 1,
            rect.width.saturating_sub(4),
            rect.height.saturating_sub(2),
        );
        if inner.width < 40 || inner.height < 12 {
            self.hits = hits;
            return;
        }
        let modal = self.overlay.is_some();
        self.render_tabs(buf, inner, &mut hits);
        self.render_header(buf, inner, &state);

        let detail_rows: u16 = if self.tab == Tab::History { 3 } else { 0 };
        let bottom_rows = 3 + detail_rows;
        let body_top = inner.y + 2;
        let body_h = inner.height.saturating_sub(2 + bottom_rows);
        let (list_w, side) = if self.tab == Tab::History {
            (inner.width, None)
        } else {
            let list_w = inner.width * 55 / 100;
            (
                list_w,
                Some(Rect::new(
                    inner.x + list_w + 1,
                    body_top,
                    inner.width.saturating_sub(list_w + 1),
                    body_h,
                )),
            )
        };
        let body = Rect::new(inner.x, body_top, list_w, body_h);
        self.list_body = body;
        self.render_list(buf, body, &state, &mut hits);
        if let Some(side) = side {
            self.render_side(buf, side);
        }

        let mut y = body_top + body_h;
        if self.tab == Tab::History {
            self.render_history_detail(buf, Rect::new(inner.x, y, inner.width, 3), &state);
            y += 3;
        }
        // The note under the lists: a result, or why something did not work.
        if let Some(job) = &self.job {
            let spin = crate::activity::spinner_frame(
                self.lost
                    .scan_elapsed()
                    .max(std::time::Duration::from_millis(1)),
            );
            let text = format!("{spin} {}", job.label());
            dialog_kit::put(buf, inner.x, y, &text, inner.width, style_text());
        } else if let Some(note) = &self.note {
            let style = if note.error {
                Style::default().fg(th().status.danger).bg(th().ui.panel)
            } else {
                Style::default().fg(th().status.success).bg(th().ui.panel)
            };
            dialog_kit::put(buf, inner.x, y, &note.text, inner.width, style);
        }
        y += 1;
        self.render_buttons(buf, inner, y, &state, &mut hits);
        y += 1;
        self.render_hints(buf, inner, y);

        if modal {
            hits.clear();
        }
        self.render_overlay(buf, rect, &mut hits);
        crate::pointer_map::from_hits(&hits, |h| match h {
            Hit::NameField => Some(crate::pointer_map::Kind::Field),
            Hit::Body => None,
            // A row (a rule, a menu entry) lights on hover and keeps the arrow.
            Hit::Row(_) | Hit::MenuItem(_) => Some(crate::pointer_map::Kind::Plain),
            _ => Some(crate::pointer_map::Kind::Hand),
        });
        self.hits = hits;
    }

    fn render_tabs(&mut self, buf: &mut Buffer, inner: Rect, hits: &mut Vec<(Rect, Hit)>) {
        let mut x = inner.x;
        let tabs = [
            (Tab::History, m::TAB_HISTORY.get().to_string()),
            (
                Tab::Reflog,
                m::tab_reflog(self.reflog.refname.trim_start_matches("refs/heads/")),
            ),
            (Tab::Lost, m::TAB_LOST.get().to_string()),
        ];
        for (tab, label) in tabs {
            let hovered = self.hover == Some(Hit::Tab(tab));
            let state = if self.tab == tab {
                ButtonState::Primary {
                    hovered,
                    focused: false,
                }
            } else {
                ButtonState::Normal {
                    hovered,
                    focused: false,
                }
            };
            let rect = dialog_kit::button(buf, x, inner.y, &label, state);
            hits.push((rect, Hit::Tab(tab)));
            x += rect.width + 1;
        }
    }

    fn render_header(&self, buf: &mut Buffer, inner: Rect, state: &JournalState) {
        let y = inner.y + 1;
        let text = match self.tab {
            Tab::History => {
                let n = self.history.undo_count as u64;
                let mut text = if self.max_depth == 0 {
                    m::header_unlimited(n)
                } else {
                    m::header_count(n, self.max_depth as u64)
                };
                let redo = (self.history.rows.len() - self.history.undo_count) as u64;
                if redo > 0 {
                    text.push_str("  |  ");
                    text.push_str(&m::header_redo(redo));
                }
                if state.disabled {
                    text = gitpoulpi_i18n::msg::undo_ui::DISABLED.get().to_string();
                }
                text
            }
            Tab::Reflog => m::reflog_for(self.reflog.refname.trim_start_matches("refs/heads/")),
            Tab::Lost => m::LOST_INTRO.get().to_string(),
        };
        dialog_kit::put(buf, inner.x, y, &text, inner.width, style_muted());
    }

    /// Row text for the open tab: `(left, right_columns, right style)`.
    fn row_cells(&self, index: usize, state: &JournalState) -> Option<RowCells> {
        match self.tab {
            Tab::History => {
                let at = self.history.row_at(index)?;
                let row = self.history.rows.get(at)?;
                let st = rows::row_state(row, at, state);
                // One fixed-width icon column, one glyph per action family.
                let icon = pad_right(label::family(row.entry.summary.kind).glyph(), ICON_W);
                let tag = rows::tag(row, &st);
                let style = match st {
                    rows::RowState::Blocked(_) => Style::default().fg(th().status.warning),
                    rows::RowState::Undone => Style::default().fg(th().ui.muted),
                    rows::RowState::Undoable => Style::default().fg(th().ui.accent),
                };
                Some((
                    format!("{icon} {}", label::entry_text(&row.entry)),
                    rows::age(row.entry.summary.at),
                    tag,
                    style,
                    row.redo,
                ))
            }
            Tab::Reflog => {
                if self.reflog.is_more_row(index) {
                    // The row that asks for the next page (Enter, a click, or
                    // the cursor getting near does it on its own).
                    let text = if self.reflog.fetching_more() {
                        m::LOADING.get().to_string()
                    } else {
                        format!("... {}", m::BTN_LOAD_MORE.get())
                    };
                    return Some((
                        text,
                        String::new(),
                        String::new(),
                        Style::default().fg(th().ui.accent),
                        false,
                    ));
                }
                let e = self.reflog.entries.get(index)?;
                Some((
                    format!("{}  {}", short(e.new), e.message),
                    rows::age(e.time),
                    rows::person(&e.committer).to_string(),
                    Style::default().fg(th().ui.muted),
                    false,
                ))
            }
            Tab::Lost => {
                let c = self.lost.commits.get(index)?;
                let source = match c.source {
                    gitpoulpi_core::domain::LostSource::Reflog => m::SOURCE_REFLOG.get(),
                    gitpoulpi_core::domain::LostSource::Pin => m::SOURCE_PIN.get(),
                    gitpoulpi_core::domain::LostSource::Dangling => m::SOURCE_DANGLING.get(),
                };
                Some((
                    format!("{}  {}", short(c.oid), c.summary),
                    rows::age(c.time),
                    format!("[{source}]"),
                    Style::default().fg(th().ui.accent),
                    false,
                ))
            }
        }
    }

    fn row_count(&self) -> usize {
        match self.tab {
            Tab::History => self.history.lines.len(),
            Tab::Reflog => self.reflog.rows(),
            Tab::Lost => self.lost.commits.len(),
        }
    }

    pub(crate) fn list_mut(&mut self) -> &mut super::ListState {
        match self.tab {
            Tab::History => &mut self.history.list,
            Tab::Reflog => &mut self.reflog.list,
            Tab::Lost => &mut self.lost.list,
        }
    }

    pub(crate) fn list_len(&self) -> usize {
        self.row_count()
    }

    fn render_list(
        &mut self,
        buf: &mut Buffer,
        body: Rect,
        state: &JournalState,
        hits: &mut Vec<(Rect, Hit)>,
    ) {
        Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(th().ui.muted))
            .style(Style::default().bg(th().ui.panel))
            .render(body, buf);
        let content = Rect::new(
            body.x + 1,
            body.y + 1,
            body.width.saturating_sub(2),
            body.height.saturating_sub(2),
        );
        let len = self.row_count();
        let view_rows = content.height as usize;
        {
            let list = self.list_mut();
            list.view_rows = view_rows;
            list.clamp(len);
            list.offset = list.offset.min(len.saturating_sub(view_rows.max(1)));
        }
        if len == 0 {
            let text = self.empty_text();
            dialog_kit::put(
                buf,
                content.x + 1,
                content.y,
                &text,
                content.width.saturating_sub(1),
                style_muted(),
            );
            return;
        }
        if self.tab == Tab::History && self.history.place_scroll {
            // First paint: show the undone entries above the newest undo
            // entry (as many as fit), a few lines of the stack under it.
            self.history.place_scroll = false;
            let sel = self.history.list.selected;
            self.history.list.offset = (sel + 4).saturating_sub(view_rows).min(sel);
            self.history.snap(1);
        }
        let list = *self.list_mut();
        let cell_x = content.x;
        // Age and tag columns: one width for the whole visible list, the tag
        // column reserved even on rows without a tag, so every age lines up.
        let (mut age_w, mut tag_w) = (0usize, 0usize);
        for k in 0..view_rows {
            if list.offset + k >= len {
                break;
            }
            let Some((_, age, tag, _, _)) = self.row_cells(list.offset + k, state) else {
                continue;
            };
            age_w = age_w.max(display_width(&age));
            tag_w = tag_w.max(display_width(&tag));
        }
        for k in 0..view_rows {
            let index = list.offset + k;
            if index >= len {
                break;
            }
            let y = content.y + k as u16;
            if self.tab == Tab::History {
                if let Some(text) = self.separator_text(index) {
                    let width = content.width.saturating_sub(2);
                    let line = rule(&text, width as usize);
                    dialog_kit::put(buf, cell_x + 1, y, &line, width, style_muted());
                    continue;
                }
            }
            let Some((left, age, tag, tag_style, dim)) = self.row_cells(index, state) else {
                continue;
            };
            let selected = index == list.selected;
            let hovered = self.hover == Some(Hit::Row(index));
            let row_rect = Rect::new(cell_x, y, content.width, 1);
            dialog_kit::fill(buf, row_rect, style_row(selected, hovered));
            let right = if tag_w > 0 {
                format!("{}  {}", pad_left(&age, age_w), pad_right(&tag, tag_w))
            } else {
                pad_left(&age, age_w)
            };
            let right_w = display_width(&right) as u16;
            let left_w = content.width.saturating_sub(right_w + 3);
            let left_style = if dim {
                style_row(selected, hovered).fg(th().ui.muted)
            } else {
                style_row(selected, hovered)
            };
            dialog_kit::put(buf, cell_x + 1, y, &left, left_w, left_style);
            if content.width > right_w + 2 {
                let style = tag_style.bg(style_row(selected, hovered).bg.unwrap_or(th().ui.panel));
                dialog_kit::put(
                    buf,
                    cell_x + content.width - right_w - 1,
                    y,
                    &right,
                    right_w,
                    style,
                );
            }
            hits.push((row_rect, Hit::Row(index)));
        }
        if let Some(track) = crate::scrollbar::Track::vertical(body, len, list.offset) {
            crate::scrollbar::render_buf(buf, track, true, false);
        }
    }

    /// The words of a separator line of the history list, `None` on an entry.
    fn separator_text(&self, line: usize) -> Option<String> {
        match self.history.lines.get(line)? {
            super::Line::Row(_) => None,
            super::Line::Day(day) => Some(rows::day_heading(*day)),
            super::Line::Redo => Some(
                gitpoulpi_i18n::msg::undo_label::DIVIDER_REDO
                    .get()
                    .to_string(),
            ),
        }
    }

    fn empty_text(&self) -> String {
        match self.tab {
            Tab::History => match (&self.history.error, self.history.loaded) {
                (Some(e), _) => e.clone(),
                (None, true) => m::HISTORY_EMPTY.get().to_string(),
                (None, false) => m::LOADING.get().to_string(),
            },
            Tab::Reflog => match (&self.reflog.error, self.reflog.loaded) {
                (Some(e), _) => e.clone(),
                (None, true) => m::reflog_empty(self.reflog.refname.clone()),
                (None, false) => m::LOADING.get().to_string(),
            },
            Tab::Lost => match (&self.lost.error, self.lost.scanning(), self.lost.loaded) {
                (Some(e), _, _) => e.clone(),
                (None, true, _) => m::LOST_SCANNING.get().to_string(),
                (None, false, _) => m::LOST_EMPTY.get().to_string(),
            },
        }
    }

    fn render_side(&self, buf: &mut Buffer, side: Rect) {
        Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(th().ui.muted))
            .style(Style::default().bg(th().ui.panel))
            .render(side, buf);
        let inner = Rect::new(
            side.x + 2,
            side.y + 1,
            side.width.saturating_sub(4),
            side.height.saturating_sub(2),
        );
        let (detail, oid) = match self.tab {
            Tab::Reflog => (&self.reflog.detail, self.selected_reflog().map(|e| e.new)),
            Tab::Lost => (&self.lost.detail, self.selected_lost().map(|c| c.oid)),
            Tab::History => return,
        };
        let Some(oid) = oid else {
            return;
        };
        let mut y = inner.y;
        let bottom = inner.y + inner.height;
        let line = |buf: &mut Buffer, text: &str, style: Style, y: &mut u16| {
            if *y < bottom {
                dialog_kit::put(buf, inner.x, *y, text, inner.width, style);
                *y += 1;
            }
        };
        line(
            buf,
            &short(oid),
            Style::default()
                .fg(th().ui.accent)
                .bg(th().ui.panel)
                .add_modifier(Modifier::BOLD),
            &mut y,
        );
        let Some(loaded) = detail.current() else {
            let spin = crate::activity::spinner_frame(self.lost.scan_elapsed());
            let text = format!("{spin} {}", m::DETAIL_LOADING.get());
            line(buf, &text, style_muted(), &mut y);
            return;
        };
        if let Some(node) = &loaded.node {
            line(
                buf,
                &m::detail_author(&node.author_name, rows::age(node.timestamp)),
                style_muted(),
                &mut y,
            );
            y += 1;
            for text in node.message.lines().take(6) {
                line(buf, text, style_text(), &mut y);
            }
            y += 1;
        }
        if loaded.file_count > 0 {
            line(
                buf,
                &m::detail_files(loaded.file_count as u64),
                style_muted(),
                &mut y,
            );
            for (status, path) in &loaded.files {
                let text = format!("{}  {}", rows::status_letter(*status), path);
                line(buf, &text, style_text(), &mut y);
            }
            let more = loaded.file_count.saturating_sub(loaded.files.len());
            if more > 0 {
                line(
                    buf,
                    &m::detail_more_files(more as u64),
                    style_muted(),
                    &mut y,
                );
            }
        }
    }

    fn render_history_detail(&self, buf: &mut Buffer, area: Rect, state: &JournalState) {
        let Some(row) = self.selected_history() else {
            return;
        };
        let st = rows::row_state(row, self.history.selected_row().unwrap_or(0), state);
        let lines = rows::history_detail(row, &st);
        for (k, text) in lines.iter().enumerate().take(area.height as usize) {
            let style = match (&st, k) {
                (rows::RowState::Blocked(_), 0) => {
                    Style::default().fg(th().status.warning).bg(th().ui.panel)
                }
                _ => style_muted(),
            };
            dialog_kit::put(buf, area.x, area.y + k as u16, text, area.width, style);
        }
    }

    fn render_buttons(
        &mut self,
        buf: &mut Buffer,
        inner: Rect,
        y: u16,
        state: &JournalState,
        hits: &mut Vec<(Rect, Hit)>,
    ) {
        let close_w = dialog_kit::button_width(m::BTN_CLOSE.get());
        let close_x = inner.right().saturating_sub(close_w);
        let close = dialog_kit::button(
            buf,
            close_x,
            y,
            m::BTN_CLOSE.get(),
            ButtonState::Normal {
                hovered: self.hover == Some(Hit::Close),
                focused: false,
            },
        );
        let mut x = inner.x;
        let busy = self.job.is_some();
        for act in self.actions(state) {
            let label = act_label(act, self, state);
            let w = dialog_kit::button_width(&label);
            if x + w + 1 >= close_x {
                break;
            }
            let hovered = self.hover == Some(Hit::Act(act));
            let st = if busy {
                ButtonState::Disabled
            } else if matches!(act, Act::UndoTo | Act::RestoreRef) {
                ButtonState::Primary {
                    hovered,
                    focused: false,
                }
            } else {
                ButtonState::Normal {
                    hovered,
                    focused: false,
                }
            };
            let rect = dialog_kit::button(buf, x, y, &label, st);
            if !busy {
                hits.push((rect, Hit::Act(act)));
            }
            x += rect.width + 1;
        }
        hits.push((close, Hit::Close));
    }

    fn render_hints(&self, buf: &mut Buffer, inner: Rect, y: u16) {
        let tab = dialog_kit::key_name(crate::keymap::chord::Key::Tab);
        let mut pairs: Vec<(String, &str)> = vec![
            ("\u{2191}\u{2193}".to_string(), m::HINT_MOVE.get()),
            (tab, m::HINT_VIEW.get()),
        ];
        match self.tab {
            Tab::History => {
                pairs.push(("u".into(), m::HINT_UNDO_TO.get()));
                pairs.push(("r".into(), m::HINT_RESTORE.get()));
                pairs.push(("y".into(), m::HINT_REDO_TO.get()));
            }
            Tab::Reflog => {
                pairs.push(("r".into(), m::HINT_RESTORE.get()));
                pairs.push(("b".into(), m::HINT_BRANCH.get()));
                pairs.push(("t".into(), m::HINT_TAG.get()));
                pairs.push(("w".into(), m::HINT_APPLY.get()));
                pairs.push(("z".into(), m::HINT_STASH.get()));
                pairs.push(("s".into(), m::HINT_SHOW.get()));
                if self.reflog.more {
                    pairs.push(("m".into(), m::BTN_LOAD_MORE.get()));
                }
            }
            Tab::Lost => {
                pairs.push(("b".into(), m::HINT_BRANCH.get()));
                pairs.push(("t".into(), m::HINT_TAG.get()));
                pairs.push(("c".into(), m::HINT_PICK.get()));
                pairs.push(("w".into(), m::HINT_APPLY.get()));
                pairs.push(("z".into(), m::HINT_STASH.get()));
            }
        }
        pairs.push((".".into(), m::HINT_MENU.get()));
        let view: Vec<(&str, &str)> = pairs.iter().map(|(k, w)| (k.as_str(), *w)).collect();
        dialog_kit::hints(buf, inner.x, y, inner.width, &view);
    }

    // -- overlays -----------------------------------------------------------

    fn render_overlay(&mut self, buf: &mut Buffer, rect: Rect, hits: &mut Vec<(Rect, Hit)>) {
        let hover = self.hover;
        let key = self.undo_key();
        let Some(overlay) = self.overlay.as_mut() else {
            return;
        };
        match overlay {
            Overlay::Confirm(c) => render_confirm(buf, rect, c, hover, hits),
            Overlay::Name(n) => render_name(buf, rect, n, hover, hits),
            Overlay::Mode(p) => render_mode(buf, rect, p, hover, hits, &key),
            Overlay::Menu(menu) => render_menu(buf, rect, menu, hover, hits),
        }
    }
}

/// A menu entry's text. The offer's own words need the panel, so the menu
/// shows the generic "Restore ... to where it was" form.
fn act_label_static(act: Act) -> String {
    match act {
        Act::UndoTo => m::BTN_UNDO_TO.get().to_string(),
        Act::RedoTo => m::BTN_REDO_TO.get().to_string(),
        Act::RestoreOnly => m::BTN_RESTORE_ONLY.get().to_string(),
        Act::Offer => m::MENU_OFFER.get().to_string(),
        Act::ShowInLog => m::BTN_SHOW_IN_LOG.get().to_string(),
        Act::CreateBranch => m::BTN_CREATE_BRANCH.get().to_string(),
        Act::RestoreRef => m::MENU_RESTORE_REF.get().to_string(),
        Act::CherryPick => m::BTN_CHERRY_PICK.get().to_string(),
        Act::CreateTag => m::BTN_CREATE_TAG.get().to_string(),
        Act::ApplyToWorktree => m::BTN_APPLY.get().to_string(),
        Act::SaveAsStash => m::BTN_STASH.get().to_string(),
        Act::LoadMore => m::BTN_LOAD_MORE.get().to_string(),
    }
}

fn popup_frame(
    buf: &mut Buffer,
    outer: Rect,
    want_w: u16,
    want_h: u16,
    title: &str,
    red: bool,
) -> Rect {
    let rect = dialog_kit::centered(outer, want_w, want_h);
    dialog_kit::panel(buf, rect, title);
    if red {
        // Recolor the border in place: re-rendering a Block would erase the title.
        let style = Style::default().fg(th().status.danger);
        for x in rect.x..rect.right() {
            buf[(x, rect.y)].set_style(style);
            buf[(x, rect.bottom() - 1)].set_style(style);
        }
        for y in rect.y..rect.bottom() {
            buf[(rect.x, y)].set_style(style);
            buf[(rect.right() - 1, y)].set_style(style);
        }
    }
    rect
}

fn danger_button(buf: &mut Buffer, x: u16, y: u16, label: &str, hovered: bool) -> Rect {
    let bg = if hovered {
        crate::hover_fill(th().chrome.confirm_danger, th().ui.on_button, 30)
    } else {
        th().chrome.confirm_danger
    };
    let text = format!(" {label} ");
    dialog_kit::put(
        buf,
        x,
        y,
        &text,
        display_width(&text) as u16,
        Style::default()
            .fg(th().ui.on_button)
            .bg(bg)
            .add_modifier(Modifier::BOLD),
    )
}

fn render_confirm(
    buf: &mut Buffer,
    outer: Rect,
    c: &Confirm,
    hover: Option<Hit>,
    hits: &mut Vec<(Rect, Hit)>,
) {
    let shown = c.lines.len().min(10);
    let extra = u16::from(c.lines.len() > shown) + u16::from(c.warning.is_some());
    let h = 6 + shown as u16 + extra;
    let rect = popup_frame(
        buf,
        outer,
        outer.width.saturating_sub(8).min(90),
        h,
        &c.title,
        c.red,
    );
    let inner = Rect::new(
        rect.x + 2,
        rect.y + 1,
        rect.width.saturating_sub(4),
        rect.height - 2,
    );
    let mut y = inner.y;
    for line in c.lines.iter().take(shown) {
        let text = format!("\u{2022} {line}");
        dialog_kit::put(buf, inner.x, y, &text, inner.width, style_text());
        y += 1;
    }
    if c.lines.len() > shown {
        let more = m::detail_more_files((c.lines.len() - shown) as u64);
        dialog_kit::put(buf, inner.x, y, &more, inner.width, style_muted());
        y += 1;
    }
    if let Some(w) = &c.warning {
        dialog_kit::put(
            buf,
            inner.x,
            y,
            w,
            inner.width,
            Style::default().fg(th().status.danger).bg(th().ui.panel),
        );
    }
    let by = rect.y + rect.height - 2;
    let confirm_w = dialog_kit::button_width(&c.button);
    let cancel_w = dialog_kit::button_width(gitpoulpi_i18n::msg::dialog::CANCEL.get());
    let confirm_x = inner.right().saturating_sub(confirm_w);
    let cancel_x = confirm_x.saturating_sub(cancel_w + 1);
    let cancel = dialog_kit::button(
        buf,
        cancel_x,
        by,
        gitpoulpi_i18n::msg::dialog::CANCEL.get(),
        ButtonState::Normal {
            hovered: hover == Some(Hit::Cancel),
            focused: false,
        },
    );
    let confirm = if c.red {
        danger_button(buf, confirm_x, by, &c.button, hover == Some(Hit::Confirm))
    } else {
        dialog_kit::button(
            buf,
            confirm_x,
            by,
            &c.button,
            ButtonState::Primary {
                hovered: hover == Some(Hit::Confirm),
                focused: false,
            },
        )
    };
    hits.push((confirm, Hit::Confirm));
    hits.push((cancel, Hit::Cancel));
    // The popup body swallows clicks that miss its buttons.
    hits.push((rect, Hit::Body));
}

fn render_name(
    buf: &mut Buffer,
    outer: Rect,
    n: &mut NamePrompt,
    hover: Option<Hit>,
    hits: &mut Vec<(Rect, Hit)>,
) {
    let rect = popup_frame(
        buf,
        outer,
        outer.width.saturating_sub(8).min(80),
        8,
        m::TITLE_NAME.get(),
        false,
    );
    let inner = Rect::new(
        rect.x + 2,
        rect.y + 1,
        rect.width.saturating_sub(4),
        rect.height - 2,
    );
    n.edit.sync(Some(super::NameField::Name));
    n.edit.begin_frame();
    dialog_kit::put(buf, inner.x, inner.y, &n.prompt, inner.width, style_text());
    let field = Rect::new(inner.x, inner.y + 2, inner.width, 1);
    dialog_kit::text_input(
        buf,
        field,
        &n.value,
        "",
        true,
        hover == Some(Hit::NameField),
        (&n.edit, super::NameField::Name),
    );
    if let Some(error) = &n.error {
        dialog_kit::put(
            buf,
            inner.x,
            inner.y + 3,
            error,
            inner.width,
            Style::default().fg(th().status.danger).bg(th().ui.panel),
        );
    }
    let by = rect.y + rect.height - 2;
    let ok_label = m::BTN_CREATE_OK.get();
    let ok_w = dialog_kit::button_width(ok_label);
    let cancel_w = dialog_kit::button_width(gitpoulpi_i18n::msg::dialog::CANCEL.get());
    let ok_x = inner.right().saturating_sub(ok_w);
    let cancel_x = ok_x.saturating_sub(cancel_w + 1);
    let cancel = dialog_kit::button(
        buf,
        cancel_x,
        by,
        gitpoulpi_i18n::msg::dialog::CANCEL.get(),
        ButtonState::Normal {
            hovered: hover == Some(Hit::Cancel),
            focused: false,
        },
    );
    let ok = dialog_kit::button(
        buf,
        ok_x,
        by,
        ok_label,
        ButtonState::Primary {
            hovered: hover == Some(Hit::Confirm),
            focused: false,
        },
    );
    hits.push((ok, Hit::Confirm));
    hits.push((cancel, Hit::Cancel));
    hits.push((field, Hit::NameField));
    hits.push((rect, Hit::Body));
}

fn render_mode(
    buf: &mut Buffer,
    outer: Rect,
    p: &mut ModePick,
    hover: Option<Hit>,
    hits: &mut Vec<(Rect, Hit)>,
    undo_key: &str,
) {
    let hard = p.combo.selected() == Some("hard");
    let rect = popup_frame(
        buf,
        outer,
        outer.width.saturating_sub(8).min(96),
        if hard { 11 } else { 10 },
        m::MODE_TITLE.get(),
        false,
    );
    let inner = Rect::new(
        rect.x + 2,
        rect.y + 1,
        rect.width.saturating_sub(4),
        rect.height - 2,
    );
    dialog_kit::put(
        buf,
        inner.x,
        inner.y,
        &m::mode_explain(p.refname.clone()),
        inner.width,
        style_text(),
    );
    let field = Rect::new(inner.x, inner.y + 2, inner.width.min(60), 1);
    p.combo
        .render_field(buf, field, true, hover == Some(Hit::ModeField));
    hits.push((field, Hit::ModeField));
    if hard {
        dialog_kit::put(
            buf,
            inner.x,
            inner.y + 4,
            &m::mode_hard_warn(undo_key.to_string()),
            inner.width,
            Style::default()
                .fg(th().status.danger)
                .bg(th().ui.panel)
                .add_modifier(Modifier::BOLD),
        );
    }
    let by = rect.y + rect.height - 2;
    let ok_w = dialog_kit::button_width(m::BTN_CONTINUE.get());
    let cancel_w = dialog_kit::button_width(gitpoulpi_i18n::msg::dialog::CANCEL.get());
    let ok_x = inner.right().saturating_sub(ok_w);
    let cancel_x = ok_x.saturating_sub(cancel_w + 1);
    let cancel = dialog_kit::button(
        buf,
        cancel_x,
        by,
        gitpoulpi_i18n::msg::dialog::CANCEL.get(),
        ButtonState::Normal {
            hovered: hover == Some(Hit::Cancel),
            focused: false,
        },
    );
    let ok = if hard {
        danger_button(
            buf,
            ok_x,
            by,
            m::BTN_CONTINUE.get(),
            hover == Some(Hit::Confirm),
        )
    } else {
        dialog_kit::button(
            buf,
            ok_x,
            by,
            m::BTN_CONTINUE.get(),
            ButtonState::Primary {
                hovered: hover == Some(Hit::Confirm),
                focused: false,
            },
        )
    };
    hits.insert(0, (cancel, Hit::Cancel));
    hits.insert(0, (ok, Hit::Confirm));
    hits.push((rect, Hit::Body));
    // The open list is painted last and its rows come first.
    p.combo.render_popup(buf, rect);
}

fn render_menu(
    buf: &mut Buffer,
    outer: Rect,
    menu: &mut RowMenu,
    hover: Option<Hit>,
    hits: &mut Vec<(Rect, Hit)>,
) {
    let labels: Vec<String> = menu.items.iter().map(|a| act_label_static(*a)).collect();
    let w = labels.iter().map(|l| display_width(l)).max().unwrap_or(10) as u16 + 4;
    let h = labels.len() as u16 + 2;
    let x = menu.at.0.min(outer.right().saturating_sub(w));
    let y = menu.at.1.min(outer.bottom().saturating_sub(h));
    let rect = Rect::new(x, y, w.min(outer.width), h.min(outer.height));
    dialog_kit::panel(buf, rect, "");
    let mut entries = Vec::new();
    for (k, text) in labels.iter().enumerate() {
        let ry = rect.y + 1 + k as u16;
        let selected = k == menu.selected;
        let hovered = hover == Some(Hit::MenuItem(k));
        let row = Rect::new(rect.x + 1, ry, rect.width.saturating_sub(2), 1);
        dialog_kit::fill(buf, row, style_row(selected, hovered));
        dialog_kit::put(
            buf,
            row.x + 1,
            ry,
            text,
            row.width.saturating_sub(1),
            style_row(selected, hovered),
        );
        entries.push((row, Hit::MenuItem(k)));
    }
    hits.extend(entries);
    hits.push((rect, Hit::Body));
}
