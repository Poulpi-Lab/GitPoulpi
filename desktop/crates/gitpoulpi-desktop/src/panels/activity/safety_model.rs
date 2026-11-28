//! The data behind the undo-safety panel: the three views (Undo history, Reflog, Lost work), their rows
//! and what each row can do. Plain data and pure functions, so the tests pin the rules without a window.
//! The wording of an entry is `gitpoulpi_link::undo_label`, the same text the terminal UI shows.

use gitpoulpi_core::domain::{
    EntrySource, JournalListing, JournalState, ListedEntry, LostCommit, Oid, ReflogEntry,
};
use gitpoulpi_i18n::msg::undo_safety as m;
use gitpoulpi_link::error_text::undo_refusal;
use gitpoulpi_link::undo_label as label;

pub use gitpoulpi_app::palette::vocab::safety_panel::Tab;

/// Everything a row, button or menu entry can do: one list so buttons, keys and the palette agree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Act {
    UndoTo,
    RedoTo,
    /// The newest entries when one sequence run made them.
    UndoSequence,
    RestoreOnly,
    /// Move the branch back to where the blocked action left it.
    Offer,
    ShowInLog,
    CreateBranch,
    RestoreRef,
    CherryPick,
    CreateTag,
    /// The commit's changes onto the working tree.
    ApplyToWorktree,
    /// The commit's changes kept as a stash.
    SaveAsStash,
    /// Reflog: the next page of positions.
    LoadMore,
}

impl Act {
    /// The button's words.
    pub fn label(self, state: &JournalState, refname: &str) -> String {
        match self {
            Act::UndoTo => m::BTN_UNDO_TO.get().to_string(),
            Act::RedoTo => m::BTN_REDO_TO.get().to_string(),
            Act::UndoSequence => m::BTN_UNDO_SEQUENCE.get().to_string(),
            Act::RestoreOnly => m::BTN_RESTORE_ONLY.get().to_string(),
            Act::Offer => match label::offer_of(state) {
                Some((name, _, op)) => m::btn_offer(name, op),
                None => m::MENU_OFFER.get().to_string(),
            },
            Act::ShowInLog => m::BTN_SHOW_IN_LOG.get().to_string(),
            Act::CreateBranch => m::BTN_CREATE_BRANCH.get().to_string(),
            Act::RestoreRef => m::btn_restore_ref(refname),
            Act::CherryPick => m::BTN_CHERRY_PICK.get().to_string(),
            Act::CreateTag => m::BTN_CREATE_TAG.get().to_string(),
            Act::ApplyToWorktree => m::BTN_APPLY.get().to_string(),
            Act::SaveAsStash => m::BTN_STASH.get().to_string(),
            Act::LoadMore => m::BTN_LOAD_MORE.get().to_string(),
        }
    }

    /// The key of the button (one letter, from the catalog).
    pub fn key(self) -> &'static str {
        match self {
            Act::UndoTo => m::KEY_UNDO_TO.get(),
            Act::RedoTo => m::KEY_REDO_TO.get(),
            Act::UndoSequence => m::KEY_UNDO_SEQUENCE.get(),
            Act::RestoreOnly => m::KEY_RESTORE_ONLY.get(),
            Act::Offer => m::KEY_OFFER.get(),
            Act::ShowInLog => m::KEY_SHOW_IN_LOG.get(),
            Act::CreateBranch => m::KEY_CREATE_BRANCH.get(),
            Act::RestoreRef => m::KEY_RESTORE_REF.get(),
            Act::CherryPick => m::KEY_CHERRY_PICK.get(),
            Act::CreateTag => m::KEY_CREATE_TAG.get(),
            Act::ApplyToWorktree => m::KEY_APPLY.get(),
            Act::SaveAsStash => m::KEY_STASH.get(),
            Act::LoadMore => m::KEY_LOAD_MORE.get(),
        }
    }
}

/// One journal entry in the history, and which stack it is on.
#[derive(Debug, Clone)]
pub struct HistoryRow {
    pub entry: ListedEntry,
    /// On the redo stack (undone).
    pub redo: bool,
}

/// One painted line of the history list: separators scroll like rows but are never selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Line {
    /// An entry: its index in [`HistoryModel::rows`].
    Row(usize),
    /// A day heading (days since the epoch, UTC) when the list spans several.
    Day(i64),
    /// The divider between the undone entries above and the undo stack.
    Redo,
}

#[derive(Default)]
pub struct HistoryModel {
    /// Undo entries newest first, then the undone ones, next-to-redo first.
    pub rows: Vec<HistoryRow>,
    pub undo_count: usize,
    pub lines: Vec<Line>,
    /// The selected line.
    pub selected: usize,
    pub loaded: bool,
    pub error: Option<String>,
}

impl HistoryModel {
    pub fn set(&mut self, listing: JournalListing) {
        let keep = self.selected_row().map(|r| self.rows[r].entry.summary.id);
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
            self.selected = line;
        }
        self.selected = self.selected.min(self.lines.len().saturating_sub(1));
        self.snap(1);
        self.loaded = true;
        self.error = None;
    }

    pub fn row_at(&self, line: usize) -> Option<usize> {
        match self.lines.get(line)? {
            Line::Row(row) => Some(*row),
            _ => None,
        }
    }

    pub fn selected_row(&self) -> Option<usize> {
        self.row_at(self.selected)
    }

    pub fn selected(&self) -> Option<&HistoryRow> {
        self.rows.get(self.selected_row()?)
    }

    fn line_of(&self, row: usize) -> Option<usize> {
        self.lines.iter().position(|l| *l == Line::Row(row))
    }

    /// Selects the line holding entry `id`.
    pub fn select_entry(&mut self, id: u64) -> bool {
        let at = self
            .rows
            .iter()
            .position(|r| r.entry.summary.id == id)
            .and_then(|row| self.line_of(row));
        if let Some(line) = at {
            self.selected = line;
        }
        at.is_some()
    }

    /// Moves the selection by `delta` rows, skipping separators.
    pub fn move_by(&mut self, delta: isize) {
        let mut at = self.selected as isize;
        let step = delta.signum();
        let mut left = delta.abs();
        while left > 0 {
            let next = at + step;
            if next < 0 || next as usize >= self.lines.len() {
                break;
            }
            at = next;
            if self.row_at(at as usize).is_some() {
                left -= 1;
            }
        }
        self.selected = at as usize;
        self.snap(step);
    }

    /// Moves the selection off a separator: on in the direction it was going, else back.
    pub fn snap(&mut self, dir: isize) {
        let len = self.lines.len();
        let at = self.selected;
        if len == 0 || self.row_at(at).is_some() {
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
            self.selected = to;
        }
    }

    /// The rows an "up to here" on the selected row covers, in the order they are undone (or redone).
    pub fn steps_for_selection(
        &self,
    ) -> Option<(gitpoulpi_core::domain::Direction, Vec<&HistoryRow>)> {
        use gitpoulpi_core::domain::Direction;
        let sel = self.selected_row()?;
        let row = self.rows.get(sel)?;
        if row.redo {
            Some((
                Direction::Redo,
                self.rows[self.undo_count..=sel].iter().collect(),
            ))
        } else {
            Some((Direction::Undo, self.rows[..=sel].iter().collect()))
        }
    }
}

/// The painted lines: the undone entries on top (the farthest from the line first), the divider, then
/// the undo stack newest first. Day headings only when the entries span several days.
pub fn build_lines(rows: &[HistoryRow], undo_count: usize) -> Vec<Line> {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowState {
    /// On the undo stack and the repository agrees.
    Undoable,
    /// The newest entry, and the repository moved under it: why.
    Blocked(String),
    /// On the redo stack.
    Undone,
}

/// The state of history row `index` given the daemon's view of the top entries.
pub fn row_state(row: &HistoryRow, index: usize, state: &JournalState) -> RowState {
    if row.redo {
        return RowState::Undone;
    }
    match (&state.blocked, index) {
        (Some(refusal), 0) => RowState::Blocked(undo_refusal(refusal)),
        _ => RowState::Undoable,
    }
}

fn is_outside(row: &HistoryRow) -> bool {
    row.entry.source == EntrySource::Outside
}

/// The short tag at the right end of a history row.
pub fn tag(row: &HistoryRow, state: &RowState) -> String {
    match state {
        RowState::Blocked(_) => m::TAG_BLOCKED.get().to_string(),
        RowState::Undone => m::TAG_UNDONE.get().to_string(),
        RowState::Undoable if is_outside(row) => m::TAG_OUTSIDE.get().to_string(),
        RowState::Undoable if row.entry.summary.origin.is_some() => row
            .entry
            .summary
            .origin
            .as_ref()
            .map(|o| format!("\u{27F3} {}", o.name))
            .unwrap_or_default(),
        RowState::Undoable if row.entry.restore.restorable() => m::TAG_RESTORABLE.get().to_string(),
        RowState::Undoable => String::new(),
    }
}

/// `Name <email>` to `Name`.
fn person(committer: &str) -> &str {
    committer.split(" <").next().unwrap_or(committer).trim()
}

/// The lines under the history list for the selected row: its state, what restoring it brings back.
pub fn history_detail(row: &HistoryRow, state: &RowState) -> Vec<String> {
    let mut lines = Vec::new();
    match state {
        RowState::Blocked(why) => lines.push(m::detail_blocked(why)),
        RowState::Undone => lines.push(m::DETAIL_UNDONE.get().to_string()),
        RowState::Undoable => lines.push(m::DETAIL_UNDOABLE.get().to_string()),
    }
    if let Some(o) = &row.entry.summary.origin {
        lines.push(m::detail_sequence(&o.name, o.step, o.of));
    }
    if is_outside(row) {
        let who = row
            .entry
            .outside
            .as_ref()
            .and_then(|i| i.committer.as_deref())
            .map(person)
            .unwrap_or("?");
        lines.push(m::detail_outside(who));
    }
    if !row.redo {
        if row.entry.restore.restorable() {
            let items: Vec<String> = row
                .entry
                .restore
                .items
                .iter()
                .map(label::restore_item)
                .collect();
            lines.push(m::detail_restorable(items.join(", ")));
        } else if let Some(why) = &row.entry.restore.not_restorable {
            lines.push(m::detail_not_restorable(label::not_restorable(why)));
        }
    }
    lines
}

/// How long ago, in the language's relative-time words.
pub fn age(then: i64) -> String {
    use gitpoulpi_i18n::msg::reltime as r;
    let secs = (crate::util::dates::now_secs() - then).max(0) as u64;
    match secs {
        0..=59 => r::JUST_NOW.get().to_string(),
        60..=3599 => r::minutes_ago(secs / 60),
        3600..=86_399 => r::hours_ago(secs / 3600),
        86_400..=604_799 => r::days_ago(secs / 86_400),
        604_800..=2_591_999 => r::weeks_ago(secs / 604_800),
        2_592_000..=31_535_999 => r::months_ago(secs / 2_592_000),
        _ => r::years_ago(secs / 31_536_000),
    }
}

/// The heading of a day group: Today, Yesterday, else the date.
pub fn day_heading(day: i64) -> String {
    use gitpoulpi_i18n::msg::undo_label as l;
    let today = crate::util::dates::now_secs().div_euclid(86_400);
    match today - day {
        0 => l::DAY_TODAY.get().to_string(),
        1 => l::DAY_YESTERDAY.get().to_string(),
        _ => crate::util::dates::iso_date(day * 86_400),
    }
}

/// A list with a cursor: the Reflog and Lost work views.
#[derive(Default)]
pub struct Cursor {
    pub selected: usize,
}

impl Cursor {
    pub fn move_by(&mut self, delta: isize, len: usize) {
        if len > 0 {
            self.selected = self
                .selected
                .saturating_add_signed(delta)
                .min(len.saturating_sub(1));
        }
    }

    pub fn clamp(&mut self, len: usize) {
        self.selected = self.selected.min(len.saturating_sub(1));
    }
}

#[derive(Default)]
pub struct ReflogModel {
    /// `HEAD` or a full branch name.
    pub refname: String,
    pub entries: Vec<ReflogEntry>,
    pub more: bool,
    pub cursor: Cursor,
    pub loaded: bool,
    pub fetching_more: bool,
    pub error: Option<String>,
}

impl ReflogModel {
    /// The list's rows: every position, then the "Load more" row while there are more pages.
    pub fn rows(&self) -> usize {
        self.entries.len() + usize::from(self.more)
    }

    /// The "Load more" row is the one after the last position.
    pub fn is_more_row(&self, index: usize) -> bool {
        self.more && index == self.entries.len()
    }

    pub fn selected(&self) -> Option<&ReflogEntry> {
        self.entries.get(self.cursor.selected)
    }

    /// The local branch the view can restore; `None` for a tag-like or unreadable name.
    pub fn target(&self) -> Option<String> {
        let name = &self.refname;
        if name == "HEAD" {
            return Some("HEAD".to_string());
        }
        name.strip_prefix("refs/heads/")
            .map(str::to_string)
            .or_else(|| (!name.starts_with("refs/")).then(|| name.clone()))
    }
}

#[derive(Default)]
pub struct LostModel {
    pub commits: Vec<LostCommit>,
    pub cursor: Cursor,
    pub partial: bool,
    pub loaded: bool,
    pub scanning: bool,
    pub error: Option<String>,
}

impl LostModel {
    pub fn selected(&self) -> Option<&LostCommit> {
        self.commits.get(self.cursor.selected)
    }
}

/// The commit the selected row of the Reflog or Lost work view names.
pub fn selected_commit(tab: Tab, reflog: &ReflogModel, lost: &LostModel) -> Option<Oid> {
    match tab {
        Tab::Reflog => reflog.selected().map(|e| e.new),
        Tab::Lost => lost.selected().map(|c| c.oid),
        Tab::History => None,
    }
}

/// The actions that apply to the selected row of the open view, in the order the buttons list them.
pub fn actions(
    tab: Tab,
    history: &HistoryModel,
    reflog: &ReflogModel,
    lost: &LostModel,
    state: &JournalState,
) -> Vec<Act> {
    match tab {
        Tab::History => {
            let Some(row) = history.selected() else {
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
                if history.selected_row() == Some(0) && label::offer_of(state).is_some() {
                    acts.push(Act::Offer);
                }
            }
            acts
        }
        Tab::Reflog => {
            let mut acts = Vec::new();
            if reflog.selected().is_some() {
                acts.extend([
                    Act::ShowInLog,
                    Act::CreateBranch,
                    Act::CreateTag,
                    Act::ApplyToWorktree,
                    Act::SaveAsStash,
                ]);
                if reflog.target().is_some() {
                    acts.push(Act::RestoreRef);
                }
            }
            if reflog.more && !reflog.fetching_more {
                acts.push(Act::LoadMore);
            }
            acts
        }
        Tab::Lost => {
            if lost.selected().is_none() {
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

#[cfg(test)]
mod tests;
