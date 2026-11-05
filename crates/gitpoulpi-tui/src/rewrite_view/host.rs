//! S23 in the workbench: the branches the host protects, and the three ways
//! through when one of them blocks the rewrite.
//!
//! The host is read by the window (hosting is client side and only a linked
//! repository is asked): it hands the view a [`HostHandle`], the answer of the
//! lookup arrives later on its channel. What it says reaches the daemon in
//! the recipe's `scope_options.host_protection` (see `request_recipe`), where
//! its patterns count as protected names: the Refs tab lists those branches
//! with the same checkbox and all/none toggle, labelled "protected on
//! GitHub". When one blocks (a refused preview, a push the host rejected) the
//! "How to get through" button opens the overlay that explains the three
//! ways with the exact settings page of the provider. Instructions only:
//! nothing here changes anything on the host.

use std::sync::mpsc;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use gitpoulpi_core::domain::rewrite::{HostProtection, HowStep, Loss, PushRefStatus, Way};
use gitpoulpi_i18n::{msg::rewrite_host as h, M};

use super::bars::BarId;
use super::render::{bold, dim, plain, row_bg, wrap, RewriteHit};
use super::{Notice, RewriteView};
use crate::dialog_kit::{self, ButtonState};
use crate::th;

/// What the window learned about the repository's host.
pub(crate) struct HostHandle {
    /// The host and its settings pages (nothing asked yet).
    pub base: HostProtection,
    /// The answer of the lookup, when the repository is linked.
    pub answer: Option<mpsc::Receiver<HostProtection>>,
}

/// The host side of the workbench.
pub(crate) struct HostState {
    /// The window has not yet handed the host over.
    wanted: bool,
    pub info: Option<HostProtection>,
    answer: Option<mpsc::Receiver<HostProtection>>,
    /// The last preview was refused because every branch in scope is
    /// protected: the how-to is the way out.
    pub refused: bool,
    /// The how-to overlay, when open.
    pub howto: Option<Howto>,
    /// Opens a link in the browser (replaced in tests).
    pub opener: fn(&str) -> Result<(), String>,
}

impl Default for HostState {
    fn default() -> Self {
        Self {
            wanted: true,
            info: None,
            answer: None,
            refused: false,
            howto: None,
            opener: crate::repo_menu::open_url,
        }
    }
}

/// The open overlay: how far the text is scrolled and which link is picked.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Howto {
    pub scroll: usize,
    pub link: usize,
}

/// One line of the overlay.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Line {
    Heading(String),
    Text(String),
    Dim(String),
    /// Link `.0` of the flat list.
    Link(usize, String),
    Gap,
}

fn way_label(way: Way) -> M {
    match way {
        Way::LiftProtection => h::WAY_LIFT,
        Way::NewBranchName => h::WAY_RENAME,
        Way::Recreate => h::WAY_RECREATE,
    }
}

fn step_label(step: HowStep) -> M {
    match step {
        HowStep::LiftGithub => h::STEP_LIFT_GITHUB,
        HowStep::LiftGitlab => h::STEP_LIFT_GITLAB,
        HowStep::LiftAzure => h::STEP_LIFT_AZURE,
        HowStep::LiftBitbucket => h::STEP_LIFT_BITBUCKET,
        HowStep::LiftOther => h::STEP_LIFT_OTHER,
        HowStep::PushNewName => h::STEP_PUSH_NEW_NAME,
        HowStep::SwitchGithub => h::STEP_SWITCH_GITHUB,
        HowStep::SwitchGitlab => h::STEP_SWITCH_GITLAB,
        HowStep::SwitchAzure => h::STEP_SWITCH_AZURE,
        HowStep::SwitchBitbucket => h::STEP_SWITCH_BITBUCKET,
        HowStep::SwitchOther => h::STEP_SWITCH_OTHER,
        HowStep::Recreate => h::STEP_RECREATE,
    }
}

fn loss_label(loss: Loss) -> M {
    match loss {
        Loss::PullRequests => h::LOSS_PULL_REQUESTS,
        Loss::Issues => h::LOSS_ISSUES,
        Loss::Forks => h::LOSS_FORKS,
        Loss::Stars => h::LOSS_STARS,
        Loss::CiSettings => h::LOSS_CI,
        Loss::Webhooks => h::LOSS_WEBHOOKS,
    }
}

impl RewriteView {
    // ------------------------------------------------------------ the host

    /// The window has not handed the host over yet.
    pub(crate) fn wants_host(&self) -> bool {
        self.host.wanted
    }

    /// The window's answer: the host (`None`: none can be named) and, for a
    /// linked repository, the lookup on its way.
    pub(crate) fn set_host(&mut self, handle: Option<HostHandle>) {
        self.host.wanted = false;
        if let Some(handle) = handle {
            self.host.info = Some(handle.base);
            self.host.answer = handle.answer;
        }
    }

    /// Takes the lookup's answer when it has come. A change of what is
    /// protected makes the plan on screen stale.
    pub(super) fn poll_host(&mut self) {
        let Some(rx) = &self.host.answer else {
            return;
        };
        match rx.try_recv() {
            Ok(info) => {
                self.host.answer = None;
                self.host.info = Some(info);
                self.touch();
            }
            Err(mpsc::TryRecvError::Disconnected) => self.host.answer = None,
            Err(mpsc::TryRecvError::Empty) => {}
        }
    }

    /// What the host protects, once it was asked.
    pub(crate) fn host(&self) -> Option<&HostProtection> {
        self.host.info.as_ref()
    }

    /// "protected on GitHub" when the host protects `name` (short or full).
    pub(crate) fn host_label(&self, name: &str) -> Option<String> {
        let host = self.host()?;
        host.covers(name).then(|| h::protected_on(host.provider()))
    }

    /// The branches the host protects that stand in the way: the ones of the
    /// preview's protected list, the ones a push was refused for, else
    /// whatever the host named.
    fn howto_branches(&self) -> Vec<String> {
        let Some(host) = self.host() else {
            return Vec::new();
        };
        let short = |n: &str| n.strip_prefix("refs/heads/").unwrap_or(n).to_string();
        let mut out: Vec<String> = self
            .protected()
            .iter()
            .filter(|e| host.covers(&e.name))
            .map(|e| short(&e.name))
            .collect();
        if let Some(sent) = &self.push.sent {
            for r in sent
                .results
                .iter()
                .filter(|r| r.status == PushRefStatus::RejectedProtectedByHost)
            {
                let name = short(&r.name);
                if !out.contains(&name) {
                    out.push(name);
                }
            }
        }
        out
    }

    /// Whether the "How to get through" button is offered: a branch the host
    /// protects is excluded, a push was refused by the host, or the preview
    /// was refused because every branch is protected.
    pub(crate) fn howto_offered(&self) -> bool {
        let Some(host) = self.host() else {
            return false;
        };
        (self.host.refused && !host.patterns.is_empty()) || !self.howto_branches().is_empty()
    }

    // ------------------------------------------------------------ the how-to

    pub(crate) fn open_howto(&mut self) {
        if self.host().is_some() {
            self.host.howto = Some(Howto::default());
        }
    }

    pub(crate) fn close_howto(&mut self) {
        self.host.howto = None;
    }

    /// The links of the overlay, in the order they are listed.
    pub(crate) fn howto_links(&self) -> Vec<String> {
        let Some(host) = self.host() else {
            return Vec::new();
        };
        host.howto(&[])
            .ways
            .into_iter()
            .flat_map(|w| w.links)
            .collect()
    }

    fn howto_lines(&self, width: usize) -> Vec<Line> {
        let Some(host) = self.host() else {
            return Vec::new();
        };
        let mut branches = self.howto_branches();
        if branches.is_empty() {
            branches = host.patterns.clone();
        }
        let names = if branches.is_empty() {
            h::ANY_BRANCH.get().to_string()
        } else {
            branches.join(", ")
        };
        let howto = host.howto(&branches);
        let mut lines: Vec<Line> = wrap(&h::intro(&names, host.provider()), width)
            .into_iter()
            .map(Line::Text)
            .collect();
        let note = match (&host.unknown, host.asked) {
            (Some(why), true) => Some(h::unknown(why)),
            (None, false) => Some(h::NOT_ASKED.get().to_string()),
            _ => None,
        };
        if let Some(note) = note {
            lines.extend(wrap(&note, width).into_iter().map(Line::Dim));
        }
        let mut link = 0;
        for way in &howto.ways {
            lines.push(Line::Gap);
            lines.push(Line::Heading(way_label(way.way).get().to_string()));
            for step in &way.steps {
                lines.extend(
                    wrap(step_label(*step).get(), width.saturating_sub(2))
                        .into_iter()
                        .map(|l| Line::Text(format!("  {l}"))),
                );
            }
            for url in &way.links {
                lines.push(Line::Link(link, url.clone()));
                link += 1;
            }
            if way.way == Way::Recreate {
                let losses: Vec<&str> = howto.losses.iter().map(|l| loss_label(*l).get()).collect();
                let text = format!("{}: {}.", h::LOSES.get(), losses.join("; "));
                lines.extend(
                    wrap(&text, width.saturating_sub(2))
                        .into_iter()
                        .map(|l| Line::Dim(format!("  {l}"))),
                );
            }
        }
        lines
    }

    fn picked_link(&self) -> Option<String> {
        let at = self.host.howto?.link;
        self.howto_links().get(at).cloned()
    }

    pub(crate) fn howto_copy(&mut self) {
        let Some(url) = self.picked_link() else {
            return;
        };
        let said = crate::clipboard::copy_shared(h::WHAT_LINK.get(), &url);
        self.notice = Some(Notice {
            text: said,
            error: false,
        });
    }

    pub(crate) fn howto_open(&mut self) {
        let Some(url) = self.picked_link() else {
            return;
        };
        self.notice = Some(match (self.host.opener)(&url) {
            Ok(()) => Notice {
                text: h::OPENED.get().to_string(),
                error: false,
            },
            Err(why) => Notice {
                text: why,
                error: true,
            },
        });
    }

    pub(crate) fn howto_pick(&mut self, link: usize) {
        let count = self.howto_links().len();
        if let Some(open) = &mut self.host.howto {
            open.link = link.min(count.saturating_sub(1));
        }
    }

    /// A key while the overlay is open.
    pub(super) fn on_howto_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;
        let Some(open) = self.host.howto else {
            return;
        };
        let links = self.howto_links().len();
        match key.code {
            KeyCode::Esc => self.close_howto(),
            KeyCode::Up => {
                if let Some(o) = &mut self.host.howto {
                    o.scroll = open.scroll.saturating_sub(1);
                }
            }
            KeyCode::Down => {
                if let Some(o) = &mut self.host.howto {
                    o.scroll = open.scroll + 1;
                }
            }
            KeyCode::PageUp => {
                if let Some(o) = &mut self.host.howto {
                    o.scroll = open.scroll.saturating_sub(8);
                }
            }
            KeyCode::PageDown => {
                if let Some(o) = &mut self.host.howto {
                    o.scroll = open.scroll + 8;
                }
            }
            KeyCode::Left => self.howto_pick(open.link.saturating_sub(1)),
            KeyCode::Right | KeyCode::Tab => {
                self.howto_pick((open.link + 1).min(links.saturating_sub(1)))
            }
            KeyCode::Char('c') | KeyCode::Char('C') | KeyCode::Enter => self.howto_copy(),
            KeyCode::Char('o') | KeyCode::Char('O') => self.howto_open(),
            _ => {}
        }
    }

    /// The overlay's buttons and links, as the pointer reaches them.
    pub(super) fn on_howto_press(&mut self, hit: RewriteHit) {
        match hit {
            RewriteHit::HostClose | RewriteHit::HostShield => self.close_howto(),
            RewriteHit::HostLink(i) => {
                self.howto_pick(i);
                self.howto_copy();
            }
            RewriteHit::HostCopy => self.howto_copy(),
            RewriteHit::HostOpen => self.howto_open(),
            _ => {}
        }
    }

    // ------------------------------------------------------------- painting

    /// The Refs tab's dim line when the host's protection could not be read.
    pub(super) fn host_unknown_line(&self) -> Option<String> {
        let host = self.host()?;
        match (&host.unknown, host.asked) {
            (Some(why), true) => Some(h::refs_unknown(why)),
            _ => None,
        }
    }

    pub(super) fn paint_howto(
        &self,
        buf: &mut Buffer,
        bounds: Rect,
        hovered: Option<RewriteHit>,
    ) -> Vec<(Rect, RewriteHit)> {
        let mut hits = Vec::new();
        let Some(open) = self.host.howto else {
            return hits;
        };
        let rect = Rect::new(
            bounds.x + 2,
            bounds.y + 1,
            bounds.width.saturating_sub(4),
            bounds.height.saturating_sub(2),
        );
        if rect.width < 40 || rect.height < 8 {
            hits.push((bounds, RewriteHit::HostShield));
            return hits;
        }
        dialog_kit::panel(buf, rect, h::TITLE.get());
        if let Some(cross) =
            crate::chrome::close_cross::paint(buf, rect, hovered == Some(RewriteHit::HostClose))
        {
            hits.push((cross, RewriteHit::HostClose));
        }
        let inner = Rect::new(rect.x + 2, rect.y + 1, rect.width - 4, rect.height - 2);
        let body = Rect::new(
            inner.x,
            inner.y,
            inner.width,
            inner.height.saturating_sub(3),
        );
        let lines = self.howto_lines(body.width as usize);
        let room = body.height as usize;
        // Keep the picked link in view, then the scroll inside the text.
        let picked_row = lines
            .iter()
            .position(|l| matches!(l, Line::Link(i, _) if *i == open.link));
        let mut top = open.scroll.min(lines.len().saturating_sub(room));
        if self.bar_free(BarId::Howto, Some(open.link)) {
            // The bar or the wheel put the text here, and the picked link
            // has not moved since.
        } else if let Some(row) = picked_row {
            if row < top {
                top = row;
            } else if row >= top + room {
                top = row + 1 - room;
            }
        }
        self.paint_bar_on(
            buf,
            &mut hits,
            hovered == Some(RewriteHit::Bar(BarId::Howto)),
            BarId::Howto,
            (rect.right() - 1, body.y, body.height),
            lines.len(),
            room,
            top,
            Some(open.link),
        );
        for (slot, line) in lines.iter().skip(top).take(room).enumerate() {
            let y = body.y + slot as u16;
            match line {
                Line::Gap => {}
                Line::Heading(text) => {
                    dialog_kit::put(buf, body.x, y, text, body.width, bold(false));
                }
                Line::Text(text) => {
                    dialog_kit::put(buf, body.x, y, text, body.width, plain());
                }
                Line::Dim(text) => {
                    dialog_kit::put(buf, body.x, y, text, body.width, dim());
                }
                Line::Link(i, url) => {
                    let hit = RewriteHit::HostLink(*i);
                    let selected = *i == open.link;
                    let bg = row_bg(selected, hovered == Some(hit));
                    let band = Rect::new(body.x, y, body.width, 1);
                    dialog_kit::fill(buf, band, Style::default().bg(bg));
                    let label = format!("  {}: {url}", h::LINK_LABEL.get());
                    dialog_kit::put(
                        buf,
                        band.x,
                        y,
                        &label,
                        band.width,
                        Style::default().fg(th().ui.text).bg(bg),
                    );
                    hits.push((band, hit));
                }
            }
        }
        // The buttons and the key hint.
        let by = inner.bottom().saturating_sub(2);
        let mut x = inner.x;
        for (label, hit) in [
            (h::BTN_COPY.get(), RewriteHit::HostCopy),
            (h::BTN_OPEN.get(), RewriteHit::HostOpen),
            (h::BTN_CLOSE.get(), RewriteHit::HostClose),
        ] {
            let state = ButtonState::Normal {
                hovered: hovered == Some(hit),
                focused: false,
            };
            let r = dialog_kit::button(buf, x, by, label, state);
            hits.push((r, hit));
            x += r.width + 1;
        }
        dialog_kit::put(
            buf,
            inner.x,
            inner.bottom().saturating_sub(1),
            h::KEYS_HINT.get(),
            inner.width,
            dim(),
        );
        hits.push((rect, RewriteHit::HostPanel));
        hits.push((bounds, RewriteHit::HostShield));
        hits
    }
}
