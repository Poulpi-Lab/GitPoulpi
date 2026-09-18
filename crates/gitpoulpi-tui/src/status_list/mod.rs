//! The Status pane's file list (STAGED/UNSTAGED/CONFLICTED sections):
//! its display lines and list states, selection and scrolling, row
//! rendering, the per-section bulk buttons, and applying a file's context
//! menu action.

use super::*;

mod actions;
mod header;
mod rows;

pub(crate) use actions::*;
pub(crate) use header::*;
pub(crate) use rows::*;

/// One rendered line of the working-changes pane, flattened across both
/// sections so the pane stays a single `List` with one `ListState`.
///
/// Section headers are rows rather than block titles because the
/// two sections scroll together in one list - making them separate
/// bordered blocks (the way the Refs sidebar's sub-panes are) would mean
/// two independent scroll positions for what reads as one document.
#[derive(Clone)]
pub(crate) enum StatusPaneLine {
    Folder {
        path: PathBuf,
        /// This row's entry in `collapsed_status_folders` ([`fold_key`]).
        fold_key: PathBuf,
        depth: usize,
        label: String,
        collapsed: bool,
    },
    File {
        section: StatusSection,
        path: PathBuf,
        depth: usize,
        label: String,
        status: Option<gitpoulpi_core::domain::FileStatus>,
    },
}

impl StatusPaneLine {
    /// The file a row acts on, if it is one. Section headers and folder
    /// rows return `None`, so callers mapping a selection to a stage/
    /// discard target skip them.
    pub(crate) fn file(&self) -> Option<(StatusSection, &PathBuf)> {
        match self {
            StatusPaneLine::File { section, path, .. } => Some((*section, path)),
            _ => None,
        }
    }
}

/// Moves the working-changes selection by `delta`, crossing from one
/// sub-pane into the other at the edges.
///
/// Stepping off the bottom of `Unstaged Files` lands on the first row of
/// `Staged Files` rather than sticking, and vice versa - the same
/// edge-crossing the Refs sidebar's sections got, so the two panes behave
/// alike. Collapsed and empty sub-panes are skipped, since there's nothing
/// in them to land on.
pub(crate) fn move_status_selection(
    states: &mut StatusStates,
    lines: &[(StatusSection, Vec<StatusPaneLine>)],
    collapsed: &HashSet<StatusSection>,
    active: &mut StatusSection,
    delta: i32,
) {
    let usable = |s: StatusSection| !collapsed.contains(&s) && !section_lines(lines, s).is_empty();
    // If the active sub-pane has nothing to select, move to one that has.
    if !usable(*active) {
        if let Some(first) = StatusSection::ORDER.into_iter().find(|s| usable(*s)) {
            *active = first;
        } else {
            return;
        }
    }
    let len = section_lines(lines, *active).len();
    let cur = states.state(*active).selected();
    let next = match (cur, delta) {
        (Some(i), d) if d < 0 && i == 0 => None,
        (Some(i), d) if d > 0 && i + 1 >= len => None,
        _ => move_index(cur, len, delta),
    };
    match next {
        Some(i) => states.select(*active, Some(i)),
        None => {
            // At an edge: hand off to the neighboring sub-pane if there
            // is a usable one in that direction, otherwise stay put.
            let order = StatusSection::ORDER;
            let pos = order.iter().position(|s| s == active).unwrap_or(0);
            let neighbor = if delta > 0 {
                order.iter().skip(pos + 1).copied().find(|s| usable(*s))
            } else {
                order[..pos].iter().rev().copied().find(|s| usable(*s))
            };
            match neighbor {
                Some(s) => {
                    *active = s;
                    let n = section_lines(lines, s).len();
                    let landing = if delta > 0 { 0 } else { n.saturating_sub(1) };
                    states.select(s, Some(landing));
                }
                None => {
                    // Clamp into range rather than leaving it unselected.
                    states.select(*active, move_index(cur, len, 0));
                }
            }
        }
    }
}

/// Scrolls whichever working-changes sub-pane the cursor is over --
/// The wheel moves that sub-pane's viewport only, never
/// its selection. `visible` is that sub-pane's content-row capacity
/// (its bordered area's height minus 2).
///
/// Resolved against the same `(section, area)` list the click and hover
/// paths use, so the wheel can't end up scrolling a different sub-pane
/// from the one under the pointer.
pub(crate) fn scroll_status_pane(
    areas: &[(StatusSection, Rect)],
    states: &mut StatusStates,
    lines: &[(StatusSection, Vec<StatusPaneLine>)],
    col: u16,
    row: u16,
    delta: i32,
) {
    if let Some((section, area)) = areas
        .iter()
        .copied()
        .find(|(_, a)| a.contains((col, row).into()))
    {
        let len = section_lines(lines, section).len();
        let visible = area.height.saturating_sub(2) as usize;
        let state = states.state_mut(section);
        let new_offset = scroll_offset(state.offset(), len, visible, delta);
        *state.offset_mut() = new_offset;
        *states.follow_mut(section) = false;
    }
}

/// One sub-pane's rows out of the per-section list rebuilt each frame.
pub(crate) fn section_lines(
    all: &[(StatusSection, Vec<StatusPaneLine>)],
    section: StatusSection,
) -> &[StatusPaneLine] {
    all.iter()
        .find(|(s, _)| *s == section)
        .map(|(_, l)| l.as_slice())
        .unwrap_or(&[])
}

/// What "Collapse All" puts in `collapsed_status_folders`: every folder of
/// each sub-pane, under that sub-pane's [`fold_key`].
pub(crate) fn all_status_fold_keys(
    all: &[(StatusSection, Vec<StatusPaneLine>)],
) -> HashSet<PathBuf> {
    all.iter()
        .flat_map(|(section, lines)| {
            let paths: Vec<PathBuf> = lines
                .iter()
                .filter_map(|l| l.file().map(|(_, p)| p.clone()))
                .collect();
            crate::status_pane::all_folders(&paths)
                .into_iter()
                .map(move |folder| fold_key(*section, &folder))
        })
        .collect()
}

/// A folder's entry in `collapsed_status_folders` for one sub-pane. Each
/// sub-pane folds on its own - `src/` open in Unstaged and closed in
/// Staged - so the key carries the sub-pane as a first component no file
/// can have (`<` and `>` are not allowed in a Windows file name). Unprefixed
/// keys are the View all files list's.
pub(crate) fn fold_key(section: StatusSection, folder: &Path) -> PathBuf {
    Path::new(fold_scope(section)).join(folder)
}

fn fold_scope(section: StatusSection) -> &'static str {
    match section {
        StatusSection::Unstaged => "<unstaged>",
        StatusSection::Staged => "<staged>",
    }
}

/// Per-section `ListState`s for the working-changes pane.
///
/// `Unstaged Files` and `Staged Files` are two genuinely separate
/// sub-panes - own border, own chevron/count title, own scroll position -
/// exactly like the Refs sidebar's sections, rather than two headers
/// inside one list. Mirrors `SidebarStates`.
#[derive(Default)]
pub(crate) struct StatusStates {
    pub(crate) unstaged: ListState,
    pub(crate) staged: ListState,
    follow: FollowFlags<2>,
}

impl StatusStates {
    pub(crate) fn state_mut(&mut self, section: StatusSection) -> &mut ListState {
        match section {
            StatusSection::Unstaged => &mut self.unstaged,
            StatusSection::Staged => &mut self.staged,
        }
    }

    pub(crate) fn state(&self, section: StatusSection) -> &ListState {
        match section {
            StatusSection::Unstaged => &self.unstaged,
            StatusSection::Staged => &self.staged,
        }
    }

    pub(crate) fn follow(&self, section: StatusSection) -> bool {
        self.follow.get(section as usize)
    }

    pub(crate) fn follow_mut(&mut self, section: StatusSection) -> &mut bool {
        self.follow.get_mut(section as usize)
    }

    /// Selects like `state_mut(section).select(..)`, but also marks the
    /// section as following the selection again - every keyboard move
    /// and click goes through this rather than reaching for `state_mut`
    /// directly, so a wheel-scrolled sub-pane snaps back the next time
    /// something other than the wheel touches it.
    pub(crate) fn select(&mut self, section: StatusSection, index: Option<usize>) {
        self.state_mut(section).select(index);
        *self.follow_mut(section) = true;
    }
}

/// Builds one section's rows: `build_tree_rows` output mapped onto the
/// pane's own row type, with each file's status resolved for that half.
///
/// No section-header row - the header is the sub-pane's block title now,
/// which is what makes the two halves scroll independently.
pub(crate) fn build_status_section_lines(
    tree: &WorkingTree,
    section: StatusSection,
    mode: FileListMode,
    collapsed_folders: &HashSet<PathBuf>,
    filter: &str,
) -> Vec<StatusPaneLine> {
    let paths = section_paths(tree, section);
    let scope = fold_scope(section);
    let collapsed_folders: HashSet<PathBuf> = collapsed_folders
        .iter()
        .filter_map(|key| key.strip_prefix(scope).ok().map(Path::to_path_buf))
        .collect();
    // Path -> first matching entry, built once: a `find` per row was
    // O(rows x files) -- 4M path compares a frame on a 2,000-file tree.
    // `entry().or_insert` keeps the first entry per path, as `find` did.
    let mut by_path: std::collections::HashMap<
        &std::path::Path,
        &gitpoulpi_core::domain::StatusEntry,
    > = std::collections::HashMap::with_capacity(tree.files.len());
    for f in &tree.files {
        by_path.entry(f.path.as_path()).or_insert(f);
    }
    build_tree_rows(&paths, mode, &collapsed_folders, filter)
        .into_iter()
        .map(|row| match row {
            TreeRow::Folder {
                depth,
                path,
                label,
                collapsed,
            } => StatusPaneLine::Folder {
                fold_key: fold_key(section, &path),
                path,
                depth,
                label,
                collapsed,
            },
            TreeRow::File {
                depth,
                label,
                index,
            } => {
                let path = paths[index].clone();
                let status = by_path
                    .get(path.as_path())
                    .and_then(|e| entry_status(e, section));
                StatusPaneLine::File {
                    section,
                    path,
                    depth,
                    label,
                    status,
                }
            }
        })
        .collect()
}

/// The third pane's own title in working-changes mode.
///
/// A conflicted repository is a *mode* the user is in, not a property of
/// one row, so it is announced by the pane rather than left to be
/// inferred from red `!` markers - which is how you end up committing a
/// file full of `<<<<<<<`. This is the pane's existing title string, so
/// it costs no rows and cannot shift the list arithmetic underneath it.
pub(crate) fn working_changes_title(
    change_count: usize,
    branch: &str,
    conflict_count: usize,
) -> String {
    let n = change_count as u64;
    if conflict_count == 0 {
        gitpoulpi_i18n::msg::status_list::title_changes(n, branch)
    } else {
        gitpoulpi_i18n::msg::status_list::title_changes_conflicted(n, branch, conflict_count)
    }
}

/// Resolves the currently-selected status row to a file path, but only if
/// it's actually a file row in the section the caller expects ('s' only
/// acts on an unstaged row, 'u' only on a staged one) - landing on a
/// section header or the wrong section is a no-op rather than an error.
pub(crate) fn selected_status_path(
    lines: &[StatusPaneLine],
    selected: Option<usize>,
    want_staged_section: bool,
) -> Option<PathBuf> {
    let want = if want_staged_section {
        StatusSection::Staged
    } else {
        StatusSection::Unstaged
    };
    match lines.get(selected?)?.file() {
        Some((section, path)) if section == want => Some(path.clone()),
        _ => None,
    }
}

/// The file selected in one sub-pane of the third pane, resolved against
/// the row model that sub-pane actually draws.
///
/// The palette's stage/unstage commands and the `Ctrl+G s`/`u` chord both
/// need this. Rebuilding the lines here rather than reading a cached index
/// is the whole point: the two halves have independent selections, and in
/// Tree mode folder rows shift every file row below them, so resolving a
/// selection against any other model silently acts on the wrong file.
pub(crate) fn selected_status_path_in(
    status_result: &gitpoulpi_core::Result<WorkingTree>,
    states: &StatusStates,
    section: StatusSection,
    mode: FileListMode,
    collapsed_folders: &HashSet<PathBuf>,
) -> Option<PathBuf> {
    let tree = status_result.as_ref().ok()?;
    let lines = build_status_section_lines(tree, section, mode, collapsed_folders, "");
    let selected = states.state(section).selected();
    selected_status_path(&lines, selected, section == StatusSection::Staged)
}

/// The unmerged path the third pane's selection is sitting on, if any.
///
/// Only the Unstaged half is consulted, because `section_paths` pins a
/// conflicted file there and nowhere else. Asking the Staged half too
/// would not find the same file twice - it would find a *different*,
/// already-resolved one and offer to take a side of it, which the backend
/// would then fail on for having no stages 1/2/3.
pub(crate) fn selected_conflicted_path(
    status_result: &gitpoulpi_core::Result<WorkingTree>,
    states: &StatusStates,
    mode: FileListMode,
    collapsed_folders: &HashSet<PathBuf>,
    conflicts: &[gitpoulpi_core::domain::ConflictEntry],
) -> Option<PathBuf> {
    let path = selected_status_path_in(
        status_result,
        states,
        StatusSection::Unstaged,
        mode,
        collapsed_folders,
    )?;
    conflicts.iter().any(|c| c.path == path).then_some(path)
}

#[cfg(test)]
mod tests;
