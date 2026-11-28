//! "Interactively rebase from here..." / "Cherry pick N commits...": the
//! popup that builds an [`gitpoulpi_core::domain::InteractiveRebase`]
//! plan and submits it in one shot.
//!
//! Rows are shown newest-first; the plan
//! sent to the backend is oldest-first ([`RebasePlan::steps`] reverses).
//! Squash/Fixup combine a row into the older commit that follows it
//! *below* it on screen -- refused on the last (oldest) row, which has
//! none.
//!
//! Reword opens the same summary+description editor as the standalone
//! "Edit commit message..." popup ([`crate::reword_ui::RewordForm`]),
//! embedded rather than duplicated.

mod render;

use crate::keymap::term::TermKeymap;
use gitpoulpi_core::domain::RebaseAction;
pub(crate) use gitpoulpi_link::rebase_plan::{RebasePlan, RebaseRow};

use super::*;
use crate::keymap::Action as Bind;
use crate::reword_ui::{RewordForm, RewordFormOutcome, RewordPart};
use crate::theme::plain_button_style;

/// The popup's focusable/clickable parts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RebasePart {
    /// The drag handle / commit text of a row - press to select and to
    /// start dragging it up or down the plan.
    Row(usize),
    /// A row's action button ("Pick v"): opens or closes its dropdown.
    Dropdown(usize),
    /// One choice of a row's open dropdown (Pick, Reword, Squash, Drop,
    /// and for a rebase Fixup / Edit).
    Action(usize, RebaseAction),
    /// The "split..." button of a row: open the split editor on its commit.
    Split(usize),
    Autosquash,
    Reset,
    Cancel,
    Start,
    /// A part of the embedded Reword editor, opened on a `Reword` row.
    Editing(RewordPart),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum RebaseFormOutcome {
    Stay,
    Close,
    Submit(Action),
    /// Leave the popup for the split editor on `.0`; the popup is rebuilt
    /// on upstream `.1` afterwards.
    Split(Oid, Oid),
}

/// The interactive-rebase / cherry-pick popup.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RebaseForm {
    /// `None` when the plan targets HEAD directly (a multi-commit
    /// cherry-pick) rather than a linear range after a commit.
    pub(crate) upstream: Option<Oid>,
    pub(crate) onto: Option<Oid>,
    /// What the header names: "feature" for a rebase, or "HEAD" for a
    /// cherry-pick.
    pub(crate) title: String,
    /// The rows and the selection; the form derefs to it.
    plan: RebasePlan,
    /// A branch to check out first, when the plan is for a branch that is
    /// not the checked-out one (a ref dropped on another: see
    /// `ref_drop`). `None` for a plan about HEAD.
    pub(crate) checkout: Option<String>,
    editing: Option<RewordForm>,
    /// A row is being carried by the mouse (press on a row, not yet
    /// released): moving over another row moves it there.
    dragging: bool,
    /// The row whose action dropdown is open, and which of its choices is
    /// highlighted (`Space` opens it, Up/Down move, Enter picks).
    dropdown: Option<usize>,
    dropdown_cursor: usize,
    /// The last press on a row (when, which): a second press on the same
    /// row within `DOUBLE_CLICK_WINDOW` opens its reword editor.
    last_row_press: Option<(std::time::Instant, usize)>,
    /// The commit whose "split..." was pressed once on an edited plan: a
    /// second press confirms that the edits are discarded.
    split_armed: Option<Oid>,
    /// Short SHA of the commit a cherry-pick lands on, shown next to the
    /// branch name in the header.
    pub(crate) head_sha: Option<String>,
    /// What a rebase replays onto, for the header ("Rebasing X onto Y").
    pub(crate) onto_label: Option<String>,
    /// The first row the wheel or the scrollbar put on top, and the row that
    /// was selected then: any move of the selection follows it again.
    scroll: std::cell::Cell<Option<(usize, usize)>>,
    /// The window the last paint showed: first row, rows visible, rows in all.
    window: std::cell::Cell<(usize, usize, usize)>,
    /// The list's scrollbar as the last paint drew it, and the thumb held.
    bar: std::cell::Cell<Option<crate::scrollbar::Track>>,
    bar_drag: crate::scrollbar::BarDrag,
}

impl std::ops::Deref for RebaseForm {
    type Target = RebasePlan;
    fn deref(&self) -> &RebasePlan {
        &self.plan
    }
}

impl std::ops::DerefMut for RebaseForm {
    fn deref_mut(&mut self) -> &mut RebasePlan {
        &mut self.plan
    }
}

impl RebaseForm {
    /// `rows` in newest-first order, exactly as they should first appear.
    pub(crate) fn new(
        title: String,
        upstream: Option<Oid>,
        onto: Option<Oid>,
        rows: Vec<RebaseRow>,
    ) -> Self {
        Self {
            upstream,
            onto,
            title,
            plan: RebasePlan::new(rows),
            checkout: None,
            editing: None,
            dragging: false,
            dropdown: None,
            dropdown_cursor: 0,
            last_row_press: None,
            split_armed: None,
            head_sha: None,
            onto_label: None,
            scroll: std::cell::Cell::new(None),
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
        self.scroll.set(Some((to, self.cursor)));
    }

    /// The list's scrollbar: a press on it, a drag of the thumb. `true` when
    /// the bar took the event.
    pub(crate) fn on_bar_mouse(&mut self, mouse: crossterm::event::MouseEvent) -> bool {
        let (offset, _, count) = self.window.get();
        match self.bar_drag.on_mouse(self.bar.get(), mouse, count, offset) {
            Some(top) => {
                self.scroll.set(Some((top, self.cursor)));
                true
            }
            None => false,
        }
    }

    fn open_dropdown(&mut self, row: usize) {
        self.materialize();
        if row >= self.rows.len() {
            return;
        }
        self.cursor = row;
        self.dropdown = Some(row);
        let current = self.rows[row].action;
        self.dropdown_cursor = self
            .choices()
            .iter()
            .position(|a| *a == current)
            .unwrap_or(0);
    }

    /// Picks `action` for row `i` (a dropdown choice, a click, a key).
    /// Choosing Reword on an already-reworded row reopens its editor.
    fn choose(&mut self, i: usize, action: RebaseAction) {
        self.materialize();
        self.dropdown = None;
        if i >= self.rows.len() {
            return;
        }
        self.cursor = i;
        if action == RebaseAction::Reword && self.rows[i].action == RebaseAction::Reword {
            self.open_reword();
        } else {
            self.set_action(action);
        }
    }

    /// The same view for a multi-commit cherry-pick onto HEAD: `head` is
    /// the branch (or `HEAD`) the commits will land on; `rows` newest
    /// first.
    pub(crate) fn new_cherry_pick(head: String, rows: Vec<RebaseRow>) -> Self {
        let mut form = Self::new(head, None, None, rows);
        form.cherry_pick = true;
        form
    }

    /// The pointer moved, button held, over `part` (or off every row).
    pub(crate) fn on_drag(&mut self, part: Option<RebasePart>) {
        if !self.dragging {
            return;
        }
        if let Some(RebasePart::Row(j) | RebasePart::Action(j, _)) = part {
            let from = self.cursor;
            self.move_row_to(from, j);
        }
    }

    pub(crate) fn on_release(&mut self) {
        self.dragging = false;
        if let Some(editing) = &mut self.editing {
            editing.on_release();
        }
    }

    /// Sets the focused row's action; the new message is asked for the moment
    /// Reword is chosen.
    fn set_action(&mut self, action: RebaseAction) {
        if self.plan.apply_action(action) && action == RebaseAction::Reword {
            self.open_reword();
        }
    }

    pub(crate) fn reset(&mut self) {
        self.plan.reset();
        self.editing = None;
        self.dropdown = None;
    }

    /// Whether rows offer "split...": a rebase of the checked-out branch.
    pub(crate) fn can_split(&self) -> bool {
        !self.cherry_pick && self.checkout.is_none() && self.upstream.is_some()
    }

    /// "split..." on row `i`. On a plan with edits the first press only
    /// says they would be lost; the second one leaves the popup.
    fn request_split(&mut self, i: usize) -> RebaseFormOutcome {
        let Some(upstream) = self.upstream.filter(|_| self.can_split()) else {
            return RebaseFormOutcome::Stay;
        };
        let display = self.display_rows();
        let Some(row) = display.get(i) else {
            return RebaseFormOutcome::Stay;
        };
        if row.action == RebaseAction::Drop {
            return RebaseFormOutcome::Stay;
        }
        let commit = row.commit;
        self.cursor = i;
        let edited = self.plan.is_edited();
        if edited && self.split_armed != Some(commit) {
            self.split_armed = Some(commit);
            self.error = Some(gitpoulpi_i18n::msg::rebase_ui::SPLIT_CONFIRM.get().into());
            return RebaseFormOutcome::Stay;
        }
        RebaseFormOutcome::Split(commit, upstream)
    }

    /// Takes the pending "press split again" back (anything else was done).
    fn disarm_split(&mut self) {
        if self.split_armed.take().is_some() {
            self.error = None;
        }
    }

    fn open_reword(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        self.materialize();
        let row = &self.rows[self.cursor];
        if row.action != RebaseAction::Reword {
            return;
        }
        let message = row
            .new_message
            .clone()
            .unwrap_or_else(|| row.base_message().to_string());
        self.editing = Some(RewordForm::new(row.commit, &message, None).embedded());
    }

    pub(crate) fn try_submit(&mut self) -> RebaseFormOutcome {
        if self.rows.is_empty() {
            self.error = Some(gitpoulpi_i18n::msg::rebase_ui::ERR_NOTHING.get().into());
            return RebaseFormOutcome::Stay;
        }
        let (upstream, onto) = (self.upstream, self.onto);
        let checkout = self.checkout.clone();
        RebaseFormOutcome::Submit(self.plan.action(upstream, onto, checkout.as_deref()))
    }

    pub(crate) fn on_key(&mut self, key: KeyEvent) -> RebaseFormOutcome {
        if let Some(editing) = &mut self.editing {
            let outcome = editing.on_key(key);
            return self.finish_editing(outcome);
        }

        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        let km = crate::keymap::current();
        if !(!ctrl && !alt && km.is(Bind::RebaseSplit, &key)) {
            self.disarm_split();
        }
        // An open action dropdown owns the arrows, Enter, Space and Esc;
        // anything else (p/r/s/d, Ctrl+S ...) closes it and is handled as
        // if it had been typed with the list shut.
        if let Some(row) = self.dropdown {
            let n = self.choices().len();
            match key.code {
                KeyCode::Esc => {
                    self.dropdown = None;
                    return RebaseFormOutcome::Stay;
                }
                KeyCode::Up if !alt && !ctrl => {
                    self.dropdown_cursor = self.dropdown_cursor.saturating_sub(1);
                    return RebaseFormOutcome::Stay;
                }
                KeyCode::Down if !alt && !ctrl => {
                    self.dropdown_cursor = (self.dropdown_cursor + 1).min(n - 1);
                    return RebaseFormOutcome::Stay;
                }
                KeyCode::Enter | KeyCode::Char(' ') if !ctrl => {
                    let action = self.choices()[self.dropdown_cursor.min(n - 1)];
                    self.choose(row, action);
                    return RebaseFormOutcome::Stay;
                }
                _ => self.dropdown = None,
            }
        }
        match key.code {
            KeyCode::Esc => RebaseFormOutcome::Close,
            KeyCode::Char(' ') if !ctrl && !alt => {
                self.open_dropdown(self.cursor);
                RebaseFormOutcome::Stay
            }
            // Ctrl+Enter is indistinguishable from Enter on many terminals
            // (ssh, tmux, Windows Terminal without win32-input-mode), and
            // plain Enter opens the reword editor: Ctrl+S / Ctrl+J are
            // reliable twins (the default chords; `Ctrl+J` is `Ctrl+Enter`'s
            // spelling class).
            _ if km.is(Bind::RebaseStart, &key) => self.try_submit(),
            _ if km.is(Bind::RebaseMoveUp, &key) => {
                self.move_row(-1);
                RebaseFormOutcome::Stay
            }
            _ if km.is(Bind::RebaseMoveDown, &key) => {
                self.move_row(1);
                RebaseFormOutcome::Stay
            }
            // The Reset button's keyboard twin.
            _ if km.is(Bind::RebaseReset, &key) => {
                self.reset();
                RebaseFormOutcome::Stay
            }
            KeyCode::Up => {
                self.cursor = self.cursor.saturating_sub(1);
                RebaseFormOutcome::Stay
            }
            KeyCode::Down => {
                if self.cursor + 1 < self.rows.len() {
                    self.cursor += 1;
                }
                RebaseFormOutcome::Stay
            }
            _ if !ctrl && !alt && km.is(Bind::RebasePick, &key) => {
                self.set_action(RebaseAction::Pick);
                RebaseFormOutcome::Stay
            }
            _ if !ctrl && !alt && km.is(Bind::RebaseReword, &key) => {
                self.set_action(RebaseAction::Reword);
                RebaseFormOutcome::Stay
            }
            _ if !ctrl && !alt && km.is(Bind::RebaseSquash, &key) => {
                self.set_action(RebaseAction::Squash);
                RebaseFormOutcome::Stay
            }
            _ if !ctrl && !alt && km.is(Bind::RebaseFixup, &key) => {
                self.set_action(RebaseAction::Fixup);
                RebaseFormOutcome::Stay
            }
            _ if !ctrl && !alt && km.is(Bind::RebaseDrop, &key) => {
                self.set_action(RebaseAction::Drop);
                RebaseFormOutcome::Stay
            }
            _ if !ctrl && !alt && km.is(Bind::RebaseEdit, &key) => {
                self.set_action(RebaseAction::Edit);
                RebaseFormOutcome::Stay
            }
            _ if !ctrl && !alt && km.is(Bind::RebaseSplit, &key) => self.request_split(self.cursor),
            _ if !ctrl && !alt && km.is(Bind::RebaseAutosquash, &key) => {
                self.autosquash = !self.autosquash;
                RebaseFormOutcome::Stay
            }
            // The keyboard twin of double-clicking a row: reword it.
            KeyCode::Enter => {
                self.choose(self.cursor, RebaseAction::Reword);
                RebaseFormOutcome::Stay
            }
            _ => RebaseFormOutcome::Stay,
        }
    }

    /// What the embedded Reword popup's outcome means for the plan.
    fn finish_editing(&mut self, outcome: RewordFormOutcome) -> RebaseFormOutcome {
        match outcome {
            RewordFormOutcome::Stay => {}
            RewordFormOutcome::Close | RewordFormOutcome::ModifyDeeper => self.editing = None,
            RewordFormOutcome::Submit(Action::Reword { message, .. }) => {
                self.set_new_message(message);
                self.editing = None;
            }
            RewordFormOutcome::Submit(_) => self.editing = None,
        }
        RebaseFormOutcome::Stay
    }

    /// A press at (`col`, `row`) on `hit`: [`Self::on_click`], and inside
    /// the Reword popup's fields it also places the caret.
    pub(crate) fn on_press(&mut self, hit: RebasePart, col: u16, row: u16) -> RebaseFormOutcome {
        if let (RebasePart::Editing(part), Some(editing)) = (hit, &mut self.editing) {
            let outcome = editing.on_press(part, col, row);
            return self.finish_editing(outcome);
        }
        self.on_click(hit)
    }

    /// Text pasted while the Reword popup is open goes into it.
    pub(crate) fn on_paste(&mut self, text: &str) {
        if let Some(editing) = &mut self.editing {
            editing.on_paste(text);
        }
    }

    /// A drag with the button held at (`col`, `row`); `true` when the
    /// Reword popup used it to extend a text selection.
    pub(crate) fn on_text_drag(&mut self, col: u16, row: u16) -> bool {
        self.editing
            .as_mut()
            .is_some_and(|editing| editing.on_drag(col, row))
    }

    pub(crate) fn on_click(&mut self, hit: RebasePart) -> RebaseFormOutcome {
        if !matches!(hit, RebasePart::Split(_)) {
            self.disarm_split();
        }
        if !matches!(
            hit,
            RebasePart::Dropdown(_) | RebasePart::Action(..) | RebasePart::Editing(_)
        ) {
            self.dropdown = None;
        }
        match hit {
            RebasePart::Dropdown(i) => {
                if self.dropdown == Some(i) {
                    self.dropdown = None;
                } else {
                    self.open_dropdown(i);
                }
                RebaseFormOutcome::Stay
            }
            RebasePart::Editing(part) => {
                let Some(editing) = &mut self.editing else {
                    return RebaseFormOutcome::Stay;
                };
                let outcome = editing.on_click(part);
                self.finish_editing(outcome)
            }
            RebasePart::Row(i) => {
                self.materialize();
                if i < self.rows.len() {
                    let now = std::time::Instant::now();
                    let double = self.last_row_press.is_some_and(|(t, r)| {
                        r == i && now.duration_since(t) < crate::pane_layout::DOUBLE_CLICK_WINDOW
                    });
                    self.cursor = i;
                    if double {
                        // Double-click: reword the row.
                        self.last_row_press = None;
                        self.dragging = false;
                        self.choose(i, RebaseAction::Reword);
                    } else {
                        self.last_row_press = Some((now, i));
                        self.dragging = true;
                    }
                }
                RebaseFormOutcome::Stay
            }
            RebasePart::Action(i, action) => {
                self.choose(i, action);
                RebaseFormOutcome::Stay
            }
            RebasePart::Split(i) => self.request_split(i),
            RebasePart::Autosquash => {
                self.autosquash = !self.autosquash;
                RebaseFormOutcome::Stay
            }
            RebasePart::Reset => {
                self.reset();
                RebaseFormOutcome::Stay
            }
            RebasePart::Cancel => RebaseFormOutcome::Close,
            RebasePart::Start => self.try_submit(),
        }
    }

    fn action_label(action: RebaseAction) -> &'static str {
        use gitpoulpi_i18n::msg::rebase_ui as m;
        match action {
            RebaseAction::Pick => m::ACTION_PICK,
            RebaseAction::Reword => m::ACTION_REWORD,
            RebaseAction::Squash => m::ACTION_SQUASH,
            RebaseAction::Fixup => m::ACTION_FIXUP,
            RebaseAction::Drop => m::ACTION_DROP,
            RebaseAction::Edit => m::ACTION_EDIT,
        }
        .get()
    }

    /// The widest action label, in cells (at least 6): the dropdown button
    /// and its list size themselves on it, so a longer translation fits.
    fn action_label_width() -> usize {
        use RebaseAction::*;
        [Pick, Reword, Squash, Fixup, Drop, Edit]
            .into_iter()
            .map(|a| display_width(Self::action_label(a)))
            .max()
            .unwrap_or(6)
            .max(6)
    }

    fn action_color(action: RebaseAction) -> Color {
        match action {
            RebaseAction::Pick | RebaseAction::Reword => th().refs.local,
            RebaseAction::Squash | RebaseAction::Fixup => th().status.warning,
            RebaseAction::Drop => th().status.danger,
            RebaseAction::Edit => th().chrome.action_edit,
        }
    }
}

/// The popup's own target under the pointer - same pattern as
/// `reword_hit_at`.
pub(crate) fn rebase_hit_at(hits: &[(Rect, HitTarget)], col: u16, row: u16) -> Option<RebasePart> {
    hits.iter().find_map(|(rect, target)| match target {
        HitTarget::RebaseForm(part) if rect.contains((col, row).into()) => Some(*part),
        _ => None,
    })
}

#[cfg(test)]
mod tests;
