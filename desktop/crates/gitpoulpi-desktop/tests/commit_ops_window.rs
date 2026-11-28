//! Headless tests of the Log's commit operations on the real main window: the commit context menu (mouse
//! and keyboard), cherry-pick (one and several), revert, reset, detached checkout, copying fields, the
//! comparison of several selected rows, the commit details pane and the banner of an operation stopped on
//! conflicts. Fixture repositories and a temporary config directory; no network, no visible window.

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_core::domain::{Direction, Oid, OperationState};
use gitpoulpi_desktop::app::{self, BannerButton, Overrides, Workspace};
use gitpoulpi_desktop::commands;
use gitpoulpi_desktop::daemon::Backend;
use gitpoulpi_desktop::dialogs;
use gitpoulpi_i18n::msg;
use gitpoulpi_testkit::{isolate_process_env, FixtureRepo, FixtureRepoBuilder};
use gpui_kit::component::WindowExt as _;
use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, Keystroke, Modifiers, TestAppContext};

type Opened = (AnyWindowHandle, Entity<Workspace>);

const NONE: Modifiers = Modifiers {
    control: false,
    alt: false,
    shift: false,
    platform: false,
    function: false,
};
const CTRL: Modifiers = Modifiers {
    control: true,
    ..NONE
};

fn git_commit(repo: &FixtureRepo, file: &str, contents: &str, message: &str) {
    std::fs::write(repo.root().join(file), contents).unwrap();
    repo.git(&["add", "--", file]);
    repo.git(&["commit", "-q", "-m", message]);
}

/// main: initial, add a, add b. side (from add b): s1, s2. The checkout is on main.
fn repo_with_side() -> FixtureRepo {
    isolate_process_env();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .commit_file("a.txt", "a\n", "add a")
        .unwrap()
        .commit_file("b.txt", "b\n", "add b")
        .unwrap()
        .build();
    let main = repo.git(&["branch", "--show-current"]).trim().to_string();
    repo.git(&["checkout", "-q", "-b", "side"]);
    git_commit(&repo, "s1.txt", "one\n", "side one");
    git_commit(&repo, "s2.txt", "two\n", "side two");
    repo.git(&["checkout", "-q", &main]);
    repo
}

fn oid(repo: &FixtureRepo, rev: &str) -> Oid {
    Oid::from_hex(repo.git(&["rev-parse", rev]).trim()).unwrap()
}

fn subjects(repo: &FixtureRepo) -> Vec<String> {
    repo.git(&["log", "--format=%s", "HEAD"])
        .lines()
        .map(str::to_string)
        .collect()
}

fn open(cx: &mut TestAppContext, repo: &Path, prefs_dir: &Path) -> Opened {
    let backend = Backend::start(repo).unwrap();
    cx.executor().allow_parking();
    cx.update(|cx| app::init(cx, Some(prefs_dir.to_path_buf()), &Overrides::default()).unwrap());
    cx.update(|cx| app::open_main_window(backend, cx).unwrap())
}

fn wait_until(
    cx: &mut TestAppContext,
    what: &str,
    mut done: impl FnMut(&mut TestAppContext) -> bool,
) {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        cx.run_until_parked();
        if done(cx) {
            return;
        }
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(25));
    }
}

/// Opens the window on `repo` and waits for the Log's rows.
fn open_log(cx: &mut TestAppContext, repo: &FixtureRepo, rows: usize) -> Opened {
    let prefs = tempfile::tempdir().unwrap();
    let opened = open(cx, repo.root(), prefs.path());
    std::mem::forget(prefs);
    let ws = opened.1.clone();
    wait_until(cx, "the Log", |cx| {
        ws.read_with(cx, |ws, cx| ws.log.read(cx).row_count()) >= rows
    });
    run(cx, opened.0, "palette.focus_log");
    opened
}

fn run(cx: &mut TestAppContext, window: AnyWindowHandle, id: &'static str) {
    cx.update_window(window, |_, w, cx| commands::run_id(id, w, cx))
        .unwrap();
    cx.run_until_parked();
}

fn press(cx: &mut TestAppContext, window: AnyWindowHandle, keys: &'static str) {
    for key in keys.split(' ') {
        cx.update_window(window, |_, w, cx| {
            w.dispatch_keystroke(Keystroke::parse(key).unwrap(), cx);
        })
        .unwrap();
        cx.run_until_parked();
    }
}

fn select(cx: &mut TestAppContext, ws: &Entity<Workspace>, oid: Oid) {
    ws.update(cx, |ws, cx| {
        ws.log.update(cx, |l, cx| l.select_oid(oid, cx))
    });
    cx.run_until_parked();
    assert_eq!(
        ws.read_with(cx, |ws, cx| ws.log.read(cx).selected_commit()),
        Some(oid)
    );
}

fn undo(cx: &mut TestAppContext, window: AnyWindowHandle, ws: &Entity<Workspace>) {
    wait_until(cx, "an entry to undo", |cx| {
        ws.read_with(cx, |ws, _| ws.backend().can_undo_redo(Direction::Undo))
    });
    run(cx, window, "global.undo");
}

fn dialog_open(cx: &mut TestAppContext, window: AnyWindowHandle) -> bool {
    cx.update_window(window, |_, w, cx| w.has_active_dialog(cx))
        .unwrap()
}

fn answer(cx: &mut TestAppContext, window: AnyWindowHandle, ix: usize) {
    let done = cx
        .update_window(window, |_, w, cx| dialogs::answer_open(ix, w, cx))
        .unwrap();
    assert!(done, "a question is open");
    cx.run_until_parked();
}

fn notice(cx: &mut TestAppContext, ws: &Entity<Workspace>) -> String {
    ws.read_with(cx, |ws, _| {
        ws.notice().map(|n| n.text.clone()).unwrap_or_default()
    })
}

fn click_row(
    cx: &mut TestAppContext,
    ws: &Entity<Workspace>,
    window: AnyWindowHandle,
    oid: Oid,
    m: Modifiers,
) {
    cx.update_window(window, |_, w, cx| {
        ws.update(cx, |ws, cx| {
            ws.log.update(cx, |l, cx| {
                let ix = l.row_of(oid).expect("the commit is in the Log");
                l.click_row(ix, m, w, cx)
            })
        })
    })
    .unwrap();
    cx.run_until_parked();
}

fn menu_labels(cx: &mut TestAppContext, ws: &Entity<Workspace>) -> Option<Vec<String>> {
    ws.read_with(cx, |ws, cx| {
        ws.log.read(cx).menu().map(|m| m.read(cx).labels())
    })
}

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

fn oid_of_head(repo: &FixtureRepo) -> Oid {
    oid(repo, "HEAD")
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

// ---- an operation stopped on conflicts --------------------------------------------------------------

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
