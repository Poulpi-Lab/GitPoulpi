//! Staging from the working-changes lists: stage, unstage, discard, the selection gestures and the
//! keyboard. Every method here is what a command, a button, a key or a drop ends in, so they all behave
//! alike; each one submits at most one action, which is one entry of the undo journal.
//!
//! Stage and unstage never ask. Discard always asks first (the terminal UI's rule: it cannot be taken
//! back by anything but the journal's Undo of the *next* action, and untracked files are gone for good),
//! saying how many files and how many are untracked.

use std::path::PathBuf;

use gitpoulpi_app::palette::vocab::status_pane::StatusSection;
use gitpoulpi_app::palette::vocab::tree_fold::TreeFold;
use gitpoulpi_i18n::msg;
use gitpoulpi_link::Action;
use gpui_kit::{Context, Focusable as _, KeyDownEvent, Modifiers, Window};

use super::status::{conflicts, section_paths, section_rows, ListView};
use super::{Mode, RightPanel};
use crate::daemon::staging;
use crate::dialogs::{self, Choice, Spec};

/// What a drag from one list carries to the other.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DraggedFiles {
    pub from: StatusSection,
    pub paths: Vec<String>,
}

impl RightPanel {
    /// The rows shown for `section` and every file it holds.
    pub(super) fn section_view(
        &self,
        section: StatusSection,
    ) -> (Vec<super::FileRow>, Vec<String>) {
        (
            section_rows(
                &self.status,
                section,
                self.tree,
                self.sort_az,
                &self.collapsed,
            ),
            section_paths(&self.status, section),
        )
    }

    /// The files `section` shows as rows (folders left out), in display order.
    pub fn row_paths(&self, section: StatusSection) -> Vec<String> {
        self.section_view(section)
            .0
            .into_iter()
            .filter(|r| !r.folder)
            .map(|r| r.path)
            .collect()
    }

    /// How many files `section` lists.
    pub fn section_len(&self, section: StatusSection) -> usize {
        section_paths(&self.status, section).len()
    }

    /// How many files are conflicted.
    pub fn conflict_count(&self) -> usize {
        conflicts(&self.status)
    }

    /// The paths the selection (or the cursor) holds in `section`.
    pub fn selected_paths(&self, section: StatusSection) -> Vec<PathBuf> {
        if !self.shows_working_changes() {
            return Vec::new();
        }
        self.selection
            .paths(section, &section_paths(&self.status, section))
    }

    /// What the palette needs to know: is a file of each section selected.
    pub fn selected_in(&self, section: StatusSection) -> bool {
        !self.selected_paths(section).is_empty()
    }

    /// The rows of the selection's section are walked by the keyboard.
    pub fn selection_len(&self) -> usize {
        self.selection.len()
    }

    // ---- selection ---------------------------------------------------------------------------------

    /// A click on row `ix` of `section`: Ctrl toggles, Shift takes the range, a plain click starts over.
    /// A folder row also folds or unfolds on a plain click.
    pub fn click_row(
        &mut self,
        section: StatusSection,
        ix: usize,
        mods: Modifiers,
        cx: &mut Context<Self>,
    ) {
        let (rows, files) = self.section_view(section);
        let list = ListView {
            section,
            rows: &rows,
            files: &files,
        };
        if mods.shift {
            self.selection.extend_to(&list, ix);
        } else if mods.control || mods.platform {
            self.selection.toggle(&list, ix);
        } else {
            self.selection.click(&list, ix);
            match rows.get(ix) {
                Some(row) if row.folder => {
                    if !self.collapsed.remove(&row.path) {
                        self.collapsed.insert(row.path.clone());
                    }
                }
                // A plain click on a file selects it and opens its diff, like the terminal UI.
                Some(_) => self.open_selected_diff(cx),
                None => {}
            }
        }
        cx.notify();
    }

    /// Up / Down (Shift extends) in the list the cursor is in (Unstaged when there is none).
    pub fn move_cursor(&mut self, delta: isize, extend: bool, cx: &mut Context<Self>) {
        let section = self.selection.section().unwrap_or(StatusSection::Unstaged);
        let (rows, files) = self.section_view(section);
        let list = ListView {
            section,
            rows: &rows,
            files: &files,
        };
        self.selection.step(&list, delta, extend);
        cx.notify();
    }

    /// Ctrl+A: every file of the list the cursor is in.
    pub fn select_all(&mut self, cx: &mut Context<Self>) {
        let section = self.selection.section().unwrap_or(StatusSection::Unstaged);
        let (rows, files) = self.section_view(section);
        self.selection.select_all(&ListView {
            section,
            rows: &rows,
            files: &files,
        });
        cx.notify();
    }

    /// Esc.
    pub fn clear_selection(&mut self, cx: &mut Context<Self>) {
        self.selection.clear();
        cx.notify();
    }

    /// Space: the cursor row joins the selection or leaves it.
    pub fn toggle_selection(&mut self, cx: &mut Context<Self>) {
        let Some((section, path)) = self.selection.cursor().cloned() else {
            return;
        };
        let (rows, files) = self.section_view(section);
        if let Some(ix) = rows.iter().position(|r| r.path == path) {
            let list = ListView {
                section,
                rows: &rows,
                files: &files,
            };
            self.selection.toggle(&list, ix);
        }
        cx.notify();
    }

    /// The palette's "Focus Status > Unstaged / Staged": the pane takes the keyboard, the cursor goes to
    /// the first file of that list.
    pub fn focus_section(
        &mut self,
        section: StatusSection,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus.focus(window, cx);
        let (rows, files) = self.section_view(section);
        let list = ListView {
            section,
            rows: &rows,
            files: &files,
        };
        if let Some(ix) = rows.iter().position(|r| !r.folder) {
            self.selection.click(&list, ix);
        }
        cx.notify();
    }

    // ---- folders -----------------------------------------------------------------------------------

    /// Every folder of both lists in Tree mode, whatever is folded.
    fn all_folders(&self) -> Vec<String> {
        let none = std::collections::HashSet::new();
        [StatusSection::Unstaged, StatusSection::Staged]
            .into_iter()
            .flat_map(|section| {
                section_rows(&self.status, section, true, true, &none)
                    .into_iter()
                    .filter(|r| r.folder)
                    .map(|r| r.path)
            })
            .collect()
    }

    /// Folds or unfolds folders (`=`, `-`, `*`, `_`): all of them, or the cursor's folder and the ones
    /// below it. Only Tree mode has folders.
    pub fn fold(&mut self, fold: TreeFold, cx: &mut Context<Self>) {
        if !self.tree {
            return;
        }
        let below = self
            .selection
            .cursor()
            .map(|(_, p)| p.clone())
            .filter(|p| self.all_folders().contains(p));
        let under = |p: &str, root: &str| p == root || p.starts_with(&format!("{root}/"));
        match fold {
            TreeFold::ExpandAll => self.collapsed.clear(),
            TreeFold::CollapseAll => self.collapsed = self.all_folders().into_iter().collect(),
            TreeFold::ExpandBelow => {
                if let Some(root) = below {
                    self.collapsed.retain(|p| !under(p, &root));
                }
            }
            TreeFold::CollapseBelow => {
                if let Some(root) = below {
                    let folders = self.all_folders();
                    self.collapsed
                        .extend(folders.into_iter().filter(|p| under(p, &root)));
                }
            }
        }
        cx.notify();
    }

    /// `z`: fold every folder, or unfold them all when some are folded.
    pub fn toggle_fold_all(&mut self, cx: &mut Context<Self>) {
        let fold = if self.collapsed.is_empty() {
            TreeFold::CollapseAll
        } else {
            TreeFold::ExpandAll
        };
        self.fold(fold, cx);
    }

    /// Whether any folder is folded (for tests and the "Collapse all" control).
    pub fn folded_count(&self) -> usize {
        self.collapsed.len()
    }

    // ---- stage / unstage ---------------------------------------------------------------------------

    fn need_working(&mut self, cx: &mut Context<Self>) -> bool {
        if matches!(self.mode, Mode::Wip) {
            return true;
        }
        self.say(msg::input::SELECT_FILE_STATUS_OR_COMMIT.get(), cx);
        false
    }

    /// Stages (`stage`) or unstages `paths` as one action.
    pub fn stage_paths(&mut self, paths: Vec<PathBuf>, stage: bool, cx: &mut Context<Self>) {
        if let Some(action) = staging::stage_action(&paths, stage) {
            self.backend.submit(action);
            cx.notify();
        }
    }

    /// "Stage Selected File" (the selection, or the cursor row).
    pub fn stage_selected(&mut self, cx: &mut Context<Self>) {
        if !self.need_working(cx) {
            return;
        }
        let paths = self.selected_paths(StatusSection::Unstaged);
        if paths.is_empty() {
            return self.say(msg::input::SELECT_UNSTAGED_FILE.get(), cx);
        }
        self.stage_paths(paths, true, cx);
    }

    /// "Unstage Selected File".
    pub fn unstage_selected(&mut self, cx: &mut Context<Self>) {
        if !self.need_working(cx) {
            return;
        }
        let paths = self.selected_paths(StatusSection::Staged);
        if paths.is_empty() {
            return self.say(msg::input::SELECT_STAGED_FILE.get(), cx);
        }
        self.stage_paths(paths, false, cx);
    }

    /// Ctrl+Space: stage or unstage what is selected, by the list it is in.
    pub fn toggle_stage_row(&mut self, cx: &mut Context<Self>) {
        match self.selection.section() {
            Some(StatusSection::Unstaged) => self.stage_selected(cx),
            Some(StatusSection::Staged) => self.unstage_selected(cx),
            None => {
                if self.need_working(cx) {
                    self.say(msg::input::SELECT_FILE_STATUS_OR_COMMIT.get(), cx);
                }
            }
        }
    }

    /// "Stage all changes": one action.
    pub fn stage_all(&mut self, cx: &mut Context<Self>) {
        if self.need_working(cx) && self.section_len(StatusSection::Unstaged) > 0 {
            self.backend.submit(Action::StageAll);
            self.say(msg::input::STAGING_ALL.get(), cx);
        }
    }

    /// "Unstage all changes": one action.
    pub fn unstage_all(&mut self, cx: &mut Context<Self>) {
        if self.need_working(cx) && self.section_len(StatusSection::Staged) > 0 {
            self.backend.submit(Action::UnstageAll);
        }
    }

    /// Drop of `paths` dragged from `from` onto the other list: they cross over.
    pub fn drop_files(
        &mut self,
        onto: StatusSection,
        dragged: &DraggedFiles,
        cx: &mut Context<Self>,
    ) {
        if dragged.from == onto {
            return;
        }
        let paths: Vec<PathBuf> = dragged.paths.iter().map(PathBuf::from).collect();
        self.stage_paths(paths, onto == StatusSection::Staged, cx);
    }

    // ---- discard -----------------------------------------------------------------------------------

    /// "Discard Selected File...": asks, then throws the selected files' changes away.
    pub fn discard_selected(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.need_working(cx) {
            return;
        }
        let paths = self.selected_paths(StatusSection::Unstaged);
        if paths.is_empty() {
            return self.say(msg::input::SELECT_UNSTAGED_FILE.get(), cx);
        }
        self.ask_discard(paths, window, cx);
    }

    /// "Discard all changes": every Unstaged file, always asked.
    pub fn discard_all(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.need_working(cx) {
            return;
        }
        let paths: Vec<PathBuf> = section_paths(&self.status, StatusSection::Unstaged)
            .into_iter()
            .map(PathBuf::from)
            .collect();
        if paths.is_empty() {
            return self.say(msg::status_list::NOTHING_TO_DISCARD.get(), cx);
        }
        self.ask_discard(paths, window, cx);
    }

    /// The confirmation, then the one `DiscardPaths` action.
    pub fn ask_discard(
        &mut self,
        paths: Vec<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let prompt = gitpoulpi_app::status_rows::discard_confirmation_prompt(&self.status, &paths);
        let me = cx.weak_entity();
        let spec = Spec {
            title: prompt,
            lines: paths
                .iter()
                .take(12)
                .map(|p| p.to_string_lossy().into_owned())
                .chain((paths.len() > 12).then(|| format!("... {}", paths.len() - 12)))
                .collect(),
            choices: vec![
                Choice::destructive(msg::discard_changes::DISCARD.get()),
                Choice::new(msg::dialog::CANCEL.get()),
            ],
            hint: msg::diff_actions::CONFIRM_HINT.get().to_string(),
            highlight_first: true,
            yes_no_keys: true,
        };
        dialogs::open(
            window,
            cx,
            spec,
            move |ix, cx| {
                if ix == 0 {
                    let _ = me.update(cx, |this, cx| this.submit_discard(paths, cx));
                }
            },
            || {},
        );
    }

    fn submit_discard(&mut self, paths: Vec<PathBuf>, cx: &mut Context<Self>) {
        if let Some(action) = staging::discard_action(&paths) {
            self.backend.submit(action);
            self.selection.clear();
            cx.notify();
        }
    }

    // ---- opening a diff ----------------------------------------------------------------------------

    /// Enter, `d` and a double click: the cursor row's diff in the centre.
    pub fn open_selected_diff(&mut self, cx: &mut Context<Self>) {
        let Some((section, path)) = self.selection.cursor().cloned() else {
            return self.say(msg::input::SELECT_FILE_STATUS_OR_COMMIT.get(), cx);
        };
        self.open_working(path, section == StatusSection::Staged, cx);
    }

    // ---- keyboard ----------------------------------------------------------------------------------

    /// The list keys of the pane, when no text field has the keyboard: Up / Down (Shift extends), Space
    /// toggles, Enter opens the diff, Ctrl+A selects all, Escape clears.
    pub(super) fn key_down(
        &mut self,
        ev: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if matches!(self.mode, Mode::Commit(..) | Mode::Compare(..)) {
            if !self.filter.read(cx).focus_handle(cx).is_focused(window) {
                self.commit_key(ev, cx);
            }
            return;
        }
        if !matches!(self.mode, Mode::Wip) || self.field_focused(window, cx) {
            return;
        }
        let ks = &ev.keystroke;
        let m = &ks.modifiers;
        let handled = match (ks.key.as_str(), m.control || m.platform, m.alt) {
            ("up", false, false) => {
                self.move_cursor(-1, m.shift, cx);
                true
            }
            ("down", false, false) => {
                self.move_cursor(1, m.shift, cx);
                true
            }
            ("space", false, false) => {
                self.toggle_selection(cx);
                true
            }
            ("enter", false, false) => {
                self.open_selected_diff(cx);
                true
            }
            ("a", true, false) => {
                self.select_all(cx);
                true
            }
            ("escape", false, false) if !self.selection.is_empty() => {
                self.clear_selection(cx);
                true
            }
            _ => false,
        };
        if handled {
            cx.stop_propagation();
        }
    }
}
