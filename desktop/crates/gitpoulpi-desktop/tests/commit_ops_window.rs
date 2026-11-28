//! Headless tests of the Log's commit operations on the real main window: the commit context menu (mouse
//! and keyboard), cherry-pick (one and several), revert, reset, detached checkout, copying fields, the
//! comparison of several selected rows and the commit details pane (the operation banner is in
//! `operation_banner_window`). Fixture repositories and a temporary config directory; no network, no visible window.

mod commit_ops_common;

use commit_ops_common::*;
use gitpoulpi_core::domain::Oid;
use gitpoulpi_desktop::commands;
use gitpoulpi_desktop::dialogs;
use gitpoulpi_i18n::msg;
use gpui_kit::{AppContext as _, Modifiers, TestAppContext};

// ---- the menu ---------------------------------------------------------------------------------------

#[gpui_kit::test]
fn the_menu_key_opens_the_commit_menu_with_the_terminal_uis_rows_and_the_keyboard_runs_one(
    cx: &mut TestAppContext,
) {
    let repo = repo_with_side();
    let (window, ws) = open_log(cx, &repo, 4);
    let target = oid(&repo, "HEAD~1");
    select(cx, &ws, target);
    let head = repo.git(&["branch", "--show-current"]).trim().to_string();

    press(cx, window, "shift-f10");
    let labels = menu_labels(cx, &ws).expect("Shift+F10 opened the menu");
    assert_eq!(labels[0], msg::menu_log::cherry_pick_onto(&head));
    assert_eq!(labels[1], msg::menu_log::REVERT.get());
    assert_eq!(labels[2], msg::menu_log::reset_to(&head));
    let at = |l: &str| labels.iter().position(|x| x == l).unwrap();
    assert!(at(msg::menu_log::CREATE_BRANCH_HERE.get()) < at(msg::menu_log::CHECKOUT_COMMIT.get()));
    assert_eq!(labels.last().unwrap(), msg::menu_ref::COPY.get());

    let focused = {
        let menu = ws.read_with(cx, |ws, cx| ws.log.read(cx).menu().unwrap().clone());
        cx.update_window(window, |_, w, cx| {
            gpui_kit::Focusable::focus_handle(menu.read(cx), cx).is_focused(w)
        })
        .unwrap()
    };
    assert!(focused, "the menu has the keyboard");
    // Esc closes it; the menu key again, then Down to "Check out this commit" and Enter.
    press(cx, window, "escape");
    assert!(menu_labels(cx, &ws).is_none(), "Esc closed the menu");
    press(cx, window, ".");
    assert!(menu_labels(cx, &ws).is_some(), "the dot opens it too");
    let steps = at(msg::menu_log::CHECKOUT_COMMIT.get());
    // Rows are listed without their rules: Down walks them in order, so the index is the step count.
    for _ in 0..steps {
        press(cx, window, "down");
    }
    press(cx, window, "enter");
    assert!(
        menu_labels(cx, &ws).is_none(),
        "running a row closes the menu"
    );
    wait_until(cx, "HEAD detached on the commit", |_| {
        oid_of_head(&repo) == target
    });
    assert!(repo.git(&["branch", "--show-current"]).trim().is_empty());
    undo(cx, window, &ws);
    wait_until(cx, "the branch back", |_| {
        repo.git(&["branch", "--show-current"]).trim() == head
    });
}

#[gpui_kit::test]
fn a_menu_row_that_cannot_run_stays_listed_with_its_reason_and_the_menu_closes_by_its_control(
    cx: &mut TestAppContext,
) {
    let repo = repo_with_side();
    let (window, ws) = open_log(cx, &repo, 4);
    select(cx, &ws, oid(&repo, "HEAD"));
    press(cx, window, "shift-f10");
    // "Create branch here" belongs to the refs commands, which this window does not run yet.
    let (disabled, current) = ws.read_with(cx, |ws, cx| {
        let menu = ws.log.read(cx).menu().unwrap().read(cx);
        let row = menu
            .items()
            .iter()
            .find(|i| i.label == msg::menu_log::EDIT_MESSAGE.get())
            .cloned()
            .unwrap();
        (row.disabled, menu.depth())
    });
    assert_eq!(current, 0);
    assert!(
        disabled.is_some(),
        "a row the desktop does not run yet says why"
    );
    let menu = ws.read_with(cx, |ws, cx| ws.log.read(cx).menu().unwrap().clone());
    cx.update_window(window, |_, w, cx| menu.update(cx, |m, cx| m.close(w, cx)))
        .unwrap();
    cx.run_until_parked();
    assert!(menu_labels(cx, &ws).is_none());
}

// ---- checkout, cherry-pick, revert, reset -----------------------------------------------------------

#[gpui_kit::test]
fn cherry_pick_applies_one_commit_and_one_undo_takes_it_back(cx: &mut TestAppContext) {
    let repo = repo_with_side();
    let (window, ws) = open_log(cx, &repo, 5);
    let before = oid_of_head(&repo);
    select(cx, &ws, oid(&repo, "side~1"));
    run(cx, window, "git_prefix.cherry_pick");
    wait_until(cx, "the commit applied", |_| {
        subjects(&repo)[0] == "side one"
    });
    assert_eq!(subjects(&repo).len(), 4);
    undo(cx, window, &ws);
    wait_until(cx, "HEAD back", |_| oid_of_head(&repo) == before);
}

#[gpui_kit::test]
fn cherry_picking_several_commits_applies_them_oldest_first_as_one_undo_entry(
    cx: &mut TestAppContext,
) {
    let repo = repo_with_side();
    let (window, ws) = open_log(cx, &repo, 5);
    let before = oid_of_head(&repo);
    // Newest first in the clicks: the order of selection must not matter.
    click_row(cx, &ws, window, oid(&repo, "side"), NONE);
    click_row(cx, &ws, window, oid(&repo, "side~1"), CTRL);
    assert_eq!(ws.read_with(cx, |ws, cx| ws.log.read(cx).marked_count()), 2);
    run(cx, window, "git_prefix.cherry_pick");
    wait_until(cx, "both applied", |_| {
        subjects(&repo)[..2] == ["side two", "side one"]
    });
    undo(cx, window, &ws);
    wait_until(cx, "one undo took both back", |_| {
        oid_of_head(&repo) == before
    });
}

#[gpui_kit::test]
fn revert_asks_first_and_undo_takes_the_revert_back(cx: &mut TestAppContext) {
    let repo = repo_with_side();
    let (window, ws) = open_log(cx, &repo, 4);
    let before = oid_of_head(&repo);
    select(cx, &ws, before);
    run(cx, window, "git_prefix.revert");
    assert!(dialog_open(cx, window), "a revert asks first");
    answer(cx, window, 1);
    assert_eq!(oid_of_head(&repo), before, "Cancel reverts nothing");

    run(cx, window, "git_prefix.revert");
    answer(cx, window, 0);
    wait_until(cx, "the revert commit", |_| {
        subjects(&repo)[0].starts_with("Revert")
    });
    undo(cx, window, &ws);
    wait_until(cx, "the revert undone", |_| oid_of_head(&repo) == before);
}

#[gpui_kit::test]
fn soft_and_mixed_resets_move_the_branch_without_asking_and_undo(cx: &mut TestAppContext) {
    let repo = repo_with_side();
    let (window, ws) = open_log(cx, &repo, 4);
    let tip = oid_of_head(&repo);
    let target = oid(&repo, "HEAD~1");
    select(cx, &ws, target);
    run(cx, window, "palette.reset_to_selected_commit_soft");
    assert!(!dialog_open(cx, window), "a soft reset never asks");
    wait_until(cx, "the branch moved", |_| oid_of_head(&repo) == target);
    assert!(
        repo.git(&["diff", "--cached", "--name-only"])
            .contains("b.txt"),
        "soft keeps the changes staged"
    );
    undo(cx, window, &ws);
    wait_until(cx, "the branch back", |_| oid_of_head(&repo) == tip);

    select(cx, &ws, target);
    run(cx, window, "git_prefix.reset_mixed");
    assert!(!dialog_open(cx, window));
    wait_until(cx, "the branch moved", |_| oid_of_head(&repo) == target);
    assert!(
        repo.git(&["diff", "--cached", "--name-only"])
            .trim()
            .is_empty(),
        "mixed unstages"
    );
    assert!(repo.root().join("b.txt").exists(), "mixed keeps the files");
}

#[gpui_kit::test]
fn a_hard_reset_asks_first_and_cancel_changes_nothing(cx: &mut TestAppContext) {
    let repo = repo_with_side();
    let (window, ws) = open_log(cx, &repo, 4);
    let tip = oid_of_head(&repo);
    let target = oid(&repo, "HEAD~1");
    select(cx, &ws, target);
    run(cx, window, "palette.reset_to_selected_commit_hard");
    assert!(dialog_open(cx, window), "reset --hard asks first");
    let title = cx
        .update(|cx| dialogs::current(cx).map(|d| d.read(cx).spec().title.clone()))
        .unwrap();
    assert!(title.contains("--hard"), "{title}");
    answer(cx, window, 1);
    assert_eq!(oid_of_head(&repo), tip, "Cancel moved nothing");

    run(cx, window, "palette.reset_to_selected_commit_hard");
    answer(cx, window, 0);
    wait_until(cx, "the branch and the files moved", |_| {
        oid_of_head(&repo) == target && !repo.root().join("b.txt").exists()
    });
    // The hard reset is journaled: one Undo puts the branch and the files back.
    undo(cx, window, &ws);
    wait_until(cx, "the hard reset undone", |_| {
        oid_of_head(&repo) == tip && repo.root().join("b.txt").exists()
    });
}

// ---- copy -------------------------------------------------------------------------------------------

#[gpui_kit::test]
fn copying_a_field_puts_it_on_the_clipboard_and_says_so(cx: &mut TestAppContext) {
    let repo = repo_with_side();
    let (window, ws) = open_log(cx, &repo, 4);
    let head = oid_of_head(&repo);
    select(cx, &ws, head);
    for (id, want) in [
        (
            "palette.copy_commit_field_short_sha",
            head.to_hex()[..7].to_string(),
        ),
        ("palette.copy_commit_field_long_sha", head.to_hex()),
        ("palette.copy_commit_field_message", "add b".to_string()),
        (
            "palette.copy_commit_field_author_name",
            repo.git(&["log", "-1", "--format=%an"]).trim().to_string(),
        ),
    ] {
        run(cx, window, id);
        let got = cx.read_from_clipboard().and_then(|c| c.text());
        assert_eq!(got.as_deref(), Some(want.as_str()), "{id}");
        assert!(notice(cx, &ws).contains("copied"), "{}", notice(cx, &ws));
    }
    run(cx, window, "palette.copy_commit_field_date");
    let date = cx.read_from_clipboard().and_then(|c| c.text()).unwrap();
    assert!(date.contains(':'), "{date}");
}

// ---- several rows and the comparison ----------------------------------------------------------------

#[gpui_kit::test]
fn ctrl_click_and_shift_click_select_rows_and_the_right_pane_compares_them(
    cx: &mut TestAppContext,
) {
    let repo = repo_with_side();
    let (window, ws) = open_log(cx, &repo, 5);
    let (a, b) = (oid(&repo, "side"), oid(&repo, "side~1"));
    click_row(cx, &ws, window, a, NONE);
    click_row(cx, &ws, window, b, CTRL);
    wait_until(cx, "the comparison files", |cx| {
        ws.read_with(cx, |ws, cx| {
            ws.right.read(cx).compare_spec().is_some()
                && !ws.right.read(cx).commit_file_paths().is_empty()
        })
    });
    let spec = ws
        .read_with(cx, |ws, cx| ws.right.read(cx).compare_spec().cloned())
        .unwrap();
    assert_eq!(spec.rows, 2);
    assert!(spec.can_swap);
    let files = ws.read_with(cx, |ws, cx| ws.right.read(cx).commit_file_paths());
    assert_eq!(
        files,
        vec!["s2.txt"],
        "older against newer: only side two differs"
    );

    // x swaps the two ends; n goes to the next compared row; Esc goes back to one commit.
    press(cx, window, "x");
    assert!(ws
        .read_with(cx, |ws, cx| ws
            .right
            .read(cx)
            .compare_spec()
            .map(|s| s.swapped))
        .unwrap());
    let before = ws.read_with(cx, |ws, cx| ws.log.read(cx).selected_row());
    press(cx, window, "n");
    assert_ne!(
        ws.read_with(cx, |ws, cx| ws.log.read(cx).selected_row()),
        before
    );
    press(cx, window, "escape");
    wait_until(cx, "back to one commit", |cx| {
        ws.read_with(cx, |ws, cx| ws.right.read(cx).shows_commit())
    });
    assert_eq!(ws.read_with(cx, |ws, cx| ws.log.read(cx).marked_count()), 0);

    // Shift+click takes the range between the anchor and the row.
    click_row(cx, &ws, window, oid(&repo, "side"), NONE);
    click_row(
        cx,
        &ws,
        window,
        oid(&repo, "HEAD"),
        Modifiers {
            shift: true,
            ..NONE
        },
    );
    assert!(ws.read_with(cx, |ws, cx| ws.log.read(cx).marked_count()) >= 3);
}

// ---- the details pane -------------------------------------------------------------------------------

#[gpui_kit::test]
fn the_details_pane_shows_parents_refs_and_files_and_a_parent_click_jumps_to_it(
    cx: &mut TestAppContext,
) {
    let repo = repo_with_side();
    let (window, ws) = open_log(cx, &repo, 5);
    let tip = oid(&repo, "side");
    select(cx, &ws, tip);
    wait_until(cx, "the files of the commit", |cx| {
        ws.read_with(cx, |ws, cx| {
            !ws.right.read(cx).commit_file_paths().is_empty()
        })
    });
    assert_eq!(
        ws.read_with(cx, |ws, cx| ws.right.read(cx).commit_file_paths()),
        vec!["s2.txt"]
    );
    assert_eq!(
        ws.read_with(cx, |ws, cx| ws.right.read(cx).shown_parents()),
        vec![oid(&repo, "side~1")]
    );
    assert_eq!(
        ws.read_with(cx, |ws, cx| ws.right.read(cx).shown_ref_names()),
        vec!["side".to_string()]
    );

    // A click on the parent selects it in the Log.
    ws.update(cx, |ws, cx| {
        ws.right.update(cx, |r, cx| r.jump_to_parent(0, cx))
    });
    cx.run_until_parked();
    assert_eq!(
        ws.read_with(cx, |ws, cx| ws.log.read(cx).selected_commit()),
        Some(oid(&repo, "side~1"))
    );

    // Down and Enter in the files list open the file's diff in the centre.
    wait_until(cx, "the files of the parent", |cx| {
        ws.read_with(cx, |ws, cx| {
            !ws.right.read(cx).commit_file_paths().is_empty()
        })
    });
    run(cx, window, "palette.focus_status");
    press(cx, window, "down enter");
    wait_until(cx, "the diff of the commit's file", |cx| {
        ws.read_with(cx, |ws, _| ws.diff_is_open())
    });
}

#[gpui_kit::test]
fn a_right_click_selects_its_row_unless_it_is_inside_the_selection(cx: &mut TestAppContext) {
    let repo = repo_with_side();
    let (window, ws) = open_log(cx, &repo, 5);
    let (a, b, c) = (oid(&repo, "side"), oid(&repo, "side~1"), oid(&repo, "HEAD"));
    let log = ws.read_with(cx, |ws, _| ws.log.clone());
    let right_click = |cx: &mut TestAppContext, oid: Oid| {
        cx.update_window(window, |_, w, cx| {
            let palette = commands::context::window_state(w, cx).palette_context();
            log.update(cx, |l, cx| {
                let ix = l.row_of(oid).unwrap();
                let at = gpui_kit::point(gpui_kit::px(300.0), gpui_kit::px(200.0));
                l.open_menu_on(ix, at, palette, w, cx)
            })
        })
        .unwrap();
        cx.run_until_parked();
    };
    click_row(cx, &ws, window, a, NONE);
    right_click(cx, c);
    assert_eq!(
        ws.read_with(cx, |ws, cx| ws.log.read(cx).selected_commit()),
        Some(c),
        "a right-click outside the selection selects that row"
    );
    assert!(menu_labels(cx, &ws).is_some(), "the menu opened");
    press(cx, window, "escape");

    click_row(cx, &ws, window, a, NONE);
    click_row(cx, &ws, window, b, CTRL);
    right_click(cx, a);
    let labels = menu_labels(cx, &ws).unwrap();
    assert_eq!(
        labels.len(),
        3,
        "several commits: the multi-commit rows only: {labels:?}"
    );
    assert!(labels[0].contains('2'), "{labels:?}");
    assert_eq!(ws.read_with(cx, |ws, cx| ws.log.read(cx).marked_count()), 2);
}
