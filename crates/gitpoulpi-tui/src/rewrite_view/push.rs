//! The result panel's second half (slice S13): after an Apply, send the
//! rewritten refs back to the remote with their leases, copy the note for
//! collaborators, and show where the commit map is.
//!
//! The list offers the refs the run moved that a remote has. A protected
//! branch starts unticked even when the recipe included it: including it
//! rewrote it locally, ticking it here is the second gate, the one that
//! publishes. The answer of the push replaces the ticks with one row per
//! ref; a rejected row offers "Fetch and review" (the workbench closes and
//! the remotes are fetched, so the Log shows what moved).

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::Style;

use gitpoulpi_core::domain::rewrite::{
    NoteRef, NoteRefKind, PushRefResult, PushRefStatus, PushRewritten, PushRewrittenOutcome,
    PushState, RewriteMaps, RewriteNote, RewriteOutcome,
};
use gitpoulpi_core::domain::{Oid, OperationKind};
use gitpoulpi_i18n::msg::{rewrite_note as n, rewrite_push as p, rewrite_view as t};
use gitpoulpi_i18n::M;
use gitpoulpi_ipc::rewrite::{MapsArgs, NoteArgs};

use super::bars::BarId;
use super::preview::{ref_short, short};
use super::render::{bad, bold, dim, plain, row_bg, warn, wrap, RewriteHit, Ui};
use super::{Focus, Notice, RewriteKeyOutcome, RewriteView, Run};
use crate::daemon_link::Reply;
use crate::dialog_kit;
use crate::keymap::Action as Bind;
use crate::th;

/// A ref the run moved that a remote has: one line of the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PushPick {
    pub name: String,
    pub remote: String,
    pub old: Oid,
    pub new: Option<Oid>,
    pub protected: bool,
    pub ticked: bool,
}

#[derive(Default)]
pub(crate) struct PushPanel {
    pub picks: Vec<PushPick>,
    pub cursor: usize,
    pub(super) sending: Option<Reply<PushRewrittenOutcome>>,
    /// The answer of the last push.
    pub sent: Option<PushRewrittenOutcome>,
    pub(super) note_wait: Option<Reply<RewriteNote>>,
    pub(super) maps_wait: Option<Reply<RewriteMaps>>,
    /// Where the two maps are, once asked.
    pub maps: Option<RewriteMaps>,
    /// S21: the run moved files into or out of LFS (the note says so and
    /// "Push LFS objects" is offered).
    pub lfs: bool,
    pub(super) lfs_wait: Option<Reply<()>>,
    /// The fetch the missing-objects refusal offers, and what it is for.
    pub(super) lfs_fetch_wait: Option<Reply<()>>,
}

impl PushPanel {
    /// The panel of a run that just finished.
    pub(super) fn of(outcome: &RewriteOutcome) -> Self {
        let picks = outcome
            .moved
            .iter()
            .filter_map(|c| {
                let PushState::OnRemote { remote, .. } = &c.pushed else {
                    return None;
                };
                Some(PushPick {
                    name: c.name.clone(),
                    remote: remote.clone(),
                    old: c.old,
                    new: c.new,
                    protected: c.protected,
                    ticked: !c.protected,
                })
            })
            .collect();
        Self {
            picks,
            ..Self::default()
        }
    }

    /// Rows the cursor can stand on: the picks, or the answer's rows.
    fn rows(&self) -> usize {
        match &self.sent {
            Some(s) => s.results.len(),
            None => self.picks.len(),
        }
    }

    pub(crate) fn sending(&self) -> bool {
        self.sending.is_some()
    }

    fn ticked(&self) -> Vec<&PushPick> {
        self.picks.iter().filter(|p| p.ticked).collect()
    }

    /// The request for the ticked refs. Protected ones are named in
    /// `include_protected`, the list's ticks being the whole decision.
    pub(crate) fn request(&self, run: &gitpoulpi_core::domain::rewrite::RunId) -> PushRewritten {
        let ticked = self.ticked();
        let mut req = PushRewritten::new(run.clone());
        req.refs = Some(ticked.iter().map(|p| p.name.clone()).collect());
        req.include_protected = ticked
            .iter()
            .filter(|p| p.protected)
            .map(|p| p.name.clone())
            .collect();
        req
    }
}

/// The status of a pushed ref, in words.
fn status_text(status: PushRefStatus) -> (M, Style) {
    match status {
        PushRefStatus::Pushed => (n::PUSH_PUSHED, plain()),
        PushRefStatus::WouldPush => (n::PUSH_WOULD, plain()),
        PushRefStatus::UpToDate => (n::PUSH_UP_TO_DATE, dim()),
        PushRefStatus::RejectedRemoteMoved => (n::PUSH_REMOTE_MOVED, bad()),
        PushRefStatus::RejectedProtectedByHost => (n::PUSH_PROTECTED_HOST, bad()),
        PushRefStatus::RejectedOther => (n::PUSH_REJECTED, bad()),
        PushRefStatus::SkippedProtected => (n::PUSH_SKIPPED_PROTECTED, warn()),
        PushRefStatus::NotPushed => (n::PUSH_NOT_PUSHED, warn()),
    }
}

fn first_line(text: &str) -> &str {
    text.lines().next().unwrap_or("").trim()
}

/// The note for collaborators in the interface language. Git commands stay
/// as they are (`rewrite_note`'s keys carry the sentences around them).
pub(super) fn note_text(note: &RewriteNote) -> String {
    note_text_with(note, false)
}

/// [`note_text`], with the LFS paragraph when the run moved files into or out
/// of LFS.
pub(super) fn note_text_with(note: &RewriteNote, lfs: bool) -> String {
    if note.refs.is_empty() {
        return format!("{}\n", n::NOTHING.get());
    }
    let mut out = format!("{}\n\n", n::INTRO.get());
    for r in &note.refs {
        out.push_str(&note_ref(r));
        out.push('\n');
    }
    if lfs {
        out.push_str(n::LFS.get());
        out.push_str("\n    git lfs install\n    git lfs pull\n\n"); // i18n-english: git commands
    }
    out.push_str(n::OUTRO.get());
    out.push('\n');
    out
}

fn note_ref(r: &NoteRef) -> String {
    let (old, new) = (twelve(&r.old), twelve(&r.new));
    let (b, rem) = (&r.short, &r.remote);
    match r.kind {
        NoteRefKind::Branch => format!(
            "{head}\n  {clean}\n    git fetch {rem}\n    git checkout {b}\n    git reset --hard {rem}/{b}\n  {local}\n    git fetch {rem}\n    git rebase --onto {rem}/{b} {old} {b}\n", // i18n-english: git commands
            head = n::branch_head(b, rem, &old, &new),
            clean = n::BRANCH_CLEAN.get(),
            local = n::branch_local(b),
        ),
        NoteRefKind::Tag => format!(
            "{head}\n  {replace}\n    git fetch {rem} +refs/tags/{b}:refs/tags/{b}\n", // i18n-english: git commands
            head = n::tag_head(b, rem, &old, &new),
            replace = n::TAG_REPLACE.get(),
        ),
    }
}

fn twelve(oid: &Oid) -> String {
    let hex = oid.to_hex();
    hex[..hex.len().min(12)].to_string()
}

impl RewriteView {
    pub(crate) fn pushing(&self) -> bool {
        self.push.sending()
    }

    fn done_run(&self) -> Option<gitpoulpi_core::domain::rewrite::RunId> {
        match &self.run {
            Run::Done(outcome) => Some(outcome.run.clone()),
            _ => None,
        }
    }

    // --------------------------------------------------------------- calls

    /// "Push rewritten branches": the ticked refs, with their leases.
    pub(crate) fn push_send(&mut self) {
        let Some(run) = self.done_run() else {
            return;
        };
        if self.busy() {
            return;
        }
        if self.push.ticked().is_empty() {
            self.notice = Some(Notice {
                text: p::NONE_TICKED.get().to_string(),
                error: true,
            });
            return;
        }
        let req = self.push.request(&run);
        self.notice = None;
        self.push.sending = Some(self.requester.call(
            OperationKind::RewritePush,
            serde_json::to_value(req).unwrap_or_default(),
        ));
    }

    /// "Push LFS objects": the existing LFS push, for every ref, to the
    /// remote the rewritten refs are on. The pre-push hook `git lfs install`
    /// leaves does the same on a plain push, so this is the explicit, earlier
    /// way.
    pub(crate) fn lfs_push(&mut self) {
        if !self.push.lfs || self.push.lfs_wait.is_some() || self.busy() {
            return;
        }
        let remote = self
            .push
            .picks
            .first()
            .map_or_else(|| "origin".to_string(), |p| p.remote.clone());
        self.notice = Some(Notice {
            text: p::LFS_PUSHING.get().to_string(),
            error: false,
        });
        self.push.lfs_wait = Some(self.requester.call(
            OperationKind::LfsPush,
            serde_json::json!({ "remote": remote, "opts": { "all_refs": true } }),
        ));
    }

    /// The missing-objects refusal's offer: fetch every LFS object from the
    /// remote (the existing LFS fetch), then preview again.
    pub(crate) fn lfs_fetch(&mut self) {
        if self.push.lfs_fetch_wait.is_some() || self.busy() {
            return;
        }
        self.notice = Some(Notice {
            text: p::LFS_FETCHING.get().to_string(),
            error: false,
        });
        self.push.lfs_fetch_wait = Some(self.requester.call(
            OperationKind::LfsFetch,
            serde_json::json!({ "remote": "origin", "opts": { "all_refs": true } }),
        ));
    }

    /// Asks for the note; the answer is copied by [`Self::poll_push`].
    pub(crate) fn copy_note(&mut self) {
        let Some(run) = self.done_run() else {
            return;
        };
        if self.push.note_wait.is_some() {
            return;
        }
        self.push.note_wait = Some(self.requester.call(
            OperationKind::RewriteNote,
            serde_json::to_value(NoteArgs { run }).unwrap_or_default(),
        ));
    }

    /// "Open the commit map": where the two files are.
    pub(crate) fn open_maps(&mut self) {
        let Some(run) = self.done_run() else {
            return;
        };
        if self.push.maps_wait.is_some() {
            return;
        }
        self.push.maps_wait = Some(self.requester.call(
            OperationKind::RewriteMaps,
            serde_json::to_value(MapsArgs { run, out: None }).unwrap_or_default(),
        ));
    }

    /// Copies one of the two map paths (0: commit map, 1: ref map).
    pub(crate) fn copy_map_path(&mut self, which: usize) {
        let Some(maps) = &self.push.maps else {
            return;
        };
        let path = if which == 0 {
            &maps.commit_map
        } else {
            &maps.ref_map
        };
        let said = crate::clipboard::copy_shared(p::WHAT_PATH.get(), &path.display().to_string());
        self.notice = Some(Notice {
            text: said,
            error: false,
        });
    }

    /// Collects the push, the note and the maps. Never waits.
    pub(super) fn poll_push(&mut self) {
        if let Some(answer) = self.push.sending.as_ref().and_then(|r| r.take()) {
            self.push.sending = None;
            match answer {
                Ok(out) => {
                    self.push.cursor = 0;
                    self.push.sent = Some(out);
                }
                Err(error) => {
                    self.notice = Some(Notice {
                        text: crate::daemon_link::error_text::error_text(&error),
                        error: true,
                    });
                }
            }
        }
        if let Some(answer) = self.push.lfs_wait.as_ref().and_then(|r| r.take()) {
            self.push.lfs_wait = None;
            self.notice = Some(match answer {
                Ok(()) => Notice {
                    text: p::LFS_PUSHED.get().to_string(),
                    error: false,
                },
                Err(error) => Notice {
                    text: crate::daemon_link::error_text::error_text(&error),
                    error: true,
                },
            });
        }
        if let Some(answer) = self.push.lfs_fetch_wait.as_ref().and_then(|r| r.take()) {
            self.push.lfs_fetch_wait = None;
            self.notice = Some(match answer {
                Ok(()) => {
                    // What was missing is here now: preview again.
                    self.refusal = None;
                    Notice {
                        text: p::LFS_FETCHED.get().to_string(),
                        error: false,
                    }
                }
                Err(error) => Notice {
                    text: crate::daemon_link::error_text::error_text(&error),
                    error: true,
                },
            });
        }
        if let Some(answer) = self.push.note_wait.as_ref().and_then(|r| r.take()) {
            self.push.note_wait = None;
            match answer {
                Ok(note) => {
                    let said = crate::clipboard::copy_shared(
                        p::WHAT_NOTE.get(),
                        &note_text_with(&note, self.push.lfs),
                    );
                    self.notice = Some(Notice {
                        text: said,
                        error: false,
                    });
                }
                Err(error) => {
                    self.notice = Some(Notice {
                        text: crate::daemon_link::error_text::error_text(&error),
                        error: true,
                    });
                }
            }
        }
        if let Some(answer) = self.push.maps_wait.as_ref().and_then(|r| r.take()) {
            self.push.maps_wait = None;
            match answer {
                Ok(maps) => self.push.maps = Some(maps),
                Err(error) => {
                    self.notice = Some(Notice {
                        text: crate::daemon_link::error_text::error_text(&error),
                        error: true,
                    });
                }
            }
        }
    }

    // ---------------------------------------------------------------- ticks

    pub(crate) fn move_push_cursor(&mut self, delta: isize) {
        let last = self.push.rows().saturating_sub(1);
        self.push.cursor = (self.push.cursor as isize + delta).clamp(0, last as isize) as usize;
    }

    pub(crate) fn toggle_pick(&mut self, index: usize) {
        if self.push.sent.is_some() {
            return;
        }
        if let Some(pick) = self.push.picks.get_mut(index) {
            pick.ticked = !pick.ticked;
        }
    }

    /// "Check all": every protected branch of the list is ticked.
    pub(crate) fn check_all_picks(&mut self) {
        if self.push.sent.is_none() {
            for pick in self.push.picks.iter_mut().filter(|p| p.protected) {
                pick.ticked = true;
            }
        }
    }

    /// "Uncheck all": every protected branch of the list is left out.
    pub(crate) fn uncheck_all_picks(&mut self) {
        if self.push.sent.is_none() {
            for pick in self.push.picks.iter_mut().filter(|p| p.protected) {
                pick.ticked = false;
            }
        }
    }

    fn rejected_at(&self, index: usize) -> bool {
        self.push
            .sent
            .as_ref()
            .and_then(|s| s.results.get(index))
            .is_some_and(|r| r.status.is_rejected())
    }

    // ---------------------------------------------------------------- keys

    /// A key while the result panel is up. `None`: not for the panel.
    pub(super) fn on_result_key(
        &mut self,
        key: KeyEvent,
        plain: bool,
        km: &crate::keymap::Keymap,
    ) -> Option<RewriteKeyOutcome> {
        match key.code {
            _ if km.is(Bind::RewriteProtectedAll, &key) => self.check_all_picks(),
            _ if km.is(Bind::RewriteProtectedNone, &key) => self.uncheck_all_picks(),
            KeyCode::Up | KeyCode::Char('k') if plain => self.move_push_cursor(-1),
            KeyCode::Down | KeyCode::Char('j') if plain => self.move_push_cursor(1),
            KeyCode::Char(' ') | KeyCode::Enter if plain => self.toggle_pick(self.push.cursor),
            KeyCode::Char('p') if plain => self.push_send(),
            KeyCode::Char('c') if plain => self.copy_note(),
            KeyCode::Char('m') if plain => self.open_maps(),
            KeyCode::Char('l') if plain => self.lfs_push(),
            KeyCode::Char('e') if plain => self.open_purge(false),
            KeyCode::Char('1') if plain => self.copy_map_path(0),
            KeyCode::Char('2') if plain => self.copy_map_path(1),
            KeyCode::Char('f') if plain => {
                if self.rejected_at(self.push.cursor) {
                    return Some(RewriteKeyOutcome::FetchReview);
                }
            }
            _ => return None,
        }
        Some(RewriteKeyOutcome::Stay)
    }

    /// A click on one of the result panel's parts.
    pub(super) fn on_push_press(&mut self, hit: RewriteHit) -> RewriteKeyOutcome {
        self.focus = Focus::Preview;
        match hit {
            RewriteHit::PushRow(i) => {
                self.push.cursor = i;
                self.toggle_pick(i);
            }
            RewriteHit::PushAll => self.check_all_picks(),
            RewriteHit::PushNone => self.uncheck_all_picks(),
            RewriteHit::PushSend => self.push_send(),
            RewriteHit::CopyNote => self.copy_note(),
            RewriteHit::OpenMaps => self.open_maps(),
            RewriteHit::LfsPush => self.lfs_push(),
            RewriteHit::LfsFetch => self.lfs_fetch(),
            RewriteHit::MapCopy(i) => self.copy_map_path(i),
            RewriteHit::FetchReview(_) => return RewriteKeyOutcome::FetchReview,
            _ => {}
        }
        RewriteKeyOutcome::Stay
    }

    // --------------------------------------------------------------- paint

    /// The result panel: what was rewritten, the push list, the buttons.
    pub(super) fn paint_result(&self, ui: &mut Ui<'_>, inner: Rect) {
        let Run::Done(outcome) = &self.run else {
            return;
        };
        let x = inner.x + 1;
        let w = inner.width.saturating_sub(1);
        let mut y = inner.y;
        ui.text(x, y, w, &self.result_headline(outcome), bold(true));
        y += 1;

        // From the bottom: the Undo row, the hint, then (with something to
        // push) the push buttons and the warning, then the maps.
        let bottom = inner.bottom() - 1;
        let picks = !self.push.picks.is_empty();
        let mut end = bottom.saturating_sub(1);
        ui.text(x, end, w, t::DONE_HINT.get(), dim());
        if picks {
            end = end.saturating_sub(1);
            self.paint_push_buttons(ui, inner, end);
            end = end.saturating_sub(1);
            ui.text(x, end, w, n::PUSH_UNDO_WARNING.get(), warn());
            if self.push.sent.as_ref().is_some_and(|s| s.atomic_fallback) {
                end = end.saturating_sub(1);
                ui.text(x, end, w, n::PUSH_ATOMIC_FALLBACK.get(), dim());
            }
            if let Some(maps) = &self.push.maps {
                end = end.saturating_sub(1);
                let lines = [
                    (p::MAPS_COMMIT.get(), &maps.commit_map, 0usize),
                    (p::MAPS_REF.get(), &maps.ref_map, 1usize),
                ];
                for (label, path, which) in lines.into_iter().rev() {
                    if end <= y {
                        break;
                    }
                    let hit = RewriteHit::MapCopy(which);
                    let hot = ui.hot(hit);
                    let bg = row_bg(false, hot);
                    let band = Rect::new(inner.x, end, inner.width, 1);
                    dialog_kit::fill(ui.buf, band, Style::default().bg(bg));
                    let used = ui.text(
                        x,
                        end,
                        w,
                        &format!("{label}  "),
                        Style::default().fg(th().ui.muted).bg(bg),
                    );
                    ui.text(
                        x + used,
                        end,
                        w.saturating_sub(used),
                        &path.display().to_string(),
                        Style::default().fg(th().ui.text).bg(bg),
                    );
                    ui.fine.push((band, hit));
                    end = end.saturating_sub(1);
                }
                if end > y {
                    ui.text(x, end, w, p::MAPS_HINT.get(), dim());
                    end = end.saturating_sub(1);
                }
            }
        }
        let mut bx = x;
        bx += ui.button(bx, bottom, t::BTN_UNDO.get(), RewriteHit::Undo, true, true) + 1;
        bx += ui.button(
            bx,
            bottom,
            gitpoulpi_i18n::msg::rewrite_purge::BUTTON_OPEN.get(),
            RewriteHit::PurgeOpen,
            false,
            !self.busy(),
        ) + 1;
        ui.button(
            bx,
            bottom,
            t::BTN_CLOSE.get(),
            RewriteHit::ResultClose,
            false,
            true,
        );

        // The list fills what is left.
        let list_end = end + 1;
        if picks {
            self.paint_push_list(ui, inner, y, list_end);
        } else {
            let room = list_end.saturating_sub(y) as usize;
            let moved = &outcome.moved;
            for change in moved.iter().take(room.saturating_sub(1)) {
                ui.text(x, y, w, &super::preview::ref_line(change), plain());
                y += 1;
            }
            if moved.len() > room.saturating_sub(1) && y < list_end {
                ui.text(
                    x,
                    y,
                    w,
                    &t::done_more(moved.len() - room.saturating_sub(1)),
                    dim(),
                );
                y += 1;
            }
            if y < list_end {
                ui.text(x, y, w, n::PUSH_NOTHING.get(), dim());
            }
        }
    }

    fn paint_push_buttons(&self, ui: &mut Ui<'_>, inner: Rect, y: u16) {
        let busy = self.busy();
        let mut x = inner.x + 1;
        x += ui.button(
            x,
            y,
            p::BTN_PUSH.get(),
            RewriteHit::PushSend,
            self.push.sent.is_none(),
            !busy,
        ) + 1;
        x += ui.button(x, y, p::BTN_NOTE.get(), RewriteHit::CopyNote, false, true) + 1;
        x += ui.button(x, y, p::BTN_MAPS.get(), RewriteHit::OpenMaps, false, true) + 1;
        if self.push.lfs {
            let idle = !busy && self.push.lfs_wait.is_none();
            x += ui.button(x, y, p::BTN_LFS.get(), RewriteHit::LfsPush, false, idle) + 1;
        }
        if self.push.sending() {
            ui.text(
                x + 1,
                y,
                inner.right().saturating_sub(x + 2),
                p::PUSHING.get(),
                dim(),
            );
        }
    }

    /// The heading, then the ticks (or the answer's rows), then the other
    /// refs the run moved.
    fn paint_push_list(&self, ui: &mut Ui<'_>, inner: Rect, first: u16, end: u16) {
        let x = inner.x + 1;
        let w = inner.width.saturating_sub(1);
        let mut y = first;
        if y >= end {
            return;
        }
        ui.text(x, y, w, p::HEADING.get(), bold(false));
        if self.push.sent.is_none() && self.push.picks.iter().any(|p| p.protected) {
            let none = t::PROTECTED_NONE.get();
            let all = t::PROTECTED_ALL.get();
            let nw = dialog_kit::button_width(none);
            let aw = dialog_kit::button_width(all);
            if inner.width > nw + aw + 40 {
                let nx = inner.right() - nw - 1;
                let ax = nx - aw - 1;
                ui.button(ax, y, all, RewriteHit::PushAll, false, true);
                ui.button(nx, y, none, RewriteHit::PushNone, false, true);
            }
        }
        y += 1;
        let room = end.saturating_sub(y) as usize;
        if room == 0 {
            return;
        }
        let rows = self.push.rows();
        let cursor = self.push.cursor.min(rows.saturating_sub(1));
        let top = self.settle(
            BarId::Push,
            Some(cursor),
            self.paint.push_top.get(),
            room,
            rows,
        );
        self.paint.push_top.set(top);
        self.paint_bar(
            ui,
            BarId::Push,
            (inner.right(), y, room as u16),
            rows,
            room,
            top,
            Some(cursor),
        );
        let focused = self.focus == Focus::Preview;
        match &self.push.sent {
            Some(sent) if sent.results.is_empty() => {
                ui.text(x, y, w, n::PUSH_NOTHING.get(), dim());
            }
            Some(sent) => {
                for (slot, result) in sent.results.iter().enumerate().skip(top).take(room) {
                    let ry = y + (slot - top) as u16;
                    self.paint_result_row(ui, inner, ry, slot, result, slot == cursor && focused);
                }
            }
            None => {
                for (slot, pick) in self.push.picks.iter().enumerate().skip(top).take(room) {
                    let ry = y + (slot - top) as u16;
                    let hit = RewriteHit::PushRow(slot);
                    let bg = row_bg(slot == cursor && focused, ui.hot(hit));
                    let band = Rect::new(inner.x, ry, inner.width, 1);
                    dialog_kit::fill(ui.buf, band, Style::default().bg(bg));
                    let new = match &pick.new {
                        Some(id) => short(id),
                        None => t::REF_DELETED.get().to_string(),
                    };
                    let line = format!(
                        "{} {}  {} \u{2192} {}  {}",
                        if pick.ticked { "[x]" } else { "[ ]" },
                        ref_short(&pick.name),
                        short(&pick.old),
                        new,
                        pick.remote
                    );
                    let used = ui.text(x, ry, w, &line, Style::default().fg(th().ui.text).bg(bg));
                    if pick.protected {
                        let tag = self
                            .host_label(&pick.name)
                            .unwrap_or_else(|| p::PROTECTED_TAG.get().to_string());
                        ui.text(
                            x + used + 2,
                            ry,
                            w.saturating_sub(used + 2),
                            &tag,
                            Style::default().fg(th().status.danger).bg(bg),
                        );
                    }
                    ui.fine.push((band, hit));
                }
            }
        }
    }

    fn paint_result_row(
        &self,
        ui: &mut Ui<'_>,
        inner: Rect,
        y: u16,
        slot: usize,
        result: &PushRefResult,
        selected: bool,
    ) {
        let x = inner.x + 1;
        let band = Rect::new(inner.x, y, inner.width, 1);
        dialog_kit::fill(ui.buf, band, Style::default().bg(row_bg(selected, false)));
        let (status, style) = status_text(result.status);
        let mut text = format!(
            "{}  {}  {}",
            ref_short(&result.name),
            result.remote,
            status.get()
        );
        if matches!(
            result.status,
            PushRefStatus::RejectedOther | PushRefStatus::NotPushed
        ) {
            if let Some(detail) = result.detail.as_deref().map(first_line) {
                if !detail.is_empty() {
                    text.push_str(": ");
                    text.push_str(detail);
                }
            }
        }
        let mut right = inner.right();
        if result.status.is_rejected() {
            let label = p::BTN_FETCH.get();
            let bw = dialog_kit::button_width(label);
            if inner.width > bw + 30 {
                right -= bw + 1;
                ui.button(right, y, label, RewriteHit::FetchReview(slot), false, true);
            }
        }
        let style = style.bg(row_bg(selected, false));
        ui.text(x, y, right.saturating_sub(x + 1), &text, style);
        ui.coarse.push((band, RewriteHit::PushRow(slot)));
    }
}
