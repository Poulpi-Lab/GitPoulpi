//! Focus, pane layout and commit-message size commands.

use crate::event_loop::*;

use super::{shared::*, CommandCtx};

pub(super) fn run(cx: &mut CommandCtx<'_>, cmd: PaletteCommand) -> bool {
    if matches!(
        cmd,
        PaletteCommand::FocusRefs
            | PaletteCommand::FocusLog
            | PaletteCommand::FocusStatus
            | PaletteCommand::FocusRefsFilter
            | PaletteCommand::FocusRefsSection(_)
            | PaletteCommand::FocusStatusSection(_)
            | PaletteCommand::CycleFocus
            | PaletteCommand::CycleFocusBack
    ) {
        release_diff_keys(cx);
    }
    match cmd {
        // -- Navigation ------------------------
        PaletteCommand::FocusRefs => {
            // A collapsed pane cannot take
            // focus -- it has nothing to show
            // -- so this lands on the center
            // instead, same as `Tab` skipping
            // over it.
            if *cx.left_pane_collapsed {
                *cx.focus = Focus::Log;
            } else {
                *cx.focus = Focus::Refs;
                land_on_open_ref_section(
                    &mut *cx.active_ref_section,
                    &*cx.collapsed_ref_sections,
                    cx.visible_sidebar,
                );
            }
        }
        PaletteCommand::FocusLog => {
            *cx.focus = Focus::Log;
            *cx.right_panel = RightPanel::CommitDetail;
        }
        PaletteCommand::FocusStatus => {
            *cx.focus = if *cx.right_pane_collapsed {
                Focus::Log
            } else {
                Focus::Status
            };
        }
        // The `Ctrl+N` badge fine-grained
        // targets: same fallback
        // rule as `FocusRefs`/`FocusStatus`
        // above: a collapsed pane has nothing
        // to focus, so land on the center.
        PaletteCommand::FocusRefsFilter => {
            if *cx.left_pane_collapsed {
                *cx.focus = Focus::Log;
            } else {
                *cx.focus = Focus::Refs;
                *cx.input_mode = InputMode::FilteringRefs(cx.refs_filter.clone());
            }
        }
        PaletteCommand::FocusRefsSection(section) => {
            if *cx.left_pane_collapsed || !cx.visible_sidebar.contains(&section) {
                *cx.focus = Focus::Log;
            } else {
                *cx.focus = Focus::Refs;
                *cx.active_ref_section = section;
            }
        }
        PaletteCommand::FocusStatusSection(section) => {
            if *cx.right_pane_collapsed {
                *cx.focus = Focus::Log;
            } else {
                *cx.focus = Focus::Status;
                *cx.active_status_section = section;
                *cx.right_panel = RightPanel::Status;
            }
        }
        // Same path as `f` and the toolbar's
        // magnifier button.
        PaletteCommand::SearchLog => {
            *cx.central_view = CentralView::Log;
            *cx.focus = Focus::Log;
            *cx.input_mode = InputMode::SearchingLog(LogSearch::new());
        }
        PaletteCommand::ToggleRefsPaneCollapsed => {
            *cx.left_pane_collapsed = !*cx.left_pane_collapsed;
            if *cx.left_pane_collapsed && *cx.focus == Focus::Refs {
                *cx.focus = Focus::Log;
            }
            persist_pane_layout(
                cx.session,
                *cx.divider1_pct,
                *cx.divider2_pct,
                cx.left_auto_collapse.persisted(*cx.left_pane_collapsed),
                cx.right_auto_collapse.persisted(*cx.right_pane_collapsed),
            );
        }
        PaletteCommand::GrowCommitMessage => resize_commit_message(cx, Resize::By(1)),
        PaletteCommand::ShrinkCommitMessage => resize_commit_message(cx, Resize::By(-1)),
        PaletteCommand::ResetCommitMessageSize => resize_commit_message(cx, Resize::Reset),
        PaletteCommand::ScrollCommitMessageDown => scroll_commit_message(cx, 3),
        PaletteCommand::ScrollCommitMessageUp => scroll_commit_message(cx, -3),
        PaletteCommand::ToggleCommitFileSort => toggle_commit_sort(cx),
        PaletteCommand::GrowRefsSection => resize_sidebar_section(cx, Resize::By(1)),
        PaletteCommand::ShrinkRefsSection => resize_sidebar_section(cx, Resize::By(-1)),
        PaletteCommand::ResetRefsSectionSizes => resize_sidebar_section(cx, Resize::Reset),
        PaletteCommand::ResetLayout => super::shared::reset_layout(cx),
        PaletteCommand::ResizePane(resize) => resize_pane(cx, resize),
        PaletteCommand::ToggleStatusPaneCollapsed => {
            *cx.right_pane_collapsed = !*cx.right_pane_collapsed;
            if *cx.right_pane_collapsed && *cx.focus == Focus::Status {
                *cx.focus = Focus::Log;
            }
            persist_pane_layout(
                cx.session,
                *cx.divider1_pct,
                *cx.divider2_pct,
                cx.left_auto_collapse.persisted(*cx.left_pane_collapsed),
                cx.right_auto_collapse.persisted(*cx.right_pane_collapsed),
            );
        }
        PaletteCommand::CycleFocus | PaletteCommand::CycleFocusBack => {
            *cx.focus = cycle_focus_skipping_collapsed(
                *cx.focus,
                cmd == PaletteCommand::CycleFocusBack,
                *cx.left_pane_collapsed,
                *cx.right_pane_collapsed,
            );
            if *cx.focus == Focus::Log {
                *cx.right_panel = RightPanel::CommitDetail;
            }
            if *cx.focus == Focus::Refs {
                land_on_open_ref_section(
                    &mut *cx.active_ref_section,
                    &*cx.collapsed_ref_sections,
                    cx.visible_sidebar,
                );
            }
        }

        _ => return false,
    }
    true
}
