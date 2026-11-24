//! The open diff: opening, hunks, modes and the stage/discard actions.

use crate::event_loop::*;
use gitpoulpi_i18n::msg;

use super::{shared::*, CommandCtx};

pub(super) fn run(cx: &mut CommandCtx<'_>, cmd: PaletteCommand) -> bool {
    match cmd {
        PaletteCommand::HistoryView(action) => super::history::run_action(cx, action),
        PaletteCommand::CommitColumn(cmd) => return super::commit_column::run(cx, cmd),
        // -- Diff ------------------------------
        // Two sources, one command: the
        // commit-detail file list when that is
        // what has a selection, otherwise the
        // third pane's own row -- which is the
        // only way to reach a diff that can be
        // staged out of.
        PaletteCommand::OpenDiff => {
            let commit_row = cx
                .commit_file_state
                .selected()
                .filter(|_| *cx.right_panel == RightPanel::CommitDetail)
                .and_then(|i| cx.commit_file_rows.get(i))
                .and_then(|r| r.file_index());
            let status_row = section_lines(&*cx.status_section_lines, *cx.active_status_section)
                .get(
                    cx.status_states
                        .state(*cx.active_status_section)
                        .selected()
                        .unwrap_or(usize::MAX),
                )
                .and_then(StatusPaneLine::file)
                .map(|(section, path)| (section, path.clone()));
            match (commit_row, cx.commit_files.as_ref(), status_row) {
                (Some(i), Some(outcome), _) => {
                    let path = outcome
                        .files
                        .as_ref()
                        .ok()
                        .and_then(|f| f.get(i))
                        .map(|f| f.path.clone())
                        .unwrap_or_default();
                    *cx.central_view = CentralView::Diff;
                    *cx.diff_view_state = Some(
                        match cx.compare.as_ref().filter(|c| c.id == outcome.commit) {
                            Some(c) => DiffViewState::for_compare(c.id, c.target.clone(), i, path),
                            None => DiffViewState::for_commit(outcome.commit, i, path),
                        }
                        .resume_last_used(),
                    );
                    *cx.ref_popup = None;
                    cx.open_menu.close_column_menu();
                }
                (_, _, Some((section, path))) => {
                    *cx.central_view = CentralView::Diff;
                    *cx.diff_view_state = Some(working_diff_state(section, path));
                    *cx.ref_popup = None;
                    cx.open_menu.close_column_menu();
                }
                _ => {
                    *cx.last_message = Some(msg::input::SELECT_FILE_STATUS_OR_COMMIT.get().into());
                }
            }
        }
        PaletteCommand::CloseDiff => {
            *cx.central_view = CentralView::Log;
            *cx.diff_view_state = None;
        }
        PaletteCommand::NextHunk | PaletteCommand::PrevHunk => {
            if let Some(s) = cx.diff_view_state.as_mut() {
                if let Err(why) = diff_view::jump_to_hunk(
                    s,
                    &*cx.diff_hunk_starts,
                    cmd == PaletteCommand::NextHunk,
                    cx.diff_body_area.height.saturating_sub(2),
                ) {
                    *cx.last_message = Some(why.into());
                }
            }
        }
        PaletteCommand::DiffMode(m) => {
            if let Some(s) = cx.diff_view_state.as_mut() {
                persist_diff_view_toggle(DiffViewToggle::Mode(m), s, cx.diff_config_path);
            }
        }
        PaletteCommand::FilesTogglePathTree => {
            toggle_file_list_mode(cx);
        }
        PaletteCommand::FilesToggleViewAll => {
            toggle_view_all_files(cx);
        }
        PaletteCommand::CommitToggleSkipHooks => {
            *cx.commit_skip_hooks = !*cx.commit_skip_hooks;
        }
        PaletteCommand::CompareSelection => compare_selection(cx),
        PaletteCommand::CompareSwap => compare_swap(cx),
        PaletteCommand::CompareGotoNext => compare_goto_next(cx),
        PaletteCommand::ToggleDiffHunksOnly => {
            if let Some(s) = cx.diff_view_state.as_mut() {
                persist_diff_view_toggle(DiffViewToggle::ToggleHunksOnly, s, cx.diff_config_path);
            }
        }
        PaletteCommand::ToggleWordWrap => {
            if let Some(s) = cx.diff_view_state.as_mut() {
                persist_diff_view_toggle(DiffViewToggle::ToggleWordWrap, s, cx.diff_config_path);
            }
        }
        PaletteCommand::DiffChooseEncoding => {
            if let Some(s) = cx.diff_view_state.as_ref() {
                crate::diff_encoding_menu::open(s.encoding);
            }
        }
        PaletteCommand::EditorToggleFollowSelection => {
            *cx.last_message = Some(crate::editor_prefs::toggle_follow_selection(
                cx.diff_config_path,
            ));
        }
        PaletteCommand::ToggleMinimap => {
            *cx.last_message = Some(toggle_minimap(cx.diff_config_path));
        }
        PaletteCommand::ToggleDiffIntraline => {
            *cx.last_message = Some(crate::diff_actions::toggle_diff_intraline(
                cx.diff_config_path,
            ));
        }
        PaletteCommand::ImageZoomIn
        | PaletteCommand::ImageZoomOut
        | PaletteCommand::ImageFit
        | PaletteCommand::ImageActualSize => {
            use crate::image_view::pane::ZoomOp;
            let op = match cmd {
                PaletteCommand::ImageZoomIn => ZoomOp::In,
                PaletteCommand::ImageZoomOut => ZoomOp::Out,
                PaletteCommand::ImageFit => ZoomOp::Fit,
                _ => ZoomOp::Actual,
            };
            // The Diff view's pictures, else the File View's.
            if !(*cx.central_view == CentralView::Diff && crate::image_view::split::run_zoom(op)) {
                crate::image_view::pane::run_zoom(op);
            }
        }
        PaletteCommand::ImageFlipSide => {
            crate::image_view::split::flip();
        }
        PaletteCommand::ImageWidenBefore => {
            crate::image_view::split::widen(1);
        }
        PaletteCommand::ImageWidenAfter => {
            crate::image_view::split::widen(-1);
        }
        PaletteCommand::ImageResetSplit => {
            crate::image_view::split::reset_split();
        }
        PaletteCommand::ImageEditIn => {
            crate::image_view::edit_in::open(crate::image_view::edit_in::Target::Pane);
        }
        PaletteCommand::ImageEditDefault => {
            *cx.last_message = Some(crate::image_view::edit_in::run_default(
                &crate::image_view::edit_in::Target::Pane,
                cx.repo_root,
            ));
        }
        PaletteCommand::OpenWith => {
            if let Some(subject) = crate::open_with::view::shown() {
                crate::open_with::popup::open(subject);
            }
        }
        PaletteCommand::OpenWithDefault => {
            *cx.last_message = Some(match crate::open_with::view::shown() {
                Some(subject) => {
                    crate::open_with::popup::run_default(&subject, cx.repo_root, cx.link)
                }
                None => gitpoulpi_i18n::msg::palette::WHY_NO_BINARY
                    .get()
                    .to_string(),
            });
        }
        PaletteCommand::ToggleWhitespace => {
            if let Some(s) = cx.diff_view_state.as_mut() {
                persist_diff_view_toggle(DiffViewToggle::ToggleWhitespace, s, cx.diff_config_path);
            }
        }
        // Both directions run the same code:
        // the open diff's own half decides
        // which way a hunk moves, and the
        // command that doesn't apply is
        // already grayed out with the reason
        // (see `palette::unavailable_reason`).
        PaletteCommand::StageHunk
        | PaletteCommand::UnstageHunk
        | PaletteCommand::StageLines
        | PaletteCommand::UnstageLines
        | PaletteCommand::SelectLines
        | PaletteCommand::DiscardHunk
        | PaletteCommand::DiscardLines => {
            let action = match cmd {
                PaletteCommand::SelectLines => DiffStageAction::SelectLines,
                PaletteCommand::StageLines | PaletteCommand::UnstageLines => DiffStageAction::Lines,
                PaletteCommand::DiscardHunk => DiffStageAction::DiscardHunk,
                PaletteCommand::DiscardLines => DiffStageAction::DiscardLines,
                _ => DiffStageAction::Hunk,
            };
            if let Some(s) = cx.diff_view_state.as_mut() {
                *cx.last_message = Some(apply_diff_stage(
                    action,
                    s,
                    &*cx.diff_rows,
                    None,
                    cx.link,
                    &mut *cx.input_mode,
                ));
            }
        }

        _ => return false,
    }
    true
}
