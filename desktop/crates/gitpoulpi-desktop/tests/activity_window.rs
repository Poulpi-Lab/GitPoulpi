//! Headless tests of the undo history, the Reflog and Lost work views on the real main window: what each
//! entry says, "undo up to here" and "redo up to here" with their confirmations, "restore only this", a
//! risky undo asking again, the reflog and lost-work recoveries, and the palette and keys that open
//! the panel. Fixture repositories and a temporary config directory; no network, no visible window.

mod activity_common;
mod refs_ops_common;

use activity_common::*;
use gitpoulpi_core::domain::Direction;
use gitpoulpi_desktop::commands;
use gitpoulpi_desktop::panels::activity::resolve;
use gitpoulpi_desktop::panels::activity::safety_model::{Act, Tab};
use gitpoulpi_i18n::msg;
use gitpoulpi_link::Action;
use gpui_kit::{AppContext as _, TestAppContext};
use refs_ops_common::*;

// ---- the history -----------------------------------------------------------------------------------

#[gpui_kit::test]
fn the_history_lists_every_entry_newest_first_and_the_newest_is_selected(cx: &mut TestAppContext) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    for name in ["u1", "u2", "u3"] {
        new_tag(cx, &ws, &repo, name);
        wait_until(cx, "the entry", |cx| {
            ws.read_with(cx, |ws, _| ws.backend().can_undo_redo(Direction::Undo))
        });
    }
    let view = open_safety(cx, window, "global.undo_history", Tab::History);
    wait_until(cx, "the three entries", |cx| {
        history_texts(cx, &view)
            .iter()
            .filter(|t| t.contains("u"))
            .count()
            >= 3
    });
    let texts = history_texts(cx, &view);
    let at = |name: &str| texts.iter().position(|t| t.contains(name)).unwrap();
    assert!(
        at("u3") < at("u2") && at("u2") < at("u1"),
        "newest first: {texts:?}"
    );
    // The newest is selected and offers Undo up to here; nothing is undone yet, so no Redo.
    let acts = view.read_with(cx, |v, _| v.actions());
    assert!(acts.contains(&Act::UndoTo));
    assert!(!acts.contains(&Act::RedoTo));
    assert!(view.read_with(cx, |v, _| v.history.rows.iter().all(|r| !r.redo)));
}

#[gpui_kit::test]
fn undo_up_to_here_asks_with_the_list_then_undoes_them_and_redo_up_to_here_brings_them_back(
    cx: &mut TestAppContext,
) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    for name in ["u1", "u2", "u3"] {
        new_tag(cx, &ws, &repo, name);
        wait_until(cx, "the entry", |cx| {
            ws.read_with(cx, |ws, _| ws.backend().can_undo_redo(Direction::Undo))
        });
    }
    let view = open_safety(cx, window, "global.undo_history", Tab::History);
    wait_until(cx, "the entries", |cx| {
        history_texts(cx, &view)
            .iter()
            .filter(|t| t.contains("u"))
            .count()
            >= 3
    });
    // Select u2 (the middle one): u3 and u2 go, u1 stays.
    let id = view.read_with(cx, |v, _| {
        v.history
            .rows
            .iter()
            .find(|r| gitpoulpi_link::undo_label::entry_text(&r.entry).contains("u2"))
            .map(|r| r.entry.summary.id)
            .unwrap()
    });
    view.update(cx, |v, cx| {
        v.history.select_entry(id);
        cx.notify();
    });
    let picked = view.read_with(cx, |v, _| v.history.selected_row().unwrap());
    assert!(picked >= 1, "u3 is newer");
    act(cx, &view, Act::UndoTo);
    assert!(dialog_open(cx, window), "undo up to here asks first");
    assert_eq!(
        cx.update(|cx| gitpoulpi_desktop::dialogs::current(cx))
            .unwrap()
            .read_with(cx, |q, _| q.spec().lines.len()),
        picked + 1,
        "the question lists the entries from the newest up to the chosen one"
    );
    // Cancel changes nothing.
    answer(cx, window, 1);
    cx.run_until_parked();
    assert!(has_tag(&repo, "u3") && has_tag(&repo, "u2"));
    // Undo them.
    act(cx, &view, Act::UndoTo);
    answer(cx, window, 0);
    wait_until(cx, "u3 and u2 gone", |_| {
        !has_tag(&repo, "u3") && !has_tag(&repo, "u2")
    });
    settle(cx, &view);
    assert!(has_tag(&repo, "u1"), "the older entry stays");
    // They are on the redo stack now.
    wait_until(cx, "the redo rows", |cx| {
        view.read_with(cx, |v, _| {
            v.history.rows.iter().filter(|r| r.redo).count() == 2
        })
    });
    let redo_id = view.read_with(cx, |v, _| {
        v.history
            .rows
            .iter()
            .rfind(|r| r.redo)
            .unwrap()
            .entry
            .summary
            .id
    });
    view.update(cx, |v, cx| {
        v.history.select_entry(redo_id);
        cx.notify();
    });
    let acts = view.read_with(cx, |v, _| v.actions());
    assert_eq!(
        acts,
        vec![Act::RedoTo],
        "an undone entry can only be redone"
    );
    act(cx, &view, Act::RedoTo);
    assert!(dialog_open(cx, window));
    answer(cx, window, 0);
    wait_until(cx, "u2 and u3 back", |_| {
        has_tag(&repo, "u2") && has_tag(&repo, "u3")
    });
    settle(cx, &view);
    assert!(
        !notice(cx, &ws).is_empty(),
        "the status line says what was redone"
    );
}

#[gpui_kit::test]
fn restore_only_this_shows_its_plan_and_keeps_the_later_actions(cx: &mut TestAppContext) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    new_tag(cx, &ws, &repo, "keepme");
    wait_until(cx, "the entry", |cx| {
        ws.read_with(cx, |ws, _| ws.backend().can_undo_redo(Direction::Undo))
    });
    ws.read_with(cx, |ws, _| {
        ws.backend().submit(Action::TagDelete("keepme".into()))
    });
    wait_until(cx, "the delete", |_| !has_tag(&repo, "keepme"));
    new_tag(cx, &ws, &repo, "later");
    let view = open_safety(cx, window, "global.undo_history", Tab::History);
    wait_until(cx, "the entries", |cx| history_texts(cx, &view).len() >= 3);
    // Select the delete entry: it is restorable on its own.
    let id = view.read_with(cx, |v, _| {
        v.history
            .rows
            .iter()
            .find(|r| r.entry.restore.restorable())
            .map(|r| r.entry.summary.id)
            .expect("the deleted branch can be restored on its own")
    });
    view.update(cx, |v, cx| {
        v.history.select_entry(id);
        cx.notify();
    });
    assert!(view
        .read_with(cx, |v, _| v.actions())
        .contains(&Act::RestoreOnly));
    act(cx, &view, Act::RestoreOnly);
    wait_until(cx, "the plan", |cx| dialog_open(cx, window));
    assert_eq!(question_title(cx), msg::undo_safety::BTN_RESTORE_ONLY.get());
    answer(cx, window, 0);
    wait_until(cx, "the branch restored", |_| has_tag(&repo, "keepme"));
    settle(cx, &view);
    assert!(has_tag(&repo, "later"), "the later action stays");
    // One action, one entry: a plain Undo takes the restore back.
    undo(cx, window, &ws);
    wait_until(cx, "the restore undone", |_| !has_tag(&repo, "keepme"));
    assert!(has_tag(&repo, "later"));
}

#[gpui_kit::test]
fn a_risky_undo_the_daemon_flags_asks_again_and_cancel_leaves_everything(cx: &mut TestAppContext) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    new_tag(cx, &ws, &repo, "risky");
    wait_until(cx, "the entry", |cx| {
        ws.read_with(cx, |ws, _| ws.backend().can_undo_redo(Direction::Undo))
    });
    let expect = ws
        .read_with(cx, |ws, _| ws.backend().with_link(|l| l.journal_state()))
        .undo
        .map(|e| e.id);
    let confirm = || gitpoulpi_link::undo_ui::PendingConfirm {
        direction: Direction::Undo,
        expect,
        prompt: "The branch has commits nobody else has. Undo anyway?".to_string(),
        sequence: None,
    };
    let backend = ws.read_with(cx, |ws, _| ws.backend().clone());
    cx.update_window(window, |_, w, cx| {
        resolve::ask_undo_confirm(confirm(), backend.clone(), w, cx)
    })
    .unwrap();
    assert!(dialog_open(cx, window));
    assert_eq!(
        question_title(cx),
        "The branch has commits nobody else has. Undo anyway?"
    );
    // Nothing is highlighted: a stray Enter answers nothing, and Cancel does nothing.
    answer(cx, window, 1);
    assert!(has_tag(&repo, "risky"));
    cx.update_window(window, |_, w, cx| {
        resolve::ask_undo_confirm(confirm(), backend.clone(), w, cx)
    })
    .unwrap();
    answer(cx, window, 0);
    wait_until(cx, "the confirmed undo", |_| !has_tag(&repo, "risky"));
}

// ---- Reflog and Lost work --------------------------------------------------------------------------

#[gpui_kit::test]
fn the_reflog_lists_positions_and_creates_a_branch_where_one_was(cx: &mut TestAppContext) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    let view = open_safety(cx, window, "palette.reflog_head", Tab::Reflog);
    let (at, short) = view.read_with(cx, |v, _| {
        let e = v.reflog.selected().expect("a position is selected").clone();
        (e.new, gitpoulpi_core::domain::short(e.new))
    });
    assert!(
        view.read_with(cx, |v, _| v.reflog.entries.len()) >= 2,
        "the fixture has history"
    );
    // Create branch here: a name form, prefilled, then the branch is at that commit.
    act(cx, &view, Act::CreateBranch);
    wait_until(cx, "the name form", |cx| {
        cx.update(|cx| gitpoulpi_desktop::form_dialog::current(cx))
            .is_some()
    });
    type_into(cx, window, 0, "recovered");
    submit(cx, window);
    wait_until(cx, "the branch", |_| has_branch(&repo, "recovered"));
    assert_eq!(repo.git(&["rev-parse", "recovered"]).trim(), at.to_hex());
    settle(cx, &view);
    assert!(view
        .read_with(cx, |v, _| v.note.clone())
        .unwrap()
        .0
        .contains(&short));
    // A taken name is refused with the daemon's words, nothing is created twice.
    act(cx, &view, Act::CreateBranch);
    wait_until(cx, "the name form", |cx| {
        cx.update(|cx| gitpoulpi_desktop::form_dialog::current(cx))
            .is_some()
    });
    type_into(cx, window, 0, "recovered");
    submit(cx, window);
    settle(cx, &view);
    assert!(
        view.read_with(cx, |v, _| v.note.clone()).unwrap().1,
        "the failure is an error note"
    );
    let _ = ws;
}

#[gpui_kit::test]
fn lost_work_finds_a_commit_no_branch_reaches_and_keeps_it_under_a_tag(cx: &mut TestAppContext) {
    let repo = repo();
    std::fs::write(repo.root().join("lost.txt"), "lost\n").unwrap();
    repo.git(&["add", "lost.txt"]);
    repo.git(&["commit", "-q", "-m", "work that gets lost"]);
    let lost = repo.git(&["rev-parse", "HEAD"]).trim().to_string();
    repo.git(&["reset", "--hard", "-q", "HEAD~1"]);
    let (window, _ws) = open(cx, &repo);
    let view = open_safety(cx, window, "palette.lost_commits", Tab::Lost);
    wait_until(cx, "the lost commit", |cx| {
        view.read_with(cx, |v, _| {
            v.lost
                .commits
                .iter()
                .any(|c| c.summary == "work that gets lost")
        })
    });
    let ix = view.read_with(cx, |v, _| {
        v.lost
            .commits
            .iter()
            .position(|c| c.oid.to_hex() == lost)
            .unwrap()
    });
    view.update(cx, |v, cx| v.select(ix, cx));
    // No "Show in Log" for a commit the Log does not have.
    let acts = view.read_with(cx, |v, _| v.actions());
    assert!(!acts.contains(&Act::ShowInLog) && acts.contains(&Act::CherryPick));
    act(cx, &view, Act::CreateTag);
    wait_until(cx, "the name form", |cx| {
        cx.update(|cx| gitpoulpi_desktop::form_dialog::current(cx))
            .is_some()
    });
    type_into(cx, window, 0, "rescued");
    submit(cx, window);
    wait_until(cx, "the tag", |_| {
        repo.git(&["rev-parse", "rescued^{commit}"]).trim() == lost
    });
}

// ---- opening it ------------------------------------------------------------------------------------

#[gpui_kit::test]
fn the_panel_opens_from_the_key_the_palette_and_the_toolbar_menu_commands(cx: &mut TestAppContext) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    // The key.
    press(cx, window, "alt-z");
    wait_until(cx, "the panel", |cx| {
        cx.update(|cx| gitpoulpi_desktop::panels::activity::safety::current(cx))
            .is_some()
    });
    assert_eq!(safety_view(cx).read_with(cx, |v, _| v.tab), Tab::History);
    // The palette rows of the open panel are available; the Reflog row switches views.
    let state = cx
        .update_window(window, |_, w, cx| commands::context::window_state(w, cx))
        .unwrap();
    assert_eq!(state.safety_tab, Some(Tab::History));
    let rows = commands::palette::rows("Undo History: Refresh", state.palette_context());
    assert!(
        rows.iter().any(|r| !r.dim),
        "a panel row is available while the panel is open"
    );
    // The toolbar menu's three commands are registry ids the dispatcher runs.
    for id in [
        "global.undo_history",
        "palette.reflog_head",
        "palette.lost_commits",
    ] {
        assert!(commands::implemented_ids().contains(id), "{id}");
    }
    run(cx, window, "palette.reflog_head");
    assert_eq!(safety_view(cx).read_with(cx, |v, _| v.tab), Tab::Reflog);
    run(cx, window, "palette.lost_commits");
    assert_eq!(safety_view(cx).read_with(cx, |v, _| v.tab), Tab::Lost);
    // Closed, the panel's rows say what to open first.
    run(cx, window, "palette.safety_close");
    cx.run_until_parked();
    let state = cx
        .update_window(window, |_, w, cx| commands::context::window_state(w, cx))
        .unwrap();
    assert_eq!(state.safety_tab, None);
    let rows = commands::palette::rows("Undo History: Undo Up to Here", state.palette_context());
    assert!(
        rows.iter().all(|r| r.dim),
        "unavailable until the panel is open"
    );
    let _ = ws;
}
