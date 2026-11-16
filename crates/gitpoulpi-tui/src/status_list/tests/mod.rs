use super::*;

mod actions;
mod header;
mod i18n;
mod rows;
mod selection;

fn tree(entries: &[(&str, bool, bool)]) -> WorkingTree {
    use gitpoulpi_core::domain::{FileStatus, StatusEntry};
    WorkingTree {
        files: entries
            .iter()
            .map(|(p, staged, unstaged)| StatusEntry {
                renamed_from: None,
                path: PathBuf::from(p),
                index: staged.then_some(FileStatus::Staged),
                worktree: unstaged.then_some(FileStatus::Modified),
            })
            .collect(),
        ..WorkingTree::default()
    }
}

fn entry(
    path: &str,
    worktree: Option<gitpoulpi_core::domain::FileStatus>,
) -> gitpoulpi_core::domain::StatusEntry {
    gitpoulpi_core::domain::StatusEntry {
        renamed_from: None,
        path: PathBuf::from(path),
        index: None,
        worktree,
    }
}
