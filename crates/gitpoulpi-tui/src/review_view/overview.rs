//! The Overview page: the request's conversation as the shared timeline with
//! its docked composer (`item_view`, embedded), the same an issue has, and
//! above it, only after a run of the AI, the AI block (the draft card, the
//! summary and the suggestions the reviewer accepts or dismisses).
//!
//! The conversation owns its keys and its mouse; this module routes them to
//! it (the review's own keys first where they do not clash, see
//! `overview_key`) and paints the AI block, which is the review's.

use gitpoulpi_i18n::msg::review_ai as ai_msg;
use gitpoulpi_i18n::msg::review_view as msg;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};

use super::draw::{block, muted, panel_style, sev_color, Hit};
use super::{ReviewEffect, ReviewView};
use crate::commit_detail::wrap_text;
use crate::dialog_kit::{self, ButtonState};
use crate::item_view::ItemEffect;
use crate::md_render;
use crate::review_cmd::{ReviewCmd, Tab};
use crate::th;

/// One row of the AI block.
enum L {
    /// A line of Markdown text, already wrapped and styled.
    Md(std::rc::Rc<md_render::Doc>, usize),
    Dim(String),
    Blank,
    /// The heading of the block, with the number of suggestions.
    SugHeading(usize),
    /// The head of suggestion `n` of the list.
    Sug(usize),
    /// The heading of the draft card (with the stale mark).
    DraftHeading(bool),
    /// The verdict chip and the draft's three buttons.
    DraftBar,
    SugText(usize, String),
}

/// The conversation keeps at least this many rows; the AI block gives way.
const MIN_CONVERSATION_ROWS: u16 = 10;

impl ReviewView {
    /// The AI block's rows (empty before a run) and the addresses its
    /// Markdown links to.
    fn strip_lines(&mut self, width: usize) -> (Vec<L>, Vec<String>) {
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
        } // The palette's "Open Link in Message" twin reaches every link of the block.
        let mut urls: Vec<String> = Vec::new();
        for doc in &all_docs {
            for l in &doc.links {
                if !urls.contains(&l.url) {
                    urls.push(l.url.clone());
                }
            }
        }
        (lines, urls)
    }

    /// Paints the Overview into `body`.
    pub(super) fn overview(&mut self, buf: &mut Buffer, body: Rect, hits: &mut Vec<(Rect, Hit)>) {
        if body.height < 6 {
            return;
        }
        self.md_links.clear();
        let width = body.width.saturating_sub(6) as usize;
        let (lines, urls) = self.strip_lines(width);
        crate::link_confirm::set_message_urls(urls);
        // The AI block: its rows plus the frame, half of the page at most.
        let room = body.height.saturating_sub(MIN_CONVERSATION_ROWS);
        let want = u16::try_from(lines.len())
            .unwrap_or(u16::MAX)
            .saturating_add(2);
        let strip_h = if lines.is_empty() {
            0
        } else {
            want.min(room).min((body.height / 2).max(4))
        };
        let strip_h = if strip_h < 4 { 0 } else { strip_h };
        self.ov_strip = Rect::new(body.x, body.y, body.width, strip_h);
        if strip_h == 0 {
            self.ov_len = 0;
            self.ov_rows = 0;
            self.ov_offset = 0;
            self.ov_bar = None;
        } else {
            self.paint_strip(buf, self.ov_strip, &lines, hits);
        }
        let rest = Rect::new(body.x, body.y + strip_h, body.width, body.height - strip_h);
        self.ov.render_embedded(rest, buf);
    }

    fn paint_strip(
        &mut self,
        buf: &mut Buffer,
        scroll: Rect,
        lines: &[L],
        hits: &mut Vec<(Rect, Hit)>,
    ) {
        let inner = block(
            buf,
            scroll,
            msg::OVERVIEW_TITLE.get(),
            self.focus == super::model::Focus::List,
        );
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
    }
}

impl ReviewView {
    /// Whether the conversation has the keyboard whatever the key: its
    /// composer is focused, or one of its popups or questions is open.
    fn overview_owns_keys(&self) -> bool {
        !self.ov.takes_palette()
    }

    /// A key on the Overview. `None`: the review's own (it goes on to
    /// `key_cmd`), else the conversation took it.
    ///
    /// The timeline's keys are the same as an issue's (`r` reply, `Q` quote,
    /// `e` / `d` edit and delete, `l` links, `g` diagram, `m` load older,
    /// `n` next unread, `o` browser, `c` new comment). The review keeps what
    /// does not clash: the pages, Finish, Checkout, the metadata keys that
    /// have no timeline meaning (`a`, `p`), the patch sets and the AI keys;
    /// `y` / `w` / `n` act on the first suggestion while there is one. The
    /// rest (labels, milestone, Gerrit's abandon) stay on the buttons and the
    /// palette.
    pub(super) fn overview_key(&mut self, key: KeyEvent) -> Option<ReviewEffect> {
        let alt_page = key.modifiers.contains(KeyModifiers::ALT)
            && matches!(key.code, KeyCode::Char('1' | '2' | '3'));
        if alt_page {
            return None;
        }
        if self.overview_owns_keys() {
            let effect = self.ov.embedded_key(key)?;
            return Some(self.on_overview_effect(effect));
        }
        if let Some(cmd) = super::input::key_cmd(Tab::Overview, &key) {
            let own = matches!(
                cmd,
                ReviewCmd::TabOverview
                    | ReviewCmd::TabFiles
                    | ReviewCmd::TabCommits
                    | ReviewCmd::Refresh
                    | ReviewCmd::FinishReview
                    | ReviewCmd::Checkout
                    | ReviewCmd::EditAssignees
                    | ReviewCmd::EditReviewers
                    | ReviewCmd::PickPatchSet
                    | ReviewCmd::ComparePatchSet
                    | ReviewCmd::SubmitChange
                    | ReviewCmd::SuggestReview
                    | ReviewCmd::AcceptAllSuggestions
                    | ReviewCmd::DismissAllSuggestions
                    | ReviewCmd::UseSummary
                    | ReviewCmd::UseDraft
                    | ReviewCmd::EditDraftText
            );
            let on_suggestion = matches!(
                cmd,
                ReviewCmd::AcceptSuggestion
                    | ReviewCmd::EditSuggestion
                    | ReviewCmd::DismissSuggestion
            ) && self.current_suggestion().is_some();
            if own || on_suggestion {
                return None;
            }
        }
        let effect = self.ov.embedded_key(key)?;
        Some(self.on_overview_effect(effect))
    }

    /// A mouse event on the Overview. `None`: not the conversation's.
    pub(super) fn overview_mouse(&mut self, mouse: MouseEvent) -> Option<ReviewEffect> {
        let at = (mouse.column, mouse.row).into();
        let popup = !self.ov.takes_palette_for_mouse();
        if !popup && self.ov_strip.contains(at) {
            self.ov.clear_hover();
            return None;
        }
        if self.ov.owns_mouse(&mouse) {
            let effect = self.ov.on_mouse(mouse);
            return Some(self.on_overview_effect(effect));
        }
        self.ov.clear_hover();
        None
    }

    /// What the conversation asked of the review.
    pub(super) fn on_overview_effect(&mut self, effect: ItemEffect) -> ReviewEffect {
        match effect {
            ItemEffect::None | ItemEffect::ShowCommit(_) => ReviewEffect::None,
            ItemEffect::Close => {
                // A suggestion being made is stopped first; the view stays.
                if self.cancel_suggest() {
                    ReviewEffect::None
                } else {
                    ReviewEffect::Close
                }
            }
            ItemEffect::Message(text) => {
                self.message = Some(text);
                ReviewEffect::None
            }
            ItemEffect::OpenUrl(url) => {
                self.message = Some(match crate::repo_menu::open_url(&url) {
                    Ok(()) => gitpoulpi_i18n::msg::hosting_ui::opened_in_browser(&url),
                    Err(e) => msg::browser_failed(&e),
                });
                ReviewEffect::None
            }
            ItemEffect::OpenDiagram(source) => {
                crate::mermaid_popup::open_script(&source);
                ReviewEffect::None
            }
        }
    }

    /// Drains the conversation's worker (every frame).
    pub(super) fn poll_overview(&mut self) -> Option<ReviewEffect> {
        let effect = self.ov.poll();
        match effect {
            ItemEffect::None => None,
            other => Some(self.on_overview_effect(other)),
        }
    }

    /// Gives the conversation's composer its "Write with AI" help once it
    /// is open (the request's context comes from what the review loaded).
    pub(super) fn sync_overview_ai(&mut self) {
        if !self.ov.composer_needs_ai() {
            return;
        }
        let reply = self.ov.composer_replies_to().map(str::to_string);
        if let Some((scope, source)) = self.overview_ai(reply.as_deref()) {
            self.ov.bind_composer_ai(scope, source);
        }
    }
}
