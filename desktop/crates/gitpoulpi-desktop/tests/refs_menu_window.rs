//! Headless tests of the ref menu (by keyboard and by mouse), the drop menu, a Log ref pill and the
//! auto-stash question.

mod refs_ops_common;

use gitpoulpi_desktop::commands::refs_ops::RefOp;
use gpui_kit::{AppContext as _, TestAppContext};
use refs_ops_common::*;

// -- the mouse --------------------------------------------------------------------------------------
#[gpui_kit::test]
fn the_context_menu_opens_from_the_keyboard_on_the_selected_row_and_runs_its_items(
    cx: &mut TestAppContext,
) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    select(cx, &ws, local("topic"));
    cx.update_window(window, |_, w, cx| {
        use gpui_kit::Focusable as _;
        let handle = ws.read(cx).refs.read(cx).focus_handle(cx);
        handle.focus(w, cx);
    })
    .unwrap();
    cx.run_until_parked();
    press(cx, window, "shift-f10");
    let menu = ws
        .read_with(cx, |ws, _| ws.ref_menu().cloned())
        .expect("Shift+F10 opens the menu");
    let labels = menu.read_with(cx, |m, _| m.labels());
    let at = |needle: &str| {
        labels
            .iter()
            .position(|l| l.contains(needle))
            .unwrap_or_else(|| panic!("{needle} in {labels:?}"))
    };
    assert!(at("Reveal") < at("Check out") && at("Check out") < at("Merge"));
    assert!(at("Rename") < at("Delete topic"));
    // Rows the desktop cannot run yet are dimmed with the reason.
    let worktree = at("worktree");
    assert!(menu.read_with(cx, |m, _| m.refusal(worktree)).is_some());
    // Escape closes it.
    press(cx, window, "escape");
    assert!(ws.read_with(cx, |ws, _| ws.ref_menu().is_none()));

    // Open again and walk to "Check out topic" with the arrow keys, Enter runs it.
    press(cx, window, "shift-f10");
    let menu = ws.read_with(cx, |ws, _| ws.ref_menu().cloned()).unwrap();
    let target = at("Check out");
    let presses = labels[..target].iter().filter(|l| !l.is_empty()).count();
    for _ in 0..presses {
        press(cx, window, "down");
    }
    assert_eq!(menu.read_with(cx, |m, _| m.selected()), target);
    press(cx, window, "enter");
    wait_until(cx, "checked out through the menu", |_| {
        head(&repo) == "topic"
    });
    assert!(
        ws.read_with(cx, |ws, _| ws.ref_menu().is_none()),
        "closes once it ran"
    );
    undo(cx, window, &ws);
    wait_until(cx, "back on main", |_| head(&repo) == "main");
}

#[gpui_kit::test]
fn the_copy_submenu_puts_the_name_the_full_ref_and_the_sha_on_the_clipboard(
    cx: &mut TestAppContext,
) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    let read = |cx: &mut TestAppContext| {
        cx.update(|cx| cx.read_from_clipboard().and_then(|c| c.text()))
            .unwrap_or_default()
    };
    use gitpoulpi_app::palette::vocab::ref_menu::RefField;
    run_op(
        cx,
        window,
        &ws,
        RefOp::Copy(RefField::Name),
        Some(local("topic")),
    );
    assert_eq!(read(cx), "topic");
    run_op(
        cx,
        window,
        &ws,
        RefOp::Copy(RefField::FullRef),
        Some(local("topic")),
    );
    assert_eq!(read(cx), "refs/heads/topic");
    run_op(
        cx,
        window,
        &ws,
        RefOp::Copy(RefField::TargetSha),
        Some(local("topic")),
    );
    assert_eq!(read(cx), repo.git(&["rev-parse", "topic"]).trim());
}

#[gpui_kit::test]
fn dropping_a_branch_on_the_checked_out_one_offers_merge_rebase_and_fast_forward(
    cx: &mut TestAppContext,
) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    // The keyboard twin of the drag: the palette command opens the same menu.
    select(cx, &ws, local("topic"));
    run(cx, window, "palette.drop_selected_ref_on_current");
    let menu = ws
        .read_with(cx, |ws, _| ws.ref_menu().cloned())
        .expect("the drop menu");
    let labels = menu.read_with(cx, |m, _| m.labels());
    assert_eq!(labels.len(), 3, "{labels:?}");
    assert!(
        labels[0].contains("Merge")
            && labels[1].contains("Rebase")
            && labels[2].contains("Fast-forward")
    );
    // Activate the Merge row.
    cx.update_window(window, |_, _, cx| {
        menu.update(cx, |m, cx| m.activate(0, cx))
    })
    .unwrap();
    cx.run_until_parked();
    wait_until(cx, "merged by the drop", |_| {
        is_ancestor(&repo, "topic", "main")
    });
    undo(cx, window, &ws);
    wait_until(cx, "drop undone", |_| !is_ancestor(&repo, "topic", "main"));
}

/// The menu key in the Log: the commit menu, or the ref menu once a ref pill of the commit was clicked
/// (the keyboard twin of the pill's right click). The Refs sidebar's own rows have the ref menu.
#[gpui_kit::test]
fn the_menu_key_opens_the_menu_of_the_focused_pane_and_of_a_picked_pill(cx: &mut TestAppContext) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    let main_oid = wait_for_oid(cx, &ws, "main");
    ws.update(cx, |ws, cx| {
        ws.log.update(cx, |l, cx| l.select_oid(main_oid, cx))
    });
    run(cx, window, "palette.focus_log");
    // Nothing picked: the commit menu (the Log's own), not the ref menu.
    press(cx, window, "shift-f10");
    assert!(ws.read_with(cx, |ws, _| ws.ref_menu().is_none()));
    let commit_menu = ws.read_with(cx, |ws, cx| ws.log.read(cx).menu().cloned());
    assert!(commit_menu.is_some(), "the commit menu opened");
    press(cx, window, "escape");
    assert!(ws.read_with(cx, |ws, cx| ws.log.read(cx).menu().is_none()));

    // A picked pill: the ref menu of that ref.
    ws.update(cx, |ws, cx| ws.pick_log_pill(main_oid, local("main"), cx));
    press(cx, window, "shift-f10");
    let menu = ws
        .read_with(cx, |ws, _| ws.ref_menu().cloned())
        .expect("the ref menu opened for the picked pill");
    assert!(menu
        .read_with(cx, |m, _| m.labels())
        .iter()
        .any(|l| l.contains("main")));
    press(cx, window, "escape");
    // Another row drops the pick: the commit menu again.
    let topic_oid = wait_for_oid(cx, &ws, "topic");
    ws.update(cx, |ws, cx| {
        ws.log.update(cx, |l, cx| l.select_oid(topic_oid, cx))
    });
    press(cx, window, "shift-f10");
    assert!(ws.read_with(cx, |ws, _| ws.ref_menu().is_none()));
    assert!(ws.read_with(cx, |ws, cx| ws.log.read(cx).menu().is_some()));
}

fn wait_for_oid(
    cx: &mut TestAppContext,
    ws: &gpui_kit::Entity<gitpoulpi_desktop::app::Workspace>,
    branch: &str,
) -> gitpoulpi_core::domain::Oid {
    let mut found = None;
    wait_until(cx, "the branch in the snapshot", |cx| {
        found = ws.read_with(cx, |ws, cx| {
            ws.refs.read(cx).snapshot().oid_of(&local(branch))
        });
        found.is_some()
    });
    found.unwrap()
}

/// The commit menu's "create branch here" and "check out" rows are the one implementation of those ids:
/// enabled, and the checkout asks the shared auto-stash question when local changes are in the way.
#[gpui_kit::test]
fn the_commit_menu_creates_a_branch_here_and_checks_out_through_the_shared_autostash_question(
    cx: &mut TestAppContext,
) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    let topic_oid = wait_for_oid(cx, &ws, "topic");
    ws.update(cx, |ws, cx| {
        ws.log.update(cx, |l, cx| l.select_oid(topic_oid, cx))
    });
    run(cx, window, "palette.focus_log");
    run(cx, window, "palette.create_branch_at_selected_commit");
    type_into(cx, window, 0, "from-topic");
    form(cx).update(cx, |f, cx| f.set_check(1, false, cx));
    submit(cx, window);
    wait_until(cx, "the branch at the commit", |_| {
        has_branch(&repo, "from-topic")
            && repo.git(&["rev-parse", "from-topic"]) == repo.git(&["rev-parse", "topic"])
    });
    undo(cx, window, &ws);
    wait_until(cx, "the branch undone", |_| {
        !has_branch(&repo, "from-topic")
    });

    // A local edit of a file topic changed blocks the detached checkout: the same question as a ref's.
    repo.git(&["checkout", "-q", "topic"]);
    std::fs::write(repo.root().join("a.txt"), "topic's a\n").unwrap();
    repo.git(&["commit", "-qam", "topic edits a"]);
    let topic_tip = repo.git(&["rev-parse", "topic"]).trim().to_string();
    repo.git(&["checkout", "-q", "main"]);
    std::fs::write(repo.root().join("a.txt"), "local edit\n").unwrap();
    wait_until(cx, "the window to see the new tip", |cx| {
        ws.read_with(cx, |ws, cx| {
            ws.refs
                .read(cx)
                .snapshot()
                .oid_of(&local("topic"))
                .map(|o| o.to_hex())
                == Some(topic_tip.clone())
        })
    });
    let tip = wait_for_oid(cx, &ws, "topic");
    ws.update(cx, |ws, cx| {
        ws.log.update(cx, |l, cx| l.select_oid(tip, cx))
    });
    run(cx, window, "palette.checkout_selected_commit");
    wait_until(cx, "the auto-stash question", |cx| dialog_open(cx, window));
    answer(cx, window, 1);
    assert_eq!(head(&repo), "main", "Cancel changes nothing");
    run(cx, window, "palette.checkout_selected_commit");
    wait_until(cx, "the question again", |cx| dialog_open(cx, window));
    answer(cx, window, 0);
    wait_until(cx, "detached at the commit", |_| {
        head(&repo).is_empty() && repo.git(&["rev-parse", "HEAD"]).trim() == topic_tip
    });
}

// -- auto-stash -------------------------------------------------------------------------------------

#[gpui_kit::test]
fn a_checkout_blocked_by_local_changes_asks_to_stash_and_continue(cx: &mut TestAppContext) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    // topic changes a.txt; a local edit of it on main blocks the checkout.
    repo.git(&["checkout", "-q", "topic"]);
    std::fs::write(repo.root().join("a.txt"), "topic's a\n").unwrap();
    repo.git(&["commit", "-qam", "topic edits a"]);
    repo.git(&["checkout", "-q", "main"]);
    std::fs::write(repo.root().join("a.txt"), "local edit\n").unwrap();
    wait_until(cx, "the window to see the edit", |cx| {
        ws.read_with(cx, |ws, cx| {
            ws.refs.read(cx).snapshot().head().map(|h| h.0.to_string()) == Some("main".into())
        })
    });
    select(cx, &ws, local("topic"));
    run(cx, window, "palette.checkout_selected_ref");
    wait_until(cx, "the auto-stash question", |cx| dialog_open(cx, window));
    let title = question_title(cx);
    assert!(title.to_lowercase().contains("local changes"), "{title}");
    // Cancel changes nothing.
    answer(cx, window, 1);
    cx.run_until_parked();
    assert_eq!(head(&repo), "main");
    run(cx, window, "palette.checkout_selected_ref");
    wait_until(cx, "the question again", |cx| dialog_open(cx, window));
    answer(cx, window, 0);
    wait_until(cx, "checked out after stashing", |_| head(&repo) == "topic");
    // The stash is popped again once the checkout is done: the edit is back (or conflicts with topic's).
    wait_until(cx, "the stash handled", |_| true);
}

#[gpui_kit::test]
fn a_right_click_opens_the_menu_a_double_click_checks_out_and_a_drag_opens_the_drop_menu(
    cx: &mut TestAppContext,
) {
    use gpui_kit::{MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent};
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    frames(cx, window, 3);
    let none = gpui_kit::Modifiers::default();
    let main_at = row_point(cx, window, &ws, local("main"));
    let topic_at = row_point(cx, window, &ws, local("topic"));

    // Right click on `topic`: its menu, with the same first rows as the keyboard's.
    sim(
        cx,
        window,
        MouseDownEvent {
            position: topic_at,
            modifiers: none,
            button: MouseButton::Right,
            click_count: 1,
            first_mouse: false,
        },
    );
    cx.run_until_parked();
    let menu = ws
        .read_with(cx, |ws, _| ws.ref_menu().cloned())
        .expect("a right click opens the menu");
    let labels = menu.read_with(cx, |m, _| m.labels());
    assert!(
        labels[0].contains("Reveal") && labels.iter().any(|l| l.contains("Check out topic")),
        "{labels:?}"
    );
    press(cx, window, "escape");
    assert!(ws.read_with(cx, |ws, _| ws.ref_menu().is_none()));

    // Double click: checks the branch out.
    for count in 1..=2 {
        sim(
            cx,
            window,
            MouseDownEvent {
                position: topic_at,
                modifiers: none,
                button: MouseButton::Left,
                click_count: count,
                first_mouse: false,
            },
        );
        sim(
            cx,
            window,
            MouseUpEvent {
                position: topic_at,
                modifiers: none,
                button: MouseButton::Left,
                click_count: count,
            },
        );
    }
    cx.run_until_parked();
    wait_until(cx, "topic checked out by a double click", |_| {
        head(&repo) == "topic"
    });
    undo(cx, window, &ws);
    wait_until(cx, "back on main", |_| head(&repo) == "main");

    // Drag topic onto main: the drop menu.
    sim(
        cx,
        window,
        MouseDownEvent {
            position: topic_at,
            modifiers: none,
            button: MouseButton::Left,
            click_count: 1,
            first_mouse: false,
        },
    );
    for p in [
        gpui_kit::Point::new(topic_at.x + gpui_kit::px(4.0), topic_at.y),
        main_at,
    ] {
        sim(
            cx,
            window,
            MouseMoveEvent {
                position: p,
                modifiers: none,
                pressed_button: Some(MouseButton::Left),
            },
        );
    }
    sim(
        cx,
        window,
        MouseUpEvent {
            position: main_at,
            modifiers: none,
            button: MouseButton::Left,
            click_count: 1,
        },
    );
    cx.run_until_parked();
    let menu = ws
        .read_with(cx, |ws, _| ws.ref_menu().cloned())
        .expect("a drop opens the menu");
    let labels = menu.read_with(cx, |m, _| m.labels());
    assert_eq!(labels.len(), 3, "{labels:?}");
}
