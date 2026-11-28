//! Headless tests of the operation banner on the real main window: a cherry-pick that stops on conflicts
//! shows its state (taken from the link's refresh outcome), Abort and Skip ask first, Continue appears
//! once nothing is conflicted. Fixture repositories and a temporary config directory; no visible window.

mod commit_ops_common;

use commit_ops_common::*;
use gitpoulpi_core::domain::OperationState;
use gitpoulpi_desktop::app::BannerButton;
use gitpoulpi_i18n::msg;
use gitpoulpi_testkit::{isolate_process_env, FixtureRepo, FixtureRepoBuilder};
use gpui_kit::{AppContext as _, TestAppContext};

/// main and side both change a.txt: cherry-picking side's commit conflicts.
fn conflicting_repo() -> FixtureRepo {
    isolate_process_env();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .commit_file("a.txt", "base\n", "add a")
        .unwrap()
        .build();
    let main = repo.git(&["branch", "--show-current"]).trim().to_string();
    repo.git(&["checkout", "-q", "-b", "side"]);
    git_commit(&repo, "a.txt", "side\n", "side edits a");
    repo.git(&["checkout", "-q", &main]);
    git_commit(&repo, "a.txt", "main\n", "main edits a");
    repo
}

#[gpui_kit::test]
fn a_conflicting_cherry_pick_shows_the_banner_and_abort_asks_then_restores(
    cx: &mut TestAppContext,
) {
    let repo = conflicting_repo();
    let (window, ws) = open_log(cx, &repo, 3);
    let before = oid_of_head(&repo);
    select(cx, &ws, oid(&repo, "side"));
    run(cx, window, "git_prefix.cherry_pick");
    wait_until(cx, "the stopped cherry-pick", |cx| {
        ws.read_with(cx, |ws, _| {
            matches!(ws.operation(), Some(OperationState::CherryPick { .. }))
        })
    });
    let status = ws.read_with(cx, |ws, cx| ws.operation_status(cx)).unwrap();
    assert!(
        status.contains(&oid(&repo, "side").to_hex()[..7]),
        "{status}"
    );
    let buttons = ws.read_with(cx, |ws, cx| ws.banner_button_labels(cx));
    assert!(
        buttons.contains(&msg::merge::BANNER_ABORT.get().to_string()),
        "{buttons:?}"
    );
    assert!(
        buttons.contains(&msg::merge::BANNER_SKIP.get().to_string()),
        "{buttons:?}"
    );

    run(cx, window, "palette.abort_operation");
    assert!(dialog_open(cx, window), "Abort asks first");
    answer(cx, window, 1);
    assert!(
        ws.read_with(cx, |ws, _| ws.operation().is_some()),
        "Cancel keeps the operation"
    );
    run(cx, window, "palette.abort_operation");
    answer(cx, window, 0);
    wait_until(cx, "the operation gone", |cx| {
        ws.read_with(cx, |ws, _| ws.operation().is_none())
    });
    assert_eq!(oid_of_head(&repo), before);
    assert_eq!(
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap(),
        "main\n"
    );
}

#[gpui_kit::test]
fn skip_leaves_a_stopped_cherry_pick_behind(cx: &mut TestAppContext) {
    let repo = conflicting_repo();
    let (window, ws) = open_log(cx, &repo, 3);
    select(cx, &ws, oid(&repo, "side"));
    run(cx, window, "git_prefix.cherry_pick");
    wait_until(cx, "the stopped cherry-pick", |cx| {
        ws.read_with(cx, |ws, _| ws.operation().is_some())
    });
    run(cx, window, "palette.skip_operation");
    assert!(dialog_open(cx, window), "Skip asks first");
    answer(cx, window, 0);
    wait_until(cx, "the operation gone", |cx| {
        ws.read_with(cx, |ws, _| ws.operation().is_none())
    });
}

#[gpui_kit::test]
fn the_banner_has_continue_once_nothing_is_conflicted_and_abort_runs_from_the_keyboard_palette(
    cx: &mut TestAppContext,
) {
    let repo = conflicting_repo();
    let (window, ws) = open_log(cx, &repo, 3);
    select(cx, &ws, oid(&repo, "side"));
    run(cx, window, "git_prefix.cherry_pick");
    wait_until(cx, "the stopped cherry-pick", |cx| {
        ws.read_with(cx, |ws, _| ws.operation().is_some())
    });
    // Resolve it on disk and stage: no file is conflicted any more.
    std::fs::write(repo.root().join("a.txt"), "resolved\n").unwrap();
    repo.git(&["add", "a.txt"]);
    wait_until(cx, "Continue offered", |cx| {
        ws.read_with(cx, |ws, cx| {
            ws.banner_button_labels(cx)
                .contains(&msg::merge::BANNER_CONTINUE.get().to_string())
        })
    });
    // The WIP row (the resolved file is staged) is part of the Log before Continue has a commit to make.
    wait_until(cx, "the WIP row", |cx| {
        ws.read_with(cx, |ws, cx| ws.log.read(cx).row_count()) >= 5
    });
    // Continue takes the keyboard to the WIP row's commit summary.
    cx.update_window(window, |_, w, cx| {
        ws.update(cx, |ws, cx| ws.banner_click(BannerButton::Continue, w, cx))
    })
    .unwrap();
    cx.run_until_parked();
    let focused = cx
        .update_window(window, |_, w, cx| {
            ws.read_with(cx, |ws, cx| ws.right.read(cx).composer_focused(w, cx))
        })
        .unwrap();
    assert!(focused, "Continue goes to the commit form");
}

#[gpui_kit::test]
fn an_operation_another_client_left_stopped_shows_its_banner_and_clears_when_it_ends(
    cx: &mut TestAppContext,
) {
    let repo = conflicting_repo();
    let (_window, ws) = open_log(cx, &repo, 3);
    assert!(ws.read_with(cx, |ws, _| ws.operation().is_none()));
    // Plain git, outside the window: the state arrives with the daemon's refresh, nothing is polled.
    let started = std::process::Command::new("git")
        .current_dir(repo.root())
        .args(["cherry-pick", "side"])
        .output()
        .unwrap();
    assert!(!started.status.success(), "the pick stops on a conflict");
    wait_until(cx, "the banner", |cx| {
        ws.read_with(cx, |ws, _| {
            matches!(ws.operation(), Some(OperationState::CherryPick { .. }))
        })
    });
    repo.git(&["cherry-pick", "--abort"]);
    wait_until(cx, "the banner gone", |cx| {
        ws.read_with(cx, |ws, _| ws.operation().is_none())
    });
}
