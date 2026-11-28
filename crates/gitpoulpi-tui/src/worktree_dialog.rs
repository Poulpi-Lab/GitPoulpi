//! The Create worktree dialog: a linked worktree
//! (`git worktree add`) from a branch or a commit, on a new branch or on an
//! existing one, optionally opened as a tab.
//!
//! ```text
//! ╭ Create worktree from main ───────────────────────────────────╮
//! │  Name            worktree-1a2b3c4d    [ Name after branch ]  │
//! │  Folder          C:\src\GitPoulpi-worktree-1a2b3c4d      │
//! │  Ref             main                                 ▾      │
//! │  Worktree branch worktree-1a2b3c4d      ▾     [ Regenerate ] │
//! │                  new branch from main     [ ✦ Name with AI ] │
//! │                                                              │
//! │  [x] Open the new worktree as a tab                          │
//! │  Will create C:\src\GitPoulpi-worktree-1a2b3c4d on ...   │
//! │  Tab next · Enter create               [ Cancel ] [ Create ] │
//! ╰──────────────────────────────────────────────────────────────╯
//! ```
//!
//! **Names.** The Name and the new branch both start as `worktree-<id>`
//! (8 random hex digits, the same for both). The Folder follows the Name
//! until it is edited by hand. The *Worktree branch* is one field with two
//! meanings: a name that is not a local branch creates a new branch from the
//! Ref; the name of a local branch that is not checked out elsewhere checks
//! that branch out as it is (the Ref is then unused, and says so).
//! "Name after branch" sets the Name to `worktree-<branch>` (no doubled
//! word), "Regenerate" draws a new id (see [`WorktreeDialog::regenerate`]
//! for what it replaces) and the ✦ button opens a box in the dialog where
//! the work is described, so the AI names both (never the status line).
//!
//! Pure, like `init_dialog`: validation, keys, clicks and drawing. The
//! owner (`repo_dialogs`) opens the shared folder picker for Browse, hands
//! [`WorktreeOutcome::NameWithAi`] to the AI worker and turns
//! [`WorktreeOutcome::Submit`] into a daemon action.

use crate::keymap::term::TermKeymap;
use crate::th;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use gitpoulpi_core::domain::{
    short, worktree_generated_name, worktree_id, worktree_name_after_branch, WorktreeAddOptions,
    WorktreeStart,
};
use gitpoulpi_i18n::msg;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use crate::combo::{Combo, ComboOption, ComboOutcome};
use crate::dialog_kit::{self, ButtonState, CheckState};
use crate::init_dialog::branch_name_error;
use crate::keymap::chord::Key;

/// What a context menu or a palette command hands the dialog: where the
/// worktree starts and how to word it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorktreeSeed {
    pub(crate) start: WorktreeStart,
    /// What the dialog calls the start: `feature`, `origin/feature`,
    /// `v1.0`, `a1b2c3d`.
    pub(crate) label: String,
    /// A branch name the caller wants for the worktree (a remote branch's
    /// local name); empty = the dialog proposes `worktree-<id>`.
    pub(crate) new_branch: String,
    /// The start branch is already checked out in a worktree, so git
    /// would refuse to check it out again: a new branch is required.
    pub(crate) branch_busy: bool,
}

/// A local branch the Ref combobox offers, and whether it is checked out in
/// a worktree already.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BranchChoice {
    pub(crate) name: String,
    pub(crate) busy: bool,
}

/// The next dialog opens with the branch combobox open ("Choose a branch...").
static PICK_REF: AtomicBool = AtomicBool::new(false);

/// "Choose a branch...": the dialog the next `worktree_seed` opens starts
/// on its Worktree branch list.
pub(crate) fn ask_to_pick_ref() {
    PICK_REF.store(true, Ordering::Relaxed);
}

/// The dialog's focusable parts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WtFocus {
    Name,
    NameAfter,
    Path,
    Browse,
    Ref,
    Branch,
    Regenerate,
    /// The ✦ button that opens or closes the description box.
    AiToggle,
    Describe,
    AiRun,
    AiHide,
    OpenTab,
    Create,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WorktreeOutcome {
    Stay,
    Cancel,
    /// Open the folder picker, starting here.
    Browse(PathBuf),
    /// Create the worktree.
    Submit {
        path: PathBuf,
        opts: WorktreeAddOptions,
        open_tab: bool,
    },
    /// Ask the AI to name the worktree and its branch for `description`;
    /// `branches` are the repository's local branch names (the prompt shows
    /// them so the answer follows their style).
    NameWithAi {
        description: String,
        branches: Vec<String>,
    },
    /// Stop the AI request that is running.
    StopAi,
}

#[derive(Debug, Clone)]
pub(crate) struct WorktreeDialog {
    pub(crate) seed: WorktreeSeed,
    /// The worktree's name: what the default folder is built from.
    pub(crate) name: String,
    /// The new worktree's folder.
    pub(crate) path: String,
    /// The Worktree branch field: a new branch's name, an existing local
    /// branch to check out, or empty (check the Ref out as it is).
    pub(crate) branch: String,
    pub(crate) open_tab: bool,
    pub(crate) focus: WtFocus,
    pub(crate) message: Option<String>,
    pub(crate) hover: Option<WtFocus>,
    /// The Name was typed by hand: Regenerate leaves it alone.
    name_typed: bool,
    /// The branch name was typed by hand (or came from the caller): Regenerate
    /// leaves it alone. An existing branch is never replaced either.
    branch_typed: bool,
    /// The Folder still follows the Name.
    path_auto: bool,
    repo_root: PathBuf,
    /// Where a new branch starts (the Ref combobox's choice).
    start: WorktreeStart,
    /// Whether the start branch is checked out elsewhere already.
    branch_busy: bool,
    branches: Vec<BranchChoice>,
    combo: Combo,
    branch_combo: Combo,
    /// AI is on (the host tells it every tick): the ✦ button exists.
    ai: bool,
    ai_open: bool,
    ai_text: String,
    ai_running: bool,
    ai_error: Option<String>,
    hits: Vec<(Rect, WtFocus)>,
    close: dialog_kit::CloseHit,
    /// What the fields held when the dialog opened (dirty = different).
    pristine: String,
    edit: crate::text_edit::FieldEditor<WtFocus>,
}

/// `name` made folder-safe: letters, digits, `-`, `_` and `.` stay, the rest
/// becomes `-`.
pub(crate) fn folder_safe(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || "-_.".contains(c) {
                c
            } else {
                '-'
            }
        })
        .collect()
}

/// `<repo parent>/<repo name>-<label>`, the label made folder-safe: a
/// sibling of the repository, named after what it holds.
pub(crate) fn default_worktree_path(repo_root: &Path, label: &str) -> PathBuf {
    let safe = folder_safe(label);
    let name = repo_root
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "worktree".into());
    let parent = repo_root.parent().unwrap_or(repo_root);
    parent.join(format!("{name}-{safe}"))
}

#[cfg(test)]
thread_local! {
    /// Ids the next dialogs draw instead of random ones (newest last).
    static TEST_IDS: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// A fresh 8-hex-digit id for a generated name.
fn next_id() -> String {
    #[cfg(test)]
    if let Some(id) = TEST_IDS.with(|ids| ids.borrow_mut().pop()) {
        return id;
    }
    let mut bytes = [0u8; 4];
    if getrandom::fill(&mut bytes).is_err() {
        // No OS randomness: the clock still differs between two draws.
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.subsec_nanos());
        bytes = nanos.to_le_bytes();
    }
    worktree_id(bytes)
}

/// The seed of "From the current branch": the checked-out branch cannot be
/// checked out twice, so the dialog proposes a new branch on it (the
/// dialog names it `worktree-<id>`); a detached HEAD starts a detached
/// worktree there.
pub(crate) fn seed_from_head(
    refs: &[gitpoulpi_core::domain::Ref],
    worktrees: &[gitpoulpi_core::domain::Worktree],
) -> Option<WorktreeSeed> {
    use gitpoulpi_core::domain::RefKind;
    if let Some(head) = refs.iter().find(|r| r.is_head) {
        if let RefKind::LocalBranch { name } = &head.kind {
            return Some(WorktreeSeed {
                start: WorktreeStart::Branch(name.clone()),
                label: name.clone(),
                new_branch: String::new(),
                branch_busy: true,
            });
        }
    }
    // Detached: the main worktree's HEAD.
    let _ = worktrees;
    let oid = crate::refs_sidebar::head_oid(refs)?;
    Some(WorktreeSeed {
        start: WorktreeStart::Commit(oid),
        label: short(oid),
        new_branch: String::new(),
        branch_busy: false,
    })
}

/// The seed of "Choose a branch...": the dialog opens on its branch list.
pub(crate) fn seed_to_choose(
    refs: &[gitpoulpi_core::domain::Ref],
    worktrees: &[gitpoulpi_core::domain::Worktree],
) -> Option<WorktreeSeed> {
    seed_from_head(refs, worktrees)
}

/// The Ref combobox's value for a start.
fn choice_value(start: &WorktreeStart) -> String {
    match start {
        WorktreeStart::Branch(b) => format!("b:{b}"),
        WorktreeStart::Commit(oid) => format!("c:{}", oid.to_hex()),
    }
}

/// Dialog heights: the default, and with the description box open.
const HEIGHT: u16 = 16;
const HEIGHT_AI: u16 = 22;

impl WorktreeDialog {
    /// What `Ctrl+N` badges: every field and button.
    pub(crate) fn nav_targets(&self) -> Vec<(Rect, (u16, u16))> {
        dialog_kit::nav_cells(self.hits.iter().map(|(r, _)| *r))
    }

    /// The dialog for `seed` with no other branch to choose from (the ref
    /// stays what the seed says). Tests and the stub-compatible entry.
    #[cfg(test)]
    pub(crate) fn new(seed: WorktreeSeed, repo_root: &Path) -> Self {
        Self::with_choices(seed, repo_root, Vec::new(), false)
    }

    /// The dialog for `seed`, whose Ref combobox lists `branches` and, when
    /// the seed starts on a commit, that commit. `pick_ref` opens the
    /// Worktree branch list ("Choose a branch...").
    pub(crate) fn with_choices(
        seed: WorktreeSeed,
        repo_root: &Path,
        branches: Vec<BranchChoice>,
        pick_ref: bool,
    ) -> Self {
        let generated = worktree_generated_name(&next_id());
        // What the branch field starts with: a branch the caller chose to
        // open (not checked out elsewhere) is checked out as it is, a name
        // the caller proposed stays, anything else gets the generated name.
        let (branch, branch_typed) = match &seed.start {
            WorktreeStart::Branch(b) if !seed.branch_busy => (b.clone(), true),
            WorktreeStart::Commit(_) if !seed.new_branch.is_empty() => {
                (seed.new_branch.clone(), true)
            }
            _ => (generated.clone(), false),
        };
        let mut dialog = Self {
            name: generated,
            path: String::new(),
            branch,
            focus: if pick_ref {
                WtFocus::Branch
            } else {
                WtFocus::Name
            },
            start: seed.start.clone(),
            branch_busy: seed.branch_busy,
            seed,
            open_tab: true,
            message: None,
            hover: None,
            name_typed: false,
            branch_typed,
            path_auto: true,
            repo_root: repo_root.to_path_buf(),
            branches,
            combo: Combo::new(msg::worktree::REF_HINT.get()),
            branch_combo: Combo::new(msg::worktree::BRANCH_FIELD.get()),
            ai: false,
            ai_open: false,
            ai_text: String::new(),
            ai_running: false,
            ai_error: None,
            hits: Vec::new(),
            close: Default::default(),
            pristine: String::new(),
            edit: Default::default(),
        };
        dialog.build_combo();
        dialog.build_branch_combo();
        dialog.derive_path();
        dialog.pristine = dialog.signature();
        if pick_ref {
            dialog.branch_combo.open();
        }
        dialog
    }

    /// The dialog for the next `worktree_seed`: the other branches come
    /// from the window's refs.
    pub(crate) fn take_pick_ref() -> bool {
        PICK_REF.swap(false, Ordering::Relaxed)
    }

    fn build_combo(&mut self) {
        let mut options: Vec<ComboOption> = self
            .branches
            .iter()
            .map(|b| ComboOption::new(b.name.clone(), format!("b:{}", b.name)))
            .collect();
        // The commit the dialog was opened on (a Log row, a tag, a remote
        // branch): offered as itself, first.
        if let WorktreeStart::Commit(oid) = &self.seed.start {
            options.insert(
                0,
                ComboOption::new(
                    msg::worktree::commit_choice(&self.seed.label, short(*oid)),
                    choice_value(&self.seed.start),
                ),
            );
        } else if let WorktreeStart::Branch(b) = &self.seed.start {
            if !self.branches.iter().any(|x| x.name == *b) {
                options.insert(0, ComboOption::new(b.clone(), format!("b:{b}")));
            }
        }
        self.combo = Combo::new(msg::worktree::REF_HINT.get()).with_search();
        self.combo.set_options(options);
        self.combo.select(Some(choice_value(&self.start)));
    }

    /// The Worktree branch list: "no branch" (the Ref as it is), then the
    /// local branches not checked out elsewhere; a typed name that is not
    /// listed is a new branch (`with_custom`).
    fn build_branch_combo(&mut self) {
        let mut options = vec![ComboOption::new(self.no_branch_label(), String::new())];
        let mut seen = std::collections::HashSet::new();
        for b in self.branches.iter().filter(|b| !b.busy) {
            if seen.insert(b.name.clone()) {
                options.push(ComboOption::new(b.name.clone(), b.name.clone()));
            }
        }
        if let WorktreeStart::Branch(b) = &self.seed.start {
            if !self.seed.branch_busy && seen.insert(b.clone()) {
                options.push(ComboOption::new(b.clone(), b.clone()));
            }
        }
        self.branch_combo = Combo::new(msg::worktree::BRANCH_FIELD.get())
            .with_search()
            .with_custom();
        self.branch_combo.set_options(options);
        self.branch_combo.select(Some(self.branch.clone()));
    }

    /// What an empty branch field means, as the list's first row.
    fn no_branch_label(&self) -> String {
        match &self.start {
            WorktreeStart::Branch(b) if !self.branch_busy => msg::worktree::branch_hint_checkout(b),
            WorktreeStart::Branch(_) => msg::worktree::BRANCH_HINT_REQUIRED.get().into(),
            WorktreeStart::Commit(_) => msg::worktree::BRANCH_HINT_DETACHED.get().into(),
        }
    }

    /// Whether `name` is a local branch: `Some(busy)` (checked out in
    /// another worktree) or `None` (not a branch here: a new one).
    fn branch_status(&self, name: &str) -> Option<bool> {
        if let Some(b) = self.branches.iter().find(|b| b.name == name) {
            return Some(b.busy);
        }
        match &self.seed.start {
            WorktreeStart::Branch(b) if b == name => Some(self.seed.branch_busy),
            _ => None,
        }
    }

    /// The branch field names an existing local branch that can be checked
    /// out: the Ref does not matter then.
    fn checks_out_existing(&self) -> bool {
        let branch = self.branch.trim();
        !branch.is_empty() && self.branch_status(branch) == Some(false)
    }

    /// What the new branch starts from, worded.
    fn start_label(&self) -> String {
        match &self.start {
            WorktreeStart::Branch(b) => b.clone(),
            WorktreeStart::Commit(oid) => {
                if matches!(&self.seed.start, WorktreeStart::Commit(s) if s == oid)
                    && !self.seed.label.is_empty()
                {
                    self.seed.label.clone()
                } else {
                    short(*oid)
                }
            }
        }
    }

    fn derive_path(&mut self) {
        if self.path_auto {
            self.path = default_worktree_path(&self.repo_root, self.name.trim())
                .display()
                .to_string();
        }
    }

    /// A sequence step's options: the Name (and so the Folder), where the
    /// worktree starts, the branch it gets and whether it opens as a tab.
    pub(crate) fn apply_run_seed(&mut self, run: &crate::repo_dialogs::WorktreeRunSeed) {
        self.name = run.name.clone();
        self.name_typed = true;
        self.derive_path();
        if let Some(reference) = &run.reference {
            self.pick_ref(&format!("b:{reference}"));
        }
        // A start that is checked out elsewhere needs a branch of its own,
        // named like the worktree; one that is free is checked out as it is.
        match (&run.reference, self.branch_busy) {
            (Some(reference), false) => self.set_branch(reference.clone(), true),
            _ => self.set_branch(run.name.clone(), true),
        }
        self.open_tab = run.open_tab;
        self.pristine = self.signature();
    }

    /// Sets the branch field (and the list's selection).
    fn set_branch(&mut self, branch: String, typed: bool) {
        self.branch_combo.select(Some(branch.clone()));
        self.branch = branch;
        self.branch_typed = typed;
        self.message = None;
    }

    /// The Ref combobox picked `value` (`b:<branch>` or `c:<oid>`): where a
    /// new branch starts. Names do not follow it.
    fn pick_ref(&mut self, value: &str) {
        if let Some(branch) = value.strip_prefix("b:") {
            self.start = WorktreeStart::Branch(branch.to_string());
            self.branch_busy = self
                .branches
                .iter()
                .find(|b| b.name == branch)
                .map(|b| b.busy)
                .unwrap_or(self.seed.branch_busy && self.seed.start == self.start);
        } else if matches!(&self.seed.start, WorktreeStart::Commit(_))
            && value == choice_value(&self.seed.start)
        {
            self.start = self.seed.start.clone();
            self.branch_busy = false;
        }
        self.combo.select(Some(choice_value(&self.start)));
        self.message = None;
        // "no branch" now means something else.
        let keep = self.branch.clone();
        let open = self.branch_combo.is_open();
        self.build_branch_combo();
        self.branch_combo.select(Some(keep));
        if open {
            self.branch_combo.open();
        }
    }

    /// The Worktree branch list picked or typed a value.
    fn branch_outcome(&mut self, outcome: ComboOutcome) {
        match outcome {
            ComboOutcome::Picked(value) => self.set_branch(value, false),
            ComboOutcome::Custom(text) => self.set_branch(text.trim().to_string(), true),
            _ => {}
        }
    }

    /// "Name after branch": the Name becomes `worktree-<branch>` (no
    /// doubled word), the Folder follows.
    fn name_after_branch(&mut self) {
        let name = worktree_name_after_branch(&self.branch);
        if name.is_empty() {
            self.message = Some(msg::worktree::NEED_BRANCH.get().into());
            return;
        }
        self.name = name;
        self.name_typed = false;
        self.derive_path();
        self.message = None;
    }

    /// "Regenerate": a new `worktree-<id>`. What it replaces:
    ///
    /// * the **Name**, unless it was typed by hand (a name from "Name after
    ///   branch", from the AI or from a previous Regenerate is replaced);
    /// * the **branch**, only when it is a generated or AI-proposed new
    ///   branch name: never one the user typed (or the caller proposed), nor
    ///   an existing branch, nor an empty field.
    ///
    /// The Folder follows the Name as always.
    pub(crate) fn regenerate(&mut self) {
        let generated = worktree_generated_name(&next_id());
        if !self.name_typed {
            self.name = generated.clone();
            self.derive_path();
        }
        let replaceable = !self.branch.trim().is_empty()
            && !self.branch_typed
            && self.branch_status(self.branch.trim()).is_none();
        if replaceable {
            self.set_branch(generated, false);
        }
        self.message = None;
    }

    fn signature(&self) -> String {
        format!(
            "{}\n{}\n{}\n{}\n{}",
            self.name,
            self.path,
            self.branch,
            choice_value(&self.start),
            self.open_tab
        )
    }

    /// The worktree Create would add, or why it cannot.
    pub(crate) fn plan(&self) -> Result<(PathBuf, WorktreeAddOptions), String> {
        let name = self.name.trim();
        if name.is_empty() {
            return Err(msg::worktree::NEED_NAME.get().into());
        }
        if name.contains(['/', '\\']) || name == "." || name == ".." {
            return Err(msg::worktree::BAD_NAME.get().into());
        }
        let path = self.path.trim();
        if path.is_empty() {
            return Err(msg::worktree::CHOOSE_FOLDER.get().into());
        }
        let path = PathBuf::from(path);
        if !path.is_absolute() {
            return Err(msg::worktree::NOT_ABSOLUTE.get().into());
        }
        if path.is_file() {
            return Err(msg::worktree::is_a_file(path.display()));
        }
        if std::fs::read_dir(&path).is_ok_and(|mut d| d.next().is_some()) {
            return Err(msg::worktree::not_empty(path.display()));
        }
        let branch = self.branch.trim();
        if branch.is_empty() {
            if self.branch_busy {
                let label = match &self.start {
                    WorktreeStart::Branch(b) => b.clone(),
                    WorktreeStart::Commit(_) => self.seed.label.clone(),
                };
                return Err(msg::worktree::already_checked_out(label));
            }
            return Ok((
                path,
                WorktreeAddOptions {
                    start: self.start.clone(),
                    new_branch: None,
                },
            ));
        }
        match self.branch_status(branch) {
            // An existing branch another worktree has: git would refuse.
            Some(true) => Err(msg::worktree::already_checked_out(branch)),
            // An existing branch: checked out as it is, whatever the Ref.
            Some(false) => Ok((
                path,
                WorktreeAddOptions {
                    start: WorktreeStart::Branch(branch.to_string()),
                    new_branch: None,
                },
            )),
            None => {
                if let Some(reason) = branch_name_error(branch) {
                    return Err(reason);
                }
                Ok((
                    path,
                    WorktreeAddOptions {
                        start: self.start.clone(),
                        new_branch: Some(branch.to_string()),
                    },
                ))
            }
        }
    }

    /// The parts `Tab` visits, in order.
    fn order(&self) -> Vec<WtFocus> {
        let mut order = vec![
            WtFocus::Name,
            WtFocus::NameAfter,
            WtFocus::Path,
            WtFocus::Browse,
            WtFocus::Ref,
            WtFocus::Branch,
            WtFocus::Regenerate,
        ];
        if self.ai {
            order.push(WtFocus::AiToggle);
            if self.ai_open {
                order.extend([WtFocus::Describe, WtFocus::AiRun, WtFocus::AiHide]);
            }
        }
        order.extend([WtFocus::OpenTab, WtFocus::Create, WtFocus::Cancel]);
        order
    }

    /// AI went on or off, or its request ended without an answer for us:
    /// the ✦ button and the busy state follow (called every tick).
    pub(crate) fn sync_ai(&mut self, enabled: bool, running: bool) {
        self.ai = enabled;
        if !enabled {
            self.ai_open = false;
            self.ai_running = false;
        } else if self.ai_running && !running {
            self.ai_running = false;
        }
        if !self.order().contains(&self.focus) {
            self.focus = WtFocus::Name;
        }
    }

    /// The AI answered (`Ok((slug, branch))`) or failed. The Name becomes
    /// `worktree-<slug>` (no doubled word) and the branch field the new
    /// branch, both still editable and replaceable by Regenerate. A branch
    /// that already exists is made unique, so the answer never silently
    /// checks something out. Returns whether the dialog was waiting for it.
    pub(crate) fn ai_done(&mut self, result: Result<(String, String), String>) -> bool {
        if !self.ai_running {
            return false;
        }
        self.ai_running = false;
        match result {
            Ok((slug, branch)) => {
                let mut unique = branch.clone();
                let mut n = 2;
                while self.branch_status(&unique).is_some() {
                    unique = format!("{branch}-{n}");
                    n += 1;
                }
                self.name = worktree_name_after_branch(&slug);
                self.name_typed = false;
                self.derive_path();
                self.set_branch(unique, false);
                self.ai_open = false;
                self.ai_error = None;
                self.focus = WtFocus::Name;
            }
            // Canceled: nothing to report.
            Err(why) if why == "canceled" => self.ai_error = None,
            Err(why) => self.ai_error = Some(msg::worktree::ai_failed(why)),
        }
        true
    }

    fn start_ai(&mut self) -> WorktreeOutcome {
        if self.ai_running {
            return WorktreeOutcome::StopAi;
        }
        let description = self.ai_text.trim().to_string();
        if description.is_empty() {
            self.ai_error = Some(msg::worktree::AI_NEED_TEXT.get().into());
            return WorktreeOutcome::Stay;
        }
        self.ai_error = None;
        self.ai_running = true;
        let mut branches: Vec<String> = self.branches.iter().map(|b| b.name.clone()).collect();
        if let WorktreeStart::Branch(b) = &self.seed.start {
            if !branches.contains(b) {
                branches.push(b.clone());
            }
        }
        WorktreeOutcome::NameWithAi {
            description,
            branches,
        }
    }

    /// Opens or closes the description box (hiding it stops a request).
    fn toggle_ai(&mut self) -> WorktreeOutcome {
        self.ai_open = !self.ai_open;
        if self.ai_open {
            self.focus = WtFocus::Describe;
            return WorktreeOutcome::Stay;
        }
        if self.focus == WtFocus::Describe
            || self.focus == WtFocus::AiRun
            || self.focus == WtFocus::AiHide
        {
            self.focus = WtFocus::AiToggle;
        }
        if self.ai_running {
            return WorktreeOutcome::StopAi;
        }
        WorktreeOutcome::Stay
    }

    fn field_parts(
        &mut self,
    ) -> (
        &mut crate::text_edit::FieldEditor<WtFocus>,
        Option<(WtFocus, &mut String)>,
    ) {
        let focus = self.focus;
        let field = match focus {
            WtFocus::Name => Some(&mut self.name),
            WtFocus::Path => Some(&mut self.path),
            WtFocus::Describe => Some(&mut self.ai_text),
            _ => None,
        };
        self.edit.sync(field.is_some().then_some(focus));
        (&mut self.edit, field.map(|f| (focus, f)))
    }

    /// A field's text changed by typing: the sync rules.
    fn edited(&mut self, field: WtFocus) {
        self.message = None;
        match field {
            WtFocus::Name => {
                self.name_typed = true;
                self.derive_path();
            }
            WtFocus::Path => self.path_auto = false,
            WtFocus::Describe => self.ai_error = None,
            _ => {}
        }
    }

    /// A folder chosen in the picker: the worktree goes *inside* it,
    /// under the name the Folder field already had - Browse chooses
    /// where, the name stays what it was.
    pub(crate) fn set_parent(&mut self, dir: &Path) {
        let leaf = Path::new(self.path.trim())
            .file_name()
            .map(|n| n.to_os_string())
            .unwrap_or_else(|| "worktree".into());
        self.path = dir.join(leaf).display().to_string();
        self.path_auto = false;
        self.focus = WtFocus::Branch;
        self.message = None;
    }

    pub(crate) fn paste(&mut self, text: &str) {
        if self.combo.is_open() {
            self.combo.paste(text);
            return;
        }
        if self.branch_combo.is_open() {
            self.branch_combo.paste(text);
            return;
        }
        let clean: String = text
            .chars()
            .map(|c| if c == '\n' || c == '\r' { ' ' } else { c })
            .filter(|c| !c.is_control())
            .collect();
        let focus = self.focus;
        if let (edit, Some((id, field))) = self.field_parts() {
            edit.caret_mut(id).insert(field, &clean);
            self.edited(focus);
        }
    }

    fn submit(&mut self) -> WorktreeOutcome {
        match self.plan() {
            Ok((path, opts)) => WorktreeOutcome::Submit {
                path,
                opts,
                open_tab: self.open_tab,
            },
            Err(reason) => {
                self.message = Some(reason);
                WorktreeOutcome::Stay
            }
        }
    }

    /// Where Browse starts: the Folder field's parent.
    fn browse(&self) -> WorktreeOutcome {
        let field = PathBuf::from(self.path.trim());
        WorktreeOutcome::Browse(field.parent().map(Path::to_path_buf).unwrap_or(field))
    }

    fn activate(&mut self, part: WtFocus) -> WorktreeOutcome {
        match part {
            WtFocus::Browse => self.browse(),
            WtFocus::Cancel => WorktreeOutcome::Cancel,
            WtFocus::OpenTab => {
                self.open_tab = !self.open_tab;
                WorktreeOutcome::Stay
            }
            WtFocus::Ref => {
                if !self.checks_out_existing() {
                    self.combo.open();
                }
                WorktreeOutcome::Stay
            }
            WtFocus::Branch => {
                self.branch_combo.open();
                WorktreeOutcome::Stay
            }
            WtFocus::NameAfter => {
                self.name_after_branch();
                WorktreeOutcome::Stay
            }
            WtFocus::Regenerate => {
                self.regenerate();
                WorktreeOutcome::Stay
            }
            WtFocus::AiToggle => self.toggle_ai(),
            WtFocus::AiRun => self.start_ai(),
            WtFocus::AiHide => {
                self.focus = WtFocus::AiToggle;
                self.ai_open = false;
                if self.ai_running {
                    WorktreeOutcome::StopAi
                } else {
                    WorktreeOutcome::Stay
                }
            }
            _ => self.submit(),
        }
    }

    pub(crate) fn on_key(&mut self, key: KeyEvent) -> WorktreeOutcome {
        if self.combo.is_open() {
            let outcome = self.combo.on_key(key);
            if let ComboOutcome::Picked(value) = outcome {
                self.pick_ref(&value);
            }
            return WorktreeOutcome::Stay;
        }
        if self.branch_combo.is_open() {
            let outcome = self.branch_combo.on_key(key);
            self.branch_outcome(outcome);
            return WorktreeOutcome::Stay;
        }
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        // The buttons' fixed Alt twins (shown on their labels).
        if key.modifiers.contains(KeyModifiers::ALT) {
            match key.code {
                KeyCode::Char('n' | 'N') => return self.activate(WtFocus::NameAfter),
                KeyCode::Char('r' | 'R') => return self.activate(WtFocus::Regenerate),
                KeyCode::Char('a' | 'A') if self.ai => return self.activate(WtFocus::AiToggle),
                _ => {}
            }
        }
        if self.focus == WtFocus::Ref
            && matches!(
                key.code,
                KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Down
            )
        {
            return self.activate(WtFocus::Ref);
        }
        if self.focus == WtFocus::Branch {
            match key.code {
                KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Down => {
                    return self.activate(WtFocus::Branch)
                }
                // Typing on the field starts the list's search line, where
                // a name that is not a branch is offered as a new one.
                KeyCode::Char(_)
                    if !key
                        .modifiers
                        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
                {
                    self.branch_combo.open();
                    let outcome = self.branch_combo.on_key(key);
                    self.branch_outcome(outcome);
                    return WorktreeOutcome::Stay;
                }
                _ => {}
            }
        }
        let focus = self.focus;
        if focus == WtFocus::Describe {
            if key.code == KeyCode::Esc {
                // Esc leaves the box (stopping a request), not the dialog.
                let stop = self.ai_running;
                self.ai_open = false;
                self.focus = WtFocus::AiToggle;
                return if stop {
                    WorktreeOutcome::StopAi
                } else {
                    WorktreeOutcome::Stay
                };
            }
            if let (edit, Some((id, field))) = self.field_parts() {
                // Enter is a newline here (Tab leaves the field).
                if key.code == KeyCode::Enter {
                    edit.caret_mut(id).insert(field, "\n");
                    self.edited(focus);
                    return WorktreeOutcome::Stay;
                }
                if edit.claims(id, field, &key, true) {
                    let result = edit.on_key(id, field, &key, true);
                    match result {
                        crate::text_edit::KeyEdit::Edited => self.edited(focus),
                        crate::text_edit::KeyEdit::Copy(s) | crate::text_edit::KeyEdit::Cut(s) => {
                            self.message = Some(crate::clipboard::copy_shared(
                                msg::pull_request::NOUN_SELECTION.get(),
                                &s,
                            ));
                        }
                        _ => {}
                    }
                    return WorktreeOutcome::Stay;
                }
            }
        } else if let (edit, Some((id, field))) = self.field_parts() {
            if let Some((result, copied)) = edit.dialog_key(id, field, &key) {
                if result == crate::text_edit::KeyEdit::Edited {
                    self.edited(focus);
                }
                if copied.is_some() {
                    self.message = copied;
                }
                return WorktreeOutcome::Stay;
            }
        }
        match key.code {
            KeyCode::Esc => {
                // A running request stops first; the dialog stays.
                if self.ai_running {
                    return WorktreeOutcome::StopAi;
                }
                return WorktreeOutcome::Cancel;
            }
            KeyCode::Tab | KeyCode::BackTab | KeyCode::Down | KeyCode::Up => {
                let order = self.order();
                let forward =
                    matches!(key.code, KeyCode::Down) || (key.code == KeyCode::Tab && !shift);
                let at = order.iter().position(|f| *f == self.focus).unwrap_or(0);
                let next = if forward {
                    (at + 1) % order.len()
                } else {
                    (at + order.len() - 1) % order.len()
                };
                self.focus = order[next];
            }
            _ if crate::keymap::current().is(crate::keymap::Action::DialogBrowse, &key) => {
                return self.browse()
            }
            // Enter on a button or the checkbox does what a click does;
            // on a field it is the dialog's default button (`activate`).
            KeyCode::Enter => return self.activate(self.focus),
            KeyCode::Char(' ') if self.field_parts().1.is_none() => {
                return self.activate(self.focus)
            }
            _ => {}
        }
        WorktreeOutcome::Stay
    }

    fn field_mouse(&mut self, mouse: MouseEvent) -> bool {
        match self.field_parts() {
            (edit, Some((id, field))) => {
                edit.dialog_mouse(Some(id), field, mouse.kind, mouse.column, mouse.row)
            }
            (edit, None) => edit.dialog_mouse(None, "", mouse.kind, mouse.column, mouse.row),
        }
    }

    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent) -> WorktreeOutcome {
        if self.combo.is_open() {
            let outcome = self.combo.on_mouse(mouse);
            if let ComboOutcome::Picked(value) = outcome {
                self.pick_ref(&value);
            }
            return WorktreeOutcome::Stay;
        }
        if self.branch_combo.is_open() {
            let outcome = self.branch_combo.on_mouse(mouse);
            self.branch_outcome(outcome);
            return WorktreeOutcome::Stay;
        }
        let hit = dialog_kit::hit_at(&self.hits, mouse.column, mouse.row);
        if !matches!(mouse.kind, MouseEventKind::Down(_)) && self.field_mouse(mouse) {
            return WorktreeOutcome::Stay;
        }
        match mouse.kind {
            MouseEventKind::Moved | MouseEventKind::Drag(_) => self.hover = hit,
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(part) = hit {
                    self.focus = part;
                    if !matches!(part, WtFocus::Name | WtFocus::Path | WtFocus::Describe) {
                        return self.activate(part);
                    }
                    self.field_mouse(mouse);
                }
            }
            _ => {}
        }
        WorktreeOutcome::Stay
    }

    /// The dialog's rectangle in `frame`: taller while the description box
    /// is open.
    pub(crate) fn rect(&self, frame: Rect) -> Rect {
        let height = if self.ai_open { HEIGHT_AI } else { HEIGHT };
        WIN.place(
            frame,
            dialog_kit::centered(frame, frame.width.saturating_sub(4).min(90), height),
        )
    }

    /// A button label with its Alt twin when the dialog is wide enough.
    fn labelled(&self, label: &str, key: char, wide: bool) -> String {
        if wide {
            format!("{label} {}", dialog_kit::alt_key(Key::Char(key)))
        } else {
            label.to_string()
        }
    }

    pub(crate) fn render(&mut self, rect: Rect, buf: &mut Buffer) {
        self.field_parts();
        self.edit.begin_frame();
        self.close.framed(
            buf,
            rect,
            &msg::worktree::title(&self.seed.label),
            &WIN,
            true,
        );
        let mut hits = Vec::new();
        let inner = Rect::new(
            rect.x + 2,
            rect.y + 1,
            rect.width.saturating_sub(4),
            rect.height.saturating_sub(2),
        );
        if inner.width < 30 || inner.height < 8 {
            dialog_kit::too_small::note(buf, rect);
            self.hits = hits;
            return;
        }
        let wide = inner.width >= 76;
        // The widest label plus a gap (14 at least), whatever the language.
        let label_w: u16 = [
            msg::worktree::NAME,
            msg::worktree::FOLDER,
            msg::worktree::REF_FIELD,
            msg::worktree::BRANCH_FIELD,
        ]
        .iter()
        .map(|m| crate::display_width(m.get()) as u16 + 2)
        .max()
        .unwrap_or(14)
        .max(14);
        let label = |buf: &mut Buffer, y: u16, text: &str| {
            dialog_kit::put(
                buf,
                inner.x,
                y,
                text,
                label_w,
                Style::default().fg(th().ui.muted).bg(th().ui.panel),
            );
        };
        let field_x = inner.x + label_w;
        let browse_w = dialog_kit::button_width(msg::dialog::BROWSE.get());
        let field_w = inner.width.saturating_sub(label_w);
        let normal = |part: WtFocus, hover: Option<WtFocus>, focus: WtFocus| ButtonState::Normal {
            hovered: hover == Some(part),
            focused: focus == part,
        };

        // Name, with "Name after branch" at its right.
        let mut y = inner.y;
        label(buf, y, msg::worktree::NAME.get());
        let after_label = self.labelled(msg::worktree::NAME_AFTER_BRANCH.get(), 'n', wide);
        let after_w = dialog_kit::button_width(&after_label);
        let name_w = field_w.saturating_sub(after_w + 1).max(8);
        let name = Rect::new(field_x, y, name_w, 1);
        dialog_kit::text_input(
            buf,
            name,
            &self.name,
            msg::worktree::NAME_HINT.get(),
            self.focus == WtFocus::Name,
            self.hover == Some(WtFocus::Name),
            (&self.edit, WtFocus::Name),
        );
        hits.push((name, WtFocus::Name));
        if name.right() + 1 + after_w <= inner.right() {
            let after = dialog_kit::button(
                buf,
                name.right() + 1,
                y,
                &after_label,
                normal(WtFocus::NameAfter, self.hover, self.focus),
            );
            hits.push((after, WtFocus::NameAfter));
        }

        y += 2;
        label(buf, y, msg::worktree::FOLDER.get());
        let path = Rect::new(field_x, y, field_w.saturating_sub(browse_w + 1), 1);
        dialog_kit::text_input(
            buf,
            path,
            &self.path,
            msg::worktree::FOLDER_HINT.get(),
            self.focus == WtFocus::Path,
            self.hover == Some(WtFocus::Path),
            (&self.edit, WtFocus::Path),
        );
        let browse = dialog_kit::button(
            buf,
            path.right() + 1,
            y,
            msg::dialog::BROWSE.get(),
            normal(WtFocus::Browse, self.hover, self.focus),
        );
        hits.push((browse, WtFocus::Browse));
        hits.push((path, WtFocus::Path));

        // Ref: the start of a new branch; unused while an existing branch
        // is chosen, and it says so.
        y += 2;
        label(buf, y, msg::worktree::REF_FIELD.get());
        let ref_rect = Rect::new(field_x, y, field_w, 1);
        if self.checks_out_existing() {
            let bg = crate::input_bg(false, false);
            dialog_kit::fill(buf, ref_rect, Style::default().bg(bg));
            dialog_kit::put(
                buf,
                ref_rect.x + 1,
                y,
                msg::worktree::REF_UNUSED.get(),
                ref_rect.width.saturating_sub(1),
                Style::default().fg(th().ui.disabled).bg(bg),
            );
        } else {
            self.combo.render_field(
                buf,
                ref_rect,
                self.focus == WtFocus::Ref,
                self.hover == Some(WtFocus::Ref),
            );
        }
        hits.push((ref_rect, WtFocus::Ref));

        // The branch, with Regenerate at its right.
        y += 2;
        label(buf, y, msg::worktree::BRANCH_FIELD.get());
        let regen_label = self.labelled(msg::worktree::REGENERATE.get(), 'r', wide);
        let regen_w = dialog_kit::button_width(&regen_label);
        let branch_w = field_w.saturating_sub(regen_w + 1).max(8);
        let branch = Rect::new(field_x, y, branch_w, 1);
        self.branch_combo.render_field(
            buf,
            branch,
            self.focus == WtFocus::Branch,
            self.hover == Some(WtFocus::Branch),
        );
        hits.push((branch, WtFocus::Branch));
        if branch.right() + 1 + regen_w <= inner.right() {
            let regen = dialog_kit::button(
                buf,
                branch.right() + 1,
                y,
                &regen_label,
                normal(WtFocus::Regenerate, self.hover, self.focus),
            );
            hits.push((regen, WtFocus::Regenerate));
        }

        // What the branch field means now, and the ✦ button at its right.
        y += 1;
        let hint = if self.branch.trim().is_empty() {
            String::new()
        } else if self.checks_out_existing() {
            msg::worktree::HINT_EXISTING_BRANCH.get().to_string()
        } else if self.branch_status(self.branch.trim()).is_none() {
            msg::worktree::hint_new_branch(self.start_label())
        } else {
            String::new()
        };
        let mut hint_room = field_w;
        if self.ai {
            let ai_label = format!(
                "{} {}",
                crate::theme::AI_SPARK,
                self.labelled(msg::worktree::AI_BUTTON.get(), 'a', wide)
            );
            let w = dialog_kit::button_width(&ai_label);
            let x = inner.right().saturating_sub(w);
            let rect = dialog_kit::button(
                buf,
                x,
                y,
                &ai_label,
                normal(WtFocus::AiToggle, self.hover, self.focus),
            );
            // The sparkle keeps the AI purple, as on every AI control.
            for col in rect.x..rect.right() {
                if let Some(cell) = buf.cell_mut((col, rect.y)) {
                    if cell.symbol() == crate::theme::AI_SPARK {
                        cell.set_fg(th().status.ai);
                    }
                }
            }
            hits.push((rect, WtFocus::AiToggle));
            hint_room = x.saturating_sub(field_x + 1);
        }
        dialog_kit::put(
            buf,
            field_x,
            y,
            &crate::tail_fit(&hint, hint_room as usize),
            hint_room,
            Style::default().fg(th().ui.muted).bg(th().ui.panel),
        );

        let bottom = inner.y + inner.height;
        let plan_y = bottom.saturating_sub(3);
        let check_y = plan_y.saturating_sub(2);

        // The description box, between the hint and the checkbox.
        if self.ai && self.ai_open {
            let top = y + 1;
            dialog_kit::put(
                buf,
                inner.x,
                top,
                msg::worktree::AI_DESCRIBE.get(),
                inner.width,
                Style::default().fg(th().ui.muted).bg(th().ui.panel),
            );
            let room = check_y.saturating_sub(top + 1);
            let area_h = room.saturating_sub(2).clamp(1, 3);
            let area = Rect::new(inner.x, top + 1, inner.width, area_h);
            dialog_kit::text_area(
                buf,
                area,
                &self.ai_text,
                msg::worktree::AI_DESCRIBE_HINT.get(),
                self.focus == WtFocus::Describe,
                self.hover == Some(WtFocus::Describe),
                (&self.edit, WtFocus::Describe),
            );
            hits.push((area, WtFocus::Describe));
            let by = area.bottom();
            let run_label = if self.ai_running {
                msg::worktree::AI_STOP.get()
            } else {
                msg::worktree::AI_RUN.get()
            };
            let run = dialog_kit::button(
                buf,
                inner.x,
                by,
                run_label,
                ButtonState::Primary {
                    hovered: self.hover == Some(WtFocus::AiRun),
                    focused: self.focus == WtFocus::AiRun,
                },
            );
            hits.push((run, WtFocus::AiRun));
            let hide = dialog_kit::button(
                buf,
                run.right() + 1,
                by,
                msg::worktree::AI_HIDE.get(),
                normal(WtFocus::AiHide, self.hover, self.focus),
            );
            hits.push((hide, WtFocus::AiHide));
            let status_x = hide.right() + 2;
            let status_room = inner.right().saturating_sub(status_x);
            if self.ai_running {
                dialog_kit::put(
                    buf,
                    status_x,
                    by,
                    &format!(
                        "{} {}",
                        crate::theme::AI_SPARK,
                        msg::worktree::AI_WORKING.get()
                    ),
                    status_room,
                    Style::default().fg(th().status.ai).bg(th().ui.panel),
                );
            } else if let Some(error) = &self.ai_error {
                dialog_kit::put(
                    buf,
                    status_x,
                    by,
                    error,
                    status_room,
                    Style::default().fg(th().status.danger).bg(th().ui.panel),
                );
            }
        }

        let open_tab = dialog_kit::checkbox(
            buf,
            field_x,
            check_y,
            field_w,
            msg::worktree::OPEN_TAB.get(),
            self.open_tab,
            CheckState::Enabled {
                hovered: self.hover == Some(WtFocus::OpenTab),
                focused: self.focus == WtFocus::OpenTab,
            },
        );
        hits.push((open_tab, WtFocus::OpenTab));

        let plan = self.plan();
        match &plan {
            Ok((path, opts)) => {
                let on = match (&opts.new_branch, &opts.start) {
                    (Some(n), _) => msg::worktree::on_new_branch(n),
                    (None, WorktreeStart::Branch(b)) => msg::worktree::on_branch(b),
                    (None, WorktreeStart::Commit(_)) => msg::worktree::DETACHED.get().into(),
                };
                let x = inner.x
                    + dialog_kit::put(
                        buf,
                        inner.x,
                        plan_y,
                        msg::worktree::WILL_CREATE.get(),
                        inner.width,
                        Style::default().fg(th().ui.muted).bg(th().ui.panel),
                    )
                    .width;
                let room = inner.right().saturating_sub(x);
                let text = format!("{}{on}", path.display());
                dialog_kit::put(
                    buf,
                    x,
                    plan_y,
                    &crate::tail_fit(&text, room as usize),
                    room,
                    Style::default().fg(th().ui.text).bg(th().ui.panel),
                );
            }
            Err(reason) => {
                dialog_kit::put(
                    buf,
                    inner.x,
                    plan_y,
                    reason,
                    inner.width,
                    Style::default().fg(th().status.warning).bg(th().ui.panel),
                );
            }
        }
        if let Some(message) = &self.message {
            dialog_kit::put(
                buf,
                inner.x,
                plan_y + 1,
                message,
                inner.width,
                Style::default().fg(th().status.danger).bg(th().ui.panel),
            );
        }

        let footer = bottom.saturating_sub(1);
        // Right-aligned; a narrow dialog clips the labels, never drops one.
        use dialog_kit::button_row::Btn;
        let [cancel, create] = dialog_kit::button_row::paint(
            buf,
            inner.right(),
            footer,
            inner.width,
            0,
            &[
                Btn {
                    label: msg::dialog::CANCEL.get(),
                    state: normal(WtFocus::Cancel, self.hover, self.focus),
                },
                Btn {
                    label: msg::dialog::CREATE.get(),
                    state: if plan.is_err() {
                        ButtonState::Disabled
                    } else {
                        ButtonState::Primary {
                            hovered: self.hover == Some(WtFocus::Create),
                            focused: self.focus == WtFocus::Create,
                        }
                    },
                },
            ],
        )[..] else {
            unreachable!("two buttons in, two rects out")
        };
        let cancel_x = cancel.x;
        hits.insert(0, (cancel, WtFocus::Cancel));
        hits.insert(0, (create, WtFocus::Create));
        let browse = dialog_kit::key_label(crate::keymap::Action::DialogBrowse);
        let tab = dialog_kit::key_name(Key::Tab);
        let enter = dialog_kit::key_name(Key::Enter);
        let space = dialog_kit::key_name(Key::Space);
        dialog_kit::hints(
            buf,
            inner.x,
            footer,
            cancel_x.saturating_sub(inner.x + 1),
            &[
                (tab.as_str(), msg::dialog::HINT_NEXT.get()),
                (enter.as_str(), msg::dialog::HINT_CREATE.get()),
                (browse.as_str(), msg::dialog::HINT_BROWSE.get()),
                (space.as_str(), msg::dialog::HINT_TOGGLE.get()),
            ],
        );
        self.hits = hits;
        // The lists hang over everything else.
        self.combo.render_popup(buf, buf.area);
        self.branch_combo.render_popup(buf, buf.area);
    }
}

/// The dialog's size in a frame.
impl dialog_kit::Dismiss for WorktreeDialog {
    fn is_dirty(&self) -> bool {
        self.signature() != self.pristine
    }

    fn close_hit(&self) -> &dialog_kit::CloseHit {
        &self.close
    }

    fn close_hit_mut(&mut self) -> &mut dialog_kit::CloseHit {
        &mut self.close
    }
}

const WIN: dialog_kit::Win = dialog_kit::Win::movable("worktree");

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::{is_generated_worktree_name, Oid};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn alt(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::ALT)
    }

    fn seed(start: WorktreeStart, label: &str, busy: bool) -> WorktreeSeed {
        WorktreeSeed {
            start,
            label: label.into(),
            new_branch: String::new(),
            branch_busy: busy,
        }
    }

    /// The next dialogs draw these ids (in this order).
    fn ids(list: &[&str]) {
        TEST_IDS.with(|ids| {
            let mut ids = ids.borrow_mut();
            ids.clear();
            ids.extend(list.iter().rev().map(|s| s.to_string()));
        });
    }

    fn two_branches() -> Vec<BranchChoice> {
        vec![
            BranchChoice {
                name: "main".into(),
                busy: true,
            },
            BranchChoice {
                name: "other".into(),
                busy: false,
            },
            BranchChoice {
                name: "feature/login".into(),
                busy: false,
            },
        ]
    }

    fn on_main(root: &Path) -> WorktreeDialog {
        WorktreeDialog::with_choices(
            seed(WorktreeStart::Branch("main".into()), "main", true),
            root,
            two_branches(),
            false,
        )
    }

    fn type_into(dialog: &mut WorktreeDialog, focus: WtFocus, text: &str) {
        dialog.focus = focus;
        for c in text.chars() {
            dialog.on_key(key(KeyCode::Char(c)));
        }
    }

    #[test]
    fn it_opens_with_the_same_generated_name_for_the_worktree_and_the_branch() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("repo");
        // Real randomness: eight lowercase hex digits, the same for both.
        let dialog = on_main(&root);
        assert!(is_generated_worktree_name(&dialog.name), "{}", dialog.name);
        assert_eq!(dialog.branch, dialog.name);
        assert_eq!(
            PathBuf::from(&dialog.path),
            tmp.path().join(format!("repo-{}", dialog.name))
        );
        // Two dialogs do not share an id.
        assert_ne!(on_main(&root).name, dialog.name);
        let (_, opts) = dialog.plan().unwrap();
        assert_eq!(opts.new_branch.as_deref(), Some(dialog.name.as_str()));
        assert_eq!(opts.start, WorktreeStart::Branch("main".into()));
    }

    #[test]
    fn a_branch_the_caller_opens_is_checked_out_as_it_is() {
        let tmp = tempfile::tempdir().unwrap();
        ids(&["0a0a0a0a"]);
        let dialog = WorktreeDialog::with_choices(
            seed(WorktreeStart::Branch("other".into()), "other", false),
            &tmp.path().join("repo"),
            two_branches(),
            false,
        );
        // The Name keeps its default; the branch field shows the branch.
        assert_eq!(dialog.name, "worktree-0a0a0a0a");
        assert_eq!(dialog.branch, "other");
        assert!(dialog.checks_out_existing());
        let (_, opts) = dialog.plan().unwrap();
        assert_eq!(opts.start, WorktreeStart::Branch("other".into()));
        assert_eq!(opts.new_branch, None);
    }

    #[test]
    fn a_commit_gets_a_generated_branch_unless_the_caller_names_one() {
        let tmp = tempfile::tempdir().unwrap();
        let oid = Oid::from_hex(&"c".repeat(40)).unwrap();
        ids(&["11111111", "22222222"]);
        let dialog = WorktreeDialog::with_choices(
            seed(WorktreeStart::Commit(oid), "ccccccc", false),
            &tmp.path().join("repo"),
            two_branches(),
            false,
        );
        assert_eq!(dialog.branch, "worktree-11111111");
        let (_, opts) = dialog.plan().unwrap();
        assert_eq!(opts.start, WorktreeStart::Commit(oid));
        assert_eq!(opts.new_branch.as_deref(), Some("worktree-11111111"));
        assert!(dialog
            .combo
            .options
            .iter()
            .any(|o| o.value == format!("c:{}", oid.to_hex())));
        // A remote branch's local name stays, and Regenerate keeps it.
        let mut remote = WorktreeDialog::with_choices(
            WorktreeSeed {
                new_branch: "feature".into(),
                ..seed(WorktreeStart::Commit(oid), "origin/feature", false)
            },
            &tmp.path().join("repo"),
            two_branches(),
            false,
        );
        assert_eq!(remote.branch, "feature");
        remote.regenerate();
        assert_eq!(remote.branch, "feature");
        // Clearing the branch makes the worktree detached.
        remote.set_branch(String::new(), true);
        let (_, opts) = remote.plan().unwrap();
        assert_eq!(opts.new_branch, None);
        assert_eq!(opts.start, WorktreeStart::Commit(oid));
    }

    #[test]
    fn the_folder_follows_the_name_until_the_folder_is_edited() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("repo");
        ids(&["aaaaaaaa"]);
        let mut dialog = on_main(&root);
        assert_eq!(
            PathBuf::from(&dialog.path),
            tmp.path().join("repo-worktree-aaaaaaaa")
        );
        // Typing in Name re-proposes the Folder.
        type_into(&mut dialog, WtFocus::Name, "-2");
        assert_eq!(dialog.name, "worktree-aaaaaaaa-2");
        assert_eq!(
            PathBuf::from(&dialog.path),
            tmp.path().join("repo-worktree-aaaaaaaa-2")
        );
        // Editing the Folder by hand stops the sync for good.
        type_into(&mut dialog, WtFocus::Path, "x");
        let edited = dialog.path.clone();
        assert!(edited.ends_with("aaaaaaaa-2x"), "{edited}");
        type_into(&mut dialog, WtFocus::Name, "y");
        assert_eq!(
            dialog.path, edited,
            "the Folder was edited: Name no longer moves it"
        );
    }

    #[test]
    fn name_after_branch_uses_the_branch_without_doubling_the_word() {
        let tmp = tempfile::tempdir().unwrap();
        let mut dialog = on_main(&tmp.path().join("repo"));
        // An existing branch with a slash.
        dialog.set_branch("feature/login".into(), false);
        dialog.activate(WtFocus::NameAfter);
        assert_eq!(dialog.name, "worktree-feature-login");
        assert_eq!(
            PathBuf::from(&dialog.path),
            tmp.path().join("repo-worktree-feature-login")
        );
        // The word is already there: used as it is.
        for (branch, name) in [
            ("worktree-foo", "worktree-foo"),
            ("worktree", "worktree"),
            ("topic/worktree-x", "topic-worktree-x"),
            ("topic", "worktree-topic"),
        ] {
            dialog.set_branch(branch.into(), true);
            dialog.on_key(alt('n'));
            assert_eq!(dialog.name, name, "{branch}");
        }
        // No branch: nothing to name it after.
        dialog.set_branch(String::new(), true);
        dialog.activate(WtFocus::NameAfter);
        assert!(dialog.message.is_some());
        assert_eq!(dialog.name, "worktree-topic");
    }

    #[test]
    fn regenerate_replaces_only_what_was_generated() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("repo");

        // Both generated: both change, the same way, and the Folder follows.
        ids(&["00000001", "00000002"]);
        let mut dialog = on_main(&root);
        assert_eq!(dialog.name, "worktree-00000001");
        assert_eq!(dialog.branch, "worktree-00000001");
        dialog.on_key(alt('r'));
        assert_eq!(dialog.name, "worktree-00000002");
        assert_eq!(dialog.branch, "worktree-00000002");
        assert!(
            dialog.path.ends_with("repo-worktree-00000002"),
            "{}",
            dialog.path
        );

        // A name typed by hand stays; the generated branch still changes.
        ids(&["00000003", "00000004"]);
        let mut dialog = on_main(&root);
        type_into(&mut dialog, WtFocus::Name, "-mine");
        dialog.activate(WtFocus::Regenerate);
        assert_eq!(dialog.name, "worktree-00000003-mine");
        assert_eq!(dialog.branch, "worktree-00000004");

        // A branch typed by hand stays; the generated name changes.
        ids(&["00000005", "00000006"]);
        let mut dialog = on_main(&root);
        dialog.set_branch("my-topic".into(), true);
        dialog.activate(WtFocus::Regenerate);
        assert_eq!(dialog.name, "worktree-00000006");
        assert_eq!(dialog.branch, "my-topic");

        // An existing branch is never replaced (even if nobody "typed" it).
        ids(&["00000007", "00000008"]);
        let mut dialog = on_main(&root);
        dialog.set_branch("other".into(), false);
        dialog.activate(WtFocus::Regenerate);
        assert_eq!(dialog.branch, "other");
        assert_eq!(dialog.name, "worktree-00000008");

        // A name from "Name after branch" is replaced like a generated one.
        ids(&["00000009", "0000000a"]);
        let mut dialog = on_main(&root);
        dialog.set_branch("other".into(), false);
        dialog.activate(WtFocus::NameAfter);
        assert_eq!(dialog.name, "worktree-other");
        dialog.activate(WtFocus::Regenerate);
        assert_eq!(dialog.name, "worktree-0000000a");
    }

    #[test]
    fn the_branch_list_picks_an_existing_branch_or_makes_a_new_one() {
        let tmp = tempfile::tempdir().unwrap();
        let mut dialog = on_main(&tmp.path().join("repo"));
        // The list holds "no branch" and the branches free to check out.
        let values: Vec<&str> = dialog
            .branch_combo
            .options
            .iter()
            .map(|o| o.value.as_str())
            .collect();
        assert_eq!(values, ["", "other", "feature/login"]);
        // Picking `other` checks it out: the Ref says it is not used.
        dialog.branch_outcome(ComboOutcome::Picked("other".into()));
        assert_eq!(dialog.branch, "other");
        assert!(dialog.checks_out_existing());
        let (_, opts) = dialog.plan().unwrap();
        assert_eq!(opts.start, WorktreeStart::Branch("other".into()));
        assert_eq!(opts.new_branch, None);
        // The Ref does not open while it is unused.
        dialog.focus = WtFocus::Ref;
        dialog.on_key(key(KeyCode::Enter));
        assert!(!dialog.combo.is_open());
        // A typed name is a new branch from the Ref.
        dialog.branch_outcome(ComboOutcome::Custom(" topic/new ".into()));
        assert_eq!(dialog.branch, "topic/new");
        assert!(!dialog.checks_out_existing());
        let (_, opts) = dialog.plan().unwrap();
        assert_eq!(opts.start, WorktreeStart::Branch("main".into()));
        assert_eq!(opts.new_branch.as_deref(), Some("topic/new"));
        // A branch another worktree has cannot be checked out twice.
        dialog.branch_outcome(ComboOutcome::Custom("main".into()));
        assert!(dialog.plan().unwrap_err().contains("already checked out"));
        // Not a valid name.
        dialog.branch_outcome(ComboOutcome::Custom("bad name".into()));
        assert!(dialog
            .plan()
            .unwrap_err()
            .contains("not a valid branch name"));
    }

    #[test]
    fn typing_on_the_branch_field_opens_its_list_with_the_search() {
        let tmp = tempfile::tempdir().unwrap();
        let mut dialog = on_main(&tmp.path().join("repo"));
        dialog.focus = WtFocus::Branch;
        dialog.on_key(key(KeyCode::Char('o')));
        assert!(dialog.branch_combo.is_open());
        for c in "th".chars() {
            dialog.on_key(key(KeyCode::Char(c)));
        }
        // Enter picks the first match: `other`.
        dialog.on_key(key(KeyCode::Enter));
        assert_eq!(dialog.branch, "other");
        assert!(!dialog.branch_combo.is_open());
    }

    #[test]
    fn choosing_a_ref_moves_where_a_new_branch_starts_and_nothing_else() {
        let tmp = tempfile::tempdir().unwrap();
        ids(&["12121212"]);
        let mut dialog = on_main(&tmp.path().join("repo"));
        dialog.pick_ref("b:other");
        assert_eq!(dialog.start, WorktreeStart::Branch("other".into()));
        assert_eq!(dialog.name, "worktree-12121212");
        assert_eq!(dialog.branch, "worktree-12121212");
        let (_, opts) = dialog.plan().unwrap();
        assert_eq!(opts.start, WorktreeStart::Branch("other".into()));
        assert_eq!(opts.new_branch.as_deref(), Some("worktree-12121212"));
    }

    #[test]
    fn the_ref_on_a_busy_branch_still_asks_for_a_branch_when_the_field_is_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let mut dialog = on_main(&tmp.path().join("repo"));
        dialog.set_branch(String::new(), true);
        assert!(dialog.plan().unwrap_err().contains("name a new branch"));
        dialog.pick_ref("b:other");
        let (_, opts) = dialog.plan().unwrap();
        assert_eq!(opts.start, WorktreeStart::Branch("other".into()));
        assert_eq!(opts.new_branch, None);
    }

    #[test]
    fn enter_submits_with_the_branch_and_the_tab_choice() {
        let tmp = tempfile::tempdir().unwrap();
        let oid = Oid::from_hex(&"a".repeat(40)).unwrap();
        let mut dialog = WorktreeDialog::new(
            seed(WorktreeStart::Commit(oid), "aaaaaaa", false),
            tmp.path(),
        );
        dialog.set_branch("topic".into(), true);
        dialog.focus = WtFocus::OpenTab;
        dialog.on_key(key(KeyCode::Char(' ')));
        assert!(!dialog.open_tab);
        dialog.focus = WtFocus::Create;
        match dialog.on_key(key(KeyCode::Enter)) {
            WorktreeOutcome::Submit { opts, open_tab, .. } => {
                assert_eq!(opts.new_branch.as_deref(), Some("topic"));
                assert_eq!(opts.start, WorktreeStart::Commit(oid));
                assert!(!open_tab);
            }
            other => panic!("expected a submit, got {other:?}"),
        }
    }

    #[test]
    fn refusals_say_why_and_submit_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join("full")).unwrap();
        std::fs::write(tmp.path().join("full").join("x"), "x").unwrap();
        let mut dialog = WorktreeDialog::with_choices(
            seed(WorktreeStart::Branch("feature".into()), "feature", false),
            tmp.path(),
            Vec::new(),
            false,
        );
        dialog.path = tmp.path().join("full").display().to_string();
        assert!(dialog.plan().unwrap_err().contains("not empty"));
        dialog.focus = WtFocus::Create;
        assert_eq!(dialog.on_key(key(KeyCode::Enter)), WorktreeOutcome::Stay);
        assert!(dialog.message.as_deref().unwrap().contains("not empty"));
        dialog.path = "relative".into();
        assert!(dialog.plan().unwrap_err().contains("full path"));
        dialog.name = "a/b".into();
        assert!(dialog.plan().unwrap_err().contains("slashes"));
        dialog.name = String::new();
        assert!(dialog.plan().unwrap_err().contains("name"));
        let file = tmp.path().join("file");
        std::fs::write(&file, "x").unwrap();
        dialog.name = "ok".into();
        dialog.path = file.display().to_string();
        assert!(dialog.plan().unwrap_err().contains("is a file"));
    }

    #[test]
    fn browse_starts_at_the_folders_parent_and_keeps_the_name() {
        let tmp = tempfile::tempdir().unwrap();
        ids(&["abcdef01"]);
        let mut dialog = on_main(&tmp.path().join("repo"));
        assert_eq!(
            dialog.on_key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL)),
            WorktreeOutcome::Browse(tmp.path().to_path_buf())
        );
        dialog.set_parent(&tmp.path().join("elsewhere"));
        assert_eq!(
            PathBuf::from(&dialog.path),
            tmp.path().join("elsewhere").join("repo-worktree-abcdef01")
        );
    }

    #[test]
    fn the_seed_for_the_current_branch_is_busy_and_proposes_no_name_of_its_own() {
        use gitpoulpi_core::domain::{Ref, RefKind};
        let branch = |name: &str, head: bool| Ref {
            kind: RefKind::LocalBranch { name: name.into() },
            target: Oid([1; 20]),
            visibility: Default::default(),
            is_head: head,
            upstream: None,
            tag_sync: None,
            multi_remote: None,
        };
        let refs = vec![branch("main", true), branch("main-wt", false)];
        let seed = seed_from_head(&refs, &[]).unwrap();
        assert_eq!(seed.start, WorktreeStart::Branch("main".into()));
        assert_eq!(seed.new_branch, "", "the dialog proposes worktree-<id>");
        assert!(seed.branch_busy);
        assert_eq!(seed_to_choose(&refs, &[]).unwrap(), seed);
    }

    #[test]
    fn it_renders_and_every_button_is_where_it_is_hit() {
        let tmp = tempfile::tempdir().unwrap();
        for width in [72u16, 140] {
            ids(&["feedbeef"]);
            let mut dialog = WorktreeDialog::with_choices(
                seed(WorktreeStart::Branch("feature".into()), "feature", false),
                &tmp.path().join("repo"),
                Vec::new(),
                false,
            );
            dialog.sync_ai(true, false);
            let mut buf = Buffer::empty(Rect::new(0, 0, width, 30));
            let rect = dialog.rect(buf.area);
            dialog.render(rect, &mut buf);
            let text = dialog_kit::buffer_text(&buf);
            for needle in [
                "Create worktree from feature",
                "Folder",
                "Worktree branch",
                "Name after branch",
                "Regenerate",
                "Name with AI",
                "checks out this branch",
                "not used",
                "Open the new worktree as a tab",
                "Will create",
                "Browse",
                "Create",
                "Cancel",
            ] {
                assert!(text.contains(needle), "{width}: {needle}\n{text}");
            }
            let at = |dialog: &WorktreeDialog, part| {
                let (r, _) = dialog.hits.iter().find(|(_, p)| *p == part).unwrap();
                MouseEvent {
                    kind: MouseEventKind::Down(MouseButton::Left),
                    column: r.x + 1,
                    row: r.y,
                    modifiers: KeyModifiers::NONE,
                }
            };
            assert!(matches!(
                dialog.on_mouse(at(&dialog, WtFocus::Browse)),
                WorktreeOutcome::Browse(_)
            ));
            assert_eq!(
                dialog.on_mouse(at(&dialog, WtFocus::OpenTab)),
                WorktreeOutcome::Stay
            );
            assert!(!dialog.open_tab, "a click on the checkbox toggles it");
            // Name after branch: `feature` is the branch field's value.
            dialog.on_mouse(at(&dialog, WtFocus::NameAfter));
            assert_eq!(dialog.name, "worktree-feature");
            // Regenerate leaves the caller's branch and renames the Name.
            dialog.on_mouse(at(&dialog, WtFocus::Regenerate));
            assert_ne!(dialog.name, "worktree-feature");
            assert_eq!(dialog.branch, "feature");
            assert!(matches!(
                dialog.on_mouse(at(&dialog, WtFocus::Create)),
                WorktreeOutcome::Submit { .. }
            ));
            assert_eq!(
                dialog.on_mouse(at(&dialog, WtFocus::Cancel)),
                WorktreeOutcome::Cancel
            );
        }
    }

    /// Draws the dialog and finds `part`'s rect.
    fn drawn(dialog: &mut WorktreeDialog, part: WtFocus) -> Rect {
        let mut buf = Buffer::empty(Rect::new(0, 0, 140, 40));
        let rect = dialog.rect(buf.area);
        dialog.render(rect, &mut buf);
        dialog.hits.iter().find(|(_, p)| *p == part).unwrap().0
    }

    #[test]
    fn the_ai_button_exists_only_while_ai_is_on() {
        let tmp = tempfile::tempdir().unwrap();
        let mut dialog = on_main(&tmp.path().join("repo"));
        assert!(!dialog.order().contains(&WtFocus::AiToggle));
        dialog.sync_ai(true, false);
        assert!(dialog.order().contains(&WtFocus::AiToggle));
        dialog.activate(WtFocus::AiToggle);
        assert_eq!(dialog.focus, WtFocus::Describe);
        assert!(dialog.order().contains(&WtFocus::Describe));
        // Switched off while open: the box goes and the focus moves.
        dialog.sync_ai(false, false);
        assert!(!dialog.ai_open);
        assert_eq!(dialog.focus, WtFocus::Name);
        // The box makes the dialog taller.
        let frame = Rect::new(0, 0, 140, 40);
        let closed = dialog.rect(frame).height;
        dialog.sync_ai(true, false);
        dialog.activate(WtFocus::AiToggle);
        assert!(dialog.rect(frame).height > closed);
    }

    #[test]
    fn describing_the_work_asks_the_ai_and_the_answer_fills_both_fields() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("repo");
        let mut dialog = on_main(&root);
        dialog.sync_ai(true, false);
        // Nothing described yet: an inline reason, no request.
        dialog.activate(WtFocus::AiToggle);
        assert_eq!(dialog.activate(WtFocus::AiRun), WorktreeOutcome::Stay);
        assert!(dialog.ai_error.is_some());
        // Alt+A twin, typed text (Enter is a newline), then the button.
        type_into(&mut dialog, WtFocus::Describe, "fix the crash");
        dialog.on_key(key(KeyCode::Enter));
        type_into(&mut dialog, WtFocus::Describe, "on empty config");
        assert_eq!(dialog.ai_text, "fix the crash\non empty config");
        let out = dialog.activate(WtFocus::AiRun);
        let WorktreeOutcome::NameWithAi {
            description,
            branches,
        } = out
        else {
            panic!("{out:?}")
        };
        assert_eq!(description, "fix the crash\non empty config");
        assert_eq!(branches, ["main", "other", "feature/login"]);
        assert!(dialog.ai_running);
        // The same button now stops it; Esc stops it without closing.
        assert_eq!(dialog.activate(WtFocus::AiRun), WorktreeOutcome::StopAi);
        dialog.focus = WtFocus::Name;
        assert_eq!(dialog.on_key(key(KeyCode::Esc)), WorktreeOutcome::StopAi);

        // The answer: Name as worktree-<slug>, the branch as a new branch.
        assert!(dialog.ai_done(Ok((
            "crash-on-empty-config".into(),
            "fix/empty-config".into()
        ))));
        assert_eq!(dialog.name, "worktree-crash-on-empty-config");
        assert_eq!(dialog.branch, "fix/empty-config");
        assert!(dialog.path.ends_with("repo-worktree-crash-on-empty-config"));
        assert!(!dialog.ai_open && !dialog.ai_running);
        let (_, opts) = dialog.plan().unwrap();
        assert_eq!(opts.new_branch.as_deref(), Some("fix/empty-config"));
        // An answer nobody asked for is dropped.
        assert!(!dialog.ai_done(Ok(("x".into(), "y".into()))));
        // The AI's names are replaceable by Regenerate (not "typed").
        dialog.activate(WtFocus::Regenerate);
        assert!(is_generated_worktree_name(&dialog.name));
        assert!(is_generated_worktree_name(&dialog.branch));
    }

    #[test]
    fn an_ai_answer_never_silently_checks_out_an_existing_branch() {
        let tmp = tempfile::tempdir().unwrap();
        let mut dialog = on_main(&tmp.path().join("repo"));
        dialog.sync_ai(true, false);
        dialog.ai_text = "x".into();
        dialog.activate(WtFocus::AiToggle);
        dialog.activate(WtFocus::AiRun);
        dialog.ai_done(Ok(("worktree-thing".into(), "other".into())));
        assert_eq!(dialog.branch, "other-2");
        assert_eq!(dialog.name, "worktree-thing");
        assert!(!dialog.checks_out_existing());
    }

    #[test]
    fn an_ai_failure_is_an_inline_error_and_a_cancel_is_silent() {
        let tmp = tempfile::tempdir().unwrap();
        let mut dialog = on_main(&tmp.path().join("repo"));
        dialog.sync_ai(true, false);
        dialog.ai_text = "x".into();
        dialog.activate(WtFocus::AiToggle);
        dialog.activate(WtFocus::AiRun);
        let before = (dialog.name.clone(), dialog.branch.clone());
        assert!(dialog.ai_done(Err("no profile".into())));
        assert_eq!(dialog.ai_error.as_deref(), Some("AI: no profile"));
        assert_eq!((dialog.name.clone(), dialog.branch.clone()), before);
        assert!(dialog.ai_open, "the box stays open to try again");
        let run = drawn(&mut dialog, WtFocus::AiRun);
        assert!(run.width > 0);
        dialog.activate(WtFocus::AiRun);
        assert!(dialog.ai_done(Err("canceled".into())));
        assert_eq!(dialog.ai_error, None);
        // The request ending behind our back (the host says so) frees it.
        dialog.activate(WtFocus::AiRun);
        dialog.sync_ai(true, false);
        assert!(!dialog.ai_running);
    }

    #[test]
    fn the_description_box_draws_and_its_buttons_are_hit() {
        let tmp = tempfile::tempdir().unwrap();
        let mut dialog = on_main(&tmp.path().join("repo"));
        dialog.sync_ai(true, false);
        dialog.activate(WtFocus::AiToggle);
        // The placeholder shows while the box is not the focused field.
        dialog.focus = WtFocus::Name;
        let mut buf = Buffer::empty(Rect::new(0, 0, 100, 40));
        let rect = dialog.rect(buf.area);
        dialog.render(rect, &mut buf);
        let text = dialog_kit::buffer_text(&buf);
        for needle in [
            "Describe the feature, bug or task",
            "e.g. fix the crash",
            "Name it",
            "Hide",
        ] {
            assert!(text.contains(needle), "{needle}\n{text}");
        }
        let hide = drawn(&mut dialog, WtFocus::AiHide);
        dialog.on_mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: hide.x + 1,
            row: hide.y,
            modifiers: KeyModifiers::NONE,
        });
        assert!(!dialog.ai_open);
        assert_eq!(dialog.focus, WtFocus::AiToggle);
    }
}
