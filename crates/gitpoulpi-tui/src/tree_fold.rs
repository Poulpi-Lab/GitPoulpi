//! Expand / collapse every folder of a tree, as one vocabulary for all of
//! them: the refs sections (LOCAL, REMOTE, TAGS), the Unstaged and Staged
//! lists, the commit's changed files and View all files.
//!
//! Pure: this decides *which folders* an operation touches and edits a set of
//! collapsed folders; `event_loop::input::commands::folds` finds the tree under
//! the cursor, the menus and the keys call it, so none can drift.
//!
//! **While a filter is active** (the refs name filter, the commit-files
//! filter) the tree shows the folders that hold a match, all open, whatever
//! the user had folded: `FilterOverlay` swaps the live collapsed set for an
//! empty one, and parks the user's own set until the filter changes or clears.
//! These operations act on the live set, so during a filter they act on the
//! visible tree and are forgotten with it; the saved state is never touched.

use crate::context_menu::Entry;
use crate::keymap::Action;
use gitpoulpi_i18n::msg::tree_fold as msg;
use std::collections::HashSet;
use std::hash::Hash;
use std::path::{Path, PathBuf};

/// One expand/collapse operation. "Below" acts on a folder and everything
/// under it; the others on every folder of the tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum TreeFold {
    ExpandAll,
    CollapseAll,
    ExpandBelow,
    CollapseBelow,
}

/// Which pane's key table a fold operation is looked up in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FoldCtx {
    Refs,
    Status,
    LogFiles,
}

impl TreeFold {
    pub(crate) const ALL: [TreeFold; 4] = [
        TreeFold::ExpandAll,
        TreeFold::CollapseAll,
        TreeFold::ExpandBelow,
        TreeFold::CollapseBelow,
    ];

    pub(crate) fn collapses(self) -> bool {
        matches!(self, TreeFold::CollapseAll | TreeFold::CollapseBelow)
    }

    pub(crate) fn below(self) -> bool {
        matches!(self, TreeFold::ExpandBelow | TreeFold::CollapseBelow)
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            TreeFold::ExpandAll => msg::EXPAND_ALL.get(),
            TreeFold::CollapseAll => msg::COLLAPSE_ALL.get(),
            TreeFold::ExpandBelow => msg::EXPAND_BELOW.get(),
            TreeFold::CollapseBelow => msg::COLLAPSE_BELOW.get(),
        }
    }

    /// The registry action of this operation in `ctx`'s pane.
    pub(crate) fn key(self, ctx: FoldCtx) -> Action {
        use FoldCtx as C;
        use TreeFold as T;
        match (ctx, self) {
            (C::Refs, T::ExpandAll) => Action::RefsExpandAll,
            (C::Refs, T::CollapseAll) => Action::RefsCollapseAll,
            (C::Refs, T::ExpandBelow) => Action::RefsExpandBelow,
            (C::Refs, T::CollapseBelow) => Action::RefsCollapseBelow,
            (C::Status, T::ExpandAll) => Action::StatusExpandAll,
            (C::Status, T::CollapseAll) => Action::StatusCollapseAllFolders,
            (C::Status, T::ExpandBelow) => Action::StatusExpandBelow,
            (C::Status, T::CollapseBelow) => Action::StatusCollapseBelow,
            (C::LogFiles, T::ExpandAll) => Action::LogFilesExpandAll,
            (C::LogFiles, T::CollapseAll) => Action::LogFilesCollapseAllFolders,
            (C::LogFiles, T::ExpandBelow) => Action::LogFilesExpandBelow,
            (C::LogFiles, T::CollapseBelow) => Action::LogFilesCollapseBelow,
        }
    }

    /// The operation a bound key stands for in `ctx`, if any.
    pub(crate) fn from_key(
        km: &crate::keymap::Keymap,
        ctx: FoldCtx,
        key: &crossterm::event::KeyEvent,
    ) -> Option<TreeFold> {
        TreeFold::ALL.into_iter().find(|op| km.is(op.key(ctx), key))
    }
}

/// Collapses or expands `folders` in `set`.
pub(crate) fn apply<K: Eq + Hash>(
    op: TreeFold,
    set: &mut HashSet<K>,
    folders: impl IntoIterator<Item = K>,
) {
    for folder in folders {
        if op.collapses() {
            set.insert(folder);
        } else {
            set.remove(&folder);
        }
    }
}

/// The menu rows of the operations in `ops`, each showing its key in `ctx`.
pub(crate) fn entries<A>(
    ops: &[TreeFold],
    ctx: FoldCtx,
    wrap: impl Fn(TreeFold) -> A,
) -> Vec<Entry<A>> {
    ops.iter()
        .map(|op| Entry::action(op.label(), wrap(*op)).with_key(op.key(ctx)))
        .collect()
}

/// Every folder on the way to the files in `files` (a file's own directories
/// and their parents), as repo-relative paths.
pub(crate) fn file_folders(files: &[PathBuf]) -> HashSet<PathBuf> {
    crate::status_pane::all_folders(files)
}

/// `folders` restricted to `root` and what lies under it.
pub(crate) fn file_folders_below(files: &[PathBuf], root: &Path) -> HashSet<PathBuf> {
    file_folders(files)
        .into_iter()
        .filter(|f| f.starts_with(root))
        .collect()
}

/// The folder paths of a refs section's rows (`RefTreeRow::Folder`), all of
/// them or those at and under `root`. A path is under `root` when it is
/// `root` itself or starts with `root/`.
pub(crate) fn ref_folders(
    rows: &[crate::refs_tree::RefTreeRow],
    root: Option<&str>,
) -> Vec<String> {
    rows.iter()
        .filter_map(|r| r.folder_path())
        .filter(|p| match root {
            None => true,
            Some(root) => *p == root || p.starts_with(&format!("{root}/")),
        })
        .map(str::to_string)
        .collect()
}

/// While a filter is active, the folder state the tree is drawn with is a
/// separate, initially empty one (every folder with a match open), and the
/// user's own state is parked here untouched. Generic over the collapsed-set
/// type; `live` is the field every handler already reads and edits.
#[derive(Debug, Default)]
pub(crate) struct FilterOverlay<S> {
    applied: String,
    parked: Option<S>,
}

impl<S: Clone + Default> FilterOverlay<S> {
    /// Brings the overlay in line with `filter`, once per change: parks the
    /// real state and clears `live` for a new non-empty filter, puts the real
    /// state back when the filter changes or clears. Cheap when nothing
    /// changed, so it runs every frame.
    pub(crate) fn sync(&mut self, filter: &str, live: &mut S) {
        if self.applied == filter {
            return;
        }
        if let Some(real) = self.parked.take() {
            *live = real;
        }
        if !filter.is_empty() {
            self.parked = Some(std::mem::take(live));
        }
        self.applied = filter.to_string();
    }

    /// The user's own state: what is saved to disk.
    pub(crate) fn real<'a>(&'a self, live: &'a S) -> &'a S {
        self.parked.as_ref().unwrap_or(live)
    }

    pub(crate) fn real_mut<'a>(&'a mut self, live: &'a mut S) -> &'a mut S {
        self.parked.as_mut().unwrap_or(live)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::refs_tree::{ref_display_rows, CollapsedFolders, RefRowKind};

    fn names(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    fn paths(v: &[&str]) -> Vec<PathBuf> {
        v.iter().map(PathBuf::from).collect()
    }

    #[test]
    fn collapse_inserts_and_expand_removes() {
        let mut set: HashSet<&str> = HashSet::from(["a"]);
        apply(TreeFold::CollapseAll, &mut set, ["a", "b"]);
        assert_eq!(set, HashSet::from(["a", "b"]));
        apply(TreeFold::ExpandBelow, &mut set, ["a"]);
        assert_eq!(set, HashSet::from(["b"]));
    }

    #[test]
    fn ref_folders_below_a_root_include_the_root_and_not_a_sibling_prefix() {
        let all = names(&["feature/a/x", "feature/b", "featured/c", "fix/z"]);
        let rows = ref_display_rows(&all, RefRowKind::Local, true, &CollapsedFolders::new(), &[]);
        let mut every = ref_folders(&rows, None);
        every.sort();
        assert_eq!(every, ["feature", "feature/a", "featured", "fix"]);
        let mut below = ref_folders(&rows, Some("feature"));
        below.sort();
        assert_eq!(
            below,
            ["feature", "feature/a"],
            "`featured` is not under `feature`"
        );
    }

    #[test]
    fn file_folders_below_a_root_are_that_folder_and_its_subfolders() {
        let files = paths(&["src/a/x.rs", "src/b.rs", "docs/c.md", "srcx/d.rs"]);
        let below = file_folders_below(&files, Path::new("src"));
        assert_eq!(
            below,
            HashSet::from([PathBuf::from("src"), PathBuf::from("src/a")])
        );
        assert_eq!(file_folders(&files).len(), 4);
    }

    #[test]
    fn a_filter_parks_the_users_folds_and_clearing_it_brings_them_back() {
        let mut live: HashSet<&str> = HashSet::from(["a", "b"]);
        let mut overlay: FilterOverlay<HashSet<&str>> = FilterOverlay::default();
        overlay.sync("", &mut live);
        assert_eq!(live.len(), 2);
        overlay.sync("x", &mut live);
        assert!(live.is_empty(), "every folder with a match is open");
        assert_eq!(overlay.real(&live).len(), 2, "the saved state is untouched");
        // A fold made during the filter lives in the overlay only.
        live.insert("c");
        assert_eq!(overlay.real(&live).len(), 2);
        // A new filter starts over from the user's state, forgetting `c`.
        overlay.sync("xy", &mut live);
        assert!(live.is_empty());
        overlay.sync("", &mut live);
        assert_eq!(live, HashSet::from(["a", "b"]));
        assert_eq!(overlay.real(&live), &live);
    }
}
