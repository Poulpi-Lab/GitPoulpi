//! The Refs sidebar's tree: flat ref names in, displayed rows out.
//!
//! One pure function, [`ref_display_rows`], from *(the section's ref names,
//! the repository's remotes, which folders are collapsed)* to the exact list
//! of rows drawn. It is the single source of truth for both halves of the
//! pane: the render pass builds its `ListItem`s from it and every click,
//! `Up`/`Down`, drag and `Enter` resolves a row index through the same call.
//! A second walk recomputing "how many rows are there" would drift from the
//! one that drew them and every click would land on the wrong branch (the
//! discipline `log_table` and `build_status_pane_lines` enforce too).
//!
//! It is its own module because every interesting case is a *data* case (a
//! remote whose name contains a `/`, a branch that shadows a remote's name, a
//! folder and a branch competing for one segment), cheap to pin when the
//! function takes and returns data.
//!
//! **The shape.** `origin/feature/login` in the REMOTE section is
//! three nodes: the remote `origin` at the top
//! level (carrying its own icon), the folder `feature` under it, and the
//! leaf `login`. The branch's *displayed* text is the leaf only; its
//! `full_name` stays the `origin/feature/login` every other part of the
//! app keys on (hide-in-graph, solo, drag and drop).

use gitpoulpi_core::domain::split_remote_ref;
use std::collections::{BTreeMap, HashSet};

/// Mirrors `gitpoulpi_core::domain::RefKind` without the payload: what
/// color/icon a *ref* gets, in the Log column's pills and the sidebar's ref
/// rows. Which sidebar sub-pane is active is `SidebarSection`'s job:
/// stashes and worktrees are sections without being refs, and must never
/// acquire a pill.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RefRowKind {
    Local,
    Remote,
    Tag,
}

/// The set of folder rows the user has collapsed, keyed by section kind
/// and folder path so LOCAL's `feature` and REMOTE's `feature` collapse
/// independently.
pub type CollapsedFolders = HashSet<(RefRowKind, String)>;

/// One visible row of a Refs sub-pane's list, in display order - either
/// the flat "one row per ref" list (what the
/// whole pane uses with `ui.ref_tree` off), or a `/`-split folder tree's
/// flattened, collapse-aware rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefTreeRow {
    Folder {
        /// The collapse key: the folder's path *within its section*,
        /// which for a remote's own node is the remote's name.
        path: String,
        /// What is drawn: the last segment only.
        label: String,
        depth: u8,
        /// True for the top-level node that *is* a remote (`origin`,
        /// `gh`), as opposed to a `/`-derived folder inside one. It gets
        /// the remote icon; a plain folder does not. Always false outside
        /// the REMOTE section.
        is_remote_root: bool,
    },
    Leaf {
        /// The name every other part of the app keys on: a local branch
        /// or tag's bare name, a remote branch's `remote/name`. Never the
        /// abbreviated label.
        full_name: String,
        /// What is drawn: the leaf segment only in the tree view, the
        /// whole name when flat.
        display_name: String,
        depth: u8,
    },
}

impl RefTreeRow {
    pub fn depth(&self) -> u8 {
        match self {
            RefTreeRow::Folder { depth, .. } => *depth,
            RefTreeRow::Leaf { depth, .. } => *depth,
        }
    }

    /// The leaf's full name, or `None` for a folder row. Folders are not
    /// refs: nothing can be checked out, dragged, hidden or soloed from
    /// one, and every call site that forgets this acts on the wrong row.
    pub fn leaf_name(&self) -> Option<&str> {
        match self {
            RefTreeRow::Leaf { full_name, .. } => Some(full_name),
            RefTreeRow::Folder { .. } => None,
        }
    }

    /// The folder's collapse-key path, or `None` for a leaf row. Used both
    /// by the tests below and by "hide/solo this whole folder" (event_loop's
    /// selection resolvers and hit-testing), which need to know a folder
    /// row's path the same way `leaf_name` gives a leaf's.
    pub fn folder_path(&self) -> Option<&str> {
        match self {
            RefTreeRow::Folder { path, .. } => Some(path),
            RefTreeRow::Leaf { .. } => None,
        }
    }

    /// True for a REMOTE-section folder row that *is* a remote's own node
    /// (`origin`, `gh`), as opposed to a `/`-derived folder under one.
    /// Always false for a leaf and for a plain folder.
    pub fn is_remote_root(&self) -> bool {
        matches!(
            self,
            RefTreeRow::Folder {
                is_remote_root: true,
                ..
            }
        )
    }
}

enum Node {
    Leaf(String),
    Branch(BTreeMap<String, Node>),
}

/// Split one section entry into the path segments it occupies in the tree.
///
/// For LOCAL and TAGS that is just `name.split('/')`. For REMOTE the first
/// segment is the **remote**, resolved against the repository's actual
/// remote list rather than "everything before the first `/`": a remote may
/// be named `grp/sub`, and guessing puts its branches under a folder no
/// remote is called. The rest of the name then splits normally, so
/// `origin/feature/login` becomes `["origin", "feature", "login"]`.
fn segments<'a>(full_name: &'a str, kind: RefRowKind, remotes: &[String]) -> Vec<&'a str> {
    if kind != RefRowKind::Remote {
        return full_name.split('/').collect();
    }
    match split_remote_ref(full_name, remotes) {
        Some((remote, branch)) => {
            // `remote` is a prefix of `full_name` by construction, so
            // slicing it back out keeps the borrow on `full_name` and
            // avoids allocating a segment per row.
            let mut out = vec![&full_name[..remote.len()]];
            out.extend(branch.split('/'));
            out
        }
        None => vec![full_name],
    }
}

/// The rows a Refs sub-pane displays, flat or as a tree.
///
/// `hierarchical` off, or no name containing a `/`, produces the flat
/// one-row-per-ref list. Tags go hierarchical like branches: tags are named
/// by purpose (`releases/v0.2.0`, `backups/sweep`).
pub fn ref_display_rows(
    names: &[String],
    kind: RefRowKind,
    hierarchical: bool,
    collapsed_folders: &CollapsedFolders,
    remotes: &[String],
) -> Vec<RefTreeRow> {
    // A configured workflow groups LOCAL and REMOTE by branch type (base
    // branches, one folder per type, the rest) - one place, so every caller
    // of this function agrees on the rows (`flow::view::grouped_rows`).
    if let Some(rows) = crate::flow::view::grouped_rows(names, kind, collapsed_folders) {
        return rows;
    }
    let flat = || {
        names
            .iter()
            .map(|n| RefTreeRow::Leaf {
                full_name: n.clone(),
                display_name: n.clone(),
                depth: 0,
            })
            .collect::<Vec<_>>()
    };
    if !hierarchical || !names.iter().any(|n| n.contains('/')) {
        return flat();
    }

    let mut root: BTreeMap<String, Node> = BTreeMap::new();
    for full_name in names {
        let parts = segments(full_name, kind, remotes);
        let mut node_map = &mut root;
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                node_map.insert(part.to_string(), Node::Leaf(full_name.clone()));
            } else {
                let entry = node_map
                    .entry(part.to_string())
                    .or_insert_with(|| Node::Branch(BTreeMap::new()));
                match entry {
                    Node::Branch(children) => node_map = children,
                    // A ref named the same as another ref's folder segment
                    // (both `release` and `release/1.0`): git forbids that
                    // within one namespace, so it can only happen across two
                    // remotes, which the remote-level split already
                    // separates. Drop the conflicting deeper entry rather
                    // than panicking or losing the leaf already there.
                    Node::Leaf(_) => break,
                }
            }
        }
    }

    let mut out = Vec::new();
    flatten(&root, "", 0, kind, collapsed_folders, &mut out);
    out
}

fn flatten(
    node_map: &BTreeMap<String, Node>,
    path_prefix: &str,
    depth: u8,
    kind: RefRowKind,
    collapsed_folders: &CollapsedFolders,
    out: &mut Vec<RefTreeRow>,
) {
    for (segment, node) in node_map {
        let path = if path_prefix.is_empty() {
            segment.clone()
        } else {
            format!("{path_prefix}/{segment}")
        };
        match node {
            Node::Leaf(full_name) => out.push(RefTreeRow::Leaf {
                full_name: full_name.clone(),
                display_name: segment.clone(),
                depth,
            }),
            Node::Branch(children) => {
                out.push(RefTreeRow::Folder {
                    path: path.clone(),
                    label: segment.clone(),
                    depth,
                    is_remote_root: kind == RefRowKind::Remote && depth == 0,
                });
                if !collapsed_folders.contains(&(kind, path.clone())) {
                    flatten(children, &path, depth + 1, kind, collapsed_folders, out);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    fn collapsed(v: &[(RefRowKind, &str)]) -> CollapsedFolders {
        v.iter().map(|(k, p)| (*k, p.to_string())).collect()
    }

    /// `(depth, label, is_folder)` per row - the shape assertions read
    /// against, since that is what the pane actually draws.
    fn shape(rows: &[RefTreeRow]) -> Vec<(u8, String, bool)> {
        rows.iter()
            .map(|r| match r {
                RefTreeRow::Folder { label, depth, .. } => (*depth, label.clone(), true),
                RefTreeRow::Leaf {
                    display_name,
                    depth,
                    ..
                } => (*depth, display_name.clone(), false),
            })
            .collect()
    }

    fn labels(rows: &[RefTreeRow]) -> Vec<String> {
        shape(rows).into_iter().map(|(_, l, _)| l).collect()
    }

    #[test]
    fn flat_when_the_tree_is_off() {
        let rows = ref_display_rows(
            &names(&["feature/login", "main"]),
            RefRowKind::Local,
            false,
            &collapsed(&[]),
            &[],
        );
        assert_eq!(labels(&rows), vec!["feature/login", "main"]);
        assert!(rows.iter().all(|r| r.depth() == 0));
    }

    #[test]
    fn a_slash_makes_a_folder_in_the_local_section() {
        let rows = ref_display_rows(
            &names(&["bugfix/math-overflow", "feature/login", "main"]),
            RefRowKind::Local,
            true,
            &collapsed(&[]),
            &[],
        );
        assert_eq!(
            shape(&rows),
            vec![
                (0, "bugfix".into(), true),
                (1, "math-overflow".into(), false),
                (0, "feature".into(), true),
                (1, "login".into(), false),
                (0, "main".into(), false),
            ]
        );
    }

    #[test]
    fn nesting_goes_as_deep_as_the_name_does() {
        let rows = ref_display_rows(
            &names(&["test/by/me"]),
            RefRowKind::Local,
            true,
            &collapsed(&[]),
            &[],
        );
        assert_eq!(
            shape(&rows),
            vec![
                (0, "test".into(), true),
                (1, "by".into(), true),
                (2, "me".into(), false),
            ]
        );
    }

    /// The remote's name becomes the
    /// top-level node, and the branch under it shows its leaf only.
    #[test]
    fn the_remote_name_becomes_the_top_level_folder() {
        let rows = ref_display_rows(
            &names(&["origin/feature/login", "origin/main"]),
            RefRowKind::Remote,
            true,
            &collapsed(&[]),
            &names(&["origin"]),
        );
        assert_eq!(
            shape(&rows),
            vec![
                (0, "origin".into(), true),
                (1, "feature".into(), true),
                (2, "login".into(), false),
                (1, "main".into(), false),
            ]
        );
        assert!(matches!(
            rows[0],
            RefTreeRow::Folder {
                is_remote_root: true,
                ..
            }
        ));
        // The `/`-derived folder inside a remote is *not* a remote root:
        // it must not get the remote icon.
        assert!(matches!(
            rows[1],
            RefTreeRow::Folder {
                is_remote_root: false,
                ..
            }
        ));
        // The leaf still carries the name the rest of the app keys on.
        assert_eq!(rows[2].leaf_name(), Some("origin/feature/login"));
    }

    #[test]
    fn two_remotes_are_two_top_level_nodes() {
        let rows = ref_display_rows(
            &names(&["gh/main", "origin/feature/login", "origin/main"]),
            RefRowKind::Remote,
            true,
            &collapsed(&[]),
            &names(&["gh", "origin"]),
        );
        assert_eq!(
            shape(&rows),
            vec![
                (0, "gh".into(), true),
                (1, "main".into(), false),
                (0, "origin".into(), true),
                (1, "feature".into(), true),
                (2, "login".into(), false),
                (1, "main".into(), false),
            ]
        );
    }

    /// A remote may legally be named `grp/sub`. Splitting on the first
    /// `/` would invent a top-level folder `grp` holding a folder `sub`,
    /// and `sub` would look like a branch folder rather than part of the
    /// remote's name.
    #[test]
    fn a_remote_whose_name_contains_a_slash_is_one_node() {
        let rows = ref_display_rows(
            &names(&["grp/sub/feature/login"]),
            RefRowKind::Remote,
            true,
            &collapsed(&[]),
            &names(&["grp/sub"]),
        );
        assert_eq!(
            shape(&rows),
            vec![
                (0, "grp/sub".into(), true),
                (1, "feature".into(), true),
                (2, "login".into(), false),
            ]
        );
        assert!(matches!(
            rows[0],
            RefTreeRow::Folder {
                is_remote_root: true,
                ..
            }
        ));
    }

    /// A branch whose own name is a remote's name. `origin/origin` is
    /// the remote `origin`'s branch `origin`, not the remote `origin`
    /// twice - and with a remote also called `origin/origin` present,
    /// longest-match must claim it instead.
    #[test]
    fn a_branch_that_shadows_a_remote_name_stays_a_branch() {
        let rows = ref_display_rows(
            &names(&["origin/origin"]),
            RefRowKind::Remote,
            true,
            &collapsed(&[]),
            &names(&["origin"]),
        );
        assert_eq!(
            shape(&rows),
            vec![(0, "origin".into(), true), (1, "origin".into(), false)]
        );
        assert_eq!(rows[1].leaf_name(), Some("origin/origin"));

        let rows = ref_display_rows(
            &names(&["origin/origin/main"]),
            RefRowKind::Remote,
            true,
            &collapsed(&[]),
            &names(&["origin", "origin/origin"]),
        );
        assert_eq!(
            shape(&rows),
            vec![(0, "origin/origin".into(), true), (1, "main".into(), false)]
        );
    }

    /// With no remote list to resolve against - the first frame, before
    /// the remotes have come back from the daemon - the rows must still
    /// be sane rather than empty or unsplit.
    #[test]
    fn an_unknown_remote_still_becomes_a_top_level_node() {
        let rows = ref_display_rows(
            &names(&["origin/feature/login"]),
            RefRowKind::Remote,
            true,
            &collapsed(&[]),
            &[],
        );
        assert_eq!(
            shape(&rows),
            vec![
                (0, "origin".into(), true),
                (1, "feature".into(), true),
                (2, "login".into(), false),
            ]
        );
    }

    #[test]
    fn collapsing_a_folder_hides_exactly_its_subtree() {
        let all = names(&[
            "origin/feature/login",
            "origin/feature/logout",
            "origin/main",
        ]);
        let remotes = names(&["origin"]);
        let rows = ref_display_rows(
            &all,
            RefRowKind::Remote,
            true,
            &collapsed(&[(RefRowKind::Remote, "origin/feature")]),
            &remotes,
        );
        assert_eq!(
            shape(&rows),
            vec![
                (0, "origin".into(), true),
                (1, "feature".into(), true),
                (1, "main".into(), false),
            ]
        );

        // Collapsing the remote itself hides everything under it, and
        // the remote's own row stays.
        let rows = ref_display_rows(
            &all,
            RefRowKind::Remote,
            true,
            &collapsed(&[(RefRowKind::Remote, "origin")]),
            &remotes,
        );
        assert_eq!(shape(&rows), vec![(0, "origin".into(), true)]);
    }

    /// The collapse key carries the section kind, so a folder named
    /// `feature` collapsed in LOCAL must leave REMOTE's alone.
    #[test]
    fn collapse_state_does_not_leak_between_sections() {
        let rows = ref_display_rows(
            &names(&["origin/feature/login"]),
            RefRowKind::Remote,
            true,
            &collapsed(&[(RefRowKind::Local, "feature")]),
            &names(&["origin"]),
        );
        assert_eq!(rows.len(), 3);
    }

    #[test]
    fn tags_form_folders_like_branches() {
        let all = names(&[
            "backups/a",
            "plain",
            "releases/v1.0.0",
            "releases/v1.1.0",
            "releases-old",
        ]);
        let rows = ref_display_rows(&all, RefRowKind::Tag, true, &collapsed(&[]), &[]);
        assert_eq!(
            shape(&rows),
            vec![
                (0, "backups".into(), true),
                (1, "a".into(), false),
                (0, "plain".into(), false),
                (0, "releases".into(), true),
                (1, "v1.0.0".into(), false),
                (1, "v1.1.0".into(), false),
                (0, "releases-old".into(), false),
            ]
        );
        // The leaf keeps the full tag name; collapsing is per kind.
        assert_eq!(rows[1].leaf_name(), Some("backups/a"));
        let rows = ref_display_rows(
            &all,
            RefRowKind::Tag,
            true,
            &collapsed(&[
                (RefRowKind::Tag, "releases"),
                (RefRowKind::Local, "backups"),
            ]),
            &[],
        );
        assert_eq!(
            labels(&rows),
            vec!["backups", "a", "plain", "releases", "releases-old"]
        );
        // Flat when the tree is off.
        let rows = ref_display_rows(&all, RefRowKind::Tag, false, &collapsed(&[]), &[]);
        assert_eq!(rows.len(), 5);
    }

    /// Every row index the pane can select has to resolve to something,
    /// and only leaves may resolve to a ref. This is the invariant every
    /// click, drag and `Enter` in the sidebar depends on.
    #[test]
    fn only_leaves_carry_a_ref_name() {
        let rows = ref_display_rows(
            &names(&["gh/main", "origin/feature/login"]),
            RefRowKind::Remote,
            true,
            &collapsed(&[]),
            &names(&["gh", "origin"]),
        );
        for row in &rows {
            assert_eq!(row.leaf_name().is_some(), row.folder_path().is_none());
        }
        let leaves: Vec<&str> = rows.iter().filter_map(|r| r.leaf_name()).collect();
        assert_eq!(leaves, vec!["gh/main", "origin/feature/login"]);
    }
}

// -- which folders stay collapsed, per repository --------------------------
//
// Not a config key: a folder only exists in the repository whose branch
// names make it, so this lives beside the other per-repo state (`RepoState`,
// `<repo>/.gitpoulpi/repo-state.toml`), not in `config.toml`. Two frontends
// disagreeing about which folder is folded is not a bug (ADR-0006's test),
// so it is a window's state, not the daemon's.

use gitpoulpi_session::RepoState;
use std::path::{Path, PathBuf};

fn repo_state_path(repo_root: &Path) -> PathBuf {
    let state_dir = gitpoulpi_config::Settings::resolve_ambient()
        .map(|s| s.config().session.per_repo_state_dir.clone())
        .unwrap_or_else(|_| gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION.to_string());
    RepoState::path_for(repo_root, &state_dir)
}

/// The collapsed folders remembered for this repository. A missing or
/// unreadable file is an empty set - everything expanded, which is the
/// state a first run is in anyway.
pub fn load_collapsed(repo_root: &Path) -> CollapsedFolders {
    let state = RepoState::load(&repo_state_path(repo_root)).unwrap_or_default();
    state
        .collapsed_local_ref_folders
        .into_iter()
        .map(|p| (RefRowKind::Local, p))
        .chain(
            state
                .collapsed_remote_ref_folders
                .into_iter()
                .map(|p| (RefRowKind::Remote, p)),
        )
        .chain(
            state
                .collapsed_tag_ref_folders
                .into_iter()
                .map(|p| (RefRowKind::Tag, p)),
        )
        .collect()
}

/// Write the collapsed set back, leaving every other field of
/// `repo-state.toml` as it was. Load-modify-save, because the file also
/// carries the repo-scoped preferences the Preferences screen writes, and
/// clobbering those would be a silent loss.
pub fn save_collapsed(repo_root: &Path, collapsed: &CollapsedFolders) {
    let path = repo_state_path(repo_root);
    let mut state = RepoState::load(&path).unwrap_or_default();
    let mut local: Vec<String> = collapsed
        .iter()
        .filter(|(k, _)| *k == RefRowKind::Local)
        .map(|(_, p)| p.clone())
        .collect();
    let mut remote: Vec<String> = collapsed
        .iter()
        .filter(|(k, _)| *k == RefRowKind::Remote)
        .map(|(_, p)| p.clone())
        .collect();
    // A `HashSet`'s iteration order is not stable, and this file is
    // committed-adjacent (gitignored, but read by hand and diffed): an
    // unsorted list would churn on every save for no reason.
    let mut tag: Vec<String> = collapsed
        .iter()
        .filter(|(k, _)| *k == RefRowKind::Tag)
        .map(|(_, p)| p.clone())
        .collect();
    local.sort();
    remote.sort();
    tag.sort();
    state.collapsed_tag_ref_folders = tag;
    state.collapsed_local_ref_folders = local;
    state.collapsed_remote_ref_folders = remote;
    let _ = state.save(&path);
}

// -- hide-in-graph / solo, per repository -----------------------------
//
// Same reasoning as the collapsed set above: which refs are hidden or
// soloed is a display filter over *this* repository's branches, so it lives
// in `repo-state.toml` (`RepoState::ref_visibility`, `RefVisibility::
// {Shown,Hidden,Solo}`), not `config.toml`. A ref absent from the map is
// `Shown`.
//
// Hiding or soloing a whole folder or remote is **not** a third kind of
// entry: it expands to the concrete leaf refs the folder held at the moment
// of the click (see `refs_sidebar::refs_under_folder`). So a branch created
// later under a hidden or soloed folder **starts visible**, like any new
// branch; re-hiding the folder picks it up.

use gitpoulpi_core::domain::RefVisibility;

// -- hidden remotes -----------------------------------------------------
//
// A remote hidden as a whole (`RepoState::hidden_remotes`) lives in the
// window's hidden set as the key `<remote>/`: a trailing slash no ref name
// can have, so it never collides with a branch, and every site that already
// carries the set (clear, count, the daemon submit, the CLI reload) carries
// the remote with it. Reading "is this ref hidden" goes through
// `hidden_in_graph`, the one predicate; writing a remote goes through
// `toggle_remote_hidden`.

/// The hidden-set key of a whole remote.
pub(crate) fn remote_key(remote: &str) -> String {
    format!("{remote}/")
}

/// Whether `path` (a remote branch's `remote/name`, or a folder path under a
/// remote) lies under a remote hidden as a whole.
pub(crate) fn hidden_through_remote(hidden: &HashSet<String>, path: &str) -> bool {
    hidden.iter().any(|k| k.ends_with('/')) && {
        let with_slash = format!("{path}/");
        with_slash
            .match_indices('/')
            .any(|(i, _)| hidden.contains(&with_slash[..=i]))
    }
}

/// Whether `remote` itself is hidden as a whole.
pub(crate) fn remote_hidden(hidden: &HashSet<String>, remote: &str) -> bool {
    hidden.contains(&remote_key(remote))
}

/// The one "is this ref hidden from the graph" predicate: named in the set,
/// or a remote branch of a remote hidden as a whole (`remote_branch` says it
/// is one: a local branch `archive/x` is not covered by the remote `archive`).
pub(crate) fn hidden_in_graph(
    hidden: &HashSet<String>,
    full_name: &str,
    remote_branch: bool,
) -> bool {
    hidden.contains(full_name) || (remote_branch && hidden_through_remote(hidden, full_name))
}

/// The hidden set split for the daemon: individually hidden refs and whole
/// remotes (without the key's slash), both sorted.
pub(crate) fn split_hidden(hidden: &HashSet<String>) -> (Vec<String>, Vec<String>) {
    let mut names: Vec<String> = Vec::new();
    let mut remotes: Vec<String> = Vec::new();
    for k in hidden {
        match k.strip_suffix('/') {
            Some(remote) => remotes.push(remote.to_string()),
            None => names.push(k.clone()),
        }
    }
    names.sort();
    remotes.sort();
    (names, remotes)
}

/// Hides `remote` as a whole (`hide`), or shows it again. Hiding absorbs the
/// individually hidden `<remote>/...` entries (now covered); showing clears
/// the flag AND every such entry, so a remote whose branches were hidden one
/// by one comes back whole.
pub(crate) fn set_remote_hidden(hidden: &mut HashSet<String>, remote: &str, hide: bool) {
    let key = remote_key(remote);
    hidden.retain(|n| !n.starts_with(&key));
    if hide {
        hidden.insert(key);
    }
}

/// A remote was renamed: its hides (the flag and the individual
/// `<old>/...` branches) move to the new name. Whether anything moved.
pub(crate) fn rename_remote_in_hidden(hidden: &mut HashSet<String>, old: &str, new: &str) -> bool {
    let old_prefix = remote_key(old);
    let moved: Vec<String> = hidden
        .iter()
        .filter(|n| n.starts_with(&old_prefix))
        .cloned()
        .collect();
    for name in &moved {
        hidden.remove(name);
        hidden.insert(format!("{}{}", remote_key(new), &name[old_prefix.len()..]));
    }
    !moved.is_empty()
}

/// Forgets the hidden remotes that no longer exist (removed from the
/// repository), with the branches hidden under them. The removed names.
pub(crate) fn drop_gone_remotes(hidden: &mut HashSet<String>, existing: &[String]) -> Vec<String> {
    let gone: Vec<String> = split_hidden(hidden)
        .1
        .into_iter()
        .filter(|r| !existing.contains(r))
        .collect();
    for remote in &gone {
        set_remote_hidden(hidden, remote, false);
    }
    gone
}

/// Flips `remote`; returns whether it is hidden afterwards.
pub(crate) fn toggle_remote_hidden(hidden: &mut HashSet<String>, remote: &str) -> bool {
    let hide = !remote_hidden(hidden, remote);
    set_remote_hidden(hidden, remote, hide);
    hide
}

/// The remote (without the key's slash) a ref or folder path is hidden
/// through, the longest one.
pub(crate) fn hiding_remote_of(hidden: &HashSet<String>, path: &str) -> Option<String> {
    let with_slash = format!("{path}/");
    with_slash
        .match_indices('/')
        .map(|(i, _)| &with_slash[..=i])
        .rfind(|k| hidden.contains(*k))
        .map(|k| k.trim_end_matches('/').to_string())
}

/// "Hide in graph" on one ref of the Refs sidebar's `kind` section. A remote
/// branch of a remote hidden as a whole stays hidden with it: the answer is
/// the note for the status line, and nothing changed. Otherwise flips the
/// ref's own entry.
pub(crate) fn toggle_hide_ref(
    hidden: &mut HashSet<String>,
    kind: Option<RefRowKind>,
    full_name: &str,
) -> Option<String> {
    let remote_branch = kind == Some(RefRowKind::Remote);
    if remote_branch && !hidden.contains(full_name) {
        if let Some(remote) = hiding_remote_of(hidden, full_name) {
            return Some(gitpoulpi_i18n::msg::remotes::branch_hidden_with_remote(
                full_name, &remote,
            ));
        }
    }
    if !hidden.remove(full_name) {
        hidden.insert(full_name.to_string());
    }
    None
}

/// What "hide this folder" does on the REMOTE section's rows that are about a
/// whole remote: its root row flips the remote's flag (`Done` carries the
/// status line), a folder under a hidden remote is refused (`Done` with the
/// note, nothing changed). `Plain` is every other folder (the caller expands
/// to the refs under it as before).
pub(crate) enum FolderHide {
    Done(String),
    Plain,
}

pub(crate) fn hide_folder(
    hidden: &mut HashSet<String>,
    kind: Option<RefRowKind>,
    path: &str,
    is_remote_root: bool,
) -> FolderHide {
    if kind != Some(RefRowKind::Remote) {
        return FolderHide::Plain;
    }
    if is_remote_root {
        let now_hidden = toggle_remote_hidden(hidden, path);
        return FolderHide::Done(if now_hidden {
            gitpoulpi_i18n::msg::remotes::remote_hidden(path)
        } else {
            gitpoulpi_i18n::msg::remotes::remote_shown(path)
        });
    }
    match hiding_remote_of(hidden, path) {
        Some(remote) => FolderHide::Done(gitpoulpi_i18n::msg::remotes::branch_hidden_with_remote(
            path, &remote,
        )),
        None => FolderHide::Plain,
    }
}

/// The hidden and soloed ref sets remembered for this repository, in that
/// order (hidden remotes ride in the first, see above). A missing or
/// unreadable file is two empty sets - everything shown, nothing soloed, the
/// state a first run is in anyway.
pub fn load_ref_visibility(repo_root: &Path) -> (HashSet<String>, HashSet<String>) {
    let state = RepoState::load(&repo_state_path(repo_root)).unwrap_or_default();
    let mut hidden: HashSet<String> = state.hidden_remotes.iter().map(|r| remote_key(r)).collect();
    let mut solo = HashSet::new();
    for (name, vis) in state.ref_visibility {
        match vis {
            RefVisibility::Hidden => {
                hidden.insert(name);
            }
            RefVisibility::Solo => {
                solo.insert(name);
            }
            RefVisibility::Shown => {}
        }
    }
    (hidden, solo)
}

/// Send the hidden/soloed sets to the daemon, which writes them and tells
/// every other frontend (`Event::RefVisibilityChanged`). Names are sorted so
/// the request is deterministic.
pub fn submit_ref_visibility(
    link: &crate::daemon_link::DaemonLink,
    hidden: &HashSet<String>,
    solo: &HashSet<String>,
) {
    let (hidden, hidden_remotes) = split_hidden(hidden);
    let mut solo: Vec<String> = solo.iter().cloned().collect();
    solo.sort();
    link.submit_action(crate::daemon_link::Action::SetRefVisibility {
        hidden,
        solo,
        hidden_remotes,
    });
}

/// Write the hidden/soloed sets back, leaving every other field of
/// `repo-state.toml` as it was (load-modify-save, like `save_collapsed`). A
/// ref that is neither hidden nor soloed is absent from the map rather than
/// written as `Shown`: the file records only deviations from the default.
#[cfg(test)]
pub fn save_ref_visibility(repo_root: &Path, hidden: &HashSet<String>, solo: &HashSet<String>) {
    let path = repo_state_path(repo_root);
    let mut state = RepoState::load(&path).unwrap_or_default();
    let mut map: BTreeMap<String, RefVisibility> = BTreeMap::new();
    let (names, remotes) = split_hidden(hidden);
    state.hidden_remotes = remotes;
    for name in names {
        map.insert(name, RefVisibility::Hidden);
    }
    for name in solo {
        // A ref cannot be both hidden and soloed in this UI (Solo owns the
        // gutter icon exclusively while active -- see the render pass in
        // `event_loop.rs`), but if it ever happened, Solo is the more
        // surprising one to lose silently, so it wins the entry.
        map.insert(name.clone(), RefVisibility::Solo);
    }
    state.ref_visibility = map;
    let _ = state.save(&path);
}

#[cfg(test)]
mod ref_visibility_persistence_tests {
    use super::*;

    #[test]
    fn hidden_and_soloed_refs_round_trip_and_keep_their_state() {
        let dir = tempfile::tempdir().unwrap();
        let mut hidden = HashSet::new();
        hidden.insert("feature/x".to_string());
        hidden.insert("origin/main".to_string());
        let mut solo = HashSet::new();
        solo.insert("main".to_string());
        save_ref_visibility(dir.path(), &hidden, &solo);

        let (back_hidden, back_solo) = load_ref_visibility(dir.path());
        assert_eq!(back_hidden, hidden);
        assert_eq!(back_solo, solo);
    }

    #[test]
    fn a_hidden_remote_round_trips_as_a_key_and_covers_its_branches() {
        let dir = tempfile::tempdir().unwrap();
        let mut hidden = HashSet::new();
        assert!(toggle_remote_hidden(&mut hidden, "archive"));
        hidden.insert("side".to_string());
        save_ref_visibility(dir.path(), &hidden, &HashSet::new());
        let state = RepoState::load(&repo_state_path(dir.path())).unwrap();
        assert_eq!(state.hidden_remotes, ["archive"]);
        assert_eq!(
            state.ref_visibility.len(),
            1,
            "the remote is not a ref entry"
        );

        let (back, _) = load_ref_visibility(dir.path());
        assert_eq!(back, hidden);
        assert!(hidden_in_graph(&back, "archive/old-a", true));
        assert!(hidden_in_graph(&back, "archive/grp/deep", true));
        assert!(
            !hidden_in_graph(&back, "archive/old-a", false),
            "a local branch of that name is not under the remote"
        );
        assert!(!hidden_in_graph(&back, "archive2/x", true));
        assert!(hidden_in_graph(&back, "side", false));
        assert_eq!(
            split_hidden(&back),
            (vec!["side".to_string()], vec!["archive".to_string()])
        );
    }

    #[test]
    fn toggling_a_remote_absorbs_then_restores_its_individual_hides() {
        let mut hidden: HashSet<String> = ["archive/old-a", "archive/old-b", "origin/main"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert!(toggle_remote_hidden(&mut hidden, "archive"));
        assert!(!hidden.contains("archive/old-a"), "covered by the flag now");
        assert!(hidden.contains("origin/main"));
        // Showing clears the flag and every individual entry under it.
        hidden.insert("archive/old-c".to_string());
        assert!(!toggle_remote_hidden(&mut hidden, "archive"));
        assert_eq!(hidden.len(), 1);
        assert!(hidden.contains("origin/main"));
    }

    #[test]
    fn a_renamed_remote_takes_its_hides_along_and_a_removed_one_drops_them() {
        let mut hidden: HashSet<String> = ["old/", "old/x", "origin/main", "side"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert!(rename_remote_in_hidden(&mut hidden, "old", "new"));
        assert!(hidden.contains("new/") && hidden.contains("new/x"));
        assert!(!hidden.contains("old/") && !hidden.contains("old/x"));
        assert!(hidden.contains("origin/main") && hidden.contains("side"));
        assert!(!rename_remote_in_hidden(&mut hidden, "nothing", "else"));

        let gone = drop_gone_remotes(&mut hidden, &["origin".to_string()]);
        assert_eq!(gone, vec!["new"]);
        assert!(!hidden.contains("new/") && !hidden.contains("new/x"));
        assert!(
            hidden.contains("origin/main"),
            "branches hidden one by one stay"
        );
    }

    #[test]
    fn the_eye_on_a_branch_of_a_hidden_remote_says_to_show_the_remote_first() {
        let mut hidden = HashSet::new();
        set_remote_hidden(&mut hidden, "archive", true);
        let before = hidden.clone();
        let note = toggle_hide_ref(&mut hidden, Some(RefRowKind::Remote), "archive/old-a");
        assert!(note.is_some_and(|n| n.contains("archive/old-a") && n.contains("archive")));
        assert_eq!(hidden, before, "nothing changed");
        // Elsewhere it flips as ever.
        assert_eq!(
            toggle_hide_ref(&mut hidden, Some(RefRowKind::Remote), "origin/main"),
            None
        );
        assert!(hidden.contains("origin/main"));
        assert_eq!(
            toggle_hide_ref(&mut hidden, Some(RefRowKind::Local), "archive/x"),
            None
        );
        assert!(
            hidden.contains("archive/x"),
            "a local branch is not under the remote"
        );
    }

    #[test]
    fn a_remote_root_row_flips_the_remote_and_a_folder_under_it_is_refused() {
        let mut hidden = HashSet::new();
        let kind = Some(RefRowKind::Remote);
        assert!(matches!(
            hide_folder(&mut hidden, kind, "archive", true),
            FolderHide::Done(_)
        ));
        assert!(remote_hidden(&hidden, "archive"));
        let before = hidden.clone();
        assert!(matches!(
            hide_folder(&mut hidden, kind, "archive/grp", false),
            FolderHide::Done(_)
        ));
        assert_eq!(hidden, before);
        assert!(matches!(
            hide_folder(&mut hidden, kind, "archive", true),
            FolderHide::Done(_)
        ));
        assert!(hidden.is_empty());
        // A plain folder, or one of another section, is the old expansion.
        assert!(matches!(
            hide_folder(&mut hidden, kind, "origin/feature", false),
            FolderHide::Plain
        ));
        assert!(matches!(
            hide_folder(&mut hidden, Some(RefRowKind::Local), "archive", true),
            FolderHide::Plain
        ));
    }

    #[test]
    fn a_nested_remote_name_is_a_prefix_on_a_path_boundary() {
        let mut hidden = HashSet::new();
        toggle_remote_hidden(&mut hidden, "grp/sub");
        assert!(hidden_through_remote(&hidden, "grp/sub/main"));
        assert!(hidden_through_remote(&hidden, "grp/sub/feature/x"));
        assert!(!hidden_through_remote(&hidden, "grp/main"));
        assert!(!hidden_through_remote(&hidden, "grp/subway/x"));
        assert!(remote_hidden(&hidden, "grp/sub"));
    }

    #[test]
    fn a_repository_that_has_never_been_opened_has_nothing_hidden_or_soloed() {
        let dir = tempfile::tempdir().unwrap();
        let (hidden, solo) = load_ref_visibility(dir.path());
        assert!(hidden.is_empty());
        assert!(solo.is_empty());
    }

    /// The Preferences screen's own fields must survive a save from here,
    /// the same guarantee `save_collapsed` gives.
    #[test]
    fn saving_ref_visibility_leaves_the_rest_of_the_file_alone() {
        let dir = tempfile::tempdir().unwrap();
        let path = repo_state_path(dir.path());
        let mut state = RepoState::default();
        state.preferences.encoding = Some("utf-8".into());
        state.save(&path).unwrap();

        let mut hidden = HashSet::new();
        hidden.insert("feature/x".to_string());
        save_ref_visibility(dir.path(), &hidden, &HashSet::new());

        let back = RepoState::load(&path).unwrap();
        assert_eq!(back.preferences.encoding.as_deref(), Some("utf-8"));
        assert_eq!(
            back.ref_visibility.get("feature/x"),
            Some(&RefVisibility::Hidden)
        );
    }

    /// Un-hiding (removing from the set and saving again) must actually
    /// clear the entry, not leave a stale `Hidden` behind.
    #[test]
    fn showing_a_ref_again_removes_it_from_the_saved_map() {
        let dir = tempfile::tempdir().unwrap();
        let mut hidden = HashSet::new();
        hidden.insert("feature/x".to_string());
        save_ref_visibility(dir.path(), &hidden, &HashSet::new());
        assert!(load_ref_visibility(dir.path()).0.contains("feature/x"));

        hidden.remove("feature/x");
        save_ref_visibility(dir.path(), &hidden, &HashSet::new());
        assert!(load_ref_visibility(dir.path()).0.is_empty());
    }
}

#[cfg(test)]
mod persistence_tests {
    use super::*;

    #[test]
    fn a_collapsed_folder_survives_a_round_trip_and_keeps_its_section() {
        let dir = tempfile::tempdir().unwrap();
        let mut set = CollapsedFolders::new();
        set.insert((RefRowKind::Local, "feature".into()));
        set.insert((RefRowKind::Remote, "origin/feature".into()));
        save_collapsed(dir.path(), &set);

        let back = load_collapsed(dir.path());
        assert_eq!(back, set);
        assert!(back.contains(&(RefRowKind::Local, "feature".to_string())));
        assert!(!back.contains(&(RefRowKind::Remote, "feature".to_string())));
    }

    /// The sidebar must not eat the Preferences screen's own fields.
    #[test]
    fn saving_the_collapsed_set_leaves_the_rest_of_the_file_alone() {
        let dir = tempfile::tempdir().unwrap();
        let path = repo_state_path(dir.path());
        let mut state = RepoState::default();
        state.preferences.encoding = Some("utf-8".into());
        state.save(&path).unwrap();

        let mut set = CollapsedFolders::new();
        set.insert((RefRowKind::Local, "feature".into()));
        save_collapsed(dir.path(), &set);

        let back = RepoState::load(&path).unwrap();
        assert_eq!(back.preferences.encoding.as_deref(), Some("utf-8"));
        assert_eq!(
            back.collapsed_local_ref_folders,
            vec!["feature".to_string()]
        );
    }

    #[test]
    fn a_repository_that_has_never_been_opened_has_nothing_collapsed() {
        let dir = tempfile::tempdir().unwrap();
        assert!(load_collapsed(dir.path()).is_empty());
    }
}

// -- what double-clicking a remote branch should do ------------------------

use gitpoulpi_core::domain::{Ref, RefKind};

/// The three ways "check this remote branch out" can go, decided from the
/// refs the daemon already reported rather than a second round trip (two
/// reads could disagree about a branch created in between).
///
/// A pure function of `(the remote branch, every ref, the remotes)`, so each
/// case is a test rather than needing a real repository in a given state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteCheckout {
    /// A local branch already tracks it: check *that* branch out. Its
    /// name may be nothing like the remote branch's - tracking is a
    /// config pair, not a naming convention - so the name is carried
    /// rather than derived.
    CheckoutLocal { local: String },
    /// Nothing tracks it and the name is free: offer to create a local
    /// branch tracking it. The caller confirms before submitting; this
    /// only says what would be created.
    OfferCreate { local: String, upstream: String },
    /// The obvious local name is taken by a branch that tracks something
    /// else (or nothing). Neither checking that branch out nor silently
    /// picking another name is what the user asked for, so this says so
    /// and does nothing.
    NameTaken {
        local: String,
        tracks: Option<String>,
    },
    /// The row is already checked out. Nothing to do, and saying so
    /// beats a checkout that appears to do nothing.
    AlreadyOnIt { local: String },
}

/// Decide what a double-click (or `Enter`) on the remote branch
/// `upstream` should do.
///
/// `upstream` is spelled as the sidebar spells it, `origin/feature/login`.
/// The default local name is the part after the **remote**, resolved
/// against `remotes` - never "everything after the first `/`", which
/// would propose `sub/feature/login` for a remote named `grp/sub`.
pub fn remote_checkout_plan(upstream: &str, refs: &[Ref], remotes: &[String]) -> RemoteCheckout {
    // Any local branch whose configured upstream *is* this ref wins,
    // whatever it is called. This is the case the user means by "the
    // mapped local branch".
    if let Some(local) = refs.iter().find_map(|r| match (&r.kind, &r.upstream) {
        (RefKind::LocalBranch { name }, Some(up)) if up == upstream => Some((name, r.is_head)),
        _ => None,
    }) {
        return if local.1 {
            RemoteCheckout::AlreadyOnIt {
                local: local.0.clone(),
            }
        } else {
            RemoteCheckout::CheckoutLocal {
                local: local.0.clone(),
            }
        };
    }

    let local_name = match split_remote_ref(upstream, remotes) {
        Some((_, branch)) => branch.to_string(),
        // Not a `<remote>/<branch>` shape at all. Nothing sensible to
        // create; report the name as taken by nothing so the caller says
        // something rather than acting.
        None => {
            return RemoteCheckout::NameTaken {
                local: upstream.to_string(),
                tracks: None,
            }
        }
    };

    match refs
        .iter()
        .find(|r| matches!(&r.kind, RefKind::LocalBranch { name } if *name == local_name))
    {
        Some(existing) => RemoteCheckout::NameTaken {
            local: local_name,
            tracks: existing.upstream.clone(),
        },
        None => RemoteCheckout::OfferCreate {
            local: local_name,
            upstream: upstream.to_string(),
        },
    }
}

#[cfg(test)]
mod checkout_tests {
    use super::*;
    use gitpoulpi_core::domain::{Oid, RefVisibility};

    fn oid(n: u8) -> Oid {
        Oid([n; 20])
    }

    fn local(name: &str, upstream: Option<&str>, is_head: bool) -> Ref {
        Ref {
            kind: RefKind::LocalBranch { name: name.into() },
            target: oid(1),
            visibility: RefVisibility::Shown,
            is_head,
            upstream: upstream.map(str::to_string),
            tag_sync: None,
        }
    }

    fn remote(remote: &str, name: &str) -> Ref {
        Ref {
            kind: RefKind::RemoteBranch {
                remote: remote.into(),
                name: name.into(),
            },
            target: oid(1),
            visibility: RefVisibility::Shown,
            is_head: false,
            upstream: None,
            tag_sync: None,
        }
    }

    fn names(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn a_local_branch_that_tracks_it_is_what_gets_checked_out() {
        let refs = vec![
            local("feature/login", Some("origin/feature/login"), false),
            remote("origin", "feature/login"),
        ];
        assert_eq!(
            remote_checkout_plan("origin/feature/login", &refs, &names(&["origin"])),
            RemoteCheckout::CheckoutLocal {
                local: "feature/login".into()
            }
        );
    }

    /// Tracking is configuration, not a naming rule: a branch called
    /// `work` may perfectly well track `origin/feature/login`, and it is
    /// still the branch to check out.
    #[test]
    fn the_tracking_branch_wins_even_under_a_different_name() {
        let refs = vec![
            local("work", Some("origin/feature/login"), false),
            remote("origin", "feature/login"),
        ];
        assert_eq!(
            remote_checkout_plan("origin/feature/login", &refs, &names(&["origin"])),
            RemoteCheckout::CheckoutLocal {
                local: "work".into()
            }
        );
    }

    #[test]
    fn nothing_tracks_it_and_the_name_is_free_so_creating_is_offered() {
        let refs = vec![
            local("main", Some("origin/main"), true),
            remote("origin", "feature/login"),
        ];
        assert_eq!(
            remote_checkout_plan("origin/feature/login", &refs, &names(&["origin"])),
            RemoteCheckout::OfferCreate {
                local: "feature/login".into(),
                upstream: "origin/feature/login".into(),
            }
        );
    }

    /// The proposed name comes from the part after the *remote*, and a
    /// remote may be named `grp/sub`. Splitting on the first `/` would
    /// propose `sub/feature/login`.
    #[test]
    fn the_proposed_name_strips_the_whole_remote_name() {
        let refs = vec![remote("grp/sub", "feature/login")];
        assert_eq!(
            remote_checkout_plan("grp/sub/feature/login", &refs, &names(&["grp/sub"])),
            RemoteCheckout::OfferCreate {
                local: "feature/login".into(),
                upstream: "grp/sub/feature/login".into(),
            }
        );
    }

    #[test]
    fn a_name_taken_by_a_branch_tracking_something_else_is_reported_not_guessed() {
        let refs = vec![
            local("feature/login", Some("gh/feature/login"), false),
            remote("origin", "feature/login"),
        ];
        assert_eq!(
            remote_checkout_plan("origin/feature/login", &refs, &names(&["gh", "origin"])),
            RemoteCheckout::NameTaken {
                local: "feature/login".into(),
                tracks: Some("gh/feature/login".into()),
            }
        );
    }

    #[test]
    fn a_name_taken_by_a_branch_tracking_nothing_is_reported_too() {
        let refs = vec![
            local("feature/login", None, false),
            remote("origin", "feature/login"),
        ];
        assert_eq!(
            remote_checkout_plan("origin/feature/login", &refs, &names(&["origin"])),
            RemoteCheckout::NameTaken {
                local: "feature/login".into(),
                tracks: None,
            }
        );
    }

    #[test]
    fn the_branch_already_checked_out_says_so_rather_than_checking_out_again() {
        let refs = vec![
            local("main", Some("origin/main"), true),
            remote("origin", "main"),
        ];
        assert_eq!(
            remote_checkout_plan("origin/main", &refs, &names(&["origin"])),
            RemoteCheckout::AlreadyOnIt {
                local: "main".into()
            }
        );
    }
}
