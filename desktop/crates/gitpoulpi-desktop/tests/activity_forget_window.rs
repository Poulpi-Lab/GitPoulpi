//! Forgetting a blocked undo entry. An outside change is normally journaled as an entry of its own (so
//! nothing is blocked); this test switches that off with `GITPOULPI_GENERAL_JOURNAL_OUTSIDE`, which the
//! daemon reads process-wide, so it has a binary of its own and the entry it moves a tag under is
//! blocked for certain.

mod activity_common;
mod refs_ops_common;

use activity_common::*;
use gitpoulpi_core::domain::Direction;
use gpui_kit::TestAppContext;
use refs_ops_common::*;

#[gpui_kit::test]
fn forgetting_a_blocked_entry_asks_first_and_never_without_one(cx: &mut TestAppContext) {
    std::env::set_var("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false");
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    // Nothing recorded: the command says so instead of asking.
    run(cx, window, "palette.undo_forget");
    assert!(!dialog_open(cx, window));
    new_tag(cx, &ws, &repo, "moved");
    wait_until(cx, "the entry", |cx| {
        ws.read_with(cx, |ws, _| ws.backend().can_undo_redo(Direction::Undo))
    });
    // Something outside moved the branch: the newest entry is blocked and can be forgotten.
    repo.git(&["tag", "-f", "moved", "HEAD~1"]);
    // The daemon looks at the repository on a refresh: ask for one, then wait until the entry reads as
    // blocked for good (a moved tag is a `RefMoved`, which can be forgotten).
    ws.read_with(cx, |ws, _| ws.backend().with_link(|l| l.request_refresh()));
    wait_until(cx, "the blocked entry", |cx| {
        ws.read_with(cx, |ws, _| {
            ws.backend()
                .with_link(|l| l.journal_state())
                .blocked
                .is_some_and(|b| b.forgettable())
        })
    });
    let before = ws.read_with(cx, |ws, _| {
        ws.backend()
            .with_link(|l| l.journal_state())
            .undo
            .map(|e| e.id)
    });
    run(cx, window, "palette.undo_forget");
    assert!(dialog_open(cx, window), "forgetting asks first");
    answer(cx, window, 1);
    let same = ws.read_with(cx, |ws, _| {
        ws.backend()
            .with_link(|l| l.journal_state())
            .undo
            .map(|e| e.id)
    });
    assert_eq!(before, same, "Cancel keeps the entry");
    run(cx, window, "palette.undo_forget");
    answer(cx, window, 0);
    wait_until(cx, "the entry dropped", |cx| {
        ws.read_with(cx, |ws, _| {
            ws.backend()
                .with_link(|l| l.journal_state())
                .undo
                .map(|e| e.id)
        }) != before
    });
}
