//! Painting the right-hand panes of the Reflog and Lost work views: the
//! commit header, the changed files (the shared `file_rows` list with its
//! scrollbar and the Path / Tree toggle) and the selected file's hunks. Every
//! hit rect comes out of the walk that paints.

use std::path::PathBuf;

use gitpoulpi_core::domain::short;
use gitpoulpi_i18n::msg::commit_detail as cd;
use gitpoulpi_i18n::msg::undo_safety as m;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{
    Block, BorderType, Borders, List, ListState as RState, StatefulWidget, Widget,
};

use super::panes::{DiffShown, Divider, Pane};
use super::render::Hit;
use super::{rows, SafetyPanel, Tab};
use crate::dialog_kit;
use crate::status_pane::FileListMode;
use crate::th;

/// How many message lines the header shows under the commit id.
const HEADER_MESSAGE_LINES: u16 = 3;

fn muted() -> Style {
    Style::default().fg(th().ui.muted).bg(th().ui.panel)
}

fn text() -> Style {
    Style::default().fg(th().ui.text).bg(th().ui.panel)
}

impl SafetyPanel {
    /// What `Ctrl+N` badges: the view tabs and the panes.
    pub(crate) fn nav_targets(&self) -> Vec<(Rect, (u16, u16))> {
        dialog_kit::nav_cells_in(&self.hits, |h| matches!(h, Hit::Tab(_) | Hit::Pane(_)))
    }

    /// The arrange mode (keyboard twin of the frame's mouse gestures).
    pub(crate) fn arrange(&mut self) {
        self.overlay = None;
        crate::dialog_kit::frame::arrange_top();
    }

    /// The commit the panes show: the list's cursor.
    pub(super) fn pane_commit(&self) -> Option<gitpoulpi_core::domain::Oid> {
        match self.tab {
            Tab::Reflog => self.selected_reflog().map(|e| e.new),
            Tab::Lost => self.selected_lost().map(|c| c.oid),
            Tab::History => None,
        }
    }

    pub(super) fn detail(&self) -> Option<&super::detail::Loaded> {
        match self.tab {
            Tab::Reflog => self.reflog.detail.current(),
            Tab::Lost => self.lost.detail.current(),
            Tab::History => None,
        }
    }

    /// Every frame, before painting: the file rows of the commit under the
    /// list's cursor and the read of the selected file's hunks. Never waits.
    pub(super) fn poll_panes(&mut self, want: Option<gitpoulpi_core::domain::Oid>) {
        if self.tab == Tab::History {
            return;
        }
        self.panes.follow(want);
        let loaded = match self.tab {
            Tab::Reflog => self.reflog.detail.current(),
            _ => self.lost.detail.current(),
        };
        let paths: &[PathBuf] = loaded.map_or(&[], |l| l.paths.as_slice());
        self.panes.rebuild(paths);
        let wanted = want.and_then(|oid| {
            self.panes
                .selected_path(paths)
                .map(|path| (oid, path.clone()))
        });
        self.panes.diff.poll(&self.requester, wanted.as_ref());
    }

    /// How many rows the header would like (the id line and the message).
    pub(super) fn header_lines(&self) -> u16 {
        let message = self
            .detail()
            .and_then(|d| d.node.as_ref())
            .map_or(0, |n| n.message.lines().count() as u16);
        1 + message.min(HEADER_MESSAGE_LINES)
    }

    fn pane_border(&self, pane: Pane) -> Style {
        let color = if self.panes.focus == pane {
            th().ui.accent
        } else {
            th().ui.muted
        };
        Style::default().fg(color).bg(th().ui.panel)
    }

    fn pane_box(&self, buf: &mut Buffer, rect: Rect, pane: Pane, title: &str) {
        Block::default()
            .title(format!(" {title} "))
            .title_style(match self.panes.focus == pane {
                true => text().add_modifier(Modifier::BOLD),
                false => muted(),
            })
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(self.pane_border(pane))
            .style(Style::default().bg(th().ui.panel))
            .render(rect, buf);
    }

    /// The right pane: header, files, diff (and the two grab zones).
    pub(super) fn render_right(&mut self, buf: &mut Buffer, hits: &mut Vec<(Rect, Hit)>) {
        let lay = self.panes.layout;
        // The dividers first: their zones sit on the border cells of the boxes
        // below, and the first rect that holds a point answers.
        hits.push((lay.grab_v, Hit::Divider(Divider::Vertical)));
        hits.push((lay.grab_h, Hit::Divider(Divider::Horizontal)));
        self.render_commit_header(buf, lay.header);
        self.render_files(buf, lay.files, hits);
        self.render_diff(buf, lay.diff, hits);
        // The divider grips are painted over the borders they live on.
        let grip = Style::default().fg(th().ui.accent).bg(th().ui.panel);
        let held = |d| self.panes.drag == Some(d);
        let hover = |d| self.hover == Some(Hit::Divider(d));
        let lit = |d| held(d) || hover(d);
        let vgrip = if lit(Divider::Vertical) {
            "\u{2503}"
        } else {
            "\u{2502}"
        };
        if lay.grab_v.height > 2 {
            let x = lay.grab_v.x;
            let mid = lay.grab_v.y + lay.grab_v.height / 2;
            dialog_kit::put(buf, x, mid, vgrip, 1, grip);
        }
        let hmark = if lit(Divider::Horizontal) {
            "\u{2550}"
        } else {
            "\u{2500}"
        };
        if lay.grab_h.width > 6 {
            let x = lay.grab_h.x + lay.grab_h.width / 2;
            dialog_kit::put(buf, x, lay.grab_h.y, hmark, 1, grip);
        }
        crate::pointer_map::register(lay.grab_v, crate::pointer_map::Kind::ColResize);
        crate::pointer_map::register(lay.grab_h, crate::pointer_map::Kind::RowResize);
    }

    fn render_commit_header(&self, buf: &mut Buffer, area: Rect) {
        if area.height == 0 {
            return;
        }
        dialog_kit::fill(buf, area, Style::default().bg(th().ui.panel));
        let Some(oid) = self.pane_commit() else {
            return;
        };
        let id_style = Style::default()
            .fg(th().ui.accent)
            .bg(th().ui.panel)
            .add_modifier(Modifier::BOLD);
        let id = dialog_kit::put(buf, area.x + 1, area.y, &short(oid), area.width, id_style);
        let Some(loaded) = self.detail() else {
            let spin = crate::activity::spinner_frame(self.lost.scan_elapsed());
            let line = format!("{spin} {}", m::DETAIL_LOADING.get());
            dialog_kit::put(buf, id.right() + 2, area.y, &line, area.width, muted());
            return;
        };
        let Some(node) = &loaded.node else {
            return;
        };
        let who = m::detail_author(&node.author_name, rows::age(node.timestamp));
        let x = id.right() + 2;
        dialog_kit::put(
            buf,
            x,
            area.y,
            &who,
            area.right().saturating_sub(x),
            muted(),
        );
        for (k, line) in node
            .message
            .lines()
            .take(HEADER_MESSAGE_LINES as usize)
            .enumerate()
        {
            let y = area.y + 1 + k as u16;
            if y >= area.bottom() {
                break;
            }
            dialog_kit::put(
                buf,
                area.x + 1,
                y,
                line,
                area.width.saturating_sub(1),
                text(),
            );
        }
    }

    fn render_files(&mut self, buf: &mut Buffer, rect: Rect, hits: &mut Vec<(Rect, Hit)>) {
        let paths: Vec<PathBuf> = self.detail().map(|d| d.paths.clone()).unwrap_or_default();
        let count = paths.len();
        let title = if count > 0 {
            m::detail_files(count as u64)
        } else {
            String::new()
        };
        self.pane_box(buf, rect, Pane::Files, &title);
        let inner = Rect::new(
            rect.x + 1,
            rect.y + 1,
            rect.width.saturating_sub(2),
            rect.height.saturating_sub(2),
        );
        // The Path | Tree toggle, on the top border at the right.
        let mut tx = rect.right().saturating_sub(2);
        for mode in [FileListMode::Tree, FileListMode::Path] {
            let label = format!(
                " {} ",
                match mode {
                    FileListMode::Path => cd::MODE_PATH.get(),
                    FileListMode::Tree => cd::MODE_TREE.get(),
                }
            );
            let w = crate::display_width(&label) as u16;
            if tx < rect.x + 2 + w {
                break;
            }
            tx -= w;
            let active = self.panes.mode == mode;
            let hovered = self.hover == Some(Hit::Mode(mode));
            let style = if active {
                Style::default()
                    .fg(th().ui.on_button)
                    .bg(th().ui.accent)
                    .add_modifier(Modifier::BOLD)
            } else if hovered {
                Style::default().fg(th().ui.text).bg(th().ui.hover)
            } else {
                muted()
            };
            let r = dialog_kit::put(buf, tx, rect.y, &label, w, style);
            hits.push((r, Hit::Mode(mode)));
        }
        let len = self.panes.tree.len();
        let view_rows = inner.height as usize;
        {
            let files = &mut self.panes.files;
            files.view_rows = view_rows;
            files.clamp_selected(len);
            files.offset = files.offset.min(len.saturating_sub(view_rows.max(1)));
        }
        if len == 0 {
            let words = if self.detail().is_none() {
                m::DETAIL_LOADING.get().to_string()
            } else {
                cd::NO_FILE_CHANGES_PAREN.get().to_string()
            };
            dialog_kit::put(
                buf,
                inner.x + 1,
                inner.y,
                &words,
                inner.width.saturating_sub(1),
                muted(),
            );
            hits.push((rect, Hit::Pane(Pane::Files)));
            return;
        }
        let statuses = self
            .detail()
            .map(|d| d.statuses.clone())
            .unwrap_or_default();
        let items = crate::file_rows::FileRows {
            rows: &self.panes.tree,
            paths: &paths,
            status_of: &|p| statuses.get(p).copied(),
            is_lfs: &|_| false,
            file_picked: &|_| false,
            folder_picked: &|_| false,
        }
        .items();
        let files = self.panes.files;
        let selected = (files.selected >= files.offset
            && files.selected < files.offset + view_rows)
            .then_some(files.selected);
        let mut state = RState::default().with_offset(files.offset);
        // `select(None)` would reset the offset a scrollbar set.
        if selected.is_some() {
            state.select(selected);
        }
        StatefulWidget::render(
            List::new(items).highlight_style(
                Style::default()
                    .bg(th().ui.selection)
                    .add_modifier(Modifier::BOLD),
            ),
            inner,
            buf,
            &mut state,
        );
        for k in 0..view_rows {
            let index = files.offset + k;
            if index >= len {
                break;
            }
            let row = Rect::new(inner.x, inner.y + k as u16, inner.width, 1);
            if self.hover == Some(Hit::File(index)) && selected != Some(index) {
                crate::refs_sidebar::paint_row_hover(buf, row);
            }
            hits.push((row, Hit::File(index)));
        }
        if let Some(track) = crate::scrollbar::Track::vertical(rect, len, files.offset) {
            crate::scrollbar::render_buf(buf, track, true, false);
        }
        hits.push((rect, Hit::Pane(Pane::Files)));
    }

    /// The path the diff is wanted for: the file under the files' cursor.
    pub(super) fn wanted_diff(&self) -> Option<super::panes::DiffKey> {
        let oid = self.pane_commit()?;
        let paths = &self.detail()?.paths;
        let path = self.panes.selected_path(paths)?;
        Some((oid, path.clone()))
    }

    fn render_diff(&mut self, buf: &mut Buffer, rect: Rect, hits: &mut Vec<(Rect, Hit)>) {
        let want = self.wanted_diff();
        let title = match &want {
            Some((_, path)) => m::pane_diff(path.display().to_string()),
            None => m::PANE_DIFF_PLAIN.get().to_string(),
        };
        self.pane_box(buf, rect, Pane::Diff, &title);
        hits.push((rect, Hit::Pane(Pane::Diff)));
        let inner = Rect::new(
            rect.x + 1,
            rect.y + 1,
            rect.width.saturating_sub(2),
            rect.height.saturating_sub(2),
        );
        if inner.width == 0 || inner.height == 0 {
            return;
        }
        let say = |buf: &mut Buffer, words: &str, style: Style| {
            dialog_kit::put(
                buf,
                inner.x + 1,
                inner.y,
                words,
                inner.width.saturating_sub(1),
                style,
            );
        };
        // The diff is borrowed out of the pane for the lines; take a copy of
        // what is needed so the pane can lay it out.
        self.panes.diff_len = 0;
        let ready = match self.panes.diff.shown(want.as_ref()) {
            DiffShown::Nothing => {
                say(buf, m::PANE_DIFF_NONE.get(), muted());
                return;
            }
            DiffShown::Loading => {
                let spin = crate::activity::spinner_frame(self.lost.scan_elapsed());
                say(
                    buf,
                    &format!("{spin} {}", m::PANE_DIFF_LOADING.get()),
                    muted(),
                );
                return;
            }
            DiffShown::Failed(why) => {
                let why = why.to_string();
                say(
                    buf,
                    &why,
                    Style::default().fg(th().status.danger).bg(th().ui.panel),
                );
                return;
            }
            DiffShown::Ready(file) => file.clone(),
        };
        let Some(key) = want else {
            return;
        };
        let lines = self.panes.diff.lines(&key, &ready, inner.width);
        let len = lines.len();
        self.panes.diff_len = len;
        let view_rows = inner.height as usize;
        {
            let s = &mut self.panes.scroll;
            s.view_rows = view_rows;
            s.offset = s.offset.min(len.saturating_sub(view_rows.max(1)));
        }
        let offset = self.panes.scroll.offset;
        let end = (offset + view_rows).min(len);
        for (k, line) in lines[offset..end].iter().enumerate() {
            buf.set_line(inner.x, inner.y + k as u16, line, inner.width);
        }
        if let Some(track) = crate::scrollbar::Track::vertical(rect, len, offset) {
            crate::scrollbar::render_buf(buf, track, true, false);
        }
    }
}
