//! The left pane's section show/hide: the palette commands, the checklist
//! menu and the one function every surface (click, key, palette) runs.

use super::CommandCtx;
use crate::event_loop::*;
use crate::refs_sidebar::{available_sections, can_hide, visible_sections_with};
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
        PaletteCommand::UseSidebarSectionsForAll => {
            apply(cx, RefContextAction::UseSidebarSectionsForAll);
        }
        _ => return false,
    }
    true
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
        &*cx.hidden_sidebar_sections,
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
        RefContextAction::ToggleSidebarSection(section) => {
            if cx.hidden_sidebar_sections.remove(&section) {
                // Shown again.
                cx.sidebar_scope.touch(&*cx.hidden_sidebar_sections);
            } else if can_hide(section, &*cx.hidden_sidebar_sections, &available) {
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
        &*cx.hidden_sidebar_sections,
        &available,
        cx.sidebar_scope.is_global(),
    );
    if let Some((_, key, menu)) = cx.open_menu.ref_menu_mut() {
        if key == sections_menu::MENU_KEY {
            menu.replace_entries(entries);
        }
    }
}
