//! The left pane's section show/hide: the palette commands, the checklist
//! menu and the one function every surface (click, key, palette) runs.

use super::CommandCtx;
use crate::event_loop::*;
use crate::refs_sidebar::{
    available_sections, can_hide, effective_hidden, set_worktrees_shown, visible_sections_with,
};
use crate::sections_menu;
use gitpoulpi_i18n::msg;

pub(super) fn run(cx: &mut CommandCtx<'_>, cmd: PaletteCommand) -> bool {
    match cmd {
        PaletteCommand::SidebarSectionsMenu => {
            // Under the pane's top-left, where the right-click would be.
            let (col, row) = (cx.refs_area.x + 2, cx.refs_area.y + 1);
            open_menu(cx, col, row);
        }
        PaletteCommand::ToggleSidebarSection(section) => {
            apply(cx, RefContextAction::ToggleSidebarSection(section));
        }
        PaletteCommand::ResetSidebarSections => apply(cx, RefContextAction::ResetSidebarSections),
        PaletteCommand::ShowWorktreesSection => {
            show_worktrees(cx);
        }
        PaletteCommand::UseSidebarSectionsForAll => {
            apply(cx, RefContextAction::UseSidebarSectionsForAll);
        }
        _ => return false,
    }
    true
}

fn linked(cx: &CommandCtx<'_>) -> usize {
    crate::refs_sidebar::linked_worktree_count(cx.worktrees_result.as_deref().unwrap_or(&[]))
}

fn availability(cx: &CommandCtx<'_>) -> Vec<SidebarSection> {
    available_sections(
        crate::refs_sidebar::linked_worktree_count(cx.worktrees_result.as_deref().unwrap_or(&[])),
        cx.stashes_result.as_deref().unwrap_or(&[]).len(),
        cx.hosting.sections_visible(),
        cx.submodules_result.len(),
    )
}

/// Opens the checklist at `(col, row)`.
pub(in crate::event_loop) fn open_menu(cx: &mut CommandCtx<'_>, col: u16, row: u16) {
    let entries = sections_menu::entries(
        &effective_hidden(&*cx.hidden_sidebar_sections, linked(cx)),
        &availability(cx),
        cx.sidebar_scope.is_global(),
    );
    *cx.focus = Focus::Refs;
    *cx.open_menu = OpenMenu::Ref((
        SidebarSection::Local,
        sections_menu::MENU_KEY.to_string(),
        ContextMenu::new(msg::sidebar::SECTIONS_MENU_TITLE.get(), entries, col, row),
    ));
}

/// Runs a checklist action. The checklist, when it is open, stays open with
/// its boxes redrawn; the pane repaints on the next frame.
pub(in crate::event_loop) fn apply(cx: &mut CommandCtx<'_>, action: RefContextAction) {
    let available = availability(cx);
    match action {
        RefContextAction::ToggleSidebarSection(SidebarSection::Worktrees) => {
            // The stored set keeps WORKTREES the way it is shown: its own
            // entry with linked worktrees, the "shown while empty" mark
            // without any.
            let count = linked(cx);
            let shown = !effective_hidden(&*cx.hidden_sidebar_sections, count)
                .contains(&SidebarSection::Worktrees);
            if !shown
                || can_hide(
                    SidebarSection::Worktrees,
                    &effective_hidden(&*cx.hidden_sidebar_sections, count),
                    &available,
                )
            {
                set_worktrees_shown(&mut *cx.hidden_sidebar_sections, count, !shown);
                cx.sidebar_scope.touch(&*cx.hidden_sidebar_sections);
            } else {
                *cx.last_message = Some(msg::sidebar::SECTIONS_LAST_ONE.get().into());
            }
        }
        RefContextAction::ToggleSidebarSection(section) => {
            if cx.hidden_sidebar_sections.remove(&section) {
                // Shown again.
                cx.sidebar_scope.touch(&*cx.hidden_sidebar_sections);
            } else if can_hide(
                section,
                &effective_hidden(&*cx.hidden_sidebar_sections, linked(cx)),
                &available,
            ) {
                cx.hidden_sidebar_sections.insert(section);
                cx.sidebar_scope.touch(&*cx.hidden_sidebar_sections);
            } else {
                *cx.last_message = Some(msg::sidebar::SECTIONS_LAST_ONE.get().into());
            }
        }
        RefContextAction::ResetSidebarSections => {
            cx.sidebar_scope.reset(&mut *cx.hidden_sidebar_sections);
            *cx.last_message = Some(msg::sidebar::SECTIONS_RESET_DONE.get().into());
        }
        RefContextAction::UseSidebarSectionsForAll => {
            cx.sidebar_scope.use_for_all(&*cx.hidden_sidebar_sections);
            *cx.last_message = Some(msg::sidebar::SECTIONS_FOR_ALL_DONE.get().into());
        }
        _ => return,
    }
    // A hidden section cannot keep the keyboard cursor.
    let visible = visible_sections_with(
        crate::refs_sidebar::linked_worktree_count(cx.worktrees_result.as_deref().unwrap_or(&[])),
        cx.stashes_result.as_deref().unwrap_or(&[]).len(),
        cx.hosting.sections_visible(),
        cx.submodules_result.len(),
        &*cx.hidden_sidebar_sections,
    );
    if !visible.contains(&*cx.active_ref_section) {
        if let Some(first) = visible.first() {
            *cx.active_ref_section = *first;
        }
    }
    let entries = sections_menu::entries(
        &effective_hidden(&*cx.hidden_sidebar_sections, linked(cx)),
        &available,
        cx.sidebar_scope.is_global(),
    );
    if let Some((_, key, menu)) = cx.open_menu.ref_menu_mut() {
        if key == sections_menu::MENU_KEY {
            menu.replace_entries(entries);
        }
    }
}

/// Shows WORKTREES (even in a repository with no worktree yet, where it
/// then holds the "no worktree yet" hint and the `+`) and puts the keyboard
/// cursor on it. What "Show Worktrees section" and the create-worktree
/// commands do first.
pub(in crate::event_loop) fn show_worktrees(cx: &mut CommandCtx<'_>) {
    let count = linked(cx);
    let hidden = effective_hidden(&*cx.hidden_sidebar_sections, count);
    if hidden.contains(&SidebarSection::Worktrees) {
        set_worktrees_shown(&mut *cx.hidden_sidebar_sections, count, true);
        cx.sidebar_scope.touch(&*cx.hidden_sidebar_sections);
    }
    *cx.left_pane_collapsed = false;
    *cx.active_ref_section = SidebarSection::Worktrees;
}
