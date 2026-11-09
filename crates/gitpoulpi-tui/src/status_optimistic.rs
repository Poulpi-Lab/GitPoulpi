//! The optimistic half of staging: move rows between Unstaged and Staged
//! in the render loop's own copy of the status the moment the user clicks,
//! then let the scoped refresh that follows the daemon's answer replace it
//! (or, on an error, put the previous copy back). Display only: the daemon
//! stays the owner of the real state.

use std::path::Path;

use gitpoulpi_core::domain::{FileStatus, StatusEntry, WorkingTree};

use crate::daemon_link::Action;

/// Whether `action` is one the overlay knows how to apply.
pub(crate) fn is_optimistic(action: &Action) -> bool {
    matches!(
        action,
        Action::StageFile(_)
            | Action::UnstageFile(_)
            | Action::StageAll
            | Action::UnstageAll
            | Action::StageFolder(_)
            | Action::UnstageFolder(_)
    )
}

enum Scope<'a> {
    Path(&'a Path),
    Folder(&'a Path),
    All,
}

impl Scope<'_> {
    fn contains(&self, path: &Path) -> bool {
        match self {
            Scope::Path(p) => path == *p,
            Scope::Folder(dir) => path.starts_with(dir),
            Scope::All => true,
        }
    }
}

/// Applies `action` to `tree`. Returns whether anything moved. Conflicted
/// entries are never touched (staging them is a resolution, not a move).
pub(crate) fn apply(tree: &mut WorkingTree, action: &Action) -> bool {
    let (stage, scope) = match action {
        Action::StageFile(p) => (true, Scope::Path(p)),
        Action::UnstageFile(p) => (false, Scope::Path(p)),
        Action::StageFolder(p) => (true, Scope::Folder(p)),
        Action::UnstageFolder(p) => (false, Scope::Folder(p)),
        Action::StageAll => (true, Scope::All),
        Action::UnstageAll => (false, Scope::All),
        _ => return false,
    };
    let mut moved = false;
    for entry in &mut tree.files {
        if !scope.contains(&entry.path) {
            continue;
        }
        moved |= if stage {
            stage_entry(entry)
        } else {
            unstage_entry(entry)
        };
    }
    moved
}

fn is_conflicted(entry: &StatusEntry) -> bool {
    entry.index == Some(FileStatus::Conflicted) || entry.worktree == Some(FileStatus::Conflicted)
}

fn stage_entry(entry: &mut StatusEntry) -> bool {
    if is_conflicted(entry) {
        return false;
    }
    let Some(worktree) = entry.worktree.take() else {
        return false;
    };
    if entry.index.is_none() {
        entry.index = Some(match worktree {
            FileStatus::Untracked => FileStatus::Added,
            other => other,
        });
    }
    true
}

fn unstage_entry(entry: &mut StatusEntry) -> bool {
    if is_conflicted(entry) {
        return false;
    }
    let Some(index) = entry.index.take() else {
        return false;
    };
    if entry.worktree.is_none() {
        entry.worktree = Some(match index {
            FileStatus::Added => FileStatus::Untracked,
            FileStatus::Renamed => FileStatus::Modified,
            other => other,
        });
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(path: &str, index: Option<FileStatus>, worktree: Option<FileStatus>) -> StatusEntry {
        StatusEntry {
            path: path.into(),
            index,
            worktree,
        }
    }

    fn tree() -> WorkingTree {
        WorkingTree {
            files: vec![
                entry("a.txt", None, Some(FileStatus::Modified)),
                entry("src/b.txt", None, Some(FileStatus::Untracked)),
                entry("src/c.txt", Some(FileStatus::Modified), None),
                entry(
                    "d.txt",
                    Some(FileStatus::Conflicted),
                    Some(FileStatus::Conflicted),
                ),
            ],
            ..WorkingTree::default()
        }
    }

    #[test]
    fn stage_all_moves_every_unstaged_row_and_skips_conflicts() {
        let mut t = tree();
        assert!(apply(&mut t, &Action::StageAll));
        assert_eq!(t.files[0], entry("a.txt", Some(FileStatus::Modified), None));
        assert_eq!(
            t.files[1],
            entry("src/b.txt", Some(FileStatus::Added), None)
        );
        assert_eq!(t.files[3].index, Some(FileStatus::Conflicted));
        assert!(t.files[3].worktree.is_some());
    }

    #[test]
    fn unstage_folder_only_touches_that_folder() {
        let mut t = tree();
        assert!(apply(&mut t, &Action::UnstageFolder("src".into())));
        assert_eq!(
            t.files[2],
            entry("src/c.txt", None, Some(FileStatus::Modified))
        );
        assert_eq!(t.files[0].index, None);
    }

    #[test]
    fn a_staged_add_unstages_to_untracked_and_back() {
        let mut t = tree();
        apply(&mut t, &Action::StageFile("src/b.txt".into()));
        apply(&mut t, &Action::UnstageFile("src/b.txt".into()));
        assert_eq!(
            t.files[1],
            entry("src/b.txt", None, Some(FileStatus::Untracked))
        );
    }

    #[test]
    fn an_unrelated_action_changes_nothing() {
        let mut t = tree();
        assert!(!apply(&mut t, &Action::StashPush));
        assert_eq!(t.files, tree().files);
    }
}
