//! The Blame view's actions: one function behind the keys, the card and
//! menu rows, the border drag's keyboard twins and the palette's
//! `Blame: ...` entries.

use crate::blame_view::BlameOp;
use crate::event_loop::*;
use gitpoulpi_i18n::msg::blame_view as msg;

use super::CommandCtx;

/// How many cells `Wider` / `Narrower` move the border.
const WIDTH_STEP: u16 = 4;

pub(super) fn run_palette(cx: &mut CommandCtx<'_>, cmd: PaletteCommand) -> bool {
    match cmd {
        PaletteCommand::BlameOp(op) => {
            run(cx, op);
            true
        }
        _ => false,
    }
}

/// Persists the annotation width the border was left at.
pub(in crate::event_loop) fn save_width(cx: &mut CommandCtx<'_>, width: u16) {
    if let Some(path) = cx.diff_config_path {
        let _ = gitpoulpi_config::set_in_file(path, "ui.blame_width", &width.to_string());
    }
}

pub(in crate::event_loop) fn run(cx: &mut CommandCtx<'_>, op: BlameOp) {
    let viewport = cx.file_insight_area.height.saturating_sub(2) as usize;
    let Some(insight) = cx.file_insight.as_mut() else {
        *cx.last_message = Some(msg::WHY_NO_BLAME.get().into());
        return;
    };
    if insight.blame.is_none() {
        *cx.last_message = Some(msg::WHY_NO_BLAME.get().into());
        return;
    }
    match op {
        BlameOp::NextHunk | BlameOp::PrevHunk | BlameOp::NextSame | BlameOp::PrevSame => {
            if let Err(why) = insight.blame_nav(op, viewport) {
                *cx.last_message = Some(why.get().into());
            }
            return;
        }
        BlameOp::ToggleCard => {
            if let Some(b) = insight.blame.as_mut() {
                b.pinned = !b.pinned;
            }
            return;
        }
        BlameOp::Goto => {
            if let Some(b) = insight.blame.as_mut() {
                b.goto = Some(String::new());
            }
            return;
        }
        BlameOp::Wider | BlameOp::Narrower => {
            let now = crate::live_settings::blame_width();
            let next = if op == BlameOp::Wider {
                now.saturating_add(WIDTH_STEP)
            } else {
                now.saturating_sub(WIDTH_STEP)
            };
            crate::live_settings::install_blame_width(next);
            save_width(cx, crate::live_settings::blame_width());
            return;
        }
        _ => {}
    }
    // The rest act on the commit of the selected line's hunk.
    let Some(commit) = insight.blame_selected_commit() else {
        *cx.last_message = Some(msg::NOT_COMMITTED_YET.get().into());
        insight.blame_close_popups();
        return;
    };
    let path = insight.request.path.clone();
    let history = FileInsightRequest {
        kind: FileInsightKind::History,
        ..insight.request.clone()
    };
    let parent = insight.blame_parent_target();
    insight.blame_close_popups();
    match op {
        BlameOp::CopySha => {
            *cx.last_message = Some(cx.clipboard.copy(msg::WHAT_SHA.get(), &commit.to_hex()));
        }
        BlameOp::ShowInLog | BlameOp::ShowDiff => {
            if select_log_commit(&*cx.log_result, &*cx.log_order, &mut *cx.log_state, commit) {
                close_file_panel(&mut *cx.file_insight, &mut *cx.diff_view_state);
                *cx.focus = Focus::Log;
                *cx.right_panel = RightPanel::CommitDetail;
                if op == BlameOp::ShowDiff {
                    crate::blame_view::want_diff(commit, path);
                }
            } else {
                *cx.last_message = Some(gitpoulpi_i18n::msg::input::not_in_loaded_log(
                    &commit.to_hex()[..8],
                ));
            }
        }
        BlameOp::BlameParent => match parent {
            Ok(request) => open_insight_request(request, cx.link, &mut *cx.file_insight),
            Err(why) => *cx.last_message = Some(why.get().into()),
        },
        BlameOp::OpenHistory => open_insight_request(history, cx.link, &mut *cx.file_insight),
        _ => {}
    }
}
