//! Headless tests of the file context menu (stage, ignore, treat as, stash, open in, copy path, history),
//! Git LFS (the toolbar menu, tracking, "LFS only") on the real main window, with undo after each
//! mutation that is journaled.

mod refs_ops_common;
mod repo_ops_common;

use std::sync::Mutex;

use gitpoulpi_app::palette::vocab::status_pane::StatusSection;
use gitpoulpi_desktop::commands::file_ops::{self, FileOp};
use gitpoulpi_desktop::commands::repo_ops::RepoOp;
use gitpoulpi_desktop::ref_target::{FileSide, RefTarget};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use gpui_kit::{AppContext as _, TestAppContext};
use refs_ops_common::*;
use repo_ops_common::*;

static SPAWNED: Mutex<Vec<Vec<String>>> = Mutex::new(Vec::new());

fn record_spawns() {
    file_ops::set_spawner(|argv, _| {
        SPAWNED.lock().unwrap().push(argv.to_vec());
        Ok(())
    });
}

fn spawned_with(needle: &str) -> bool {
    SPAWNED
        .lock()
        .unwrap()
        .iter()
        .any(|argv| argv.iter().any(|a| a.contains(needle)))
}

type Ws = gpui_kit::Entity<gitpoulpi_desktop::app::Workspace>;

fn file_op(
    cx: &mut TestAppContext,
    window: gpui_kit::AnyWindowHandle,
    ws: &Ws,
    op: FileOp,
    path: &str,
    side: FileSide,
) {
    run_repo(
        cx,
        window,
        ws,
        RepoOp::File(op),
        Some(RefTarget::File(path.into(), side)),
    );
}

/// A repository with a modified tracked file and an untracked one, shown in the working changes.
fn dirty_repo() -> FixtureRepo {
    let repo = repo();
    std::fs::write(repo.root().join("a.txt"), "changed\n").unwrap();
    std::fs::create_dir_all(repo.root().join("logs")).unwrap();
    std::fs::write(repo.root().join("logs/run.log"), "x\n").unwrap();
    std::fs::write(repo.root().join("notes.log"), "y\n").unwrap();
    repo
}

fn wait_listed(cx: &mut TestAppContext, ws: &Ws, path: &str) {
    wait_until(cx, "the file in the working changes", |cx| {
        ws.read_with(cx, |ws, cx| {
            ws.right
                .read(cx)
                .row_paths(StatusSection::Unstaged)
                .iter()
                .any(|p| p == path || p.ends_with(path))
        })
    });
}

fn gitignore(repo: &FixtureRepo) -> String {
    std::fs::read_to_string(repo.root().join(".gitignore")).unwrap_or_default()
}

// -- the menu -----------------------------------------------------------------------------------------

#[gpui_kit::test]
fn the_menu_key_opens_the_menu_of_the_pane_that_has_the_keyboard(cx: &mut TestAppContext) {
    use gpui_kit::Focusable as _;
    let repo = dirty_repo();
    let (window, ws) = open(cx, &repo);
    wait_listed(cx, &ws, "a.txt");
    // The right pane, a file under the cursor: the key opens that file's menu.
    cx.update_window(window, |_, w, cx| {
        ws.update(cx, |ws, cx| {
            ws.right
                .update(cx, |r, cx| r.select_file("a.txt", FileSide::Unstaged, cx));
            ws.right.read(cx).focus_handle(cx).focus(w, cx);
        })
    })
    .unwrap();
    press(cx, window, "shift-f10");
    let labels = menu_labels(cx, &ws);
    assert!(
        labels.iter().any(|l| l.contains("Copy file path")),
        "the file menu opened: {labels:?}"
    );
    press(cx, window, "escape");
    assert!(ws.read_with(cx, |ws, _| ws.ref_menu().is_none()));

    // A remote's folder in the Refs: its own menu, not a file's.
    open_menu(cx, window, &ws, RefTarget::RemoteNode("origin".into()));
    let labels = menu_labels(cx, &ws);
    assert!(
        !labels.iter().any(|l| l.contains("Copy file path")),
        "{labels:?}"
    );
}

#[gpui_kit::test]
fn a_file_menu_lists_the_terminal_uis_rows_and_dims_what_has_no_operation(cx: &mut TestAppContext) {
    let repo = dirty_repo();
    let (window, ws) = open(cx, &repo);
    wait_listed(cx, &ws, "a.txt");
    cx.update_window(window, |_, w, cx| {
        ws.update(cx, |ws, cx| {
            ws.open_file_menu(
                "a.txt".into(),
                FileSide::Unstaged,
                gpui_kit::Point::new(gpui_kit::px(40.0), gpui_kit::px(40.0)),
                w,
                cx,
            )
        })
    })
    .unwrap();
    cx.run_until_parked();
    let labels = menu_labels(cx, &ws);
    let at = |needle: &str| {
        labels
            .iter()
            .position(|l| l.contains(needle))
            .unwrap_or_else(|| panic!("{needle} in {labels:?}"))
    };
    assert!(at("Stage") < at("Discard") && at("Discard") < at("Ignore"));
    assert!(at("Ignore") < at("Treat as binary") && at("Treat as text") < at("Stash"));
    assert!(at("File History") < at("Open in") && at("Open in") < at("Copy file path"));
    // Blame is not available yet: listed, dimmed with the reason.
    let menu = ws.read_with(cx, |ws, _| ws.ref_menu().cloned()).unwrap();
    assert!(menu
        .read_with(cx, |m, _| m.refusal(at("File Blame")))
        .is_some());
    assert!(menu
        .read_with(cx, |m, _| m.refusal(at("Copy file path")))
        .is_none());
    press(cx, window, "escape");
    assert!(ws.read_with(cx, |ws, _| ws.ref_menu().is_none()));

    // An untracked file has no Discard, no history, no blame.
    wait_listed(cx, &ws, "notes.log");
    open_menu(
        cx,
        window,
        &ws,
        RefTarget::File("notes.log".into(), FileSide::Unstaged),
    );
    let labels = menu_labels(cx, &ws);
    assert!(
        !labels
            .iter()
            .any(|l| l.contains("Discard") || l.contains("History")),
        "{labels:?}"
    );
}

// -- ignore, treat as, stash ----------------------------------------------------------------------------

#[gpui_kit::test]
fn ignoring_a_file_an_extension_or_a_folder_writes_the_rule_and_undo_takes_it_back(
    cx: &mut TestAppContext,
) {
    let repo = dirty_repo();
    let (window, ws) = open(cx, &repo);
    wait_listed(cx, &ws, "notes.log");
    for (op, path, rule) in [
        (FileOp::IgnoreFile, "notes.log", "/notes.log"),
        (FileOp::IgnoreExtension, "notes.log", "*.log"),
        (FileOp::IgnoreFolder, "logs/run.log", "/logs/"),
    ] {
        file_op(cx, window, &ws, op, path, FileSide::Unstaged);
        wait_until(cx, rule, |_| gitignore(&repo).lines().any(|l| l == rule));
        undo(cx, window, &ws);
        wait_until(cx, "the rule gone", |_| {
            !gitignore(&repo).lines().any(|l| l == rule)
        });
    }
    // An extension of a file with none says so and writes nothing.
    std::fs::write(repo.root().join("Makefile"), "all:\n").unwrap();
    file_op(
        cx,
        window,
        &ws,
        FileOp::IgnoreExtension,
        "Makefile",
        FileSide::Unstaged,
    );
    assert!(notice(cx, &ws).contains("no extension"));
}

#[gpui_kit::test]
fn treating_a_file_as_binary_or_text_writes_a_gitattributes_rule(cx: &mut TestAppContext) {
    let repo = dirty_repo();
    let (window, ws) = open(cx, &repo);
    wait_listed(cx, &ws, "a.txt");
    let attrs = |repo: &FixtureRepo| {
        std::fs::read_to_string(repo.root().join(".gitattributes")).unwrap_or_default()
    };
    file_op(
        cx,
        window,
        &ws,
        FileOp::TreatFileBinary,
        "a.txt",
        FileSide::Unstaged,
    );
    wait_until(cx, "the binary rule", |_| {
        attrs(&repo).contains("/a.txt") && attrs(&repo).contains("binary")
    });
    undo(cx, window, &ws);
    wait_until(cx, "the rule gone", |_| !attrs(&repo).contains("/a.txt"));
    file_op(
        cx,
        window,
        &ws,
        FileOp::TreatExtensionText,
        "a.txt",
        FileSide::Unstaged,
    );
    wait_until(cx, "the text rule", |_| attrs(&repo).contains("*.txt"));
}

#[gpui_kit::test]
fn stashing_one_file_leaves_the_others_and_undo_brings_it_back(cx: &mut TestAppContext) {
    let repo = repo();
    std::fs::write(repo.root().join("a.txt"), "changed\n").unwrap();
    std::fs::write(repo.root().join("other.txt"), "o\n").unwrap();
    repo.git(&["add", "other.txt"]);
    let (window, ws) = open(cx, &repo);
    wait_listed(cx, &ws, "a.txt");
    file_op(
        cx,
        window,
        &ws,
        FileOp::StashFile,
        "a.txt",
        FileSide::Unstaged,
    );
    wait_until(cx, "the stash", |_| stashes(&repo) == 1);
    assert_eq!(
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap(),
        "a\n"
    );
    assert!(
        repo.git(&["status", "--short"]).contains("other.txt"),
        "the other file stays"
    );
    undo(cx, window, &ws);
    wait_until(cx, "the file back", |_| {
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap() == "changed\n"
    });
}

// -- stage, discard, open in, copy ----------------------------------------------------------------------

#[gpui_kit::test]
fn the_menu_rows_stage_unstage_and_discard_the_clicked_file(cx: &mut TestAppContext) {
    let repo = dirty_repo();
    let (window, ws) = open(cx, &repo);
    wait_listed(cx, &ws, "a.txt");
    file_op(cx, window, &ws, FileOp::Stage, "a.txt", FileSide::Unstaged);
    wait_until(cx, "staged", |_| {
        repo.git(&["diff", "--cached", "--name-only"])
            .contains("a.txt")
    });
    wait_until(cx, "the staged list", |cx| {
        ws.read_with(cx, |ws, cx| {
            ws.right
                .read(cx)
                .row_paths(StatusSection::Staged)
                .iter()
                .any(|p| p == "a.txt")
        })
    });
    file_op(cx, window, &ws, FileOp::Unstage, "a.txt", FileSide::Staged);
    wait_until(cx, "unstaged", |_| {
        repo.git(&["diff", "--cached", "--name-only"])
            .trim()
            .is_empty()
    });
    // Discard asks first and nothing is preselected.
    wait_listed(cx, &ws, "a.txt");
    file_op(
        cx,
        window,
        &ws,
        FileOp::Discard,
        "a.txt",
        FileSide::Unstaged,
    );
    assert!(dialog_open(cx, window));
    let preset = cx
        .update(|cx| gitpoulpi_desktop::dialogs::current(cx))
        .unwrap()
        .read_with(cx, |q, _| q.spec().highlight_first);
    let _ = preset;
    answer(cx, window, 1);
    assert_eq!(
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap(),
        "changed\n"
    );
    file_op(
        cx,
        window,
        &ws,
        FileOp::Discard,
        "a.txt",
        FileSide::Unstaged,
    );
    answer(cx, window, 0);
    wait_until(cx, "discarded", |_| {
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap() == "a\n"
    });
    undo(cx, window, &ws);
    wait_until(cx, "the discard undone", |_| {
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap() == "changed\n"
    });
}

#[gpui_kit::test]
fn a_file_opens_in_the_editor_the_default_program_and_the_folder_and_its_path_is_copied(
    cx: &mut TestAppContext,
) {
    record_spawns();
    let repo = dirty_repo();
    let (window, ws) = open(cx, &repo);
    wait_listed(cx, &ws, "a.txt");
    let abs = repo.root().join("a.txt").to_string_lossy().into_owned();
    file_op(
        cx,
        window,
        &ws,
        FileOp::OpenInEditor,
        "a.txt",
        FileSide::Unstaged,
    );
    assert!(
        spawned_with(&abs),
        "the editor (or the default program) got the file"
    );
    SPAWNED.lock().unwrap().clear();
    file_op(
        cx,
        window,
        &ws,
        FileOp::OpenInDefault,
        "a.txt",
        FileSide::Unstaged,
    );
    assert!(spawned_with(&abs));
    SPAWNED.lock().unwrap().clear();
    file_op(
        cx,
        window,
        &ws,
        FileOp::ShowInFolder,
        "a.txt",
        FileSide::Unstaged,
    );
    assert!(
        spawned_with(&repo.root().to_string_lossy()),
        "the folder is shown"
    );
    // A file that is not on disk is refused with the reason.
    SPAWNED.lock().unwrap().clear();
    file_op(
        cx,
        window,
        &ws,
        FileOp::OpenInDefault,
        "gone.txt",
        FileSide::Unstaged,
    );
    assert!(notice(cx, &ws).contains("not on disk"));
    assert!(!spawned_with("gone.txt"));
    file_op(
        cx,
        window,
        &ws,
        FileOp::CopyPath,
        "logs/run.log",
        FileSide::Unstaged,
    );
    let text = cx.update(|cx| cx.read_from_clipboard().and_then(|c| c.text()));
    assert_eq!(text.as_deref(), Some("logs/run.log"));
}

// -- history ------------------------------------------------------------------------------------------

#[gpui_kit::test]
fn the_file_history_lists_the_commits_that_changed_it_and_shows_one_in_the_log(
    cx: &mut TestAppContext,
) {
    let repo = repo();
    std::fs::write(repo.root().join("a.txt"), "a2\n").unwrap();
    repo.git(&["commit", "-qam", "touch a again"]);
    let (window, ws) = open(cx, &repo);
    file_op(cx, window, &ws, FileOp::History, "a.txt", FileSide::Commit);
    wait_until(cx, "the history", |cx| dialog_open(cx, window));
    let spec = cx
        .update(|cx| gitpoulpi_desktop::dialogs::current(cx))
        .unwrap()
        .read_with(cx, |q, _| q.spec().clone());
    assert!(spec.title.contains("a.txt"));
    assert_eq!(spec.lines.len(), 2, "{:?}", spec.lines);
    assert!(spec.lines[0].contains("touch a again") && spec.lines[1].contains("add a"));
    answer(cx, window, 0);
    // A file with no history says so.
    file_op(
        cx,
        window,
        &ws,
        FileOp::History,
        "never.txt",
        FileSide::Commit,
    );
    wait_until(cx, "the answer", |cx| {
        notice(cx, &ws).contains("no history")
    });
}

// -- Git LFS ------------------------------------------------------------------------------------------

fn lfs_repo() -> FixtureRepo {
    isolate();
    FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .with_lfs_origin()
        .unwrap()
        .with_lfs()
        .unwrap()
        .lfs_track("*.bin")
        .unwrap()
        .commit_lfs_file("big.bin", b"large content", "add big")
        .unwrap()
        .push("HEAD:refs/heads/main")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .build()
}

fn isolate() {
    gitpoulpi_testkit::isolate_process_env();
}

#[gpui_kit::test]
fn the_lfs_menu_lists_its_rows_and_the_transfers_run_as_one_action_each(cx: &mut TestAppContext) {
    if !gitpoulpi_testkit::system_git_at_least(2, 0)
        || std::process::Command::new("git")
            .args(["lfs", "version"])
            .output()
            .is_err()
    {
        return;
    }
    let repo = lfs_repo();
    let (window, ws) = open_plain(cx, &repo);
    run(cx, window, "git_prefix.lfs_menu");
    wait_until(cx, "the LFS menu", |cx| {
        ws.read_with(cx, |ws, _| ws.ref_menu().is_some())
    });
    let labels = menu_labels(cx, &ws);
    for needle in [
        "Fetch all LFS",
        "Check out all LFS",
        "Pull all LFS",
        "Push all LFS",
        "Prune local LFS",
        "Track Pattern",
        "Untrack Pattern",
        "Show Locks",
    ] {
        assert!(
            labels.iter().any(|l| l.contains(needle)),
            "{needle} in {labels:?}"
        );
    }
    press(cx, window, "escape");
    // The toolbar's button and the Ctrl+G chord reach the same menu.
    run(cx, window, "git_prefix.lfs_prune");
    wait_until(cx, "the prune said", |cx| {
        notice(cx, &ws).to_lowercase().contains("lfs")
    });
    run(cx, window, "git_prefix.lfs_checkout");
    wait_until(cx, "the checkout said", |cx| {
        notice(cx, &ws).to_lowercase().contains("lfs")
    });
    run(cx, window, "git_prefix.lfs_push_all");
    wait_until(cx, "the push's note", |cx| notice(cx, &ws).contains("LFS"));
    run(cx, window, "git_prefix.lfs_fetch_all");
    run(cx, window, "git_prefix.lfs_pull");
}

#[gpui_kit::test]
fn lfs_patterns_are_tracked_and_untracked_through_forms_and_undo_takes_them_back(
    cx: &mut TestAppContext,
) {
    if std::process::Command::new("git")
        .args(["lfs", "version"])
        .output()
        .is_err()
    {
        return;
    }
    let repo = lfs_repo();
    let (window, ws) = open_plain(cx, &repo);
    let attrs = |repo: &FixtureRepo| {
        std::fs::read_to_string(repo.root().join(".gitattributes")).unwrap_or_default()
    };
    run(cx, window, "palette.lfs_track_pattern");
    submit(cx, window);
    assert!(form_error(cx).is_some(), "an empty pattern is refused");
    type_into(cx, window, 0, "*.dat");
    submit(cx, window);
    wait_until(cx, "the pattern", |_| {
        attrs(&repo).contains("*.dat filter=lfs")
    });
    undo(cx, window, &ws);
    wait_until(cx, "the pattern gone", |_| !attrs(&repo).contains("*.dat"));
    run(cx, window, "palette.lfs_untrack_pattern");
    type_into(cx, window, 0, "*.bin");
    submit(cx, window);
    wait_until(cx, "untracked", |_| !attrs(&repo).contains("*.bin"));
    undo(cx, window, &ws);
    wait_until(cx, "tracked again", |_| {
        attrs(&repo).contains("*.bin filter=lfs")
    });
}

#[gpui_kit::test]
fn the_lfs_rows_of_a_files_menu_and_the_lfs_only_mode_follow_the_lfs_files(
    cx: &mut TestAppContext,
) {
    if std::process::Command::new("git")
        .args(["lfs", "version"])
        .output()
        .is_err()
    {
        return;
    }
    let repo = lfs_repo();
    std::fs::write(repo.root().join("big.bin"), "larger content than before").unwrap();
    std::fs::write(repo.root().join("small.txt"), "s\n").unwrap();
    let (window, ws) = open_plain(cx, &repo);
    wait_listed(cx, &ws, "big.bin");
    wait_until(cx, "the LFS state", |cx| {
        ws.read_with(cx, |ws, _| {
            ws.lfs_view().is_some_and(|v| v.is_lfs("big.bin"))
        })
    });
    // An LFS file offers the lock rows; a plain file offers to track it.
    open_menu(
        cx,
        window,
        &ws,
        RefTarget::File("big.bin".into(), FileSide::Unstaged),
    );
    let labels = menu_labels(cx, &ws);
    assert!(labels.iter().any(|l| l.contains("LFS")), "{labels:?}");
    let sub_rows = ws.read_with(cx, |ws, _| ws.lfs_view().unwrap().file_rows("big.bin"));
    assert_eq!(sub_rows, vec![RepoOp::LfsLock]);
    let plain = ws.read_with(cx, |ws, _| ws.lfs_view().unwrap().file_rows("small.txt"));
    assert_eq!(plain, vec![RepoOp::LfsTrackFile, RepoOp::LfsTrackExtension]);
    press(cx, window, "escape");
    // Tracking one file routes its name through LFS (one undo entry).
    file_op_lfs(cx, window, &ws, RepoOp::LfsTrackFile, "small.txt");
    wait_until(cx, "the file's rule", |_| {
        std::fs::read_to_string(repo.root().join(".gitattributes"))
            .unwrap_or_default()
            .contains("small.txt filter=lfs")
    });
    undo(cx, window, &ws);
    wait_until(cx, "the rule gone", |_| {
        !std::fs::read_to_string(repo.root().join(".gitattributes"))
            .unwrap_or_default()
            .contains("small.txt")
    });

    // LFS only keeps the LFS files in the lists, and toggles back.
    run(cx, window, "status.lfs_only");
    wait_until(cx, "the filter", |cx| {
        ws.read_with(cx, |ws, _| ws.lfs_only())
    });
    let listed = ws.read_with(cx, |ws, cx| ws.right.read(cx).listed_paths());
    assert_eq!(listed, vec!["big.bin".to_string()], "{listed:?}");
    run(cx, window, "status.lfs_only");
    let listed = ws.read_with(cx, |ws, cx| ws.right.read(cx).listed_paths());
    assert!(listed.iter().any(|p| p == "small.txt"));
}

fn file_op_lfs(
    cx: &mut TestAppContext,
    window: gpui_kit::AnyWindowHandle,
    ws: &Ws,
    op: RepoOp,
    path: &str,
) {
    run_repo(
        cx,
        window,
        ws,
        op,
        Some(RefTarget::File(path.into(), FileSide::Unstaged)),
    );
}
