//! The Radar: the pull/merge requests, issues and work in progress of
//! every repository of a scope (the active workspace's, else the open tabs),
//! in one list; the maintainer's "scratchpad".
//!
//! A full view of the Home window (the window with no repository behind it, so
//! nothing underneath steals input), held in `RepoManager::radar`. This
//! module owns everything about it and the event loop only forwards keys,
//! mouse events and the frame (`RadarSlot`), then acts on the [`Outcome`].
//!
//! - `model`: items, groups, pins and snoozes, the rows of a view. Pure.
//! - `worker`: the per-repository hosting fetch, off the input thread.
//! - `scope`: which repositories.
//! - `pending`: what the repository's own window does once it is open.
//! - `render`: the paint walk, which also produces the hit rects.
//!
//! Hosting stays opt-in per repository and client side (no daemon): a
//! repository that is not linked to a provider is never asked, it shows a
//! "Link..." line when its remotes suggest one. Only GitHub, GitLab, Gitea and Forgejo are
//! named in the UI.

mod account;
mod account_fs;
mod account_ui;
mod actions;
mod columns;
mod input;
mod item_host;
mod labels;
mod model;
pub(crate) mod pending;
mod render;
mod review_choice;
mod review_flow;
pub(crate) mod scope;
mod vocab;
mod worker;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

use gitpoulpi_i18n::msg;
use gitpoulpi_session::{ItemKey, RadarState, SnoozeFor};

use crate::combo::{Combo, ComboOption, ComboOutcome};
use crate::context_menu::{Entry, Menu, MenuOutcome};
use crate::hosting_task::{unix_now, RadarFactory};
use crate::repo_switcher::RepoFacts;

use account::{AccountAnswer, CloneIndex, Offer};
use labels::{
    act_label, heading_title, note_of, snooze_from_key, snooze_key, snooze_label, view_label,
    who_label,
};
use model::{Board, BoardInput, Group, Heading, Item, ItemKind, Row, View, Who, WipFacts};
use pending::Pending;
use scope::{ScopeChoice, ScopeRepo, BROWSE_FOLDER, CLONE_FOLDER, OPEN_TABS};
pub(crate) use vocab::{
    Act, ColumnCmd, ConversationRequest, Outcome, RadarCommand, ReviewRequest, SizedColumn, Spot,
};
use worker::{FsPrepare, Msg, RadarWorker, RepoAnswer, Setup};

/// Where a click landed. Row indices index the view's rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Hit {
    Back,
    Refresh,
    Scope,
    View,
    Who,
    /// The `Not cloned` checkbox.
    NotCloned,
    /// The `Drafts` checkbox.
    Drafts,
    Filter,
    Heading(usize),
    Star(usize),
    Menu(usize),
    Row(usize),
    /// The checks / conflict glyphs of a request's row: hovering them shows
    /// the tooltip, a click is a click on the row.
    Status(usize),
    NoteAction(usize),
    /// The `[Turn on]` button of a hint row.
    TurnOn(usize),
    /// The snooze duration combobox (opened from a row; never painted as a
    /// field of its own, so never a click target).
    Snooze,
    /// The separator that sizes a column (drag it, double-click to fit).
    Divider(Cell),
    /// A sizable column's title in the header: a right-click opens the menu
    /// that fits the columns.
    Header(Cell),
    /// A cell whose text is cut: hovering it shows the full text.
    Cut(usize, Cell),
    /// A button of the Actions column of an item row.
    Action(usize, actions::Slot),
}

/// The cells of an item row that can be cut (tooltip) or sized (divider).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Cell {
    Repo,
    Title,
    /// The `#` column (number and check marks).
    Num,
    Author,
    /// The age column.
    Age,
    Labels,
}

/// A repository with something to say (an error, a missing link).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Note {
    pub(crate) path: PathBuf,
    pub(crate) name: String,
    pub(crate) text: String,
    pub(crate) act: Option<(Act, Option<String>)>,
}

/// The slot `RepoManager` holds: the Radar when it is showing. Not
/// `Clone`able in any useful sense (a worker thread is behind it), so a clone
/// of a manager starts with it closed.
#[derive(Default)]
pub(crate) struct RadarSlot(Option<Box<RadarUi>>);

impl Clone for RadarSlot {
    fn clone(&self) -> Self {
        Self(None)
    }
}

impl std::fmt::Debug for RadarSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(if self.0.is_some() {
            "RadarSlot(open)"
        } else {
            "RadarSlot(closed)"
        })
    }
}

impl RadarSlot {
    pub(crate) fn is_open(&self) -> bool {
        self.0.is_some()
    }

    /// Opens the view for `session`: production wiring (real worker, real
    /// config directory).
    pub(crate) fn open(&mut self, session: &str) {
        // Off in the settings: no view, and so no worker thread and no
        // hosting fetch (everything the Radar does starts here).
        if self.0.is_none() && crate::build_features::radar_enabled() {
            self.0 = Some(Box::new(RadarUi::production(session)));
        }
    }

    pub(crate) fn close(&mut self) {
        self.0 = None;
    }

    pub(crate) fn get(&mut self) -> Option<&mut RadarUi> {
        self.0.as_deref_mut()
    }

    pub(crate) fn peek(&self) -> Option<&RadarUi> {
        self.0.as_deref()
    }

    /// The parts `Ctrl+N` badges, with the rects the last paint gave them.
    pub(crate) fn nav_targets(&self) -> Vec<(Rect, Spot)> {
        self.0
            .as_deref()
            .map(|ui| ui.nav_targets())
            .unwrap_or_default()
    }

    /// A pressed badge: focus (or open, or run) that part.
    pub(crate) fn nav_focus(&mut self, spot: Spot) -> Outcome {
        match self.0.as_deref_mut() {
            Some(ui) => ui.nav_focus(spot),
            None => Outcome::None,
        }
    }

    /// The pending key prefix's hint, shown in the footer while it waits.
    pub(crate) fn set_mode_hint(&mut self, hint: Option<String>) {
        if let Some(ui) = self.0.as_deref_mut() {
            ui.mode_hint = hint;
        }
    }

    pub(crate) fn on_key(&mut self, key: KeyEvent) -> Outcome {
        self.get().map_or(Outcome::None, |ui| ui.on_key(key))
    }

    /// Whether the pointer is on a column separator or sizing one: the
    /// resize pointer shape.
    pub(crate) fn resize_pointer(&self) -> bool {
        self.0.as_deref().is_some_and(RadarUi::resize_pointer)
    }

    /// Whether a separator is being dragged: its release is routed here
    /// wherever the pointer is.
    pub(crate) fn sizing_column(&self) -> bool {
        self.0.as_deref().is_some_and(RadarUi::sizing_column)
    }

    /// The pointer left the body: the hover goes.
    pub(crate) fn pointer_left(&mut self) {
        if let Some(ui) = self.0.as_deref_mut() {
            ui.pointer_left();
        }
    }

    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent, double: bool) -> Outcome {
        self.get()
            .map_or(Outcome::None, |ui| ui.on_mouse(mouse, double))
    }

    pub(crate) fn command(&mut self, command: RadarCommand) -> Outcome {
        self.get().map_or(Outcome::None, |ui| ui.command(command))
    }

    pub(crate) fn render(&mut self, frame: &mut ratatui::Frame, body: Rect) {
        if let Some(ui) = self.get() {
            ui.render(frame, body);
        }
    }

    /// The I-beam over the filter box.
    pub(crate) fn pointer_text_at(&self) -> bool {
        self.peek().is_some_and(RadarUi::pointer_text_at)
    }

    /// The pointing hand over a control, a row or a popup row.
    pub(crate) fn pointer_hand_at(&self, col: u16, row: u16) -> bool {
        self.peek().is_some_and(|ui| ui.pointer_hand_at(col, row))
    }

    /// A popup (a combobox list, the row menu) owns the pointer.
    pub(crate) fn popup_open(&self) -> bool {
        self.peek().is_some_and(RadarUi::popup_open)
    }

    /// Whether a row is selected (the palette grays row actions out without).
    pub(crate) fn row_selected(&self) -> bool {
        self.peek().is_some_and(RadarUi::has_row_selected)
    }

    /// The text filter has the keyboard (so `?` and `t` are typed, not keys).
    pub(crate) fn typing(&self) -> bool {
        self.0.as_ref().is_some_and(|ui| ui.typing())
    }

    pub(crate) fn animating(&self) -> bool {
        self.0.as_ref().is_some_and(|ui| ui.animating())
    }

    /// When the loop must wake to show a tooltip whose dwell is running.
    pub(crate) fn next_deadline(&self) -> Option<Instant> {
        let ui = self.0.as_ref()?;
        match (ui.tooltip_due(), ui.item_deadline()) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        }
    }

    /// Drains the workers; true when something on screen changed.
    pub(crate) fn poll(&mut self) -> bool {
        self.0.as_mut().is_some_and(|ui| ui.poll())
    }

    /// A line the hosted review said since the last call.
    pub(crate) fn take_said(&mut self) -> Option<String> {
        self.0.as_mut().and_then(|ui| ui.said.take())
    }

    /// Whether an API review is showing over the Radar.
    pub(crate) fn hosts_review(&self) -> bool {
        self.peek().is_some_and(RadarUi::hosts_review)
    }

    /// Hands the Radar the review the event loop built.
    #[cfg(feature = "review")]
    pub(crate) fn host_review(&mut self, view: Box<crate::review_view::ReviewView>) {
        if let Some(ui) = self.get() {
            ui.host_review(view);
        }
    }

    /// The hosted review can hand the keyboard to the palette.
    pub(crate) fn review_takes_palette(&self) -> bool {
        self.peek().is_some_and(RadarUi::review_takes_palette)
    }

    /// What the palette needs to gray a review command out.
    pub(crate) fn review_avail(&self) -> Option<crate::review_cmd::ReviewAvail> {
        self.peek().and_then(RadarUi::review_avail)
    }

    /// What the palette may ask of the AI help of the text box open in the
    /// hosted review or conversation.
    pub(crate) fn composer_avail(&self) -> Option<crate::composer_cmd::ComposerAvail> {
        self.peek().and_then(|ui| {
            ui.review_composer_avail()
                .or_else(|| ui.item_composer_avail())
        })
    }

    pub(crate) fn run_composer(&mut self, cmd: crate::composer_cmd::ComposerCmd) -> Option<String> {
        let ui = self.get()?;
        if ui.hosts_review() {
            ui.run_review_composer(cmd)
        } else {
            ui.run_item_composer(cmd)
        }
    }

    pub(crate) fn run_review(&mut self, cmd: crate::review_cmd::ReviewCmd) -> Outcome {
        self.get().map_or(Outcome::None, |ui| ui.run_review(cmd))
    }

    /// Whether a conversation is showing over the Radar.
    pub(crate) fn hosts_item(&self) -> bool {
        self.peek().is_some_and(RadarUi::hosts_item)
    }

    /// Hands the Radar the conversation the event loop built.
    pub(crate) fn host_item(&mut self, view: Box<crate::item_view::ItemView>) {
        if let Some(ui) = self.get() {
            ui.host_item(view);
        }
    }

    pub(crate) fn item_takes_palette(&self) -> bool {
        self.peek().is_some_and(RadarUi::item_takes_palette)
    }

    /// What the palette needs to gray a conversation command out.
    pub(crate) fn item_avail(&self) -> Option<crate::item_cmd::ItemAvail> {
        self.peek().and_then(RadarUi::item_avail)
    }

    pub(crate) fn run_item(&mut self, cmd: crate::item_cmd::ItemCmd) -> Outcome {
        self.get().map_or(Outcome::None, |ui| ui.run_item(cmd))
    }

    /// Another client wrote to an item shown over the Radar.
    pub(crate) fn on_hosting_changed(&mut self, change: &gitpoulpi_ipc::HostingChangeArgs) {
        if let Some(ui) = self.get() {
            ui.on_hosting_changed(change);
        }
    }

    pub(crate) fn paste(&mut self, text: &str) {
        if let Some(ui) = self.get() {
            ui.api_paste(text);
            ui.item_paste(text);
        }
    }
}

/// The groups that start folded: the snoozed ones, a count until asked for.
const FOLDED_BY_DEFAULT: [Heading; 1] = [Heading::Snoozed];

pub(crate) struct RadarUi {
    session: String,
    config_dir: Option<PathBuf>,
    worker: Option<RadarWorker>,
    round: u64,
    in_flight: bool,
    started: Option<Instant>,
    probe: Option<crate::repo_probe::Probe>,

    scopes: Vec<ScopeChoice>,
    scope: String,
    repos: Vec<ScopeRepo>,
    /// A folder scope's repository scan, running on a thread: the scope it
    /// was started for and what it found.
    scan: Option<std::sync::mpsc::Receiver<(String, Vec<PathBuf>)>>,
    answers: Vec<RepoAnswer>,
    /// The account-wide rows, per integration with a list turned on.
    account_answers: Vec<AccountAnswer>,
    /// The integrations being asked for account-wide rows.
    active: Vec<String>,
    /// The code hosts whose lists are off: a hint row each.
    offers: Vec<Offer>,
    /// The clones the machine knows, to place account-wide rows.
    clones: CloneIndex,
    /// The next round asks the account-wide lists whatever their age.
    force_account: bool,
    wips: HashMap<PathBuf, RepoFacts>,

    state: RadarState,
    state_path: Option<PathBuf>,
    board: Board,
    notes: Vec<Note>,
    rows: Vec<Row>,
    built_at: i64,
    now: fn() -> i64,

    view: View,
    who: Who,
    query: String,
    filter_focused: bool,
    edit: crate::text_edit::FieldEditor<()>,
    folded: HashSet<Heading>,

    selected: usize,
    scroll: usize,
    list_height: usize,
    follow: bool,
    /// The list's scrollbar as last painted, and the thumb held.
    bar: Option<crate::scrollbar::Track>,
    bar_drag: crate::scrollbar::BarDrag,

    scope_combo: Combo,
    view_combo: Combo,
    who_combo: Combo,
    snooze_combo: Combo,
    /// The item (index into `board.items`) the open snooze combo is for.
    snooze_for: Option<usize>,
    menu: Option<Menu<Act>>,
    menu_rects: Vec<Rect>,
    menu_hover: Option<usize>,

    hits: Vec<(Rect, Hit)>,
    hover: Option<Hit>,
    /// Where a [`Hit::Status`] hover began, and where the pointer is: the
    /// tooltip shows after the dwell.
    hover_since: Option<Instant>,
    hover_pos: (u16, u16),
    /// The sizable column the keys act on (`C`), and a separator drag.
    col_pick: Option<Cell>,
    col_drag: Option<columns::Drag>,
    /// The last paint's repository / author widths and the repository
    /// column's limit.
    col_eff: [usize; 4],
    /// What each sizable column needs to show every row whole (same slots as
    /// `col_eff`), measured by the last paint: what a fit reads.
    col_content: [usize; 4],
    /// What each sizable column measures when the user has not sized it.
    col_auto: [usize; 4],
    repo_max: usize,
    message: Option<String>,
    /// The `Ctrl+N` prefix hint, set each frame by the event loop.
    mode_hint: Option<String>,
    /// The review choice dialog, when a Review asks.
    asking: Option<review_flow::Asking>,
    /// The review through the API, hosted over the Radar (`review_flow`).
    api_review: Option<Box<crate::review_view::ReviewView>>,
    /// The conversation of a not-cloned item, hosted over the Radar
    /// (`item_host`).
    api_item: Option<Box<crate::item_view::ItemView>>,
    /// A line the hosted review said while polling, for the status line.
    said: Option<String>,
}

impl RadarUi {
    /// The production wiring: the real hosting fetch, the real config
    /// directory, the repositories' own probe for WIP.
    pub(crate) fn production(session: &str) -> Self {
        let config_dir =
            gitpoulpi_config::ConfigPaths::discover().map(|p| p.config_dir().to_path_buf());
        let worker = RadarWorker::spawn(
            Arc::new(RadarFactory::default()),
            Arc::new(FsPrepare::new(config_dir.clone())),
        );
        let mut ui = Self::with_parts(session, config_dir, Some(worker), unix_now);
        ui.refresh();
        ui
    }

    /// Everything injectable: tests hand over a fake worker and a clock.
    pub(crate) fn with_parts(
        session: &str,
        config_dir: Option<PathBuf>,
        worker: Option<RadarWorker>,
        now: fn() -> i64,
    ) -> Self {
        let state_path = config_dir.as_deref().map(RadarState::file_in);
        let mut state = state_path
            .as_deref()
            .map(RadarState::load)
            .unwrap_or_default();
        state.prune(now());
        let scopes = config_dir
            .as_deref()
            .map(scope::choices_in)
            .unwrap_or_default();
        let (scope, repos) = config_dir
            .as_deref()
            .map(|d| scope::radar_scope_saved(d, session, state.scope.as_deref()))
            .unwrap_or_else(|| (OPEN_TABS.to_string(), Vec::new()));
        let mut ui = Self {
            session: session.to_string(),
            config_dir,
            worker,
            round: 0,
            in_flight: false,
            started: None,
            probe: None,
            scopes,
            scope,
            repos,
            scan: None,
            answers: Vec::new(),
            account_answers: Vec::new(),
            active: Vec::new(),
            offers: Vec::new(),
            clones: CloneIndex::default(),
            force_account: false,
            wips: HashMap::new(),
            state,
            state_path,
            board: Board::default(),
            notes: Vec::new(),
            rows: vec![Row::Empty],
            built_at: now(),
            now,
            view: View::All,
            who: Who::Everyone,
            query: String::new(),
            filter_focused: false,
            edit: Default::default(),
            folded: FOLDED_BY_DEFAULT.into_iter().collect(),
            selected: 0,
            scroll: 0,
            list_height: 10,
            follow: true,
            bar: None,
            bar_drag: Default::default(),
            scope_combo: Combo::new(""),
            view_combo: Combo::new(""),
            who_combo: Combo::new(""),
            snooze_combo: Combo::new(msg::radar::SNOOZE_PLACEHOLDER.get()).with_click_picks(),
            snooze_for: None,
            menu: None,
            menu_rects: Vec::new(),
            menu_hover: None,
            hits: Vec::new(),
            hover: None,
            hover_since: None,
            hover_pos: (0, 0),
            col_pick: None,
            col_drag: None,
            col_eff: [0; 4],
            col_content: [0; 4],
            col_auto: [0; 4],
            repo_max: columns::MAX_WIDTH,
            message: None,
            mode_hint: None,
            asking: None,
            api_review: None,
            api_item: None,
            said: None,
        };
        ui.fill_combos();
        ui.rebuild();
        ui
    }

    // -- the data ---------------------------------------------------------

    /// Asks every repository of the scope again: the cached rows stay on
    /// screen and are replaced as the hosts answer.
    pub(crate) fn refresh(&mut self) {
        // A folder scope not scanned yet: scan first (off this thread), then
        // ask what it found.
        if self.folder_needs_scan() {
            self.start_scan();
            return;
        }
        let jobs = worker::unique_jobs(self.repos.iter().map(|r| (r.path.clone(), r.name.clone())));
        // WIP: the repositories' own state, local and cheap. What the Home
        // tab's probe already learned shows at once.
        for repo in &self.repos {
            if let Some(facts) = crate::repo_probe::cached(&repo.path) {
                self.wips.insert(repo.path.clone(), facts);
            }
        }
        if self.worker.is_some() {
            self.probe = Some(crate::repo_probe::start(
                jobs.iter().map(|j| j.path.clone()).collect(),
            ));
        }
        if let Some(worker) = self.worker.as_mut() {
            let force = std::mem::take(&mut self.force_account);
            self.round = worker.start(jobs, force);
            self.in_flight = true;
            self.started = Some(Instant::now());
        }
        self.rebuild();
    }

    /// The Refresh button, `r` and the palette: a folder scope is scanned
    /// again (a repository cloned since), then everything is asked again.
    pub(crate) fn reload(&mut self) {
        self.force_account = true;
        if scope::is_folder_scope(&self.scope) && self.scope_folder().is_some() {
            self.start_scan();
        } else {
            self.refresh();
        }
    }

    /// The folder of the current folder scope, when it is set and exists.
    fn scope_folder(&self) -> Option<PathBuf> {
        scope::folder_of(self.config_dir.as_deref()?, &self.scope)
    }

    fn folder_needs_scan(&self) -> bool {
        self.scope_folder()
            .is_some_and(|root| scope::cached_repos(&root).is_none())
    }

    fn start_scan(&mut self) {
        let Some(root) = self.scope_folder() else {
            return;
        };
        let (tx, rx) = std::sync::mpsc::channel();
        let scope = self.scope.clone();
        std::thread::spawn(move || {
            let found = scope::scan_and_cache(&root);
            let _ = crate::redraw::NotifySend::send_notify(&tx, (scope, found));
        });
        self.scan = Some(rx);
        self.in_flight = true;
        self.started = Some(Instant::now());
        self.rebuild();
    }

    /// The worker's answers, the probe's facts, and a snooze that ended by
    /// the clock. True when something on screen changed.
    pub(crate) fn poll(&mut self) -> bool {
        let mut changed = self.api_poll() | self.item_poll();
        if let Some(Ok((scope, found))) = self.scan.as_ref().map(|rx| rx.try_recv()) {
            self.scan = None;
            if scope == self.scope {
                self.repos = scope::folder_repos(found);
                self.answers.clear();
                self.wips.clear();
                self.refresh();
                changed = true;
            }
        }
        while let Some(msg) = self.worker.as_ref().and_then(RadarWorker::try_recv) {
            match msg {
                Msg::Answer(answer) if answer.round == self.round => {
                    self.apply_answer(*answer);
                    changed = true;
                }
                Msg::Account(answer) if answer.round == self.round => {
                    self.apply_account(*answer);
                    changed = true;
                }
                Msg::Setup(setup) if setup.round == self.round => {
                    self.apply_setup(*setup);
                    changed = true;
                }
                Msg::Done(round) if round == self.round => {
                    self.in_flight = false;
                    changed = true;
                }
                _ => {}
            }
        }
        if let Some(probe) = &self.probe {
            for (path, facts) in probe.drain() {
                self.wips.insert(path, facts);
                changed = true;
            }
        }
        let now = (self.now)();
        if changed
            || self
                .state
                .snoozes
                .iter()
                .any(|s| s.until.is_some_and(|u| u > self.built_at && u <= now))
        {
            self.rebuild();
            changed = true;
        }
        changed
    }

    /// Folds one answer in. A failed or paused one keeps the lists the
    /// cache (or an earlier answer) had: stale beats blank.
    fn apply_answer(&mut self, mut new: RepoAnswer) {
        let old = self.answers.iter().position(|a| a.path == new.path);
        if let Some(i) = old {
            let prev = &self.answers[i];
            if new.cached && !prev.cached {
                return;
            }
            let failed = new.snapshot.problem.is_some() || new.paused.is_some();
            if failed {
                if new.snapshot.prs.is_empty() {
                    new.snapshot.prs = prev.snapshot.prs.clone();
                    new.snapshot.can_pulls |= prev.snapshot.can_pulls;
                }
                new.snapshot.fetched_at = prev.snapshot.fetched_at;
            }
            if new.snapshot.issues_problem.is_some() || new.paused.is_some() {
                if new.snapshot.issues.is_empty() {
                    new.snapshot.issues = prev.snapshot.issues.clone();
                }
                new.snapshot.fetched_at = prev.snapshot.fetched_at;
            }
            if new.host.is_empty() {
                new.host = prev.host.clone();
            }
            if new.me.is_empty() {
                new.me = prev.me.clone();
            }
            self.answers[i] = new;
        } else {
            self.answers.push(new);
        }
    }

    /// Recomputes the board, the notes and the rows after any change.
    fn rebuild(&mut self) {
        // The selection's key is read while rows and board still agree.
        let keep = self.selected_key();
        let now = (self.now)();
        self.built_at = now;
        let order = |path: &PathBuf| {
            self.repos
                .iter()
                .position(|r| &r.path == path)
                .unwrap_or(usize::MAX)
        };
        let mut answers: Vec<RepoAnswer> = self
            .answers
            .iter()
            .filter(|a| self.repos.iter().any(|r| r.path == a.path))
            .cloned()
            .collect();
        answers.sort_by_key(|a| order(&a.path));
        let wips: Vec<(PathBuf, String, WipFacts)> = self
            .repos
            .iter()
            .filter_map(|r| {
                let facts = self.wips.get(&r.path)?;
                Some((r.path.clone(), r.name.clone(), WipFacts::of(facts)?))
            })
            .collect();
        self.board = Board::build(&BoardInput {
            answers: &answers,
            accounts: &self.account_answers,
            clones: &self.clones,
            wips: &wips,
            state: &self.state,
            now,
        });
        self.notes = answers
            .iter()
            .filter_map(|a| note_of(a, now, scope::is_folder_scope(&self.scope)))
            .chain(account::notes(&self.account_answers, now))
            .collect();
        self.relayout_keeping(keep);
    }

    /// Rows from the board and the view's filters; keeps the selection on
    /// the same item when it is still listed.
    fn relayout(&mut self) {
        let keep = self.selected_key();
        self.relayout_keeping(keep);
    }

    /// `relayout` with the selection's key read by the caller, before it
    /// replaced the board: `rows` index the board, so they are only read
    /// while both are of the same generation.
    fn relayout_keeping(&mut self, keep: Option<ItemKey>) {
        self.rows = model::rows(
            &self.board,
            self.view,
            self.who,
            &self.query,
            &self.folded,
            self.notes.len(),
            self.offers.len(),
        );
        if let Some(key) = keep {
            if let Some(i) = self
                .rows
                .iter()
                .position(|r| matches!(r, Row::Item(i) if self.board.items.get(*i).is_some_and(|it| it.key == key)))
            {
                self.selected = i;
            }
        }
        self.selected = self.selected.min(self.rows.len().saturating_sub(1));
        if matches!(self.rows.get(self.selected), Some(Row::Blank)) {
            self.selected += 1;
        }
        self.follow = true;
    }

    fn selected_key(&self) -> Option<ItemKey> {
        match self.rows.get(self.selected)? {
            Row::Item(i) => Some(self.board.items.get(*i)?.key.clone()),
            _ => None,
        }
    }

    fn selected_item(&self) -> Option<usize> {
        match self.rows.get(self.selected)? {
            Row::Item(i) => Some(*i),
            _ => None,
        }
    }

    fn save_state(&self) {
        if let Some(path) = &self.state_path {
            let _ = self.state.save(path);
        }
    }

    // -- the comboboxes -----------------------------------------------------

    fn fill_combos(&mut self) {
        let mut scope = vec![ComboOption::new(msg::radar::SCOPE_TABS.get(), OPEN_TABS)];
        scope.extend(
            self.scopes
                .iter()
                .map(|s| ComboOption::new(s.name.clone(), s.id.clone())),
        );
        // The two folders of Settings: greyed while one is not set, and
        // picking it then opens its Settings row.
        for (id, name) in [
            (CLONE_FOLDER, msg::radar::SCOPE_CLONE.get()),
            (BROWSE_FOLDER, msg::radar::SCOPE_BROWSE.get()),
        ] {
            let set = self
                .config_dir
                .as_deref()
                .is_some_and(|d| scope::folder_of(d, id).is_some());
            scope.push(if set {
                ComboOption::new(name, id)
            } else {
                ComboOption::new(msg::radar::scope_unset(name), id).greyed()
            });
        }
        self.scope_combo.set_options(scope);
        self.scope_combo.select(Some(self.scope.clone()));
        self.view_combo.set_options(
            View::ALL
                .iter()
                .map(|v| ComboOption::new(view_label(*v), v.key()))
                .collect(),
        );
        self.view_combo.select(Some(self.view.key().to_string()));
        self.who_combo.set_options(
            Who::ALL
                .iter()
                .map(|w| ComboOption::new(who_label(*w), w.key()))
                .collect(),
        );
        self.who_combo.select(Some(self.who.key().to_string()));
        self.snooze_combo.set_options(
            SnoozeFor::ALL
                .iter()
                .map(|s| ComboOption::new(snooze_label(*s), snooze_key(*s)))
                .collect(),
        );
    }

    /// Repositories of the answers that are not linked to a host: in a folder
    /// scope they only contribute their work in progress.
    fn unlinked_count(&self) -> u64 {
        self.answers
            .iter()
            .filter(|a| !a.linked() && a.snapshot.problem.is_none() && a.paused.is_none())
            .count() as u64
    }

    fn any_combo_open(&self) -> bool {
        self.scope_combo.is_open()
            || self.view_combo.is_open()
            || self.who_combo.is_open()
            || self.snooze_combo.is_open()
    }

    fn close_popups(&mut self) {
        self.scope_combo.close();
        self.view_combo.close();
        self.who_combo.close();
        self.snooze_combo.close();
        self.menu = None;
        self.menu_hover = None;
        self.snooze_for = None;
    }

    /// A pick in one of the four comboboxes.
    fn picked(&mut self, which: Hit, value: &str) -> Outcome {
        match which {
            Hit::Scope => {
                // An unset folder: nothing to cover; send the owner to the row.
                if scope::is_folder_scope(value) {
                    let unset = self
                        .config_dir
                        .as_deref()
                        .is_none_or(|d| scope::folder_of(d, value).is_none());
                    if unset {
                        self.scope_combo.select(Some(self.scope.clone()));
                        return Outcome::Setting(if value == CLONE_FOLDER {
                            "repos.clone_root"
                        } else {
                            "repos.browse_root"
                        });
                    }
                }
                self.scope = value.to_string();
                self.scope_combo.select(Some(self.scope.clone()));
                self.state.scope = Some(self.scope.clone());
                self.save_state();
                if let Some(dir) = &self.config_dir {
                    self.repos = scope::repos_in(dir, &self.session, &self.scope);
                }
                self.answers.clear();
                self.wips.clear();
                self.refresh();
            }
            Hit::View => {
                if let Some(v) = View::from_key(value) {
                    self.view = v;
                    self.view_combo.select(Some(value.to_string()));
                    self.selected = 0;
                    self.relayout();
                }
            }
            Hit::Who => {
                if let Some(w) = Who::from_key(value) {
                    self.who = w;
                    self.who_combo.select(Some(value.to_string()));
                    self.selected = 0;
                    self.relayout();
                }
            }
            _ => {
                if let (Some(i), Some(how)) = (self.snooze_for.take(), snooze_from_key(value)) {
                    return self.snooze(i, how);
                }
            }
        }
        Outcome::None
    }

    fn combo_of(&mut self, which: Hit) -> &mut Combo {
        match which {
            Hit::Scope => &mut self.scope_combo,
            Hit::View => &mut self.view_combo,
            Hit::Who => &mut self.who_combo,
            _ => &mut self.snooze_combo,
        }
    }

    /// Opens one of the header comboboxes (closing the others).
    pub(crate) fn open_combo(&mut self, which: Hit) {
        self.close_popups();
        // Settings may have changed a folder since the options were made.
        self.fill_combos();
        self.filter_focused = false;
        self.combo_of(which).open();
    }

    // -- actions ------------------------------------------------------------

    fn describe(item: &Item) -> String {
        match item.kind {
            ItemKind::Wip => item.repo_name.clone(),
            _ => format!("{} {}", item.label, item.title),
        }
    }

    /// Runs `act` on the selected row. The one place a key, a menu row, a
    /// button and a palette command all arrive at.
    pub(crate) fn run(&mut self, act: Act) -> Outcome {
        self.message = None;
        // The header menu's rows act on the columns, not on a row.
        match act {
            Act::FitColumn(col) => return self.fit_column(col.cell()),
            Act::FitAllColumns => return self.fit_all_columns(),
            _ => {}
        }
        // A note's own actions.
        if let Some(Row::Note(i)) = self.rows.get(self.selected) {
            let Some(note) = self.notes.get(*i).cloned() else {
                return Outcome::None;
            };
            return match (act, note.act) {
                (Act::Link | Act::Reconnect, Some((Act::Link, _))) => {
                    Outcome::OpenRepo(note.path, Some(Pending::Link))
                }
                (Act::Link | Act::Reconnect, Some((Act::Reconnect, profile))) => {
                    Outcome::Integrations(profile)
                }
                (Act::OpenRepo, _) if !note.path.as_os_str().is_empty() => {
                    Outcome::OpenRepo(note.path, None)
                }
                _ => Outcome::None,
            };
        }
        if act == Act::TurnOn {
            let offer = match self.rows.get(self.selected) {
                Some(Row::Hint(i)) => *i,
                _ => 0,
            };
            return self.turn_on(offer);
        }
        if let Some(Row::Hint(i)) = self.rows.get(self.selected) {
            let offer = *i;
            return match act {
                Act::Link | Act::OpenRepo => self.turn_on(offer),
                _ => Outcome::None,
            };
        }
        let Some(index) = self.selected_item() else {
            return self.say(msg::radar::MSG_SELECT_ROW.get().to_string());
        };
        let Some(item) = self.board.items.get(index).cloned() else {
            return Outcome::None;
        };
        match act {
            Act::OpenBrowser => {
                if item.url.is_empty() {
                    return self.say(msg::radar::MSG_NO_PAGE.get().to_string());
                }
                match crate::repo_menu::open_url(&item.url) {
                    Ok(()) => self.say(msg::hosting_ui::opened_in_browser(&item.url)),
                    Err(err) => self.say(err),
                }
            }
            Act::OpenRepo => self.open_repo(&item, None),
            Act::Checkout => {
                if item.kind != ItemKind::Pr {
                    return self.say(msg::radar::MSG_NOT_PR.get().to_string());
                }
                self.open_repo(
                    &item,
                    Some(Pending::CheckoutPr {
                        number: item.number,
                    }),
                )
            }
            Act::Worktree => match (&item.kind, &item.branch) {
                (ItemKind::Pr, Some(branch)) => self.open_repo(
                    &item,
                    Some(Pending::Worktree {
                        branch: branch.clone(),
                    }),
                ),
                _ => self.say(msg::radar::MSG_NOT_PR.get().to_string()),
            },
            Act::Show => match item.kind {
                ItemKind::Wip => self.open_repo(&item, None),
                kind => self.open_repo(
                    &item,
                    Some(Pending::Show {
                        pr: kind == ItemKind::Pr,
                        number: item.number,
                    }),
                ),
            },
            Act::StartWork => {
                if item.kind != ItemKind::Issue {
                    return self.say(msg::radar::MSG_ONLY_ISSUE.get().to_string());
                }
                let branch = crate::hosting_ui::issue_branch_name(item.number, &item.title);
                self.open_repo(&item, Some(Pending::StartWork { branch }))
            }
            Act::Pin => {
                let pinned = self.state.toggle_pin(&item.key);
                self.save_state();
                self.rebuild();
                let what = Self::describe(&item);
                self.say(if pinned {
                    msg::radar::msg_pinned(what)
                } else {
                    msg::radar::msg_unpinned(what)
                })
            }
            Act::Snooze => {
                self.snooze_for = Some(index);
                let anchor = self
                    .hits
                    .iter()
                    .find(|(_, h)| *h == Hit::Row(self.selected))
                    .map(|(r, _)| Rect::new(r.x + 4, r.y, 22, 1))
                    .unwrap_or(Rect::new(4, 6, 22, 1));
                self.close_popups_keep_snooze();
                self.snooze_combo.set_anchor(anchor);
                self.snooze_combo.open();
                Outcome::None
            }
            Act::Unsnooze => {
                self.state.unsnooze(&item.key);
                self.save_state();
                self.rebuild();
                self.say(msg::radar::msg_unsnoozed(Self::describe(&item)))
            }
            Act::EditAssignees | Act::EditReviewers | Act::EditLabels | Act::SetMilestone => {
                if act == Act::EditAssignees && Self::assignment_blocked(&item) {
                    return self.say(
                        gitpoulpi_i18n::msg::hosting_ui::WHY_ASSIGNMENT_BLOCKED
                            .get()
                            .to_string(),
                    );
                }
                match item.kind {
                    ItemKind::Wip => self.say(msg::radar::MSG_SELECT_ROW.get().to_string()),
                    _ if !item.meta => self.say(msg::meta_picker::UNSUPPORTED.get().to_string()),
                    ItemKind::Issue if act == Act::EditReviewers => {
                        self.say(msg::radar::MSG_NOT_PR.get().to_string())
                    }
                    kind => self.open_repo(
                        &item,
                        Some(Pending::EditMeta {
                            pr: kind == ItemKind::Pr,
                            number: item.number,
                            act,
                        }),
                    ),
                }
            }
            Act::Conversation => match item.kind {
                // Not cloned: the view needs only the profile and the
                // project, so the Radar hosts it (`item_host`).
                ItemKind::Issue | ItemKind::Pr
                    if crate::build_features::CONVERSATION && item.repo.is_none() =>
                {
                    self.conversation_without_clone(&item)
                }
                ItemKind::Issue if crate::build_features::CONVERSATION => self.open_repo(
                    &item,
                    Some(Pending::Conversation {
                        number: item.number,
                    }),
                ),
                _ => self.say(
                    gitpoulpi_i18n::msg::item_view::WHY_NO_CONVERSATION
                        .get()
                        .to_string(),
                ),
            },
            Act::Review | Act::ReviewApi | Act::CloneReview => self.review(item, act),
            Act::Link | Act::Reconnect | Act::TurnOn | Act::FitColumn(_) | Act::FitAllColumns => {
                Outcome::None
            }
        }
    }

    fn close_popups_keep_snooze(&mut self) {
        let keep = self.snooze_for;
        self.close_popups();
        self.snooze_for = keep;
    }

    /// Whether the item's repository blocks assignment.
    fn assignment_blocked(item: &Item) -> bool {
        item.repo
            .as_deref()
            .is_some_and(gitpoulpi_daemon::hosting_scope::assignment_blocked)
    }

    fn open_repo(&mut self, item: &Item, pending: Option<Pending>) -> Outcome {
        let Some(repo) = item.repo.as_ref() else {
            return self.say(msg::radar::MSG_CLONE_FIRST.get().to_string());
        };
        if !repo.exists() {
            return self.say(msg::radar::msg_not_a_repo(repo.display()));
        }
        Outcome::OpenRepo(repo.clone(), pending)
    }

    fn snooze(&mut self, index: usize, how: SnoozeFor) -> Outcome {
        let Some(item) = self.board.items.get(index).cloned() else {
            return Outcome::None;
        };
        self.state
            .snooze(&item.key, how, (self.now)(), &item.stamp());
        self.save_state();
        self.rebuild();
        self.say(msg::radar::msg_snoozed(
            Self::describe(&item),
            snooze_label(how),
        ))
    }

    fn say(&mut self, text: String) -> Outcome {
        self.message = Some(text.clone());
        Outcome::Message(text)
    }

    /// The menu of the selected row, anchored at `(col, row)`.
    pub(crate) fn open_menu(&mut self, col: u16, row: u16) {
        let entries = match self.rows.get(self.selected) {
            Some(Row::Item(i)) => self.item_entries(*i),
            Some(Row::Note(i)) => {
                let note = self.notes.get(*i);
                let own = note.and_then(|n| n.act.clone());
                let has_repo = note.is_some_and(|n| !n.path.as_os_str().is_empty());
                let mut entries: Vec<Entry<Act>> = own
                    .into_iter()
                    .map(|(act, _)| Entry::action(act_label(act), act))
                    .collect();
                if has_repo {
                    entries.push(Entry::action(act_label(Act::OpenRepo), Act::OpenRepo));
                }
                if entries.is_empty() {
                    return;
                }
                entries
            }
            Some(Row::Hint(_)) => vec![Entry::action(act_label(Act::TurnOn), Act::TurnOn)],
            _ => return,
        };
        self.close_popups();
        let caption = match self.rows.get(self.selected) {
            Some(Row::Item(i)) => self
                .board
                .items
                .get(*i)
                .map(Self::describe)
                .unwrap_or_default(),
            Some(Row::Note(i)) => self.notes[*i].name.clone(),
            Some(Row::Hint(i)) => self.offers[*i].label.clone(),
            _ => String::new(),
        };
        self.menu = Some(Menu::new(caption, entries, col, row));
    }

    fn item_entries(&self, index: usize) -> Vec<Entry<Act>> {
        let Some(item) = self.board.items.get(index) else {
            return Vec::new();
        };
        let pinned = self.state.is_pinned(&item.key);
        let snoozed = self.board.snoozed.contains(&index);
        let mut out = Vec::new();
        if !item.url.is_empty() {
            out.push(Entry::action(act_label(Act::OpenBrowser), Act::OpenBrowser));
        }
        // Not cloned: only what needs no repository (the keys still answer
        // "clone it first").
        let cloned = item.repo.is_some();
        if cloned {
            out.push(Entry::action(act_label(Act::OpenRepo), Act::OpenRepo));
        }
        match item.kind {
            ItemKind::Pr => {
                if item.review && crate::build_features::REVIEW {
                    out.push(Entry::action(act_label(Act::Review), Act::Review));
                }
                if cloned {
                    out.push(Entry::action(act_label(Act::Checkout), Act::Checkout));
                    out.push(Entry::action(act_label(Act::Worktree), Act::Worktree));
                    out.push(Entry::action(act_label(Act::Show), Act::Show));
                }
            }
            ItemKind::Issue if cloned => {
                if crate::build_features::CONVERSATION {
                    out.push(Entry::action(
                        act_label(Act::Conversation),
                        Act::Conversation,
                    ));
                }
                out.push(Entry::action(act_label(Act::Show), Act::Show));
                out.push(Entry::action(act_label(Act::StartWork), Act::StartWork));
            }
            ItemKind::Issue | ItemKind::Wip => {}
        }
        // Not cloned: the conversation needs no checkout (`item_host`).
        if !cloned
            && crate::build_features::CONVERSATION
            && !item.integration.is_empty()
            && matches!(item.kind, ItemKind::Issue | ItemKind::Pr)
        {
            out.push(Entry::action(
                act_label(Act::Conversation),
                Act::Conversation,
            ));
        }
        if item.meta && cloned {
            out.push(Entry::Separator);
            let acts: &[Act] = if item.kind == ItemKind::Pr {
                &[
                    Act::EditAssignees,
                    Act::EditReviewers,
                    Act::EditLabels,
                    Act::SetMilestone,
                ]
            } else {
                &[Act::EditAssignees, Act::EditLabels, Act::SetMilestone]
            };
            let blocked = Self::assignment_blocked(item);
            out.extend(
                acts.iter()
                    .filter(|a| !(blocked && **a == Act::EditAssignees))
                    .map(|a| Entry::action(act_label(*a), *a)),
            );
        }
        out.push(Entry::Separator);
        out.push(Entry::action(
            if pinned {
                msg::radar::ACT_UNPIN.get()
            } else {
                act_label(Act::Pin)
            },
            Act::Pin,
        ));
        if snoozed {
            out.push(Entry::action(act_label(Act::Unsnooze), Act::Unsnooze));
        } else {
            out.push(Entry::action(act_label(Act::Snooze), Act::Snooze));
        }
        out
    }

    // -- facts for the tests and the palette ----------------------------------

    pub(crate) fn has_row_selected(&self) -> bool {
        matches!(self.rows.get(self.selected), Some(Row::Item(_)))
    }
}

#[cfg(test)]
mod tests;
