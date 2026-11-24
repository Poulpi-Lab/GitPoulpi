//! The diff pane of the Files and Commits pages: the Diff view's own render
//! (`diff_view::render_diff_annotated`) with the review's rows in it, its
//! display toggles (the same bar, keys and preferences), and the paint walk
//! that draws the rows, the review's controls over them and the scrollbar's
//! overview ruler - registering the hit rects as it goes.

use std::rc::Rc;
use std::sync::Arc;

use gitpoulpi_core::domain::{FileDiff, Oid};
use gitpoulpi_i18n::msg::review_ai as ai_msg;
use gitpoulpi_i18n::msg::review_view as msg;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Widget};

use super::diffrows::{self, file_header, is_marker, kind_of, RowKind};
use super::draw::{block, sev_color, Hit};
use super::model::{Focus, Sev};
use super::recode::{Done, Key, Sides};
use super::whole::{needs_content, Whole};
use super::{PaneId, ReviewView, SPIN_AFTER};
use crate::dialog_kit::{self, ButtonState};
use crate::diff_recode::Recoded;
use crate::diff_view::{
    annot_region, apply_cursor, change_marks, diff_bar_of, h_scroll_max, h_track_content,
    longest_line, paint_band, render_diff_annotated, AnnotSide, Annotator, DiffBarButton,
    DiffRender, DiffRenderOpts, DiffViewMode,
};
use crate::review_cmd::Tab;
use crate::scroll_marks::{self, Mark};
use crate::th;
use crate::ui_kit::display_width;

/// The Diff view's display toggles, as the diff pane's bar offers them (the
/// view tabs, Edit and the minimap stay with the Diff view).
pub(super) const BAR_BUTTONS: &[DiffBarButton] = &[
    DiffBarButton::PrevHunk,
    DiffBarButton::NextHunk,
    DiffBarButton::Mode(DiffViewMode::Inline),
    DiffBarButton::Mode(DiffViewMode::Split),
    DiffBarButton::ToggleHunksOnly,
    DiffBarButton::ToggleWhitespace,
    DiffBarButton::ToggleWordWrap,
    DiffBarButton::ToggleMinimap,
    DiffBarButton::Encoding,
];

/// One file ready to render: its diff (the re-read one when an encoding is
/// chosen), the new side's whole text for the whole-file layout, and what to
/// say above it.
struct Prepared<'a> {
    original: &'a FileDiff,
    recoded: Option<Arc<Recoded>>,
    /// `true`: the whole-file layout needs the new side of this file.
    wants_whole: bool,
    whole: Option<Whole>,
    notice: Option<String>,
}

impl Prepared<'_> {
    fn diff(&self) -> &FileDiff {
        self.recoded.as_ref().map_or(self.original, |r| &r.diff)
    }

    fn lines(&self) -> Option<&[String]> {
        match &self.whole {
            Some(Whole::Ready(lines)) if self.wants_whole => Some(&lines[..]),
            _ => None,
        }
    }

    /// The widest line, in cells: what the sideways scroll runs over.
    fn longest(&self) -> usize {
        longest_line(self.diff(), self.lines())
    }
}

impl ReviewView {
    /// What the encoding dropdown applies to: the open file on the Files page,
    /// the file under the cursor on the Commits page.
    pub(super) fn enc_file(&self) -> Option<String> {
        match self.tab {
            Tab::Files => self.current_path(),
            Tab::Commits => {
                let repo = self.git.as_ref()?;
                let commit = repo.commits.get(self.commit_sel)?;
                let files = self.commit_diffs.get(&commit.id)?;
                let rows = self.commits_pane.rows();
                let at = self.commits_pane.cursor.min(rows.len().checked_sub(1)?);
                let index = rows[..=at]
                    .iter()
                    .rev()
                    .find_map(|r| r.annot.filter(|t| t.kind == diffrows::K_FILE))
                    .map_or(0, |t| t.index as usize);
                files.get(index).map(|f| f.path.display().to_string())
            }
            Tab::Overview => None,
        }
    }

    /// The two blobs a file of the open page is made of: the request's merge
    /// base and head on the Files page, a commit and its first parent on the
    /// Commits page (`commit` is its index in the list).
    pub(super) fn sides_of(&self, commit: Option<usize>) -> Option<Sides> {
        // A re-read in another encoding needs the bytes of both sides from a
        // clone.
        if self.source.is_api() {
            return None;
        }
        let repo = self.git.as_ref()?;
        match commit {
            None => Some(Sides {
                old: Some(repo.diff_base),
                new: repo.head,
            }),
            Some(i) => {
                let c = repo.commits.get(i)?;
                Some(Sides {
                    old: c.parents.first().copied(),
                    new: c.id,
                })
            }
        }
    }

    /// `fd` ready to render under the current toggles and the encoding chosen
    /// for `path`. `(at, path)` name where the whole-file layout reads the new
    /// side from; `sides` where a re-read in another encoding reads both.
    fn prepare<'a>(
        &self,
        fd: &'a FileDiff,
        (at, path): (Oid, &str),
        sides: Option<Sides>,
    ) -> Prepared<'a> {
        let mut p = Prepared {
            original: fd,
            recoded: None,
            wants_whole: !self.diff_opts.hunks_only && needs_content(fd),
            whole: None,
            notice: None,
        };
        let charset = crate::diff_recode::remembered(std::path::Path::new(path));
        if let (Some(charset), Some(sides)) = (charset, sides) {
            let key = Key::new(path, fd.status, sides, charset);
            match self.recodes.get(&key) {
                Some(Done::Ready(r)) => p.recoded = Some(Arc::clone(r)),
                Some(Done::Failed(why)) => {
                    p.notice = Some(gitpoulpi_i18n::msg::diff_view::enc_failed(
                        charset.name(),
                        why,
                    ));
                }
                None if self.recodes.slow(&key) => {
                    p.notice = Some(gitpoulpi_i18n::msg::diff_view::enc_reading(charset.name()));
                }
                None => {}
            }
        }
        if let Some(r) = &p.recoded {
            // The re-read brings the new side with it.
            p.wants_whole = !self.diff_opts.hunks_only && needs_content(&r.diff);
            p.whole = Some(Whole::Ready(Arc::clone(&r.new_lines)));
        } else if p.wants_whole {
            p.whole = self.blobs.get(at, path).cloned();
        }
        p
    }

    /// A file the host sent no patch for says so, instead of looking empty.
    fn note_omitted(&self, p: &mut Prepared, commit: Option<Oid>, path: &str) {
        if self.source.omitted(commit, path) {
            p.notice = Some(msg::NOT_SHOWN_BY_HOST.get().to_string());
        }
    }

    /// A prepared file as the Diff view renders it, `annotator`'s rows in it.
    fn render_prepared(
        &self,
        p: &Prepared,
        annotator: Option<&dyn Annotator>,
        width: u16,
        h_offset: usize,
    ) -> DiffRender {
        let mut opts = DiffRenderOpts::for_state(&self.diff_opts, width, p.lines());
        opts.h_offset = h_offset;
        let render = render_diff_annotated(p.diff(), &opts, annotator);
        match (&p.notice, p.wants_whole, &p.whole) {
            (Some(why), _, _) => render.with_notice_on_top(why.clone()),
            (None, true, None) => {
                render.with_notice_on_top(gitpoulpi_i18n::msg::diff_content::LOADING_WHOLE.get())
            }
            (None, true, Some(Whole::Unavailable(why))) => {
                render.with_notice_on_top(gitpoulpi_i18n::msg::diff_content::whole_unavailable(why))
            }
            _ => render,
        }
    }

    /// How far the text may scroll sideways, and where the offset stands once
    /// clamped to it.
    fn h_extent(&self, longest: usize, width: u16) -> (usize, usize) {
        if self.diff_opts.word_wrap {
            return (0, 0);
        }
        let max = h_scroll_max(longest, self.diff_opts.mode, width);
        (max, self.diff_opts.h_offset.min(max))
    }

    /// The Files page's render of the open file, threads and all, and the
    /// widest line it holds.
    pub(super) fn build_files_render(
        &self,
        fd: &FileDiff,
        path: &str,
        width: u16,
    ) -> (DiffRender, usize) {
        let threads = self.threads_of(path);
        let pending = self.pending_of(path);
        let ghosts = self.ghosts_of(path);
        let annotator =
            diffrows::ReviewAnnotator::new(&threads, &pending, &ghosts, &self.collapsed);
        let at = self.git.as_ref().map_or(Oid([0u8; 20]), |g| g.head);
        let mut p = self.prepare(fd, (at, path), self.sides_of(None));
        self.note_omitted(&mut p, None, path);
        let longest = p.longest();
        let (_, h) = self.h_extent(longest, width);
        (
            self.render_prepared(&p, Some(&annotator), width, h),
            longest,
        )
    }

    /// The Commits page's render of a commit: its files one under the other.
    pub(super) fn build_commit_render(
        &self,
        commit: Oid,
        files: &[FileDiff],
        width: u16,
    ) -> (DiffRender, usize) {
        let index = self
            .git
            .as_ref()
            .and_then(|g| g.commits.iter().position(|c| c.id == commit));
        let sides = self.sides_of(index);
        let prepared: Vec<Prepared> = files
            .iter()
            .map(|fd| {
                let path = fd.path.display().to_string();
                let mut p = self.prepare(fd, (commit, &path), sides);
                self.note_omitted(&mut p, Some(commit), &path);
                p
            })
            .collect();
        let longest = prepared.iter().map(Prepared::longest).max().unwrap_or(0);
        let (_, h) = self.h_extent(longest, width);
        let parts = prepared
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let path = p.original.path.display().to_string();
                (
                    file_header(i, &path),
                    self.render_prepared(p, None, width, h),
                )
            })
            .collect();
        (
            DiffRender::stack(parts, self.diff_opts.mode, width),
            longest,
        )
    }

    pub(super) fn set_pane_render(
        &mut self,
        id: PaneId,
        key: super::pane::BuildKey,
        r: Option<(DiffRender, usize)>,
    ) {
        let (render, longest) = match r {
            Some((render, longest)) => (Some(Rc::new(render)), longest),
            None => (None, 0),
        };
        let (max, _) = self.h_extent(longest, key.1);
        let pane = match id {
            PaneId::Files => &mut self.files_pane,
            PaneId::Commits => &mut self.commits_pane,
        };
        pane.longest = longest;
        pane.h_max = max;
        pane.set_render(key, render);
    }

    // -- the diff pane ----------------------------------------------------------------

    pub(super) fn diff_pane(
        &mut self,
        buf: &mut Buffer,
        outer: Rect,
        id: PaneId,
        title: &str,
        waiting: Option<(String, bool)>,
        hits: &mut Vec<(Rect, Hit)>,
    ) {
        let page_tab = match id {
            PaneId::Files => Tab::Files,
            PaneId::Commits => Tab::Commits,
        };
        let focused = self.tab == page_tab && self.focus == Focus::Diff;
        let inner = block(buf, outer, title, focused);
        let spinner = self.spinner();
        let elapsed = self.started.elapsed();
        let hover = self.hover;
        let editing = id == PaneId::Files;
        let resolvable_caps = self
            .host
            .as_ref()
            .is_some_and(|h| h.detail.capabilities.resolve);
        let threads: Vec<(String, bool, bool, bool)> = self
            .host
            .as_ref()
            .map(|h| {
                h.threads
                    .iter()
                    .map(|t| (t.id.clone(), t.resolved, t.outdated, t.resolvable))
                    .collect()
            })
            .unwrap_or_default();
        let collapsed = self.collapsed.clone();
        let ghost_sev: Vec<Sev> = self.sug.items.iter().map(|i| i.severity).collect();
        let mode = self.diff_opts.mode;
        let hunks_only = self.diff_opts.hunks_only;
        let word_wrap = self.diff_opts.word_wrap;
        let h_offset = self.diff_opts.h_offset;
        let bar_hover = match hover {
            Some(Hit::Bar(b)) => Some(b),
            _ => None,
        };
        // The encoding button names the choice made for the file it applies to.
        let mut bar_state = self.diff_opts.clone();
        bar_state.encoding = self
            .enc_file()
            .and_then(|p| crate::diff_recode::remembered(std::path::Path::new(&p)));
        let bar = diff_bar_of(
            BAR_BUTTONS,
            &bar_state,
            bar_hover,
            hover == Some(Hit::BarMore),
            inner.width,
        );
        // The minimap takes its columns off the text's right, as in the Diff view.
        let pad = crate::minimap::padding_for(outer, crate::live_settings::minimap());
        let pane = match id {
            PaneId::Files => &mut self.files_pane,
            PaneId::Commits => &mut self.commits_pane,
        };
        pane.area = inner;
        if let Some((text, error)) = waiting {
            if pane.is_empty() && (error || elapsed >= SPIN_AFTER) {
                let style = if error {
                    Style::default().fg(th().status.danger).bg(th().ui.panel)
                } else {
                    Style::default().fg(th().ui.accent).bg(th().ui.panel)
                };
                let shown = if error {
                    text
                } else {
                    format!("{spinner} {text}")
                };
                dialog_kit::put(
                    buf,
                    inner.x + 1,
                    inner.y,
                    &shown,
                    inner.width.saturating_sub(2),
                    style,
                );
            }
            pane.bar = None;
            return;
        }

        // The Diff view's toggles on the first row.
        let body = if inner.height >= 4 {
            let rect = Rect::new(inner.x, inner.y, inner.width, 1);
            dialog_kit::fill(buf, rect, Style::default().bg(th().ui.panel));
            Paragraph::new(bar.line).render(rect, buf);
            for (dx, w, button) in &bar.buttons {
                let rect = Rect::new(inner.x + dx, inner.y, *w, 1);
                if *button == DiffBarButton::Encoding {
                    self.bar_slots.encoding = Some(rect);
                }
                hits.push((rect, Hit::Bar(*button)));
            }
            // The buttons that did not fit, behind a `⋯`.
            if let Some((dx, w)) = bar.overflow {
                let rect = Rect::new(inner.x + dx, inner.y, w, 1);
                hits.push((rect, Hit::BarMore));
                self.bar_slots.more = Some((rect, bar.hidden.clone()));
            }
            Rect::new(
                inner.x,
                inner.y + 1,
                inner.width.saturating_sub(pad),
                inner.height - 1,
            )
        } else {
            Rect::new(
                inner.x,
                inner.y,
                inner.width.saturating_sub(pad),
                inner.height,
            )
        };
        pane.area = body;
        let Some(render) = pane.render.clone() else {
            pane.bar = None;
            return;
        };
        let len = render.lines.len();
        pane.offset = pane.offset.min(len.saturating_sub(body.height as usize));
        let first = pane.offset;
        let last = (first + body.height as usize).min(len);
        let (lo, hi) = pane.selection();
        let show_cursor = focused || len <= 1;
        let mut lines: Vec<Line<'static>> = render.lines[first..last].to_vec();
        if let Some(Hit::Row(over, row)) = hover {
            if over == id && (first..last).contains(&row) {
                paint_band(&mut lines[row - first], body.width, th().ui.hover);
            }
        }
        apply_cursor(
            &mut lines,
            first,
            body.width,
            if show_cursor { pane.cursor } else { usize::MAX },
            pane.anchor.map(|_| (lo, hi)),
        );
        Paragraph::new(lines).render(body, buf);
        // The mouse text selection, over the code (a buffer pass, like the Diff view's).
        let sel_state = pane.view_state(&self.diff_opts);
        crate::diff_view::paint_text_selection(buf, body, &render, &sel_state);
        // The code is selectable text: the I-beam (a button registered later answers first).
        crate::text_select::register_pointer(
            body,
            &render.segs[first..last.min(render.segs.len())],
        );

        for n in 0..(last - first) {
            let i = first + n;
            let y = body.y + n as u16;
            let drow = &render.rows[i];
            let full = Rect::new(body.x, y, body.width, 1);
            let in_selection = pane.anchor.is_some() && i >= lo && i <= hi;
            let is_cursor = i == pane.cursor && show_cursor;
            let hovered_row = hover == Some(Hit::Row(id, i));
            let band = if in_selection {
                th().diff.range_bg
            } else if is_cursor {
                th().ui.selection
            } else if hovered_row {
                th().ui.hover
            } else {
                th().ui.panel
            };
            // Annotation rows sit on the side colour unless the cursor is on them.
            let abg = if is_cursor { band } else { th().ui.side_bg };
            let kind = kind_of(drow);
            if let Some(tag) = drow.annot {
                let (start, span) = annot_region(mode, body.width, tag.side);
                let x0 = body.x + start as u16;
                let right = x0 + span as u16;
                let text = render
                    .annot_text
                    .get(&i)
                    .map(String::as_str)
                    .unwrap_or_default();
                let wide = matches!(tag.side, AnnotSide::Full | AnnotSide::Span)
                    || mode == DiffViewMode::Inline;
                if kind != RowKind::Heading {
                    let rect = if wide {
                        full
                    } else {
                        Rect::new(x0, y, span as u16, 1)
                    };
                    dialog_kit::fill(buf, rect, Style::default().bg(abg));
                }
                match kind {
                    RowKind::FileHeader => {
                        dialog_kit::fill(buf, full, Style::default().bg(th().ui.side_bg));
                        dialog_kit::put(
                            buf,
                            body.x,
                            y,
                            text,
                            body.width,
                            Style::default()
                                .fg(th().ui.accent)
                                .bg(th().ui.side_bg)
                                .add_modifier(Modifier::BOLD),
                        );
                    }
                    RowKind::Heading => {
                        dialog_kit::put(
                            buf,
                            body.x,
                            y,
                            text,
                            body.width,
                            Style::default()
                                .fg(th().status.warning)
                                .bg(th().ui.panel)
                                .add_modifier(Modifier::BOLD),
                        );
                    }
                    RowKind::ThreadHead { thread } => {
                        let (thread_id, resolved, outdated, resolvable) =
                            threads.get(thread).cloned().unwrap_or_default();
                        let folded = collapsed.contains(&thread_id);
                        // Buttons, right to left.
                        let mut right = right;
                        if editing {
                            if resolvable && resolvable_caps {
                                let label = if resolved {
                                    msg::REOPEN.get()
                                } else {
                                    msg::RESOLVE.get()
                                };
                                let rect = dialog_kit::button_row::right_clipped(
                                    buf,
                                    right,
                                    y,
                                    right.saturating_sub(x0),
                                    8,
                                    label,
                                    ButtonState::Normal {
                                        hovered: hover == Some(Hit::ThreadResolve(i)),
                                        focused: false,
                                    },
                                );
                                hits.push((rect, Hit::ThreadResolve(i)));
                                right = rect.x.saturating_sub(1);
                            }
                            let label = msg::REPLY.get();
                            let rect = dialog_kit::button_row::right_clipped(
                                buf,
                                right,
                                y,
                                right.saturating_sub(x0),
                                8,
                                label,
                                ButtonState::Normal {
                                    hovered: hover == Some(Hit::ThreadReply(i)),
                                    focused: false,
                                },
                            );
                            hits.push((rect, Hit::ThreadReply(i)));
                            right = rect.x.saturating_sub(1);
                        }
                        let glyph = crate::fold_mark::glyph(folded);
                        let fold = dialog_kit::put(
                            buf,
                            x0,
                            y,
                            glyph,
                            1,
                            Style::default().fg(th().ui.accent).bg(abg),
                        );
                        let x = x0 + 2;
                        let mut chips = String::new();
                        if resolved {
                            chips.push_str(&format!(" {} ", msg::CHIP_RESOLVED.get()));
                        }
                        if outdated {
                            chips.push_str(&format!(" {} ", msg::CHIP_OUTDATED.get()));
                        }
                        let chip_w = display_width(&chips) as u16;
                        let label_max = right.saturating_sub(x + chip_w + 1);
                        let used = dialog_kit::put(
                            buf,
                            x,
                            y,
                            text,
                            label_max,
                            Style::default()
                                .fg(th().ui.text)
                                .bg(abg)
                                .add_modifier(Modifier::BOLD),
                        );
                        if chip_w > 0 {
                            dialog_kit::put(
                                buf,
                                used.right() + 1,
                                y,
                                &chips,
                                chip_w,
                                Style::default().fg(th().chrome.on_bright).bg(if resolved {
                                    th().status.success
                                } else {
                                    th().status.warning
                                }),
                            );
                        }
                        hits.push((
                            Rect::new(fold.x, y, (right.saturating_sub(fold.x)).max(1), 1),
                            Hit::Fold(i),
                        ));
                    }
                    RowKind::ThreadBody { meta, .. } => {
                        dialog_kit::put(
                            buf,
                            x0,
                            y,
                            "\u{2502}",
                            1,
                            Style::default().fg(th().ui.border).bg(abg),
                        );
                        let style = if meta {
                            Style::default()
                                .fg(th().ui.muted)
                                .bg(abg)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(th().ui.text).bg(abg)
                        };
                        dialog_kit::put(buf, x0 + 2, y, text, right.saturating_sub(x0 + 2), style);
                    }
                    RowKind::PendingHead { .. } => {
                        let mut right = right;
                        if editing {
                            for (label, hit) in [
                                (msg::DELETE.get(), Hit::PendingDelete(i)),
                                (msg::EDIT.get(), Hit::PendingEdit(i)),
                            ] {
                                let rect = dialog_kit::button_row::right_clipped(
                                    buf,
                                    right,
                                    y,
                                    right.saturating_sub(x0),
                                    8,
                                    label,
                                    ButtonState::Normal {
                                        hovered: hover == Some(hit),
                                        focused: false,
                                    },
                                );
                                hits.push((rect, hit));
                                right = rect.x.saturating_sub(1);
                            }
                        }
                        dialog_kit::put(
                            buf,
                            x0,
                            y,
                            &format!("\u{25CF} {text}"),
                            right.saturating_sub(x0),
                            Style::default()
                                .fg(th().status.warning)
                                .bg(abg)
                                .add_modifier(Modifier::BOLD),
                        );
                    }
                    RowKind::PendingBody { .. } => {
                        dialog_kit::put(
                            buf,
                            x0,
                            y,
                            "\u{2502}",
                            1,
                            Style::default().fg(th().status.warning).bg(abg),
                        );
                        dialog_kit::put(
                            buf,
                            x0 + 2,
                            y,
                            text,
                            right.saturating_sub(x0 + 2),
                            Style::default().fg(th().ui.text).bg(abg),
                        );
                    }
                    RowKind::GhostHead { item } => {
                        let sev = ghost_sev.get(item).copied().unwrap_or(Sev::Info);
                        let mut right = right;
                        if editing {
                            for (label, hit) in [
                                (ai_msg::DISMISS.get(), Hit::GhostDismiss(i)),
                                (ai_msg::EDIT_ACCEPT.get(), Hit::GhostEdit(i)),
                                (ai_msg::ACCEPT.get(), Hit::GhostAccept(i)),
                            ] {
                                let rect = dialog_kit::button_row::right_clipped(
                                    buf,
                                    right,
                                    y,
                                    right.saturating_sub(x0),
                                    8,
                                    label,
                                    ButtonState::Normal {
                                        hovered: hover == Some(hit),
                                        focused: false,
                                    },
                                );
                                hits.push((rect, hit));
                                right = rect.x.saturating_sub(1);
                            }
                        }
                        dialog_kit::put(
                            buf,
                            x0,
                            y,
                            &format!("{} {text}", crate::theme::AI_SPARK),
                            right.saturating_sub(x0),
                            Style::default()
                                .fg(sev_color(sev))
                                .bg(abg)
                                .add_modifier(Modifier::BOLD),
                        );
                    }
                    RowKind::GhostBody { .. } => {
                        dialog_kit::put(
                            buf,
                            x0,
                            y,
                            "\u{250A}",
                            1,
                            Style::default().fg(th().status.ai).bg(abg),
                        );
                        dialog_kit::put(
                            buf,
                            x0 + 2,
                            y,
                            text,
                            right.saturating_sub(x0 + 2),
                            Style::default()
                                .fg(th().ui.text)
                                .bg(abg)
                                .add_modifier(Modifier::ITALIC),
                        );
                    }
                    _ => {}
                }
            } else if editing
                && (is_cursor || hovered_row)
                && body.width > 16
                && matches!(kind, RowKind::Line { .. })
            {
                let plus = dialog_kit::put(
                    buf,
                    body.x,
                    y,
                    " + ",
                    3,
                    Style::default()
                        .fg(th().ui.on_accent)
                        .bg(if hover == Some(Hit::Add(id, i)) {
                            crate::hover_fill(th().ui.accent, th().ui.on_accent, 30)
                        } else {
                            th().ui.accent
                        })
                        .add_modifier(Modifier::BOLD),
                );
                hits.push((plus, Hit::Add(id, i)));
            }
            // The whole row is a target, after the buttons it holds.
            hits.push((full, Hit::Row(id, i)));
        }

        // The scrollbar, with the overview ruler: the changes (whole file
        // only - in hunks every row is near one), the review's own rows, the cursor.
        pane.bar = crate::scrollbar::Track::vertical(outer, len, pane.offset);
        if let Some(track) = pane.bar {
            crate::scrollbar::render_buf(buf, track, true, false);
            let mut marks: Vec<(usize, Mark)> = if hunks_only {
                Vec::new()
            } else {
                change_marks(&render)
            };
            marks.extend(
                render
                    .rows
                    .iter()
                    .enumerate()
                    .filter(|(_, r)| is_marker(r))
                    .map(|(i, _)| (i, Mark::Thread)),
            );
            marks.push((pane.cursor, Mark::Cursor));
            let cells = scroll_marks::project(track.inner_len(), len, marks);
            scroll_marks::paint_buf(buf, track, &cells);
        }

        // The minimap beside the text, and the sideways scrollbar on the bottom border.
        pane.map = None;
        if pad > 0 {
            if let Some(rect) = crate::minimap::rect_in(outer) {
                let key = Rc::as_ptr(&render) as *const () as usize as u64;
                pane.minimap.pin(render.clone());
                pane.minimap.draw_buf(
                    buf,
                    rect,
                    key,
                    len,
                    (pane.offset, body.height as usize),
                    |i| crate::diff_view::minimap_row(&render, i),
                );
                pane.map = Some(rect);
            }
        }
        pane.h_bar = None;
        if !word_wrap {
            let content = h_track_content(
                pane.longest,
                mode,
                body.width,
                outer.width.saturating_sub(2),
            );
            pane.h_bar =
                crate::scrollbar::Track::horizontal(outer, content, h_offset.min(pane.h_max));
            if let Some(track) = pane.h_bar {
                crate::scrollbar::render_buf(buf, track, false, false);
            }
        }
    }
}
