use super::host::NotifySend as _;
use super::*;

/// Handle the render loop holds: ways to ask for a refresh or submit a
/// mutation, and ways to receive the results (both request and
/// manual/periodic refreshes and daemon-pushed live updates arrive
/// through the same `outcome_rx`; action failures arrive on
/// `action_error_rx`; a selected commit's changed files arrive on
/// `commit_files_rx`).
pub struct DaemonLink {
    /// Which link thread this is, for the tickets it carries.
    pub(super) link_id: u64,
    /// The identity every hook raised for this link carries.
    pub(super) context: LinkContext,
    pub(super) manual_refresh_tx: tokio::sync::mpsc::UnboundedSender<()>,
    pub(super) action_tx: tokio::sync::mpsc::UnboundedSender<(Option<Ticket>, Action)>,
    pub(super) commit_files_tx: tokio::sync::mpsc::UnboundedSender<CommitFilesRequest>,
    pub(super) working_diff_tx: tokio::sync::mpsc::UnboundedSender<FileDiffRequest>,
    pub(super) file_insight_tx: tokio::sync::mpsc::UnboundedSender<FileInsightRequest>,
    pub(super) merge_tx: tokio::sync::mpsc::UnboundedSender<MergeRequest>,
    pub(super) log_page_tx:
        tokio::sync::mpsc::UnboundedSender<gitpoulpi_app::graph::LogPageRequest>,
    pub(super) commit_stats_tx: tokio::sync::mpsc::UnboundedSender<Vec<Oid>>,
    // Asked from the Log menu and the palette (plan slice S7).
    #[allow(dead_code)]
    pub(super) split_source_tx: tokio::sync::mpsc::UnboundedSender<Oid>,
    /// How deep the periodic full refresh reads, in commits.
    ///
    /// An `Arc<AtomicUsize>` shared with the link thread rather than a
    /// message, because it is a standing fact the next refresh (including
    /// one already in flight on the 2-second ticker) has to read; sent as a
    /// message it would race the ticker and the log would truncate back to
    /// the initial page under a user who scrolled into page three.
    pub(super) log_depth: Arc<AtomicUsize>,
    /// The refs the Log's walk leaves out (see `log_filter`).
    pub(super) log_filter: LogFilterCell,
    pub outcome_rx: std_mpsc::Receiver<RefreshOutcome>,
    pub action_error_rx: std_mpsc::Receiver<String>,
    /// Non-error informational status messages an action wants surfaced
    /// (currently just the outcome of a merge - fast-forwarded / merged /
    /// up to date / conflicted), kept separate from `action_error_rx` so
    /// the render loop doesn't have to fake an "error: " prefix onto text
    /// that isn't one.
    pub action_message_rx: std_mpsc::Receiver<String>,
    /// A push the remote refused because it moved on (or a lease that
    /// found it moved) - typed, so the render loop can offer "pull
    /// first / force push / cancel" rather than print a sentence.
    pub push_rejected_rx: std_mpsc::Receiver<PushRejectedNotice>,
    /// A checkout/merge/rebase/pull the daemon refused because local
    /// changes are in the way while `general.auto_stash` is `ask`: the
    /// render loop opens its "stash and continue?" dialog.
    pub autostash_rx: std_mpsc::Receiver<AutoStashNeeded>,
    pub commit_files_rx: std_mpsc::Receiver<CommitFilesOutcome>,
    pub working_diff_rx: std_mpsc::Receiver<WorkingDiffOutcome>,
    /// Answers to `request_file_insight`, for the File Insight overlay.
    pub file_insight_rx: std_mpsc::Receiver<FileInsightOutcome>,
    /// Answers to `request_merge`, for the merge conflict editor.
    pub merge_rx: std_mpsc::Receiver<MergeReply>,
    /// Answers to `request_log_page`, for the graph's lazy loading.
    pub log_page_rx: std_mpsc::Receiver<LogPageOutcome>,
    /// Answers to `request_commit_stats`, for the Log's "Changes" column.
    pub commit_stats_rx: std_mpsc::Receiver<CommitStatsOutcome>,
    /// Answers to `request_split_source`, for the split editor.
    pub split_source_rx: std_mpsc::Receiver<SplitSourceOutcome>,
    /// Why the link stopped, sent once when it does for any reason other
    /// than this handle being dropped: the daemon could not be started or
    /// reached, the connection was lost, or the link thread panicked.
    /// Nothing else arrives after it - the render loop turns it into an
    /// explicit error with a Retry (see `LinkFailure` in `event_loop.rs`)
    /// rather than waiting for a `RefreshOutcome` that will never come.
    pub failure_rx: std_mpsc::Receiver<String>,
    /// Shared with the link thread: when to read the LFS state, and the
    /// latest LFS transfer report.
    pub(super) lfs_gate: Arc<crate::lfs_ui::LfsGate>,
    /// What is running and the generation counter (see `activity`).
    pub(super) activity: Arc<crate::activity::Activity>,
}

/// Sends the link's end-of-life reason on drop, so a *panic* on the link
/// thread still reports - unwinding runs this, where code after
/// `block_on` would never be reached.
pub(super) struct LinkEndReport {
    pub(super) tx: std_mpsc::Sender<String>,
    pub(super) reason: Option<String>,
}

impl Drop for LinkEndReport {
    fn drop(&mut self) {
        if let Some(reason) = self.reason.take() {
            let _ = self.tx.send_notify(reason);
        }
    }
}

impl DaemonLink {
    /// The latest report of an LFS fetch / pull / push, if one has run
    /// (its last report has `done: true`).
    pub fn lfs_progress(&self) -> Option<(OperationKind, gitpoulpi_core::domain::LfsProgress)> {
        self.lfs_gate.progress()
    }

    /// The daemon posed the LFS filters before a pull or push and the
    /// status line has not said so yet (consumed).
    pub fn take_lfs_filters_posed(&self) -> bool {
        self.lfs_gate.take_filters_posed()
    }

    /// The next completion message should carry the "filters posed" note
    /// (consumed).
    pub fn take_lfs_filters_posed_note(&self) -> bool {
        self.lfs_gate.take_filters_posed_note()
    }

    /// This link's identity: what its host hooks and events carry.
    pub fn id(&self) -> LinkId {
        self.context.id()
    }

    /// The context ([`LinkId`] and repository root) of this link.
    pub fn context(&self) -> &LinkContext {
        &self.context
    }

    pub fn request_refresh(&self) {
        let _ = self.manual_refresh_tx.send(());
    }

    /// The agent-request mailbox (`ui_viewer`).
    pub fn ui(&self) -> &crate::ui_viewer::UiBridge {
        &self.activity.ui
    }

    /// The undo journal as last reported by the daemon.
    pub fn journal_state(&self) -> gitpoulpi_core::domain::JournalState {
        self.activity.journal.state()
    }

    /// A change made outside GitPoulpi was recorded since the last call.
    pub fn take_outside_news(&self) -> bool {
        self.activity.journal.take_outside_news()
    }

    /// The shared activity registry (the journal cell and the request queue),
    /// for a panel that keeps reading them.
    pub fn activity_handle(&self) -> Arc<crate::activity::Activity> {
        Arc::clone(&self.activity)
    }

    /// An undo or redo is running (the buttons are disabled meanwhile).
    pub fn undo_busy(&self) -> bool {
        self.activity.journal.busy()
    }

    /// The confirmation the daemon asked for, once.
    pub fn take_undo_confirm(&self) -> Option<crate::undo_ui::PendingConfirm> {
        self.activity.journal.take_confirm()
    }

    /// Starts an undo or redo of the entry the buttons show. Returns the
    /// status-line text: what started, or why nothing did.
    pub fn start_undo_redo(&self, direction: gitpoulpi_core::domain::Direction) -> String {
        let _in_link = self.context.enter();
        let state = self.journal_state();
        if let Some(why) = crate::undo_ui::reason(&state, direction, self.undo_busy()) {
            return why;
        }
        let expect = crate::undo_ui::top_id(&state, direction);
        self.send_undo_redo(direction, expect, false);
        match direction {
            gitpoulpi_core::domain::Direction::Undo => "undoing...".into(),
            gitpoulpi_core::domain::Direction::Redo => "redoing...".into(),
        }
    }

    /// "Undo whole sequence" on the newest entries of one run: the status
    /// text, or why it cannot.
    pub fn start_undo_sequence(&self) -> String {
        let _in_link = self.context.enter();
        let state = self.journal_state();
        if let Some(why) = crate::undo_ui::sequence_reason(&state, self.undo_busy()) {
            return why;
        }
        let Some(group) = state.sequence else {
            return gitpoulpi_i18n::msg::undo_ui::WHY_NO_SEQUENCE.get().into();
        };
        self.send_undo_sequence(*group, false);
        gitpoulpi_i18n::msg::undo_ui::UNDOING_SEQUENCE.get().into()
    }

    /// The same for a group and an answer to a risky entry.
    pub fn send_undo_sequence(
        &self,
        group: gitpoulpi_core::domain::SequenceGroup,
        confirmed: bool,
    ) {
        self.activity.journal.begin();
        self.submit_action(Action::UndoSequence { group, confirmed });
    }

    /// The same, after the user agreed to a risky one.
    pub fn send_undo_redo(
        &self,
        direction: gitpoulpi_core::domain::Direction,
        expect: Option<u64>,
        confirmed: bool,
    ) {
        // Marked busy at submit time, on the caller's thread: the button
        // grays out on the very next frame, not when the link picks it up.
        self.activity.journal.begin();
        self.submit_action(match direction {
            gitpoulpi_core::domain::Direction::Undo => Action::Undo { expect, confirmed },
            gitpoulpi_core::domain::Direction::Redo => Action::Redo { expect, confirmed },
        });
    }

    /// Whether the daemon announced a hidden/solo change since the last call.
    pub fn take_pending_auto_stash_changed(&self) -> bool {
        self.activity.take_pending_auto_stash_changed()
    }

    pub fn take_pending_auto_stash_gone(&self) -> Option<u32> {
        self.activity.take_pending_auto_stash_gone()
    }

    pub fn take_hosting_item_changes(&self) -> Vec<gitpoulpi_ipc::HostingChangeArgs> {
        self.activity.take_hosting_item_changes()
    }

    pub fn take_ref_visibility_changed(&self) -> bool {
        self.activity.take_ref_visibility_changed()
    }

    pub fn submit_action(&self, action: Action) {
        // A sequence step armed a ticket for this very action (`tickets::arm`).
        self.submit_as(tickets::take_armed(&action), action);
    }

    /// [`DaemonLink::submit_action`] that can be waited for: the returned
    /// ticket is reported on [`take_done`] when the action ended (see
    /// `tickets`).
    #[allow(dead_code)]
    pub fn submit_tracked(&self, action: Action) -> Ticket {
        let ticket = tickets::issue();
        self.submit_as(Some(ticket), action);
        ticket
    }

    /// Submits `action` under `ticket`, or untracked: what a dialog calls to
    /// continue the action it held (auto-stash, sensitive files, a diverged
    /// push) so the continuation reports under the original ticket.
    pub fn submit_as(&self, ticket: Option<Ticket>, action: Action) {
        // A renamed file is two paths for git: the row acts on both.
        let actions = self.activity.expand_renames(action);
        if let Some(ticket) = ticket {
            tickets::begin(ticket, actions.len(), self.link_id);
        }
        for action in actions {
            self.submit_one(ticket, action);
        }
    }

    /// Remembers the rename pairs of the status just shown, for
    /// `submit_action`.
    pub fn set_rename_pairs(&self, tree: &gitpoulpi_core::domain::WorkingTree) {
        self.activity.set_rename_pairs(tree);
    }

    fn submit_one(&self, ticket: Option<Ticket>, action: Action) {
        // The hooks below run on this (the caller's) thread: name the link.
        let _in_link = self.context.enter();
        // A transfer identical to one still running joins it: nothing is
        // sent, and the press says so (the daemon's join is the backstop for
        // the other clients).
        if let Some(kind) = self.activity.claim_transfer(&action) {
            super::host::jobs(super::host::JobEvent::Say(kind.already()));
            // A tracked one has nothing of its own to wait for: the one it
            // joined belongs to someone else.
            tickets::report_ticket(
                ticket,
                StepOutcome::Failed {
                    text: kind.already().to_string(),
                },
            );
            return;
        }
        self.activity.submitted(&action);
        let _ = self.action_tx.send((ticket, action));
    }

    /// The window's own fetch (auto-fetch, fetch on open): silent when a
    /// fetch is already going, and flagged `auto` so the daemon also skips
    /// it while any transfer of the repository runs.
    pub fn submit_auto_fetch(&self) {
        let _in_link = self.context.enter();
        let action = Action::FetchAll;
        if self.activity.claim_transfer(&action).is_some() {
            return;
        }
        super::host::expect_background_fetch();
        self.activity.submitted(&action);
        let _ = self.action_tx.send((None, action));
    }

    /// The generation counter: it moves whenever a mutation is submitted,
    /// starts or ends.
    pub fn generation(&self) -> u64 {
        self.activity.generation()
    }

    /// Staging actions submitted since the last call, for the optimistic
    /// overlay (`status_optimistic`).
    pub fn take_optimistic(&self) -> Vec<Action> {
        self.activity.take_optimistic()
    }

    /// The status-line text for what the link has been doing for a while
    /// ("Staging all changes..."), `None` when nothing is slow.
    pub fn in_flight_text(&self) -> Option<String> {
        let _in_link = self.context.enter();
        self.activity.in_flight_text(crate::activity::SHOW_AFTER)
    }

    /// When the in-flight text of an operation that has not run long enough
    /// yet will first show (a redraw is due then).
    pub fn in_flight_text_due_at(&self) -> Option<std::time::Instant> {
        let _in_link = self.context.enter();
        self.activity.text_due_at()
    }

    /// The in-flight text is on screen and its spinner moves.
    pub fn in_flight_spinning(&self) -> bool {
        let _in_link = self.context.enter();
        self.activity.spinning()
    }

    /// The Log selection's changed files (and its tree with `with_tree`).
    /// A newer call replaces this one if it has not started yet, so
    /// scrolling never queues up work behind the row the user stops on.
    pub fn request_commit_files(&self, commit: Oid, with_tree: bool) {
        self.send_fetch(commit, with_tree, CommitFilesPurpose::Selection);
    }

    /// The changed files of a comparison (`compare::CompareSpec`), answered
    /// on `commit_files_rx` under the key `id`. A newer call replaces this
    /// one if it has not started.
    pub fn request_compare_files(&self, id: Oid, target: gitpoulpi_core::domain::DiffTarget) {
        let _ = self
            .commit_files_tx
            .send(CommitFilesRequest::Compare { id, target });
    }

    /// HEAD's tree for the WIP row's `View all files`.
    /// Independent of the selection: never dropped for a newer one.
    pub fn request_head_tree(&self, head: Oid) {
        self.send_fetch(head, true, CommitFilesPurpose::Wip);
    }

    /// Fetches these commits' files into the link's cache, answering
    /// nobody, when there is nothing more urgent to do - the Log
    /// selection's neighbors, so j/k lands on a commit already held.
    pub fn prefetch_commit_files(&self, commits: Vec<Oid>) {
        let _ = self
            .commit_files_tx
            .send(CommitFilesRequest::Prefetch(commits));
    }

    pub(super) fn send_fetch(&self, commit: Oid, with_tree: bool, purpose: CommitFilesPurpose) {
        let _ = self
            .commit_files_tx
            .send(CommitFilesRequest::Fetch(QueuedFetch {
                commit,
                with_tree,
                purpose,
                sent: std::time::Instant::now(),
            }));
    }

    /// Asks for the files/lines changed by these commits (the Log's
    /// "Changes" column): the ones on screen, nothing else. A newer call
    /// replaces this one if it has not started, so scrolling never queues
    /// work for rows that are already gone.
    pub fn request_commit_stats(&self, ids: Vec<Oid>) {
        let _ = self.commit_stats_tx.send(ids);
    }

    /// Asks for the numbered changes of `commit` (the split editor's
    /// source). Fire-and-forget: the answer arrives on `split_source_rx`.
    #[allow(dead_code)] // the entry points are plan slice S7
    pub fn request_split_source(&self, commit: Oid) {
        let _ = self.split_source_tx.send(commit);
    }

    pub fn request_working_diff(&self, request: FileDiffRequest) {
        let _ = self.working_diff_tx.send(request);
    }

    /// Asks for one file-centric read (contents / blame / per-path log).
    /// Fire-and-forget like the rest: the answer arrives on
    /// `file_insight_rx`, tagged with the request it belongs to.
    pub fn request_file_insight(&self, request: FileInsightRequest) {
        // The History intent is answered by the split view's own paged
        // reads (`HistoryPage`), never by an unbounded log walk.
        if request.kind == FileInsightKind::History {
            return;
        }
        let _ = self.file_insight_tx.send(request);
    }

    /// Asks for one page of older history. Fire-and-forget: this returns
    /// immediately and the answer arrives on `log_page_rx`. **Nothing
    /// here may ever block** - it is called from the render loop, once
    /// per frame at most, and `docs/developer/where-we-are.md` records
    /// what happens when a subprocess call gets onto that loop.
    pub fn request_log_page(&self, request: gitpoulpi_app::graph::LogPageRequest) {
        let _ = self.log_page_tx.send(request);
    }

    /// Tells the periodic refresh to read this deep from now on. Called
    /// after a page is appended, or the next 2-second poll would ask for the
    /// initial page only and truncate the log under the user.
    pub fn set_log_depth(&self, commits: usize) {
        self.log_depth.store(commits, Ordering::Relaxed);
    }

    /// Adopts the filter the Log's walk starts from (hidden refs). True when
    /// it differs from the one in use, which also moves the generation: the
    /// caller restarts its paging and asks for a fresh read. Answers read
    /// under an older generation are dropped by the window.
    pub fn set_log_filter(&self, filter: gitpoulpi_core::domain::RefFilter) -> bool {
        self.log_filter.set(filter)
    }

    /// The generation of the filter in use; a Log answer with another one
    /// was read for a different walk.
    pub fn log_filter_generation(&self) -> u64 {
        self.log_filter.generation()
    }

    /// The filter in use and its generation, for the reads the window sends
    /// itself (the commit total).
    pub fn log_filter(&self) -> (u64, gitpoulpi_core::domain::RefFilter) {
        self.log_filter.snapshot()
    }
}

/// Spawns the link's background thread and returns immediately; the first
/// refresh result arrives asynchronously on `outcome_rx` once the daemon
/// connection is up (auto-starting the daemon in-process if none is found
/// running for this session - see `ensure_daemon`).
pub fn start(session: String, repo: RepoRef, host: Arc<dyn LinkHost>) -> DaemonLink {
    let port_file = session_port_file(&session);
    start_at(session, repo, port_file, host)
}

/// [`start`] against an explicit discovery file (see `ensure_daemon_at`).
pub fn start_at(
    session: String,
    repo: RepoRef,
    port_file: Option<PathBuf>,
    host: Arc<dyn LinkHost>,
) -> DaemonLink {
    // Before the thread starts, so even its first wake finds the host (the
    // fallback for a hook raised off this link's threads; the link itself
    // keeps its own, see `LinkContext`).
    super::host::install(Arc::clone(&host));
    let link_id = tickets::new_link_id();
    let context = LinkContext::new(LinkId::new(link_id), repo.root.clone(), host);
    let (manual_refresh_tx, manual_refresh_rx) = tokio::sync::mpsc::unbounded_channel();
    let (action_tx, action_rx) = tokio::sync::mpsc::unbounded_channel();
    let (commit_files_tx, commit_files_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (working_diff_tx, working_diff_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (outcome_tx, outcome_rx) = std_mpsc::channel();
    let (action_error_tx, action_error_rx) = std_mpsc::channel();
    let (action_message_tx, action_message_rx) = std_mpsc::channel();
    let (push_rejected_tx, push_rejected_rx) = std_mpsc::channel();
    let (autostash_tx, autostash_rx) = std_mpsc::channel();
    let (commit_files_out_tx, commit_files_rx) = std_mpsc::channel();
    let (working_diff_out_tx, working_diff_rx) = std_mpsc::channel();
    let (file_insight_tx, file_insight_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (file_insight_out_tx, file_insight_rx) = std_mpsc::channel();
    let (merge_tx, merge_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (merge_out_tx, merge_rx) = std_mpsc::channel();
    let (log_page_tx, log_page_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (log_page_out_tx, log_page_rx) = std_mpsc::channel();
    let (commit_stats_tx, commit_stats_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (commit_stats_out_tx, commit_stats_rx) = std_mpsc::channel();
    let stats_worker = StatsWorker {
        rx: commit_stats_task_rx,
        out_tx: commit_stats_out_tx,
    };
    let (split_source_tx, split_source_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (split_source_out_tx, split_source_rx) = std_mpsc::channel();
    let split_worker = SplitSourceWorker {
        rx: split_source_task_rx,
        out_tx: split_source_out_tx,
    };
    let log_depth = Arc::new(AtomicUsize::new(0));
    let link_log_depth = Arc::clone(&log_depth);
    // Seeded from the persisted visibility so the first load is already
    // filtered (and the window's first comparison finds no change).
    let log_filter = if repo.root == std::path::Path::new(crate::no_repository::NO_REPOSITORY) {
        LogFilterCell::default()
    } else {
        LogFilterCell::seeded(super::host::initial_log_filter(&repo.root))
    };
    let link_log_filter = log_filter.clone();
    let lfs_gate = Arc::new(crate::lfs_ui::LfsGate::default());
    let link_lfs_gate = Arc::clone(&lfs_gate);
    // Undo off in the config: the first frames must already leave the
    // buttons out, not wait for the daemon's first journal report.
    let undo_enabled = gitpoulpi_config::Settings::resolve_ambient()
        .map(|s| s.config().general.undo_enabled)
        .unwrap_or(true);
    let activity = Arc::new(crate::activity::Activity::with_undo_enabled(undo_enabled));
    let link_activity = Arc::clone(&activity);
    let (failure_tx, failure_rx) = std_mpsc::channel();
    let channels = LinkChannels {
        manual_refresh_rx,
        action_rx,
        commit_files_task_rx,
        working_diff_task_rx,
        file_insight_task_rx,
        merge_task_rx,
        log_page_task_rx,
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
        stats_worker: Some(stats_worker),
        split_worker: Some(split_worker),
        log_depth: link_log_depth,
        log_filter: link_log_filter,
        lfs_gate: link_lfs_gate,
        activity: link_activity,
    };

    let thread_context = context.clone();
    std::thread::spawn(move || {
        let _in_link = thread_context.enter();
        let mut report = LinkEndReport {
            tx: failure_tx,
            reason: Some(
                gitpoulpi_i18n::msg::link_error::LINK_PANICKED
                    .get()
                    .to_string(),
            ),
        };
        // Every thread of this runtime carries the link, so the hooks its
        // tasks raise name it (`link_ctx`).
        let worker_context = thread_context.clone();
        let rt = match tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .on_thread_start(move || worker_context.install_on_thread())
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                report.reason = Some(gitpoulpi_i18n::msg::link_error::runtime_failed(e));
                return;
            }
        };
        report.reason = rt.block_on(link_loop(port_file, session, repo, channels));
        // Its window is gone or its daemon: nothing in flight will be
        // answered to this link any more (before the runtime drops its tasks).
        tickets::abandon_link(link_id);
    });

    DaemonLink {
        link_id,
        context,
        failure_rx,
        manual_refresh_tx,
        action_tx,
        commit_files_tx,
        working_diff_tx,
        file_insight_tx,
        merge_tx,
        outcome_rx,
        action_error_rx,
        action_message_rx,
        push_rejected_rx,
        autostash_rx,
        commit_files_rx,
        working_diff_rx,
        file_insight_rx,
        merge_rx,
        log_page_tx,
        commit_stats_tx,
        commit_stats_rx,
        split_source_tx,
        split_source_rx,
        log_depth,
        log_filter,
        log_page_rx,
        lfs_gate,
        activity,
    }
}
