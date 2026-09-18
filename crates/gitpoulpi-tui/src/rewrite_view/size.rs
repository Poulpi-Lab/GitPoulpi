//! The Size tab (slice S18): analyse what the history weighs, list the
//! largest files, the biggest folders and the files deleted from HEAD that
//! the history still stores, and turn ticked rows into a removal rule.
//!
//! The analysis reads sizes and paths only. "Remove from history" writes a
//! `RemovePaths` rule (exact paths, glob characters escaped), or, from the
//! Files list, a `RemoveBlobsOver` rule whose limit sits just under the
//! smallest ticked file. The space those rules free comes back only after
//! the rewrite, a purge of the old objects and a gc, and the tab says so.

use std::collections::HashSet;
use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::Style;

use gitpoulpi_core::domain::rewrite::{
    glob_for, human_size, BlobRow, BlobState, DeletedRow, FolderRow, RewriteProgress, Rule,
    RuleKind, Scope, SizeAnalysis,
};
use gitpoulpi_i18n::msg::{rewrite_size as z, rewrite_view as t};
use gitpoulpi_i18n::M;
use gitpoulpi_ipc::rewrite::AnalyzeArgs;

use super::render::{bad, bold, dim, plain, row_bg, warn, wrap, RewriteHit, Ui};
use super::{DetailForm, Focus, Notice, PreviewTab, RewriteView};
use crate::daemon_link::Reply;
use crate::dialog_kit;
use crate::th;

/// Rows asked for in each list.
const TOP: u32 = 200;

/// An analysis on its way.
pub(crate) struct SizeRun {
    pub(super) reply: Reply<SizeAnalysis>,
    pub since: Instant,
    /// The daemon job, known from the first progress report that is its own.
    pub job: Option<u64>,
    pub progress: Option<RewriteProgress>,
    /// The progress the connection already held when the job was sent: the
    /// last report of an earlier job, which is not this one's.
    baseline: Option<Option<(u64, RewriteProgress)>>,
    scope: Scope,
}

/// A finished analysis.
pub(crate) struct SizeResult {
    pub analysis: SizeAnalysis,
    /// The scope it read, to tell when the recipe's has moved on.
    pub scope: Scope,
}

/// Which list is on screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum SizeView {
    #[default]
    Files,
    Folders,
    Deleted,
}

impl SizeView {
    pub(crate) const ALL: [SizeView; 3] = [SizeView::Files, SizeView::Folders, SizeView::Deleted];

    fn label(self) -> M {
        match self {
            SizeView::Files => z::VIEW_FILES,
            SizeView::Folders => z::VIEW_FOLDERS,
            SizeView::Deleted => z::VIEW_DELETED,
        }
    }

    fn next(self) -> Self {
        match self {
            SizeView::Files => SizeView::Folders,
            SizeView::Folders => SizeView::Deleted,
            SizeView::Deleted => SizeView::Files,
        }
    }
}

/// How the list is ordered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum SizeSort {
    #[default]
    Size,
    Path,
    DeletedFirst,
}

impl SizeSort {
    pub(crate) const ALL: [SizeSort; 3] = [SizeSort::Size, SizeSort::Path, SizeSort::DeletedFirst];

    fn label(self) -> M {
        match self {
            SizeSort::Size => z::SORT_SIZE,
            SizeSort::Path => z::SORT_PATH,
            SizeSort::DeletedFirst => z::SORT_DELETED,
        }
    }

    fn next(self) -> Self {
        match self {
            SizeSort::Size => SizeSort::Path,
            SizeSort::Path => SizeSort::DeletedFirst,
            SizeSort::DeletedFirst => SizeSort::Size,
        }
    }
}

#[derive(Default)]
pub(crate) struct SizeState {
    pub running: Option<SizeRun>,
    pub result: Option<SizeResult>,
    pub cursor: usize,
    pub view: SizeView,
    pub sort: SizeSort,
    /// Ticked rows, by `Row::key`.
    pub ticked: HashSet<String>,
    /// A rewrite ran since the analysis.
    pub stale: bool,
    /// Why the last analysis failed.
    pub error: Option<String>,
}

/// A row of whichever list is shown.
#[derive(Clone, Copy)]
pub(super) enum Row<'a> {
    Blob(&'a BlobRow),
    Folder(&'a FolderRow),
    Deleted(&'a DeletedRow),
}

impl Row<'_> {
    /// Stable between sorts: what the tick set holds.
    fn key(&self) -> String {
        match self {
            Row::Blob(b) => format!("b:{}", b.blob.to_hex()),
            Row::Folder(f) => format!("f:{}", f.path),
            Row::Deleted(d) => format!("d:{}", d.path),
        }
    }

    pub(super) fn path(&self) -> &str {
        match self {
            Row::Blob(b) => &b.path,
            Row::Folder(f) => &f.path,
            Row::Deleted(d) => &d.path,
        }
    }

    fn size(&self) -> u64 {
        match self {
            Row::Blob(b) => b.size,
            Row::Folder(f) => f.size,
            Row::Deleted(d) => d.size,
        }
    }

    /// Not in HEAD any more.
    fn gone(&self) -> bool {
        match self {
            Row::Blob(b) => b.state == BlobState::Deleted,
            Row::Folder(f) => !f.in_head,
            Row::Deleted(_) => true,
        }
    }
}

fn short(oid: &gitpoulpi_core::domain::Oid) -> String {
    super::preview::short(oid)
}

impl RewriteView {
    pub(crate) fn analysing(&self) -> bool {
        self.size.running.is_some()
    }

    /// Why the rows on screen should not be trusted any more.
    pub(crate) fn size_stale_reason(&self) -> Option<M> {
        let result = self.size.result.as_ref()?;
        if self.size.stale {
            Some(z::STALE_REWRITE)
        } else if result.scope != self.recipe.scope {
            Some(z::STALE_SCOPE)
        } else {
            None
        }
    }

    // ------------------------------------------------------------- daemon

    /// "Analyse sizes": the scope of the workbench, as a job.
    pub(crate) fn start_analysis(&mut self) {
        if self.busy() {
            return;
        }
        self.notice = None;
        self.size.error = None;
        let scope = self.recipe.scope.clone();
        let args = AnalyzeArgs {
            scope: scope.clone(),
            top: TOP,
        };
        let reply = self.requester.call(
            gitpoulpi_core::domain::OperationKind::RewriteAnalyze,
            serde_json::to_value(args).unwrap_or_default(),
        );
        self.size.running = Some(SizeRun {
            reply,
            since: Instant::now(),
            job: None,
            progress: None,
            baseline: None,
            scope,
        });
    }

    /// The palette and F7: open the tab and run.
    pub(crate) fn analyse_from_anywhere(&mut self) {
        self.tab = PreviewTab::Size;
        self.focus = Focus::Preview;
        self.start_analysis();
    }

    /// Stops the running analysis (the job is the daemon's).
    pub(crate) fn cancel_analysis(&mut self) {
        if let Some(job) = self.size.running.as_ref().and_then(|r| r.job) {
            crate::jobs_ui::cancel(job);
        }
    }

    /// Collects the analysis' progress and answer. Never waits.
    pub(super) fn poll_size(&mut self, progress: Option<(u64, RewriteProgress)>) {
        let Some(run) = self.size.running.as_mut() else {
            return;
        };
        match run.baseline {
            None => run.baseline = Some(progress),
            Some(base) => {
                if let Some((job, report)) = progress {
                    if base != Some((job, report)) {
                        run.job = Some(job);
                        run.progress = Some(report);
                    }
                }
            }
        }
        let Some(answer) = run.reply.take() else {
            return;
        };
        let scope = run.scope.clone();
        self.size.running = None;
        match answer {
            Ok(analysis) => {
                self.size.result = Some(SizeResult { analysis, scope });
                self.size.cursor = 0;
                self.size.ticked.clear();
                self.size.stale = false;
            }
            Err(error) => {
                use gitpoulpi_core::TypedError as T;
                if matches!(error.typed(), Some(T::Canceled { .. })) {
                    self.notice = Some(Notice {
                        text: z::CANCELED.get().to_string(),
                        error: false,
                    });
                } else {
                    self.size.error = Some(crate::daemon_link::error_text::error_text(&error));
                }
            }
        }
    }

    // ----------------------------------------------------------- the list

    /// The rows of the list on screen, in the chosen order.
    pub(super) fn rows(&self) -> Vec<Row<'_>> {
        let Some(result) = self.size.result.as_ref() else {
            return Vec::new();
        };
        let a = &result.analysis;
        let mut rows: Vec<Row<'_>> = match self.size.view {
            SizeView::Files => a.blobs.iter().map(Row::Blob).collect(),
            SizeView::Folders => a.folders.iter().map(Row::Folder).collect(),
            SizeView::Deleted => a.deleted.iter().map(Row::Deleted).collect(),
        };
        match self.size.sort {
            SizeSort::Size => rows.sort_by(|x, y| (y.size(), x.path()).cmp(&(x.size(), y.path()))),
            SizeSort::Path => rows.sort_by(|x, y| (x.path(), y.size()).cmp(&(y.path(), x.size()))),
            SizeSort::DeletedFirst => rows.sort_by(|x, y| {
                (!x.gone(), y.size(), x.path()).cmp(&(!y.gone(), x.size(), y.path()))
            }),
        }
        rows
    }

    pub(crate) fn size_len(&self) -> usize {
        self.rows().len()
    }

    pub(crate) fn ticked_rows(&self) -> usize {
        let rows = self.rows();
        rows.iter()
            .filter(|r| self.size.ticked.contains(&r.key()))
            .count()
    }

    pub(crate) fn move_size_cursor(&mut self, delta: isize) {
        let last = self.size_len().saturating_sub(1);
        self.size.cursor = (self.size.cursor as isize + delta).clamp(0, last as isize) as usize;
    }

    /// Ticks or unticks row `index` of the list on screen.
    pub(crate) fn toggle_size_row(&mut self, index: usize) {
        let Some(key) = self.rows().get(index).map(Row::key) else {
            return;
        };
        if !self.size.ticked.remove(&key) {
            self.size.ticked.insert(key);
        }
    }

    pub(crate) fn check_all_size(&mut self) {
        let keys: Vec<String> = self.rows().iter().map(Row::key).collect();
        self.size.ticked.extend(keys);
    }

    pub(crate) fn uncheck_all_size(&mut self) {
        let keys: Vec<String> = self.rows().iter().map(Row::key).collect();
        for key in keys {
            self.size.ticked.remove(&key);
        }
    }

    pub(crate) fn set_size_view(&mut self, view: SizeView) {
        self.size.view = view;
        self.size.cursor = 0;
        self.paint.size_top.set(0);
    }

    pub(crate) fn set_size_sort(&mut self, sort: SizeSort) {
        self.size.sort = sort;
        self.size.cursor = 0;
        self.paint.size_top.set(0);
    }

    /// Bytes the ticked rows hold (an upper bound of what removing them
    /// frees: identical content kept elsewhere stays).
    fn ticked_bytes(&self) -> u64 {
        self.rows()
            .iter()
            .filter(|r| self.size.ticked.contains(&r.key()))
            .map(Row::size)
            .sum()
    }

    /// The size limit "Remove every file over" would set: just under the
    /// smallest ticked file (Files list only).
    pub(crate) fn over_limit(&self) -> Option<u64> {
        if self.size.view != SizeView::Files {
            return None;
        }
        self.rows()
            .iter()
            .filter(|r| self.size.ticked.contains(&r.key()))
            .map(Row::size)
            .min()
            .map(|s| s.saturating_sub(1))
    }

    /// The listed files above `limit`: how many and how big.
    fn over_estimate(&self, limit: u64) -> (u64, u64) {
        let Some(result) = self.size.result.as_ref() else {
            return (0, 0);
        };
        result
            .analysis
            .blobs
            .iter()
            .filter(|b| b.size > limit)
            .fold((0, 0), |(n, bytes), b| (n + 1, bytes + b.size))
    }

    /// "Remove from history": the ticked rows' paths become one `RemovePaths`
    /// rule (a path already in a rule is not repeated).
    pub(crate) fn remove_ticked_paths(&mut self) {
        let mut globs: Vec<String> = Vec::new();
        for row in self.rows() {
            if self.size.ticked.contains(&row.key()) {
                let glob = glob_for(row.path());
                if !globs.contains(&glob) {
                    globs.push(glob);
                }
            }
        }
        if globs.is_empty() {
            return self.size_notice(z::NONE_TICKED.get().to_string(), true);
        }
        let held: Vec<(usize, &String)> = self
            .recipe
            .rules
            .iter()
            .enumerate()
            .filter_map(|(i, r)| match &r.kind {
                RuleKind::RemovePaths { globs } => Some(globs.iter().map(move |g| (i, g))),
                _ => None,
            })
            .flatten()
            .collect();
        let fresh: Vec<String> = globs
            .iter()
            .filter(|g| !held.iter().any(|(_, h)| h == g))
            .cloned()
            .collect();
        if fresh.is_empty() {
            let at = held
                .iter()
                .find(|(_, h)| globs.contains(h))
                .map_or(1, |(i, _)| i + 1);
            return self.size_notice(z::already(at), false);
        }
        let n = fresh.len() as u64;
        self.recipe
            .rules
            .push(Rule::new(RuleKind::RemovePaths { globs: fresh }));
        let index = self.recipe.rules.len() - 1;
        self.after_rule_added(index);
        self.size_notice(z::added_paths(n, index + 1), false);
    }

    /// "Remove every file over": a size rule just under the smallest ticked file.
    pub(crate) fn remove_over(&mut self) {
        if self.size.view != SizeView::Files {
            return self.size_notice(z::OVER_NEEDS_FILES.get().to_string(), true);
        }
        let Some(limit) = self.over_limit() else {
            return self.size_notice(z::OVER_NEEDS_TICKED.get().to_string(), true);
        };
        let existing = self
            .recipe
            .rules
            .iter()
            .position(|r| matches!(r.kind, RuleKind::RemoveBlobsOver { .. }));
        let index = match existing {
            Some(i) => {
                // One size limit is enough: the lower one wins.
                if let RuleKind::RemoveBlobsOver { bytes } = &mut self.recipe.rules[i].kind {
                    *bytes = (*bytes).min(limit);
                }
                i
            }
            None => {
                self.recipe
                    .rules
                    .push(Rule::new(RuleKind::RemoveBlobsOver { bytes: limit }));
                self.recipe.rules.len() - 1
            }
        };
        let bytes = match &self.recipe.rules[index].kind {
            RuleKind::RemoveBlobsOver { bytes } => *bytes,
            _ => limit,
        };
        self.after_rule_added(index);
        self.size_notice(z::added_over(index + 1, human_size(bytes)), false);
    }

    fn after_rule_added(&mut self, index: usize) {
        self.rules.cursor = index;
        self.detail = DetailForm::default();
        self.sync_detail();
        self.touch();
    }

    fn size_notice(&mut self, text: String, error: bool) {
        self.notice = Some(Notice { text, error });
    }

    // --------------------------------------------------------------- keys

    pub(super) fn on_size_key(&mut self, key: KeyEvent, plain: bool) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') if plain => self.move_size_cursor(-1),
            KeyCode::Down | KeyCode::Char('j') if plain => self.move_size_cursor(1),
            KeyCode::PageUp => self.move_size_cursor(-10),
            KeyCode::PageDown => self.move_size_cursor(10),
            KeyCode::Char(' ') | KeyCode::Enter if plain => self.toggle_size_row(self.size.cursor),
            KeyCode::Char('s') if plain => self.start_analysis(),
            KeyCode::Char('v') if plain => self.set_size_view(self.size.view.next()),
            KeyCode::Char('o') if plain => self.set_size_sort(self.size.sort.next()),
            KeyCode::Char('r') if plain => self.remove_ticked_paths(),
            KeyCode::Char('b') if plain => self.remove_over(),
            _ => {}
        }
    }

    // -------------------------------------------------------------- paint

    pub(super) fn paint_size(&self, ui: &mut Ui<'_>, body: Rect) {
        let x = body.x + 1;
        let w = body.width.saturating_sub(1);
        let width = body.width.saturating_sub(2) as usize;
        let mut y = body.y;
        let bottom = body.bottom();
        let lines = |ui: &mut Ui<'_>, y: &mut u16, text: &str, style: Style, max: usize| {
            for line in wrap(text, width).into_iter().take(max) {
                if *y >= bottom {
                    return;
                }
                ui.text(x, *y, w, &line, style);
                *y += 1;
            }
        };

        // The run button (a Cancel while it runs) and the figures.
        if y < bottom {
            if self.analysing() {
                ui.button(x, y, t::BTN_CANCEL.get(), RewriteHit::Cancel, false, true);
            } else {
                ui.button(
                    x,
                    y,
                    z::BTN_ANALYSE.get(),
                    RewriteHit::SizeRun,
                    self.size.result.is_none(),
                    !self.busy(),
                );
            }
            y += 1;
        }
        if let Some(r) = &self.size.result {
            let a = &r.analysis;
            let summary = z::summary(
                a.commits,
                a.blobs_total,
                human_size(a.bytes_total),
                human_size(a.disk_total),
            );
            lines(ui, &mut y, &summary, dim(), 2);
            let totals = z::totals(
                human_size(a.bytes_in_head),
                human_size(a.bytes_history_only),
            );
            lines(ui, &mut y, &totals, dim(), 1);
            if a.blobs_cut + a.folders_cut + a.deleted_cut > 0 {
                let cut = a.blobs_cut.max(a.folders_cut).max(a.deleted_cut);
                lines(ui, &mut y, &z::capped(a.top, cut), dim(), 2);
            }
        }
        if let Some(reason) = self.size_stale_reason() {
            lines(ui, &mut y, reason.get(), warn(), 1);
        }
        if let Some(why) = &self.size.error {
            lines(ui, &mut y, why, bad(), 2);
        }
        if y >= bottom {
            return;
        }
        if self.size.result.is_none() {
            if !self.analysing() {
                lines(ui, &mut y, z::INTRO.get(), plain(), 3);
                lines(ui, &mut y, z::IDLE.get(), dim(), 3);
            }
            return;
        }

        // Which list and which order, as buttons.
        let mut cx = x;
        cx += ui.text(cx, y, 12, z::SHOW_LABEL.get(), dim()) + 1;
        for (i, view) in SizeView::ALL.iter().enumerate() {
            cx += ui.button(
                cx,
                y,
                view.label().get(),
                RewriteHit::SizeView(i as u8),
                *view == self.size.view,
                true,
            ) + 1;
        }
        cx += 1;
        cx += ui.text(cx, y, 10, z::SORT_LABEL.get(), dim()) + 1;
        for (i, sort) in SizeSort::ALL.iter().enumerate() {
            cx += ui.button(
                cx,
                y,
                sort.label().get(),
                RewriteHit::SizeSort(i as u8),
                *sort == self.size.sort,
                true,
            ) + 1;
        }
        y += 1;
        if y >= bottom {
            return;
        }

        let rows = self.rows();
        if rows.is_empty() {
            lines(ui, &mut y, z::NONE_FOUND.get(), plain(), 1);
            return;
        }

        // Estimate, list, detail, actions: the estimate, detail and actions
        // go first when the pane is short.
        let actions_y = bottom - 1;
        let spare = bottom.saturating_sub(y);
        let detail_h: u16 = if spare >= 9 { 3 } else { 0 };
        let over = self.over_limit();
        let estimate_h: u16 = if spare >= 6 { 1 } else { 0 }
            + u16::from(spare >= 10 && over.is_some() && self.size.view == SizeView::Files);
        let list_end = actions_y.saturating_sub(detail_h + estimate_h);
        let room = list_end.saturating_sub(y) as usize;
        let cursor = self.size.cursor.min(rows.len() - 1);
        let mut top = self.paint.size_top.get();
        if cursor < top {
            top = cursor;
        } else if room > 0 && cursor >= top + room {
            top = cursor + 1 - room;
        }
        top = top.min(rows.len().saturating_sub(room));
        self.paint.size_top.set(top);
        let focused = self.focus == Focus::Preview;
        let size_w = 10usize;
        let state_w = (width / 6).clamp(8, 14);
        for (slot, row) in rows.iter().enumerate().skip(top).take(room) {
            let ry = y + (slot - top) as u16;
            let band = Rect::new(body.x, ry, body.width, 1);
            let hit = RewriteHit::SizeRow(slot);
            let bg = row_bg(slot == cursor && focused, ui.hot(hit));
            dialog_kit::fill(ui.buf, band, Style::default().bg(bg));
            let on = self.size.ticked.contains(&row.key());
            let style = Style::default().fg(th().ui.text).bg(bg);
            let mut cx = band.x + 1;
            cx += ui.text(cx, ry, 4, if on { "[x] " } else { "[ ] " }, style);
            let size = human_size(row.size());
            let pad = size_w.saturating_sub(size.chars().count());
            ui.text(
                cx + pad as u16,
                ry,
                size_w as u16,
                &size,
                Style::default().fg(th().ui.accent).bg(bg),
            );
            cx += size_w as u16 + 1;
            let state = match row {
                Row::Blob(b) => match b.state {
                    BlobState::InHead => z::STATE_HEAD.get(),
                    BlobState::Replaced => z::STATE_OLD.get(),
                    BlobState::Deleted => z::STATE_DELETED.get(),
                },
                Row::Folder(f) if !f.in_head => z::FOLDER_GONE.get(),
                Row::Folder(_) => "",
                Row::Deleted(_) => z::STATE_DELETED.get(),
            };
            let state_style = if row.gone() {
                Style::default().fg(th().status.warning).bg(bg)
            } else {
                Style::default().fg(th().ui.muted).bg(bg)
            };
            ui.text(cx, ry, state_w as u16, state, state_style);
            cx += state_w as u16 + 1;
            let path = match row {
                Row::Blob(b) if b.more_paths > 0 => z::paths_more(u64::from(b.more_paths), &b.path),
                Row::Folder(f) => format!("{}/", f.path),
                other => other.path().to_string(),
            };
            ui.text(
                cx,
                ry,
                band.right().saturating_sub(cx + 1),
                &path,
                Style::default().fg(th().ui.text).bg(bg),
            );
            ui.fine.push((band, hit));
        }

        let n = self.ticked_rows();
        if estimate_h > 0 && n > 0 {
            let text = z::estimate(n as u64, human_size(self.ticked_bytes()));
            let line = wrap(&text, width).into_iter().next().unwrap_or_default();
            ui.text(x, list_end, w, &line, warn());
            if let (Some(limit), true) = (over, estimate_h > 1) {
                let (count, bytes) = self.over_estimate(limit);
                let text = z::estimate_over(count, human_size(bytes));
                let line = wrap(&text, width).into_iter().next().unwrap_or_default();
                ui.text(x, list_end + 1, w, &line, dim());
            }
        }
        if detail_h > 0 {
            let dy = list_end + estimate_h;
            let row = rows[cursor];
            ui.text(x, dy, w, row.path(), bold(false));
            match row {
                Row::Blob(b) => {
                    ui.text(
                        x,
                        dy + 1,
                        w,
                        &z::detail_span(short(&b.first_commit), short(&b.last_commit)),
                        dim(),
                    );
                    ui.text(
                        x,
                        dy + 2,
                        w,
                        &z::detail_blob(human_size(b.size), human_size(b.disk_size)),
                        dim(),
                    );
                }
                Row::Folder(f) => {
                    ui.text(
                        x,
                        dy + 1,
                        w,
                        &z::detail_folder(f.blobs, human_size(f.size)),
                        dim(),
                    );
                }
                Row::Deleted(d) => {
                    ui.text(
                        x,
                        dy + 1,
                        w,
                        &z::detail_span(short(&d.first_commit), short(&d.last_commit)),
                        dim(),
                    );
                    ui.text(
                        x,
                        dy + 2,
                        w,
                        &z::detail_deleted(d.versions, human_size(d.size)),
                        dim(),
                    );
                }
            }
        }

        // Actions.
        let remove = if n == 0 {
            z::BTN_REMOVE.get().to_string()
        } else {
            z::btn_remove_n(n)
        };
        let mut ax = x;
        ax += ui.button(
            ax,
            actions_y,
            &remove,
            RewriteHit::SizeRemovePaths,
            n > 0,
            n > 0 && !self.busy(),
        ) + 1;
        let over_label = match over {
            Some(limit) => z::btn_remove_over(human_size(limit + 1)),
            None => z::btn_remove_over("..."),
        };
        ax += ui.button(
            ax,
            actions_y,
            &over_label,
            RewriteHit::SizeRemoveOver,
            false,
            over.is_some() && !self.busy(),
        ) + 1;
        ax += ui.button(
            ax,
            actions_y,
            t::PROTECTED_ALL.get(),
            RewriteHit::SizeAll,
            false,
            true,
        ) + 1;
        ui.button(
            ax,
            actions_y,
            t::PROTECTED_NONE.get(),
            RewriteHit::SizeNone,
            false,
            true,
        );
    }

    /// The tab a click on the Size hits belongs to.
    pub(super) fn on_size_press(&mut self, hit: RewriteHit) {
        self.focus = Focus::Preview;
        self.tab = PreviewTab::Size;
        match hit {
            RewriteHit::SizeRun => self.start_analysis(),
            RewriteHit::SizeRemovePaths => self.remove_ticked_paths(),
            RewriteHit::SizeRemoveOver => self.remove_over(),
            RewriteHit::SizeAll => self.check_all_size(),
            RewriteHit::SizeNone => self.uncheck_all_size(),
            RewriteHit::SizeView(i) => {
                if let Some(view) = SizeView::ALL.get(usize::from(i)) {
                    self.set_size_view(*view);
                }
            }
            RewriteHit::SizeSort(i) => {
                if let Some(sort) = SizeSort::ALL.get(usize::from(i)) {
                    self.set_size_sort(*sort);
                }
            }
            RewriteHit::SizeRow(i) => {
                self.size.cursor = i;
                self.toggle_size_row(i);
            }
            _ => {}
        }
    }
}
