//! Live updates and the write path of the window.
//!
//! The link's thread wakes the link's [`crate::daemon::DesktopHost`] whenever it queued something. A
//! task on the window's foreground executor waits for that wake ([`Workspace::start_pump`]), takes what
//! the link has ([`gitpoulpi_link::DaemonLink::drain`]) and applies it: a refresh the daemon pushed
//! because *any* client changed the repository reloads only the panels its [`RefreshSummary`] says
//! changed (the Log, the Refs sidebar, the working tree), the undo journal moving redraws the toolbar,
//! and failures and messages of actions go to the status line (`commands::status`, the one message
//! channel). Nothing here polls: no wake, no work.

use gitpoulpi_core::domain::OperationState;
use gitpoulpi_link::{LinkEvent, RefreshSummary};
use gpui_kit::Context;

use super::workspace::Workspace;
use crate::commands::status::{self, Notice};
use crate::daemon::host::rejection_text;
use crate::daemon::transfer::Transfer;
use crate::strings;

/// What the window remembers between two wakes.
#[derive(Default)]
pub struct LiveState {
    /// Something reached the status line since the last transfer started (so a late "started" line
    /// does not overwrite a result that already arrived).
    settled: bool,
    /// Refreshes applied to the panels, for tests and diagnostics.
    pub refreshes: u64,
}

/// Everything one wake took out of the link.
#[derive(Default)]
struct Drained {
    /// What the refreshes since the last wake changed, all of them together.
    changed: Option<RefreshSummary>,
    /// The merge, cherry-pick, revert or rebase in progress as the newest refresh read it (`None`
    /// inside: nothing is in progress). Absent when no refresh arrived or the read failed.
    operation: Option<Option<OperationState>>,
    errors: Vec<String>,
    messages: Vec<String>,
    failure: Option<String>,
    /// Another frontend hid, showed or soloed refs.
    visibility: bool,
    /// Local changes are in the way of an action and auto-stash is `ask`.
    autostash: Vec<gitpoulpi_link::AutoStashNeeded>,
    /// Pushes the remote refused: the question (pull first, force push, cancel) is asked once.
    rejected: Vec<gitpoulpi_link::PushRejectedNotice>,
    /// Undos and redos the daemon found risky.
    undo_confirms: Vec<gitpoulpi_link::undo_ui::PendingConfirm>,
    /// The newest LFS snapshot a refresh carried.
    lfs: Option<gitpoulpi_link::lfs_ui::LfsSnapshot>,
}

/// `a` and `b` together: a part changed when it changed in either.
fn both(a: RefreshSummary, b: RefreshSummary) -> RefreshSummary {
    RefreshSummary {
        first: a.first || b.first,
        log: a.log || b.log,
        head: a.head || b.head,
        refs: a.refs || b.refs,
        working_tree: a.working_tree || b.working_tree,
        stashes: a.stashes || b.stashes,
        worktrees: a.worktrees || b.worktrees,
        submodules: a.submodules || b.submodules,
        conflicts: a.conflicts || b.conflicts,
        remotes: a.remotes || b.remotes,
        config: a.config || b.config,
        lfs: a.lfs || b.lfs,
    }
}

fn sort(events: Vec<LinkEvent>) -> Drained {
    let mut d = Drained::default();
    for event in events {
        match event {
            LinkEvent::Refreshed(outcome) => {
                // The banner follows the newest outcome, the first one included: it is the only
                // place the window learns an operation another frontend left stopped.
                if let Ok(state) = &outcome.operation_state_result {
                    d.operation = Some(state.clone());
                }
                if let Some(lfs) = &outcome.lfs {
                    d.lfs = Some(lfs.clone());
                }
                // The first outcome only says what the panels loaded themselves just now.
                if !outcome.summary.first {
                    d.changed = Some(match d.changed {
                        Some(seen) => both(seen, outcome.summary),
                        None => outcome.summary,
                    });
                }
            }
            LinkEvent::ActionError(text) => d.errors.push(text),
            LinkEvent::PushRejected(notice) => {
                d.errors.push(rejection_text(&notice.rejection));
                d.rejected.push(notice);
            }
            LinkEvent::UndoConfirm(confirm) => d.undo_confirms.push(confirm),
            LinkEvent::ActionMessage(text) => d.messages.push(text),
            LinkEvent::RefVisibilityChanged => d.visibility = true,
            LinkEvent::AutoStashNeeded(needed) => d.autostash.push(needed),
            LinkEvent::Failed(why) => d.failure = Some(why),
            // The journal moving is the wake itself: the toolbar asks the link when it draws. The
            // other events (auto-stash, file insight, merge, ...) have no desktop surface yet.
            _ => {}
        }
    }
    d
}

impl Workspace {
    /// Starts the task that applies what the link queues. Call once from `Workspace::new`.
    pub(super) fn start_pump(&mut self, cx: &mut Context<Self>) {
        let notifier = self.backend.notifier().clone();
        let mut seen = notifier.epoch();
        self._pump = Some(cx.spawn(async move |this, cx| loop {
            notifier.changed(&mut seen).await;
            if this.update(cx, |this, cx| this.on_link_wake(cx)).is_err() {
                break;
            }
        }));
    }

    fn on_link_wake(&mut self, cx: &mut Context<Self>) {
        let drained = sort(self.backend.with_link(|link| link.drain()));
        let host = self.backend.host();
        let news = host.take_news();
        // An action's error outranks its message, then what the host said, then a success; the link's
        // own failure only when nothing else spoke.
        let notice = drained
            .errors
            .into_iter()
            .last()
            .map(Notice::error)
            .or_else(|| drained.messages.into_iter().last().map(Notice::info))
            .or_else(|| news.said.into_iter().last().map(Notice::info))
            .or_else(|| {
                news.done
                    .map(|title| Notice::info(strings::finished(&title)))
            })
            .or_else(|| drained.failure.map(Notice::error));
        if let Some(n) = notice {
            self.live.settled = true;
            status::set_notice(cx, n);
        }
        if let Some(state) = drained.operation {
            self.operation = state;
            let merging = matches!(
                self.operation,
                Some(gitpoulpi_core::domain::OperationState::Merge { .. })
            );
            self.right.update(cx, |r, cx| r.set_merging(merging, cx));
        }
        if drained.visibility {
            self.refs.update(cx, |r, cx| r.on_visibility_event(cx));
        }
        if let Some(lfs) = drained.lfs {
            self.set_lfs_snapshot(lfs, cx);
        }
        if let Some(changed) = drained.changed {
            self.apply_changes(changed, cx);
        }
        // What became of the commit that was sent, and the stages the guard refused.
        self.right.update(cx, |r, cx| r.settle(cx));
        if !drained.autostash.is_empty() {
            self.ask_autostash(drained.autostash, cx);
        }
        if !drained.rejected.is_empty() || !drained.undo_confirms.is_empty() {
            self.ask_link_questions(drained.rejected, drained.undo_confirms, cx);
        }
        let prompts = host.take_prompts();
        if !prompts.is_empty() {
            self.ask_sensitive(prompts, cx);
        }
        cx.notify();
    }

    /// The questions the link raises about a refused push and a risky undo.
    fn ask_link_questions(
        &mut self,
        rejected: Vec<gitpoulpi_link::PushRejectedNotice>,
        confirms: Vec<gitpoulpi_link::undo_ui::PendingConfirm>,
        cx: &mut Context<Self>,
    ) {
        use crate::panels::activity::resolve;
        let (window, backend) = (self.window, self.backend.clone());
        cx.defer(move |cx| {
            let _ = window.update(cx, move |_, window, cx| {
                for notice in rejected {
                    resolve::ask_rejected(notice, backend.clone(), window, cx);
                }
                for confirm in confirms {
                    resolve::ask_undo_confirm(confirm, backend.clone(), window, cx);
                }
            });
        });
    }

    /// Reloads the panels the refresh says changed, and only those.
    fn apply_changes(&mut self, changed: RefreshSummary, cx: &mut Context<Self>) {
        // The Log shows the commits, the ref chips, the stash rows and the WIP row.
        let log =
            changed.log || changed.head || changed.refs || changed.stashes || changed.working_tree;
        let refs = changed.refs
            || changed.head
            || changed.stashes
            || changed.worktrees
            || changed.submodules;
        if log {
            self.log.update(cx, |l, cx| l.reload(cx));
        }
        if changed.working_tree {
            self.refresh_working_diff(cx);
        }
        if refs {
            self.refs.update(cx, |r, cx| r.reload(cx));
        }
        if log || refs {
            self.live.refreshes += 1;
        }
    }

    /// The window's link to the daemon.
    pub fn backend(&self) -> &crate::daemon::Shared {
        &self.backend
    }

    /// The last line the status line shows (an action's result, a command's answer).
    pub fn notice(&self, cx: &gpui_kit::App) -> Option<Notice> {
        status::notice(cx)
    }

    /// How many times a daemon-pushed refresh reloaded the panels.
    pub fn live_refreshes(&self) -> u64 {
        self.live.refreshes
    }

    /// Starts Fetch, Pull, Push, Undo or Redo; the status line follows what the daemon does.
    pub fn transfer(&mut self, what: Transfer, cx: &mut Context<Self>) {
        status::clear(cx);
        self.live.settled = false;
        let backend = self.backend.clone();
        cx.spawn(async move |this, cx| {
            let started = backend.start_transfer(what).await;
            let _ = this.update(cx, |this, cx| {
                match started {
                    Ok(text) if !this.live.settled => status::set(cx, text),
                    Ok(_) => {}
                    Err(why) => status::set_error(cx, why),
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// The repository switch restarts the link: its refreshes start from scratch.
    pub(super) fn reset_live(&mut self, cx: &mut Context<Self>) {
        self.live = LiveState::default();
        self.operation = None;
        self.right.update(cx, |r, cx| r.set_merging(false, cx));
        status::clear(cx);
    }
}
