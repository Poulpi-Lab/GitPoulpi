//! The worktree commands of the palette (acting on the WORKTREES row the
//! Refs pane has selected) and the header `+` menu both the mouse and the
//! keyboard open.

use super::CommandCtx;
use crate::event_loop::*;
use gitpoulpi_i18n::msg::worktree_manage as msg;

/// The selected row of the WORKTREES section, when it is a real worktree.
fn selected(cx: &CommandCtx<'_>) -> Option<(usize, gitpoulpi_core::domain::Worktree)> {
    let list = cx.worktrees_result.as_deref().unwrap_or(&[]);
    if crate::refs_sidebar::linked_worktree_count(list) == 0 {
        return None;
    }
    let index = cx
        .sidebar_states
        .state(SidebarSection::Worktrees)
        .selected()?;
    list.get(index).cloned().map(|w| (index, w))
}

/// Where a menu of the section opens: under the header, or the Refs pane's
/// corner while the section is not on screen yet.
fn anchor(cx: &CommandCtx<'_>) -> (u16, u16) {
    cx.sidebar_section_areas
        .iter()
        .find(|(s, _)| *s == SidebarSection::Worktrees)
        .map(|(_, a)| (a.x + 2, a.y + 1))
        .unwrap_or((cx.refs_area.x + 2, cx.refs_area.y + 1))
}

/// The row's position, for a menu hanging from the selected row.
fn row_anchor(cx: &CommandCtx<'_>, index: usize) -> (u16, u16) {
    let state = cx.sidebar_states.state(SidebarSection::Worktrees);
    match cx
        .sidebar_section_areas
        .iter()
        .find(|(s, _)| *s == SidebarSection::Worktrees)
    {
        Some((_, a)) => (
            a.x + 2,
            a.y + 1 + (index - state.offset().min(index)) as u16,
        ),
        None => anchor(cx),
    }
}

/// Opens the `+` menu at `(col, row)`: a worktree from the current branch,
/// from the Log's selected commit, or on a branch chosen in the dialog.
pub(in crate::event_loop) fn open_plus_menu(cx: &mut CommandCtx<'_>, col: u16, row: u16) {
    let commit =
        crate::log_table::selected_log_commit(&*cx.log_result, &*cx.log_state, &*cx.log_order);
    *cx.focus = Focus::Refs;
    *cx.active_ref_section = SidebarSection::Worktrees;
    *cx.open_menu = OpenMenu::Ref((
        SidebarSection::Worktrees,
        PLUS_KEY.to_string(),
        ContextMenu::new(
            msg::PLUS_TITLE.get(),
            crate::ref_menu::worktree_plus_entries(commit.is_some()),
            col,
            row,
        ),
    ));
}

/// The row identity of the `+` menu (a path never starts with it).
pub(in crate::event_loop) const PLUS_KEY: &str = "\u{1}worktree-plus";

/// Opens the context menu of the selected worktree row (the keyboard twin
/// of the right-click).
pub(in crate::event_loop) fn open_row_menu(cx: &mut CommandCtx<'_>) {
    match selected(cx) {
        Some((index, wt)) => {
            let (col, row) = row_anchor(cx, index);
            *cx.open_menu = OpenMenu::Ref((
                SidebarSection::Worktrees,
                wt.path.display().to_string(),
                ContextMenu::new(
                    crate::worktree_remove_dialog::dir_name(&wt.path),
                    crate::ref_menu::worktree_menu_entries(&wt),
                    col,
                    row,
                ),
            ));
        }
        None => *cx.last_message = Some(msg::SELECT_A_WORKTREE.get().to_string()),
    }
}

pub(super) fn run(cx: &mut CommandCtx<'_>, cmd: PaletteCommand) -> bool {
    use crate::worktree_remove_dialog::{ask, Request};
    match cmd {
        PaletteCommand::WorktreeOpenInSelected => match selected(cx) {
            Some((index, wt)) => {
                let tools = crate::tab_dir::worktree_entries();
                if tools.is_empty() {
                    *cx.last_message = Some(msg::MENU_OPEN_IN_NONE_WHY.get().to_string());
                } else {
                    let (col, row) = row_anchor(cx, index);
                    let entries = tools
                        .into_iter()
                        .map(|(i, label)| {
                            crate::context_menu::Entry::action(
                                label,
                                RefContextAction::WorktreeOpenWith(i as u16),
                            )
                        })
                        .collect();
                    *cx.open_menu = OpenMenu::Ref((
                        SidebarSection::Worktrees,
                        wt.path.display().to_string(),
                        ContextMenu::new(msg::MENU_OPEN_IN.get(), entries, col, row),
                    ));
                }
            }
            None => *cx.last_message = Some(msg::SELECT_A_WORKTREE.get().to_string()),
        },
        PaletteCommand::WorktreeMergeSelected | PaletteCommand::WorktreeRemoveSelected => {
            match selected(cx) {
                Some((_, wt)) if cmd == PaletteCommand::WorktreeRemoveSelected && wt.is_main => {
                    *cx.last_message = Some(msg::MAIN_CANNOT_BE_REMOVED.get().to_string());
                }
                Some((_, wt)) if cmd == PaletteCommand::WorktreeMergeSelected => {
                    if wt.branch.is_none() {
                        *cx.last_message = Some(msg::DETACHED_NO_MERGE.get().to_string());
                    } else {
                        ask(Request::Merge(wt.path));
                    }
                }
                Some((_, wt)) => ask(Request::Remove(wt.path)),
                None => *cx.last_message = Some(msg::SELECT_A_WORKTREE.get().to_string()),
            }
        }
        PaletteCommand::CreateWorktreeMenu => {
            super::show_worktrees(cx);
            let (col, row) = anchor(cx);
            open_plus_menu(cx, col, row);
        }
        PaletteCommand::CreateWorktreeFromCurrentBranch
        | PaletteCommand::CreateWorktreeChooseBranch => {
            super::show_worktrees(cx);
            let refs = cx.refs_result.as_deref().unwrap_or(&[]);
            let worktrees = cx.worktrees_result.as_deref().unwrap_or(&[]);
            let choose = cmd == PaletteCommand::CreateWorktreeChooseBranch;
            let seed = if choose {
                crate::worktree_dialog::seed_to_choose(refs, worktrees)
            } else {
                crate::worktree_dialog::seed_from_head(refs, worktrees)
            };
            match seed {
                Some(seed) => {
                    if choose {
                        crate::worktree_dialog::ask_to_pick_ref();
                    }
                    *cx.worktree_seed = Some(seed);
                }
                None => *cx.last_message = Some(msg::NO_HEAD.get().to_string()),
            }
        }
        _ => return false,
    }
    true
}
