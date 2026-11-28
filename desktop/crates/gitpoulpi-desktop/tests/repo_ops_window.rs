//! Headless tests of worktrees, submodules, remotes and the push / pull / fetch rows of a branch's menu on
//! the real main window: the forms and their refusals, the questions (removing a worktree, removing a
//! remote), the menus of the rows, and undo after each mutation that is journaled.

mod refs_ops_common;
mod repo_ops_common;

use gitpoulpi_desktop::commands::repo_ops::RepoOp;
use gitpoulpi_desktop::panels::refs::SectionKind;
use gitpoulpi_desktop::ref_target::RefTarget;
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use gpui_kit::TestAppContext;
use refs_ops_common::*;
use repo_ops_common::*;

fn worktree_paths(repo: &FixtureRepo) -> Vec<String> {
    repo.git(&["worktree", "list", "--porcelain"])
        .lines()
        .filter_map(|l| l.strip_prefix("worktree ").map(str::to_string))
        .collect()
}

fn wait_worktree_row(
    cx: &mut TestAppContext,
    ws: &gpui_kit::Entity<gitpoulpi_desktop::app::Workspace>,
) -> String {
    wait_until(cx, "a worktree row", |cx| {
        ws.read_with(cx, |ws, cx| {
            ws.refs.read(cx).row_labels(SectionKind::Worktrees).len() >= 2
        })
    });
    ws.read_with(cx, |ws, cx| {
        ws.refs
            .read(cx)
            .worktrees()
            .iter()
            .find(|w| !w.is_main)
            .unwrap()
            .path
            .to_string_lossy()
            .into_owned()
    })
}

// -- worktrees ----------------------------------------------------------------------------------------

#[gpui_kit::test]
fn a_worktree_is_created_from_the_current_branch_after_the_form_checks_the_entry(
    cx: &mut TestAppContext,
) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    let folder = repo.root().parent().unwrap().join("made-wt");
    run(cx, window, "palette.create_worktree_from_current_branch");
    // The current branch is checked out here: the form proposes a new branch.
    let proposed = form(cx).read_with(cx, |f, cx| f.values(cx));
    assert!(proposed.text(1).starts_with("worktree-"), "{proposed:?}");
    type_into(cx, window, 0, "relative/dir");
    submit(cx, window);
    assert!(form_error(cx).is_some(), "a relative folder is refused");
    type_into(cx, window, 0, &folder.to_string_lossy());
    type_into(cx, window, 1, "main");
    submit(cx, window);
    assert!(form_error(cx).is_some(), "main is checked out already");
    type_into(cx, window, 1, "not a branch");
    submit(cx, window);
    assert!(form_error(cx).is_some(), "a bad branch name is refused");
    type_into(cx, window, 1, "wt-branch");
    let f = form(cx);
    cx.update(|cx| f.update(cx, |f, cx| f.set_check(2, false, cx)));
    submit(cx, window);
    wait_until(cx, "the worktree", |_| folder.join(".git").exists());
    assert!(has_branch(&repo, "wt-branch"));
    assert!(worktree_paths(&repo).iter().any(|p| p.ends_with("made-wt")));
    // Not opened (the box was cleared): the window still shows the repository.
    assert_eq!(
        ws.read_with(cx, |ws, _| ws.backend().repo_root()),
        repo.root()
    );
    wait_worktree_row(cx, &ws);
    // Not undoable: the daemon does not journal a worktree (its folder lives outside the repository).
    assert!(!ws.read_with(cx, |ws, _| ws
        .backend()
        .can_undo_redo(gitpoulpi_core::domain::Direction::Undo)));
    // The folder is not empty any more.
    run(cx, window, "palette.create_worktree_from_current_branch");
    wait_form(cx);
    type_into(cx, window, 0, &folder.to_string_lossy());
    submit(cx, window);
    assert!(form_error(cx).unwrap().contains("not empty"));
}

#[gpui_kit::test]
fn a_worktree_made_from_a_commit_or_a_tag_opens_in_the_window(cx: &mut TestAppContext) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    // The + menu of the WORKTREES header: from the branch, from the selected commit, choosing one.
    run(cx, window, "palette.create_worktree_menu");
    let labels = menu_labels(cx, &ws);
    assert_eq!(labels.len(), 3, "{labels:?}");
    assert!(labels[0].contains("current branch") && labels[1].contains("selected commit"));
    press(cx, window, "escape");

    // A tag: detached start, no branch proposed; the form opens the worktree afterwards by default.
    let folder = repo.root().parent().unwrap().join("tag-wt");
    run_repo(
        cx,
        window,
        &ws,
        RepoOp::WorktreeFromRef,
        Some(RefTarget::Tag("v1".into())),
    );
    let values = form(cx).read_with(cx, |f, cx| f.values(cx));
    assert!(values.check(2), "open after creating is on by default");
    type_into(cx, window, 0, &folder.to_string_lossy());
    type_into(cx, window, 1, "");
    submit(cx, window);
    wait_until(cx, "the worktree", |_| folder.join(".git").exists());
    wait_until(cx, "the window follows into it", |cx| {
        ws.read_with(cx, |ws, _| ws.backend().repo_root().ends_with("tag-wt"))
    });
    assert!(repo.git(&["worktree", "list"]).contains("tag-wt"));
    assert!(
        repo.git(&["worktree", "list"]).contains("detached"),
        "a tag start is detached"
    );

    // From the selected commit, with the Log's selection: refused without one, accepted with a branch.
    run_repo(cx, window, &ws, RepoOp::WorktreeChoose, None);
    wait_form(cx);
    let chosen = form(cx).read_with(cx, |f, cx| f.values(cx));
    assert_eq!(
        chosen.text(1),
        "",
        "Choose a branch starts with an empty branch field"
    );
}

#[gpui_kit::test]
fn a_worktree_opens_in_the_window_and_the_main_one_cannot_be_removed(cx: &mut TestAppContext) {
    let (builder, wt) = FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
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
        .checkout("main")
        .unwrap()
        .worktree("linked", "feature")
        .unwrap();
    let repo = builder.build();
    let (window, ws) = open(cx, &repo);
    let linked = wait_worktree_row(cx, &ws);
    assert!(linked.ends_with("linked"));
    // The menu of the linked row: open, open in, merge (not available), remove.
    let main_path = repo.root().to_string_lossy().into_owned();
    open_menu(cx, window, &ws, RefTarget::Worktree(linked.clone()));
    let labels = menu_labels(cx, &ws);
    assert!(
        labels.iter().any(|l| l.contains("Open in a new tab")),
        "{labels:?}"
    );
    assert!(
        labels.iter().any(|l| l.contains("Remove worktree")),
        "{labels:?}"
    );
    press(cx, window, "escape");
    assert!(ws.read_with(cx, |ws, _| ws.ref_menu().is_none()));
    // The main worktree's menu has no Remove that works.
    run_repo(
        cx,
        window,
        &ws,
        RepoOp::WorktreeRemove,
        Some(RefTarget::Worktree(main_path)),
    );
    assert!(
        !dialog_open(cx, window),
        "nothing to ask for the main worktree"
    );
    assert!(notice(cx, &ws).contains("main worktree"));
    // Opening switches the window's repository to the worktree's folder.
    run_repo(
        cx,
        window,
        &ws,
        RepoOp::WorktreeOpen,
        Some(RefTarget::Worktree(linked.clone())),
    );
    wait_until(cx, "the repository switch", |cx| {
        ws.read_with(cx, |ws, _| {
            ws.backend().repo_root().to_string_lossy() == linked.as_str()
        })
    });
    drop(wt);
}

#[gpui_kit::test]
fn removing_a_clean_worktree_asks_once_and_a_dirty_one_offers_a_stash(cx: &mut TestAppContext) {
    let (builder, _) = FixtureRepoBuilder::new()
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
        .checkout("main")
        .unwrap()
        .worktree("clean", "clean-b")
        .unwrap();
    let (builder, _) = builder.worktree("dirty", "dirty-b").unwrap();
    let repo = builder.build();
    let (window, ws) = open(cx, &repo);
    wait_until(cx, "both worktrees", |cx| {
        ws.read_with(cx, |ws, cx| ws.refs.read(cx).worktrees().len() >= 3)
    });
    let path_of = |cx: &mut TestAppContext, tail: &str| {
        ws.read_with(cx, |ws, cx| {
            ws.refs
                .read(cx)
                .worktrees()
                .iter()
                .find(|w| w.path.ends_with(tail))
                .unwrap()
                .path
                .to_string_lossy()
                .into_owned()
        })
    };
    let (clean, dirty) = (path_of(cx, "clean"), path_of(cx, "dirty"));

    // Clean: one question; Cancel keeps the folder.
    run_repo(
        cx,
        window,
        &ws,
        RepoOp::WorktreeRemove,
        Some(RefTarget::Worktree(clean.clone())),
    );
    wait_until(cx, "the question", |cx| dialog_open(cx, window));
    assert_eq!(
        cx.update(|cx| gitpoulpi_desktop::dialogs::current(cx))
            .unwrap()
            .read_with(cx, |q, _| q.spec().choices.len()),
        2
    );
    answer(cx, window, 1);
    assert!(std::path::Path::new(&clean).exists());
    run_repo(
        cx,
        window,
        &ws,
        RepoOp::WorktreeRemove,
        Some(RefTarget::Worktree(clean.clone())),
    );
    wait_until(cx, "the question again", |cx| dialog_open(cx, window));
    answer(cx, window, 0);
    wait_until(cx, "the folder gone", |_| {
        !std::path::Path::new(&clean).exists()
    });
    assert!(has_branch(&repo, "clean-b"), "the branch stays");

    // Dirty: three answers, nothing preselected; "Stash and remove" saves the changes first.
    std::fs::write(std::path::Path::new(&dirty).join("a.txt"), "edited\n").unwrap();
    run_repo(
        cx,
        window,
        &ws,
        RepoOp::WorktreeRemove,
        Some(RefTarget::Worktree(dirty.clone())),
    );
    wait_until(cx, "the dirty question", |cx| dialog_open(cx, window));
    let q = cx
        .update(|cx| gitpoulpi_desktop::dialogs::current(cx))
        .unwrap();
    let (n, preset) = q.read_with(cx, |q, _| {
        (q.spec().choices.len(), q.spec().highlight_first)
    });
    assert_eq!(n, 3);
    assert!(!preset, "no answer is the default");
    answer(cx, window, 0);
    wait_until(cx, "the dirty folder gone", |_| {
        !std::path::Path::new(&dirty).exists()
    });
    assert_eq!(stashes(&repo), 1, "the changes were stashed");
}

// -- submodules ---------------------------------------------------------------------------------------

#[gpui_kit::test]
fn a_submodule_updates_initializes_and_opens(cx: &mut TestAppContext) {
    let builder = FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .with_origin()
        .unwrap()
        .with_submodule("vendor/lib")
        .unwrap();
    let repo = builder.build();
    repo.git(&["submodule", "deinit", "-f", "vendor/lib"]);
    let (window, ws) = open_plain(cx, &repo);
    wait_until(cx, "the submodule row", |cx| {
        ws.read_with(cx, |ws, cx| {
            !ws.refs
                .read(cx)
                .row_labels(SectionKind::Submodules)
                .is_empty()
        })
    });
    let sub = RefTarget::Submodule("vendor/lib".into());
    // Not cloned: its menu offers Update and Initialize, Open is dimmed with the reason.
    open_menu(cx, window, &ws, sub.clone());
    let menu = ws.read_with(cx, |ws, _| ws.ref_menu().cloned()).unwrap();
    let labels = menu.read_with(cx, |m, _| m.labels());
    assert!(labels.iter().any(|l| l.contains("Update")), "{labels:?}");
    let open_row = labels.iter().position(|l| l.contains("Open")).unwrap();
    assert!(menu.read_with(cx, |m, _| m.refusal(open_row)).is_some());
    press(cx, window, "escape");
    // Open of an uncloned submodule says so and changes nothing.
    run_repo(cx, window, &ws, RepoOp::SubmoduleOpen, Some(sub.clone()));
    assert!(notice(cx, &ws).contains("not cloned"));
    run_repo(cx, window, &ws, RepoOp::SubmoduleUpdate, Some(sub.clone()));
    wait_until(cx, "the submodule cloned", |_| {
        repo.root().join("vendor/lib/lib.txt").exists()
    });
    wait_until(cx, "the row to know", |cx| {
        ws.read_with(cx, |ws, cx| {
            ws.refs
                .read(cx)
                .submodules()
                .iter()
                .all(|s| s.state != gitpoulpi_core::domain::SubmoduleState::Uninitialized)
        })
    });
    // Now it opens as the window's repository.
    run_repo(cx, window, &ws, RepoOp::SubmoduleOpen, Some(sub));
    wait_until(cx, "the switch", |cx| {
        ws.read_with(cx, |ws, _| ws.backend().repo_root().ends_with("vendor/lib"))
    });
}

// -- remotes ------------------------------------------------------------------------------------------

fn remote_names(repo: &FixtureRepo) -> Vec<String> {
    repo.git(&["remote"]).lines().map(str::to_string).collect()
}

#[gpui_kit::test]
fn a_remote_is_added_edited_fetched_and_removed_after_asking(cx: &mut TestAppContext) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    let other = repo.root().parent().unwrap().join("other.git");
    repo.git(&["init", "--bare", &other.to_string_lossy()]);

    // Add: the name and the URL are checked before anything is submitted.
    run(cx, window, "refs.add_remote");
    wait_form(cx);
    type_into(cx, window, 0, "origin");
    type_into(cx, window, 1, &other.to_string_lossy());
    submit(cx, window);
    assert!(form_error(cx).unwrap().contains("already exists"));
    type_into(cx, window, 0, "up stream");
    submit(cx, window);
    assert!(form_error(cx).is_some(), "a name with a space is refused");
    type_into(cx, window, 0, "second");
    type_into(cx, window, 1, "nonsense");
    submit(cx, window);
    assert!(form_error(cx).is_some(), "a URL that is none is refused");
    type_into(cx, window, 1, &other.to_string_lossy());
    submit(cx, window);
    wait_until(cx, "the remote", |_| {
        remote_names(&repo).contains(&"second".to_string())
    });
    undo(cx, window, &ws);
    wait_until(cx, "the remote gone", |_| {
        !remote_names(&repo).contains(&"second".to_string())
    });
    // Undo can be taken back.
    run(cx, window, "global.redo");
    wait_until(cx, "the remote again", |_| {
        remote_names(&repo).contains(&"second".to_string())
    });

    // Edit: rename and a new URL, each only when it changed.
    let node = RefTarget::RemoteNode("second".into());
    run_repo(cx, window, &ws, RepoOp::RemoteEdit, Some(node));
    wait_form(cx);
    assert_eq!(
        form(cx).read_with(cx, |f, cx| f.values(cx)).text(0),
        "second"
    );
    type_into(cx, window, 0, "origin");
    submit(cx, window);
    assert!(form_error(cx).unwrap().contains("already exists"));
    type_into(cx, window, 0, "renamed");
    submit(cx, window);
    wait_until(cx, "renamed", |_| {
        remote_names(&repo).contains(&"renamed".to_string())
    });
    undo(cx, window, &ws);
    wait_until(cx, "the old name back", |_| {
        remote_names(&repo).contains(&"second".to_string())
    });

    // Fetch one remote.
    repo.git(&["push", "second", "HEAD:refs/heads/side"]);
    run_repo(
        cx,
        window,
        &ws,
        RepoOp::RemoteFetch,
        Some(RefTarget::RemoteNode("second".into())),
    );
    wait_until(cx, "fetched", |_| {
        repo.try_git(&["rev-parse", "refs/remotes/second/side"])
            .is_some()
    });

    // Remove: asks first (nothing preselected); Cancel keeps it, the other answer removes it.
    run_repo(
        cx,
        window,
        &ws,
        RepoOp::RemoteRemove,
        Some(RefTarget::RemoteNode("second".into())),
    );
    assert!(dialog_open(cx, window));
    let preset = cx
        .update(|cx| gitpoulpi_desktop::dialogs::current(cx))
        .unwrap()
        .read_with(cx, |q, _| q.spec().highlight_first);
    assert!(!preset);
    answer(cx, window, 1);
    assert!(remote_names(&repo).contains(&"second".to_string()));
    run_repo(
        cx,
        window,
        &ws,
        RepoOp::RemoteRemove,
        Some(RefTarget::RemoteNode("second".into())),
    );
    answer(cx, window, 0);
    wait_until(cx, "removed", |_| {
        !remote_names(&repo).contains(&"second".to_string())
    });
    undo(cx, window, &ws);
    wait_until(cx, "restored", |_| {
        remote_names(&repo).contains(&"second".to_string())
    });
}

#[gpui_kit::test]
fn a_remote_has_a_menu_with_the_rows_in_the_terminal_uis_order(cx: &mut TestAppContext) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    run_repo(
        cx,
        window,
        &ws,
        RepoOp::RemoteActions,
        Some(RefTarget::RemoteNode("origin".into())),
    );
    let labels = menu_labels(cx, &ws);
    let at = |needle: &str| {
        labels
            .iter()
            .position(|l| l.contains(needle))
            .unwrap_or_else(|| panic!("{needle} in {labels:?}"))
    };
    assert!(at("Fetch origin") < at("Copy remote name"));
    assert!(at("Copy remote name") < at("Edit origin"));
    assert!(at("Edit origin") < at("Remove origin"));
    // The rows the daemon has no operation for stay listed and dimmed.
    let menu = ws.read_with(cx, |ws, _| ws.ref_menu().cloned()).unwrap();
    assert!(menu
        .read_with(cx, |m, _| m.refusal(at("default")))
        .is_some());
    press(cx, window, "escape");
    // Copy puts the name on the clipboard.
    run_repo(
        cx,
        window,
        &ws,
        RepoOp::RemoteCopyName,
        Some(RefTarget::RemoteNode("origin".into())),
    );
    let text = cx.update(|cx| cx.read_from_clipboard().and_then(|c| c.text()));
    assert_eq!(text.as_deref(), Some("origin"));
}

// -- push, pull and fetch of a branch -----------------------------------------------------------------

#[gpui_kit::test]
fn a_branch_menu_pushes_pulls_and_fetches_its_remote(cx: &mut TestAppContext) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    // A new commit on topic is pushed from the branch's own row (HEAD stays on main).
    repo.git(&["checkout", "-q", "topic"]);
    std::fs::write(repo.root().join("t2.txt"), "t2\n").unwrap();
    repo.git(&["add", "t2.txt"]);
    repo.git(&["commit", "-q", "-m", "more topic"]);
    repo.git(&["checkout", "-q", "main"]);
    run_repo(cx, window, &ws, RepoOp::BranchPush, Some(local("topic")));
    wait_until(cx, "pushed", |_| {
        repo.git(&["ls-remote", "origin", "refs/heads/topic"])
            .split_whitespace()
            .next()
            == Some(repo.git(&["rev-parse", "topic"]).trim())
    });
    undo(cx, window, &ws);
    wait_until(cx, "the push undone", |_| {
        repo.git(&["ls-remote", "origin", "refs/heads/topic"])
            .split_whitespace()
            .next()
            != Some(repo.git(&["rev-parse", "topic"]).trim())
    });
    // Fetch the branch's remote: a commit made elsewhere arrives.
    repo.push_from_elsewhere("main", "from elsewhere");
    run_repo(cx, window, &ws, RepoOp::BranchFetch, Some(local("main")));
    wait_until(cx, "fetched", |_| {
        repo.git(&["log", "-1", "--format=%s", "origin/main"])
            .trim()
            == "from elsewhere"
    });
    // Pull the checked-out branch.
    run_repo(cx, window, &ws, RepoOp::BranchPull, Some(local("main")));
    wait_until(cx, "pulled", |_| {
        repo.git(&["log", "-1", "--format=%s", "main"]).trim() == "from elsewhere"
    });
    // The menu rows are enabled now.
    ws.update(cx, |_, _| {});
    open_menu(cx, window, &ws, local("topic"));
    let menu = ws.read_with(cx, |ws, _| ws.ref_menu().cloned()).unwrap();
    let labels = menu.read_with(cx, |m, _| m.labels());
    for needle in ["Push topic", "worktree"] {
        let ix = labels.iter().position(|l| l.contains(needle)).unwrap();
        assert!(
            menu.read_with(cx, |m, _| m.refusal(ix)).is_none(),
            "{needle}"
        );
    }
}
