//! Which paths a working-tree section lists, the status a row shows, and the
//! discard confirmation's prompt: the rules both frontends' Unstaged/Staged
//! lists follow.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use gitpoulpi_core::domain::{FileStatus, StatusEntry, WorkingTree};
use gitpoulpi_i18n::msg::status_pane as msg;

use crate::palette::vocab::status_pane::StatusSection;

/// Whether this row is an unmerged path: a file git holds at stages 1/2/3
/// rather than stage 0.
///
/// Porcelain v2 reports an unmerged path with BOTH columns set (`u` records
/// become `index: Conflicted, worktree: Conflicted`, see the cli backend's
/// `status.rs`), so the naive per-column split below would list the same
/// conflicted file in Unstaged *and* Staged at once.
pub fn entry_is_conflicted(entry: &StatusEntry) -> bool {
    entry.index == Some(FileStatus::Conflicted) || entry.worktree == Some(FileStatus::Conflicted)
}

/// Splits the working tree into the two sections. A file that is
/// both staged and further modified appears in both, which is why this
/// returns paths per section instead of partitioning.
///
/// A *conflicted* file is the exception: it is pinned to the Unstaged half
/// alone. It has work left to do, "unstage this conflict" is not a
/// `GitBackend` operation, and listing it twice would show Take Ours / Take
/// Theirs rows in a section where they read as already-resolved.
///
/// `hidden_unstaged` are the old paths of unstaged moves
/// (`gitpoulpi_link::rename_pairs::RenamePairs::hidden_unstaged_sources`):
/// an unstaged move is one row, at the new path, and the deleted file it came
/// from is not listed on its own.
pub fn section_paths_hiding(
    tree: &WorkingTree,
    section: StatusSection,
    hidden_unstaged: &HashSet<&Path>,
) -> Vec<PathBuf> {
    tree.files
        .iter()
        .filter(|f| match section {
            StatusSection::Staged => f.index.is_some() && !entry_is_conflicted(f),
            StatusSection::Unstaged => {
                f.worktree.is_some() && !hidden_unstaged.contains(f.path.as_path())
            }
        })
        .map(|f| f.path.clone())
        .collect()
}

/// The status a row should show: the index status in the staged section,
/// the worktree status in the unstaged one.
pub fn entry_status(entry: &StatusEntry, section: StatusSection) -> Option<FileStatus> {
    match section {
        StatusSection::Staged => entry.index,
        // The new side of an unstaged move reads as a rename, not as a new file.
        StatusSection::Unstaged if entry.renamed_from.is_some() && entry.index.is_none() => {
            Some(FileStatus::Renamed)
        }
        StatusSection::Unstaged => entry.worktree,
    }
}

/// The confirm modal's prompt for discarding `paths`: the total count and,
/// when any are untracked, how many ("Discard 12 files (3 untracked)?").
/// Untracked is read from each path's Unstaged status in `tree`, not
/// assumed, since a caller could be asked to discard a path this working
/// tree no longer has an entry for (a race with another client).
pub fn discard_confirmation_prompt(tree: &WorkingTree, paths: &[PathBuf]) -> String {
    let untracked = paths
        .iter()
        .filter(|p| {
            tree.files
                .iter()
                .any(|f| &f.path == *p && matches!(f.worktree, Some(FileStatus::Untracked)))
        })
        .count();
    let n = paths.len() as u64;
    if untracked == 0 {
        msg::discard_prompt(n)
    } else {
        msg::discard_prompt_untracked(n, untracked)
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
            renamed_from: None
        }
    }

    fn tree(files: Vec<StatusEntry>) -> WorkingTree {
        WorkingTree {
            files,
            ..WorkingTree::default()
        }
    }

    #[test]
    fn a_conflicted_file_is_listed_in_unstaged_only() {
        let t = tree(vec![entry(
            "c.rs",
            Some(FileStatus::Conflicted),
            Some(FileStatus::Conflicted),
        )]);
        let none = HashSet::new();
        assert_eq!(
            section_paths_hiding(&t, StatusSection::Unstaged, &none),
            vec![PathBuf::from("c.rs")]
        );
        assert!(section_paths_hiding(&t, StatusSection::Staged, &none).is_empty());
    }

    #[test]
    fn a_hidden_move_source_is_not_listed() {
        let t = tree(vec![
            entry("new.rs", None, Some(FileStatus::Untracked)),
            entry("old.rs", None, Some(FileStatus::Deleted)),
        ]);
        let hidden: HashSet<&Path> = [Path::new("old.rs")].into();
        assert_eq!(
            section_paths_hiding(&t, StatusSection::Unstaged, &hidden),
            vec![PathBuf::from("new.rs")]
        );
    }

    #[test]
    fn the_status_follows_the_section() {
        let e = entry(
            "a.rs",
            Some(FileStatus::Added),
            Some(FileStatus::Modified),
        );
        assert_eq!(
            entry_status(&e, StatusSection::Staged),
            Some(FileStatus::Added)
        );
        assert_eq!(
            entry_status(&e, StatusSection::Unstaged),
            Some(FileStatus::Modified)
        );
        let mut moved = entry("b.rs", None, Some(FileStatus::Untracked));
        moved.renamed_from = Some("a.rs".into());
        assert_eq!(
            entry_status(&moved, StatusSection::Unstaged),
            Some(FileStatus::Renamed)
        );
    }

    #[test]
    fn the_discard_prompt_counts_untracked_files() {
        let t = tree(vec![
            entry("u.rs", None, Some(FileStatus::Untracked)),
            entry("m.rs", None, Some(FileStatus::Modified)),
        ]);
        let both = vec![PathBuf::from("u.rs"), PathBuf::from("m.rs")];
        assert_eq!(
            discard_confirmation_prompt(&t, &both),
            msg::discard_prompt_untracked(2, 1)
        );
        assert_eq!(
            discard_confirmation_prompt(&t, &both[1..]),
            msg::discard_prompt(1)
        );
    }
}
