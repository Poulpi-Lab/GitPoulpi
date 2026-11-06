use super::*;
use crate::redraw::NotifySend as _;

pub(super) const AUTO_REFRESH_INTERVAL: Duration = Duration::from_secs(2);

/// The link thread's ends of every channel `DaemonLink` talks over, and the
/// state it shares with the handle.
pub(super) struct LinkChannels {
    pub(super) manual_refresh_rx: tokio::sync::mpsc::UnboundedReceiver<()>,
    pub(super) action_rx: tokio::sync::mpsc::UnboundedReceiver<Action>,
    pub(super) commit_files_task_rx: tokio::sync::mpsc::UnboundedReceiver<CommitFilesRequest>,
    pub(super) working_diff_task_rx: tokio::sync::mpsc::UnboundedReceiver<FileDiffRequest>,
    pub(super) file_insight_task_rx: tokio::sync::mpsc::UnboundedReceiver<FileInsightRequest>,
    pub(super) merge_task_rx: tokio::sync::mpsc::UnboundedReceiver<MergeRequest>,
    pub(super) log_page_task_rx: tokio::sync::mpsc::UnboundedReceiver<crate::graph::LogPageRequest>,
    pub(super) outcome_tx: std_mpsc::Sender<RefreshOutcome>,
    pub(super) action_error_tx: std_mpsc::Sender<String>,
    pub(super) action_message_tx: std_mpsc::Sender<String>,
    pub(super) push_rejected_tx: std_mpsc::Sender<PushRejection>,
    pub(super) autostash_tx: std_mpsc::Sender<AutoStashNeeded>,
    pub(super) commit_files_out_tx: std_mpsc::Sender<CommitFilesOutcome>,
    pub(super) working_diff_out_tx: std_mpsc::Sender<WorkingDiffOutcome>,
    pub(super) file_insight_out_tx: std_mpsc::Sender<FileInsightOutcome>,
    pub(super) merge_out_tx: std_mpsc::Sender<MergeReply>,
    pub(super) log_page_out_tx: std_mpsc::Sender<LogPageOutcome>,
    pub(super) stats_worker: Option<StatsWorker>,
    pub(super) split_worker: Option<SplitSourceWorker>,
    pub(super) log_depth: Arc<AtomicUsize>,
    pub(super) log_filter: LogFilterCell,
    pub(super) lfs_gate: Arc<crate::lfs_ui::LfsGate>,
    pub(super) activity: Arc<crate::activity::Activity>,
}

pub(super) async fn link_loop(
    port_file: Option<PathBuf>,
    session: String,
    repo: RepoRef,
    channels: LinkChannels,
) -> Option<String> {
    let endpoint = match ensure_daemon_at(port_file.as_deref(), &session).await {
        Ok(endpoint) => endpoint,
        Err(e) => return Some(e),
    };
    // A daemon from another build is replaced when it is idle (once per link
    // start; see `takeover`).
    let endpoint = match takeover_stale_daemon(&endpoint, &session, port_file.as_deref()).await {
        Takeover::Unchanged => endpoint,
        Takeover::Busy(message) => return Some(message),
        Takeover::Restarted { endpoint, notice } => {
            let _ = channels.action_message_tx.send(notice);
            endpoint
        }
    };
    link_loop_with_port(endpoint, session, repo, channels, Default::default()).await
}

/// The part of `link_loop` that's independent of how the port was found -
/// split out so tests can point it at a fixture daemon directly
/// without touching the real config dir / discovery file.
///
/// Returns why it stopped: `None` when the `DaemonLink` was dropped (the
/// window ended - nothing to report), `Some(reason)` when it could not
/// connect or lost its connection. `start` forwards that on
/// `DaemonLink::failure_rx`.
pub(super) async fn link_loop_with_port(
    endpoint: gitpoulpi_ipc::Endpoint,
    session: String,
    repo: RepoRef,
    channels: LinkChannels,
    commit_files_cache: SharedCommitFilesCache,
) -> Option<String> {
    let LinkChannels {
        mut manual_refresh_rx,
        mut action_rx,
        commit_files_task_rx,
        mut working_diff_task_rx,
        mut file_insight_task_rx,
        mut merge_task_rx,
        mut log_page_task_rx,
        outcome_tx,
        action_error_tx,
        action_message_tx,
        push_rejected_tx,
        autostash_tx,
        commit_files_out_tx,
        working_diff_out_tx,
        file_insight_out_tx,
        merge_out_tx,
        log_page_out_tx,
        stats_worker,
        split_worker,
        log_depth,
        log_filter,
        lfs_gate,
        activity,
    } = channels;
    // Never a silent `return`: the render loop only learns the link is alive
    // from a `RefreshOutcome`, so a dropped connect error leaves it on
    // "Loading repository..." forever.
    let stream = match gitpoulpi_ipc::discovery::connect(&endpoint).await {
        Ok(s) => s,
        Err(e) => {
            return Some(gitpoulpi_i18n::msg::link_error::connect_failed(
                endpoint.port,
                e,
            ))
        }
    };
    let (read_half, write_half) = stream.into_split();
    let conn = LinkConn {
        write_half: Arc::new(AsyncMutex::new(write_half)),
        pending: Arc::new(std::sync::Mutex::new(PendingRequests::default())),
        next_id: Arc::new(AtomicU64::new(1)),
        session,
        repo,
        memo: Default::default(),
    };
    let pending = Arc::clone(&conn.pending);

    // The reader task must never itself call do_refresh(): that awaits a
    // Response coming back over the *same* socket this task is the only
    // one reading, so it would block waiting on a reply only it could
    // deliver -- a guaranteed self-deadlock the moment a RepoChanged event
    // arrives. Instead it just signals the outer loop below (a separate
    // task) to do the refresh, the same way a manual/periodic refresh does.
    let (event_refresh_tx, mut event_refresh_rx) =
        tokio::sync::mpsc::unbounded_channel::<ChangeScope>();
    // `lfs_gate` (see `lfs_ui::LfsGate`) arrives from `start_at`: the
    // reader marks it dirty on every `RepoChanged` and records LFS
    // transfer reports, the loop marks it on a manual refresh.
    tokio::spawn(read_loop(
        read_half,
        Arc::clone(&pending),
        conn.repo.id.clone(),
        Arc::clone(&lfs_gate),
        Arc::clone(&activity),
        event_refresh_tx,
    ));

    // Best-effort build check: ask the daemon who it is before trusting it
    // with real work. Only acts when *both* sides resolved a git sha (a
    // `None` vs `None` comparison is meaningless): a false "different build"
    // warning is worse than a missed one. A `Hello` that errors, including
    // an old daemon that predates `OperationKind::Hello` and drops the
    // connection on it, is swallowed rather than failing the link over a
    // diagnostic.
    if let Ok(info) = conn
        .call::<gitpoulpi_ipc::DaemonInfo>(OperationKind::Hello, serde_json::json!({}))
        .await
    {
        if let (Some(daemon_sha), Some(my_sha)) =
            (info.git_sha.as_deref(), this_build_git_sha().as_deref())
        {
            if daemon_sha != my_sha {
                return Some(gitpoulpi_i18n::msg::link_error::build_mismatch(
                    daemon_sha, my_sha,
                ));
            }
        }
    }

    // The background jobs of the whole session: the cell every window's
    // indicator reads is seeded here, then kept current by the reader's
    // `JobChanged` / `JobRemoved`; cancels go out through this link.
    crate::jobs_ui::set_requester(activity.raw.requester());
    if let Ok(list) = conn
        .call::<Vec<gitpoulpi_ipc::JobInfo>>(OperationKind::JobList, serde_json::json!({}))
        .await
    {
        crate::jobs_ui::seed(list);
    }

    // The selected commit's files have a task of their own: it keeps the cache, drops superseded selections and prefetches
    // the neighbors, none of which this select loop should wait on.
    // Started before the first refresh so a selection made while the
    // repository is still loading is served as soon as it can be.
    if let Some(StatsWorker { rx, out_tx }) = stats_worker {
        tokio::spawn(commit_stats_worker(conn.clone(), rx, out_tx));
    }
    if let Some(SplitSourceWorker { rx, out_tx }) = split_worker {
        tokio::spawn(split_source_worker(conn.clone(), rx, out_tx));
    }
    tokio::spawn(commit_files_worker(
        conn.clone(),
        commit_files_task_rx,
        commit_files_out_tx,
        commit_files_cache,
    ));

    // The first load is awaited here: nothing else can be on screen yet.
    // It is only what the first paint needs (the graph, the refs and what
    // shapes them: status, stash rows, remotes, rebase/conflict state).
    // The worktree list and the LFS probes (processes to spawn) follow as a
    // second outcome, off the critical path (`FIRST_LOAD_LATER`).
    //
    // A window with no repository (Home) has nothing to read: every
    // repository-scoped request would go to the placeholder path and fail,
    // and the 2 s poll would log those failures for ever. The link then only
    // serves the session-wide requests (jobs, raw requests, presence); a
    // window that opens a repository gets a new link (`start`).
    let has_repo = conn.repo.root != std::path::Path::new(crate::tabs::NO_REPOSITORY);
    let first_generation = activity.generation();
    if has_repo {
        let opened = std::time::Instant::now();
        let mut first = collect_refresh(
            &conn,
            &log_depth,
            &log_filter,
            &lfs_gate,
            FIRST_LOAD_NOW,
            true,
        )
        .await;
        crate::activity_log::timing_record(
            &conn.repo.root,
            gitpoulpi_i18n::msg::activity_log::OP_OPEN.get().to_string(),
            u64::try_from(opened.elapsed().as_millis()).unwrap_or(u64::MAX),
        );
        first.generation = first_generation;
        let _ = outcome_tx.send_notify(first);
    }

    // Every later refresh runs on a spawned task (single flight, with a
    // "dirty again" queue), so the select loop below never waits for one:
    // an action is sent the moment it is submitted.
    let (refresh_done_tx, mut refresh_done_rx) =
        tokio::sync::mpsc::unbounded_channel::<(RefreshOutcome, u64)>();
    let mut refresher = Refresher {
        conn: conn.clone(),
        log_depth: Arc::clone(&log_depth),
        log_filter: log_filter.clone(),
        lfs_gate: Arc::clone(&lfs_gate),
        activity: Arc::clone(&activity),
        done_tx: refresh_done_tx,
        inflight: 0,
        latest_started: None,
        queued: None,
        last_delivered: None,
        last_lfs: None,
    };
    if has_repo {
        refresher.want(FIRST_LOAD_LATER);
        tokio::spawn(refresh_journal(
            refresher.conn.clone(),
            Arc::clone(&activity),
        ));
    }
    // Network actions run on their own task and report back here.
    let (action_done_tx, mut action_done_rx) =
        tokio::sync::mpsc::unbounded_channel::<(u64, bool)>();

    // Presence for the daemon's viewer registry, and the answers to an
    // agent's requests (`ui_viewer`).
    let mut ui_attached = false;
    send_ui_updates(&conn, &activity.ui, &mut ui_attached).await;
    let mut raw_rx = activity.raw.take_receiver();
    let mut ticker = tokio::time::interval(AUTO_REFRESH_INTERVAL);
    ticker.tick().await; // first tick fires immediately; the explicit refresh above already covered it.
    loop {
        // What the refresh at the end of this pass has to re-read. The
        // poll and a manual refresh read everything; a daemon event reads
        // what its scope names (and the events already queued behind it
        // are folded in, so a burst costs one refresh).
        let mut scope = ChangeScope::ALL;
        tokio::select! {
            _ = activity.ui.woken() => {
                send_ui_updates(&conn, &activity.ui, &mut ui_attached).await;
                continue;
            }
            Some(request) = async { raw_rx.as_mut()?.recv().await } => {
                spawn_raw(&conn, request);
                continue;
            }
            _ = ticker.tick() => {
                // A poll while a refresh is running or waiting adds nothing.
                if refresher.inflight > 0 || refresher.queued.is_some() {
                    continue;
                }
            }
            received = manual_refresh_rx.recv() => {
                // `None`: the DaemonLink was dropped -- the window ended,
                // nothing to report.
                received?;
                lfs_gate.mark_dirty();
            }
            done = refresh_done_rx.recv() => {
                if let Some((outcome, started_at)) = done {
                    refresher.finished(outcome, started_at, &outcome_tx);
                }
                continue;
            }
            done = action_done_rx.recv() => {
                if let Some((op, ok)) = done {
                    refresher.action_finished(op, ok);
                }
                continue;
            }
            received = event_refresh_rx.recv() => {
                if let Some(first) = received {
                    scope = first;
                    while let Ok(more) = event_refresh_rx.try_recv() {
                        scope |= more;
                    }
                } else {
                    // The reader task exited: the connection is gone,
                    // and it recorded why before it went.
                    return Some(
                        pending
                            .lock()
                            .unwrap()
                            .closed
                            .clone()
                            .unwrap_or_else(|| {
                                gitpoulpi_i18n::msg::link_error::CONNECTION_WAS_LOST
                                    .get()
                                    .to_string()
                            }),
                    );
                }
            }
            action = action_rx.recv() => {
                let Some(action) = action else {
                    return None; // DaemonLink dropped.
                };
                // Locks: from now on this remote's list is kept fresh. A
                // lock/unlock finishes with the flag set again (below) so
                // the next poll re-reads it; *reading* locks changes
                // nothing here, so no `RepoChanged` follows and the loop
                // falls through to the refresh at its end instead.
                let lock_remote = action.lfs_locks_remote().map(str::to_string);
                if let Some(remote) = &lock_remote {
                    lfs_gate.want_locks(remote.clone());
                }
                let refresh_after = matches!(action, Action::LfsLocks(_));
                // Registered the moment it is received: this is what the
                // status line's "Staging..." spinner and the generation
                // counter read (see `activity`).
                let op = activity.begin_as(
                    action.activity_label(),
                    !action.is_network(),
                    action.runs_as_job(),
                );
                if action.is_network() {
                    // Fetch/pull/push can take a while (real network I/O);
                    // spawned rather than awaited inline so a slow one
                    // doesn't stall status polling or other actions, same
                    // reasoning as the commit-files fetch below.
                    let conn = conn.clone();
                    let action_error_tx = action_error_tx.clone();
                    let action_message_tx = action_message_tx.clone();
                    let push_rejected_tx = push_rejected_tx.clone();
                    let autostash_tx = autostash_tx.clone();
                    let gate = Arc::clone(&lfs_gate);
                    let net_activity = Arc::clone(&activity);
                    let action_done_tx = action_done_tx.clone();
                    let transfer = crate::activity::TransferKind::of(&action).map(|_| action.clone());
                    tokio::spawn(async move {
                        let ok = run_action(
                            &conn,
                            action,
                            &action_error_tx,
                            &action_message_tx,
                            &push_rejected_tx,
                            &autostash_tx,
                            &net_activity,
                        )
                        .await;
                        if let Some(remote) = lock_remote {
                            gate.want_locks(remote);
                            gate.mark_dirty();
                        }
                        if let Some(transfer) = &transfer {
                            net_activity.release_transfer(transfer);
                        }
                        let _ = action_done_tx.send((op, ok));
                    });
                } else {
                    let ok = run_action(
                        &conn,
                        action,
                        &action_error_tx,
                        &action_message_tx,
                        &push_rejected_tx,
                        &autostash_tx,
                        &activity,
                    )
                    .await;
                    refresher.action_finished(op, ok);
                }
                // A successful mutation triggers the daemon's own
                // RepoChanged broadcast (picked up by event_refresh_rx
                // above), so no explicit refresh here -- only a failed
                // action needs nothing further, its error already went
                // out on action_error_tx.
                if !refresh_after {
                    continue;
                }
            }
            request = working_diff_task_rx.recv() => {
                let Some(FileDiffRequest { target, path, also }) = request else {
                    return None; // DaemonLink dropped.
                };
                // Spawned for the same reason the commit-files fetch is:
                // one half's diff must not hold up a refresh or a newer
                // request for the other half.
                let conn = conn.clone();
                let working_diff_out_tx = working_diff_out_tx.clone();
                tokio::spawn(async move {
                    let mut args = serde_json::json!({ "target": target.clone() });
                    if let Some(path) = &path {
                        args["paths"] = match &also {
                            Some(also) => serde_json::json!([path, also]),
                            None => serde_json::json!([path]),
                        };
                    }
                    let mut files = conn.call::<Vec<FileDiff>>(OperationKind::Diff,
                        args,
                    )
                    .await;
                    // The History view's rename: the delete and the add of
                    // the two names become one diff.
                    if let (Some(path), Some(also), Ok(listed)) = (&path, &also, &mut files) {
                        let split = std::mem::take(listed);
                        *listed = crate::history_view::merge_rename(split, also, path);
                    }
                    // The Unstaged half lists untracked files too, and
                    // the backend's diff does not carry them. For one
                    // file only when it is not among the tracked changes.
                    if let (DiffTarget::WorkingTreeVsIndex, Ok(listed)) = (&target, &mut files) {
                        if path.is_none() || listed.is_empty() {
                            append_untracked_diffs(
                                &conn,
                                listed,
                                path.as_deref(),
                            )
                            .await;
                        }
                    }
                    let _ = working_diff_out_tx.send_notify(WorkingDiffOutcome {
                        target,
                        path,
                        files,
                    });
                });
                continue;
            }
            request = file_insight_task_rx.recv() => {
                let Some(request) = request else {
                    return None; // DaemonLink dropped.
                };
                                // Spawned, never awaited inline: blame on a long file is the
                                // slowest read in the app and awaiting it here would stall
                                // every refresh, action and event behind it.
                let conn = conn.clone();
                let file_insight_out_tx = file_insight_out_tx.clone();
                tokio::spawn(async move {
                    let result = run_file_insight(
                        &conn,
                        &request,
                    )
                    .await;
                    let _ = file_insight_out_tx.send_notify(FileInsightOutcome { request, result });
                });
                continue;
            }
            Some(request) = merge_task_rx.recv() => {
                // A closed channel only disables this arm: the other
                // arms notice a dropped DaemonLink.
                // Spawned, like every on-demand read; a save is tracked as
                // an operation (spawn_merge) so the refresh follows it.
                spawn_merge(&conn, &activity, &merge_out_tx, &action_done_tx, request);
                continue;
            }
            request = log_page_task_rx.recv() => {
                let Some(request) = request else {
                    return None; // DaemonLink dropped.
                };
                                // Spawned, like every other on-demand read, and the one that
                                // matters most: a page is a full revwalk over history the
                                // client has already seen, the slowest request the link makes.
                                // Awaited inline it would freeze the UI the moment the user
                                // scrolled to the bottom, the failure lazy loading exists to
                                // prevent.
                let conn = conn.clone();
                let log_page_out_tx = log_page_out_tx.clone();
                // The filter is read here, with its generation: the page
                // and the generation it is stamped with are one snapshot.
                let (log_filter_gen, hidden) = log_filter.snapshot();
                tokio::spawn(async move {
                    let result = conn.call::<CommitGraph>(OperationKind::Log,
                        serde_json::json!({
                                                        // The same walk as the refresh (`all_refs`
                                                        // included), or the page would be a slice of a
                                                        // *different* ordering and would not tile onto
                                                        // what is held. The contract check
                                                        // `check_log_skip_pages_history` pins that pages
                                                        // of one walk tile; not pages of two.
                            "opts": LogOptions {
                                limit: Some(request.limit as u32),
                                skip: request.skip as u32,
                                start_ref: None,
                                all_refs: true,
                                hidden,
                                ..LogOptions::default()
                            }
                        }),
                    )
                    .await;
                    let _ = log_page_out_tx.send_notify(LogPageOutcome {
                        request,
                        result,
                        log_filter_gen,
                    });
                });
                continue;
            }
        }
        if has_repo {
            refresher.want(scope);
        }
    }
}

/// Sends this window's presence (the first one is `UiAttach`, the next
/// `UiPresence`) and the answers the render loop queued. A window that is
/// not allowed to be driven never publishes, so it never attaches. Answers
/// to a request of a previous connection are refused by the daemon; that is
/// ignored.
async fn send_ui_updates(
    conn: &LinkConn,
    bridge: &crate::ui_viewer::UiBridge,
    attached: &mut bool,
) {
    let (info, reports) = bridge.take_outgoing();
    if let Some(info) = info {
        let kind = if *attached {
            OperationKind::UiPresence
        } else {
            OperationKind::UiAttach
        };
        let args = serde_json::to_value(info).unwrap_or_default();
        if conn.call::<serde_json::Value>(kind, args).await.is_ok() {
            *attached = true;
        }
    }
    for report in reports {
        let args = serde_json::to_value(report).unwrap_or_default();
        let _ = conn
            .call::<serde_json::Value>(OperationKind::UiReport, args)
            .await;
    }
}
