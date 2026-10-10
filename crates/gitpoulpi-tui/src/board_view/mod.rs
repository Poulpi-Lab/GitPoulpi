//! The Kanban board of an issue source (slice L2): columns by status, list
//! or label, cards you move with the mouse (drag) or the keyboard
//! (`Shift+Left/Right`, "Move card to..."), filters, live refresh.
//!
//! Which columns a source has and what a move changes is
//! `gitpoulpi_hosting::board` (pure, shared with whatever frontend comes
//! next); this module is the view. It is a full-window modal owned by
//! `RepoDialogs` (`RepoDialog::Board`), like the review view: keys, mouse,
//! paste and the per-frame poll all arrive there, so popups and hit rects
//! follow the same rules as the other dialogs. It never waits: the load and
//! every move run on a worker, a move shows on the board at once (optimistic),
//! carries a spinner after 150 ms, and snaps back with the error when the
//! provider refuses it.
//!
//! The cards come from a load of their own (every state, the declared
//! columns); the sidebar's refreshes (a notes folder changed, the host's
//! cadence) bump `hosting_ui::issues_generation`, which makes the board read
//! again.

mod draw;
mod input;
mod load;
mod prompt;
#[cfg(test)]
mod tests;

use std::path::PathBuf;
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

use gitpoulpi_hosting::board::{
    self, BoardColumn, BoardFilter, BoardSource, BoardSpec, ColumnRole, FilterChoices, MovePlan,
};
use gitpoulpi_hosting::{Integration, IssueSummary, MetaKind};
use gitpoulpi_i18n::msg::board as msg;
use ratatui::layout::Rect;

use crate::board_cmd::BoardAvail;
use crate::combo::{Combo, ComboOption};
use crate::hosting_task::ClientFactory;
use crate::issue_form::IssueSeed;
use crate::meta_picker::{MetaEdit, MetaEditOutcome, MetaField, MetaTarget};
use crate::text_edit::FieldEditor;
use load::Loaded;
use prompt::LabelsPrompt;

/// A move in flight for longer than this shows a spinner on its card (maintainer
/// rule 9).
pub(super) const SPIN_AFTER: Duration = Duration::from_millis(150);

/// What the board opens on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BoardTarget {
    /// The provider profile, its display name and the project (an absolute
    /// folder for a local source): also what the New issue form opens with.
    pub(crate) seed: IssueSeed,
    /// Every configured profile (the token source reads them all).
    pub(crate) all: Vec<Integration>,
    /// The ISSUES section's extra query (Jira JQL, Azure WIQL, Trello lists).
    pub(crate) query: Option<String>,
    /// The repository the work tree belongs to (the saved label set lives in
    /// its state file).
    pub(crate) repo_root: PathBuf,
    /// The labels that are the columns of a host board, in order.
    pub(crate) labels: Vec<String>,
}

impl BoardTarget {
    /// What the title and the info line call the source: the project, or
    /// for a notes folder its last two path components.
    pub(crate) fn project_label(&self) -> String {
        if self.seed.integration.kind.capabilities().local_folder {
            let path = std::path::Path::new(&self.seed.project);
            let parts: Vec<String> = path
                .components()
                .filter_map(|c| match c {
                    std::path::Component::Normal(s) => Some(s.to_string_lossy().into_owned()),
                    _ => None,
                })
                .collect();
            let from = parts.len().saturating_sub(2);
            parts[from..].join("/")
        } else {
            self.seed.project.clone()
        }
    }
}

/// The column a New issue form should start in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NewIn {
    /// The status / list the new issue starts in (local Markdown, Trello).
    pub(crate) column: Option<String>,
    /// The label to add (a host board's status label).
    pub(crate) label: Option<String>,
}

/// What the owner of the view has to do after it handled something.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BoardEffect {
    None,
    /// Leave the view.
    Close,
    Message(String),
    /// Hand this URL to the system (a note, or the issue's page).
    OpenUrl(String),
    /// Park the board and open the New issue form for it.
    NewIssue(Box<IssueSeed>, NewIn),
}

/// Which filter combobox.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FilterKind {
    Assignee,
    Label,
    Milestone,
}

impl FilterKind {
    pub(super) const ALL: [FilterKind; 3] = [
        FilterKind::Assignee,
        FilterKind::Label,
        FilterKind::Milestone,
    ];

    fn index(self) -> usize {
        match self {
            FilterKind::Assignee => 0,
            FilterKind::Label => 1,
            FilterKind::Milestone => 2,
        }
    }

    fn placeholder(self) -> &'static str {
        match self {
            FilterKind::Assignee => msg::ALL_ASSIGNEES.get(),
            FilterKind::Label => msg::ALL_LABELS.get(),
            FilterKind::Milestone => msg::ALL_MILESTONES.get(),
        }
    }
}

/// What the small card menu is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MenuKind {
    /// The whole card menu (right click, `.`).
    Card,
    /// Only the metadata edits (`e`).
    Edit,
}

/// A move waiting for the provider.
pub(super) struct PendingMove {
    pub(super) number: u64,
    /// The column the card shows in meanwhile.
    pub(super) to: usize,
    pub(super) caption: String,
    pub(super) column_title: String,
    pub(super) plan: MovePlan,
    pub(super) since: Instant,
    rx: mpsc::Receiver<Result<(), String>>,
    /// A further column asked for while this move was in flight (key
    /// repeat): sent once this one landed.
    queued: Option<usize>,
}

/// A card being dragged.
#[derive(Debug, Clone, Copy)]
pub(super) struct Drag {
    pub(super) number: u64,
    pub(super) from: usize,
    pub(super) start: (u16, u16),
    /// The pointer left the cell it went down on.
    pub(super) moved: bool,
    /// The column under the pointer.
    pub(super) over: Option<usize>,
}

pub(crate) struct BoardView {
    pub(super) target: BoardTarget,
    factory: Arc<dyn ClientFactory>,
    started: Instant,

    // what the provider said
    pub(super) issues: Vec<IssueSummary>,
    pub(super) spec: BoardSpec,
    pub(super) columns: Vec<BoardColumn>,
    pub(super) loaded: bool,
    load_rx: Option<mpsc::Receiver<Result<Loaded, String>>>,
    pub(super) load_err: Option<String>,
    seen_generation: u64,

    // what is shown: issue indexes per column, after the filters
    pub(super) cards: Vec<Vec<usize>>,
    pub(super) filter: BoardFilter,
    pub(super) choices: FilterChoices,
    pub(super) combos: [Combo; 3],
    pub(super) search: String,
    pub(super) searching: bool,
    pub(super) search_edit: FieldEditor<()>,

    // selection and scrolling
    pub(super) sel_col: usize,
    pub(super) sel_number: Option<u64>,
    pub(super) first_col: usize,
    /// Columns / cards the last paint fit.
    pub(super) visible_cols: usize,
    pub(super) rows_per_col: usize,
    pub(super) offsets: Vec<usize>,
    pub(super) hbar: Option<crate::scrollbar::Track>,
    pub(super) hgrab: Option<i32>,
    /// Where the columns were painted, and the selected card's rect.
    pub(super) body_rect: Rect,
    /// A column and its gap, in cells (the horizontal bar's unit).
    pub(super) col_stride: u16,
    pub(super) sel_rect: Option<Rect>,
    /// When the drag last scrolled the columns at an edge.
    pub(super) edge_scroll: Option<Instant>,

    // moves
    pub(super) moves: Vec<PendingMove>,
    pub(super) drag: Option<Drag>,

    // popups
    pub(super) menu: Option<(MenuKind, Combo)>,
    pub(super) move_combo: Option<Combo>,
    meta: Option<MetaEdit>,
    prompt: Option<LabelsPrompt>,

    pub(super) message: Option<String>,
    pub(super) hits: Vec<(Rect, draw::Hit)>,
    pub(super) hover: Option<draw::Hit>,
    pub(super) last_click: Option<(Instant, u16, u16)>,
}

impl BoardView {
    /// Opens the view and starts the load.
    pub(crate) fn open(target: BoardTarget, factory: Arc<dyn ClientFactory>) -> Self {
        let source = if target.seed.integration.kind.capabilities().local_folder {
            BoardSource::LocalStatus
        } else {
            BoardSource::OpenClosed
        };
        let spec = BoardSpec {
            source,
            declared: Vec::new(),
            labels: target.labels.clone(),
        };
        let mut view = Self {
            target,
            factory,
            started: Instant::now(),
            issues: Vec::new(),
            spec,
            columns: Vec::new(),
            loaded: false,
            load_rx: None,
            load_err: None,
            seen_generation: crate::hosting_ui::issues_generation(),
            cards: Vec::new(),
            filter: BoardFilter::default(),
            choices: FilterChoices::default(),
            combos: FilterKind::ALL.map(|k| Combo::new(k.placeholder()).with_search()),
            search: String::new(),
            searching: false,
            search_edit: FieldEditor::default(),
            sel_col: 0,
            sel_number: None,
            first_col: 0,
            visible_cols: 3,
            rows_per_col: 4,
            offsets: Vec::new(),
            hbar: None,
            hgrab: None,
            body_rect: Rect::default(),
            col_stride: 1,
            sel_rect: None,
            edge_scroll: None,
            moves: Vec::new(),
            drag: None,
            menu: None,
            move_combo: None,
            meta: None,
            prompt: None,
            message: None,
            hits: Vec::new(),
            hover: None,
            last_click: None,
        };
        view.rebuild();
        view.start_load();
        view
    }

    fn start_load(&mut self) {
        self.started = Instant::now();
        self.load_err = None;
        self.seen_generation = crate::hosting_ui::issues_generation();
        self.load_rx = Some(load::start_load(self.factory.clone(), &self.target));
    }

    /// Reads the cards again (the notes folder or the host).
    pub(crate) fn refresh(&mut self) {
        if self.load_rx.is_none() {
            self.start_load();
        }
    }

    /// Something is moving: the spinner needs frames.
    pub(crate) fn busy(&self) -> bool {
        self.load_rx.is_some()
            || !self.moves.is_empty()
            || self.meta.as_ref().is_some_and(MetaEdit::busy)
    }

    pub(super) fn spinner(&self) -> char {
        crate::activity::spinner_frame(self.started.elapsed())
    }

    /// Whether the source can move a card at all.
    pub(super) fn can_move(&self) -> bool {
        self.spec.source.movable()
    }

    pub(super) fn can_edit_meta(&self) -> bool {
        crate::meta_picker::supports(self.target.seed.integration.kind)
    }

    pub(super) fn label_columns(&self) -> bool {
        self.spec.source == BoardSource::Labels
    }

    // -- the model ---------------------------------------------------------------

    /// The title a column is shown with (the two fixed ones are translated).
    pub(super) fn column_title(&self, col: usize) -> String {
        match self.columns.get(col) {
            Some(c) if c.role == ColumnRole::Open => msg::COLUMN_OPEN.get().to_string(),
            Some(c) if c.role == ColumnRole::Closed => msg::COLUMN_CLOSED.get().to_string(),
            Some(c) => c.name.clone(),
            None => String::new(),
        }
    }

    /// The column `issue` shows in: where a move in flight put it, else its
    /// own.
    fn column_of(&self, issue: &IssueSummary) -> usize {
        match self.moves.iter().find(|m| m.number == issue.number) {
            Some(m) => m.to.min(self.columns.len().saturating_sub(1)),
            None => board::column_index(&self.spec, &self.columns, issue),
        }
    }

    /// Lays the cards out in their columns again (after a load, a move, a
    /// filter change) and keeps the selection on its card when it can.
    pub(super) fn rebuild(&mut self) {
        self.columns = board::columns(&self.spec, &self.issues);
        self.choices = board::filter_choices(&self.issues);
        let mut cards: Vec<Vec<usize>> = vec![Vec::new(); self.columns.len()];
        for (i, issue) in self.issues.iter().enumerate() {
            if self.filter.matches(issue) {
                let col = self.column_of(issue);
                if let Some(slot) = cards.get_mut(col) {
                    slot.push(i);
                }
            }
        }
        self.cards = cards;
        self.offsets.resize(self.columns.len(), 0);
        self.sel_col = self.sel_col.min(self.columns.len().saturating_sub(1));
        // Follow the selected card to its column; else keep the column.
        if let Some(number) = self.sel_number {
            let found = self
                .cards
                .iter()
                .position(|col| col.iter().any(|&i| self.issues[i].number == number));
            match found {
                Some(col) => self.sel_col = col,
                None => self.sel_number = None,
            }
        }
        if self.sel_number.is_none() {
            self.sel_number = self
                .cards
                .get(self.sel_col)
                .and_then(|col| col.first().map(|&i| self.issues[i].number));
        }
        self.sync_combos();
        self.ensure_visible();
    }

    /// Fills the filter dropdowns with the values the cards carry.
    fn sync_combos(&mut self) {
        let lists = [
            (&self.choices.assignees, self.filter.assignee.clone()),
            (&self.choices.labels, self.filter.label.clone()),
            (&self.choices.milestones, self.filter.milestone.clone()),
        ];
        for (kind, (values, current)) in FilterKind::ALL.into_iter().zip(lists) {
            let combo = &mut self.combos[kind.index()];
            let mut options = vec![ComboOption::new(kind.placeholder(), "")];
            options.extend(values.iter().map(|v| ComboOption::plain(v.clone())));
            combo.set_options(options);
            if !combo.is_open() {
                combo.select(Some(current.unwrap_or_default()));
            }
        }
    }

    pub(super) fn selected_issue(&self) -> Option<&IssueSummary> {
        let number = self.sel_number?;
        self.issues.iter().find(|i| i.number == number)
    }

    /// The row of the selected card in its column.
    pub(super) fn sel_row(&self) -> Option<usize> {
        let number = self.sel_number?;
        self.cards
            .get(self.sel_col)?
            .iter()
            .position(|&i| self.issues[i].number == number)
    }

    pub(super) fn select(&mut self, col: usize, row: Option<usize>) {
        self.sel_col = col.min(self.columns.len().saturating_sub(1));
        self.sel_number = row
            .and_then(|r| self.cards.get(self.sel_col)?.get(r).copied())
            .map(|i| self.issues[i].number);
        self.ensure_visible();
    }

    /// Scrolls so the selected card and its column are on screen.
    pub(super) fn ensure_visible(&mut self) {
        let n = self.columns.len();
        let vis = self.visible_cols.max(1);
        if self.sel_col < self.first_col {
            self.first_col = self.sel_col;
        } else if self.sel_col >= self.first_col + vis {
            self.first_col = self.sel_col + 1 - vis;
        }
        self.first_col = self.first_col.min(n.saturating_sub(vis));
        if let Some(row) = self.sel_row() {
            let rows = self.rows_per_col.max(1);
            let off = self.offsets.get_mut(self.sel_col);
            if let Some(off) = off {
                if row < *off {
                    *off = row;
                } else if row >= *off + rows {
                    *off = row + 1 - rows;
                }
            }
        }
    }

    pub(super) fn set_filter(&mut self, kind: FilterKind, value: Option<String>) {
        match kind {
            FilterKind::Assignee => self.filter.assignee = value,
            FilterKind::Label => self.filter.label = value,
            FilterKind::Milestone => self.filter.milestone = value,
        }
        self.offsets.fill(0);
        self.rebuild();
    }

    pub(super) fn clear_filters(&mut self) {
        self.filter = BoardFilter::default();
        self.search.clear();
        self.searching = false;
        self.offsets.fill(0);
        self.rebuild();
    }

    // -- polling -------------------------------------------------------------------

    /// Drains every worker. Called once per frame by the dialog host.
    pub(crate) fn poll(&mut self) -> BoardEffect {
        let mut effect = BoardEffect::None;
        if let Some(rx) = &self.load_rx {
            match rx.try_recv() {
                Ok(Ok(data)) => {
                    self.load_rx = None;
                    self.load_err = None;
                    self.spec.source = data.source;
                    self.spec.declared = data.statuses;
                    self.issues = data.issues;
                    self.loaded = true;
                    self.rebuild();
                }
                Ok(Err(why)) => {
                    self.load_rx = None;
                    crate::activity_log::report_problem(
                        crate::activity_log::ProblemSource::Hosting,
                        &why,
                    );
                    self.load_err = Some(why);
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.load_rx = None;
                    self.load_err = Some(msg::LOAD_STOPPED.get().to_string());
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        self.poll_moves();
        if let Some(edit) = self.meta.as_mut() {
            if let Some(outcome) = edit.poll() {
                if let Some(e) = self.on_meta(outcome) {
                    effect = e;
                }
            }
        }
        // The sidebar read the issues again (a note changed, the cadence):
        // so does the board, once nothing of its own is in flight.
        if self.loaded
            && self.load_rx.is_none()
            && self.moves.is_empty()
            && crate::hosting_ui::issues_generation() != self.seen_generation
        {
            self.start_load();
        }
        effect
    }

    // -- moves ---------------------------------------------------------------------

    /// Moves card `number` to column `to`: at once on the board, then on the
    /// provider. A refusal puts the card back (`poll_moves`).
    pub(super) fn move_card(&mut self, number: u64, to: usize) {
        // (A move already in flight for the card takes the new column as a
        // follow-up, below.)
        if !self.can_move() {
            self.message = Some(msg::READ_ONLY.get().to_string());
            return;
        }
        let Some(issue) = self.issues.iter().find(|i| i.number == number).cloned() else {
            return;
        };
        let Some(column) = self.columns.get(to).cloned() else {
            return;
        };
        // Asked again while the provider is still busy with this card (key
        // repeat): the board shows the new column now, the call follows.
        if let Some(m) = self.moves.iter_mut().find(|m| m.number == number) {
            m.queued = Some(to);
            m.to = to;
            self.sel_number = Some(number);
            self.rebuild();
            return;
        }
        let plan = match board::plan_move(&self.spec, &issue, &column) {
            Ok(plan) => plan,
            Err(_) => {
                self.message = Some(msg::READ_ONLY.get().to_string());
                return;
            }
        };
        if plan.is_empty() {
            return;
        }
        let caption = format!("{} {}", issue.iid_label, issue.title);
        let column_title = self.column_title(to);
        let rx = load::start_move(
            self.factory.clone(),
            &self.target,
            issue,
            self.spec.clone(),
            column,
        );
        self.moves.push(PendingMove {
            number,
            to,
            caption,
            column_title,
            plan,
            since: Instant::now(),
            rx,
            queued: None,
        });
        self.message = None;
        self.sel_number = Some(number);
        self.rebuild();
    }

    fn poll_moves(&mut self) {
        let mut done: Vec<(usize, Result<(), String>)> = Vec::new();
        for (i, m) in self.moves.iter().enumerate() {
            match m.rx.try_recv() {
                Ok(result) => done.push((i, result)),
                Err(mpsc::TryRecvError::Disconnected) => {
                    done.push((i, Err(msg::LOAD_STOPPED.get().to_string())))
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        if done.is_empty() {
            return;
        }
        let mut followups: Vec<(u64, usize)> = Vec::new();
        for (i, result) in done.into_iter().rev() {
            let m = self.moves.remove(i);
            match result {
                Ok(()) => {
                    if let Some(issue) = self.issues.iter_mut().find(|x| x.number == m.number) {
                        board::apply_plan(issue, &m.plan);
                    }
                    self.message = Some(msg::moved(&m.caption, &m.column_title));
                    if let Some(next) = m.queued {
                        followups.push((m.number, next));
                    }
                    // The sidebar's list shows the new state too.
                    crate::hosting_ui::request_issues_refresh();
                }
                Err(why) => {
                    // The card snaps back: its column is its own again.
                    crate::activity_log::report_problem(
                        crate::activity_log::ProblemSource::Hosting,
                        &why,
                    );
                    self.message = Some(msg::move_failed(&m.caption, &why));
                }
            }
        }
        self.rebuild();
        for (number, to) in followups {
            self.move_card(number, to);
        }
    }

    // -- metadata ------------------------------------------------------------------

    pub(super) fn open_meta(&mut self, field: MetaField) {
        let Some(issue) = self.selected_issue().cloned() else {
            self.message = Some(msg::WHY_NO_CARD.get().to_string());
            return;
        };
        if !self.can_edit_meta()
            || !crate::meta_picker::supports_field(self.target.seed.integration.kind, field)
        {
            self.message = Some(msg::WHY_NO_META.get().to_string());
            return;
        }
        let target = MetaTarget {
            integration: self.target.seed.integration.clone(),
            all: self.target.all.clone(),
            project: self.target.seed.project.clone(),
            kind: MetaKind::Issue,
            number: Some(issue.number),
            caption: format!("{} {}", issue.iid_label, issue.title),
        };
        self.meta = Some(MetaEdit::new(self.factory.clone(), target, field));
    }

    fn on_meta(&mut self, outcome: MetaEditOutcome) -> Option<BoardEffect> {
        match outcome {
            MetaEditOutcome::Stay => None,
            MetaEditOutcome::Cancel => {
                self.meta = None;
                None
            }
            MetaEditOutcome::Saved(message) => {
                self.meta = None;
                crate::hosting_ui::request_issues_refresh();
                if self.load_rx.is_none() {
                    self.start_load();
                }
                Some(BoardEffect::Message(message))
            }
        }
    }

    // -- the column labels --------------------------------------------------------

    pub(super) fn open_columns_prompt(&mut self) {
        let proposal = if self.target.labels.is_empty() {
            board::status_like_labels(&self.choices.labels)
        } else {
            Vec::new()
        };
        self.prompt = Some(LabelsPrompt::new(&self.target.labels, proposal));
    }

    fn save_column_labels(&mut self, labels: Vec<String>) {
        let path = crate::remotes_ui::repo_state_path(&self.target.repo_root);
        let mut state = gitpoulpi_session::RepoState::load(&path).unwrap_or_default();
        state.board_labels = labels.clone();
        if let Err(e) = state.save(&path) {
            self.message = Some(msg::columns_not_saved(e));
            return;
        }
        self.target.labels = labels.clone();
        self.spec.labels = labels;
        self.rebuild();
        self.message = Some(msg::COLUMNS_SAVED.get().to_string());
    }

    // -- the palette's view of the state ---------------------------------------------

    /// No popup is open over the view: the palette may take the keyboard.
    pub(crate) fn takes_palette(&self) -> bool {
        self.meta.is_none()
            && self.prompt.is_none()
            && self.menu.is_none()
            && self.move_combo.is_none()
            && !self.searching
            && !self.combos.iter().any(Combo::is_open)
    }

    pub(crate) fn avail(&self) -> BoardAvail {
        let row = self.sel_row();
        BoardAvail {
            loaded: self.loaded,
            has_card: self.selected_issue().is_some(),
            can_move: self.can_move(),
            has_left: row.is_some() && self.sel_col > 0,
            has_right: row.is_some() && self.sel_col + 1 < self.columns.len(),
            can_edit_meta: self.can_edit_meta(),
            label_columns: self.label_columns(),
            filtered: !self.filter.is_empty(),
        }
    }

    /// The columns of the "Move card to..." dropdown.
    pub(super) fn column_options(&self) -> Vec<ComboOption> {
        (0..self.columns.len())
            .map(|i| ComboOption::new(self.column_title(i), i.to_string()))
            .collect()
    }

    /// What the New issue form should start in for the selected column.
    pub(super) fn new_in(&self) -> NewIn {
        let column = self.columns.get(self.sel_col);
        match (self.spec.source, column) {
            (BoardSource::LocalStatus | BoardSource::Lists, Some(c)) => NewIn {
                column: Some(c.name.clone()),
                label: None,
            },
            (BoardSource::Labels, Some(c)) if c.role == ColumnRole::Named => NewIn {
                column: None,
                label: Some(c.name.clone()),
            },
            _ => NewIn {
                column: None,
                label: None,
            },
        }
    }

    /// The pending move of card `number`, for the card's spinner.
    pub(super) fn pending(&self, number: u64) -> Option<&PendingMove> {
        self.moves.iter().find(|m| m.number == number)
    }
}
