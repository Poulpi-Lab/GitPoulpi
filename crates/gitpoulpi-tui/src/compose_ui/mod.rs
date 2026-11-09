//! "Compose commits with AI": the interactive review of a plan that splits
//! the working changes into logical commits, and the one action that
//! applies it.
//!
//! The model proposes `[{message, files}]` (validated against the real
//! change set before it ever gets here, see `gitpoulpi_ai::parse::
//! composer_plan`); this popup lets the user reorder the commits, edit a
//! message, move a file to the previous/next commit or into a commit of
//! its own, and drop a commit (its files then stay uncommitted). Applying
//! runs [`Action::ComposeCommits`]: unstage everything, then per commit
//! stage its files and commit. The working tree is never touched, so
//! nothing here can lose work; a failure part-way stops and says how many
//! commits were made.
//!
//! Same look and mechanics as the rebase popup (`rebase_ui.rs`): it takes
//! over the Log's column, rows are selectable, every mouse control has a
//! key.

mod render;

use super::*;
use crate::combo::{Combo, ComboOption, ComboOutcome};
use crate::daemon_link::ComposedCommit;
use crate::keymap::Action as Bind;
use crate::reword_ui::{RewordForm, RewordFormOutcome, RewordPart};
use crate::theme::plain_button_style;
use gitpoulpi_i18n::msg::compose_ui as msg;

/// One commit of the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ComposeCommit {
    pub(crate) message: String,
    pub(crate) files: Vec<String>,
    /// Left out: its files stay uncommitted.
    pub(crate) dropped: bool,
    /// Its files are listed under it.
    pub(crate) expanded: bool,
}

/// A selectable row: a commit, or one of its files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ComposeRow {
    Commit(usize),
    File(usize, usize),
}

/// The popup's clickable parts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ComposePart {
    Row(ComposeRow),
    /// The arrow of a commit row: expand or collapse its files.
    Toggle(usize),
    /// Row `.0` of the right pane's file list (the selected proposal's).
    PaneFile(usize),
    CommitUp(usize),
    CommitDown(usize),
    CommitEdit(usize),
    CommitDrop(usize),
    FilePrev(usize, usize),
    FileNext(usize, usize),
    FileNew(usize, usize),
    /// Opens "Move file to commit...".
    FileMove(usize, usize),
    Reset,
    Cancel,
    Apply,
    /// A part of the embedded Reword popup (message and description).
    Editing(RewordPart),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ComposeOutcome {
    Stay,
    Close,
    Submit(Action),
}

pub(crate) struct ComposeForm {
    commits: Vec<ComposeCommit>,
    initial: Vec<ComposeCommit>,
    selected: ComposeRow,
    /// The Reword popup open on commit `.0` of the plan.
    editing: Option<(usize, RewordForm)>,
    /// The last press on a row: a second one on the same row within
    /// `DOUBLE_CLICK_WINDOW` opens the reword popup.
    last_row_press: Option<(std::time::Instant, ComposeRow)>,
    /// Proposals open expanded (`ui.composer_files`), also the new ones.
    default_expanded: bool,
    /// A file grabbed with the mouse, on its way to another proposal.
    drag: Option<FileDrag>,
    /// The "Move file to commit..." dropdown.
    moving: Option<MoveFile>,
    error: Option<String>,
    /// The list window the wheel scrolled to, and the row that was selected
    /// then: any move of the selection follows it again.
    wheel: std::cell::Cell<Option<(usize, ComposeRow)>>,
    /// The window the last paint showed: first row, rows visible, rows in all.
    window: std::cell::Cell<(usize, usize, usize)>,
    /// The list's scrollbar as the last paint drew it, and the thumb held.
    bar: std::cell::Cell<Option<crate::scrollbar::Track>>,
    bar_drag: crate::scrollbar::BarDrag,
}

/// A file picked up by a press: `(commit, file)` and, once the pointer has
/// moved, the proposal it is over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FileDrag {
    from: (usize, usize),
    over: Option<usize>,
    moved: bool,
}

/// The keyboard twin of the drag: pick the proposal in a combo.
struct MoveFile {
    from: (usize, usize),
    combo: std::cell::RefCell<Combo>,
}

impl std::fmt::Debug for ComposeForm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComposeForm")
            .field("commits", &self.commits)
            .finish()
    }
}

impl PartialEq for ComposeForm {
    fn eq(&self, other: &Self) -> bool {
        self.commits == other.commits
    }
}

impl ComposeForm {
    #[cfg(test)]
    pub(crate) fn new(plan: Vec<(String, Vec<String>)>) -> Self {
        Self::with_expanded(plan, true)
    }

    pub(crate) fn with_expanded(plan: Vec<(String, Vec<String>)>, expanded: bool) -> Self {
        let commits: Vec<ComposeCommit> = plan
            .into_iter()
            .map(|(message, files)| ComposeCommit {
                message,
                files,
                dropped: false,
                expanded,
            })
            .collect();
        Self {
            initial: commits.clone(),
            commits,
            selected: ComposeRow::Commit(0),
            editing: None,
            last_row_press: None,
            default_expanded: expanded,
            drag: None,
            moving: None,
            error: None,
            wheel: std::cell::Cell::new(None),
            window: std::cell::Cell::new((0, 0, 0)),
            bar: std::cell::Cell::new(None),
            bar_drag: Default::default(),
        }
    }

    /// The wheel scrolls the list `delta` rows; the selection stays put (it
    /// may leave the window). The keyboard twin is the selection keys.
    pub(crate) fn on_wheel(&mut self, delta: isize) {
        let (offset, rows, count) = self.window.get();
        let max = count.saturating_sub(rows);
        let to = (offset as isize + delta).clamp(0, max as isize) as usize;
        self.wheel.set(Some((to, self.selected)));
    }

    /// The list's scrollbar: a press on it, a drag of the thumb. `true` when
    /// the bar took the event. It scrolls the list as the wheel does.
    pub(crate) fn on_bar_mouse(&mut self, mouse: crossterm::event::MouseEvent) -> bool {
        let (offset, _, count) = self.window.get();
        match self.bar_drag.on_mouse(self.bar.get(), mouse, count, offset) {
            Some(top) => {
                self.wheel.set(Some((top, self.selected)));
                true
            }
            None => false,
        }
    }

    /// The right pane's file rows are dragged from too.
    fn pane_file(&self, j: usize) -> Option<ComposeRow> {
        let (i, c) = self.selected_commit()?;
        (j < c.files.len()).then_some(ComposeRow::File(i, j))
    }

    pub(crate) fn is_editing(&self) -> bool {
        self.editing.is_some()
    }

    /// The Move-file dropdown is open (it owns the keys and the mouse).
    pub(crate) fn is_moving(&self) -> bool {
        self.moving.is_some()
    }

    /// The selected row is a file (what the palette's Move needs).
    pub(crate) fn file_selected(&self) -> bool {
        matches!(self.selected, ComposeRow::File(..))
    }

    /// Opens "Move file to commit..." on the selected file, or on the
    /// selected proposal's first file when a commit row is selected.
    pub(crate) fn start_move(&mut self) {
        let from = match self.selected {
            ComposeRow::File(i, j) => (i, j),
            ComposeRow::Commit(i) => {
                if self.commits.get(i).is_none_or(|c| c.files.is_empty()) {
                    return;
                }
                (i, 0)
            }
        };
        let mut combo = Combo::new(msg::SELECT_COMMIT.get());
        let mut options: Vec<ComboOption> = self
            .commits
            .iter()
            .enumerate()
            .filter(|(k, _)| *k != from.0)
            .map(|(k, c)| {
                let (summary, _) = crate::reword_ui::split_message(&c.message);
                ComboOption::new(format!("{}. {summary}", k + 1), k.to_string())
            })
            .collect();
        if self.commits[from.0].files.len() > 1 {
            options.push(ComboOption::new(msg::NEW_COMMIT_OPTION.get(), "new"));
        }
        if options.is_empty() {
            self.error = Some(msg::NO_OTHER_COMMIT.get().into());
            return;
        }
        combo.set_options(options);
        combo.open();
        self.moving = Some(MoveFile {
            from,
            combo: std::cell::RefCell::new(combo),
        });
    }

    fn finish_move(&mut self, value: &str) {
        let Some(m) = self.moving.take() else { return };
        let (i, j) = m.from;
        if value == "new" {
            self.file_to_new_commit(i, j);
        } else if let Ok(k) = value.parse::<usize>() {
            self.move_file_to(i, j, k);
        }
    }

    /// A mouse event while the dropdown is open.
    pub(crate) fn on_move_mouse(&mut self, mouse: crossterm::event::MouseEvent) {
        let Some(m) = self.moving.as_mut() else {
            return;
        };
        let out = m.combo.get_mut().on_mouse(mouse);
        self.after_move_input(out);
    }

    fn on_move_key(&mut self, key: KeyEvent) {
        let Some(m) = self.moving.as_mut() else {
            return;
        };
        let out = m.combo.get_mut().on_key(key);
        self.after_move_input(out);
    }

    fn after_move_input(&mut self, out: ComboOutcome) {
        match out {
            ComboOutcome::Picked(v) => self.finish_move(&v),
            _ => {
                if self
                    .moving
                    .as_mut()
                    .is_some_and(|m| !m.combo.get_mut().is_open())
                {
                    self.moving = None;
                }
            }
        }
    }

    /// Expands or collapses commit `i`; collapsing a commit whose file is
    /// selected selects the commit.
    pub(crate) fn toggle_expanded(&mut self, i: usize, on: Option<bool>) {
        let Some(c) = self.commits.get_mut(i) else {
            return;
        };
        c.expanded = on.unwrap_or(!c.expanded);
        if !c.expanded {
            if let ComposeRow::File(k, _) = self.selected {
                if k == i {
                    self.selected = ComposeRow::Commit(i);
                }
            }
        }
    }

    fn toggle_all(&mut self) {
        let on = !self.commits.iter().all(|c| c.expanded);
        for i in 0..self.commits.len() {
            self.toggle_expanded(i, Some(on));
        }
    }

    /// The pointer moved with the button held: tracks the proposal a
    /// grabbed file is over.
    pub(crate) fn on_file_drag(&mut self, part: Option<ComposePart>) {
        let over = part.and_then(Self::part_commit);
        if let Some(d) = &mut self.drag {
            d.moved = true;
            d.over = over;
        }
    }

    fn part_commit(part: ComposePart) -> Option<usize> {
        match part {
            ComposePart::Row(ComposeRow::Commit(k) | ComposeRow::File(k, _))
            | ComposePart::Toggle(k) => Some(k),
            _ => None,
        }
    }

    /// The button went up over `part`: a grabbed file dropped on another
    /// proposal moves there.
    pub(crate) fn on_drop(&mut self, part: Option<ComposePart>) {
        let Some(d) = self.drag.take() else { return };
        if !d.moved {
            return;
        }
        if let Some(k) = part.and_then(Self::part_commit) {
            if k != d.from.0 {
                self.move_file_to(d.from.0, d.from.1, k);
            }
        }
    }

    pub(crate) fn dragging(&self) -> bool {
        self.drag.is_some_and(|d| d.moved)
    }

    /// The selected proposal (the commit itself, or the one owning the
    /// selected file) and its index: what the right pane lists.
    pub(crate) fn selected_commit(&self) -> Option<(usize, &ComposeCommit)> {
        let (ComposeRow::Commit(i) | ComposeRow::File(i, _)) = self.selected;
        self.commits.get(i).map(|c| (i, c))
    }

    #[cfg(test)]
    pub(crate) fn commits(&self) -> &[ComposeCommit] {
        &self.commits
    }

    /// Every row in display order: a commit, then its files.
    fn rows(&self) -> Vec<ComposeRow> {
        let mut rows = Vec::new();
        for (i, commit) in self.commits.iter().enumerate() {
            rows.push(ComposeRow::Commit(i));
            if commit.expanded {
                for j in 0..commit.files.len() {
                    rows.push(ComposeRow::File(i, j));
                }
            }
        }
        rows
    }

    fn move_selection(&mut self, delta: isize) {
        let rows = self.rows();
        if rows.is_empty() {
            return;
        }
        let at = rows.iter().position(|r| *r == self.selected).unwrap_or(0) as isize;
        let next = (at + delta).clamp(0, rows.len() as isize - 1) as usize;
        self.selected = rows[next];
    }

    /// Moves commit `i` up (`-1`) or down (`+1`) the plan.
    pub(crate) fn move_commit(&mut self, i: usize, delta: isize) {
        let j = i as isize + delta;
        if j < 0 || j as usize >= self.commits.len() {
            return;
        }
        self.commits.swap(i, j as usize);
        self.selected = ComposeRow::Commit(j as usize);
    }

    /// Moves file `j` of commit `i` to the previous (`-1`) or next (`+1`)
    /// commit. An emptied commit is removed.
    pub(crate) fn move_file(&mut self, i: usize, j: usize, delta: isize) {
        let target = i as isize + delta;
        if target < 0 || target as usize >= self.commits.len() || j >= self.commits[i].files.len() {
            return;
        }
        self.move_file_to(i, j, target as usize);
    }

    /// Moves file `j` of commit `i` to the end of commit `target` (which
    /// opens). The file leaves `i`, so it stays in exactly one commit; an
    /// emptied commit is removed.
    pub(crate) fn move_file_to(&mut self, i: usize, j: usize, target: usize) {
        if target >= self.commits.len() || i >= self.commits.len() || i == target {
            return;
        }
        if j >= self.commits[i].files.len() {
            return;
        }
        let file = self.commits[i].files.remove(j);
        self.commits[target].files.push(file);
        self.commits[target].expanded = true;
        let at = self.commits[target].files.len() - 1;
        self.selected = ComposeRow::File(target, at);
        self.prune_empty();
    }

    /// Moves file `j` of commit `i` into a new commit right after `i`.
    pub(crate) fn file_to_new_commit(&mut self, i: usize, j: usize) {
        if j >= self.commits[i].files.len() || self.commits[i].files.len() == 1 {
            self.error = Some(msg::ONLY_FILE.get().into());
            return;
        }
        let file = self.commits[i].files.remove(j);
        self.commits.insert(
            i + 1,
            ComposeCommit {
                message: msg::NEW_COMMIT_MESSAGE.get().into(),
                files: vec![file],
                dropped: false,
                expanded: self.default_expanded,
            },
        );
        self.selected = ComposeRow::Commit(i + 1);
    }

    pub(crate) fn toggle_drop(&mut self, i: usize) {
        if let Some(c) = self.commits.get_mut(i) {
            c.dropped = !c.dropped;
        }
    }

    fn prune_empty(&mut self) {
        // Removing a commit may shift the selected index.
        let selected_commit = match self.selected {
            ComposeRow::Commit(i) | ComposeRow::File(i, _) => i,
        };
        let mut shift = 0;
        let mut keep = Vec::new();
        for (i, c) in std::mem::take(&mut self.commits).into_iter().enumerate() {
            if c.files.is_empty() {
                if i < selected_commit {
                    shift += 1;
                }
            } else {
                keep.push(c);
            }
        }
        self.commits = keep;
        self.selected = match self.selected {
            ComposeRow::Commit(i) => ComposeRow::Commit(i.saturating_sub(shift)),
            ComposeRow::File(i, j) => ComposeRow::File(i.saturating_sub(shift), j),
        };
    }

    pub(crate) fn reset(&mut self) {
        self.commits = self.initial.clone();
        self.drag = None;
        self.moving = None;
        self.selected = ComposeRow::Commit(0);
        self.editing = None;
        self.error = None;
    }

    /// The commits that will be made, in order: not dropped, not empty.
    pub(crate) fn plan(&self) -> Vec<(String, Vec<String>)> {
        self.commits
            .iter()
            .filter(|c| !c.dropped && !c.files.is_empty())
            .map(|c| (c.message.clone(), c.files.clone()))
            .collect()
    }

    /// Opens the Reword popup (summary and description) on commit `i`.
    fn start_edit(&mut self, i: usize) {
        if let Some(c) = self.commits.get(i) {
            let form = RewordForm::new(Oid([0; 20]), &c.message, None).embedded();
            self.editing = Some((i, form));
        }
    }

    /// What the popup's outcome means for the plan.
    fn finish_editing(&mut self, outcome: RewordFormOutcome) -> ComposeOutcome {
        match outcome {
            RewordFormOutcome::Stay => {}
            RewordFormOutcome::Close | RewordFormOutcome::ModifyDeeper => self.editing = None,
            RewordFormOutcome::Submit(Action::Reword { message, .. }) => {
                if let Some((i, _)) = self.editing.take() {
                    if let Some(c) = self.commits.get_mut(i) {
                        c.message = message;
                    }
                }
            }
            RewordFormOutcome::Submit(_) => self.editing = None,
        }
        ComposeOutcome::Stay
    }

    /// A press at (`col`, `row`): [`Self::on_click`], and inside the
    /// popup's fields it also places the caret.
    pub(crate) fn on_press(&mut self, part: ComposePart, col: u16, row: u16) -> ComposeOutcome {
        if let (ComposePart::Editing(p), Some((_, form))) = (part, &mut self.editing) {
            let outcome = form.on_press(p, col, row);
            return self.finish_editing(outcome);
        }
        let outcome = self.on_click(part);
        self.drag = match part {
            ComposePart::Row(ComposeRow::File(i, j)) => Some((i, j)),
            ComposePart::PaneFile(j) => match self.selected {
                ComposeRow::File(i, k) if k == j => Some((i, j)),
                _ => None,
            },
            _ => None,
        }
        .map(|from| FileDrag {
            from,
            over: None,
            moved: false,
        });
        outcome
    }

    /// A drag with the button held; `true` when the popup used it.
    pub(crate) fn on_text_drag(&mut self, col: u16, row: u16) -> bool {
        self.editing
            .as_mut()
            .is_some_and(|(_, form)| form.on_drag(col, row))
    }

    pub(crate) fn on_release(&mut self) {
        if let Some((_, form)) = &mut self.editing {
            form.on_release();
        }
    }

    pub(crate) fn try_apply(&mut self) -> ComposeOutcome {
        let plan = self.plan();
        if plan.is_empty() {
            self.error = Some(msg::NOTHING_TO_COMMIT.get().into());
            return ComposeOutcome::Stay;
        }
        if let Some((message, _)) = plan.iter().find(|(m, _)| m.trim().is_empty()) {
            self.error = Some(msg::empty_message(format!("{message:?}")));
            return ComposeOutcome::Stay;
        }
        ComposeOutcome::Submit(Action::ComposeCommits {
            commits: plan
                .into_iter()
                .map(|(message, files)| ComposedCommit {
                    message,
                    paths: files.into_iter().map(std::path::PathBuf::from).collect(),
                })
                .collect(),
        })
    }

    pub(crate) fn on_paste(&mut self, text: &str) {
        if let Some((_, form)) = self.editing.as_mut() {
            form.on_paste(text);
        }
    }

    pub(crate) fn on_key(&mut self, key: KeyEvent) -> ComposeOutcome {
        self.error = None;
        if self.moving.is_some() {
            self.on_move_key(key);
            return ComposeOutcome::Stay;
        }
        if let Some((_, form)) = self.editing.as_mut() {
            let outcome = form.on_key(key);
            return self.finish_editing(outcome);
        }
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        match key.code {
            KeyCode::Esc => return ComposeOutcome::Close,
            _ if crate::keymap::current().is(Bind::ComposeApply, &key) => return self.try_apply(),
            KeyCode::Up if alt => self.shift(-1),
            KeyCode::Down if alt => self.shift(1),
            KeyCode::Char('K') => self.shift(-1),
            KeyCode::Char('J') => self.shift(1),
            KeyCode::Up | KeyCode::Char('k') => self.move_selection(-1),
            KeyCode::Down | KeyCode::Char('j') => self.move_selection(1),
            KeyCode::Char('e') | KeyCode::Enter => {
                let (ComposeRow::Commit(i) | ComposeRow::File(i, _)) = self.selected;
                self.start_edit(i);
            }
            KeyCode::Char('d') | KeyCode::Delete => {
                let (ComposeRow::Commit(i) | ComposeRow::File(i, _)) = self.selected;
                self.toggle_drop(i);
            }
            KeyCode::Char('n') => {
                if let ComposeRow::File(i, j) = self.selected {
                    self.file_to_new_commit(i, j);
                }
            }
            KeyCode::Char('m') => self.start_move(),
            KeyCode::Right | KeyCode::Char('l') => {
                let (ComposeRow::Commit(i) | ComposeRow::File(i, _)) = self.selected;
                self.toggle_expanded(i, Some(true));
            }
            KeyCode::Left | KeyCode::Char('h') => {
                let (ComposeRow::Commit(i) | ComposeRow::File(i, _)) = self.selected;
                self.toggle_expanded(i, Some(false));
            }
            KeyCode::Char('*') => self.toggle_all(),
            KeyCode::Char('r') => self.reset(),
            KeyCode::Char('a') => return self.try_apply(),
            _ => {}
        }
        ComposeOutcome::Stay
    }

    /// Alt+Up/Down and `K`/`J`: a commit moves up/down the plan, a file
    /// moves to the previous/next commit.
    fn shift(&mut self, delta: isize) {
        match self.selected {
            ComposeRow::Commit(i) => self.move_commit(i, delta),
            ComposeRow::File(i, j) => self.move_file(i, j, delta),
        }
    }

    pub(crate) fn on_click(&mut self, part: ComposePart) -> ComposeOutcome {
        self.error = None;
        match part {
            ComposePart::Row(row) => {
                let now = std::time::Instant::now();
                let double = self.last_row_press.is_some_and(|(t, r)| {
                    r == row && now.duration_since(t) < crate::pane_layout::DOUBLE_CLICK_WINDOW
                });
                self.selected = row;
                if double {
                    // Double-click: the keyboard twin is Enter / e.
                    self.last_row_press = None;
                    let (ComposeRow::Commit(i) | ComposeRow::File(i, _)) = row;
                    self.start_edit(i);
                } else {
                    self.last_row_press = Some((now, row));
                }
            }
            ComposePart::Toggle(i) => self.toggle_expanded(i, None),
            ComposePart::PaneFile(j) => {
                if let Some(row) = self.pane_file(j) {
                    let (ComposeRow::Commit(i) | ComposeRow::File(i, _)) = row;
                    self.toggle_expanded(i, Some(true));
                    self.selected = row;
                }
            }
            ComposePart::Editing(p) => {
                if let Some((_, form)) = &mut self.editing {
                    let outcome = form.on_click(p);
                    return self.finish_editing(outcome);
                }
            }
            ComposePart::CommitUp(i) => self.move_commit(i, -1),
            ComposePart::CommitDown(i) => self.move_commit(i, 1),
            ComposePart::CommitEdit(i) => self.start_edit(i),
            ComposePart::CommitDrop(i) => self.toggle_drop(i),
            ComposePart::FilePrev(i, j) => self.move_file(i, j, -1),
            ComposePart::FileNext(i, j) => self.move_file(i, j, 1),
            ComposePart::FileNew(i, j) => self.file_to_new_commit(i, j),
            ComposePart::FileMove(i, j) => {
                self.selected = ComposeRow::File(i, j);
                self.start_move();
            }
            ComposePart::Reset => self.reset(),
            ComposePart::Cancel => return ComposeOutcome::Close,
            ComposePart::Apply => return self.try_apply(),
        }
        ComposeOutcome::Stay
    }
}

/// The popup's own target under the pointer.
pub(crate) fn compose_hit_at(
    hits: &[(Rect, HitTarget)],
    col: u16,
    row: u16,
) -> Option<ComposePart> {
    hits.iter().find_map(|(rect, target)| match target {
        HitTarget::ComposeForm(part) if rect.contains((col, row).into()) => Some(*part),
        _ => None,
    })
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_bars;
