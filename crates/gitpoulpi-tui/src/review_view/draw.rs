//! Painting the review view, and the hit rects of that same walk (maintainer
//! rule 6): the header, the three pages and the popups. `render` clears the
//! hit list, every part pushes the rects it draws (a button before the band
//! it sits in), and the list is what the mouse handler reads next.

use gitpoulpi_core::domain::short;
use gitpoulpi_hosting::{ChecksState, MergeState, PullRequestState, ReviewState};
use gitpoulpi_i18n::msg::hosting_ui as hosting;
use gitpoulpi_i18n::msg::review_ai as ai_msg;
use gitpoulpi_i18n::msg::review_view as msg;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders, Widget};

use super::diffrows;
use super::model::{Focus, Sev};
use super::{PaneId, ReviewView, SPIN_AFTER};
use crate::commit_detail::wrap_text;
use crate::dialog_kit::{self, ButtonState};
use crate::md_render;
use crate::meta_picker::MetaField;
use crate::review_cmd::Tab;
use crate::th;
use crate::ui_kit::{display_width, tail_fit};

/// A click target of the view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Hit {
    Close,
    Tab(Tab),
    Meta(MetaField),
    Checkout,
    Browser,
    Refresh,
    Finish,
    /// Gerrit: merge / abandon the change (both ask first).
    SubmitChange,
    AbandonChange,
    /// The reply box under the conversation.
    Compose,
    File(usize),
    Viewed(usize),
    Commit(usize),
    Row(PaneId, usize),
    /// A link of the Overview's Markdown (index into `md_links`).
    MdLink(usize),
    /// The `+` at the start of a code line.
    Add(PaneId, usize),
    Fold(usize),
    ThreadReply(usize),
    ThreadResolve(usize),
    PendingEdit(usize),
    PendingDelete(usize),
    /// "✦ Suggest review" (or Cancel while it runs).
    Suggest,
    /// The buttons of a ✦ suggestion under a line: the row they are on.
    GhostAccept(usize),
    GhostEdit(usize),
    GhostDismiss(usize),
    /// The buttons of a ✦ suggestion in the Overview list: its index.
    SugAccept(usize),
    SugEdit(usize),
    SugDismiss(usize),
    SugAcceptAll,
    SugDismissAll,
    /// The draft card's buttons.
    UseDraft,
    EditDraft,
    DiscardDraft,
    /// A toggle of the diff pane's bar (the Diff view's own buttons).
    Bar(crate::diff_view::DiffBarButton),
    /// The bar's `⋯`: the list of the buttons it had no room for.
    BarMore,
}

/// How a suggestion's severity is coloured.
pub(super) fn sev_color(sev: Sev) -> ratatui::style::Color {
    match sev {
        Sev::Info => th().status.ai,
        Sev::Warning => th().status.warning,
        Sev::Issue => th().status.danger,
    }
}

fn panel_style() -> Style {
    Style::default().fg(th().ui.text).bg(th().ui.panel)
}

fn muted() -> Style {
    Style::default().fg(th().ui.muted).bg(th().ui.panel)
}

pub(super) fn block(buf: &mut Buffer, rect: Rect, title: &str, focused: bool) -> Rect {
    let block = Block::default()
        .title(format!(" {title} "))
        .border_type(BorderType::Rounded)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if focused {
            th().ui.accent
        } else {
            th().ui.border
        }))
        .style(panel_style());
    let inner = block.inner(rect);
    block.render(rect, buf);
    inner
}

fn state_look(state: PullRequestState) -> (&'static str, ratatui::style::Color) {
    match state {
        PullRequestState::Open => (hosting::STATE_OPEN.get(), th().refs.local),
        PullRequestState::Draft => (hosting::STATE_DRAFT.get(), th().ui.muted),
        PullRequestState::Merged => (hosting::STATE_MERGED.get(), th().hosting.merged),
        PullRequestState::Closed => (hosting::STATE_CLOSED.get(), th().status.danger),
    }
}

impl ReviewView {
    fn hovered(&self, hit: Hit) -> bool {
        self.hover == Some(hit)
    }

    /// Paints the whole view into `area`.
    pub(crate) fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let mut hits: Vec<(Rect, Hit)> = Vec::new();
        self.bar_slots = Default::default();
        let r = dialog_kit::centered(
            area,
            area.width.saturating_sub(2),
            area.height.saturating_sub(2),
        );
        dialog_kit::panel(buf, r, &msg::title(&self.target.iid_label));
        if r.width < 56 || r.height < 14 {
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
        self.top_border(buf, r, &mut hits);
        let inner = Rect::new(r.x + 2, r.y + 1, r.width - 4, r.height - 2);
        let mut y = inner.y;
        if self.host.is_some() {
            y = self.header(buf, inner, &mut hits);
        }
        let status_y = inner.bottom().saturating_sub(2);
        let hints_y = inner.bottom().saturating_sub(1);
        let body = Rect::new(inner.x, y, inner.width, status_y.saturating_sub(y));
        if let Some(why) = &self.host_err {
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
        } else if self.host.is_none() {
            if self.started.elapsed() >= SPIN_AFTER {
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
            match self.tab {
                Tab::Overview => self.overview(buf, body, &mut hits),
                Tab::Files => self.files_page(buf, body, &mut hits),
                Tab::Commits => self.commits_page(buf, body, &mut hits),
            }
        }
        self.status_line(buf, inner.x, status_y, inner.width);
        self.footer(buf, inner.x, hints_y, inner.width);
        crate::pointer_map::from_hits(&hits, |h| match h {
            Hit::Compose => Some(crate::pointer_map::Kind::Field),
            // A code line is text to read and comment on.
            Hit::Row(..) => None,
            // A row of the Files list lights on hover and keeps the arrow.
            Hit::File(_) | Hit::Commit(_) => Some(crate::pointer_map::Kind::Plain),
            _ => Some(crate::pointer_map::Kind::Hand),
        });
        self.hits = hits;
        // Popups last, over everything.
        let spinner = self.spinner();
        if let Some(c) = self.compose.as_mut() {
            c.render(area, buf);
        }
        if let Some(f) = self.finish.as_mut() {
            f.render(area, buf, spinner);
        }
        if let Some(o) = self.options.as_mut() {
            o.render(area, buf);
        }
        if let Some((_, c)) = self.confirm.as_mut() {
            c.render(area, buf);
        }
        if let Some(m) = self.meta.as_mut() {
            m.render(area, buf);
        }
        if let Some(m) = self.bar_menu.as_mut() {
            m.render(buf, area);
        }
    }

    /// The close mark and the three tabs, in the top border.
    fn top_border(&mut self, buf: &mut Buffer, r: Rect, hits: &mut Vec<(Rect, Hit)>) {
        let y = r.y;
        let close = " \u{2715} ";
        let cx = r.right().saturating_sub(5);
        let style = if self.hovered(Hit::Close) {
            Style::default().fg(th().ui.on_accent).bg(th().ui.accent)
        } else {
            Style::default().fg(th().ui.muted).bg(th().ui.panel)
        };
        let rect = dialog_kit::put(buf, cx, y, close, 3, style);
        hits.push((rect, Hit::Close));
        // Tabs, right-aligned before the close mark.
        let labels = [
            (Tab::Overview, msg::TAB_OVERVIEW.get()),
            (Tab::Files, msg::TAB_FILES.get()),
            (Tab::Commits, msg::TAB_COMMITS.get()),
        ];
        let total: usize = labels
            .iter()
            .map(|(_, l)| display_width(l) + 2)
            .sum::<usize>();
        let mut x = cx.saturating_sub(total as u16 + 1);
        for (tab, label) in labels {
            let text = format!(" {label} ");
            let w = display_width(&text) as u16;
            let style = if tab == self.tab {
                Style::default()
                    .bg(th().ui.accent)
                    .fg(th().ui.on_accent)
                    .add_modifier(Modifier::BOLD)
            } else if self.hovered(Hit::Tab(tab)) {
                Style::default()
                    .bg(th().ui.button_hover)
                    .fg(th().ui.on_button)
            } else {
                Style::default().bg(th().ui.button).fg(th().ui.on_button)
            };
            let rect = dialog_kit::put(buf, x, y, &text, w, style);
            hits.push((rect, Hit::Tab(tab)));
            x += w;
        }
    }

    /// Title, facts, metadata chips and actions. Returns the row the body
    /// starts on.
    fn header(&mut self, buf: &mut Buffer, inner: Rect, hits: &mut Vec<(Rect, Hit)>) -> u16 {
        let Some(h) = self.host.as_ref() else {
            return inner.y;
        };
        let mut detail = h.detail.clone();
        // GitLab's request carries no line counts: the files list has them.
        if detail.additions.is_none() && !h.files.is_empty() {
            detail.additions = Some(h.files.iter().map(|f| f.additions).sum());
            detail.deletions = Some(h.files.iter().map(|f| f.deletions).sum());
            detail.changed_files = Some(h.files.len() as u32);
        }
        let meta = h.meta.clone();
        let s = &detail.summary;
        let mut y = inner.y;

        // Row 1: the title and the state.
        let (state_text, state_color) = state_look(s.state);
        let chip = format!(" {state_text} ");
        let chip_w = display_width(&chip) as u16;
        dialog_kit::put(
            buf,
            inner.x,
            y,
            &format!("{} {}", s.iid_label, s.title),
            inner.width.saturating_sub(chip_w + 1),
            Style::default()
                .fg(th().ui.text)
                .bg(th().ui.panel)
                .add_modifier(Modifier::BOLD),
        );
        dialog_kit::put(
            buf,
            inner.right().saturating_sub(chip_w),
            y,
            &chip,
            chip_w,
            Style::default()
                .fg(th().chrome.on_bright)
                .bg(state_color)
                .add_modifier(Modifier::BOLD),
        );
        y += 1;

        // Row 2: branches, author, checks, mergeability, size, pending.
        let mut x = inner.x;
        let branches = format!("{} \u{2190} {}", s.target_branch, s.source_branch);
        let w = display_width(&branches) as u16;
        dialog_kit::put(
            buf,
            x,
            y,
            &branches,
            w,
            Style::default().fg(th().refs.local).bg(th().ui.panel),
        );
        x += w + 2;
        let mut facts: Vec<(String, ratatui::style::Color)> =
            vec![(msg::by_author(&s.author), th().ui.muted)];
        match detail.checks {
            Some(ChecksState::Passing) => facts.push((
                format!("\u{2713} {}", msg::CHECKS_PASSING.get()),
                th().status.success,
            )),
            Some(ChecksState::Failing) => facts.push((
                format!("\u{2717} {}", msg::CHECKS_FAILING.get()),
                th().status.danger,
            )),
            Some(ChecksState::Pending) => facts.push((
                format!("\u{25CF} {}", msg::CHECKS_PENDING.get()),
                th().status.warning,
            )),
            None => {}
        }
        match detail.merge_state {
            MergeState::Conflicting => facts.push((
                format!("! {}", msg::MERGE_CONFLICTS.get()),
                th().status.danger,
            )),
            MergeState::Clean => facts.push((msg::MERGE_CLEAN.get().to_string(), th().ui.muted)),
            MergeState::Blocked => {
                facts.push((msg::MERGE_BLOCKED.get().to_string(), th().status.warning))
            }
            MergeState::Unknown => {}
        }
        if let (Some(a), Some(d), Some(f)) =
            (detail.additions, detail.deletions, detail.changed_files)
        {
            facts.push((msg::size(u64::from(f), a, d), th().ui.muted));
        }
        let pending = self.pending.len();
        let pending_text = (pending > 0).then(|| msg::pending_counter(pending as u64));
        let right_w = pending_text
            .as_ref()
            .map_or(0, |t| display_width(t) as u16 + 1);
        for (text, color) in facts {
            let w = display_width(&text) as u16;
            if x + w + right_w > inner.right() {
                break;
            }
            dialog_kit::put(
                buf,
                x,
                y,
                &text,
                w,
                Style::default().fg(color).bg(th().ui.panel),
            );
            x += w + 2;
        }
        if let Some(text) = pending_text {
            let w = display_width(&text) as u16;
            dialog_kit::put(
                buf,
                inner.right().saturating_sub(w),
                y,
                &text,
                w,
                Style::default()
                    .fg(th().status.warning)
                    .bg(th().ui.panel)
                    .add_modifier(Modifier::BOLD),
            );
        }
        y += 1;

        // Row 3: assignees, reviewers, labels, milestone (each a picker).
        let list = |items: &[String]| -> String {
            if items.is_empty() {
                msg::NONE.get().to_string()
            } else {
                items.join(", ")
            }
        };
        let (assignees, reviewers, labels, milestone) = match &meta {
            Some(m) => (
                list(&m.assignees),
                list(&m.reviewers),
                list(&m.labels),
                m.milestone
                    .as_ref()
                    .map_or_else(|| msg::NONE.get().to_string(), |m| m.title.clone()),
            ),
            None => (
                list(&s.assignees),
                list(&s.reviewers),
                msg::NONE.get().to_string(),
                msg::NONE.get().to_string(),
            ),
        };
        let editable = self.can_edit_meta();
        let mut x = inner.x;
        let chips = [
            (MetaField::Assignees, msg::CHIP_ASSIGNEES.get(), assignees),
            (MetaField::Reviewers, msg::CHIP_REVIEWERS.get(), reviewers),
            (MetaField::Labels, msg::CHIP_LABELS.get(), labels),
            (MetaField::Milestone, msg::CHIP_MILESTONE.get(), milestone),
        ];
        let each = (inner.width / 4).saturating_sub(1).max(12);
        for (field, name, value) in chips {
            let text = format!("{name}: {value}");
            let hovered = self.hovered(Hit::Meta(field));
            let style = if !editable {
                muted()
            } else if hovered {
                Style::default()
                    .fg(th().ui.on_button)
                    .bg(th().ui.button_hover)
            } else {
                Style::default().fg(th().ui.on_button).bg(th().ui.button)
            };
            let w = (display_width(&text) as u16 + 2).min(each);
            let rect = dialog_kit::put(buf, x, y, &format!(" {text} "), w, style);
            if editable && rect.width > 0 {
                hits.push((rect, Hit::Meta(field)));
            }
            x += w + 1;
        }
        y += 1;

        // Row 4: actions.
        let mut x = inner.x;
        let finish_label = if pending > 0 {
            msg::finish_with(pending as u64)
        } else {
            msg::FINISH.get().to_string()
        };
        let mut buttons: Vec<(Hit, String, bool)> = vec![
            (Hit::Checkout, msg::CHECK_OUT.get().to_string(), false),
            (Hit::Browser, msg::OPEN_BROWSER.get().to_string(), false),
            (Hit::Refresh, msg::REFRESH.get().to_string(), false),
        ];
        // ✦ only with AI on: the explicit action that asks the model. While
        // it runs the same button cancels.
        if self.ai_on() {
            let label = if self.sug.running() {
                msg::CANCEL.get()
            } else {
                ai_msg::BUTTON.get()
            };
            buttons.push((
                Hit::Suggest,
                format!("{} {label}", crate::theme::AI_SPARK),
                false,
            ));
        }
        // Gerrit: only what the server says this user can do right now.
        if detail.can_submit {
            buttons.push((
                Hit::SubmitChange,
                msg::SUBMIT_BUTTON.get().to_string(),
                false,
            ));
        }
        if detail.can_abandon {
            buttons.push((
                Hit::AbandonChange,
                msg::ABANDON_BUTTON.get().to_string(),
                false,
            ));
        }
        buttons.push((Hit::Finish, finish_label, true));
        for (hit, label, primary) in buttons {
            let hovered = self.hovered(hit);
            let state = if primary {
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
            let rect = dialog_kit::button(buf, x, y, &label, state);
            hits.push((rect, hit));
            x = rect.right() + 1;
        }
        y += 1;

        // A rule under the header.
        for cx in inner.x..inner.right() {
            dialog_kit::put(
                buf,
                cx,
                y,
                "\u{2500}",
                1,
                Style::default().fg(th().ui.border).bg(th().ui.panel),
            );
        }
        y + 1
    }

    // -- Overview -------------------------------------------------------------------

    fn overview(&mut self, buf: &mut Buffer, body: Rect, hits: &mut Vec<(Rect, Hit)>) {
        let Some(h) = self.host.as_ref() else { return };
        if body.height < 6 {
            return;
        }
        let box_h = 3u16;
        let scroll = Rect::new(body.x, body.y, body.width, body.height - box_h);
        let inner = block(
            buf,
            scroll,
            msg::OVERVIEW_TITLE.get(),
            self.focus == Focus::List,
        );
        let width = inner.width.saturating_sub(4) as usize;
        self.md_links.clear();

        enum L {
            Heading(String),
            Meta(String, Option<(String, ratatui::style::Color)>),
            /// A line of Markdown text, already wrapped and styled.
            Md(std::rc::Rc<md_render::Doc>, usize),
            Dim(String),
            Blank,
            /// The heading of the ✦ block, with the number of suggestions.
            SugHeading(usize),
            /// The head of suggestion `n` of the list.
            Sug(usize),
            /// The heading of the draft card (with the stale mark).
            DraftHeading(bool),
            /// The verdict chip and the draft's three buttons.
            DraftBar,
            SugText(usize, String),
        }
        let ctx = self.target.md_context();
        let mut all_docs: Vec<std::rc::Rc<md_render::Doc>> = Vec::new();
        fn md_lines(
            cache: &mut md_render::DocCache,
            all: &mut Vec<std::rc::Rc<md_render::Doc>>,
            ctx: &md_render::Context,
            id: &str,
            text: &str,
            width: usize,
        ) -> Vec<L> {
            let doc = cache.get_text(id, text, width, ctx);
            all.push(std::rc::Rc::clone(&doc));
            (0..doc.lines.len())
                .map(|n| L::Md(std::rc::Rc::clone(&doc), n))
                .collect()
        }
        let mut lines: Vec<L> = Vec::new();
        lines.push(L::Heading(msg::DESCRIPTION.get().to_string()));
        if h.detail.body.trim().is_empty() {
            lines.push(L::Dim(msg::NO_DESCRIPTION.get().to_string()));
        } else {
            lines.extend(md_lines(
                &mut self.md_cache,
                &mut all_docs,
                &ctx,
                "description",
                &h.detail.body,
                width,
            ));
        }
        lines.push(L::Blank);
        // The AI's summary and suggestions: after a run only.
        if self.sug.enabled() && (self.sug.done || !self.sug.items.is_empty()) {
            if self.has_draft() {
                lines.push(L::DraftHeading(self.draft_stale()));
                lines.push(L::DraftBar);
                if !self.sug.body.trim().is_empty() {
                    lines.push(L::Dim(ai_msg::DRAFT_TEXT_LABEL.get().to_string()));
                    lines.extend(md_lines(
                        &mut self.md_cache,
                        &mut all_docs,
                        &ctx,
                        "draft",
                        &self.sug.body,
                        width.saturating_sub(2),
                    ));
                }
                lines.push(L::Blank);
            }
            lines.push(L::SugHeading(self.sug.items.len()));
            if !self.sug.summary.trim().is_empty()
                && self.sug.summary.trim() != self.sug.body.trim()
            {
                lines.push(L::Dim(ai_msg::SUMMARY_LABEL.get().to_string()));
                lines.extend(md_lines(
                    &mut self.md_cache,
                    &mut all_docs,
                    &ctx,
                    "summary",
                    &self.sug.summary,
                    width.saturating_sub(2),
                ));
            }
            for note in &self.sug.notes {
                lines.push(L::Dim(note.clone()));
            }
            if self.sug.items.is_empty() {
                lines.push(L::Dim(ai_msg::NOTHING.get().to_string()));
            }
            for (n, item) in self.sug.items.iter().enumerate() {
                lines.push(L::Sug(n));
                lines.extend(
                    wrap_text(&item.body, width.saturating_sub(4))
                        .into_iter()
                        .map(|t| L::SugText(n, t)),
                );
            }
            lines.push(L::Blank);
        }
        // Comments and submitted reviews, oldest first; undated last.
        enum Item<'a> {
            C(&'a gitpoulpi_hosting::ReviewComment),
            R(&'a gitpoulpi_hosting::SubmittedReview),
        }
        let mut items: Vec<(String, Item)> = h
            .conversation
            .iter()
            .map(|c| (c.created_at.clone(), Item::C(c)))
            .chain(
                h.reviews
                    .iter()
                    .map(|r| (r.submitted_at.clone(), Item::R(r))),
            )
            .collect();
        items.sort_by(|a, b| {
            let key = |s: &String| {
                if s.is_empty() {
                    "~".to_string()
                } else {
                    s.clone()
                }
            };
            key(&a.0).cmp(&key(&b.0))
        });
        lines.push(L::Heading(msg::conversation_heading(items.len() as u64)));
        if items.is_empty() {
            lines.push(L::Dim(msg::NO_CONVERSATION.get().to_string()));
        }
        for (n, (when, item)) in items.into_iter().enumerate() {
            match item {
                Item::C(c) => {
                    lines.push(L::Meta(diffrows::who(&c.author, &when), None));
                    lines.extend(md_lines(
                        &mut self.md_cache,
                        &mut all_docs,
                        &ctx,
                        &format!("item{n}"), // i18n-english
                        &c.body,
                        width.saturating_sub(2),
                    ));
                }
                Item::R(r) => {
                    let (word, color) = match r.state {
                        ReviewState::Approved => (msg::VERDICT_APPROVED.get(), th().status.success),
                        ReviewState::ChangesRequested => {
                            (msg::VERDICT_CHANGES.get(), th().status.danger)
                        }
                        ReviewState::Commented => (msg::VERDICT_COMMENTED.get(), th().ui.muted),
                        ReviewState::Dismissed => (msg::VERDICT_DISMISSED.get(), th().ui.muted),
                    };
                    lines.push(L::Meta(
                        diffrows::who(&r.author, &when),
                        Some((word.to_string(), color)),
                    ));
                    if !r.body.trim().is_empty() {
                        lines.extend(md_lines(
                            &mut self.md_cache,
                            &mut all_docs,
                            &ctx,
                            &format!("item{n}"), // i18n-english
                            &r.body,
                            width.saturating_sub(2),
                        ));
                    }
                }
            }
            lines.push(L::Blank);
        }
        // The palette's "Open Link in Message" twin reaches every link of the page.
        let mut urls: Vec<String> = Vec::new();
        for doc in &all_docs {
            for l in &doc.links {
                if !urls.contains(&l.url) {
                    urls.push(l.url.clone());
                }
            }
        }
        crate::link_confirm::set_message_urls(urls);
        self.ov_len = lines.len();
        self.ov_rows = inner.height as usize;
        self.ov_offset = self.ov_offset.min(self.ov_len.saturating_sub(self.ov_rows));
        for (i, line) in lines
            .iter()
            .skip(self.ov_offset)
            .take(inner.height as usize)
            .enumerate()
        {
            let y = inner.y + i as u16;
            let x = inner.x + 1;
            let max = inner.width.saturating_sub(2);
            match line {
                L::Heading(t) => {
                    dialog_kit::put(
                        buf,
                        x,
                        y,
                        t,
                        max,
                        Style::default()
                            .fg(th().ui.accent)
                            .bg(th().ui.panel)
                            .add_modifier(Modifier::BOLD),
                    );
                }
                L::Meta(t, verdict) => {
                    let used = dialog_kit::put(
                        buf,
                        x,
                        y,
                        t,
                        max,
                        Style::default()
                            .fg(th().ui.text)
                            .bg(th().ui.panel)
                            .add_modifier(Modifier::BOLD),
                    );
                    if let Some((word, color)) = verdict {
                        dialog_kit::put(
                            buf,
                            used.right() + 1,
                            y,
                            &format!(" {word} "),
                            max.saturating_sub(used.width + 1),
                            Style::default()
                                .fg(th().chrome.on_bright)
                                .bg(*color)
                                .add_modifier(Modifier::BOLD),
                        );
                    }
                }
                L::Md(doc, n) => {
                    let Some(line) = doc.lines.get(*n) else {
                        continue;
                    };
                    for (rect, id) in md_render::paint(
                        buf,
                        x + 2,
                        y,
                        max.saturating_sub(2),
                        line,
                        panel_style(),
                        false,
                    ) {
                        if let Some(target) = doc.links.get(id).filter(|t| !t.url.is_empty()) {
                            self.md_links.push(target.url.clone());
                            hits.push((rect, Hit::MdLink(self.md_links.len() - 1)));
                        }
                    }
                }
                L::SugHeading(n) => {
                    let ai = Style::default()
                        .fg(th().status.ai)
                        .bg(th().ui.panel)
                        .add_modifier(Modifier::BOLD);
                    let title = format!(
                        "{} {} - {}",
                        crate::theme::AI_SPARK,
                        ai_msg::AI_TITLE.get(),
                        ai_msg::suggestions_heading(*n as u64)
                    );
                    let used = dialog_kit::put(buf, x, y, &title, max, ai);
                    if *n > 0 {
                        // Right to left: Dismiss all, Accept all.
                        let mut right = x + max;
                        for (label, hit) in [
                            (ai_msg::DISMISS_ALL.get(), Hit::SugDismissAll),
                            (ai_msg::ACCEPT_ALL.get(), Hit::SugAcceptAll),
                        ] {
                            let w = dialog_kit::button_width(label);
                            if right.saturating_sub(used.right()) > w + 1 {
                                let rect = dialog_kit::button(
                                    buf,
                                    right - w,
                                    y,
                                    label,
                                    ButtonState::Normal {
                                        hovered: self.hover == Some(hit),
                                        focused: false,
                                    },
                                );
                                hits.push((rect, hit));
                                right = rect.x.saturating_sub(1);
                            }
                        }
                    }
                }
                L::DraftHeading(stale) => {
                    let ai = Style::default()
                        .fg(th().status.ai)
                        .bg(th().ui.panel)
                        .add_modifier(Modifier::BOLD);
                    let used = dialog_kit::put(
                        buf,
                        x,
                        y,
                        &format!("{} {}", crate::theme::AI_SPARK, ai_msg::DRAFT_TITLE.get()),
                        max,
                        ai,
                    );
                    if *stale {
                        dialog_kit::put(
                            buf,
                            used.right() + 2,
                            y,
                            ai_msg::DRAFT_STALE.get(),
                            max.saturating_sub(used.width + 2),
                            Style::default().fg(th().status.warning).bg(th().ui.panel),
                        );
                    }
                }
                L::DraftBar => {
                    let (word, color) = match self.sug.verdict {
                        gitpoulpi_hosting::ReviewEvent::Approve => {
                            (msg::EVENT_APPROVE.get(), th().status.success)
                        }
                        gitpoulpi_hosting::ReviewEvent::RequestChanges => {
                            (msg::EVENT_REQUEST_CHANGES.get(), th().status.danger)
                        }
                        gitpoulpi_hosting::ReviewEvent::Comment => {
                            (msg::EVENT_COMMENT.get(), th().ui.muted)
                        }
                    };
                    let chip = dialog_kit::put(
                        buf,
                        x,
                        y,
                        &format!(" {word} "),
                        max,
                        Style::default()
                            .fg(th().chrome.on_bright)
                            .bg(color)
                            .add_modifier(Modifier::BOLD),
                    );
                    let mut bx = chip.right() + 2;
                    for (label, hit, primary) in [
                        (
                            format!("{} {}", crate::theme::AI_SPARK, ai_msg::USE_DRAFT.get()),
                            Hit::UseDraft,
                            true,
                        ),
                        (ai_msg::EDIT_DRAFT.get().to_string(), Hit::EditDraft, false),
                        (
                            ai_msg::DISCARD_DRAFT.get().to_string(),
                            Hit::DiscardDraft,
                            false,
                        ),
                    ] {
                        let w = dialog_kit::button_width(&label);
                        if bx + w > x + max {
                            break;
                        }
                        let hovered = self.hover == Some(hit);
                        let state = if primary {
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
                        let rect = dialog_kit::button(buf, bx, y, &label, state);
                        hits.push((rect, hit));
                        bx = rect.right() + 1;
                    }
                }
                L::Sug(n) => {
                    let Some(item) = self.sug.items.get(*n) else {
                        continue;
                    };
                    let mut right = x + max;
                    for (label, hit) in [
                        (ai_msg::DISMISS.get(), Hit::SugDismiss(*n)),
                        (ai_msg::EDIT_ACCEPT.get(), Hit::SugEdit(*n)),
                        (ai_msg::ACCEPT.get(), Hit::SugAccept(*n)),
                    ] {
                        let w = dialog_kit::button_width(label);
                        if right.saturating_sub(x) > w + 20 {
                            let rect = dialog_kit::button(
                                buf,
                                right - w,
                                y,
                                label,
                                ButtonState::Normal {
                                    hovered: self.hover == Some(hit),
                                    focused: false,
                                },
                            );
                            hits.push((rect, hit));
                            right = rect.x.saturating_sub(1);
                        }
                    }
                    let place = if item.line > 0 {
                        format!("{}:{}", item.path, item.line)
                    } else {
                        item.path.clone()
                    };
                    dialog_kit::put(
                        buf,
                        x,
                        y,
                        &format!(
                            "{} {} - {place}",
                            crate::theme::AI_SPARK,
                            item.severity.word()
                        ),
                        right.saturating_sub(x),
                        Style::default()
                            .fg(sev_color(item.severity))
                            .bg(th().ui.panel)
                            .add_modifier(Modifier::BOLD),
                    );
                }
                L::SugText(_, t) => {
                    dialog_kit::put(
                        buf,
                        x + 1,
                        y,
                        "\u{250A}",
                        1,
                        Style::default().fg(th().status.ai).bg(th().ui.panel),
                    );
                    dialog_kit::put(buf, x + 3, y, t, max.saturating_sub(3), panel_style());
                }
                L::Dim(t) => {
                    dialog_kit::put(buf, x, y, t, max, muted());
                }
                L::Blank => {}
            }
            // The keyboard acts on the first suggestion: its whole block
            // carries the selected-row background (buttons keep theirs).
            if matches!(line, L::Sug(0) | L::SugText(0, _)) {
                let lit = dialog_kit::row_bg(true, false);
                for cx in inner.x..inner.right() {
                    let on_button = hits
                        .iter()
                        .any(|(r, _)| r.y == y && cx >= r.x && cx < r.right());
                    if !on_button && buf[(cx, y)].bg == th().ui.panel {
                        buf[(cx, y)].set_bg(lit);
                    }
                }
            }
        }
        self.ov_bar = crate::scrollbar::Track::vertical(scroll, self.ov_len, self.ov_offset);
        if let Some(track) = self.ov_bar {
            crate::scrollbar::render_buf(buf, track, true, false);
        }
        // The reply box: a click or `c` opens the comment box.
        let reply = Rect::new(body.x, body.bottom() - box_h, body.width, box_h);
        let hovered = self.hovered(Hit::Compose);
        let inner = block(buf, reply, msg::REPLY_BOX_TITLE.get(), hovered);
        dialog_kit::put(
            buf,
            inner.x + 1,
            inner.y,
            msg::REPLY_PLACEHOLDER.get(),
            inner.width.saturating_sub(2),
            muted(),
        );
        hits.push((reply, Hit::Compose));
    }

    // -- Files ----------------------------------------------------------------------

    /// The widths of a page that is a list and a diff.
    fn split(body: Rect) -> (Rect, Rect) {
        let left = (body.width / 3).clamp(24, 42).min(body.width / 2);
        (
            Rect::new(body.x, body.y, left, body.height),
            Rect::new(body.x + left, body.y, body.width - left, body.height),
        )
    }

    fn files_page(&mut self, buf: &mut Buffer, body: Rect, hits: &mut Vec<(Rect, Hit)>) {
        let (left, right) = Self::split(body);
        self.file_list(buf, left, hits);
        let title = self
            .current_path()
            .unwrap_or_else(|| msg::NO_FILES.get().to_string());
        let key_name = self.current_path().unwrap_or_default();
        let fd = self.diffs.get(&key_name).cloned();
        // The minimap's columns are not the text's (see `diff_pane`).
        let pad = crate::minimap::padding_for(right, crate::live_settings::minimap());
        let width = right.width.saturating_sub(2 + pad);
        let key = (key_name.clone(), width, self.revision);
        if self.files_pane.stale(&key) {
            let render = fd
                .as_ref()
                .map(|fd| self.build_files_render(fd, &key_name, width));
            self.set_pane_render(PaneId::Files, key, render);
        }
        let waiting = if key_name.is_empty() {
            None
        } else if let Some(why) = &self.git_err {
            Some((why.clone(), true))
        } else if let Some(why) = &self.diff_err {
            Some((why.clone(), true))
        } else if fd.is_none() {
            Some((msg::FETCHING.get().to_string(), false))
        } else {
            None
        };
        self.diff_pane(buf, right, PaneId::Files, &title, waiting, hits);
    }

    fn file_list(&mut self, buf: &mut Buffer, outer: Rect, hits: &mut Vec<(Rect, Hit)>) {
        let Some(h) = self.host.as_ref() else { return };
        let files = h.files.clone();
        let viewed_n = files
            .iter()
            .filter(|f| self.viewed.contains(&f.path))
            .count();
        let title = msg::files_title(files.len() as u64, viewed_n as u64);
        let focused = self.tab == Tab::Files && self.focus == Focus::List;
        let inner = block(buf, outer, &title, focused);
        self.list_rows = inner.height as usize;
        let max_offset = files.len().saturating_sub(self.list_rows);
        self.file_offset = self.file_offset.min(max_offset);
        for (n, (i, f)) in files
            .iter()
            .enumerate()
            .skip(self.file_offset)
            .take(inner.height as usize)
            .enumerate()
        {
            let y = inner.y + n as u16;
            let selected = i == self.file_sel;
            let bg = if selected && focused {
                th().ui.selection
            } else if selected || self.hovered(Hit::File(i)) {
                th().ui.hover
            } else {
                th().ui.panel
            };
            dialog_kit::fill(
                buf,
                Rect::new(inner.x, y, inner.width, 1),
                Style::default().bg(bg),
            );
            let viewed = self.viewed.contains(&f.path);
            let check = dialog_kit::put(
                buf,
                inner.x,
                y,
                if viewed { "[x]" } else { "[ ]" },
                3,
                Style::default()
                    .fg(if viewed {
                        th().status.success
                    } else {
                        th().ui.text
                    })
                    .bg(if self.hovered(Hit::Viewed(i)) {
                        th().ui.button_hover
                    } else {
                        bg
                    }),
            );
            hits.push((Rect::new(check.x, y, 3, 1), Hit::Viewed(i)));
            let (letter, color) = match f.status {
                gitpoulpi_hosting::FileStatus::Added => ("A", th().status.success),
                gitpoulpi_hosting::FileStatus::Removed => ("D", th().status.danger),
                gitpoulpi_hosting::FileStatus::Renamed => ("R", th().status.info),
                gitpoulpi_hosting::FileStatus::Modified => ("M", th().status.warning),
            };
            dialog_kit::put(
                buf,
                inner.x + 4,
                y,
                letter,
                1,
                Style::default().fg(color).bg(bg),
            );
            // Right side: counts of threads, pending and lines changed.
            let threads = self.open_threads_in(&f.path);
            let pend = self.pending.iter().filter(|c| c.path == f.path).count();
            let mut tail = String::new();
            if threads > 0 {
                tail.push_str(&format!("\u{25CF}{threads} "));
            }
            if pend > 0 {
                tail.push_str(&format!("\u{270E}{pend} "));
            }
            tail.push_str(&format!("+{} -{}", f.additions, f.deletions));
            let tail_w = (display_width(&tail) as u16).min(inner.width / 2);
            let name_w = inner.width.saturating_sub(6 + tail_w + 1) as usize;
            let name = tail_fit(&f.path, name_w);
            let name_style = Style::default()
                .fg(if viewed { th().ui.muted } else { th().ui.text })
                .bg(bg);
            dialog_kit::put(buf, inner.x + 6, y, &name, name_w as u16, name_style);
            dialog_kit::put(
                buf,
                inner.right().saturating_sub(tail_w),
                y,
                &tail,
                tail_w,
                Style::default().fg(th().ui.muted).bg(bg),
            );
            hits.push((Rect::new(inner.x, y, inner.width, 1), Hit::File(i)));
        }
        self.file_bar = crate::scrollbar::Track::vertical(outer, files.len(), self.file_offset);
        if let Some(track) = self.file_bar {
            crate::scrollbar::render_buf(buf, track, true, false);
        }
    }

    // -- Commits --------------------------------------------------------------------

    fn commits_page(&mut self, buf: &mut Buffer, body: Rect, hits: &mut Vec<(Rect, Hit)>) {
        let (left, right) = Self::split(body);
        let commits = self.git.as_ref().map(|g| g.commits.clone());
        let focused = self.tab == Tab::Commits && self.focus == Focus::List;
        let n = commits.as_ref().map_or(0, Vec::len);
        let inner = block(buf, left, &msg::commits_title(n as u64), focused);
        self.list_rows = inner.height as usize;
        match &commits {
            None => {
                let text = if let Some(why) = &self.git_err {
                    why.clone()
                } else if self.started.elapsed() >= SPIN_AFTER {
                    format!("{} {}", self.spinner(), msg::FETCHING.get())
                } else {
                    String::new()
                };
                dialog_kit::put(
                    buf,
                    inner.x + 1,
                    inner.y,
                    &text,
                    inner.width.saturating_sub(2),
                    muted(),
                );
            }
            Some(list) => {
                self.commit_offset = self
                    .commit_offset
                    .min(list.len().saturating_sub(self.list_rows));
                for (row, (i, c)) in list
                    .iter()
                    .enumerate()
                    .skip(self.commit_offset)
                    .take(inner.height as usize)
                    .enumerate()
                {
                    let y = inner.y + row as u16;
                    let selected = i == self.commit_sel;
                    let bg = if selected && focused {
                        th().ui.selection
                    } else if selected || self.hovered(Hit::Commit(i)) {
                        th().ui.hover
                    } else {
                        th().ui.panel
                    };
                    dialog_kit::fill(
                        buf,
                        Rect::new(inner.x, y, inner.width, 1),
                        Style::default().bg(bg),
                    );
                    let sha = short(c.id);
                    let used = dialog_kit::put(
                        buf,
                        inner.x + 1,
                        y,
                        &sha,
                        8,
                        Style::default().fg(th().ui.accent).bg(bg),
                    );
                    dialog_kit::put(
                        buf,
                        used.right() + 1,
                        y,
                        &c.summary,
                        inner.right().saturating_sub(used.right() + 2),
                        Style::default().fg(th().ui.text).bg(bg),
                    );
                    hits.push((Rect::new(inner.x, y, inner.width, 1), Hit::Commit(i)));
                }
                self.commit_bar =
                    crate::scrollbar::Track::vertical(left, list.len(), self.commit_offset);
                if let Some(track) = self.commit_bar {
                    crate::scrollbar::render_buf(buf, track, true, false);
                }
            }
        }
        // The selected commit's diff.
        let selected = commits.as_ref().and_then(|l| l.get(self.commit_sel));
        let title = selected.map_or_else(
            || msg::NO_COMMITS.get().to_string(),
            |c| format!("{} {}", short(c.id), c.summary),
        );
        let (key_name, files) = match selected {
            Some(c) => (c.id.to_hex(), self.commit_diffs.get(&c.id).cloned()),
            None => (String::new(), None),
        };
        let pad = crate::minimap::padding_for(right, crate::live_settings::minimap());
        let width = right.width.saturating_sub(2 + pad);
        let key = (key_name.clone(), width, self.revision);
        if self.commits_pane.stale(&key) {
            let render = match (&files, selected) {
                (Some(files), Some(c)) => Some(self.build_commit_render(c.id, files, width)),
                _ => None,
            };
            self.set_pane_render(PaneId::Commits, key, render);
        }
        let waiting = if key_name.is_empty() {
            None
        } else if let Some(why) = &self.diff_err {
            Some((why.clone(), true))
        } else if files.is_none() {
            Some((msg::FETCHING.get().to_string(), false))
        } else {
            None
        };
        self.diff_pane(buf, right, PaneId::Commits, &title, waiting, hits);
    }

    // -- status and footer ------------------------------------------------------------

    fn status_line(&self, buf: &mut Buffer, x: u16, y: u16, width: u16) {
        if let Some(flight) = &self.call {
            if flight.since.elapsed() >= SPIN_AFTER {
                dialog_kit::put(
                    buf,
                    x,
                    y,
                    &format!("{} {}", self.spinner(), msg::SENDING.get()),
                    width,
                    Style::default().fg(th().ui.accent).bg(th().ui.panel),
                );
            }
            return;
        }
        if let Some(text) = self.suggest_status() {
            dialog_kit::put(
                buf,
                x,
                y,
                &text,
                width,
                Style::default().fg(th().status.ai).bg(th().ui.panel),
            );
        } else if let Some(text) = &self.message {
            dialog_kit::put(
                buf,
                x,
                y,
                text,
                width,
                Style::default().fg(th().ui.accent).bg(th().ui.panel),
            );
        } else if let Some(why) = &self.git_err {
            if self.host.is_some() {
                dialog_kit::put(
                    buf,
                    x,
                    y,
                    &msg::git_failed(why),
                    width,
                    Style::default().fg(th().status.warning).bg(th().ui.panel),
                );
            }
        }
    }

    fn footer(&self, buf: &mut Buffer, x: u16, y: u16, width: u16) {
        use crate::keymap::chord::Key;
        let tabs_key = format!("{}-3", dialog_kit::alt_key(Key::Char('1')));
        let mut pairs: Vec<(&str, &str)> = match self.tab {
            Tab::Overview => vec![
                ("\u{2191}\u{2193}", msg::HINT_SCROLL.get()),
                ("c", msg::HINT_COMMENT.get()),
                ("f", msg::HINT_FINISH.get()),
                ("o", msg::HINT_BROWSER.get()),
                (tabs_key.as_str(), msg::HINT_TABS.get()),
                (msg::KEY_ESC.get(), msg::HINT_CLOSE.get()),
            ],
            Tab::Files => vec![
                ("\u{2191}\u{2193}", msg::HINT_MOVE.get()),
                ("c", msg::HINT_COMMENT_LINE.get()),
                ("r", msg::HINT_REPLY.get()),
                ("x", msg::HINT_RESOLVE.get()),
                ("v", msg::HINT_VIEWED.get()),
                ("f", msg::HINT_FINISH.get()),
                (tabs_key.as_str(), msg::HINT_TABS.get()),
                (msg::KEY_ESC.get(), msg::HINT_CLOSE.get()),
            ],
            Tab::Commits => vec![
                ("\u{2191}\u{2193}", msg::HINT_MOVE.get()),
                ("Enter", msg::HINT_OPEN_DIFF.get()),
                (tabs_key.as_str(), msg::HINT_TABS.get()),
                (msg::KEY_ESC.get(), msg::HINT_CLOSE.get()),
            ],
        };
        // The host cannot resolve (Forgejo, Gerrit's patch sets): no `x` hint.
        let can_resolve = self
            .host
            .as_ref()
            .is_none_or(|h| h.detail.capabilities.resolve);
        if !can_resolve {
            pairs.retain(|(key, _)| *key != "x");
        }
        // ✦ hints (AI on only), right after the page's first one.
        if self.ai_on() && self.tab != Tab::Commits {
            let mut ai: Vec<(&str, &str)> = vec![("s", ai_msg::BUTTON.get())];
            if self.has_draft() {
                ai.push(("u", ai_msg::USE_DRAFT.get()));
            }
            if self.sug.items.is_empty() {
                // nothing to accept yet
            } else {
                ai.push(("y", ai_msg::ACCEPT.get()));
                ai.push(("n", ai_msg::DISMISS.get()));
            }
            let at = 2.min(pairs.len());
            pairs.splice(at..at, ai);
        }
        dialog_kit::hints(buf, x, y, width, &pairs);
    }
}
