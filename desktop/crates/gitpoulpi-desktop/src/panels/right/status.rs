//! The working-tree lists as plain data: which rows a section shows and what is selected in it. No
//! window types in here, so the rules are unit-tested.
//!
//! The sections are the terminal UI's two halves (`StatusSection`): a file that is staged and edited
//! again is in both, an untracked file is in Unstaged, and a conflicted file is pinned to Unstaged
//! (marked `!`) and counted in the title. An unstaged move is one row at the new path; the deleted old
//! file is not listed on its own.

use std::collections::{BTreeSet, HashSet};
use std::path::PathBuf;

use gitpoulpi_app::palette::vocab::status_pane::StatusSection;
use gitpoulpi_core::domain::{FileStatus, StatusEntry};

use super::files::{tree_rows, FileRow};

/// Whether git stopped on this file (a merge, rebase or cherry-pick left it unmerged).
pub fn is_conflicted(e: &StatusEntry) -> bool {
    e.index == Some(FileStatus::Conflicted) || e.worktree == Some(FileStatus::Conflicted)
}

/// How many files are conflicted.
pub fn conflicts(status: &[StatusEntry]) -> usize {
    status.iter().filter(|e| is_conflicted(e)).count()
}

/// The files of `section`, in status order: `(entry index, path)`.
fn section_entries(
    status: &[StatusEntry],
    section: StatusSection,
) -> impl Iterator<Item = (usize, &StatusEntry)> {
    // The old name of an unstaged move is not a row of its own.
    let hidden: HashSet<&std::path::Path> = status
        .iter()
        .filter(|e| e.index.is_none() && e.renamed_from.is_some())
        .filter_map(|e| e.renamed_from.as_deref())
        .collect();
    status
        .iter()
        .enumerate()
        .filter(move |(_, e)| match section {
            StatusSection::Staged => e.index.is_some() && !is_conflicted(e),
            StatusSection::Unstaged => e.worktree.is_some() && !hidden.contains(e.path.as_path()),
        })
}

/// The status a row shows: the index side in Staged, the working-tree side in Unstaged (a move not yet
/// staged reads as a rename).
fn row_status(e: &StatusEntry, section: StatusSection) -> Option<FileStatus> {
    match section {
        StatusSection::Staged => e.index,
        StatusSection::Unstaged if e.renamed_from.is_some() && e.index.is_none() => {
            Some(FileStatus::Renamed)
        }
        StatusSection::Unstaged => e.worktree,
    }
}

/// Every file path of `section` (folders are not files).
pub fn section_paths(status: &[StatusEntry], section: StatusSection) -> Vec<String> {
    section_entries(status, section)
        .map(|(_, e)| e.path.to_string_lossy().into_owned())
        .collect()
}

/// The rows of one section: flat (sorted by path) or in folders, filtered, with collapsed folders'
/// contents left out.
pub fn section_rows(
    status: &[StatusEntry],
    section: StatusSection,
    tree: bool,
    sort_az: bool,
    collapsed: &HashSet<String>,
) -> Vec<FileRow> {
    let mut rows: Vec<FileRow> = section_entries(status, section)
        .filter_map(|(ix, e)| {
            let mut row = FileRow::file(
                e.path.to_string_lossy().into_owned(),
                row_status(e, section)?,
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
    if tree {
        rows = tree_rows(rows);
        rows.retain(|r| {
            !collapsed
                .iter()
                .any(|c| r.path.len() > c.len() && r.path.starts_with(&format!("{c}/")))
        });
    }
    rows
}

/// The selection of one list at a time, kept next to the list's own cursor (the row Enter opens).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Selection {
    section: Option<StatusSection>,
    files: BTreeSet<String>,
    anchor: Option<String>,
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
    fn files_of(&self, row: &FileRow) -> Vec<String> {
        if row.folder {
            let prefix = format!("{}/", row.path);
            self.files
                .iter()
                .filter(|f| f.starts_with(&prefix))
                .cloned()
                .collect()
        } else {
            vec![row.path.clone()]
        }
    }
}

impl Selection {
    /// Esc: the selection goes, the cursor stays where it is.
    pub fn clear(&mut self) {
        self.section = None;
        self.files.clear();
        self.anchor = None;
    }

    /// The paths an action acts on: the selection when it is in `section`, else the cursor's file.
    pub fn paths(&self, section: StatusSection) -> Vec<PathBuf> {
        if self.section == Some(section) && !self.files.is_empty() {
            return self.files.iter().map(PathBuf::from).collect();
        }
        match &self.cursor {
            Some((s, p)) if *s == section => vec![PathBuf::from(p)],
            _ => Vec::new(),
        }
    }

    /// The section the selection or the cursor is in.
    pub fn section(&self) -> Option<StatusSection> {
        self.section.or(self.cursor.as_ref().map(|(s, _)| *s))
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
        self.section == Some(section)
            && if row.folder {
                let prefix = format!("{}/", row.path);
                self.files.iter().any(|f| f.starts_with(&prefix))
            } else {
                self.files.contains(&row.path)
            }
    }

    pub fn is_cursor(&self, section: StatusSection, row: &FileRow) -> bool {
        matches!(&self.cursor, Some((s, p)) if *s == section && *p == row.path)
    }

    fn scope(&mut self, section: StatusSection) {
        if self.section != Some(section) {
            self.files.clear();
            self.anchor = None;
            self.section = Some(section);
        }
    }

    fn put_cursor(&mut self, section: StatusSection, row: &FileRow) {
        self.cursor = Some((section, row.path.clone()));
    }

    /// A plain click: this row alone.
    pub fn click(&mut self, list: &ListView, ix: usize) {
        let Some(row) = list.rows.get(ix) else { return };
        self.files.clear();
        self.section = Some(list.section);
        self.files.extend(list.files_of(row));
        self.anchor = Some(row.path.clone());
        self.put_cursor(list.section, row);
    }

    /// Ctrl+click or Space: the row joins the selection or leaves it.
    pub fn toggle(&mut self, list: &ListView, ix: usize) {
        let Some(row) = list.rows.get(ix) else { return };
        self.scope(list.section);
        let files = list.files_of(row);
        if files.iter().all(|f| self.files.contains(f)) {
            for f in &files {
                self.files.remove(f);
            }
        } else {
            self.files.extend(files);
        }
        self.anchor = Some(row.path.clone());
        self.put_cursor(list.section, row);
    }

    /// Shift+click or Shift+arrows: from the anchor to this row.
    pub fn extend_to(&mut self, list: &ListView, ix: usize) {
        let Some(row) = list.rows.get(ix) else { return };
        if self.section != Some(list.section) || self.anchor.is_none() {
            return self.click(list, ix);
        }
        let from = self
            .anchor
            .as_ref()
            .and_then(|a| list.rows.iter().position(|r| &r.path == a))
            .unwrap_or(ix);
        let (lo, hi) = (from.min(ix), from.max(ix));
        self.files.clear();
        for r in &list.rows[lo..=hi] {
            self.files.extend(list.files_of(r));
        }
        self.put_cursor(list.section, row);
    }

    /// Ctrl+A.
    pub fn select_all(&mut self, list: &ListView) {
        self.scope(list.section);
        self.files = list.files.iter().cloned().collect();
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
        let at = match &self.cursor {
            Some((s, p)) if *s == list.section => list.rows.iter().position(|r| &r.path == p),
            _ => None,
        };
        let ix = match at {
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
    pub fn retain(&mut self, still_there: impl Fn(StatusSection, &str) -> bool) {
        if let Some(section) = self.section {
            self.files.retain(|f| still_there(section, f));
            if self.files.is_empty() {
                self.section = None;
                self.anchor = None;
            }
        }
        if let Some((s, p)) = &self.cursor {
            if !still_there(*s, p) {
                self.cursor = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn sample() -> Vec<StatusEntry> {
        use FileStatus::*;
        vec![
            entry("a.txt", None, Some(Modified)),
            entry("b.txt", Some(Modified), Some(Modified)),
            entry("new.txt", None, Some(Untracked)),
            entry("c/d.txt", Some(Added), None),
            entry("c/e.txt", None, Some(Deleted)),
            entry("clash.txt", Some(Conflicted), Some(Conflicted)),
        ]
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
        assert_eq!(sel.paths(StatusSection::Unstaged).len(), 1);
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
            sel.paths(StatusSection::Unstaged).len(),
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
        assert_eq!(sel.paths(StatusSection::Unstaged).len(), 0);
        assert_eq!(
            sel.paths(StatusSection::Staged),
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
            sel.paths(StatusSection::Unstaged),
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
        sel.retain(|_, p| p != "a.txt");
        assert_eq!(
            sel.paths(StatusSection::Unstaged),
            vec![PathBuf::from("b.txt")]
        );
        sel.retain(|_, _| false);
        assert!(sel.is_empty() && sel.cursor().is_none());
    }
}
