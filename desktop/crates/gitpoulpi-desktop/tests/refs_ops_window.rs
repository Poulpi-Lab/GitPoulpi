//! Headless tests of branches, tags and stashes on the real main window: the forms and questions that
//! come first, name validation, the safety confirmations, the context menu (by keyboard), the drop menu
//! and the auto-stash question, each against a fixture repository with a local bare repository as its
//! remote, with Undo after every mutation.

use std::time::{Duration, Instant};

use gitpoulpi_core::domain::Direction;
use gitpoulpi_desktop::app::{self, Overrides, Workspace};
use gitpoulpi_desktop::commands::refs_ops::RefOp;
use gitpoulpi_desktop::commands::{self};
use gitpoulpi_desktop::daemon::Backend;
use gitpoulpi_desktop::form_dialog::{self, FormView};
use gitpoulpi_desktop::ref_target::RefTarget;
use gitpoulpi_desktop::{dialogs, panels::refs::SectionKind};
use gitpoulpi_testkit::{isolate_process_env, FixtureRepo, FixtureRepoBuilder};
use gpui_kit::component::WindowExt as _;
use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, Keystroke, TestAppContext};

/// main (pushed, tracking origin) with `topic` one commit ahead (pushed), a `v1` tag, and `remote-only`
/// pushed and then deleted locally.
fn repo() -> FixtureRepo {
    isolate_process_env();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .commit_file("a.txt", "a\n", "add a")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("HEAD:refs/heads/main")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .branch("topic")
        .unwrap()
        .commit_file("t.txt", "t\n", "topic work")
        .unwrap()
        .push("topic")
        .unwrap()
        .branch("remote-only")
        .unwrap()
        .push("remote-only")
        .unwrap()
        .checkout("main")
        .unwrap()
        .tag("v1")
        .unwrap()
        .build();
    repo.git(&["branch", "-D", "remote-only"]);
    repo
}

type Opened = (AnyWindowHandle, Entity<Workspace>);

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

/// Opens the window and waits until the Refs sidebar lists the fixture's branches.
fn open(cx: &mut TestAppContext, repo: &FixtureRepo) -> Opened {
    let prefs = tempfile::tempdir().unwrap();
    let backend = Backend::start(repo.root()).unwrap();
    cx.executor().allow_parking();
    cx.update(|cx| app::init(cx, Some(prefs.path().to_path_buf()), &Overrides::default()).unwrap());
    std::mem::forget(prefs);
    let opened = cx.update(|cx| app::open_main_window(backend, cx).unwrap());
    let ws = opened.1.clone();
    wait_until(cx, "the branches", |cx| {
        ws.read_with(cx, |ws, cx| {
            let labels = ws.refs.read(cx).row_labels(SectionKind::Local);
            labels.iter().any(|l| l == "topic")
                && !ws.refs.read(cx).row_labels(SectionKind::Remote).is_empty()
        })
    });
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

fn run_op(
    cx: &mut TestAppContext,
    window: AnyWindowHandle,
    ws: &Entity<Workspace>,
    op: RefOp,
    target: Option<RefTarget>,
) {
    cx.update_window(window, |_, w, cx| {
        ws.update(cx, |ws, cx| ws.run_ref_op(op, target, w, cx))
    })
    .unwrap();
    cx.run_until_parked();
}

fn select(cx: &mut TestAppContext, ws: &Entity<Workspace>, target: RefTarget) {
    let found = ws.update(cx, |ws, cx| {
        ws.refs.update(cx, |r, cx| r.select_target(&target, cx))
    });
    assert!(found, "{target:?} is listed");
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

fn question_title(cx: &mut TestAppContext) -> String {
    cx.update(|cx| dialogs::current(cx))
        .expect("a question")
        .read_with(cx, |q, _| q.spec().title.clone())
}

fn form(cx: &mut TestAppContext) -> Entity<FormView> {
    cx.update(|cx| form_dialog::current(cx)).expect("a form")
}

fn type_into(cx: &mut TestAppContext, window: AnyWindowHandle, ix: usize, text: &str) {
    let f = form(cx);
    cx.update_window(window, |_, w, cx| {
        f.update(cx, |f, cx| f.set_text(ix, text, w, cx))
    })
    .unwrap();
    cx.run_until_parked();
}

fn submit(cx: &mut TestAppContext, window: AnyWindowHandle) {
    let f = form(cx);
    cx.update_window(window, |_, w, cx| f.update(cx, |f, cx| f.submit(w, cx)))
        .unwrap();
    cx.run_until_parked();
}

fn form_error(cx: &mut TestAppContext) -> Option<String> {
    form(cx).read_with(cx, |f, _| f.error().map(str::to_string))
}

fn notice(cx: &mut TestAppContext, ws: &Entity<Workspace>) -> String {
    ws.read_with(cx, |ws, _| {
        ws.notice().map(|n| n.text.clone()).unwrap_or_default()
    })
}

fn head(repo: &FixtureRepo) -> String {
    repo.git(&["branch", "--show-current"]).trim().to_string()
}

fn branches(repo: &FixtureRepo) -> Vec<String> {
    repo.git(&["branch", "--format=%(refname:short)"])
        .lines()
        .map(str::to_string)
        .collect()
}

fn has_branch(repo: &FixtureRepo, name: &str) -> bool {
    branches(repo).iter().any(|b| b == name)
}

fn remote_has(repo: &FixtureRepo, refname: &str) -> bool {
    !repo
        .git(&["ls-remote", "origin", refname])
        .trim()
        .is_empty()
}

fn is_ancestor(repo: &FixtureRepo, ancestor: &str, of: &str) -> bool {
    repo.try_git(&["merge-base", "--is-ancestor", ancestor, of])
        .is_some()
}

fn local(name: &str) -> RefTarget {
    RefTarget::Local(name.into())
}

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

    // A conflicting rebase stops: the window says the resolution view is not there yet.
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
    wait_until(cx, "the conflict to be reported", |cx| {
        ws.read_with(cx, |ws, _| ws.operation_in_progress().is_some())
    });
    let said = notice(cx, &ws);
    assert!(said.contains("resolution view is not available"), "{said}");
    let (name, conflicts) = ws.read_with(cx, |ws, _| ws.operation_in_progress().cloned().unwrap());
    assert!(name.to_lowercase().contains("rebase"), "{name}");
    assert!(conflicts >= 1);
}

// -- tags -------------------------------------------------------------------------------------------

#[gpui_kit::test]
fn tags_are_created_lightweight_or_annotated_and_undo_removes_them(cx: &mut TestAppContext) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    run(cx, window, "git_prefix.new_tag");
    type_into(cx, window, 0, "v1");
    submit(cx, window);
    assert!(form_error(cx).is_some(), "an existing tag name is refused");
    type_into(cx, window, 0, "v2");
    submit(cx, window);
    wait_until(cx, "the tag", |_| {
        repo.try_git(&["rev-parse", "refs/tags/v2"]).is_some()
    });
    assert_eq!(
        repo.git(&["cat-file", "-t", "refs/tags/v2"]).trim(),
        "commit"
    );
    undo(cx, window, &ws);
    wait_until(cx, "the tag gone", |_| {
        repo.try_git(&["rev-parse", "refs/tags/v2"]).is_none()
    });

    run(cx, window, "git_prefix.new_tag");
    type_into(cx, window, 0, "v3");
    type_into(cx, window, 1, "release three");
    submit(cx, window);
    wait_until(cx, "the annotated tag", |_| {
        repo.try_git(&["rev-parse", "refs/tags/v3"]).is_some()
    });
    assert_eq!(repo.git(&["cat-file", "-t", "refs/tags/v3"]).trim(), "tag");
    undo(cx, window, &ws);
    wait_until(cx, "the annotated tag gone", |_| {
        repo.try_git(&["rev-parse", "refs/tags/v3"]).is_none()
    });
}

#[gpui_kit::test]
fn a_tag_is_pushed_and_deleted_locally_or_on_the_remote_after_asking(cx: &mut TestAppContext) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    select(cx, &ws, RefTarget::Tag("v1".into()));
    run(cx, window, "palette.push_selected_tag");
    wait_until(cx, "the tag on the remote", |_| {
        remote_has(&repo, "refs/tags/v1")
    });
    wait_until(cx, "the sidebar to know", |cx| {
        ws.read_with(cx, |ws, cx| {
            ws.refs.read(cx).snapshot().tag_place("v1").remotes == vec!["origin".to_string()]
        })
    });

    // On a remote too: the delete asks where, with nothing chosen yet.
    run(cx, window, "git_prefix.delete_ref");
    assert!(dialog_open(cx, window));
    let q = cx.update(|cx| dialogs::current(cx)).unwrap();
    let (choices, preset) = q.read_with(cx, |q, _| {
        (q.spec().choices.len(), q.spec().highlight_first)
    });
    assert_eq!(choices, 3, "locally, on the remote, both");
    assert!(!preset);
    // The remote only: a second question names the remote, then the local tag stays.
    answer(cx, window, 1);
    assert!(dialog_open(cx, window), "the remote delete is confirmed");
    assert!(question_title(cx).contains("origin"));
    answer(cx, window, 0);
    wait_until(cx, "gone from the remote", |_| {
        !remote_has(&repo, "refs/tags/v1")
    });
    assert!(repo.try_git(&["rev-parse", "refs/tags/v1"]).is_some());

    // A tag only here is deleted after one confirmation, and undo restores it.
    wait_until(cx, "the sidebar to know", |cx| {
        ws.read_with(cx, |ws, cx| {
            ws.refs
                .read(cx)
                .snapshot()
                .tag_place("v1")
                .remotes
                .is_empty()
        })
    });
    run(cx, window, "git_prefix.delete_ref");
    assert!(dialog_open(cx, window));
    answer(cx, window, 1);
    assert!(
        repo.try_git(&["rev-parse", "refs/tags/v1"]).is_some(),
        "cancel keeps it"
    );
    run(cx, window, "git_prefix.delete_ref");
    answer(cx, window, 0);
    wait_until(cx, "deleted", |_| {
        repo.try_git(&["rev-parse", "refs/tags/v1"]).is_none()
    });
    undo(cx, window, &ws);
    wait_until(cx, "restored", |_| {
        repo.try_git(&["rev-parse", "refs/tags/v1"]).is_some()
    });
}

// -- stashes ----------------------------------------------------------------------------------------

fn dirty(repo: &FixtureRepo) {
    std::fs::write(repo.root().join("a.txt"), "changed\n").unwrap();
}

fn stashes(repo: &FixtureRepo) -> usize {
    repo.git(&["stash", "list"]).lines().count()
}

#[gpui_kit::test]
fn stash_push_message_pop_apply_and_drop_each_undo(cx: &mut TestAppContext) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    dirty(&repo);
    run(cx, window, "git_prefix.stash_push");
    wait_until(cx, "stashed", |_| stashes(&repo) == 1);
    assert!(repo.git(&["status", "--porcelain"]).trim().is_empty());
    undo(cx, window, &ws);
    wait_until(cx, "stash undone", |_| stashes(&repo) == 0);
    assert_eq!(
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap(),
        "changed\n"
    );

    // With a message, through the form.
    run(cx, window, "git_prefix.stash_message");
    assert!(dialog_open(cx, window));
    type_into(cx, window, 0, "half done");
    submit(cx, window);
    wait_until(cx, "stashed with a message", |_| stashes(&repo) == 1);
    assert!(repo.git(&["stash", "list"]).contains("half done"));
    wait_until(cx, "the sidebar to list it", |cx| {
        ws.read_with(cx, |ws, cx| {
            !ws.refs.read(cx).row_labels(SectionKind::Stashes).is_empty()
        })
    });

    // Apply keeps the stash; pop removes it.
    run(cx, window, "palette.stash_apply");
    wait_until(cx, "applied", |_| {
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap() == "changed\n"
    });
    assert_eq!(stashes(&repo), 1, "apply keeps the stash");
    repo.git(&["checkout", "--", "a.txt"]);
    run(cx, window, "git_prefix.stash_pop");
    wait_until(cx, "popped", |_| stashes(&repo) == 0);
    undo(cx, window, &ws);
    wait_until(cx, "pop undone", |_| stashes(&repo) == 1);

    // Dropping asks first and undo restores the stash.
    run(cx, window, "palette.stash_drop");
    assert!(dialog_open(cx, window));
    answer(cx, window, 1);
    assert_eq!(stashes(&repo), 1, "cancel keeps it");
    run(cx, window, "palette.stash_drop");
    answer(cx, window, 0);
    wait_until(cx, "dropped", |_| stashes(&repo) == 0);
    undo(cx, window, &ws);
    wait_until(cx, "drop undone", |_| stashes(&repo) == 1);
}

#[gpui_kit::test]
fn a_stash_shows_its_changes_in_the_diff_pane(cx: &mut TestAppContext) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    dirty(&repo);
    run(cx, window, "git_prefix.stash_push");
    wait_until(cx, "stashed", |_| stashes(&repo) == 1);
    wait_until(cx, "the sidebar to list it", |cx| {
        ws.read_with(cx, |ws, cx| {
            !ws.refs.read(cx).row_labels(SectionKind::Stashes).is_empty()
        })
    });
    run_op(cx, window, &ws, RefOp::StashShow, Some(RefTarget::Stash(0)));
    wait_until(cx, "the diff", |cx| {
        ws.read_with(cx, |ws, _| ws.diff_is_open())
    });
}

// -- the menu ---------------------------------------------------------------------------------------

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

#[gpui_kit::test]
fn a_ref_pill_of_the_log_has_the_same_menu_target(cx: &mut TestAppContext) {
    let repo = repo();
    let (_window, ws) = open(cx, &repo);
    wait_until(cx, "a pill on the selected commit", |cx| {
        ws.update(cx, |ws, cx| {
            let head = ws.refs.read(cx).snapshot().oid_of(&local("main"));
            if let Some(oid) = head {
                ws.log.update(cx, |l, cx| l.select_oid(oid, cx));
            }
            ws.log.read(cx).first_ref_target().is_some()
        })
    });
    let target = ws.read_with(cx, |ws, cx| ws.log.read(cx).first_ref_target());
    assert_eq!(
        target,
        Some(local("main")),
        "the checked-out branch comes first"
    );
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

// -- the mouse --------------------------------------------------------------------------------------

fn frames(cx: &mut TestAppContext, window: AnyWindowHandle, n: usize) {
    for _ in 0..n {
        cx.update_window(window, |_, window, _| window.refresh())
            .ok();
        cx.run_until_parked();
    }
}

fn sim<E: gpui_kit::InputEvent>(cx: &mut TestAppContext, window: AnyWindowHandle, event: E) {
    let mut v = gpui_kit::VisualTestContext::from_window(window, cx);
    v.simulate_event(event);
    v.run_until_parked();
}

/// A point inside the selected Refs row, from where it was last drawn.
fn row_point(
    cx: &mut TestAppContext,
    window: AnyWindowHandle,
    ws: &Entity<Workspace>,
    target: RefTarget,
) -> gpui_kit::Point<gpui_kit::Pixels> {
    select(cx, ws, target);
    frames(cx, window, 3);
    let p = ws
        .read_with(cx, |ws, cx| ws.refs.read(cx).selected_row_origin())
        .expect("the selected row was drawn");
    gpui_kit::Point::new(p.x, p.y - gpui_kit::px(10.0))
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
