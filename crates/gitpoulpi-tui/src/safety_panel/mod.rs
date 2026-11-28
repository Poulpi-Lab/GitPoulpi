//! The "you can always go back" panel (the undo-safety-net design,
//! slice B): three views over what the daemon keeps.
//!
//! - **Undo history**: every journal entry, newest first, with what it did,
//!   when and its state. *Undo up to here*, *Restore only this* (the one
//!   entry's removal undone, later actions kept) and *Redo up to here*.
//! - **Reflog**: where a branch or HEAD was before, including moves made
//!   outside GitPoulpi. *Show in Log*, *Create branch here*, *Create tag
//!   here*, *Apply to working tree* / *Save as stash*, *Restore the branch
//!   to this position*.
//! - **Lost work**: commits no ref reaches. *Create branch here*, *Create
//!   tag here*, *Cherry-pick*, *Apply to working tree* / *Save as stash*
//!   (no "Show in Log": a lost commit is not in the Log).
//!
//! The panel is an IPC client like the rest of the TUI: every read and
//! every change is a request on the link's connection (`daemon_link::raw`),
//! answered into a [`Reply`] polled every frame, so nothing here ever blocks
//! the render thread. All the state is plain data and the transitions are
//! methods, so the unit tests drive it without a terminal.

mod detail;
mod input;
mod label;
#[cfg(test)]
mod label_tests;
#[cfg(test)]
mod layout_tests;
mod palette;
#[cfg(test)]
mod pane_tests;
mod panes;
mod render;
mod render_panes;
mod rows;
#[cfg(test)]
mod tests;

use std::time::Instant;

use gitpoulpi_core::domain::{
    short, Direction, ListedEntry, LostCommit, LostCommitsPage, Oid, OperationKind, ReflogEntry,
    ReflogPage, RestoreOutcome, SequenceGroup, UndoOutcome, UndoRefusal,
};
use gitpoulpi_core::Error as CoreError;
use gitpoulpi_i18n::msg::undo_safety as m;
use ratatui::layout::Rect;

use crate::combo::Combo;
use crate::daemon_link::{error_text, Reply, Requester};
use detail::Detail;

pub(crate) use label::{offer_of, summary_text};
pub(crate) use palette::SafetyCmd;
use panes::Panes;
#[cfg(test)]
pub(crate) use render::safety_rect;

/// The id the dialog's geometry is kept under (`dialog_kit::frame`).
pub(crate) const DIALOG_ID: &str = "safety";
/// The frame: movable and resizable, never below 60 x 16.
pub(crate) const WIN: crate::dialog_kit::Win = crate::dialog_kit::Win::resizable(DIALOG_ID, 60, 16);
/// The default size, at most (centered, shrunk to the screen).
const DEFAULT_SIZE: (u16, u16) = (160, 44);

/// How many reflog positions one request fetches.
const REFLOG_PAGE: usize = 200;
/// How far from the end of the loaded positions the cursor (or the bottom of
/// the visible rows) may be before the next page is fetched on its own.
const REFLOG_AHEAD: usize = 10;
/// How many lost commits one request fetches.
const LOST_PAGE: usize = 200;

mod vocab;
pub(crate) use vocab::Tab;

/// Everything a row, button or menu entry can do. One list so the
/// buttons, the right-click menu and the keys cannot drift apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Act {
    UndoTo,
    RedoTo,
    /// `Undo whole sequence` (`x`): the newest entries when one sequence run
    /// made them.
    UndoSequence,
    RestoreOnly,
    Offer,
    ShowInLog,
    CreateBranch,
    RestoreRef,
    CherryPick,
    /// `Create tag here` (`t`): a tag at the selected commit.
    CreateTag,
    /// `Apply to working tree` (`w`): the commit's changes onto the files.
    ApplyToWorktree,
    /// `Save as stash` (`s`): the commit's changes kept as a stash.
    SaveAsStash,
    /// Reflog only: fetch the next page of positions.
    LoadMore,
}

/// What the event loop has to do about something the panel did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PanelEffect {
    /// Say this in the status line.
    Message(String),
    /// Put the Log on this commit (closes the panel).
    ShowCommit(Oid),
    /// Cherry-pick this commit (closes the panel).
    CherryPick(Oid),
}

pub(crate) enum PanelOutcome {
    Stay,
    Close,
    Effect(PanelEffect),
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ListState {
    pub(crate) selected: usize,
    pub(crate) offset: usize,
    /// Rows the list showed last frame.
    pub(crate) view_rows: usize,
    /// While the scrollbar thumb is held: where in the thumb.
    pub(crate) grab: Option<i32>,
}

impl ListState {
    pub(crate) fn clamp(&mut self, len: usize) {
        self.selected = self.selected.min(len.saturating_sub(1));
        self.keep_visible();
    }

    /// Keeps the cursor inside the list without moving the scroll: what a
    /// paint does, so a scrollbar drag that left the cursor out of sight
    /// stays where it put the view (a key or a click scrolls to the cursor,
    /// `keep_visible`).
    pub(crate) fn clamp_selected(&mut self, len: usize) {
        self.selected = self.selected.min(len.saturating_sub(1));
    }

    pub(crate) fn keep_visible(&mut self) {
        let rows = self.view_rows.max(1);
        if self.selected < self.offset {
            self.offset = self.selected;
        } else if self.selected >= self.offset + rows {
            self.offset = self.selected + 1 - rows;
        }
    }

    pub(crate) fn move_by(&mut self, delta: isize, len: usize) {
        if len == 0 {
            return;
        }
        self.selected = self
            .selected
            .saturating_add_signed(delta)
            .min(len.saturating_sub(1));
        self.keep_visible();
    }
}

pub(crate) struct HistoryRow {
    pub(crate) entry: ListedEntry,
    /// On the redo stack (undone).
    pub(crate) redo: bool,
}

/// One line of the history list as painted. The list's `selected` and
/// `offset` count lines, so a separator scrolls like a row but is never
/// selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Line {
    /// An entry: its index in [`HistoryView::rows`].
    Row(usize),
    /// A day heading (days since the epoch, UTC) when the list spans several.
    Day(i64),
    /// The divider between the undone entries above and the undo stack.
    Redo,
}

#[derive(Default)]
pub(crate) struct HistoryView {
    /// Undo entries newest first, then the undone ones, next-to-redo first
    /// (the data order; the list paints the undone ones above, see `lines`).
    pub(crate) rows: Vec<HistoryRow>,
    pub(crate) undo_count: usize,
    /// What the list paints, top to bottom: the undone entries (the farthest
    /// from the line first), the divider, then the undo stack newest first.
    pub(crate) lines: Vec<Line>,
    /// The first paint has not placed the scroll yet.
    pub(crate) place_scroll: bool,
    pub(crate) list: ListState,
    load: Option<Reply<gitpoulpi_core::domain::JournalListing>>,
    pub(crate) loaded: bool,
    pub(crate) error: Option<String>,
}

#[derive(Default)]
pub(crate) struct ReflogView {
    /// `HEAD` or a full branch name.
    pub(crate) refname: String,
    pub(crate) entries: Vec<ReflogEntry>,
    pub(crate) more: bool,
    pub(crate) list: ListState,
    load: Option<Reply<ReflogPage>>,
    /// The next page, being fetched (`skip` = the positions already held).
    next: Option<Reply<ReflogPage>>,
    /// The last "load more" failed: the cursor stops asking by itself until
    /// the user asks (the button, `m`, the palette) or the view is reloaded.
    next_failed: bool,
    /// The page on its way was asked for (button, key, palette): say how many
    /// positions are held once it lands. The cursor's own paging is silent.
    announce_next: bool,
    pub(crate) loaded: bool,
    pub(crate) error: Option<String>,
    pub(crate) detail: Detail,
}

impl ReflogView {
    /// The list's rows: every position, then the "Load more" row while the
    /// reflog has more pages.
    pub(crate) fn rows(&self) -> usize {
        self.entries.len() + usize::from(self.more)
    }

    /// The "Load more" row is the one after the last position.
    pub(crate) fn is_more_row(&self, index: usize) -> bool {
        self.more && index == self.entries.len()
    }

    pub(crate) fn fetching_more(&self) -> bool {
        self.next.is_some()
    }
}

#[derive(Default)]
pub(crate) struct LostView {
    pub(crate) commits: Vec<LostCommit>,
    pub(crate) list: ListState,
    quick: Option<Reply<LostCommitsPage>>,
    deep: Option<Reply<LostCommitsPage>>,
    started: Option<Instant>,
    pub(crate) partial: bool,
    pub(crate) loaded: bool,
    pub(crate) error: Option<String>,
    pub(crate) detail: Detail,
}

impl LostView {
    pub(crate) fn scanning(&self) -> bool {
        self.quick.is_some() || self.deep.is_some()
    }

    pub(crate) fn scan_elapsed(&self) -> std::time::Duration {
        self.started.map(|t| t.elapsed()).unwrap_or_default()
    }
}

/// What a confirmation does when the user agrees.
#[derive(Debug, Clone)]
pub(crate) enum Next {
    /// Undo or redo these entries one by one; `confirmed` answers the
    /// daemon's question about the first.
    Steps {
        dir: Direction,
        ids: Vec<u64>,
        confirmed: bool,
        /// "Undo whole sequence": the run these entries belong to (words
        /// the end and the stop by the sequence, not by a count).
        sequence: Option<SequenceGroup>,
    },
    Restore {
        id: u64,
        name: Option<String>,
    },
    ReflogRestore {
        refname: String,
        oid: Oid,
        mode: Option<String>,
    },
    /// Keep the commit's changes as a stash (what is offered when applying
    /// them to the working tree failed).
    Stash {
        commit: Oid,
    },
}

pub(crate) struct Confirm {
    pub(crate) title: String,
    pub(crate) lines: Vec<String>,
    /// A warning painted red under the lines.
    pub(crate) warning: Option<String>,
    pub(crate) button: String,
    pub(crate) red: bool,
    pub(crate) next: Next,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NameField {
    Name,
}

/// What a typed name is for.
#[derive(Debug, Clone)]
pub(crate) enum NameFor {
    /// A restore whose name is taken: run it again under the new one.
    Restore { id: u64 },
    /// `Create branch here`.
    Branch { at: Oid },
    /// `Create tag here`.
    Tag { at: Oid },
}

pub(crate) struct NamePrompt {
    pub(crate) prompt: String,
    pub(crate) value: String,
    pub(crate) edit: crate::text_edit::FieldEditor<NameField>,
    pub(crate) purpose: NameFor,
    pub(crate) error: Option<String>,
}

/// The reflog restore needs the user to say what happens to local changes.
pub(crate) struct ModePick {
    pub(crate) refname: String,
    pub(crate) oid: Oid,
    pub(crate) combo: Combo,
}

pub(crate) struct RowMenu {
    pub(crate) items: Vec<Act>,
    pub(crate) selected: usize,
    pub(crate) at: (u16, u16),
}

pub(crate) enum Overlay {
    Confirm(Confirm),
    Name(NamePrompt),
    Mode(ModePick),
    Menu(RowMenu),
}

/// The thing running now (at most one).
pub(crate) enum Job {
    /// Undo or redo, one entry at a time, newest first.
    Steps {
        dir: Direction,
        ids: Vec<u64>,
        done: usize,
        confirmed: bool,
        sequence: Option<SequenceGroup>,
        pending: Reply<UndoOutcome>,
    },
    /// A restore, planned (`dry`) or real.
    Restore {
        id: u64,
        name: Option<String>,
        dry: bool,
        pending: Reply<RestoreOutcome>,
    },
    ReflogRestore {
        refname: String,
        oid: Oid,
        mode: Option<String>,
        dry: bool,
        pending: Reply<RestoreOutcome>,
    },
    Branch {
        name: String,
        at: Oid,
        pending: Reply<()>,
    },
    Tag {
        name: String,
        at: Oid,
        pending: Reply<()>,
    },
    /// `Apply to working tree`.
    Apply {
        commit: Oid,
        pending: Reply<gitpoulpi_core::domain::PatchApplied>,
    },
    /// `Save as stash`.
    Stash { commit: Oid, pending: Reply<Oid> },
}

impl Job {
    fn label(&self) -> String {
        match self {
            Job::Steps { dir, ids, done, .. } => {
                let n = ids.len() as u64;
                let word = match dir {
                    Direction::Undo => gitpoulpi_i18n::msg::undo_ui::UNDOING.get(),
                    Direction::Redo => gitpoulpi_i18n::msg::undo_ui::REDOING.get(),
                };
                if n > 1 {
                    format!("{word} {}/{n}", done + 1)
                } else {
                    word.to_string()
                }
            }
            _ => m::RUNNING.get().to_string(),
        }
    }
}

/// A line under the lists.
#[derive(Debug, Clone)]
pub(crate) struct Note {
    pub(crate) text: String,
    pub(crate) error: bool,
}

pub(crate) struct SafetyPanel {
    activity: std::sync::Arc<crate::activity::Activity>,
    requester: Requester,
    pub(crate) tab: Tab,
    pub(crate) history: HistoryView,
    pub(crate) reflog: ReflogView,
    pub(crate) lost: LostView,
    pub(crate) overlay: Option<Overlay>,
    pub(crate) job: Option<Job>,
    pub(crate) note: Option<Note>,
    /// `general.undo_depth` (0 = unlimited), read when the panel opened.
    pub(crate) max_depth: usize,
    pub(crate) hover: Option<render::Hit>,
    pub(crate) hits: Vec<(Rect, render::Hit)>,
    pub(crate) list_body: Rect,
    /// The panel frame and its close cross, as the last render painted them.
    pub(crate) frame: Rect,
    pub(crate) close: crate::dialog_kit::CloseHit,
    /// The commit list's neighbours (Reflog and Lost work): file list, diff.
    pub(crate) panes: Panes,
    /// The bound key of Undo, for the "Ctrl+Z undoes this" words.
    undo_key: String,
}

impl SafetyPanel {
    pub(crate) fn open(
        activity: std::sync::Arc<crate::activity::Activity>,
        tab: Tab,
        refname: Option<String>,
        max_depth: usize,
    ) -> Self {
        let requester = activity.raw.requester();
        let mut panel = Self {
            activity,
            requester,
            tab,
            history: HistoryView::default(),
            reflog: ReflogView {
                refname: refname.unwrap_or_else(|| "HEAD".to_string()),
                ..Default::default()
            },
            lost: LostView::default(),
            overlay: None,
            job: None,
            note: None,
            max_depth,
            hover: None,
            hits: Vec::new(),
            list_body: Rect::default(),
            frame: Rect::default(),
            close: Default::default(),
            panes: Panes::default(),
            undo_key: String::new(),
        };
        panel.start_load(tab);
        panel
    }

    /// The daemon's view of the two top entries.
    pub(crate) fn journal_state(&self) -> gitpoulpi_core::domain::JournalState {
        self.activity.journal.state()
    }

    /// The key that undoes, as the user has it bound.
    pub(crate) fn undo_key(&self) -> String {
        let key = crate::keymap::current().shown(crate::keymap::Action::GlobalUndo);
        if key.is_empty() {
            self.undo_key.clone()
        } else {
            key
        }
    }

    // -- loading ---------------------------------------------------------

    pub(crate) fn set_tab(&mut self, tab: Tab) {
        if self.tab == tab {
            return;
        }
        self.tab = tab;
        self.overlay = None;
        let loaded = match tab {
            Tab::History => self.history.loaded,
            Tab::Reflog => self.reflog.loaded,
            Tab::Lost => self.lost.loaded || self.lost.scanning(),
        };
        if !loaded {
            self.start_load(tab);
        }
    }

    /// Opens the Reflog tab on `refname` (`HEAD` or a branch).
    pub(crate) fn show_reflog_of(&mut self, refname: &str) {
        self.reflog = ReflogView {
            refname: refname.to_string(),
            ..Default::default()
        };
        self.tab = Tab::Reflog;
        self.overlay = None;
        self.start_load(Tab::Reflog);
    }

    fn start_load(&mut self, tab: Tab) {
        match tab {
            Tab::History => {
                self.history.error = None;
                self.history.load = Some(
                    self.requester
                        .call(OperationKind::JournalList, serde_json::json!({})),
                );
            }
            Tab::Reflog => {
                self.reflog.error = None;
                self.reflog.next = None;
                self.reflog.next_failed = false;
                self.reflog.announce_next = false;
                self.reflog.load = Some(self.requester.call(
                    OperationKind::Reflog,
                    serde_json::json!({ "ref": self.reflog.refname, "skip": 0, "limit": REFLOG_PAGE }),
                ));
            }
            Tab::Lost => {
                self.lost.error = None;
                self.lost.started = Some(Instant::now());
                // The cheap pass first (reflogs and undo pins) so something
                // shows at once; the dangling-object scan follows and
                // replaces it.
                self.lost.quick = Some(self.requester.call(
                    OperationKind::LostCommits,
                    serde_json::json!({ "deep": false, "limit": LOST_PAGE }),
                ));
                self.lost.deep = Some(self.requester.call(
                    OperationKind::LostCommits,
                    serde_json::json!({ "deep": true, "limit": LOST_PAGE }),
                ));
            }
        }
    }

    fn reload_current(&mut self) {
        self.history.loaded = false;
        self.reflog.loaded = false;
        self.lost.loaded = false;
        self.lost.commits.clear();
        let tab = self.tab;
        self.start_load(tab);
    }

    /// Drains every answer that has come. Never waits.
    pub(crate) fn poll(&mut self) -> Vec<PanelEffect> {
        let mut effects = Vec::new();
        if let Some(Some(result)) = self.history.load.as_ref().map(Reply::take) {
            self.history.load = None;
            match result {
                Ok(listing) => self.history.set(listing),
                Err(e) => self.history.error = Some(error_text::error_text(&e)),
            }
        }
        if let Some(Some(result)) = self.reflog.load.as_ref().map(Reply::take) {
            self.reflog.load = None;
            match result {
                Ok(page) => {
                    self.reflog.entries = page.entries;
                    self.reflog.more = page.more;
                    self.reflog.loaded = true;
                    let len = self.reflog.rows();
                    self.reflog.list.clamp(len);
                }
                Err(e) => self.reflog.error = Some(error_text::error_text(&e)),
            }
        }
        if let Some(Some(result)) = self.reflog.next.as_ref().map(Reply::take) {
            self.reflog.next = None;
            match result {
                Ok(page) => {
                    self.reflog.entries.extend(page.entries);
                    self.reflog.more = page.more;
                    if std::mem::take(&mut self.reflog.announce_next) {
                        let held = self.reflog.entries.len() as u64;
                        self.say(m::reflog_loaded(held));
                    }
                }
                Err(e) => {
                    self.reflog.next_failed = true;
                    self.reflog.announce_next = false;
                    self.complain(error_text::error_text(&e));
                }
            }
        }
        self.load_more_when_near_the_end();
        if let Some(Some(result)) = self.lost.quick.as_ref().map(Reply::take) {
            self.lost.quick = None;
            // The deep pass may have landed first; it is a superset.
            if self.lost.deep.is_some() {
                self.lost.accept(result);
            }
        }
        if let Some(Some(result)) = self.lost.deep.as_ref().map(Reply::take) {
            self.lost.deep = None;
            self.lost.accept(result);
        }
        self.poll_job(&mut effects);
        let want = match self.tab {
            Tab::Reflog => self
                .reflog
                .entries
                .get(self.reflog.list.selected)
                .map(|e| e.new),
            Tab::Lost => self
                .lost
                .commits
                .get(self.lost.list.selected)
                .map(|c| c.oid),
            Tab::History => None,
        };
        match self.tab {
            Tab::Reflog => self.reflog.detail.poll(&self.requester, want),
            Tab::Lost => self.lost.detail.poll(&self.requester, want),
            Tab::History => {}
        }
        self.poll_panes(want);
        effects
    }

    /// Asks for the next page of positions (the daemon answers on the link,
    /// never on the render thread). Nothing to do without `more`, while one is
    /// on its way or before the first page has landed.
    fn load_more_reflog(&mut self) {
        let view = &mut self.reflog;
        if !view.more || !view.loaded || view.next.is_some() || view.load.is_some() {
            return;
        }
        view.next_failed = false;
        view.next = Some(self.requester.call(
            OperationKind::Reflog,
            serde_json::json!({
                "ref": view.refname, "skip": view.entries.len(), "limit": REFLOG_PAGE,
            }),
        ));
    }

    /// The cursor, or the bottom of the rows on screen, is within
    /// [`REFLOG_AHEAD`] rows of the last position loaded: fetch the next page,
    /// so the list just keeps going.
    fn load_more_when_near_the_end(&mut self) {
        if self.tab != Tab::Reflog || self.reflog.next_failed || !self.reflog.more {
            return;
        }
        let list = self.reflog.list;
        let reached = list.selected.max(list.offset + list.view_rows);
        if reached + REFLOG_AHEAD >= self.reflog.entries.len() {
            self.load_more_reflog();
        }
    }

    /// Whether anything is still running (the frame loop keeps drawing).
    pub(crate) fn busy(&self) -> bool {
        self.job.is_some()
            || self.panes.busy()
            || (self.tab != Tab::History && self.pane_commit().is_some() && self.detail().is_none())
            || self.history.load.is_some()
            || self.reflog.load.is_some()
            || self.reflog.fetching_more()
            || self.lost.scanning()
    }

    // -- notes -----------------------------------------------------------

    fn say(&mut self, text: String) {
        self.note = Some(Note { text, error: false });
    }

    fn complain(&mut self, text: String) {
        self.note = Some(Note { text, error: true });
    }

    // -- selection facts -------------------------------------------------

    pub(crate) fn selected_history(&self) -> Option<&HistoryRow> {
        self.history.rows.get(self.history.selected_row()?)
    }

    pub(crate) fn selected_reflog(&self) -> Option<&ReflogEntry> {
        self.reflog.entries.get(self.reflog.list.selected)
    }

    pub(crate) fn selected_lost(&self) -> Option<&LostCommit> {
        self.lost.commits.get(self.lost.list.selected)
    }

    /// The local branch the Reflog tab can restore, `None` for a tag-like
    /// or unreadable name.
    pub(crate) fn reflog_target(&self) -> Option<String> {
        let name = &self.reflog.refname;
        if name == "HEAD" {
            return Some("HEAD".to_string());
        }
        name.strip_prefix("refs/heads/")
            .map(str::to_string)
            .or_else(|| (!name.starts_with("refs/")).then(|| name.clone()))
    }

    /// The actions that apply to the selected row of the open tab, in the
    /// order the buttons and the menu list them.
    pub(crate) fn actions(&self, state: &gitpoulpi_core::domain::JournalState) -> Vec<Act> {
        match self.tab {
            Tab::History => {
                let Some(row) = self.selected_history() else {
                    return Vec::new();
                };
                let mut acts = Vec::new();
                if row.redo {
                    acts.push(Act::RedoTo);
                } else {
                    acts.push(Act::UndoTo);
                    if state.sequence.is_some() {
                        acts.push(Act::UndoSequence);
                    }
                    if row.entry.restore.restorable() {
                        acts.push(Act::RestoreOnly);
                    }
                    if self.history.selected_row() == Some(0) && self.offer(state).is_some() {
                        acts.push(Act::Offer);
                    }
                }
                acts
            }
            Tab::Reflog => {
                let mut acts = Vec::new();
                if self.selected_reflog().is_some() {
                    acts.extend([
                        Act::ShowInLog,
                        Act::CreateBranch,
                        Act::CreateTag,
                        Act::ApplyToWorktree,
                        Act::SaveAsStash,
                    ]);
                    if self.reflog_target().is_some() {
                        acts.push(Act::RestoreRef);
                    }
                }
                if self.reflog.more && !self.reflog.fetching_more() {
                    acts.push(Act::LoadMore);
                }
                acts
            }
            Tab::Lost => {
                if self.selected_lost().is_none() {
                    return Vec::new();
                }
                // No "Show in Log": a lost commit is not in the Log.
                vec![
                    Act::CreateBranch,
                    Act::CreateTag,
                    Act::CherryPick,
                    Act::ApplyToWorktree,
                    Act::SaveAsStash,
                ]
            }
        }
    }

    /// The blocked top entry's reflog position, when the daemon offered one.
    pub(crate) fn offer(
        &self,
        state: &gitpoulpi_core::domain::JournalState,
    ) -> Option<(String, Oid, String)> {
        offer_of(state)
    }

    // -- acting ----------------------------------------------------------

    pub(crate) fn run(
        &mut self,
        act: Act,
        state: &gitpoulpi_core::domain::JournalState,
    ) -> PanelOutcome {
        if self.job.is_some() {
            return PanelOutcome::Stay;
        }
        match act {
            Act::UndoTo => self.ask_steps(Direction::Undo),
            Act::UndoSequence => self.ask_sequence(state),
            Act::RedoTo => self.ask_steps(Direction::Redo),
            Act::RestoreOnly => {
                if let Some(id) = self.selected_history().map(|r| r.entry.summary.id) {
                    self.start_restore(id, None);
                }
            }
            Act::Offer => {
                if let Some((refname, oid, _)) = self.offer(state) {
                    self.start_reflog_restore(refname, oid, None);
                }
            }
            Act::ShowInLog => {
                let oid = match self.tab {
                    Tab::Reflog => self.selected_reflog().map(|e| e.new),
                    Tab::Lost => self.selected_lost().map(|c| c.oid),
                    Tab::History => None,
                };
                if let Some(oid) = oid {
                    return PanelOutcome::Effect(PanelEffect::ShowCommit(oid));
                }
            }
            Act::LoadMore => {
                self.load_more_reflog();
                self.reflog.announce_next = self.reflog.fetching_more();
            }
            Act::CherryPick => {
                if let Some(c) = self.selected_lost() {
                    return PanelOutcome::Effect(PanelEffect::CherryPick(c.oid));
                }
            }
            Act::CreateTag => {
                if let Some(at) = self.selected_commit() {
                    self.overlay = Some(Overlay::Name(NamePrompt::new(
                        m::name_tag(short(at)),
                        format!("restored-{}", short(at)),
                        NameFor::Tag { at },
                    )));
                }
            }
            Act::ApplyToWorktree => {
                if let Some(commit) = self.selected_commit() {
                    let pending = self.requester.call(
                        OperationKind::ApplyCommit,
                        serde_json::json!({ "commit": commit }),
                    );
                    self.job = Some(Job::Apply { commit, pending });
                }
            }
            Act::SaveAsStash => {
                if let Some(commit) = self.selected_commit() {
                    self.start_stash(commit);
                }
            }
            Act::CreateBranch => {
                let oid = self.selected_commit();
                if let Some(at) = oid {
                    self.overlay = Some(Overlay::Name(NamePrompt::new(
                        m::name_branch(short(at)),
                        format!("restored-{}", short(at)),
                        NameFor::Branch { at },
                    )));
                }
            }
            Act::RestoreRef => {
                if let (Some(target), Some(oid)) =
                    (self.reflog_target(), self.selected_reflog().map(|e| e.new))
                {
                    self.start_reflog_restore(target, oid, None);
                }
            }
        }
        PanelOutcome::Stay
    }

    /// The commit the selected row of the Reflog or Lost work view names.
    fn selected_commit(&self) -> Option<Oid> {
        match self.tab {
            Tab::Reflog => self.selected_reflog().map(|e| e.new),
            Tab::Lost => self.selected_lost().map(|c| c.oid),
            Tab::History => None,
        }
    }

    fn start_stash(&mut self, commit: Oid) {
        let pending = self.requester.call(
            OperationKind::StashFromCommit,
            serde_json::json!({ "commit": commit }),
        );
        self.job = Some(Job::Stash { commit, pending });
    }

    /// The rows an "up to here" on the selected row covers, in the order
    /// they are undone (or redone).
    fn steps_for_selection(&self) -> Option<(Direction, Vec<&HistoryRow>)> {
        let sel = self.history.selected_row()?;
        let row = self.history.rows.get(sel)?;
        if row.redo {
            let first = self.history.undo_count;
            Some((
                Direction::Redo,
                self.history.rows[first..=sel].iter().collect(),
            ))
        } else {
            Some((Direction::Undo, self.history.rows[..=sel].iter().collect()))
        }
    }

    fn ask_steps(&mut self, dir: Direction) {
        let Some((found, rows)) = self.steps_for_selection() else {
            return;
        };
        if found != dir {
            return;
        }
        let ids: Vec<u64> = rows.iter().map(|r| r.entry.summary.id).collect();
        let lines: Vec<String> = rows.iter().map(|r| label::entry_text(&r.entry)).collect();
        let n = ids.len() as u64;
        let (title, button, red) = match dir {
            Direction::Undo => (m::confirm_undo_to(n), m::BTN_UNDO_TO.get(), n > 1),
            Direction::Redo => (m::confirm_redo_to(n), m::BTN_REDO_TO.get(), false),
        };
        self.overlay = Some(Overlay::Confirm(Confirm {
            title,
            lines,
            warning: None,
            button: button.to_string(),
            red,
            next: Next::Steps {
                dir,
                ids,
                confirmed: false,
                sequence: None,
            },
        }));
    }

    /// "Undo whole sequence": the newest entries when one run made them, in
    /// the order they are undone.
    fn ask_sequence(&mut self, state: &gitpoulpi_core::domain::JournalState) {
        let Some(group) = state.sequence.clone().map(|g| *g) else {
            return;
        };
        let lines: Vec<String> = group
            .entries
            .iter()
            .map(|e| label::text(e.kind, &e.label))
            .collect();
        self.overlay = Some(Overlay::Confirm(Confirm {
            title: m::confirm_undo_sequence(&group.name),
            lines,
            warning: Some(m::WARNING_SEQUENCE_SCOPE.get().to_string()),
            button: m::BTN_UNDO_SEQUENCE.get().to_string(),
            red: true,
            next: Next::Steps {
                dir: Direction::Undo,
                ids: group.entries.iter().map(|e| e.id).collect(),
                confirmed: false,
                sequence: Some(group),
            },
        }));
    }

    pub(crate) fn start_steps(
        &mut self,
        dir: Direction,
        ids: Vec<u64>,
        done: usize,
        confirmed: bool,
        sequence: Option<SequenceGroup>,
    ) {
        let Some(id) = ids.get(done).copied() else {
            return;
        };
        let op = match dir {
            Direction::Undo => OperationKind::Undo,
            Direction::Redo => OperationKind::Redo,
        };
        let pending = self.requester.call(
            op,
            serde_json::json!({ "expect": id, "dry_run": false, "confirmed": confirmed }),
        );
        self.job = Some(Job::Steps {
            dir,
            ids,
            done,
            confirmed,
            sequence,
            pending,
        });
    }

    fn start_restore(&mut self, id: u64, name: Option<String>) {
        let pending = self.requester.call(
            OperationKind::JournalRestore,
            serde_json::json!({ "id": id, "name": name, "dry_run": true }),
        );
        self.job = Some(Job::Restore {
            id,
            name,
            dry: true,
            pending,
        });
    }

    fn start_reflog_restore(&mut self, refname: String, oid: Oid, mode: Option<String>) {
        let pending = self.requester.call(
            OperationKind::ReflogRestore,
            serde_json::json!({ "ref": refname, "oid": oid, "mode": mode, "dry_run": true }),
        );
        self.job = Some(Job::ReflogRestore {
            refname,
            oid,
            mode,
            dry: true,
            pending,
        });
    }

    /// The user agreed to a confirmation.
    pub(crate) fn confirm(&mut self, next: Next) {
        self.overlay = None;
        match next {
            Next::Steps {
                dir,
                ids,
                confirmed,
                sequence,
            } => self.start_steps(dir, ids, 0, confirmed, sequence),
            Next::Restore { id, name } => {
                let pending = self.requester.call(
                    OperationKind::JournalRestore,
                    serde_json::json!({ "id": id, "name": name, "dry_run": false }),
                );
                self.job = Some(Job::Restore {
                    id,
                    name,
                    dry: false,
                    pending,
                });
            }
            Next::Stash { commit } => self.start_stash(commit),
            Next::ReflogRestore { refname, oid, mode } => {
                let pending = self.requester.call(
                    OperationKind::ReflogRestore,
                    serde_json::json!({
                        "ref": refname, "oid": oid, "mode": mode, "dry_run": false,
                    }),
                );
                self.job = Some(Job::ReflogRestore {
                    refname,
                    oid,
                    mode,
                    dry: false,
                    pending,
                });
            }
        }
    }

    /// A typed name was accepted.
    pub(crate) fn submit_name(&mut self, purpose: NameFor, value: String) {
        let value = value.trim().to_string();
        if value.is_empty() {
            return;
        }
        self.overlay = None;
        match purpose {
            NameFor::Restore { id } => self.start_restore(id, Some(value)),
            NameFor::Branch { at } => {
                let pending = self.requester.call(
                    OperationKind::BranchCreate,
                    serde_json::json!({ "name": value, "at": at }),
                );
                self.job = Some(Job::Branch {
                    name: value,
                    at,
                    pending,
                });
            }
            NameFor::Tag { at } => {
                let pending = self.requester.call(
                    OperationKind::TagCreate,
                    serde_json::json!({ "name": value, "at": at, "annotated_message": null }),
                );
                self.job = Some(Job::Tag {
                    name: value,
                    at,
                    pending,
                });
            }
        }
    }

    /// The mode combobox picked one.
    pub(crate) fn submit_mode(&mut self, refname: String, oid: Oid, mode: String) {
        self.overlay = None;
        self.start_reflog_restore(refname, oid, Some(mode));
    }

    fn poll_job(&mut self, effects: &mut Vec<PanelEffect>) {
        let Some(job) = self.job.take() else {
            return;
        };
        match job {
            Job::Steps {
                dir,
                ids,
                done,
                confirmed,
                sequence,
                pending,
            } => match pending.take() {
                None => {
                    self.job = Some(Job::Steps {
                        dir,
                        ids,
                        done,
                        confirmed,
                        sequence,
                        pending,
                    })
                }
                Some(Ok(_)) => {
                    if done + 1 < ids.len() {
                        self.start_steps(dir, ids, done + 1, false, sequence);
                    } else {
                        let n = ids.len() as u64;
                        let text = match (&sequence, dir) {
                            (Some(group), _) => m::done_undo_sequence(&group.name),
                            (None, Direction::Undo) => m::done_undo_to(n),
                            (None, Direction::Redo) => m::done_redo_to(n),
                        };
                        effects.push(PanelEffect::Message(text.clone()));
                        self.say(text);
                        self.reload_current();
                    }
                }
                Some(Err(CoreError::NeedsConfirmation(prompt))) => {
                    // The daemon found this step risky: ask, then resume
                    // from it with the answer.
                    let label = self
                        .history
                        .rows
                        .iter()
                        .find(|r| r.entry.summary.id == ids[done])
                        .map(|r| label::entry_text(&r.entry))
                        .unwrap_or_default();
                    let button = match dir {
                        Direction::Undo => gitpoulpi_i18n::msg::confirm_button::UNDO_ANYWAY,
                        Direction::Redo => gitpoulpi_i18n::msg::confirm_button::REDO_ANYWAY,
                    };
                    self.overlay = Some(Overlay::Confirm(Confirm {
                        title: label,
                        lines: vec![prompt],
                        warning: None,
                        button: button.get().to_string(),
                        red: true,
                        next: Next::Steps {
                            dir,
                            ids: ids[done..].to_vec(),
                            confirmed: true,
                            sequence,
                        },
                    }));
                }
                Some(Err(e)) => {
                    let stopped_at = sequence.as_ref().and_then(|g| {
                        let member = g.entries.iter().find(|x| Some(&x.id) == ids.get(done))?;
                        Some(m::sequence_stopped(
                            &g.name,
                            member.step,
                            g.of,
                            label::text(member.kind, &member.label),
                            error_text::error_text(&e),
                            (ids.len() - done) as u64,
                        ))
                    });
                    let text = if let Some(text) = stopped_at {
                        text
                    } else if done > 0 {
                        m::stopped(done as u64, ids.len() as u64, error_text::error_text(&e))
                    } else {
                        error_text::error_text(&e)
                    };
                    self.complain(text);
                    self.reload_current();
                }
            },
            Job::Restore {
                id,
                name,
                dry,
                pending,
            } => match pending.take() {
                None => {
                    self.job = Some(Job::Restore {
                        id,
                        name,
                        dry,
                        pending,
                    })
                }
                Some(Ok(outcome)) if dry => {
                    self.overlay = Some(Overlay::Confirm(Confirm {
                        title: m::BTN_RESTORE_ONLY.get().to_string(),
                        lines: outcome.steps,
                        warning: None,
                        button: m::CONFIRM_RESTORE.get().to_string(),
                        red: false,
                        next: Next::Restore { id, name },
                    }));
                }
                Some(Ok(outcome)) => {
                    let key = self.undo_key();
                    let text = if outcome.side_files.is_empty() {
                        m::done_restore(outcome.steps.join("; "), key)
                    } else {
                        m::done_restore_files(outcome.side_files.join(", "), key)
                    };
                    effects.push(PanelEffect::Message(text.clone()));
                    self.say(text);
                    self.reload_current();
                }
                Some(Err(CoreError::UndoRefused(UndoRefusal::NameTaken { name, suggested }))) => {
                    self.overlay = Some(Overlay::Name(NamePrompt::new(
                        m::name_taken(name),
                        suggested,
                        NameFor::Restore { id },
                    )));
                }
                Some(Err(e)) => self.complain(error_text::error_text(&e)),
            },
            Job::ReflogRestore {
                refname,
                oid,
                mode,
                dry,
                pending,
            } => match pending.take() {
                None => {
                    self.job = Some(Job::ReflogRestore {
                        refname,
                        oid,
                        mode,
                        dry,
                        pending,
                    })
                }
                Some(Ok(outcome)) if dry => {
                    let hard = mode.as_deref() == Some("hard");
                    let key = self.undo_key();
                    self.overlay = Some(Overlay::Confirm(Confirm {
                        title: m::confirm_reflog(refname.clone(), short(oid)),
                        lines: outcome.steps,
                        warning: hard.then(|| m::mode_hard_warn(key)),
                        button: m::CONFIRM_RESTORE.get().to_string(),
                        red: hard,
                        next: Next::ReflogRestore { refname, oid, mode },
                    }));
                }
                Some(Ok(_)) => {
                    let text = m::done_reflog(refname, short(oid), self.undo_key());
                    effects.push(PanelEffect::Message(text.clone()));
                    self.say(text);
                    self.reload_current();
                }
                Some(Err(CoreError::UndoRefused(UndoRefusal::WouldOverwrite { .. })))
                    if mode.is_none() =>
                {
                    self.overlay = Some(Overlay::Mode(ModePick::new(refname, oid)));
                }
                Some(Err(e)) => self.complain(error_text::error_text(&e)),
            },
            Job::Branch { name, at, pending } => match pending.take() {
                None => self.job = Some(Job::Branch { name, at, pending }),
                Some(Ok(())) => {
                    let text = m::done_branch(name, short(at));
                    effects.push(PanelEffect::Message(text.clone()));
                    self.say(text);
                }
                Some(Err(e)) => self.complain(error_text::error_text(&e)),
            },
            Job::Tag { name, at, pending } => match pending.take() {
                None => self.job = Some(Job::Tag { name, at, pending }),
                Some(Ok(())) => {
                    let text = m::done_tag(name, short(at));
                    effects.push(PanelEffect::Message(text.clone()));
                    self.say(text);
                }
                Some(Err(e)) => self.complain(error_text::error_text(&e)),
            },
            Job::Apply { commit, pending } => match pending.take() {
                None => self.job = Some(Job::Apply { commit, pending }),
                Some(Ok(_)) => {
                    let text = m::done_apply(short(commit), self.undo_key());
                    effects.push(PanelEffect::Message(text.clone()));
                    self.say(text);
                }
                // Nothing changed: offer to keep the changes as a stash.
                Some(Err(e)) => {
                    self.overlay = Some(Overlay::Confirm(Confirm {
                        title: m::APPLY_FAILED_TITLE.get().to_string(),
                        lines: vec![
                            error_text::error_text(&e),
                            m::APPLY_FAILED_NOTHING.get().to_string(),
                        ],
                        warning: None,
                        button: m::BTN_STASH.get().to_string(),
                        red: false,
                        next: Next::Stash { commit },
                    }));
                }
            },
            Job::Stash { commit, pending } => match pending.take() {
                None => self.job = Some(Job::Stash { commit, pending }),
                Some(Ok(_)) => {
                    let text = m::done_stash(short(commit), self.undo_key());
                    effects.push(PanelEffect::Message(text.clone()));
                    self.say(text);
                }
                Some(Err(e)) => self.complain(error_text::error_text(&e)),
            },
        }
    }
}

impl HistoryView {
    fn set(&mut self, listing: gitpoulpi_core::domain::JournalListing) {
        let keep = self.selected_row().map(|r| self.rows[r].entry.summary.id);
        let first_load = !self.loaded;
        let mut rows: Vec<HistoryRow> = listing
            .undo
            .into_iter()
            .rev()
            .map(|entry| HistoryRow { entry, redo: false })
            .collect();
        self.undo_count = rows.len();
        rows.extend(
            listing
                .redo
                .into_iter()
                .rev()
                .map(|entry| HistoryRow { entry, redo: true }),
        );
        self.rows = rows;
        self.lines = build_lines(&self.rows, self.undo_count);
        let at = keep
            .and_then(|id| self.rows.iter().position(|r| r.entry.summary.id == id))
            .or((!self.rows.is_empty()).then_some(0))
            .and_then(|row| self.line_of(row));
        if let Some(line) = at {
            self.list.selected = line;
        }
        self.list.clamp(self.lines.len());
        self.snap(1);
        self.place_scroll = first_load || self.place_scroll;
        self.loaded = true;
    }

    /// The entry index under line `line`.
    pub(crate) fn row_at(&self, line: usize) -> Option<usize> {
        match self.lines.get(line)? {
            Line::Row(row) => Some(*row),
            _ => None,
        }
    }

    pub(crate) fn selected_row(&self) -> Option<usize> {
        self.row_at(self.list.selected)
    }

    fn line_of(&self, row: usize) -> Option<usize> {
        self.lines.iter().position(|l| *l == Line::Row(row))
    }

    /// Moves the selection off a separator: on in the direction it was
    /// going, else back the other way. Keeps the heading above a first
    /// entry in view.
    pub(crate) fn snap(&mut self, dir: isize) {
        let len = self.lines.len();
        let at = self.list.selected;
        if len == 0 || self.row_at(at).is_some() {
            self.list.keep_visible();
            self.reveal_heading();
            return;
        }
        let step = |from: usize, d: isize| -> Option<usize> {
            let mut i = from as isize + d;
            while i >= 0 && (i as usize) < len {
                if self.row_at(i as usize).is_some() {
                    return Some(i as usize);
                }
                i += d;
            }
            None
        };
        let d = if dir < 0 { -1 } else { 1 };
        if let Some(to) = step(at, d).or_else(|| step(at, -d)) {
            self.list.selected = to;
        }
        self.list.keep_visible();
        self.reveal_heading();
    }

    /// A heading just above the selected entry scrolls into view with it.
    fn reveal_heading(&mut self) {
        let sel = self.list.selected;
        if sel > 0 && sel == self.list.offset && self.row_at(sel - 1).is_none() {
            self.list.offset = sel - 1;
        }
    }
}

/// The painted lines: the undone entries on top (the farthest from the
/// line first, the one Redo brings back next just above it), the divider,
/// then the undo stack newest first. Day headings only when the entries
/// span several days.
fn build_lines(rows: &[HistoryRow], undo_count: usize) -> Vec<Line> {
    let day = |row: usize| rows[row].entry.summary.at.div_euclid(86_400);
    let spans = rows.windows(2).any(|w| {
        w[0].entry.summary.at.div_euclid(86_400) != w[1].entry.summary.at.div_euclid(86_400)
    });
    let mut lines = Vec::new();
    let block = |lines: &mut Vec<Line>, order: &mut dyn Iterator<Item = usize>| {
        let mut prev: Option<i64> = None;
        for row in order {
            let d = day(row);
            if spans && prev != Some(d) {
                lines.push(Line::Day(d));
            }
            prev = Some(d);
            lines.push(Line::Row(row));
        }
    };
    block(&mut lines, &mut (undo_count..rows.len()).rev());
    if rows.len() > undo_count {
        lines.push(Line::Redo);
    }
    block(&mut lines, &mut (0..undo_count));
    lines
}

impl LostView {
    fn accept(&mut self, result: Result<LostCommitsPage, CoreError>) {
        match result {
            Ok(page) => {
                let keep = self.commits.get(self.list.selected).map(|c| c.oid);
                self.commits = page.commits;
                self.partial = page.partial;
                if let Some(oid) = keep {
                    if let Some(at) = self.commits.iter().position(|c| c.oid == oid) {
                        self.list.selected = at;
                    }
                }
                self.list.clamp(self.commits.len());
                self.loaded = true;
            }
            Err(e) => self.error = Some(error_text::error_text(&e)),
        }
    }
}

impl NamePrompt {
    fn new(prompt: String, value: String, purpose: NameFor) -> Self {
        Self {
            prompt,
            value,
            edit: Default::default(),
            purpose,
            error: None,
        }
    }
}

impl ModePick {
    fn new(refname: String, oid: Oid) -> Self {
        let mut combo = Combo::new(m::MODE_PLACEHOLDER.get());
        combo.set_options(vec![
            crate::combo::ComboOption::new(m::MODE_KEEP.get(), "keep"),
            crate::combo::ComboOption::new(m::MODE_SOFT.get(), "soft"),
            crate::combo::ComboOption::new(m::MODE_MIXED.get(), "mixed"),
            crate::combo::ComboOption::new(m::MODE_HARD.get(), "hard"),
        ]);
        combo.select(Some("keep".to_string()));
        Self {
            refname,
            oid,
            combo,
        }
    }
}

static OPEN_REQUEST: std::sync::Mutex<Option<(Tab, Option<String>)>> = std::sync::Mutex::new(None);

/// A menu row asks for the panel on `tab` (the Reflog of `refname`); the
/// window opens it on its next tick, which owns the dialogs and the link.
pub(crate) fn ask_to_open(tab: Tab, refname: Option<String>) {
    if let Ok(mut slot) = OPEN_REQUEST.lock() {
        *slot = Some((tab, refname));
    }
    crate::redraw::poke();
}

pub(crate) fn take_open_request() -> Option<(Tab, Option<String>)> {
    OPEN_REQUEST.lock().ok().and_then(|mut slot| slot.take())
}
