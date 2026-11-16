//! The in-app code review: a pull / merge request opened inside the
//! repository window, with Overview, Files and Commits pages (Alt+1/2/3).
//!
//! GitHub and GitLab only (`meta_picker::supports`). The host gives the
//! request, its threads, conversation and metadata (`gitpoulpi-hosting`'s
//! review API); the diff comes from git: the request's head and base are
//! fetched into `refs/gitpoulpi/pr/<n>` and diffed from their merge base
//! with the daemon's own diff, so a comment's line numbers are the diff's.
//!
//! The view is a full-window modal owned by `RepoDialogs`
//! (`RepoDialog::Review`): keys, mouse, paste and the per-frame poll all
//! arrive there, so popups and hit rects follow the same rules as the other
//! dialogs. It never waits: every host or daemon call runs on a worker and
//! shows an in-flight line after 150 ms.
//!
//! New code comments are **pending**: kept locally (`RepoState::reviews`,
//! so they survive a restart) until Finish review sends them with the
//! verdict in one call. Replies, resolving and conversation comments go to
//! the host at once, and the UI says so. A later slice (R3) adds suggested
//! comments the reviewer accepts into the pending list: the hook is
//! [`ReviewView::add_pending`], the one place a pending comment is born.

mod ai;
#[cfg(feature = "ai")]
mod ai_context;
mod ai_reply;
#[cfg_attr(not(feature = "ai"), path = "ai_worker_stub.rs")]
mod ai_worker;
mod barmenu;
mod diffpane;
mod diffrows;
mod draw;
mod input;
mod load;
mod model;
mod options;
mod overview;
mod pane;
mod patchset;
mod popups;
mod recode;
mod source;
#[cfg(test)]
mod tests;
mod whole;

use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

use gitpoulpi_core::domain::{FileDiff, Oid};
use gitpoulpi_hosting::{DiffSide, PendingComment, ReviewEvent, ReviewSubmission, ReviewThread};
use gitpoulpi_i18n::msg::review_view as msg;
use ratatui::layout::Rect;

use crate::daemon_link::Requester;
use crate::hosting_task::ClientFactory;
use crate::item_view::ItemView;
use crate::meta_picker::{MetaEdit, MetaEditOutcome, MetaField, MetaTarget};

use crate::review_cmd::{ReviewAvail, Tab};
use load::{Call, Done};
pub(crate) use model::ReviewTarget;
use model::{DraftStore, Focus, GitData, HostData};
use pane::DiffPane;
use popups::{ComposeBox, Confirm, FinishDialog};
use source::{ApiCode, CodeSource, Pending};

/// Where the diff pane's bar last painted its encoding button and `⋯`.
#[derive(Default)]
struct BarSlots {
    encoding: Option<Rect>,
    more: Option<(Rect, Vec<crate::diff_view::DiffBarButton>)>,
}

/// A request in flight for longer than this shows a spinner (maintainer rule 9).
const SPIN_AFTER: Duration = Duration::from_millis(150);

/// Whether the review view exists for this provider.
pub(crate) fn supports(kind: gitpoulpi_config::IntegrationKind) -> bool {
    crate::meta_picker::supports(kind) || kind.family() == gitpoulpi_config::ApiFamily::Gerrit
}

/// What the owner of the view has to do after it handled something.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReviewEffect {
    None,
    /// Leave the view.
    Close,
    Message(String),
    /// Leave the view and check out the request's branch (the existing
    /// flow, which knows whether the branch is here or must be fetched).
    Checkout(u64),
}

/// What the confirmation asks about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfirmFor {
    DiscardReview,
    /// Merge the change on the server (Gerrit).
    SubmitChange,
    /// Abandon the change on the server (Gerrit).
    AbandonChange,
}

/// A call to the host in flight.
struct InFlight {
    rx: mpsc::Receiver<Result<Done, String>>,
    since: Instant,
    /// `true`: the Finish review dialog is waiting for it.
    finishing: bool,
    /// The comment box's text, to put back if the host refuses it.
    restore: Option<(popups::ComposeFor, String)>,
}

/// Which diff pane a click or key is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PaneId {
    Files,
    Commits,
}

pub(crate) struct ReviewView {
    pub(super) target: ReviewTarget,
    factory: Arc<dyn ClientFactory>,
    source: CodeSource,
    /// Where the pending review is kept.
    store: DraftStore,
    cancel: Arc<AtomicBool>,
    started: Instant,

    // what the host and the repository said
    host: Option<HostData>,
    host_rx: Option<mpsc::Receiver<Result<HostData, String>>>,
    host_err: Option<String>,
    git: Option<GitData>,
    git_rx: Option<mpsc::Receiver<Result<GitData, String>>>,
    git_err: Option<String>,

    // the local review
    pub(super) pending: Vec<PendingComment>,
    pub(super) viewed: BTreeSet<String>,
    collapsed: HashSet<String>,
    /// Bumped whenever rows must be rebuilt (threads, pending, folds).
    revision: u64,

    // pages
    pub(super) tab: Tab,
    pub(super) focus: Focus,
    ov_offset: usize,
    ov_bar: Option<crate::scrollbar::Track>,
    ov_drag: crate::scrollbar::BarDrag,
    ov_len: usize,
    /// The conversation of the Overview: the shared timeline and docked
    /// composer (`item_view`), the same as an issue's.
    ov: ItemView,
    /// Where the Overview painted the ✦ block (the draft card and the
    /// suggestions); the conversation has the rest.
    ov_strip: Rect,
    /// The ✦ block's laid-out Markdown, and the addresses of the links
    /// painted this frame (`Hit::MdLink` indexes it).
    md_cache: crate::md_render::DocCache,
    md_links: Vec<String>,
    ov_rows: usize,
    file_sel: usize,
    file_offset: usize,
    /// Rows of the file / commit list as last painted.
    list_rows: usize,
    file_bar: Option<crate::scrollbar::Track>,
    file_drag: crate::scrollbar::BarDrag,
    commit_sel: usize,
    commit_offset: usize,
    commit_bar: Option<crate::scrollbar::Track>,
    commit_drag: crate::scrollbar::BarDrag,
    files_pane: DiffPane,
    commits_pane: DiffPane,
    /// The Diff view's display toggles (layout, hunks only, whitespace,
    /// wrap), read from and written back to the same preferences it uses.
    diff_opts: crate::diff_view::DiffViewState,
    /// The new side of the files, for the whole-file layout.
    blobs: whole::Blobs,
    /// Files read again in an encoding the reviewer chose.
    recodes: recode::Recodes,
    /// The bar's encoding list or `⋯` list while one is open.
    bar_menu: Option<barmenu::BarMenu>,
    /// Where the last paint put the bar's encoding button and `⋯` (and what the
    /// latter hides): what the menus hang from.
    bar_slots: BarSlots,
    diffs: HashMap<String, FileDiff>,
    diff_wait: Option<(String, Pending<Vec<FileDiff>>)>,
    diff_err: Option<String>,
    commit_diffs: HashMap<Oid, Vec<FileDiff>>,
    commit_wait: Option<(Oid, Pending<Vec<FileDiff>>)>,

    // the ✦ suggestions (empty and inert while AI is off)
    sug: ai::Suggest,

    // popups
    compose: Option<ComposeBox>,
    finish: Option<FinishDialog>,
    /// The "✦ Review options" dialog.
    options: Option<options::OptionsDialog>,
    confirm: Option<(ConfirmFor, Confirm)>,
    meta: Option<MetaEdit>,

    // the Gerrit patch-set combobox (patchset.rs)
    ps_combo: Option<crate::combo::Combo>,
    ps_field: Rect,
    ps_hover: bool,
    /// The "compare with" combobox next to it (older patch sets, or the base).
    cmp_combo: Option<crate::combo::Combo>,
    cmp_field: Rect,
    cmp_hover: bool,
    /// The patch set the fetched code is at.
    git_ps: Option<u32>,

    // in flight and feedback
    call: Option<InFlight>,
    message: Option<String>,

    // the paint walk's hit rects
    hits: Vec<(Rect, draw::Hit)>,
    hover: Option<draw::Hit>,
    last_click: Option<(Instant, u16, u16)>,
}

impl Drop for ReviewView {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

impl ReviewView {
    /// Opens the view and starts both loads.
    pub(crate) fn open(
        target: ReviewTarget,
        factory: Arc<dyn ClientFactory>,
        requester: Requester,
        ai_enabled: bool,
        session: String,
    ) -> Self {
        let store = match &target.repo_root {
            Some(root) => DraftStore::Repo(root.clone()),
            None => DraftStore::Remote(None),
        };
        Self::build(
            target,
            factory,
            CodeSource::Git(requester),
            store,
            ai_enabled,
            session,
        )
    }

    /// Opens the view on a request of a repository that is not cloned: the
    /// code comes from the host's API and the pending review is kept in
    /// `<config_dir>/remote-reviews.toml`. The Home window hosts it over the
    /// Radar (`radar::review_flow`).
    pub(crate) fn open_api(
        target: ReviewTarget,
        factory: Arc<dyn ClientFactory>,
        ai_enabled: bool,
        session: String,
        config_dir: Option<std::path::PathBuf>,
    ) -> Self {
        let store = DraftStore::Remote(
            config_dir
                .as_deref()
                .map(gitpoulpi_session::RemoteReviews::file_in),
        );
        let source = CodeSource::Api(Box::new(ApiCode::new(factory.clone(), target.clone())));
        Self::build(target, factory, source, store, ai_enabled, session)
    }

    fn build(
        target: ReviewTarget,
        factory: Arc<dyn ClientFactory>,
        source: CodeSource,
        store: DraftStore,
        ai_enabled: bool,
        session: String,
    ) -> Self {
        let draft = store.load(&target);
        let ov = ItemView::open_embedded(target.item_target(), factory.clone());
        let mut view = Self {
            target,
            factory,
            source,
            store,
            cancel: Arc::new(AtomicBool::new(false)),
            started: Instant::now(),
            host: None,
            host_rx: None,
            host_err: None,
            git: None,
            git_rx: None,
            git_err: None,
            pending: draft.comments.iter().map(pending_of).collect(),
            viewed: draft.viewed.into_iter().collect(),
            collapsed: HashSet::new(),
            revision: 0,
            tab: Tab::Overview,
            focus: Focus::List,
            ov_offset: 0,
            ov_bar: None,
            ov_drag: Default::default(),
            ov_len: 0,
            ov,
            ov_strip: Rect::default(),
            md_cache: Default::default(),
            md_links: Vec::new(),
            ov_rows: 0,
            file_sel: 0,
            file_offset: 0,
            list_rows: 10,
            file_bar: None,
            file_drag: Default::default(),
            commit_sel: 0,
            commit_offset: 0,
            commit_bar: None,
            commit_drag: Default::default(),
            files_pane: DiffPane::default(),
            commits_pane: DiffPane::default(),
            diff_opts: crate::diff_view::DiffViewState::for_commit(
                Oid([0u8; 20]),
                0,
                std::path::PathBuf::new(),
            )
            .resume_last_used(),
            blobs: whole::Blobs::default(),
            recodes: recode::Recodes::default(),
            bar_menu: None,
            bar_slots: BarSlots::default(),
            diffs: HashMap::new(),
            diff_wait: None,
            diff_err: None,
            commit_diffs: HashMap::new(),
            commit_wait: None,
            sug: ai::Suggest::new(ai_enabled, session),
            compose: None,
            finish: None,
            options: None,
            confirm: None,
            meta: None,
            ps_combo: None,
            ps_field: Rect::default(),
            ps_hover: false,
            cmp_combo: None,
            cmp_field: Rect::default(),
            cmp_hover: false,
            git_ps: None,
            call: None,
            message: None,
            hits: Vec::new(),
            hover: None,
            last_click: None,
        };
        if let Some(ai) = draft.ai {
            view.restore_draft(ai);
        }
        view.git_ps = view.wanted_patch_set();
        view.start_loads();
        view
    }

    fn start_loads(&mut self) {
        self.started = Instant::now();
        self.host_err = None;
        self.git_err = None;
        self.host_rx = Some(load::start_host(self.factory.clone(), self.target.clone()));
        if let Some(picked) = self.target.patch_set {
            self.git_ps = Some(picked);
        }
        self.restart_git();
    }

    /// Fetches the refs and reads the commits again (at `git_ps`).
    pub(super) fn restart_git(&mut self) {
        self.git_err = None;
        self.git_rx = Some(
            self.source
                .start(&self.target, self.git_ps, self.cancel.clone()),
        );
    }

    /// Another client wrote to this request (a comment, a review, a state
    /// change): the conversation and the host data are read again.
    pub(crate) fn on_hosting_changed(&mut self, change: &gitpoulpi_ipc::HostingChangeArgs) {
        use gitpoulpi_ipc::HostingItemKind as K;
        if change.integration_id != self.target.integration.id
            || change.project != self.target.project
            || change.number != self.target.number
            || change.kind != K::PullRequest
        {
            return;
        }
        self.ov.on_hosting_changed(change);
        // The threads, the verdicts and the chips read the host too.
        if self.host_rx.is_none() {
            self.host_rx = Some(load::start_host(self.factory.clone(), self.target.clone()));
        }
    }

    /// Reads the request again (the host and the refs). Pending comments and
    /// viewed ticks stay.
    pub(crate) fn refresh(&mut self) {
        self.ov.request_refresh();
        self.diffs.clear();
        self.commit_diffs.clear();
        self.blobs.clear();
        self.recodes.clear();
        self.diff_wait = None;
        self.commit_wait = None;
        self.diff_err = None;
        self.source.reset();
        self.revision += 1;
        self.start_loads();
    }

    fn loading(&self) -> bool {
        self.host_rx.is_some() || self.git_rx.is_some()
    }

    /// Something is moving: the spinner needs frames.
    pub(crate) fn busy(&self) -> bool {
        self.ov.busy()
            || self.loading()
            || self.call.is_some()
            || self.diff_wait.is_some()
            || self.commit_wait.is_some()
            || self.blobs.busy()
            || self.recodes.busy()
            || self.sug.running()
            || self.meta.as_ref().is_some_and(MetaEdit::busy)
    }

    pub(super) fn spinner(&self) -> char {
        crate::activity::spinner_frame(self.started.elapsed())
    }

    // -- data accessors ---------------------------------------------------------

    pub(super) fn can_range(&self) -> bool {
        self.host
            .as_ref()
            .is_some_and(|h| h.detail.capabilities.line_ranges)
    }

    fn file_count(&self) -> usize {
        self.host.as_ref().map_or(0, |h| h.files.len())
    }

    fn current_path(&self) -> Option<String> {
        self.host
            .as_ref()
            .and_then(|h| h.files.get(self.file_sel))
            .map(|f| f.path.clone())
    }

    /// `(index, thread)` of the threads on the current file.
    fn threads_of(&self, path: &str) -> Vec<(usize, &ReviewThread)> {
        let previous = self
            .host
            .as_ref()
            .and_then(|h| h.files.iter().find(|f| f.path == path))
            .and_then(|f| f.previous_path.clone());
        self.host
            .as_ref()
            .map(|h| {
                h.threads
                    .iter()
                    .enumerate()
                    .filter(|(_, t)| t.path == path || Some(&t.path) == previous.as_ref())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn pending_of(&self, path: &str) -> Vec<(usize, &PendingComment)> {
        self.pending
            .iter()
            .enumerate()
            .filter(|(_, c)| c.path == path)
            .collect()
    }

    /// Open threads (not resolved, not outdated) on `path`.
    pub(super) fn open_threads_in(&self, path: &str) -> usize {
        self.threads_of(path)
            .iter()
            .filter(|(_, t)| !t.resolved)
            .count()
    }

    /// Whether the repository blocks assignment.
    pub(super) fn assignment_blocked(&self) -> bool {
        gitpoulpi_hosting::scope::assignment_blocked(
            &self.target.integration.id,
            &self.target.project,
        )
    }

    pub(super) fn can_edit_meta(&self) -> bool {
        self.host.is_some() && crate::meta_picker::supports(self.target.integration.kind)
    }

    // -- the local review ---------------------------------------------------------

    fn read_draft(&self) -> gitpoulpi_session::ReviewDraft {
        self.store.load(&self.target)
    }

    /// Writes the pending comments and viewed ticks to the repository's
    /// state file (a small file; the same load-edit-save the other per-repo
    /// choices use).
    fn save_draft(&self) {
        let draft = gitpoulpi_session::ReviewDraft {
            comments: self.pending.iter().map(draft_of).collect(),
            viewed: self.viewed.iter().cloned().collect(),
            head_sha: self
                .host
                .as_ref()
                .map(|h| h.detail.head_sha.clone())
                .unwrap_or_default(),
            ai: self.ai_draft(),
        };
        self.store.save(&self.target, draft);
    }

    /// The one place a pending comment is born (R3's accepted suggestions
    /// come through here too).
    pub(crate) fn add_pending(&mut self, comment: PendingComment) {
        self.add_pending_batch(vec![comment]);
    }

    /// [`Self::add_pending`] for several at once (one write of the draft).
    pub(crate) fn add_pending_batch(&mut self, comments: Vec<PendingComment>) {
        if comments.is_empty() {
            return;
        }
        self.pending.extend(comments);
        self.revision += 1;
        self.save_draft();
    }

    fn delete_pending(&mut self, index: usize) {
        if index < self.pending.len() {
            self.pending.remove(index);
            self.revision += 1;
            self.save_draft();
            self.message = Some(msg::pending_deleted(self.pending.len() as u64));
        }
    }

    fn toggle_viewed(&mut self, path: &str) {
        if !self.viewed.remove(path) {
            self.viewed.insert(path.to_string());
        }
        self.save_draft();
    }

    // -- polling ------------------------------------------------------------------

    /// Drains every worker. Called once per frame by the dialog host.
    pub(crate) fn poll(&mut self) -> ReviewEffect {
        let mut effect = ReviewEffect::None;
        if let Some(rx) = &self.host_rx {
            match rx.try_recv() {
                Ok(Ok(data)) => {
                    self.host_rx = None;
                    self.host_err = None;
                    self.host = Some(data);
                    self.sync_patch_sets();
                    self.follow_shown_patch_set();
                    self.revision += 1;
                    self.file_sel = self.file_sel.min(self.file_count().saturating_sub(1));
                    // Resolved threads start folded.
                    if let Some(h) = &self.host {
                        for t in h.threads.iter().filter(|t| t.resolved) {
                            self.collapsed.insert(t.id.clone());
                        }
                        let head = &h.detail.head_sha;
                        let saved = self.read_draft().head_sha;
                        let stale = !self.pending.is_empty() && saved != *head && !saved.is_empty();
                        if stale {
                            self.message = Some(msg::MOVED_SINCE.get().to_string());
                        }
                    }
                }
                Ok(Err(why)) => {
                    self.host_rx = None;
                    crate::activity_log::report_problem(
                        crate::activity_log::ProblemSource::Hosting,
                        &why,
                    );
                    self.host_err = Some(why);
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.host_rx = None;
                    self.host_err = Some(msg::LOAD_STOPPED.get().to_string());
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        if let Some(rx) = &self.git_rx {
            match rx.try_recv() {
                Ok(Ok(data)) => {
                    self.git_rx = None;
                    self.git_err = None;
                    self.git = Some(data);
                    self.revision += 1;
                }
                Ok(Err(why)) => {
                    self.git_rx = None;
                    self.git_err = Some(why);
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.git_rx = None;
                    self.git_err = Some(msg::LOAD_STOPPED.get().to_string());
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        self.sync_overview_ai();
        if let Some(e) = self.poll_overview() {
            effect = e;
        }
        self.poll_diffs();
        if self.blobs.poll() || self.recodes.poll() {
            self.revision += 1;
        }
        self.request_diffs();
        self.poll_suggest();
        if let Some(e) = self.poll_call() {
            effect = e;
        }
        if let Some(edit) = self.meta.as_mut() {
            if let Some(outcome) = edit.poll() {
                if let Some(e) = self.on_meta(outcome) {
                    effect = e;
                }
            }
        }
        effect
    }

    fn poll_diffs(&mut self) {
        if let Some((path, reply)) = &self.diff_wait {
            if let Some(answer) = reply.take() {
                let path = path.clone();
                self.diff_wait = None;
                match answer {
                    Ok(mut files) => {
                        let fd = files.pop().unwrap_or(FileDiff {
                            path: path.clone().into(),
                            status: gitpoulpi_core::domain::FileStatus::Modified,
                            hunks: Vec::new(),
                        });
                        self.diffs.insert(path, fd);
                        self.revision += 1;
                    }
                    Err(why) => self.diff_err = Some(why),
                }
            }
        }
        if let Some((oid, reply)) = &self.commit_wait {
            if let Some(answer) = reply.take() {
                let oid = *oid;
                self.commit_wait = None;
                match answer {
                    Ok(files) => {
                        self.commit_diffs.insert(oid, files);
                        self.revision += 1;
                    }
                    Err(why) => self.diff_err = Some(why),
                }
            }
        }
    }

    /// Asks the daemon for the diff the open page needs and does not have.
    fn request_diffs(&mut self) {
        // While the refs are being read again (another patch set, another
        // compare base) `git` is the old reading: a diff asked for now would
        // be cached against the wrong base.
        if self.git_rx.is_some() {
            return;
        }
        let Some(git) = &self.git else { return };
        let (head, base) = (git.head, git.diff_base);
        if self.tab == Tab::Files && self.diff_wait.is_none() && self.diff_err.is_none() {
            if let Some(path) = self.current_path() {
                if !self.diffs.contains_key(&path) {
                    let reply = self.source.file_diff(base, head, &path);
                    self.diff_wait = Some((path, reply));
                }
            }
        }
        if self.tab == Tab::Commits && self.commit_wait.is_none() && self.diff_err.is_none() {
            if let Some(commit) = git.commits.get(self.commit_sel) {
                let oid = commit.id;
                if !self.commit_diffs.contains_key(&oid) {
                    // Compared with an older patch set: against it.
                    let against = commit.parents.first().copied().filter(|_| git.comparing);
                    let reply = self.source.commit_diff(oid, against);
                    self.commit_wait = Some((oid, reply));
                }
            }
        }
        self.request_whole();
        self.request_recodes();
    }

    /// With the Hunks toggle off, the new side of the files the page shows.
    fn request_whole(&mut self) {
        if self.diff_opts.hunks_only {
            return;
        }
        let Some(git) = &self.git else { return };
        let mut wanted: Vec<(Oid, String)> = Vec::new();
        match self.tab {
            Tab::Files => {
                if let Some(path) = self.current_path() {
                    if self.diffs.get(&path).is_some_and(whole::needs_content) {
                        wanted.push((git.head, path));
                    }
                }
            }
            Tab::Commits => {
                if let Some(commit) = git.commits.get(self.commit_sel) {
                    for file in self.commit_diffs.get(&commit.id).into_iter().flatten() {
                        if whole::needs_content(file) {
                            wanted.push((commit.id, file.path.display().to_string()));
                        }
                    }
                }
            }
            Tab::Overview => {}
        }
        // A file read in a chosen encoding brings its own new side.
        // (Only a clone can read a file in another encoding.)
        if !self.source.is_api() {
            wanted.retain(|(_, path)| {
                crate::diff_recode::remembered(std::path::Path::new(path)).is_none()
            });
        }
        self.blobs.request(&self.source, wanted);
    }

    /// The files of the open page the reviewer chose an encoding for, to read
    /// again (`recode`).
    fn request_recodes(&mut self) {
        let mut wanted: Vec<recode::Key> = Vec::new();
        match self.tab {
            Tab::Files => {
                if let (Some(path), Some(sides)) = (self.current_path(), self.sides_of(None)) {
                    if let (Some(fd), Some(cs)) = (
                        self.diffs.get(&path),
                        crate::diff_recode::remembered(std::path::Path::new(&path)),
                    ) {
                        wanted.push(recode::Key::new(&path, fd.status, sides, cs));
                    }
                }
            }
            Tab::Commits => {
                let sides = self.sides_of(Some(self.commit_sel));
                let id = self
                    .git
                    .as_ref()
                    .and_then(|g| g.commits.get(self.commit_sel));
                if let (Some(sides), Some(c)) = (sides, id) {
                    for fd in self.commit_diffs.get(&c.id).into_iter().flatten() {
                        let path = fd.path.display().to_string();
                        if let Some(cs) =
                            crate::diff_recode::remembered(std::path::Path::new(&path))
                        {
                            wanted.push(recode::Key::new(&path, fd.status, sides, cs));
                        }
                    }
                }
            }
            Tab::Overview => {}
        }
        if let Some(requester) = self.source.requester() {
            self.recodes.request(requester, wanted);
        }
    }

    // -- calls to the host ----------------------------------------------------------

    fn start_call(
        &mut self,
        call: Call,
        finishing: bool,
        restore: Option<(popups::ComposeFor, String)>,
    ) -> bool {
        if self.call.is_some() {
            self.message = Some(msg::BUSY_CALL.get().to_string());
            return false;
        }
        self.message = None;
        self.call = Some(InFlight {
            rx: load::start_call(self.factory.clone(), self.target.clone(), call),
            since: Instant::now(),
            finishing,
            restore,
        });
        true
    }

    fn poll_call(&mut self) -> Option<ReviewEffect> {
        let flight = self.call.as_ref()?;
        let result = match flight.rx.try_recv() {
            Ok(result) => result,
            Err(mpsc::TryRecvError::Empty) => return None,
            Err(mpsc::TryRecvError::Disconnected) => Err(msg::LOAD_STOPPED.get().to_string()),
        };
        let flight = self.call.take()?;
        match result {
            Ok(done) => {
                self.apply_done(done);
                if flight.finishing {
                    self.finish = None;
                }
                None
            }
            Err(why) => {
                crate::activity_log::report_problem(
                    crate::activity_log::ProblemSource::Hosting,
                    &why,
                );
                match self.finish.as_mut() {
                    Some(dialog) if flight.finishing => {
                        dialog.busy = false;
                        dialog.message = Some(why);
                    }
                    _ => self.message = Some(msg::call_failed(&why)),
                }
                // The words are not lost: the box opens again with them.
                if let Some((purpose, text)) = flight.restore {
                    self.open_compose(purpose, text);
                }
                None
            }
        }
    }

    fn apply_done(&mut self, done: Done) {
        match done {
            Done::Reply { thread_id, comment } => {
                // The Overview shows the thread as well.
                self.ov.request_refresh();
                if let Some(t) = self
                    .host
                    .as_mut()
                    .and_then(|h| h.threads.iter_mut().find(|t| t.id == thread_id))
                {
                    t.comments.push(comment);
                }
                self.message = Some(msg::REPLY_POSTED.get().to_string());
            }
            Done::Resolved {
                thread_id,
                resolved,
            } => {
                self.ov.request_refresh();
                if let Some(t) = self
                    .host
                    .as_mut()
                    .and_then(|h| h.threads.iter_mut().find(|t| t.id == thread_id))
                {
                    t.resolved = resolved;
                }
                if resolved {
                    self.collapsed.insert(thread_id);
                    self.message = Some(msg::THREAD_RESOLVED.get().to_string());
                } else {
                    self.collapsed.remove(&thread_id);
                    self.message = Some(msg::THREAD_REOPENED.get().to_string());
                }
            }
            Done::Submitted(outcome) => {
                let sent = self.pending.len();
                self.pending.clear();
                // The ✦ draft was the review's text: it is spent.
                self.clear_draft();
                self.save_draft();
                self.message = Some(msg::review_submitted(sent as u64));
                let _ = outcome;
                // The submitted comments are threads now: read them back.
                self.refresh();
                crate::hosting_ui::request_issues_refresh();
            }
            Done::ChangeSubmitted => {
                self.message = Some(msg::change_submitted(&self.target.iid_label));
                self.refresh();
                crate::hosting_ui::request_issues_refresh();
            }
            Done::ChangeAbandoned => {
                self.message = Some(msg::change_abandoned(&self.target.iid_label));
                self.refresh();
                crate::hosting_ui::request_issues_refresh();
            }
        }
        self.revision += 1;
    }

    // -- the metadata pickers ---------------------------------------------------------

    pub(super) fn open_meta(&mut self, field: MetaField) {
        if field == MetaField::Assignees && self.assignment_blocked() {
            self.message = Some(
                gitpoulpi_i18n::msg::hosting_ui::WHY_ASSIGNMENT_BLOCKED
                    .get()
                    .to_string(),
            );
            return;
        }
        if !self.can_edit_meta() {
            self.message = Some(msg::WHY_NO_META.get().to_string());
            return;
        }
        let target = MetaTarget {
            integration: self.target.integration.clone(),
            all: self.target.all.clone(),
            project: self.target.project.clone(),
            kind: gitpoulpi_hosting::MetaKind::PullRequest,
            number: Some(self.target.number),
            caption: self.target.caption(),
        };
        self.meta = Some(MetaEdit::new(self.factory.clone(), target, field));
    }

    fn on_meta(&mut self, outcome: MetaEditOutcome) -> Option<ReviewEffect> {
        match outcome {
            MetaEditOutcome::Stay => None,
            MetaEditOutcome::Cancel => {
                self.meta = None;
                None
            }
            MetaEditOutcome::Saved(message) => {
                self.meta = None;
                crate::hosting_ui::request_issues_refresh();
                // The chips read the host: ask again.
                self.host_rx = Some(load::start_host(self.factory.clone(), self.target.clone()));
                Some(ReviewEffect::Message(message))
            }
        }
    }

    // -- the finish dialog ---------------------------------------------------------

    fn open_finish(&mut self) {
        let Some(h) = &self.host else {
            self.message = Some(msg::NOT_LOADED.get().to_string());
            return;
        };
        if !h.detail.capabilities.review {
            self.message = Some(msg::WHY_NO_REVIEW.get().to_string());
            return;
        }
        let summary = (self.ai_on() && !self.sug.summary.trim().is_empty())
            .then(|| self.sug.summary.trim().to_string());
        let draft_key = crate::md_editor::draft::key(&[
            &self.target.integration.id,
            &self.target.project,
            &format!("pr#{}", self.target.number),
            "finish",
        ]);
        let ai = self.composer_ai(None);
        self.finish = Some(
            FinishDialog::new(
                h.detail.capabilities,
                self.pending.len(),
                summary,
                &h.detail.vote_labels,
            )
            .for_target(self.target.md_context(), draft_key)
            .with_ai(ai),
        );
    }

    fn submit_review(&mut self, event: ReviewEvent, body: String, votes: Vec<(String, i32)>) {
        let Some(h) = &self.host else { return };
        let review = ReviewSubmission {
            votes: votes
                .into_iter()
                .map(|(label, value)| gitpoulpi_hosting::ReviewVote { label, value })
                .collect(),
            event,
            body,
            comments: self.pending.clone(),
            head_sha: Some(h.detail.head_sha.clone()),
        };
        if self.start_call(Call::Submit(review), true, None) {
            if let Some(dialog) = self.finish.as_mut() {
                dialog.busy = true;
                dialog.message = None;
            }
        }
    }

    /// What the palette may ask of the AI help of the open comment box or
    /// Finish dialog (`None`: none is open, or has no help).
    pub(crate) fn composer_avail(&self) -> Option<crate::composer_cmd::ComposerAvail> {
        self.compose
            .as_ref()
            .and_then(ComposeBox::composer_avail)
            .or_else(|| self.finish.as_ref().and_then(FinishDialog::composer_avail))
            .or_else(|| {
                (self.tab == Tab::Overview)
                    .then(|| self.ov.composer_avail())
                    .flatten()
            })
    }

    /// Runs a palette row on that help; the sentence for the status line.
    pub(crate) fn run_composer(&mut self, cmd: crate::composer_cmd::ComposerCmd) -> Option<String> {
        if let Some(c) = self.compose.as_mut() {
            c.run_composer(cmd)
        } else if let Some(f) = self.finish.as_mut() {
            f.run_composer(cmd)
        } else if self.tab == Tab::Overview {
            self.ov.run_composer(cmd)
        } else {
            None
        }
    }

    /// What the palette needs to gray a conversation command out: the
    /// Overview's conversation has the commands of an issue's (`None`: the
    /// page is not the Overview).
    pub(crate) fn item_avail(&self) -> Option<crate::item_cmd::ItemAvail> {
        (self.tab == Tab::Overview).then(|| self.ov.avail())
    }

    /// Runs a conversation command of the palette on the Overview.
    pub(crate) fn run_item(&mut self, cmd: crate::item_cmd::ItemCmd) -> ReviewEffect {
        let effect = self.ov.run(cmd);
        self.on_overview_effect(effect)
    }

    /// No popup is open over the view: the palette may take the keyboard.
    pub(crate) fn takes_palette(&self) -> bool {
        (self.tab != Tab::Overview || self.ov.takes_palette())
            && self.compose.is_none()
            && self.finish.is_none()
            && self.options.is_none()
            && self.confirm.is_none()
            && self.meta.is_none()
            && !self.patch_sets_open()
            && self.bar_menu.is_none()
    }

    // -- the palette's view of the state ---------------------------------------------

    pub(crate) fn avail(&self) -> ReviewAvail {
        let caps = self.host.as_ref().map(|h| h.detail.capabilities);
        let thread = self.thread_at_cursor();
        ReviewAvail {
            tab: self.tab,
            loaded: self.host.is_some(),
            diff_ready: self.tab == Tab::Files && !self.files_pane.is_empty(),
            on_code: self.tab == Tab::Files && self.comment_target().is_ok(),
            on_thread: thread.is_some() && self.tab == Tab::Files,
            can_resolve: thread.is_some_and(|i| {
                self.host
                    .as_ref()
                    .and_then(|h| h.threads.get(i))
                    .is_some_and(|t| t.resolvable)
            }) && caps.is_some_and(|c| c.resolve),
            on_pending: self.pending_at_cursor().is_some() && self.tab == Tab::Files,
            pending: self.pending.len(),
            can_edit_meta: self.can_edit_meta(),
            assignment_blocked: self.assignment_blocked(),
            patch_sets: self.has_patch_sets(),
            can_submit: self.host.as_ref().is_some_and(|h| h.detail.can_submit),
            can_abandon: self.host.as_ref().is_some_and(|h| h.detail.can_abandon),
            busy: self.call.is_some(),
            ai: self.ai_on(),
            ai_running: self.sug.running(),
            ai_ready: self.host.is_some() && self.git_err.is_none(),
            suggestions: self.sug.items.len(),
            on_suggestion: self.current_suggestion().is_some(),
            has_summary: !self.sug.summary.trim().is_empty(),
            has_draft: self.has_draft(),
        }
    }
}

fn pending_of(c: &gitpoulpi_session::DraftComment) -> PendingComment {
    PendingComment {
        path: c.path.clone(),
        previous_path: c.previous_path.clone(),
        line: c.line,
        side: DiffSide::parse(&c.side).unwrap_or_default(),
        start_line: c.start_line,
        other_line: c.other_line,
        body: c.body.clone(),
    }
}

fn draft_of(c: &PendingComment) -> gitpoulpi_session::DraftComment {
    gitpoulpi_session::DraftComment {
        path: c.path.clone(),
        previous_path: c.previous_path.clone(),
        line: c.line,
        side: c.side.as_str().to_string(),
        start_line: c.start_line,
        other_line: c.other_line,
        body: c.body.clone(),
    }
}
