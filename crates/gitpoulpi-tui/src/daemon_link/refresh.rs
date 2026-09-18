use super::*;

/// Reads the journal and publishes it to the render loop. Spawned beside
/// every refresh (a poll sees a commit made in a shell, so the buttons'
/// blocked state follows within one poll) and once at start.
pub(super) async fn refresh_journal(conn: LinkConn, activity: Arc<crate::activity::Activity>) {
    if let Ok(state) = conn
        .call::<gitpoulpi_core::domain::JournalState>(
            OperationKind::JournalState,
            serde_json::json!({}),
        )
        .await
    {
        activity.journal.set(state);
    }
}

/// How many commits the main `Log` request asks for: more than a screenful,
/// or a tall window shows blank space under the last commit that scrolling
/// cannot reveal.
///
/// A process-wide atomic rather than a threaded field: the refresh runs on a
/// spawned task far from anything that has seen a `Config`, and the value is
/// the same for every repository in the process. `run` publishes it from
/// `general.initial_commits_in_graph` before the first refresh; Preferences
/// republishes it on change, so it applies to the next refresh.
///
/// **No floor is enforced here.** Both production call sites pass
/// `GeneralConfig::initial_commits_in_graph_clamped()`, which applies
/// `MIN_INITIAL_COMMITS_IN_GRAPH` itself, and a second clamp would stop
/// callers that deliberately want a small number, such as
/// `a_paged_log_tiles_onto_the_initial_load_over_the_real_link`, which goes
/// red if the `.max(MIN_INITIAL_COMMITS_IN_GRAPH)` clamp is restored.
pub(super) static LOG_PAGE_LIMIT: std::sync::atomic::AtomicU32 =
    std::sync::atomic::AtomicU32::new(gitpoulpi_config::DEFAULT_INITIAL_COMMITS_IN_GRAPH);

pub(crate) fn set_log_page_limit(limit: u32) {
    LOG_PAGE_LIMIT.store(limit, std::sync::atomic::Ordering::Relaxed);
}

pub(crate) fn log_page_limit() -> u32 {
    LOG_PAGE_LIMIT.load(std::sync::atomic::Ordering::Relaxed)
}

/// How deep one full refresh reads: the configured initial load, or what
/// lazy loading has grown the client to, whichever is larger.
pub(super) fn refresh_limit(log_depth: &Arc<AtomicUsize>) -> u32 {
    let held = log_depth.load(Ordering::Relaxed).min(u32::MAX as usize) as u32;
    held.max(log_page_limit())
}

/// What the first load reads before the first paint: everything but the
/// linked-worktree list and the LFS state.
pub(super) const FIRST_LOAD_NOW: ChangeScope = ChangeScope::INDEX
    .union(ChangeScope::WORKTREE)
    .union(ChangeScope::REFS)
    .union(ChangeScope::LOG)
    .union(ChangeScope::STASHES)
    .union(ChangeScope::CONFIG)
    .union(ChangeScope::CONFLICTS);

/// What can change the SUBMODULES list: a checkout or pull (refs), an edit
/// in the working tree (`.gitmodules`, a submodule's own checkout) and the
/// worktree list's own scope (the first load's second outcome).
pub(crate) const SUBMODULES_SCOPE: ChangeScope = ChangeScope::WORKTREES
    .union(ChangeScope::REFS)
    .union(ChangeScope::WORKTREE);

/// What follows the first paint, as a second outcome.
pub(crate) const FIRST_LOAD_LATER: ChangeScope = ChangeScope::WORKTREES.union(ChangeScope::LFS);

/// The last answer to each conditional read: the decoded value (a
/// `Box<dyn Any>` of the read's type), its etag and its content hash.
type Held = (u64, u64, Box<dyn std::any::Any + Send>);

#[derive(Default)]
pub(super) struct ReadMemo(std::collections::HashMap<(OperationKind, String), Held>);

impl LinkConn {
    /// `call` for a read the daemon's `ReadCache` can answer "not
    /// modified": the etag of the answer held is sent along, and a
    /// not-modified reply is turned back into that held answer, so an idle
    /// poll moves a few bytes instead of the whole log. Returns the result
    /// and its content hash (the same one `json_hash` gives; kept from the
    /// answer held when nothing changed, so it is not computed again).
    ///
    /// A daemon from before this existed ignores the key and answers in
    /// full, which is handled like any change.
    pub(super) async fn call_cached<T>(
        &self,
        command: OperationKind,
        args: serde_json::Value,
    ) -> (GitResult<T>, u64)
    where
        T: serde::de::DeserializeOwned + serde::Serialize + Clone + Send + 'static,
    {
        let memo_key = (command, args.to_string());
        let held = self
            .memo
            .lock()
            .unwrap()
            .0
            .get(&memo_key)
            .map(|(etag, ..)| *etag);
        let sent = match held {
            Some(etag) => gitpoulpi_ipc::etag::with_if_none_match(args, etag),
            None => args,
        };
        let raw = call_raw(
            &self.write_half,
            &self.pending,
            &self.next_id,
            &self.session,
            &self.repo,
            command,
            sent,
        )
        .await;
        let value = match raw {
            Ok(value) => value,
            Err(e) => {
                self.memo.lock().unwrap().0.remove(&memo_key);
                let result = Err(e);
                let hash = json_hash::<T>(&result);
                return (result, hash);
            }
        };
        if gitpoulpi_ipc::etag::is_not_modified(&value) {
            let kept = self
                .memo
                .lock()
                .unwrap()
                .0
                .get(&memo_key)
                .and_then(|(_, hash, any)| any.downcast_ref::<T>().map(|v| (v.clone(), *hash)));
            if let Some((held, hash)) = kept {
                return (Ok(held), hash);
            }
            // Not modified, yet nothing held (cannot happen: the key is
            // only sent with a held answer). Forget and fail loudly.
            self.memo.lock().unwrap().0.remove(&memo_key);
            let result = Err(CoreError::Backend(
                gitpoulpi_i18n::msg::link_error::NOT_MODIFIED_NO_ANSWER
                    .get()
                    .to_string(),
            ));
            let hash = json_hash::<T>(&result);
            return (result, hash);
        }
        let etag = gitpoulpi_ipc::etag::of_value(&value);
        let result: GitResult<T> = serde_json::from_value(value).map_err(|e| {
            CoreError::Backend(gitpoulpi_i18n::msg::link_error::malformed_response(e))
        });
        let hash = json_hash(&result);
        let mut memo = self.memo.lock().unwrap();
        match &result {
            Ok(v) => {
                memo.0.insert(memo_key, (etag, hash, Box::new(v.clone())));
            }
            Err(_) => {
                memo.0.remove(&memo_key);
            }
        }
        (result, hash)
    }
}

/// One refresh's reads, gathered into an outcome (the caller delivers it).
#[allow(clippy::too_many_arguments)]
pub(super) async fn collect_refresh(
    conn: &LinkConn,
    log_depth: &Arc<AtomicUsize>,
    lfs_gate: &Arc<crate::lfs_ui::LfsGate>,
    scope: ChangeScope,
    defer_lfs: bool,
) -> RefreshOutcome {
    let status_fut = conn.call::<WorkingTree>(OperationKind::Status, serde_json::json!({}));
    let log_limit = refresh_limit(log_depth);
    let log_ms = std::sync::atomic::AtomicU64::new(0);
    let log_inner = gated_hashed(
        scope.intersects(ChangeScope::LOG),
        conn.call_cached::<CommitGraph>(
            OperationKind::Log,
            // `all_refs: true` -- the graph must show every branch's commits
            // (like `git log --all --graph`), not just the checked-out
            // branch's own ancestry.
            serde_json::json!({
                "opts": LogOptions {
                                        // NOT a screenful: the count comes from
                                        // `general.initial_commits_in_graph` via `LOG_PAGE_LIMIT`
                                        // above (default 5000), far more than
                                        // any terminal shows, so scrolling always has somewhere
                                        // to go.
                                        //
                                        // `log_depth` is how much the client currently *holds*:
                                        // pages appended since the last refresh count, or the
                                        // next poll would truncate the log back to the initial
                                        // page under a user who scrolled into page three. It
                                        // never goes below the configured initial load, and
                                        // `LazyLog` clamps it to the ceiling.
                    limit: Some(log_limit),
                    skip: 0,
                    start_ref: None,
                    all_refs: true,
                    ..LogOptions::default()
                }
            }),
        ),
    );
    // The commit graph's own read, for the Activity Log's timing line.
    let log_fut = async {
        let began = std::time::Instant::now();
        let result = log_inner.await;
        log_ms.store(
            u64::try_from(began.elapsed().as_millis()).unwrap_or(u64::MAX),
            std::sync::atomic::Ordering::Relaxed,
        );
        result
    };
    let refs_fut = gated_hashed(
        scope.intersects(ChangeScope::REFS),
        conn.call_cached::<Vec<Ref>>(OperationKind::Refs, serde_json::json!({})),
    );
    let stashes_fut = gated(
        scope.intersects(ChangeScope::STASHES),
        conn.call::<Vec<Stash>>(OperationKind::Stashes, serde_json::json!({})),
    );
    let worktrees_fut = gated(
        crate::build_features::WORKTREE && scope.intersects(ChangeScope::WORKTREES),
        conn.call::<Vec<Worktree>>(OperationKind::Worktrees, serde_json::json!({})),
    );
    // No `.gitmodules`, no request: most repositories have none, and the
    // poll runs every two seconds.
    let submodules_fut = gated(
        crate::build_features::SUBMODULE && scope.intersects(SUBMODULES_SCOPE),
        async {
            if conn.repo.root.join(".gitmodules").is_file() {
                conn.call::<Vec<Submodule>>(OperationKind::Submodules, serde_json::json!({}))
                    .await
            } else {
                Ok(Vec::new())
            }
        },
    );
    let conflicts_fut = gated(
        scope.intersects(ChangeScope::CONFLICTS),
        conn.call::<Vec<ConflictEntry>>(OperationKind::ConflictEntries, serde_json::json!({})),
    );
    let remotes_fut = gated(
        scope.intersects(ChangeScope::CONFIG),
        conn.call::<Vec<Remote>>(OperationKind::Remotes, serde_json::json!({})),
    );
    let ssh_command_fut = gated(
        scope.intersects(ChangeScope::CONFIG),
        conn.call::<Option<String>>(OperationKind::SshCommand, serde_json::json!({})),
    );
    // `git sparse-checkout` leaves `info/sparse-checkout` in the git dir once
    // it has been used: its absence is the answer for the (overwhelming)
    // majority of repositories, and costs a stat instead of a process.
    let sparse_fut = gated(scope.intersects(ChangeScope::CONFIG), async {
        if sparse_checkout_file(&conn.repo.root) {
            conn.call::<gitpoulpi_core::domain::SparseState>(
                OperationKind::SparseStatus,
                serde_json::json!({}),
            )
            .await
        } else {
            Ok(gitpoulpi_core::domain::SparseState::default())
        }
    });
    let repo_config_fut = gated(
        scope.intersects(ChangeScope::CONFIG),
        conn.call::<std::collections::BTreeMap<String, Option<String>>>(
            OperationKind::RepoConfig,
            serde_json::json!({}),
        ),
    );
    // One read for every operation kind: the rebase banner's state and the
    // merge / cherry-pick / revert banner's come out of it.
    let operation_state_fut = gated(
        scope.intersects(ChangeScope::CONFLICTS),
        conn.call::<Option<OperationState>>(OperationKind::OperationState, serde_json::json!({})),
    );
    // A file stat: the stale-lock strip. A poll is the only way to notice a
    // lock a dead git process left behind.
    let index_lock_fut = gated(
        scope.intersects(ChangeScope::CONFLICTS),
        conn.call::<Option<gitpoulpi_core::domain::IndexLock>>(
            OperationKind::IndexLock,
            serde_json::json!({}),
        ),
    );
    // Git LFS state, only when the gate says it is worth the spawns (and
    // only when the `lfs_enabled` preference is on): status first, then
    // the pointer files if the repository uses LFS at all.
    // A scope that cannot have changed LFS state leaves the gate alone (its
    // dirty flag is kept for the refresh that can use it).
    // `defer_lfs` is the first load's: the LFS probes spawn processes (about a
    // second on a slow box), and the graph must not wait for them.
    let want_lfs = !defer_lfs
        && scope.touches_lfs()
        && lfs_gate.due()
        && crate::lfs_ui::resolved_enabled(&conn.repo.root);
    let lock_remote = if want_lfs { lfs_gate.locks_due() } else { None };
    let lfs_fut = async {
        if !want_lfs {
            return None;
        }
        let status = conn
            .call::<gitpoulpi_core::domain::LfsRepoStatus>(
                OperationKind::LfsStatus,
                serde_json::json!({}),
            )
            .await
            .ok();
        let files = if status.as_ref().is_some_and(|s| s.uses_lfs) {
            conn.call::<Vec<gitpoulpi_core::domain::LfsFile>>(
                OperationKind::LfsFiles,
                serde_json::json!({}),
            )
            .await
            .unwrap_or_default()
        } else {
            Vec::new()
        };
        // The server's locks, only for a user who has asked about them.
        let locks = match &lock_remote {
            Some(remote) => conn
                .call::<Vec<gitpoulpi_core::domain::LfsLock>>(
                    OperationKind::LfsLocks,
                    serde_json::json!({ "remote": remote }),
                )
                .await
                .ok(),
            None => None,
        };
        if locks.is_some() {
            lfs_gate.locks_taken();
        }
        Some(crate::lfs_ui::LfsSnapshot {
            status,
            files,
            locks,
        })
    };
    // All eight requests go out at once. The daemon admits reads under a
    // shared lock and runs each on its own blocking task, so the refresh
    // costs the slowest query rather than the sum of all eight.
    let started = std::time::Instant::now();
    let (
        status_result,
        (log_result, log_hash),
        (refs_result, refs_hash),
        stashes_result,
        worktrees_result,
        submodules_result,
        conflicts_result,
        remotes_result,
        ssh_command_result,
        sparse_result,
        repo_config_result,
        operation_state_result,
        index_lock_result,
        lfs,
    ) = tokio::join!(
        status_fut,
        log_fut,
        refs_fut,
        stashes_fut,
        worktrees_fut,
        submodules_fut,
        conflicts_fut,
        remotes_fut,
        ssh_command_fut,
        sparse_fut,
        repo_config_fut,
        operation_state_fut,
        index_lock_fut,
        lfs_fut
    );
    let rebase_state_result = match &operation_state_result {
        Ok(Some(OperationState::Rebase(state))) => Ok(Some(state.clone())),
        Ok(_) => Ok(None),
        Err(e) => Err(CoreError::Backend(e.to_string())),
    };
    if let Some(snapshot) = &lfs {
        lfs_gate.taken(snapshot.status.as_ref().is_some_and(|s| s.uses_lfs));
    }
    if let Ok(tree) = &status_result {
        lfs_gate.observe_status(tree);
    }
    gitpoulpi_ipc::timing::record("tui", || {
        format!(
            "refresh {} all eight queries in {}",
            conn.repo.root.display(),
            gitpoulpi_ipc::timing::ms(started.elapsed())
        )
    });
    if defer_lfs && scope.intersects(ChangeScope::LOG) {
        // The first load (the only one that defers LFS): the graph's own
        // line, then the caller adds the whole open.
        crate::activity_log::timing_record(
            &conn.repo.root,
            gitpoulpi_i18n::msg::activity_log::OP_GRAPH
                .get()
                .to_string(),
            log_ms.load(std::sync::atomic::Ordering::Relaxed),
        );
    }
    let in_scope = |part: ChangeScope| scope.intersects(part);
    let stashes_hash = in_scope(ChangeScope::STASHES).then(|| json_hash(&stashes_result));
    let read_hashes = ReadHashes {
        log: log_hash,
        refs: refs_hash,
        stashes: stashes_hash,
        lfs: lfs.as_ref().map(|_| debug_hash(&lfs)),
    };
    let fingerprint = fingerprint(
        scope,
        &status_result,
        [
            log_hash,
            refs_hash,
            stashes_hash,
            scope
                .intersects(ChangeScope::WORKTREES)
                .then(|| json_hash(&worktrees_result)),
            scope
                .intersects(SUBMODULES_SCOPE)
                .then(|| json_hash(&submodules_result)),
            scope
                .intersects(ChangeScope::CONFLICTS)
                .then(|| json_hash(&conflicts_result)),
            scope
                .intersects(ChangeScope::CONFLICTS)
                .then(|| json_hash(&operation_state_result)),
            scope
                .intersects(ChangeScope::CONFLICTS)
                .then(|| json_hash(&index_lock_result)),
            Some(json_hash(&remotes_result)),
            Some(json_hash(&ssh_command_result)),
            Some(json_hash(&sparse_result)),
            Some(json_hash(&repo_config_result)),
        ],
    );
    RefreshOutcome {
        generation: 0,
        fingerprint,
        read_hashes,
        log_result,
        log_limit: log_limit as usize,
        status_result,
        refs_result,
        stashes_result,
        worktrees_result,
        submodules_result,
        conflicts_result,
        remotes_result,
        ssh_command_result,
        sparse_result,
        repo_config_result,
        rebase_state_result,
        operation_state_result,
        index_lock_result,
        lfs,
        scope,
    }
}

/// [`gated`] for a read that comes with its content hash: `None` when the
/// scope leaves the read out.
async fn gated_hashed<T>(
    run: bool,
    fut: impl std::future::Future<Output = (GitResult<T>, u64)>,
) -> (GitResult<T>, Option<u64>) {
    if run {
        let (result, hash) = fut.await;
        (result, Some(hash))
    } else {
        (skipped(), None)
    }
}

/// A query this refresh's scope leaves out: never sent, and answered with
/// this placeholder, which the render loop ignores (see
/// `RefreshOutcome::scope`).
/// Whether `git sparse-checkout` has ever been used in the working tree at
/// `root`: `<git dir>/info/sparse-checkout` exists. The git dir is `.git`, or
/// where a linked worktree's / submodule's `.git` file points.
fn sparse_checkout_file(root: &std::path::Path) -> bool {
    let dot_git = root.join(".git");
    let git_dir = if dot_git.is_dir() {
        dot_git
    } else {
        let Ok(text) = std::fs::read_to_string(&dot_git) else {
            return false;
        };
        let Some(target) = text.trim().strip_prefix("gitdir:") else {
            return false;
        };
        root.join(target.trim())
    };
    git_dir.join("info").join("sparse-checkout").is_file()
}

pub(super) fn skipped<T>() -> GitResult<T> {
    Err(CoreError::Backend(
        gitpoulpi_i18n::msg::link_error::NOT_REREAD
            .get()
            .to_string(),
    ))
}

/// `fut` when `run`, else [`skipped`]. The future is only built, never
/// polled, when it is left out, so no request is sent.
pub(super) async fn gated<T>(
    run: bool,
    fut: impl std::future::Future<Output = GitResult<T>>,
) -> GitResult<T> {
    if run {
        fut.await
    } else {
        skipped()
    }
}

/// Folds the scope, the status and the per-field hashes into one number.
fn fingerprint<S: serde::Serialize>(
    scope: ChangeScope,
    status: &GitResult<S>,
    fields: [Option<u64>; 12],
) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    format!("{scope:?}").hash(&mut hasher);
    json_hash(status).hash(&mut hasher);
    fields.hash(&mut hasher);
    hasher.finish()
}

/// Hashes a result by its serialized form (its error text when it is one),
/// without building the string.
fn json_hash<T: serde::Serialize>(result: &GitResult<T>) -> u64 {
    use std::hash::{Hash, Hasher};
    struct Sink(std::collections::hash_map::DefaultHasher);
    impl std::io::Write for Sink {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.write(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut sink = Sink(std::collections::hash_map::DefaultHasher::new());
    match result {
        Ok(value) => {
            0u8.hash(&mut sink.0);
            let _ = serde_json::to_writer(&mut sink, value);
        }
        Err(err) => {
            1u8.hash(&mut sink.0);
            err.to_string().hash(&mut sink.0);
        }
    }
    sink.0.finish()
}

fn debug_hash<T: std::fmt::Debug>(value: &T) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    format!("{value:?}").hash(&mut hasher);
    hasher.finish()
}
