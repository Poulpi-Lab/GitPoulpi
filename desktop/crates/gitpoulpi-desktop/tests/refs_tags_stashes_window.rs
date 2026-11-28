//! Headless tests of tags and stashes on the real main window: annotated and lightweight tags, pushing and
//! deleting them (locally or on the remote), stash push / pop / apply / drop and showing a stash.

mod refs_ops_common;

use gitpoulpi_desktop::commands::refs_ops::RefOp;
use gitpoulpi_desktop::ref_target::RefTarget;
use gitpoulpi_desktop::{dialogs, panels::refs::SectionKind};
use gpui_kit::TestAppContext;
use refs_ops_common::*;

// -- tags

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
