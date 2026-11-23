use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::Result;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TabEntry {
    pub repo_path: PathBuf,
    pub active: bool,
}

/// The three-pane split's persisted shape: the two divider percentages
/// (`pane_areas`' `divider1_pct`/`divider2_pct` in `gitpoulpi-tui`) and
/// whether the left (Refs) and/or right (Status) pane is collapsed.
///
/// Collapsing never touches the percentages - it only asks the renderer
/// to ignore them for a frame - which is what makes restoring a pane
/// bring back its previous width instead of a default. Window-scoped
/// (a session, not a repository): the same reasoning `GlobalSession`
/// itself already carries, and why this lives here rather than in
/// `RepoState`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PaneLayout {
    pub divider1_pct: u16,
    pub divider2_pct: u16,
    pub left_collapsed: bool,
    pub right_collapsed: bool,
    /// The heights (border rows included) the user dragged the Refs
    /// sidebar's sub-sections to, indexed by the section's position in
    /// the sidebar's fixed order (Local, Remote, Worktrees, Stashes, Tags,
    /// Pull requests, Issues). `0` (or a missing/short entry) means "no
    /// override, use the automatic height". A `Vec` rather than an array
    /// so a session file written by a build with more or fewer sections
    /// still loads.
    pub sidebar_heights: Vec<u16>,
    /// Content rows the user dragged the right panel's commit message
    /// block to (`0` = automatic: as tall as the text, up to a cap).
    pub commit_message_rows: u16,
    /// The merge editor's A / Base / B width weights (shares of 10,000, in
    /// that order): a ratio, so a size saved on a wide terminal still
    /// fits a narrow one. Empty (or not three entries) = the equal share.
    pub merge_cols: Vec<u16>,
    /// The merge editor's Output height, in percent of the body. `0` = the
    /// default share. The short-terminal "top only / Output only" halves
    /// are transient and never stored.
    pub merge_output_pct: u16,
    /// The folder picker's left column, in per mille of its pane row.
    /// `0` = the proportional default.
    pub picker_split_permille: u16,
    /// The History view's list width, in percent of the central column.
    /// `0` = the default share.
    pub history_split_pct: u16,
    /// The commit column left of the Diff / File View / Blame / Edit views:
    /// `0` = automatic (shown from `COMMIT_COLUMN_AUTO_WIDTH` terminal
    /// columns up), `1` = the user turned it on, `2` = the user turned it off.
    pub commit_column: u8,
    /// Its width in cells (border included). `0` = the default.
    pub commit_column_cols: u16,
    /// Which list it shows when shown: `0` = every commit with the graph
    /// (what a session written before this field meant), `1` = the open
    /// file's commits. Kept while the column is hidden.
    pub commit_column_mode: u8,
    /// Legacy: the part widths used to live here; they are now
    /// `ui.commit_column_columns_widths` in `config.toml`. Only read, once,
    /// to migrate a session written before (lanes, author, date, refs,
    /// message; `0` = automatic), and never written again.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub commit_column_widths: Vec<u16>,
    /// Legacy: the shown parts used to live here; they are now
    /// `ui.commit_column_columns_visible` in `config.toml`. Only read, once,
    /// to migrate a session written before, and never written again.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub commit_column_visible: Vec<String>,
}

/// The two pane dividers stay at least this far (percent) from the edges
/// and from each other (`MIN_PANE_PERCENT` in `gitpoulpi-tui`).
const MIN_DIVIDER_GAP: u16 = 10;
/// Bounds of the merge Output share the drag itself keeps to.
pub const MERGE_OUTPUT_PCT_RANGE: (u16, u16) = (10, 85);
/// The folder picker's left column bounds (per mille); the renderer clamps
/// again to cells.
pub const PICKER_SPLIT_RANGE: (u16, u16) = (50, 800);
/// The History view's list width bounds (percent); the renderer clamps
/// again to cells.
pub const HISTORY_SPLIT_RANGE: (u16, u16) = (15, 75);
/// The commit column's width bounds (cells); the renderer clamps again to
/// what the central column leaves.
pub const COMMIT_COLUMN_COLS_RANGE: (u16, u16) = (20, 400);
/// The widths of the commit column's parts (cells), when dragged.
pub const COMMIT_COLUMN_PART_RANGE: (u16, u16) = (2, 300);

impl PaneLayout {
    /// This layout with every value brought into the range the renderer
    /// can draw: what a hand-edited or older/newer file may hold must not
    /// break the screen. Idempotent.
    pub fn sanitized(mut self) -> Self {
        self.divider1_pct = self
            .divider1_pct
            .clamp(MIN_DIVIDER_GAP, 100 - 2 * MIN_DIVIDER_GAP);
        self.divider2_pct = self
            .divider2_pct
            .clamp(self.divider1_pct + MIN_DIVIDER_GAP, 100 - MIN_DIVIDER_GAP);
        if self.merge_cols.len() != 3 {
            self.merge_cols.clear();
        } else {
            for w in &mut self.merge_cols {
                *w = (*w).clamp(1, 10_000);
            }
        }
        if self.merge_output_pct != 0 {
            let (lo, hi) = MERGE_OUTPUT_PCT_RANGE;
            self.merge_output_pct = self.merge_output_pct.clamp(lo, hi);
        }
        if self.picker_split_permille != 0 {
            let (lo, hi) = PICKER_SPLIT_RANGE;
            self.picker_split_permille = self.picker_split_permille.clamp(lo, hi);
        }
        if self.history_split_pct != 0 {
            let (lo, hi) = HISTORY_SPLIT_RANGE;
            self.history_split_pct = self.history_split_pct.clamp(lo, hi);
        }
        if self.commit_column > 2 {
            self.commit_column = 0;
        }
        if self.commit_column_cols != 0 {
            let (lo, hi) = COMMIT_COLUMN_COLS_RANGE;
            self.commit_column_cols = self.commit_column_cols.clamp(lo, hi);
        }
        if self.commit_column_mode > 1 {
            self.commit_column_mode = 0;
        }
        self.commit_column_widths.truncate(5);
        self.commit_column_visible
            .retain(|n| matches!(n.as_str(), "ref" | "graph" | "author" | "date" | "message"));
        self.commit_column_visible.dedup();
        let (lo, hi) = COMMIT_COLUMN_PART_RANGE;
        for w in &mut self.commit_column_widths {
            if *w != 0 {
                *w = (*w).clamp(lo, hi);
            }
        }
        self
    }
}

impl Default for PaneLayout {
    fn default() -> Self {
        Self {
            divider1_pct: 20,
            divider2_pct: 65,
            left_collapsed: false,
            right_collapsed: false,
            sidebar_heights: Vec::new(),
            commit_message_rows: 0,
            merge_cols: Vec::new(),
            merge_output_pct: 0,
            picker_split_permille: 0,
            history_split_pct: 0,
            commit_column: 0,
            commit_column_cols: 0,
            commit_column_mode: 0,
            commit_column_widths: Vec::new(),
            commit_column_visible: Vec::new(),
        }
    }
}

/// Per-named-session global state: tab list + layout, persisted to
/// `sessions/<name>.toml` (§5). Independent from other named sessions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalSession {
    /// The workspace this window is in (`workspaces.toml` id). `None` = no
    /// workspace: `tabs` is the tab list. Persisted so the next launch comes
    /// back in the same workspace. Written before the tables (TOML wants
    /// plain values first).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_workspace: Option<String>,
    /// The user closed this session's Radar tab (its `x`); it stays
    /// closed on the next launch until reopened. Independent of the global
    /// `general.radar_enabled`. Old files: open.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub radar_closed: bool,
    /// The Workspaces tab is open (`+` menu, `Alt+J`, the palette); it
    /// stays open on the next launch until closed with its `x`. Old files:
    /// closed.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub workspaces_open: bool,
    #[serde(default)]
    pub tabs: Vec<TabEntry>,
    #[serde(default)]
    pub layout: PaneLayout,
    /// Each workspace's own tab list, `[workspace_tabs.<id>]`: its
    /// repositories at first, then what the user opens or closes while it is
    /// active. Old files have none (serde default).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub workspace_tabs: BTreeMap<String, WorkspaceTabs>,
}

/// One workspace's remembered tabs inside a session file.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceTabs {
    #[serde(default)]
    pub tabs: Vec<TabEntry>,
}

impl GlobalSession {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&raw)?)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            gitpoulpi_config::secure_fs::create_private_dir_all(parent)?;
        }
        let raw = toml::to_string_pretty(self)?;
        gitpoulpi_config::secure_fs::write_private(path, raw.as_bytes())?;
        Ok(())
    }

    /// Opens `repo_path` as a tab and focuses it, or focuses the
    /// existing tab if it is already open. Returns its index.
    ///
    /// Already open focuses it rather than being an error, so *Open
    /// recent* works for the repository the user is most likely to pick.
    ///
    /// Paths are compared *normalized*, not as given - see
    /// [`same_repo_path`]: the daemon keys on the root
    /// `GitBackend::discover` reports, but a path from a command line is
    /// whatever the user typed.
    pub fn open(&mut self, repo_path: &Path) -> usize {
        if let Some(index) = self.index_of(repo_path) {
            self.activate(index);
            return index;
        }
        for tab in &mut self.tabs {
            tab.active = false;
        }
        self.tabs.push(TabEntry {
            repo_path: repo_path.to_path_buf(),
            active: true,
        });
        self.tabs.len() - 1
    }

    /// Closes the tab at `index` and focuses a neighbor, preferring the
    /// one to its left - the same rule browsers use, and the one that
    /// keeps closing several tabs in a row from jumping across the bar.
    /// Returns whether anything was closed.
    pub fn close(&mut self, index: usize) -> bool {
        if index >= self.tabs.len() {
            return false;
        }
        let was_active = self.tabs[index].active;
        self.tabs.remove(index);
        if was_active && !self.tabs.is_empty() {
            self.activate(index.saturating_sub(1).min(self.tabs.len() - 1));
        }
        true
    }

    /// Moves the tab at `from` so it ends up at index `to` (the other tabs
    /// shift over). Each tab keeps its own `active` flag, so the focused
    /// tab stays the same repository wherever it lands. Returns whether
    /// anything moved: out of range, or `from == to`, is a no-op.
    pub fn move_tab(&mut self, from: usize, to: usize) -> bool {
        if from >= self.tabs.len() || to >= self.tabs.len() || from == to {
            return false;
        }
        let tab = self.tabs.remove(from);
        self.tabs.insert(to, tab);
        true
    }

    /// Groups linked-worktree tabs behind their repository's tab
    /// ("Automatically group worktree tabs together"): every tab whose
    /// `main_of` names a repository that is also open follows that tab, in
    /// their own relative order; everything else keeps its place. A worktree
    /// whose repository tab is not open stays where it is. Returns whether
    /// the order changed. Each tab keeps its own `active` flag.
    pub fn group_worktrees(&mut self, main_of: impl Fn(&Path) -> Option<PathBuf>) -> bool {
        let parent: Vec<Option<usize>> = self
            .tabs
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let main = main_of(&t.repo_path)?;
                self.tabs
                    .iter()
                    .position(|o| same_repo_path(&o.repo_path, &main))
                    .filter(|&p| p != i && main_of(&self.tabs[p].repo_path).is_none())
            })
            .collect();
        let mut order = Vec::with_capacity(self.tabs.len());
        for i in 0..self.tabs.len() {
            if parent[i].is_some() {
                continue;
            }
            order.push(i);
            order.extend((0..self.tabs.len()).filter(|&c| parent[c] == Some(i)));
        }
        if order.iter().enumerate().all(|(at, &i)| at == i) {
            return false;
        }
        let mut old: Vec<Option<TabEntry>> = std::mem::take(&mut self.tabs)
            .into_iter()
            .map(Some)
            .collect();
        self.tabs = order.into_iter().filter_map(|i| old[i].take()).collect();
        true
    }

    pub fn close_path(&mut self, repo_path: &Path) -> bool {
        match self.index_of(repo_path) {
            Some(index) => self.close(index),
            None => false,
        }
    }

    /// Focuses exactly one tab. Out-of-range is a no-op rather than a
    /// panic: the index can come from a click on a bar that has since
    /// been redrawn.
    pub fn activate(&mut self, index: usize) -> bool {
        if index >= self.tabs.len() {
            return false;
        }
        for (i, tab) in self.tabs.iter_mut().enumerate() {
            tab.active = i == index;
        }
        true
    }

    pub fn index_of(&self, repo_path: &Path) -> Option<usize> {
        self.tabs
            .iter()
            .position(|t| same_repo_path(&t.repo_path, repo_path))
    }

    /// The focused tab.
    ///
    /// A session restored from a file that marks two tabs active (a
    /// hand-edit, or a half-written save) answers with the first, and a
    /// file that marks none answers with the first tab rather than
    /// `None`: a non-empty session always has something on screen.
    pub fn active(&self) -> Option<&TabEntry> {
        self.tabs
            .iter()
            .find(|t| t.active)
            .or_else(|| self.tabs.first())
    }

    pub fn active_index(&self) -> Option<usize> {
        self.tabs
            .iter()
            .position(|t| t.active)
            .or(if self.tabs.is_empty() { None } else { Some(0) })
    }

    /// Focuses the next/previous tab, wrapping. `delta` is signed so one
    /// function serves both directions and the wrap arithmetic is
    /// written once.
    pub fn cycle(&mut self, delta: isize) -> Option<usize> {
        let len = self.tabs.len();
        if len == 0 {
            return None;
        }
        let current = self.active_index().unwrap_or(0) as isize;
        let next = current.rem_euclid(len as isize);
        let next = (next + delta).rem_euclid(len as isize) as usize;
        self.activate(next);
        Some(next)
    }
}

/// Whether two paths name the same repository.
///
/// Not `==`. A tab's path is the root `GitBackend::discover` reported,
/// which git canonicalises; the path handed to `gitpoulpi repo close` is
/// whatever was typed, made absolute and nothing more. The same directory
/// reaches this function under two spellings:
///
/// - **macOS** puts temporary directories under `/var`, a symlink to
///   `/private/var`. `discover` returns the latter.
/// - **Windows** hands out 8.3 short names when a profile directory's long
///   name is too long (`runneradmin` becomes `RUNNER~1`) and compares paths
///   case-insensitively, which `PathBuf` does not.
/// - **Linux** has neither.
///
/// `canonicalize` answers all three, but only for a path that still
/// exists, and on Windows it returns a verbatim prefix. So: canonicalise
/// when we can, strip that prefix, and fall back to a lexical cleanup for
/// a deleted repository, which still has to be closable.
pub fn same_repo_path(a: &Path, b: &Path) -> bool {
    let (a, b) = (normalize(a), normalize(b));
    if cfg!(windows) {
        a.as_os_str()
            .to_string_lossy()
            .eq_ignore_ascii_case(&b.as_os_str().to_string_lossy())
    } else {
        a == b
    }
}

/// The main working tree of the repository `path` is a *linked worktree* of,
/// read from the file system (`.git` is a `gitdir:` file whose common dir is
/// the main tree's `.git`). `None` for a plain repository, the main tree
/// itself, a submodule, or a path that is not a repository.
pub fn linked_worktree_main(path: &Path) -> Option<PathBuf> {
    if !path.join(".git").is_file() {
        return None;
    }
    let common = gitpoulpi_config::git_common_dir(path)?;
    if common.file_name()? != ".git" {
        return None;
    }
    let main = common.parent()?.to_path_buf();
    (!same_repo_path(path, &main)).then_some(main)
}

/// A string two spellings of one repository share: the normalized path,
/// lower-cased on Windows. For callers joining many paths, who would
/// otherwise canonicalise each one per comparison.
pub fn repo_path_key(path: &Path) -> String {
    let text = normalize(path).to_string_lossy().into_owned();
    if cfg!(windows) {
        text.to_lowercase()
    } else {
        text
    }
}

/// The comparable form of `path`. See [`same_repo_path`].
///
/// The verbatim-prefix strip is [`gitpoulpi_core::paths::strip_verbatim_prefix`],
/// shared with `RepoRef::from_root`.
fn normalize(path: &Path) -> PathBuf {
    match std::fs::canonicalize(path) {
        Ok(real) => gitpoulpi_core::paths::strip_verbatim_prefix(&real),
        // Gone from disk, or unreadable. A lexical cleanup still collapses
        // the `.` and `..` a typed path carries, and the deepest ancestor
        // that exists is canonicalised, so a deleted repository under a
        // symlinked directory (macOS `/var`) is spelled like a live one.
        Err(_) => {
            let clean = lexical(path);
            canonical_ancestor(&clean).unwrap_or(clean)
        }
    }
}

/// `path` with its deepest existing ancestor canonicalised and the missing
/// rest appended as is. `None` when no ancestor can be canonicalised.
fn canonical_ancestor(path: &Path) -> Option<PathBuf> {
    let mut rest = Vec::new();
    let mut probe = path;
    loop {
        rest.push(probe.file_name()?);
        probe = probe.parent()?;
        if let Ok(real) = std::fs::canonicalize(probe) {
            let mut out = gitpoulpi_core::paths::strip_verbatim_prefix(&real);
            out.extend(rest.iter().rev());
            return Some(out);
        }
    }
}

/// `.` and `..` resolved without touching the filesystem.
fn lexical(path: &Path) -> PathBuf {
    use std::path::Component;
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    out.push("..");
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn paths(s: &GlobalSession) -> Vec<String> {
        s.tabs
            .iter()
            .map(|t| t.repo_path.to_string_lossy().into_owned())
            .collect()
    }

    /// `/wt-a*` are worktrees of `/a`, `/wt-b*` of `/b`, everything else is plain.
    fn fake_main(p: &Path) -> Option<PathBuf> {
        let name = p.file_name()?.to_str()?;
        name.strip_prefix("wt-a")
            .map(|_| PathBuf::from("/a"))
            .or_else(|| name.strip_prefix("wt-b").map(|_| PathBuf::from("/b")))
    }

    /// A layout written before the column kept its list and its part widths
    /// loads as it always did: all commits, every width automatic. Red when
    /// the new fields have no default.
    #[test]
    fn a_layout_from_before_the_list_and_the_part_widths_loads_with_defaults() {
        let layout: PaneLayout =
            toml::from_str("commit_column = 1\ncommit_column_cols = 40\n").unwrap();
        assert_eq!((layout.commit_column, layout.commit_column_cols), (1, 40));
        assert_eq!(layout.commit_column_mode, 0);
        assert!(layout.commit_column_widths.is_empty());
    }

    #[test]
    fn the_part_widths_and_the_list_are_brought_into_range() {
        let layout = PaneLayout {
            commit_column_mode: 9,
            commit_column_widths: vec![0, 5000, 1, 7, 9, 11],
            ..PaneLayout::default()
        }
        .sanitized();
        assert_eq!(layout.commit_column_mode, 0);
        assert_eq!(layout.commit_column_widths, vec![0, 300, 2, 7, 9]);
    }

    #[test]
    fn worktree_tabs_follow_their_repository_tab_in_their_own_order() {
        let mut s = GlobalSession::default();
        for p in ["/wt-a2", "/a", "/c", "/wt-b1", "/wt-a1", "/b"] {
            s.open(Path::new(p));
        }
        s.activate(4);
        assert!(s.group_worktrees(fake_main));
        // The repositories keep their order; each one's worktrees follow it.
        assert_eq!(paths(&s), ["/a", "/wt-a2", "/wt-a1", "/c", "/b", "/wt-b1"]);
        // The focused tab is still the same repository.
        assert_eq!(s.active().unwrap().repo_path, Path::new("/wt-a1"));
        // Already grouped: nothing moves.
        assert!(!s.group_worktrees(fake_main));
    }

    #[test]
    fn a_worktree_whose_repository_is_not_open_stays_where_it_is() {
        let mut s = GlobalSession::default();
        for p in ["/x", "/wt-a1", "/y"] {
            s.open(Path::new(p));
        }
        assert!(!s.group_worktrees(fake_main));
        assert_eq!(paths(&s), ["/x", "/wt-a1", "/y"]);
    }

    #[test]
    fn a_worktree_dropped_away_snaps_back_next_to_its_group() {
        let mut s = GlobalSession::default();
        for p in ["/a", "/wt-a1", "/c"] {
            s.open(Path::new(p));
        }
        s.move_tab(1, 2);
        assert_eq!(paths(&s), ["/a", "/c", "/wt-a1"]);
        assert!(s.group_worktrees(fake_main));
        assert_eq!(paths(&s), ["/a", "/wt-a1", "/c"]);
    }

    #[test]
    fn a_real_linked_worktree_names_its_main_tree() {
        let dir = tempfile::tempdir().expect("tempdir");
        let main = dir.path().join("main");
        let wt = dir.path().join("wt");
        let git = |args: &[&str], cwd: &Path| {
            let ok = std::process::Command::new("git")
                .args(["-c", "user.name=t", "-c", "user.email=t@t"])
                .args(args)
                .current_dir(cwd)
                .status()
                .expect("git")
                .success();
            assert!(ok, "git {args:?}");
        };
        std::fs::create_dir(&main).unwrap();
        git(&["init", "-q"], &main);
        git(&["commit", "-q", "--allow-empty", "-m", "x"], &main);
        git(
            &["worktree", "add", "-q", wt.to_str().unwrap(), "-b", "o"],
            &main,
        );
        assert_eq!(linked_worktree_main(&main), None);
        let found = linked_worktree_main(&wt).expect("a linked worktree");
        assert!(same_repo_path(&found, &main), "{found:?}");
    }

    #[test]
    fn the_layout_round_trips_through_the_session_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("s.toml");
        let mut session = GlobalSession::default();
        session.layout.merge_cols = vec![3000, 4000, 3000];
        session.layout.merge_output_pct = 65;
        session.layout.picker_split_permille = 400;
        session.save(&file).expect("save");
        let back = GlobalSession::load(&file).expect("load").layout;
        assert_eq!(back, session.layout);
    }

    #[test]
    fn an_old_session_file_without_the_new_fields_loads_the_defaults() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("s.toml");
        std::fs::write(&file, "[layout]\ndivider1_pct = 30\ndivider2_pct = 70\n").unwrap();
        let layout = GlobalSession::load(&file).expect("load").layout;
        assert_eq!(layout.divider1_pct, 30);
        assert!(layout.merge_cols.is_empty());
        assert_eq!(layout.merge_output_pct, 0);
    }

    /// The tab lists of workspaces and the active one round-trip, an old
    /// file without them loads, and a file with none written stays as it was.
    #[test]
    fn workspace_tabs_round_trip_and_old_files_load() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("s.toml");
        std::fs::write(&file, "[[tabs]]\nrepo_path = \"/r/a\"\nactive = true\n").unwrap();
        let old = GlobalSession::load(&file).expect("load");
        assert_eq!(old.tabs.len(), 1);
        assert!(old.active_workspace.is_none() && old.workspace_tabs.is_empty());
        old.save(&file).unwrap();
        let raw = std::fs::read_to_string(&file).unwrap();
        assert!(!raw.contains("workspace"), "{raw}");

        let mut session = old;
        session.active_workspace = Some("acme".into());
        session.workspace_tabs.insert(
            "acme".into(),
            WorkspaceTabs {
                tabs: vec![
                    TabEntry {
                        repo_path: PathBuf::from("/r/api"),
                        active: false,
                    },
                    TabEntry {
                        repo_path: PathBuf::from("/r/web"),
                        active: true,
                    },
                ],
            },
        );
        session.save(&file).unwrap();
        let raw = std::fs::read_to_string(&file).unwrap();
        assert!(raw.contains("[[workspace_tabs.acme.tabs]]"), "{raw}");
        let back = GlobalSession::load(&file).expect("load");
        assert_eq!(back.active_workspace.as_deref(), Some("acme"));
        assert_eq!(back.workspace_tabs, session.workspace_tabs);
        assert_eq!(back.tabs, session.tabs);
    }

    #[test]
    fn sanitizing_clamps_a_layout_a_file_could_hold() {
        let wild = PaneLayout {
            divider1_pct: 95,
            divider2_pct: 3,
            merge_cols: vec![0, 60_000, 5],
            merge_output_pct: 200,
            picker_split_permille: 5000,
            ..PaneLayout::default()
        }
        .sanitized();
        assert!(wild.divider1_pct >= 10 && wild.divider1_pct <= 80);
        assert!(wild.divider2_pct >= wild.divider1_pct + 10 && wild.divider2_pct <= 90);
        assert_eq!(wild.merge_cols, vec![1, 10_000, 5]);
        assert_eq!(wild.merge_output_pct, 85);
        assert_eq!(wild.picker_split_permille, 800);
        let short = PaneLayout {
            merge_cols: vec![1, 2],
            ..PaneLayout::default()
        }
        .sanitized();
        assert!(short.merge_cols.is_empty(), "two weights are not a split");
        assert_eq!(PaneLayout::default().sanitized(), PaneLayout::default());
        assert_eq!(wild.clone().sanitized(), wild, "idempotent");
    }

    fn at(n: usize) -> PathBuf {
        PathBuf::from(format!("/repos/r{n}"))
    }

    /// The verbatim-prefix strip this module relies on
    /// (`gitpoulpi_core::paths::strip_verbatim_prefix`) must actually
    /// strip. Kept here too, thin, so a regression trips a local test
    /// rather than only one in `gitpoulpi-core`.
    #[test]
    fn the_verbatim_prefix_is_actually_stripped() {
        use gitpoulpi_core::paths::strip_verbatim_prefix;

        let plain = PathBuf::from(r"\\?\C:\src\GitPoulpi");
        assert_eq!(
            strip_verbatim_prefix(&plain).to_string_lossy(),
            r"C:\src\GitPoulpi"
        );

        let unc = PathBuf::from(r"\\?\UNC\server\share\repo");
        assert_eq!(
            strip_verbatim_prefix(&unc).to_string_lossy(),
            r"\\server\share\repo"
        );

        // And a path that never carried one is returned untouched.
        let untouched = PathBuf::from(r"C:\src\GitPoulpi");
        assert_eq!(
            strip_verbatim_prefix(&untouched),
            untouched,
            "a plain path must not be trimmed"
        );
    }

    /// A repository must be closable by a path that names it, not only
    /// by the exact string it was opened with.
    ///
    /// `repo open` stores the root `discover` canonicalised, `repo close`
    /// passes what the user typed made absolute; comparing them as strings
    /// left the tab open on macOS and Windows. The `.`/`..` case below
    /// fails on every platform without `same_repo_path`; the symlink and
    /// short-name halves are what those two platforms hit.
    #[test]
    fn a_tab_closes_by_any_spelling_of_its_path() {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = dir.path().join("work");
        std::fs::create_dir_all(&repo).expect("create");

        let mut session = GlobalSession::default();
        session.open(&repo);
        assert_eq!(session.tabs.len(), 1);

        let detour = repo.join("..").join("work").join(".");
        assert_ne!(detour, repo, "the point of the test is two spellings");
        assert!(
            session.close_path(&detour),
            "a detour through `..` and `.` names the same repository"
        );
        assert!(session.tabs.is_empty(), "{:?}", session.tabs);
    }

    /// And opening it twice under two spellings is one tab, not two.
    #[test]
    fn two_spellings_of_one_repository_are_one_tab() {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = dir.path().join("work");
        std::fs::create_dir_all(&repo).expect("create");

        let mut session = GlobalSession::default();
        let first = session.open(&repo);
        let second = session.open(&repo.join(".").join("..").join("work"));
        assert_eq!(first, second, "{:?}", session.tabs);
        assert_eq!(session.tabs.len(), 1);
    }

    /// A deleted repository under a symlinked directory is the same
    /// repository as its canonical spelling (macOS temp dirs are behind
    /// `/var` -> `/private/var`).
    #[cfg(unix)]
    #[test]
    fn a_vanished_path_behind_a_symlink_matches_its_canonical_spelling() {
        let dir = tempfile::tempdir().expect("tempdir");
        let real = dir.path().join("real");
        std::fs::create_dir_all(&real).expect("create");
        let link = dir.path().join("link");
        std::os::unix::fs::symlink(&real, &link).expect("symlink");
        assert!(same_repo_path(
            &link.join("gone"),
            &real.canonicalize().unwrap().join("gone")
        ));
        assert!(same_repo_path(
            &real.join("x").join("..").join("gone"),
            &link.join("gone")
        ));
        assert!(!same_repo_path(&link.join("gone"), &link.join("other")));
    }

    /// A repository deleted from disk must still be closable: the tab is
    /// the thing being closed, and `canonicalize` cannot help once the
    /// directory is gone.
    #[test]
    fn a_vanished_repository_is_still_closable() {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = dir.path().join("gone");
        std::fs::create_dir_all(&repo).expect("create");

        let mut session = GlobalSession::default();
        session.open(&repo);
        std::fs::remove_dir_all(&repo).expect("remove");

        assert!(session.close_path(&repo), "{:?}", session.tabs);
        assert!(session.tabs.is_empty());
    }

    #[test]
    fn opening_appends_and_focuses() {
        let mut session = GlobalSession::default();
        assert_eq!(session.open(&at(1)), 0);
        assert_eq!(session.open(&at(2)), 1);
        assert_eq!(session.active().unwrap().repo_path, at(2));
        assert_eq!(session.tabs.iter().filter(|t| t.active).count(), 1);
    }

    #[test]
    fn opening_an_already_open_repository_focuses_it_instead_of_duplicating() {
        let mut session = GlobalSession::default();
        session.open(&at(1));
        session.open(&at(2));
        assert_eq!(session.open(&at(1)), 0);
        assert_eq!(session.tabs.len(), 2);
        assert_eq!(session.active().unwrap().repo_path, at(1));
    }

    #[test]
    fn moving_a_tab_reorders_and_keeps_the_focus_on_the_same_repository() {
        let mut s = GlobalSession::default();
        for n in 1..=4 {
            s.open(&at(n));
        }
        s.activate(1);
        assert!(s.move_tab(1, 3));
        let order: Vec<_> = s.tabs.iter().map(|t| t.repo_path.clone()).collect();
        assert_eq!(order, vec![at(1), at(3), at(4), at(2)]);
        assert_eq!(s.active_index(), Some(3));
        assert!(s.move_tab(3, 0));
        assert_eq!(s.tabs[0].repo_path, at(2));
        assert_eq!(s.active_index(), Some(0));
        assert!(!s.move_tab(2, 2), "same slot is a no-op");
        assert!(!s.move_tab(0, 9), "out of range is a no-op");
        assert!(!s.move_tab(9, 0), "out of range is a no-op");
    }

    #[test]
    fn closing_the_focused_tab_focuses_its_left_neighbor() {
        let mut session = GlobalSession::default();
        session.open(&at(1));
        session.open(&at(2));
        session.open(&at(3));
        assert!(session.close(2));
        assert_eq!(session.active().unwrap().repo_path, at(2));

        // Closing the leftmost focused tab has no left neighbor, so it
        // takes the one that slid into its place.
        session.activate(0);
        assert!(session.close(0));
        assert_eq!(session.active().unwrap().repo_path, at(2));
    }

    /// Closing a tab that is not focused must not move the focus.
    #[test]
    fn closing_an_unfocused_tab_leaves_the_focus_alone() {
        let mut session = GlobalSession::default();
        session.open(&at(1));
        session.open(&at(2));
        assert!(session.close(0));
        assert_eq!(session.active().unwrap().repo_path, at(2));
    }

    #[test]
    fn cycling_wraps_in_both_directions() {
        let mut session = GlobalSession::default();
        session.open(&at(1));
        session.open(&at(2));
        session.open(&at(3));
        session.activate(0);

        assert_eq!(session.cycle(1), Some(1));
        assert_eq!(session.cycle(-1), Some(0));
        assert_eq!(session.cycle(-1), Some(2), "must wrap to the last tab");
        assert_eq!(session.cycle(1), Some(0), "must wrap to the first tab");
    }

    #[test]
    fn an_empty_session_has_no_active_tab_and_cannot_cycle() {
        let mut session = GlobalSession::default();
        assert!(session.active().is_none());
        assert_eq!(session.active_index(), None);
        assert_eq!(session.cycle(1), None);
        assert!(!session.close(0));
        assert!(!session.activate(0));
    }

    /// A file that marks nothing active still has to put something on
    /// screen, and one that marks two must pick one.
    #[test]
    fn a_hand_edited_file_still_resolves_to_one_focused_tab() {
        let none_active = GlobalSession {
            layout: Default::default(),
            tabs: vec![
                TabEntry {
                    repo_path: at(1),
                    active: false,
                },
                TabEntry {
                    repo_path: at(2),
                    active: false,
                },
            ],
            ..GlobalSession::default()
        };
        assert_eq!(none_active.active().unwrap().repo_path, at(1));
        assert_eq!(none_active.active_index(), Some(0));

        let two_active = GlobalSession {
            layout: Default::default(),
            tabs: vec![
                TabEntry {
                    repo_path: at(1),
                    active: true,
                },
                TabEntry {
                    repo_path: at(2),
                    active: true,
                },
            ],
            ..GlobalSession::default()
        };
        assert_eq!(two_active.active().unwrap().repo_path, at(1));
    }

    #[test]
    fn a_round_trip_through_the_file_preserves_the_tabs_and_the_focus() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("work.toml");

        let mut session = GlobalSession::default();
        session.open(&at(1));
        session.open(&at(2));
        session.activate(0);
        session.save(&file).unwrap();

        let loaded = GlobalSession::load(&file).unwrap();
        assert_eq!(loaded.tabs.len(), 2);
        assert_eq!(loaded.active().unwrap().repo_path, at(1));
    }
}
