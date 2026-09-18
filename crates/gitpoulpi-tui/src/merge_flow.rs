//! The merge editor's life in the window: opening a file through the daemon
//! link, writing it back, moving to the next conflicted file, and noticing
//! that the file changed behind the editor's back.
//!
//! The editor itself ([`crate::merge_view`]) is pure state and never talks
//! to the daemon; it hands the window an [`Effect`] and this module turns
//! it into a [`MergeRequest`] and the answer ([`MergeReply`]) back into
//! the view. The flow after "Save & Mark Resolved" is settled (maintainer
//! answer 3 of the merge-conflict-editor design): the editor
//! closes and the conflicted-files list is back in front, with the file
//! moved out of it; the next file is opened by the user. Likewise a merge,
//! cherry-pick or revert that stops on conflicts only shows the files and
//! the operation banner: nothing opens by itself.

use std::path::{Path, PathBuf};
use std::sync::mpsc;

use gitpoulpi_core::domain::{
    ConflictEntry, ConflictRefusal, ConflictResolution, MergeDocument, ResolveOptions,
    WorktreeFileState,
};
use gitpoulpi_core::Error as CoreError;
use gitpoulpi_i18n::msg::merge as tr;

use crate::daemon_link::{DaemonLink, MergeReply, MergeRequest};
use crate::merge_view::input::Strip;
use crate::merge_view::model::PicksCache;
use crate::merge_view::{Effect, MergeView, Outcome, Step, Then};

/// A file being read.
#[derive(Debug, Clone)]
pub(crate) struct Opening {
    pub path: PathBuf,
}

/// What an agent asked the editor to open, applied when the file arrives
/// (`ui show conflict`, `ui propose resolution`).
pub(crate) struct AgentOpen {
    pub path: PathBuf,
    /// Scroll to this conflict (1-based).
    pub conflict: Option<usize>,
    pub proposal: Option<AgentProposal>,
}

/// The suggestion a proposal carries, filled into the editor on arrival.
pub(crate) struct AgentProposal {
    pub expected: [Option<gitpoulpi_core::domain::Oid>; 3],
    pub picks: Option<Vec<gitpoulpi_core::domain::ConflictPicks>>,
    pub content: Option<String>,
}

/// The window's side of the editor.
#[derive(Default)]
pub(crate) struct MergeFlow {
    /// An agent's request waiting for its file.
    agent: Option<AgentOpen>,
    /// Picks and Output of files left with work in them, restored when the
    /// same stages are opened again (plan §2.5 case 3).
    cache: PicksCache,
    /// "Full file" is on, for the whole session: the next editor opens in it.
    full_file: bool,
    /// The file the daemon is reading.
    opening: Option<Opening>,
    /// A big file's model, alignment and view are being built on a worker
    /// (the UI thread would stall for a tenth of a second or more).
    building: Option<PathBuf>,
    built: Option<(mpsc::Sender<Built>, mpsc::Receiver<Built>)>,
}

/// A big file's view, built off the UI thread.
struct Built {
    path: PathBuf,
    disk: Option<String>,
    edited_outside: bool,
    view: Box<MergeView>,
}

/// Files with more lines than this (on the longest side) are built on a
/// worker thread.
const BUILD_OFF_THREAD_LINES: usize = 3_000;

/// What the flow needs of the window, borrowed for one call.
pub(crate) struct Env<'a> {
    pub link: &'a DaemonLink,
    pub conflicts: &'a [ConflictEntry],
    pub merge: &'a mut Option<MergeView>,
    /// The repository's working directory (the external merge tool runs
    /// there).
    pub root: &'a Path,
    pub last_message: &'a mut Option<String>,
}

/// The conflict entry's three stage oids differ from what the editor opened
/// on.
fn stages_differ(a: &ConflictEntry, b: &ConflictEntry) -> bool {
    ResolveOptions::oids_of(a) != ResolveOptions::oids_of(b)
}

/// `(k, n)` for the header's "file k of N".
fn file_pos(conflicts: &[ConflictEntry], path: &Path) -> (usize, usize) {
    let n = conflicts.len().max(1);
    let k = conflicts
        .iter()
        .position(|e| e.path == path)
        .map_or(1, |i| i + 1);
    (k, n)
}

impl MergeFlow {
    /// Opens `request.path` for an agent; the view is filled in when the
    /// file arrives.
    pub(crate) fn open_for_agent(&mut self, env: &mut Env<'_>, request: AgentOpen) {
        let path = request.path.clone();
        self.agent = Some(request);
        // A read already under way for the same file answers both; another
        // one is dropped by `open_path` and the request waits for the next.
        if self.opening.is_none() {
            self.open_path(env, path);
        }
    }

    /// An agent's request is waiting for its file.
    pub(crate) fn agent_pending(&self) -> bool {
        self.agent.is_some()
    }

    /// A file is being read: a spinner may show (the loop keeps drawing).
    pub(crate) fn loading(&self) -> bool {
        self.opening.is_some() || self.building.is_some()
    }

    /// Views finished on the worker are put in front.
    pub(crate) fn poll_built(&mut self, env: &mut Env<'_>) {
        while let Some(done) = self.built.as_ref().and_then(|(_, rx)| rx.try_recv().ok()) {
            if self.building.as_ref() == Some(&done.path) {
                self.building = None;
            }
            self.finish_open(env, done.path, done.disk, done.edited_outside, *done.view);
        }
    }

    /// Asks the daemon for `path`'s document; the view appears when the
    /// answer lands.
    pub(crate) fn open_path(&mut self, env: &mut Env<'_>, path: PathBuf) {
        if self.opening.is_some() {
            return;
        }
        *env.last_message = Some(tr::msg_opening(path.display()));
        self.opening = Some(Opening { path: path.clone() });
        env.link.request_merge(MergeRequest::Show { path });
    }

    /// The first conflicted file (the banner's Resolve, the palette with
    /// nothing selected).
    pub(crate) fn open_first(&mut self, env: &mut Env<'_>) {
        match env.conflicts.first().map(|e| e.path.clone()) {
            Some(path) => self.open_path(env, path),
            None => *env.last_message = Some(tr::MSG_NO_CONFLICTS.get().to_string()),
        }
    }

    /// Runs what a key, a click or a command asked of the window.
    pub(crate) fn apply(&mut self, env: &mut Env<'_>, out: Outcome) {
        if let Some(message) = out.message {
            *env.last_message = Some(message);
        }
        let Some(effect) = out.effect else {
            return;
        };
        match effect {
            Effect::Close => self.close(env),
            Effect::Save { stage, then } => self.save(env, stage, then),
            Effect::Goto(step) => self.goto(env, step),
            Effect::Unresolve => {
                if let Some(path) = env.merge.as_ref().map(|v| v.path().to_path_buf()) {
                    *env.last_message = Some(tr::msg_unresolving(path.display()));
                    env.link.request_merge(MergeRequest::Unresolve { path });
                }
            }
            Effect::Reload => {
                if let Some(path) = env.merge.as_ref().map(|v| v.path().to_path_buf()) {
                    self.open_path(env, path);
                }
            }
            Effect::UseFile(side) => self.resolve_whole(env, ConflictResolution::TakeSide(side)),
            Effect::Delete => self.resolve_whole(env, ConflictResolution::Delete),
            Effect::ExternalTool => self.external_tool(env),
            // The window's layout, not the editor's: the command layer
            // answers it before it gets here.
            Effect::TogglePanes => {}
            // Needs the AI worker, which the window owns (`ai_ui`).
            Effect::AiSuggest { .. } => {}
        }
    }

    /// Closes the editor, forgetting what was in it.
    pub(crate) fn close(&mut self, env: &mut Env<'_>) {
        if let Some(view) = env.merge.take() {
            self.full_file = view.full_file;
            self.cache.forget(&view.model);
        }
    }

    fn save(&mut self, env: &mut Env<'_>, stage: bool, then: Then) {
        let Some(view) = env.merge.as_mut() else {
            return;
        };
        let request = view.save_request(stage);
        view.set_saving(Some(then));
        *env.last_message = Some(tr::msg_saving(request.path.display()));
        env.link.request_merge(MergeRequest::Resolve {
            path: request.path,
            resolution: request.resolution,
            stage: request.stage,
            expected: request.expected,
        });
    }

    /// Resolves the whole file at once (a side as it is, or the deletion),
    /// no merge at all.
    fn resolve_whole(&mut self, env: &mut Env<'_>, resolution: ConflictResolution) {
        let Some(view) = env.merge.as_mut() else {
            return;
        };
        if view.is_saving() {
            return;
        }
        let path = view.path().to_path_buf();
        let expected = ResolveOptions::oids_of(&view.model.merge().entry);
        view.set_saving(Some(Then::Close));
        *env.last_message = Some(tr::msg_saving(path.display()));
        env.link.request_merge(MergeRequest::Resolve {
            path,
            resolution,
            stage: true,
            expected,
        });
    }

    /// Hands the file to the configured external merge tool (`git
    /// mergetool` when none is set). The tool stages the file itself; the
    /// next refresh shows it resolved.
    fn external_tool(&mut self, env: &mut Env<'_>) {
        let Some(path) = env.merge.as_ref().map(|v| v.path().to_path_buf()) else {
            return;
        };
        let template = crate::external_tools::current().merge_tool;
        crate::external_tools::open_external_merge(env.root.to_path_buf(), path.clone(), template);
        *env.last_message = Some(tr::msg_external_started(path.display()));
    }

    /// The previous / next conflicted file, wrapping.
    fn goto(&mut self, env: &mut Env<'_>, step: Step) {
        let Some(current) = env.merge.as_ref().map(|v| v.path().to_path_buf()) else {
            return;
        };
        let n = env.conflicts.len();
        let others: Vec<&ConflictEntry> =
            env.conflicts.iter().filter(|e| e.path != current).collect();
        if others.is_empty() {
            *env.last_message = Some(tr::MSG_ONE_FILE.get().to_string());
            return;
        }
        let at = env
            .conflicts
            .iter()
            .position(|e| e.path == current)
            .unwrap_or(0);
        let target = match step {
            Step::Next => (at + 1) % n,
            Step::Prev => (at + n - 1) % n,
        };
        let path = env.conflicts[target].path.clone();
        self.open_path(env, path);
    }

    /// An answer from the daemon.
    pub(crate) fn on_reply(&mut self, env: &mut Env<'_>, reply: MergeReply) {
        match reply {
            MergeReply::Opened { path, result, disk } => {
                if self.opening.as_ref().is_some_and(|o| o.path == path) {
                    self.opening = None;
                }
                match result {
                    Ok(doc) => self.opened(env, path, *doc, disk),
                    Err(e) => {
                        *env.last_message = Some(crate::activity_log::report_problem_clear(
                            crate::activity_log::ProblemSource::Save,
                            &tr::msg_open_failed(path.display(), e.to_string()),
                        ));
                        self.agent_failed(env, &path, None);
                    }
                }
            }
            MergeReply::Saved {
                path,
                stage,
                result,
            } => self.saved(env, path, stage, result),
            MergeReply::Unresolved { path, result } => match result {
                Ok(()) => self.open_path(env, path),
                Err(e) => {
                    *env.last_message = Some(crate::activity_log::report_problem_clear(
                        crate::activity_log::ProblemSource::Save,
                        &tr::msg_save_failed(path.display(), e.to_string()),
                    ));
                }
            },
        }
    }

    fn opened(
        &mut self,
        env: &mut Env<'_>,
        path: PathBuf,
        doc: MergeDocument,
        disk: Option<String>,
    ) {
        let edited_outside = matches!(doc.worktree, WorktreeFileState::EditedNoMarkers { .. });
        let big = doc
            .side_lines()
            .iter()
            .flatten()
            .map(|l| l.len())
            .max()
            .unwrap_or(0)
            .max(doc.regions.len())
            >= BUILD_OFF_THREAD_LINES;
        let job = self.cache.prepare(doc);
        if big {
            let (tx, _) = self.built.get_or_insert_with(mpsc::channel);
            let tx = tx.clone();
            self.building = Some(path.clone());
            let spawned = std::thread::Builder::new()
                .name("gitpoulpi-merge-build".into())
                .spawn({
                    let path = path.clone();
                    move || {
                        let mut view = MergeView::new(job());
                        view.warm_up();
                        let _ = tx.send(Built {
                            path,
                            disk,
                            edited_outside,
                            view: Box::new(view),
                        });
                        crate::redraw::poke();
                    }
                });
            if spawned.is_err() {
                self.building = None;
                *env.last_message = Some(tr::msg_open_failed(path.display(), "no thread"));
            }
            return;
        }
        let view = MergeView::new(job());
        self.finish_open(env, path, disk, edited_outside, view);
    }

    /// The view is built: carries the screen state over, puts it in front
    /// and applies what an agent asked.
    fn finish_open(
        &mut self,
        env: &mut Env<'_>,
        path: PathBuf,
        disk: Option<String>,
        edited_outside: bool,
        mut view: MergeView,
    ) {
        view.file_pos = file_pos(env.conflicts, &path);
        view.full_file = self.full_file;
        if let Some(old) = env.merge.as_ref() {
            view.carry_screen_state(old);
        }
        self.full_file = view.full_file;
        // The file on disk was edited by something else: offer to continue
        // from it, unless what the cache restored is exactly what is there.
        // A card has no text to continue from: the file on disk is whatever
        // git left (binary bytes, a pointer...), not an edit.
        if edited_outside && view.card().is_none() {
            let same = disk.as_deref().is_some_and(|d| {
                d.replace("\r\n", "\n") == view.model.output_file().text.replace("\r\n", "\n")
            });
            if !same {
                view.set_strip(Some(Strip::EditedOutside { disk }));
            }
        }
        view.reveal_current();
        *env.merge = Some(view);
        *env.last_message = None;
        self.apply_agent_request(env, &path);
    }

    /// The file an agent asked for is open: bring its conflict into view
    /// and, for a proposal, fill in the picks or the content (the user's own
    /// Save & Mark Resolved is the only thing that confirms it).
    fn apply_agent_request(&mut self, env: &mut Env<'_>, path: &Path) {
        let Some(request) = self.agent.take_if(|a| a.path == path) else {
            return;
        };
        let Some(view) = env.merge.as_mut() else {
            return;
        };
        if let Some(n) = request.conflict {
            if !view.show_conflict(n) && view.card().is_none() {
                *env.last_message = Some(tr::msg_agent_no_conflict(n as u64, path.display()));
            }
        }
        let Some(proposal) = request.proposal else {
            return;
        };
        if ResolveOptions::oids_of(&view.model.merge().entry) != proposal.expected {
            self.agent_failed(env, path, Some(tr::MSG_AGENT_STALE.get().to_string()));
            return;
        }
        let filled = match (&proposal.picks, &proposal.content) {
            (Some(picks), _) => view.propose_picks(picks),
            (None, Some(text)) => view.propose_content(text),
            (None, None) => false,
        };
        if !filled {
            self.agent_failed(env, path, Some(tr::MSG_AGENT_NO_FIT.get().to_string()));
            return;
        }
        if let Some(view) = env.merge.as_ref() {
            crate::ui_intent::proposal::set_proposed(&view.model.output_text());
        }
    }

    /// An agent's request could not be carried out: its proposal (if it was
    /// one) fails, with `message` on the status line.
    fn agent_failed(&mut self, env: &mut Env<'_>, path: &Path, message: Option<String>) {
        if !self.agent.as_ref().map_or(true, |a| a.path == path) {
            return;
        }
        self.agent = None;
        if let Some(active) = crate::ui_intent::proposal::close() {
            crate::ui_intent::proposal::settle(
                &active.id,
                gitpoulpi_ipc::ui::ProposalStatus::Failed,
            );
        }
        if let Some(message) = message {
            *env.last_message = Some(message);
        }
        // A proposal that cannot be applied leaves nothing of it on screen.
        if env
            .merge
            .as_ref()
            .is_some_and(|v| v.path() == path && !v.model.is_dirty())
        {
            self.close(env);
        }
    }

    fn saved(
        &mut self,
        env: &mut Env<'_>,
        path: PathBuf,
        stage: bool,
        result: Result<(), CoreError>,
    ) {
        let then = env
            .merge
            .as_mut()
            .and_then(|v| v.set_saving(None))
            .unwrap_or(Then::Stay);
        match result {
            Ok(()) if stage => {
                // An agent's proposal: this is the user's confirm.
                if let Some(view) = env.merge.as_ref() {
                    crate::ui_intent::proposal::confirmed(
                        crate::ui_intent::proposal::Dialog::Conflict,
                        &view.model.output_text(),
                    );
                }
                // Back to the conflicted-files list.
                let left = env.conflicts.iter().filter(|e| e.path != path).count();
                self.close(env);
                *env.last_message = Some(if left == 0 {
                    tr::msg_resolved(path.display())
                } else {
                    tr::msg_resolved_left(left as u64, path.display())
                });
            }
            Ok(()) => {
                if let Some(view) = env.merge.as_mut() {
                    // Remember the picks: reopening the same stages restores
                    // them, where the file on disk alone could not.
                    self.cache.store(&view.model);
                    view.model.mark_saved();
                }
                *env.last_message = Some(tr::msg_saved(path.display()));
                match then {
                    Then::Close => self.close(env),
                    Then::File(step) => self.goto(env, step),
                    Then::Stay => {}
                }
            }
            Err(CoreError::ConflictRefused(ConflictRefusal::Stale)) => {
                // "Resolved elsewhere" is the more specific of the two: it
                // keeps its Unresolve button.
                if let Some(view) = env
                    .merge
                    .as_mut()
                    .filter(|v| !matches!(v.strip(), Some(Strip::Resolved)))
                {
                    view.set_strip(Some(Strip::Changed));
                }
                *env.last_message = Some(tr::msg_stale(path.display()));
            }
            Err(e) => {
                *env.last_message = Some(crate::activity_log::report_problem_clear(
                    crate::activity_log::ProblemSource::Save,
                    &tr::msg_save_failed(path.display(), e.to_string()),
                ));
            }
        }
    }

    /// A refresh landed: tell the open editor when its file was resolved or
    /// re-conflicted by somebody else, and keep "file k of N" right.
    pub(crate) fn on_conflicts(&mut self, env: &mut Env<'_>) {
        let Some(view) = env.merge.as_mut() else {
            return;
        };
        view.file_pos = file_pos(env.conflicts, view.path());
        // A save in flight changes the index by itself.
        if view.is_saving() {
            return;
        }
        match env.conflicts.iter().find(|e| e.path == view.path()) {
            None => {
                if !matches!(view.strip(), Some(Strip::Resolved)) {
                    view.set_strip(Some(Strip::Resolved));
                }
            }
            Some(entry) if stages_differ(entry, &view.model.merge().entry) => {
                if !matches!(view.strip(), Some(Strip::Changed)) {
                    view.set_strip(Some(Strip::Changed));
                }
            }
            Some(_) => {}
        }
    }

    /// The operation ended (or none is running): what the cache kept is
    /// of no use any more.
    pub(crate) fn operation_ended(&mut self) {
        self.cache.clear();
    }
}
