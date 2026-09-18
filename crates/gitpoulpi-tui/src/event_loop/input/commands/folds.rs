//! Expand / collapse every folder of a tree: the refs sections and the
//! right pane's file lists. The menus, the keys and the palette all land
//! here (`crate::tree_fold` has the pure half), so none can drift.
//!
//! While a filter is active the collapsed set these edit is the filter's own
//! overlay (see `tree_fold::FilterOverlay`): the operation acts on the visible
//! tree and is forgotten when the filter changes or clears.

use super::{focused_list, list_cursor, list_rows, CommandCtx};
use crate::event_loop::*;
use crate::tree_fold::{self, FoldCtx, TreeFold};
use gitpoulpi_i18n::msg::tree_fold as msg;

pub(super) fn run(cx: &mut CommandCtx<'_>, cmd: PaletteCommand) -> bool {
    match cmd {
        PaletteCommand::RefsFold(op) => fold_refs_cursor(cx, op),
        PaletteCommand::FilesFold(op) => fold_files_cursor(cx, op),
        _ => return false,
    }
    true
}

/// The refs section under the keyboard: expands/collapses all of its folders,
/// or (a "below" operation) the cursor's folder row and everything under it.
fn fold_refs_cursor(cx: &mut CommandCtx<'_>, op: TreeFold) {
    let section = *cx.active_ref_section;
    if !op.below() {
        fold_refs(cx, section, op, None);
        return;
    }
    let selected = selected_ref_folder(
        &*cx.refs_result,
        &*cx.refs_filter,
        section,
        cx.sidebar_states.state(section).selected(),
        *cx.hierarchical_refs,
        &*cx.collapsed_ref_folders,
        &*cx.remote_names,
    );
    match selected {
        Some((path, _)) => fold_refs(cx, section, op, Some(&path)),
        None => *cx.last_message = Some(msg::MSG_SELECT_FOLDER.get().into()),
    }
}

/// Folds a refs section: every folder of it, or `root` and what is under it.
/// Only the folders on screen count (a name filter hides the others).
pub(in crate::event_loop) fn fold_refs(
    cx: &mut CommandCtx<'_>,
    section: SidebarSection,
    op: TreeFold,
    root: Option<&str>,
) {
    let Some(kind) = section.ref_kind() else {
        *cx.last_message = Some(msg::MSG_NO_FOLDERS.get().into());
        return;
    };
    let names = cx
        .refs_result
        .as_ref()
        .map(|refs| group_refs(refs, &*cx.refs_filter).names(kind).to_vec())
        .unwrap_or_default();
    let open = CollapsedFolders::new();
    let rows = ref_display_rows(
        &names,
        kind,
        *cx.hierarchical_refs,
        &open,
        &*cx.remote_names,
    );
    let folders = tree_fold::ref_folders(&rows, root.filter(|_| op.below()));
    if folders.is_empty() {
        *cx.last_message = Some(msg::MSG_NO_FOLDERS.get().into());
        return;
    }
    tree_fold::apply(
        op,
        &mut *cx.collapsed_ref_folders,
        folders.into_iter().map(|p| (kind, p)),
    );
    // The cursor must not point past the rows that are left.
    let left = ref_display_rows(
        &names,
        kind,
        *cx.hierarchical_refs,
        &*cx.collapsed_ref_folders,
        &*cx.remote_names,
    )
    .len();
    let state = cx.sidebar_states.state_mut(section);
    if let Some(selected) = state.selected() {
        if selected >= left {
            state.select(left.checked_sub(1));
        }
    }
}

/// The file list under the keyboard (the focused one).
fn fold_files_cursor(cx: &mut CommandCtx<'_>, op: TreeFold) {
    let Some(list) = focused_list(cx) else {
        *cx.last_message = Some(msg::MSG_FOCUS_FILE_LIST.get().into());
        return;
    };
    if !op.below() {
        fold_files(cx, list, op, None);
        return;
    }
    let rows = list_rows(cx, list);
    let folder = list_cursor(cx, list)
        .and_then(|i| rows.rows.get(i))
        .filter(|r| r.folder)
        .map(|r| r.path.clone());
    match folder {
        Some(path) => fold_files(cx, list, op, Some(&path)),
        None => *cx.last_message = Some(msg::MSG_SELECT_FOLDER.get().into()),
    }
}

/// Folds a file list (Unstaged, Staged, the commit's files, View all files):
/// every folder of it, or `root` and its subfolders.
pub(in crate::event_loop) fn fold_files(
    cx: &mut CommandCtx<'_>,
    list: FileList,
    op: TreeFold,
    root: Option<&Path>,
) {
    // Every file of the list, folded away or not (the rows only hold the
    // visible ones, and a folder inside a folded one must still be reached).
    let files = match list {
        FileList::Unstaged | FileList::Staged => {
            let section = if list == FileList::Unstaged {
                StatusSection::Unstaged
            } else {
                StatusSection::Staged
            };
            cx.status_result
                .as_ref()
                .map(|tree| crate::status_pane::section_paths(tree, section))
                .unwrap_or_default()
        }
        FileList::Commit | FileList::WipAll => list_rows(cx, list).files,
    };
    let folders = match root.filter(|_| op.below()) {
        Some(root) => tree_fold::file_folders_below(&files, root),
        None => tree_fold::file_folders(&files),
    };
    if folders.is_empty() {
        *cx.last_message = Some(msg::MSG_NO_FOLDERS.get().into());
        return;
    }
    match list {
        // Each sub-pane folds on its own, under its own key prefix.
        FileList::Unstaged | FileList::Staged => {
            let section = if list == FileList::Unstaged {
                StatusSection::Unstaged
            } else {
                StatusSection::Staged
            };
            tree_fold::apply(
                op,
                &mut *cx.collapsed_status_folders,
                folders
                    .iter()
                    .map(|f| crate::status_list::fold_key(section, f)),
            );
        }
        // View all files keeps the unprefixed keys of the same set.
        FileList::WipAll => tree_fold::apply(op, &mut *cx.collapsed_status_folders, folders),
        FileList::Commit => tree_fold::apply(op, &mut *cx.collapsed_commit_folders, folders),
    }
}

/// A fold key (`+`, `-`, `*`, `_`...) in the pane that has the keyboard.
/// `true` when it was one.
pub(in crate::event_loop) fn fold_key(
    cx: &mut CommandCtx<'_>,
    km: &crate::keymap::Keymap,
    key: &KeyEvent,
) -> bool {
    let ctx = match *cx.focus {
        Focus::Refs => FoldCtx::Refs,
        _ => match focused_list(cx) {
            Some(list) => crate::pane_menu::fold_ctx(list),
            None => return false,
        },
    };
    let Some(op) = TreeFold::from_key(km, ctx, key) else {
        return false;
    };
    if ctx == FoldCtx::Refs {
        fold_refs_cursor(cx, op);
    } else {
        fold_files_cursor(cx, op);
    }
    true
}

/// Whether a ref-menu row is one of the fold family (it needs the window's
/// state, not `RefActionCtx`).
pub(in crate::event_loop) fn is_ref_fold_action(action: RefContextAction) -> bool {
    matches!(
        action,
        RefContextAction::Fold(_) | RefContextAction::OpenSectionsChecklist
    )
}

/// Runs a fold row of a refs menu and closes the menu: on a folder row it
/// acts on that folder (`key` is its path), on a section's own menu
/// (`ref_menu::SECTION_MENU_KEY`) on the whole section.
pub(in crate::event_loop) fn apply_ref_fold_action(
    cx: &mut CommandCtx<'_>,
    action: RefContextAction,
) {
    let Some((section, key, taken)) = cx.open_menu.take_ref_menu() else {
        return;
    };
    match action {
        RefContextAction::Fold(op) => {
            let root = (key != crate::ref_menu::SECTION_MENU_KEY).then_some(key.as_str());
            fold_refs(cx, section, op, root);
        }
        RefContextAction::OpenSectionsChecklist => {
            super::open_sidebar_sections_menu(cx, taken.anchor_col, taken.anchor_row);
        }
        _ => {}
    }
}
