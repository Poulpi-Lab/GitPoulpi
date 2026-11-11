//! The purge screen (slice S15): after a rewrite, delete the old history
//! from this clone for good.
//!
//! An overlay of the workbench, like the blob diff drill-in, opened by the
//! result panel's button, the palette (`Rewrite: Purge Old History...`) and
//! the Undo history. It shows what the daemon's check says
//! (`RewritePurgeCheck`): the journal entries that lose their undo, what
//! would be deleted, **what still holds the old history** with a fix per row
//! (fetch the remotes, drop a stash after a confirmation, otherwise copy the
//! name), and the steps to take on each host afterwards. The check is asked
//! again after every fix and on "Check again". `[Purge]` is enabled only when
//! `PurgeCheck::can_run()`, asks a confirmation with no default (it is
//! irreversible), then runs `RewritePurge` as a job that cannot be stopped.

use std::cell::Cell;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use gitpoulpi_core::domain::rewrite::{
    HostKind, HostStep, PurgeBlocker, PurgeBlockerKind, PurgeCheck, PurgeOutcome, PurgeStatus,
    PurgeStep,
};
use gitpoulpi_core::domain::OperationKind;
use gitpoulpi_i18n::msg::{rewrite_purge as p, rewrite_view as t};
use gitpoulpi_i18n::M;
use gitpoulpi_ipc::rewrite::PurgeArgs;

use super::bars::BarId;
use super::render::{bad, bold, dim, plain, row_bg, warn, wrap, RewriteHit};
use super::{RewriteKeyOutcome, RewriteView};
use crate::daemon_link::Reply;
use crate::dialog_kit::{self, button, button_width, ButtonState};
use crate::th;

/// A question the screen waits on. Neither has a default answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PurgeConfirm {
    /// Delete the old history for good.
    Purge { cursor: Option<usize> },
    /// Drop stash `id` (`name` is `stash@{id}`): it is work.
    Drop {
        name: String,
        id: u32,
        cursor: Option<usize>,
    },
}

impl PurgeConfirm {
    fn cursor_mut(&mut self) -> &mut Option<usize> {
        match self {
            PurgeConfirm::Purge { cursor } | PurgeConfirm::Drop { cursor, .. } => cursor,
        }
    }
}

/// What a blocker row can do about the blocker itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Fix {
    /// Fetch the remotes: the remote-tracking refs move to the new history.
    Fetch,
    /// Drop stash `.0` (after a confirmation).
    Drop(u32),
    /// Nothing the screen can do: the name can be copied.
    None,
}

/// The fix a blocker offers.
pub(crate) fn fix_of(blocker: &PurgeBlocker) -> Fix {
    match blocker.kind {
        PurgeBlockerKind::RemoteTracking => Fix::Fetch,
        PurgeBlockerKind::Stash => stash_index(&blocker.name).map_or(Fix::None, Fix::Drop),
        _ => Fix::None,
    }
}

/// `stash@{3}` -> 3.
fn stash_index(name: &str) -> Option<u32> {
    name.strip_prefix("stash@{")?
        .strip_suffix('}')?
        .parse()
        .ok()
}

/// The open purge screen.
pub(crate) struct PurgeScreen {
    /// Which journal entries: empty until the first check answers, then the
    /// rewrite entries it named (so every later check and the run are about
    /// the same ones).
    pub args: PurgeArgs,
    pub check: Option<PurgeCheck>,
    pub(super) check_wait: Option<Reply<PurgeCheck>>,
    pub(super) fix_wait: Option<Reply<serde_json::Value>>,
    pub(super) run_wait: Option<Reply<PurgeOutcome>>,
    /// The part of the work the running purge is in, and its percentage.
    pub progress: Option<(Option<String>, Option<u8>)>,
    /// The last failure, in words.
    pub error: Option<String>,
    /// The blocker under the cursor.
    pub cursor: usize,
    pub confirm: Option<PurgeConfirm>,
    /// How the purge ended (`blocked` goes back to the list instead).
    pub outcome: Option<PurgeOutcome>,
    /// Opened by the palette: closing it closes the workbench too.
    pub standalone: bool,
    /// First row shown, and whether the next paint scrolls to the cursor.
    pub(super) scroll: Cell<usize>,
    follow: Cell<bool>,
    room: Cell<usize>,
}

impl PurgeScreen {
    fn new(standalone: bool) -> Self {
        Self {
            args: PurgeArgs::default(),
            check: None,
            check_wait: None,
            fix_wait: None,
            run_wait: None,
            progress: None,
            error: None,
            cursor: 0,
            confirm: None,
            outcome: None,
            standalone,
            scroll: Cell::new(0),
            follow: Cell::new(false),
            room: Cell::new(1),
        }
    }

    pub(crate) fn blockers(&self) -> &[PurgeBlocker] {
        self.check.as_ref().map_or(&[][..], |c| &c.outcome.blockers)
    }

    /// A request of this screen is on its way.
    pub(crate) fn busy(&self) -> bool {
        self.check_wait.is_some() || self.fix_wait.is_some() || self.run_wait.is_some()
    }

    pub(crate) fn running(&self) -> bool {
        self.run_wait.is_some()
    }

    /// `[Purge]` is available: nothing blocks, nothing is going on.
    pub(crate) fn can_purge(&self) -> bool {
        self.outcome.is_none()
            && !self.busy()
            && self.confirm.is_none()
            && self.check.as_ref().is_some_and(PurgeCheck::can_run)
    }

    fn rewrite_ids(check: &PurgeCheck) -> Vec<u64> {
        check
            .entries
            .iter()
            .filter(|e| e.rewrite)
            .map(|e| e.id)
            .collect()
    }
}

// ----------------------------------------------------------------- wording

fn blocker_text(b: &PurgeBlocker) -> String {
    match b.kind {
        PurgeBlockerKind::Branch => p::blocker_branch(&b.name),
        PurgeBlockerKind::Tag => p::blocker_tag(&b.name),
        PurgeBlockerKind::RemoteTracking => p::blocker_remote_tracking(&b.name),
        PurgeBlockerKind::Stash => p::blocker_stash(&b.name),
        PurgeBlockerKind::Worktree => p::blocker_worktree(&b.name),
        PurgeBlockerKind::UndoPin => p::blocker_undo_pin(&b.name),
        PurgeBlockerKind::Notes => p::blocker_notes(&b.name),
        PurgeBlockerKind::Replace => p::blocker_replace(&b.name),
        PurgeBlockerKind::Other => p::blocker_other(&b.name),
    }
}

fn host_name(host: HostKind) -> M {
    match host {
        HostKind::GitHub => p::HOST_GITHUB,
        HostKind::GitLab => p::HOST_GITLAB,
        HostKind::AzureDevOps => p::HOST_AZURE_DEVOPS,
        HostKind::Bitbucket => p::HOST_BITBUCKET,
        HostKind::Other => p::HOST_OTHER,
    }
}

fn host_step_text(step: HostStep) -> M {
    match step {
        HostStep::RotateSecret => p::HOST_ROTATE_SECRET,
        HostStep::PushRewritten => p::HOST_PUSH_REWRITTEN,
        HostStep::TellCollaborators => p::HOST_TELL_COLLABORATORS,
        HostStep::GithubSupport => p::HOST_GITHUB_SUPPORT,
        HostStep::GithubForks => p::HOST_GITHUB_FORKS,
        HostStep::GithubRecreate => p::HOST_GITHUB_RECREATE,
        HostStep::GitlabCleanup => p::HOST_GITLAB_CLEANUP,
        HostStep::GitlabExpire => p::HOST_GITLAB_EXPIRE,
        HostStep::GitlabRecreate => p::HOST_GITLAB_RECREATE,
        HostStep::AzureSupport => p::HOST_AZURE_SUPPORT,
        HostStep::AzureRecreate => p::HOST_AZURE_RECREATE,
        HostStep::BitbucketSupport => p::HOST_BITBUCKET_SUPPORT,
        HostStep::BitbucketForks => p::HOST_BITBUCKET_FORKS,
        HostStep::BitbucketRecreate => p::HOST_BITBUCKET_RECREATE,
        HostStep::ServerGc => p::HOST_SERVER_GC,
    }
}

/// The first link of a host's steps (the English text is the one that
/// names the address; every translation keeps it).
fn host_link(steps: &[HostStep]) -> Option<String> {
    steps.iter().find_map(|s| {
        let text = s.text();
        let at = text.find("https://")?;
        let link: String = text[at..]
            .chars()
            .take_while(|c| !c.is_whitespace() && *c != ')')
            .collect();
        Some(link.trim_end_matches(['.', ',', ';']).to_string())
    })
}

/// The job's phase name in the interface language.
fn step_text(phase: &str) -> String {
    let steps = [
        (PurgeStep::Checking, p::STEP_CHECKING),
        (PurgeStep::Pins, p::STEP_PINS),
        (PurgeStep::Reflogs, p::STEP_REFLOGS),
        (PurgeStep::Collecting, p::STEP_COLLECTING),
        (PurgeStep::Verifying, p::STEP_VERIFYING),
    ];
    steps
        .iter()
        .find(|(s, _)| s.name() == phase)
        .map_or_else(|| phase.to_string(), |(_, m)| m.get().to_string())
}

fn entry_line(e: &gitpoulpi_core::domain::rewrite::PurgeEntryInfo) -> String {
    if e.rewrite {
        p::entry_rewrite(e.id, &e.label)
    } else {
        p::entry_also(e.id, &e.label)
    }
}

fn good() -> Style {
    Style::default().fg(th().status.success).bg(th().ui.panel)
}

// -------------------------------------------------------------- the rows

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tone {
    Plain,
    Dim,
    Warn,
    Bad,
    Good,
    Bold,
}

/// One line of the scrolling body.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Row {
    Text(String, Tone),
    Gap,
    /// Line `line` of blocker `idx`.
    Blocker {
        idx: usize,
        line: usize,
        text: String,
    },
    /// The "Copy link" row of host `.0`.
    Link(usize),
}

fn wrapped(rows: &mut Vec<Row>, text: &str, width: usize, tone: Tone, indent: &str) {
    wrapped_with(rows, text, width, tone, indent, indent);
}

/// `first` leads the first line, `rest` the others (a bullet and its
/// hanging indent).
fn wrapped_with(
    rows: &mut Vec<Row>,
    text: &str,
    width: usize,
    tone: Tone,
    first: &str,
    rest: &str,
) {
    let room = width.saturating_sub(first.len()).max(10);
    for (i, line) in wrap(text, room).into_iter().enumerate() {
        let lead = if i == 0 { first } else { rest };
        rows.push(Row::Text(format!("{lead}{line}"), tone));
    }
}

impl PurgeScreen {
    /// The body as rows for a text width of `width`.
    fn rows(&self, width: usize, btn_w: usize) -> Vec<Row> {
        let mut rows = Vec::new();
        wrapped(&mut rows, p::INTRO.get(), width, Tone::Dim, "");
        wrapped(&mut rows, p::ROTATE_FIRST.get(), width, Tone::Warn, "");
        rows.push(Row::Gap);
        if let Some(out) = &self.outcome {
            let (text, tone) = match out.status {
                PurgeStatus::Purged => (p::done(out.old_objects), Tone::Good),
                _ => (p::incomplete(out.remaining), Tone::Bad),
            };
            wrapped(&mut rows, &text, width, tone, "");
            rows.push(Row::Gap);
        }
        let Some(check) = &self.check else {
            if self.check_wait.is_some() {
                rows.push(Row::Text(p::CHECKING.get().to_string(), Tone::Dim));
            }
            return rows;
        };
        if self.outcome.is_none() {
            if check.entries.is_empty() {
                rows.push(Row::Text(p::NOTHING.get().to_string(), Tone::Dim));
            } else {
                rows.push(Row::Text(p::UNDO_LOST.get().to_string(), Tone::Bold));
                for e in &check.entries {
                    let tone = if e.rewrite { Tone::Plain } else { Tone::Dim };
                    wrapped(&mut rows, &entry_line(e), width, tone, "  ");
                }
                rows.push(Row::Gap);
                let o = &check.outcome;
                wrapped(
                    &mut rows,
                    &p::objects_lost(o.old_commits, o.old_blobs),
                    width,
                    Tone::Plain,
                    "",
                );
                wrapped(
                    &mut rows,
                    &p::reflog_lost(o.reflog_entries, o.reflogs.len()),
                    width,
                    Tone::Plain,
                    "",
                );
                wrapped(&mut rows, p::NO_STOP.get(), width, Tone::Warn, "");
                rows.push(Row::Gap);
                if o.blockers.is_empty() {
                    rows.push(Row::Text(p::READY.get().to_string(), Tone::Good));
                } else {
                    rows.push(Row::Text(p::BLOCKERS_TITLE.get().to_string(), Tone::Bad));
                    for (idx, b) in o.blockers.iter().enumerate() {
                        let room = width.saturating_sub(btn_w + 4).max(20);
                        for (line, text) in wrap(&blocker_text(b), room).into_iter().enumerate() {
                            rows.push(Row::Blocker { idx, line, text });
                        }
                    }
                    wrapped(&mut rows, p::NOT_READY.get(), width, Tone::Dim, "");
                }
            }
        }
        if !check.hosts.is_empty() {
            rows.push(Row::Gap);
            rows.push(Row::Text(p::HOSTS_HEADING.get().to_string(), Tone::Bold));
            for (i, host) in check.hosts.iter().enumerate() {
                let title = p::host_title(&host.remote, host_name(host.host).get());
                rows.push(Row::Text(title, Tone::Bold));
                for step in &host.steps {
                    wrapped_with(
                        &mut rows,
                        host_step_text(*step).get(),
                        width.saturating_sub(2),
                        Tone::Plain,
                        "- ",
                        "  ",
                    );
                }
                if host_link(&host.steps).is_some() {
                    rows.push(Row::Link(i));
                }
            }
        }
        rows
    }
}

// ------------------------------------------------------------ the model

impl RewriteView {
    pub(crate) fn purge_open(&self) -> bool {
        self.purge.is_some()
    }

    /// A purge is running: the window cannot be closed or left.
    pub(crate) fn purging(&self) -> bool {
        self.purge.as_ref().is_some_and(PurgeScreen::running)
    }

    /// Opens the screen and asks for the check. `standalone`: closing it
    /// closes the workbench too (the palette's way in).
    pub(crate) fn open_purge(&mut self, standalone: bool) {
        if self.busy() {
            return;
        }
        self.purge = Some(PurgeScreen::new(standalone));
        self.purge_check();
    }

    /// The workbench opened on the purge screen (the palette row).
    pub(crate) fn with_purge(mut self) -> Self {
        self.open_purge(true);
        self
    }

    /// The screen with a check already answered (the layout tests of every
    /// language).
    #[cfg(test)]
    pub(crate) fn purge_for_layout(&mut self, check: PurgeCheck, confirm: Option<PurgeConfirm>) {
        let mut screen = PurgeScreen::new(false);
        screen.check = Some(check);
        screen.confirm = confirm;
        self.purge = Some(screen);
    }

    /// Asks the daemon what a purge would do and what blocks it.
    pub(crate) fn purge_check(&mut self) {
        let Some(s) = &mut self.purge else {
            return;
        };
        if s.check_wait.is_some() || s.run_wait.is_some() {
            return;
        }
        s.error = None;
        s.check_wait = Some(self.requester.call(
            OperationKind::RewritePurgeCheck,
            serde_json::to_value(&s.args).unwrap_or_default(),
        ));
    }

    /// Collects the answers. Never waits.
    pub(super) fn poll_purge(&mut self) {
        let Some(s) = &mut self.purge else {
            return;
        };
        if s.run_wait.is_some() {
            if let Some(progress) = crate::jobs_ui::running_progress(OperationKind::RewritePurge) {
                s.progress = Some(progress);
            }
        }
        let mut recheck = false;
        if let Some(answer) = s.check_wait.as_ref().and_then(|r| r.take()) {
            s.check_wait = None;
            match answer {
                Ok(check) => {
                    let ids = PurgeScreen::rewrite_ids(&check);
                    if !ids.is_empty() {
                        s.args = PurgeArgs {
                            entries: ids,
                            all: false,
                        };
                    }
                    let last = check.outcome.blockers.len().saturating_sub(1);
                    s.cursor = s.cursor.min(last);
                    s.check = Some(check);
                }
                Err(error) => {
                    s.error = Some(crate::daemon_link::error_text::error_text(&error));
                }
            }
        }
        if let Some(answer) = s.fix_wait.as_ref().and_then(|r| r.take()) {
            s.fix_wait = None;
            if let Err(error) = answer {
                s.error = Some(crate::daemon_link::error_text::error_text(&error));
            }
            recheck = true;
        }
        if let Some(answer) = s.run_wait.as_ref().and_then(|r| r.take()) {
            s.run_wait = None;
            s.progress = None;
            match answer {
                Ok(outcome) if outcome.status == PurgeStatus::Blocked => {
                    s.error = Some(p::OUTCOME_BLOCKED.get().to_string());
                    recheck = true;
                }
                Ok(outcome) => {
                    s.outcome = Some(outcome);
                    self.secrets.stale = true;
                    self.size.stale = true;
                }
                Err(error) => {
                    s.error = Some(crate::daemon_link::error_text::error_text(&error));
                    recheck = true;
                }
            }
        }
        if recheck {
            if let Some(s) = &mut self.purge {
                // Keep the failure shown: `purge_check` clears it.
                let keep = s.error.take();
                self.purge_check();
                if let Some(s) = &mut self.purge {
                    s.error = keep;
                }
            }
        }
    }

    /// The fix of blocker `index`: fetch, or (after a confirmation) drop.
    pub(crate) fn purge_fix(&mut self, index: usize) {
        let Some(s) = &mut self.purge else {
            return;
        };
        if s.busy() || s.confirm.is_some() || s.outcome.is_some() {
            return;
        }
        s.cursor = index;
        let Some(blocker) = s.blockers().get(index).cloned() else {
            return;
        };
        match fix_of(&blocker) {
            Fix::Fetch => {
                s.error = None;
                s.fix_wait = Some(self.requester.call(
                    OperationKind::FetchAll,
                    serde_json::json!({ "opts": { "prune": true } }),
                ));
            }
            Fix::Drop(id) => {
                s.confirm = Some(PurgeConfirm::Drop {
                    name: blocker.name,
                    id,
                    cursor: None,
                });
            }
            Fix::None => self.purge_copy(index),
        }
    }

    /// Copies the name of blocker `index`.
    pub(crate) fn purge_copy(&mut self, index: usize) {
        let Some(s) = &mut self.purge else {
            return;
        };
        let Some(name) = s.blockers().get(index).map(|b| b.name.clone()) else {
            return;
        };
        s.cursor = index;
        let said = crate::clipboard::copy_shared(p::WHAT_NAME.get(), &name);
        self.notice = Some(super::Notice {
            text: said,
            error: false,
        });
    }

    /// Copies the first link of host `index`'s steps.
    pub(crate) fn purge_copy_link(&mut self, index: usize) {
        let Some(link) = self
            .purge
            .as_ref()
            .and_then(|s| s.check.as_ref())
            .and_then(|c| c.hosts.get(index))
            .and_then(|h| host_link(&h.steps))
        else {
            return;
        };
        let said = crate::clipboard::copy_shared(p::WHAT_LINK.get(), &link);
        self.notice = Some(super::Notice {
            text: said,
            error: false,
        });
    }

    /// "Purge": asks the confirmation (the run is irreversible).
    pub(crate) fn purge_request(&mut self) {
        let Some(s) = &mut self.purge else {
            return;
        };
        if s.can_purge() {
            s.confirm = Some(PurgeConfirm::Purge { cursor: None });
        }
    }

    /// Answers the open confirmation with option `choice` (0: go ahead).
    pub(crate) fn purge_answer(&mut self, choice: usize) {
        let Some(s) = &mut self.purge else {
            return;
        };
        let Some(confirm) = s.confirm.take() else {
            return;
        };
        if choice != 0 {
            return;
        }
        match confirm {
            PurgeConfirm::Purge { .. } => {
                if !s.can_purge_after_confirm() {
                    return;
                }
                s.error = None;
                s.progress = None;
                s.run_wait = Some(self.requester.call(
                    OperationKind::RewritePurge,
                    serde_json::to_value(&s.args).unwrap_or_default(),
                ));
            }
            PurgeConfirm::Drop { id, .. } => {
                s.error = None;
                s.fix_wait = Some(
                    self.requester
                        .call(OperationKind::StashDrop, serde_json::json!({ "id": id })),
                );
            }
        }
    }

    /// Leaves the open confirmation unanswered.
    pub(crate) fn purge_dismiss_confirm(&mut self) {
        if let Some(s) = &mut self.purge {
            s.confirm = None;
        }
    }

    /// The cross, Esc and a click outside: closes the screen, unless the
    /// purge is running (it cannot be stopped).
    pub(crate) fn purge_close(&mut self) -> RewriteKeyOutcome {
        let Some(s) = &self.purge else {
            return RewriteKeyOutcome::Stay;
        };
        if s.running() {
            return RewriteKeyOutcome::Stay;
        }
        let standalone = s.standalone;
        self.purge = None;
        if standalone {
            RewriteKeyOutcome::Close
        } else {
            RewriteKeyOutcome::Stay
        }
    }

    pub(crate) fn purge_move(&mut self, delta: isize) {
        let Some(s) = &mut self.purge else {
            return;
        };
        let n = s.blockers().len();
        if n == 0 {
            s.scroll
                .set((s.scroll.get() as isize + delta).max(0) as usize);
            return;
        }
        s.cursor = (s.cursor as isize + delta).clamp(0, n as isize - 1) as usize;
        s.follow.set(true);
    }

    /// The wheel over the screen: scrolls the body.
    pub(crate) fn purge_wheel(&mut self, delta: isize) {
        if let Some(s) = &self.purge {
            s.scroll
                .set((s.scroll.get() as isize + delta).max(0) as usize);
        }
    }

    fn purge_page(&mut self, sign: isize) {
        if let Some(s) = &self.purge {
            let page = s.room.get().max(2) as isize - 1;
            s.scroll
                .set((s.scroll.get() as isize + sign * page).max(0) as usize);
        }
    }

    // --------------------------------------------------------------- keys

    /// A key while the purge screen is open.
    pub(super) fn on_purge_key(&mut self, key: KeyEvent) -> RewriteKeyOutcome {
        let Some(s) = &mut self.purge else {
            return RewriteKeyOutcome::Stay;
        };
        if let Some(confirm) = &mut s.confirm {
            let options = 2;
            match key.code {
                KeyCode::Esc => s.confirm = None,
                KeyCode::Up | KeyCode::Char('k') => {
                    let c = confirm.cursor_mut();
                    *c = Some(c.map_or(options - 1, |c| c.saturating_sub(1)));
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let c = confirm.cursor_mut();
                    *c = Some(c.map_or(0, |c| (c + 1).min(options - 1)));
                }
                KeyCode::Char(d @ '1'..='2') => self.purge_answer(d as usize - '1' as usize),
                KeyCode::Enter => {
                    if let Some(at) = *confirm.cursor_mut() {
                        self.purge_answer(at);
                    }
                }
                _ => {}
            }
            return RewriteKeyOutcome::Stay;
        }
        if s.running() {
            // It cannot be stopped: every key waits.
            return RewriteKeyOutcome::Stay;
        }
        let plain = !key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
        let at = s.cursor;
        let done = s.outcome.is_some();
        match key.code {
            KeyCode::Esc | KeyCode::Backspace => return self.purge_close(),
            KeyCode::Up | KeyCode::Char('k') if plain => self.purge_move(-1),
            KeyCode::Down | KeyCode::Char('j') if plain => self.purge_move(1),
            KeyCode::PageUp => self.purge_page(-1),
            KeyCode::PageDown => self.purge_page(1),
            KeyCode::Home => {
                if let Some(s) = &mut self.purge {
                    s.cursor = 0;
                    s.scroll.set(0);
                }
            }
            KeyCode::End => {
                let last = self.purge.as_ref().map_or(0, |s| s.blockers().len());
                if let Some(s) = &mut self.purge {
                    s.cursor = last.saturating_sub(1);
                    s.follow.set(true);
                }
            }
            KeyCode::Enter if plain && !done => self.purge_fix(at),
            KeyCode::Char('c') if plain && !done => self.purge_copy(at),
            KeyCode::Char('r') | KeyCode::F(5) if plain && !done => self.purge_check(),
            KeyCode::Char('p') if plain => self.purge_request(),
            _ => {}
        }
        RewriteKeyOutcome::Stay
    }

    /// A press on one of the screen's parts.
    pub(super) fn on_purge_press(&mut self, hit: RewriteHit) -> RewriteKeyOutcome {
        match hit {
            RewriteHit::PurgeClose | RewriteHit::PurgeDone => return self.purge_close(),
            RewriteHit::PurgeShield => {
                if self.purge.as_ref().is_some_and(|s| s.confirm.is_some()) {
                    self.purge_dismiss_confirm();
                } else {
                    return self.purge_close();
                }
            }
            RewriteHit::PurgeCancel => self.purge_dismiss_confirm(),
            RewriteHit::PurgeAnswer(i) => self.purge_answer(i),
            RewriteHit::PurgeRow(i) => {
                if let Some(s) = &mut self.purge {
                    s.cursor = i;
                }
            }
            RewriteHit::PurgeFix(i) => self.purge_fix(i),
            RewriteHit::PurgeCopy(i) => self.purge_copy(i),
            RewriteHit::PurgeLink(i) => self.purge_copy_link(i),
            RewriteHit::PurgeRun => self.purge_request(),
            RewriteHit::PurgeCheck => self.purge_check(),
            _ => {}
        }
        RewriteKeyOutcome::Stay
    }

    /// A press outside the workbench's frame: leaves a confirmation, or
    /// closes the screen (nothing is lost by closing it).
    pub(super) fn purge_outside_press(&mut self) -> RewriteKeyOutcome {
        if self.purge.as_ref().is_some_and(|s| s.confirm.is_some()) {
            self.purge_dismiss_confirm();
            return RewriteKeyOutcome::Stay;
        }
        self.purge_close()
    }
}

impl PurgeScreen {
    /// The confirmation was answered yes: the check must still be good.
    fn can_purge_after_confirm(&self) -> bool {
        self.outcome.is_none()
            && !self.busy()
            && self.check.as_ref().is_some_and(PurgeCheck::can_run)
    }
}

// ----------------------------------------------------------------- paint

fn tone_style(tone: Tone) -> Style {
    match tone {
        Tone::Plain => plain(),
        Tone::Dim => dim(),
        Tone::Warn => warn(),
        Tone::Bad => bad(),
        Tone::Good => good(),
        Tone::Bold => bold(false),
    }
}

impl RewriteView {
    /// Paints the screen over the workbench and returns its click targets:
    /// the cross, the buttons, the rows, the panel and a shield over the
    /// rest (a click there closes it).
    pub(super) fn paint_purge(
        &self,
        buf: &mut Buffer,
        bounds: Rect,
        hovered: Option<RewriteHit>,
    ) -> Vec<(Rect, RewriteHit)> {
        let mut hits: Vec<(Rect, RewriteHit)> = Vec::new();
        let Some(s) = &self.purge else {
            return hits;
        };
        let rect = Rect::new(
            bounds.x + 2,
            bounds.y + 1,
            bounds.width.saturating_sub(4),
            bounds.height.saturating_sub(2),
        );
        if rect.width < 50 || rect.height < 12 {
            hits.push((bounds, RewriteHit::PurgeShield));
            return hits;
        }
        dialog_kit::panel(buf, rect, p::SCREEN_TITLE.get());
        let cross =
            crate::chrome::close_cross::paint(buf, rect, hovered == Some(RewriteHit::PurgeClose));
        let inner = Rect::new(rect.x + 1, rect.y + 1, rect.width - 2, rect.height - 2);
        let body = Rect::new(
            inner.x + 1,
            inner.y,
            inner.width - 2,
            inner.height.saturating_sub(4),
        );
        let hot = |h: RewriteHit| hovered == Some(h);
        let btn = |buf: &mut Buffer,
                   x: u16,
                   y: u16,
                   label: &str,
                   h: RewriteHit,
                   primary: bool,
                   on: bool| {
            let state = if !on {
                ButtonState::Disabled
            } else if primary {
                ButtonState::Primary {
                    hovered: hot(h),
                    focused: false,
                }
            } else {
                ButtonState::Normal {
                    hovered: hot(h),
                    focused: false,
                }
            };
            (button(buf, x, y, label, state), h)
        };

        // The body, scrolled.
        let copy_w = button_width(p::BUTTON_COPY.get());
        let fix_w = button_width(p::BUTTON_FETCH.get()).max(button_width(p::BUTTON_DROP.get()));
        let rows = s.rows(body.width as usize, (copy_w + fix_w + 1) as usize);
        let room = body.height as usize;
        s.room.set(room);
        let mut scroll = s.scroll.get();
        if s.follow.take() {
            let first = rows
                .iter()
                .position(|r| matches!(r, Row::Blocker { idx, .. } if *idx == s.cursor));
            let last = rows
                .iter()
                .rposition(|r| matches!(r, Row::Blocker { idx, .. } if *idx == s.cursor));
            if let (Some(first), Some(last)) = (first, last) {
                if first < scroll {
                    scroll = first;
                } else if last >= scroll + room {
                    scroll = last + 1 - room;
                }
            }
        }
        scroll = scroll.min(rows.len().saturating_sub(room));
        s.scroll.set(scroll);
        let mut fine: Vec<(Rect, RewriteHit)> = Vec::new();
        let mut coarse: Vec<(Rect, RewriteHit)> = Vec::new();
        self.paint_bar_on(
            buf,
            &mut fine,
            hot(RewriteHit::Bar(BarId::Purge)),
            BarId::Purge,
            (inner.right(), body.y, body.height),
            rows.len(),
            room,
            scroll,
            None,
        );
        let blockers = s.blockers();
        for (slot, row) in rows.iter().enumerate().skip(scroll).take(room) {
            let y = body.y + (slot - scroll) as u16;
            match row {
                Row::Gap => {}
                Row::Text(text, tone) => {
                    dialog_kit::put(buf, body.x, y, text, body.width, tone_style(*tone));
                }
                Row::Link(i) => {
                    let label = p::BUTTON_COPY_LINK.get();
                    let h = RewriteHit::PurgeLink(*i);
                    let (r, h) = btn(buf, body.x + 2, y, label, h, false, true);
                    fine.push((r, h));
                }
                Row::Blocker { idx, line, text } => {
                    let h = RewriteHit::PurgeRow(*idx);
                    let bg = row_bg(*idx == s.cursor, hot(h));
                    let band = Rect::new(inner.x, y, inner.width, 1);
                    dialog_kit::fill(buf, band, Style::default().bg(bg));
                    let text_w = body.width.saturating_sub(copy_w + fix_w + 3);
                    dialog_kit::put(
                        buf,
                        body.x + 2,
                        y,
                        text,
                        text_w,
                        Style::default().fg(th().ui.text).bg(bg),
                    );
                    if *line == 0 {
                        let enabled = !s.busy() && s.confirm.is_none() && s.outcome.is_none();
                        let mut x = body.right().saturating_sub(copy_w);
                        let (r, h2) = btn(
                            buf,
                            x,
                            y,
                            p::BUTTON_COPY.get(),
                            RewriteHit::PurgeCopy(*idx),
                            false,
                            true,
                        );
                        fine.push((r, h2));
                        let fix = blockers.get(*idx).map(fix_of).unwrap_or(Fix::None);
                        let label = match fix {
                            Fix::Fetch => Some(p::BUTTON_FETCH),
                            Fix::Drop(_) => Some(p::BUTTON_DROP),
                            Fix::None => None,
                        };
                        if let Some(label) = label {
                            x = x.saturating_sub(fix_w + 1);
                            let (r, h2) = btn(
                                buf,
                                x,
                                y,
                                label.get(),
                                RewriteHit::PurgeFix(*idx),
                                true,
                                enabled,
                            );
                            fine.push((r, h2));
                        }
                    }
                    coarse.push((band, h));
                }
            }
        }

        // The footer: a failure, the hint or the running line, the buttons.
        let msg_y = inner.bottom() - 3;
        let hint_y = inner.bottom() - 2;
        let btn_y = inner.bottom() - 1;
        if s.running() {
            let (phase, percent) = s.progress.clone().unwrap_or((None, None));
            let step = phase
                .as_deref()
                .map(step_text)
                .unwrap_or_else(|| p::STEP_CHECKING.get().to_string());
            dialog_kit::put(
                buf,
                body.x,
                msg_y,
                &p::running(step, percent.unwrap_or(0)),
                body.width,
                bold(true),
            );
            dialog_kit::put(buf, body.x, hint_y, p::NO_STOP.get(), body.width, warn());
        } else {
            if let Some(error) = &s.error {
                dialog_kit::put(buf, body.x, msg_y, error, body.width, bad());
            } else if s.check_wait.is_some() && s.check.is_some() {
                dialog_kit::put(buf, body.x, msg_y, p::CHECKING.get(), body.width, dim());
            }
            dialog_kit::put(buf, body.x, hint_y, p::HINT.get(), body.width, dim());
            let mut x = body.x;
            if s.outcome.is_none() {
                let (r, h) = btn(
                    buf,
                    x,
                    btn_y,
                    p::BUTTON_PURGE.get(),
                    RewriteHit::PurgeRun,
                    true,
                    s.can_purge(),
                );
                x += r.width + 1;
                fine.push((r, h));
                let (r, h) = btn(
                    buf,
                    x,
                    btn_y,
                    p::BUTTON_CHECK.get(),
                    RewriteHit::PurgeCheck,
                    false,
                    !s.busy(),
                );
                x += r.width + 1;
                fine.push((r, h));
            }
            let (r, h) = btn(
                buf,
                x,
                btn_y,
                t::BTN_CLOSE.get(),
                RewriteHit::PurgeDone,
                s.outcome.is_some(),
                true,
            );
            fine.push((r, h));
        }

        // Front to back: the confirmation, the cross, the buttons, the
        // rows, the panel, the rest of the workbench.
        if let Some(confirm) = &s.confirm {
            hits.extend(self.paint_purge_confirm(buf, bounds, confirm, hovered));
            hits.push((bounds, RewriteHit::PurgeShield));
            return hits;
        }
        if let Some(cross) = cross {
            hits.push((cross, RewriteHit::PurgeClose));
        }
        hits.extend(fine);
        hits.extend(coarse);
        hits.push((rect, RewriteHit::PurgeBody));
        hits.push((bounds, RewriteHit::PurgeShield));
        hits
    }

    /// The confirmation: what is lost, numbered answers, no default.
    fn paint_purge_confirm(
        &self,
        buf: &mut Buffer,
        bounds: Rect,
        confirm: &PurgeConfirm,
        hovered: Option<RewriteHit>,
    ) -> Vec<(Rect, RewriteHit)> {
        let mut hits = Vec::new();
        let width = bounds.width.saturating_sub(6).clamp(30, 80);
        let text_w = width as usize - 4;
        let mut lines: Vec<(String, Style)> = Vec::new();
        let (title, options, cursor): (String, [M; 2], Option<usize>) = match confirm {
            PurgeConfirm::Purge { cursor } => {
                for l in wrap(p::CONFIRM_INTRO.get(), text_w) {
                    lines.push((l, plain()));
                }
                if let Some(check) = self.purge.as_ref().and_then(|s| s.check.as_ref()) {
                    lines.push((String::new(), plain()));
                    lines.push((p::confirm_undo(check.entries.len() as u64), bold(false)));
                    for e in check.entries.iter().take(6) {
                        for l in wrap(&entry_line(e), text_w - 2) {
                            lines.push((format!("  {l}"), plain()));
                        }
                    }
                    if check.entries.len() > 6 {
                        lines.push((format!("  (+{})", check.entries.len() - 6), dim()));
                    }
                    let o = &check.outcome;
                    lines.push((String::new(), plain()));
                    for l in wrap(&p::objects_lost(o.old_commits, o.old_blobs), text_w) {
                        lines.push((l, plain()));
                    }
                    for l in wrap(&p::reflog_lost(o.reflog_entries, o.reflogs.len()), text_w) {
                        lines.push((l, plain()));
                    }
                }
                (
                    p::CONFIRM_TITLE.get().to_string(),
                    [p::CONFIRM_YES, p::CONFIRM_NO],
                    *cursor,
                )
            }
            PurgeConfirm::Drop { name, cursor, .. } => {
                for l in wrap(&p::drop_prompt(name), text_w) {
                    lines.push((l, plain()));
                }
                (p::drop_title(name), [p::DROP_YES, p::DROP_NO], *cursor)
            }
        };
        let height = (lines.len() + options.len() + 5).min(bounds.height as usize) as u16;
        let rect = WIN_PURGE.place(bounds, dialog_kit::centered(bounds, width, height));
        dialog_kit::panel(buf, rect, &title);
        if let Some(cross) =
            crate::chrome::close_cross::paint(buf, rect, hovered == Some(RewriteHit::PurgeCancel))
        {
            WIN_PURGE.decorate(buf, rect, Some(cross), true);
            hits.push((cross, RewriteHit::PurgeCancel));
        }
        let last_text = rect.bottom().saturating_sub(options.len() as u16 + 2);
        for (y, (line, style)) in (rect.y + 2..last_text).zip(&lines) {
            dialog_kit::put(buf, rect.x + 2, y, line, rect.width - 4, *style);
        }
        let first_option = rect.bottom() - 1 - options.len() as u16;
        for (i, option) in options.iter().enumerate() {
            let y = first_option + i as u16;
            let band = Rect::new(rect.x + 1, y, rect.width - 2, 1);
            let h = RewriteHit::PurgeAnswer(i);
            let bg = row_bg(cursor == Some(i), hovered == Some(h));
            dialog_kit::fill(buf, band, Style::default().bg(bg));
            dialog_kit::put(
                buf,
                band.x + 1,
                y,
                &format!("{}  {}", i + 1, option.get()),
                band.width - 2,
                Style::default().fg(th().ui.text).bg(bg),
            );
            hits.push((band, h));
        }
        hits.push((rect, RewriteHit::PurgeBody));
        hits
    }
}

/// The dialog moves (its size follows its content).
const WIN_PURGE: crate::dialog_kit::Win = crate::dialog_kit::Win::movable("rewrite_purge");
