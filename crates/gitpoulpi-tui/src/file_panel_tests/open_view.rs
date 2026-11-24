use super::*;

#[test]
fn the_central_view_follows_what_is_open() {
    let mut v = CentralView::Log;
    sync_central_view(&mut v, false, false, true, false);
    assert_eq!(v, CentralView::File, "an open FileInsight is the File View");
    sync_central_view(&mut v, false, false, false, true);
    assert_eq!(
        v,
        CentralView::Diff,
        "closing it returns to the diff under it"
    );
    sync_central_view(&mut v, false, false, false, false);
    assert_eq!(v, CentralView::Log, "nothing open is the graph");
    let mut v = CentralView::Log;
    sync_central_view(&mut v, false, false, false, true);
    assert_eq!(
        v,
        CentralView::Log,
        "a kept diff state does not steal the graph"
    );
    // The editor is on top of both, and closing it goes
    // back to what is under it.
    let mut v = CentralView::Diff;
    sync_central_view(&mut v, false, true, false, true);
    assert_eq!(v, CentralView::Editor);
    sync_central_view(&mut v, false, false, false, true);
    assert_eq!(v, CentralView::Diff, "back to the diff it was opened on");
    let mut v = CentralView::Editor;
    sync_central_view(&mut v, false, false, true, true);
    assert_eq!(v, CentralView::File, "back to the File View");
    // The merge editor is on top of everything, and closing it goes back
    // to what is under it.
    let mut v = CentralView::Editor;
    sync_central_view(&mut v, true, true, true, true);
    assert_eq!(v, CentralView::Merge);
    sync_central_view(&mut v, false, false, false, true);
    assert_eq!(v, CentralView::Diff, "back to the diff under it");
    sync_central_view(&mut v, true, false, false, false);
    assert_eq!(v, CentralView::Merge, "over the graph too");
    sync_central_view(&mut v, false, false, false, false);
    assert_eq!(v, CentralView::Log);
}

/// In View all files the row index is into the whole tree,
/// not into the changed list - an unchanged file opens as content,
/// a changed one as a diff at its index in `files`.
#[test]
fn view_all_files_opens_unchanged_files_as_content_and_changed_ones_as_diffs() {
    let o = outcome(&["src/b.rs"]);
    let all = paths(&["README.md", "src/a.rs", "src/b.rs"]);
    assert_eq!(
        commit_file_open(&o, &all, 0),
        Some(CommitFileOpen::Content {
            path: PathBuf::from("README.md")
        })
    );
    assert_eq!(
        commit_file_open(&o, &all, 2),
        Some(CommitFileOpen::Diff {
            file_index: 0,
            path: PathBuf::from("src/b.rs")
        })
    );
    assert_eq!(commit_file_open(&o, &all, 9), None);
}

#[test]
fn a_view_all_files_marker_is_looked_up_by_path_not_by_position() {
    let o = outcome(&["src/b.rs"]);
    assert_eq!(commit_file_status(Some(&o), Path::new("README.md")), None);
    assert_eq!(
        commit_file_status(Some(&o), Path::new("src/b.rs")),
        Some(FileStatus::Modified)
    );
}

#[test]
fn the_open_row_is_found_by_path_for_a_diff_and_for_a_file_view() {
    let all = paths(&["README.md", "src/a.rs", "src/b.rs"]);
    let diff = DiffViewState::for_commit(oid(7), 0, PathBuf::from("src/b.rs"));
    assert_eq!(
        commit_open_path_index(Some(&diff), None, CentralView::Diff, Some(oid(7)), &all),
        Some(2)
    );
    let file = FileInsight::loading(FileInsightRequest {
        path: PathBuf::from("README.md"),
        at: oid(7),
        kind: FileInsightKind::Content,
        worktree: false,
        staged: false,
    });
    assert_eq!(
        commit_open_path_index(None, Some(&file), CentralView::File, Some(oid(7)), &all),
        Some(0)
    );
    assert_eq!(
        commit_open_path_index(None, Some(&file), CentralView::File, Some(oid(8)), &all),
        None,
        "another commit's file view is not this row"
    );
}

#[test]
fn the_head_tree_is_asked_for_once_per_head_and_only_when_wanted() {
    let mut wip = WipAllFiles::default();
    assert_eq!(wip.tree_to_request(false, Some(oid(1))), None);
    assert_eq!(wip.tree_to_request(true, Some(oid(1))), Some(oid(1)));
    assert_eq!(
        wip.tree_to_request(true, Some(oid(1))),
        None,
        "already asked"
    );
    assert_eq!(
        wip.tree_to_request(true, Some(oid(2))),
        Some(oid(2)),
        "new HEAD"
    );
    let answer = CommitFilesOutcome {
        commit: oid(1),
        files: Ok(Vec::new()),
        tree: Some(Ok(paths(&["a"]))),
        hunks_pending: false,
        lfs: None,
    };
    wip.accept(&answer);
    assert!(
        wip.head_tree.is_none(),
        "an answer for an old HEAD is dropped"
    );
}

/// Edit opens the working-directory copy
/// of the path shown by any Content view - a WIP diff, a
/// working-tree File View, a commit's diff, or a File View of a
/// commit's version - and only refuses Blame/History (not "this
/// file's own bytes"). A missing working copy is *not* refused here:
/// that is `Editor::open`'s job once `open_editor` tries the path
/// (covered by `editor.rs`'s own "not in the working tree" test).
#[test]
fn edit_targets_the_working_copy_of_any_content_view() {
    let insight = |worktree: bool, kind: FileInsightKind| {
        FileInsight::loading(FileInsightRequest {
            path: PathBuf::from("a.txt"),
            at: oid(7),
            kind,
            worktree,
            staged: false,
        })
    };
    let wip = DiffViewState::for_working(DiffSource::Unstaged, PathBuf::from("a.txt"));
    let commit = DiffViewState::for_commit(oid(7), 0, PathBuf::from("a.txt"));
    // The diff arm: a working diff and a commit's diff both resolve.
    assert_eq!(
        edit_target(CentralView::Diff, None, Some(&wip)),
        Ok(PathBuf::from("a.txt"))
    );
    assert_eq!(
        edit_target(CentralView::Diff, None, Some(&commit)),
        Ok(PathBuf::from("a.txt")),
        "a commit's diff now offers Edit too - it opens the working copy, not the commit's blob"
    );
    assert!(edit_target(CentralView::Diff, None, None).is_err());
    // The File View arm: worktree and commit-version Content views
    // both resolve; Blame/History are refused regardless of `worktree`.
    let on_disk = insight(true, FileInsightKind::Content);
    assert_eq!(
        edit_target(CentralView::File, Some(&on_disk), None),
        Ok(PathBuf::from("a.txt"))
    );
    let at_commit = insight(false, FileInsightKind::Content);
    assert_eq!(
        edit_target(CentralView::File, Some(&at_commit), None),
        Ok(PathBuf::from("a.txt")),
        "a File View of a commit's version now offers Edit too"
    );
    let blame = insight(true, FileInsightKind::Blame);
    assert!(edit_target(CentralView::File, Some(&blame), None).is_err());
    assert!(file_view_editable(&on_disk));
    assert!(file_view_editable(&at_commit));
    assert!(!file_view_editable(&blame));
    assert!(
        file_bar(FileInsightKind::Content, false, true, None, false, 120)
            .buttons
            .iter()
            .any(|(_, _, b)| *b == DiffBarButton::EditWorkingFile)
    );
}

/// A clean editor gives way to another file opened under it;
/// one with unsaved changes never does.
#[test]
fn only_a_clean_editor_gives_way_to_another_file() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("a.txt"),
        b"x
",
    )
    .unwrap();
    let mut ed = crate::editor::Editor::open(dir.path(), Path::new("a.txt")).unwrap();
    let same = DiffViewState::for_working(DiffSource::Unstaged, PathBuf::from("a.txt"));
    let other = DiffViewState::for_working(DiffSource::Unstaged, PathBuf::from("b.txt"));
    assert!(!editor_should_yield(&ed, None, Some(&same)));
    assert!(editor_should_yield(&ed, None, Some(&other)));
    ed.paste("dirty ");
    assert!(!editor_should_yield(&ed, None, Some(&other)));
    assert!(!editor_should_yield(&ed, None, None));
}
