//! Which foldable panes are collapsed, remembered per repository.
//!
//! A tab switch rebuilds the window and a restart rebuilds everything, so
//! without this every section opens expanded again. The state lives in
//! `RepoState::folds` (`<repo>/.gitpoulpi/repo-state.toml`), next to the
//! collapsed ref folders (`refs_tree`). It is written by change
//! detection from the window tick -- one comparison per frame, a disk
//! write only when the user actually folded something -- so no toggle
//! site has to remember to save.

use crate::refs_sidebar::SidebarSection;
#[allow(unused_imports)]
use crate::refs_sidebar::SidebarSectionExt;
use crate::status_pane::StatusSection;
use gitpoulpi_session::{FoldState, RepoState};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Most collapsed tree-folder paths kept per tree; beyond it the
/// longest-collapsed ones are forgotten, so the file cannot grow forever.
pub(crate) const MAX_TREE_FOLDS: usize = 500;

pub(crate) fn state_path(repo_root: &Path) -> PathBuf {
    let state_dir = gitpoulpi_config::Settings::resolve_ambient()
        .map(|s| s.config().session.per_repo_state_dir.clone())
        .unwrap_or_else(|_| gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION.to_string());
    RepoState::path_for(repo_root, &state_dir)
}

pub(crate) fn status_key(section: StatusSection) -> &'static str {
    match section {
        StatusSection::Unstaged => "unstaged",
        StatusSection::Staged => "staged",
    }
}

/// Everything that is folded in one window, in a comparable, sorted form.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct Folds {
    pub(crate) sidebar: Vec<String>,
    pub(crate) status: Vec<String>,
    pub(crate) pr_groups: Vec<String>,
    /// Collapsed folders of the Unstaged/Staged trees, oldest first.
    pub(crate) status_folders: Vec<String>,
    /// Collapsed folders of the commit-files tree, oldest first.
    pub(crate) commit_folders: Vec<String>,
}

fn path_key(p: &Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

/// The stored list for a live set: paths still collapsed keep their
/// place in `prev`, newly collapsed ones go last (sorted, for a stable
/// result), and the oldest beyond [`MAX_TREE_FOLDS`] are dropped.
fn merge_lru(prev: &[String], live: &HashSet<PathBuf>) -> Vec<String> {
    let live: HashSet<String> = live.iter().map(|p| path_key(p)).collect();
    let mut out: Vec<String> = prev.iter().filter(|p| live.contains(*p)).cloned().collect();
    let have: HashSet<String> = out.iter().cloned().collect();
    let mut added: Vec<String> = live.into_iter().filter(|p| !have.contains(p)).collect();
    added.sort();
    out.extend(added);
    if out.len() > MAX_TREE_FOLDS {
        out.drain(..out.len() - MAX_TREE_FOLDS);
    }
    out
}

impl Folds {
    /// Adds the two file trees' collapsed folders, ordered against
    /// what was last saved (`prev`) so the oldest can be evicted.
    /// The live sets are trimmed to what is kept, so an evicted folder
    /// is expanded again rather than re-added (and re-saved) every frame.
    pub(crate) fn with_trees(
        mut self,
        status: &mut HashSet<PathBuf>,
        commit: &mut HashSet<PathBuf>,
        prev: &Folds,
    ) -> Self {
        self.status_folders = merge_lru(&prev.status_folders, status);
        self.commit_folders = merge_lru(&prev.commit_folders, commit);
        status.retain(|p| self.status_folders.contains(&path_key(p)));
        commit.retain(|p| self.commit_folders.contains(&path_key(p)));
        self
    }

    pub(crate) fn capture(
        sidebar: &HashSet<String>,
        status: &HashSet<StatusSection>,
        pr_groups: Vec<String>,
    ) -> Self {
        let mut sidebar: Vec<String> = sidebar.iter().cloned().collect();
        let mut status: Vec<String> = status.iter().map(|s| status_key(*s).into()).collect();
        let mut pr_groups = pr_groups;
        sidebar.sort();
        status.sort();
        pr_groups.sort();
        Self {
            sidebar,
            status,
            pr_groups,
            status_folders: Vec::new(),
            commit_folders: Vec::new(),
        }
    }
}

/// What was saved for this repository; `None` for the PR groups when the
/// user never chose, so the built-in default applies.
pub(crate) struct Restored {
    pub(crate) sidebar: HashSet<String>,
    pub(crate) status: HashSet<StatusSection>,
    pub(crate) pr_groups: Option<Vec<String>>,
    pub(crate) status_folders: Vec<String>,
    pub(crate) commit_folders: Vec<String>,
}

impl Restored {
    pub(crate) fn status_set(&self) -> HashSet<PathBuf> {
        self.status_folders.iter().map(PathBuf::from).collect()
    }
    pub(crate) fn commit_set(&self) -> HashSet<PathBuf> {
        self.commit_folders.iter().map(PathBuf::from).collect()
    }
}

pub(crate) fn load(repo_root: &Path) -> Restored {
    let folds = RepoState::load(&state_path(repo_root))
        .unwrap_or_default()
        .folds;
    let known: HashSet<&str> = SidebarSection::ORDER.iter().map(|s| s.title()).collect();
    Restored {
        sidebar: folds
            .sidebar_sections
            .into_iter()
            .filter(|t| known.contains(t.as_str()))
            .collect(),
        status: StatusSection::ORDER
            .into_iter()
            .filter(|s| folds.status_sections.iter().any(|k| k == status_key(*s)))
            .collect(),
        pr_groups: folds.pull_request_groups,
        status_folders: folds.status_folders,
        commit_folders: folds.commit_folders,
    }
}

/// Load-modify-save: the file also holds preferences and ref filters.
pub(crate) fn save(repo_root: &Path, folds: &Folds) {
    let path = state_path(repo_root);
    let mut state = RepoState::load(&path).unwrap_or_default();
    state.folds = FoldState {
        sidebar_sections: folds.sidebar.clone(),
        status_sections: folds.status.clone(),
        pull_request_groups: Some(folds.pr_groups.clone()),
        status_folders: folds.status_folders.clone(),
        commit_folders: folds.commit_folders.clone(),
    };
    let _ = state.save(&path);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folds_round_trip_and_unknown_sections_are_dropped() {
        let dir = tempfile::tempdir().unwrap();
        let sidebar: HashSet<String> = ["TAGS", "ISSUES"].map(String::from).into();
        let status: HashSet<StatusSection> = [StatusSection::Staged].into();
        let folds = Folds::capture(&sidebar, &status, vec!["mine".into()]);
        save(dir.path(), &folds);
        let back = load(dir.path());
        assert_eq!(back.sidebar, sidebar);
        assert_eq!(back.status, status);
        assert_eq!(back.pr_groups, Some(vec!["mine".to_string()]));
    }

    #[test]
    fn tree_folds_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let mut status: HashSet<PathBuf> = [PathBuf::from("src/ui")].into();
        let mut commit: HashSet<PathBuf> = [PathBuf::from("docs"), PathBuf::from("a/b")].into();
        let folds = Folds::default().with_trees(&mut status, &mut commit, &Folds::default());
        save(dir.path(), &folds);
        let back = load(dir.path());
        assert_eq!(back.status_set(), status);
        assert_eq!(back.commit_set(), commit);
    }

    #[test]
    fn tree_folds_are_bounded_and_the_oldest_go_first() {
        let mut first: HashSet<PathBuf> = (0..MAX_TREE_FOLDS)
            .map(|i| PathBuf::from(format!("d{i:04}")))
            .collect();
        let prev = Folds::default().with_trees(&mut first, &mut HashSet::new(), &Folds::default());
        assert_eq!(prev.status_folders.len(), MAX_TREE_FOLDS);
        let mut more = first.clone();
        more.insert(PathBuf::from("new"));
        let next = Folds::default().with_trees(&mut more, &mut HashSet::new(), &prev);
        assert_eq!(next.status_folders.len(), MAX_TREE_FOLDS);
        assert_eq!(next.status_folders.last().unwrap(), "new");
        assert!(!next.status_folders.contains(&"d0000".to_string()));
        // Unfolding drops the entry; unchanged sets compare equal.
        assert!(!more.contains(&PathBuf::from("d0000")));
        let same = Folds::default().with_trees(&mut more, &mut HashSet::new(), &next);
        assert_eq!(same.status_folders, next.status_folders);
    }

    #[test]
    fn a_repo_never_folded_has_no_folds_and_the_default_pr_groups() {
        let dir = tempfile::tempdir().unwrap();
        let back = load(dir.path());
        assert!(back.sidebar.is_empty() && back.status.is_empty());
        assert!(back.pr_groups.is_none());
    }
}
