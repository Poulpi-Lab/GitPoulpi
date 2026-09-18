//! "Interactively rebase from here..." / "Cherry pick N commits...": the
//! popup that builds an [`gitpoulpi_core::domain::InteractiveRebase`]
//! plan and submits it in one shot.
//!
//! Rows are shown newest-first; the plan
//! sent to the backend is oldest-first ([`RebaseForm::steps`] reverses).
//! Squash/Fixup combine a row into the older commit that follows it
//! *below* it on screen -- refused on the last (oldest) row, which has
//! none.
//!
//! Reword opens the same summary+description editor as the standalone
//! "Edit commit message..." popup ([`crate::reword_ui::RewordForm`]),
//! embedded rather than duplicated.

mod render;

use gitpoulpi_core::domain::{autosquash_order, InteractiveRebase, RebaseAction, RebaseStep};

use super::*;
use crate::keymap::Action as Bind;
use crate::reword_ui::{RewordForm, RewordFormOutcome, RewordPart};
use crate::theme::plain_button_style;

/// One row of the plan, in the popup's own (newest-first) display order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RebaseRow {
    pub(crate) commit: Oid,
    pub(crate) summary: String,
    pub(crate) action: RebaseAction,
    /// Set once a `Reword` row's message has been edited; `None` keeps
    /// the commit's own summary as the message.
    pub(crate) new_message: Option<String>,
    /// The commit's full message (summary and body), what a Reword opens
    /// on and what a squash into another row contributes.
    pub(crate) message: String,
    /// Set while the row is a Reword only because commits are squashed
    /// into it (see [`RebaseForm::sync_squash_receivers`]).
    auto: Option<AutoReword>,
    /// The user took the automatic Reword away by hand: it is not put
    /// back while commits are squashed into the row.
    no_auto: bool,
}

/// What an automatic Reword remembers: the action to give back when
/// nothing is squashed into the row any more, and the combined message
/// it last wrote (an edited message is no longer this text).
#[derive(Debug, Clone, PartialEq, Eq)]
struct AutoReword {
    previous: RebaseAction,
    text: String,
}

impl RebaseRow {
    /// A `Pick` row for a commit with its full `message`.
    pub(crate) fn pick(commit: Oid, summary: &str, message: &str) -> Self {
        Self {
            commit,
            summary: summary.to_string(),
            action: RebaseAction::Pick,
            new_message: None,
            message: message.to_string(),
            auto: None,
            no_auto: false,
        }
    }

    /// The message this row starts from: its full message, or its
    /// summary when only that is known.
    fn base_message(&self) -> &str {
        if self.message.trim().is_empty() {
            &self.summary
        } else {
            &self.message
        }
    }
}

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
    rows: Vec<RebaseRow>,
    cursor: usize,
    pub(crate) autosquash: bool,
    /// A branch to check out first, when the plan is for a branch that is
    /// not the checked-out one (a ref dropped on another: see
    /// `ref_drop`). `None` for a plan about HEAD.
    pub(crate) checkout: Option<String>,
    editing: Option<RewordForm>,
    initial: Vec<RebaseRow>,
    error: Option<String>,
    /// `true` for a multi-commit cherry-pick: the plan replays the rows'
    /// (arbitrary) commits onto HEAD instead of rewriting a range.
    pub(crate) cherry_pick: bool,
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
            initial: rows.clone(),
            rows,
            cursor: 0,
            autosquash: false,
            checkout: None,
            editing: None,
            error: None,
            cherry_pick: false,
            dragging: false,
            dropdown: None,
            dropdown_cursor: 0,
            last_row_press: None,
            split_armed: None,
            head_sha: None,
            onto_label: None,
        }
    }

    /// The actions a row's dropdown offers: four for a
    /// cherry-pick, plus Fixup and Edit for a rebase.
    fn choices(&self) -> &'static [RebaseAction] {
        if self.cherry_pick {
            &[
                RebaseAction::Pick,
                RebaseAction::Reword,
                RebaseAction::Squash,
                RebaseAction::Drop,
            ]
        } else {
            &[
                RebaseAction::Pick,
                RebaseAction::Reword,
                RebaseAction::Squash,
                RebaseAction::Fixup,
                RebaseAction::Drop,
                RebaseAction::Edit,
            ]
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

    /// Bakes the autosquash preview into `rows` before any edit, so the
    /// row a click or key refers to is the row that was drawn.
    fn materialize(&mut self) {
        if self.autosquash {
            self.rows = self.display_rows();
            self.autosquash = false;
        }
    }

    /// Moves the row at `from` to position `to` (a drag), the cursor
    /// following it.
    fn move_row_to(&mut self, from: usize, to: usize) {
        if from >= self.rows.len() || to >= self.rows.len() || from == to {
            return;
        }
        let row = self.rows.remove(from);
        self.rows.insert(to, row);
        self.cursor = to;
        self.sync_squash_receivers();
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

    /// The plan the backend would run, oldest first - the reverse of
    /// display order, with `req.autosquash` handled by the backend
    /// itself rather than pre-applied here (so the preview and the
    /// actually-run plan can never disagree).
    pub(crate) fn steps(&self) -> Vec<RebaseStep> {
        self.rows
            .iter()
            .rev()
            .map(|r| RebaseStep {
                commit: r.commit,
                action: r.action,
                message: if r.action == RebaseAction::Reword {
                    Some(
                        r.new_message
                            .clone()
                            .unwrap_or_else(|| r.base_message().to_string()),
                    )
                } else {
                    None
                },
            })
            .collect()
    }

    /// The rows as they would be drawn once `autosquash` reordering is
    /// applied - a pure preview, computed on demand rather than kept in
    /// sync as a second copy of `rows`.
    fn display_rows(&self) -> Vec<RebaseRow> {
        if !self.autosquash {
            return self.rows.clone();
        }
        // `autosquash_order` reasons oldest-first; reverse in, reverse
        // out.
        let mut oldest_first: Vec<(Oid, String, RebaseAction)> = self
            .rows
            .iter()
            .rev()
            .map(|r| (r.commit, r.summary.clone(), r.action))
            .collect();
        autosquash_order(&mut oldest_first);
        let by_commit: std::collections::HashMap<Oid, &RebaseRow> =
            self.rows.iter().map(|r| (r.commit, r)).collect();
        oldest_first
            .into_iter()
            .rev()
            .map(|(commit, _, action)| {
                let base = by_commit.get(&commit).expect("row came from self.rows");
                RebaseRow {
                    commit,
                    summary: base.summary.clone(),
                    action,
                    new_message: base.new_message.clone(),
                    message: base.message.clone(),
                    auto: base.auto.clone(),
                    no_auto: base.no_auto,
                }
            })
            .collect()
    }

    /// Keeps every row that commits are squashed into a Reword carrying
    /// their combined message.
    ///
    /// The receiver of a Squash is the first row below it (older) that
    /// is not itself a Squash / Fixup. When at least one Squash folds
    /// into a `Pick` (or an unedited `Reword`), that row becomes a
    /// Reword whose message is its own followed by the full messages of
    /// the squashed commits, oldest first - git's combined squash
    /// message without the `# This is a combination` comments. Fixup
    /// drops its message as in git and never triggers this. When nothing
    /// is squashed into the row any more it gets its previous action
    /// back, unless the user edited the text meanwhile.
    fn sync_squash_receivers(&mut self) {
        for r in 0..self.rows.len() {
            if matches!(
                self.rows[r].action,
                RebaseAction::Squash | RebaseAction::Fixup
            ) {
                continue;
            }
            let mut squashed: Vec<String> = Vec::new();
            let mut k = r;
            while k > 0
                && matches!(
                    self.rows[k - 1].action,
                    RebaseAction::Squash | RebaseAction::Fixup
                )
            {
                k -= 1;
                if self.rows[k].action == RebaseAction::Squash {
                    squashed.push(self.rows[k].base_message().trim().to_string());
                }
            }
            let row = &mut self.rows[r];
            if squashed.is_empty() {
                if let Some(auto) = row.auto.take() {
                    let untouched = row.new_message.is_none()
                        || row.new_message.as_deref() == Some(auto.text.as_str());
                    if row.action == RebaseAction::Reword && untouched {
                        row.action = auto.previous;
                        row.new_message = None;
                    }
                }
                continue;
            }
            let mut combined = row.base_message().trim().to_string();
            for message in &squashed {
                combined.push_str("\n\n");
                combined.push_str(message);
            }
            match &mut row.auto {
                Some(auto) => {
                    if row.action != RebaseAction::Reword {
                        row.auto = None;
                    } else if row.new_message.is_none()
                        || row.new_message.as_deref() == Some(auto.text.as_str())
                    {
                        auto.text = combined.clone();
                        row.new_message = Some(combined);
                    }
                }
                None => {
                    if !row.no_auto
                        && (row.action == RebaseAction::Pick
                            || (row.action == RebaseAction::Reword && row.new_message.is_none()))
                    {
                        row.auto = Some(AutoReword {
                            previous: row.action,
                            text: combined.clone(),
                        });
                        row.action = RebaseAction::Reword;
                        row.new_message = Some(combined);
                    }
                }
            }
        }
    }

    /// Stores the message the popup returned for the focused row. Text
    /// that is still the automatic combined message stays "unedited".
    fn set_new_message(&mut self, message: String) {
        use gitpoulpi_core::domain::normalize_reword_message;
        let row = &mut self.rows[self.cursor];
        if let Some(auto) = &row.auto {
            if normalize_reword_message(&message) == normalize_reword_message(&auto.text) {
                row.new_message = Some(auto.text.clone());
                return;
            }
        }
        row.new_message = Some(message);
    }

    fn is_last_row(&self, index: usize) -> bool {
        index + 1 == self.rows.len()
    }

    /// Sets the focused row's action, refusing Squash/Fixup on the last
    /// (oldest, no predecessor) row.
    fn set_action(&mut self, action: RebaseAction) {
        if self.rows.is_empty() {
            return;
        }
        self.materialize();
        if matches!(action, RebaseAction::Squash | RebaseAction::Fixup)
            && self.is_last_row(self.cursor)
        {
            self.error = Some(gitpoulpi_i18n::msg::rebase_ui::ERR_OLDEST.get().into());
            return;
        }
        self.error = None;
        self.rows[self.cursor].action = action;
        // The row was chosen by hand: it is no longer an automatic Reword
        // (unless Reword itself was chosen), and an unedited combined
        // message goes with it.
        if action != RebaseAction::Reword {
            let row = &mut self.rows[self.cursor];
            if let Some(auto) = row.auto.take() {
                row.no_auto = true;
                if row.new_message.as_deref() == Some(auto.text.as_str()) {
                    row.new_message = None;
                }
            }
        } else {
            self.rows[self.cursor].no_auto = false;
        }
        self.sync_squash_receivers();
        // The new message is asked for the moment Reword is chosen.
        if action == RebaseAction::Reword {
            self.open_reword();
        }
    }

    fn move_row(&mut self, delta: isize) {
        self.materialize();
        let n = self.rows.len();
        if n < 2 {
            return;
        }
        let at = self.cursor as isize;
        let to = at + delta;
        if to < 0 || to >= n as isize {
            return;
        }
        self.rows.swap(at as usize, to as usize);
        self.cursor = to as usize;
        self.sync_squash_receivers();
    }

    pub(crate) fn reset(&mut self) {
        self.rows = self.initial.clone();
        self.sync_squash_receivers();
        self.cursor = 0;
        self.autosquash = false;
        self.editing = None;
        self.dropdown = None;
        self.error = None;
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
        let edited = self.autosquash || self.rows != self.initial;
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
        // Bake the autosquash preview in, so that the receivers of the
        // squashes it creates carry their combined messages.
        self.materialize();
        self.sync_squash_receivers();
        let rebase = Action::RebaseInteractive(InteractiveRebase {
            upstream: self.upstream,
            onto: self.onto,
            steps: self.steps(),
            autosquash: self.autosquash,
            cherry_pick: self.cherry_pick,
        });
        RebaseFormOutcome::Submit(match &self.checkout {
            // Stops if the checkout is refused, so the plan can never be
            // replayed on the wrong branch.
            Some(branch) => Action::CheckoutThen {
                branch: branch.clone(),
                then: Box::new(rebase),
            },
            None => rebase,
        })
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
