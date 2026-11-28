//! The palette's `Jobs: Cancel <job>` rows read process-wide facts (the palette's families are plain
//! function pointers, see `panels::activity::families`) that every window of the process shares. This
//! binary therefore holds one test and no other window runs next to it.

mod activity_common;
mod refs_ops_common;

use activity_common::*;
use gitpoulpi_core::domain::OperationKind;
use gitpoulpi_desktop::commands;
use gitpoulpi_ipc::JobState;
use gpui_kit::{AppContext as _, TestAppContext};
use refs_ops_common::*;

#[gpui_kit::test]
fn the_palette_has_one_cancel_row_per_cancellable_job_and_cancel_all(cx: &mut TestAppContext) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    // The link's own job list arrives with the first load: feeding a job before it would be wiped by it.
    wait_until(cx, "the first load", |cx| {
        ws.read_with(cx, |ws, cx| ws.log.read(cx).loaded_commits()) > 0
    });
    feed_job(
        cx,
        &ws,
        job(1, OperationKind::Push, JobState::Running, "/work/tool"),
    );
    let state = cx
        .update_window(window, |_, w, cx| commands::context::window_state(w, cx))
        .unwrap();
    let rows = commands::palette::rows("Jobs: Cancel", state.palette_context());
    assert!(
        rows.iter()
            .any(|r| !r.dim && r.label.contains("Push: tool")),
        "a Cancel row for the push: {:?}",
        rows.iter().map(|r| r.label.clone()).collect::<Vec<_>>()
    );
    run(cx, window, "palette.jobs_cancel_all");
}
