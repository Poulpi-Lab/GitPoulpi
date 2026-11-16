//! A renamed or moved file is one row in the file lists but two paths for
//! git: the deleted old file and the new one. The row acts on both.
//!
//! The backends say which paths pair up (`StatusEntry::renamed_from`); this
//! keeps the pairs of the latest status and expands the staging and discard
//! actions the window submits, in one place (`DaemonLink::submit_action`),
//! so the mouse, the keys, the menu and the palette all behave alike.
//!
//! - Stage (unstaged pair): stage the new path and the old one (git adds the
//!   deletion), two journal entries.
//! - Unstage (a staged rename): unstage both paths.
//! - Discard: ONE `DiscardPaths [new, old]`: the old file is restored, the
//!   new one removed, one confirmation and one undo entry.
//! - Stage/Unstage Folder: a pair split across the folder's edge brings the
//!   path outside the folder along.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use gitpoulpi_core::domain::{FileStatus, StatusEntry, WorkingTree};

use crate::daemon_link::Action;

/// `new path -> old path`, for each half of the file lists.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct RenamePairs {
    /// An unstaged move: the new file is untracked (or intent-to-add), the
    /// old one deleted in the working tree.
    pub(crate) unstaged: HashMap<PathBuf, PathBuf>,
    /// A rename in the index.
    pub(crate) staged: HashMap<PathBuf, PathBuf>,
}

fn is_new_side(e: &StatusEntry) -> bool {
    e.index.is_none()
        && matches!(
            e.worktree,
            Some(FileStatus::Untracked) | Some(FileStatus::Added)
        )
}

impl RenamePairs {
    pub(crate) fn of(tree: &WorkingTree) -> Self {
        let mut pairs = RenamePairs::default();
        for e in &tree.files {
            let Some(from) = &e.renamed_from else {
                continue;
            };
            if is_new_side(e) {
                pairs.unstaged.insert(e.path.clone(), from.clone());
            } else if e.index == Some(FileStatus::Renamed) {
                pairs.staged.insert(e.path.clone(), from.clone());
            }
        }
        pairs
    }

    /// The old paths of the unstaged pairs: their own rows are not listed
    /// (the new path's row stands for both).
    pub(crate) fn hidden_unstaged_sources(&self) -> HashSet<&Path> {
        self.unstaged.values().map(PathBuf::as_path).collect()
    }

    /// The actions to send for `action`, in order: the action itself plus
    /// what its rename partners need.
    pub(crate) fn expand(&self, action: Action) -> Vec<Action> {
        if self.unstaged.is_empty() && self.staged.is_empty() {
            return vec![action];
        }
        match action {
            Action::StageFile(p) => match self.unstaged.get(&p) {
                Some(old) => vec![Action::StageFile(p), Action::StageFile(old.clone())],
                None => vec![Action::StageFile(p)],
            },
            Action::UnstageFile(p) => match self.staged.get(&p) {
                Some(old) => vec![Action::UnstageFile(p), Action::UnstageFile(old.clone())],
                None => vec![Action::UnstageFile(p)],
            },
            Action::DiscardFile { path, source } => match self.unstaged.get(&path) {
                Some(old) => vec![Action::DiscardPaths(vec![path, old.clone()])],
                None => vec![Action::DiscardFile { path, source }],
            },
            Action::DiscardPaths(mut paths) => {
                let extra: Vec<PathBuf> = paths
                    .iter()
                    .filter_map(|p| self.unstaged.get(p))
                    .filter(|old| !paths.contains(old))
                    .cloned()
                    .collect();
                paths.extend(extra);
                vec![Action::DiscardPaths(paths)]
            }
            Action::StageFolder(f) => {
                let mut out = vec![Action::StageFolder(f.clone())];
                out.extend(
                    outside_partners(&self.unstaged, &f)
                        .into_iter()
                        .map(Action::StageFile),
                );
                out
            }
            Action::UnstageFolder(f) => {
                let mut out = vec![Action::UnstageFolder(f.clone())];
                out.extend(
                    outside_partners(&self.staged, &f)
                        .into_iter()
                        .map(Action::UnstageFile),
                );
                out
            }
            other => vec![other],
        }
    }
}

/// The old paths of pairs whose new path is inside `folder` and the old one
/// outside it, in path order.
fn outside_partners(pairs: &HashMap<PathBuf, PathBuf>, folder: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = pairs
        .iter()
        .filter(|(new, old)| new.starts_with(folder) && !old.starts_with(folder))
        .map(|(_, old)| old.clone())
        .collect();
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(
        path: &str,
        index: Option<FileStatus>,
        worktree: Option<FileStatus>,
        from: Option<&str>,
    ) -> StatusEntry {
        StatusEntry {
            path: path.into(),
            index,
            worktree,
            renamed_from: from.map(PathBuf::from),
        }
    }

    fn pairs() -> RenamePairs {
        RenamePairs::of(&WorkingTree {
            files: vec![
                entry("old.rs", None, Some(FileStatus::Deleted), None),
                entry(
                    "sub/new.rs",
                    None,
                    Some(FileStatus::Untracked),
                    Some("old.rs"),
                ),
                entry("kept.rs", Some(FileStatus::Renamed), None, Some("was.rs")),
            ],
            ..WorkingTree::default()
        })
    }

    #[test]
    fn staging_an_unstaged_move_stages_both_paths() {
        let out = pairs().expand(Action::StageFile("sub/new.rs".into()));
        assert_eq!(
            out,
            vec![
                Action::StageFile("sub/new.rs".into()),
                Action::StageFile("old.rs".into())
            ]
        );
        // The deleted file's own row is not listed.
        assert!(pairs()
            .hidden_unstaged_sources()
            .contains(Path::new("old.rs")));
    }

    #[test]
    fn unstaging_a_staged_rename_unstages_both_paths() {
        let out = pairs().expand(Action::UnstageFile("kept.rs".into()));
        assert_eq!(
            out,
            vec![
                Action::UnstageFile("kept.rs".into()),
                Action::UnstageFile("was.rs".into())
            ]
        );
    }

    #[test]
    fn discarding_a_move_is_one_request_over_both_paths() {
        let out = pairs().expand(Action::DiscardFile {
            path: "sub/new.rs".into(),
            source: gitpoulpi_core::domain::Oid([0; 20]),
        });
        assert_eq!(
            out,
            vec![Action::DiscardPaths(vec![
                "sub/new.rs".into(),
                "old.rs".into()
            ])]
        );
        let out = pairs().expand(Action::DiscardPaths(vec!["sub/new.rs".into()]));
        assert_eq!(
            out,
            vec![Action::DiscardPaths(vec![
                "sub/new.rs".into(),
                "old.rs".into()
            ])]
        );
    }

    #[test]
    fn a_folder_brings_the_partner_outside_it() {
        let out = pairs().expand(Action::StageFolder("sub".into()));
        assert_eq!(
            out,
            vec![
                Action::StageFolder("sub".into()),
                Action::StageFile("old.rs".into())
            ]
        );
    }

    #[test]
    fn other_files_are_untouched() {
        let out = pairs().expand(Action::StageFile("other.rs".into()));
        assert_eq!(out, vec![Action::StageFile("other.rs".into())]);
    }
}
