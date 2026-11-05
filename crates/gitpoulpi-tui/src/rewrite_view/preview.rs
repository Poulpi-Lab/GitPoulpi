//! The preview pane: tabs (Summary, Commits, Refs, Impact), the progress of
//! a running preview, the stale and out-of-date banners, and the result
//! panel that replaces it after an Apply.
//!
//! Nothing here asks the daemon for anything while it paints: the Commits
//! tab is a virtual table over the rows already paged in, and the next page
//! is asked for by `poll` from the window the last paint recorded.

use ratatui::layout::Rect;
use ratatui::style::Style;

use gitpoulpi_core::domain::rewrite::trailers;
use gitpoulpi_core::domain::rewrite::{
    CommitChange, PathChange, PushState, RefChange, RewriteField, RewritePhase, RewriteWarning,
};
use gitpoulpi_core::domain::Oid;
use gitpoulpi_i18n::{msg::rewrite_view as t, M};

use super::bars::BarId;
use super::render::{bad_bold, bold, dim, pane, plain, row_name, warn, wrap, RewriteHit, Ui};
use super::{Focus, PlanState, PreviewTab, RewriteView, Run};
use crate::dialog_kit;
use crate::display_width;
use crate::th;

/// The narrowest progress bar worth drawing; the tabs give way before it.
const MIN_BAR: u16 = 4;

/// `old → new` of a ref or commit, seven hex digits each.
pub(super) fn short(oid: &Oid) -> String {
    let hex = oid.to_hex();
    hex[..hex.len().min(7)].to_string()
}

pub(super) fn ref_short(name: &str) -> &str {
    name.strip_prefix("refs/heads/")
        .or_else(|| name.strip_prefix("refs/tags/"))
        .unwrap_or(name)
}

pub(super) fn field_label(field: &RewriteField) -> M {
    match field {
        RewriteField::AuthorName => t::FLD_AUTHOR_NAME,
        RewriteField::AuthorEmail => t::FLD_AUTHOR_EMAIL,
        RewriteField::AuthorDate => t::FLD_AUTHOR_DATE,
        RewriteField::CommitterName => t::FLD_COMMITTER_NAME,
        RewriteField::CommitterEmail => t::FLD_COMMITTER_EMAIL,
        RewriteField::CommitterDate => t::FLD_COMMITTER_DATE,
        RewriteField::Message => t::FLD_MESSAGE,
        RewriteField::Tree => t::FLD_TREE,
        RewriteField::Signature => t::FLD_SIGNATURE,
        RewriteField::Encoding => t::FLD_ENCODING,
    }
}

fn one_line(text: &str) -> String {
    text.lines().next().unwrap_or("").trim().to_string()
}

/// What changed in a message whose subject did not: the trailers added
/// (`+Key: value`) and removed (`-Key: value`), else the first line that
/// differs. `None` when the subject itself changed (the row then shows the
/// two subjects).
fn message_delta(old: &str, new: &str) -> Option<String> {
    if one_line(old) != one_line(new) {
        return None;
    }
    let list = |m: &str| -> Vec<String> {
        trailers::parse(m)
            .into_iter()
            .map(|t| format!("{}: {}", t.key, t.value))
            .collect()
    };
    let (before, after) = (list(old), list(new));
    let mut parts: Vec<String> = Vec::new();
    let mut left = before.clone();
    for t in &after {
        match left.iter().position(|b| b == t) {
            Some(i) => {
                left.remove(i);
            }
            None => parts.push(format!("+{t}")),
        }
    }
    parts.extend(left.iter().map(|t| format!("-{t}")));
    if !parts.is_empty() {
        return Some(parts.join(" "));
    }
    // A body edit: the first line that differs.
    let (mut a, mut b) = (old.lines(), new.lines());
    loop {
        match (a.next(), b.next()) {
            (Some(x), Some(y)) if x == y => continue,
            (Some(x), Some(y)) => return Some(format!("{} \u{2192} {}", x.trim(), y.trim())),
            (Some(x), None) => return Some(format!("-{}", x.trim())),
            (None, Some(y)) => return Some(format!("+{}", y.trim())),
            (None, None) => return None,
        }
    }
}

/// What one commit row says: the ids, then what changed in it.
fn row_text(row: &CommitChange) -> String {
    let new = match &row.new {
        Some(id) => short(id),
        None => t::ROW_DROPPED.get().to_string(),
    };
    let mut out = format!("{} \u{2192} {}", short(&row.old), new);
    let mut parts: Vec<String> = row
        .fields
        .iter()
        // A tree id says nothing: the files that changed are listed below.
        .filter(|f| f.field != RewriteField::Tree || row.paths.is_empty())
        .map(|f| {
            if f.field == RewriteField::Tree {
                return field_label(&f.field).get().to_string();
            }
            if f.field == RewriteField::Message {
                if let Some(delta) = message_delta(&f.old, &f.new) {
                    return format!("{}: {delta}", field_label(&f.field).get());
                }
            }
            format!(
                "{}: {} \u{2192} {}",
                field_label(&f.field).get(),
                one_line(&f.old),
                one_line(&f.new)
            )
        })
        .collect();
    for path in &row.paths {
        parts.push(match path {
            PathChange::Removed { path } => t::path_removed(path),
            PathChange::Renamed { from, to } => t::path_renamed(from, to),
            PathChange::Modified { path } => t::path_modified(path),
            PathChange::LfsImported { path } => t::path_lfs_in(path),
            PathChange::LfsExported { path } => t::path_lfs_out(path),
        });
    }
    if !parts.is_empty() {
        out.push_str("  ");
        out.push_str(&parts.join("; "));
    }
    out
}

pub(super) fn ref_line(change: &RefChange) -> String {
    let new = match &change.new {
        Some(id) => short(id),
        None => t::REF_DELETED.get().to_string(),
    };
    let mut out = format!(
        "{}  {} \u{2192} {}",
        ref_short(&change.name),
        short(&change.old),
        new
    );
    match &change.pushed {
        PushState::LocalOnly => {}
        PushState::OnRemote { remote, .. } => {
            out.push_str("  ");
            out.push_str(&t::ref_on_remote(remote));
        }
    }
    if let Some(path) = &change.checked_out_in {
        out.push_str("  ");
        out.push_str(&t::ref_checked_out(path.display()));
    }
    out
}

impl RewriteView {
    pub(super) fn paint_preview(&self, ui: &mut Ui<'_>, area: Rect) {
        let focused = self.focus == Focus::Preview;
        let inner = pane(ui.buf, area, t::PREVIEW_TITLE.get(), focused);
        ui.coarse.push((area, RewriteHit::Pane(Focus::Preview)));
        if inner.height < 3 || inner.width < 20 {
            return;
        }
        if let Run::Done(_) = &self.run {
            self.paint_result(ui, inner);
            return;
        }
        // Tabs, then (running) the progress at the right end: the tabs that
        // would run into it are left out while the job runs.
        let job = match &self.run {
            Run::Previewing { progress, .. } => Some((*progress, None)),
            _ => self
                .secrets
                .running
                .as_ref()
                .map(|s| {
                    (
                        s.progress,
                        Some(gitpoulpi_i18n::msg::rewrite_secrets::SCANNING),
                    )
                })
                .or_else(|| {
                    self.size.running.as_ref().map(|r| {
                        (
                            r.progress,
                            Some(gitpoulpi_i18n::msg::rewrite_size::ANALYSING),
                        )
                    })
                }),
        };
        // The bar shrinks to `MIN_BAR` cells, then the tabs that would run
        // into it are left out (3 cells: the gaps around the bar and text).
        let reserve = job.map_or(0, |(p, label)| self.progress_need(p, label) + 3);
        let tabs_end = inner.right().saturating_sub(reserve);
        let mut x = inner.x + 1;
        for tab in PreviewTab::ALL {
            let label = format!(" {} ", tab.label().get());
            let w = display_width(&label) as u16;
            let hit = RewriteHit::Tab(tab);
            let hot = ui.hot(hit);
            let style = if tab == self.tab {
                bold(true).bg(th().ui.selection)
            } else if hot {
                Style::default().fg(th().ui.text).bg(th().ui.hover)
            } else {
                Style::default().fg(th().ui.muted).bg(th().ui.panel)
            };
            if x + w > tabs_end {
                break;
            }
            ui.text(x, inner.y, w, &label, style);
            ui.fine.push((Rect::new(x, inner.y, w, 1), hit));
            x += w + 1;
        }
        let body = Rect::new(inner.x, inner.y + 1, inner.width, inner.height - 1);
        match &self.run {
            Run::Previewing { progress, .. } => {
                self.paint_progress(ui, inner, x, *progress, None);
                return self.paint_running(ui, body);
            }
            Run::Applying { .. } => {
                ui.text(
                    body.x + 1,
                    body.y,
                    body.width - 1,
                    t::APPLYING.get(),
                    plain(),
                );
                return;
            }
            _ => {}
        }
        if let Some(scan) = &self.secrets.running {
            // Labelled by what the job is, not by the phase it reports: a
            // scan reads trees and blobs under the preview's `Rewriting`.
            self.paint_progress(
                ui,
                inner,
                x,
                scan.progress,
                Some(gitpoulpi_i18n::msg::rewrite_secrets::SCANNING),
            );
        }
        if let Some(run) = &self.size.running {
            self.paint_progress(
                ui,
                inner,
                x,
                run.progress,
                Some(gitpoulpi_i18n::msg::rewrite_size::ANALYSING),
            );
        }
        if self.tab == PreviewTab::Secrets {
            return self.paint_secrets(ui, body);
        }
        if self.tab == PreviewTab::Identities {
            return self.paint_identities(ui, body);
        }
        if self.tab == PreviewTab::Size {
            return self.paint_size(ui, body);
        }
        if let Some(refusal) = &self.refusal {
            return self.paint_refusal(ui, body, refusal);
        }
        let Some(plan) = &self.plan else {
            let text = t::PREVIEW_EMPTY.get();
            for (i, line) in wrap(text, body.width as usize - 2).iter().enumerate() {
                if (i as u16) < body.height {
                    ui.text(body.x + 1, body.y + i as u16, body.width - 1, line, dim());
                }
            }
            return;
        };
        let mut body = body;
        let banner = match self.plan_state {
            PlanState::OutOfDate => Some(t::B_PREVIEW_AGAIN.get()),
            PlanState::Stale => Some(t::B_STALE.get()),
            _ => None,
        };
        if let Some(text) = banner {
            ui.text(body.x + 1, body.y, body.width - 1, text, warn());
            body = Rect::new(
                body.x,
                body.y + 1,
                body.width,
                body.height.saturating_sub(1),
            );
        }
        if body.height == 0 {
            return;
        }
        match self.tab {
            PreviewTab::Summary => self.paint_summary(ui, body, plan),
            PreviewTab::Commits => self.paint_commits(ui, body),
            PreviewTab::Refs => self.paint_refs(ui, body, plan),
            PreviewTab::Impact => self.paint_impact(ui, body, plan),
            PreviewTab::Secrets | PreviewTab::Identities | PreviewTab::Size => {}
        }
    }

    /// The least width the progress bar and its text take (the text at its
    /// widest, when every commit is done), so the tabs do not move while it
    /// counts.
    fn progress_need(
        &self,
        progress: Option<gitpoulpi_core::domain::rewrite::RewriteProgress>,
        label: Option<M>,
    ) -> u16 {
        let phase = match progress.map(|p| p.phase) {
            Some(RewritePhase::Rewriting) => t::PHASE_REWRITING,
            Some(RewritePhase::Indexing) => t::PHASE_INDEXING,
            Some(RewritePhase::Updating) => t::PHASE_UPDATING,
            Some(RewritePhase::Signing) => t::PHASE_SIGNING,
            _ => t::PHASE_WALKING,
        };
        let total = progress.map_or(0, |p| p.total);
        let text = t::progress(label.unwrap_or(phase).get(), total, total);
        MIN_BAR + 1 + display_width(&text) as u16
    }

    fn paint_progress(
        &self,
        ui: &mut Ui<'_>,
        inner: Rect,
        after_tabs: u16,
        progress: Option<gitpoulpi_core::domain::rewrite::RewriteProgress>,
        label: Option<M>,
    ) {
        let (phase, done, total) = match progress {
            Some(p) => (
                match p.phase {
                    RewritePhase::Walking => t::PHASE_WALKING,
                    RewritePhase::Rewriting => t::PHASE_REWRITING,
                    RewritePhase::Indexing => t::PHASE_INDEXING,
                    RewritePhase::Updating => t::PHASE_UPDATING,
                    RewritePhase::Signing => t::PHASE_SIGNING,
                },
                p.done,
                p.total,
            ),
            None => (t::PHASE_WALKING, 0, 0),
        };
        let text = t::progress(label.unwrap_or(phase).get(), done, total);
        // A shorter bar when the tabs leave little room (more tabs, a narrow
        // terminal), none under four cells.
        let text_w = display_width(&text) as u16;
        let room = inner.right().saturating_sub(after_tabs + text_w + 3);
        let bar_w: u16 = room.min(14);
        if bar_w < MIN_BAR {
            return;
        }
        let need = bar_w + 1 + text_w;
        let x = inner.right() - need - 1;
        let filled = if total == 0 {
            0
        } else {
            ((done.min(total) as u128 * bar_w as u128) / total as u128) as u16
        };
        let bar: String = (0..bar_w)
            .map(|i| if i < filled { '\u{2593}' } else { '\u{2591}' })
            .collect();
        ui.text(
            x,
            inner.y,
            bar_w,
            &bar,
            Style::default().fg(th().ui.accent).bg(th().ui.panel),
        );
        ui.text(x + bar_w + 1, inner.y, need - bar_w - 1, &text, dim());
    }

    fn paint_running(&self, ui: &mut Ui<'_>, body: Rect) {
        // The progress is in the header; the body says what is going on and
        // that nothing is touched yet.
        ui.text(
            body.x + 1,
            body.y,
            body.width - 1,
            t::PREVIEW_RUNNING.get(),
            plain(),
        );
    }

    // ------------------------------------------------------------ summary

    fn paint_summary(
        &self,
        ui: &mut Ui<'_>,
        body: Rect,
        plan: &gitpoulpi_core::domain::rewrite::RewritePlan,
    ) {
        let c = &plan.counts;
        let mut lines: Vec<(String, Style)> = Vec::new();
        let same = c.commits_walked.saturating_sub(c.commits_changed);
        lines.push((
            t::sum_commits(c.commits_changed, c.commits_walked, same),
            bold(false),
        ));
        if c.commits_dropped > 0 {
            lines.push((t::sum_dropped(c.commits_dropped), plain()));
        }
        lines.push((
            t::sum_refs(u64::from(c.refs_moved), u64::from(c.tags_moved)),
            plain(),
        ));
        for line in self.extract_summary() {
            lines.push((line, dim()));
        }
        if c.blobs_changed > 0 {
            lines.push((t::sum_blobs_changed(c.blobs_changed), plain()));
        }
        if c.lfs_blobs > 0 {
            lines.push((
                t::sum_lfs_in(
                    c.lfs_blobs,
                    super::rules_form::format_size(c.lfs_bytes),
                    c.lfs_objects,
                ),
                plain(),
            ));
        }
        if c.lfs_restored > 0 {
            lines.push((t::sum_lfs_out(c.lfs_restored), plain()));
        }
        if c.blobs_skipped_binary > 0 {
            lines.push((t::sum_skipped_binary(c.blobs_skipped_binary), warn()));
        }
        if c.blobs_skipped_large > 0 {
            lines.push((t::sum_skipped_large(c.blobs_skipped_large), warn()));
        }
        if c.blobs_skipped_undecodable > 0 {
            lines.push((
                t::sum_skipped_undecodable(c.blobs_skipped_undecodable),
                warn(),
            ));
        }
        if c.resigned > 0 {
            lines.push((
                super::signing::resigned_line(c.resigned, c.signing_millis),
                plain(),
            ));
        }
        if c.message_hashes_updated > 0 {
            lines.push((t::sum_hashes_updated(c.message_hashes_updated), plain()));
        }
        if c.encoding_left > 0 {
            lines.push((t::sum_encoding_left(c.encoding_left), warn()));
        }
        if c.message_hashes_left > 0 {
            lines.push((t::sum_hashes_left(c.message_hashes_left), warn()));
        }
        let rows = self.rule_rows();
        for (i, row) in rows.iter().enumerate() {
            if !row.enabled {
                continue;
            }
            let commits = plan
                .per_rule
                .iter()
                .find(|r| r.rule == i)
                .map_or(0, |r| r.commits);
            let name = row_name(row);
            if commits == 0 {
                lines.push((t::sum_rule_nothing(i + 1, name), warn()));
            } else {
                lines.push((t::sum_rule_count(i + 1, name, commits), dim()));
            }
        }
        for warning in &plan.warnings {
            if let Some(text) = warning_text(warning) {
                lines.push((text, warn()));
            }
        }
        let lines: Vec<(u16, String, Style)> = lines.into_iter().map(|(t, s)| (0, t, s)).collect();
        let y = self.paint_text(ui, body, &lines);
        // Pushed commits and an author rule: a `.mailmap` would do (S16).
        self.paint_mailmap_offer(ui, body, y);
    }

    // ------------------------------------------------------------ commits

    fn paint_commits(&self, ui: &mut Ui<'_>, body: Rect) {
        let filter = super::ROW_FILTERS
            .iter()
            .find(|(f, _)| *f == self.rows.filter)
            .map_or("", |(_, m)| m.get());
        let w = (display_width(filter) as u16 + 5).clamp(14, 22);
        ui.combo(
            Rect::new(body.x + 1, body.y, w, 1),
            filter,
            RewriteHit::FilterCombo,
            super::render::Anchor::Filter,
            false,
        );
        let count = t::counts_commits(self.rows.total as u64);
        ui.text(
            body.x + w + 3,
            body.y,
            body.width.saturating_sub(w + 4),
            &count,
            dim(),
        );
        let list = Rect::new(
            body.x,
            body.y + 1,
            body.width,
            body.height.saturating_sub(1),
        );
        let visible = list.height as usize;
        self.paint.rows_visible.set(visible);
        let total = self.rows.total.max(self.rows.loaded.len());
        let cursor = self.rows.cursor.min(total.saturating_sub(1));
        let top = self.settle(
            BarId::Commits,
            Some(cursor),
            self.paint.rows_top.get(),
            visible,
            total,
        );
        self.paint.rows_top.set(top);
        self.paint_bar(
            ui,
            BarId::Commits,
            (list.right(), list.y, list.height),
            total,
            visible,
            top,
            Some(cursor),
        );
        if total == 0 {
            ui.text(
                list.x + 1,
                list.y,
                list.width - 1,
                t::ROWS_NONE.get(),
                dim(),
            );
            return;
        }
        let focused = self.focus == Focus::Preview;
        for slot in 0..visible.min(total - top) {
            let index = top + slot;
            let y = list.y + slot as u16;
            let band = Rect::new(list.x, y, list.width, 1);
            let hit = RewriteHit::Row(index);
            let bg = super::render::row_bg(index == cursor && focused, ui.hot(hit));
            dialog_kit::fill(ui.buf, band, Style::default().bg(bg));
            match self.rows.loaded.get(index) {
                Some(row) => {
                    let mark = if row.pushed { "\u{25CF} " } else { "  " };
                    let mut x = band.x + 1;
                    x += ui.text(
                        x,
                        y,
                        2,
                        mark,
                        Style::default().fg(th().status.warning).bg(bg),
                    );
                    // A commit the rewrite removes stands out from the ones it
                    // only changes.
                    let fg = if row.new.is_none() {
                        th().status.warning
                    } else {
                        th().ui.text
                    };
                    ui.text(
                        x,
                        y,
                        band.right().saturating_sub(x + 1),
                        &row_text(row),
                        Style::default().fg(fg).bg(bg),
                    );
                }
                None => {
                    ui.text(
                        band.x + 3,
                        y,
                        band.width - 4,
                        t::ROWS_LOADING.get(),
                        Style::default().fg(th().ui.muted).bg(bg),
                    );
                }
            }
            ui.coarse.push((band, hit));
        }
    }

    // ------------------------------------------------------------ refusal

    /// A refusal with a list: the reason, what is in the way, what to do.
    fn paint_refusal(&self, ui: &mut Ui<'_>, body: Rect, refusal: &super::Refusal) {
        use gitpoulpi_core::domain::rewrite::RewriteRefusal as R;
        // S21: the missing LFS objects can be fetched from here.
        let body = if refusal.kind == R::LfsObjectsMissing {
            let by = body.bottom().saturating_sub(1);
            let idle = !self.busy() && self.push.lfs_fetch_wait.is_none();
            ui.button(
                body.x + 1,
                by,
                t::BTN_LFS_FETCH.get(),
                RewriteHit::LfsFetch,
                true,
                idle,
            );
            Rect::new(body.x, body.y, body.width, body.height.saturating_sub(2))
        } else {
            body
        };
        let width = body.width as usize - 2;
        let mut y = body.y;
        let reason = crate::daemon_link::error_text::rewrite_refusal(refusal.kind).get();
        let (what, hint) = match refusal.kind {
            R::DirtyPathsTouched => (t::REFUSAL_DIRTY_LIST, t::REFUSAL_DIRTY_HINT),
            R::CheckedOutElsewhere => (t::REFUSAL_ELSEWHERE_LIST, t::REFUSAL_ELSEWHERE_HINT),
            R::ScriptFailed => (t::REFUSAL_SCRIPT_LIST, t::REFUSAL_SCRIPT_HINT),
            R::LfsObjectsMissing => (t::REFUSAL_LFS_LIST, t::REFUSAL_LFS_HINT),
            R::PathCollision => (
                gitpoulpi_i18n::msg::rewrite_paths::REFUSAL_COLLISION_LIST,
                gitpoulpi_i18n::msg::rewrite_paths::REFUSAL_COLLISION_HINT,
            ),
            _ => (t::REFUSAL_PARTIAL_LIST, t::REFUSAL_PARTIAL_HINT),
        };
        for line in wrap(reason, width) {
            if y >= body.bottom() {
                return;
            }
            ui.text(body.x + 1, y, body.width - 1, &line, bad_bold());
            y += 1;
        }
        if !refusal.lines.is_empty() && y < body.bottom() {
            ui.text(body.x + 1, y, body.width - 1, what.get(), dim());
            y += 1;
        }
        // Keep the last two rows for the hint; what is in the way scrolls
        // between them and the reason, with the bar when it is long.
        let list_end = body.bottom().saturating_sub(2).max(y);
        let items: Vec<(u16, String, Style)> = refusal
            .lines
            .iter()
            .map(|l| (2, l.clone(), plain()))
            .collect();
        let list = Rect::new(body.x, y, body.width, list_end - y);
        let y = self.paint_text(ui, list, &items);
        for (y, line) in (y..body.bottom()).zip(wrap(hint.get(), width)) {
            ui.text(body.x + 1, y, body.width - 1, &line, dim());
        }
    }

    // --------------------------------------------------------------- refs

    fn paint_refs(
        &self,
        ui: &mut Ui<'_>,
        body: Rect,
        plan: &gitpoulpi_core::domain::rewrite::RewritePlan,
    ) {
        // S23: a line at the bottom when the host's protection is unknown.
        let note = self.host_unknown_line();
        let full = body;
        let body = if note.is_some() && body.height > 3 {
            Rect::new(body.x, body.y, body.width, body.height - 1)
        } else {
            body
        };
        if let Some(line) = &note {
            ui.text(full.x + 1, full.bottom() - 1, full.width - 1, line, dim());
        }
        let protected = self.protected();
        let block = if protected.is_empty() {
            0
        } else {
            (protected.len() as u16 + 2).min(body.height / 2 + 1)
        };
        let list_h = body.height.saturating_sub(block);
        let moving: Vec<&RefChange> = plan.refs.iter().filter(|r| r.included).collect();
        if moving.is_empty() && protected.is_empty() {
            ui.text(
                body.x + 1,
                body.y,
                body.width - 1,
                t::REFS_NONE.get(),
                dim(),
            );
        }
        let moving_top = self.settle(
            BarId::RefsMoving,
            None,
            self.paint.refs_moving_top.get(),
            list_h as usize,
            moving.len(),
        );
        self.paint.refs_moving_top.set(moving_top);
        self.paint_bar(
            ui,
            BarId::RefsMoving,
            (body.right(), body.y, list_h),
            moving.len(),
            list_h as usize,
            moving_top,
            None,
        );
        for (i, change) in moving
            .iter()
            .skip(moving_top)
            .take(list_h as usize)
            .enumerate()
        {
            ui.text(
                body.x + 1,
                body.y + i as u16,
                body.width - 1,
                &ref_line(change),
                plain(),
            );
        }
        if protected.is_empty() {
            return;
        }
        let top = body.y + list_h;
        ui.text(
            body.x + 1,
            top,
            body.width - 1,
            t::REFS_PROTECTED.get(),
            bold(false),
        );
        // Check all / Uncheck all at the right end of the heading.
        let none = t::PROTECTED_NONE.get();
        let all = t::PROTECTED_ALL.get();
        let nw = dialog_kit::button_width(none);
        let aw = dialog_kit::button_width(all);
        if body.width > nw + aw + 30 {
            let nx = body.right() - nw - 1;
            let ax = nx - aw - 1;
            ui.button(ax, top, all, RewriteHit::ProtectedAll, false, true);
            ui.button(nx, top, none, RewriteHit::ProtectedNone, false, true);
        }
        let focused = self.focus == Focus::Preview;
        let room = block.saturating_sub(1) as usize;
        let first = self.settle(
            BarId::RefsProtected,
            Some(self.refs_cursor),
            self.paint.refs_top.get(),
            room,
            protected.len(),
        );
        self.paint.refs_top.set(first);
        self.paint_bar(
            ui,
            BarId::RefsProtected,
            (
                body.right(),
                top + 1,
                (room as u16).min(body.bottom().saturating_sub(top + 1)),
            ),
            protected.len(),
            room,
            first,
            Some(self.refs_cursor),
        );
        for (slot, entry) in protected.iter().enumerate().skip(first).take(room) {
            let i = slot;
            let y = top + 1 + (slot - first) as u16;
            if y >= body.bottom() {
                break;
            }
            let hit = RewriteHit::ProtectedRow(i);
            let bg = super::render::row_bg(i == self.refs_cursor && focused, ui.hot(hit));
            let band = Rect::new(body.x, y, body.width, 1);
            dialog_kit::fill(ui.buf, band, Style::default().bg(bg));
            let mark = if entry.included { "[x]" } else { "[ ]" };
            let mut text = format!("{mark} {}", ref_short(&entry.name));
            if entry.pushed {
                text.push_str("  ");
                text.push_str(t::REF_PUSHED.get());
            }
            // S23: why it is protected, when the host is the reason.
            if let Some(label) = self.host_label(&entry.name) {
                text.push_str("  ");
                text.push_str(&label);
            }
            ui.text(
                band.x + 1,
                y,
                band.width - 1,
                &text,
                Style::default().fg(th().ui.text).bg(bg),
            );
            ui.fine.push((band, hit));
        }
    }

    // ------------------------------------------------------------- impact

    fn paint_impact(
        &self,
        ui: &mut Ui<'_>,
        body: Rect,
        plan: &gitpoulpi_core::domain::rewrite::RewritePlan,
    ) {
        let mut lines: Vec<(String, Style)> = Vec::new();
        let mut any_pushed = false;
        for change in plan.refs.iter().filter(|r| r.included) {
            if let PushState::OnRemote {
                remote,
                rewritten_on_remote,
                ..
            } = &change.pushed
            {
                any_pushed = true;
                lines.push((
                    t::impact_pushed(
                        u64::from(*rewritten_on_remote),
                        ref_short(&change.name),
                        remote,
                    ),
                    warn(),
                ));
            }
        }
        if any_pushed {
            lines.push((t::IMPACT_PRS.get().to_string(), plain()));
        } else {
            lines.push((t::IMPACT_NONE.get().to_string(), plain()));
        }
        for warning in &plan.warnings {
            if matches!(
                warning,
                RewriteWarning::SignaturesAffected { .. }
                    | RewriteWarning::SignedTagsDropped { .. }
                    | RewriteWarning::OtherBranchesKeepOldHistory { .. }
                    | RewriteWarning::StashesKeepOldHistory { .. }
            ) {
                if let Some(text) = warning_text(warning) {
                    lines.push((text, warn()));
                }
            }
        }
        let lines: Vec<(u16, String, Style)> = lines.into_iter().map(|(t, s)| (0, t, s)).collect();
        self.paint_text(ui, body, &lines);
    }

    /// Asks for the rows the Commits tab is about to show. Called by `poll`
    /// each frame, with the window the last paint recorded.
    pub(super) fn page_rows(&mut self) {
        if self.tab == PreviewTab::Commits && self.plan.is_some() {
            let upto = self.paint.rows_top.get() + self.paint.rows_visible.get();
            self.want_rows(upto);
        }
    }
}

fn warning_text(warning: &RewriteWarning) -> Option<String> {
    Some(match warning {
        RewriteWarning::SignaturesAffected { count } => t::w_signatures(*count),
        RewriteWarning::PushedRefs { count } => t::w_pushed_refs(u64::from(*count)),
        RewriteWarning::SignedTagsDropped { count } => t::w_signed_tags(u64::from(*count)),
        RewriteWarning::ProtectedExcluded { refs } => {
            let names: Vec<&str> = refs.iter().map(|r| ref_short(r)).collect();
            t::w_protected(names.join(", "))
        }
        RewriteWarning::OtherBranchesKeepOldHistory { refs } => {
            let names: Vec<&str> = refs.iter().map(|r| ref_short(r)).collect();
            t::w_other_branches(names.join(", "))
        }
        RewriteWarning::StashesKeepOldHistory { count } => t::w_stashes(u64::from(*count)),
    })
}
