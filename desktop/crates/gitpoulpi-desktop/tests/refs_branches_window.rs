//! Headless tests of branches on the real main window: the forms and questions that come first, name
//! validation, the safety confirmations, merge, rebase and fast-forward, each against a fixture repository
//! with a local bare repository as its remote, with Undo after every mutation.

mod refs_ops_common;

use gitpoulpi_core::domain::OperationState;
use gitpoulpi_desktop::app::BannerButton;
use gitpoulpi_desktop::commands::refs_ops::RefOp;
use gitpoulpi_desktop::dialogs;
use gitpoulpi_desktop::ref_target::RefTarget;
use gitpoulpi_i18n::msg;
use gpui_kit::{AppContext as _, TestAppContext};
use refs_ops_common::*;

// -- branches ---------------------------------------------------------------------------------------

#[gpui_kit::test]
fn a_new_branch_is_created_at_head_after_its_name_passes_the_checks_and_undo_removes_it(
    cx: &mut TestAppContext,
) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    run(cx, window, "git_prefix.new_branch");
    assert!(dialog_open(cx, window), "the form is open");
    // Refused names keep the form open with the reason; nothing is created.
    for bad in ["has space", "main", "x..y", ""] {
        type_into(cx, window, 0, bad);
        submit(cx, window);
        assert!(dialog_open(cx, window), "{bad:?} keeps the form open");
        assert!(form_error(cx).is_some(), "{bad:?} says why");
    }
    assert_eq!(branches(&repo).len(), 2, "main and topic only");
    type_into(cx, window, 0, "feature/new");
    form(cx).update(cx, |f, cx| f.set_check(1, false, cx));
    submit(cx, window);
    assert!(!dialog_open(cx, window));
    wait_until(cx, "the branch", |_| has_branch(&repo, "feature/new"));
    assert_eq!(head(&repo), "main", "not checked out when the box is off");
    assert_eq!(
        repo.git(&["rev-parse", "feature/new"]),
        repo.git(&["rev-parse", "main"]),
        "created at HEAD"
    );
    undo(cx, window, &ws);
    wait_until(cx, "the branch gone", |_| !has_branch(&repo, "feature/new"));
}

#[gpui_kit::test]
fn the_check_box_checks_the_new_branch_out(cx: &mut TestAppContext) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    run(cx, window, "git_prefix.new_branch");
    type_into(cx, window, 0, "go");
    let checked = form(cx).read_with(cx, |f, cx| f.values(cx).checks[1]);
    assert!(checked, "checked out by default, as the terminal UI does");
    submit(cx, window);
    wait_until(cx, "HEAD on the new branch", |_| head(&repo) == "go");
    undo(cx, window, &ws);
    wait_until(cx, "undone", |_| {
        head(&repo) == "main" || !has_branch(&repo, "go")
    });
}

#[gpui_kit::test]
fn a_double_click_checks_a_branch_out_and_a_remote_branch_goes_through_a_tracking_branch(
    cx: &mut TestAppContext,
) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    // Checkout of a local branch through the palette command on the selection.
    select(cx, &ws, local("topic"));
    run(cx, window, "palette.checkout_selected_ref");
    wait_until(cx, "topic checked out", |_| head(&repo) == "topic");
    undo(cx, window, &ws);
    wait_until(cx, "back on main", |_| head(&repo) == "main");

    // A remote branch no local branch tracks: offer to create one, never a detached checkout.
    run_op(
        cx,
        window,
        &ws,
        RefOp::Checkout,
        Some(RefTarget::Remote("origin/remote-only".into())),
    );
    assert!(dialog_open(cx, window), "asks before creating the branch");
    assert!(question_title(cx).contains("remote-only"));
    answer(cx, window, 1);
    assert!(!has_branch(&repo, "remote-only"), "cancel creates nothing");
    run_op(
        cx,
        window,
        &ws,
        RefOp::Checkout,
        Some(RefTarget::Remote("origin/remote-only".into())),
    );
    answer(cx, window, 0);
    wait_until(cx, "the tracking branch", |_| head(&repo) == "remote-only");
    assert_eq!(
        repo.git(&["config", "branch.remote-only.remote"]).trim(),
        "origin"
    );
    undo(cx, window, &ws);
    wait_until(cx, "back on main", |_| head(&repo) == "main");

    // A remote branch whose local name is taken by an unrelated branch says so and does nothing.
    run_op(
        cx,
        window,
        &ws,
        RefOp::Checkout,
        Some(RefTarget::Remote("origin/topic".into())),
    );
    assert!(!dialog_open(cx, window));
    assert!(notice(cx, &ws).contains("topic"), "{}", notice(cx, &ws));
    assert_eq!(head(&repo), "main");
}

#[gpui_kit::test]
fn a_tag_checks_out_detached(cx: &mut TestAppContext) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    run_op(
        cx,
        window,
        &ws,
        RefOp::Checkout,
        Some(RefTarget::Tag("v1".into())),
    );
    wait_until(cx, "detached", |_| head(&repo).is_empty());
    undo(cx, window, &ws);
    wait_until(cx, "on main again", |_| head(&repo) == "main");
}

#[gpui_kit::test]
fn renaming_validates_the_name_and_undo_restores_it(cx: &mut TestAppContext) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    select(cx, &ws, local("topic"));
    run(cx, window, "palette.rename_selected_branch");
    assert!(dialog_open(cx, window));
    let prefilled = form(cx).read_with(cx, |f, cx| f.values(cx).texts[0].clone());
    assert_eq!(prefilled, "topic", "the form starts on the current name");
    type_into(cx, window, 0, "main");
    submit(cx, window);
    assert!(form_error(cx).is_some(), "a taken name is refused");
    type_into(cx, window, 0, "renamed");
    submit(cx, window);
    wait_until(cx, "renamed", |_| {
        has_branch(&repo, "renamed") && !has_branch(&repo, "topic")
    });
    undo(cx, window, &ws);
    wait_until(cx, "renamed back", |_| has_branch(&repo, "topic"));
}

#[gpui_kit::test]
fn deleting_a_branch_asks_first_refuses_the_checked_out_one_and_undo_brings_it_back(
    cx: &mut TestAppContext,
) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    // The checked-out branch is refused with a message, not a dialog.
    select(cx, &ws, local("main"));
    run(cx, window, "git_prefix.delete_ref");
    assert!(!dialog_open(cx, window));
    assert!(
        notice(cx, &ws).contains("checked out"),
        "{}",
        notice(cx, &ws)
    );
    assert!(has_branch(&repo, "main"));

    select(cx, &ws, local("topic"));
    run(cx, window, "git_prefix.delete_ref");
    assert!(dialog_open(cx, window), "deleting asks");
    assert!(question_title(cx).contains("topic"));
    // Nothing is highlighted: a stray Enter answers nothing, and Cancel keeps the branch.
    let highlighted = cx
        .update(|cx| dialogs::current(cx))
        .unwrap()
        .read_with(cx, |q, _| q.spec().highlight_first);
    assert!(!highlighted, "no default destructive answer");
    answer(cx, window, 1);
    assert!(has_branch(&repo, "topic"));
    run(cx, window, "git_prefix.delete_ref");
    answer(cx, window, 0);
    wait_until(cx, "topic deleted", |_| !has_branch(&repo, "topic"));
    undo(cx, window, &ws);
    wait_until(cx, "topic back", |_| has_branch(&repo, "topic"));
}

#[gpui_kit::test]
fn the_remote_branch_can_be_deleted_after_a_confirmation_naming_the_remote(
    cx: &mut TestAppContext,
) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    select(cx, &ws, local("topic"));
    run(cx, window, "palette.delete_selected_remote_branch");
    assert!(dialog_open(cx, window));
    let title = question_title(cx);
    assert!(
        title.contains("origin") && title.contains("topic"),
        "{title}"
    );
    answer(cx, window, 1);
    assert!(remote_has(&repo, "refs/heads/topic"), "cancel keeps it");
    run(cx, window, "palette.delete_selected_remote_branch");
    answer(cx, window, 0);
    wait_until(cx, "gone from the remote", |_| {
        !remote_has(&repo, "refs/heads/topic")
    });
    assert!(has_branch(&repo, "topic"), "the local branch stays");
}

#[gpui_kit::test]
fn the_upstream_is_set_and_unset_from_a_form_that_checks_the_remote_branch(
    cx: &mut TestAppContext,
) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    select(cx, &ws, local("topic"));
    run(cx, window, "palette.set_upstream");
    assert!(dialog_open(cx, window));
    type_into(cx, window, 0, "origin/nothing");
    submit(cx, window);
    assert!(
        form_error(cx).is_some(),
        "an unknown remote branch is refused"
    );
    type_into(cx, window, 0, "origin/topic");
    submit(cx, window);
    wait_until(cx, "upstream set", |_| {
        repo.try_git(&["config", "branch.topic.remote"]).is_some()
    });
    undo(cx, window, &ws);
    wait_until(cx, "upstream undone", |_| {
        repo.try_git(&["config", "branch.topic.remote"]).is_none()
    });
    // And unset it for real: set again, then empty field.
    run(cx, window, "palette.set_upstream");
    type_into(cx, window, 0, "origin/topic");
    submit(cx, window);
    wait_until(cx, "upstream set again", |_| {
        repo.try_git(&["config", "branch.topic.remote"]).is_some()
    });
    wait_until(cx, "the form to be refreshed", |cx| {
        ws.read_with(cx, |ws, cx| {
            ws.refs
                .read(cx)
                .snapshot()
                .refs
                .iter()
                .any(|r| r.upstream.as_deref() == Some("origin/topic"))
        })
    });
    run(cx, window, "palette.set_upstream");
    type_into(cx, window, 0, "");
    submit(cx, window);
    wait_until(cx, "upstream removed", |_| {
        repo.try_git(&["config", "branch.topic.remote"]).is_none()
    });
}

#[gpui_kit::test]
fn merge_rebase_and_fast_forward_act_on_the_selected_branch_and_each_undoes_as_one_entry(
    cx: &mut TestAppContext,
) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    let main_before = repo.git(&["rev-parse", "main"]);
    let topic = repo.git(&["rev-parse", "topic"]);

    // Merge topic into main (a fast-forward here).
    select(cx, &ws, local("topic"));
    run(cx, window, "palette.merge_selected_ref");
    wait_until(cx, "merged", |_| is_ancestor(&repo, "topic", "main"));
    undo(cx, window, &ws);
    wait_until(cx, "merge undone", |_| {
        repo.git(&["rev-parse", "main"]) == main_before
    });

    // Fast-forward the checked-out branch to the selected one.
    run(cx, window, "palette.fast_forward_current_to_selected");
    wait_until(cx, "fast-forwarded", |_| {
        repo.git(&["rev-parse", "main"]) == topic
    });
    undo(cx, window, &ws);
    wait_until(cx, "fast-forward undone", |_| {
        repo.git(&["rev-parse", "main"]) == main_before
    });

    // Fast-forward the selected branch to the checked-out one: topic is ahead, so it is refused by the
    // daemon or said before sending; either way nothing moves.
    run(cx, window, "palette.fast_forward_selected_to_current");
    cx.run_until_parked();
    assert_eq!(repo.git(&["rev-parse", "topic"]), topic);
}

#[gpui_kit::test]
fn rebasing_onto_the_selected_branch_replays_the_commits_and_a_conflict_is_reported(
    cx: &mut TestAppContext,
) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    // main gets its own commit, so topic diverges from it.
    repo.git(&["commit", "--allow-empty", "-m", "main moves on"]);
    repo.git(&["checkout", "topic"]);
    wait_until(cx, "the window to follow", |cx| {
        ws.read_with(cx, |ws, cx| {
            ws.refs.read(cx).snapshot().head().map(|h| h.0.to_string()) == Some("topic".into())
        })
    });
    select(cx, &ws, local("main"));
    run(cx, window, "git_prefix.rebase_onto");
    wait_until(cx, "rebased", |_| is_ancestor(&repo, "main", "topic"));
    undo(cx, window, &ws);
    wait_until(cx, "rebase undone", |_| {
        !is_ancestor(&repo, "main", "topic")
    });

    // A conflicting rebase stops: the operation banner (the one a merge or a cherry-pick uses) shows it,
    // with Resolve (not available yet), Skip and Abort.
    repo.git(&["checkout", "main"]);
    std::fs::write(repo.root().join("t.txt"), "main's version\n").unwrap();
    repo.git(&["add", "t.txt"]);
    repo.git(&["commit", "-m", "conflicting"]);
    repo.git(&["checkout", "topic"]);
    wait_until(cx, "the window to follow", |cx| {
        ws.read_with(cx, |ws, cx| {
            ws.refs.read(cx).snapshot().head().map(|h| h.0.to_string()) == Some("topic".into())
        })
    });
    select(cx, &ws, local("main"));
    run(cx, window, "git_prefix.rebase_onto");
    wait_until(cx, "the stopped rebase", |cx| {
        ws.read_with(cx, |ws, _| {
            matches!(ws.operation(), Some(OperationState::Rebase(_)))
        })
    });
    let status = ws.read_with(cx, |ws, cx| ws.operation_status(cx)).unwrap();
    assert!(status.contains("topic"), "{status}");
    let buttons = ws.read_with(cx, |ws, cx| ws.banner_button_labels(cx));
    assert!(
        buttons.contains(&msg::rebase_banner::SKIP.get().to_string())
            && buttons.contains(&msg::merge::BANNER_ABORT.get().to_string()),
        "{buttons:?}"
    );
    // Abort asks first and brings the branch back.
    let before = repo.git(&["rev-parse", "topic"]);
    let abort = ws.read_with(cx, |ws, cx| {
        ws.banner_button_labels(cx)
            .iter()
            .position(|l| l == msg::merge::BANNER_ABORT.get())
            .unwrap()
    });
    let _ = abort;
    cx.update_window(window, |_, w, cx| {
        ws.update(cx, |ws, cx| ws.banner_click(BannerButton::Abort, w, cx))
    })
    .unwrap();
    cx.run_until_parked();
    assert!(dialog_open(cx, window), "abort asks first");
    answer(cx, window, 0);
    wait_until(cx, "the rebase aborted", |cx| {
        ws.read_with(cx, |ws, _| ws.operation().is_none())
    });
    assert_eq!(repo.git(&["rev-parse", "topic"]), before);
    assert_eq!(head(&repo), "topic");
}
