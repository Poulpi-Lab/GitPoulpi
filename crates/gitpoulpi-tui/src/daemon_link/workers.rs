use super::*;
use crate::redraw::NotifySend as _;

/// The commit-files worker's cache. Shared (rather than owned by the
/// worker) because a prefetch task fills it too, and so a test can look
/// at it.
pub(super) type SharedCommitFilesCache = Arc<std::sync::Mutex<CommitFilesCache>>;

/// A prefetch running beside the worker: which commit, and its answer
/// (already stored in the cache by the time it resolves).
pub(super) struct InFlightPrefetch {
    pub(super) handle: tokio::task::JoinHandle<GitResult<Vec<FileDiff>>>,
}

/// Takes every request already sent without waiting, so a burst of
/// selections (j held down) collapses to its newest before any work
/// starts. `false` once the `DaemonLink` is gone.
pub(super) fn drain_commit_files_requests(
    rx: &mut tokio::sync::mpsc::UnboundedReceiver<CommitFilesRequest>,
    queue: &mut CommitFilesQueue,
) -> bool {
    loop {
        match rx.try_recv() {
            Ok(request) => apply_commit_files_request(queue, request),
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => return true,
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => return false,
        }
    }
}

pub(super) fn apply_commit_files_request(
    queue: &mut CommitFilesQueue,
    request: CommitFilesRequest,
) {
    match request {
        CommitFilesRequest::Fetch(fetch) => queue.push(fetch),
        CommitFilesRequest::Prefetch(commits) => queue.replace_prefetch(commits),
        CommitFilesRequest::Compare { id, target } => queue.push_compare(id, target),
    }
}

/// How long the stats worker waits after a request for a newer one before
/// asking: holding PageDown or dragging the scrollbar fires a request per
/// frame, and only the viewport the user lands on is worth a git call.
pub(super) const STATS_DEBOUNCE: Duration = Duration::from_millis(60);

/// The Log "Changes" column's fetcher: one task, one batch in flight, the
/// newest request wins. The daemon answers from its own per-commit cache
/// and runs at most one git process for what it does not know.
pub(super) async fn commit_stats_worker(
    conn: LinkConn,
    mut rx: tokio::sync::mpsc::UnboundedReceiver<Vec<Oid>>,
    out_tx: std_mpsc::Sender<CommitStatsOutcome>,
) {
    while let Some(mut ids) = rx.recv().await {
        loop {
            tokio::time::sleep(STATS_DEBOUNCE).await;
            match rx.try_recv() {
                Ok(newer) => ids = newer,
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => return,
            }
        }
        let started = std::time::Instant::now();
        let result = conn
            .call::<Vec<CommitStat>>(
                OperationKind::CommitStats,
                serde_json::json!({ "ids": ids }),
            )
            .await;
        gitpoulpi_ipc::timing::record("tui", || {
            format!(
                "commit-stats x{}: {}",
                ids.len(),
                gitpoulpi_ipc::timing::ms(started.elapsed())
            )
        });
        if out_tx.send_notify(CommitStatsOutcome { result }).is_err() {
            return;
        }
    }
}

/// Answers `request_split_source`, one read at a time, never on the render
/// loop. The daemon runs one diff for the commit; the answer is the
/// numbering the split editor and the backend share.
pub(super) async fn split_source_worker(
    conn: LinkConn,
    mut rx: tokio::sync::mpsc::UnboundedReceiver<Oid>,
    out_tx: std_mpsc::Sender<SplitSourceOutcome>,
) {
    while let Some(commit) = rx.recv().await {
        let result = conn
            .call::<gitpoulpi_core::domain::SplitSource>(
                OperationKind::SplitSource,
                serde_json::json!({ "commit": commit }),
            )
            .await;
        if out_tx
            .send_notify(SplitSourceOutcome { commit, result })
            .is_err()
        {
            return;
        }
    }
}

/// The selected commit's files, as fast as they can be had. One task per
/// link, owning the cache:
///
/// 1. **Cache.** A commit never changes, so an answer already fetched is
///    sent back at once - revisiting a commit costs nothing.
/// 2. **Stale requests dropped.** Requests are drained before each fetch
///    and a newer selection replaces an older one still queued
///    ([`CommitFilesQueue`]), so holding j fetches the row the key is let
///    go on, not every row it passed.
/// 3. **List first.** For the selection, `DiffFiles` comes first: the list
///    of paths needs only the two trees and is sent at once
///    (`hunks_pending`), then `Diff` fills in the hunks. When a newer
///    selection is already waiting by then, the hunks are skipped.
/// 4. **Prefetch.** The neighbors the render loop names are fetched in
///    full into the cache, one at a time, *beside* the urgent work rather
///    than in front of it (the daemon runs reads concurrently), so a
///    click never waits for a prefetch.
///
/// Never touches the render loop: every answer goes out on `out_tx`.
pub(super) async fn commit_files_worker(
    conn: LinkConn,
    mut rx: tokio::sync::mpsc::UnboundedReceiver<CommitFilesRequest>,
    out_tx: std_mpsc::Sender<CommitFilesOutcome>,
    cache: SharedCommitFilesCache,
) {
    let mut queue = CommitFilesQueue::default();
    let mut prefetch: Option<InFlightPrefetch> = None;
    loop {
        if !drain_commit_files_requests(&mut rx, &mut queue) {
            return;
        }
        if prefetch.as_ref().is_some_and(|p| p.handle.is_finished()) {
            prefetch = None;
        }
        if let Some((id, target)) = queue.pop_compare() {
            // Beside the worker, not in it: a comparison over a big range
            // must not hold up the selection's own fetches.
            let conn = conn.clone();
            let out_tx = out_tx.clone();
            tokio::spawn(async move {
                let files = conn
                    .call::<DiffFileList>(
                        OperationKind::DiffFiles,
                        serde_json::json!({ "target": target }),
                    )
                    .await;
                let (files, hunks_pending) = match files {
                    Ok(list) => (Ok(list.files), !list.hunks_included),
                    Err(e) => (Err(e), false),
                };
                let _ = out_tx.send_notify(CommitFilesOutcome {
                    commit: id,
                    files,
                    tree: None,
                    hunks_pending,
                });
            });
            continue;
        }
        if let Some(fetch) = queue.pop_urgent() {
            serve_commit_files(&conn, fetch, &cache, &out_tx).await;
            continue;
        }
        if prefetch.is_none() {
            if let Some(next) = queue.pop_prefetch() {
                if !cache.lock().unwrap().has_files(next.commit) {
                    let conn = conn.clone();
                    let cache = Arc::clone(&cache);
                    let commit = next.commit;
                    prefetch = Some(InFlightPrefetch {
                        handle: tokio::spawn(async move {
                            let started = std::time::Instant::now();
                            let files = conn.commit_list(commit).await.map(|list| {
                                let mut cache = cache.lock().unwrap();
                                if list.hunks_included {
                                    cache.insert(commit, Some(list.files.clone()), None);
                                } else {
                                    cache.insert_list(commit, list.files.clone());
                                }
                                list.files
                            });
                            gitpoulpi_ipc::timing::record("tui", || {
                                format!(
                                    "commit-files {} prefetch: {}",
                                    short(commit),
                                    gitpoulpi_ipc::timing::ms(started.elapsed())
                                )
                            });
                            files
                        }),
                    });
                }
                continue;
            }
        }
        // Nothing to start: wait for a request, or for the prefetch in
        // flight to finish so the next one can go.
        let woke = {
            let prefetch_done = async {
                match prefetch.as_mut() {
                    Some(p) => {
                        let _ = (&mut p.handle).await;
                    }
                    None => std::future::pending::<()>().await,
                }
            };
            tokio::select! {
                request = rx.recv() => Some(request),
                _ = prefetch_done => None,
            }
        };
        match woke {
            Some(Some(request)) => apply_commit_files_request(&mut queue, request),
            Some(None) => return, // DaemonLink dropped.
            None => prefetch = None,
        }
    }
}

/// Answers one fetch someone is waiting on. See [`commit_files_worker`].
pub(super) async fn serve_commit_files(
    conn: &LinkConn,
    fetch: QueuedFetch,
    cache: &SharedCommitFilesCache,
    out_tx: &std_mpsc::Sender<CommitFilesOutcome>,
) {
    let QueuedFetch {
        commit,
        with_tree,
        purpose,
        sent,
    } = fetch;
    let log = |what: &str| {
        gitpoulpi_ipc::timing::record("tui", || {
            format!(
                "commit-files {} {purpose:?}: {what} after {}",
                short(commit),
                gitpoulpi_ipc::timing::ms(sent.elapsed())
            )
        })
    };

    let hit = cache.lock().unwrap().get(commit, with_tree);
    if let Some((files, tree)) = hit {
        log("cache hit");
        let hunks_pending = cache.lock().unwrap().is_lazy(commit);
        let _ = out_tx.send_notify(CommitFilesOutcome {
            commit,
            files: Ok(files),
            tree: tree.map(Ok),
            hunks_pending,
        });
        return;
    }

    // `lazy`: the files are a list without hunks. Nothing here diffs every
    // file of a commit -- the Diff view fetches the open file's hunks itself
    // -- so a 500-file commit costs a tree comparison, not 500 blob diffs.
    let held = cache.lock().unwrap().files(commit);
    let mut lazy = cache.lock().unwrap().is_lazy(commit);
    let files = match held {
        Some(files) => Ok(files),
        None => {
            // Only the selection shows a list; the WIP tree request wants
            // the tree and nobody looks at its list.
            let listed = if purpose == CommitFilesPurpose::Selection {
                conn.commit_list(commit).await
            } else {
                conn.commit_diff(commit).await.map(|files| DiffFileList {
                    files,
                    hunks_included: true,
                })
            };
            match listed {
                Ok(list) if list.hunks_included => {
                    cache
                        .lock()
                        .unwrap()
                        .insert(commit, Some(list.files.clone()), None);
                    Ok(list.files)
                }
                Ok(list) => {
                    lazy = true;
                    cache
                        .lock()
                        .unwrap()
                        .insert_list(commit, list.files.clone());
                    log("list");
                    if !with_tree {
                        let _ = out_tx.send_notify(CommitFilesOutcome {
                            commit,
                            files: Ok(list.files),
                            tree: None,
                            hunks_pending: true,
                        });
                        return;
                    }
                    Ok(list.files)
                }
                Err(e) => Err(e),
            }
        }
    };

    let tree = if with_tree {
        let tree = conn
            .call::<Vec<PathBuf>>(OperationKind::ListTree, serde_json::json!({ "at": commit }))
            .await;
        if let Ok(tree) = &tree {
            cache
                .lock()
                .unwrap()
                .insert(commit, None, Some(tree.clone()));
        }
        Some(tree)
    } else {
        None
    };
    log("full");
    let _ = out_tx.send_notify(CommitFilesOutcome {
        commit,
        files,
        tree,
        hunks_pending: lazy,
    });
}
