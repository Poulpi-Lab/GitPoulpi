//! The Identities tab (slice S16): every name and email of the scope with
//! its commit counts, which of them the repository's `.mailmap` already
//! maps, and "Merge into..." that appends the lines turning several of them
//! into one canonical identity.
//!
//! Nothing is rewritten and nothing is committed: the daemon edits the
//! `.mailmap` as one journaled step (`MailmapAdd`) that Undo takes back, and
//! the Log shows the canonical name at once (`log.use_mailmap`). The tab is
//! also where a rewrite that would touch commits already pushed offers to
//! "Use .mailmap instead" (`paint_mailmap_offer`), with the matching
//! identities already ticked.

use std::cell::Cell;
use std::collections::HashSet;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use gitpoulpi_core::domain::rewrite::{
    CompiledMatch, Ident, IdentityList, MailmapAdded, PushState, RewritePlan, RuleKind, Scope,
};
use gitpoulpi_core::domain::OperationKind;
use gitpoulpi_i18n::msg::{rewrite_identities as m, rewrite_view as t};
use gitpoulpi_ipc::mailmap::{AddArgs, IdentitiesArgs};

use super::bars::BarId;
use super::head;
use super::render::{bad, bold, dim, plain, row_bg, warn, wrap, RewriteHit, Ui};
use super::{Focus, Notice, PreviewTab, RewriteKeyOutcome, RewriteView};
use crate::daemon_link::Reply;
use crate::dialog_kit;
use crate::th;

/// The dialog that picks which identity the others become.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MergePick {
    /// The ticked identities, then the ones a rule offered, in that order.
    pub candidates: Vec<Candidate>,
    pub cursor: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Candidate {
    pub ident: Ident,
    /// Commits it authored (0 for an identity only a rule names).
    pub commits: u64,
}

pub(crate) struct Loaded {
    pub list: IdentityList,
    /// The scope the list was read for, to tell when the recipe's moved on.
    pub scope: Scope,
}

#[derive(Default)]
pub(crate) struct IdentitiesState {
    pub(super) load: Option<(Reply<IdentityList>, Scope)>,
    pub loaded: Option<Loaded>,
    pub cursor: usize,
    pub ticked: HashSet<Ident>,
    /// Identities an identity rule would set, offered as the canonical one
    /// even though no commit carries them yet.
    pub extras: Vec<Ident>,
    /// Ticks to make when the list arrives (the rule's matches).
    select_rules: Vec<CompiledMatch>,
    pub pick: Option<MergePick>,
    pub(super) saving: Option<Reply<MailmapAdded>>,
    /// What the last merge wrote (Undo is offered while it is the latest).
    pub done: Option<MailmapAdded>,
    pub error: Option<String>,
    /// First row shown.
    pub(super) top: Cell<usize>,
}

fn show(i: &Ident) -> String {
    if i.name.is_empty() {
        format!("<{}>", i.email)
    } else {
        format!("{} <{}>", i.name, i.email)
    }
}

impl RewriteView {
    // ------------------------------------------------------------- queries

    pub(crate) fn identities_loading(&self) -> bool {
        self.identities.load.is_some()
    }

    fn identity_rows(&self) -> &[gitpoulpi_core::domain::rewrite::IdentityCount] {
        self.identities
            .loaded
            .as_ref()
            .map_or(&[][..], |l| &l.list.identities)
    }

    pub(crate) fn ticked_identities(&self) -> usize {
        self.identity_rows()
            .iter()
            .filter(|r| self.identities.ticked.contains(&r.ident()))
            .count()
    }

    /// "Merge into..." has something to merge: two identities to choose
    /// between, one of them ticked.
    pub(crate) fn can_merge(&self) -> bool {
        let ticked = self.ticked_identities();
        ticked >= 1
            && ticked + self.identities.extras.len() >= 2
            && !self.busy()
            && self.identities.saving.is_none()
            && self.identities.load.is_none()
    }

    // -------------------------------------------------------------- daemon

    /// Asks the daemon for the identities of the recipe's scope.
    pub(crate) fn load_identities(&mut self) {
        if self.identities.load.is_some() {
            return;
        }
        self.identities.error = None;
        let scope = self.recipe.scope.clone();
        let args = IdentitiesArgs {
            scope: scope.clone(),
            options: self.recipe.scope_options.clone(),
        };
        let reply = self.requester.call(
            OperationKind::MailmapIdentities,
            serde_json::to_value(args).unwrap_or_default(),
        );
        self.identities.load = Some((reply, scope));
    }

    /// Collects the answers and reads the list when the tab is on screen.
    /// Never waits.
    pub(super) fn poll_identities(&mut self) {
        if let Some((reply, _)) = &self.identities.load {
            if let Some(answer) = reply.take() {
                let scope = self
                    .identities
                    .load
                    .take()
                    .map(|(_, s)| s)
                    .unwrap_or_default();
                match answer {
                    Ok(list) => {
                        let rows = &list.identities;
                        self.identities.cursor =
                            self.identities.cursor.min(rows.len().saturating_sub(1));
                        // A tick for an identity the list no longer has goes.
                        let known: HashSet<Ident> = rows.iter().map(|r| r.ident()).collect();
                        self.identities.ticked.retain(|i| known.contains(i));
                        let rules = std::mem::take(&mut self.identities.select_rules);
                        if !rules.is_empty() {
                            for row in rows {
                                if rules.iter().any(|r| r.matches(&row.ident())) {
                                    self.identities.ticked.insert(row.ident());
                                }
                            }
                        }
                        self.identities.loaded = Some(Loaded { list, scope });
                    }
                    Err(error) => {
                        self.identities.error =
                            Some(crate::daemon_link::error_text::error_text(&error));
                    }
                }
            }
        }
        if let Some(reply) = &self.identities.saving {
            if let Some(answer) = reply.take() {
                self.identities.saving = None;
                match answer {
                    Ok(added) => {
                        self.notice = Some(Notice {
                            text: if added.added == 0 {
                                m::MERGED_NONE.get().to_string()
                            } else {
                                m::merged(added.added as u64, &added.file)
                            },
                            error: false,
                        });
                        self.identities.done = (added.added > 0).then_some(added);
                        self.identities.ticked.clear();
                        self.identities.extras.clear();
                        // What the list says is mapped has changed.
                        self.load_identities();
                    }
                    Err(error) => {
                        self.identities.error =
                            Some(crate::daemon_link::error_text::error_text(&error));
                    }
                }
            }
        }
        // Reading the list is the tab's job: when it is on screen without
        // one (first time, or the scope moved), ask.
        if self.tab == PreviewTab::Identities
            && self.identities.load.is_none()
            && self.identities.error.is_none()
            && !self.busy()
        {
            let stale = match &self.identities.loaded {
                None => true,
                Some(l) => l.scope != self.recipe.scope,
            };
            if stale {
                self.load_identities();
            }
        }
    }

    // ----------------------------------------------------------- the list

    pub(crate) fn move_identity_cursor(&mut self, delta: isize) {
        let last = self.identity_rows().len().saturating_sub(1);
        self.identities.cursor =
            (self.identities.cursor as isize + delta).clamp(0, last as isize) as usize;
    }

    pub(crate) fn toggle_identity(&mut self, index: usize) {
        let Some(ident) = self.identity_rows().get(index).map(|r| r.ident()) else {
            return;
        };
        if !self.identities.ticked.remove(&ident) {
            self.identities.ticked.insert(ident);
        }
    }

    pub(crate) fn check_all_identities(&mut self) {
        self.identities.ticked = self.identity_rows().iter().map(|r| r.ident()).collect();
    }

    pub(crate) fn uncheck_all_identities(&mut self) {
        self.identities.ticked.clear();
    }

    // ------------------------------------------------------------- merging

    /// "Merge into...": the dialog that picks the identity to keep.
    pub(crate) fn open_merge_pick(&mut self) {
        if !self.can_merge() {
            if self.ticked_identities() + self.identities.extras.len() < 2 && !self.busy() {
                self.notice = Some(Notice {
                    text: m::NEED_TWO.get().to_string(),
                    error: true,
                });
            }
            return;
        }
        let mut candidates: Vec<Candidate> = self
            .identity_rows()
            .iter()
            .filter(|r| self.identities.ticked.contains(&r.ident()))
            .map(|r| Candidate {
                ident: r.ident(),
                commits: r.as_author,
            })
            .collect();
        // The one with most commits is the natural choice, listed first.
        candidates.sort_by_key(|c| std::cmp::Reverse(c.commits));
        for extra in &self.identities.extras {
            if !candidates.iter().any(|c| c.ident == *extra) {
                candidates.push(Candidate {
                    ident: extra.clone(),
                    commits: 0,
                });
            }
        }
        self.notice = None;
        self.identities.pick = Some(MergePick {
            candidates,
            cursor: 0,
        });
    }

    /// Candidate `index` becomes the canonical identity: the others are
    /// mapped to it.
    pub(crate) fn merge_into(&mut self, index: usize) {
        let Some(args) = self.merge_args(index) else {
            return;
        };
        self.identities.error = None;
        self.identities.done = None;
        self.identities.saving = Some(self.requester.call(
            OperationKind::MailmapAdd,
            serde_json::to_value(args).unwrap_or_default(),
        ));
    }

    /// What candidate `index` being kept asks of the daemon; closes the
    /// dialog.
    pub(super) fn merge_args(&mut self, index: usize) -> Option<AddArgs> {
        let pick = self.identities.pick.take()?;
        let canonical = pick.candidates.get(index)?.ident.clone();
        let from: Vec<Ident> = pick
            .candidates
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != index)
            .map(|(_, c)| c.ident.clone())
            .collect();
        Some(AddArgs {
            canonical,
            from,
            file: None,
        })
    }

    pub(crate) fn close_merge_pick(&mut self) {
        self.identities.pick = None;
    }

    // ------------------------------------------------------ the suggestion

    /// A rewrite that changes an author on commits already pushed: a
    /// `.mailmap` would fix the display without the force push.
    pub(crate) fn mailmap_offer(&self) -> bool {
        let Some(plan) = &self.plan else {
            return false;
        };
        self.offer_matches(plan)
    }

    fn offer_matches(&self, plan: &RewritePlan) -> bool {
        let pushed = plan
            .refs
            .iter()
            .any(|r| matches!(r.pushed, PushState::OnRemote { .. }));
        if !pushed {
            return false;
        }
        self.recipe.rules.iter().enumerate().any(|(i, rule)| {
            rule.enabled
                && matches!(&rule.kind, RuleKind::Identity(_))
                && plan.per_rule.iter().any(|c| c.rule == i && c.commits > 0)
        })
    }

    /// "Use .mailmap": the Identities tab with the identities the rule
    /// matches ticked and the identity it would set offered as the one to keep.
    pub(crate) fn use_mailmap_instead(&mut self) {
        let mut rules = Vec::new();
        let mut extras = Vec::new();
        for rule in &self.recipe.rules {
            if !rule.enabled {
                continue;
            }
            let RuleKind::Identity(r) = &rule.kind else {
                continue;
            };
            if let Ok(m) = CompiledMatch::new(&r.when) {
                rules.push(m);
            }
            if let (Some(name), Some(email)) = (&r.set.name, &r.set.email) {
                let ident = Ident::new(name, email);
                if !extras.contains(&ident) {
                    extras.push(ident);
                }
            }
        }
        self.focus = Focus::Preview;
        self.tab = PreviewTab::Identities;
        self.refusal = None;
        self.identities.select_rules = rules;
        self.identities.extras = extras;
        self.identities.done = None;
        self.identities.error = None;
        // Read again: the ticks are made when the answer comes.
        self.identities.loaded = None;
        self.load_identities();
    }

    /// The offer under the Summary: the text and the button. Returns the
    /// next free row.
    pub(super) fn paint_mailmap_offer(&self, ui: &mut Ui<'_>, body: Rect, y: u16) -> u16 {
        let mut y = y;
        if !self.mailmap_offer() || y + 2 >= body.bottom() {
            return y;
        }
        y += 1;
        let width = body.width.saturating_sub(2) as usize;
        for line in wrap(m::OFFER.get(), width) {
            if y >= body.bottom() {
                return y;
            }
            ui.text(body.x + 1, y, body.width - 1, &line, bold(false));
            y += 1;
        }
        if y < body.bottom() {
            ui.button(
                body.x + 1,
                y,
                m::OFFER_BTN.get(),
                RewriteHit::IdentOffer,
                false,
                !self.busy(),
            );
            y += 1;
        }
        y
    }

    // --------------------------------------------------------------- keys

    pub(super) fn on_identities_key(&mut self, key: KeyEvent, plain: bool) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') if plain => self.move_identity_cursor(-1),
            KeyCode::Down | KeyCode::Char('j') if plain => self.move_identity_cursor(1),
            KeyCode::PageUp => self.move_identity_cursor(-10),
            KeyCode::PageDown => self.move_identity_cursor(10),
            KeyCode::Home => self.identities.cursor = 0,
            KeyCode::End => self.move_identity_cursor(isize::MAX / 2),
            KeyCode::Char(' ') | KeyCode::Enter if plain => {
                self.toggle_identity(self.identities.cursor)
            }
            KeyCode::Char('m') if plain => self.open_merge_pick(),
            KeyCode::Char('r') if plain && self.identities.load.is_none() => {
                self.identities.error = None;
                self.load_identities();
            }
            _ => {}
        }
    }

    /// The `u` key and the Undo button: the caller runs the journal's undo.
    pub(super) fn identities_undo(&mut self) -> RewriteKeyOutcome {
        if self.identities.done.is_some() {
            RewriteKeyOutcome::Undo
        } else {
            RewriteKeyOutcome::Stay
        }
    }

    /// A key while the "Merge into..." dialog is open.
    pub(super) fn on_identity_pick_key(&mut self, key: KeyEvent) -> RewriteKeyOutcome {
        let Some(pick) = self.identities.pick.as_mut() else {
            return RewriteKeyOutcome::Stay;
        };
        let last = pick.candidates.len().saturating_sub(1);
        match key.code {
            KeyCode::Esc => self.identities.pick = None,
            KeyCode::Up | KeyCode::Char('k') => pick.cursor = pick.cursor.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('j') => pick.cursor = (pick.cursor + 1).min(last),
            KeyCode::Char(d @ '1'..='9') => {
                let at = d as usize - '1' as usize;
                if at <= last {
                    self.merge_into(at);
                }
            }
            KeyCode::Enter => {
                let at = pick.cursor;
                self.merge_into(at);
            }
            _ => {}
        }
        RewriteKeyOutcome::Stay
    }

    /// A press on one of the tab's parts.
    pub(super) fn on_identities_press(&mut self, hit: RewriteHit) -> RewriteKeyOutcome {
        match hit {
            RewriteHit::IdentOffer => {
                self.use_mailmap_instead();
                return RewriteKeyOutcome::Stay;
            }
            RewriteHit::IdentPick(i) => {
                self.merge_into(i);
                return RewriteKeyOutcome::Stay;
            }
            RewriteHit::IdentPickCancel | RewriteHit::IdentPickShield => {
                self.close_merge_pick();
                return RewriteKeyOutcome::Stay;
            }
            RewriteHit::IdentUndo => return self.identities_undo(),
            RewriteHit::IdentPickBody => return RewriteKeyOutcome::Stay,
            _ => {}
        }
        self.focus = Focus::Preview;
        self.tab = PreviewTab::Identities;
        match hit {
            RewriteHit::IdentMerge => self.open_merge_pick(),
            RewriteHit::IdentRefresh => {
                self.identities.error = None;
                self.load_identities();
            }
            RewriteHit::IdentAll => self.check_all_identities(),
            RewriteHit::IdentNone => self.uncheck_all_identities(),
            RewriteHit::IdentRow(i) => {
                self.identities.cursor = i;
                self.toggle_identity(i);
            }
            _ => {}
        }
        RewriteKeyOutcome::Stay
    }

    // -------------------------------------------------------------- paint

    pub(super) fn paint_identities(&self, ui: &mut Ui<'_>, body: Rect) {
        let x = body.x + 1;
        let w = body.width.saturating_sub(1);
        let width = body.width.saturating_sub(2) as usize;
        let mut y = body.y;
        let bottom = body.bottom();
        if body.height >= 9 {
            for line in wrap(m::INTRO.get(), width).into_iter().take(2) {
                ui.text(x, y, w, &line, dim());
                y += 1;
            }
        }

        // Buttons and the count.
        if y < bottom {
            let mut bx = x;
            bx += ui.button(
                bx,
                y,
                m::BTN_MERGE.get(),
                RewriteHit::IdentMerge,
                self.can_merge(),
                self.can_merge(),
            ) + 1;
            bx += ui.button(
                bx,
                y,
                m::BTN_REFRESH.get(),
                RewriteHit::IdentRefresh,
                false,
                self.identities.load.is_none() && !self.busy(),
            ) + 1;
            bx += ui.button(
                bx,
                y,
                t::PROTECTED_ALL.get(),
                RewriteHit::IdentAll,
                false,
                true,
            ) + 1;
            bx += ui.button(
                bx,
                y,
                t::PROTECTED_NONE.get(),
                RewriteHit::IdentNone,
                false,
                true,
            ) + 1;
            let total = self.identity_rows().len() as u64;
            let ticked = self.ticked_identities() as u64;
            ui.text(
                bx,
                y,
                body.right().saturating_sub(bx + 1),
                &m::selected(ticked, total),
                dim(),
            );
            y += 1;
        }

        // One status line: working, failed, what the last merge wrote (with
        // its Undo) or the missing file. In a short pane it makes way for the
        // list's rows.
        if y < bottom && bottom - y > head::MIN_LIST + 1 {
            if let Some(why) = &self.identities.error {
                ui.text(x, y, w, why, bad());
            } else if self.identities.load.is_some() || self.identities.saving.is_some() {
                ui.text(x, y, w, m::LOADING.get(), dim());
            } else if let Some(done) = &self.identities.done {
                let text = m::merged(done.added as u64, &done.file);
                let used = ui.text(x, y, w, &text, plain());
                ui.button(
                    x + used + 2,
                    y,
                    m::BTN_UNDO.get(),
                    RewriteHit::IdentUndo,
                    false,
                    true,
                );
            } else if self
                .identities
                .loaded
                .as_ref()
                .is_some_and(|l| !l.list.has_mailmap)
            {
                ui.text(x, y, w, &m::no_file(m::BTN_MERGE.get()), dim());
            }
            y += 1;
        }
        if y >= bottom {
            return;
        }
        let rows = self.identity_rows();
        if rows.is_empty() {
            if self.identities.loaded.is_some() {
                ui.text(x, y, w, m::NONE.get(), plain());
            }
            return;
        }
        // The cursor's counts get a line of their own when the rows have no
        // room for a counts column beside the longest name.
        let identity_text = |row: &gitpoulpi_core::domain::rewrite::IdentityCount| {
            let ident = show(&row.ident());
            match &row.mapped_to {
                Some(to) => format!("{ident}  {}", m::mapped(show(to))),
                None => ident,
            }
        };
        let counts_w = rows
            .iter()
            .map(|r| m::counts(r.as_author, r.as_committer).chars().count())
            .max()
            .unwrap_or(0);
        let ident_w = rows
            .iter()
            .map(|r| identity_text(r).chars().count())
            .max()
            .unwrap_or(0);
        let counts_inline = 5 + ident_w + 2 + counts_w < body.width as usize;
        let detail_h: u16 = u16::from(!counts_inline && bottom - y > head::MIN_LIST);
        let room = (bottom - y - detail_h) as usize;
        let cursor = self.identities.cursor.min(rows.len() - 1);
        let top = self.settle(
            BarId::Identities,
            Some(cursor),
            self.identities.top.get(),
            room,
            rows.len(),
        );
        self.identities.top.set(top);
        self.paint_bar(
            ui,
            BarId::Identities,
            (body.right(), y, room as u16),
            rows.len(),
            room,
            top,
            Some(cursor),
        );
        let focused = self.focus == Focus::Preview;
        for (slot, row) in rows.iter().enumerate().skip(top).take(room) {
            let ry = y + (slot - top) as u16;
            let band = Rect::new(body.x, ry, body.width, 1);
            let hit = RewriteHit::IdentRow(slot);
            let bg = row_bg(slot == cursor && focused, ui.hot(hit));
            dialog_kit::fill(ui.buf, band, Style::default().bg(bg));
            let on = self.identities.ticked.contains(&row.ident());
            let style = Style::default().fg(th().ui.text).bg(bg);
            let mut cx = band.x + 1;
            cx += ui.text(cx, ry, 4, if on { "[x] " } else { "[ ] " }, style);
            let right = band.right().saturating_sub(1);
            let counts_x = if counts_inline {
                right.saturating_sub(counts_w as u16)
            } else {
                right
            };
            ui.text(
                cx,
                ry,
                counts_x.saturating_sub(cx + 1),
                &identity_text(row),
                if row.mapped_to.is_some() {
                    Style::default().fg(th().ui.muted).bg(bg)
                } else {
                    style
                },
            );
            if counts_inline {
                ui.text(
                    counts_x,
                    ry,
                    right.saturating_sub(counts_x),
                    &m::counts(row.as_author, row.as_committer),
                    Style::default().fg(th().ui.muted).bg(bg),
                );
            }
            ui.coarse.push((band, hit));
        }
        if detail_h > 0 {
            let row = &rows[cursor];
            ui.text(
                x,
                bottom - 1,
                w,
                &m::counts(row.as_author, row.as_committer),
                dim(),
            );
        }
    }

    /// The "Merge into..." dialog, in front of the workbench: the ticked
    /// identities (and the one a rule offers) as numbered rows.
    pub(super) fn paint_identity_pick(
        &self,
        buf: &mut Buffer,
        bounds: Rect,
        hovered: Option<RewriteHit>,
    ) -> Vec<(Rect, RewriteHit)> {
        let mut hits = Vec::new();
        let Some(pick) = &self.identities.pick else {
            return hits;
        };
        let width = bounds.width.saturating_sub(6).clamp(34, 78);
        let prompt = wrap(
            &m::pick_prompt(pick.candidates.len() as u64),
            width as usize - 4,
        );
        let room = (bounds.height as usize).saturating_sub(prompt.len() + 6);
        let shown = pick.candidates.len().min(room.max(1));
        let height = (prompt.len() + shown + 5) as u16;
        let rect = WIN_IDENT.place(bounds, dialog_kit::centered(bounds, width, height));
        dialog_kit::panel(buf, rect, m::PICK_TITLE.get());
        if let Some(cross) = crate::chrome::close_cross::paint(
            buf,
            rect,
            hovered == Some(RewriteHit::IdentPickCancel),
        ) {
            WIN_IDENT.decorate(buf, rect, Some(cross), true);
            hits.push((cross, RewriteHit::IdentPickCancel));
        }
        let mut y = rect.y + 2;
        for line in &prompt {
            dialog_kit::put(buf, rect.x + 2, y, line, rect.width - 4, plain());
            y += 1;
        }
        y += 1;
        // Scrolled to keep the cursor in view when there are many.
        let first = self.settle(
            BarId::IdentPick,
            Some(pick.cursor),
            self.paint.ident_pick_top.get(),
            shown,
            pick.candidates.len(),
        );
        self.paint.ident_pick_top.set(first);
        self.paint_bar_on(
            buf,
            &mut hits,
            hovered == Some(RewriteHit::Bar(BarId::IdentPick)),
            BarId::IdentPick,
            (rect.right() - 1, y, shown as u16),
            pick.candidates.len(),
            shown,
            first,
            Some(pick.cursor),
        );
        for (i, c) in pick.candidates.iter().enumerate().skip(first).take(shown) {
            if y >= rect.bottom() - 1 {
                break;
            }
            let band = Rect::new(rect.x + 1, y, rect.width - 2, 1);
            let hit = RewriteHit::IdentPick(i);
            let bg = row_bg(i == pick.cursor, hovered == Some(hit));
            dialog_kit::fill(buf, band, Style::default().bg(bg));
            let text = if c.commits > 0 {
                format!(
                    "{}  {}  ({})",
                    i + 1,
                    show(&c.ident),
                    t::counts_commits(c.commits)
                )
            } else {
                format!("{}  {}", i + 1, show(&c.ident))
            };
            dialog_kit::put(
                buf,
                band.x + 1,
                y,
                &text,
                band.width - 2,
                Style::default().fg(th().ui.text).bg(bg),
            );
            hits.push((band, hit));
            y += 1;
        }
        hits.push((rect, RewriteHit::IdentPickBody));
        hits.push((bounds, RewriteHit::IdentPickShield));
        hits
    }
}

/// The dialog moves (its size follows its content).
const WIN_IDENT: crate::dialog_kit::Win = crate::dialog_kit::Win::movable("rewrite_identities");
