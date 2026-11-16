//! Painting the timeline, and the hit rects of that same walk (maintainer
//! rule 6): `paint` clears the hit list, and every row pushes what it drew
//! before the band it sits in, so a button or a link is found before the row
//! under it. The hit list is what the mouse handler reads next.

use gitpoulpi_hosting::conversation::{ActorKind, TimelineEntry};
use gitpoulpi_hosting::review::model::ReviewState;
use gitpoulpi_i18n::msg::{conversation_ui as m, item_view as iv};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use super::rows::{event_label, short_time, Row, What, BODY_INDENT};
use super::{Btn, Conversation, Hit, Load};
use crate::dialog_kit::{self, ButtonState};
use crate::md_render::{self, sanitize};
use crate::th;
use crate::ui_kit::display_width;

fn base() -> Style {
    Style::default().fg(th().ui.text).bg(th().ui.panel)
}

fn muted() -> Style {
    Style::default().fg(th().ui.muted).bg(th().ui.panel)
}

/// Writes `parts` one after the other from `x`; returns where they ended.
/// How a code thread's headline spends `max` cells on its link: whether the
/// "Thread" label still shows, and the `path:line` text, ellipsized at its
/// start when even alone it does not fit. The link itself never goes: the
/// label gives way first, then the path shortens.
fn place_fit(place: &str, max: u16) -> (bool, String) {
    let label_w = display_width(m::THREAD.get()) + 2;
    let max = usize::from(max);
    let w = display_width(place);
    if label_w + w <= max {
        return (true, place.to_string());
    }
    if w <= max {
        return (false, place.to_string());
    }
    if max == 0 {
        return (false, String::new());
    }
    let mut tail: Vec<char> = Vec::new();
    let mut used = 1;
    for ch in place.chars().rev() {
        let cw = display_width(ch.encode_utf8(&mut [0; 4]));
        if used + cw > max {
            break;
        }
        used += cw;
        tail.push(ch);
    }
    tail.reverse();
    (
        false,
        format!("\u{2026}{}", tail.into_iter().collect::<String>()),
    )
}

/// `path:line` of a thread (the line it was written on when it is outdated).
pub(crate) fn thread_place(t: &gitpoulpi_hosting::conversation::CodeThread) -> String {
    match t.line.or(t.original_line) {
        Some(line) => format!("{}:{line}", sanitize(&t.path)),
        None => sanitize(&t.path),
    }
}

fn put_parts(
    buf: &mut Buffer,
    mut x: u16,
    y: u16,
    limit: u16,
    parts: &[(String, Style)],
    bg: ratatui::style::Color,
) -> u16 {
    for (text, style) in parts {
        if x >= limit {
            break;
        }
        let r = dialog_kit::put(buf, x, y, text, limit - x, style.bg(bg));
        x = r.right();
    }
    x
}

impl Conversation {
    /// Paints the timeline into `area`. `focused` lights the selection.
    pub(crate) fn paint(&mut self, buf: &mut Buffer, area: Rect, focused: bool) {
        self.hits.clear();
        self.links.clear();
        self.bar = None;
        self.area = area;
        dialog_kit::fill(buf, area, base());
        if area.width < 12 || area.height == 0 {
            self.view_rows = 0;
            return;
        }
        match self.load.clone() {
            Load::Loading => {
                if self.started.elapsed() >= super::SPIN_AFTER {
                    dialog_kit::put(
                        buf,
                        area.x + 1,
                        area.y,
                        &format!("{} {}", self.spinner(), iv::LOADING.get()),
                        area.width - 1,
                        Style::default().fg(th().ui.accent).bg(th().ui.panel),
                    );
                }
                return;
            }
            Load::Failed(why) => {
                dialog_kit::put(
                    buf,
                    area.x + 1,
                    area.y,
                    &iv::load_failed(&why),
                    area.width - 1,
                    Style::default().fg(th().status.danger).bg(th().ui.panel),
                );
                dialog_kit::put(
                    buf,
                    area.x + 1,
                    area.y + 1,
                    iv::RETRY_HINT.get(),
                    area.width - 1,
                    muted(),
                );
                return;
            }
            Load::Ready => {}
        }
        let mut top = area.y;
        // Banners: offline, a notice, "Load older".
        if let Some(o) = self.offline.clone().filter(|o| !o.why.is_empty()) {
            let text = match self.age_text() {
                Some(age) => iv::offline_banner(&age, sanitize(&o.why)),
                None => iv::offline_banner_unknown(sanitize(&o.why)),
            };
            dialog_kit::put(
                buf,
                area.x + 1,
                top,
                &text,
                area.width - 1,
                Style::default().fg(th().status.warning).bg(th().ui.panel),
            );
            top += 1;
        }
        if let Some(n) = self.notice.clone() {
            dialog_kit::put(
                buf,
                area.x + 1,
                top,
                &sanitize(&n),
                area.width - 1,
                Style::default().fg(th().status.danger).bg(th().ui.panel),
            );
            top += 1;
        }
        if self.older.is_some() && top < area.bottom() {
            let label = match self.older_count() {
                Some(n) => iv::load_older_n(u64::from(n)),
                None => iv::LOAD_OLDER.get().to_string(),
            };
            let state = if self.loading_older {
                ButtonState::Disabled
            } else {
                ButtonState::Normal {
                    hovered: self.hover == Some(Hit::LoadOlder),
                    focused: false,
                }
            };
            let r = dialog_kit::button(buf, area.x + 1, top, &label, state);
            self.hits.push((r, Hit::LoadOlder));
            if self.loading_older {
                dialog_kit::put(
                    buf,
                    r.right() + 1,
                    top,
                    &format!("{} {}", self.spinner(), iv::LOADING.get()),
                    area.right().saturating_sub(r.right() + 2),
                    Style::default().fg(th().ui.accent).bg(th().ui.panel),
                );
            }
            top += 1;
        }
        if top >= area.bottom() {
            self.view_rows = 0;
            return;
        }
        let body = Rect::new(area.x, top, area.width, area.bottom() - top);
        self.view_rows = usize::from(body.height);
        let content_w = usize::from(body.width.saturating_sub(2));
        self.ensure_layout(content_w);
        self.follow_selection();
        let layout = std::sync::Arc::clone(&self.layout);
        let total = layout.rows.len();
        let band_w = body.width - 1;
        let content_x = body.x + 1;
        let content_max = body.width.saturating_sub(2);
        for line in 0..usize::from(body.height) {
            let index = self.top + line;
            let Some(row) = layout.rows.get(index) else {
                break;
            };
            let y = body.y + line as u16;
            let item = row.item();
            let selected = item == Some(self.sel) && focused;
            let hovered = item.is_some()
                && matches!(self.hover, Some(Hit::Item(i) | Hit::Btn(_, i)) if Some(i) == item);
            let lit = item == Some(self.sel) || hovered;
            let bg = dialog_kit::row_bg(lit && (selected || item == Some(self.sel)), hovered);
            if item.is_some() {
                dialog_kit::row_band(buf, Rect::new(body.x, y, band_w, 1), bg);
            }
            if let Some(i) = item {
                if layout.items.get(i).is_some_and(|it| it.unread) {
                    dialog_kit::put(
                        buf,
                        body.x,
                        y,
                        "\u{258C}",
                        1,
                        Style::default().fg(th().ui.accent).bg(bg),
                    );
                }
            }
            let mut row_hits: Vec<(Rect, Hit)> = Vec::new();
            match row {
                Row::Head(i) => {
                    let end = self.paint_head(buf, content_x, y, content_max, *i, bg);
                    if *i == self.sel {
                        self.paint_buttons(buf, body, y, end, *i, &mut row_hits);
                    }
                }
                Row::ThreadHead(i) => {
                    let end = self.paint_thread_head(buf, content_x, y, content_max, *i, bg);
                    if let Some(rect) = self.thread_place_rect(content_x, y, content_max, *i) {
                        crate::pointer_map::hand(rect);
                        row_hits.push((rect, Hit::Code(*i)));
                    }
                    if *i == self.sel {
                        self.paint_buttons(buf, body, y, end, *i, &mut row_hits);
                    }
                }
                Row::Hunk(_, text) => {
                    let style = if text.starts_with('+') {
                        Style::default().fg(th().diff.add_fg)
                    } else if text.starts_with('-') {
                        Style::default().fg(th().diff.del_fg)
                    } else if text.starts_with("@@") {
                        Style::default().fg(th().diff.hunk_fg)
                    } else {
                        Style::default().fg(th().ui.muted)
                    };
                    dialog_kit::put(
                        buf,
                        content_x + BODY_INDENT,
                        y,
                        text,
                        content_max.saturating_sub(BODY_INDENT),
                        style.bg(bg),
                    );
                }
                Row::Sub(i, ci) => self.paint_sub(buf, content_x, y, content_max, *i, *ci, bg),
                Row::Body(_, doc, n, indent) => {
                    if let Some(md) = doc.lines.get(*n) {
                        let hits = md_render::paint(
                            buf,
                            content_x + indent,
                            y,
                            content_max.saturating_sub(*indent),
                            md,
                            Style::default().fg(th().ui.text).bg(bg),
                            false,
                        );
                        for (rect, id) in hits {
                            if let Some(target) = doc.links.get(id) {
                                crate::pointer_map::hand(rect);
                                row_hits.push((rect, Hit::Link(self.links.len())));
                                self.links.push(target.clone());
                            }
                        }
                    }
                }
                Row::Empty(_) => {
                    dialog_kit::put(
                        buf,
                        content_x + BODY_INDENT,
                        y,
                        iv::NO_DESCRIPTION.get(),
                        content_max.saturating_sub(BODY_INDENT),
                        Style::default()
                            .fg(th().ui.muted)
                            .bg(bg)
                            .add_modifier(Modifier::ITALIC),
                    );
                }
                Row::Reactions(_, text) => {
                    dialog_kit::put(
                        buf,
                        content_x + BODY_INDENT,
                        y,
                        text,
                        content_max.saturating_sub(BODY_INDENT),
                        Style::default().fg(th().ui.muted).bg(bg),
                    );
                }
                Row::Event(i) => self.paint_event(buf, content_x, y, content_max, *i, bg),
                Row::Blank => {}
            }
            // Inner first: buttons and links, then the row under them.
            self.hits.extend(row_hits);
            if let Some(i) = item {
                self.hits
                    .push((Rect::new(body.x, y, band_w, 1), Hit::Item(i)));
            }
        }
        // The bar.
        let strip = Rect::new(body.right() - 1, body.y, 1, body.height);
        self.bar =
            crate::scrollbar::Track::column(strip, total, usize::from(body.height), self.top);
        if let Some(track) = self.bar {
            crate::scrollbar::render_buf(buf, track, true, false);
        }
    }

    // -- one row each ----------------------------------------------------------

    fn paint_head(
        &self,
        buf: &mut Buffer,
        x: u16,
        y: u16,
        max: u16,
        item: usize,
        bg: ratatui::style::Color,
    ) -> u16 {
        let Some(it) = self.layout.items.get(item) else {
            return x;
        };
        let strong = Style::default()
            .fg(th().ui.text)
            .add_modifier(Modifier::BOLD);
        let dim = Style::default().fg(th().ui.muted);
        let mut parts: Vec<(String, Style)> = Vec::new();
        let limit = x + max;
        match it.what {
            What::Body => {
                if let Some(h) = &self.header {
                    parts.push((author_text(&h.author), strong));
                    parts.push((format!("  {}", iv::OPENED_THIS.get()), dim));
                    parts.push((format!("  {}", short_time(&h.created_at)), dim));
                }
            }
            What::Entry(e) => match self.entries.get(e) {
                Some(TimelineEntry::Comment(c)) => {
                    parts.push((author_text(&c.author), strong));
                    parts.push((format!("  {}", short_time(&c.created_at)), dim));
                    if c.edited {
                        parts.push((format!("  ({})", m::EDITED.get()), dim));
                    }
                    if c.internal {
                        parts.push((
                            format!("  {}", m::INTERNAL.get()),
                            Style::default().fg(th().status.warning),
                        ));
                    }
                }
                Some(TimelineEntry::Review(r)) => {
                    parts.push((author_text(&r.author), strong));
                    let (word, color) = verdict(r.state);
                    parts.push((format!("  {word}"), Style::default().fg(color)));
                    parts.push((format!("  {}", short_time(&r.submitted_at)), dim));
                }
                _ => {}
            },
        }
        if it.unread {
            parts.push((
                format!("  {}", m::NEW.get()),
                Style::default()
                    .fg(th().ui.accent)
                    .add_modifier(Modifier::BOLD),
            ));
        }
        put_parts(buf, x, y, limit, &parts, bg)
    }

    fn paint_thread_head(
        &self,
        buf: &mut Buffer,
        x: u16,
        y: u16,
        max: u16,
        item: usize,
        bg: ratatui::style::Color,
    ) -> u16 {
        let Some(What::Entry(e)) = self.layout.items.get(item).map(|i| i.what) else {
            return x;
        };
        let Some(TimelineEntry::Thread(t)) = self.entries.get(e) else {
            return x;
        };
        let place = thread_place(t);
        let link = self.code_links;
        let hot = self.hover == Some(Hit::Code(item));
        let place_style = if link {
            // A link: the accent, underlined; lit under the pointer.
            let s = Style::default()
                .fg(th().ui.accent)
                .add_modifier(Modifier::UNDERLINED);
            if hot {
                s.add_modifier(Modifier::BOLD)
            } else {
                s
            }
        } else {
            Style::default().fg(th().ui.text)
        };
        let (show_label, place) = if link {
            place_fit(&place, max)
        } else {
            (true, place)
        };
        let mut parts = Vec::new();
        if show_label {
            parts.push((
                m::THREAD.get().to_string(),
                Style::default()
                    .fg(th().ui.accent)
                    .add_modifier(Modifier::BOLD),
            ));
            parts.push(("  ".to_string(), Style::default()));
        }
        parts.push((place, place_style));
        if t.resolved {
            parts.push((
                format!("  {}", m::RESOLVED.get()),
                Style::default().fg(th().status.success),
            ));
        } else if t.resolvable {
            parts.push((
                format!("  {}", m::UNRESOLVED.get()),
                Style::default().fg(th().status.warning),
            ));
        }
        if t.outdated {
            parts.push((
                format!("  {}", m::OUTDATED.get()),
                Style::default().fg(th().ui.muted),
            ));
        }
        if self.layout.items.get(item).is_some_and(|i| i.unread) {
            parts.push((
                format!("  {}", m::NEW.get()),
                Style::default()
                    .fg(th().ui.accent)
                    .add_modifier(Modifier::BOLD),
            ));
        }
        put_parts(buf, x, y, x + max, &parts, bg)
    }

    /// Where the "path:line" of a thread's headline was painted, when it is
    /// a link (the review's Overview).
    fn thread_place_rect(&self, x: u16, y: u16, max: u16, item: usize) -> Option<Rect> {
        if !self.code_links {
            return None;
        }
        let What::Entry(e) = self.layout.items.get(item)?.what else {
            return None;
        };
        let TimelineEntry::Thread(t) = self.entries.get(e)? else {
            return None;
        };
        let (show_label, text) = place_fit(&thread_place(t), max);
        let start = x + if show_label {
            display_width(m::THREAD.get()) as u16 + 2
        } else {
            0
        };
        let limit = x + max;
        if start >= limit || text.is_empty() {
            return None;
        }
        let w = (display_width(&text) as u16).min(limit - start);
        Some(Rect::new(start, y, w, 1))
    }

    #[allow(clippy::too_many_arguments)]
    fn paint_sub(
        &self,
        buf: &mut Buffer,
        x: u16,
        y: u16,
        max: u16,
        item: usize,
        ci: usize,
        bg: ratatui::style::Color,
    ) {
        let Some(What::Entry(e)) = self.layout.items.get(item).map(|i| i.what) else {
            return;
        };
        let Some(TimelineEntry::Thread(t)) = self.entries.get(e) else {
            return;
        };
        let Some(c) = t.comments.get(ci) else {
            return;
        };
        let mut parts = vec![
            (
                author_text(&c.author),
                Style::default()
                    .fg(th().ui.text)
                    .add_modifier(Modifier::BOLD),
            ),
            (
                format!("  {}", short_time(&c.created_at)),
                Style::default().fg(th().ui.muted),
            ),
        ];
        if c.edited {
            parts.push((
                format!("  ({})", m::EDITED.get()),
                Style::default().fg(th().ui.muted),
            ));
        }
        put_parts(buf, x + BODY_INDENT, y, x + max, &parts, bg);
    }

    fn paint_event(
        &self,
        buf: &mut Buffer,
        x: u16,
        y: u16,
        max: u16,
        item: usize,
        bg: ratatui::style::Color,
    ) {
        let Some(What::Entry(e)) = self.layout.items.get(item).map(|i| i.what) else {
            return;
        };
        let Some(TimelineEntry::Event(ev)) = self.entries.get(e) else {
            return;
        };
        let mut parts: Vec<(String, Style)> = Vec::new();
        if let Some(actor) = &ev.actor {
            parts.push((
                format!("{} ", author_text(actor)),
                Style::default().fg(th().ui.text),
            ));
        }
        parts.push((
            format!("{}  ", event_label(&ev.kind)),
            Style::default().fg(th().ui.muted),
        ));
        parts.push((short_time(&ev.at), Style::default().fg(th().ui.muted)));
        if self.layout.items.get(item).is_some_and(|i| i.unread) {
            parts.push((
                format!("  {}", m::NEW.get()),
                Style::default()
                    .fg(th().ui.accent)
                    .add_modifier(Modifier::BOLD),
            ));
        }
        put_parts(buf, x + BODY_INDENT, y, x + max, &parts, bg);
    }

    /// The buttons of the selected item, right-aligned on its head row, as
    /// many as fit after what the head wrote (`from`).
    fn paint_buttons(
        &self,
        buf: &mut Buffer,
        body: Rect,
        y: u16,
        from: u16,
        item: usize,
        hits: &mut Vec<(Rect, Hit)>,
    ) {
        let mut wanted: Vec<(Btn, &str, ButtonState)> = Vec::new();
        let hovered = |b: Btn| self.hover == Some(Hit::Btn(b, item));
        let normal = |b: Btn| ButtonState::Normal {
            hovered: hovered(b),
            focused: false,
        };
        let is_event = matches!(self.selected_entry(), Some(TimelineEntry::Event(_)));
        if !is_event {
            wanted.push((
                Btn::Reply,
                m::BTN_REPLY.get(),
                if self.reply_reason().is_some() {
                    ButtonState::Disabled
                } else {
                    normal(Btn::Reply)
                },
            ));
        }
        if !is_event
            && self.caps.as_ref().is_some_and(|c| c.comment)
            && matches!(self.reply_to(), super::write::ReplyTo::Discussion { .. })
        {
            wanted.push((
                Btn::Quote,
                m::BTN_QUOTE.get(),
                if self.reply_reason().is_some() {
                    ButtonState::Disabled
                } else {
                    normal(Btn::Quote)
                },
            ));
        }
        if self.can_edit_selected() {
            wanted.push((
                Btn::Edit,
                m::BTN_EDIT.get(),
                if self.is_writing() {
                    ButtonState::Disabled
                } else {
                    normal(Btn::Edit)
                },
            ));
        }
        if self.can_delete_selected() {
            wanted.push((
                Btn::Delete,
                m::BTN_DELETE.get(),
                if self.is_writing() {
                    ButtonState::Disabled
                } else {
                    normal(Btn::Delete)
                },
            ));
        }
        if self.selected().is_some_and(|i| !i.links.is_empty()) {
            wanted.push((Btn::Links, m::BTN_LINKS.get(), normal(Btn::Links)));
        }
        // Right to left; a button that would run into the head is dropped.
        let mut right = body.right().saturating_sub(1);
        for (btn, label, state) in wanted.into_iter().rev() {
            let w = dialog_kit::button_width(label);
            if right < w + from + 1 {
                break;
            }
            let rect = dialog_kit::button(buf, right - w, y, label, state);
            hits.push((rect, Hit::Btn(btn, item)));
            right = right - w - 1;
        }
    }
}

fn author_text(actor: &gitpoulpi_hosting::conversation::Actor) -> String {
    let name = if actor.display.trim().is_empty() {
        &actor.login
    } else {
        &actor.display
    };
    let name = sanitize(name);
    if actor.kind == ActorKind::Bot && !name.contains("[bot]") {
        format!("{name} ({})", m::BOT.get())
    } else {
        name
    }
}

fn verdict(state: ReviewState) -> (&'static str, ratatui::style::Color) {
    match state {
        ReviewState::Approved => (m::REVIEW_APPROVED.get(), th().status.success),
        ReviewState::ChangesRequested => (m::REVIEW_CHANGES.get(), th().status.danger),
        ReviewState::Commented => (m::REVIEW_COMMENTED.get(), th().ui.muted),
        ReviewState::Dismissed => (m::REVIEW_DISMISSED.get(), th().ui.muted),
    }
}

/// How wide `text` paints (the buttons' room check).
pub(super) fn width(text: &str) -> usize {
    display_width(text)
}
