//! The blob diff drill-in: a row of the Commits tab opens the files the
//! content rules changed in that commit (left) and the diff of the selected
//! one (right), a master-detail overlay with its own cross.
//!
//! The diff is the daemon's (`RewriteBlobDiff`): the old file from the
//! repository, the new one from the preview pack. When a secret rule made the
//! change the removed lines arrive already hidden (`redacted`); this file
//! only words that, it never holds a secret.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use gitpoulpi_core::domain::rewrite::{
    BlobChange, BlobChangeKind, BlobLineKind, CommitChange, PathChange, REDACTED_LINE,
};
use gitpoulpi_core::domain::OperationKind;
use gitpoulpi_i18n::msg::rewrite_view as t;
use gitpoulpi_ipc::rewrite::BlobDiffArgs;

use super::bars::BarId;
use super::render::{bad, bold, dim, plain, row_bg, wrap, RewriteHit};
use super::rules_form::format_size;
use super::{Notice, RewriteView};
use crate::daemon_link::Reply;
use crate::dialog_kit;
use crate::th;

/// What the right half shows.
pub(crate) enum DrillDiff {
    Loading,
    Ready(Box<BlobChange>),
    Failed(String),
}

/// The open drill-in: one commit's changed files and the diff of one.
pub(crate) struct Drill {
    pub commit: CommitChange,
    pub cursor: usize,
    /// First diff line shown.
    pub scroll: usize,
    /// The keyboard drives the diff (scroll) instead of the file list.
    pub in_diff: bool,
    pub diff: DrillDiff,
    pub(super) pending: Option<(usize, Reply<BlobChange>)>,
}

/// One line of the rendered diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DiffLine {
    Header(String),
    Note(String),
    Hunk(String),
    Context(String),
    Removed(String),
    Added(String),
    /// Removed lines the daemon hid because they held a secret.
    Hidden(String),
}

impl DiffLine {
    pub(crate) fn text(&self) -> &str {
        match self {
            DiffLine::Header(s)
            | DiffLine::Note(s)
            | DiffLine::Hunk(s)
            | DiffLine::Context(s)
            | DiffLine::Removed(s)
            | DiffLine::Added(s)
            | DiffLine::Hidden(s) => s,
        }
    }
}

/// The path a [`PathChange`] row is about (the new path of a rename).
pub(crate) fn path_of(change: &PathChange) -> &str {
    match change {
        PathChange::Removed { path }
        | PathChange::Modified { path }
        | PathChange::LfsImported { path }
        | PathChange::LfsExported { path } => path,
        PathChange::Renamed { to, .. } => to,
    }
}

/// The diff as lines: the file's fate and sizes, then the hunks. A run of
/// hidden removed lines is one row that says why.
pub(crate) fn diff_lines(change: &BlobChange) -> Vec<DiffLine> {
    let mut out = vec![DiffLine::Header(match change.kind {
        BlobChangeKind::Modified => t::path_modified(&change.path),
        BlobChangeKind::Removed => t::path_removed(&change.path),
        BlobChangeKind::Unchanged => t::diff_unchanged(&change.path),
    })];
    out.push(DiffLine::Note(t::diff_sizes(
        format_size(change.old_size),
        format_size(change.new_size),
    )));
    if change.binary {
        out.push(DiffLine::Note(t::DIFF_BINARY.get().to_string()));
        return out;
    }
    if change.redacted && change.hunks.is_empty() {
        out.push(DiffLine::Hidden(t::DIFF_HIDDEN.get().to_string()));
    }
    for hunk in &change.hunks {
        out.push(DiffLine::Hunk(format!(
            "@@ -{} +{} @@",
            hunk.old_start, hunk.new_start
        )));
        let mut hidden_run = false;
        for line in &hunk.lines {
            let hidden = line.kind == BlobLineKind::Removed
                && (change.redacted || line.text == REDACTED_LINE);
            if hidden {
                if !hidden_run {
                    out.push(DiffLine::Hidden(t::DIFF_HIDDEN.get().to_string()));
                }
                hidden_run = true;
                continue;
            }
            hidden_run = false;
            out.push(match line.kind {
                BlobLineKind::Context => DiffLine::Context(line.text.clone()),
                BlobLineKind::Removed => DiffLine::Removed(line.text.clone()),
                BlobLineKind::Added => DiffLine::Added(line.text.clone()),
            });
        }
    }
    out
}

impl RewriteView {
    /// Opens the drill-in on row `index` of the Commits tab. A commit with
    /// no changed file (its details changed, not its content) says so.
    pub(crate) fn open_drill(&mut self, index: usize) {
        let Some(row) = self.rows.loaded.get(index).cloned() else {
            return;
        };
        if row.paths.is_empty() {
            self.notice = Some(Notice {
                text: t::DRILL_NONE.get().to_string(),
                error: false,
            });
            return;
        }
        self.rows.cursor = index;
        self.drill = Some(Drill {
            commit: row,
            cursor: 0,
            scroll: 0,
            in_diff: false,
            diff: DrillDiff::Loading,
            pending: None,
        });
        self.request_diff();
    }

    pub(crate) fn close_drill(&mut self) {
        self.drill = None;
    }

    /// Asks for the diff of the file under the drill's cursor.
    fn request_diff(&mut self) {
        let (Some(plan), Some(drill)) = (&self.plan, &mut self.drill) else {
            return;
        };
        let Some(path) = drill.commit.paths.get(drill.cursor).map(path_of) else {
            return;
        };
        let args = BlobDiffArgs {
            run: plan.run.clone(),
            commit: drill.commit.old,
            path: path.to_string(),
        };
        drill.scroll = 0;
        drill.diff = DrillDiff::Loading;
        drill.pending = Some((
            drill.cursor,
            self.requester.call(
                OperationKind::RewriteBlobDiff,
                serde_json::to_value(args).unwrap_or_default(),
            ),
        ));
    }

    /// Selects file `index` of the list and loads its diff.
    pub(crate) fn drill_select(&mut self, index: usize) {
        let Some(drill) = &mut self.drill else {
            return;
        };
        let last = drill.commit.paths.len().saturating_sub(1);
        let index = index.min(last);
        if index != drill.cursor || matches!(drill.diff, DrillDiff::Failed(_)) {
            drill.cursor = index;
            self.request_diff();
        }
    }

    /// Takes the answer of the diff request, if it has come.
    pub(super) fn poll_drill(&mut self) {
        let Some(drill) = &mut self.drill else {
            return;
        };
        let Some((for_cursor, reply)) = &drill.pending else {
            return;
        };
        let for_cursor = *for_cursor;
        let Some(answer) = reply.take() else {
            return;
        };
        drill.pending = None;
        if for_cursor != drill.cursor {
            return;
        }
        drill.diff = match answer {
            Ok(change) => DrillDiff::Ready(Box::new(change)),
            Err(error) => DrillDiff::Failed(crate::daemon_link::error_text::error_text(&error)),
        };
    }

    fn drill_lines(&self) -> usize {
        match self.drill.as_ref().map(|d| &d.diff) {
            Some(DrillDiff::Ready(change)) => diff_lines(change).len(),
            _ => 0,
        }
    }

    pub(crate) fn drill_scroll(&mut self, delta: isize) {
        let total = self.drill_lines();
        let room = self.paint.drill_rows.get().max(1);
        let max = total.saturating_sub(room);
        if let Some(d) = &mut self.drill {
            d.scroll = (d.scroll as isize + delta).clamp(0, max as isize) as usize;
        }
    }

    pub(crate) fn drill_focus_diff(&mut self, on: bool) {
        if let Some(d) = &mut self.drill {
            d.in_diff = on;
        }
    }

    /// Keys while the drill-in is open.
    pub(super) fn on_drill_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;
        let Some(drill) = &self.drill else {
            return;
        };
        let (in_diff, cursor) = (drill.in_diff, drill.cursor);
        let page = self.paint.drill_rows.get().max(2) as isize - 1;
        match key.code {
            KeyCode::Esc | KeyCode::Backspace => self.close_drill(),
            KeyCode::Tab | KeyCode::BackTab => self.drill_focus_diff(!in_diff),
            KeyCode::Left => self.drill_focus_diff(false),
            KeyCode::Right | KeyCode::Enter => self.drill_focus_diff(true),
            KeyCode::Up | KeyCode::Char('k') if in_diff => self.drill_scroll(-1),
            KeyCode::Down | KeyCode::Char('j') if in_diff => self.drill_scroll(1),
            KeyCode::Up | KeyCode::Char('k') => self.drill_select(cursor.saturating_sub(1)),
            KeyCode::Down | KeyCode::Char('j') => self.drill_select(cursor + 1),
            KeyCode::PageUp => self.drill_scroll(-page),
            KeyCode::PageDown => self.drill_scroll(page),
            KeyCode::Home => self.drill_scroll(isize::MIN / 2),
            KeyCode::End => self.drill_scroll(isize::MAX / 2),
            _ => {}
        }
    }

    /// Paints the overlay and returns its click targets: the cross, the
    /// files, the diff, and a shield over the rest of the workbench (a click
    /// there closes it).
    pub(super) fn paint_drill(
        &self,
        buf: &mut Buffer,
        bounds: Rect,
        hovered: Option<RewriteHit>,
    ) -> Vec<(Rect, RewriteHit)> {
        let mut hits = Vec::new();
        let Some(drill) = &self.drill else {
            return hits;
        };
        let rect = Rect::new(
            bounds.x + 2,
            bounds.y + 1,
            bounds.width.saturating_sub(4),
            bounds.height.saturating_sub(2),
        );
        if rect.width < 40 || rect.height < 6 {
            hits.push((bounds, RewriteHit::DrillShield));
            return hits;
        }
        let hex = drill.commit.old.to_hex();
        let title = t::drill_title(&hex[..hex.len().min(7)]);
        dialog_kit::panel(buf, rect, &title);
        if let Some(cross) =
            crate::chrome::close_cross::paint(buf, rect, hovered == Some(RewriteHit::DrillClose))
        {
            hits.push((cross, RewriteHit::DrillClose));
        }
        let inner = Rect::new(rect.x + 1, rect.y + 1, rect.width - 2, rect.height - 2);
        let list_w = (inner.width / 3).clamp(18, 40);
        let list = Rect::new(inner.x, inner.y, list_w, inner.height);
        let diff = Rect::new(
            list.right() + 1,
            inner.y,
            inner.width - list_w - 1,
            inner.height,
        );
        // The divider.
        for y in inner.y..inner.bottom() {
            dialog_kit::put(
                buf,
                list.right(),
                y,
                "\u{2502}",
                1,
                Style::default().fg(th().ui.muted).bg(th().ui.panel),
            );
        }
        // Files.
        let paths = &drill.commit.paths;
        let visible = list.height as usize;
        let top = self.settle(
            BarId::DrillFiles,
            Some(drill.cursor),
            self.paint.drill_top.get(),
            visible,
            paths.len(),
        );
        self.paint.drill_top.set(top);
        // The bar sits on the divider, beside the rows.
        self.paint_bar_on(
            buf,
            &mut hits,
            hovered == Some(RewriteHit::Bar(BarId::DrillFiles)),
            BarId::DrillFiles,
            (list.right(), list.y, list.height),
            paths.len(),
            visible,
            top,
            Some(drill.cursor),
        );
        let mut note_rows = 0;
        for (slot, change) in paths.iter().enumerate().skip(top).take(visible) {
            let y = list.y + (slot - top) as u16;
            let band = Rect::new(list.x, y, list.width, 1);
            let hit = RewriteHit::DrillPath(slot);
            let bg = row_bg(slot == drill.cursor, hovered == Some(hit));
            dialog_kit::fill(buf, band, Style::default().bg(bg));
            let mark = match change {
                PathChange::Removed { .. } => "- ",
                PathChange::Renamed { .. } => "> ",
                PathChange::Modified { .. } => "~ ",
                PathChange::LfsImported { .. } => "@ ",
                PathChange::LfsExported { .. } => "# ",
            };
            dialog_kit::put(
                buf,
                band.x + 1,
                y,
                &format!("{mark}{}", path_of(change)),
                band.width - 1,
                Style::default().fg(th().ui.text).bg(bg),
            );
            hits.push((band, hit));
            note_rows = (slot - top) as u16 + 1;
        }
        if paths.len() >= 64 && note_rows < list.height {
            dialog_kit::put(
                buf,
                list.x + 1,
                list.y + note_rows,
                &t::drill_capped(64),
                list.width - 1,
                dim(),
            );
        }
        // The diff.
        self.paint.drill_rows.set(diff.height as usize);
        dialog_kit::fill(buf, diff, Style::default().bg(th().ui.panel));
        match &drill.diff {
            DrillDiff::Loading => {
                dialog_kit::put(
                    buf,
                    diff.x + 1,
                    diff.y,
                    t::ROWS_LOADING.get(),
                    diff.width - 1,
                    dim(),
                );
            }
            DrillDiff::Failed(why) => {
                for (i, line) in wrap(&t::diff_failed(why), diff.width as usize - 2)
                    .iter()
                    .take(diff.height as usize)
                    .enumerate()
                {
                    dialog_kit::put(
                        buf,
                        diff.x + 1,
                        diff.y + i as u16,
                        line,
                        diff.width - 1,
                        bad(),
                    );
                }
            }
            DrillDiff::Ready(change) => {
                let lines = diff_lines(change);
                let max = lines.len().saturating_sub(diff.height as usize);
                let scroll = drill.scroll.min(max);
                // On the panel's own border column, right of the diff.
                self.paint_bar_on(
                    buf,
                    &mut hits,
                    hovered == Some(RewriteHit::Bar(BarId::DrillDiff)),
                    BarId::DrillDiff,
                    (diff.right(), diff.y, diff.height),
                    lines.len(),
                    diff.height as usize,
                    scroll,
                    None,
                );
                for (i, line) in lines
                    .iter()
                    .skip(scroll)
                    .take(diff.height as usize)
                    .enumerate()
                {
                    let y = diff.y + i as u16;
                    let (prefix, style) = match line {
                        DiffLine::Header(_) => ("", bold(drill.in_diff)),
                        DiffLine::Note(_) => ("", dim()),
                        DiffLine::Hunk(_) => {
                            ("", Style::default().fg(th().diff.hunk_fg).bg(th().ui.panel))
                        }
                        DiffLine::Context(_) => ("  ", plain()),
                        DiffLine::Removed(_) => (
                            "- ",
                            Style::default().fg(th().diff.del_fg).bg(th().diff.del_bg),
                        ),
                        DiffLine::Added(_) => (
                            "+ ",
                            Style::default().fg(th().diff.add_fg).bg(th().diff.add_bg),
                        ),
                        DiffLine::Hidden(_) => (
                            "- ",
                            Style::default()
                                .fg(th().status.warning)
                                .bg(th().diff.del_bg),
                        ),
                    };
                    if matches!(
                        line,
                        DiffLine::Added(_) | DiffLine::Removed(_) | DiffLine::Hidden(_)
                    ) {
                        dialog_kit::fill(
                            buf,
                            Rect::new(diff.x, y, diff.width, 1),
                            Style::default().bg(style.bg.unwrap_or(th().ui.panel)),
                        );
                    }
                    dialog_kit::put(
                        buf,
                        diff.x + 1,
                        y,
                        &format!("{prefix}{}", line.text()),
                        diff.width - 1,
                        style,
                    );
                }
            }
        }
        hits.push((diff, RewriteHit::DrillDiff));
        hits.push((bounds, RewriteHit::DrillShield));
        hits
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::rewrite::{BlobHunk, BlobLine, RunId};
    use gitpoulpi_core::domain::Oid;

    fn change(redacted: bool, lines: Vec<(BlobLineKind, &str)>) -> BlobChange {
        BlobChange {
            run: RunId("r".into()),
            commit: Oid([1; 20]),
            path: "a.txt".into(),
            kind: BlobChangeKind::Modified,
            binary: false,
            redacted,
            old_size: 10,
            new_size: 20,
            hunks: vec![BlobHunk {
                old_start: 1,
                new_start: 1,
                lines: lines
                    .into_iter()
                    .map(|(kind, text)| BlobLine {
                        kind,
                        text: text.into(),
                    })
                    .collect(),
            }],
        }
    }

    #[test]
    fn hidden_removed_lines_collapse_into_one_row_that_says_why() {
        let c = change(
            true,
            vec![
                (BlobLineKind::Context, "k"),
                (BlobLineKind::Removed, REDACTED_LINE),
                (BlobLineKind::Removed, REDACTED_LINE),
                (BlobLineKind::Added, "token=***REMOVED***"),
            ],
        );
        let lines = diff_lines(&c);
        let hidden = lines
            .iter()
            .filter(|l| matches!(l, DiffLine::Hidden(_)))
            .count();
        assert_eq!(hidden, 1);
        assert!(lines
            .iter()
            .any(|l| matches!(l, DiffLine::Hidden(s) if s == t::DIFF_HIDDEN.get())));
        assert!(lines.iter().all(|l| !matches!(l, DiffLine::Removed(_))));
    }

    #[test]
    fn an_ordinary_diff_keeps_removed_and_added_lines() {
        let c = change(
            false,
            vec![(BlobLineKind::Removed, "old"), (BlobLineKind::Added, "new")],
        );
        let lines = diff_lines(&c);
        assert!(lines.contains(&DiffLine::Removed("old".into())));
        assert!(lines.contains(&DiffLine::Added("new".into())));
    }

    #[test]
    fn a_binary_file_shows_sizes_only() {
        let mut c = change(false, vec![]);
        c.binary = true;
        c.hunks.clear();
        let lines = diff_lines(&c);
        assert_eq!(lines.len(), 3);
        assert!(matches!(&lines[2], DiffLine::Note(s) if s == t::DIFF_BINARY.get()));
    }
}
