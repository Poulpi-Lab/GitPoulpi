//! The Create worktree dialog: a linked worktree
//! (`git worktree add`) from a branch or a commit, optionally on a new
//! branch, optionally opened as a tab.
//!
//! ```text
//! ╭ Create worktree from feature ────────────────────────────────╮
//! │  Name          feature█                                      │
//! │  Folder        C:\Projets\GitPoulpi-feature     [ Browse… ]  │
//! │  Ref           feature                                  ▾    │
//! │  New branch    (empty = check feature out there)             │
//! │                [x] Open the new worktree as a tab            │
//! │                                                              │
//! │  Will create C:\Projets\GitPoulpi-feature on feature         │
//! │  Tab next · Enter create               [ Cancel ] [ Create ] │
//! ╰──────────────────────────────────────────────────────────────╯
//! ```
//!
//! The Name proposes the Folder (a sibling of the repository named
//! `<repo>-<name>`) until the Folder is edited by hand; the Ref is a
//! combobox of the local branches plus the commit the dialog was opened
//! on, and changing it re-proposes the Name (until that was typed over
//! too).
//!
//! Pure, like `init_dialog`: validation, keys, clicks and drawing. The
//! owner (`repo_dialogs`) opens the shared folder picker for Browse and
//! turns [`WorktreeOutcome::Submit`] into a daemon action.

use crate::th;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use gitpoulpi_core::domain::{short, WorktreeAddOptions, WorktreeStart};
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
    /// The New branch field's first value (`""` = no new branch).
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

/// The next dialog opens with the Ref combobox open ("Choose a branch...").
static PICK_REF: AtomicBool = AtomicBool::new(false);

/// "Choose a branch...": the dialog the next `worktree_seed` opens starts
/// on its Ref combobox.
pub(crate) fn ask_to_pick_ref() {
    PICK_REF.store(true, Ordering::Relaxed);
}

/// The dialog's focusable parts, in `Tab` order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WtFocus {
    Name,
    Path,
    Browse,
    Ref,
    NewBranch,
    OpenTab,
    Create,
    Cancel,
}

const ORDER: [WtFocus; 8] = [
    WtFocus::Name,
    WtFocus::Path,
    WtFocus::Browse,
    WtFocus::Ref,
    WtFocus::NewBranch,
    WtFocus::OpenTab,
    WtFocus::Create,
    WtFocus::Cancel,
];

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
}

#[derive(Debug, Clone)]
pub(crate) struct WorktreeDialog {
    pub(crate) seed: WorktreeSeed,
    /// The worktree's name: what the default folder is built from.
    pub(crate) name: String,
    /// The new worktree's folder.
    pub(crate) path: String,
    pub(crate) new_branch: String,
    pub(crate) open_tab: bool,
    pub(crate) focus: WtFocus,
    pub(crate) message: Option<String>,
    pub(crate) hover: Option<WtFocus>,
    /// Name still follows the Ref, Folder still follows the Name.
    name_auto: bool,
    path_auto: bool,
    repo_root: PathBuf,
    /// Where the worktree starts now (the Ref combobox's choice).
    start: WorktreeStart,
    /// Whether the start branch is checked out elsewhere already.
    branch_busy: bool,
    branches: Vec<BranchChoice>,
    combo: Combo,
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

/// The seed of "From the current branch": the checked-out branch cannot be
/// checked out twice, so the dialog proposes a new branch on it (`<b>-wt`,
/// numbered when taken); a detached HEAD starts a detached worktree there.
pub(crate) fn seed_from_head(
    refs: &[gitpoulpi_core::domain::Ref],
    worktrees: &[gitpoulpi_core::domain::Worktree],
) -> Option<WorktreeSeed> {
    use gitpoulpi_core::domain::RefKind;
    let taken = |name: &str| {
        refs.iter()
            .any(|r| matches!(&r.kind, RefKind::LocalBranch { name: n } if n == name))
    };
    if let Some(head) = refs.iter().find(|r| r.is_head) {
        if let RefKind::LocalBranch { name } = &head.kind {
            let mut proposal = format!("{name}-wt");
            let mut n = 2;
            while taken(&proposal) {
                proposal = format!("{name}-wt{n}");
                n += 1;
            }
            return Some(WorktreeSeed {
                start: WorktreeStart::Branch(name.clone()),
                label: name.clone(),
                new_branch: proposal,
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

/// The seed of "Choose a branch...": the dialog opens on its Ref list.
pub(crate) fn seed_to_choose(
    refs: &[gitpoulpi_core::domain::Ref],
    worktrees: &[gitpoulpi_core::domain::Worktree],
) -> Option<WorktreeSeed> {
    let mut seed = seed_from_head(refs, worktrees)?;
    seed.new_branch.clear();
    Some(seed)
}

/// The Ref combobox's value for a start.
fn choice_value(start: &WorktreeStart) -> String {
    match start {
        WorktreeStart::Branch(b) => format!("b:{b}"),
        WorktreeStart::Commit(oid) => format!("c:{}", oid.to_hex()),
    }
}

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
    /// the seed starts on a commit, that commit. `pick_ref` opens it
    /// focused on the Ref, its list open.
    pub(crate) fn with_choices(
        seed: WorktreeSeed,
        repo_root: &Path,
        branches: Vec<BranchChoice>,
        pick_ref: bool,
    ) -> Self {
        let mut dialog = Self {
            name: String::new(),
            path: String::new(),
            new_branch: seed.new_branch.clone(),
            focus: if pick_ref {
                WtFocus::Ref
            } else if seed.branch_busy {
                WtFocus::NewBranch
            } else {
                WtFocus::Path
            },
            start: seed.start.clone(),
            branch_busy: seed.branch_busy,
            seed,
            open_tab: true,
            message: None,
            hover: None,
            name_auto: true,
            path_auto: true,
            repo_root: repo_root.to_path_buf(),
            branches,
            combo: Combo::new(msg::worktree::REF_HINT.get()),
            hits: Vec::new(),
            close: Default::default(),
            pristine: String::new(),
            edit: Default::default(),
        };
        dialog.build_combo();
        dialog.derive_name();
        dialog.pristine = dialog.signature();
        if pick_ref {
            dialog.combo.open();
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

    /// The proposed name for the current start (and new branch).
    fn proposed_name(&self) -> String {
        if !self.new_branch.trim().is_empty() {
            return folder_safe(self.new_branch.trim());
        }
        match &self.start {
            WorktreeStart::Branch(b) => folder_safe(b),
            WorktreeStart::Commit(oid) => {
                if matches!(&self.seed.start, WorktreeStart::Commit(s) if s == oid)
                    && !self.seed.label.is_empty()
                {
                    folder_safe(&self.seed.label)
                } else {
                    short(*oid)
                }
            }
        }
    }

    /// Re-proposes the Name (while it was not typed) and the Folder (while
    /// it was not typed).
    fn derive_name(&mut self) {
        if self.name_auto {
            self.name = self.proposed_name();
        }
        self.derive_path();
    }

    fn derive_path(&mut self) {
        if self.path_auto {
            self.path = default_worktree_path(&self.repo_root, self.name.trim())
                .display()
                .to_string();
        }
    }

    /// The Ref combobox picked `value` (`b:<branch>` or `c:<oid>`).
    fn pick_ref(&mut self, value: &str) {
        if let Some(branch) = value.strip_prefix("b:") {
            // The seed's own proposed branch name belongs to the seed's start:
            // another branch is checked out as it is.
            if self.seed.start != WorktreeStart::Branch(branch.to_string())
                && !self.seed.new_branch.is_empty()
                && self.new_branch == self.seed.new_branch
            {
                self.new_branch.clear();
            } else if self.seed.start == WorktreeStart::Branch(branch.to_string())
                && self.new_branch.is_empty()
            {
                self.new_branch = self.seed.new_branch.clone();
            }
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
            // A remote branch seeds its own new branch: back to it.
            if self.new_branch.trim().is_empty() {
                self.new_branch = self.seed.new_branch.clone();
            }
        }
        self.combo.select(Some(choice_value(&self.start)));
        self.message = None;
        self.derive_name();
    }

    fn signature(&self) -> String {
        format!(
            "{}\n{}\n{}\n{}\n{}",
            self.name,
            self.path,
            self.new_branch,
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
        let branch = self.new_branch.trim();
        if branch.is_empty() {
            if self.branch_busy {
                let label = match &self.start {
                    WorktreeStart::Branch(b) => b.clone(),
                    WorktreeStart::Commit(_) => self.seed.label.clone(),
                };
                return Err(msg::worktree::already_checked_out(label));
            }
        } else if let Some(reason) = branch_name_error(branch) {
            return Err(reason);
        } else if self.branches.iter().any(|b| b.name == branch) {
            return Err(msg::worktree::branch_exists(branch));
        }
        Ok((
            path,
            WorktreeAddOptions {
                start: self.start.clone(),
                new_branch: (!branch.is_empty()).then(|| branch.to_string()),
            },
        ))
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
            WtFocus::NewBranch => Some(&mut self.new_branch),
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
                self.name_auto = false;
                self.derive_path();
            }
            WtFocus::Path => self.path_auto = false,
            WtFocus::NewBranch => self.derive_name(),
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
        self.focus = WtFocus::NewBranch;
        self.message = None;
    }

    pub(crate) fn paste(&mut self, text: &str) {
        if self.combo.is_open() {
            self.combo.paste(text);
            return;
        }
        let clean: String = text.chars().filter(|c| !c.is_control()).collect();
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
                self.combo.open();
                WorktreeOutcome::Stay
            }
            _ => self.submit(),
        }
    }

    fn combo_outcome(&mut self, outcome: ComboOutcome) {
        if let ComboOutcome::Picked(value) = outcome {
            self.pick_ref(&value);
        }
    }

    pub(crate) fn on_key(&mut self, key: KeyEvent) -> WorktreeOutcome {
        if self.combo.is_open() {
            let outcome = self.combo.on_key(key);
            self.combo_outcome(outcome);
            return WorktreeOutcome::Stay;
        }
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        if self.focus == WtFocus::Ref
            && matches!(
                key.code,
                KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Down
            )
        {
            self.combo.open();
            return WorktreeOutcome::Stay;
        }
        let focus = self.focus;
        if let (edit, Some((id, field))) = self.field_parts() {
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
            KeyCode::Esc => return WorktreeOutcome::Cancel,
            KeyCode::Tab | KeyCode::BackTab | KeyCode::Down | KeyCode::Up => {
                let forward =
                    matches!(key.code, KeyCode::Down) || (key.code == KeyCode::Tab && !shift);
                let at = ORDER.iter().position(|f| *f == self.focus).unwrap_or(0);
                let next = if forward {
                    (at + 1) % ORDER.len()
                } else {
                    (at + ORDER.len() - 1) % ORDER.len()
                };
                self.focus = ORDER[next];
            }
            _ if crate::keymap::current().is(crate::keymap::Action::DialogBrowse, &key) => {
                return self.browse()
            }
            // Enter on the checkbox toggles it like Space; anywhere else
            // it is the dialog's default button (`activate`).
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
            self.combo_outcome(outcome);
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
                    if matches!(
                        part,
                        WtFocus::Browse
                            | WtFocus::Create
                            | WtFocus::Cancel
                            | WtFocus::OpenTab
                            | WtFocus::Ref
                    ) {
                        return self.activate(part);
                    }
                    self.field_mouse(mouse);
                }
            }
            _ => {}
        }
        WorktreeOutcome::Stay
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
            self.hits = hits;
            return;
        }
        // The widest label plus a gap (14 at least), whatever the language.
        let label_w: u16 = [
            msg::worktree::NAME,
            msg::worktree::FOLDER,
            msg::worktree::REF_FIELD,
            msg::worktree::NEW_BRANCH,
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

        let mut y = inner.y;
        label(buf, y, msg::worktree::NAME.get());
        let name = Rect::new(field_x, y, field_w, 1);
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
            ButtonState::Normal {
                hovered: self.hover == Some(WtFocus::Browse),
                focused: self.focus == WtFocus::Browse,
            },
        );
        hits.push((browse, WtFocus::Browse));
        hits.push((path, WtFocus::Path));

        y += 2;
        label(buf, y, msg::worktree::REF_FIELD.get());
        let ref_rect = Rect::new(field_x, y, field_w, 1);
        self.combo.render_field(
            buf,
            ref_rect,
            self.focus == WtFocus::Ref,
            self.hover == Some(WtFocus::Ref),
        );
        hits.push((ref_rect, WtFocus::Ref));

        y += 2;
        label(buf, y, msg::worktree::NEW_BRANCH.get());
        let branch = Rect::new(field_x, y, field_w, 1);
        let placeholder = match &self.start {
            WorktreeStart::Branch(b) if !self.branch_busy => msg::worktree::branch_hint_checkout(b),
            WorktreeStart::Branch(_) => msg::worktree::BRANCH_HINT_REQUIRED.get().into(),
            WorktreeStart::Commit(_) => msg::worktree::BRANCH_HINT_DETACHED.get().into(),
        };
        dialog_kit::text_input(
            buf,
            branch,
            &self.new_branch,
            &placeholder,
            self.focus == WtFocus::NewBranch,
            self.hover == Some(WtFocus::NewBranch),
            (&self.edit, WtFocus::NewBranch),
        );
        hits.push((branch, WtFocus::NewBranch));

        y += 2;
        let open_tab = dialog_kit::checkbox(
            buf,
            field_x,
            y,
            field_w,
            msg::worktree::OPEN_TAB.get(),
            self.open_tab,
            CheckState::Enabled {
                hovered: self.hover == Some(WtFocus::OpenTab),
                focused: self.focus == WtFocus::OpenTab,
            },
        );
        hits.push((open_tab, WtFocus::OpenTab));

        let bottom = inner.y + inner.height;
        let plan_y = bottom.saturating_sub(3);
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
        let create_w = dialog_kit::button_width(msg::dialog::CREATE.get());
        let cancel_w = dialog_kit::button_width(msg::dialog::CANCEL.get());
        let create_x = inner.right().saturating_sub(create_w);
        let cancel_x = create_x.saturating_sub(cancel_w + 1);
        let cancel = dialog_kit::button(
            buf,
            cancel_x,
            footer,
            msg::dialog::CANCEL.get(),
            ButtonState::Normal {
                hovered: self.hover == Some(WtFocus::Cancel),
                focused: self.focus == WtFocus::Cancel,
            },
        );
        let create = dialog_kit::button(
            buf,
            create_x,
            footer,
            msg::dialog::CREATE.get(),
            if plan.is_err() {
                ButtonState::Disabled
            } else {
                ButtonState::Primary {
                    hovered: self.hover == Some(WtFocus::Create),
                    focused: self.focus == WtFocus::Create,
                }
            },
        );
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
        // The Ref list hangs over everything else.
        self.combo.render_popup(buf, buf.area);
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

pub(crate) fn worktree_rect(frame: Rect) -> Rect {
    WIN.place(
        frame,
        dialog_kit::centered(frame, frame.width.saturating_sub(4).min(90), 16),
    )
}

const WIN: dialog_kit::Win = dialog_kit::Win::movable("worktree");

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::Oid;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn seed(start: WorktreeStart, label: &str, busy: bool) -> WorktreeSeed {
        WorktreeSeed {
            start,
            label: label.into(),
            new_branch: String::new(),
            branch_busy: busy,
        }
    }

    #[test]
    fn it_proposes_a_sibling_folder_named_after_the_start() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("GitPoulpi");
        let dialog = WorktreeDialog::new(
            seed(
                WorktreeStart::Branch("feature/login".into()),
                "feature/login",
                false,
            ),
            &root,
        );
        assert_eq!(
            PathBuf::from(&dialog.path),
            tmp.path().join("GitPoulpi-feature-login")
        );
        let (path, opts) = dialog.plan().unwrap();
        assert_eq!(path, tmp.path().join("GitPoulpi-feature-login"));
        assert_eq!(opts.start, WorktreeStart::Branch("feature/login".into()));
        assert_eq!(opts.new_branch, None);
    }

    #[test]
    fn enter_submits_with_the_new_branch_and_the_tab_choice() {
        let tmp = tempfile::tempdir().unwrap();
        let oid = Oid::from_hex(&"a".repeat(40)).unwrap();
        let mut dialog = WorktreeDialog::new(
            seed(WorktreeStart::Commit(oid), "aaaaaaa", false),
            tmp.path(),
        );
        dialog.focus = WtFocus::NewBranch;
        for c in "topic".chars() {
            dialog.on_key(key(KeyCode::Char(c)));
        }
        // Tab to the checkbox, Space unticks it.
        dialog.on_key(key(KeyCode::Tab));
        assert_eq!(dialog.focus, WtFocus::OpenTab);
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

        // The checked-out branch needs a new branch name.
        let mut busy = WorktreeDialog::new(
            seed(WorktreeStart::Branch("main".into()), "main", true),
            tmp.path(),
        );
        assert_eq!(busy.focus, WtFocus::NewBranch);
        assert!(busy.plan().unwrap_err().contains("name a new branch"));
        busy.focus = WtFocus::Create;
        assert_eq!(busy.on_key(key(KeyCode::Enter)), WorktreeOutcome::Stay);
        assert!(busy.message.as_deref().unwrap().contains("new branch"));

        let mut dialog = WorktreeDialog::new(
            seed(WorktreeStart::Branch("feature".into()), "feature", false),
            tmp.path(),
        );
        dialog.path = tmp.path().join("full").display().to_string();
        assert!(dialog.plan().unwrap_err().contains("not empty"));
        dialog.path = "relative".into();
        assert!(dialog.plan().unwrap_err().contains("full path"));
        dialog.path = tmp.path().join("ok").display().to_string();
        dialog.new_branch = "bad name".into();
        assert!(dialog
            .plan()
            .unwrap_err()
            .contains("not a valid branch name"));
    }

    #[test]
    fn browse_starts_at_the_folders_parent_and_keeps_the_name() {
        let tmp = tempfile::tempdir().unwrap();
        let mut dialog = WorktreeDialog::new(
            seed(WorktreeStart::Branch("feature".into()), "feature", false),
            &tmp.path().join("repo"),
        );
        assert_eq!(
            dialog.on_key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL)),
            WorktreeOutcome::Browse(tmp.path().to_path_buf())
        );
        dialog.set_parent(&tmp.path().join("elsewhere"));
        assert_eq!(
            PathBuf::from(&dialog.path),
            tmp.path().join("elsewhere").join("repo-feature")
        );
    }

    #[test]
    fn it_renders_and_every_button_is_where_it_is_hit() {
        let tmp = tempfile::tempdir().unwrap();
        for width in [72u16, 140] {
            let mut dialog = WorktreeDialog::new(
                seed(WorktreeStart::Branch("feature".into()), "feature", false),
                &tmp.path().join("repo"),
            );
            let mut buf = Buffer::empty(Rect::new(0, 0, width, 30));
            let rect = worktree_rect(buf.area);
            dialog.render(rect, &mut buf);
            let text = dialog_kit::buffer_text(&buf);
            for needle in [
                "Create worktree from feature",
                "Folder",
                "New branch",
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

    fn type_into(dialog: &mut WorktreeDialog, focus: WtFocus, text: &str) {
        dialog.focus = focus;
        for c in text.chars() {
            dialog.on_key(key(KeyCode::Char(c)));
        }
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
        ]
    }

    #[test]
    fn the_folder_follows_the_name_until_the_folder_is_edited() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("repo");
        let mut dialog = WorktreeDialog::with_choices(
            seed(WorktreeStart::Branch("other".into()), "other", false),
            &root,
            two_branches(),
            false,
        );
        assert_eq!(dialog.name, "other");
        assert_eq!(PathBuf::from(&dialog.path), tmp.path().join("repo-other"));
        // Typing in Name re-proposes the Folder.
        type_into(&mut dialog, WtFocus::Name, "-2");
        assert_eq!(dialog.name, "other-2");
        assert_eq!(PathBuf::from(&dialog.path), tmp.path().join("repo-other-2"));
        // Editing the Folder by hand stops the sync for good.
        type_into(&mut dialog, WtFocus::Path, "x");
        let edited = dialog.path.clone();
        assert!(edited.ends_with("repo-other-2x"), "{edited}");
        type_into(&mut dialog, WtFocus::Name, "y");
        assert_eq!(
            dialog.path, edited,
            "the Folder was edited: Name no longer moves it"
        );
    }

    #[test]
    fn changing_the_ref_re_proposes_the_name_and_checks_the_busy_branch() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("repo");
        let mut dialog = WorktreeDialog::with_choices(
            seed(WorktreeStart::Branch("main".into()), "main", true),
            &root,
            two_branches(),
            false,
        );
        assert!(dialog.plan().unwrap_err().contains("name a new branch"));
        dialog.pick_ref("b:other");
        assert_eq!(dialog.start, WorktreeStart::Branch("other".into()));
        assert_eq!(dialog.name, "other");
        assert_eq!(PathBuf::from(&dialog.path), tmp.path().join("repo-other"));
        let (path, opts) = dialog.plan().unwrap();
        assert_eq!(path, tmp.path().join("repo-other"));
        assert_eq!(opts.start, WorktreeStart::Branch("other".into()));
        assert_eq!(opts.new_branch, None);
        // A hand-typed Name is not replaced by the next ref.
        type_into(&mut dialog, WtFocus::Name, "!");
        dialog.pick_ref("b:main");
        assert_eq!(dialog.name, "other!");
    }

    #[test]
    fn the_commit_the_dialog_was_opened_on_is_a_choice_and_gets_a_short_name() {
        let tmp = tempfile::tempdir().unwrap();
        let oid = Oid::from_hex(&"c".repeat(40)).unwrap();
        let dialog = WorktreeDialog::with_choices(
            seed(WorktreeStart::Commit(oid), "ccccccc", false),
            &tmp.path().join("repo"),
            two_branches(),
            false,
        );
        assert_eq!(dialog.name, "ccccccc");
        assert!(dialog
            .combo
            .options
            .iter()
            .any(|o| o.value == format!("c:{}", oid.to_hex())));
        let (_, opts) = dialog.plan().unwrap();
        assert_eq!(opts.start, WorktreeStart::Commit(oid));
        assert_eq!(opts.new_branch, None, "detached unless a branch is named");
    }

    #[test]
    fn the_seed_for_the_current_branch_proposes_a_free_new_branch() {
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
        assert_eq!(seed.new_branch, "main-wt2", "the first proposal was taken");
        assert!(seed.branch_busy);
        assert_eq!(seed_to_choose(&refs, &[]).unwrap().new_branch, "");
    }

    #[test]
    fn validation_says_why_inline() {
        let tmp = tempfile::tempdir().unwrap();
        let mut dialog = WorktreeDialog::with_choices(
            seed(WorktreeStart::Branch("other".into()), "other", false),
            &tmp.path().join("repo"),
            two_branches(),
            false,
        );
        dialog.name = "a/b".into();
        assert!(dialog.plan().unwrap_err().contains("slashes"));
        dialog.name = String::new();
        assert!(dialog.plan().unwrap_err().contains("name"));
        dialog.name = "ok".into();
        dialog.new_branch = "main".into();
        assert!(dialog.plan().unwrap_err().contains("already exists"));
        let file = tmp.path().join("file");
        std::fs::write(&file, "x").unwrap();
        dialog.new_branch.clear();
        dialog.path = file.display().to_string();
        assert!(dialog.plan().unwrap_err().contains("is a file"));
    }
}
