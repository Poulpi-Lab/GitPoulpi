//! The working-tree lists as plain data: which rows a section shows and what is selected in it. No
//! window types in here, so the rules are unit-tested.
//!
//! The rules are the shared ones (`gitpoulpi_app::status_rows`: which files a section lists, the status
//! a row shows; `gitpoulpi_app::file_selection`: the multi-select gestures). A file that is staged and
//! edited again is in both sections, an untracked file is in Unstaged, and a conflicted file is pinned
//! to Unstaged (marked `!`) and counted in the title. An unstaged move is one row at the new path; the
//! deleted old file is not listed on its own.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use gitpoulpi_app::file_selection::{FileList, FileSelection, ListRows, SelRow};
use gitpoulpi_app::palette::vocab::status_pane::StatusSection;
use gitpoulpi_app::status_rows::{entry_is_conflicted, entry_status, section_paths_hiding};
use gitpoulpi_core::domain::{StatusEntry, WorkingTree};
use gitpoulpi_link::rename_pairs::RenamePairs;

use super::files::{tree_rows, FileRow};

/// How many files are conflicted.
pub fn conflicts(tree: &WorkingTree) -> usize {
    tree.files.iter().filter(|e| entry_is_conflicted(e)).count()
}

/// The files of `section`, in status order: `(entry index, entry)`.
fn section_entries(
    tree: &WorkingTree,
    section: StatusSection,
) -> impl Iterator<Item = (usize, &StatusEntry)> {
    // The old name of an unstaged move is not a row of its own.
    let pairs = RenamePairs::of(tree);
    let listed: HashSet<PathBuf> =
        section_paths_hiding(tree, section, &pairs.hidden_unstaged_sources())
            .into_iter()
            .collect();
    tree.files
        .iter()
        .enumerate()
        .filter(move |(_, e)| listed.contains(&e.path))
}

/// Every file path of `section` (folders are not files).
pub fn section_paths(tree: &WorkingTree, section: StatusSection) -> Vec<String> {
    let pairs = RenamePairs::of(tree);
    section_paths_hiding(tree, section, &pairs.hidden_unstaged_sources())
        .into_iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect()
}

/// The rows of one section: flat (sorted by path) or in folders, filtered, with collapsed folders'
/// contents left out.
pub fn section_rows(
    tree: &WorkingTree,
    section: StatusSection,
    folders: bool,
    sort_az: bool,
    collapsed: &HashSet<String>,
) -> Vec<FileRow> {
    let mut rows: Vec<FileRow> = section_entries(tree, section)
        .filter_map(|(ix, e)| {
            let mut row = FileRow::file(
                e.path.to_string_lossy().into_owned(),
                entry_status(e, section)?,
                0,
                0,
                ix,
            );
            row.renamed_from = e
                .renamed_from
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned());
            Some(row)
        })
        .collect();
    rows.sort_by(|a, b| a.path.cmp(&b.path));
    if !sort_az {
        rows.reverse();
    }
    if folders {
        rows = tree_rows(rows);
        rows.retain(|r| {
            !collapsed
                .iter()
                .any(|c| r.path.len() > c.len() && r.path.starts_with(&format!("{c}/")))
        });
    }
    rows
}

fn file_list(section: StatusSection) -> FileList {
    match section {
        StatusSection::Unstaged => FileList::Unstaged,
        StatusSection::Staged => FileList::Staged,
    }
}

fn section_of(list: FileList) -> Option<StatusSection> {
    match list {
        FileList::Unstaged => Some(StatusSection::Unstaged),
        FileList::Staged => Some(StatusSection::Staged),
        FileList::Commit | FileList::WipAll => None,
    }
}

/// The selection of one list at a time (the shared `FileSelection`), kept next to the list's own
/// cursor (the row Enter opens).
#[derive(Clone, Debug, Default)]
pub struct Selection {
    files: FileSelection,
    /// The row the keyboard is on.
    cursor: Option<(StatusSection, String)>,
}

/// A list as the selection sees it.
pub struct ListView<'a> {
    pub section: StatusSection,
    /// The rows shown, in order.
    pub rows: &'a [FileRow],
    /// Every file of the section: a folder row selects the ones beneath it, shown or not.
    pub files: &'a [String],
}

impl ListView<'_> {
    fn rows(&self) -> ListRows {
        ListRows {
            rows: self
                .rows
                .iter()
                .map(|r| SelRow {
                    path: PathBuf::from(&r.path),
                    folder: r.folder,
                })
                .collect(),
            files: self.files.iter().map(PathBuf::from).collect(),
        }
    }

    /// The index of the row the cursor is on, when it is in this list.
    fn cursor_ix(&self, cursor: &Option<(StatusSection, String)>) -> Option<usize> {
        match cursor {
            Some((s, p)) if *s == self.section => self.rows.iter().position(|r| &r.path == p),
            _ => None,
        }
    }
}

impl Selection {
    /// Esc: the selection goes, the cursor stays where it is.
    pub fn clear(&mut self) {
        self.files.clear();
    }

    /// The paths an action acts on: the selection when it is in `section` (`files` is everything the
    /// section lists, in the list's order), else the cursor's file.
    pub fn paths(&self, section: StatusSection, files: &[String]) -> Vec<PathBuf> {
        if self.files.list() == Some(file_list(section)) {
            return files
                .iter()
                .filter(|f| {
                    self.files
                        .contains(file_list(section), Path::new(f.as_str()), false)
                })
                .map(PathBuf::from)
                .collect();
        }
        match &self.cursor {
            Some((s, p)) if *s == section => vec![PathBuf::from(p)],
            _ => Vec::new(),
        }
    }

    /// The section the selection or the cursor is in.
    pub fn section(&self) -> Option<StatusSection> {
        self.files
            .list()
            .and_then(section_of)
            .or(self.cursor.as_ref().map(|(s, _)| *s))
    }

    pub fn cursor(&self) -> Option<&(StatusSection, String)> {
        self.cursor.as_ref()
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    pub fn contains(&self, section: StatusSection, row: &FileRow) -> bool {
        self.files
            .contains(file_list(section), Path::new(&row.path), row.folder)
    }

    pub fn is_cursor(&self, section: StatusSection, row: &FileRow) -> bool {
        matches!(&self.cursor, Some((s, p)) if *s == section && *p == row.path)
    }

    fn put_cursor(&mut self, section: StatusSection, row: &FileRow) {
        self.cursor = Some((section, row.path.clone()));
    }

    /// A plain click: this row alone.
    pub fn click(&mut self, list: &ListView, ix: usize) {
        let Some(row) = list.rows.get(ix) else { return };
        let (fl, rows) = (file_list(list.section), list.rows());
        self.files.reset_to(fl, Some(PathBuf::from(&row.path)));
        self.files.toggle(fl, &rows, ix, None);
        self.put_cursor(list.section, row);
    }

    /// Ctrl+click or Space: the row joins the selection or leaves it. The row the cursor was on joins
    /// the moment a second one is added.
    pub fn toggle(&mut self, list: &ListView, ix: usize) {
        let Some(row) = list.rows.get(ix) else { return };
        let cursor = list.cursor_ix(&self.cursor);
        self.files
            .toggle(file_list(list.section), &list.rows(), ix, cursor);
        self.put_cursor(list.section, row);
    }

    /// Shift+click or Shift+arrows: from the anchor (else the cursor) to this row.
    pub fn extend_to(&mut self, list: &ListView, ix: usize) {
        let Some(row) = list.rows.get(ix) else { return };
        let cursor = list.cursor_ix(&self.cursor);
        if self.files.list() != Some(file_list(list.section)) && cursor.is_none() {
            return self.click(list, ix);
        }
        self.files
            .extend_to(file_list(list.section), &list.rows(), ix, cursor);
        self.put_cursor(list.section, row);
    }

    /// Ctrl+A.
    pub fn select_all(&mut self, list: &ListView) {
        self.files.select_all(file_list(list.section), &list.rows());
        if self.cursor.as_ref().is_none_or(|(s, _)| *s != list.section) {
            if let Some(first) = list.rows.first() {
                self.put_cursor(list.section, first);
            }
        }
    }

    /// Up / Down: the cursor moves; with `extend` the selection follows from the anchor.
    pub fn step(&mut self, list: &ListView, delta: isize, extend: bool) {
        if list.rows.is_empty() {
            return;
        }
        let ix = match list.cursor_ix(&self.cursor) {
            Some(at) => at.saturating_add_signed(delta).min(list.rows.len() - 1),
            None if delta < 0 => list.rows.len() - 1,
            None => 0,
        };
        if extend {
            self.extend_to(list, ix);
        } else {
            self.click(list, ix);
        }
    }

    /// Drops what no longer exists after the working tree changed (a staged file leaves Unstaged).
    /// `files_of` lists every file a section holds now.
    pub fn retain(&mut self, files_of: impl Fn(StatusSection) -> Vec<String>) {
        if let Some(list) = self.files.list() {
            if let Some(section) = section_of(list) {
                let now: Vec<PathBuf> = files_of(section).iter().map(PathBuf::from).collect();
                let rows = ListRows {
                    rows: now
                        .iter()
                        .map(|p| SelRow {
                            path: p.clone(),
                            folder: false,
                        })
                        .collect(),
                    files: now.clone(),
                };
                let kept = self.files.files_in_order(&rows);
                if kept.len() != self.files.len() {
                    self.files.reset_to(list, None);
                    for path in &kept {
                        if let Some(ix) = now.iter().position(|p| p == path) {
                            self.files.toggle(list, &rows, ix, None);
                        }
                    }
                }
            }
        }
        if let Some((s, p)) = &self.cursor {
            if !files_of(*s).iter().any(|f| f == p) {
                self.cursor = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::FileStatus;

    fn entry(path: &str, index: Option<FileStatus>, worktree: Option<FileStatus>) -> StatusEntry {
        StatusEntry {
            path: path.into(),
            index,
            worktree,
            renamed_from: None,
        }
    }

    fn view<'a>(section: StatusSection, rows: &'a [FileRow], files: &'a [String]) -> ListView<'a> {
        ListView {
            section,
            rows,
            files,
        }
    }

    fn tree(files: Vec<StatusEntry>) -> WorkingTree {
        WorkingTree {
            files,
            ..WorkingTree::default()
        }
    }

    fn sample() -> WorkingTree {
        use FileStatus::*;
        tree(vec![
            entry("a.txt", None, Some(Modified)),
            entry("b.txt", Some(Modified), Some(Modified)),
            entry("new.txt", None, Some(Untracked)),
            entry("c/d.txt", Some(Added), None),
            entry("c/e.txt", None, Some(Deleted)),
            entry("clash.txt", Some(Conflicted), Some(Conflicted)),
        ])
    }

    #[test]
    fn a_partly_staged_file_is_in_both_sections() {
        let s = sample();
        let un = section_paths(&s, StatusSection::Unstaged);
        let st = section_paths(&s, StatusSection::Staged);
        assert_eq!(
            un,
            vec!["a.txt", "b.txt", "new.txt", "c/e.txt", "clash.txt"],
            "untracked and conflicted files are in Unstaged"
        );
        assert_eq!(st, vec!["b.txt", "c/d.txt"], "a conflict is never Staged");
        assert_eq!(conflicts(&s), 1);
    }

    #[test]
    fn an_unstaged_move_is_one_row_at_the_new_path() {
        let mut s = vec![
            entry("old.txt", None, Some(FileStatus::Deleted)),
            entry("new.txt", None, Some(FileStatus::Untracked)),
        ];
        s[1].renamed_from = Some("old.txt".into());
        let s = tree(s);
        let rows = section_rows(&s, StatusSection::Unstaged, false, true, &HashSet::new());
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].status, FileStatus::Renamed);
        assert_eq!(rows[0].label(), "new.txt <- old.txt");
    }

    #[test]
    fn tree_mode_folds_and_a_collapsed_folder_hides_its_files() {
        let s = sample();
        let none = HashSet::new();
        let rows = section_rows(&s, StatusSection::Unstaged, true, true, &none);
        assert!(rows.iter().any(|r| r.folder && r.path == "c"));
        let folded: HashSet<String> = ["c".to_string()].into();
        let rows = section_rows(&s, StatusSection::Unstaged, true, true, &folded);
        assert!(rows.iter().any(|r| r.path == "c"));
        assert!(!rows.iter().any(|r| r.path == "c/e.txt"));
        let desc = section_rows(&s, StatusSection::Unstaged, false, false, &none);
        assert_eq!(desc[0].path, "new.txt", "Z-A reverses");
    }

    #[test]
    fn click_ctrl_click_and_shift_click_select_like_the_log() {
        let s = sample();
        let rows = section_rows(&s, StatusSection::Unstaged, false, true, &HashSet::new());
        let files = section_paths(&s, StatusSection::Unstaged);
        let list = view(StatusSection::Unstaged, &rows, &files);
        let mut sel = Selection::default();
        sel.click(&list, 0);
        assert_eq!(
            sel.paths(
                StatusSection::Unstaged,
                &section_paths(&s, StatusSection::Unstaged)
            )
            .len(),
            1
        );
        sel.toggle(&list, 2);
        assert_eq!(sel.len(), 2);
        sel.toggle(&list, 2);
        assert_eq!(sel.len(), 1, "ctrl+click again leaves it");
        sel.click(&list, 0);
        sel.extend_to(&list, 3);
        assert_eq!(sel.len(), 4, "shift+click takes the range from the anchor");
        sel.click(&list, 1);
        assert_eq!(sel.len(), 1, "a plain click starts over");
        sel.select_all(&list);
        assert_eq!(sel.len(), 5);
        sel.clear();
        assert!(sel.is_empty());
        assert_eq!(
            sel.paths(
                StatusSection::Unstaged,
                &section_paths(&s, StatusSection::Unstaged)
            )
            .len(),
            1,
            "the cursor row is still what an action acts on"
        );
    }

    #[test]
    fn a_selection_belongs_to_one_section() {
        let s = sample();
        let un_rows = section_rows(&s, StatusSection::Unstaged, false, true, &HashSet::new());
        let un_files = section_paths(&s, StatusSection::Unstaged);
        let st_rows = section_rows(&s, StatusSection::Staged, false, true, &HashSet::new());
        let st_files = section_paths(&s, StatusSection::Staged);
        let mut sel = Selection::default();
        sel.click(&view(StatusSection::Unstaged, &un_rows, &un_files), 0);
        sel.toggle(&view(StatusSection::Staged, &st_rows, &st_files), 0);
        assert_eq!(
            sel.paths(
                StatusSection::Unstaged,
                &section_paths(&s, StatusSection::Unstaged)
            )
            .len(),
            0
        );
        assert_eq!(
            sel.paths(
                StatusSection::Staged,
                &section_paths(&s, StatusSection::Staged)
            ),
            vec![PathBuf::from("b.txt")]
        );
    }

    #[test]
    fn a_folder_row_selects_every_file_beneath_it() {
        let s = sample();
        let folded: HashSet<String> = ["c".to_string()].into();
        let rows = section_rows(&s, StatusSection::Unstaged, true, true, &folded);
        let files = section_paths(&s, StatusSection::Unstaged);
        let list = view(StatusSection::Unstaged, &rows, &files);
        let ix = rows.iter().position(|r| r.folder).unwrap();
        let mut sel = Selection::default();
        sel.click(&list, ix);
        assert_eq!(
            sel.paths(
                StatusSection::Unstaged,
                &section_paths(&s, StatusSection::Unstaged)
            ),
            vec![PathBuf::from("c/e.txt")]
        );
    }

    #[test]
    fn the_cursor_walks_and_extends() {
        let s = sample();
        let rows = section_rows(&s, StatusSection::Unstaged, false, true, &HashSet::new());
        let files = section_paths(&s, StatusSection::Unstaged);
        let list = view(StatusSection::Unstaged, &rows, &files);
        let mut sel = Selection::default();
        sel.step(&list, 1, false);
        assert_eq!(sel.cursor().unwrap().1, rows[0].path);
        sel.step(&list, 1, false);
        sel.step(&list, 1, true);
        assert_eq!(sel.len(), 2);
        sel.step(&list, -10, false);
        assert_eq!(sel.cursor().unwrap().1, rows[0].path);
    }

    #[test]
    fn what_vanished_leaves_the_selection() {
        let s = sample();
        let rows = section_rows(&s, StatusSection::Unstaged, false, true, &HashSet::new());
        let files = section_paths(&s, StatusSection::Unstaged);
        let list = view(StatusSection::Unstaged, &rows, &files);
        let mut sel = Selection::default();
        sel.click(&list, 0);
        sel.toggle(&list, 1);
        let mut now = s.clone();
        now.files
            .retain(|e| e.path != std::path::Path::new("a.txt"));
        sel.retain(|section| section_paths(&now, section));
        assert_eq!(
            sel.paths(
                StatusSection::Unstaged,
                &section_paths(&s, StatusSection::Unstaged)
            ),
            vec![PathBuf::from("b.txt")]
        );
        sel.retain(|_| Vec::new());
        assert!(sel.is_empty() && sel.cursor().is_none());
    }
}
