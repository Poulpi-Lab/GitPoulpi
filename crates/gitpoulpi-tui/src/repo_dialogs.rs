//! The Repository Management tab's three dialogs -- Browse (the folder
//! picker, to open a repository), Init and Clone -- and everything they
//! run in the background.
//!
//! The dialogs themselves are pure (`folder_picker`, `init_dialog`,
//! `clone_dialog`); this module owns them, the folder lister's threads
//! (`folder_listing::Lister`), the `git init` / `git clone` tasks
//! (`init_task`, `clone_task`), and turns their outcomes into
//! [`DialogEffect`]s the event loop applies (open a tab, say something).
//! The event loop only routes keys, mouse events and pastes here, drains
//! [`RepoDialogs::poll`] once per iteration, and draws
//! [`RepoDialogs::render`] -- nothing in here ever waits.

use super::*;
use gitpoulpi_i18n::msg;

use std::sync::Mutex;

use crossterm::event::MouseEvent;

#[cfg(feature = "hosting")]
use crate::board_cmd::BoardCmd;
#[cfg(feature = "hosting")]
use crate::board_view::{BoardEffect, BoardTarget, BoardView};
use crate::clone_dialog::{hosted_sources, CloneDialog, CloneOutcome};
use crate::folder_listing::{Lister, ListerEvent};
use crate::folder_picker::{picker_rect, FolderPicker, PickerOutcome};
use crate::init_dialog::{init_rect, InitDialog, InitOutcome};
#[cfg(feature = "hosting")]
use crate::issue_form::{
    issue_rect, IssueDialog, IssueEvent, IssueOutcome, IssueReceiver, IssueSeed,
};
#[cfg(feature = "hosting")]
use crate::item_cmd::ItemCmd;
#[cfg(feature = "hosting")]
use crate::item_view::{ItemEffect, ItemTarget, ItemView};
#[cfg(feature = "hosting")]
use crate::meta_picker::{MetaEdit, MetaEditOutcome, MetaField, MetaTarget};
#[cfg(feature = "hosting")]
use crate::pull_request::{
    pull_request_rect, PullRequestDialog, PullRequestEvent, PullRequestOutcome,
    PullRequestReceiver, PullRequestSeed,
};
#[cfg(feature = "hosting")]
use crate::review_cmd::ReviewCmd;
#[cfg(feature = "hosting")]
use crate::review_view::{ReviewEffect, ReviewTarget, ReviewView};
use crate::safety_panel::{PanelEffect, PanelOutcome, SafetyPanel};
use crate::sparse_dialog::{sparse_rect, SparseDialog, SparseOutcome};
use crate::workspace_ui::dialog::{rect as workspace_rect, Outcome as WsOutcome, WorkspaceDialog};
use crate::worktree_dialog::{WorktreeDialog, WorktreeOutcome};

/// The last folder a picker chose, across windows (the management tab is
/// its own window, restarted on every visit): the next picker, and the
/// Init dialog's folder, start there.
static LAST_FOLDER: Mutex<Option<PathBuf>> = Mutex::new(None);

fn last_folder() -> Option<PathBuf> {
    LAST_FOLDER.lock().ok().and_then(|g| g.clone())
}

fn remember_folder(dir: &Path) {
    if let Ok(mut guard) = LAST_FOLDER.lock() {
        *guard = Some(dir.to_path_buf());
    }
}

/// The open dialog. Init and Clone can have the folder picker open over
/// them (their Browse button).
#[allow(clippy::large_enum_variant)]
pub(crate) enum RepoDialog {
    Browse(FolderPicker),
    Init {
        form: InitDialog,
        picker: Option<FolderPicker>,
    },
    Clone {
        form: Box<CloneDialog>,
        picker: Option<FolderPicker>,
    },
    /// Create worktree (a repository tab's ref / commit menus), with the picker over it for Browse.
    Worktree {
        form: Box<WorktreeDialog>,
        picker: Option<FolderPicker>,
    },
    /// Remove a worktree, or choose where to merge its branch.
    #[cfg(feature = "worktree")]
    WorktreeRemove(Box<crate::worktree_remove_dialog::WorktreeRemoveDialog>),
    /// Start a pull request: the token-
    /// holding path only - no token opens a browser URL directly and
    /// never reaches this. No picker: nothing here names a folder.
    #[cfg(feature = "hosting")]
    PullRequest {
        form: Box<PullRequestDialog>,
    },
    /// New issue, for the provider and repository the ISSUES section is
    /// linked to.
    #[cfg(feature = "hosting")]
    Issue {
        form: Box<IssueDialog>,
    },
    /// Assignees, reviewers, labels or milestone of an existing request or
    /// issue (`meta_picker`).
    #[cfg(feature = "hosting")]
    Meta(Box<MetaEdit>),
    /// The in-app code review of a pull / merge request (`review_view`).
    #[cfg(feature = "hosting")]
    Review(Box<ReviewView>),
    /// The Kanban board of the ISSUES source (`board_view`).
    #[cfg(feature = "hosting")]
    Board(Box<BoardView>),
    /// The conversation of an issue or a request (`item_view`).
    #[cfg(feature = "hosting")]
    Item(Box<ItemView>),
    /// "Create patch...": choose a folder to
    /// save `source` into, under the first free `name[-n].patch`.
    SavePatch {
        source: gitpoulpi_core::domain::PatchSource,
        name: String,
        picker: FolderPicker,
    },
    /// "Apply patch...": choose the patch file to apply.
    ApplyPatch(FolderPicker),
    /// "Repository: Sparse Checkout...": see and edit the checked-out
    /// directories.
    Sparse(SparseDialog),
    /// Flow: Start / Finish / Sync / Promote / Set parent (`flow::dialog`).
    Flow(Box<crate::flow::FlowDialog>),
    /// Host-driven flows: New issue, Pick up, a branch appeared
    /// (`flow::issue_flow`).
    #[cfg(feature = "hosting")]
    IssueFlow(Box<crate::flow::issue_flow::IssueFlowDialog>),
    /// Undo history, Reflog or Lost work (`safety_panel`).
    Safety(Box<SafetyPanel>),
    /// New / Edit workspace (`workspace_ui::dialog`), with the picker over it
    /// for Browse.
    Workspace {
        form: Box<WorkspaceDialog>,
        picker: Option<FolderPicker>,
    },
    /// A workspace repository gone from disk: choose where it lives now.
    Locate {
        id: String,
        old: PathBuf,
        picker: FolderPicker,
    },
}

/// What the event loop has to do about something a dialog did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DialogEffect {
    /// Say this in the status line.
    Message(String),
    /// Open (or focus) the repository at this path, as a tab.
    Open(PathBuf),
    /// Open the Integrations settings (the Clone dialog's connect note).
    OpenIntegrations,
    /// A repository was just created or cloned: open it as a tab and
    /// move to it.
    Created { repo: RepoRef, message: String },
    /// Create worktree was submitted: the event loop sends it to the
    /// daemon and, with `open_tab`, opens the worktree once it is listed.
    AddWorktree {
        path: PathBuf,
        opts: gitpoulpi_core::domain::WorktreeAddOptions,
        open_tab: bool,
    },
    /// Remove worktree was confirmed (with whatever saves its changes).
    #[cfg(feature = "worktree")]
    RemoveWorktree {
        path: PathBuf,
        opts: gitpoulpi_core::domain::WorktreeRemoveOptions,
        delete_branch: Option<String>,
    },
    /// "Open to commit": the worktree as a tab, commit message focused.
    #[cfg(feature = "worktree")]
    OpenWorktreeToCommit(PathBuf),
    /// Merge a worktree's branch into the branch checked out at `into`.
    #[cfg(feature = "worktree")]
    MergeWorktree {
        into: PathBuf,
        from: gitpoulpi_core::domain::Ref,
        then_remove: Option<(PathBuf, Option<String>)>,
    },
    /// Create patch was submitted: the event loop sends it to the daemon.
    CreatePatch {
        source: gitpoulpi_core::domain::PatchSource,
        dest: PathBuf,
    },
    /// Apply patch was submitted: the event loop sends it to the daemon.
    ApplyPatch(PathBuf),
    /// The Sparse checkout dialog's Apply: make the checkout exactly these
    /// directories.
    SparseSet(Vec<String>),
    /// The Sparse checkout dialog's Disable: back to a full checkout.
    SparseDisable,
    /// The panel's Show in Log: put the Log on this commit.
    ShowCommit(gitpoulpi_core::domain::Oid),
    /// A conversation's commit link: put the Log on the commit this
    /// abbreviated id names.
    #[cfg_attr(not(feature = "hosting"), allow(dead_code))]
    ShowCommitPrefix(String),
    /// The panel's Cherry-pick.
    CherryPick(gitpoulpi_core::domain::Oid),
    /// The review view's Check out: the existing pull request checkout flow.
    #[cfg_attr(not(feature = "hosting"), allow(dead_code))]
    CheckoutPullRequest(u64),
    /// The pull request form's `Generate title and description`: the event
    /// loop starts the AI request for this branch pair.
    #[cfg_attr(not(feature = "hosting"), allow(dead_code))]
    GeneratePullRequest { source: String, target: String },
    /// The Create worktree dialog's ✦: the event loop starts the AI request
    /// that names the worktree and its branch.
    #[cfg_attr(not(feature = "ai"), allow(dead_code))]
    NameWorktree {
        description: String,
        branches: Vec<String>,
    },
    /// Stop the running worktree naming (Stop, Esc, closing the dialog).
    #[cfg_attr(not(feature = "ai"), allow(dead_code))]
    StopWorktreeAi,
    /// A workspace walk's line for the Home page's footer (`None` = over).
    WorkspaceProgress(Option<String>),
}

/// What a Create patch menu row or palette command hands `RepoDialogs`
/// to open the Create patch dialog with - `WorktreeSeed`'s counterpart
/// for patches.
pub(crate) struct PatchSeed {
    pub(crate) source: gitpoulpi_core::domain::PatchSource,
    /// The proposed base filename (without `.patch`); see
    /// `first_free_patch_name`.
    pub(crate) name: String,
}

pub(crate) struct RepoDialogs {
    session: String,
    pub dialog: Option<RepoDialog>,
    lister: Lister,
    init_rx: Option<init_task::Receiver>,
    /// Repository lists being fetched for the Clone dialog's hosted
    /// sources.
    #[cfg(feature = "hosting")]
    projects_rx: Vec<crate::hosting_task::ProjectsReceiver>,
    #[cfg(feature = "hosting")]
    pr_rx: Option<PullRequestReceiver>,
    #[cfg(feature = "hosting")]
    issue_rx: Option<IssueReceiver>,
    /// Jobs-list entries of the two sends above (0: none), see
    /// [`RepoDialogs::list_send`].
    #[cfg(feature = "hosting")]
    send_jobs: [u64; 2],
    /// The board that opened the New issue form, kept to come back to.
    #[cfg(feature = "hosting")]
    parked_board: Option<Box<BoardView>>,
    /// Builds the host client for the New issue call (a fake in tests).
    #[cfg(feature = "hosting")]
    issue_factory: std::sync::Arc<dyn crate::hosting_task::ClientFactory>,
    last_click: Option<(Instant, u16, u16)>,
    /// The request the open Clone dialog was prefilled for (Radar: "Clone it,
    /// then review"); moved to `after_clone` when the clone starts.
    review_clone: Option<crate::after_clone::Carry>,
    /// The repository's sparse checkout as the last refresh read it: what
    /// the Sparse checkout dialog opens with.
    sparse: gitpoulpi_core::domain::SparseState,
    /// Workspace jobs in flight (`workspace_ui::task`): a save, a walk.
    ws_jobs: Vec<crate::workspace_ui::task::Receiver>,
    /// The frame the dialogs were last drawn in: where "outside" starts.
    area: Rect,
    /// Where the composers' "Write with AI" help reads its settings from;
    /// `None` while AI is off (no composer shows the help).
    ai_scope: Option<crate::md_editor::ai::AiScope>,
}

/// Workspace walks whose window went away (a tab switch): they keep running,
/// listed in the jobs panel, and the next window takes them back to report
/// how they ended (see the background-ops design, B11).
static PARKED_WS: std::sync::Mutex<Vec<crate::workspace_ui::task::Receiver>> =
    std::sync::Mutex::new(Vec::new());

impl Drop for RepoDialogs {
    fn drop(&mut self) {
        // A send to the host dies with its dialog (the answer has nowhere to
        // go), so its entry in the jobs list ends with it.
        #[cfg(feature = "hosting")]
        for id in self.send_jobs {
            if id != 0 {
                crate::jobs_ui::local_finish(id, gitpoulpi_ipc::JobState::Done);
            }
        }
        let jobs = std::mem::take(&mut self.ws_jobs);
        PARKED_WS
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .extend(jobs);
    }
}

impl RepoDialogs {
    pub(crate) fn new(session: &str) -> Self {
        Self {
            session: session.to_string(),
            dialog: None,
            lister: Lister::default(),
            init_rx: None,
            #[cfg(feature = "hosting")]
            projects_rx: Vec::new(),
            #[cfg(feature = "hosting")]
            pr_rx: None,
            #[cfg(feature = "hosting")]
            issue_rx: None,
            #[cfg(feature = "hosting")]
            send_jobs: [0; 2],
            #[cfg(feature = "hosting")]
            parked_board: None,
            #[cfg(feature = "hosting")]
            issue_factory: std::sync::Arc::new(crate::hosting_task::DefaultFactory),
            last_click: None,
            review_clone: None,
            sparse: Default::default(),
            ws_jobs: std::mem::take(&mut *PARKED_WS.lock().unwrap_or_else(|e| e.into_inner())),
            area: Rect::default(),
            ai_scope: None,
        }
    }

    /// A workspace job is running (a walk or a save): one at a time.
    pub(crate) fn ws_busy(&self) -> bool {
        !self.ws_jobs.is_empty()
    }

    /// Lists a create-PR / create-issue call in the jobs panel. Past the
    /// send it cannot be stopped (the host may already have acted), so the
    /// entry is there to be seen, with the reason on its disabled Cancel.
    #[cfg(feature = "hosting")]
    fn list_send(&mut self, slot: usize, what: String) {
        crate::jobs_ui::local_finish(self.send_jobs[slot], gitpoulpi_ipc::JobState::Done);
        self.send_jobs[slot] = crate::jobs_ui::local_start_sent(what);
    }

    #[cfg(feature = "hosting")]
    fn unlist_send(&mut self, slot: usize, state: gitpoulpi_ipc::JobState) {
        crate::jobs_ui::local_finish(std::mem::take(&mut self.send_jobs[slot]), state);
    }

    /// Starts a walk over workspace `name`'s repositories and lists it in the
    /// jobs panel, where its Cancel stops it after the repository it is on.
    pub(crate) fn start_job(&mut self, mut job: crate::workspace_ui::task::Receiver, name: &str) {
        let stop = job.stopper();
        job.workspace = name.to_string();
        job.job = crate::jobs_ui::local_start(
            msg::jobs::label(msg::jobs::KIND_WORKSPACE.get(), name),
            std::sync::Arc::new(move || stop.store(true, std::sync::atomic::Ordering::SeqCst)),
        );
        self.ws_jobs.push(job);
    }

    /// Opens the New / Edit workspace form for `pending`.
    fn open_workspace(&mut self, pending: crate::workspace_ui::apply::Pending) {
        if let Some(seed) = crate::workspace_ui::apply::seed(pending, &self.session) {
            self.dialog = Some(RepoDialog::Workspace {
                form: Box::new(WorkspaceDialog::new(seed)),
                picker: None,
            });
        }
    }

    /// Locate...: the folder picker, to say where a missing repository is now.
    pub(crate) fn open_locate(&mut self, id: String, old: PathBuf) {
        let start = old
            .parent()
            .filter(|p| p.is_dir())
            .map(Path::to_path_buf)
            .or_else(last_folder)
            .or_else(gitpoulpi_config::home_dir)
            .unwrap_or_default();
        let title = crate::workspace_ui::apply::locate_title(&old);
        let picker = self.new_picker(&title, msg::folder_picker::CONFIRM_SELECT.get(), start);
        self.dialog = Some(RepoDialog::Locate { id, old, picker });
        self.pump();
    }

    pub(crate) fn set_sparse(&mut self, state: &gitpoulpi_core::domain::SparseState) {
        self.sparse = state.clone();
    }

    /// Opens a Flow dialog (`flow::dialog`).
    pub(crate) fn open_flow(
        &mut self,
        seed: crate::flow::FlowSeed,
        actions: std::sync::Arc<dyn crate::flow::FlowActions>,
    ) {
        self.dialog = Some(RepoDialog::Flow(Box::new(crate::flow::FlowDialog::new(
            seed, actions,
        ))));
    }

    /// Opens a host-driven flow dialog (`flow::issue_flow`).
    #[cfg(feature = "hosting")]
    pub(crate) fn open_issue_flow(&mut self, seed: crate::flow::issue_flow::IssueFlowSeed) {
        self.dialog = Some(RepoDialog::IssueFlow(Box::new(
            crate::flow::issue_flow::IssueFlowDialog::new(seed),
        )));
    }

    /// Opens the Sparse checkout dialog on the state the last refresh read.
    pub(crate) fn open_sparse(&mut self) {
        self.dialog = Some(RepoDialog::Sparse(SparseDialog::new(self.sparse.clone())));
    }

    /// Opens the Undo history / Reflog / Lost work panel on `tab`
    /// (`refname`: the Reflog tab's ref, `HEAD` by default).
    pub(crate) fn open_safety(
        &mut self,
        link: &crate::daemon_link::DaemonLink,
        tab: crate::safety_panel::Tab,
        refname: Option<String>,
    ) {
        let depth = gitpoulpi_config::Settings::resolve_ambient()
            .ok()
            .map_or(100, |s| s.config().general.undo_depth as usize);
        // Open already: switch it to what was asked for instead of
        // stacking a second panel.
        if let Some(RepoDialog::Safety(panel)) = self.dialog.as_mut() {
            match (tab, refname) {
                (crate::safety_panel::Tab::Reflog, Some(name)) => panel.show_reflog_of(&name),
                (tab, _) => panel.set_tab(tab),
            }
            return;
        }
        self.dialog = Some(RepoDialog::Safety(Box::new(SafetyPanel::open(
            link.activity_handle(),
            tab,
            refname,
            depth,
        ))));
    }

    /// The view the Undo history / Reflog / Lost work panel is open on.
    pub(crate) fn safety_tab(&self) -> Option<crate::safety_panel::Tab> {
        match &self.dialog {
            Some(RepoDialog::Safety(panel)) => Some(panel.tab),
            _ => None,
        }
    }

    /// Whether the open panel can hand the keyboard to the palette (no
    /// name being typed over it).
    pub(crate) fn safety_wants_palette(&self) -> bool {
        matches!(&self.dialog, Some(RepoDialog::Safety(panel)) if panel.takes_palette())
    }

    /// What the palette may ask of the AI writing help of the text box open
    /// in the top dialog (the new request / issue forms, the review's boxes,
    /// the conversation's composer); `None`: no such text box.
    pub(crate) fn composer_avail(&self) -> Option<crate::composer_cmd::ComposerAvail> {
        match self.dialog.as_ref()? {
            #[cfg(feature = "hosting")]
            RepoDialog::PullRequest { form } => form.composer_avail(),
            #[cfg(feature = "hosting")]
            RepoDialog::Issue { form } => form.composer_avail(),
            #[cfg(feature = "hosting")]
            RepoDialog::Review(view) => view.composer_avail(),
            #[cfg(feature = "hosting")]
            RepoDialog::Item(view) => view.composer_avail(),
            _ => None,
        }
    }

    /// Runs a palette row on that help; the sentence for the status line.
    pub(crate) fn run_composer(&mut self, cmd: crate::composer_cmd::ComposerCmd) -> Option<String> {
        #[cfg(not(feature = "hosting"))]
        let _ = cmd;
        match self.dialog.as_mut()? {
            #[cfg(feature = "hosting")]
            RepoDialog::PullRequest { form } => form.run_composer(cmd),
            #[cfg(feature = "hosting")]
            RepoDialog::Issue { form } => form.run_composer(cmd),
            #[cfg(feature = "hosting")]
            RepoDialog::Review(view) => view.run_composer(cmd),
            #[cfg(feature = "hosting")]
            RepoDialog::Item(view) => view.run_composer(cmd),
            _ => None,
        }
    }

    /// Whether the open review view can hand the keyboard to the palette (no
    /// comment box or picker open over it).
    pub(crate) fn review_wants_palette(&self) -> bool {
        #[cfg(feature = "hosting")]
        {
            matches!(&self.dialog, Some(RepoDialog::Review(view)) if view.takes_palette())
        }
        #[cfg(not(feature = "hosting"))]
        {
            false
        }
    }

    /// Whether the open conversation can hand the keyboard to the palette (no
    /// composer focused, no menu or question open over it).
    pub(crate) fn item_wants_palette(&self) -> bool {
        #[cfg(feature = "hosting")]
        {
            matches!(&self.dialog, Some(RepoDialog::Item(view)) if view.takes_palette())
        }
        #[cfg(not(feature = "hosting"))]
        {
            false
        }
    }

    /// Whether the open board can hand the keyboard to the palette (no
    /// popup or text filter open over it).
    pub(crate) fn board_wants_palette(&self) -> bool {
        #[cfg(feature = "hosting")]
        {
            matches!(&self.dialog, Some(RepoDialog::Board(view)) if view.takes_palette())
        }
        #[cfg(not(feature = "hosting"))]
        {
            false
        }
    }

    /// Runs a palette row on the open panel: what the window has to do about
    /// it, and why nothing happened when it could not.
    pub(crate) fn run_safety(
        &mut self,
        cmd: crate::safety_panel::SafetyCmd,
        state: &gitpoulpi_core::domain::JournalState,
    ) -> (Option<DialogEffect>, Option<String>) {
        let Some(RepoDialog::Safety(panel)) = self.dialog.as_mut() else {
            return (
                None,
                Some(cmd.unavailable(None).unwrap_or_default().to_string()),
            );
        };
        let (outcome, why) = panel.run_command(cmd, state);
        (self.on_safety(outcome), why)
    }

    /// The Undo history panel is waiting on the daemon: its spinner moves.
    pub(crate) fn animating(&self) -> bool {
        match &self.dialog {
            Some(RepoDialog::Safety(panel)) => panel.busy(),
            Some(RepoDialog::Flow(flow)) => flow.animating(),
            #[cfg(feature = "hosting")]
            Some(RepoDialog::IssueFlow(flow)) => flow.animating(),
            #[cfg(feature = "hosting")]
            Some(RepoDialog::Meta(edit)) => edit.busy(),
            #[cfg(feature = "hosting")]
            Some(RepoDialog::Review(view)) => view.busy(),
            #[cfg(feature = "hosting")]
            Some(RepoDialog::Board(view)) => view.busy(),
            #[cfg(feature = "hosting")]
            Some(RepoDialog::Item(view)) => view.busy(),
            #[cfg(feature = "hosting")]
            Some(RepoDialog::PullRequest { form }) => form.animating(),
            #[cfg(feature = "hosting")]
            Some(RepoDialog::Issue { form }) => form.animating(),
            _ => false,
        }
    }

    /// When the Clone dialog's highlighted source asks for its list, or the
    /// open conversation its timed refresh.
    pub(crate) fn next_deadline(&self) -> Option<std::time::Instant> {
        match &self.dialog {
            Some(RepoDialog::Clone { form, .. }) => form.next_deadline(),
            #[cfg(feature = "hosting")]
            Some(RepoDialog::Item(view)) => view.next_deadline(),
            _ => None,
        }
    }

    pub(crate) fn is_open(&self) -> bool {
        self.dialog.is_some()
    }

    /// Closes whatever is open. A running clone or init keeps running and
    /// still opens its repository when it lands.
    pub(crate) fn close(&mut self) {
        self.dialog = None;
        self.review_clone = None;
    }

    /// Opens the dialog a management entry asks for (`Show` opens none).
    pub(crate) fn open(&mut self, entry: ManagementEntry, cwd: &Path) {
        let settings = gitpoulpi_config::Settings::resolve_ambient().ok();
        match entry {
            ManagementEntry::Show
            | ManagementEntry::NewTab
            | ManagementEntry::Radar
            | ManagementEntry::ReleaseNotes => {}
            ManagementEntry::Workspaces => {
                if let Some(pending) = crate::workspace_ui::apply::take_pending() {
                    self.open_workspace(pending);
                }
            }
            ManagementEntry::Browse => {
                let configured = settings
                    .as_ref()
                    .and_then(|s| s.config().repos.browse_root_override());
                let start = browse_start_for(configured, last_folder(), cwd);
                let picker = self.new_picker(
                    msg::folder_picker::TITLE_OPEN.get(),
                    msg::folder_picker::CONFIRM_OPEN.get(),
                    start,
                );
                self.dialog = Some(RepoDialog::Browse(picker));
            }
            ManagementEntry::Init => {
                let branch = settings
                    .as_ref()
                    .map(|s| s.config().general.default_branch_name.clone())
                    .unwrap_or_default();
                let parent = last_folder().unwrap_or_else(|| cwd.to_path_buf());
                self.dialog = Some(RepoDialog::Init {
                    form: InitDialog::new(&parent, &branch),
                    picker: None,
                });
            }
            ManagementEntry::Clone => {
                self.review_clone = None;
                let root = settings
                    .as_ref()
                    .and_then(|s| s.config().repos.clone_root_or_default())
                    .or_else(|| gitpoulpi_config::default_clone_root(Some(cwd)));
                let hosted = hosted_sources(&crate::pull_request::load_integrations());
                let start = settings
                    .as_ref()
                    .map(|s| s.config().repos.clone_start_source.clone())
                    .unwrap_or_default();
                let last = crate::session_store::config_dir()
                    .map(|d| {
                        gitpoulpi_session::CloneSource::load(
                            &gitpoulpi_session::CloneSource::file_in(&d),
                        )
                    })
                    .and_then(|c| c.last_profile);
                #[cfg(feature = "hosting")]
                self.projects_rx.clear();
                self.dialog = Some(RepoDialog::Clone {
                    form: Box::new(
                        CloneDialog::new(root.as_deref())
                            .with_hosted(hosted)
                            .with_start_source(&start, last.as_deref()),
                    ),
                    picker: None,
                });
            }
        }
        self.pump();
    }

    /// Opens the Clone dialog for the Radar's "Clone it, then review": the
    /// ordinary dialog in the connected-account mode of the profile the
    /// request came from, its project selected once the list has loaded
    /// (else the dialog with `url` filled in), `note` under it, whose
    /// Submit hands `carry` to `after_clone` keyed by the destination.
    pub(crate) fn open_clone_for_review(
        &mut self,
        url: &str,
        carry: crate::after_clone::Carry,
        note: String,
        cwd: &Path,
    ) {
        let settings = gitpoulpi_config::Settings::resolve_ambient().ok();
        let root = settings
            .as_ref()
            .and_then(|s| s.config().repos.clone_root_or_default())
            .or_else(|| gitpoulpi_config::default_clone_root(Some(cwd)));
        let hosted = hosted_sources(&crate::pull_request::load_integrations());
        let mut form = CloneDialog::new(root.as_deref())
            .with_hosted(hosted)
            .with_url(url)
            .with_target(&carry.integration, &carry.project, note.clone());
        // Not a source of the dialog (or none in this build): the URL stands.
        if form.message.is_none() {
            form.message = Some(note);
        }
        #[cfg(feature = "hosting")]
        self.projects_rx.clear();
        self.dialog = Some(RepoDialog::Clone {
            form: Box::new(form),
            picker: None,
        });
        self.review_clone = Some(carry);
        self.pump();
    }

    /// Opens Create worktree for `seed`, proposing a folder beside
    /// `repo_root`.
    #[cfg(all(test, feature = "worktree"))]
    pub(crate) fn open_worktree(&mut self, seed: WorktreeSeed, repo_root: &Path) {
        self.open_worktree_with(seed, repo_root, Vec::new(), false);
    }

    /// [`Self::open_worktree`] with the branches the Ref combobox lists;
    /// `pick_ref` opens it on that list ("Choose a branch...").
    pub(crate) fn open_worktree_with(
        &mut self,
        seed: WorktreeSeed,
        repo_root: &Path,
        branches: Vec<crate::worktree_dialog::BranchChoice>,
        pick_ref: bool,
    ) {
        self.dialog = Some(RepoDialog::Worktree {
            form: Box::new(WorktreeDialog::with_choices(
                seed, repo_root, branches, pick_ref,
            )),
            picker: None,
        });
    }

    /// Opens Remove worktree (or the merge picker) for `seed`.
    #[cfg(feature = "worktree")]
    pub(crate) fn open_worktree_remove(
        &mut self,
        seed: crate::worktree_remove_dialog::Seed,
        requester: &crate::daemon_link::Requester,
    ) {
        self.dialog = Some(RepoDialog::WorktreeRemove(Box::new(
            crate::worktree_remove_dialog::WorktreeRemoveDialog::new(seed, requester),
        )));
    }

    /// Opens Start a pull request's form for `seed` - reached only when
    /// a token is available; the ref menu handles the no-token case
    /// itself and never calls this. `tip_subject` prefills Title.
    #[cfg(feature = "hosting")]
    pub(crate) fn open_pull_request(
        &mut self,
        seed: PullRequestSeed,
        tip_subject: &str,
        ai_enabled: bool,
    ) {
        let mut form = PullRequestDialog::new(seed, tip_subject);
        form.set_ai_generate(ai_enabled);
        if let Some(scope) = self.ai_scope.clone() {
            form.bind_ai(scope);
        }
        self.dialog = Some(RepoDialog::PullRequest {
            form: Box::new(form),
        });
    }

    /// The AI answer for the open pull request form (if it is still open).
    pub(crate) fn pull_request_generated(&mut self, result: Result<(String, String), String>) {
        #[cfg(not(feature = "hosting"))]
        let _ = result;
        #[cfg(feature = "hosting")]
        if let Some(RepoDialog::PullRequest { form }) = self.dialog.as_mut() {
            match result {
                Ok((title, body)) => form.set_generated(title, body),
                Err(why) => form.generation_failed(format!("AI: {why}")),
            }
        }
    }

    /// Opens the New issue form for `seed`, starting in the board column
    /// `at` names.
    #[cfg(feature = "hosting")]
    fn open_issue_in(&mut self, seed: IssueSeed, at: crate::board_view::NewIn) {
        self.open_issue_form(seed);
        if let Some(RepoDialog::Issue { form }) = self.dialog.as_mut() {
            form.start_in(at.column, at.label);
        }
    }

    /// Opens the New issue form for `seed`.
    #[cfg(feature = "hosting")]
    pub(crate) fn open_issue(&mut self, seed: IssueSeed) {
        // A form of the sidebar's does not lead back to a board.
        self.parked_board = None;
        self.open_issue_form(seed);
    }

    #[cfg(feature = "hosting")]
    fn open_issue_form(&mut self, seed: IssueSeed) {
        let mut form = Box::new(IssueDialog::new(seed));
        if let Some(scope) = self.ai_scope.clone() {
            form.bind_ai(scope);
        }
        // Jira / Azure issue types, Trello lists: asked when the form opens.
        form.fetch_options(
            self.issue_factory.clone(),
            crate::pull_request::load_integrations(),
        );
        self.dialog = Some(RepoDialog::Issue { form });
    }

    /// Opens the picker for one field of an existing request or issue.
    #[cfg(feature = "hosting")]
    pub(crate) fn open_meta(&mut self, target: MetaTarget, field: MetaField) {
        self.dialog = Some(RepoDialog::Meta(Box::new(MetaEdit::new(
            self.issue_factory.clone(),
            target,
            field,
        ))));
    }

    /// Opens the in-app review of a request. `requester` is the window's
    /// path to the daemon (the fetch of the request's refs, its diff).
    #[cfg(feature = "hosting")]
    #[cfg_attr(not(feature = "review"), allow(unreachable_code))]
    pub(crate) fn open_review(
        &mut self,
        target: ReviewTarget,
        requester: crate::daemon_link::Requester,
        ai_enabled: bool,
        session: &str,
    ) {
        self.dialog = Some(RepoDialog::Review(Box::new(ReviewView::open(
            target,
            self.issue_factory.clone(),
            requester,
            ai_enabled,
            session.to_string(),
        ))));
    }

    /// What the palette needs to gray a review command out (`None`: the
    /// review is not open).
    pub(crate) fn review_avail(&self) -> Option<crate::review_cmd::ReviewAvail> {
        #[cfg(feature = "hosting")]
        if let Some(RepoDialog::Review(view)) = &self.dialog {
            return Some(view.avail());
        }
        None
    }

    /// Opens the conversation of an issue or a request.
    #[cfg(feature = "hosting")]
    #[cfg_attr(not(feature = "conversation"), allow(unreachable_code))]
    pub(crate) fn open_item(&mut self, target: ItemTarget) {
        self.dialog = Some(RepoDialog::Item(Box::new(ItemView::open(
            target,
            self.issue_factory.clone(),
        ))));
    }

    /// Another client posted, edited or deleted a comment: the open
    /// conversation of that item reads itself again.
    pub(crate) fn on_hosting_changed(&mut self, change: &gitpoulpi_ipc::HostingChangeArgs) {
        #[cfg(feature = "hosting")]
        match self.dialog.as_mut() {
            Some(RepoDialog::Item(view)) => view.on_hosting_changed(change),
            Some(RepoDialog::Review(view)) => view.on_hosting_changed(change),
            _ => {}
        }
        #[cfg(not(feature = "hosting"))]
        let _ = change;
    }

    /// What the palette needs to gray a conversation command out (`None`: no
    /// conversation is open).
    pub(crate) fn item_avail(&self) -> Option<crate::item_cmd::ItemAvail> {
        #[cfg(feature = "hosting")]
        match &self.dialog {
            Some(RepoDialog::Item(view)) => return Some(view.avail()),
            // The review's Overview is the same conversation.
            Some(RepoDialog::Review(view)) => return view.item_avail(),
            _ => {}
        }
        None
    }

    /// Runs a palette command on the open conversation.
    #[cfg(feature = "hosting")]
    pub(crate) fn run_item(&mut self, cmd: ItemCmd) -> Option<DialogEffect> {
        let effect = match self.dialog.as_mut() {
            Some(RepoDialog::Item(view)) => view.run(cmd),
            Some(RepoDialog::Review(view)) => {
                let effect = view.run_item(cmd);
                return self.on_review(effect);
            }
            _ => return None,
        };
        self.on_item(effect)
    }

    /// What the conversation asked of the window.
    #[cfg(feature = "hosting")]
    fn on_item(&mut self, effect: ItemEffect) -> Option<DialogEffect> {
        match effect {
            ItemEffect::None => None,
            ItemEffect::Message(message) => Some(DialogEffect::Message(message)),
            ItemEffect::Close => {
                self.dialog = None;
                self.resume_board();
                None
            }
            ItemEffect::OpenUrl(url) => Some(DialogEffect::Message(
                match crate::repo_menu::open_url(&url) {
                    Ok(()) => msg::hosting_ui::opened_in_browser(&url),
                    Err(err) => crate::activity_log::report_problem_clear(
                        crate::activity_log::ProblemSource::Tool,
                        &err,
                    ),
                },
            )),
            ItemEffect::OpenDiagram(source) => {
                crate::mermaid_popup::open_script(&source);
                None
            }
            // Only the review's Overview opens code.
            ItemEffect::OpenCode(_) => Some(DialogEffect::Message(
                msg::item_view::WHY_NO_CODE_HERE.get().to_string(),
            )),
            ItemEffect::ShowCommit(sha) => {
                self.dialog = None;
                self.parked_board = None;
                Some(DialogEffect::ShowCommitPrefix(sha))
            }
        }
    }

    /// Opens the Kanban board of the ISSUES source.
    #[cfg(feature = "hosting")]
    #[cfg_attr(not(feature = "board"), allow(unreachable_code))]
    pub(crate) fn open_board(&mut self, target: BoardTarget) {
        self.dialog = Some(RepoDialog::Board(Box::new(BoardView::open(
            target,
            self.issue_factory.clone(),
        ))));
    }

    /// What the palette needs to gray a board command out (`None`: the
    /// board is not open).
    pub(crate) fn board_avail(&self) -> Option<crate::board_cmd::BoardAvail> {
        #[cfg(feature = "hosting")]
        if let Some(RepoDialog::Board(view)) = &self.dialog {
            return Some(view.avail());
        }
        None
    }

    /// Runs a palette command on the open board.
    #[cfg(feature = "hosting")]
    pub(crate) fn run_board(&mut self, cmd: BoardCmd) -> Option<DialogEffect> {
        let effect = match self.dialog.as_mut() {
            Some(RepoDialog::Board(view)) => view.run(cmd),
            _ => return None,
        };
        self.on_board(effect)
    }

    /// What the board asked of the window.
    #[cfg(feature = "hosting")]
    fn on_board(&mut self, effect: BoardEffect) -> Option<DialogEffect> {
        match effect {
            BoardEffect::None => None,
            BoardEffect::Message(message) => Some(DialogEffect::Message(message)),
            BoardEffect::Close => {
                self.dialog = None;
                None
            }
            BoardEffect::OpenUrl(url) => Some(DialogEffect::Message(
                match crate::repo_menu::open_url(&url) {
                    Ok(()) => msg::hosting_ui::opened_in_browser(&url),
                    Err(err) => crate::activity_log::report_problem_clear(
                        crate::activity_log::ProblemSource::Tool,
                        &err,
                    ),
                },
            )),
            BoardEffect::Item(target) => {
                // The board waits behind the conversation and comes back with it.
                if let Some(RepoDialog::Board(view)) = self.dialog.take() {
                    self.parked_board = Some(view);
                }
                self.open_item(*target);
                None
            }
            BoardEffect::NewIssue(seed, at) => {
                // The board waits behind the form and comes back with it.
                if let Some(RepoDialog::Board(view)) = self.dialog.take() {
                    self.parked_board = Some(view);
                }
                self.open_issue_in(*seed, at);
                None
            }
        }
    }

    /// Brings the board back after the New issue form closed.
    #[cfg(feature = "hosting")]
    fn resume_board(&mut self) {
        if let Some(mut view) = self.parked_board.take() {
            view.refresh();
            self.dialog = Some(RepoDialog::Board(view));
        }
    }

    /// Runs a palette command on the open review view.
    #[cfg(feature = "hosting")]
    pub(crate) fn run_review(&mut self, cmd: ReviewCmd) -> Option<DialogEffect> {
        let effect = match self.dialog.as_mut() {
            Some(RepoDialog::Review(view)) => view.run(cmd),
            _ => return None,
        };
        self.on_review(effect)
    }

    /// What the review view asked of the window.
    #[cfg(feature = "hosting")]
    fn on_review(&mut self, effect: ReviewEffect) -> Option<DialogEffect> {
        match effect {
            ReviewEffect::None => None,
            ReviewEffect::Message(message) => Some(DialogEffect::Message(message)),
            ReviewEffect::Close => {
                self.forget_review();
                self.dialog = None;
                None
            }
            ReviewEffect::CloseWith(message) => {
                self.forget_review();
                self.dialog = None;
                Some(DialogEffect::Message(message))
            }
            ReviewEffect::Checkout(number) => {
                self.forget_review();
                self.dialog = None;
                Some(DialogEffect::CheckoutPullRequest(number))
            }
        }
    }

    /// The review was closed or finished: nothing to reopen with the tab.
    #[cfg(feature = "hosting")]
    fn forget_review(&self) {
        if let Some(RepoDialog::Review(view)) = &self.dialog {
            view.forget_resume();
        }
    }

    /// The window is ending: a review still open is written down, so coming
    /// back to the tab reopens it (`review_view::resume`).
    pub(crate) fn save_review_mark(&self) {
        #[cfg(feature = "hosting")]
        if let Some(RepoDialog::Review(view)) = &self.dialog {
            view.save_resume_mark();
        }
    }

    /// Puts a reopened review back where the marker says, once it loaded.
    #[cfg(all(feature = "hosting", feature = "review"))]
    pub(crate) fn restore_review_position(&mut self, pos: gitpoulpi_session::ResumePos) {
        if let Some(RepoDialog::Review(view)) = &mut self.dialog {
            view.restore_position(pos);
        }
    }

    /// Whether any dialog is up (a reopened review does not take the screen
    /// from one the reviewer opened first).
    pub(crate) fn any_open(&self) -> bool {
        self.dialog.is_some()
    }

    /// What a closed or finished metadata picker leaves behind.
    #[cfg(feature = "hosting")]
    fn on_meta(&mut self, outcome: MetaEditOutcome) -> Option<DialogEffect> {
        match outcome {
            MetaEditOutcome::Stay => None,
            MetaEditOutcome::Cancel => {
                self.dialog = None;
                None
            }
            MetaEditOutcome::Saved(message) => {
                self.dialog = None;
                // The lists and the Radar's cache re-fetch on the next poll.
                crate::hosting_ui::request_issues_refresh();
                Some(DialogEffect::Message(message))
            }
        }
    }

    /// Whether the Create worktree dialog is the open dialog.
    pub(crate) fn worktree_open(&self) -> bool {
        matches!(self.dialog, Some(RepoDialog::Worktree { .. }))
    }

    /// AI went on or off, or the repository changed: the Markdown composers
    /// opened from now on get (or lose) their "Write with AI" help (called
    /// every tick).
    pub(crate) fn sync_composer_ai(&mut self, enabled: bool, root: &std::path::Path) {
        if !enabled {
            self.ai_scope = None;
        } else if self
            .ai_scope
            .as_ref()
            .is_none_or(|s| s.root.as_deref() != Some(root))
        {
            self.ai_scope = Some(crate::md_editor::ai::AiScope {
                session: self.session.clone(),
                root: Some(root.to_path_buf()),
            });
        }
    }

    /// AI went on or off, or a request ended: the Create worktree dialog's
    /// ✦ button follows (called every tick).
    pub(crate) fn sync_worktree_ai(&mut self, enabled: bool, running: bool) {
        if let Some(RepoDialog::Worktree { form, .. }) = self.dialog.as_mut() {
            form.sync_ai(enabled, running);
        }
    }

    /// The AI's names for the open Create worktree dialog: `(slug, branch)`.
    pub(crate) fn worktree_named(&mut self, result: Result<(String, String), String>) {
        if let Some(RepoDialog::Worktree { form, .. }) = self.dialog.as_mut() {
            form.ai_done(result);
        }
    }

    /// Whether the pull request form is the open dialog.
    pub(crate) fn pull_request_open(&self) -> bool {
        #[cfg(feature = "hosting")]
        {
            matches!(self.dialog, Some(RepoDialog::PullRequest { .. }))
        }
        #[cfg(not(feature = "hosting"))]
        {
            false
        }
    }

    /// Opens "Create patch...": a folder picker (confirm "Save here")
    /// starting beside `repo_root`, proposing `seed.name` as the base of
    /// the patch's filename.
    pub(crate) fn open_save_patch(&mut self, seed: PatchSeed, repo_root: &Path) {
        let start = last_folder().unwrap_or_else(|| repo_root.to_path_buf());
        let picker = self.new_picker(
            msg::folder_picker::TITLE_PATCH.get(),
            msg::folder_picker::CONFIRM_PATCH.get(),
            start,
        );
        self.dialog = Some(RepoDialog::SavePatch {
            source: seed.source,
            name: seed.name,
            picker,
        });
        self.pump();
    }

    /// Opens "Apply patch...": a file picker starting beside `repo_root`.
    pub(crate) fn open_apply_patch(&mut self, repo_root: &Path) {
        let start = last_folder().unwrap_or_else(|| repo_root.to_path_buf());
        let mut picker = self.new_picker(
            msg::folder_picker::TITLE_APPLY.get(),
            msg::folder_picker::CONFIRM_APPLY.get(),
            start,
        );
        picker.set_file_mode();
        self.dialog = Some(RepoDialog::ApplyPatch(picker));
        self.pump();
    }

    /// A picker, with its quick places being worked out in the background.
    fn new_picker(&self, title: &str, confirm: &str, start: PathBuf) -> FolderPicker {
        let mut picker = FolderPicker::new(title, confirm, start);
        let settings = gitpoulpi_config::Settings::resolve_ambient().ok();
        picker.set_show_hidden(
            settings
                .as_ref()
                .is_some_and(|s| s.config().repos.show_hidden),
        );
        picker.set_show_siblings(
            settings
                .as_ref()
                .is_some_and(|s| s.config().repos.show_siblings),
        );
        let mut extra: Vec<(String, PathBuf)> = Vec::new();
        if let Some(root) = gitpoulpi_config::Settings::resolve_ambient()
            .ok()
            .and_then(|s| s.config().repos.clone_root_or_default())
        {
            extra.push((msg::folder_picker::CLONE_FOLDER.get().into(), root));
        }
        // Favourite repositories' parents first, then recent ones'.
        let mut known = session_store::known_repos(&self.session);
        known.sort_by_key(|k| (k.favorite.is_none(), k.favorite, k.recent));
        for repo in known {
            if let Some(parent) = dir_picker::parent_of(&repo.path) {
                if extra.iter().any(|(_, p)| *p == parent) {
                    continue;
                }
                let label = parent
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| parent.display().to_string());
                extra.push((label, parent));
            }
            if extra.len() >= 7 {
                break;
            }
        }
        self.lister
            .request_places(gitpoulpi_config::home_dir(), extra);
        picker
    }

    /// The picker that has the keyboard, if one is up.
    fn picker_mut(&mut self) -> Option<&mut FolderPicker> {
        match self.dialog.as_mut()? {
            RepoDialog::Browse(picker)
            | RepoDialog::ApplyPatch(picker)
            | RepoDialog::SavePatch { picker, .. } => Some(picker),
            RepoDialog::Init { picker, .. }
            | RepoDialog::Clone { picker, .. }
            | RepoDialog::Workspace { picker, .. }
            | RepoDialog::Worktree { picker, .. } => picker.as_mut(),
            RepoDialog::Locate { picker, .. } => Some(picker),
            #[cfg(feature = "hosting")]
            RepoDialog::PullRequest { .. }
            | RepoDialog::Issue { .. }
            | RepoDialog::Meta(_)
            | RepoDialog::Board(_)
            | RepoDialog::Item(_)
            | RepoDialog::Review(_) => None,
            #[cfg(feature = "hosting")]
            RepoDialog::IssueFlow(_) => None,
            RepoDialog::Sparse(_) | RepoDialog::Safety(_) | RepoDialog::Flow(_) => None,
            #[cfg(feature = "worktree")]
            RepoDialog::WorktreeRemove(_) => None,
        }
    }

    /// The picker on top, shared (it needs no keyboard to be read).
    fn picker_ref(&self) -> Option<&FolderPicker> {
        match self.dialog.as_ref()? {
            RepoDialog::Browse(picker)
            | RepoDialog::ApplyPatch(picker)
            | RepoDialog::SavePatch { picker, .. }
            | RepoDialog::Locate { picker, .. } => Some(picker),
            RepoDialog::Init { picker, .. }
            | RepoDialog::Clone { picker, .. }
            | RepoDialog::Workspace { picker, .. }
            | RepoDialog::Worktree { picker, .. } => picker.as_ref(),
            _ => None,
        }
    }

    /// What `Ctrl+N` badges in the dialog on top: its picker when one is up
    /// over it, else its own controls. Empty for a dialog that offers none.
    pub(crate) fn nav_targets(&mut self) -> Vec<(Rect, (u16, u16))> {
        if let Some(picker) = self.picker_ref() {
            return picker.nav_targets();
        }
        match self.dialog.as_mut() {
            Some(RepoDialog::Init { form, .. }) => form.nav_targets(),
            Some(RepoDialog::Clone { form, .. }) => form.nav_targets(),
            Some(RepoDialog::Worktree { form, .. }) => form.nav_targets(),
            #[cfg(feature = "worktree")]
            Some(RepoDialog::WorktreeRemove(form)) => form.nav_targets(),
            Some(RepoDialog::Workspace { form, .. }) => form.nav_targets(),
            Some(RepoDialog::Sparse(form)) => form.nav_targets(),
            Some(RepoDialog::Safety(panel)) => panel.nav_targets(),
            Some(RepoDialog::Flow(form)) => form.nav_targets(),
            #[cfg(feature = "hosting")]
            Some(RepoDialog::IssueFlow(form)) => form.nav_targets(),
            #[cfg(feature = "hosting")]
            Some(RepoDialog::PullRequest { form }) => form.nav_targets(),
            #[cfg(feature = "hosting")]
            Some(RepoDialog::Issue { form }) => form.nav_targets(),
            #[cfg(feature = "hosting")]
            Some(RepoDialog::Item(view)) => view.nav_targets(),
            _ => Vec::new(),
        }
    }

    /// Whether a click at the point does something: the pointing hand. What
    /// the paint registered answers first (a text field is the arrow; a
    /// button, a checkbox, a combo row, a cross is a hand), then the dialog's
    /// own rows, then its keyboard-badged controls.
    pub(crate) fn pointer_hand_at(&mut self, col: u16, row: u16) -> bool {
        use crate::pointer_map::{self, Kind};
        match pointer_map::at(col, row) {
            Some(Kind::Field | Kind::Plain) => return false,
            Some(k) if k.resize_shape().is_some() => return false,
            Some(Kind::Hand) => return true,
            Some(_) | None => {}
        }
        if self.picker_ref().is_none() {
            if let Some(RepoDialog::Workspace { form, .. }) = self.dialog.as_ref() {
                if form.pointer_hand_at(col, row) {
                    return true;
                }
            }
        }
        let at = ratatui::layout::Position::new(col, row);
        self.nav_targets().iter().any(|(r, _)| r.contains(at))
    }

    /// A pressed badge: a left click at its cell, answered like the mouse's.
    pub(crate) fn nav_press(&mut self, (column, row): (u16, u16)) -> Option<DialogEffect> {
        let at = |kind| MouseEvent {
            kind,
            column,
            row,
            modifiers: KeyModifiers::NONE,
        };
        let effect = self.on_mouse(at(MouseEventKind::Down(MouseButton::Left)));
        if self.is_open() {
            self.on_mouse(at(MouseEventKind::Up(MouseButton::Left)));
        }
        effect
    }

    /// Sends every folder read a picker asked for to the lister.
    fn pump(&mut self) {
        let wants = self.picker_mut().map(|p| p.wants()).unwrap_or_default();
        for dir in wants {
            self.lister.request(dir);
        }
    }

    // -- input ------------------------------------------------------------

    pub(crate) fn on_key(&mut self, key: KeyEvent) -> Option<DialogEffect> {
        let effect = if let Some(picker) = self.picker_mut() {
            let outcome = picker.on_key(key);
            self.on_picker(outcome)
        } else {
            match self.dialog.as_mut() {
                Some(RepoDialog::Init { form, .. }) => {
                    let outcome = form.on_key(key);
                    self.on_init(outcome)
                }
                Some(RepoDialog::Clone { form, .. }) => {
                    let outcome = form.on_key(key);
                    self.on_clone(outcome)
                }
                Some(RepoDialog::Worktree { form, .. }) => {
                    let outcome = form.on_key(key);
                    self.on_worktree(outcome)
                }
                #[cfg(feature = "worktree")]
                Some(RepoDialog::WorktreeRemove(form)) => {
                    let outcome = form.on_key(key);
                    self.on_worktree_remove(outcome)
                }
                Some(RepoDialog::Workspace { form, .. }) => {
                    let outcome = form.on_key(key);
                    self.on_workspace(outcome)
                }
                Some(RepoDialog::Sparse(form)) => {
                    let outcome = form.on_key(key);
                    self.on_sparse(outcome)
                }
                Some(RepoDialog::Flow(form)) => {
                    let outcome = form.on_key(key);
                    self.on_flow(outcome)
                }
                #[cfg(feature = "hosting")]
                Some(RepoDialog::IssueFlow(form)) => {
                    let outcome = form.on_key(key);
                    self.on_issue_flow(outcome)
                }
                Some(RepoDialog::Safety(panel)) => {
                    let outcome = panel.on_key(key);
                    self.on_safety(outcome)
                }
                #[cfg(feature = "hosting")]
                Some(RepoDialog::PullRequest { form }) => {
                    let outcome = form.on_key(key);
                    self.on_pull_request(outcome)
                }
                #[cfg(feature = "hosting")]
                Some(RepoDialog::Issue { form }) => {
                    let outcome = form.on_key(key);
                    self.on_issue(outcome)
                }
                #[cfg(feature = "hosting")]
                Some(RepoDialog::Meta(edit)) => {
                    let outcome = edit.on_key(key);
                    self.on_meta(outcome)
                }
                #[cfg(feature = "hosting")]
                Some(RepoDialog::Review(view)) => {
                    let effect = view.on_key(key);
                    self.on_review(effect)
                }
                #[cfg(feature = "hosting")]
                Some(RepoDialog::Board(view)) => {
                    let effect = view.on_key(key);
                    self.on_board(effect)
                }
                #[cfg(feature = "hosting")]
                Some(RepoDialog::Item(view)) => {
                    let effect = view.on_key(key);
                    self.on_item(effect)
                }
                _ => None,
            }
        };
        self.pump();
        effect
    }

    pub(crate) fn on_paste(&mut self, text: &str) {
        if let Some(picker) = self.picker_mut() {
            picker.paste(text);
        } else {
            match self.dialog.as_mut() {
                Some(RepoDialog::Init { form, .. }) => form.paste(text),
                Some(RepoDialog::Clone { form, .. }) => form.paste(text),
                Some(RepoDialog::Worktree { form, .. }) => form.paste(text),
                #[cfg(feature = "worktree")]
                Some(RepoDialog::WorktreeRemove(form)) => form.paste(text),
                Some(RepoDialog::Workspace { form, .. }) => form.paste(text),
                Some(RepoDialog::Sparse(form)) => form.paste(text),
                Some(RepoDialog::Flow(form)) => form.paste(text),
                #[cfg(feature = "hosting")]
                Some(RepoDialog::IssueFlow(form)) => form.paste(text),
                Some(RepoDialog::Safety(panel)) => panel.on_paste(text),
                #[cfg(feature = "hosting")]
                Some(RepoDialog::PullRequest { form }) => form.paste(text),
                #[cfg(feature = "hosting")]
                Some(RepoDialog::Issue { form }) => form.paste(text),
                #[cfg(feature = "hosting")]
                Some(RepoDialog::Meta(edit)) => edit.paste(text),
                #[cfg(feature = "hosting")]
                Some(RepoDialog::Review(view)) => view.paste(text),
                #[cfg(feature = "hosting")]
                Some(RepoDialog::Board(view)) => view.paste(text),
                #[cfg(feature = "hosting")]
                Some(RepoDialog::Item(view)) => view.paste(text),
                _ => {}
            }
        }
        self.pump();
    }

    /// The closable dialog (or picker) on top and its frame.
    fn closable(&mut self) -> Option<(&mut dyn dialog_kit::Dismiss, Rect)> {
        let area = self.area;
        Some(match self.dialog.as_mut()? {
            RepoDialog::Browse(picker)
            | RepoDialog::ApplyPatch(picker)
            | RepoDialog::SavePatch { picker, .. }
            | RepoDialog::Locate { picker, .. } => (picker, picker_rect(area)),
            RepoDialog::Init { form, picker } => match picker {
                Some(p) => (p, picker_rect(area)),
                None => (form, init_rect(area)),
            },
            RepoDialog::Clone { form, picker } => match picker {
                Some(p) => (p, picker_rect(area)),
                None => {
                    let rect = form.rect(area);
                    (form.as_mut(), rect)
                }
            },
            RepoDialog::Worktree { form, picker } => match picker {
                Some(p) => (p, picker_rect(area)),
                None => {
                    let rect = form.rect(area);
                    (form.as_mut(), rect)
                }
            },
            #[cfg(feature = "worktree")]
            RepoDialog::WorktreeRemove(form) => {
                (form.as_mut(), crate::worktree_remove_dialog::rect(area))
            }
            RepoDialog::Workspace { form, picker } => match picker {
                Some(p) => (p, picker_rect(area)),
                None => (form.as_mut(), workspace_rect(area)),
            },
            RepoDialog::Sparse(form) => (form, sparse_rect(area)),
            RepoDialog::Flow(form) => (form.as_mut(), crate::flow::flow_rect(area)),
            _ => return None,
        })
    }

    /// The close cross and the click outside a dialog: both ask the one
    /// `dialog_kit::dismiss` rule. `Some` when the event was consumed.
    fn close_mouse(&mut self, mouse: MouseEvent) -> Option<Option<DialogEffect>> {
        use dialog_kit::{dismiss, DismissAction, DismissKind};
        let (col, row) = (mouse.column, mouse.row);
        let on_picker = self.picker_mut().is_some();
        let (dialog, frame) = self.closable()?;
        match mouse.kind {
            MouseEventKind::Moved | MouseEventKind::Drag(_) => {
                dialog.close_hit_mut().set_hover(col, row);
                None
            }
            MouseEventKind::Down(MouseButton::Left) => {
                let kind = if dialog.close_hit().hit(col, row) {
                    DismissKind::Cross
                } else if !frame.contains((col, row).into()) {
                    DismissKind::Outside
                } else {
                    return None;
                };
                match dismiss(&*dialog, kind) {
                    DismissAction::Swallow => {
                        dialog.close_hit_mut().request_flash();
                        Some(None)
                    }
                    DismissAction::Close | DismissAction::Confirm => {
                        // A picker closes whole; a form takes the Esc path,
                        // which closes the innermost thing first.
                        Some(if on_picker {
                            let effect = self.on_picker(PickerOutcome::Cancel);
                            self.pump();
                            effect
                        } else {
                            self.on_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))
                        })
                    }
                }
            }
            _ => None,
        }
    }

    /// Every mouse event while a dialog is open lands here: the dialog is
    /// modal, so nothing under it sees a click, a wheel notch or a hover.
    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent) -> Option<DialogEffect> {
        if let Some(effect) = self.close_mouse(mouse) {
            return effect;
        }
        let mut double = false;
        if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
            let now = Instant::now();
            double = self.last_click.is_some_and(|(t, c, r)| {
                now.duration_since(t) < DOUBLE_CLICK_WINDOW && c == mouse.column && r == mouse.row
            });
            // A double click is consumed: a third click starts over.
            self.last_click = (!double).then_some((now, mouse.column, mouse.row));
        }
        let effect = if let Some(picker) = self.picker_mut() {
            let outcome = picker.on_mouse(mouse, double);
            self.on_picker(outcome)
        } else {
            match self.dialog.as_mut() {
                Some(RepoDialog::Init { form, .. }) => {
                    let outcome = form.on_mouse(mouse);
                    self.on_init(outcome)
                }
                Some(RepoDialog::Clone { form, .. }) => {
                    let outcome = form.on_mouse(mouse);
                    self.on_clone(outcome)
                }
                Some(RepoDialog::Worktree { form, .. }) => {
                    let outcome = form.on_mouse(mouse);
                    self.on_worktree(outcome)
                }
                #[cfg(feature = "worktree")]
                Some(RepoDialog::WorktreeRemove(form)) => {
                    let outcome = form.on_mouse(mouse);
                    self.on_worktree_remove(outcome)
                }
                Some(RepoDialog::Workspace { form, .. }) => {
                    let outcome = form.on_mouse(mouse);
                    self.on_workspace(outcome)
                }
                Some(RepoDialog::Sparse(form)) => {
                    let outcome = form.on_mouse(mouse);
                    self.on_sparse(outcome)
                }
                Some(RepoDialog::Flow(form)) => {
                    let outcome = form.on_mouse(mouse);
                    self.on_flow(outcome)
                }
                #[cfg(feature = "hosting")]
                Some(RepoDialog::IssueFlow(form)) => {
                    let outcome = form.on_mouse(mouse);
                    self.on_issue_flow(outcome)
                }
                Some(RepoDialog::Safety(panel)) => {
                    let outcome = panel.on_mouse(mouse);
                    self.on_safety(outcome)
                }
                #[cfg(feature = "hosting")]
                Some(RepoDialog::PullRequest { form }) => {
                    let outcome = form.on_mouse(mouse);
                    self.on_pull_request(outcome)
                }
                #[cfg(feature = "hosting")]
                Some(RepoDialog::Issue { form }) => {
                    let outcome = form.on_mouse(mouse);
                    self.on_issue(outcome)
                }
                #[cfg(feature = "hosting")]
                Some(RepoDialog::Meta(edit)) => {
                    let outcome = edit.on_mouse(mouse);
                    self.on_meta(outcome)
                }
                #[cfg(feature = "hosting")]
                Some(RepoDialog::Review(view)) => {
                    let effect = view.on_mouse(mouse);
                    self.on_review(effect)
                }
                #[cfg(feature = "hosting")]
                Some(RepoDialog::Board(view)) => {
                    let effect = view.on_mouse(mouse);
                    self.on_board(effect)
                }
                #[cfg(feature = "hosting")]
                Some(RepoDialog::Item(view)) => {
                    let effect = view.on_mouse(mouse);
                    self.on_item(effect)
                }
                _ => None,
            }
        };
        self.pump();
        effect
    }

    fn on_picker(&mut self, outcome: PickerOutcome) -> Option<DialogEffect> {
        match outcome {
            PickerOutcome::Stay => None,
            PickerOutcome::Cancel => {
                match self.dialog.as_mut() {
                    Some(
                        RepoDialog::Init { picker, .. }
                        | RepoDialog::Clone { picker, .. }
                        | RepoDialog::Workspace { picker, .. }
                        | RepoDialog::Worktree { picker, .. },
                    ) => *picker = None,
                    _ => self.dialog = None,
                }
                None
            }
            PickerOutcome::Choose(dir) => {
                match self.dialog.as_mut() {
                    Some(RepoDialog::Browse(picker)) => {
                        // The repository the folder is in: its root, or
                        // any folder inside it.
                        if find_repo_root(&dir).is_none() {
                            picker.message = Some(msg::repo_dialogs::not_a_repo(dir.display()));
                            return None;
                        }
                        remember_folder(dir.parent().unwrap_or(&dir));
                        self.dialog = None;
                        return Some(DialogEffect::Open(dir));
                    }
                    Some(RepoDialog::Init { form, picker }) => {
                        form.set_parent(&dir);
                        *picker = None;
                    }
                    Some(RepoDialog::Clone { form, picker }) => {
                        form.set_where(&dir);
                        *picker = None;
                    }
                    Some(RepoDialog::Worktree { form, picker }) => {
                        form.set_parent(&dir);
                        *picker = None;
                    }
                    // Start a pull request has no folder field and never
                    // opens a picker; reachable only if that changes.
                    #[cfg(feature = "hosting")]
                    Some(
                        RepoDialog::PullRequest { .. }
                        | RepoDialog::Issue { .. }
                        | RepoDialog::Meta(_)
                        | RepoDialog::Board(_)
                        | RepoDialog::Item(_)
                        | RepoDialog::Review(_),
                    ) => {}
                    Some(RepoDialog::SavePatch { source, name, .. }) => {
                        let dest = first_free_patch_name(&dir, name);
                        let source = source.clone();
                        self.dialog = None;
                        remember_folder(&dir);
                        return Some(DialogEffect::CreatePatch { source, dest });
                    }
                    Some(RepoDialog::Workspace { form, picker }) => {
                        // Any folder inside a repository adds that repository.
                        match find_repo_root(&dir) {
                            Some(root) => {
                                form.add_repo(&root);
                                *picker = None;
                            }
                            None => {
                                if let Some(p) = picker.as_mut() {
                                    p.message = Some(msg::repo_dialogs::not_a_repo(dir.display()));
                                }
                                return None;
                            }
                        }
                    }
                    Some(RepoDialog::Locate { id, old, picker }) => {
                        let Some(root) = find_repo_root(&dir) else {
                            picker.message = Some(msg::repo_dialogs::not_a_repo(dir.display()));
                            return None;
                        };
                        let (id, old) = (id.clone(), old.clone());
                        remember_folder(root.parent().unwrap_or(&root));
                        self.dialog = None;
                        return Some(DialogEffect::Message(
                            crate::workspace_ui::apply::locate_now(&id, &old, &root),
                        ));
                    }
                    // No folder field, no picker.
                    #[cfg(feature = "hosting")]
                    Some(RepoDialog::IssueFlow(_)) => {}
                    Some(RepoDialog::Sparse(_) | RepoDialog::Safety(_) | RepoDialog::Flow(_)) => {}
                    #[cfg(feature = "worktree")]
                    Some(RepoDialog::WorktreeRemove(_)) => {}
                    Some(RepoDialog::ApplyPatch(_)) => {
                        remember_folder(dir.parent().unwrap_or(&dir));
                        self.dialog = None;
                        return Some(DialogEffect::ApplyPatch(dir));
                    }
                    None => {}
                }
                remember_folder(&dir);
                None
            }
            PickerOutcome::CreateFolder(path) => {
                let result = std::fs::create_dir(&path);
                if let Some(picker) = self.picker_mut() {
                    match result {
                        Ok(()) => {
                            let name = path
                                .file_name()
                                .map(|n| n.to_string_lossy().into_owned())
                                .unwrap_or_default();
                            picker.folder_created(&name);
                        }
                        Err(err) => picker.folder_create_failed(&err.to_string()),
                    }
                }
                None
            }
            PickerOutcome::ShowSiblingsChanged(show) => {
                if let Some(paths) = gitpoulpi_config::ConfigPaths::discover() {
                    let _ = gitpoulpi_config::set_in_file(
                        &paths.config_file(),
                        "repos.show_siblings",
                        &show.to_string(),
                    );
                }
                None
            }
            PickerOutcome::ShowHiddenChanged(show) => {
                if let Some(paths) = gitpoulpi_config::ConfigPaths::discover() {
                    let _ = gitpoulpi_config::set_in_file(
                        &paths.config_file(),
                        "repos.show_hidden",
                        &show.to_string(),
                    );
                }
                None
            }
        }
    }

    /// Where a Browse button's picker starts: the field's folder when it
    /// names one, else the last folder chosen, else the working directory.
    fn browse_start(field: &Path, cwd: &Path) -> PathBuf {
        if field.is_absolute() {
            field.to_path_buf()
        } else {
            last_folder().unwrap_or_else(|| cwd.to_path_buf())
        }
    }

    fn on_init(&mut self, outcome: InitOutcome) -> Option<DialogEffect> {
        match outcome {
            InitOutcome::Stay => None,
            InitOutcome::Cancel => {
                self.dialog = None;
                self.init_rx
                    .is_some()
                    .then(|| DialogEffect::Message(msg::repo_dialogs::STILL_CREATING.get().into()))
            }
            InitOutcome::Browse(field) => {
                let cwd = std::env::current_dir().unwrap_or_default();
                let start = Self::browse_start(&field, &cwd);
                let new = self.new_picker(
                    msg::folder_picker::TITLE_INIT_WHERE.get(),
                    msg::folder_picker::CONFIRM_SELECT.get(),
                    start,
                );
                if let Some(RepoDialog::Init { picker, .. }) = self.dialog.as_mut() {
                    *picker = Some(new);
                }
                None
            }
            InitOutcome::Submit(path, opts) => {
                self.init_rx = Some(init_task::start(self.session.clone(), path, opts));
                if let Some(RepoDialog::Init { form, .. }) = self.dialog.as_mut() {
                    form.busy = true;
                    form.message = None;
                }
                None
            }
        }
    }

    fn on_workspace(&mut self, outcome: WsOutcome) -> Option<DialogEffect> {
        match outcome {
            WsOutcome::Stay => None,
            WsOutcome::Cancel => {
                self.dialog = None;
                None
            }
            WsOutcome::Browse => {
                let start = last_folder()
                    .or_else(gitpoulpi_config::home_dir)
                    .unwrap_or_default();
                let new = self.new_picker(
                    msg::folder_picker::TITLE_OPEN.get(),
                    msg::folder_picker::CONFIRM_SELECT.get(),
                    start,
                );
                if let Some(RepoDialog::Workspace { picker, .. }) = self.dialog.as_mut() {
                    *picker = Some(new);
                }
                None
            }
            WsOutcome::Submit(draft) => {
                if let Some(RepoDialog::Workspace { form, .. }) = self.dialog.as_mut() {
                    form.busy = true;
                    form.message = None;
                }
                self.ws_jobs
                    .push(crate::workspace_ui::task::save(self.session.clone(), draft));
                None
            }
        }
    }

    fn on_safety(&mut self, outcome: PanelOutcome) -> Option<DialogEffect> {
        match outcome {
            PanelOutcome::Stay => None,
            PanelOutcome::Close => {
                self.dialog = None;
                None
            }
            PanelOutcome::Effect(effect) => {
                self.dialog = None;
                Some(match effect {
                    PanelEffect::Message(text) => DialogEffect::Message(text),
                    PanelEffect::ShowCommit(oid) => DialogEffect::ShowCommit(oid),
                    PanelEffect::CherryPick(oid) => DialogEffect::CherryPick(oid),
                })
            }
        }
    }

    /// Closed on submit: the daemon answers on the status line (the
    /// in-flight text, then success or git's refusal), like Create worktree.
    fn on_flow(&mut self, outcome: crate::flow::FlowOutcome) -> Option<DialogEffect> {
        if outcome == crate::flow::FlowOutcome::Close {
            self.dialog = None;
        }
        None
    }

    #[cfg(feature = "hosting")]
    fn on_issue_flow(
        &mut self,
        outcome: crate::flow::issue_flow::IssueFlowOutcome,
    ) -> Option<DialogEffect> {
        if outcome == crate::flow::issue_flow::IssueFlowOutcome::Close {
            self.dialog = None;
        }
        None
    }

    fn on_sparse(&mut self, outcome: SparseOutcome) -> Option<DialogEffect> {
        match outcome {
            SparseOutcome::Stay => None,
            SparseOutcome::Cancel => {
                self.dialog = None;
                None
            }
            SparseOutcome::Apply(dirs) => {
                self.dialog = None;
                Some(DialogEffect::SparseSet(dirs))
            }
            SparseOutcome::Disable => {
                self.dialog = None;
                Some(DialogEffect::SparseDisable)
            }
        }
    }

    fn on_worktree(&mut self, outcome: WorktreeOutcome) -> Option<DialogEffect> {
        match outcome {
            WorktreeOutcome::Stay => None,
            WorktreeOutcome::Cancel => {
                self.dialog = None;
                None
            }
            WorktreeOutcome::NameWithAi {
                description,
                branches,
            } => Some(DialogEffect::NameWorktree {
                description,
                branches,
            }),
            WorktreeOutcome::StopAi => Some(DialogEffect::StopWorktreeAi),
            WorktreeOutcome::Browse(start) => {
                let cwd = std::env::current_dir().unwrap_or_default();
                let start = Self::browse_start(&start, &cwd);
                let new = self.new_picker(
                    msg::folder_picker::TITLE_WORKTREE_WHERE.get(),
                    msg::folder_picker::CONFIRM_SELECT.get(),
                    start,
                );
                if let Some(RepoDialog::Worktree { picker, .. }) = self.dialog.as_mut() {
                    *picker = Some(new);
                }
                None
            }
            // Closed on submit: the daemon answers on the status line
            // (success or git's refusal), like every other repository
            // action, and the tab opens once the worktree is listed.
            WorktreeOutcome::Submit {
                path,
                opts,
                open_tab,
            } => {
                self.dialog = None;
                Some(DialogEffect::AddWorktree {
                    path,
                    opts,
                    open_tab,
                })
            }
        }
    }

    #[cfg(feature = "worktree")]
    fn on_worktree_remove(
        &mut self,
        outcome: crate::worktree_remove_dialog::RemoveOutcome,
    ) -> Option<DialogEffect> {
        use crate::worktree_remove_dialog::RemoveOutcome as O;
        match outcome {
            O::Stay => None,
            O::Cancel => {
                self.dialog = None;
                None
            }
            O::Remove {
                path,
                opts,
                delete_branch,
            } => {
                self.dialog = None;
                Some(DialogEffect::RemoveWorktree {
                    path,
                    opts,
                    delete_branch,
                })
            }
            O::OpenToCommit(path) => {
                self.dialog = None;
                Some(DialogEffect::OpenWorktreeToCommit(path))
            }
            O::Merge {
                into,
                from,
                then_remove,
            } => {
                self.dialog = None;
                Some(DialogEffect::MergeWorktree {
                    into,
                    from,
                    then_remove,
                })
            }
        }
    }

    #[cfg(feature = "hosting")]
    fn on_pull_request(&mut self, outcome: PullRequestOutcome) -> Option<DialogEffect> {
        match outcome {
            PullRequestOutcome::Stay => None,
            PullRequestOutcome::Cancel => {
                self.dialog = None;
                self.pr_rx.is_some().then(|| {
                    DialogEffect::Message(msg::repo_dialogs::PR_STILL_OPENING.get().into())
                })
            }
            // The form only offers the button while AI is on; the event
            // loop starts the request (it owns the AI worker).
            PullRequestOutcome::Generate(req) => Some(DialogEffect::GeneratePullRequest {
                source: req.source_branch,
                target: req.target_branch,
            }),
            PullRequestOutcome::OpenWeb(url) => {
                let provider = match self.dialog.as_ref() {
                    Some(RepoDialog::PullRequest { form }) => form.provider_name(),
                    _ => "",
                };
                self.open_web(&url, provider)
            }
            PullRequestOutcome::Submit { integration, req } => {
                self.list_send(0, msg::jobs::KIND_SEND.get().to_string());
                self.pr_rx = Some(crate::pull_request::start_create(integration, *req));
                if let Some(RepoDialog::PullRequest { form }) = self.dialog.as_mut() {
                    form.busy = true;
                    form.message = None;
                }
                None
            }
        }
    }

    /// Opens a provider's prefilled web form in the browser. The dialog
    /// stays open, so the user can still create through the API; the
    /// status line says so; a missing browser is said on the form.
    #[cfg(feature = "hosting")]
    fn open_web(&mut self, url: &str, provider: &str) -> Option<DialogEffect> {
        match crate::repo_menu::open_url(url) {
            Ok(()) => Some(DialogEffect::Message(msg::repo_dialogs::web_form_opened(
                provider,
            ))),
            Err(why) => {
                crate::activity_log::report_problem(crate::activity_log::ProblemSource::Tool, &why);
                let message = msg::repo_dialogs::web_form_no_browser(url);
                match self.dialog.as_mut() {
                    Some(RepoDialog::PullRequest { form }) => form.message = Some(message),
                    Some(RepoDialog::Issue { form }) => form.message = Some(message),
                    _ => {}
                }
                None
            }
        }
    }

    #[cfg(feature = "hosting")]
    fn on_issue(&mut self, outcome: IssueOutcome) -> Option<DialogEffect> {
        match outcome {
            IssueOutcome::Stay => None,
            IssueOutcome::Cancel => {
                self.dialog = None;
                self.resume_board();
                self.issue_rx.is_some().then(|| {
                    DialogEffect::Message(msg::repo_dialogs::ISSUE_STILL_CREATING.get().into())
                })
            }
            IssueOutcome::OpenWeb(url) => {
                let provider = match self.dialog.as_ref() {
                    Some(RepoDialog::Issue { form }) => form.provider_name(),
                    _ => "",
                };
                self.open_web(&url, provider)
            }
            IssueOutcome::Submit { integration, req } => {
                self.list_send(1, msg::jobs::KIND_SEND.get().to_string());
                self.issue_rx = Some(crate::issue_form::start_create(
                    self.issue_factory.clone(),
                    integration,
                    crate::pull_request::load_integrations(),
                    *req,
                ));
                if let Some(RepoDialog::Issue { form }) = self.dialog.as_mut() {
                    form.busy = true;
                    form.message = None;
                }
                None
            }
        }
    }

    /// Writes the source the user moved the Clone dialog to (once per
    /// change), so the next opening can start on it.
    fn remember_clone_source(&mut self) {
        let Some(RepoDialog::Clone { form, .. }) = self.dialog.as_mut() else {
            return;
        };
        let Some(profile) = form.take_source_change() else {
            return;
        };
        if let Some(dir) = crate::session_store::config_dir() {
            let path = gitpoulpi_session::CloneSource::file_in(&dir);
            let state = gitpoulpi_session::CloneSource {
                last_profile: profile,
            };
            if let Err(err) = state.save(&path) {
                tracing::debug!("clone source not saved: {err}");
            }
        }
    }

    fn on_clone(&mut self, outcome: CloneOutcome) -> Option<DialogEffect> {
        self.remember_clone_source();
        match outcome {
            CloneOutcome::Stay => None,
            CloneOutcome::Open(path) => {
                self.dialog = None;
                Some(DialogEffect::Open(path))
            }
            CloneOutcome::OpenIntegrations => {
                self.dialog = None;
                Some(DialogEffect::OpenIntegrations)
            }
            #[cfg(not(feature = "hosting"))]
            CloneOutcome::LoadProjects(_) => None,
            #[cfg(feature = "hosting")]
            CloneOutcome::LoadProjects(profile_id) => {
                let all = crate::pull_request::load_integrations();
                match all.iter().find(|i| i.id == profile_id).cloned() {
                    Some(integration) => {
                        let cache = clone_list_job(&integration);
                        self.projects_rx.push(crate::hosting_task::start_projects(
                            std::sync::Arc::new(crate::hosting_task::DefaultFactory),
                            integration,
                            all,
                            cache,
                        ))
                    }
                    None => {
                        if let Some(RepoDialog::Clone { form, .. }) = self.dialog.as_mut() {
                            form.set_projects(
                                &profile_id,
                                Err(msg::repo_dialogs::PROFILE_GONE.get().into()),
                            );
                        }
                    }
                }
                None
            }
            CloneOutcome::Cancel => {
                self.dialog = None;
                #[cfg(feature = "hosting")]
                self.projects_rx.clear();
                self.review_clone = None;
                None
            }
            CloneOutcome::Browse(field) => {
                let cwd = std::env::current_dir().unwrap_or_default();
                let start = Self::browse_start(&field, &cwd);
                let new = self.new_picker(
                    msg::clone::WHERE_TO.get(),
                    msg::folder_picker::CONFIRM_SELECT.get(),
                    start,
                );
                if let Some(RepoDialog::Clone { picker, .. }) = self.dialog.as_mut() {
                    *picker = Some(new);
                }
                None
            }
            CloneOutcome::Submit { url, dest, opts } => {
                // The clone is the daemon's job and outlives this dialog and
                // this window (B9): the dialog closes, the jobs indicator
                // shows the progress and carries the Cancel.
                if clone_task::is_running(&dest) {
                    return Some(DialogEffect::Message(msg::repo_dialogs::clone_already(
                        dest.display(),
                    )));
                }
                let name = dest.file_name().map_or_else(
                    || dest.display().to_string(),
                    |n| n.to_string_lossy().into(),
                );
                if let Some(carry) = self.review_clone.take() {
                    crate::after_clone::set(&dest, carry);
                }
                clone_task::start(self.session.clone(), url, dest, opts);
                self.dialog = None;
                Some(DialogEffect::Message(msg::repo_dialogs::clone_started(
                    name,
                )))
            }
        }
    }

    // -- background answers ------------------------------------------------

    /// Drains every background answer: folder listings and quick places
    /// for the pickers, and the init / clone tasks. Never waits.
    pub(crate) fn poll(&mut self) -> Vec<DialogEffect> {
        let mut effects = Vec::new();
        // The request a Clone dialog was prefilled for only counts while
        // that dialog is the open one.
        if !matches!(self.dialog, Some(RepoDialog::Clone { .. })) {
            self.review_clone = None;
        }
        for event in self.lister.drain() {
            match event {
                ListerEvent::Listed(dir, listing) => {
                    if let Some(picker) = self.picker_mut() {
                        picker.deliver(&dir, listing);
                    }
                }
                ListerEvent::Places(places) => {
                    if let Some(picker) = self.picker_mut() {
                        picker.set_places(places);
                    }
                }
            }
        }
        self.pump();
        // The Clone dialog's destination probe, a Clone waiting for it, and
        // the rested source's list request.
        let clone_done = match self.dialog.as_mut() {
            Some(RepoDialog::Clone { form, .. }) => Some(form.poll()),
            _ => None,
        };
        if let Some(outcome) = clone_done {
            effects.extend(self.on_clone(outcome));
        }
        // The Undo history / Reflog / Lost work panel's answers; a result
        // is a status line, and the panel stays open.
        if let Some(RepoDialog::Safety(panel)) = self.dialog.as_mut() {
            for effect in panel.poll() {
                if let PanelEffect::Message(text) = effect {
                    effects.push(DialogEffect::Message(text));
                }
            }
        }
        // The Flow dialog's previews and runs.
        if let Some(RepoDialog::Flow(flow)) = self.dialog.as_mut() {
            if let Some(said) = flow.poll() {
                effects.push(DialogEffect::Message(said));
            }
            if let Some(spec) = flow.take_merge_request() {
                crate::flow::mr::offer(spec);
            }
        }
        // The host-driven flow dialog's host calls and pick-ups.
        #[cfg(feature = "hosting")]
        if let Some(RepoDialog::IssueFlow(flow)) = self.dialog.as_mut() {
            if let Some(said) = flow.poll() {
                effects.push(DialogEffect::Message(said));
            }
        }
        // Workspace jobs: a save closes its form, a walk reports as it goes.
        let mut finished = Vec::new();
        for (i, job) in self.ws_jobs.iter().enumerate() {
            while let Some(event) = job.try_recv() {
                use crate::workspace_ui::task::Event as E;
                match event {
                    E::Progress(line) => effects.push(DialogEffect::WorkspaceProgress(Some(line))),
                    E::Done(line) => {
                        finished.push(i);
                        crate::workspace_ui::apply::changed();
                        crate::workspace_ui::chip::refresh(&self.session);
                        if job.job != 0 {
                            crate::activity_log::note_workspace(&line, &job.workspace);
                        }
                        effects.push(DialogEffect::WorkspaceProgress(None));
                        effects.push(DialogEffect::Message(line));
                    }
                    E::Saved { name, created, .. } => {
                        finished.push(i);
                        if matches!(self.dialog, Some(RepoDialog::Workspace { .. })) {
                            self.dialog = None;
                        }
                        crate::workspace_ui::apply::changed();
                        crate::workspace_ui::chip::refresh(&self.session);
                        effects.push(DialogEffect::Message(if created {
                            msg::workspaces::created(name)
                        } else {
                            msg::workspaces::saved(name)
                        }));
                    }
                    E::Failed(why) => {
                        finished.push(i);
                        effects.push(DialogEffect::WorkspaceProgress(None));
                        match self.dialog.as_mut() {
                            Some(RepoDialog::Workspace { form, .. }) => {
                                form.busy = false;
                                form.message = Some(why);
                            }
                            _ => effects.push(DialogEffect::Message(why)),
                        }
                    }
                }
            }
        }
        for i in finished.into_iter().rev() {
            let job = self.ws_jobs.remove(i);
            if job.job != 0 {
                crate::jobs_ui::local_finish(
                    job.job,
                    if job.stopped() {
                        gitpoulpi_ipc::JobState::Canceled
                    } else {
                        gitpoulpi_ipc::JobState::Done
                    },
                );
            }
        }
        // Provider project lists for the pull-request form's repository
        // dropdowns.
        #[cfg(feature = "hosting")]
        {
            if let Some(RepoDialog::PullRequest { form }) = self.dialog.as_mut() {
                form.poll();
            }
            if let Some(RepoDialog::Issue { form }) = self.dialog.as_mut() {
                form.poll();
            }
            // ...and for the metadata picker of an existing request or issue.
            let meta_done = match self.dialog.as_mut() {
                Some(RepoDialog::Meta(edit)) => edit.poll(),
                _ => None,
            };
            if let Some(outcome) = meta_done {
                effects.extend(self.on_meta(outcome));
            }
            // ...and for the review view's loads, diffs and calls.
            let review_done = match self.dialog.as_mut() {
                Some(RepoDialog::Review(view)) => Some(view.poll()),
                _ => None,
            };
            if let Some(effect) = review_done {
                effects.extend(self.on_review(effect));
            }
            // ...and for the board's loads and moves.
            let board_done = match self.dialog.as_mut() {
                Some(RepoDialog::Board(view)) => Some(view.poll()),
                _ => None,
            };
            if let Some(effect) = board_done {
                effects.extend(self.on_board(effect));
            }
            // ...and for the conversation's reads, layout and timer.
            let item_done = match self.dialog.as_mut() {
                Some(RepoDialog::Item(view)) => Some(view.poll()),
                _ => None,
            };
            if let Some(effect) = item_done {
                effects.extend(self.on_item(effect));
            }
            // ...and for the Clone dialog's hosted sources.
            // No Clone dialog open: nobody waits for these, stop the fetches.
            if !matches!(self.dialog, Some(RepoDialog::Clone { .. })) {
                #[cfg(feature = "hosting")]
                self.projects_rx.clear();
            }
            let mut landed = Vec::new();
            self.projects_rx.retain(|rx| {
                // A fetch sends its saved copy, then one final answer: only
                // the final one ends it.
                let mut open = true;
                while let Some(answer) = rx.try_recv() {
                    open = answer.phase != crate::hosting_task::ListPhase::Final;
                    landed.push(answer);
                    if !open {
                        break;
                    }
                }
                open
            });
            for answer in landed {
                if let Some(RepoDialog::Clone { form, .. }) = self.dialog.as_mut() {
                    match answer.phase {
                        crate::hosting_task::ListPhase::Cached => {
                            if let Ok(list) = answer.result {
                                form.set_cached_projects(&answer.profile_id, list);
                            }
                        }
                        crate::hosting_task::ListPhase::Final => {
                            form.set_projects_answer(
                                &answer.profile_id,
                                answer.result,
                                answer.unauthorized,
                            );
                        }
                    }
                }
            }
        }

        if let Some(finished) = self.init_rx.as_ref().and_then(|rx| rx.try_recv()) {
            self.init_rx = None;
            match finished.result {
                Ok(repo) => {
                    if matches!(self.dialog, Some(RepoDialog::Init { .. })) {
                        self.dialog = None;
                    }
                    let message = msg::repo_dialogs::created(repo.root.display());
                    effects.push(DialogEffect::Created { repo, message });
                }
                Err(e) => {
                    let message = msg::repo_dialogs::create_failed(finished.path.display(), &e);
                    crate::activity_log::report_problem(
                        crate::activity_log::ProblemSource::Create,
                        &message,
                    );
                    if let Some(RepoDialog::Init { form, .. }) = self.dialog.as_mut() {
                        form.busy = false;
                        form.message = Some(message);
                    }
                }
            }
        }

        // Clones are process-wide: whichever window is current when one ends
        // reports it (a tab switch in between loses nothing).
        for end in clone_task::drain() {
            // A clone started for a request owes it a link and a review; one
            // that did not land owes nothing.
            let carry = crate::after_clone::take(&end.dest);
            match end.result {
                Ok(repo) => {
                    let mut message = msg::repo_dialogs::cloned_into(repo.root.display());
                    if let Some(carry) = carry {
                        message = finish_review_clone(&repo.root, carry, message);
                    }
                    effects.push(DialogEffect::Created { repo, message });
                }
                Err(clone_task::CloneFailure::Canceled) => {
                    effects.push(DialogEffect::Message(msg::repo_dialogs::clone_canceled(
                        end.dest.display(),
                    )));
                }
                Err(clone_task::CloneFailure::Failed(e)) => {
                    let message = msg::repo_dialogs::clone_failed(end.dest.display(), &e);
                    // No effect message: the status line's error says it.
                    crate::activity_log::report_problem_in(
                        crate::activity_log::ProblemSource::Clone,
                        &message,
                        &end.dest,
                    );
                }
            }
        }
        #[cfg(feature = "hosting")]
        {
            if let Some(PullRequestEvent::Finished(result)) =
                self.pr_rx.as_ref().and_then(|rx| rx.try_recv())
            {
                self.pr_rx = None;
                self.unlist_send(
                    0,
                    if result.is_ok() {
                        gitpoulpi_ipc::JobState::Done
                    } else {
                        gitpoulpi_ipc::JobState::Failed
                    },
                );
                match result {
                    Ok(created) => {
                        if matches!(self.dialog, Some(RepoDialog::PullRequest { .. })) {
                            self.dialog = None;
                        }
                        // The same thing a no-token flow does on success: open
                        // it, and say so on the status line either way (the
                        // browser may fail to launch headless/over SSH).
                        let opened = crate::repo_menu::open_url(&created.web_url);
                        let mut message = match opened {
                            Ok(()) => {
                                msg::repo_dialogs::pr_opened(created.number, &created.web_url)
                            }
                            Err(_) => msg::repo_dialogs::pr_opened_no_browser(
                                created.number,
                                &created.web_url,
                            ),
                        };
                        // The request exists but its metadata did not all land.
                        if let Some(warning) = &created.warning {
                            message = format!("{message} - {warning}");
                        }
                        effects.push(DialogEffect::Message(message));
                    }
                    Err(e) => {
                        crate::activity_log::report_problem_as(
                            crate::activity_log::ProblemSource::Hosting,
                            "Hosting request: create pull request", // i18n-english
                            "",
                            &e,
                        );
                        if let Some(RepoDialog::PullRequest { form }) = self.dialog.as_mut() {
                            form.busy = false;
                            form.message = Some(msg::repo_dialogs::pr_failed(&e));
                        }
                    }
                }
            }

            if let Some(IssueEvent::Finished(result)) =
                self.issue_rx.as_ref().and_then(|rx| rx.try_recv())
            {
                self.issue_rx = None;
                self.unlist_send(
                    1,
                    if result.is_ok() {
                        gitpoulpi_ipc::JobState::Done
                    } else {
                        gitpoulpi_ipc::JobState::Failed
                    },
                );
                match result {
                    Ok(created) => {
                        if matches!(self.dialog, Some(RepoDialog::Issue { .. })) {
                            self.dialog = None;
                            self.resume_board();
                        }
                        // The ISSUES section re-fetches on its next poll.
                        crate::hosting_ui::request_issues_refresh();
                        effects.push(DialogEffect::Message(msg::repo_dialogs::issue_created(
                            created.number,
                            &created.web_url,
                            crate::dialog_kit::key_name(crate::keymap::chord::Key::Enter),
                        )));
                    }
                    Err(e) => {
                        crate::activity_log::report_problem_as(
                            crate::activity_log::ProblemSource::Hosting,
                            "Hosting request: create issue", // i18n-english
                            "",
                            &e,
                        );
                        if let Some(RepoDialog::Issue { form }) = self.dialog.as_mut() {
                            form.busy = false;
                            form.message = Some(msg::repo_dialogs::issue_failed(&e));
                        }
                    }
                }
            }
        }

        effects
    }

    // -- drawing ------------------------------------------------------------

    pub(crate) fn render(&mut self, area: Rect, buf: &mut Buffer) {
        self.area = area;
        match self.dialog.as_mut() {
            None => {}
            Some(RepoDialog::Browse(picker)) => picker.render(picker_rect(area), buf),
            Some(RepoDialog::Init { form, picker }) => {
                form.render(init_rect(area), buf);
                if let Some(picker) = picker {
                    picker.render(picker_rect(area), buf);
                }
            }
            Some(RepoDialog::Clone { form, picker }) => {
                form.render(form.rect(area), buf);
                if let Some(picker) = picker {
                    picker.render(picker_rect(area), buf);
                }
            }
            Some(RepoDialog::Worktree { form, picker }) => {
                form.render(form.rect(area), buf);
                if let Some(picker) = picker {
                    picker.render(picker_rect(area), buf);
                }
            }
            #[cfg(feature = "worktree")]
            Some(RepoDialog::WorktreeRemove(form)) => {
                form.render(crate::worktree_remove_dialog::rect(area), buf);
            }
            #[cfg(feature = "hosting")]
            Some(RepoDialog::PullRequest { form }) => {
                form.render(pull_request_rect(area), buf);
            }
            #[cfg(feature = "hosting")]
            Some(RepoDialog::Issue { form }) => form.render(issue_rect(area), buf),
            #[cfg(feature = "hosting")]
            Some(RepoDialog::Meta(edit)) => edit.render(area, buf),
            #[cfg(feature = "hosting")]
            Some(RepoDialog::Review(view)) => view.render_all(area, buf),
            #[cfg(feature = "hosting")]
            Some(RepoDialog::Board(view)) => view.render(area, buf),
            #[cfg(feature = "hosting")]
            Some(RepoDialog::Item(view)) => view.render(area, buf),
            Some(RepoDialog::SavePatch { picker, .. }) => picker.render(picker_rect(area), buf),
            Some(RepoDialog::ApplyPatch(picker)) => picker.render(picker_rect(area), buf),
            Some(RepoDialog::Workspace { form, picker }) => {
                form.render(workspace_rect(area), buf);
                if let Some(picker) = picker {
                    picker.render(picker_rect(area), buf);
                }
            }
            Some(RepoDialog::Locate { picker, .. }) => picker.render(picker_rect(area), buf),
            Some(RepoDialog::Sparse(form)) => form.render(sparse_rect(area), buf),
            Some(RepoDialog::Flow(form)) => form.render(crate::flow::flow_rect(area), buf),
            #[cfg(feature = "hosting")]
            Some(RepoDialog::IssueFlow(form)) => {
                form.render(crate::flow::issue_flow::issue_flow_rect(area), buf)
            }
            Some(RepoDialog::Safety(panel)) => panel.render_in(area, buf),
        }
    }
}

/// The first `<dir>/<base>.patch`, `<dir>/<base>-2.patch`, ... that does
/// not already exist - never overwrites a patch left there earlier.
/// A clone started from the Radar for a request has landed: link the new
/// repository to the request's host and carry the review to its window (the
/// carried action is set only now, so `radar::pending`'s expiry never runs
/// down while the clone does). `message` says the clone landed; the line
/// returned adds what follows.
/// Where the Clone dialog keeps the saved repository list of `integration`
/// (under the cache directory, so `GITPOULPI_CONFIG_DIR` isolates tests), or
/// `None` when the profile has no host to key it on.
#[cfg(feature = "hosting")]
fn clone_list_job(
    integration: &gitpoulpi_config::Integration,
) -> Option<crate::hosting_task::ListCacheJob> {
    let dir = gitpoulpi_config::ConfigPaths::discover()?
        .cache_dir()
        .join("clone-lists");
    let login = integration.user.as_ref().map(|u| u.login.as_str());
    let key = gitpoulpi_hosting::list_cache::ListKey::of(integration, login)?;
    Some(crate::hosting_task::ListCacheJob { dir, key })
}

fn finish_review_clone(root: &Path, carry: crate::after_clone::Carry, message: String) -> String {
    let all = crate::pull_request::load_integrations();
    let state = crate::remotes_ui::repo_state_path(root);
    match crate::after_clone::write_sources(&state, &carry, &all) {
        Ok(()) => {
            crate::radar::pending::set(crate::radar::pending::Pending::Review {
                number: carry.number,
                repo: Some(root.to_path_buf()),
            });
            msg::radar::msg_cloned_review(message, carry.label)
        }
        Err(why) => msg::radar::msg_cloned_link_failed(message, why),
    }
}

fn first_free_patch_name(dir: &Path, base: &str) -> PathBuf {
    let candidate = dir.join(format!("{base}.patch"));
    if !candidate.exists() {
        return candidate;
    }
    (2..)
        .map(|n| dir.join(format!("{base}-{n}.patch")))
        .find(|candidate| !candidate.exists())
        .expect("an unbounded counter always finds a free name")
}

/// Where the Repository Management tab's **Browse** entry opens: the
/// configured `repos.browse_root` when it is set, else the last folder
/// chosen this run, else the home folder, else `cwd` when even that is
/// unknown. Pure, so the fallback chain is tested without touching real
/// config files or process environment.
fn browse_start_for(configured: Option<PathBuf>, last: Option<PathBuf>, cwd: &Path) -> PathBuf {
    configured
        .or(last)
        .or_else(gitpoulpi_config::home_dir)
        .unwrap_or_else(|| cwd.to_path_buf())
}

/// Carries out one [`DialogEffect`]; returns the line to show, if any.
pub(crate) fn apply_dialog_effect(effect: DialogEffect, ctx: &mut TabCtx) -> Option<String> {
    match effect {
        DialogEffect::Message(message) => Some(message),
        // Started by the event loop's `apply_effect!`, which owns the AI
        // worker; nothing to do here.
        DialogEffect::GeneratePullRequest { .. } => None,
        DialogEffect::NameWorktree { .. } | DialogEffect::StopWorktreeAi => None,
        // Handled by the event loop's `apply_effect`.
        DialogEffect::CheckoutPullRequest(_) => None,
        DialogEffect::Open(path) => ctx.switcher_action(repo_switcher::SwitcherAction::Open(path)),
        // Handled by the event loop's `apply_effect`.
        DialogEffect::OpenIntegrations => None,
        DialogEffect::Created { repo, message } => {
            // The new repository becomes a tab and the window moves to
            // it: creating something and then having to go and find it is
            // the gap these dialogs exist to close.
            session_store::open_tab(ctx.session, &repo.root, None);
            *ctx.tabs = session_store::tabs(ctx.session);
            *ctx.active_tab = ctx
                .tabs
                .iter()
                .position(|t| t.path == repo.root)
                .unwrap_or(0);
            session_store::activate_tab(ctx.session, *ctx.active_tab);
            ctx.dialogs.close();
            *ctx.switch_to = Some(repo);
            Some(message)
        }
        DialogEffect::AddWorktree {
            path,
            opts,
            open_tab,
        } => {
            *ctx.pending_worktree_tab = open_tab.then(|| path.clone());
            let shown = path.display().to_string();
            ctx.link.submit_action(Action::WorktreeAdd { path, opts });
            Some(msg::repo_dialogs::creating_worktree(shown))
        }
        #[cfg(feature = "worktree")]
        DialogEffect::RemoveWorktree {
            path,
            opts,
            delete_branch,
        } => {
            let shown = path.display().to_string();
            ctx.link.submit_action(Action::WorktreeRemove {
                path,
                opts: Box::new(opts),
                delete_branch,
            });
            Some(msg::worktree_manage::removing(shown))
        }
        #[cfg(feature = "worktree")]
        DialogEffect::OpenWorktreeToCommit(path) => {
            crate::worktree_remove_dialog::want_commit_focus(&path);
            ctx.dialogs.close();
            ctx.switcher_action(repo_switcher::SwitcherAction::Open(path))
        }
        #[cfg(feature = "worktree")]
        DialogEffect::MergeWorktree {
            into,
            from,
            then_remove,
        } => {
            let branch = match &from.kind {
                gitpoulpi_core::domain::RefKind::LocalBranch { name } => name.clone(),
                _ => String::new(),
            };
            ctx.link.submit_action(Action::WorktreeMerge {
                worktree: into,
                from: Box::new(from),
                then_remove,
            });
            Some(msg::worktree_manage::merging(branch))
        }
        DialogEffect::CreatePatch { source, dest } => {
            let shown = dest.display().to_string();
            ctx.link.submit_action(Action::PatchCreate { source, dest });
            Some(msg::repo_dialogs::writing_patch(shown))
        }
        DialogEffect::ApplyPatch(path) => {
            let shown = path.display().to_string();
            ctx.link.submit_action(Action::PatchApply { path });
            Some(msg::repo_dialogs::applying(shown))
        }
        DialogEffect::SparseSet(dirs) => {
            ctx.link.submit_action(Action::SparseSet { dirs });
            Some(msg::sparse::APPLYING.get().to_string())
        }
        DialogEffect::ShowCommit(_) => None,
        // Handled by the event loop's `apply_effect`.
        DialogEffect::ShowCommitPrefix(_) => None,
        DialogEffect::WorkspaceProgress(line) => {
            if let Some(manager) = ctx.management.as_mut() {
                manager.ws.progress = line;
            }
            None
        }
        DialogEffect::CherryPick(oid) => {
            ctx.link.submit_action(Action::CherryPick(oid));
            Some(msg::undo_safety::cherry_picking(short_oid(oid)))
        }
        DialogEffect::SparseDisable => {
            ctx.link.submit_action(Action::SparseDisable);
            Some(msg::sparse::DISABLING.get().to_string())
        }
    }
}

/// Opens the worktree Create worktree asked for as a tab, once `listed`
/// (the latest refresh's worktrees) has it - i.e. once the daemon has
/// actually made it. `None` while it has not appeared yet; the pending
/// path is dropped by the caller when the action fails instead.
pub(crate) fn open_pending_worktree_tab(
    listed: &[gitpoulpi_core::domain::Worktree],
    ctx: &mut TabCtx,
) -> Option<String> {
    let wanted = ctx.pending_worktree_tab.clone()?;
    let canonical = |p: &Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    if !listed
        .iter()
        .any(|w| canonical(&w.path) == canonical(&wanted))
    {
        return None;
    }
    *ctx.pending_worktree_tab = None;
    ctx.switcher_action(repo_switcher::SwitcherAction::Open(wanted))
}

fn short_oid(oid: gitpoulpi_core::domain::Oid) -> String {
    gitpoulpi_core::domain::short(oid)
}

#[cfg(test)]
mod close_tests;

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    /// Waits (bounded) for the lister's threads, delivering as it goes.
    fn settle(dialogs: &mut RepoDialogs) {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            dialogs.poll();
            let loading = dialogs.picker_mut().is_some_and(|p| p.listing().is_none());
            if !loading {
                return;
            }
            assert!(Instant::now() < deadline, "the lister never answered");
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    /// `repos.browse_root` wins outright when set; unset,
    /// Browse falls back to the last folder chosen, then home, then cwd.
    #[test]
    fn browse_start_prefers_the_configured_root_then_the_last_folder_then_home_then_cwd() {
        let cwd = Path::new("/cwd");
        let configured = PathBuf::from("/configured");
        let last = PathBuf::from("/last");
        assert_eq!(
            browse_start_for(Some(configured.clone()), Some(last.clone()), cwd),
            configured,
            "the setting wins even over a folder already chosen"
        );
        assert_eq!(
            browse_start_for(None, Some(last.clone()), cwd),
            last,
            "unset, the last folder chosen wins"
        );
        // Neither set: home when it is known, else cwd -- never a panic
        // or an empty path either way.
        let expected = gitpoulpi_config::home_dir().unwrap_or_else(|| cwd.to_path_buf());
        assert_eq!(browse_start_for(None, None, cwd), expected);
    }

    #[test]
    fn browse_opens_a_chosen_repository_and_refuses_a_plain_folder() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("repo").join(".git")).unwrap();
        std::fs::create_dir(tmp.path().join("plain")).unwrap();
        let mut dialogs = RepoDialogs::new("test-session");
        dialogs.dialog = Some(RepoDialog::Browse(FolderPicker::new(
            "Open a repository",
            "Open",
            tmp.path().to_path_buf(),
        )));
        dialogs.pump();
        settle(&mut dialogs);

        let ctrl_s = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
        // Nothing selected: the folder shown, which is not a repository.
        assert_eq!(dialogs.on_key(ctrl_s), None);
        let message = dialogs.picker_mut().unwrap().message.clone().unwrap();
        assert!(message.contains("not a git repository"), "{message}");
        assert!(dialogs.is_open());

        // Enter goes into `plain`? No -- select `repo` (the parent row,
        // then plain, then repo) and choose it.
        for _ in 0..3 {
            dialogs.on_key(key(KeyCode::Down));
        }
        assert_eq!(
            dialogs.picker_mut().unwrap().chosen(),
            tmp.path().join("repo")
        );
        assert_eq!(
            dialogs.on_key(ctrl_s),
            Some(DialogEffect::Open(tmp.path().join("repo")))
        );
        assert!(!dialogs.is_open());
    }

    /// Create worktree: Browse goes through the shared picker (Esc leaves
    /// the picker, not the dialog), and Create closes the dialog into an
    /// `AddWorktree` effect for the event loop to send.
    #[cfg(feature = "worktree")]
    #[test]
    fn create_worktree_browses_through_the_picker_and_submits_an_effect() {
        let tmp = tempfile::tempdir().unwrap();
        let mut dialogs = RepoDialogs::new("test-session");
        dialogs.open_worktree(
            WorktreeSeed {
                start: gitpoulpi_core::domain::WorktreeStart::Branch("feature".into()),
                label: "feature".into(),
                new_branch: String::new(),
                branch_busy: false,
            },
            &tmp.path().join("repo"),
        );
        dialogs.on_key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL));
        settle(&mut dialogs);
        assert_eq!(dialogs.picker_mut().unwrap().dir, tmp.path());
        dialogs.on_key(key(KeyCode::Esc));
        assert!(matches!(
            dialogs.dialog,
            Some(RepoDialog::Worktree { picker: None, .. })
        ));
        if let Some(RepoDialog::Worktree { form, .. }) = dialogs.dialog.as_mut() {
            form.focus = crate::worktree_dialog::WtFocus::Create;
        }
        match dialogs.on_key(key(KeyCode::Enter)) {
            Some(DialogEffect::AddWorktree {
                path,
                opts,
                open_tab,
            }) => {
                // The folder is named after the generated `worktree-<id>`.
                assert_eq!(path.parent(), Some(tmp.path()));
                let leaf = path.file_name().unwrap().to_string_lossy().into_owned();
                assert!(
                    gitpoulpi_core::domain::is_generated_worktree_name(
                        leaf.strip_prefix("repo-").unwrap_or(&leaf)
                    ),
                    "{leaf}"
                );
                // The branch the dialog was opened on is checked out as it is.
                assert_eq!(
                    opts.start,
                    gitpoulpi_core::domain::WorktreeStart::Branch("feature".into())
                );
                assert_eq!(opts.new_branch, None);
                assert!(open_tab);
            }
            other => panic!("expected AddWorktree, got {other:?}"),
        }
        assert!(!dialogs.is_open());
    }

    /// Create patch: choosing a folder that already has `name.patch`
    /// picks the next free `name-2.patch` instead of overwriting it.
    #[test]
    fn save_patch_picks_a_free_file_name() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("feature.patch"), b"already here").unwrap();
        let mut dialogs = RepoDialogs::new("test-session");
        dialogs.open_save_patch(
            PatchSeed {
                source: gitpoulpi_core::domain::PatchSource::Commits(vec![
                    gitpoulpi_core::domain::Oid::from_hex(
                        "1111111111111111111111111111111111111111",
                    )
                    .unwrap(),
                ]),
                name: "feature".into(),
            },
            tmp.path(),
        );
        // `open_save_patch` starts at `last_folder()` when the static has one,
        // shared across every test in this binary, so another test choosing a
        // folder first leaves it non-empty. Force the picker back onto
        // `tmp.path()` so the verdict does not depend on run order.
        dialogs
            .picker_mut()
            .unwrap()
            .navigate(tmp.path().to_path_buf());
        settle(&mut dialogs);

        let ctrl_s = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
        match dialogs.on_key(ctrl_s) {
            Some(DialogEffect::CreatePatch { dest, .. }) => {
                assert_eq!(dest, tmp.path().join("feature-2.patch"));
            }
            other => panic!("expected CreatePatch, got {other:?}"),
        }
        assert!(!dialogs.is_open());
        assert_eq!(
            std::fs::read(tmp.path().join("feature.patch")).unwrap(),
            b"already here",
            "the existing patch must survive untouched"
        );
    }

    #[test]
    fn init_and_clone_browse_through_the_picker_and_take_its_folder() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join("target")).unwrap();
        let mut dialogs = RepoDialogs::new("test-session");
        dialogs.dialog = Some(RepoDialog::Init {
            form: InitDialog::new(tmp.path(), "main"),
            picker: None,
        });
        dialogs.on_key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL));
        settle(&mut dialogs);
        assert_eq!(dialogs.picker_mut().unwrap().dir, tmp.path());
        dialogs.on_key(key(KeyCode::Down));
        dialogs.on_key(key(KeyCode::Down));
        dialogs.on_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));
        match &dialogs.dialog {
            Some(RepoDialog::Init { form, picker }) => {
                assert!(picker.is_none());
                assert_eq!(PathBuf::from(&form.parent), tmp.path().join("target"));
            }
            _ => panic!("still the Init dialog"),
        }

        dialogs.dialog = Some(RepoDialog::Clone {
            form: Box::new(CloneDialog::new(Some(tmp.path()))),
            picker: None,
        });
        dialogs.on_key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL));
        settle(&mut dialogs);
        // Esc leaves the picker, not the Clone dialog.
        dialogs.on_key(key(KeyCode::Esc));
        assert!(matches!(
            dialogs.dialog,
            Some(RepoDialog::Clone { picker: None, .. })
        ));
        dialogs.on_key(key(KeyCode::Esc));
        assert!(!dialogs.is_open());
    }
}
