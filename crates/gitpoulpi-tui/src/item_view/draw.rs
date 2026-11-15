//! Painting the view, and the hit rects of that same walk (maintainer rule
//! 6): `render` clears the hit list and each part pushes the rects it draws.
//! The timeline paints through the shared widget and keeps its own list.

use gitpoulpi_hosting::conversation::{
    IdentityKind, MarkupFlavor, PostingIdentity, TimelineEntry, TokenSource,
};
use gitpoulpi_i18n::msg::{hosting_ui as hosting, item_view as m};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use super::{Bar, Compose, Focus, Hit, ItemView};
use crate::dialog_kit::{self, ButtonState, CheckState};
use crate::md_render::sanitize;
use crate::th;
use crate::ui_kit::truncate_with_ellipsis;

/// The view is not drawn smaller than this.
const MIN_W: u16 = 40;
const MIN_H: u16 = 10;

fn panel() -> Style {
    Style::default().fg(th().ui.text).bg(th().ui.panel)
}

fn muted() -> Style {
    Style::default().fg(th().ui.muted).bg(th().ui.panel)
}

/// The host's markup, as the composer footer names it. Product names are
/// not translated.
fn flavor_name(flavor: MarkupFlavor) -> &'static str {
    match flavor {
        MarkupFlavor::Gfm => "Markdown (GitHub)", // i18n-english
        MarkupFlavor::GitlabMd => "Markdown (GitLab)", // i18n-english
        MarkupFlavor::GiteaMd => "Markdown (Gitea)", // i18n-english
        MarkupFlavor::AzureMd => "Markdown (Azure DevOps)", // i18n-english
        MarkupFlavor::BitbucketMd => "Markdown (Bitbucket)", // i18n-english
        MarkupFlavor::YoutrackMd => "Markdown (YouTrack)", // i18n-english
        MarkupFlavor::TrelloMd => "Markdown (Trello)", // i18n-english
        MarkupFlavor::ObsidianMd => "Markdown (Obsidian)", // i18n-english
        MarkupFlavor::CommonMark => "Markdown",   // i18n-english
        MarkupFlavor::JiraWiki => "Jira wiki markup", // i18n-english
        MarkupFlavor::Textile => "Textile",       // i18n-english
        MarkupFlavor::Html => "HTML",             // i18n-english
        MarkupFlavor::PlainText => "Plain text",  // i18n-english
    }
}

/// "Posting as ..." for `id`: who a post would be made as, always shown.
pub(crate) fn identity_line(id: &PostingIdentity) -> String {
    let name = sanitize(&id.display);
    match id.kind {
        IdentityKind::LocalAuthor => return m::writing_as_local(name),
        IdentityKind::AppBot if id.login.is_none() => {
            return m::POSTING_APP_BOT_UNKNOWN.get().to_string();
        }
        IdentityKind::AppBot => return m::posting_as_app_bot(name),
        IdentityKind::ProjectOrGroupBot if !name.is_empty() => {
            return m::posting_as_project_bot(name);
        }
        _ => {}
    }
    if name.is_empty() {
        return m::POSTING_UNKNOWN.get().to_string();
    }
    match id.source {
        TokenSource::RepoTokenCommand => m::posting_as_command(name),
        TokenSource::Profile { .. } => m::posting_as_profile(name),
    }
}

impl ItemView {
    /// Paints the whole view into `area`.
    pub(crate) fn render(&mut self, area: Rect, buf: &mut Buffer) {
        self.hits.clear();
        let r = dialog_kit::centered(
            area,
            area.width.saturating_sub(2),
            area.height.saturating_sub(2),
        );
        self.frame = r;
        let title = self.title(r.width);
        self.guard.close.panel(buf, r, &title);
        if r.width < MIN_W || r.height < MIN_H {
            dialog_kit::put(
                buf,
                r.x + 2,
                r.y + 1,
                m::TOO_SMALL.get(),
                r.width.saturating_sub(4),
                muted(),
            );
            self.guard.render(buf);
            return;
        }
        let inner = Rect::new(r.x + 2, r.y + 1, r.width - 4, r.height - 2);
        self.info_line(buf, inner);
        self.bar_line(buf, Rect::new(inner.x, inner.y + 1, inner.width, 1));
        self.identity_row(buf, Rect::new(inner.x, inner.y + 2, inner.width, 1));
        let status_y = inner.bottom() - 1;
        let body_top = inner.y + 3;
        // The composer docks under the timeline when there is room for both.
        let dock_h = self.dock_height(inner.height);
        let dock_top = status_y.saturating_sub(dock_h);
        let timeline = Rect::new(
            r.x + 1,
            body_top,
            r.width - 2,
            dock_top.saturating_sub(body_top),
        );
        let focused = self.focus == Focus::Timeline;
        self.conv.paint(buf, timeline, focused);
        if dock_h > 0 {
            self.dock(buf, Rect::new(inner.x, dock_top, inner.width, dock_h));
        } else {
            self.composer_rect = Rect::default();
            if let Some(md) = self.md.as_mut() {
                // Not drawn: it must not catch clicks from its last place.
                md.render(buf, Rect::default(), false, "");
            }
        }
        self.footer(buf, inner.x, status_y, inner.width);
        // The popups, last, over everything.
        if let Some(menu) = self.menu.as_mut() {
            menu.combo.render_popup(buf, area);
        }
        if let Some((confirm, _)) = self.delete_ask.as_mut() {
            confirm.render(buf, area);
        }
        self.guard.render(buf);
    }

    fn title(&self, width: u16) -> String {
        let (reference, title) = match &self.conv.header {
            Some(h) => (h.reference.clone(), h.title.clone()),
            None => (self.target.reference.clone(), self.target.title.clone()),
        };
        let text = format!("{}  {}", sanitize(&reference), sanitize(&title));
        truncate_with_ellipsis(text.trim(), usize::from(width.saturating_sub(12)))
    }

    /// How many rows the composer takes (0: none, not enough room).
    fn dock_height(&self, inner_h: u16) -> u16 {
        if !self.composer_open || !self.composer_available() || self.md.is_none() {
            return 0;
        }
        if inner_h < 16 {
            // A small window shows the composer only while it has the keyboard.
            return if self.focus == Focus::Composer && inner_h >= 11 {
                (inner_h / 2).min(8)
            } else {
                0
            };
        }
        (inner_h / 3).clamp(8, 11)
    }

    // -- the header ------------------------------------------------------------------

    fn info_line(&self, buf: &mut Buffer, inner: Rect) {
        let Some(h) = &self.conv.header else {
            dialog_kit::put(buf, inner.x, inner.y, "", inner.width, muted());
            return;
        };
        let (word, color) = if h.open {
            (hosting::STATE_OPEN.get().to_string(), th().refs.local)
        } else if h.state_label == "merged" {
            (hosting::STATE_MERGED.get().to_string(), th().hosting.merged)
        } else if h.state_label == "closed" {
            (hosting::STATE_CLOSED.get().to_string(), th().status.danger)
        } else {
            (sanitize(&h.state_label), th().ui.muted)
        };
        let mut x = inner.x;
        let chip = dialog_kit::put(
            buf,
            x,
            inner.y,
            &format!("\u{25CF} {word}"),
            inner.width,
            Style::default()
                .fg(color)
                .bg(th().ui.panel)
                .add_modifier(Modifier::BOLD),
        );
        x = chip.right() + 2;
        let comments = self
            .conv
            .entries
            .iter()
            .filter(|e| matches!(e, TimelineEntry::Comment(_)))
            .count() as u64;
        let mut text = m::opened_by(
            sanitize(&h.author.display),
            crate::conversation_ui::rows::short_time(&h.created_at),
        );
        if !h.labels.is_empty() {
            text.push_str("   ");
            text.push_str(&sanitize(&h.labels.join("  ")));
        }
        text.push_str("   ");
        text.push_str(&m::comments_n(comments));
        let used = dialog_kit::put(
            buf,
            x,
            inner.y,
            &text,
            inner.right().saturating_sub(x),
            panel(),
        );
        // A refresh in flight.
        if self.conv.spinning() {
            let label = format!("{} {}", self.conv.spinner(), m::REFRESHING.get());
            let w = crate::ui_kit::display_width(&label) as u16;
            if used.right() + 2 + w <= inner.right() {
                dialog_kit::put(
                    buf,
                    inner.right() - w,
                    inner.y,
                    &label,
                    w,
                    Style::default().fg(th().ui.accent).bg(th().ui.panel),
                );
            }
        }
    }

    fn bar_line(&mut self, buf: &mut Buffer, row: Rect) {
        let mut x = row.x;
        let focused = |b: Bar| self.focus == Focus::Bar(b);
        let hovered = |b: Bar| self.hover == Some(Hit::Bar(b));
        let mut hits: Vec<(Rect, Hit)> = Vec::new();
        for bar in Bar::ALL {
            let state = ButtonState::Normal {
                hovered: hovered(bar),
                focused: focused(bar),
            };
            match bar {
                Bar::Back => {
                    let state = if self.stack.is_empty() {
                        ButtonState::Disabled
                    } else {
                        state
                    };
                    let r = dialog_kit::button(buf, x, row.y, m::BTN_BACK.get(), state);
                    hits.push((r, Hit::Bar(bar)));
                    x = r.right() + 1;
                }
                Bar::Refresh => {
                    let r = dialog_kit::button(buf, x, row.y, m::BTN_REFRESH.get(), state);
                    hits.push((r, Hit::Bar(bar)));
                    x = r.right() + 1;
                }
                Bar::Browser => {
                    let r = dialog_kit::button(buf, x, row.y, m::BTN_BROWSER.get(), state);
                    hits.push((r, Hit::Bar(bar)));
                    x = r.right() + 2;
                }
                Bar::Events => {
                    if self.conv.caps.as_ref().is_some_and(|c| c.events) {
                        let r = dialog_kit::checkbox(
                            buf,
                            x,
                            row.y,
                            row.right().saturating_sub(x),
                            m::SHOW_EVENTS.get(),
                            self.conv.events,
                            CheckState::Enabled {
                                hovered: hovered(bar),
                                focused: focused(bar),
                            },
                        );
                        crate::pointer_map::hand(r);
                        hits.push((r, Hit::Bar(bar)));
                    }
                }
            }
        }
        self.hits.extend(hits);
    }

    fn identity_row(&self, buf: &mut Buffer, row: Rect) {
        let text = match (&self.conv.caps, &self.conv.identity) {
            (Some(c), _) if !c.comment => m::READ_ONLY_NOTE.get().to_string(),
            (_, Some(Ok(id))) => identity_line(id),
            (_, Some(Err(_))) => m::POSTING_UNKNOWN.get().to_string(),
            _ => String::new(),
        };
        dialog_kit::put(buf, row.x, row.y, &text, row.width, muted());
    }

    // -- the composer dock -------------------------------------------------------------

    fn dock(&mut self, buf: &mut Buffer, rect: Rect) {
        // The rule with the title.
        let line: String = "\u{2500}".repeat(usize::from(rect.width));
        dialog_kit::put(
            buf,
            rect.x,
            rect.y,
            &line,
            rect.width,
            Style::default().fg(th().ui.border).bg(th().ui.panel),
        );
        let title = match &self.compose {
            Compose::New => m::NEW_COMMENT.get().to_string(),
            Compose::Reply { label, .. } => m::composer_reply_title(sanitize(label)),
            Compose::Edit { .. } => m::COMPOSER_EDIT_TITLE.get().to_string(),
        };
        let sending = self.conv.is_writing();
        let title = if sending {
            format!(" {title} - {} ", m::SENDING.get())
        } else {
            format!(" {title} ")
        };
        dialog_kit::put(
            buf,
            rect.x + 1,
            rect.y,
            &title,
            rect.width.saturating_sub(2),
            Style::default()
                .fg(th().ui.text)
                .bg(th().ui.panel)
                .add_modifier(Modifier::BOLD),
        );
        // A refusal sits under the text, in its own row.
        let error_row = self.compose_error.is_some() && rect.height >= 5;
        let editor = Rect::new(
            rect.x,
            rect.y + 1,
            rect.width,
            rect.height.saturating_sub(if error_row { 3 } else { 2 }),
        );
        if let (true, Some(error)) = (error_row, &self.compose_error) {
            dialog_kit::put(
                buf,
                rect.x + 1,
                rect.bottom() - 2,
                &sanitize(error),
                rect.width.saturating_sub(2),
                Style::default().fg(th().status.danger).bg(th().ui.panel),
            );
        }
        self.composer_rect = editor;
        let focused = self.focus == Focus::Composer && !self.guard.is_open();
        let write_flavor = self.conv.caps.as_ref().map(|c| c.write_flavor);
        let send_reason = self.send_reason();
        if let Some(md) = self.md.as_mut() {
            md.set_sent_as(match write_flavor {
                Some(f @ (MarkupFlavor::JiraWiki | MarkupFlavor::Textile)) => {
                    Some(flavor_name(f).to_string())
                }
                _ => None,
            });
            md.render(buf, editor, focused, m::COMPOSER_PLACEHOLDER.get());
        }
        // The footer: the markup it is written in, and the buttons.
        let y = rect.bottom() - 1;
        let mut left = rect.x + 1;
        if let Some(f) = write_flavor {
            let used = dialog_kit::put(buf, left, y, flavor_name(f), rect.width / 3, muted());
            left = used.right() + 2;
        }
        let hovered = |h: Hit| self.hover == Some(h);
        let mut internal_hit = None;
        if self.conv.caps.as_ref().is_some_and(|c| c.internal_notes) && self.compose == Compose::New
        {
            let label_w = crate::ui_kit::display_width(m::INTERNAL_NOTE.get()) as u16 + 4;
            if left + label_w + 40 < rect.right() {
                let r = dialog_kit::checkbox(
                    buf,
                    left,
                    y,
                    label_w,
                    m::INTERNAL_NOTE.get(),
                    self.internal,
                    CheckState::Enabled {
                        hovered: hovered(Hit::Internal),
                        focused: false,
                    },
                );
                crate::pointer_map::hand(r);
                internal_hit = Some(r);
            }
        }
        let send_label = match &self.compose {
            Compose::New => m::BTN_SEND.get(),
            Compose::Reply { .. } => gitpoulpi_i18n::msg::conversation_ui::BTN_REPLY.get(),
            Compose::Edit { .. } => m::BTN_SAVE.get(),
        };
        let cancel_label = m::BTN_CANCEL.get();
        let send_w = dialog_kit::button_width(send_label);
        let cancel_w = dialog_kit::button_width(cancel_label);
        let key = dialog_kit::key_label(crate::keymap::Action::MdEditorSubmit);
        let key_w = crate::ui_kit::display_width(&key) as u16;
        let mut right = rect.right();
        let send_x = right.saturating_sub(send_w);
        let send = dialog_kit::button(
            buf,
            send_x,
            y,
            send_label,
            if send_reason.is_some() {
                ButtonState::Disabled
            } else {
                ButtonState::Primary {
                    hovered: hovered(Hit::Send),
                    focused: false,
                }
            },
        );
        right = send.x.saturating_sub(1);
        let cancel_x = right.saturating_sub(cancel_w);
        let cancel = dialog_kit::button(
            buf,
            cancel_x,
            y,
            cancel_label,
            ButtonState::Normal {
                hovered: hovered(Hit::Cancel),
                focused: false,
            },
        );
        if cancel.x > rect.x + key_w + 4 {
            dialog_kit::put(
                buf,
                cancel.x - key_w - 2,
                y,
                &key,
                key_w,
                Style::default().fg(th().ui.accent).bg(th().ui.panel),
            );
        }
        self.hits.push((send, Hit::Send));
        self.hits.push((cancel, Hit::Cancel));
        if let Some(r) = internal_hit {
            self.hits.push((r, Hit::Internal));
        }
    }

    // -- the last row -------------------------------------------------------------------

    fn footer(&self, buf: &mut Buffer, x: u16, y: u16, width: u16) {
        if let Some(message) = &self.message {
            dialog_kit::put(
                buf,
                x,
                y,
                &sanitize(message),
                width,
                Style::default().fg(th().status.warning).bg(th().ui.panel),
            );
            return;
        }
        let back = dialog_kit::alt_key(crate::keymap::chord::Key::Left);
        dialog_kit::hints(
            buf,
            x,
            y,
            width,
            &[
                ("\u{2191}\u{2193}", m::HINT_MOVE.get()),
                ("r", m::HINT_REPLY.get()),
                ("c", m::HINT_COMMENT.get()),
                ("l", m::HINT_LINKS.get()),
                ("n", m::HINT_UNREAD.get()),
                ("o", m::HINT_BROWSER.get()),
                (back.as_str(), m::HINT_BACK.get()),
                ("Esc", m::HINT_CLOSE.get()),
            ],
        );
    }

    /// What `Ctrl+N` badges: every control of the last paint.
    pub(crate) fn nav_targets(&self) -> Vec<(Rect, (u16, u16))> {
        let mut rects: Vec<Rect> = self.hits.iter().map(|(r, _)| *r).collect();
        if let Some(r) = self.guard.close.rect() {
            rects.push(r);
        }
        rects.extend(
            self.conv
                .hits
                .iter()
                .map(|(r, _)| *r)
                .filter(|r| r.width < self.conv.area.width.saturating_sub(2)),
        );
        if self.composer_rect.width > 0 {
            rects.push(self.composer_rect);
        }
        dialog_kit::nav_cells(rects)
    }
}
