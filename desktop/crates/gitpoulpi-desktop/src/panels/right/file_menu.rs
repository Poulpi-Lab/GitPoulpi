//! What the right pane tells the file commands: which file is selected, what its status is, a right
//! click (which selects the row first, like any list) asking the window for the file menu, and the
//! "LFS only" filter of the working-changes lists.

use std::collections::HashSet;

use gitpoulpi_app::palette::vocab::status_pane::StatusSection;
use gitpoulpi_core::domain::{FileStatus, WorkingTree};
use gpui_kit::{Context, Pixels, Point};

use super::status::{section_paths, ListView};
use super::{Mode, RightEvent, RightPanel};
use crate::ref_target::FileSide;

fn section_of(side: FileSide) -> Option<StatusSection> {
    match side {
        FileSide::Unstaged => Some(StatusSection::Unstaged),
        FileSide::Staged => Some(StatusSection::Staged),
        FileSide::Commit => None,
    }
}

impl RightPanel {
    /// The file the cursor is on: in the working changes the selected row of either list, in a commit the
    /// selected changed file.
    pub fn current_file(&self) -> Option<(String, FileSide)> {
        if self.shows_working_changes() {
            let (section, path) = self.selection.cursor()?;
            let side = match section {
                StatusSection::Unstaged => FileSide::Unstaged,
                StatusSection::Staged => FileSide::Staged,
            };
            return Some((path.clone(), side));
        }
        if self.shows_commit() {
            let ix = self.selected?;
            let row = self.rows.iter().find(|r| r.ix == ix && !r.folder)?;
            return Some((row.path.clone(), FileSide::Commit));
        }
        None
    }

    /// The status of a file as the row shows it: the index status in Staged, the working tree's in
    /// Unstaged, the commit's own in a commit.
    pub fn status_of(&self, path: &str, side: FileSide) -> Option<FileStatus> {
        match side {
            FileSide::Unstaged => self
                .status
                .files
                .iter()
                .find(|e| e.path.to_string_lossy() == path)
                .and_then(|e| e.worktree),
            FileSide::Staged => self
                .status
                .files
                .iter()
                .find(|e| e.path.to_string_lossy() == path)
                .and_then(|e| e.index),
            FileSide::Commit => self
                .rows
                .iter()
                .find(|r| r.path == path && !r.folder)
                .map(|r| r.status),
        }
    }

    /// Selects the row of `path` in the list `side` names.
    pub fn select_file(&mut self, path: &str, side: FileSide, cx: &mut Context<Self>) {
        match section_of(side) {
            Some(section) => {
                let (rows, files) = self.section_view(section);
                if let Some(ix) = rows.iter().position(|r| !r.folder && r.path == path) {
                    if !self.selection.contains(section, &rows[ix]) {
                        self.selection.click(
                            &ListView {
                                section,
                                rows: &rows,
                                files: &files,
                            },
                            ix,
                        );
                    }
                }
            }
            None => {
                if let Some(row) = self.rows.iter().find(|r| r.path == path && !r.folder) {
                    self.selected = Some(row.ix);
                }
            }
        }
        cx.notify();
    }

    /// A right click on a file row: the row is selected (a selection it is part of stays) and the window
    /// is asked for the menu.
    pub(super) fn file_row_menu(
        &mut self,
        path: String,
        side: FileSide,
        at: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        self.select_file(&path, side, cx);
        cx.emit(RightEvent::FileMenu { path, side, at });
    }

    /// `Some(files)`: the working-changes lists keep only those files ("LFS only"); `None`: all of them.
    pub fn set_lfs_only(&mut self, only: Option<HashSet<String>>, cx: &mut Context<Self>) {
        if self.lfs_only == only {
            return;
        }
        self.lfs_only = only;
        self.apply_status_filter();
        if matches!(self.mode, Mode::Wip) {
            cx.notify();
        }
    }

    /// Whether "LFS only" is on.
    pub fn lfs_only(&self) -> bool {
        self.lfs_only.is_some()
    }

    /// The paths the working-changes lists show, both lists together.
    pub fn listed_paths(&self) -> Vec<String> {
        let mut out = section_paths(&self.status, StatusSection::Unstaged);
        out.extend(section_paths(&self.status, StatusSection::Staged));
        out
    }

    /// `status` is `raw_status` minus what "LFS only" hides.
    pub(super) fn apply_status_filter(&mut self) {
        self.status = match &self.lfs_only {
            Some(only) => WorkingTree {
                files: self
                    .raw_status
                    .files
                    .iter()
                    .filter(|e| only.contains(&e.path.to_string_lossy().replace('\\', "/")))
                    .cloned()
                    .collect(),
                ..self.raw_status.clone()
            },
            None => self.raw_status.clone(),
        };
    }
}
