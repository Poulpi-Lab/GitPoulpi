use super::*;
use tokio::net::tcp::OwnedReadHalf;

/// The link's reader task: the only reader of the daemon socket. It hands
/// each `Response` to the request waiting for it and turns each pushed
/// `Event` for this repository into state the render loop reads (the LFS
/// gate, the undo journal) or into a refresh request on `event_refresh_tx`.
///
/// A new inbound event kind is one more `Some(ServerMessage::Event(..))`
/// arm below. It must never await a request of its own: this task is what
/// completes every waiting request.
///
/// Returns once the connection is gone, after failing every waiting request
/// (see `PendingRequests`); dropping `event_refresh_tx` tells the link loop.
pub(super) async fn read_loop(
    mut read_half: OwnedReadHalf,
    pending: PendingMap,
    repo_id: gitpoulpi_core::domain::RepoId,
    lfs_gate: Arc<crate::lfs_ui::LfsGate>,
    activity: Arc<crate::activity::Activity>,
    event_refresh_tx: tokio::sync::mpsc::UnboundedSender<ChangeScope>,
) {
    let reason = loop {
        let msg: Option<ServerMessage> = match read_message(&mut read_half).await {
            Ok(m) => m,
            Err(e) => break gitpoulpi_i18n::msg::link_error::read_failed(e),
        };
        match msg {
            Some(ServerMessage::Response(resp)) => {
                if let Some(tx) = pending.lock().unwrap().waiting.remove(&resp.request_id) {
                    let _ = tx.send(resp);
                }
            }
            // Both events mean "look again". `ConflictDetected` stays separate from
            // `RepoChanged` because it also fires for an operation that changed nothing
            // (a `ResolveConflict` that left other files unmerged), and the pane that
            // draws the remaining conflicts still has to hear about it.
            Some(ServerMessage::Event(Event::RepoChanged {
                repo: changed,
                scope,
            })) if changed == repo_id => {
                if scope.touches_lfs() {
                    lfs_gate.mark_dirty();
                }
                let _ = event_refresh_tx.send(scope);
            }
            Some(ServerMessage::Event(Event::ConflictDetected { repo: changed, .. }))
                if changed == repo_id =>
            {
                let _ = event_refresh_tx
                    .send(ChangeScope::INDEX | ChangeScope::WORKTREE | ChangeScope::CONFLICTS);
            }
            // Another frontend (or this one) recorded, undid or
            // redid something, or an outside change flipped the
            // blocked state: the buttons follow at once.
            Some(ServerMessage::Event(Event::JournalChanged {
                repo: changed,
                state,
            })) if changed == repo_id => {
                apply_journal(&activity, state);
            }
            // `gitpoulpi refs hide|show|solo` (or another TUI): the render
            // loop reloads the filter from the repo state.
            Some(ServerMessage::Event(Event::RefVisibilityChanged { repo: changed }))
                if changed == repo_id =>
            {
                activity.mark_ref_visibility_changed();
            }
            // The CLI or the MCP server posted, edited or deleted a comment:
            // an open conversation of that item reads itself again.
            Some(ServerMessage::Event(Event::HostingItemChanged {
                repo: changed,
                change,
            })) if changed == repo_id => {
                activity.mark_hosting_item_changed(change);
            }
            // The daemon wrote, changed or cleared the repo's pending
            // auto-stash record: the render loop re-reads it, so the
            // banner appears and vanishes live.
            Some(ServerMessage::Event(Event::PendingAutoStashChanged { repo: changed }))
                if changed == repo_id =>
            {
                activity.mark_pending_auto_stash_changed();
            }
            // The record's stash vanished outside GitPoulpi and the daemon
            // cleared the record: the render loop says so on the status line.
            Some(ServerMessage::Event(Event::PendingAutoStashGone {
                repo: changed,
                stash,
            })) if changed == repo_id => {
                activity.mark_pending_auto_stash_gone(stash.0);
            }
            // An LFS transfer report: kept in the gate for the
            // render loop to read (latest value wins).
            Some(ServerMessage::Event(Event::LfsProgress {
                repo: changed,
                operation,
                progress,
            })) if changed == repo_id => {
                lfs_gate.set_progress(operation, progress);
            }
            // A grouped push moved one of its targets: the push-group dialog's
            // rows (also while it is closed, so reopening shows the truth).
            Some(ServerMessage::Event(Event::RemoteGroupProgress {
                repo: changed,
                run_id,
                remote,
                status,
            })) if changed == repo_id => {
                super::host::push_group(super::host::PushGroupEvent::Progress {
                    run_id: &run_id,
                    remote: &remote,
                    status,
                });
            }
            // A history rewrite preview's progress: kept for the workbench
            // to read each frame (latest value wins).
            Some(ServerMessage::Event(Event::RewriteProgress {
                repo: changed,
                job,
                progress,
            })) if changed == repo_id => {
                activity.set_rewrite_progress(job, progress);
            }
            // The daemon installed the LFS filters before a pull or push:
            // the render loop says so on the status line.
            Some(ServerMessage::Event(Event::LfsFiltersPosed { repo: changed }))
                if changed == repo_id =>
            {
                lfs_gate.set_filters_posed();
            }
            // The updater is replacing GitPoulpi: leave for good rather than
            // restart the daemon it is stopping (the install-channels design).
            Some(ServerMessage::Event(Event::ShuttingDown { reason, .. }))
                if reason == gitpoulpi_ipc::ShutdownArgs::UPDATE =>
            {
                quit_for_update()
            }
            // The daemon is about to exit (a `gitpoulpi daemon stop`/`restart` run
            // from elsewhere). Takes the same stopped-link path as a dropped
            // connection (`r`/Refresh starts a fresh daemon), and reacting to the
            // event saves the ~150ms the daemon spends flushing this broadcast before
            // it tears down.
            Some(ServerMessage::Event(Event::DaemonStopping { .. })) => {
                break gitpoulpi_i18n::msg::link_error::DAEMON_STOPPING
                    .get()
                    .to_string()
            }
            // An agent asks this window (this process, by viewer id) to
            // move its view or describe it: handed to the render loop.
            Some(ServerMessage::Event(Event::UiIntent {
                viewer,
                intent_id,
                intent,
                origin,
            })) if viewer == *crate::ui_viewer::viewer_id() => {
                activity.ui.push_inbound(crate::ui_viewer::Inbound::Intent {
                    id: intent_id,
                    intent: Box::new(intent),
                    origin,
                });
            }
            // A launcher click found GitPoulpi open and asks this window to come
            // to the front. The methods run programs, so off this task (it must
            // never block); the answer tells the launcher whether to start a
            // new window instead.
            Some(ServerMessage::Event(Event::RaiseWindow { viewer, raise_id }))
                if viewer == *crate::ui_viewer::viewer_id() =>
            {
                let activity = Arc::clone(&activity);
                let link = super::link_ctx::current();
                std::thread::spawn(move || {
                    let _in_link = link.as_ref().map(|link| link.enter());
                    let result = super::host::raise_window();
                    match &result {
                        Ok(how) => tracing::info!(how, "raised the window for a launcher click"),
                        Err(why) => tracing::info!(why, "could not raise the window"),
                    }
                    activity.ui.report_raise(raise_id, result);
                });
            }
            Some(ServerMessage::Event(Event::UiStateRequest {
                viewer,
                query_id,
                include_text,
            })) if viewer == *crate::ui_viewer::viewer_id() => {
                activity.ui.push_inbound(crate::ui_viewer::Inbound::State {
                    id: query_id,
                    include_text,
                });
            }
            // The session's background jobs (clone, fetch, push...): kept in
            // the process-wide cell every window paints its indicator from.
            Some(ServerMessage::Event(Event::JobChanged { job })) => {
                super::host::jobs(super::host::JobEvent::Changed(job))
            }
            Some(ServerMessage::Event(Event::JobRemoved { id })) => {
                super::host::jobs(super::host::JobEvent::Removed(id))
            }
            Some(_) => {}
            // The daemon closing the connection on us: a daemon that stopped, or one
            // of a *different GitPoulpi build* dropping a request it cannot parse (an
            // operation it does not know) without answering.
            None => {
                break gitpoulpi_i18n::msg::link_error::DAEMON_CLOSED
                    .get()
                    .to_string()
            }
        }
    };
    // The one place a waiting request can still be completed is
    // this task; it is ending, so fail them all now (see
    // `PendingRequests`). Dropping `event_refresh_tx` right after
    // tells the loop below the connection is gone.
    pending.lock().unwrap().close(reason);
    drop(event_refresh_tx);
}

/// Ends this process for an update. The session (tabs, layout, filters) is
/// written as it changes, so there is nothing left to save; what remains is
/// handing the terminal back and saying why, since the updater takes this
/// terminal over next on Unix.
fn quit_for_update() -> ! {
    super::host::before_exit();
    eprintln!("gitpoulpi: closing for an update"); // i18n-english
    std::process::exit(0)
}

/// Stores the daemon's journal report and, when it differs from the held one
/// and the host asked for it (`LinkHost::wake_on_journal`), wakes the host:
/// the undo and redo buttons of a frontend that waits on `wake` follow at once.
pub(super) fn apply_journal(
    activity: &crate::activity::Activity,
    state: gitpoulpi_core::domain::JournalState,
) {
    if activity.journal.set(state) && super::host::wake_on_journal() {
        super::host::wake();
    }
}
